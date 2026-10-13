//! Fixtures analíticos propios, sin oráculos ni fuentes externas.
//! Cita: provenance/ENG-MAT-003.md, derivación y ciclo de histéresis.

use engine_contracts::{conformance::check_uniaxial_law, UniaxialLaw};
use engine_core::{ContractId, Diagnostic, EResult, Real};
use engine_laws::uniaxial::{
    SteelBilinearKinematic, SteelBilinearKinematicFactory, SteelBilinearKinematicParams,
    UniaxialFactory, STEEL_BILINEAR_KINEMATIC,
};

fn close(actual: Real, expected: Real) {
    assert!(
        (actual - expected).abs() <= 1e-12 + 1e-12 * expected.abs(),
        "actual={actual}, esperado={expected}"
    );
}

fn code<T>(result: EResult<T>, expected: &str) {
    assert!(Diagnostic::from_code(expected).is_some());
    match result {
        Err(error) => assert_eq!(error.code, expected, "{error}"),
        Ok(_) => panic!("Se esperaba {expected}"),
    }
}

#[test]
fn full_hysteresis_cycle_matches_hand_table() -> EResult<()> {
    // Fuente: derivación propia de retorno 1D; E=200, fy=1, H=200/9.
    // Cita: provenance/ENG-MAT-003.md, tabla analítica del ciclo.
    // Columnas: ε objetivo, σ, α, ε_p, tangente. Se evitan las inversiones
    // sobre la superficie, donde el redondeo puede decidir la rama.
    let table = [
        (0.0, 0.0, 0.0, 0.0, 200.0),
        (0.0025, 0.5, 0.0, 0.0, 200.0),
        (0.005, 1.0, 0.0, 0.0, 200.0),
        (0.02, 1.3, 0.3, 0.0135, 20.0),
        (0.0175, 0.8, 0.3, 0.0135, 200.0),
        (0.015, 0.3, 0.3, 0.0135, 200.0),
        (0.0125, -0.2, 0.3, 0.0135, 200.0),
        (0.005, -0.8, 0.2, 0.009, 20.0),
        (-0.02, -1.3, -0.3, -0.0135, 20.0),
        (-0.0175, -0.8, -0.3, -0.0135, 200.0),
        (-0.015, -0.3, -0.3, -0.0135, 200.0),
        (-0.0125, 0.2, -0.3, -0.0135, 200.0),
        (-0.005, 0.8, -0.2, -0.009, 20.0),
        (0.02, 1.3, 0.3, 0.0135, 20.0),
    ];
    let law = SteelBilinearKinematic::new(200.0, 1.0, 0.1)?;
    let mut committed = law.initial_state();
    for (strain, stress, alpha, plastic_strain, tangent) in table {
        let trial = law.respond(strain, &committed)?;
        close(trial.stress, stress);
        close(trial.trial_state[0], plastic_strain);
        close(trial.trial_state[1], alpha);
        close(trial.tangent, tangent);
        close(trial.stress, 200.0 * (strain - trial.trial_state[0]));
        close(trial.trial_state[1], (200.0 / 9.0) * trial.trial_state[0]);
        // Repetir una deformación aceptada reproduce tensión e historia.
        let repeated = law.respond(strain, &trial.trial_state)?;
        close(repeated.stress, trial.stress);
        for (&actual, &expected) in repeated.trial_state.iter().zip(&trial.trial_state) {
            close(actual, expected);
        }
        committed = trial.trial_state;
    }
    Ok(())
}

#[test]
fn bauschinger_elastic_range_is_two_fy_after_each_reversal() -> EResult<()> {
    for b in [0.0, 0.1, 0.8] {
        let law = SteelBilinearKinematic::new(200.0, 1.0, b)?;
        let mut committed = law.initial_state();
        for (peak, direction) in [(0.02, -1.0), (-0.02, 1.0), (0.03, -1.0), (-0.03, 1.0)] {
            let at_peak = law.respond(peak, &committed)?;
            committed = at_peak.trial_state;
            let boundary = peak + direction * 0.01; // 2fy/E = 0.01
            let at_boundary = law.respond(boundary, &committed)?;
            close((at_boundary.stress - at_peak.stress).abs(), 2.0);
            close(at_boundary.stress - committed[1], direction);
            close(at_boundary.trial_state[0], committed[0]);
            close(at_boundary.trial_state[1], committed[1]);
            let inside = law.respond(boundary - direction * 1e-6, &committed)?;
            assert_eq!(inside.tangent, 200.0);
            assert_eq!(inside.trial_state, committed);
            let outside = law.respond(boundary + direction * 1e-6, &committed)?;
            close(outside.tangent, b * 200.0);
            assert!(direction * (outside.trial_state[0] - committed[0]) > 0.0);
        }
    }
    Ok(())
}

#[test]
fn closed_cycle_energy_is_hand_calculated_parallelogram_area() -> EResult<()> {
    let law = SteelBilinearKinematic::new(200.0, 1.0, 0.1)?;
    let first = law.respond(0.02, &law.initial_state())?;
    let mut committed = first.trial_state.clone();
    let (mut strain_previous, mut stress_previous) = (0.02, first.stress);
    let (mut work, mut plastic_dissipation) = (0.0, 0.0);
    // Vértices A→B→C→D→A del paralelogramo, no incluye la carga inicial.
    // Cada lado es lineal: la regla trapezoidal integra exactamente σ dε.
    for strain in [0.01, -0.02, -0.01, 0.02] {
        let trial = law.respond(strain, &committed)?;
        work += 0.5 * (trial.stress + stress_previous) * (strain - strain_previous);
        plastic_dissipation += (trial.trial_state[0] - committed[0]).abs(); // fy=1
        committed = trial.trial_state;
        (strain_previous, stress_previous) = (strain, trial.stress);
    }
    // A=(.02,1.3), B=(.01,-.7), C=(-.02,-1.3), D=(-.01,.7).
    // Área = 4fy(1−b)(ε_max−fy/E) = 4*.9*.015 = .054 Pa.
    close(work, 0.054);
    close(plastic_dissipation, 0.054);
    close(stress_previous, first.stress);
    for (&actual, &expected) in committed.iter().zip(&first.trial_state) {
        close(actual, expected);
    }
    Ok(())
}

#[test]
fn perfect_plasticity_has_plateaus_and_elastic_unloading() -> EResult<()> {
    let law = SteelBilinearKinematic::new(200.0, 1.0, 0.0)?;
    let mut committed = law.initial_state();
    for (strain, stress, plastic, tangent) in [
        (0.02, 1.0, 0.015, 0.0),
        (0.03, 1.0, 0.025, 0.0),
        (0.0275, 0.5, 0.025, 200.0),
        (-0.02, -1.0, -0.015, 0.0),
        (-0.03, -1.0, -0.025, 0.0),
        (-0.0275, -0.5, -0.025, 200.0),
        (0.02, 1.0, 0.015, 0.0),
    ] {
        let trial = law.respond(strain, &committed)?;
        close(trial.stress, stress);
        close(trial.tangent, tangent);
        close(trial.trial_state[0], plastic);
        assert_eq!(trial.trial_state[1], 0.0);
        committed = trial.trial_state;
    }
    Ok(())
}

#[test]
fn trial_history_is_owned_and_acceptance_is_explicit() -> EResult<()> {
    let law = SteelBilinearKinematic::new(200.0, 1.0, 0.1)?;
    let original = law;
    let mut committed = law.initial_state();
    let first = law.respond(0.02, &committed)?;
    let unaccepted = law.respond(0.015, &committed)?;
    close(unaccepted.stress, 1.2);
    assert_eq!(committed, vec![0.0, 0.0]);
    assert_eq!(law, original);
    assert_eq!(first, law.respond(0.02, &committed)?);
    let mut discarded_state = first.trial_state.clone();
    discarded_state.fill(42.0);
    assert_eq!(first, law.respond(0.02, &committed)?);
    committed = first.trial_state; // único punto de aceptación
    let accepted_snapshot = committed.clone();
    close(law.respond(0.015, &committed)?.stress, 0.3);
    law.respond(-0.02, &committed)?; // prueba descartada
    assert_eq!(committed, accepted_snapshot);
    close(law.respond(0.015, &committed)?.stress, 0.3);
    Ok(())
}

#[test]
fn tangent_matches_finite_differences_with_fixed_committed_history() -> EResult<()> {
    for b in [0.0, 0.1, 0.8] {
        let law = SteelBilinearKinematic::new(200.0, 1.0, b)?;
        let initial = law.initial_state();
        let tensile = law.respond(0.02, &initial)?.trial_state;
        let compressive = law.respond(-0.02, &tensile)?.trial_state;
        for committed in [&initial, &tensile, &compressive] {
            for strain in [-0.035, -0.024, -0.016, -0.003, 0.003, 0.016, 0.024, 0.035] {
                let base = law.respond(strain, committed)?;
                let h = 1e-7;
                let plus = law.respond(strain + h, committed)?;
                let minus = law.respond(strain - h, committed)?;
                let fd = (plus.stress - minus.stress) / (2.0 * h);
                assert!((fd - base.tangent).abs() <= 1e-6 * base.tangent.abs().max(1.0));
            }
        }
    }
    Ok(())
}

#[test]
fn uniaxial_conformance_kit_accepts_history_cycles() -> EResult<()> {
    for b in [0.0, 0.1, 0.8] {
        let law = SteelBilinearKinematic::new(200.0, 1.0, b)?;
        assert_eq!(law.state_len(), 2);
        assert_eq!(law.initial_state(), vec![0.0, 0.0]);
        assert_eq!(law.initial_tangent(), 200.0);
        let diagnostics = check_uniaxial_law(
            &law,
            &[
                0.0, 0.0025, 0.02, 0.0175, 0.005, -0.02, -0.0175, -0.005, 0.02,
            ],
            1e-7,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }
    Ok(())
}

#[test]
fn typed_factory_descriptor_and_parameters() -> EResult<()> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<SteelBilinearKinematic>();
    send_sync::<SteelBilinearKinematicFactory>();
    let factory: &dyn UniaxialFactory<Params = SteelBilinearKinematicParams> =
        &SteelBilinearKinematicFactory;
    let params = SteelBilinearKinematicParams {
        e: 200.0,
        fy: 1.0,
        b: 0.1,
    };
    assert_eq!(factory.descriptor(), STEEL_BILINEAR_KINEMATIC);
    assert_eq!(STEEL_BILINEAR_KINEMATIC.kind, "steel.bilinear_kinematic");
    assert_eq!(
        STEEL_BILINEAR_KINEMATIC.contract,
        ContractId::new("uniaxial", 1, 0)
    );
    assert_eq!(STEEL_BILINEAR_KINEMATIC.version, (0, 1, 0));
    let law = factory.build(params)?;
    assert_eq!(law.kind(), factory.descriptor().kind);
    assert_eq!(law.contract(), factory.descriptor().contract);
    close(law.respond(0.02, &law.initial_state())?.stress, 1.3);
    assert_eq!(
        SteelBilinearKinematic::new(200.0, 1.0, 0.1)?.params(),
        &params
    );
    code(
        factory.build(SteelBilinearKinematicParams { b: 1.0, ..params }),
        "ENG-E-0010",
    );
    Ok(())
}

#[test]
fn invalid_parameters_strain_and_history_use_catalog_diagnostics() -> EResult<()> {
    for bad in [0.0, -1.0] {
        code(SteelBilinearKinematic::new(bad, 1.0, 0.1), "ENG-E-0010");
        code(SteelBilinearKinematic::new(200.0, bad, 0.1), "ENG-E-0010");
    }
    for bad in [-0.1, 1.0, 2.0] {
        code(SteelBilinearKinematic::new(200.0, 1.0, bad), "ENG-E-0010");
    }
    let law = SteelBilinearKinematic::new(200.0, 1.0, 0.1)?;
    for bad in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        code(SteelBilinearKinematic::new(bad, 1.0, 0.1), "ENG-E-0008");
        code(SteelBilinearKinematic::new(200.0, bad, 0.1), "ENG-E-0008");
        code(SteelBilinearKinematic::new(200.0, 1.0, bad), "ENG-E-0008");
        code(law.respond(bad, &[0.0, 0.0]), "ENG-E-0008");
        code(law.respond(0.0, &[bad, 0.0]), "ENG-E-0008");
        code(law.respond(0.0, &[0.0, bad]), "ENG-E-0008");
    }
    for history in [vec![], vec![0.0], vec![0.0; 3], vec![Real::NAN]] {
        code(law.respond(0.0, &history), "ENG-E-0010");
    }
    Ok(())
}

#[test]
fn extreme_scales_avoid_unnecessary_hardening_and_yield_strain_overflow() -> EResult<()> {
    let law = SteelBilinearKinematic::new(1e300, Real::MAX, 0.9)?;
    let elastic = law.respond(1e-300, &law.initial_state())?;
    close(elastic.stress, 1.0);
    assert_eq!(elastic.tangent, 1e300);
    assert_eq!(elastic.trial_state, law.initial_state());
    // H y E+H no son representables; σ y la tangente sí lo son.
    let law = SteelBilinearKinematic::new(1e300, 1.0, 1.0 - Real::EPSILON)?;
    let plastic = law.respond(2e-300, &law.initial_state())?;
    close(plastic.stress, 1.0 + law.params().b);
    assert_eq!(plastic.tangent, law.params().b * 1e300);
    assert!(plastic.trial_state[0] > 0.0 && plastic.trial_state[0].is_finite());
    close(plastic.trial_state[1], law.params().b);
    // fy/E desbordaría aunque la respuesta elástica sea representable.
    let law = SteelBilinearKinematic::new(1e-300, Real::MAX, 0.1)?;
    close(law.respond(1e300, &law.initial_state())?.stress, 1.0);
    // No se pierde la meseta por cancelar σ_tr−EΔγ a escala 1e300.
    let law = SteelBilinearKinematic::new(1e300, 1.0, 0.0)?;
    assert_eq!(law.respond(1.0, &law.initial_state())?.stress, 1.0);
    Ok(())
}

#[test]
fn unrepresentable_differences_and_results_are_reported() -> EResult<()> {
    let law = SteelBilinearKinematic::new(1.0, 1.0, 0.1)?;
    code(law.respond(Real::MAX, &[-Real::MAX, 0.0]), "ENG-E-0008"); // ε−ε_p
    code(law.respond(Real::MAX, &[0.0, -Real::MAX]), "ENG-E-0008"); // ξ
    let law = SteelBilinearKinematic::new(Real::MAX, 1.0, 0.1)?;
    code(law.respond(2.0, &[0.0, 0.0]), "ENG-E-0008"); // σ_tr
    let law = SteelBilinearKinematic::new(1e-300, 1.0, 0.0)?;
    code(law.respond(0.0, &[0.0, -Real::MAX]), "ENG-E-0008"); // Δγ
    let law = SteelBilinearKinematic::new(1.0, 1.0, 0.0)?;
    code(law.respond(0.0, &[Real::MAX, Real::MAX]), "ENG-E-0008");
    Ok(())
}
