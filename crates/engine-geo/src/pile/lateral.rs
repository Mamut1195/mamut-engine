//! Semi-infinite lateral beam on a homogeneous, bilateral linear foundation.
//!
//! Depth is positive downward. The displacement and head force use the same
//! lateral direction. The signed actions are defined by `M = -EI*y''`,
//! `V = EI*y'''` and `p = -k*y`; they are not frame-element conventions.
//! See provenance/ENG-GEO-005.md for the derivation and limitations.
use engine_core::math;
use engine_core::{EResult, EngineError, Real};

/// Rotational condition at the head.
#[derive(Clone, Copy, Debug)]
pub enum Head {
    /// Prescribed head moment, including zero for a moment-free head.
    Free {
        /// Signed head moment in N m, using `M = -EI*y''`.
        moment_nm: Real,
    },
    /// Zero head slope; the moment is a reaction.
    Fixed,
}

/// Displacement and signed actions at a depth.
#[derive(Clone, Copy, Debug)]
pub struct Response {
    /// Lateral displacement in m.
    pub y_m: Real,
    /// Bending moment in N m.
    pub moment_nm: Real,
    /// Shear in N.
    pub shear_n: Real,
    /// Distributed soil force on the beam in N/m.
    pub soil_force_n_per_m: Real,
}

/// Evaluates the decaying semi-infinite solution at depth `z` in m.
///
/// `ei` is bending rigidity in N m^2, `k` is foundation stiffness per
/// beam length in N/m^2, and `h` is the signed head shear in N. `ei` and
/// `k` must be positive and `z` nonnegative. Loads may have either sign.
///
/// Returns ENG-E-0008 for nonfinite inputs or intermediate results and
/// ENG-E-0010 for invalid ranges or underflowed stiffness scales. Extreme
/// finite inputs may be rejected, and ordinary rounding and underflow of
/// response terms are possible. This is not a finite or nonlinear pile solver.
pub fn evaluate(ei: Real, k: Real, h: Real, head: Head, z: Real) -> EResult<Response> {
    for value in [ei, k, h, z] {
        finite(value)?;
    }
    if let Head::Free { moment_nm } = head {
        finite(moment_nm)?;
    }
    if ei <= 0.0 || k <= 0.0 || z < 0.0 {
        return Err(invalid());
    }

    let ratio = finite(k / ei)?;
    if ratio == 0.0 {
        return Err(invalid());
    }
    let beta = math::sqrt(math::sqrt(ratio)) / math::sqrt(2.0);
    let beta_squared = finite(beta * beta)?;
    let moment_scale = finite(finite(2.0 * ei)? * beta_squared)?;
    let shear_scale = finite(moment_scale * beta)?;
    if beta <= 0.0 || beta_squared <= 0.0 || moment_scale <= 0.0 || shear_scale <= 0.0 {
        return Err(invalid());
    }

    let (a, b) = match head {
        Head::Free { moment_nm } => {
            let b = finite(moment_nm / moment_scale)?;
            (finite(finite(h / shear_scale)? - b)?, b)
        }
        Head::Fixed => {
            let a = finite(finite(h / shear_scale)? * 0.5)?;
            (a, a)
        }
    };
    let x = finite(beta * z)?;
    let decay = finite(math::exp(-x))?;
    let (sin, cos) = math::sin_cos(x);
    finite(sin)?;
    finite(cos)?;

    let y_shape = finite(finite(a * cos)? + finite(b * sin)?)?;
    let m_shape = finite(finite(b * cos)? - finite(a * sin)?)?;
    let v_shape = finite(finite(finite(a + b)? * cos)? + finite(finite(b - a)? * sin)?)?;
    let y = finite(decay * y_shape)?;
    let moment = finite(moment_scale * finite(decay * m_shape)?)?;
    let shear = finite(shear_scale * finite(decay * v_shape)?)?;
    Ok(Response {
        y_m: y,
        moment_nm: moment,
        shear_n: shear,
        soil_force_n_per_m: finite(-k * y)?,
    })
}

fn finite(value: Real) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            "Pile lateral: nonfinite input or intermediate result.",
        ))
    }
}

fn invalid() -> EngineError {
    EngineError::new(
        "ENG-E-0010",
        "Pile lateral: invalid range or underflowed stiffness scale.",
    )
}
