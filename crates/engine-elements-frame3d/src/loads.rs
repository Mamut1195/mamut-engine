//! Trabajo ∫(qx u+qy v+qz w+mx rx−my w′+mz v′)dx; empotramiento=−f.
use crate::{finite, geometry::Geometry};
use engine_contracts::MemberLoad;
use engine_core::{EResult, Real};

pub(crate) fn components(g: &Geometry, load: &MemberLoad) -> EResult<[Real; 6]> {
    let q = g.local_triplet(load.force_per_length)?;
    let m = g.local_triplet(load.moment_per_length)?;
    Ok([q[0], q[1], q[2], m[0], m[1], m[2]])
}
pub(crate) fn sum(g: &Geometry, loads: &[MemberLoad]) -> EResult<[Real; 6]> {
    let mut q = [0.0; 6];
    for load in loads {
        for (sum, value) in q.iter_mut().zip(components(g, load)?) {
            *sum = finite(*sum + value, "Suma de cargas uniformes")?;
        }
    }
    Ok(q)
}
pub(crate) fn local_vector(l: Real, [qx, qy, qz, mx, my, mz]: [Real; 6]) -> EResult<[Real; 12]> {
    let axial = finite(qx * (l / 2.0), "qx L/2")?;
    let vy = finite(qy * (l / 2.0), "qy L/2")?;
    let vz = finite(qz * (l / 2.0), "qz L/2")?;
    let t = finite(mx * (l / 2.0), "mx L/2")?;
    let by = finite((qz * (l / 12.0)) * l, "qz L²/12")?;
    let bz = finite((qy * (l / 12.0)) * l, "qy L²/12")?;
    Ok([
        axial,
        finite(vy - mz, "fy i")?,
        finite(vz + my, "fz i")?,
        t,
        -by,
        bz,
        axial,
        finite(vy + mz, "fy j")?,
        finite(vz - my, "fz j")?,
        t,
        by,
        -bz,
    ])
}
