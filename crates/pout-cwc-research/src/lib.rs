//! An isolated, finite in-memory CWC experiment using artificial counters.
//!
//! No operational service depends on this crate. Matching execution results do
//! not establish useful-service scarcity, consensus security or durable commits.

pub mod codec;
pub mod model;
pub mod parallel;
pub mod serial;

pub use codec::{
    decode_block, decode_execution, decode_results, decode_state, encode_block, encode_execution,
    encode_results, encode_state, state_root,
};
pub use model::{
    AbortReason, AccessSets, Block, Execution, InvalidInput, Key, Operation, OperationResult,
    Outcome, State, MAX_BLOCK_BYTES, MAX_EXECUTION_BYTES, MAX_OPERATIONS, MAX_RESULTS_BYTES,
    MAX_STATE_BYTES, MAX_STATE_KEYS, VERSION,
};
#[cfg(feature = "research-controls")]
pub use parallel::research::{execute_reverse_first_pair, ControlledRun, UnsupportedSchedule};
pub use parallel::{dependency_plan, execute_parallel, DependencyPlan};
pub use serial::execute_serial;
