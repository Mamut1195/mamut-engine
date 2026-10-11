//! Contratos v1 del motor y plantillas independientes del backend numérico.
//!
//! Entradas y salidas en SI: m, N, Pa, N·m, kg y rad; deformación axial
//! adimensional y curvatura en m⁻¹. Ejes diestros, Z vertical; D2 usa X-Y,
//! Z = 0, con Ux, Uy, Rz. Orden de vectores: nodo y después GDL declarado.
//! Las fuerzas de extremo son acciones sobre las caras del miembro: i negativa,
//! j positiva; axial positivo en tracción. La elevación D2 sigue pendiente.
//! Historia plana, finita y sin punteros; responder es puro, aceptar es explícito.
//! Los tipos matriciales **se reemplazan por el backend de ENG-LA-007**;
//! aquí solo se almacena información, sin factorización ni solver.
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
mod matrix;
mod traits;
mod types;
pub use matrix::*;
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
            "Magnitud no finita en el contrato.",
        ))
    }
}
pub(crate) fn size(actual: usize, expected: usize) -> EResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(EngineError::new(
            "ENG-E-0010",
            format!("Longitud {actual}; se esperaba {expected}."),
        ))
    }
}
pub(crate) fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
#[cfg(test)]
mod tests;
