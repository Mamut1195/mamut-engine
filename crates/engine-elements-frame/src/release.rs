//! Condensación estática elástica: u_r=−Krr⁻¹Kra u_a+Krr⁻¹ f_r.
use crate::{
    finite,
    geometry::{multiply, Geometry},
    positive, EulerFrame2D, Matrix,
};
use engine_core::{EResult, Real};

/// Liberaciones de momento alrededor de z local en extremos i y j.
/// Dos liberaciones son válidas: Kflexión=0 y giros nodales desconectados.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MomentReleases {
    /// Libera el momento del primer extremo; su giro interno se recupera.
    pub i: bool,
    /// Libera el momento del segundo extremo; su giro interno se recupera.
    pub j: bool,
}
impl MomentReleases {
    pub(crate) fn transform(self, l: Real) -> EResult<Matrix> {
        let mut h = [[0.0; 6]; 6];
        for (i, row) in h.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        let coefficient = finite(
            if self.i && self.j { 1.0 / l } else { 1.5 / l },
            "Condensation 1/L",
        )?;
        for (released, r, other) in [(self.i, 2, 5), (self.j, 5, 2)] {
            if released {
                h[r] = [0.0; 6];
                h[r][1] = -coefficient;
                h[r][4] = coefficient;
                if !(self.i && self.j) {
                    h[r][other] = -0.5;
                }
            }
        }
        Ok(h)
    }
}
pub(crate) fn displacements(
    beam: &EulerFrame2D,
    g: &Geometry,
    u: &[Real; 6],
    f: &[Real; 6],
) -> EResult<[Real; 6]> {
    let mut internal = multiply(&beam.releases.transform(g.length)?, u)?;
    if !beam.releases.i && !beam.releases.j {
        return Ok(internal);
    }
    let b = positive(beam.ei / g.length, "EI/L")?;
    let b4 = positive(4.0 * b, "4EI/L")?;
    if beam.releases.i && beam.releases.j {
        // Inversa normalizada de [[4,2],[2,4]]: [[1/3,-1/6],[-1/6,1/3]].
        let ri = finite(f[2] / b, "f_i/(EI/L)")?;
        let rj = finite(f[5] / b, "f_j/(EI/L)")?;
        internal[2] = finite(internal[2] + ri / 3.0 - rj / 6.0, "Recovered theta_i")?;
        internal[5] = finite(internal[5] + rj / 3.0 - ri / 6.0, "Recovered theta_j")?;
    } else {
        let r = if beam.releases.i { 2 } else { 5 };
        internal[r] = finite(internal[r] + f[r] / b4, "Recovered released theta")?;
    }
    Ok(internal)
}
pub(crate) fn force(f: &[Real; 6], h: &Matrix) -> EResult<[Real; 6]> {
    let mut out = [0.0; 6];
    for (i, value) in out.iter_mut().enumerate() {
        *value = finite((0..6).map(|r| h[r][i] * f[r]).sum(), "Transformed load")?;
    }
    Ok(out)
}
pub(crate) fn project(m: &Matrix, h: &Matrix) -> EResult<Matrix> {
    let mut mh = [[0.0; 6]; 6];
    for (r, row) in mh.iter_mut().enumerate() {
        for (j, value) in row.iter_mut().enumerate() {
            *value = finite((0..6).map(|a| m[r][a] * h[a][j]).sum(), "M H")?;
        }
    }
    let mut out = [[0.0; 6]; 6];
    for i in 0..6 {
        for j in 0..=i {
            let v = finite((0..6).map(|a| h[a][i] * mh[a][j]).sum(), "Hᵀ M H")?;
            out[i][j] = v;
            out[j][i] = v;
        }
    }
    Ok(out)
}
