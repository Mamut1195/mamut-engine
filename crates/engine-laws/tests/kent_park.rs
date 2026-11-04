//! Two hand points of the Kent–Park envelope, plus domain and tangent checks.
//! Citation: Sharma, Reddy, Vaze, Ghosh, Kushwaha and Eligehausen,
//! BARC/2009/E/026, 2009, Appendix A, section A.1.1.3, equations (A.9) and
//! (A.10), with Z from (A.11)–(A.13). Sheet: docs/fichas/concrete.kent_park.md.
//! The 1 psi factor below converts (A.12) to pascals. It is not the constitutive equation.

use engine_contracts::{conformance::check_uniaxial_law, UniaxialLaw};
use engine_core::{ContractId, Diagnostic, EResult, Real};
use engine_laws::uniaxial::{
    KentPark, KentParkFactory, KentParkParams, UniaxialFactory, KENT_PARK,
};

/// 1 psi in pascals. Same unit factor as the law, not a fitted constant.
fn psi_in_pa() -> Real {
    4.4482216152605 / (0.0254 * 0.0254)
}

struct Fixture {
    source: &'static str,
    citation: &'static str,
    params: KentParkParams,
    /// Engine strain, stress (Pa), tangent (Pa). Two rows only.
    rows: [(Real, Real, Real); 2],
    absolute: Real,
    relative: Real,
}

fn hand_fixture() -> Fixture {
    // f'_c = 3000 psi, rho_s = 0. (A.12) gives ε_50u = 0.0045.
    // (A.11) gives Z = 200. ε_20c = 0.006.
    // (A.9) at ε_c = 0.001: f_c/f'_c = 3/4, K/f'_c = 500.
    // (A.10) at ε_c = 0.004: f_c/f'_c = 3/5, K/f'_c = -200.
    let fc = 3000.0 * psi_in_pa();
    Fixture {
        source: "analytical",
        citation: "Sharma, Reddy, Vaze, Ghosh, Kushwaha and Eligehausen, BARC/2009/E/026 (2009), Appendix A, A.1.1.3, (A.9) and (A.10); Z from (A.11)–(A.13). docs/fichas/concrete.kent_park.md.",
        params: KentParkParams {
            fc,
            rho_s: 0.0,
            b_double_prime: 0.2,
            s_h: 0.1,
        },
        rows: [(-0.001, -0.75 * fc, 500.0 * fc), (-0.004, -0.6 * fc, -200.0 * fc)],
        absolute: 1e-9,
        relative: 1e-12,
    }
}

fn close(actual: Real, expected: Real) {
    let fixture = hand_fixture();
    assert!(actual.is_finite() && expected.is_finite());
    let limit = fixture.absolute.max(fixture.relative * expected.abs());
    assert!(
        (actual - expected).abs() <= limit,
        "{actual} versus {expected}, limit={limit}; {}: {}",
        fixture.source,
        fixture.citation
    );
}

fn code<T>(result: EResult<T>, expected: &str) {
    assert!(Diagnostic::from_code(expected).is_some());
    assert_eq!(result.err().map(|error| error.code), Some(expected));
}

#[test]
fn native_analytical_benchmark() -> EResult<()> {
    let fixture = hand_fixture();
    assert!(!fixture.source.is_empty() && !fixture.citation.is_empty());
    assert_eq!(fixture.rows.len(), 2);
    let law = KentPark::new(
        fixture.params.fc,
        fixture.params.rho_s,
        fixture.params.b_double_prime,
        fixture.params.s_h,
    )?;
    let mut committed = law.initial_state();
    assert!(committed.is_empty());
    for (strain, stress, tangent) in fixture.rows {
        let trial = law.respond(strain, &committed)?;
        close(trial.stress, stress);
        close(trial.tangent, tangent);
        assert!(trial.trial_state.is_empty());
        committed = trial.trial_state;
    }
    Ok(())
}

#[test]
fn tangents_on_both_branches_and_at_the_peak() -> EResult<()> {
    let fixture = hand_fixture();
    let law = KentPark::new(
        fixture.params.fc,
        fixture.params.rho_s,
        fixture.params.b_double_prime,
        fixture.params.s_h,
    )?;
    let parabola = law.respond(-0.001, &[])?;
    let falling = law.respond(-0.004, &[])?;
    close(parabola.tangent, 500.0 * fixture.params.fc);
    close(falling.tangent, -200.0 * fixture.params.fc);
    assert!(parabola.tangent > 0.0);
    assert!(falling.tangent < 0.0);
    // (A.9) at ε_c = 0.002. The factored derivative is exactly zero.
    let peak = law.respond(-0.002, &[])?;
    assert_eq!(peak.stress, -fixture.params.fc);
    assert_eq!(peak.tangent, 0.0);
    let diagnostics = check_uniaxial_law(&law, &[-0.001, -0.004], 1e-6);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    Ok(())
}

#[test]
fn initial_state_matches_the_origin_tangent() -> EResult<()> {
    let fixture = hand_fixture();
    let law = KentPark::new(
        fixture.params.fc,
        fixture.params.rho_s,
        fixture.params.b_double_prime,
        fixture.params.s_h,
    )?;
    assert_eq!(law.state_len(), 0);
    assert!(law.initial_state().is_empty());
    let origin = law.respond(0.0, &law.initial_state())?;
    assert_eq!(origin.stress, 0.0);
    assert_eq!(origin.tangent, law.initial_tangent());
    assert_eq!(origin.tangent, 1000.0 * fixture.params.fc);
    assert!(origin.trial_state.is_empty());
    let tension = law.respond(0.001, &[])?;
    assert_eq!((tension.stress, tension.tangent), (0.0, 0.0));
    Ok(())
}

#[test]
fn factory_descriptor_and_state_ownership() -> EResult<()> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<KentPark>();
    send_sync::<KentParkFactory>();
    let fixture = hand_fixture();
    let factory: &dyn UniaxialFactory<Params = KentParkParams> = &KentParkFactory;
    assert_eq!(factory.descriptor(), KENT_PARK);
    assert_eq!(KENT_PARK.kind, "concrete.kent_park");
    assert_eq!(KENT_PARK.contract, ContractId::new("uniaxial", 1, 0));
    assert_eq!(KENT_PARK.version, (0, 1, 0));
    let law = factory.build(fixture.params)?;
    assert_eq!(law.kind(), KENT_PARK.kind);
    assert_eq!(law.contract(), KENT_PARK.contract);
    assert_eq!(law.state_len(), 0);
    let committed = law.initial_state();
    let snapshot = committed.clone();
    let first = law.respond(-0.001, &committed)?;
    law.respond(-0.004, &committed)?;
    assert_eq!(committed, snapshot);
    assert_eq!(first, law.respond(-0.001, &committed)?);
    let concrete = KentPark::new(
        fixture.params.fc,
        fixture.params.rho_s,
        fixture.params.b_double_prime,
        fixture.params.s_h,
    )?;
    let original = concrete;
    concrete.respond(-0.001, &[])?;
    assert_eq!(concrete, original);
    assert_eq!(concrete.params(), &fixture.params);
    Ok(())
}

#[test]
fn invalid_parameters_and_inputs_use_catalog() -> EResult<()> {
    let psi = psi_in_pa();
    let fc = 3000.0 * psi;
    for bad_fc in [0.0, -0.0, -1.0, 1000.0 * psi, 999.0 * psi, 1.0] {
        code(KentPark::new(bad_fc, 0.0, 0.2, 0.1), "ENG-E-0010");
    }
    code(KentPark::new(fc, -1.0, 0.2, 0.1), "ENG-E-0010");
    code(KentPark::new(fc, 0.0, 0.0, 0.1), "ENG-E-0010");
    code(KentPark::new(fc, 0.0, -0.2, 0.1), "ENG-E-0010");
    code(KentPark::new(fc, 0.0, 0.2, 0.0), "ENG-E-0010");
    code(KentPark::new(fc, 0.0, 0.2, -0.1), "ENG-E-0010");
    let law = KentPark::new(fc, 0.0, 0.2, 0.1)?;
    for bad in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        code(KentPark::new(bad, 0.0, 0.2, 0.1), "ENG-E-0008");
        code(KentPark::new(fc, bad, 0.2, 0.1), "ENG-E-0008");
        code(KentPark::new(fc, 0.0, bad, 0.1), "ENG-E-0008");
        code(KentPark::new(fc, 0.0, 0.2, bad), "ENG-E-0008");
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
    code(KentPark::new(Real::MAX, 0.0, 0.2, 0.1), "ENG-E-0008");
    code(KentPark::new(fc, Real::MAX, 0.2, 0.1), "ENG-E-0008");
    Ok(())
}
