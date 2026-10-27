//! Shared containment for adapter operations before returning to PyO3.
use std::panic::{catch_unwind, AssertUnwindSafe};

pub(super) fn boundary<T>(f: impl FnOnce() -> T) -> Result<T, ()> {
    // Engine state is not reused after a failure: each call creates its own run.
    catch_unwind(AssertUnwindSafe(f)).map_err(|_| ())
}
