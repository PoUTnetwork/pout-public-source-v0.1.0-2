//! Nondefault, finite controls for an isolated integration experiment.
//!
//! This uses the actual executor, graph derivation and evaluation path. Only the
//! first two independent worker completion boundaries are controlled. These
//! observations are local test evidence, never protocol input or consensus proof.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use super::{dependency_plan, execute_with_hooks, Hooks};
use crate::{AbortReason, Block, Outcome, State};

const CONTROLLER_BUDGET: Duration = Duration::from_secs(5);
const CANCELLATION_POLL: Duration = Duration::from_millis(25);

/// Unsupported local controller configuration; not candidate invalidity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum UnsupportedSchedule {
    #[error("reverse-pair controller requires two or four configured workers")]
    WorkerCount,
    #[error("reverse-pair controller requires first derived wave exactly [0, 1]")]
    FirstWave,
}

/// Outcome from the real executor plus a bounded first-pair observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlledRun {
    pub outcome: Outcome,
    /// At most two actual after-evaluation hook completions, never reordered.
    pub completion_order: Vec<u16>,
    /// Number of distinct real worker thread IDs observed at first-pair entry.
    pub distinct_worker_count: usize,
}

/// Run the actual parallel executor while completing operations 1 then 0.
///
/// Only workers 2/4 and first derived wave exactly `[0, 1]` are admitted, making
/// the pair belong to separate real worker assignments. Later waves run normally.
/// Every owned worker is joined by the existing executor before return. A fixed
/// five-second local synchronization budget or cancellation produces `Aborted`,
/// never deterministic rejection. No controller can change access sets, graph,
/// arithmetic, canonical results or the final acceptance-only state boundary.
pub fn execute_reverse_first_pair(
    initial: &State,
    block: &Block,
    workers: usize,
    cancellation: &AtomicBool,
) -> Result<ControlledRun, UnsupportedSchedule> {
    if !matches!(workers, 2 | 4) {
        return Err(UnsupportedSchedule::WorkerCount);
    }
    if dependency_plan(block).waves.first().map(Vec::as_slice) != Some(&[0, 1][..]) {
        return Err(UnsupportedSchedule::FirstWave);
    }
    let control = ReversePair::new(cancellation);
    let outcome = execute_with_hooks(
        initial,
        block,
        workers,
        cancellation,
        &Hooks {
            reverse_pair: Some(&control),
            ..Hooks::default()
        },
    );
    Ok(control.finish(outcome))
}

#[derive(Default)]
struct Observed {
    entrants: [Option<ThreadId>; 2],
    completed: [Option<u16>; 2],
    completed_count: usize,
}

pub(super) struct ReversePair<'a> {
    observed: Mutex<Observed>,
    changed: Condvar,
    deadline: Instant,
    cancellation: &'a AtomicBool,
}

impl<'a> ReversePair<'a> {
    fn new(cancellation: &'a AtomicBool) -> Self {
        Self {
            observed: Mutex::new(Observed::default()),
            changed: Condvar::new(),
            deadline: Instant::now() + CONTROLLER_BUDGET,
            cancellation,
        }
    }

    fn wait<'g>(
        &self,
        mut observed: MutexGuard<'g, Observed>,
        ready: impl Fn(&Observed) -> bool,
    ) -> Result<MutexGuard<'g, Observed>, AbortReason> {
        loop {
            if self.cancellation.load(Ordering::Acquire) {
                return Err(AbortReason::Cancelled);
            }
            let remaining = self
                .deadline
                .checked_duration_since(Instant::now())
                .ok_or(AbortReason::LocalResourceFailure)?;
            if ready(&observed) {
                return Ok(observed);
            }
            observed = self
                .changed
                .wait_timeout(observed, remaining.min(CANCELLATION_POLL))
                .map_err(|_| AbortReason::LocalResourceFailure)?
                .0;
        }
    }

    pub(super) fn before_operation(&self, index: u16) -> Result<(), AbortReason> {
        if index > 1 {
            return Ok(());
        }
        let mut observed = self
            .observed
            .lock()
            .map_err(|_| AbortReason::LocalResourceFailure)?;
        if observed.entrants[usize::from(index)]
            .replace(thread::current().id())
            .is_some()
        {
            return Err(AbortReason::LocalResourceFailure);
        }
        self.changed.notify_all();
        drop(self.wait(observed, |state| state.entrants.iter().all(Option::is_some))?);
        Ok(())
    }

    pub(super) fn after_operation(&self, index: u16) -> Result<(), AbortReason> {
        if index > 1 {
            return Ok(());
        }
        let observed = self
            .observed
            .lock()
            .map_err(|_| AbortReason::LocalResourceFailure)?;
        let mut observed = self.wait(observed, |state| {
            index == 1 || state.completed[..state.completed_count].contains(&Some(1))
        })?;
        if observed.completed_count >= 2 {
            return Err(AbortReason::LocalResourceFailure);
        }
        let position = observed.completed_count;
        observed.completed[position] = Some(index);
        observed.completed_count += 1;
        self.changed.notify_all();
        Ok(())
    }

    fn finish(self, outcome: Outcome) -> ControlledRun {
        let observed = self
            .observed
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let distinct_worker_count = match observed.entrants {
            [None, None] => 0,
            [Some(first), Some(second)] if first != second => 2,
            _ => 1,
        };
        ControlledRun {
            outcome,
            completion_order: observed.completed.into_iter().flatten().collect(),
            distinct_worker_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encode_execution, execute_parallel, execute_serial, Operation};

    fn fixture() -> (State, Block) {
        let (a, b) = ([1; 32], [2; 32]);
        (
            State::new(vec![(a, 10), (b, 20)]).unwrap(),
            Block::new(vec![
                Operation::Add { key: a, amount: 5 },
                Operation::Add { key: b, amount: 7 },
                Operation::Move {
                    source: a,
                    destination: b,
                    amount: 3,
                },
            ])
            .unwrap(),
        )
    }

    #[test]
    fn actual_reverse_pair_matches_canonical_serial_and_default_execution() {
        let (initial, block) = fixture();
        let original = initial.clone();
        let expected = execute_serial(&initial, &block);
        let Outcome::Accepted(expected_execution) = &expected else {
            panic!("valid fixture");
        };
        for workers in [1, 2, 4] {
            assert_eq!(
                execute_parallel(&initial, &block, workers, &AtomicBool::new(false)),
                expected
            );
        }
        for workers in [2, 4] {
            let run =
                execute_reverse_first_pair(&initial, &block, workers, &AtomicBool::new(false))
                    .unwrap();
            assert_eq!(run.completion_order, vec![1, 0]);
            assert_eq!(run.distinct_worker_count, 2);
            assert_eq!(run.outcome, expected);
            let Outcome::Accepted(actual) = run.outcome else {
                panic!("controlled valid fixture");
            };
            assert_eq!(
                encode_execution(&actual),
                encode_execution(expected_execution)
            );
        }
        assert_eq!(initial, original);
    }

    #[test]
    fn unsupported_controller_shapes_are_not_protocol_rejections() {
        let (initial, block) = fixture();
        let cancel = AtomicBool::new(false);
        for workers in [0, 1, 3, 5, usize::MAX] {
            assert_eq!(
                execute_reverse_first_pair(&initial, &block, workers, &cancel),
                Err(UnsupportedSchedule::WorkerCount)
            );
        }
        for operations in [
            vec![],
            vec![Operation::Add {
                key: [1; 32],
                amount: 1,
            }],
            vec![
                Operation::Add {
                    key: [1; 32],
                    amount: 1,
                },
                Operation::Add {
                    key: [1; 32],
                    amount: 2,
                },
            ],
            vec![
                Operation::Add {
                    key: [1; 32],
                    amount: 1,
                },
                Operation::Add {
                    key: [2; 32],
                    amount: 1,
                },
                Operation::Add {
                    key: [3; 32],
                    amount: 1,
                },
            ],
        ] {
            assert_eq!(
                execute_reverse_first_pair(&initial, &Block::new(operations).unwrap(), 2, &cancel),
                Err(UnsupportedSchedule::FirstWave)
            );
        }
    }

    #[test]
    fn precancellation_exposes_no_trace_or_state_and_normal_retry_succeeds() {
        let (initial, block) = fixture();
        let original = initial.clone();
        let run = execute_reverse_first_pair(&initial, &block, 2, &AtomicBool::new(true)).unwrap();
        assert_eq!(run.outcome, Outcome::Aborted(AbortReason::Cancelled));
        assert!(run.completion_order.is_empty());
        assert_eq!(run.distinct_worker_count, 0);
        assert_eq!(initial, original);
        assert_eq!(
            execute_reverse_first_pair(&initial, &block, 2, &AtomicBool::new(false))
                .unwrap()
                .outcome,
            execute_serial(&initial, &block)
        );
    }

    #[test]
    fn exhausted_local_controller_budget_aborts_instead_of_rejecting_candidate() {
        let (initial, block) = fixture();
        let original = initial.clone();
        let cancel = AtomicBool::new(false);
        let mut control = ReversePair::new(&cancel);
        control.deadline = Instant::now();
        let outcome = execute_with_hooks(
            &initial,
            &block,
            2,
            &cancel,
            &Hooks {
                reverse_pair: Some(&control),
                ..Hooks::default()
            },
        );
        let run = control.finish(outcome);
        assert_eq!(
            run.outcome,
            Outcome::Aborted(AbortReason::LocalResourceFailure)
        );
        assert!(run.completion_order.is_empty());
        assert_eq!(initial, original);
        assert!(matches!(
            execute_serial(&initial, &block),
            Outcome::Accepted(_)
        ));
    }

    #[test]
    fn later_rejection_publishes_neither_completed_first_pair_nor_partial_state() {
        let (initial, valid) = fixture();
        let original = initial.clone();
        let mut operations = valid.operations().to_vec();
        operations[2] = Operation::Move {
            source: [1; 32],
            destination: [2; 32],
            amount: 500,
        };
        let block = Block::new(operations).unwrap();
        let run = execute_reverse_first_pair(&initial, &block, 2, &AtomicBool::new(false)).unwrap();
        assert_eq!(run.completion_order, vec![1, 0]);
        assert_eq!(run.distinct_worker_count, 2);
        assert_eq!(run.outcome, Outcome::Rejected);
        assert_eq!(run.outcome, execute_serial(&initial, &block));
        assert_eq!(initial, original);
    }
}
