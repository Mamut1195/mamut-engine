//! ε=d·(u_j−u_i)/L; N=Aσ; Hessiana elástica (EA/L)bbᵀ.
use crate::{finite, geometry::Geometry, positive, vector, LinearTruss};
use engine_contracts::UniaxialLaw;
use engine_core::{EResult, Real};
use engine_linalg::DMat;

pub(crate) fn axial(
    bar: &LinearTruss,
    g: &Geometry,
    u: &[Real],
    committed: &[Real],
) -> EResult<Real> {
    vector(u, 2 * g.axes, "u_e (m)")?;
    vector(committed, 0, "Historia aceptada")?;
    let mut extension = 0.0;
    for axis in 0..g.axes {
        let relative = finite(u[g.axes + axis] - u[axis], "Desplazamiento relativo (m)")?;
        extension = finite(extension + g.d[axis] * relative, "Elongación (m)")?;
    }
    let strain = finite(extension / g.length, "ε")?;
    // Suposición: la tensión se evalúa con la ley uniaxial en la deformación ε,
    // pero solo se usa `response.stress`; la tangente de la ley no interviene.
    // La rigidez `matrix` usa EA/L fijo (barra lineal), por lo que N y K solo son
    // consistentes si la ley es elástica lineal con módulo E = ea/área. Con una
    // ley no lineal habría que usar la tangente (fuera de alcance de ENG-ELM-001).
    // `committed` es la historia aceptada y no se modifica aquí.
    let response = bar.material.respond(strain, committed)?;
    finite(bar.area * response.stress, "N (N)")
}
pub(crate) fn matrix(bar: &LinearTruss, g: &Geometry) -> EResult<DMat> {
    let k = positive(bar.ea / g.length, "EA/L (N/m)")?;
    let b = g.b();
    let n = b.len();
    let mut values = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..=i {
            let value = k * (b[i] * b[j]);
            values[i * n + j] = value;
            values[j * n + i] = value;
        }
    }
    DMat::new(n, n, values)
}
