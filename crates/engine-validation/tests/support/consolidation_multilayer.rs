//! Spectral reference independent of the FE matrices and time stepping.
//! Derivation, spectrum, projection and tail bound: provenance/ENG-GEO-006.md.
use super::super::table::{MULTILAYER, MULTILAYER_CITATION, MULTILAYER_SOURCE};
use super::{geo, load, measured, require};
use engine_core::math;
use engine_core::EResult;
use engine_geo::consolidation::{
    ConsolidationLayer, ConsolidationOptions, ConsolidationProfile, Drainage,
};
use std::f64::consts::PI;

struct Mode {
    omega: f64,
    // X_i(xi) = A_i cos(omega xi) + B_i sin(omega xi), xi in [0,1].
    coefficients: [(f64, f64); 3],
    integral: f64,
    norm: f64,
}
impl Mode {
    fn new(omega: f64) -> Self {
        let c = math::cos(omega);
        let s = math::sin(omega);
        let impedances = [0.001, 0.0015, 0.003];
        let mut coefficients = [(0.0, 1.0); 3];
        for i in 1..3 {
            let (a, b) = coefficients[i - 1];
            coefficients[i] = (
                a * c + b * s,
                impedances[i - 1] / impedances[i] * (-a * s + b * c),
            );
        }
        let mut integral = 0.0;
        let mut norm = 0.0;
        for (&impedance, &(a, b)) in impedances.iter().zip(&coefficients) {
            integral += impedance * (a * s + b * (1.0 - c)) / omega;
            let cross = math::sin(2.0 * omega) / (4.0 * omega);
            norm +=
                impedance * (a * a * (0.5 + cross) + b * b * (0.5 - cross) + a * b * s * s / omega);
        }
        Self {
            omega,
            coefficients,
            integral,
            norm,
        }
    }
    fn shape(&self, layer: usize, xi: f64) -> f64 {
        let (a, b) = self.coefficients[layer];
        a * math::cos(self.omega * xi) + b * math::sin(self.omega * xi)
    }
    // k*u_z (gamma_w times hydraulic flux with reversed sign), per unit mode.
    fn gradient_flux(&self, layer: usize, xi: f64) -> f64 {
        let (a, b) = self.coefficients[layer];
        let impedance = [0.001, 0.0015, 0.003][layer];
        10.0 * impedance
            * self.omega
            * (-a * math::sin(self.omega * xi) + b * math::cos(self.omega * xi))
    }
}
fn modes() -> Vec<Mode> {
    let alpha = math::atan(1.0 / math::sqrt(6.5));
    (0..16)
        .flat_map(|n| {
            let base = n as f64 * PI;
            [base + alpha, base + PI / 2.0, base + PI - alpha].map(Mode::new)
        })
        .collect()
}
fn pressure(modes: &[Mode], t: f64, layer: usize, xi: f64) -> f64 {
    modes
        .iter()
        .map(|m| {
            100.0 * m.integral / m.norm * m.shape(layer, xi) * math::exp(-m.omega * m.omega * t)
        })
        .sum()
}

pub fn run() -> EResult<()> {
    require(
        !MULTILAYER_SOURCE.is_empty() && !MULTILAYER_CITATION.is_empty(),
        "Multicapa sin procedencia",
    )?;
    let modes = modes();
    for m in &modes {
        require(m.norm > 0.0, "Norma modal no positiva")?;
        require(m.shape(0, 0.0) == 0.0, "Modo no satisface dren superior")?;
        require(
            m.gradient_flux(2, 1.0).abs() < 1e-12,
            "Modo no satisface cara inferior impermeable",
        )?;
        for i in 0..2 {
            require(
                (m.shape(i, 1.0) - m.shape(i + 1, 0.0)).abs() < 1e-12
                    && (m.gradient_flux(i, 1.0) - m.gradient_flux(i + 1, 0.0)).abs() < 1e-12,
                "Modo no satisface continuidad de presión/flujo",
            )?;
        }
    }
    let layers = MULTILAYER
        .iter()
        .map(|&(h, cv, mv, k)| ConsolidationLayer {
            thickness_m: measured(h),
            cv_m2_s: Some(measured(cv)),
            mv_per_kpa: Some(measured(mv)),
            k_m_s: Some(measured(k)),
            elements: 32,
        })
        .collect();
    let profile =
        ConsolidationProfile::new(layers, Some(measured(10.0)), Drainage::Top).map_err(geo)?;
    let times = [0.1, 0.5, 1.0, 3.0, 10.0, 30.0];
    let result = profile
        .solve(
            &[load(0.0, 100.0), load(30.0, 100.0)],
            &times,
            ConsolidationOptions::new(0.002),
        )
        .map_err(geo)?;
    let mut max_pressure: f64 = 0.0;
    let mut max_settlement: f64 = 0.0;
    let mut max_degree: f64 = 0.0;
    let mut max_flux: f64 = 0.0;
    for (&t, state) in times.iter().zip(&result.states) {
        let residual: f64 = modes
            .iter()
            .map(|m| 100.0 * m.integral * m.integral / m.norm * math::exp(-m.omega * m.omega * t))
            .sum();
        let exact_settlement = 0.55 - residual;
        let exact_degree = exact_settlement / 0.55;
        let settlement_error = (state.settlement_m - exact_settlement).abs() / 0.55;
        let degree_error = (state.mean_degree.unwrap_or(f64::NAN) / exact_degree - 1.0).abs();
        max_settlement = max_settlement.max(settlement_error);
        max_degree = max_degree.max(degree_error);
        require(
            settlement_error < 0.0015 && degree_error < 0.005,
            format!("Multicapa t={t}: error s/s∞={settlement_error}, relativo U={degree_error}"),
        )?;
        for (j, &z) in result.depths_m.iter().enumerate() {
            let (i, xi) = if z <= 0.5 {
                (0, z / 0.5)
            } else if z <= 2.0 {
                (1, (z - 0.5) / 1.5)
            } else {
                (2, z - 2.0)
            };
            let error =
                (state.excess_pore_pressure_kpa[j] - pressure(&modes, t, i, xi)).abs() / 100.0;
            max_pressure = max_pressure.max(error);
            require(
                error < 0.004,
                format!("Multicapa t={t} z={z}: error u/100={error}"),
            )?;
        }
        // At t=1 both interfaces carry a significant nonzero flux. P1 slopes
        // approximate the common analytical k*u_z from each side separately.
        if t == 1.0 {
            let u = &state.excess_pore_pressure_kpa;
            let z = &result.depths_m;
            for (i, j) in [(0, 32), (1, 64)] {
                let exact: f64 = modes
                    .iter()
                    .map(|m| {
                        100.0 * m.integral / m.norm
                            * m.gradient_flux(i, 1.0)
                            * math::exp(-m.omega * m.omega * t)
                    })
                    .sum();
                require(exact.abs() > 0.01, "Interfaz sin flujo significativo")?;
                for (e, k) in [(j - 1, MULTILAYER[i].3), (j, MULTILAYER[i + 1].3)] {
                    let computed = k * (u[e + 1] - u[e]) / (z[e + 1] - z[e]);
                    let error = (computed / exact - 1.0).abs();
                    max_flux = max_flux.max(error);
                    require(
                        error < 0.03,
                        format!("Multicapa interfaz {i}: error flujo={error}"),
                    )?;
                }
            }
        }
    }
    println!(
        "Multicapa no transformable: max |error u|/100={max_pressure:.10}, |error s|/s∞={max_settlement:.10}, relativo U={max_degree:.10}, relativo flujo={max_flux:.10}"
    );
    Ok(())
}
