//! Own analytical cases; derivations in provenance/ENG-GEO-005.md.
use engine_analysis::{
    newton::NewtonOptions,
    pile::{solve, Head, PileInput},
};
use engine_contracts::Status;
use engine_core::EResult;
use engine_geo::{
    pile::{lateral, spring::EndPolicy, PileDiscretization},
    py, tz,
};
use engine_linalg::ReferenceBackend;

fn curves(n: usize) -> EResult<(Vec<py::PyCurve>, Vec<tz::TzCurve>, tz::qz::QzCurve)> {
    let py = py::user(vec![(0.0, 0.0), (10.0, 40.0)])?;
    let shaft = tz::user(
        Some(vec![(0.0, 0.0), (10.0, 0.0)]),
        tz::UserTraction::Symmetric,
    )?;
    let tip = tz::qz::user(
        Some(vec![(0.0, 0.0), (10.0, 100.0)]),
        tz::UserTraction::Symmetric,
    )?;
    Ok((vec![py; n], vec![shaft; n], tip))
}

#[test]
fn finite_pile_refines_toward_semi_infinite_head_and_checks_axial_series() -> EResult<()> {
    let mut errors = Vec::new();
    for elements in [16, 32, 64] {
        let mesh = PileDiscretization::uniform(12.0, elements)?;
        let (py, shaft, tip) = curves(elements + 1)?;
        let input = PileInput {
            mesh: &mesh,
            young_pa: 1.0,
            area_m2: 100.0,
            inertia_m4: 1.0,
            diameter_m: 1.0,
            lateral: &py,
            shaft: &shaft,
            tip: &tip,
            end_policy: EndPolicy::ExtendEndSegments,
            head: Head::Free { moment_nm: 2.0 },
            horizontal_n: 4.0,
            vertical_n: 10.0,
        };
        let result = solve(
            ReferenceBackend,
            &input,
            &[0.5, 1.0],
            NewtonOptions::default(),
        )?;
        assert_eq!(result.status, Status::Completed);
        assert_eq!(result.accepted.len(), 2);
        let full = &result.accepted[1];
        let exact = lateral::evaluate(1.0, 4.0, 4.0, lateral::Head::Free { moment_nm: 2.0 }, 0.0)?;
        errors.push((full.profile[0].y_m - exact.y_m).abs());
        // EA=100 N, L=12 m and tip stiffness=10 N/m: springs in series.
        assert!((full.head_settlement_m - 2.2).abs() < 1e-8);
        assert!((result.accepted[0].head_settlement_m - 1.1).abs() < 1e-8);
        assert!((full.profile[0].soil_force_n_per_m + 4.0 * full.profile[0].y_m).abs() < 1e-10);
    }
    assert!(errors[1] < errors[0] && errors[2] < errors[1]);
    assert!(errors[2] < 0.02);
    Ok(())
}

#[test]
fn fixed_head_and_softening_rejection() -> EResult<()> {
    let mesh = PileDiscretization::uniform(4.0, 4)?;
    let (py, shaft, tip) = curves(5)?;
    let input = PileInput {
        mesh: &mesh,
        young_pa: 1.0,
        area_m2: 100.0,
        inertia_m4: 1.0,
        diameter_m: 1.0,
        lateral: &py,
        shaft: &shaft,
        tip: &tip,
        end_policy: EndPolicy::ExtendEndSegments,
        head: Head::Fixed,
        horizontal_n: 4.0,
        vertical_n: 10.0,
    };
    assert_eq!(
        solve(ReferenceBackend, &input, &[1.0], NewtonOptions::default())?.status,
        Status::Completed
    );
    let mut bad_shaft = shaft.clone();
    bad_shaft[0] = tz::user(
        Some(vec![(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)]),
        tz::UserTraction::Symmetric,
    )?;
    let bad = PileInput {
        shaft: &bad_shaft,
        ..input
    };
    assert_eq!(
        solve(ReferenceBackend, &bad, &[1.0], NewtonOptions::default())
            .err()
            .map(|e| e.code),
        Some("ENG-E-0010")
    );
    Ok(())
}

#[test]
fn profiles_refine_toward_semi_infinite_solution() -> EResult<()> {
    for fixed in [false, true] {
        let mut errors = Vec::new();
        for elements in [32, 64] {
            let mesh = PileDiscretization::uniform(12.0, elements)?;
            let (py, shaft, tip) = curves(elements + 1)?;
            let input = PileInput {
                mesh: &mesh,
                young_pa: 1.0,
                area_m2: 100.0,
                inertia_m4: 1.0,
                diameter_m: 1.0,
                lateral: &py,
                shaft: &shaft,
                tip: &tip,
                end_policy: EndPolicy::ExtendEndSegments,
                head: if fixed {
                    Head::Fixed
                } else {
                    Head::Free { moment_nm: 2.0 }
                },
                horizontal_n: 4.0,
                vertical_n: 0.0,
            };
            let result = solve(ReferenceBackend, &input, &[1.0], NewtonOptions::default())?;
            assert_eq!(result.status, Status::Completed);
            let profile = &result.accepted[0].profile;
            let exact_head = if fixed {
                lateral::Head::Fixed
            } else {
                lateral::Head::Free { moment_nm: 2.0 }
            };
            let mut max_error = [0.0_f64; 4];
            // These depths are nodes on both meshes.
            for eighth in 0..=3 {
                let station = &profile[eighth * elements / 8];
                let exact = lateral::evaluate(1.0, 4.0, 4.0, exact_head, station.depth_m)?;
                let differences = [
                    (station.y_m - exact.y_m).abs(),
                    (station.moment_nm - exact.moment_nm).abs(),
                    (station.shear_n - exact.shear_n).abs(),
                    (station.soil_force_n_per_m - exact.soil_force_n_per_m).abs(),
                ];
                for (maximum, difference) in max_error.iter_mut().zip(differences) {
                    *maximum = maximum.max(difference);
                }
            }
            if elements == 64 {
                assert!(max_error[0] < 0.03, "displacement: {max_error:?}");
                assert!(max_error[1] < 0.08, "moment: {max_error:?}");
                assert!(max_error[2] < 0.5, "shear: {max_error:?}");
                assert!(max_error[3] < 0.12, "soil force: {max_error:?}");
            }
            errors.push(max_error);

            // Independent global force and, for the free head, moment balance.
            let mut soil_force = 0.0;
            let mut soil_moment = 0.0;
            for (station, weight) in profile.iter().zip(mesh.tributaries_m()) {
                let force = station.soil_force_n_per_m * weight;
                soil_force += force;
                soil_moment += station.depth_m * force;
            }
            assert!((4.0 + soil_force).abs() < 1e-7);
            if !fixed {
                assert!((2.0 + soil_moment).abs() < 1e-7);
            }
        }
        for component in 0..4 {
            assert!(
                errors[1][component] < errors[0][component],
                "component {component}: {errors:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn nonlinear_axial_settlement_and_unloading() -> EResult<()> {
    let mesh = PileDiscretization::uniform(2.0, 1)?;
    let (py, mut shaft, _) = curves(2)?;
    shaft[0] = tz::user(
        Some(vec![(0.0, 0.0), (0.1, 3.0), (0.3, 5.0)]),
        tz::UserTraction::Symmetric,
    )?;
    let tip = tz::qz::user(
        Some(vec![(0.0, 0.0), (1.0, 50.0)]),
        tz::UserTraction::Symmetric,
    )?;
    let input = PileInput {
        mesh: &mesh,
        young_pa: 1.0,
        area_m2: 100.0,
        inertia_m4: 1.0,
        diameter_m: 1.0 / std::f64::consts::PI,
        lateral: &py,
        shaft: &shaft,
        tip: &tip,
        end_policy: EndPolicy::ExtendEndSegments,
        head: Head::Free { moment_nm: 0.0 },
        horizontal_n: 0.0,
        vertical_n: 9.0,
    };
    let result = solve(
        ReferenceBackend,
        &input,
        &[1.0 / 3.0, 1.0, -1.0 / 3.0],
        NewtonOptions::default(),
    )?;
    assert_eq!(result.status, Status::Completed);
    assert_eq!(result.accepted.len(), 3);
    for (step, head, toe) in [
        (&result.accepted[0], 3.0 / 55.0, 3.0 / 110.0),
        (&result.accepted[1], 0.2, 0.1),
        (&result.accepted[2], -3.0 / 55.0, -3.0 / 110.0),
    ] {
        assert!((step.head_settlement_m - head).abs() < 1e-9);
        assert!((step.profile[1].settlement_m - toe).abs() < 1e-9);
    }
    Ok(())
}

#[test]
fn failed_newton_step_is_not_accepted_or_followed() -> EResult<()> {
    let mesh = PileDiscretization::uniform(2.0, 1)?;
    let (py, mut shaft, _) = curves(2)?;
    shaft[0] = tz::user(
        Some(vec![(0.0, 0.0), (0.1, 3.0), (0.3, 5.0)]),
        tz::UserTraction::Symmetric,
    )?;
    let tip = tz::qz::user(
        Some(vec![(0.0, 0.0), (1.0, 50.0)]),
        tz::UserTraction::Symmetric,
    )?;
    let input = PileInput {
        mesh: &mesh,
        young_pa: 1.0,
        area_m2: 100.0,
        inertia_m4: 1.0,
        diameter_m: 1.0 / std::f64::consts::PI,
        lateral: &py,
        shaft: &shaft,
        tip: &tip,
        end_policy: EndPolicy::ExtendEndSegments,
        head: Head::Free { moment_nm: 0.0 },
        horizontal_n: 0.0,
        vertical_n: 9.0,
    };
    let options = NewtonOptions {
        max_iterations: 1,
        ..NewtonOptions::default()
    };
    let result = solve(ReferenceBackend, &input, &[0.0, 1.0, 0.5], options)?;
    assert_eq!(result.status, Status::NotConverged);
    assert_eq!(result.accepted.len(), 1);
    assert_eq!(result.accepted[0].factor, 0.0);
    assert_eq!(result.accepted[0].head_settlement_m, 0.0);
    assert_eq!(result.reports.len(), 2);
    assert_eq!(result.reports[0].load_factor, 0.0);
    assert!(result.reports[0].converged);
    assert_eq!(result.reports[1].load_factor, 1.0);
    assert!(!result.reports[1].converged);
    assert_eq!(result.reports[1].iterations, 1);
    Ok(())
}

#[test]
fn finite_pile_rejects_invalid_loads_and_curve_counts() -> EResult<()> {
    let mesh = PileDiscretization::uniform(2.0, 1)?;
    let (py, shaft, tip) = curves(2)?;
    let input = PileInput {
        mesh: &mesh,
        young_pa: 1.0,
        area_m2: 100.0,
        inertia_m4: 1.0,
        diameter_m: 1.0,
        lateral: &py,
        shaft: &shaft,
        tip: &tip,
        end_policy: EndPolicy::ExtendEndSegments,
        head: Head::Free { moment_nm: 0.0 },
        horizontal_n: 4.0,
        vertical_n: 10.0,
    };
    assert_eq!(
        solve(ReferenceBackend, &input, &[], NewtonOptions::default())
            .err()
            .map(|e| e.code),
        Some("ENG-E-0010")
    );
    assert_eq!(
        solve(
            ReferenceBackend,
            &input,
            &[1.0, f64::NAN],
            NewtonOptions::default()
        )
        .err()
        .map(|e| e.code),
        Some("ENG-E-0008")
    );
    let bad = PileInput {
        lateral: &py[..1],
        ..input
    };
    assert_eq!(
        solve(ReferenceBackend, &bad, &[1.0], NewtonOptions::default())
            .err()
            .map(|e| e.code),
        Some("ENG-E-0010")
    );
    Ok(())
}
