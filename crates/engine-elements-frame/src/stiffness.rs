//! ∫ EA B_uᵀB_u + EI B_vᵀB_v dx; Schur de los giros liberados.
use crate::{finite, geometry::Geometry, positive, EulerFrame2D, Matrix};
use engine_contracts::UniaxialLaw;
use engine_core::{EResult, Real};

pub(crate) fn axial(beam: &EulerFrame2D, g: &Geometry, u: &[Real; 6]) -> EResult<Real> {
    let strain = finite(finite(u[3] - u[0], "Elongación (m)")? / g.length, "ε")?;
    finite(
        beam.area * beam.material.respond(strain, &[])?.stress,
        "N (N)",
    )
}
pub(crate) fn local_matrix(beam: &EulerFrame2D, g: &Geometry) -> EResult<Matrix> {
    let l = g.length;
    let a = positive(beam.ea / l, "EA/L")?;
    let b = positive(beam.ei / l, "EI/L")?;
    let c = positive(6.0 * b / l, "6EI/L²")?;
    let d = positive(2.0 * c / l, "12EI/L³")?;
    let b4 = positive(4.0 * b, "4EI/L")?;
    let b2 = positive(2.0 * b, "2EI/L")?;
    Ok([
        [a, 0.0, 0.0, -a, 0.0, 0.0],
        [0.0, d, c, 0.0, -d, c],
        [0.0, c, b4, 0.0, -c, b2],
        [-a, 0.0, 0.0, a, 0.0, 0.0],
        [0.0, -d, -c, 0.0, d, -c],
        [0.0, c, b2, 0.0, -c, b4],
    ])
}
pub(crate) fn condensed(beam: &EulerFrame2D, g: &Geometry) -> EResult<Matrix> {
    if !beam.releases.i && !beam.releases.j {
        return local_matrix(beam, g);
    }
    // Schur analítico: dos liberaciones anulan flexión exactamente, sin
    // residuos de cancelación; una deja 3EI/L³ (v_j-v_i-L theta_f)²/2.
    let a = positive(beam.ea / g.length, "EA/L")?;
    let mut k = [[0.0; 6]; 6];
    k[0][0] = a;
    k[0][3] = -a;
    k[3][0] = -a;
    k[3][3] = a;
    if beam.releases.i && beam.releases.j {
        return Ok(k);
    }
    let b = positive(beam.ei / g.length, "EI/L")?;
    let c = positive(3.0 * b / g.length, "3EI/L²")?;
    let d = positive(c / g.length, "3EI/L³")?;
    let theta = if beam.releases.i { 5 } else { 2 };
    k[1][1] = d;
    k[1][4] = -d;
    k[4][1] = -d;
    k[4][4] = d;
    k[1][theta] = c;
    k[theta][1] = c;
    k[4][theta] = -c;
    k[theta][4] = -c;
    k[theta][theta] = positive(3.0 * b, "3EI/L")?;
    Ok(k)
}
