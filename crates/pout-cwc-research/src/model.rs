//! Finite typed state, operations and outcomes. No host data enters execution.

use thiserror::Error;

pub type Key = [u8; 32];
pub const VERSION: u16 = 1;
pub const MAX_STATE_KEYS: usize = 1_024;
pub const MAX_OPERATIONS: usize = 256;
pub const MAX_BLOCK_BYTES: usize = 65_536;
pub const MAX_STATE_BYTES: usize = 41_000;
pub const MAX_RESULTS_BYTES: usize = 21_300;
pub const MAX_EXECUTION_BYTES: usize = 65_536;

/// Construction/decoding errors. `LocalResourceFailure` describes an incomplete
/// local attempt, not evidence that the candidate is deterministically invalid.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum InvalidInput {
    #[error("malformed or truncated input")]
    Malformed,
    #[error("unsupported encoding version")]
    UnsupportedVersion,
    #[error("finite model resource limit exceeded")]
    LimitExceeded,
    #[error("duplicate state key")]
    DuplicateKey,
    #[error("non-canonical ordering or trailing input")]
    NonCanonical,
    #[error("invalid canonical operation result")]
    InvalidResult,
    #[error("state root does not match canonical state")]
    RootMismatch,
    /// Local allocation failure is not deterministic candidate invalidity.
    #[error("local allocation prevented decoding")]
    LocalResourceFailure,
}

/// A finite map whose private representation is strictly sorted by key bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    entries: Vec<(Key, u64)>,
}

impl State {
    /// Construct canonical state. Callers own the allocation of the supplied Vec;
    /// decoders check input-driven bounds before allocating one.
    pub fn new(mut entries: Vec<(Key, u64)>) -> Result<Self, InvalidInput> {
        if entries.len() > MAX_STATE_KEYS {
            return Err(InvalidInput::LimitExceeded);
        }
        entries.sort_unstable_by_key(|(key, _)| *key);
        if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(InvalidInput::DuplicateKey);
        }
        Ok(Self { entries })
    }

    pub fn entries(&self) -> &[(Key, u64)] {
        &self.entries
    }

    pub fn get(&self, key: &Key) -> Option<u64> {
        self.entries
            .binary_search_by_key(key, |(entry_key, _)| *entry_key)
            .ok()
            .map(|index| self.entries[index].1)
    }

    pub fn try_clone(&self) -> Result<Self, AbortReason> {
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(self.entries.len())
            .map_err(|_| AbortReason::LocalResourceFailure)?;
        entries.extend_from_slice(&self.entries);
        Ok(Self { entries })
    }

    pub(crate) fn set_existing(&mut self, key: &Key, value: u64) -> Result<(), InvalidCandidate> {
        let index = self
            .entries
            .binary_search_by_key(key, |(entry_key, _)| *entry_key)
            .map_err(|_| InvalidCandidate::MissingKey)?;
        self.entries[index].1 = value;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operation {
    AssertEquals {
        key: Key,
        value: u64,
    },
    Set {
        key: Key,
        value: u64,
    },
    Add {
        key: Key,
        amount: u64,
    },
    Move {
        source: Key,
        destination: Key,
        amount: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessSets {
    pub reads: Vec<Key>,
    pub writes: Vec<Key>,
}

impl Operation {
    /// Access sets are derived from typed operations, never supplied by a block.
    /// Key existence is validated structurally, including the key of `Set`.
    pub fn access_sets(&self) -> AccessSets {
        match self {
            Self::AssertEquals { key, .. } => AccessSets {
                reads: vec![*key],
                writes: Vec::new(),
            },
            Self::Set { key, .. } => AccessSets {
                reads: Vec::new(),
                writes: vec![*key],
            },
            Self::Add { key, .. } => AccessSets {
                reads: vec![*key],
                writes: vec![*key],
            },
            Self::Move {
                source,
                destination,
                ..
            } => {
                let mut keys = vec![*source, *destination];
                keys.sort_unstable();
                keys.dedup();
                AccessSets {
                    reads: keys.clone(),
                    writes: keys,
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    operations: Vec<Operation>,
}

impl Block {
    pub fn new(operations: Vec<Operation>) -> Result<Self, InvalidInput> {
        if operations.len() > MAX_OPERATIONS {
            return Err(InvalidInput::LimitExceeded);
        }
        Ok(Self { operations })
    }

    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }
}

/// One result in canonical operation order. Values are strictly key-sorted and
/// contain one observed assertion value, or the operation's post-write values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationResult {
    pub index: u16,
    pub values: Vec<(Key, u64)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Execution {
    pub final_state: State,
    pub results: Vec<OperationResult>,
    pub state_root: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbortReason {
    Cancelled,
    InvalidWorkerCount,
    WorkerPanic,
    WorkerSpawnFailure,
    LocalResourceFailure,
}

/// Only complete successful candidates contain a new state. A local abort does
/// not assert protocol invalidity. Rejection deliberately exposes no worker's
/// timing-dependent operation error or speculative result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Accepted(Execution),
    Rejected,
    Aborted(AbortReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InvalidCandidate {
    MissingKey,
    SameKeyMove,
    Arithmetic,
    Assertion,
}

#[derive(Debug)]
pub(crate) struct Evaluation {
    pub writes: Vec<(Key, u64)>,
    pub result: OperationResult,
}

pub(crate) fn validate_candidate(state: &State, block: &Block) -> Result<(), InvalidCandidate> {
    for operation in block.operations() {
        match operation {
            Operation::AssertEquals { key, .. }
            | Operation::Set { key, .. }
            | Operation::Add { key, .. } => {
                state.get(key).ok_or(InvalidCandidate::MissingKey)?;
            }
            Operation::Move {
                source,
                destination,
                ..
            } => {
                if source == destination {
                    return Err(InvalidCandidate::SameKeyMove);
                }
                state.get(source).ok_or(InvalidCandidate::MissingKey)?;
                state.get(destination).ok_or(InvalidCandidate::MissingKey)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn evaluate_operation(
    state: &State,
    operation: &Operation,
    index: u16,
) -> Result<Evaluation, InvalidCandidate> {
    let mut values = match operation {
        Operation::AssertEquals { key, value } => {
            let observed = state.get(key).ok_or(InvalidCandidate::MissingKey)?;
            if observed != *value {
                return Err(InvalidCandidate::Assertion);
            }
            return Ok(Evaluation {
                writes: Vec::new(),
                result: OperationResult {
                    index,
                    values: vec![(*key, observed)],
                },
            });
        }
        Operation::Set { key, value } => {
            state.get(key).ok_or(InvalidCandidate::MissingKey)?;
            vec![(*key, *value)]
        }
        Operation::Add { key, amount } => {
            let next = state
                .get(key)
                .ok_or(InvalidCandidate::MissingKey)?
                .checked_add(*amount)
                .ok_or(InvalidCandidate::Arithmetic)?;
            vec![(*key, next)]
        }
        Operation::Move {
            source,
            destination,
            amount,
        } => {
            if source == destination {
                return Err(InvalidCandidate::SameKeyMove);
            }
            let next_source = state
                .get(source)
                .ok_or(InvalidCandidate::MissingKey)?
                .checked_sub(*amount)
                .ok_or(InvalidCandidate::Arithmetic)?;
            let next_destination = state
                .get(destination)
                .ok_or(InvalidCandidate::MissingKey)?
                .checked_add(*amount)
                .ok_or(InvalidCandidate::Arithmetic)?;
            vec![(*source, next_source), (*destination, next_destination)]
        }
    };
    values.sort_unstable_by_key(|(key, _)| *key);
    Ok(Evaluation {
        writes: values.clone(),
        result: OperationResult { index, values },
    })
}

pub(crate) fn finish_execution(state: State, results: Vec<OperationResult>) -> Execution {
    let state_root = crate::codec::state_root(&state);
    Execution {
        final_state: state,
        results,
        state_root,
    }
}
