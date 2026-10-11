//! Segunda variación de longitud: (N/L)[P,−P;−P,P], P=I−ddᵀ.
//! Derivación propia en provenance/ENG-ELM-001.md; McGuire et al.: cita PENDIENTE.
use crate::{finite, geometry::Geometry};
use engine_core::{EResult, Real};
use engine_linalg::DMat;

pub(crate) fn matrix(g: &Geometry, axial: Real) -> EResult<DMat> {
    let scale = finite(axial / g.length, "N/L (N/m)")?;
    let n = 2 * g.axes;
    let mut values = vec![0.0; n * n];
    for i in 0..g.axes {
        for j in 0..=i {
            let identity = if i == j { 1.0 } else { 0.0 };
            let value = scale * (identity - g.d[i] * g.d[j]);
            for a in 0..2 {
                for b in 0..2 {
                    let v = if a == b { value } else { -value };
                    values[(a * g.axes + i) * n + b * g.axes + j] = v;
                    values[(b * g.axes + j) * n + a * g.axes + i] = v;
                }
            }
        }
    }
    DMat::new(n, n, values)
}
