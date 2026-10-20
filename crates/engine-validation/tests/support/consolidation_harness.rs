use super::table::{Fixture, FIXTURES};
use engine_core::math;
use engine_core::{EResult, EngineError};
use engine_geo::{
    consolidation::{
        ConsolidationLayer, ConsolidationOptions, ConsolidationProfile, Drainage, LoadPoint,
    },
    Parameter, Provenance,
};
use std::f64::consts::PI;

#[path = "consolidation_multilayer.rs"]
mod multilayer;

pub fn measured(value: f64) -> Parameter {
    Parameter::new(value, Provenance::Measured)
}
pub fn layer(h: f64, cv: f64, mv: f64, elements: usize) -> ConsolidationLayer {
    ConsolidationLayer {
        thickness_m: measured(h),
        cv_m2_s: Some(measured(cv)),
        mv_per_kpa: Some(measured(mv)),
        k_m_s: Some(measured(cv * mv * 10.0)),
        elements,
    }
}
pub fn load(t: f64, stress: f64) -> LoadPoint {
    LoadPoint {
        time_s: t,
        delta_stress_kpa: Some(measured(stress)),
    }
}
fn error(message: impl Into<String>) -> EngineError {
    EngineError::new("ENG-E-0012", message)
}
fn geo(e: engine_geo::Diagnostic) -> EngineError {
    EngineError::new(e.code, format!("{} {:?}", e.message, e.hint))
}
fn require(ok: bool, message: impl Into<String>) -> EResult<()> {
    if ok {
        Ok(())
    } else {
        Err(error(message))
    }
}

// Remaining mean pressure and pressure at normalized distance x from the drain.
// For a ramp of duration r, integrate each exponential analytically.
pub fn remaining(tv: f64, x: f64, ramp: bool) -> (f64, f64) {
    let mut mean = 0.0;
    let mut pressure = 0.0;
    for m in 0..2048 {
        let w = PI * (2 * m + 1) as f64 / 2.0;
        let decay = if ramp {
            let r = 0.2;
            let active = tv.min(r);
            math::exp(-w * w * (tv - active)) * (-math::exp_m1(-w * w * active)) / (w * w * r)
        } else {
            math::exp(-w * w * tv)
        };
        mean += 2.0 / (w * w) * decay;
        pressure += 2.0 / w * math::sin(w * x) * decay;
    }
    (mean, pressure)
}
fn run_fixture(f: Fixture) -> EResult<()> {
    require(
        !f.source.is_empty() && !f.citation.is_empty() && !f.tolerance.justification.is_empty(),
        "Fixture sin cita",
    )?;
    let layers = if f.transformed_layers {
        vec![layer(1.0, 0.25, 0.002, 32), layer(2.0, 1.0, 0.001, 32)]
    } else {
        vec![layer(
            2.0,
            0.25,
            0.002,
            if f.drainage == Drainage::Double {
                128
            } else {
                64
            },
        )]
    };
    let profile =
        ConsolidationProfile::new(layers, Some(measured(10.0)), f.drainage).map_err(geo)?;
    // Hdr²/cv = 16 s, except double drainage: 4 s.
    let scale = if f.drainage == Drainage::Double {
        4.0
    } else {
        16.0
    };
    let history = if f.unload {
        vec![
            load(0.0, 0.0),
            load(0.2 * scale, 100.0),
            load(0.4 * scale, 100.0),
            load(0.6 * scale, 0.0),
            load(scale, 0.0),
        ]
    } else if f.ramp {
        vec![load(0.0, 0.0), load(0.2 * scale, 100.0), load(scale, 100.0)]
    } else {
        vec![load(0.0, 100.0), load(scale, 100.0)]
    };
    let tvs = [0.01, 0.05, 0.1, 0.2, 0.4, 0.6, 1.0];
    let times: Vec<_> = tvs.iter().map(|tv| tv * scale).collect();
    let result = profile
        .solve(&history, &times, ConsolidationOptions::new(0.0005 * scale))
        .map_err(geo)?;
    let mut max_u_error: f64 = 0.0;
    let mut max_pressure_error: f64 = 0.0;
    for (&tv, state) in tvs.iter().zip(&result.states) {
        let mean = remaining(tv, 0.0, f.ramp).0
            - if f.unload && tv > 0.4 {
                remaining(tv - 0.4, 0.0, true).0
            } else {
                0.0
            };
        let stress = if f.unload {
            if tv <= 0.2 {
                tv / 0.2
            } else if tv <= 0.4 {
                1.0
            } else if tv < 0.6 {
                1.0 - (tv - 0.4) / 0.2
            } else {
                0.0
            }
        } else if f.ramp {
            (tv / 0.2).min(1.0)
        } else {
            1.0
        };
        let expected_settlement = 0.004 * 100.0 * (stress - mean);
        let settlement_error = (state.settlement_m - expected_settlement).abs() / 0.4;
        require(
            settlement_error < f.tolerance.settlement_normalized,
            format!("{}: error s/s∞ {settlement_error}", f.name),
        )?;
        if stress == 0.0 {
            require(
                state.mean_degree.is_none(),
                "U debe ser indefinido sin tensión aplicada",
            )?;
        } else {
            let expected_degree = 1.0 - mean / stress;
            let degree = state.mean_degree.ok_or_else(|| error("Falta U"))?;
            let relative = (degree - expected_degree).abs() / expected_degree.abs().max(1e-15);
            max_u_error = max_u_error.max(relative);
            require(
                relative < f.tolerance.degree_relative,
                format!(
                    "{} Tv={tv}: U={degree}, serie={expected_degree}, error relativo={relative}",
                    f.name
                ),
            )?;
        }
        for (i, &z) in result.depths_m.iter().enumerate() {
            let x = if f.transformed_layers {
                if z <= 1.0 {
                    z / 2.0
                } else {
                    (2.0 + z - 1.0) / 4.0
                }
            } else {
                match f.drainage {
                    Drainage::Top => z / 2.0,
                    Drainage::Bottom => (2.0 - z) / 2.0,
                    Drainage::Double => z.min(2.0 - z),
                }
            };
            let expected_u = remaining(tv, x, f.ramp).1
                - if f.unload && tv > 0.4 {
                    remaining(tv - 0.4, x, true).1
                } else {
                    0.0
                };
            let pressure_error = (state.excess_pore_pressure_kpa[i] / 100.0 - expected_u).abs();
            max_pressure_error = max_pressure_error.max(pressure_error);
            require(
                pressure_error < f.tolerance.pressure_normalized,
                format!("{} Tv={tv} z={z}: error u/Δσ={pressure_error}", f.name),
            )?;
        }
    }
    // At Tv=.2 the transformed solution has nonzero continuous interfacial flux.
    if f.transformed_layers {
        let state = &result.states[3];
        let u = &state.excess_pore_pressure_kpa;
        let z = &result.depths_m;
        let left = -0.25 * 0.002 * (u[32] - u[31]) / (z[32] - z[31]);
        let right = -0.001 * (u[33] - u[32]) / (z[33] - z[32]);
        let mut exact = 0.0;
        for m in 0..128 {
            let w = PI * (2 * m + 1) as f64 / 2.0;
            exact -= 0.001 * 100.0 / 4.0 * 2.0 * math::cos(w * 0.5) * math::exp(-w * w * 0.2);
        }
        require(exact.abs() > 0.001, "El benchmark debe tener flujo no nulo")?;
        require(
            (left / exact - 1.0).abs() < f.tolerance.flux_relative
                && (right / exact - 1.0).abs() < f.tolerance.flux_relative,
            "Flujo de interfaz no converge al analítico",
        )?;
    }
    println!(
        "{}: max error relativo U={:.8}%, max |error u|/100={:.8}%",
        f.name,
        100.0 * max_u_error,
        100.0 * max_pressure_error
    );
    Ok(())
}

pub fn convergence() -> EResult<()> {
    let exact = 1.0 - remaining(0.2, 0.0, false).0;
    let mut previous = f64::INFINITY;
    for n in [8, 16, 32, 64] {
        let profile = ConsolidationProfile::new(
            vec![layer(2.0, 0.25, 0.002, n)],
            Some(measured(10.0)),
            Drainage::Top,
        )
        .map_err(geo)?;
        let r = profile
            .solve(
                &[load(0.0, 100.0), load(3.2, 100.0)],
                &[3.2],
                ConsolidationOptions::new(0.00025 * 16.0),
            )
            .map_err(geo)?;
        let degree = r.states[0].mean_degree.ok_or_else(|| error("U ausente"))?;
        let err = (degree - exact).abs();
        require(err < previous, "No converge la malla")?;
        previous = err;
        println!("mesh n={n}, dTv=.00025, U={degree:.12}, |U-serie|={err:.12}");
    }
    let profile = ConsolidationProfile::new(
        vec![layer(2.0, 0.25, 0.002, 64)],
        Some(measured(10.0)),
        Drainage::Top,
    )
    .map_err(geo)?;
    let mut temporal = Vec::new();
    for d in [0.02, 0.01, 0.005, 0.0025, 0.00125, 0.00025] {
        let r = profile
            .solve(
                &[load(0.0, 100.0), load(3.2, 100.0)],
                &[3.2],
                ConsolidationOptions::new(d * 16.0),
            )
            .map_err(geo)?;
        temporal.push(r.states[0].mean_degree.ok_or_else(|| error("U ausente"))?);
        println!(
            "time n=64, dTv={d}, U={:.12}, |U-serie|={:.12}",
            temporal[temporal.len() - 1],
            (temporal[temporal.len() - 1] - exact).abs()
        );
    }
    let reference = temporal[5];
    for pair in temporal[..5].windows(2) {
        require(
            (pair[1] - reference).abs() < (pair[0] - reference).abs(),
            "No converge el paso temporal",
        )?;
    }
    Ok(())
}
pub fn run_all() -> EResult<usize> {
    for &f in FIXTURES {
        run_fixture(f)?;
    }
    convergence()?;
    multilayer::run()?;
    Ok(FIXTURES.len() + 1)
}
