//! Explicit placeholder for the faer adapter: includes no library or operations.
use engine_core::{EResult, EngineError};

/// Adapter PENDING clarification of faer 0.24.4 license notices.
///
/// The temporary test with an optional dependency, `default-features=false`, and
/// `features=["std","sparse-linalg"]` compiled with Rust 1.85.1 on native and
/// wasm; both backends passed the same native conformance. However, the
/// package includes `COPYING.EIGEN.MPL2` despite declaring MIT. Without network access
/// the scope of that notice could not be verified through official documentation.
/// That dependency is not included in the delivery: see the crate README.md and
/// `provenance/ENG-LA-007.md`. This stub does not implement [`crate::LinAlgBackend`]
/// or silently substitute the reference backend for faer.
#[derive(Debug)]
pub struct FaerBackend {
    _private: (),
}
impl FaerBackend {
    /// Always returns ENG-E-0009 with the reason for pending integration.
    /// The feature reserves the name without enabling a functional backend.
    pub fn new() -> EResult<Self> {
        Err(EngineError::new(
            "ENG-E-0009",
            "faer backend pending: verify the scope of COPYING.EIGEN.MPL2 in faer 0.24.4 before adding a dependency with exclusively permissive licensing.",
        ))
    }
}
