//! Integral rho A ∫ HᵀH dx, con H=[(1−x/L)I, (x/L)I].
use crate::{geometry::Geometry, positive, LinearTruss};
use engine_contracts::MassKind;
use engine_core::EResult;
use engine_linalg::DMat;

pub(crate) fn matrix(bar: &LinearTruss, g: &Geometry, kind: MassKind) -> EResult<Option<DMat>> {
    if bar.density == 0.0 {
        return Ok(None);
    }
    let per_length = positive(bar.density * bar.area, "rho A (kg/m)")?;
    let total = positive(per_length * g.length, "Masa total (kg)")?;
    let n = 2 * g.axes;
    let mut values = vec![0.0; n * n];
    for axis in 0..g.axes {
        let i = axis;
        let j = g.axes + axis;
        let (diagonal, off) = match kind {
            MassKind::Lumped => (positive(total / 2.0, "Masa nodal (kg)")?, 0.0),
            MassKind::Consistent => (
                positive(total / 3.0, "Masa diagonal (kg)")?,
                positive(total / 6.0, "Masa acoplada (kg)")?,
            ),
        };
        values[i * n + i] = diagonal;
        values[j * n + j] = diagonal;
        values[i * n + j] = off;
        values[j * n + i] = off;
    }
    Ok(Some(DMat::new(n, n, values)?))
}
