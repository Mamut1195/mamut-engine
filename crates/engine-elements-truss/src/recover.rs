//! Axial equilibrium dN/dx=−q; N(x)=N_elástico+qL(1/2−x/L).
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
        q = finite(q + loads::axial(g, load)?, "Sum of axial q (N/m)")?;
    }
    let half = finite(q * (g.length / 2.0), "qL/2 (N)")?;
    let mut end_forces_local = vec![0.0; 2 * g.axes];
    end_forces_local[0] = finite(-n - half, "Action i (N)")?;
    end_forces_local[g.axes] = finite(n - half, "Action j (N)")?;
    let mut out = Vec::new();
    out.try_reserve_exact(stations.len())
        .map_err(|_| invalid("Could not allocate memory for result stations."))?;
    for &position in stations {
        finite(position, "Station x/L")?;
        if !(0.0..=1.0).contains(&position) {
            return Err(invalid("Station is outside [0,1]."));
        }
        // This form avoids overflowing qL when both end actions fit.
        let axial = finite(n + half * (1.0 - 2.0 * position), "Station N (N)")?;
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

/// Fixed-end uniform temperature: constant N, no distributed q, no gradient.
///
/// End actions use the same local slots as mechanical recovery:
/// index 0 is -N and index `axes` is +N.
pub(crate) fn fixed_end_uniform_temperature(
    g: &Geometry,
    axial: Real,
    stations: &[Real],
) -> EResult<ElementOutput> {
    let mut end_forces_local = vec![0.0; 2 * g.axes];
    end_forces_local[0] = finite(-axial, "Action i (N)")?;
    end_forces_local[g.axes] = finite(axial, "Action j (N)")?;
    let mut out = Vec::new();
    out.try_reserve_exact(stations.len())
        .map_err(|_| invalid("Could not allocate memory for result stations."))?;
    for &position in stations {
        finite(position, "Station x/L")?;
        if !(0.0..=1.0).contains(&position) {
            return Err(invalid("Station is outside [0,1]."));
        }
        out.push(StationResult {
            position,
            forces_local: [axial, 0.0, 0.0, 0.0, 0.0, 0.0],
        });
    }
    Ok(ElementOutput {
        end_forces_local,
        stations: out,
    })
}
