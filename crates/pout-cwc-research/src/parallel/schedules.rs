//! Finite, test-only scheduling exploration on the real wave executor.
//!
//! This enumerates operation-completion boundaries, not CPU instructions or all
//! possible inputs. A wave barrier and the executor's contiguous worker chunks
//! restrict which of the six three-operation permutations are feasible. No
//! external graph, order or clock enters the normal executor.

use super::*;
use crate::codec::{encode_execution, encode_results, encode_state};
use crate::model::{Key, Operation};
use crate::serial::execute_serial;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

const PERMUTATIONS: [[u16; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];
const CONTROLLER_BUDGET: Duration = Duration::from_secs(5);

fn key(value: u8) -> Key {
    [value; 32]
}

#[derive(Clone)]
struct Access {
    reads: BTreeSet<Key>,
    writes: BTreeSet<Key>,
}

// Independent access reconstruction: deliberately does not call access_sets(),
// dependency_plan(), Graph::derive() or the operation evaluator.
fn access(operation: &Operation) -> Access {
    let (reads, writes): (Vec<Key>, Vec<Key>) = match operation {
        Operation::AssertEquals { key, .. } => (vec![*key], vec![]),
        Operation::Set { key, .. } => (vec![], vec![*key]),
        Operation::Add { key, .. } => (vec![*key], vec![*key]),
        Operation::Move {
            source,
            destination,
            ..
        } => (vec![*source, *destination], vec![*source, *destination]),
    };
    Access {
        reads: reads.into_iter().collect(),
        writes: writes.into_iter().collect(),
    }
}

fn conflict(left: &Access, right: &Access) -> bool {
    !left.writes.is_disjoint(&right.reads)
        || !left.writes.is_disjoint(&right.writes)
        || !left.reads.is_disjoint(&right.writes)
}

fn independently_derived_plan(block: &Block) -> DependencyPlan {
    let accesses: Vec<_> = block.operations().iter().map(access).collect();
    let mut edges = Vec::new();
    let mut depths = vec![0; accesses.len()];
    for later in 0..accesses.len() {
        for earlier in 0..later {
            if conflict(&accesses[earlier], &accesses[later]) {
                edges.push((earlier as u16, later as u16));
                depths[later] = depths[later].max(depths[earlier] + 1);
            }
        }
    }
    // Public plans list edges by (earlier, later), not discovery order.
    edges.sort_unstable();
    let waves = (0..=depths.iter().copied().max().unwrap_or(0))
        .map(|depth| {
            depths
                .iter()
                .enumerate()
                .filter(|(_, actual)| **actual == depth)
                .map(|(index, _)| index as u16)
                .collect()
        })
        .collect();
    DependencyPlan { edges, waves }
}

#[derive(Clone)]
struct Assignment {
    wave: usize,
    worker: usize,
    operations: Vec<u16>,
}

fn assignments(plan: &DependencyPlan, workers: usize) -> Vec<Assignment> {
    let mut assignments = Vec::new();
    for (wave_index, wave) in plan.waves.iter().enumerate() {
        let chunk_size = wave.len().div_ceil(workers.min(wave.len()));
        for (worker, operations) in wave.chunks(chunk_size).enumerate() {
            assignments.push(Assignment {
                wave: wave_index,
                worker,
                operations: operations.to_vec(),
            });
        }
    }
    assignments
}

fn feasible(plan: &DependencyPlan, workers: usize, order: &[u16; 3]) -> bool {
    if !matches!(workers, 1 | 2 | 4) {
        return false;
    }
    let mut position = [usize::MAX; 3];
    for (offset, index) in order.iter().enumerate() {
        let Some(position) = position.get_mut(usize::from(*index)) else {
            return false;
        };
        if *position != usize::MAX {
            return false;
        }
        *position = offset;
    }
    // Every member of a previous wave completes before any later-wave member,
    // even if a looser topological schedule would permit another ordering.
    for wave_pair in plan.waves.windows(2) {
        if wave_pair[0].iter().any(|before| {
            wave_pair[1]
                .iter()
                .any(|after| position[usize::from(*before)] > position[usize::from(*after)])
        }) {
            return false;
        }
    }
    assignments(plan, workers).iter().all(|assignment| {
        assignment
            .operations
            .windows(2)
            .all(|pair| position[usize::from(pair[0])] < position[usize::from(pair[1])])
    })
}

#[derive(Default)]
struct Observations {
    admitted: BTreeMap<u16, thread::ThreadId>,
    active: BTreeSet<u16>,
    completed: Vec<u16>,
    spawns: Vec<(usize, usize)>,
    abandoned: BTreeSet<u16>,
    peak_active: usize,
}

struct ControlledSchedule {
    plan: DependencyPlan,
    assignments: Vec<Assignment>,
    accesses: Vec<Access>,
    order: [u16; 3],
    observations: Mutex<Observations>,
    changed: Condvar,
    deadline: Instant,
}

impl ControlledSchedule {
    fn new(block: &Block, workers: usize, order: [u16; 3]) -> Option<Self> {
        if block.operations().len() != 3 {
            return None;
        }
        let plan = independently_derived_plan(block);
        if !feasible(&plan, workers, &order) {
            return None;
        }
        Some(Self {
            assignments: assignments(&plan, workers),
            plan,
            accesses: block.operations().iter().map(access).collect(),
            order,
            observations: Mutex::new(Observations::default()),
            changed: Condvar::new(),
            deadline: Instant::now() + CONTROLLER_BUDGET,
        })
    }

    fn wait<'a>(
        &self,
        mut observed: MutexGuard<'a, Observations>,
        predicate: impl Fn(&Observations) -> bool,
    ) -> Result<MutexGuard<'a, Observations>, AbortReason> {
        loop {
            // This is a finite local test-controller budget, never candidate
            // invalidity or a protocol deadline. Expiry is always an abort.
            let Some(remaining) = self.deadline.checked_duration_since(Instant::now()) else {
                return Err(AbortReason::LocalResourceFailure);
            };
            if predicate(&observed) {
                return Ok(observed);
            }
            let (next, _) = self.changed.wait_timeout(observed, remaining).unwrap();
            observed = next;
        }
    }

    fn assert_complete(&self) {
        let observed = self.observations.lock().unwrap();
        assert_eq!(observed.completed, self.order);
        assert_eq!(observed.admitted.len(), 3);
        assert!(observed.active.is_empty());
        assert!(observed.abandoned.is_empty());
        let expected_spawns: Vec<_> = self
            .assignments
            .iter()
            .map(|assignment| (assignment.wave, assignment.worker))
            .collect();
        assert_eq!(observed.spawns, expected_spawns);
        let mut owner_threads = Vec::new();
        for assignment in &self.assignments {
            let owner = observed.admitted[&assignment.operations[0]];
            assert!(assignment
                .operations
                .iter()
                .all(|index| observed.admitted[index] == owner));
            assert!(!owner_threads.contains(&owner));
            owner_threads.push(owner);
        }
        let expected_peak = self
            .plan
            .waves
            .iter()
            .enumerate()
            .map(|(wave, _)| {
                self.assignments
                    .iter()
                    .filter(|assignment| assignment.wave == wave)
                    .count()
            })
            .max()
            .unwrap();
        // Every assignment's first operation meets a bounded admission barrier,
        // so the test observes real simultaneous worker ownership, not just a
        // permutation manufactured from sequential completion records.
        assert_eq!(observed.peak_active, expected_peak);
    }

    fn abandon(&self, index: u16) {
        let mut observed = self.observations.lock().unwrap();
        assert!(observed.active.remove(&index));
        assert!(observed.abandoned.insert(index));
        self.changed.notify_all();
    }
}

impl TestHooks for ControlledSchedule {
    fn before_spawn(&self, worker: usize) -> Result<(), AbortReason> {
        let mut observed = self.observations.lock().unwrap();
        let wave = self
            .plan
            .waves
            .iter()
            .position(|wave| wave.iter().any(|index| !observed.completed.contains(index)))
            .unwrap();
        observed.spawns.push((wave, worker));
        Ok(())
    }

    fn before_operation(&self, index: u16) -> Result<(), AbortReason> {
        let assignment = self
            .assignments
            .iter()
            .find(|assignment| assignment.operations.contains(&index))
            .unwrap();
        let mut observed = self.observations.lock().unwrap();
        for other in &observed.active {
            assert!(
                !conflict(
                    &self.accesses[usize::from(index)],
                    &self.accesses[usize::from(*other)]
                ),
                "conflicting operations {index} and {other} active together"
            );
        }
        assert!(observed
            .admitted
            .insert(index, thread::current().id())
            .is_none());
        assert!(observed.active.insert(index));
        observed.peak_active = observed.peak_active.max(observed.active.len());
        self.changed.notify_all();
        let waited = self.wait(observed, |observed| {
            self.assignments
                .iter()
                .filter(|other| other.wave == assignment.wave)
                .all(|other| observed.admitted.contains_key(&other.operations[0]))
        });
        match waited {
            Ok(_) => Ok(()),
            Err(reason) => {
                self.abandon(index);
                Err(reason)
            }
        }
    }

    fn after_operation(&self, index: u16) -> Result<(), AbortReason> {
        let observed = self.observations.lock().unwrap();
        let waited = self.wait(observed, |observed| {
            self.order.get(observed.completed.len()) == Some(&index)
        });
        let mut observed = match waited {
            Ok(observed) => observed,
            Err(reason) => {
                self.abandon(index);
                return Err(reason);
            }
        };
        assert!(observed.active.remove(&index));
        observed.completed.push(index);
        self.changed.notify_all();
        Ok(())
    }
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

fn assert_same_complete_outcome(actual: &Outcome, reference: &Outcome) {
    assert_eq!(actual, reference);
    if let (Outcome::Accepted(actual), Outcome::Accepted(reference)) = (actual, reference) {
        assert_eq!(actual.results.len(), 3);
        assert_eq!(
            actual
                .results
                .iter()
                .map(|result| result.index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(
            encode_state(&actual.final_state),
            encode_state(&reference.final_state)
        );
        assert_eq!(
            encode_results(&actual.results),
            encode_results(&reference.results)
        );
        assert_eq!(encode_execution(actual), encode_execution(reference));
        assert_eq!(actual.state_root, reference.state_root);
    }
}

fn add(index: u8) -> Operation {
    Operation::Add {
        key: key(index),
        amount: 1,
    }
}

fn set(index: u8, value: u64) -> Operation {
    Operation::Set {
        key: key(index),
        value,
    }
}

fn assert_value(index: u8, value: u64) -> Operation {
    Operation::AssertEquals {
        key: key(index),
        value,
    }
}

fn move_value(source: u8, destination: u8, amount: u64) -> Operation {
    Operation::Move {
        source: key(source),
        destination: key(destination),
        amount,
    }
}

type Fixture = (&'static str, [Operation; 3], Vec<(u16, u16)>);

fn accepted_fixtures() -> Vec<Fixture> {
    vec![
        ("independent", [add(1), add(2), add(3)], vec![]),
        (
            "edge01",
            [set(1, 12), assert_value(1, 12), add(2)],
            vec![(0, 1)],
        ),
        (
            "edge02",
            [set(1, 12), add(2), assert_value(1, 12)],
            vec![(0, 2)],
        ),
        (
            "edge12",
            [add(2), set(1, 12), assert_value(1, 12)],
            vec![(1, 2)],
        ),
        (
            "fork",
            [set(1, 10), assert_value(1, 10), assert_value(1, 10)],
            vec![(0, 1), (0, 2)],
        ),
        (
            "join",
            [
                assert_value(1, 10),
                assert_value(2, 20),
                move_value(1, 2, 0),
            ],
            vec![(0, 2), (1, 2)],
        ),
        (
            "chain_without_direct02",
            [set(1, 10), move_value(1, 2, 0), assert_value(2, 20)],
            vec![(0, 1), (1, 2)],
        ),
        (
            "same_key_complete",
            [add(1), add(1), add(1)],
            vec![(0, 1), (0, 2), (1, 2)],
        ),
        (
            "write_after_read",
            [assert_value(1, 10), set(1, 12), add(2)],
            vec![(0, 1)],
        ),
        (
            "overlapping_moves",
            [
                move_value(1, 2, 1),
                move_value(2, 3, 2),
                move_value(1, 3, 1),
            ],
            vec![(0, 1), (0, 2), (1, 2)],
        ),
        (
            "shared_reads",
            [assert_value(1, 10), assert_value(1, 10), add(2)],
            vec![],
        ),
    ]
}

#[test]
fn all_eight_forward_graph_shapes_and_feasible_three_operation_completion_orders() {
    let state = State::new(vec![(key(1), 10), (key(2), 20), (key(3), 30)]).unwrap();
    let original = state.clone();
    let mut graph_masks = BTreeSet::new();
    let mut schedules = 0;
    for (name, operations, expected_edges) in accepted_fixtures() {
        let block = Block::new(operations.to_vec()).unwrap();
        let plan = independently_derived_plan(&block);
        assert_eq!(plan.edges, expected_edges, "fixture {name}");
        assert_eq!(dependency_plan(&block), plan, "fixture {name}");
        let mask = [(0, 1), (0, 2), (1, 2)]
            .iter()
            .enumerate()
            .fold(0_u8, |mask, (bit, edge)| {
                mask | (u8::from(plan.edges.contains(edge)) << bit)
            });
        graph_masks.insert(mask);
        let reference = execute_serial(&state, &block);
        assert!(matches!(reference, Outcome::Accepted(_)), "fixture {name}");
        for workers in [1, 2, 4] {
            for order in PERMUTATIONS {
                let Some(controller) = ControlledSchedule::new(&block, workers, order) else {
                    continue;
                };
                let actual = run_hooked(
                    &state,
                    &block,
                    workers,
                    &AtomicBool::new(false),
                    &controller,
                );
                assert_same_complete_outcome(&actual, &reference);
                controller.assert_complete();
                assert_eq!(state, original);
                schedules += 1;
            }
        }
    }
    assert_eq!(graph_masks, (0_u8..8).collect());
    assert_eq!(schedules, 59);
    println!("controlled_schedule_coverage accepted_fixtures=11 graph_masks=8 schedules=59 workers=1,2,4");
}

#[test]
fn infeasible_orders_are_rejected_by_the_test_controller_before_worker_creation() {
    let block = Block::new(vec![add(1), add(2), add(3)]).unwrap();
    let plan = independently_derived_plan(&block);
    let feasible_orders = |workers| {
        PERMUTATIONS
            .into_iter()
            .filter(|order| feasible(&plan, workers, order))
            .collect::<Vec<_>>()
    };
    assert_eq!(feasible_orders(1), vec![[0, 1, 2]]);
    assert_eq!(feasible_orders(2), vec![[0, 1, 2], [0, 2, 1], [2, 0, 1]]);
    assert_eq!(feasible_orders(4), PERMUTATIONS);
    assert!(ControlledSchedule::new(&block, 2, [1, 0, 2]).is_none());
    assert!(ControlledSchedule::new(&block, 4, [0, 0, 2]).is_none());
    assert!(ControlledSchedule::new(&block, 4, [0, 1, 3]).is_none());
    let dependent = Block::new(vec![set(1, 12), assert_value(1, 12), add(2)]).unwrap();
    // Topological 0,1,2 is nevertheless infeasible: wave0=[0,2] must finish
    // before wave1=[1]. The controller mirrors the actual barrier contract.
    assert!(ControlledSchedule::new(&dependent, 4, [0, 1, 2]).is_none());
}

#[test]
fn rejected_three_operation_waves_never_publish_independent_successes() {
    let state = State::new(vec![(key(1), u64::MAX), (key(2), 20), (key(3), 30)]).unwrap();
    let original = state.clone();
    let mut schedules = 0;
    for operations in [
        [assert_value(1, 0), add(2), add(3)],
        [add(1), add(2), add(3)],
    ] {
        let block = Block::new(operations.to_vec()).unwrap();
        assert_eq!(execute_serial(&state, &block), Outcome::Rejected);
        for workers in [1, 2, 4] {
            for order in PERMUTATIONS {
                let Some(controller) = ControlledSchedule::new(&block, workers, order) else {
                    continue;
                };
                assert_eq!(
                    run_hooked(
                        &state,
                        &block,
                        workers,
                        &AtomicBool::new(false),
                        &controller
                    ),
                    Outcome::Rejected
                );
                controller.assert_complete();
                assert_eq!(state, original);
                schedules += 1;
            }
        }
    }
    assert_eq!(schedules, 20);
    println!(
        "controlled_schedule_coverage rejected_fixtures=2 schedules=20 causes=assertion,overflow"
    );
}

#[test]
fn expired_local_schedule_budget_aborts_without_invalidating_a_valid_candidate() {
    let state = State::new(vec![(key(1), 10), (key(2), 20), (key(3), 30)]).unwrap();
    let original = state.clone();
    let block = Block::new(vec![add(1), add(2), add(3)]).unwrap();
    let mut controller = ControlledSchedule::new(&block, 4, [0, 1, 2]).unwrap();
    controller.deadline = Instant::now();
    assert_eq!(
        run_hooked(&state, &block, 4, &AtomicBool::new(false), &controller),
        Outcome::Aborted(AbortReason::LocalResourceFailure)
    );
    assert_eq!(state, original);
    let observed = controller.observations.lock().unwrap();
    assert!(observed.active.is_empty());
    assert_eq!(observed.abandoned, [0, 1, 2].into_iter().collect());
    assert!(observed.completed.is_empty());
    drop(observed);
    assert_same_complete_outcome(
        &execute_parallel(&state, &block, 4, &AtomicBool::new(false)),
        &execute_serial(&state, &block),
    );
}

#[derive(Clone, Copy)]
enum Failure {
    Panic,
    Cancel,
    Spawn,
}

#[derive(Default)]
struct FailureObservations {
    entered: BTreeMap<u16, thread::ThreadId>,
    completed: BTreeSet<u16>,
    triggered: bool,
}

struct ControlledFailure<'a> {
    failure: Failure,
    cancellation: &'a AtomicBool,
    observations: Mutex<FailureObservations>,
    changed: Condvar,
    deadline: Instant,
}

impl ControlledFailure<'_> {
    fn wait<'a>(
        &self,
        mut observed: MutexGuard<'a, FailureObservations>,
        predicate: impl Fn(&FailureObservations) -> bool,
    ) -> Result<MutexGuard<'a, FailureObservations>, AbortReason> {
        while !predicate(&observed) {
            let Some(remaining) = self.deadline.checked_duration_since(Instant::now()) else {
                return Err(AbortReason::LocalResourceFailure);
            };
            let (next, _) = self.changed.wait_timeout(observed, remaining).unwrap();
            observed = next;
        }
        Ok(observed)
    }
}

impl TestHooks for ControlledFailure<'_> {
    fn before_spawn(&self, worker: usize) -> Result<(), AbortReason> {
        if matches!(self.failure, Failure::Spawn) && worker == 2 {
            self.observations.lock().unwrap().triggered = true;
            self.changed.notify_all();
            Err(AbortReason::WorkerSpawnFailure)
        } else {
            Ok(())
        }
    }

    fn before_operation(&self, index: u16) -> Result<(), AbortReason> {
        let mut observed = self.observations.lock().unwrap();
        assert!(observed
            .entered
            .insert(index, thread::current().id())
            .is_none());
        self.changed.notify_all();
        if matches!(self.failure, Failure::Panic) && index == 0 {
            let mut observed = self.wait(observed, |observed| {
                observed.completed.contains(&1) && observed.completed.contains(&2)
            })?;
            observed.triggered = true;
            // Panic outside the mutex: sibling completion and ownership remain
            // inspectable rather than being obscured by lock poisoning.
            drop(observed);
            panic!("controlled three-operation synthetic worker panic");
        }
        Ok(())
    }

    fn after_operation(&self, index: u16) -> Result<(), AbortReason> {
        let observed = self.observations.lock().unwrap();
        let mut observed = match self.failure {
            Failure::Cancel if index == 0 => {
                let mut observed = self.wait(observed, |observed| observed.entered.len() == 3)?;
                observed.triggered = true;
                self.cancellation.store(true, Ordering::Release);
                self.changed.notify_all();
                observed
            }
            Failure::Cancel | Failure::Spawn => {
                self.wait(observed, |observed| observed.triggered)?
            }
            Failure::Panic => observed,
        };
        assert!(observed.completed.insert(index));
        self.changed.notify_all();
        Ok(())
    }
}

#[test]
fn three_operation_panic_joins_two_successful_siblings_and_preserves_retry() {
    assert_controlled_failure(Failure::Panic, AbortReason::WorkerPanic, &[1, 2]);
}

#[test]
fn three_operation_cancel_joins_all_evaluated_workers_and_preserves_retry() {
    assert_controlled_failure(Failure::Cancel, AbortReason::Cancelled, &[0, 1, 2]);
}

#[test]
fn third_spawn_failure_joins_both_owned_workers_and_preserves_retry() {
    assert_controlled_failure(Failure::Spawn, AbortReason::WorkerSpawnFailure, &[0, 1]);
}

fn assert_controlled_failure(failure: Failure, reason: AbortReason, completed: &[u16]) {
    let state = State::new(vec![(key(1), 10), (key(2), 20), (key(3), 30)]).unwrap();
    let original = state.clone();
    let block = Block::new(vec![add(1), add(2), add(3)]).unwrap();
    let cancellation = AtomicBool::new(false);
    let controller = ControlledFailure {
        failure,
        cancellation: &cancellation,
        observations: Mutex::new(FailureObservations::default()),
        changed: Condvar::new(),
        deadline: Instant::now() + CONTROLLER_BUDGET,
    };
    assert_eq!(
        run_hooked(&state, &block, 4, &cancellation, &controller),
        Outcome::Aborted(reason)
    );
    assert_eq!(state, original);
    let observed = controller.observations.lock().unwrap();
    assert!(observed.triggered);
    assert_eq!(observed.completed, completed.iter().copied().collect());
    let expected_entries = if matches!(failure, Failure::Spawn) {
        2
    } else {
        3
    };
    assert_eq!(observed.entered.len(), expected_entries);
    let owned_threads: BTreeSet<_> = observed
        .entered
        .values()
        .map(|id| format!("{id:?}"))
        .collect();
    assert_eq!(owned_threads.len(), expected_entries);
    drop(observed);
    cancellation.store(false, Ordering::Release);
    assert_same_complete_outcome(
        &execute_parallel(&state, &block, 4, &cancellation),
        &execute_serial(&state, &block),
    );
    assert_eq!(
        controller.observations.lock().unwrap().completed,
        completed.iter().copied().collect()
    );
    assert_eq!(state, original);
}
