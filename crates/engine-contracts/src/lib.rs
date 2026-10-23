#![forbid(unsafe_code)]
//! Engine v1 contracts and templates independent of the numerical backend.
//!
//! SI inputs and outputs: m, N, Pa, N·m, kg, and rad; dimensionless axial
//! strain and curvature in m⁻¹. Right-handed axes, Z vertical; D2 uses X-Y,
//! Z = 0, with Ux, Uy, Rz. Vector order: node, then declared DOF.
//! End forces are actions on member faces: i negative,
//! j positive; axial force positive in tension. D2 elevation is still pending.
//! Flat, finite, pointer-free history; response is pure, acceptance is explicit.
//! The definitive matrix types are re-exported from `engine-linalg`;
//! the contracts retain an API independent of the numerical library.
//!
//! ```
//! use engine_contracts::{examples::LinearElastic, conformance::check_uniaxial_law};
//! let ley = LinearElastic::new(200e9)?;
//! assert!(check_uniaxial_law(&ley, &[-0.001, 0.0, 0.001], 1e-7).is_empty());
//! # Ok::<(), engine_contracts::EngineError>(())
//! ```
#![deny(missing_docs)]

pub use engine_core::{
    ContractId, Diagnostic, Dim, Dof, DofIndex, EResult, EngineError, NodeId, Real,
};
mod traits;
mod types;
pub use engine_linalg::{DMat, SparsityPattern, SymMatrix};
pub use traits::*;
pub use types::*;
pub mod conformance;
pub mod examples;

pub(crate) fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            "Non-finite quantity in contract.",
        ))
    }
}
pub(crate) fn size(actual: usize, expected: usize) -> EResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(EngineError::new(
            "ENG-E-0010",
            format!("Length {actual}; expected {expected}."),
        ))
    }
}
pub(crate) fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
#[cfg(test)]
mod tests;
