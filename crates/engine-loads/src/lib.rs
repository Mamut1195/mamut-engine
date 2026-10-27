#![forbid(unsafe_code)]
//! Load cases and pure functions in SI, with no implicit code factors.
//!
//! Model nodal loads and assembler prescribed displacements
//! are reused. Span loads act on a 2D Euler–Bernoulli beam
//! without releases: XY plane, local x from i to j, local y=(-sin,cos), z=+Z.
//! [`equivalent_nodal_forces_local`] follows the applied load sign;
//! [`fixed_end_forces_local`] returns its opposite, the reactions of the
//! restrained ends on the member at zero displacement.
//! Both use the order `[N_i,V_i,M_i,N_j,V_j,M_j]`.
//!
//! Functions validate inputs and results: ENG-E-0008 for nonfinite values or
//! overflow, ENG-E-0010 for ranges or repeated components in a case,
//! ENG-E-0011 for missing DOFs, ENG-E-0003 for broken references, and ENG-E-0013
//! for conversions that cannot be represented through the uniform contract.
//! Public lists are revalidated on every operation.
//!
//! ```
//! use engine_loads::{Beam2D, LoadAxes, SpanLoad, equivalent_nodal_forces_local};
//! let beam = Beam2D::new(4.0, 0.0)?;
//! let load = SpanLoad::Uniform { q: [0.0, 6.0], axes: LoadAxes::Local };
//! let f = equivalent_nodal_forces_local(&beam, &load)?;
//! assert!((f[1] - 12.0).abs() < 1e-12);
//! assert!((f[2] - 8.0).abs() < 1e-12);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

pub mod mass;
mod nodal;
mod span;
mod weight;

pub use engine_assembly::PrescribedDof as ImposedDisplacement;
pub use engine_model::{DofValue, LoadCase, NodalLoad};
pub use nodal::{assemble_imposed_displacements, assemble_nodal_loads, validate_case};
pub use span::{
    equivalent_nodal_forces_local, fixed_end_forces_local, to_member_load, Beam2D, LoadAxes,
    SpanLoad,
};
pub use weight::{self_weight, self_weight_per_length};

use engine_core::{catalog, EResult, EngineError, Real};

fn error(code: &'static str, context: &str) -> EngineError {
    // All codes used belong to the catalog; no unwrap/expect.
    let message = match catalog::lookup(code) {
        Some(info) => format!("{} {context}", info.message),
        None => context.to_owned(),
    };
    EngineError::new(code, message)
}

fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(error("ENG-E-0008", "Non-finite load input or result."));
    }
    Ok(())
}

fn length(value: Real) -> EResult<()> {
    finite(&[value])?;
    if value <= 0.0 {
        return Err(error("ENG-E-0010", "Length must be positive."));
    }
    Ok(())
}
