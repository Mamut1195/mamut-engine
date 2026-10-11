//! Equilibrio axial dN/dx=−q; N(x)=N_elástico+qL(1/2−x/L).
use crate::{finite, geometry::Geometry, invalid, loads, stiffness, LinearTruss};
use engine_contracts::{ElementOutput, MemberLoad, StationResult};
use engine_core::{EResult, Real};

pub(crate) fn output(
    bar: &LinearTruss,
    g: &Geometry,
    u: &[Real],
    committed: &[Real],
    member_loads: &[MemberLoad],
    stations: &[Real],
) -> EResult<ElementOutput> {
    let n = stiffness::axial(bar, g, u, committed)?;
    let mut q = 0.0;
    for load in member_loads {
        q = finite(q + loads::axial(g, load)?, "Suma q axial (N/m)")?;
    }
    let half = finite(q * (g.length / 2.0), "qL/2 (N)")?;
    let mut end_forces_local = vec![0.0; 2 * g.axes];
    end_forces_local[0] = finite(-n - half, "Acción i (N)")?;
    end_forces_local[g.axes] = finite(n - half, "Acción j (N)")?;
    let mut out = Vec::new();
    out.try_reserve_exact(stations.len())
        .map_err(|_| invalid("No se pudo reservar memoria para las estaciones de resultados."))?;
    for &position in stations {
        finite(position, "Estación x/L")?;
        if !(0.0..=1.0).contains(&position) {
            return Err(invalid("Estación fuera de [0,1]."));
        }
        // Esta forma evita desbordar qL cuando ambas acciones de extremo caben.
        let axial = finite(n + half * (1.0 - 2.0 * position), "N de estación (N)")?;
        out.push(StationResult {
            position,
            forces_local: [axial, 0.0, 0.0, 0.0, 0.0, 0.0],
        });
    }
    let stations = out;
    Ok(ElementOutput {
        end_forces_local,
        stations,
    })
}
