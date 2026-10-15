use engine_geo::{
    consolidation::{
        ConsolidationLayer, ConsolidationOptions, ConsolidationProfile, Drainage, LoadPoint,
    },
    Parameter, Provenance,
};
fn p(v: f64) -> Parameter {
    Parameter::new(v, Provenance::Measured)
}
fn layer(n: usize) -> ConsolidationLayer {
    ConsolidationLayer {
        thickness_m: p(1.0),
        cv_m2_s: Some(p(1.0)),
        mv_per_kpa: Some(p(0.001)),
        k_m_s: Some(p(0.01)),
        elements: n,
    }
}
fn point(t: f64, v: f64) -> LoadPoint {
    LoadPoint {
        time_s: t,
        delta_stress_kpa: Some(p(v)),
    }
}
fn profile(n: usize, drainage: Drainage) -> ConsolidationProfile {
    ConsolidationProfile::new(vec![layer(n)], Some(p(10.0)), drainage).expect("perfil")
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() < 2e-12 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}

#[test]
fn theta_matches_hand_derived_single_free_dof_and_consistent_source() {
    // One linear element: Mff=mv L/3, Kff=cv mv/L, Ff=mv L/2.
    // The two loads distinguish the full source from reduced Mff*1.
    for drainage in [Drainage::Top, Drainage::Bottom] {
        let model = profile(1, drainage);
        for theta in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let dt = 0.01;
            let m = 0.001 / 3.0;
            let k = 0.001;
            let config = ConsolidationOptions {
                theta,
                max_step_s: dt,
            };
            let step = model
                .solve(&[point(0.0, 100.0), point(dt, 100.0)], &[0.0, dt], config)
                .expect("step");
            let free = usize::from(drainage == Drainage::Top);
            let expected = 100.0 * (m - (1.0 - theta) * dt * k) / (m + theta * dt * k);
            close(step.states[1].excess_pore_pressure_kpa[free], expected);
            close(step.states[1].excess_pore_pressure_kpa[1 - free], 0.0);
            close(
                step.states[1].settlement_m,
                0.001 * (100.0 - expected / 2.0),
            );
            close(step.states[0].settlement_m, 0.05); // Explicit documented IC interpolation error.
            let ramp = model
                .solve(&[point(0.0, 0.0), point(dt, 100.0)], &[dt], config)
                .expect("ramp");
            close(
                ramp.states[0].excess_pore_pressure_kpa[free],
                (0.001 / 2.0) * 100.0 / (m + theta * dt * k),
            );
        }
    }
}

#[test]
fn load_knots_are_integrated_even_without_output_at_the_knots() {
    let model = profile(8, Drainage::Double);
    let loads = [
        point(0.0, 0.0),
        point(0.2, 100.0),
        point(0.35, -50.0),
        point(0.5, 0.0),
        point(1.0, 0.0),
    ];
    let options = ConsolidationOptions {
        theta: 1.0,
        max_step_s: 0.1,
    };
    let sparse = model
        .solve(&loads, &[1.0], options)
        .expect("sin salidas intermedias");
    let dense = model
        .solve(&loads, &[0.2, 0.35, 0.5, 1.0], options)
        .expect("con salidas en quiebres");
    assert_eq!(
        sparse.states[0].excess_pore_pressure_kpa,
        dense.states[3].excess_pore_pressure_kpa
    );
    close(sparse.states[0].settlement_m, dense.states[3].settlement_m);
    assert!(sparse.states[0].mean_degree.is_none());
    assert!(sparse.states[0]
        .excess_pore_pressure_kpa
        .iter()
        .any(|u| u.abs() > 1e-5));
}

#[test]
fn zero_load_remains_zero_and_pressure_queries_interpolate_and_validate() {
    let r = profile(4, Drainage::Top)
        .solve(
            &[point(0.0, 0.0), point(1.0, 0.0)],
            &[0.0, 0.4, 1.0],
            ConsolidationOptions::new(0.1),
        )
        .expect("zero");
    for state in &r.states {
        assert!(state.excess_pore_pressure_kpa.iter().all(|&v| v == 0.0));
        assert_eq!(state.settlement_m, 0.0);
        assert!(state.mean_degree.is_none());
    }
    let r = profile(4, Drainage::Top)
        .solve(
            &[point(0.0, 100.0), point(1.0, 100.0)],
            &[0.2],
            ConsolidationOptions::new(0.01),
        )
        .expect("step");
    close(
        r.pressure_at(0, 0.375).expect("interpolación"),
        0.5 * (r.states[0].excess_pore_pressure_kpa[1] + r.states[0].excess_pore_pressure_kpa[2]),
    );
    for z in [-0.1, 1.1] {
        assert_eq!(r.pressure_at(0, z).expect_err("fuera").code, "ENG-E-0010");
    }
    assert_eq!(
        r.pressure_at(1, 0.0).expect_err("índice").code,
        "ENG-E-0010"
    );
    assert_eq!(
        r.pressure_at(0, f64::NAN).expect_err("nan").code,
        "ENG-E-0008"
    );
}

#[test]
fn provenance_reports_every_assumed_physical_parameter_including_history() {
    let a = |v| Parameter::new(v, Provenance::Assumed);
    let mut l = layer(2);
    l.thickness_m = a(1.0);
    l.cv_m2_s = Some(a(1.0));
    l.mv_per_kpa = Some(a(0.001));
    l.k_m_s = Some(a(0.01));
    let model = ConsolidationProfile::new(vec![l], Some(a(10.0)), Drainage::Top).expect("assumed");
    let r = model
        .solve(
            &[
                LoadPoint {
                    time_s: 0.0,
                    delta_stress_kpa: Some(a(0.0)),
                },
                point(1.0, 100.0),
            ],
            &[0.1],
            ConsolidationOptions::new(0.01),
        )
        .expect("solve");
    assert_eq!(
        r.assumed_parameters,
        [
            "water_unit_weight_kn_m3",
            "layers[0].thickness_m",
            "layers[0].cv_m2_s",
            "layers[0].mv_per_kpa",
            "layers[0].k_m_s",
            "history[0].delta_stress_kpa"
        ]
    );
}

#[test]
fn missing_parameters_are_errors_even_in_deep_layers() {
    for field in 0..3 {
        let mut l = layer(2);
        match field {
            0 => l.cv_m2_s = None,
            1 => l.mv_per_kpa = None,
            _ => l.k_m_s = None,
        }
        let e = ConsolidationProfile::new(vec![layer(2), l], Some(p(10.0)), Drainage::Top)
            .expect_err("ausente");
        assert_eq!(e.code, "ENG-E-0003");
        assert!(e.hint.as_deref().expect("contexto").contains("layers[1]"));
    }
    assert_eq!(
        ConsolidationProfile::new(vec![layer(2)], None, Drainage::Top)
            .expect_err("γw ausente")
            .code,
        "ENG-E-0003"
    );
    assert_eq!(
        profile(2, Drainage::Top)
            .solve(
                &[
                    point(0.0, 0.0),
                    LoadPoint {
                        time_s: 1.0,
                        delta_stress_kpa: None
                    }
                ],
                &[0.0],
                ConsolidationOptions::new(1.0)
            )
            .expect_err("carga ausente")
            .code,
        "ENG-E-0003"
    );
}

#[test]
fn middle_layer_cv_discrepancy_is_rejected_without_substitution() {
    // Distinct H, cv, mv, k; only cv of the middle layer is altered below.
    let layers: Vec<_> = [
        (0.5, 0.25, 0.002, 0.005),
        (1.5, 2.25, 0.001, 0.0225),
        (1.0, 1.0, 0.003, 0.03),
    ]
    .into_iter()
    .map(|(h, cv, mv, k)| ConsolidationLayer {
        thickness_m: p(h),
        cv_m2_s: Some(p(cv)),
        mv_per_kpa: Some(p(mv)),
        k_m_s: Some(p(k)),
        elements: 2,
    })
    .collect();
    ConsolidationProfile::new(layers.clone(), Some(p(10.0)), Drainage::Top)
        .expect("las tres capas compatibles deben aceptarse");
    for factor in [1e-1, 10.0] {
        let mut incompatible = layers.clone();
        incompatible[1].cv_m2_s = Some(p(2.25 * factor));
        let error = ConsolidationProfile::new(incompatible, Some(p(10.0)), Drainage::Top)
            .expect_err(
                "cv intermedio incompatible debe rechazarse, sin sustituirlo por k/(mv γw)",
            );
        assert_eq!(error.code, "ENG-E-0010");
        let hint = error.hint.as_deref().expect("contexto de discrepancia");
        assert!(hint.contains("layers[1]") && hint.contains("cv incompatible"));
    }
}

#[test]
fn cv_consistency_relative_tolerance_boundary() {
    // Exact powers of two: (k / water) / mv = (2^-7 / 2^3) / 2^-10 = 1.
    // Thus cv / derived_cv is exactly the f64 value of 1 + delta, without
    // extra rounding from the material parameters. See provenance r34.
    let water = 8.0;
    let k = 0.0078125;
    let mv = 0.0009765625;
    let derived_cv: f64 = (k / water) / mv;
    assert_eq!(derived_cv.to_bits(), 1.0_f64.to_bits());

    for (delta, accepted) in [(5e-11, true), (2e-10, false)] {
        let cv = 1.0 + delta;
        let ratio = cv / derived_cv;
        assert_eq!(ratio.to_bits(), (1.0_f64 + delta).to_bits());
        assert_eq!((ratio - 1.0).abs() <= 1e-10, accepted);
        let l = ConsolidationLayer {
            thickness_m: p(1.0),
            cv_m2_s: Some(p(cv)),
            mv_per_kpa: Some(p(mv)),
            k_m_s: Some(p(k)),
            elements: 2,
        };
        let result = ConsolidationProfile::new(vec![l], Some(p(water)), Drainage::Top);
        if accepted {
            result.expect("razón 1+5e-11 debe aceptarse");
        } else {
            let error = result.expect_err("razón 1+2e-10 debe rechazarse");
            assert_eq!(error.code, "ENG-E-0010");
            let hint = error.hint.as_deref().expect("contexto de discrepancia");
            assert!(hint.contains("layers[0]") && hint.contains("cv incompatible"));
        }
    }
}

#[test]
fn invalid_materials_geometry_and_extreme_dimensions_have_diagnostics() {
    let mut cases = Vec::new();
    let mut l = layer(2);
    l.cv_m2_s = Some(p(2.0));
    cases.push((l, "ENG-E-0010"));
    let mut l = layer(2);
    l.mv_per_kpa = Some(p(0.0));
    cases.push((l, "ENG-E-0010"));
    let mut l = layer(2);
    l.k_m_s = Some(p(-0.01));
    cases.push((l, "ENG-E-0010"));
    let mut l = layer(2);
    l.thickness_m = p(f64::NAN);
    cases.push((l, "ENG-E-0008"));
    let mut l = layer(2);
    l.cv_m2_s = Some(p(f64::INFINITY));
    cases.push((l, "ENG-E-0008"));
    let mut l = layer(2);
    l.thickness_m = p(0.0);
    cases.push((l, "ENG-E-0010"));
    cases.push((layer(0), "ENG-E-0010"));
    cases.push((layer(usize::MAX), "ENG-E-0010"));
    for (l, code) in cases {
        assert_eq!(
            ConsolidationProfile::new(vec![l], Some(p(10.0)), Drainage::Top)
                .expect_err("inválido")
                .code,
            code
        );
    }
    assert_eq!(
        ConsolidationProfile::new(vec![layer(1)], Some(p(10.0)), Drainage::Double)
            .expect_err("sin nodo libre")
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        ConsolidationProfile::new(
            vec![layer(usize::MAX), layer(1)],
            Some(p(10.0)),
            Drainage::Top
        )
        .expect_err("overflow")
        .code,
        "ENG-E-0010"
    );
    // Thickness lost when accumulating at an existing large depth.
    let mut l = layer(1);
    l.thickness_m = p(1e20);
    assert_eq!(
        ConsolidationProfile::new(vec![l, layer(1)], Some(p(10.0)), Drainage::Top)
            .expect_err("escala")
            .code,
        "ENG-E-0010"
    );
}

#[test]
fn theta_stability_and_numerical_controls_are_checked() {
    let model = profile(4, Drainage::Top);
    let history = [point(0.0, 100.0), point(1.0, 100.0)];
    for (theta, dt, code) in [
        (-0.1, 0.01, "ENG-E-0010"),
        (1.1, 0.01, "ENG-E-0010"),
        (f64::NAN, 0.01, "ENG-E-0008"),
        (0.5, 0.0, "ENG-E-0010"),
        (0.5, f64::INFINITY, "ENG-E-0008"),
        (0.0, 1.0, "ENG-E-0010"),
        (0.25, 1.0, "ENG-E-0010"),
        (0.5, 1e-10, "ENG-E-0010"),
    ] {
        assert_eq!(
            model
                .solve(
                    &history,
                    &[1.0],
                    ConsolidationOptions {
                        theta,
                        max_step_s: dt
                    }
                )
                .expect_err("controles inválidos")
                .code,
            code
        );
    }
    for theta in [0.0, 0.25, 0.5, 1.0] {
        model
            .solve(
                &history,
                &[0.1],
                ConsolidationOptions {
                    theta,
                    max_step_s: 0.001,
                },
            )
            .expect("estable");
    }
}

#[test]
fn history_and_output_times_never_extrapolate_or_accept_nonfinite_data() {
    let model = profile(2, Drainage::Top);
    let valid = [point(0.0, 0.0), point(1.0, 100.0)];
    for times in [
        vec![],
        vec![-0.1],
        vec![1.1],
        vec![0.5, 0.5],
        vec![0.8, 0.2],
    ] {
        assert_eq!(
            model
                .solve(&valid, &times, ConsolidationOptions::new(0.1))
                .expect_err("tiempos")
                .code,
            "ENG-E-0010"
        );
    }
    assert_eq!(
        model
            .solve(&valid, &[f64::NAN], ConsolidationOptions::new(0.1))
            .expect_err("nan")
            .code,
        "ENG-E-0008"
    );
    for h in [
        vec![],
        vec![point(0.0, 0.0)],
        vec![point(0.1, 0.0), point(1.0, 0.0)],
        vec![point(0.0, 0.0), point(0.0, 1.0)],
        vec![point(0.0, 0.0), point(-1.0, 1.0)],
    ] {
        assert_eq!(
            model
                .solve(&h, &[0.0], ConsolidationOptions::new(0.1))
                .expect_err("historia")
                .code,
            "ENG-E-0010"
        );
    }
    assert_eq!(
        model
            .solve(
                &[point(0.0, 0.0), point(1.0, f64::INFINITY)],
                &[0.0],
                ConsolidationOptions::new(0.1)
            )
            .expect_err("carga")
            .code,
        "ENG-E-0008"
    );
}

#[test]
fn drained_boundaries_long_time_settlement_and_bitwise_repeatability() {
    let model = profile(8, Drainage::Double);
    let run = || {
        model
            .solve(
                &[point(0.0, 100.0), point(10.0, 100.0)],
                &[0.0, 0.01, 0.2, 10.0],
                ConsolidationOptions::new(0.01),
            )
            .expect("run")
    };
    let a = run();
    let b = run();
    for (x, y) in a.states.iter().zip(&b.states) {
        assert_eq!(x.settlement_m.to_bits(), y.settlement_m.to_bits());
        assert_eq!(x.excess_pore_pressure_kpa, y.excess_pore_pressure_kpa);
        assert_eq!(x.excess_pore_pressure_kpa[0], 0.0);
        assert_eq!(x.excess_pore_pressure_kpa[8], 0.0);
    }
    close(a.states[3].settlement_m, 0.1);
    close(a.states[3].mean_degree.expect("U"), 1.0);
}
