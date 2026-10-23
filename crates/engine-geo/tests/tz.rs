//! Contratos y diagnósticos de ENG-GEO-004; fuente publicada en el harness.
#[path = "support/axial_benchmarks.rs"]
mod benchmarks;
use engine_core::EngineError;
use engine_geo::{qz, tz};

fn params() -> tz::ApiTzParams {
    tz::ApiTzParams {
        tmax_compression_pa: Some(50_000.0),
        tmax_tension_pa: Some(30_000.0),
        zpeak_m: Some(0.01),
        plateau_z_m: Some(0.03),
    }
}
fn tip_params() -> qz::ApiQzParams {
    qz::ApiQzParams {
        qp_n: Some(1_000_000.0),
        diameter_m: Some(1.0),
        plateau_z_m: Some(0.2),
    }
}
fn error<T: std::fmt::Debug>(result: Result<T, EngineError>, code: &str, name: &str) {
    let e = result.expect_err("Debe rechazar el dato");
    assert_eq!(e.code, code, "{e:?}");
    assert!(e.message.contains(name), "{e:?}");
}

#[test]
fn published_vertices_and_hand_interpolation() {
    assert_eq!(benchmarks::run_all().expect("Fixtures axiales"), 5);
}

#[test]
fn every_required_tz_parameter_is_missing_without_fallback() {
    for field in 0..4 {
        let mut p = params();
        let name = match field {
            0 => {
                p.tmax_compression_pa = None;
                "tmax_compression_pa"
            }
            1 => {
                p.tmax_tension_pa = None;
                "tmax_tension_pa"
            }
            2 => {
                p.zpeak_m = None;
                "zpeak_m"
            }
            _ => {
                p.plateau_z_m = None;
                "plateau_z_m"
            }
        };
        error(tz::api_sand(p), "ENG-E-0003", name);
        error(tz::api_clay(p, Some(0.8)), "ENG-E-0003", name);
    }
    error(tz::api_clay(params(), None), "ENG-E-0003", "tres/tmax");
}

#[test]
fn every_required_qz_parameter_is_missing_without_fallback() {
    for field in 0..3 {
        let mut p = tip_params();
        let name = match field {
            0 => {
                p.qp_n = None;
                "qp_n"
            }
            1 => {
                p.diameter_m = None;
                "diameter_m"
            }
            _ => {
                p.plateau_z_m = None;
                "plateau_z_m"
            }
        };
        error(qz::api(p), "ENG-E-0003", name);
    }
    error(
        tz::user(None, tz::UserTraction::Explicit),
        "ENG-E-0003",
        "points",
    );
    error(
        qz::user(None, tz::UserTraction::Symmetric),
        "ENG-E-0003",
        "points",
    );
}

#[test]
fn scalar_domains_and_nonfinite_parameters() {
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let code = if value.is_finite() {
            "ENG-E-0010"
        } else {
            "ENG-E-0008"
        };
        for field in 0..4 {
            let mut p = params();
            match field {
                0 => p.tmax_compression_pa = Some(value),
                1 => p.tmax_tension_pa = Some(value),
                2 => p.zpeak_m = Some(value),
                _ => p.plateau_z_m = Some(value),
            }
            error(tz::api_sand(p), code, "");
            error(tz::api_clay(p, Some(0.8)), code, "");
        }
        for field in 0..3 {
            let mut p = tip_params();
            match field {
                0 => p.qp_n = Some(value),
                1 => p.diameter_m = Some(value),
                _ => p.plateau_z_m = Some(value),
            }
            error(qz::api(p), code, "");
        }
        error(tz::api_clay(params(), Some(value)), code, "tres/tmax");
    }
    error(
        tz::api_clay(params(), Some(1.001)),
        "ENG-E-0010",
        "tres/tmax",
    );
}

#[test]
fn residual_range_is_not_imposed_and_plateau_is_explicit() {
    for ratio in [0.1, 0.69, 0.7, 0.8, 0.9, 0.91, 1.0] {
        let clay = tz::api_clay(params(), Some(ratio)).unwrap();
        assert_eq!(clay.at(0.03).unwrap(), 50_000.0 * ratio);
    }
    for z in [0.009, 0.01] {
        let mut p = params();
        p.plateau_z_m = Some(z);
        error(tz::api_sand(p), "ENG-E-0010", "plateau_z_m");
    }
    for z in [0.019, 0.02] {
        let mut p = params();
        p.plateau_z_m = Some(z);
        error(tz::api_clay(p, Some(0.8)), "ENG-E-0010", "plateau_z_m");
    }
    let mut p = tip_params();
    p.plateau_z_m = Some(0.1);
    error(qz::api(p), "ENG-E-0010", "plateau_z_m");
}

#[test]
fn scales_are_user_controlled_including_equal_tension() {
    let mut p = params();
    p.zpeak_m = Some(0.02);
    p.plateau_z_m = Some(0.1);
    p.tmax_tension_pa = Some(50_000.0);
    let sand = tz::api_sand(p).unwrap();
    assert_eq!(sand.at(0.0032).unwrap(), 15_000.0);
    assert_eq!(sand.at(-0.0032).unwrap(), -15_000.0);
    let mut p = tip_params();
    p.diameter_m = Some(2.0);
    p.plateau_z_m = Some(0.3);
    p.qp_n = Some(2_000_000.0);
    let tip = qz::api(p).unwrap();
    assert_eq!(tip.at(0.026).unwrap(), 1_000_000.0);
}

#[test]
fn user_tables_require_origin_order_finiteness_and_explicit_traction() {
    for points in [
        vec![],
        vec![(0.0, 0.0)],
        vec![(-1.0, -1.0), (1.0, 1.0)],
        vec![(0.0, 0.0), (0.0, 1.0)],
        vec![(1.0, 1.0), (0.0, 0.0)],
    ] {
        error(
            tz::user(Some(points.clone()), tz::UserTraction::Explicit),
            "ENG-E-0010",
            "",
        );
        error(
            qz::user(Some(points), tz::UserTraction::Explicit),
            "ENG-E-0010",
            "",
        );
    }
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for points in [vec![(0.0, 0.0), (v, 1.0)], vec![(0.0, 0.0), (1.0, v)]] {
            error(
                tz::user(Some(points.clone()), tz::UserTraction::Symmetric),
                "ENG-E-0008",
                "",
            );
            error(
                qz::user(Some(points), tz::UserTraction::Symmetric),
                "ENG-E-0008",
                "",
            );
        }
    }
    error(
        tz::user(
            Some(vec![(0.0, 0.0), (1.0, 1.0)]),
            tz::UserTraction::Explicit,
        ),
        "ENG-E-0010",
        "Explicit",
    );
    error(
        tz::user(
            Some(vec![(-1.0, -1.0), (0.0, 0.0)]),
            tz::UserTraction::Explicit,
        ),
        "ENG-E-0010",
        "Explicit",
    );
    error(
        tz::user(
            Some(vec![(-1.0, -1.0), (0.0, 0.0), (1.0, 1.0)]),
            tz::UserTraction::Symmetric,
        ),
        "ENG-E-0010",
        "Symmetric",
    );
}

#[test]
fn explicit_user_tables_reject_resistance_with_opposite_sign() {
    for points in [
        vec![(-2.0, -2.0), (-1.0, 1.0), (0.0, 0.0), (1.0, 2.0)],
        vec![(-1.0, -2.0), (0.0, 0.0), (1.0, -1.0), (2.0, 2.0)],
    ] {
        error(
            tz::user(Some(points.clone()), tz::UserTraction::Explicit),
            "ENG-E-0010",
            "Explicit option requires resistance ≤ 0 for z < 0 and ≥ 0 for z > 0",
        );
        error(
            qz::user(Some(points), tz::UserTraction::Explicit),
            "ENG-E-0010",
            "Explicit option requires resistance ≤ 0 for z < 0 and ≥ 0 for z > 0",
        );
    }
}

#[test]
fn explicit_user_tables_accept_correct_signs_including_zero() {
    let points = vec![
        (-2.0, -3.0),
        (-1.0, 0.0),
        (0.0, 0.0),
        (1.0, 0.0),
        (2.0, 4.0),
    ];
    let shaft = tz::user(Some(points.clone()), tz::UserTraction::Explicit).unwrap();
    let tip = qz::user(Some(points.clone()), tz::UserTraction::Explicit).unwrap();
    assert_eq!(shaft.points(), points);
    assert_eq!(tip.points(), points);
}

#[test]
fn user_symmetry_softening_and_extreme_extrapolation_are_explicit() {
    let curve = tz::user(
        Some(vec![(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)]),
        tz::UserTraction::Symmetric,
    )
    .unwrap();
    assert_eq!(
        curve.points(),
        &[
            (-2.0, -1.0),
            (-1.0, -2.0),
            (0.0, 0.0),
            (1.0, 2.0),
            (2.0, 1.0)
        ]
    );
    assert_eq!(curve.at(1.5).unwrap(), 1.5);
    assert_eq!(curve.at(-1.5).unwrap(), -1.5);
    assert_eq!(curve.at(3.0).unwrap(), 0.0);
    let gap = qz::user(
        Some(vec![(-1.0, 0.0), (0.0, 0.0), (1.0, 10.0)]),
        tz::UserTraction::Explicit,
    )
    .unwrap();
    assert_eq!(gap.at(-2.0).unwrap(), 0.0);
    assert_eq!(gap.at(2.0).unwrap(), 20.0);
}

#[test]
fn adapter_preserves_units_signs_gap_and_plateaus() {
    let sand = tz::api_sand(params()).unwrap();
    let points = sand.multilinear_points(Some(2.0)).unwrap();
    let expected: Vec<_> = sand
        .points()
        .iter()
        .rev()
        .map(|&(z, t)| (-z / 2.0, -t))
        .collect();
    assert_eq!(points, expected);
    assert_eq!(points[0], (-0.015, -50_000.0));
    assert_eq!(*points.last().unwrap(), (0.015, 30_000.0));
    let tip = qz::api(tip_params()).unwrap();
    let points = tip.multilinear_points(Some(2.0), Some(0.5)).unwrap();
    assert!(points.contains(&(-0.05, -2_000_000.0)));
    assert_eq!(*points.last().unwrap(), (0.1, 0.0));
    assert!(points
        .windows(2)
        .all(|p| p[1].0 > p[0].0 && p[1].1 >= p[0].1));
}

#[test]
fn adapter_rejects_softening_without_changing_the_curve() {
    let clay = tz::api_clay(params(), Some(0.8)).unwrap();
    error(
        clay.multilinear_points(Some(1.0)),
        "ENG-E-0010",
        "ENG-MAT-002",
    );
    assert_eq!(clay.at(0.015).unwrap(), 45_000.0);
    assert!(tz::api_clay(params(), Some(1.0))
        .unwrap()
        .multilinear_points(Some(1.0))
        .is_ok());
    for make in [tz::UserTraction::Explicit, tz::UserTraction::Symmetric] {
        let input = if make == tz::UserTraction::Explicit {
            vec![(-1.0, -1.0), (0.0, 0.0), (1.0, 2.0), (2.0, 1.0)]
        } else {
            vec![(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)]
        };
        error(
            tz::user(Some(input.clone()), make)
                .unwrap()
                .multilinear_points(Some(1.0)),
            "ENG-E-0010",
            "softening",
        );
        error(
            qz::user(Some(input), make)
                .unwrap()
                .multilinear_points(Some(1.0), Some(1.0)),
            "ENG-E-0010",
            "softening",
        );
    }
}

#[test]
fn adapter_requires_explicit_scales_and_checks_numerics() {
    let sand = tz::api_sand(params()).unwrap();
    let tip = qz::api(tip_params()).unwrap();
    error(
        sand.multilinear_points(None),
        "ENG-E-0003",
        "reference_length_m",
    );
    error(
        tip.multilinear_points(None, Some(1.0)),
        "ENG-E-0003",
        "reference_length_m",
    );
    error(
        tip.multilinear_points(Some(1.0), None),
        "ENG-E-0003",
        "reference_area_m2",
    );
    for value in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        let code = if value.is_finite() {
            "ENG-E-0010"
        } else {
            "ENG-E-0008"
        };
        error(
            sand.multilinear_points(Some(value)),
            code,
            "reference_length_m",
        );
        error(
            tip.multilinear_points(Some(1.0), Some(value)),
            code,
            "reference_area_m2",
        );
    }
    error(
        sand.multilinear_points(Some(f64::from_bits(1))),
        "ENG-E-0008",
        "",
    );
    error(
        tip.multilinear_points(Some(1.0), Some(f64::from_bits(1))),
        "ENG-E-0008",
        "",
    );
}

#[test]
fn adapter_rejects_resistance_reduced_to_zero_by_area() {
    // Tabla artificial: pendientes finitas; Q/A subdesborda a cero.
    let tip = qz::user(
        Some(vec![(-1.0, 0.0), (0.0, 0.0), (1.0, 1e-300)]),
        tz::UserTraction::Explicit,
    )
    .unwrap();
    error(
        tip.multilinear_points(Some(1.0), Some(1e300)),
        "ENG-E-0010",
        "Resistance reduced to zero when adapting units",
    );
}

#[test]
fn nonfinite_queries_overflow_and_underflow_are_typed() {
    let sand = tz::api_sand(params()).unwrap();
    let tip = qz::api(tip_params()).unwrap();
    for z in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        error(sand.at(z), "ENG-E-0008", "z");
        error(tip.at(z), "ENG-E-0008", "z");
    }
    assert_eq!(sand.at(f64::MAX).unwrap(), 50_000.0);
    assert_eq!(tip.at(-f64::MAX).unwrap(), 0.0);
    let mut p = params();
    p.zpeak_m = Some(f64::MAX);
    p.plateau_z_m = Some(f64::MAX);
    error(tz::api_clay(p, Some(0.8)), "ENG-E-0008", "zres_m");
    let mut p = params();
    p.tmax_compression_pa = Some(f64::from_bits(1));
    error(tz::api_sand(p), "ENG-E-0010", "product");
    let mut p = tip_params();
    p.qp_n = Some(f64::from_bits(1));
    error(qz::api(p), "ENG-E-0010", "product");
    let mut p = params();
    p.zpeak_m = Some(f64::from_bits(1));
    error(tz::api_sand(p), "ENG-E-0010", "product");
    let linear = tz::user(
        Some(vec![(0.0, 0.0), (1.0, 2.0)]),
        tz::UserTraction::Symmetric,
    )
    .unwrap();
    error(linear.at(f64::MAX), "ENG-E-0008", "increment");
    error(
        tz::user(
            Some(vec![(0.0, 0.0), (f64::from_bits(1), 1.0)]),
            tz::UserTraction::Symmetric,
        ),
        "ENG-E-0008",
        "slope",
    );
    error(
        tz::user(
            Some(vec![(0.0, 0.0), (f64::MAX, f64::from_bits(1))]),
            tz::UserTraction::Symmetric,
        ),
        "ENG-E-0010",
        "underflow",
    );
}
