//! Positive-face equilibrium: N′=−qx, V′=−qy, M′=−V−mz.
use crate::{
    finite,
    geometry::{multiply, Geometry},
    invalid, loads, release, stiffness, EulerFrame2D,
};
use engine_contracts::{ElementOutput, MemberLoad, StationResult};
use engine_core::{EResult, Real};

pub(crate) fn output(
    beam: &EulerFrame2D,
    g: &Geometry,
    u: &[Real],
    member_loads: &[MemberLoad],
    stations: &[Real],
) -> EResult<ElementOutput> {
    let u = g.local_vector(u)?;
    let q = loads::sum(g, member_loads)?;
    let f = loads::local_vector(g.length, q)?;
    let recovered = release::displacements(beam, g, &u, &f)?;
    let mut elastic = multiply(&stiffness::local_matrix(beam, g)?, &recovered)?;
    // Double release: condensed bending is EXACTLY zero; the product
    // K·u_recuperado with huge EI only contributes rounding noise (spurious shear,
    // J4 #31), so the elastic bending part is set to zero.
    if beam.releases.i && beam.releases.j {
        for i in [1, 2, 4, 5] {
            elastic[i] = 0.0;
        }
    }
    let mut ends = [0.0; 6];
    for i in 0..6 {
        ends[i] = finite(elastic[i] - f[i], "End action")?;
    }
    // Condensation equilibrium guarantees zero; imposing it removes only
    // the rounding residual in the action of an equilibrated internal rotation.
    if beam.releases.i {
        ends[2] = 0.0;
    }
    if beam.releases.j {
        ends[5] = 0.0;
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
        let v = finite(-ends[1] - q[1] * x, "Station Vy")?;
        let m = finite(
            -ends[2] + ends[1] * x + (q[1] * (x / 2.0)) * x - q[2] * x,
            "Station Mz",
        )?;
        out.push(StationResult {
            position,
            forces_local: [n, v, 0.0, 0.0, 0.0, m],
        });
    }
    Ok(ElementOutput {
        end_forces_local: ends.to_vec(),
        stations: out,
    })
}
