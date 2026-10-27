//! Self-weight with explicit gravity, without imposing a vertical axis on the model.
use crate::{error, finite, length};
use engine_core::{EResult, Real};

fn properties(density: Real, area: Real, gravity: [Real; 3]) -> EResult<()> {
    finite(&[density, area])?;
    finite(&gravity)?;
    if density < 0.0 || area <= 0.0 {
        return Err(error("ENG-E-0010", "rho ≥ 0 and A > 0 are required."));
    }
    Ok(())
}

/// Total global force rho·A·L·g in N; g is supplied in m/s².
/// Requires rho≥0, A>0, L>0, and finite magnitudes; applies no code factors.
/// ENG-E-0008 for overflow of results/intermediate products.
/// For multiple members, the total force is the sum of their vectors.
pub fn self_weight(
    density: Real,
    area: Real,
    member_length: Real,
    gravity: [Real; 3],
) -> EResult<[Real; 3]> {
    properties(density, area, gravity)?;
    length(member_length)?;
    let line_mass = density * area;
    let mass = line_mass * member_length;
    finite(&[line_mass, mass])?;
    let result = gravity.map(|g| mass * g);
    finite(&result)?;
    Ok(result)
}

/// Global force per unit length rho·A·g in N/m.
/// Same property validation as [`self_weight`]; allows zero gravity
/// or gravity outside the XY plane. The 2D consumer must reject nonzero Z components.
pub fn self_weight_per_length(density: Real, area: Real, gravity: [Real; 3]) -> EResult<[Real; 3]> {
    properties(density, area, gravity)?;
    let line_mass = density * area;
    finite(&[line_mass])?;
    let result = gravity.map(|g| line_mass * g);
    finite(&result)?;
    Ok(result)
}
