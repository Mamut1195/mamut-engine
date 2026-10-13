//! EA u′² + GJ rx′² + EIz v″² + EIy w″²; Schur analítico por plano.
use crate::{finite, geometry::Geometry, positive, EulerFrame3D, Matrix, PLANES};
use engine_contracts::UniaxialLaw;
use engine_core::{EResult, Real};

pub(crate) fn axial(beam: &EulerFrame3D, g: &Geometry, u: &[Real; 12]) -> EResult<Real> {
    let strain = finite(finite(u[6] - u[0], "Elongación")? / g.length, "ε")?;
    let response = beam.material.respond(strain, &[])?;
    let n = finite(beam.area * response.stress, "N (N)")?;
    // E·ε puede subdesbordar (subnormal/0) con E diminuto y A enorme aunque A·E·ε
    // sea representable (J4 r6): en ese caso se usa (A·E_t)·ε, exacto a primer
    // orden para deformaciones de ese orden.
    if strain != 0.0 && !response.stress.is_normal() && response.tangent > 0.0 {
        let alt = finite((beam.area * response.tangent) * strain, "N (N)")?;
        if alt.is_normal() {
            return Ok(alt);
        }
    }
    Ok(n)
}
pub(crate) fn pair(m: &mut Matrix, ids: [usize; 2], a: Real) {
    let [i, j] = ids;
    m[i][i] = a;
    m[j][j] = a;
    m[i][j] = -a;
    m[j][i] = -a;
}
pub(crate) fn block(m: &mut Matrix, ids: [usize; 4], sign: Real, b: [[Real; 4]; 4]) -> EResult<()> {
    let signs = [1.0, sign, 1.0, sign];
    for (i, &r) in ids.iter().enumerate() {
        for (j, &s) in ids.iter().enumerate() {
            m[r][s] = finite(signs[i] * signs[j] * b[i][j], "Bloque de matriz")?;
        }
    }
    Ok(())
}
pub(crate) fn condensed(beam: &EulerFrame3D, g: &Geometry) -> EResult<Matrix> {
    let l = g.length;
    let mut k = [[0.0; 12]; 12];
    pair(&mut k, [0, 6], positive(beam.ea / l, "EA/L")?);
    if !beam.releases.i[0] && !beam.releases.j[0] {
        pair(&mut k, [3, 9], positive(beam.gj / l, "GJ/L")?);
    }
    for ((ids, sign), ei) in PLANES.into_iter().zip([beam.eiz, beam.eiy]) {
        let axis = ids[1] - 3;
        let ri = beam.releases.i[axis];
        let rj = beam.releases.j[axis];
        if ri && rj {
            continue;
        }
        let b = positive(ei / l, "EI/L")?;
        if ri || rj {
            let c = positive(3.0 * b / l, "3EI/L²")?;
            let d = positive(c / l, "3EI/L³")?;
            let theta = if ri { ids[3] } else { ids[1] };
            pair(&mut k, [ids[0], ids[2]], d);
            k[ids[0]][theta] = sign * c;
            k[theta][ids[0]] = sign * c;
            k[ids[2]][theta] = -sign * c;
            k[theta][ids[2]] = -sign * c;
            k[theta][theta] = positive(3.0 * b, "3EI/L")?;
        } else {
            let c = positive(6.0 * b / l, "6EI/L²")?;
            let d = positive(2.0 * c / l, "12EI/L³")?;
            let b4 = positive(4.0 * b, "4EI/L")?;
            let b2 = positive(2.0 * b, "2EI/L")?;
            block(
                &mut k,
                ids,
                sign,
                [
                    [d, c, -d, c],
                    [c, b4, -c, b2],
                    [-d, -c, d, -c],
                    [c, b2, -c, b4],
                ],
            )?;
        }
    }
    Ok(k)
}
