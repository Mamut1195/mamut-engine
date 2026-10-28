//! In-house analytical fixtures; in-memory tests without external oracles.

use engine_contracts::{conformance::check_uniaxial_law, UniaxialLaw};
use engine_core::{catalog, ContractId, EResult, Real};
use engine_laws::uniaxial::{
    MultilinearElastic, MultilinearElasticFactory, MultilinearElasticParams, MultilinearMode,
    UniaxialFactory, MULTILINEAR_ELASTIC,
};

const POINTS: &[(Real, Real)] = &[
    (-3.0, -10.0),
    (-1.0, -6.0),
    (0.0, 0.0),
    (1.0, 4.0),
    (3.0, 6.0),
];

struct Fixture {
    source: &'static str,
    citation: &'static str,
    // (ε, σ, tangent), with exact binary data.
    samples: &'static [(Real, Real, Real)],
}

const ASYMMETRIC: Fixture = Fixture {
    source: "Derivación propia: pendiente secante e interpolación/extrapolación afín.",
    citation: "provenance/ENG-MAT-002.md, Derivaciones propias y fixtures",
    samples: &[
        (-4.0, -12.0, 2.0),
        (-3.0, -10.0, 2.0),
        (-2.0, -8.0, 2.0),
        (-1.0, -6.0, 6.0),
        (-0.5, -3.0, 6.0),
        (0.0, 0.0, 4.0),
        (0.5, 2.0, 4.0),
        (1.0, 4.0, 1.0),
        (2.0, 5.0, 1.0),
        (3.0, 6.0, 1.0),
        (4.0, 7.0, 1.0),
    ],
};

fn law(mode: MultilinearMode) -> MultilinearElastic {
    MultilinearElastic::new(POINTS.to_vec(), mode).unwrap()
}

fn code<T>(result: EResult<T>, expected: &str) {
    match result {
        Err(error) => {
            assert_eq!(error.code, expected, "{error}");
            assert!(catalog::lookup(error.code).is_some());
            assert!(!error.message.is_empty());
        }
        Ok(_) => panic!("Expected {expected}"),
    }
}

#[test]
fn exact_values_in_every_segment_extrapolation_and_vertex() {
    assert!(!ASYMMETRIC.source.is_empty() && !ASYMMETRIC.citation.is_empty());
    let law = law(MultilinearMode::Both);
    for &(strain, stress, tangent) in ASYMMETRIC.samples {
        let r = law.respond(strain, &[]).unwrap();
        assert_eq!((r.stress, r.tangent), (stress, tangent), "ε={strain}");
        assert!(r.trial_state.is_empty());
    }
    assert_eq!(law.initial_tangent(), 4.0);
}

#[test]
fn central_derivative_away_from_vertices_including_gaps_and_extrapolation() {
    let h = 1e-6;
    for mode in [
        MultilinearMode::Both,
        MultilinearMode::CompressionOnly,
        MultilinearMode::TensionOnly,
    ] {
        let law = law(mode);
        for strain in [-4.0, -2.0, -0.5, 0.5, 2.0, 4.0] {
            let r = law.respond(strain, &[]).unwrap();
            let plus = law.respond(strain + h, &[]).unwrap().stress;
            let minus = law.respond(strain - h, &[]).unwrap().stress;
            let fd = (plus - minus) / (2.0 * h);
            if r.tangent == 0.0 {
                assert_eq!(fd, 0.0);
            } else {
                assert!((fd / r.tangent - 1.0).abs() < 1e-6);
            }
        }
    }
}

#[test]
fn reflected_points_are_symmetric_and_independent_points_are_asymmetric() {
    let symmetric = MultilinearElastic::new(
        vec![
            (-3.0, -6.0),
            (-1.0, -4.0),
            (0.0, 0.0),
            (1.0, 4.0),
            (3.0, 6.0),
        ],
        MultilinearMode::Both,
    )
    .unwrap();
    let asymmetric = law(MultilinearMode::Both);
    for strain in [0.25, 0.5, 2.0, 4.0] {
        let plus = symmetric.respond(strain, &[]).unwrap();
        let minus = symmetric.respond(-strain, &[]).unwrap();
        assert_eq!(plus.stress, -minus.stress);
        assert_eq!(plus.tangent, minus.tangent);
        assert_ne!(
            asymmetric.respond(strain, &[]).unwrap().stress,
            -asymmetric.respond(-strain, &[]).unwrap().stress,
        );
    }
}

#[test]
fn gaps_disable_stress_and_tangent_and_keep_the_active_origin_tangent() {
    for (mode, active_sign, initial) in [
        (MultilinearMode::CompressionOnly, -1.0, 6.0),
        (MultilinearMode::TensionOnly, 1.0, 4.0),
    ] {
        let law = law(mode);
        let full = self::law(MultilinearMode::Both);
        for strain in [Real::from_bits(1), 0.5, 2.0, 4.0, Real::MAX] {
            let r = law.respond(-active_sign * strain, &[]).unwrap();
            assert_eq!((r.stress, r.tangent), (0.0, 0.0));
        }
        for strain in [0.5, 2.0, 4.0] {
            assert_eq!(
                law.respond(active_sign * strain, &[]).unwrap(),
                full.respond(active_sign * strain, &[]).unwrap()
            );
        }
        for zero in [0.0, -0.0] {
            let r = law.respond(zero, &[]).unwrap();
            assert_eq!((r.stress, r.tangent), (0.0, initial));
        }
        assert_eq!(law.initial_tangent(), initial);
    }
}

#[test]
fn continuous_at_every_vertex_and_gap_origin() {
    let h = 1.0 / 1024.0;
    for mode in [
        MultilinearMode::Both,
        MultilinearMode::CompressionOnly,
        MultilinearMode::TensionOnly,
    ] {
        let law = law(mode);
        for &(strain, _) in POINTS {
            let center = law.respond(strain, &[]).unwrap();
            let left = law.respond(strain - h, &[]).unwrap();
            let right = law.respond(strain + h, &[]).unwrap();
            assert_eq!(left.stress + left.tangent * h, center.stress);
            assert_eq!(right.stress - right.tangent * h, center.stress);
        }
    }
}

#[test]
fn unilateral_tables_and_two_point_minimum_use_end_slopes() {
    for mode in [
        MultilinearMode::Both,
        MultilinearMode::CompressionOnly,
        MultilinearMode::TensionOnly,
    ] {
        for points in [vec![(0.0, 0.0), (1.0, 2.0)], vec![(-1.0, -2.0), (0.0, 0.0)]] {
            let law = MultilinearElastic::new(points, mode).unwrap();
            assert_eq!(law.initial_tangent(), 2.0);
            for strain in [-2.0, -0.5, 0.0, 0.5, 2.0] {
                let gap = (mode == MultilinearMode::CompressionOnly && strain > 0.0)
                    || (mode == MultilinearMode::TensionOnly && strain < 0.0);
                let r = law.respond(strain, &[]).unwrap();
                assert_eq!(r.stress, if gap { 0.0 } else { 2.0 * strain });
                assert_eq!(r.tangent, if gap { 0.0 } else { 2.0 });
            }
        }
    }
}

#[test]
fn plateaus_and_zero_curve_allow_extreme_strains() {
    let plateau = MultilinearElastic::new(
        vec![
            (-2.0, -4.0),
            (-1.0, -4.0),
            (0.0, 0.0),
            (1.0, 4.0),
            (2.0, 4.0),
        ],
        MultilinearMode::Both,
    )
    .unwrap();
    for (strain, stress) in [
        (-Real::MAX, -4.0),
        (-1.5, -4.0),
        (1.5, 4.0),
        (Real::MAX, 4.0),
    ] {
        let r = plateau.respond(strain, &[]).unwrap();
        assert_eq!((r.stress, r.tangent), (stress, 0.0));
    }
    let zero = MultilinearElastic::new(vec![(-Real::MAX, 0.0), (0.0, 0.0)], MultilinearMode::Both)
        .unwrap();
    for strain in [-Real::MAX, 0.0, Real::MAX] {
        let r = zero.respond(strain, &[]).unwrap();
        assert_eq!((r.stress, r.tangent), (0.0, 0.0));
    }
}

#[test]
fn rejects_bad_lengths_order_origin_and_negative_slopes() {
    for points in [
        vec![],
        vec![(0.0, 0.0)],
        vec![(0.0, 0.0), (0.0, 1.0)],
        vec![(0.0, 0.0), (-0.0, 0.0)],
        vec![(1.0, 1.0), (0.0, 0.0)],
        vec![(-1.0, -1.0), (1.0, 1.0)],
        vec![(0.0, 1.0), (1.0, 2.0)],
        vec![(0.0, 0.0), (1.0, -1.0)],
        vec![(-1.0, 1.0), (0.0, 0.0)],
        vec![(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)],
    ] {
        for mode in [
            MultilinearMode::Both,
            MultilinearMode::CompressionOnly,
            MultilinearMode::TensionOnly,
        ] {
            code(MultilinearElastic::new(points.clone(), mode), "ENG-E-0010");
        }
    }
}

#[test]
fn rejects_nonfinite_points_strains_and_nonempty_history_even_in_gap() {
    for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        for index in 0..POINTS.len() {
            for component in 0..2 {
                let mut points = POINTS.to_vec();
                if component == 0 {
                    points[index].0 = value;
                } else {
                    points[index].1 = value;
                }
                code(
                    MultilinearElastic::new(points, MultilinearMode::Both),
                    "ENG-E-0008",
                );
            }
        }
        for mode in [
            MultilinearMode::Both,
            MultilinearMode::CompressionOnly,
            MultilinearMode::TensionOnly,
        ] {
            code(law(mode).respond(value, &[]), "ENG-E-0008");
        }
    }
    for strain in [-1.0, 0.0, 1.0] {
        for state in [vec![0.0], vec![0.0, 1.0], vec![Real::NAN]] {
            code(
                law(MultilinearMode::CompressionOnly).respond(strain, &state),
                "ENG-E-0010",
            );
            code(
                law(MultilinearMode::TensionOnly).respond(strain, &state),
                "ENG-E-0010",
            );
        }
    }
}

#[test]
fn extreme_points_slopes_and_evaluation_return_values_or_diagnostics_without_panics() {
    let tiny = Real::from_bits(1);
    let identity = MultilinearElastic::new(
        vec![(-Real::MAX, -Real::MAX), (0.0, 0.0), (Real::MAX, Real::MAX)],
        MultilinearMode::Both,
    )
    .unwrap();
    for strain in [
        -Real::MAX,
        -Real::MAX / 2.0,
        -1.0,
        -tiny,
        0.0,
        tiny,
        1.0,
        Real::MAX / 2.0,
        Real::MAX,
    ] {
        let r = identity.respond(strain, &[]).unwrap();
        assert_eq!((r.stress, r.tangent), (strain, 1.0));
    }
    let small =
        MultilinearElastic::new(vec![(0.0, 0.0), (tiny, tiny)], MultilinearMode::Both).unwrap();
    assert_eq!(small.respond(tiny, &[]).unwrap().stress, tiny);
    for points in [
        vec![(0.0, 0.0), (tiny, 1.0)],
        vec![(0.0, 0.0), (0.5, Real::MAX)],
        vec![(-2.0, -Real::MAX), (-1.0, Real::MAX), (0.0, 0.0)],
    ] {
        code(
            MultilinearElastic::new(points, MultilinearMode::Both),
            "ENG-E-0008",
        );
    }
    code(
        MultilinearElastic::new(vec![(0.0, 0.0), (Real::MAX, tiny)], MultilinearMode::Both),
        "ENG-E-0010",
    );
    let product =
        MultilinearElastic::new(vec![(0.0, 0.0), (1.0, 2.0)], MultilinearMode::Both).unwrap();
    code(product.respond(Real::MAX, &[]), "ENG-E-0008");
    code(product.respond(-Real::MAX, &[]), "ENG-E-0008");
    let sum = MultilinearElastic::new(
        vec![(0.0, 0.0), (1.0, Real::MAX / 2.0), (2.0, Real::MAX)],
        MultilinearMode::Both,
    )
    .unwrap();
    code(sum.respond(3.0, &[]), "ENG-E-0008");
    let extrapolated_identity = MultilinearElastic::new(
        vec![
            (-Real::MAX, -Real::MAX),
            (-Real::MAX / 2.0, -Real::MAX / 2.0),
            (0.0, 0.0),
        ],
        MultilinearMode::Both,
    )
    .unwrap();
    assert_eq!(
        extrapolated_identity
            .respond(Real::MAX, &[])
            .unwrap()
            .stress,
        Real::MAX
    );
}

#[test]
fn stateless_repetition_and_factory_follow_existing_contract() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<MultilinearElastic>();
    send_sync::<MultilinearElasticFactory>();
    let params = MultilinearElasticParams {
        points: POINTS.to_vec(),
        mode: MultilinearMode::Both,
    };
    let factory: &dyn UniaxialFactory<Params = MultilinearElasticParams> =
        &MultilinearElasticFactory;
    assert_eq!(factory.descriptor(), MULTILINEAR_ELASTIC);
    assert_eq!(MULTILINEAR_ELASTIC.version, (0, 1, 0));
    let built = factory.build(params.clone()).unwrap();
    let law = law(params.mode);
    assert_eq!(law.params(), &params);
    assert_eq!(built.kind(), "multilinear_elastic");
    assert_eq!(built.contract(), ContractId::new("uniaxial", 1, 0));
    assert_eq!(built.state_len(), 0);
    assert!(built.initial_state().is_empty());
    let before = law.clone();
    let state = law.initial_state();
    let reference = law.respond(0.5, &state).unwrap();
    for strain in [0.5, -2.0, 4.0, -0.5, 0.5] {
        assert_eq!(
            law.respond(strain, &state).unwrap(),
            built.respond(strain, &state).unwrap()
        );
    }
    assert_eq!(law.respond(0.5, &reference.trial_state).unwrap(), reference);
    assert_eq!(law, before);
    assert!(state.is_empty());
    code(
        factory.build(MultilinearElasticParams {
            points: vec![],
            ..params
        }),
        "ENG-E-0010",
    );
}

#[test]
fn uniaxial_conformance_uses_differentiable_samples_and_checks_initial_tangent() {
    // The kit requires differentiable ε ± h. Excluding kinks does not alter the kit:
    // it separately checks respond(0).tangent == initial_tangent().
    let samples = [-4.0, -2.0, -0.5, 0.5, 2.0, 4.0, -0.5, 0.5];
    for mode in [
        MultilinearMode::Both,
        MultilinearMode::CompressionOnly,
        MultilinearMode::TensionOnly,
    ] {
        let diagnostics = check_uniaxial_law(&law(mode), &samples, 1e-6);
        assert!(diagnostics.is_empty(), "{mode:?}: {diagnostics:?}");
    }
    // On a straight line, the origin does have a two-sided derivative and can be sampled.
    let linear =
        MultilinearElastic::new(vec![(0.0, 0.0), (1.0, 2.0)], MultilinearMode::Both).unwrap();
    let diagnostics = check_uniaxial_law(&linear, &[0.0, -2.0, 0.5, 2.0], 1e-6);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
