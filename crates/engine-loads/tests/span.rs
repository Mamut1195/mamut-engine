mod fixtures;

use engine_contracts::{Element, ElementCtx, ElementParams};
use engine_core::math;
use engine_core::{Dim, EResult};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_loads::{
    equivalent_nodal_forces_local as equivalent, fixed_end_forces_local as fixed, to_member_load,
    Beam2D, LoadAxes, SpanLoad,
};
use fixtures::close;

fn vector(actual: [f64; 6], expected: [f64; 6]) {
    for (a, e) in actual.into_iter().zip(expected) {
        close(a, e);
    }
}

fn trapezoid(qi: [f64; 2], qj: [f64; 2], a: Option<f64>, b: Option<f64>) -> SpanLoad {
    SpanLoad::Trapezoidal {
        q_i: qi,
        q_j: qj,
        a,
        b,
        axes: LoadAxes::Local,
    }
}

// Equilibrium of the restrained member, with applied exterior forces + reactions.
fn equilibrium(beam: &Beam2D, load: &SpanLoad, total: [f64; 2], first_moment: f64) {
    let reactions = fixed(beam, load).unwrap();
    close(reactions[0] + reactions[3] + total[0], 0.0);
    close(reactions[1] + reactions[4] + total[1], 0.0);
    close(
        reactions[2] + reactions[5] + beam.length() * reactions[4] + first_moment,
        0.0,
    );
    vector(equivalent(beam, load).unwrap(), reactions.map(|r| -r));
}

#[test]
fn uniform_and_equal_trapezoid_closed_tables() -> EResult<()> {
    let (l, w, qx) = (4.0, 6.0, -3.0);
    let beam = Beam2D::new(l, 0.0)?;
    let uniform = SpanLoad::Uniform {
        q: [qx, w],
        axes: LoadAxes::Local,
    };
    let trap = trapezoid([qx, w], [qx, w], None, None);
    let expected = [
        qx * l / 2.0,
        w * l / 2.0,
        w * l * l / 12.0,
        qx * l / 2.0,
        w * l / 2.0,
        -w * l * l / 12.0,
    ];
    vector(equivalent(&beam, &uniform)?, expected);
    vector(fixed(&beam, &uniform)?, expected.map(|v| -v));
    assert_eq!(equivalent(&beam, &trap)?, equivalent(&beam, &uniform)?);
    assert_eq!(
        to_member_load(&beam, &trap)?,
        to_member_load(&beam, &uniform)?
    );
    equilibrium(&beam, &uniform, [qx * l, w * l], w * l * l / 2.0);
    equilibrium(&beam, &trap, [qx * l, w * l], w * l * l / 2.0);
    Ok(())
}

#[test]
fn general_trapezoid_and_both_triangles() -> EResult<()> {
    let l = 5.0;
    let beam = Beam2D::new(l, 0.0)?;
    for (qi, qj) in [(2.0, 7.0), (0.0, 9.0), (9.0, 0.0), (-4.0, 3.0)] {
        let load = trapezoid([qi, qi], [qj, qj], None, None);
        vector(
            equivalent(&beam, &load)?,
            [
                l * (2.0 * qi + qj) / 6.0,
                l * (7.0 * qi + 3.0 * qj) / 20.0,
                l * l * (3.0 * qi + 2.0 * qj) / 60.0,
                l * (qi + 2.0 * qj) / 6.0,
                l * (3.0 * qi + 7.0 * qj) / 20.0,
                -l * l * (2.0 * qi + 3.0 * qj) / 60.0,
            ],
        );
        equilibrium(
            &beam,
            &load,
            [l * (qi + qj) / 2.0; 2],
            l * l * (qi + 2.0 * qj) / 6.0,
        );
    }
    // Increasing triangular load: axial wL/6 and wL/3; shear 3wL/20 and 7wL/20.
    let w = 6.0;
    vector(
        equivalent(&beam, &trapezoid([0.0; 2], [w; 2], None, None))?,
        [
            w * l / 6.0,
            3.0 * w * l / 20.0,
            w * l * l / 30.0,
            w * l / 3.0,
            7.0 * w * l / 20.0,
            -w * l * l / 20.0,
        ],
    );
    Ok(())
}

#[test]
fn center_asymmetric_and_endpoint_point_tables() -> EResult<()> {
    let (l, p, px) = (8.0, 10.0, -4.0);
    let beam = Beam2D::new(l, 0.0)?;
    for a in [0.0, l / 4.0, l / 2.0, 0.83 * l, l] {
        let b = l - a;
        let load = SpanLoad::Point {
            x: a,
            p: [px, p],
            axes: LoadAxes::Local,
        };
        vector(
            equivalent(&beam, &load)?,
            [
                px * b / l,
                p * b * b * (l + 2.0 * a) / math::powi(l, 3),
                p * a * b * b / (l * l),
                px * a / l,
                p * a * a * (l + 2.0 * b) / math::powi(l, 3),
                -p * a * a * b / (l * l),
            ],
        );
        equilibrium(&beam, &load, [px, p], p * a);
    }
    // At the center, each shear is P/2; each moment has magnitude PL/8.
    vector(
        equivalent(
            &beam,
            &SpanLoad::Point {
                x: l / 2.0,
                p: [0.0, p],
                axes: LoadAxes::Local,
            },
        )?,
        [0.0, p / 2.0, p * l / 8.0, 0.0, p / 2.0, -p * l / 8.0],
    );
    Ok(())
}

#[test]
fn partial_uniform_and_triangular_closed_tables() -> EResult<()> {
    let (l, w) = (4.0, 12.0);
    let beam = Beam2D::new(l, 0.0)?;
    let (a, b) = (Some(l / 4.0), Some(3.0 * l / 4.0));
    let uniform = trapezoid([w; 2], [w; 2], a, b);
    vector(
        equivalent(&beam, &uniform)?,
        [
            w * l / 4.0,
            w * l / 4.0,
            11.0 * w * l * l / 192.0,
            w * l / 4.0,
            w * l / 4.0,
            -11.0 * w * l * l / 192.0,
        ],
    );
    equilibrium(&beam, &uniform, [w * l / 2.0; 2], w * l * l / 4.0);
    let triangular = trapezoid([0.0; 2], [w; 2], a, b);
    vector(
        equivalent(&beam, &triangular)?,
        [
            5.0 * w * l / 48.0,
            61.0 * w * l / 640.0,
            31.0 * w * l * l / 1280.0,
            7.0 * w * l / 48.0,
            99.0 * w * l / 640.0,
            -127.0 * w * l * l / 3840.0,
        ],
    );
    equilibrium(&beam, &triangular, [w * l / 4.0; 2], 7.0 * w * l * l / 48.0);
    // Independent options: a=None or b=None are the respective ends.
    for load in [
        trapezoid([w; 2], [w; 2], None, Some(l / 2.0)),
        trapezoid([w; 2], [w; 2], Some(l / 2.0), None),
    ] {
        let center = if matches!(load, SpanLoad::Trapezoidal { a: None, .. }) {
            l / 4.0
        } else {
            3.0 * l / 4.0
        };
        equilibrium(&beam, &load, [w * l / 2.0; 2], w * l / 2.0 * center);
    }
    Ok(())
}

#[test]
fn superposition_full_and_partial() -> EResult<()> {
    let beam = Beam2D::new(7.0, 0.0)?;
    let (qi, qj) = ([3.0, -5.0], [-9.0, 8.0]);
    for (a, b) in [(None, None), (Some(0.7), Some(5.3))] {
        let trap = trapezoid(qi, qj, a, b);
        let uniform = trapezoid(qi, qi, a, b);
        let tri = trapezoid([0.0; 2], [qj[0] - qi[0], qj[1] - qi[1]], a, b);
        let f1 = fixed(&beam, &uniform)?;
        let f2 = fixed(&beam, &tri)?;
        vector(fixed(&beam, &trap)?, std::array::from_fn(|i| f1[i] + f2[i]));
    }
    Ok(())
}

#[test]
fn rotated_local_global_and_frame_element_agree() -> EResult<()> {
    let (l, angle) = (5.0, std::f64::consts::FRAC_PI_4);
    let beam = Beam2D::new(l, angle)?;
    let (s, c) = math::sin_cos(angle);
    let rotate = |[x, y]: [f64; 2]| [c * x - s * y, s * x + c * y];
    let local = [
        SpanLoad::Uniform {
            q: [2.0, -6.0],
            axes: LoadAxes::Local,
        },
        trapezoid([1.0, 3.0], [-2.0, 8.0], Some(1.0), Some(4.0)),
        SpanLoad::Point {
            x: 2.0,
            p: [-3.0, 9.0],
            axes: LoadAxes::Local,
        },
    ];
    let global = [
        SpanLoad::Uniform {
            q: rotate([2.0, -6.0]),
            axes: LoadAxes::Global,
        },
        SpanLoad::Trapezoidal {
            q_i: rotate([1.0, 3.0]),
            q_j: rotate([-2.0, 8.0]),
            a: Some(1.0),
            b: Some(4.0),
            axes: LoadAxes::Global,
        },
        SpanLoad::Point {
            x: 2.0,
            p: rotate([-3.0, 9.0]),
            axes: LoadAxes::Global,
        },
    ];
    for (loc, glo) in local.iter().zip(&global) {
        vector(equivalent(&beam, loc)?, equivalent(&beam, glo)?);
    }
    for angle in [0.0, angle, std::f64::consts::FRAC_PI_2, -2.1] {
        let beam = Beam2D::new(l, angle)?;
        let (s, c) = math::sin_cos(angle);
        let coords = [[0.0; 3], [l * c, l * s, 0.0]];
        let params = ElementParams::default();
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        let element = EulerFrame2D::new(200.0, 2.0, 3.0, 0.0, MomentReleases::default())?;
        for load in [&local[0], &global[0]] {
            let member_load = to_member_load(&beam, load)?;
            let own = beam.forces_to_global(equivalent(&beam, load)?)?;
            let existing = element.load_vector(&ctx, &member_load)?;
            for (a, b) in own.into_iter().zip(existing) {
                close(a, b);
            }
            // Recovered actions at u=0 are the restrained-end reactions.
            let recovered = element.recover(&ctx, &[0.0; 6], &[], &[member_load], &[])?;
            for (a, b) in fixed(&beam, load)?
                .into_iter()
                .zip(recovered.end_forces_local)
            {
                close(a, b);
            }
        }
    }
    Ok(())
}

#[test]
fn invalid_span_inputs_and_unsupported_conversion() -> EResult<()> {
    let beam = Beam2D::new(4.0, 0.0)?;
    for l in [0.0, -1.0] {
        assert_eq!(Beam2D::new(l, 0.0).unwrap_err().code, "ENG-E-0010");
    }
    for l in [f64::NAN, f64::INFINITY] {
        assert_eq!(Beam2D::new(l, 0.0).unwrap_err().code, "ENG-E-0008");
    }
    assert_eq!(Beam2D::new(1.0, f64::NAN).unwrap_err().code, "ENG-E-0008");
    for (a, b) in [(-1.0, 2.0), (2.0, 2.0), (3.0, 2.0), (0.0, 5.0), (5.0, 6.0)] {
        let load = trapezoid([1.0; 2], [1.0; 2], Some(a), Some(b));
        assert_eq!(equivalent(&beam, &load).unwrap_err().code, "ENG-E-0010");
        assert_eq!(to_member_load(&beam, &load).unwrap_err().code, "ENG-E-0010");
    }
    for x in [-0.1, 4.1] {
        assert_eq!(
            fixed(
                &beam,
                &SpanLoad::Point {
                    x,
                    p: [1.0; 2],
                    axes: LoadAxes::Local
                }
            )
            .unwrap_err()
            .code,
            "ENG-E-0010"
        );
    }
    for load in [
        SpanLoad::Uniform {
            q: [f64::INFINITY, 0.0],
            axes: LoadAxes::Local,
        },
        trapezoid([f64::NAN, 0.0], [1.0; 2], None, None),
        trapezoid([1.0; 2], [f64::INFINITY, 0.0], None, None),
        trapezoid([1.0; 2], [1.0; 2], Some(f64::NAN), None),
        trapezoid([1.0; 2], [1.0; 2], None, Some(f64::INFINITY)),
        SpanLoad::Point {
            x: f64::NAN,
            p: [1.0; 2],
            axes: LoadAxes::Local,
        },
        SpanLoad::Point {
            x: 1.0,
            p: [f64::NAN, 0.0],
            axes: LoadAxes::Global,
        },
    ] {
        assert_eq!(equivalent(&beam, &load).unwrap_err().code, "ENG-E-0008");
        assert_eq!(to_member_load(&beam, &load).unwrap_err().code, "ENG-E-0008");
    }
    for load in [
        trapezoid([1.0; 2], [2.0; 2], None, None),
        trapezoid([1.0; 2], [1.0; 2], Some(1.0), None),
        SpanLoad::Point {
            x: 2.0,
            p: [1.0; 2],
            axes: LoadAxes::Local,
        },
    ] {
        assert_eq!(to_member_load(&beam, &load).unwrap_err().code, "ENG-E-0013");
    }
    assert_eq!(
        beam.forces_to_global([f64::INFINITY; 6]).unwrap_err().code,
        "ENG-E-0008"
    );
    Ok(())
}

#[test]
fn extreme_inputs_do_not_panic_or_return_nonfinite_success() -> EResult<()> {
    let tiny = Beam2D::new(1e-300, 0.0)?;
    let uniform = SpanLoad::Uniform {
        q: [1e308; 2],
        axes: LoadAxes::Local,
    };
    let f = equivalent(&tiny, &uniform)?;
    close(f[0], 5e7);
    assert!((f[2] / (1e-292 / 12.0) - 1.0).abs() < 1e-12);
    for l in [1e-300, 1e-100, 1.0, 1e100, 1e308] {
        let beam = Beam2D::new(l, 0.0)?;
        for load in [
            uniform,
            trapezoid([0.0; 2], [1e308; 2], None, None),
            SpanLoad::Point {
                x: l / 2.0,
                p: [1e308; 2],
                axes: LoadAxes::Local,
            },
        ] {
            match equivalent(&beam, &load) {
                Ok(f) => assert!(f.iter().all(|v| v.is_finite())),
                Err(e) => assert_eq!(e.code, "ENG-E-0008"),
            }
        }
    }
    assert_eq!(
        equivalent(&Beam2D::new(10.0, 0.0)?, &uniform)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    // Zero load with extreme L does not generate 0·infinity either.
    vector(
        equivalent(
            &Beam2D::new(1e308, 0.0)?,
            &SpanLoad::Uniform {
                q: [0.0; 2],
                axes: LoadAxes::Local,
            },
        )?,
        [0.0; 6],
    );
    Ok(())
}
