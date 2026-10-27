//! rho A ∫ HᵀH dx: linear axial, cubic Hermite transverse; without rho I.
use crate::{geometry::Geometry, positive, release, EulerFrame2D, Matrix};
use engine_contracts::MassKind;
use engine_core::EResult;
use engine_linalg::DMat;

pub(crate) fn matrix(beam: &EulerFrame2D, g: &Geometry, kind: MassKind) -> EResult<Option<DMat>> {
    if beam.density == 0.0 {
        return Ok(None);
    }
    let total = positive(
        positive(beam.density * beam.area, "rho A")? * g.length,
        "Mass (kg)",
    )?;
    let mut m: Matrix = [[0.0; 6]; 6];
    match kind {
        MassKind::Lumped => {
            let half = positive(total / 2.0, "Masa nodal (kg)")?;
            for i in [0, 1, 3, 4] {
                m[i][i] = half;
            }
        }
        MassKind::Consistent => {
            let third = positive(total / 3.0, "Axial mass")?;
            let sixth = positive(total / 6.0, "Coupled axial mass")?;
            m[0][0] = third;
            m[3][3] = third;
            m[0][3] = sixth;
            m[3][0] = sixth;
            let a = positive(total / 420.0, "Hermite mass scale")?;
            let b = positive(a * g.length, "Coupled rotational mass")?;
            let c = positive(b * g.length, "Inercia nodal")?;
            let bending = [
                [156.0 * a, 22.0 * b, 54.0 * a, -13.0 * b],
                [22.0 * b, 4.0 * c, 13.0 * b, -3.0 * c],
                [54.0 * a, 13.0 * b, 156.0 * a, -22.0 * b],
                [-13.0 * b, -3.0 * c, -22.0 * b, 4.0 * c],
            ];
            let ids = [1, 2, 4, 5];
            for (i, &r) in ids.iter().enumerate() {
                for (j, &s) in ids.iter().enumerate() {
                    m[r][s] = bending[i][j];
                }
            }
        }
    }
    Ok(Some(g.global_matrix(&release::project(
        &m,
        &beam.releases.transform(g.length)?,
    )?)?))
}
