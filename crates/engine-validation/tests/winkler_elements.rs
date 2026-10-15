//! Integración polinómica independiente, contrato, transformaciones y negativos.
use engine_contracts::{
    Element, ElementCtx, ElementFactory, ElementParams, JsonValue, MassKind, MemberLoad, ModelView,
};
use engine_core::math;
use engine_core::{Dim, EResult, NodeId};
use engine_elements_frame::{EulerFrame2D, WinklerFrame2D};
use engine_elements_frame3d::{EulerFrame3D, WinklerFrame3D};
use engine_geo::winkler;
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use std::collections::BTreeMap;

fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 2e-9 + 2e-10 * b.abs(),
        "{a:.14e} != {b:.14e}"
    );
}
fn code<T>(result: EResult<T>, expected: &str) {
    assert_eq!(result.err().expect("debe rechazar").code, expected);
}
fn base(spatial: bool, orientation: [f64; 3]) -> Box<dyn Element> {
    if spatial {
        Box::new(
            EulerFrame3D::new(
                200.0,
                80.0,
                2.0,
                1.5,
                3.0,
                4.0,
                3.0,
                orientation,
                Default::default(),
            )
            .unwrap(),
        )
    } else {
        Box::new(EulerFrame2D::new(200.0, 2.0, 4.0, 3.0, Default::default()).unwrap())
    }
}
fn bed(spatial: bool, orientation: [f64; 3], k: [f64; 2]) -> Box<dyn Element> {
    if spatial {
        Box::new(
            WinklerFrame3D::new(
                EulerFrame3D::new(
                    200.0,
                    80.0,
                    2.0,
                    1.5,
                    3.0,
                    4.0,
                    3.0,
                    orientation,
                    Default::default(),
                )
                .unwrap(),
                k[0],
                k[1],
            )
            .unwrap(),
        )
    } else {
        Box::new(
            WinklerFrame2D::new(
                EulerFrame2D::new(200.0, 2.0, 4.0, 3.0, Default::default()).unwrap(),
                k[0],
            )
            .unwrap(),
        )
    }
}
fn rotate(p: [f64; 3]) -> [f64; 3] {
    // Qz(.47) Qx(.63), propia; para D2 se usa Qz por separado.
    let (s, c) = math::sin_cos(0.63_f64);
    let y = c * p[1] - s * p[2];
    let z = s * p[1] + c * p[2];
    let (s, c) = math::sin_cos(0.47_f64);
    [c * p[0] - s * y, s * p[0] + c * y, z]
}
fn rotate2(p: [f64; 3]) -> [f64; 3] {
    let (s, c) = math::sin_cos(0.47_f64);
    [c * p[0] - s * p[1], s * p[0] + c * p[1], p[2]]
}
fn rotated(spatial: bool, v: &[f64]) -> Vec<f64> {
    let mut out = Vec::new();
    if spatial {
        for triplet in v.chunks_exact(3) {
            out.extend(rotate([triplet[0], triplet[1], triplet[2]]));
        }
    } else {
        for node in v.chunks_exact(3) {
            out.extend(rotate2([node[0], node[1], 0.0])[..2].iter().copied());
            out.push(node[2]);
        }
    }
    out
}
fn multiply(k: &DMat, u: &[f64]) -> Vec<f64> {
    k.values()
        .chunks_exact(k.cols())
        .map(|row| row.iter().zip(u).map(|(a, b)| a * b).sum())
        .collect()
}

#[test]
fn foundation_is_exact_hermite_gram_and_positive_definite_on_each_transverse_plane() {
    for l in [0.3, 2.7, 6.1] {
        for k in [0.0, 1.0, 42.0] {
            let polys = [
                [1.0, 0.0, -3.0, 2.0],
                [0.0, l, -2.0 * l, l],
                [0.0, 0.0, 3.0, -2.0],
                [0.0, 0.0, -l, l],
            ];
            let gram = winkler::matrix(k, l).unwrap();
            for i in 0..4 {
                for j in 0..4 {
                    let mut exact = 0.0;
                    for a in 0..4 {
                        for b in 0..4 {
                            exact += polys[i][a] * polys[j][b] / (a + b + 1) as f64;
                        }
                    }
                    close(gram[i][j], k * l * exact);
                    assert_eq!(gram[i][j], gram[j][i]);
                }
            }
            if k > 0.0 {
                // Cholesky independiente: exige los cuatro pivotes estrictamente positivos.
                let mut chol = [[0.0; 4]; 4];
                for i in 0..4 {
                    for j in 0..=i {
                        let residual =
                            gram[i][j] - (0..j).map(|p| chol[i][p] * chol[j][p]).sum::<f64>();
                        if i == j {
                            assert!(residual > 0.0);
                            chol[i][j] = math::sqrt(residual);
                        } else {
                            chol[i][j] = residual / chol[j][j];
                        }
                    }
                }
            } else {
                assert_eq!(gram, [[0.0; 4]; 4]);
            }
            let params = ElementParams::default();
            let coords = [[0.0; 3], [l, 0.0, 0.0]];
            let ctx = ElementCtx {
                dim: Dim::D3,
                coords: &coords,
                params: &params,
            };
            let beam = WinklerFrame3D::new(
                EulerFrame3D::new(
                    200.0,
                    80.0,
                    2.0,
                    1.5,
                    3.0,
                    4.0,
                    0.0,
                    [0.0, 0.0, 1.0],
                    Default::default(),
                )
                .unwrap(),
                k,
                2.0 * k,
            )
            .unwrap();
            let full = beam.foundation_matrix(&ctx).unwrap();
            SymMatrix::new(full.clone()).unwrap();
            let y = [1, 5, 7, 11];
            let z = [2, 4, 8, 10];
            let signs = [1.0, -1.0, 1.0, -1.0];
            for i in 0..4 {
                for j in 0..4 {
                    close(full.get(y[i], y[j]).unwrap(), gram[i][j]);
                    close(
                        full.get(z[i], z[j]).unwrap(),
                        2.0 * signs[i] * signs[j] * gram[i][j],
                    );
                    assert_eq!(full.get(y[i], z[j]), Some(0.0));
                }
            }
            for i in [0, 3, 6, 9] {
                for j in 0..12 {
                    assert_eq!(full.get(i, j), Some(0.0));
                }
            }
            let ctx2 = ElementCtx {
                dim: Dim::D2,
                ..ctx
            };
            let beam2 = WinklerFrame2D::new(
                EulerFrame2D::new(200.0, 2.0, 4.0, 0.0, Default::default()).unwrap(),
                k,
            )
            .unwrap();
            let full2 = beam2.foundation_matrix(&ctx2).unwrap();
            for (i, r) in [1, 2, 4, 5].into_iter().enumerate() {
                for (j, s) in [1, 2, 4, 5].into_iter().enumerate() {
                    close(full2.get(r, s).unwrap(), gram[i][j]);
                }
            }
        }
    }
}

#[test]
fn zero_bed_reproduces_every_frame_operation_in_2d_and_3d() {
    for spatial in [false, true] {
        let orientation = [0.2, 0.3, 1.0];
        let a = base(spatial, orientation);
        let b = bed(spatial, orientation, [0.0; 2]);
        let params = ElementParams::default();
        let coords = [
            [0.3, -0.5, 0.0],
            [2.4, 1.1, if spatial { 0.7 } else { 0.0 }],
        ];
        let ctx = ElementCtx {
            dim: if spatial { Dim::D3 } else { Dim::D2 },
            coords: &coords,
            params: &params,
        };
        let n = 2 * a.dofs_per_node().len();
        let u: Vec<_> = (0..n).map(|i| 0.001 * (i as f64 - 3.0)).collect();
        let q = MemberLoad {
            force_per_length: [1.0, -3.0, if spatial { 2.0 } else { 0.0 }],
            moment_per_length: if spatial {
                [0.3, 0.4, 0.5]
            } else {
                [0.0, 0.0, 0.5]
            },
        };
        assert_eq!(
            a.respond(&ctx, &u, &[]).unwrap(),
            b.respond(&ctx, &u, &[]).unwrap()
        );
        for kind in [MassKind::Consistent, MassKind::Lumped] {
            assert_eq!(a.mass(&ctx, kind).unwrap(), b.mass(&ctx, kind).unwrap());
        }
        assert_eq!(
            a.geometric_stiffness(&ctx, &u, &[]).unwrap(),
            b.geometric_stiffness(&ctx, &u, &[]).unwrap()
        );
        assert_eq!(
            a.load_vector(&ctx, &q).unwrap(),
            b.load_vector(&ctx, &q).unwrap()
        );
        assert_eq!(
            a.recover(
                &ctx,
                &u,
                &[],
                std::slice::from_ref(&q),
                &[1.0, 0.0, 0.37, 0.37]
            )
            .unwrap(),
            b.recover(&ctx, &u, &[], &[q], &[1.0, 0.0, 0.37, 0.37])
                .unwrap()
        );
    }
}

#[test]
fn grounded_element_has_consistent_tangent_repeatability_and_rotation_covariance() {
    for spatial in [false, true] {
        let orientation = [0.2, 0.3, 1.0];
        let b = bed(spatial, orientation, [7.0, 19.0]);
        let params = ElementParams::default();
        let coords = [
            [0.3, -0.5, 0.0],
            [2.4, 1.1, if spatial { 0.7 } else { 0.0 }],
        ];
        let ctx = ElementCtx {
            dim: if spatial { Dim::D3 } else { Dim::D2 },
            coords: &coords,
            params: &params,
        };
        let n = 2 * b.dofs_per_node().len();
        let u: Vec<_> = (0..n).map(|i| 0.001 * (i as f64 - 3.0)).collect();
        let r = b.respond(&ctx, &u, &[]).unwrap();
        assert_eq!(r, b.respond(&ctx, &u, &[]).unwrap());
        assert_eq!(b.contract(), engine_core::ContractId::new("element", 1, 0));
        assert!(r.trial_state.is_empty());
        assert_eq!(
            b.kind(),
            if spatial {
                "frame3d.winkler"
            } else {
                "frame2d.winkler"
            }
        );
        SymMatrix::new(r.tangent.clone()).unwrap();
        for j in 0..n {
            let mut up = u.clone();
            let mut um = u.clone();
            up[j] += 1e-6;
            um[j] -= 1e-6;
            let fp = b.respond(&ctx, &up, &[]).unwrap().internal_force;
            let fm = b.respond(&ctx, &um, &[]).unwrap().internal_force;
            for i in 0..n {
                close((fp[i] - fm[i]) / 2e-6, r.tangent.get(i, j).unwrap());
            }
        }
        let qr = if spatial { rotate } else { rotate2 };
        let rc = coords.map(qr);
        let rb = bed(spatial, qr(orientation), [7.0, 19.0]);
        let rctx = ElementCtx { coords: &rc, ..ctx };
        let rotated_response = rb.respond(&rctx, &rotated(spatial, &u), &[]).unwrap();
        for (a, b) in rotated(spatial, &r.internal_force)
            .iter()
            .zip(&rotated_response.internal_force)
        {
            close(*a, *b);
        }
        // Verificar congruencia de K y M por sus acciones sobre una base completa.
        for j in 0..n {
            let mut e = vec![0.0; n];
            e[j] = 1.0;
            for (a, b) in rotated(spatial, &multiply(&r.tangent, &e))
                .iter()
                .zip(multiply(&rotated_response.tangent, &rotated(spatial, &e)))
            {
                close(*a, b);
            }
        }
        for kind in [MassKind::Consistent, MassKind::Lumped] {
            let m = b.mass(&ctx, kind).unwrap().unwrap();
            let rm = rb.mass(&rctx, kind).unwrap().unwrap();
            for j in 0..n {
                let mut e = vec![0.0; n];
                e[j] = 1.0;
                for (a, b) in rotated(spatial, &multiply(&m, &e))
                    .iter()
                    .zip(multiply(&rm, &rotated(spatial, &e)))
                {
                    close(*a, b);
                }
            }
        }
        let mut translation = vec![0.0; n];
        translation[1] = 1.0;
        translation[n / 2 + 1] = 1.0;
        assert!(b
            .respond(&ctx, &translation, &[])
            .unwrap()
            .internal_force
            .iter()
            .any(|f| f.abs() > 1.0));
    }
}

#[test]
fn both_bending_planes_are_independent_including_zero_stiffness_in_one_plane() {
    let params = ElementParams::default();
    let coords = [[0.0; 3], [2.7, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D3,
        coords: &coords,
        params: &params,
    };
    let a = base(true, [0.0, 0.0, 1.0]);
    for k in [[7.0, 0.0], [0.0, 19.0], [7.0, 19.0]] {
        let b = bed(true, [0.0, 0.0, 1.0], k);
        for (plane, ids) in [[1, 5, 7, 11], [2, 4, 8, 10]].into_iter().enumerate() {
            let mut u = [0.0; 12];
            for (i, &j) in ids.iter().enumerate() {
                u[j] = (i as f64 + 1.0) * 0.003;
            }
            let response = b.respond(&ctx, &u, &[]).unwrap();
            let frame = a.respond(&ctx, &u, &[]).unwrap();
            for i in 0..12 {
                if !ids.contains(&i) {
                    close(response.internal_force[i], 0.0);
                }
            }
            if k[plane] == 0.0 {
                assert_eq!(response.internal_force, frame.internal_force);
                for &r in &ids {
                    for &s in &ids {
                        assert_eq!(response.tangent.get(r, s), frame.tangent.get(r, s));
                    }
                }
            } else {
                assert!(response
                    .internal_force
                    .iter()
                    .zip(frame.internal_force)
                    .any(|(a, b)| (a - b).abs() > 1e-3));
            }
        }
    }
}

fn fields(spatial: bool) -> BTreeMap<String, JsonValue> {
    let mut p = BTreeMap::from([
        ("E".into(), JsonValue::Number(200.0)),
        ("A".into(), JsonValue::Number(2.0)),
        ("rho".into(), JsonValue::Number(3.0)),
    ]);
    if spatial {
        for (key, v) in [
            ("G", 80.0),
            ("J", 1.5),
            ("Iy", 3.0),
            ("Iz", 4.0),
            ("ky", 7.0),
            ("kz", 19.0),
        ] {
            p.insert(key.into(), JsonValue::Number(v));
        }
        p.insert(
            "orientation".into(),
            JsonValue::Array(vec![
                JsonValue::Number(0.0),
                JsonValue::Number(0.0),
                JsonValue::Number(1.0),
            ]),
        );
    } else {
        p.insert("I".into(), JsonValue::Number(4.0));
        p.insert("k".into(), JsonValue::Number(7.0));
    }
    p
}
#[test]
fn factory_schema_and_invalid_inputs_use_catalogued_diagnostics() {
    for spatial in [false, true] {
        let factory: Box<dyn ElementFactory> = if spatial {
            Box::new(engine_elements_frame3d::WinklerFactory)
        } else {
            Box::new(engine_elements_frame::WinklerFactory)
        };
        let model = ModelView {
            dim: if spatial { Dim::D3 } else { Dim::D2 },
            nodes: &[],
        };
        let schema = factory.params_schema();
        assert!(schema.contains("\"minimum\":0"));
        assert!(schema.contains("N/m²"));
        assert!(schema.contains("\"additionalProperties\":false"));
        let element = factory
            .build(&JsonValue::Object(fields(spatial)), &model)
            .unwrap();
        assert_eq!(element.kind(), factory.kind());
        let keys: &[&str] = if spatial { &["ky", "kz"] } else { &["k"] };
        for &key in keys {
            for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut p = fields(spatial);
                p.insert(key.into(), JsonValue::Number(bad));
                code(
                    factory.build(&JsonValue::Object(p), &model),
                    if bad.is_finite() {
                        "ENG-E-0010"
                    } else {
                        "ENG-E-0008"
                    },
                );
            }
            let mut p = fields(spatial);
            p.remove(key);
            code(factory.build(&JsonValue::Object(p), &model), "ENG-E-0010");
            let mut p = fields(spatial);
            p.insert(key.into(), JsonValue::Bool(true));
            code(factory.build(&JsonValue::Object(p), &model), "ENG-E-0010");
        }
        for key in ["typo", "release_i", "release_j"] {
            let mut p = fields(spatial);
            p.insert(key.into(), JsonValue::Bool(false));
            code(factory.build(&JsonValue::Object(p), &model), "ENG-E-0010");
        }
        code(factory.build(&JsonValue::Null, &model), "ENG-E-0010");
        let wrong = ModelView {
            dim: if spatial { Dim::D2 } else { Dim::D3 },
            nodes: &[],
        };
        code(
            factory.build(&JsonValue::Object(fields(spatial)), &wrong),
            "ENG-E-0010",
        );
        let duplicate = [(NodeId(1), [0.0; 3]), (NodeId(1), [1.0, 0.0, 0.0])];
        code(
            factory.build(
                &JsonValue::Object(fields(spatial)),
                &ModelView {
                    nodes: &duplicate,
                    ..model
                },
            ),
            "ENG-E-0002",
        );
        let invalid = [(NodeId(1), [f64::NAN, 0.0, 0.0])];
        code(
            factory.build(
                &JsonValue::Object(fields(spatial)),
                &ModelView {
                    nodes: &invalid,
                    ..model
                },
            ),
            "ENG-E-0008",
        );
        let params = ElementParams::default();
        let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
        let ctx = ElementCtx {
            dim: model.dim,
            coords: &coords,
            params: &params,
        };
        let n = element.dofs_per_node().len() * 2;
        let u = vec![0.0; n];
        code(element.respond(&ctx, &u, &[0.0]), "ENG-E-0010");
        code(element.respond(&ctx, &u[..n - 1], &[]), "ENG-E-0010");
        let mut nan = u.clone();
        nan[1] = f64::NAN;
        code(element.respond(&ctx, &nan, &[]), "ENG-E-0008");
        code(element.recover(&ctx, &u, &[], &[], &[1.01]), "ENG-E-0010");
        code(
            element.recover(&ctx, &u, &[], &[], &[f64::NAN]),
            "ENG-E-0008",
        );
        code(
            element.respond(
                &ElementCtx {
                    coords: &[[0.0; 3]; 2],
                    ..ctx
                },
                &u,
                &[],
            ),
            "ENG-E-0005",
        );
    }
    let released = EulerFrame2D::new(
        200.0,
        2.0,
        4.0,
        0.0,
        engine_elements_frame::MomentReleases { i: true, j: false },
    )
    .unwrap();
    code(WinklerFrame2D::new(released, 1.0), "ENG-E-0010");
    let released = EulerFrame3D::new(
        200.0,
        80.0,
        2.0,
        1.5,
        3.0,
        4.0,
        0.0,
        [0.0, 0.0, 1.0],
        engine_elements_frame3d::MomentReleases {
            i: [false, true, false],
            j: [false; 3],
        },
    )
    .unwrap();
    code(WinklerFrame3D::new(released, 1.0, 1.0), "ENG-E-0010");
    code(winkler::matrix(f64::MAX, 10.0), "ENG-E-0008");
    code(winkler::matrix(f64::from_bits(1), 1.0), "ENG-E-0010");
    code(winkler::matrix(1.0, 0.0), "ENG-E-0005");
    code(winkler::matrix(1.0, f64::NAN), "ENG-E-0008");
    code(
        winkler::integrals(1.0, 2.0, [f64::MAX; 4], 0.5),
        "ENG-E-0008",
    );
    // Resolver el bloque positivo comprueba además que no se añade regularización.
    let gram = winkler::matrix(2.0, 3.0).unwrap();
    let gram = DMat::new(4, 4, gram.into_iter().flatten().collect()).unwrap();
    let rhs = DMat::new(4, 1, vec![1.0; 4]).unwrap();
    let x = ReferenceBackend.solve_dense(&gram, &rhs).unwrap();
    for v in multiply(&gram, x.values()) {
        close(v, 1.0);
    }
}

#[test]
fn recovery_with_distributed_loads_matches_independent_quadrature_of_soil_force() {
    for spatial in [false, true] {
        let beam = bed(spatial, [0.0, 0.0, 1.0], [7.0, 19.0]);
        let l = 2.7;
        let params = ElementParams::default();
        let coords = [[0.0; 3], [l, 0.0, 0.0]];
        let ctx = ElementCtx {
            dim: if spatial { Dim::D3 } else { Dim::D2 },
            coords: &coords,
            params: &params,
        };
        let stride = beam.dofs_per_node().len();
        let u: Vec<_> = (0..2 * stride).map(|i| 0.002 * (i as f64 - 2.0)).collect();
        let load = MemberLoad {
            force_per_length: [1.0, -3.0, if spatial { 2.0 } else { 0.0 }],
            moment_per_length: if spatial {
                [0.3, 0.4, 0.5]
            } else {
                [0.0, 0.0, 0.5]
            },
        };
        let out = beam
            .recover(
                &ctx,
                &u,
                &[],
                std::slice::from_ref(&load),
                &[0.0, 0.2, 0.6, 1.0],
            )
            .unwrap();
        let response = beam.respond(&ctx, &u, &[]).unwrap();
        let nodal = beam.load_vector(&ctx, &load).unwrap();
        for (i, &f) in out.end_forces_local.iter().enumerate() {
            close(f, response.internal_force[i] - nodal[i]);
        }
        for (plane, k) in [7.0, 19.0]
            .into_iter()
            .enumerate()
            .take(if spatial { 2 } else { 1 })
        {
            let shear = plane + 1;
            let rotation = if plane == 0 { stride - 1 } else { 4 };
            let moment = if plane == 0 { 5 } else { 4 };
            let sign = if plane == 0 { 1.0 } else { -1.0 };
            for station in &out.stations {
                let x = station.position * l;
                let mut force = 0.0;
                let mut torque = 0.0;
                // Gauss sobre [0,x], independiente de los antiderivados de producción.
                for (r, weight) in [
                    (0.5 - math::sqrt(3.0_f64 / 5.0) / 2.0, 5.0 / 18.0),
                    (0.5, 4.0 / 9.0),
                    (0.5 + math::sqrt(3.0_f64 / 5.0) / 2.0, 5.0 / 18.0),
                ] {
                    let s = r * x;
                    let t = s / l;
                    let v = (1.0 - 3.0 * t * t + 2.0 * t * t * t) * u[shear]
                        + l * (t - 2.0 * t * t + t * t * t) * sign * u[rotation]
                        + (3.0 * t * t - 2.0 * t * t * t) * u[stride + shear]
                        + l * (-t * t + t * t * t) * sign * u[stride + rotation];
                    force += x * weight * k * v;
                    torque += x * weight * k * v * (x - s);
                }
                close(
                    station.forces_local[shear],
                    -out.end_forces_local[shear] - load.force_per_length[plane + 1] * x + force,
                );
                close(
                    station.forces_local[moment],
                    -out.end_forces_local[rotation]
                        + sign
                            * (out.end_forces_local[shear] * x
                                + load.force_per_length[plane + 1] * x * x / 2.0
                                - torque)
                        - load.moment_per_length[moment - 3] * x,
                );
            }
        }
    }
}
