//! Hand sums for `rectangle.fiber_bending`.
//!
//! Source: analytical. The control axial force and moment are equations (8)
//! and the tangent is equation (9) of `docs/fichas/rectangle.fiber_bending.md`.
//! Nonzero steel history is equations (13) and (14). Cracked bending is
//! equations (17) and (18). Concrete uses (A.9) as transcribed in
//! `docs/fichas/concrete.kent_park.md`; printed page: cita desconocida.
//! Positive concrete strain uses the zero stress of that sheet. Steel uses
//! the return in `crates/engine-laws/src/uniaxial/steel_bilinear.rs`;
//! textbook page: cita desconocida.

use engine_contracts::conformance::check_section_law;
use engine_contracts::SectionLaw;
use engine_core::{ContractId, Diagnostic, Dim, EResult, Real};
use engine_laws::section::{
    RectangleFiberBending, RectangleFiberBendingFactory, RectangleFiberBendingParams,
    SectionFactory, SteelBar, RECTANGLE_FIBER_BENDING,
};
use engine_laws::uniaxial::{KentParkParams, SteelBilinearKinematicParams};

const E_STEEL: Real = 200_000_000_000.0;
const FY: Real = 500_000_000.0;

/// 1 psi in pascals. Same unit factor as `concrete.kent_park`, not a fitted constant.
fn psi_in_pa() -> Real {
    4.4482216152605 / (0.0254 * 0.0254)
}

struct Fixture {
    source: &'static str,
    citation: &'static str,
    absolute: Real,
    relative: Real,
}

fn hand_fixture() -> Fixture {
    Fixture {
        source: "analytical",
        citation: "Own derivation, docs/fichas/rectangle.fiber_bending.md, equations (1)-(9), (13), (14), (17) and (18). \
Concrete ratio from (A.9) as transcribed in docs/fichas/concrete.kent_park.md; printed page: cita desconocida. \
Positive concrete strain uses the zero stress of that sheet. \
Steel return from crates/engine-laws/src/uniaxial/steel_bilinear.rs; textbook page: cita desconocida.",
        absolute: 1e-9,
        relative: 1e-12,
    }
}

fn control_params(fc: Real) -> RectangleFiberBendingParams {
    RectangleFiberBendingParams {
        width: 0.25,
        height: 0.5,
        strip_count: 4,
        concrete: KentParkParams {
            fc,
            rho_s: 0.0,
            b_double_prime: 0.2,
            s_h: 0.1,
        },
        steel: SteelBilinearKinematicParams {
            e: E_STEEL,
            fy: FY,
            b: 0.0,
        },
        bars: vec![
            SteelBar {
                area: 1.0 / 1024.0,
                y: -3.0 / 16.0,
            },
            SteelBar {
                area: 1.0 / 1024.0,
                y: 3.0 / 16.0,
            },
        ],
    }
}

fn control_section(fc: Real) -> EResult<RectangleFiberBending> {
    RectangleFiberBending::new(Dim::D2, control_params(fc))
}

/// Closed forms (8) and (9). N, Mz, ∂N/∂ε0, ∂N/∂κz, ∂Mz/∂κz.
fn closed_forms(fc: Real) -> (Real, Real, Real, Real, Real) {
    let axial = -fc * 695_875.0 / 8_388_608.0 - E_STEEL / 524_288.0;
    let moment = fc * 81_875.0 / 16_777_216.0 + 9.0 * E_STEEL / 33_554_432.0;
    let d_axial = fc * 16_375.0 / 256.0 + E_STEEL / 512.0;
    let d_couple = fc * 78_125.0 / 16_384.0;
    let d_bending = fc * 81_875.0 / 65_536.0 + 9.0 * E_STEEL / 131_072.0;
    (axial, moment, d_axial, d_couple, d_bending)
}

/// Equations (13) and (14). N, Mz, ∂N/∂ε0, ∂N/∂κz, ∂Mz/∂κz.
fn history_closed(fc: Real) -> (Real, Real, Real, Real, Real) {
    let axial = -fc * 695_875.0 / 8_388_608.0 - E_STEEL / 8_388_608.0 - FY / 1_024.0;
    let moment = fc * 81_875.0 / 16_777_216.0 - 3.0 * E_STEEL / 134_217_728.0 + 3.0 * FY / 16_384.0;
    let d_axial = fc * 16_375.0 / 256.0 + E_STEEL / 1_024.0;
    let d_couple = fc * 78_125.0 / 16_384.0 + 3.0 * E_STEEL / 16_384.0;
    let d_bending = fc * 81_875.0 / 65_536.0 + 9.0 * E_STEEL / 262_144.0;
    (axial, moment, d_axial, d_couple, d_bending)
}

/// Equations (17) and (18). N, Mz, ∂N/∂ε0, ∂N/∂κz, ∂Mz/∂κz.
fn cracking_closed(fc: Real) -> (Real, Real, Real, Real, Real) {
    let axial = -fc * 433_875.0 / 16_777_216.0;
    let moment = fc * 530_625.0 / 134_217_728.0 + 9.0 * E_STEEL / 33_554_432.0;
    let d_axial = fc * 48_375.0 / 1_024.0 + E_STEEL / 512.0;
    let d_couple = -fc * 177_875.0 / 32_768.0;
    let d_bending = fc * 210_625.0 / 262_144.0 + 9.0 * E_STEEL / 131_072.0;
    (axial, moment, d_axial, d_couple, d_bending)
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
fn control_strains_match_the_parabola_fractions() {
    let stations = [
        (-3.0 / 16.0, 1.0),
        (-1.0 / 16.0, 3.0),
        (1.0 / 16.0, 5.0),
        (3.0 / 16.0, 7.0),
    ];
    let axial_strain = -1.0 / 1024.0;
    let curvature = 1.0 / 256.0;
    for (y, k) in stations {
        let strain = axial_strain - curvature * y;
        let ratio = (-strain) / 0.002;
        let parabola = ratio * (2.0 - ratio);
        let numerator = 125.0 * k * (2048.0 - 125.0 * k);
        assert_eq!(parabola, numerator / 1_048_576.0);
        let stress = E_STEEL * strain;
        assert!(stress.abs() < FY);
    }
}

#[test]
fn native_analytical_benchmark() -> EResult<()> {
    let fixture = hand_fixture();
    assert_eq!(fixture.source, "analytical");
    assert!(fixture.citation.contains("cita desconocida"));
    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc)?;
    assert_eq!(section.kind(), "rectangle.fiber_bending");
    assert_eq!(section.dim(), Dim::D2);
    assert_eq!(section.state_len(), 4);
    assert_eq!(section.contract(), ContractId::new("section", 1, 0));
    let committed = section.initial_state();
    assert_eq!(committed, vec![0.0; 4]);
    let strain = vec![-1.0 / 1024.0, 1.0 / 256.0];
    let response = section.respond(&strain, &committed)?;
    let (axial, moment, d_axial, d_couple, d_bending) = closed_forms(fc);
    close(response.resultants[0], axial);
    close(response.resultants[1], moment);
    let tangent = response.tangent.values();
    close(tangent[0], d_axial);
    close(tangent[1], d_couple);
    close(tangent[2], d_couple);
    close(tangent[3], d_bending);
    assert_eq!(response.trial_state, committed);
    let repeated = section.respond(&strain, &committed)?;
    assert_eq!(repeated.resultants, response.resultants);
    assert_eq!(committed, vec![0.0; 4]);
    Ok(())
}

#[test]
fn constant_axial_returns_the_same_hand_point() -> EResult<()> {
    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc)?;
    let (axial, moment, _, _, _) = closed_forms(fc);
    let point = section.moment_at_constant_axial(
        axial,
        1.0 / 256.0,
        -5.0 / 4096.0,
        &section.initial_state(),
        1e-6,
        8,
    )?;
    assert!((point.axial_force - axial).abs() <= 1e-6);
    assert!((point.axial_strain - (-1.0 / 1024.0)).abs() <= 1e-9);
    close(point.moment, moment);
    assert_eq!(point.curvature, 1.0 / 256.0);
    assert_eq!(point.trial_state, vec![0.0; 4]);
    Ok(())
}

#[test]
fn tension_limit_and_origin_are_exact() -> EResult<()> {
    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc)?;
    let zeros = section.initial_state();
    let origin = section.respond(&[0.0, 0.0], &zeros)?;
    assert_eq!(origin.resultants, vec![0.0, 0.0]);
    let tension = section.respond(&[1.0 / 1024.0, 0.0], &zeros)?;
    assert_eq!(tension.resultants[0], 2.0 * E_STEEL / 1_048_576.0);
    assert_eq!(tension.resultants[1], 0.0);
    Ok(())
}

/// Source: analytical. Equations (13) and (14) of
/// `docs/fichas/rectangle.fiber_bending.md`. Steel textbook page: cita desconocida.
#[test]
fn nonzero_steel_history_matches_the_hand_sums() -> EResult<()> {
    let fixture = hand_fixture();
    assert_eq!(fixture.source, "analytical");
    assert!(fixture.citation.contains("cita desconocida"));
    assert!(fixture.citation.contains("(13)"));

    let strain_a = -1.0 / 4_096.0;
    let elastic_a = strain_a - (-1.0 / 8_192.0);
    assert_eq!(elastic_a, -1.0 / 8_192.0);
    let stress_a = E_STEEL * elastic_a;
    assert_eq!(stress_a, -E_STEEL / 8_192.0);
    assert!(stress_a.abs() < FY);

    let strain_b = -7.0 / 4_096.0;
    let elastic_b = strain_b - (1.0 / 128.0);
    assert_eq!(elastic_b, -39.0 / 4_096.0);
    let stress_trial = E_STEEL * elastic_b;
    assert_eq!(stress_trial, -1_904_296_875.0);
    assert!(stress_trial.abs() > FY);

    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc)?;
    let committed = vec![-1.0 / 8_192.0, 0.0, 1.0 / 128.0, 0.0];
    let response = section.respond(&[-1.0 / 1_024.0, 1.0 / 256.0], &committed)?;
    let (axial, moment, d_axial, d_couple, d_bending) = history_closed(fc);
    close(response.resultants[0], axial);
    close(response.resultants[1], moment);
    let tangent = response.tangent.values();
    close(tangent[0], d_axial);
    close(tangent[1], d_couple);
    close(tangent[2], d_couple);
    close(tangent[3], d_bending);
    assert_eq!(response.trial_state.len(), 4);
    assert_eq!(response.trial_state[0], committed[0]);
    assert_eq!(response.trial_state[1], 0.0);
    let plastic = 81.0 / 102_400.0;
    let gap = (response.trial_state[2] - plastic).abs();
    assert!(
        gap <= 1e-15,
        "{gap} above 1e-15; {source}: {citation}",
        source = fixture.source,
        citation = fixture.citation
    );
    assert_eq!(response.trial_state[3], 0.0);
    assert_eq!(committed, vec![-1.0 / 8_192.0, 0.0, 1.0 / 128.0, 0.0]);
    Ok(())
}

/// Source: analytical. Equations (17) and (18) of
/// `docs/fichas/rectangle.fiber_bending.md`. Printed page of the concrete
/// report: cita desconocida.
#[test]
fn cracked_bending_matches_the_hand_sums() -> EResult<()> {
    let fixture = hand_fixture();
    assert_eq!(fixture.source, "analytical");
    assert!(fixture.citation.contains("cita desconocida"));
    assert!(fixture.citation.contains("(17)"));

    let curvature = 1.0 / 256.0;
    let stations = [
        (-3.0 / 16.0, 3.0 / 4_096.0, true),
        (-1.0 / 16.0, 1.0 / 4_096.0, true),
        (1.0 / 16.0, -1.0 / 4_096.0, false),
        (3.0 / 16.0, -3.0 / 4_096.0, false),
    ];
    for (y, expected, tension) in stations {
        let strain = -curvature * y;
        assert_eq!(strain, expected);
        assert_eq!(strain > 0.0, tension);
    }
    let bar_stress = 3.0 * E_STEEL / 4_096.0;
    assert_eq!(bar_stress, 146_484_375.0);
    assert!(bar_stress < FY);

    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc)?;
    let committed = section.initial_state();
    let response = section.respond(&[0.0, curvature], &committed)?;
    let (axial, moment, d_axial, d_couple, d_bending) = cracking_closed(fc);
    close(response.resultants[0], axial);
    close(response.resultants[1], moment);
    let tangent = response.tangent.values();
    close(tangent[0], d_axial);
    close(tangent[1], d_couple);
    close(tangent[2], d_couple);
    close(tangent[3], d_bending);
    assert_eq!(response.trial_state, vec![0.0; 4]);
    assert_eq!(committed, vec![0.0; 4]);
    Ok(())
}

#[test]
fn section_conformance_on_the_control_strain() -> EResult<()> {
    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc)?;
    let diagnostics = check_section_law(&section, &[vec![-1.0 / 1024.0, 1.0 / 256.0]], 1e-6);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    Ok(())
}

#[test]
fn factory_descriptor_and_purity() -> EResult<()> {
    assert_eq!(RECTANGLE_FIBER_BENDING.kind, "rectangle.fiber_bending");
    assert_eq!(
        RECTANGLE_FIBER_BENDING.contract,
        ContractId::new("section", 1, 0)
    );
    assert_eq!(RECTANGLE_FIBER_BENDING.version, (0, 1, 0));
    let fc = 3000.0 * psi_in_pa();
    let factory: &dyn SectionFactory<Params = RectangleFiberBendingParams> =
        &RectangleFiberBendingFactory;
    assert_eq!(factory.descriptor(), RECTANGLE_FIBER_BENDING);
    let law = factory.build(Dim::D2, control_params(fc))?;
    assert_eq!(law.kind(), RECTANGLE_FIBER_BENDING.kind);
    let section = control_section(fc)?;
    let original = section.clone();
    let strain = vec![-1.0 / 1024.0, 1.0 / 256.0];
    let committed = section.initial_state();
    let first = section.respond(&strain, &committed)?;
    assert_eq!(section, original);
    assert_eq!(committed, vec![0.0; 4]);
    let accepted = section.respond(&strain, &first.trial_state)?;
    assert_eq!(accepted.resultants, first.resultants);
    code(
        RectangleFiberBending::new(Dim::D3, control_params(fc)).map(|_| ()),
        "ENG-E-0010",
    );
    Ok(())
}

#[test]
fn laws_are_send_sync_without_spawning_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<RectangleFiberBending>();
    assert_send_sync::<RectangleFiberBendingFactory>();
    assert_send_sync::<Box<dyn SectionLaw>>();
}

#[test]
fn rejected_parameters_and_histories() {
    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc).unwrap();
    code(section.respond(&[0.0], &[0.0; 4]), "ENG-E-0010");
    code(section.respond(&[0.0, 0.0], &[0.0]), "ENG-E-0010");
    code(
        section.respond(&[Real::INFINITY, 0.0], &[0.0; 4]),
        "ENG-E-0008",
    );
    code(
        section.respond(&[-1.0 / 1024.0, 1.0 / 256.0], &[Real::NAN, 0.0, 0.0, 0.0]),
        "ENG-E-0008",
    );

    let mut bad = control_params(fc);
    bad.strip_count = 0;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );
    let mut bad = control_params(fc);
    bad.strip_count = 65;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );
    let mut bad = control_params(fc);
    bad.width = 0.0;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );
    let mut bad = control_params(fc);
    bad.height = Real::NAN;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0008",
    );
    let mut bad = control_params(fc);
    bad.bars.clear();
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );
    let mut bad = control_params(fc);
    bad.bars[0].y = 0.26;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );
    let mut bad = control_params(fc);
    bad.bars[0].area = -1.0;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );
    let mut bad = control_params(fc);
    bad.concrete.fc = -1.0;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );
    let mut bad = control_params(fc);
    bad.steel.b = 1.0;
    code(
        RectangleFiberBending::new(Dim::D2, bad).map(|_| ()),
        "ENG-E-0010",
    );

    let mut edge = control_params(fc);
    edge.bars[0].y = 0.25;
    assert!(RectangleFiberBending::new(Dim::D2, edge).is_ok());
}

#[test]
fn constant_axial_errors() -> EResult<()> {
    let fc = 3000.0 * psi_in_pa();
    let section = control_section(fc)?;
    let (axial, _, _, _, _) = closed_forms(fc);
    code(
        section
            .moment_at_constant_axial(axial, 1.0 / 256.0, -5.0 / 4096.0, &[0.0; 4], 1e-6, 1)
            .map(|_| ()),
        "ENG-E-0007",
    );
    code(
        section
            .moment_at_constant_axial(axial, 1.0 / 256.0, -5.0 / 4096.0, &[0.0; 4], 0.0, 4)
            .map(|_| ()),
        "ENG-E-0010",
    );
    code(
        section
            .moment_at_constant_axial(axial, 1.0 / 256.0, -5.0 / 4096.0, &[0.0; 4], 1e-6, 0)
            .map(|_| ()),
        "ENG-E-0010",
    );
    code(
        section
            .moment_at_constant_axial(Real::NAN, 0.0, 0.0, &[0.0; 4], 1e-6, 4)
            .map(|_| ()),
        "ENG-E-0008",
    );

    let flat = RectangleFiberBending::new(
        Dim::D2,
        RectangleFiberBendingParams {
            width: 0.25,
            height: 0.5,
            strip_count: 1,
            concrete: KentParkParams {
                fc,
                rho_s: 0.0,
                b_double_prime: 0.2,
                s_h: 0.1,
            },
            steel: SteelBilinearKinematicParams {
                e: E_STEEL,
                fy: FY,
                b: 0.0,
            },
            bars: vec![SteelBar {
                area: 0.001,
                y: 0.0,
            }],
        },
    )?;
    code(
        flat.moment_at_constant_axial(0.0, 0.0, 0.01, &flat.initial_state(), 1e-6, 4)
            .map(|_| ()),
        "ENG-E-0006",
    );
    Ok(())
}
