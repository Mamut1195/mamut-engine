//! Conformidad y fronteras de la API p-y. Oráculos: provenance/ENG-GEO-003.md.
use engine_core::{EResult, Real};
use engine_geo::py::*;
use engine_geo::{Layer, Parameter, Provenance, StratifiedProfile};

fn matlock() -> MatlockParams {
    MatlockParams {
        depth_m: Some(2.0),
        diameter_m: Some(1.0),
        undrained_strength_pa: Some(20000.0),
        effective_vertical_stress_pa: Some(20000.0),
        j: Some(0.5),
        epsilon_50: Some(0.01),
    }
}
fn reese() -> ReeseParams {
    ReeseParams {
        depth_m: Some(2.0),
        diameter_m: Some(1.0),
        friction_angle_rad: Some(std::f64::consts::PI / 6.0),
        effective_unit_weight_n_m3: Some(10000.0),
        alpha_rad: Some(std::f64::consts::PI / 6.0),
        k0: Some(0.4),
        a: Some(1.0),
        b: Some(0.5),
        subgrade_modulus_n_m3: Some(11231384.38763306),
    }
}
fn sampling() -> Sampling {
    Sampling {
        positive_y_m: vec![0.003125, 0.025, 0.1],
        plateau_y_m: Some(0.4),
    }
}
fn close(a: Real, b: Real) {
    assert!((a - b).abs() <= 1e-9 + 1e-12 * b.abs(), "{a} != {b}");
}
fn code<T: std::fmt::Debug>(result: EResult<T>, expected: &str) {
    assert_eq!(result.unwrap_err().code, expected);
}

#[test]
fn matlock_landmarks_plateau_symmetry_and_secant() -> EResult<()> {
    let result = matlock_soft_clay(&matlock(), &sampling())?;
    assert_eq!(result.curve.kind(), MATLOCK_SOFT_CLAY);
    close(result.np, 5.0);
    close(result.ultimate_n_m, 100000.0);
    close(result.yc_m, 0.025);
    for (y, p) in [
        (0.003125, 25000.0),
        (0.025, 50000.0),
        (0.2, 100000.0),
        (0.4, 100000.0),
    ] {
        close(result.curve.at(y)?, p);
        close(result.curve.at(-y)?, -p);
    }
    close(result.curve.at(0.0015625)?, 12500.0); // Primer tramo secante, no cúbica.
    assert!(result.curve.points().contains(&(0.2, 100000.0)));
    code(result.curve.at(0.401), "ENG-E-0010");
    code(result.curve.at(Real::NAN), "ENG-E-0008");
    Ok(())
}
#[test]
fn matlock_non_unit_diameter_uses_j_x_over_d() -> EResult<()> {
    // Cálculo propio en forma cerrada de J2: Matlock (1970), OTC 1204,
    // Ec. 1, 3, 5 y curva estática. Np=3+15000/30000+0.5·3/0.5=6.5<9;
    // el mutante J·(x·d) daría 4.25. pu=Np·c·d=97500 N/m,
    // yc=2.5·ε50·d=0.0125 m, 8yc=0.1 m.
    // p(yc/8)=0.5·pu·(1/8)^(1/3)=24375 N/m; p(yc)=pu/2.
    let params = MatlockParams {
        depth_m: Some(3.0),
        diameter_m: Some(0.5),
        undrained_strength_pa: Some(30000.0),
        effective_vertical_stress_pa: Some(15000.0),
        j: Some(0.5),
        epsilon_50: Some(0.01),
    };
    let samples = Sampling {
        positive_y_m: vec![0.0015625, 0.0125, 0.1],
        plateau_y_m: Some(0.2),
    };
    let result = matlock_soft_clay(&params, &samples)?;
    close(result.np, 6.5);
    close(result.ultimate_n_m, 97500.0);
    close(result.yc_m, 0.0125);
    for (y, p) in [
        (0.0015625, 24375.0),
        (0.0125, 48750.0),
        (0.1, 97500.0),
        (0.2, 97500.0),
    ] {
        close(result.curve.at(y)?, p);
        close(result.curve.at(-y)?, -p);
    }
    Ok(())
}
#[test]
fn matlock_transition_eq4_matches_eq3_cap() -> EResult<()> {
    let mut p = matlock();
    // γ'=10000, c=20000, d=1, J=.5: xr=6/(.5+.5)=6 m.
    for (x, np) in [(5.0, 8.0), (6.0, 9.0), (7.0, 9.0)] {
        p.depth_m = Some(x);
        p.effective_vertical_stress_pa = Some(10000.0 * x);
        close(matlock_soft_clay(&p, &sampling())?.np, np);
    }
    p.depth_m = Some(0.0);
    p.effective_vertical_stress_pa = Some(0.0);
    close(matlock_soft_clay(&p, &sampling())?.np, 3.0);
    Ok(())
}

type ClayField = fn(&mut MatlockParams) -> &mut Option<Real>;
const CLAY_FIELDS: &[ClayField] = &[
    |p| &mut p.depth_m,
    |p| &mut p.diameter_m,
    |p| &mut p.undrained_strength_pa,
    |p| &mut p.effective_vertical_stress_pa,
    |p| &mut p.j,
    |p| &mut p.epsilon_50,
];
type SandField = fn(&mut ReeseParams) -> &mut Option<Real>;
const SAND_FIELDS: &[SandField] = &[
    |p| &mut p.depth_m,
    |p| &mut p.diameter_m,
    |p| &mut p.friction_angle_rad,
    |p| &mut p.effective_unit_weight_n_m3,
    |p| &mut p.alpha_rad,
    |p| &mut p.k0,
    |p| &mut p.a,
    |p| &mut p.b,
    |p| &mut p.subgrade_modulus_n_m3,
];
#[test]
fn every_missing_parameter_is_a_typed_error_even_at_surface() {
    for field in CLAY_FIELDS {
        let mut p = matlock();
        *field(&mut p) = None;
        code(matlock_soft_clay(&p, &sampling()), "ENG-E-0003");
    }
    for x in [0.0, 2.0] {
        for field in SAND_FIELDS {
            let mut p = reese();
            p.depth_m = Some(x);
            *field(&mut p) = None;
            code(reese_sand(&p, &sampling()), "ENG-E-0003");
        }
    }
    let mut s = sampling();
    s.plateau_y_m = None;
    code(matlock_soft_clay(&matlock(), &s), "ENG-E-0003");
    code(reese_sand(&reese(), &s), "ENG-E-0003");
}
#[test]
fn every_numeric_parameter_rejects_nan_infinities_and_negative_values() {
    for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY, -1.0] {
        let expected = if value.is_finite() {
            "ENG-E-0010"
        } else {
            "ENG-E-0008"
        };
        for field in CLAY_FIELDS {
            let mut p = matlock();
            *field(&mut p) = Some(value);
            code(matlock_soft_clay(&p, &sampling()), expected);
        }
        for field in SAND_FIELDS {
            let mut p = reese();
            *field(&mut p) = Some(value);
            code(reese_sand(&p, &sampling()), expected);
        }
    }
    for field in &CLAY_FIELDS[1..] {
        let mut p = matlock();
        *field(&mut p) = Some(0.0);
        if p.effective_vertical_stress_pa != Some(0.0) {
            code(matlock_soft_clay(&p, &sampling()), "ENG-E-0010");
        }
    }
    for field in &SAND_FIELDS[1..] {
        let mut p = reese();
        *field(&mut p) = Some(0.0);
        code(reese_sand(&p, &sampling()), "ENG-E-0010");
    }
}
#[test]
fn invalid_sampling_is_never_sorted_or_silently_repaired() {
    for ys in [
        vec![-1.0],
        vec![0.0],
        vec![0.01, 0.01],
        vec![0.02, 0.01],
        vec![0.5],
    ] {
        let s = Sampling {
            positive_y_m: ys,
            plateau_y_m: Some(0.4),
        };
        code(matlock_soft_clay(&matlock(), &s), "ENG-E-0010");
        code(reese_sand(&reese(), &s), "ENG-E-0010");
    }
    for y in [Real::NAN, Real::INFINITY] {
        let s = Sampling {
            positive_y_m: vec![y],
            plateau_y_m: Some(0.4),
        };
        code(matlock_soft_clay(&matlock(), &s), "ENG-E-0008");
        code(reese_sand(&reese(), &s), "ENG-E-0008");
        let s = Sampling {
            positive_y_m: vec![],
            plateau_y_m: Some(y),
        };
        code(matlock_soft_clay(&matlock(), &s), "ENG-E-0008");
        code(reese_sand(&reese(), &s), "ENG-E-0008");
    }
    for plateau in [0.0, -1.0, 0.01, 0.0375] {
        let s = Sampling {
            positive_y_m: vec![],
            plateau_y_m: Some(plateau),
        };
        code(matlock_soft_clay(&matlock(), &s), "ENG-E-0010");
        code(reese_sand(&reese(), &s), "ENG-E-0010");
    }
    let s = Sampling {
        positive_y_m: vec![],
        plateau_y_m: Some(0.2),
    };
    code(matlock_soft_clay(&matlock(), &s), "ENG-E-0010");
}
#[test]
fn reese_branches_landmarks_and_continuity() -> EResult<()> {
    let r = reese_sand(&reese(), &sampling())?;
    assert_eq!(r.curve.kind(), REESE_SAND);
    close(r.pct_n_m, 187189.73979388434);
    close(r.pcd_n_m, 574902.5527149864);
    close(r.transition_depth_m, 7.792966107085287);
    close(r.n, 1.25);
    let yk = r
        .yk_m
        .ok_or_else(|| engine_core::EngineError::new("ENG-E-0010", "yk ausente"))?;
    close(yk, 1.0 / 61440.0);
    for (y, p) in [
        (yk, 365.60496053493034),
        (1.0 / 60.0, 93594.86989694217),
        (0.0375, 187189.73979388434),
        (0.4, 187189.73979388434),
    ] {
        close(r.curve.at(y)?, p);
        close(r.curve.at(-y)?, -p);
        assert!(r.curve.points().iter().any(|&(v, _)| v == y));
    }
    // Muestras a ambos lados de las uniones, comparadas entre sí a escala SI.
    let s = Sampling {
        positive_y_m: vec![
            yk * (1.0 - 1e-8),
            yk * (1.0 + 1e-8),
            r.ym_m * (1.0 - 1e-8),
            r.ym_m * (1.0 + 1e-8),
            r.yu_m * (1.0 - 1e-8),
            r.yu_m * (1.0 + 1e-8),
        ],
        plateau_y_m: Some(0.4),
    };
    let dense = reese_sand(&reese(), &s)?;
    for y in [yk, r.ym_m, r.yu_m] {
        assert!(
            (dense.curve.at(y * (1.0 + 1e-8))? - dense.curve.at(y * (1.0 - 1e-8))?).abs() < 0.01
        );
    }
    // La selección profunda usa Ec. 8 y no prolonga Ec. 5.
    let mut p = reese();
    p.depth_m = Some(10.0);
    close(reese_sand(&p, &sampling())?.ultimate_n_m, 2874512.763574932);
    p.depth_m = Some(4.0);
    close(reese_sand(&p, &sampling())?.ultimate_n_m, 642092.2925088706);
    // También se admite el α=φ/2 propuesto por el método, siempre explícito.
    p.depth_m = Some(2.0);
    p.alpha_rad = Some(std::f64::consts::PI / 12.0);
    let proposed = reese_sand(&p, &sampling())?;
    close(proposed.pct_n_m, 129801.51532289962);
    close(proposed.transition_depth_m, 13.64147036875595);
    Ok(())
}
#[test]
fn reese_surface_is_null_and_degenerate_parameters_are_errors() -> EResult<()> {
    let mut p = reese();
    p.depth_m = Some(0.0);
    let r = reese_sand(&p, &sampling())?;
    assert_eq!(r.yk_m, None);
    assert!(r.curve.points().iter().all(|&(_, v)| v == 0.0));
    for a in [0.5, 0.4, 1.125, 2.0] {
        // A≤B, n=1 y n<1.
        p.a = Some(a);
        code(reese_sand(&p, &sampling()), "ENG-E-0010");
    }
    p = reese();
    for k in [100.0, 2807846.09690] {
        // yk>ym, incluido un k apenas inferior al umbral.
        p.subgrade_modulus_n_m3 = Some(k);
        code(reese_sand(&p, &sampling()), "ENG-E-0010");
    }
    for field in [SAND_FIELDS[2], SAND_FIELDS[4]] {
        p = reese();
        *field(&mut p) = Some(std::f64::consts::FRAC_PI_2);
        code(reese_sand(&p, &sampling()), "ENG-E-0010");
    }
    Ok(())
}
#[test]
fn profile_adapter_converts_kpa_without_assuming_soil_parameters() -> EResult<()> {
    let measured = |v| Parameter::new(v, Provenance::Measured);
    let profile = StratifiedProfile::new(
        vec![Layer::new(10.0, Some(measured(18.0)), Some(measured(20.0)))],
        Some(measured(3.0)),
        Some(measured(10.0)),
    )
    .unwrap();
    close(effective_stress_pa(&profile, 5.0)?, 74000.0);
    let mut p = matlock();
    p.depth_m = Some(5.0);
    p.effective_vertical_stress_pa = Some(effective_stress_pa(&profile, 5.0)?);
    close(matlock_soft_clay(&p, &sampling())?.np, 9.0);
    code(effective_stress_pa(&profile, 10.1), "ENG-E-0010");
    code(effective_stress_pa(&profile, -1.0), "ENG-E-0010");
    let negative = StratifiedProfile::new(
        vec![Layer::new(2.0, Some(measured(18.0)), Some(measured(8.0)))],
        Some(measured(0.0)),
        Some(measured(10.0)),
    )
    .unwrap();
    code(effective_stress_pa(&negative, 1.0), "ENG-E-0010");
    Ok(())
}
#[test]
fn user_points_have_strict_validation_and_bounded_evaluation() -> EResult<()> {
    let r = user(vec![(0.0, 0.0), (0.01, 100.0), (0.02, 100.0)])?;
    assert_eq!(r.kind(), USER);
    close(r.at(-0.005)?, -50.0);
    code(r.at(-0.021), "ENG-E-0010");
    code(r.at(0.021), "ENG-E-0010");
    for points in [
        vec![],
        vec![(0.0, 0.0)],
        vec![(0.01, 1.0), (0.02, 2.0)],
        vec![(0.0, 1.0), (0.01, 2.0)],
        vec![(0.0, 0.0), (0.01, -1.0)],
        vec![(0.0, 0.0), (0.01, 2.0), (0.02, 1.0)],
        vec![(0.0, 0.0), (0.0, 1.0)],
        vec![(-0.01, 0.0), (0.0, 0.0)],
        vec![(0.0, 0.0), (0.02, 1.0), (0.01, 2.0)],
    ] {
        code(user(points), "ENG-E-0010");
    }
    for v in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
        code(user(vec![(0.0, 0.0), (v, 1.0)]), "ENG-E-0008");
        code(user(vec![(0.0, 0.0), (0.01, v)]), "ENG-E-0008");
    }
    Ok(())
}
#[test]
fn numerical_overflow_underflow_and_zero_slope_loss_are_errors() {
    let mut p = matlock();
    p.undrained_strength_pa = Some(Real::MAX);
    code(matlock_soft_clay(&p, &sampling()), "ENG-E-0008");
    p = matlock();
    p.epsilon_50 = Some(Real::from_bits(1));
    p.diameter_m = Some(0.01);
    code(matlock_soft_clay(&p, &sampling()), "ENG-E-0010");
    let mut p = reese();
    p.subgrade_modulus_n_m3 = Some(Real::MAX);
    code(reese_sand(&p, &sampling()), "ENG-E-0008");
    p = reese();
    p.effective_unit_weight_n_m3 = Some(Real::MAX);
    code(reese_sand(&p, &sampling()), "ENG-E-0008");
    code(
        user(vec![(0.0, 0.0), (Real::MIN_POSITIVE, Real::MAX)]),
        "ENG-E-0008",
    );
    code(
        user(vec![(0.0, 0.0), (1e300, Real::MIN_POSITIVE)]),
        "ENG-E-0010",
    );
}
