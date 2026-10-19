//! Bounded wave execution for the isolated, in-memory CWC experiment.
//!
//! The graph is derived from typed operations. Workers borrow one immutable wave
//! snapshot, produce isolated writes, and are all joined before canonical barrier
//! commits. The candidate remains private until the entire block succeeds.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crate::model::{
    evaluate_operation, finish_execution, validate_candidate, AbortReason, Block, Evaluation,
    OperationResult, Outcome, State, MAX_OPERATIONS,
};

#[cfg(feature = "research-controls")]
pub mod research;

const PREDECESSOR_WORDS: usize = MAX_OPERATIONS.div_ceil(u64::BITS as usize);
const WORKER_STACK_BYTES: usize = 256 * 1024;

/// A bounded, inspectable graph reconstructed from the block's typed operations.
///
/// At most 32,640 edges and 256 operation indices are exposed. Callers cannot
/// provide this graph to the executor or override derived dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyPlan {
    pub edges: Vec<(u16, u16)>,
    pub waves: Vec<Vec<u16>>,
}

struct Graph {
    predecessors: [[u64; PREDECESSOR_WORDS]; MAX_OPERATIONS],
    depths: [u16; MAX_OPERATIONS],
    operations: usize,
    wave_count: usize,
}

impl Graph {
    fn derive(block: &Block, hooks: &Hooks<'_>) -> Self {
        let accesses: Vec<_> = block
            .operations()
            .iter()
            .map(|operation| operation.access_sets())
            .collect();
        let mut graph = Self {
            predecessors: [[0; PREDECESSOR_WORDS]; MAX_OPERATIONS],
            depths: [0; MAX_OPERATIONS],
            operations: accesses.len(),
            wave_count: usize::from(!accesses.is_empty()),
        };
        for later in 0..accesses.len() {
            for earlier in 0..later {
                let before = &accesses[earlier];
                let after = &accesses[later];
                let raw = before.writes.iter().any(|key| after.reads.contains(key));
                let waw = before.writes.iter().any(|key| after.writes.contains(key));
                let war = before.reads.iter().any(|key| after.writes.contains(key));
                if (hooks.retain_read_after_write() && raw) || waw || war {
                    graph.predecessors[later][earlier / 64] |= 1_u64 << (earlier % 64);
                    graph.depths[later] = graph.depths[later].max(graph.depths[earlier] + 1);
                }
            }
            graph.wave_count = graph.wave_count.max(usize::from(graph.depths[later]) + 1);
        }
        graph
    }

    fn has_edge(&self, earlier: usize, later: usize) -> bool {
        self.predecessors[later][earlier / 64] & (1_u64 << (earlier % 64)) != 0
    }

    fn wave(&self, depth: usize) -> Vec<u16> {
        (0..self.operations)
            .filter(|&index| usize::from(self.depths[index]) == depth)
            // Block construction enforces at most 256 operations.
            .map(|index| index as u16)
            .collect()
    }
}

/// Reconstruct dependencies, including read/write, write/write and
/// write-after-read conflicts. This is diagnostic output, not caller authority.
pub fn dependency_plan(block: &Block) -> DependencyPlan {
    let graph = Graph::derive(block, &Hooks::default());
    let mut edges = Vec::new();
    for earlier in 0..graph.operations {
        for later in (earlier + 1)..graph.operations {
            if graph.has_edge(earlier, later) {
                edges.push((earlier as u16, later as u16));
            }
        }
    }
    DependencyPlan {
        edges,
        waves: (0..graph.wave_count)
            .map(|depth| graph.wave(depth))
            .collect(),
    }
}

/// Execute a candidate using at most 1, 2 or 4 actual worker threads at a time.
///
/// Cancellation is local validation abandonment, never candidate rejection.
/// The caller must keep the cancellation flag sticky once set until this call
/// returns; resetting it concurrently can hide the cancellation request. A flag
/// set after a completed return does not undo the returned owned result.
/// Every owned worker is joined before return, including spawn failures and
/// panics. Pure operations are bounded; this does not forcibly terminate threads
/// or provide durable storage, process-crash recovery, or an external commit.
/// Each worker requests a 256 KiB stack, at most four coexist, and each of at
/// most 256 operations belongs to one nonempty worker assignment (therefore at
/// most 256 thread spawns over an entire candidate, rather than a persistent
/// pool). Platform stack/guard-page allocation overhead is additional.
pub fn execute_parallel(
    initial: &State,
    block: &Block,
    workers: usize,
    cancellation: &AtomicBool,
) -> Outcome {
    execute_with_hooks(initial, block, workers, cancellation, &Hooks::default())
}

fn execute_with_hooks(
    initial: &State,
    block: &Block,
    workers: usize,
    cancellation: &AtomicBool,
    hooks: &Hooks<'_>,
) -> Outcome {
    if !matches!(workers, 1 | 2 | 4) {
        return Outcome::Aborted(AbortReason::InvalidWorkerCount);
    }
    if cancellation.load(Ordering::Acquire) {
        return Outcome::Aborted(AbortReason::Cancelled);
    }
    if validate_candidate(initial, block).is_err() {
        return Outcome::Rejected;
    }

    let graph = Graph::derive(block, hooks);
    let mut candidate = match initial.try_clone() {
        Ok(candidate) => candidate,
        Err(reason) => return Outcome::Aborted(reason),
    };
    let mut results = Vec::<OperationResult>::new();
    if results.try_reserve_exact(block.operations().len()).is_err() {
        return Outcome::Aborted(AbortReason::LocalResourceFailure);
    }

    for depth in 0..graph.wave_count {
        if cancellation.load(Ordering::Acquire) {
            return Outcome::Aborted(AbortReason::Cancelled);
        }
        let wave = graph.wave(depth);
        let mut completed = match run_wave(&candidate, block, &wave, workers, cancellation, hooks) {
            Ok(completed) => completed,
            Err(reason) => return Outcome::Aborted(reason),
        };
        if cancellation.load(Ordering::Acquire) {
            return Outcome::Aborted(AbortReason::Cancelled);
        }
        if completed.iter().any(|(_, evaluation)| evaluation.is_none()) {
            return Outcome::Rejected;
        }
        completed.sort_by_key(|(index, _)| *index);
        for (_, evaluation) in completed {
            // The preceding whole-wave check keeps rejection atomic.
            let Some(evaluation) = evaluation else {
                return Outcome::Rejected;
            };
            for (key, value) in evaluation.writes {
                if candidate.set_existing(&key, value).is_err() {
                    return Outcome::Rejected;
                }
            }
            results.push(evaluation.result);
        }
    }
    if cancellation.load(Ordering::Acquire) {
        return Outcome::Aborted(AbortReason::Cancelled);
    }
    // Waves can contain noncontiguous canonical indices. Results are committed
    // only here in index order, independent of graph shape and worker completion.
    results.sort_by_key(|result| result.index);
    let execution = finish_execution(candidate, results);
    if cancellation.load(Ordering::Acquire) {
        Outcome::Aborted(AbortReason::Cancelled)
    } else {
        Outcome::Accepted(execution)
    }
}

type CompletedWave = Vec<(u16, Option<Evaluation>)>;

fn run_wave(
    snapshot: &State,
    block: &Block,
    wave: &[u16],
    configured_workers: usize,
    cancellation: &AtomicBool,
    hooks: &Hooks<'_>,
) -> Result<CompletedWave, AbortReason> {
    let workers = configured_workers.min(wave.len());
    let chunk_size = wave.len().div_ceil(workers);
    thread::scope(|scope| {
        let mut handles = Vec::new();
        handles
            .try_reserve_exact(workers)
            .map_err(|_| AbortReason::LocalResourceFailure)?;
        let mut spawn_failure = None;
        for (worker_index, assigned) in wave.chunks(chunk_size).enumerate() {
            if let Err(reason) = hooks.before_spawn(worker_index) {
                spawn_failure = Some(reason);
                break;
            }
            let handle = thread::Builder::new()
                .name(format!("cwc-{worker_index}"))
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(scope, move || {
                    let mut completed = Vec::new();
                    completed
                        .try_reserve_exact(assigned.len())
                        .map_err(|_| AbortReason::LocalResourceFailure)?;
                    for &index in assigned {
                        if cancellation.load(Ordering::Acquire) {
                            return Err(AbortReason::Cancelled);
                        }
                        hooks.before_operation(index)?;
                        let evaluation = evaluate_operation(
                            snapshot,
                            &block.operations()[usize::from(index)],
                            index,
                        )
                        .ok();
                        hooks.after_operation(index)?;
                        completed.push((index, evaluation));
                    }
                    Ok(completed)
                });
            match handle {
                Ok(handle) => handles.push(handle),
                Err(_) => {
                    spawn_failure = Some(AbortReason::WorkerSpawnFailure);
                    break;
                }
            }
        }

        let mut completed = Vec::new();
        let mut abort = spawn_failure;
        if completed.try_reserve_exact(wave.len()).is_err() {
            abort.get_or_insert(AbortReason::LocalResourceFailure);
        }
        // Never return early: even after spawn/admission/panic/cancellation
        // failures, every successfully spawned worker remains owned and joined.
        for handle in handles {
            match handle.join() {
                Ok(Ok(worker_completed)) if abort.is_none() => completed.extend(worker_completed),
                Ok(Ok(_)) => {}
                Ok(Err(reason)) => {
                    abort.get_or_insert(reason);
                }
                Err(_) => {
                    abort.get_or_insert(AbortReason::WorkerPanic);
                }
            }
        }
        match abort {
            Some(reason) => Err(reason),
            None => Ok(completed),
        }
    })
}

/// Hooks have no production behavior. The only implementation which can alter
/// scheduling/failures is compiled exclusively into this module's unit tests.
#[derive(Default)]
struct Hooks<'a> {
    #[cfg(test)]
    test: Option<&'a dyn TestHooks>,
    #[cfg(feature = "research-controls")]
    reverse_pair: Option<&'a research::ReversePair<'a>>,
    marker: std::marker::PhantomData<&'a ()>,
}

impl Hooks<'_> {
    fn retain_read_after_write(&self) -> bool {
        #[cfg(test)]
        if let Some(test) = self.test {
            return test.retain_read_after_write();
        }
        true
    }

    fn before_spawn(&self, _worker: usize) -> Result<(), AbortReason> {
        #[cfg(test)]
        if let Some(test) = self.test {
            return test.before_spawn(_worker);
        }
        Ok(())
    }

    fn before_operation(&self, _index: u16) -> Result<(), AbortReason> {
        #[cfg(feature = "research-controls")]
        if let Some(control) = self.reverse_pair {
            return control.before_operation(_index);
        }
        #[cfg(test)]
        if let Some(test) = self.test {
            return test.before_operation(_index);
        }
        Ok(())
    }

    fn after_operation(&self, _index: u16) -> Result<(), AbortReason> {
        #[cfg(feature = "research-controls")]
        if let Some(control) = self.reverse_pair {
            return control.after_operation(_index);
        }
        #[cfg(test)]
        if let Some(test) = self.test {
            return test.after_operation(_index);
        }
        Ok(())
    }
}

#[cfg(test)]
trait TestHooks: Sync {
    fn retain_read_after_write(&self) -> bool {
        true
    }
    fn before_spawn(&self, _worker: usize) -> Result<(), AbortReason> {
        Ok(())
    }
    fn before_operation(&self, _index: u16) -> Result<(), AbortReason> {
        Ok(())
    }
    fn after_operation(&self, _index: u16) -> Result<(), AbortReason> {
        Ok(())
    }
}

#[cfg(test)]
mod schedules;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Operation;
    use crate::serial::execute_serial;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Condvar, Mutex};
    use std::time::{Duration, Instant};

    fn key(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn run_hooked(
        state: &State,
        block: &Block,
        workers: usize,
        cancellation: &AtomicBool,
        test: &dyn TestHooks,
    ) -> Outcome {
        execute_with_hooks(
            state,
            block,
            workers,
            cancellation,
            &Hooks {
                test: Some(test),
                ..Hooks::default()
            },
        )
    }

    fn independent_fixture() -> (State, Block) {
        (
            State::new(vec![(key(1), 10), (key(2), 20)]).unwrap(),
            Block::new(vec![
                Operation::Add {
                    key: key(1),
                    amount: 5,
                },
                Operation::Add {
                    key: key(2),
                    amount: 7,
                },
            ])
            .unwrap(),
        )
    }

    #[derive(Default)]
    struct OverlapState {
        entrants: Vec<thread::ThreadId>,
        completion_order: Vec<u16>,
    }

    #[derive(Default)]
    struct BoundedOverlap {
        state: Mutex<OverlapState>,
        changed: Condvar,
    }

    impl BoundedOverlap {
        fn wait_until(&self, predicate: impl Fn(&OverlapState) -> bool) -> Result<(), AbortReason> {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut state = self.state.lock().unwrap();
            while !predicate(&state) {
                let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                    return Err(AbortReason::LocalResourceFailure);
                };
                let (next, timeout) = self.changed.wait_timeout(state, remaining).unwrap();
                state = next;
                if timeout.timed_out() && !predicate(&state) {
                    return Err(AbortReason::LocalResourceFailure);
                }
            }
            Ok(())
        }
    }

    impl TestHooks for BoundedOverlap {
        fn before_operation(&self, _index: u16) -> Result<(), AbortReason> {
            self.state
                .lock()
                .unwrap()
                .entrants
                .push(thread::current().id());
            self.changed.notify_all();
            self.wait_until(|state| state.entrants.len() == 2)
        }

        fn after_operation(&self, index: u16) -> Result<(), AbortReason> {
            if index == 0 {
                self.wait_until(|state| state.completion_order.contains(&1))?;
            }
            self.state.lock().unwrap().completion_order.push(index);
            self.changed.notify_all();
            Ok(())
        }
    }

    #[test]
    fn independent_operations_overlap_on_distinct_threads_and_finish_in_reverse_order() {
        let (state, block) = independent_fixture();
        let hooks = BoundedOverlap::default();
        let result = run_hooked(&state, &block, 2, &AtomicBool::new(false), &hooks);
        assert_eq!(result, execute_serial(&state, &block));
        let observed = hooks.state.lock().unwrap();
        assert_eq!(observed.entrants.len(), 2);
        assert_ne!(observed.entrants[0], observed.entrants[1]);
        assert_eq!(observed.completion_order, vec![1, 0]);
    }

    struct PanicFirst {
        later_completed: AtomicUsize,
    }

    impl TestHooks for PanicFirst {
        fn before_operation(&self, index: u16) -> Result<(), AbortReason> {
            assert_ne!(index, 0, "controlled synthetic worker panic");
            Ok(())
        }
        fn after_operation(&self, _index: u16) -> Result<(), AbortReason> {
            self.later_completed.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
    }

    #[test]
    fn worker_panic_aborts_and_joins_later_work_without_publishing_state() {
        let (state, block) = independent_fixture();
        let original = state.clone();
        let hooks = PanicFirst {
            later_completed: AtomicUsize::new(0),
        };
        assert_eq!(
            run_hooked(&state, &block, 2, &AtomicBool::new(false), &hooks),
            Outcome::Aborted(AbortReason::WorkerPanic)
        );
        assert_eq!(hooks.later_completed.load(Ordering::Acquire), 1);
        assert_eq!(state, original);
        assert!(matches!(
            execute_parallel(&state, &block, 2, &AtomicBool::new(false)),
            Outcome::Accepted(_)
        ));
        // Every worker was joined before the aborted attempt returned.
        assert_eq!(hooks.later_completed.load(Ordering::Acquire), 1);
    }

    struct CancelAfterEvaluation<'a>(&'a AtomicBool);

    impl TestHooks for CancelAfterEvaluation<'_> {
        fn after_operation(&self, _index: u16) -> Result<(), AbortReason> {
            self.0.store(true, Ordering::Release);
            Ok(())
        }
    }

    #[test]
    fn cancellation_after_evaluation_discards_private_results_and_retry_succeeds() {
        let (state, block) = independent_fixture();
        let original = state.clone();
        let cancellation = AtomicBool::new(false);
        assert_eq!(
            run_hooked(
                &state,
                &block,
                2,
                &cancellation,
                &CancelAfterEvaluation(&cancellation)
            ),
            Outcome::Aborted(AbortReason::Cancelled)
        );
        assert_eq!(state, original);
        cancellation.store(false, Ordering::Release);
        assert_eq!(
            execute_parallel(&state, &block, 2, &cancellation),
            execute_serial(&state, &block)
        );
    }

    struct FailSecondSpawn {
        completed: AtomicUsize,
    }

    impl TestHooks for FailSecondSpawn {
        fn before_spawn(&self, worker: usize) -> Result<(), AbortReason> {
            if worker == 1 {
                Err(AbortReason::WorkerSpawnFailure)
            } else {
                Ok(())
            }
        }
        fn after_operation(&self, _index: u16) -> Result<(), AbortReason> {
            self.completed.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
    }

    #[test]
    fn spawn_failure_joins_already_owned_worker_and_returns_no_candidate_state() {
        let (state, block) = independent_fixture();
        let original = state.clone();
        let hooks = FailSecondSpawn {
            completed: AtomicUsize::new(0),
        };
        assert_eq!(
            run_hooked(&state, &block, 2, &AtomicBool::new(false), &hooks),
            Outcome::Aborted(AbortReason::WorkerSpawnFailure)
        );
        assert_eq!(hooks.completed.load(Ordering::Acquire), 1);
        assert_eq!(state, original);
    }

    struct CompletionCounter(AtomicUsize);

    impl TestHooks for CompletionCounter {
        fn after_operation(&self, index: u16) -> Result<(), AbortReason> {
            if index == 1 {
                self.0.fetch_add(1, Ordering::AcqRel);
            }
            Ok(())
        }
    }

    #[test]
    fn rejected_early_operation_does_not_publish_later_successful_independent_work() {
        let state = State::new(vec![(key(1), 10), (key(2), 20)]).unwrap();
        let original = state.clone();
        let block = Block::new(vec![
            Operation::AssertEquals {
                key: key(1),
                value: 999,
            },
            Operation::Add {
                key: key(2),
                amount: 7,
            },
        ])
        .unwrap();
        let completed = CompletionCounter(AtomicUsize::new(0));
        assert_eq!(
            run_hooked(&state, &block, 2, &AtomicBool::new(false), &completed),
            Outcome::Rejected
        );
        assert_eq!(completed.0.load(Ordering::Acquire), 1);
        assert_eq!(state, original);
    }

    struct OmitReadAfterWrite;

    impl TestHooks for OmitReadAfterWrite {
        fn retain_read_after_write(&self) -> bool {
            false
        }
    }

    #[test]
    fn negative_control_missing_dependency_is_detected_by_the_serial_oracle() {
        let state = State::new(vec![(key(1), 1)]).unwrap();
        let block = Block::new(vec![
            Operation::Set {
                key: key(1),
                value: 2,
            },
            Operation::AssertEquals {
                key: key(1),
                value: 2,
            },
        ])
        .unwrap();
        let serial = execute_serial(&state, &block);
        assert!(matches!(serial, Outcome::Accepted(_)));
        assert_eq!(
            execute_parallel(&state, &block, 2, &AtomicBool::new(false)),
            serial
        );
        // Deliberately corrupt only the test scheduler: both operations now read
        // one snapshot, so the assertion sees 1. A mere serial/parallel self-check
        // without this negative control would not demonstrate sensitivity.
        let broken = run_hooked(
            &state,
            &block,
            2,
            &AtomicBool::new(false),
            &OmitReadAfterWrite,
        );
        assert_eq!(broken, Outcome::Rejected);
        assert_ne!(broken, serial);
        assert_eq!(state.get(&key(1)), Some(1));
    }

    #[test]
    fn graph_preserves_write_after_read_and_aliased_move_conflicts() {
        let block = Block::new(vec![
            Operation::AssertEquals {
                key: key(1),
                value: 10,
            },
            Operation::Set {
                key: key(1),
                value: 12,
            },
            Operation::Move {
                source: key(2),
                destination: key(1),
                amount: 1,
            },
        ])
        .unwrap();
        let plan = dependency_plan(&block);
        assert_eq!(plan.edges, vec![(0, 1), (0, 2), (1, 2)]);
        assert_eq!(plan.waves, vec![vec![0], vec![1], vec![2]]);
    }

    #[test]
    fn unsupported_worker_configuration_and_pre_cancel_are_local_aborts() {
        let (state, block) = independent_fixture();
        for workers in [0, 3, 5, usize::MAX] {
            assert_eq!(
                execute_parallel(&state, &block, workers, &AtomicBool::new(false)),
                Outcome::Aborted(AbortReason::InvalidWorkerCount)
            );
        }
        assert_eq!(
            execute_parallel(&state, &block, 4, &AtomicBool::new(true)),
            Outcome::Aborted(AbortReason::Cancelled)
        );
    }

    #[test]
    fn fixed_dependency_storage_and_maximum_hot_key_graph_are_bounded() {
        assert_eq!(
            std::mem::size_of::<[[u64; PREDECESSOR_WORDS]; MAX_OPERATIONS]>(),
            8192
        );
        let block = Block::new(
            (0..MAX_OPERATIONS)
                .map(|value| Operation::Set {
                    key: key(1),
                    value: value as u64,
                })
                .collect(),
        )
        .unwrap();
        let plan = dependency_plan(&block);
        assert_eq!(plan.edges.len(), 32_640);
        assert_eq!(plan.waves.len(), 256);
        assert!(plan.waves.iter().all(|wave| wave.len() == 1));
        let state = State::new(vec![(key(1), 0)]).unwrap();
        assert_eq!(
            execute_parallel(&state, &block, 4, &AtomicBool::new(false)),
            execute_serial(&state, &block)
        );
    }
}
