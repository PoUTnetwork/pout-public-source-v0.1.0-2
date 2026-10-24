//! Test-fixture custody: commit success or sanitized local failure before callers
//! format reports or fail assertions. This does not recover process aborts.

use std::panic::{catch_unwind, UnwindSafe};
use std::sync::OnceLock;

#[derive(Debug, Eq, PartialEq)]
pub enum InitFailure<E> {
    Experiment(E),
    Panicked,
}

/// A returned error or caught unwinding panic is retained. No panic payload,
/// witness, input or private path becomes part of the cached failure.
///
/// Serialization/output must happen AFTER this function returns. Aborts and
/// double panics are outside this in-process recovery guarantee.
pub fn get_or_init<T, E, F>(
    cell: &OnceLock<Result<T, InitFailure<E>>>,
    initialize: F,
) -> Result<&T, &InitFailure<E>>
where
    F: FnOnce() -> Result<T, E> + UnwindSafe,
{
    cell.get_or_init(|| match catch_unwind(initialize) {
        Ok(result) => result.map_err(InitFailure::Experiment),
        Err(_) => Err(InitFailure::Panicked),
    })
    .as_ref()
}
