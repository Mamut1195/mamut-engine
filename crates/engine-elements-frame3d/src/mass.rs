//! Linear axial/torsional and transverse Hermite mass; Guyan reduction.
use crate::{
    finite, geometry::Geometry, positive, release, stiffness::block, EulerFrame3D, Matrix, PLANES,
};
use engine_contracts::MassKind;
use engine_core::EResult;
use engine_linalg::DMat;

pub(crate) fn matrix(beam: &EulerFrame3D, g: &Geometry, kind: MassKind) -> EResult<Option<DMat>> {
    if beam.density == 0.0 {
        return Ok(None);
    }
    let total = positive(
        positive(beam.density * beam.area, "rho A")? * g.length,
        "Masa",
    )?;
    let polar = positive(
        beam.density * positive(finite(beam.iy + beam.iz, "Iy+Iz")?, "Iy+Iz")?,
        "rho Ip",
    )?;
    let polar = positive(polar * g.length, "Total polar inertia")?;
    let mut m: Matrix = [[0.0; 12]; 12];
    match kind {
        MassKind::Lumped => {
            let half = positive(total / 2.0, "Masa nodal")?;
            for i in [0, 1, 2, 6, 7, 8] {
                m[i][i] = half;
            }
            let half = positive(polar / 2.0, "Inercia torsional nodal")?;
            m[3][3] = half;
            m[9][9] = half;
        }
        MassKind::Consistent => {
            for (ids, total) in [([0, 6], total), ([3, 9], polar)] {
                let third = positive(total / 3.0, "Diagonal linear mass")?;
                let sixth = positive(total / 6.0, "Coupled linear mass")?;
                m[ids[0]][ids[0]] = third;
                m[ids[1]][ids[1]] = third;
                m[ids[0]][ids[1]] = sixth;
                m[ids[1]][ids[0]] = sixth;
            }
            let a = positive(total / 420.0, "Hermite mass scale")?;
            let b = positive(a * g.length, "Coupled rotational mass")?;
            let c = positive(b * g.length, "Inercia nodal")?;
            let bending = [
                [156.0 * a, 22.0 * b, 54.0 * a, -13.0 * b],
                [22.0 * b, 4.0 * c, 13.0 * b, -3.0 * c],
                [54.0 * a, 13.0 * b, 156.0 * a, -22.0 * b],
                [-13.0 * b, -3.0 * c, -22.0 * b, 4.0 * c],
            ];
            for (ids, sign) in PLANES {
                block(&mut m, ids, sign, bending)?;
            }
        }
    }
    Ok(Some(g.global_matrix(&release::project(
        &m,
        &beam.releases.transform(g.length)?,
    )?)?))
}
