//! Barra biapoyada con la deformación de control de la sección de fibras.
//!
//! Fuente: analítica. El axil y el momento son la ecuación (8) de
//! `docs/fichas/rectangle.fiber_bending.md`. El corte nulo y la fuerza nodal
//! son las ecuaciones (11) y (13) de `docs/fichas/frame2d.fiber_stations.md`.
//! Hormigón: parábola (A.9) transcrita en `docs/fichas/concrete.kent_park.md`;
//! página impresa: cita desconocida. Acero elástico: σ = Eε en
//! `crates/engine-laws/src/uniaxial/steel_bilinear.rs`.

use engine_contracts::conformance::check_element;
use engine_contracts::{Element, ElementCtx, ElementParams, MassKind, MemberLoad, SectionLaw};
use engine_core::math;
use engine_core::{Dim, EResult, Real};
use engine_elements_frame::{FiberStationBar, FIBER_STATION_COUNT};
use engine_laws::section::{RectangleFiberBending, RectangleFiberBendingParams, SteelBar};
use engine_laws::uniaxial::{KentParkParams, SteelBilinearKinematicParams};

const SOURCE: &str = "analítico";
const CITATION: &str =
    "Derivación propia, docs/fichas/frame2d.fiber_stations.md, ecuaciones (9)-(13). \
N y Mz: docs/fichas/rectangle.fiber_bending.md, ecuación (8). \
Hormigón: (A.9) en docs/fichas/concrete.kent_park.md; página impresa: cita desconocida. \
Acero: σ = Eε en crates/engine-laws/src/uniaxial/steel_bilinear.rs mientras |σ| < fy.";
const E_STEEL: Real = 200_000_000_000.0;
const LENGTH: Real = 4.0;

fn psi_in_pa() -> Real {
    4.4482216152605 / (0.0254 * 0.0254)
}

fn closed_section(fc: Real) -> (Real, Real) {
    let axial = -fc * 695_875.0 / 8_388_608.0 - E_STEEL / 524_288.0;
    let moment = fc * 81_875.0 / 16_777_216.0 + 9.0 * E_STEEL / 33_554_432.0;
    (axial, moment)
}

fn close(actual: Real, expected: Real) {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
    assert!(actual.is_finite() && expected.is_finite());
    let limit = 1e-9_f64.max(1e-12 * expected.abs());
    assert!(
        (actual - expected).abs() <= limit,
        "{actual} versus {expected}, limit={limit}; {SOURCE}: {CITATION}"
    );
}

fn control_section() -> RectangleFiberBending {
    let fc = 3000.0 * psi_in_pa();
    RectangleFiberBending::new(
        Dim::D2,
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
                fy: 500_000_000.0,
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
        },
    )
    .expect("sección de control")
}

fn bar() -> FiberStationBar {
    FiberStationBar::new(control_section())
}

fn ctx<'a>(coords: &'a [[Real; 3]], params: &'a ElementParams) -> ElementCtx<'a> {
    ElementCtx {
        dim: Dim::D2,
        coords,
        params,
    }
}

fn control_displacement() -> [Real; 6] {
    [
        0.0,
        0.0,
        LENGTH / 512.0,
        -LENGTH / 1024.0,
        0.0,
        -LENGTH / 512.0,
    ]
}

fn code<T>(result: EResult<T>, expected: &str) {
    assert_eq!(result.err().map(|error| error.code), Some(expected));
}

#[test]
fn control_station_matches_section_equilibrium() {
    let fc = 3000.0 * psi_in_pa();
    let (axial, moment) = closed_section(fc);
    let element = bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let context = ctx(&coords, &params);
    let displacement = control_displacement();
    let state = element.initial_state();
    assert_eq!(element.kind(), "frame2d.fiber_stations");
    assert_ne!(element.kind(), "frame2d.euler_elastic");
    assert_eq!(FIBER_STATION_COUNT, 3);
    assert_eq!(state.len(), 12);

    let response = element
        .respond(&context, &displacement, &state)
        .expect("respuesta");
    let expected_force = [-axial, 0.0, moment, axial, 0.0, -moment];
    for (actual, expected) in response.internal_force.iter().zip(expected_force) {
        close(*actual, expected);
    }
    assert_eq!(response.trial_state, state);

    let section = control_section();
    let direct = section
        .respond(&[-1.0 / 1024.0, 1.0 / 256.0], &section.initial_state())
        .expect("sección");
    close(direct.resultants[0], axial);
    close(direct.resultants[1], moment);

    let side = math::sqrt(3.0 / 5.0);
    let positions = [(1.0 - side) / 2.0, 0.5, (1.0 + side) / 2.0, 0.0, 1.0];
    let output = element
        .recover(&context, &displacement, &state, &[], &positions)
        .expect("recuperación");
    assert_eq!(output.end_forces_local, response.internal_force);
    for station in output.stations {
        close(station.forces_local[0], axial);
        close(station.forces_local[1], 0.0);
        close(station.forces_local[5], moment);
        assert_eq!(
            station.forces_local[0], direct.resultants[0],
            "el axil de la estación es el de la sección"
        );
        assert_eq!(station.forces_local[5], direct.resultants[1]);
    }
}

#[test]
fn element_conformance_on_the_smooth_control_displacement() {
    let element = bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let context = ctx(&coords, &params);
    let sample = control_displacement().to_vec();
    let issues = check_element(&element, &context, &[sample], 1e-6);
    assert!(
        issues.is_empty(),
        "{}",
        issues
            .iter()
            .map(|issue| issue.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    );
}

#[test]
fn zero_displacement_has_zero_force_and_repeat_is_pure() {
    let element = bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let context = ctx(&coords, &params);
    let state = element.initial_state();
    let first = element
        .respond(&context, &[0.0; 6], &state)
        .expect("origen");
    let second = element
        .respond(&context, &[0.0; 6], &state)
        .expect("repetición");
    assert_eq!(first.internal_force, vec![0.0; 6]);
    assert_eq!(first, second);
    assert!(element
        .mass(&context, MassKind::Lumped)
        .expect("masa")
        .is_none());
    assert!(element
        .geometric_stiffness(&context, &[0.0; 6], &state)
        .expect("rigidez geométrica")
        .is_none());
    assert_eq!(
        element
            .load_vector(
                &context,
                &MemberLoad {
                    force_per_length: [0.0; 3],
                    moment_per_length: [0.0; 3],
                },
            )
            .expect("carga nula"),
        vec![0.0; 6]
    );
}

#[test]
fn rejects_bad_history_geometry_stations_and_span_load() {
    let element = bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let context = ctx(&coords, &params);
    let state = element.initial_state();
    code(
        element.respond(&context, &[0.0; 6], &[0.0; 4]),
        "ENG-E-0010",
    );
    code(
        element.respond(&context, &[0.0, 0.0, 0.0, f64::NAN, 0.0, 0.0], &state),
        "ENG-E-0008",
    );
    let raised = [[0.0; 3], [LENGTH, 0.0, 0.01]];
    code(
        element.respond(&ctx(&raised, &params), &[0.0; 6], &state),
        "ENG-E-0010",
    );
    let space = ElementCtx {
        dim: Dim::D3,
        coords: &coords,
        params: &params,
    };
    code(element.respond(&space, &[0.0; 6], &state), "ENG-E-0010");
    let short = [[0.0; 3], [1e-15, 0.0, 0.0]];
    code(
        element.respond(&ctx(&short, &params), &[0.0; 6], &state),
        "ENG-E-0005",
    );
    code(
        element.recover(&context, &[0.0; 6], &state, &[], &[1.1]),
        "ENG-E-0010",
    );
    code(
        element.recover(&context, &[0.0; 6], &state, &[], &[-0.01]),
        "ENG-E-0010",
    );
    let load = MemberLoad {
        force_per_length: [0.0, -1.0, 0.0],
        moment_per_length: [0.0; 3],
    };
    code(element.load_vector(&context, &load), "ENG-E-0013");
    code(
        element.recover(&context, &[0.0; 6], &state, &[load], &[0.5]),
        "ENG-E-0013",
    );
}
