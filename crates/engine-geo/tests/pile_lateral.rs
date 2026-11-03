use engine_core::EResult;
use engine_geo::pile::lateral::{evaluate, Head};

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-12 * expected.abs().max(1.0));
}

#[test]
fn head_and_depth_responses() -> EResult<()> {
    // EI=1 and k=4 give beta=1. These values follow from the field.
    let e = 0.207_879_576_350_761_93;
    let free = Head::Free { moment_nm: 2.0 };
    let head = evaluate(1.0, 4.0, 4.0, free, 0.0)?;
    close(head.y_m, 1.0);
    close(head.moment_nm, 2.0);
    close(head.shear_n, 4.0);
    close(head.soil_force_n_per_m, -4.0);

    let depth = evaluate(1.0, 4.0, 4.0, free, core::f64::consts::FRAC_PI_2)?;
    close(depth.y_m, e);
    close(depth.moment_nm, -2.0 * e);
    close(depth.shear_n, 0.0);
    close(depth.soil_force_n_per_m, -4.0 * e);

    let fixed = evaluate(1.0, 4.0, 4.0, Head::Fixed, 0.0)?;
    close(fixed.y_m, 1.0);
    close(fixed.moment_nm, 2.0);
    close(fixed.shear_n, 4.0);
    let fixed_depth = evaluate(1.0, 4.0, 4.0, Head::Fixed, core::f64::consts::FRAC_PI_2)?;
    close(fixed_depth.y_m, e);
    close(fixed_depth.moment_nm, -2.0 * e);
    close(fixed_depth.shear_n, 0.0);

    let moment_free = evaluate(
        1.0,
        4.0,
        2.0,
        Head::Free { moment_nm: 0.0 },
        core::f64::consts::FRAC_PI_2,
    )?;
    close(moment_free.y_m, 0.0);
    close(moment_free.moment_nm, -2.0 * e);
    close(moment_free.shear_n, -2.0 * e);
    close(moment_free.soil_force_n_per_m, 0.0);

    let negative = evaluate(1.0, 4.0, -4.0, Head::Free { moment_nm: -2.0 }, 0.0)?;
    close(negative.y_m, -1.0);
    close(negative.moment_nm, -2.0);
    close(negative.shear_n, -4.0);
    close(negative.soil_force_n_per_m, 4.0);
    Ok(())
}

#[test]
fn rejects_invalid_and_nonfinite_values() {
    for (ei, k, z) in [
        (0.0, 1.0, 0.0),
        (-1.0, 1.0, 0.0),
        (1.0, -1.0, 0.0),
        (1.0, 0.0, 0.0),
        (1.0, 1.0, -1.0),
    ] {
        assert_eq!(
            evaluate(ei, k, 1.0, Head::Fixed, z).unwrap_err().code,
            "ENG-E-0010"
        );
    }
    assert_eq!(
        evaluate(1.0, 1.0, f64::NAN, Head::Fixed, 0.0)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    assert_eq!(
        evaluate(
            1.0,
            1.0,
            1.0,
            Head::Free {
                moment_nm: f64::INFINITY
            },
            0.0
        )
        .unwrap_err()
        .code,
        "ENG-E-0008"
    );
    assert_eq!(
        evaluate(f64::MIN_POSITIVE, f64::MAX, 1.0, Head::Fixed, 0.0)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    assert_eq!(
        evaluate(f64::MAX, f64::MIN_POSITIVE, 1.0, Head::Fixed, 0.0)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
}
