//! Benchmarks compartidos y adaptación dimensional a ENG-MAT-002.
#[path = "support/py_harness.rs"]
mod py_harness;
#[path = "support/py_table.rs"]
mod py_table;
use engine_contracts::UniaxialLaw;
use engine_core::EResult;
use engine_geo::py::{matlock_soft_clay, user, MatlockParams, Sampling};
use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};

#[test]
fn analytical_py_benchmarks() -> EResult<()> {
    assert_eq!(py_harness::run_all()?, py_table::FIXTURES.len());
    Ok(())
}
#[test]
fn generated_and_user_points_feed_multilinear_with_explicit_unit_conversion() -> EResult<()> {
    let p = MatlockParams {
        depth_m: Some(2.0),
        diameter_m: Some(1.0),
        undrained_strength_pa: Some(20000.0),
        effective_vertical_stress_pa: Some(20000.0),
        j: Some(0.5),
        epsilon_50: Some(0.01),
    };
    let s = Sampling {
        positive_y_m: vec![0.003125, 0.025],
        plateau_y_m: Some(0.4),
    };
    let generated = matlock_soft_clay(&p, &s)?.curve;
    let supplied = user(vec![(0.0, 0.0), (0.01, 100.0), (0.02, 150.0)])?;
    for curve in [generated, supplied] {
        let length = 2.0; // m; extensión tributaria del pilote y escala de ε explícita.
        let area = 0.5; // m²; escala de σ explícita.
        let points = curve
            .points()
            .iter()
            .map(|&(y, p)| (y / length, p * length / area))
            .collect();
        let law = MultilinearElastic::new(points, MultilinearMode::Both)?;
        for &(y, p) in curve.points() {
            let recovered = law.respond(y / length, &[])?.stress * area / length;
            assert!((recovered - p).abs() < 1e-8);
        }
        for y in [-0.005, 0.0, 0.005] {
            let recovered = law.respond(y / length, &[])?.stress * area / length;
            assert!((recovered - curve.at(y)?).abs() < 1e-8);
        }
    }
    Ok(())
}
