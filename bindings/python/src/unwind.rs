//! Contención común a las operaciones del adaptador antes de volver a PyO3.
use std::panic::{catch_unwind, AssertUnwindSafe};

pub(super) fn boundary<T>(f: impl FnOnce() -> T) -> Result<T, ()> {
    // No se reutiliza estado del motor tras un fallo: cada llamada crea su run.
    catch_unwind(AssertUnwindSafe(f)).map_err(|_| ())
}
