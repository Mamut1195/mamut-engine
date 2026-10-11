//! Equilibrio por cara positiva: N′=−qx, V′=−qy, M′=−V−mz.
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
    // Doble liberación: la flexión condensada es EXACTAMENTE cero; el producto
    // K·u_recuperado con EI enorme solo aporta ruido de redondeo (cortante espurio,
    // J4 #31), así que la parte flexional elástica se fija en cero.
    if beam.releases.i && beam.releases.j {
        for i in [1, 2, 4, 5] {
            elastic[i] = 0.0;
        }
    }
    let mut ends = [0.0; 6];
    for i in 0..6 {
        ends[i] = finite(elastic[i] - f[i], "Acción de extremo")?;
    }
    // Equilibrio de la condensación garantiza cero; imponerlo elimina solo
    // el residuo de redondeo en la acción de un giro interno equilibrado.
    if beam.releases.i {
        ends[2] = 0.0;
    }
    if beam.releases.j {
        ends[5] = 0.0;
    }
    let mut out = Vec::new();
    out.try_reserve_exact(stations.len())
        .map_err(|_| invalid("No se pudo reservar estaciones."))?;
    for &position in stations {
        finite(position, "Estación x/L")?;
        if !(0.0..=1.0).contains(&position) {
            return Err(invalid("Estación fuera de [0,1]."));
        }
        let x = position * g.length;
        let n = finite(-ends[0] - q[0] * x, "N estación")?;
        let v = finite(-ends[1] - q[1] * x, "Vy estación")?;
        let m = finite(
            -ends[2] + ends[1] * x + (q[1] * (x / 2.0)) * x - q[2] * x,
            "Mz estación",
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
