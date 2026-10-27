//! Benchmark analítico propio y conformidad del piloto ENG-CORE-011.
//! Cita: Codex gpt-6.1-sol high, Elasticidad cúbica conservativa, 2026-10-20,
//! docs/METHODS_GUIDE.md §5, ecuaciones (1)–(3) y tabla analítica.

use engine_contracts::{conformance::check_uniaxial_law, UniaxialLaw};
use engine_core::{ContractId, Diagnostic, EResult, Real};
use engine_laws::uniaxial::{
    CubicElastic, CubicElasticFactory, CubicElasticParams, Elastic, UniaxialFactory, CUBIC_ELASTIC,
};

struct Fixture {
    source: &'static str,
    citation: &'static str,
    params: CubicElasticParams,
    rows: &'static [(Real, Real, Real)],
    absolute: Real,
    relative: Real,
}

// Evaluaciones independientes fijadas en la ficha antes de implementar:
// E=8 Pa, C=16 Pa; ε=1/4 da σ=2+1/4, K=8+3=11, etc.
const FIXTURE: Fixture = Fixture {
    source: "analytical",
    citation: "Codex gpt-6.1-sol high. Elasticidad cúbica conservativa, 2026-10-20; docs/METHODS_GUIDE.md §5, (1)–(3) y tabla propia.",
    params: CubicElasticParams { e: 8.0, c: 16.0 },
    rows: &[
        (0.0, 0.0, 8.0),
        (0.25, 2.25, 11.0),
        (0.5, 6.0, 20.0),
        (1.0, 24.0, 56.0),
        (0.5, 6.0, 20.0),
        (0.0, 0.0, 8.0),
        (-0.25, -2.25, 11.0),
        (-0.5, -6.0, 20.0),
        (-1.0, -24.0, 56.0),
        (0.0, 0.0, 8.0),
    ],
    // Pa y adimensional; holgura sobre pocas operaciones f64. Véase la ficha.
    absolute: 1e-12,
    relative: 1e-12,
};

fn close(actual: Real, expected: Real) {
    assert!(actual.is_finite() && expected.is_finite());
    let limit = FIXTURE.absolute.max(FIXTURE.relative * expected.abs());
    assert!(
        (actual - expected).abs() <= limit,
        "{actual} frente a {expected}, límite={limit}; {}: {}",
        FIXTURE.source,
        FIXTURE.citation
    );
}

fn code<T>(result: EResult<T>, expected: &str) {
    assert!(Diagnostic::from_code(expected).is_some());
    assert_eq!(result.err().map(|error| error.code), Some(expected));
}

#[test]
fn native_analytical_benchmark() -> EResult<()> {
    let f = FIXTURE;
    assert!(!f.source.is_empty() && !f.citation.is_empty() && !f.rows.is_empty());
    let law = CubicElastic::new(f.params.e, f.params.c)?;
    let mut committed = law.initial_state();
    for &(strain, stress, tangent) in f.rows {
        let trial = law.respond(strain, &committed)?;
        close(trial.stress, stress);
        close(trial.tangent, tangent);
        assert!(trial.trial_state.is_empty());
        committed = trial.trial_state;
    }
    Ok(())
}

#[test]
fn conformance_kit_in_tension_unloading_and_compression() -> EResult<()> {
    for c in [0.0, 16.0, 1600.0] {
        let law = CubicElastic::new(8.0, c)?;
        let diagnostics =
            check_uniaxial_law(&law, &[0.0, 0.25, 0.5, 1.0, 0.5, -0.25, -1.0, 0.0], 1e-6);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }
    Ok(())
}

#[test]
fn factory_descriptor_and_state_ownership() -> EResult<()> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<CubicElastic>();
    send_sync::<CubicElasticFactory>();
    let factory: &dyn UniaxialFactory<Params = CubicElasticParams> = &CubicElasticFactory;
    assert_eq!(factory.descriptor(), CUBIC_ELASTIC);
    assert_eq!(CUBIC_ELASTIC.kind, "elastic.cubic");
    assert_eq!(CUBIC_ELASTIC.contract, ContractId::new("uniaxial", 1, 0));
    assert_eq!(CUBIC_ELASTIC.version, (0, 1, 0));
    let law = factory.build(FIXTURE.params)?;
    assert_eq!(law.kind(), CUBIC_ELASTIC.kind);
    assert_eq!(law.contract(), CUBIC_ELASTIC.contract);
    assert_eq!(law.state_len(), 0);
    assert_eq!(law.initial_tangent(), 8.0);
    let committed = law.initial_state();
    let snapshot = committed.clone();
    let first = law.respond(0.5, &committed)?;
    law.respond(-1.0, &committed)?; // Prueba descartada, sin aceptación.
    assert_eq!(committed, snapshot);
    assert_eq!(first, law.respond(0.5, &committed)?);
    let mut discarded = first.trial_state.clone();
    discarded.push(42.0); // Vector independiente, no altera la ley.
    assert_eq!(first, law.respond(0.5, &committed)?);
    assert_eq!(first, law.respond(0.5, &first.trial_state)?);
    let concrete = CubicElastic::new(FIXTURE.params.e, FIXTURE.params.c)?;
    let original = concrete;
    concrete.respond(1.0, &[])?;
    assert_eq!(concrete, original);
    assert_eq!(concrete.params(), &FIXTURE.params);
    code(
        factory.build(CubicElasticParams { e: 0.0, c: 1.0 }),
        "ENG-E-0010",
    );
    code(
        factory.build(CubicElasticParams { e: 1.0, c: -1.0 }),
        "ENG-E-0010",
    );
    Ok(())
}

#[test]
fn symmetry_and_linear_limit() -> EResult<()> {
    let cubic = CubicElastic::new(8.0, 16.0)?;
    let linear = CubicElastic::new(8.0, 0.0)?;
    let existing = Elastic::new(8.0)?;
    for strain in [0.0, 0.001, 0.25, 1.0, 10.0] {
        let positive = cubic.respond(strain, &[])?;
        let negative = cubic.respond(-strain, &[])?;
        close(negative.stress, -positive.stress);
        close(negative.tangent, positive.tangent);
        assert!(positive.tangent >= cubic.initial_tangent());
        assert_eq!(linear.respond(strain, &[])?, existing.respond(strain, &[])?);
    }
    // ε² desbordaría; C=0 y la respuesta lineal siguen siendo representables.
    let small = CubicElastic::new(1e-300, 0.0)?;
    close(small.respond(1e300, &[])?.stress, 1.0);
    assert_eq!(small.respond(1e300, &[])?.tangent, 1e-300);
    // Cε² es representable aunque ε² no lo sea.
    let scaled = CubicElastic::new(1e-300, 1e-300)?.respond(1e200, &[])?;
    assert!((scaled.stress / 1e300 - 1.0).abs() < 1e-14);
    assert!((scaled.tangent / 3e100 - 1.0).abs() < 1e-14);
    Ok(())
}

#[test]
fn invalid_parameters_and_inputs_use_catalog() -> EResult<()> {
    for e in [0.0, -0.0, -1.0] {
        code(CubicElastic::new(e, 16.0), "ENG-E-0010");
    }
    code(CubicElastic::new(8.0, -1.0), "ENG-E-0010");
    let law = CubicElastic::new(8.0, 16.0)?;
    for bad in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        code(CubicElastic::new(bad, 16.0), "ENG-E-0008");
        code(CubicElastic::new(8.0, bad), "ENG-E-0008");
        code(law.respond(bad, &[]), "ENG-E-0008");
    }
    for history in [
        vec![0.0],
        vec![0.0; 2],
        vec![Real::NAN],
        vec![Real::INFINITY],
    ] {
        code(law.respond(0.0, &history), "ENG-E-0010");
    }
    Ok(())
}

#[test]
fn product_and_sum_overflows_are_errors() -> EResult<()> {
    // Cε, Cε², E+q, σ, 3q y E+3q, respectivamente.
    for (e, c, strain) in [
        (1.0, Real::MAX, 2.0),
        (1.0, 1.0, 1e200),
        (Real::MAX, 1e308, 1.0),
        (Real::MAX, 0.0, 2.0),
        (1.0, 8e307, 1.0),
        (1e308, 3e307, 1.0),
    ] {
        code(CubicElastic::new(e, c)?.respond(strain, &[]), "ENG-E-0008");
    }
    Ok(())
}
