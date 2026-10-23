//! Acciones locales ±c kδ; resultantes constantes sin interpolación espacial.
use crate::{finite, geometry, invalid, loads, stiffness, LinearSpring};
use engine_contracts::{ElementCtx, ElementOutput, MemberLoad, StationResult};
use engine_core::{EResult, Real};

pub(crate) fn output(
    s: &LinearSpring,
    ctx: &ElementCtx<'_>,
    u: &[Real],
    committed: &[Real],
    member_loads: &[MemberLoad],
    stations: &[Real],
) -> EResult<ElementOutput> {
    geometry::validate(s, ctx)?;
    let force = stiffness::force(s, u, committed)?;
    output_with_force(s, force, member_loads, stations)
}

pub(crate) fn output_with_force(
    s: &LinearSpring,
    force: Real,
    member_loads: &[MemberLoad],
    stations: &[Real],
) -> EResult<ElementOutput> {
    for load in member_loads {
        loads::validate(load)?;
    }
    let mut forces_local = [0.0; 6];
    for (value, c) in forces_local.iter_mut().zip(s.local) {
        *value = finite(c * force, "Local resultant")?;
    }
    let mut out = Vec::new();
    out.try_reserve_exact(stations.len())
        .map_err(|_| invalid("Could not allocate memory for stations."))?;
    for &position in stations {
        finite(position, "Station x/L")?;
        if !(0.0..=1.0).contains(&position) {
            return Err(invalid("Station is outside [0,1]."));
        }
        out.push(StationResult {
            position,
            forces_local,
        });
    }
    Ok(ElementOutput {
        end_forces_local: stiffness::vector(s, force, true)?,
        stations: out,
    })
}
