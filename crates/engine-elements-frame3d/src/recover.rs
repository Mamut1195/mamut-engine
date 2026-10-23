//! Equilibrio por cara positiva: F′=−q, M′=−x×V−m.
use crate::{
    finite,
    geometry::{multiply, Geometry},
    invalid, loads, release, stiffness, EulerFrame3D,
};
use engine_contracts::{ElementOutput, MemberLoad, StationResult};
use engine_core::{EResult, Real};

pub(crate) fn output(
    beam: &EulerFrame3D,
    g: &Geometry,
    u: &[Real],
    member_loads: &[MemberLoad],
    stations: &[Real],
) -> EResult<ElementOutput> {
    let u = g.local_vector(u)?;
    let q = loads::sum(g, member_loads)?;
    let f = release::load(beam, g, &loads::local_vector(g.length, q)?)?;
    // Kc u−fc es algebraicamente igual a K u_rec−f en GDL conservados.
    // Evita restar fuerzas enormes de giros internos; filas liberadas son cero.
    let elastic = multiply(&stiffness::condensed(beam, g)?, &u)?;
    let mut ends = [0.0; 12];
    for i in 0..12 {
        ends[i] = finite(elastic[i] - f[i], "End action")?;
    }
    let mut out = Vec::new();
    out.try_reserve_exact(stations.len())
        .map_err(|_| invalid("Could not allocate stations."))?;
    for &position in stations {
        finite(position, "Station x/L")?;
        if !(0.0..=1.0).contains(&position) {
            return Err(invalid("Station is outside [0,1]."));
        }
        let x = position * g.length;
        let n = finite(-ends[0] - q[0] * x, "Station N")?;
        let vy = finite(-ends[1] - q[1] * x, "Station Vy")?;
        let vz = finite(-ends[2] - q[2] * x, "Station Vz")?;
        let t = finite(-ends[3] - q[3] * x, "Station T")?;
        let my = finite(
            -ends[4] - ends[2] * x - (q[2] * (x / 2.0)) * x - q[4] * x,
            "Station My",
        )?;
        let mz = finite(
            -ends[5] + ends[1] * x + (q[1] * (x / 2.0)) * x - q[5] * x,
            "Station Mz",
        )?;
        out.push(StationResult {
            position,
            forces_local: [n, vy, vz, t, my, mz],
        });
    }
    Ok(ElementOutput {
        end_forces_local: ends.to_vec(),
        stations: out,
    })
}
