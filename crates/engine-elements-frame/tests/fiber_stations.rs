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
use engine_elements_frame::{
    EulerFrame2D, FiberStationBar, MomentReleases, FIBER_STATIONS_EXPERIMENTAL,
    FIBER_STATION_COUNT, MAX_RECOVER_STATIONS,
};
use engine_laws::section::{
    ElasticFrame, ElasticFrameParams, RectangleFiberBending, RectangleFiberBendingParams, SteelBar,
};
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
    section_with_top(1.0 / 1024.0)
}

fn section_with_top(top: Real) -> RectangleFiberBending {
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
                    area: top,
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

fn published_experimental(flag: bool) -> bool {
    flag
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
    assert_eq!(MAX_RECOVER_STATIONS, 64);
    assert!(published_experimental(FIBER_STATIONS_EXPERIMENTAL));
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
        .respond(&[-1.0 / 1024.0, -1.0 / 256.0], &section.initial_state())
        .expect("sección");
    close(direct.resultants[0], axial);
    close(direct.resultants[1], -moment);

    let side = math::sqrt(3.0 / 5.0);
    let positions = [(1.0 - side) / 2.0, 0.5, (1.0 + side) / 2.0, 0.0, 1.0];
    let output = element
        .recover(&context, &displacement, &state, &[], &positions)
        .expect("recuperación");
    assert_eq!(output.end_forces_local, response.internal_force);
    for station in output.stations {
        close(station.forces_local[0], axial);
        close(station.forces_local[1], 0.0);
        close(station.forces_local[5], -moment);
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
    let mut many = vec![0.5; MAX_RECOVER_STATIONS + 1];
    code(
        element.recover(&context, &[0.0; 6], &state, &[], &many),
        "ENG-E-0010",
    );
    many.pop();
    assert_eq!(
        element
            .recover(&context, &[0.0; 6], &state, &[], &many)
            .expect("tope incluido")
            .stations
            .len(),
        MAX_RECOVER_STATIONS
    );
}

fn asymmetric_closed(fc: Real) -> (Real, Real) {
    let axial = -fc * 695_875.0 / 8_388_608.0 - 11.0 * E_STEEL / 4_194_304.0;
    let moment = -fc * 81_875.0 / 16_777_216.0 - 9.0 * E_STEEL / 67_108_864.0;
    (axial, moment)
}

fn hand_resultants(
    fc: Real,
    axial_strain: Real,
    curvature: Real,
    top: Real,
    plastic_top: Real,
) -> (Real, Real) {
    let strip = 1.0 / 32.0;
    let mut axial = 0.0;
    let mut moment = 0.0;
    for y in [-3.0 / 16.0, -1.0 / 16.0, 1.0 / 16.0, 3.0 / 16.0] {
        let strain = axial_strain - curvature * y;
        let ratio = if strain > 0.0 {
            0.0
        } else {
            let x = (-strain) / 0.002;
            x * (2.0 - x)
        };
        let force = (-fc * ratio) * strip;
        axial += force;
        moment += (-y) * force;
    }
    for (area, y, plastic) in [
        (1.0 / 1024.0, -3.0 / 16.0, 0.0),
        (top, 3.0 / 16.0, plastic_top),
    ] {
        let strain = axial_strain - curvature * y;
        let force = (E_STEEL * (strain - plastic)) * area;
        axial += force;
        moment += (-y) * force;
    }
    (axial, moment)
}

fn boole_linear(fc: Real) -> [Real; 6] {
    let weights = [7.0, 32.0, 12.0, 32.0, 7.0];
    let theta = 1.0 / 256.0;
    let mut force = [0.0; 6];
    for (index, weight) in weights.iter().enumerate() {
        let xi = index as Real / 4.0;
        let curvature = (-4.0 + 6.0 * xi) / LENGTH * theta;
        let (axial, moment) = hand_resultants(fc, -1.0 / 1024.0, curvature, 1.0 / 1024.0, 0.0);
        let axial_row = [-1.0 / LENGTH, 0.0, 0.0, 1.0 / LENGTH, 0.0, 0.0];
        let curvature_row = [
            0.0,
            (-6.0 + 12.0 * xi) / (LENGTH * LENGTH),
            (-4.0 + 6.0 * xi) / LENGTH,
            0.0,
            (6.0 - 12.0 * xi) / (LENGTH * LENGTH),
            (-2.0 + 6.0 * xi) / LENGTH,
        ];
        for dof in 0..6 {
            force[dof] += weight * (axial_row[dof] * axial + curvature_row[dof] * moment);
        }
    }
    for value in &mut force {
        *value *= LENGTH / 90.0;
    }
    force
}

fn gauss_linear(fc: Real, plastic_top_left: Real) -> [Real; 6] {
    let side = math::sqrt(3.0 / 5.0);
    let nodes = [-side, 0.0, side];
    let end = 5.0 / 9.0;
    let weights = [end, 2.0 - end - end, end];
    let theta = 1.0 / 256.0;
    let mut force = [0.0; 6];
    for (station, (&node, weight)) in nodes.iter().zip(weights).enumerate() {
        let xi = (node + 1.0) / 2.0;
        let curvature = (-4.0 + 6.0 * xi) / LENGTH * theta;
        let plastic = if station == 0 { plastic_top_left } else { 0.0 };
        let (axial, moment) = hand_resultants(fc, -1.0 / 1024.0, curvature, 1.0 / 1024.0, plastic);
        let span = weight * LENGTH / 2.0;
        let axial_row = [-1.0 / LENGTH, 0.0, 0.0, 1.0 / LENGTH, 0.0, 0.0];
        let curvature_row = [
            0.0,
            (-6.0 + 12.0 * xi) / (LENGTH * LENGTH),
            (-4.0 + 6.0 * xi) / LENGTH,
            0.0,
            (6.0 - 12.0 * xi) / (LENGTH * LENGTH),
            (-2.0 + 6.0 * xi) / LENGTH,
        ];
        for dof in 0..6 {
            force[dof] += span * (axial_row[dof] * axial + curvature_row[dof] * moment);
        }
    }
    force
}

fn close_integrated(actual: Real, expected: Real) {
    assert!(
        (actual - expected).abs() <= 1e-6,
        "{actual} versus {expected}; integral propia de la ficha, ecuaciones (8) y (15)"
    );
}

#[test]
fn asymmetric_elastic_bar_matches_hand_sum_and_elastic_frame_sign() {
    let fc = 3000.0 * psi_in_pa();
    let (axial, moment) = asymmetric_closed(fc);
    let element = FiberStationBar::new(section_with_top(4.0 / 1024.0));
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let context = ctx(&coords, &params);
    let displacement = control_displacement();
    let response = element
        .respond(&context, &displacement, &element.initial_state())
        .expect("respuesta asimétrica");
    let expected_force = [-axial, 0.0, -moment, axial, 0.0, moment];
    for (actual, expected) in response.internal_force.iter().zip(expected_force) {
        close(*actual, expected);
    }
    let output = element
        .recover(
            &context,
            &displacement,
            &element.initial_state(),
            &[],
            &[0.5],
        )
        .expect("estación");
    close(output.stations[0].forces_local[0], axial);
    close(output.stations[0].forces_local[1], 0.0);
    close(output.stations[0].forces_local[5], moment);
    close(
        output.stations[0].forces_local[5],
        -output.end_forces_local[2],
    );

    let young = 200_000_000_000.0;
    let area = 0.125;
    let inertia = 1.0 / 256.0;
    let curvature = -1.0 / 256.0;
    let axial_strain = -1.0 / 1024.0;
    let frame = ElasticFrame::new(
        Dim::D2,
        ElasticFrameParams {
            e: young,
            g: 80.0,
            area,
            iy: 1.0,
            iz: inertia,
            j: 1.0,
            ay: None,
            az: None,
        },
    )
    .expect("sección elástica");
    let elastic = frame
        .respond(&[axial_strain, curvature], &[])
        .expect("ElasticFrame");
    close(elastic.resultants[0], young * area * axial_strain);
    close(elastic.resultants[1], young * inertia * curvature);
    let euler = EulerFrame2D::new(young, area, inertia, 0.0, MomentReleases::default())
        .expect("barra de Euler");
    let euler_out = euler
        .recover(&context, &displacement, &[], &[], &[0.5])
        .expect("recuperación de Euler");
    close(
        euler_out.stations[0].forces_local[5],
        young * inertia * curvature,
    );
    close(
        euler_out.stations[0].forces_local[5],
        -euler_out.end_forces_local[2],
    );
    close(
        euler_out.stations[0].forces_local[0],
        young * area * axial_strain,
    );
    assert!(output.stations[0].forces_local[5] < 0.0);
    assert!(euler_out.stations[0].forces_local[5] < 0.0);
}

#[test]
fn linear_curvature_matches_boole_and_one_station_history() {
    let fc = 3000.0 * psi_in_pa();
    let element = bar();
    let params = ElementParams::default();
    let coords = [[0.0; 3], [LENGTH, 0.0, 0.0]];
    let context = ctx(&coords, &params);
    let displacement = [0.0, 0.0, 1.0 / 256.0, -1.0 / 256.0, 0.0, 0.0];
    let response = element
        .respond(&context, &displacement, &element.initial_state())
        .expect("curvatura lineal");
    for (actual, expected) in response.internal_force.iter().zip(boole_linear(fc)) {
        close_integrated(*actual, expected);
    }
    let mut history = element.initial_state();
    history[2] = 1.0 / 10_000.0;
    let with_history = element
        .respond(&context, &displacement, &history)
        .expect("historia en la estación izquierda");
    for (actual, expected) in with_history
        .internal_force
        .iter()
        .zip(gauss_linear(fc, 1.0 / 10_000.0))
    {
        close_integrated(*actual, expected);
    }
}
