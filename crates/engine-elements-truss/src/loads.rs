//! Fuerzas uniformes axiales globales; rechazo explícito de momentos/transversales.
use crate::{finite, geometry::Geometry, invalid, vector as check_vector};
use engine_contracts::MemberLoad;
use engine_core::{EResult, Real};

pub(crate) fn axial(g: &Geometry, load: &MemberLoad) -> EResult<Real> {
    check_vector(&load.force_per_length, 3, "q (N/m)")?;
    check_vector(&load.moment_per_length, 3, "Momento por longitud (N)")?;
    if load.moment_per_length != [0.0; 3] || (g.axes == 2 && load.force_per_length[2] != 0.0) {
        return Err(invalid(
            "Barra: momentos o carga fuera del plano no admitidos.",
        ));
    }
    let mut q = 0.0;
    for axis in 0..g.axes {
        q = finite(q + g.d[axis] * load.force_per_length[axis], "q axial (N/m)")?;
    }
    // Tolerancia relativa de proyección: solo redondeo, sin umbral absoluto
    // que permita omitir una carga transversal pequeña.
    let magnitude = load
        .force_per_length
        .iter()
        .fold(0.0_f64, |m, v| m.max(v.abs()));
    for axis in 0..g.axes {
        let residual = finite(
            load.force_per_length[axis] - q * g.d[axis],
            "q transversal (N/m)",
        )?;
        if residual.abs() > 64.0 * f64::EPSILON * magnitude {
            return Err(invalid(
                "Barra: solo carga uniforme paralela al eje, sin componente transversal.",
            ));
        }
    }
    Ok(q)
}
pub(crate) fn vector(g: &Geometry, load: &MemberLoad) -> EResult<Vec<Real>> {
    let half = finite(axial(g, load)? * (g.length / 2.0), "Carga equivalente (N)")?;
    (0..2 * g.axes)
        .map(|i| finite(half * g.d[i % g.axes], "F equivalente (N)"))
        .collect()
}
