//! ∫ Hᵀq + H′ᵀmz dx; f_eq local, empotramiento perfecto = −f_eq.
use crate::{finite, geometry::Geometry, unsupported, vector};
use engine_contracts::MemberLoad;
use engine_core::{EResult, Real};

pub(crate) fn components(g: &Geometry, load: &MemberLoad) -> EResult<[Real; 3]> {
    vector(&load.force_per_length, 3, "q global (N/m)")?;
    vector(&load.moment_per_length, 3, "m global (N)")?;
    if load.force_per_length[2] != 0.0
        || load.moment_per_length[0] != 0.0
        || load.moment_per_length[1] != 0.0
    {
        return Err(unsupported(
            "D2 beam: out-of-plane loads are not supported.",
        ));
    }
    let [qx, qy, _] = load.force_per_length;
    Ok([
        finite(g.c * qx + g.s * qy, "qx local")?,
        finite(-g.s * qx + g.c * qy, "qy local")?,
        load.moment_per_length[2],
    ])
}
pub(crate) fn sum(g: &Geometry, loads: &[MemberLoad]) -> EResult<[Real; 3]> {
    let mut q = [0.0; 3];
    for load in loads {
        for (sum, value) in q.iter_mut().zip(components(g, load)?) {
            *sum = finite(*sum + value, "Sum of uniform loads")?;
        }
    }
    Ok(q)
}
pub(crate) fn local_vector(l: Real, [qx, qy, mz]: [Real; 3]) -> EResult<[Real; 6]> {
    let axial = finite(qx * (l / 2.0), "qx L/2")?;
    let shear = finite(qy * (l / 2.0), "qy L/2")?;
    let moment = finite((qy * (l / 12.0)) * l, "qy L²/12")?;
    Ok([
        axial,
        finite(shear - mz, "f_y i")?,
        moment,
        axial,
        finite(shear + mz, "f_y j")?,
        -moment,
    ])
}
