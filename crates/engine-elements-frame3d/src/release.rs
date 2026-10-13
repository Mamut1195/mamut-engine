//! u_r=−Krr⁻¹Kra u_a+Krr⁻¹fr; formas cerradas y proyección de energía.
use crate::{
    finite,
    geometry::{multiply, Geometry},
    positive, EulerFrame3D, Matrix, PLANES,
};
use engine_core::{EResult, EngineError, Real};

/// Liberaciones de torsión y momentos alrededor de x,y,z locales en i y j.
/// No libera traslaciones. Los giros nodales liberados quedan desconectados.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MomentReleases {
    /// Extremo i: `[T,My,Mz]`, true libera la componente.
    pub i: [bool; 3],
    /// Extremo j: `[T,My,Mz]`, true libera la componente.
    pub j: [bool; 3],
}
impl MomentReleases {
    pub(crate) fn transform(self, l: Real) -> EResult<Matrix> {
        let mut h = [[0.0; 12]; 12];
        for (i, row) in h.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        for ((ids, sign), axis) in PLANES.into_iter().zip([2, 1]) {
            let both = self.i[axis] && self.j[axis];
            let coefficient = finite(if both { 1.0 / l } else { 1.5 / l }, "Condensación 1/L")?;
            for (released, r, other) in [
                (self.i[axis], ids[1], ids[3]),
                (self.j[axis], ids[3], ids[1]),
            ] {
                if released {
                    h[r] = [0.0; 12];
                    h[r][ids[0]] = -sign * coefficient;
                    h[r][ids[2]] = sign * coefficient;
                    if !both {
                        h[r][other] = -0.5;
                    }
                }
            }
        }
        for (released, r, other) in [(self.i[0], 3, 9), (self.j[0], 9, 3)] {
            if released {
                h[r] = [0.0; 12];
                if !(self.i[0] && self.j[0]) {
                    h[r][other] = 1.0;
                }
            }
        }
        Ok(h)
    }
}
fn torsion_equilibrium(beam: &EulerFrame3D, length: Real, f: &[Real; 12]) -> EResult<()> {
    let net = finite(f[3] + f[9], "Torsor neto")?.abs();
    // La escala sólo se usa con torsión liberada en ambos extremos; su guarda
    // no debe rechazar cargas finitas en elementos sin esa doble liberación.
    if !(beam.releases.i[0] && beam.releases.j[0]) {
        return Ok(());
    }
    // Tolerancia relativa a la escala de la carga: el redondeo de la proyección
    // local deja mx≈1e-16·|q| en lugar de 0 (J4 r6: falso ENG-E-0006).
    // Escala en N·m: momentos de f (N·m) y fuerzas de f (N) por la longitud (N·m);
    // no se suman magnitudes de unidades distintas. El redondeo del momento local
    // proviene de la proyección de cargas cuyos efectos aparecen como cortantes.
    let moments = [3, 4, 5, 9, 10, 11]
        .iter()
        .fold(0.0_f64, |a, &i| a.max(f[i].abs()));
    let forces = [0, 1, 2, 6, 7, 8]
        .iter()
        .fold(0.0_f64, |a, &i| a.max(f[i].abs()));
    let force_scale = finite(length * forces, "Escala L·fuerzas")?;
    let scale = finite(moments.max(force_scale), "Escala torsional")?;
    if net > 1e-12 * scale {
        return Err(EngineError::new(
            "ENG-E-0006",
            "Torsión liberada en ambos extremos: carga torsional neta sin equilibrio.",
        ));
    }
    Ok(())
}
pub(crate) fn load(beam: &EulerFrame3D, g: &Geometry, f: &[Real; 12]) -> EResult<[Real; 12]> {
    torsion_equilibrium(beam, g.length, f)?;
    force(f, &beam.releases.transform(g.length)?)
}
pub(crate) fn displacements(
    beam: &EulerFrame3D,
    g: &Geometry,
    u: &[Real; 12],
    f: &[Real; 12],
) -> EResult<[Real; 12]> {
    torsion_equilibrium(beam, g.length, f)?;
    let mut d = multiply(&beam.releases.transform(g.length)?, u)?;
    for ((ids, _), ei) in PLANES.into_iter().zip([beam.eiz, beam.eiy]) {
        let axis = ids[1] - 3;
        let ri = beam.releases.i[axis];
        let rj = beam.releases.j[axis];
        if !ri && !rj {
            continue;
        }
        let b = positive(ei / g.length, "EI/L")?;
        if ri && rj {
            let fi = finite(f[ids[1]] / b, "fi/(EI/L)")?;
            let fj = finite(f[ids[3]] / b, "fj/(EI/L)")?;
            d[ids[1]] = finite(d[ids[1]] + fi / 3.0 - fj / 6.0, "Giro i recuperado")?;
            d[ids[3]] = finite(d[ids[3]] + fj / 3.0 - fi / 6.0, "Giro j recuperado")?;
        } else {
            let r = if ri { ids[1] } else { ids[3] };
            d[r] = finite(
                d[r] + f[r] / positive(4.0 * b, "4EI/L")?,
                "Giro liberado recuperado",
            )?;
        }
    }
    let ri = beam.releases.i[0];
    let rj = beam.releases.j[0];
    if ri || rj {
        let a = positive(beam.gj / g.length, "GJ/L")?;
        if ri && rj {
            // Pseudoinversa en el subespacio de giro medio nulo.
            d[3] = finite((f[3] / a) / 2.0, "Giro torsional i")?;
            d[9] = -d[3];
        } else {
            let r = if ri { 3 } else { 9 };
            d[r] = finite(d[r] + f[r] / a, "Giro torsional liberado")?;
        }
    }
    Ok(d)
}
pub(crate) fn force(f: &[Real; 12], h: &Matrix) -> EResult<[Real; 12]> {
    let mut out = [0.0; 12];
    for (i, v) in out.iter_mut().enumerate() {
        *v = finite((0..12).map(|r| h[r][i] * f[r]).sum(), "Carga transformada")?;
    }
    Ok(out)
}
pub(crate) fn project(m: &Matrix, h: &Matrix) -> EResult<Matrix> {
    let mut mh = [[0.0; 12]; 12];
    for (r, row) in mh.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = finite((0..12).map(|a| m[r][a] * h[a][j]).sum(), "M H")?;
        }
    }
    let mut out = [[0.0; 12]; 12];
    for i in 0..12 {
        for j in 0..=i {
            let v = finite((0..12).map(|a| h[a][i] * mh[a][j]).sum(), "Hᵀ M H")?;
            out[i][j] = v;
            out[j][i] = v;
        }
    }
    Ok(out)
}
