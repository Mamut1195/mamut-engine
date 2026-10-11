//! Invariantes con tolerancias explícitas, conformidad y validación negativa.
use engine_contracts::{
    conformance::{check_element, check_element_factory},
    Element, ElementCtx, ElementFactory, ElementParams, JsonValue, MassKind, MemberLoad, ModelView,
};
use engine_core::{Dim, EResult, NodeId};
use engine_elements_truss::{LinearTruss, TrussFactory};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use std::collections::BTreeMap;

const TOL: f64 = 1e-10; // Absoluta para matrices/fuerzas SI de estos fixtures.
const CITATION: &str =
    "provenance/ENG-ELM-001.md, Derivaciones propias; bibliografía exacta PENDIENTE";
const SOURCE: &str = "analítico: energía axial, integración de formas y equilibrio propios";
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= TOL, "{a} != {b}, tol={TOL}");
}
fn code<T>(result: EResult<T>, expected: &str) {
    assert_eq!(result.err().expect("debe fallar").code, expected);
}
fn bar(dim: Dim) -> LinearTruss {
    LinearTruss::new(dim, 200.0, 2.0, 3.0).unwrap()
}
fn ctx<'a>(dim: Dim, coords: &'a [[f64; 3]], params: &'a ElementParams) -> ElementCtx<'a> {
    ElementCtx {
        dim,
        coords,
        params,
    }
}
fn multiply(m: &DMat, u: &[f64]) -> Vec<f64> {
    ReferenceBackend
        .matmul(m, &DMat::new(u.len(), 1, u.to_vec()).unwrap())
        .unwrap()
        .values()
        .to_vec()
}
fn symmetric(m: &DMat) {
    SymMatrix::new(m.clone()).unwrap();
}
fn params() -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("E".into(), JsonValue::Number(200.0)),
        ("A".into(), JsonValue::Number(2.0)),
        ("rho".into(), JsonValue::Number(3.0)),
    ]))
}

#[test]
fn both_kinds_pass_element_and_factory_conformance() {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let coordinates = if dim == Dim::D2 {
            [[0.2, -0.4, 0.0], [2.0, 1.3, 0.0]]
        } else {
            [[0.2, -0.4, 0.3], [2.0, 1.3, -0.5]]
        };
        let c = ctx(dim, &coordinates, &p);
        let element = bar(dim);
        let n = 2 * element.dofs_per_node().len();
        let u: Vec<_> = (0..n).map(|i| (i as f64 - 2.0) * 0.001).collect();
        let samples = vec![
            vec![0.0; n],
            u.clone(),
            u.iter().map(|u| -u).collect(),
            vec![0.0; n],
        ];
        let issues = check_element(&element, &c, &samples, 1e-7);
        assert!(issues.is_empty(), "{issues:?}");
        let nodes = [(NodeId(1), coordinates[0]), (NodeId(2), coordinates[1])];
        let model = ModelView { dim, nodes: &nodes };
        let factory = TrussFactory::new(dim);
        let issues = check_element_factory(&factory, &params(), &model, &c, &samples, 1e-7);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(element.state_len(), 0);
        assert_eq!(element.initial_state(), Vec::<f64>::new());
        assert_eq!(element.material().young_modulus(), 200.0);
        assert_eq!(element.area(), 2.0);
        assert_eq!(element.density(), 3.0);
        assert_eq!(element.dim(), dim);
        let initial = element.respond(&c, &u, &[]).unwrap();
        for _ in 0..10 {
            assert_eq!(initial, element.respond(&c, &u, &[]).unwrap());
        }
    }
}

#[test]
fn rigid_translations_and_infinitesimal_rotations_annihilate_k() {
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let coordinates = if dim == Dim::D2 {
            [[1.0, 2.0, 0.0], [4.0, 6.0, 0.0]]
        } else {
            [[1.0, 2.0, 3.0], [4.0, 6.0, 7.0]]
        };
        let element = bar(dim);
        let axes = element.dofs_per_node().len();
        let c = ctx(dim, &coordinates, &p);
        let k = element
            .respond(&c, &vec![0.0; 2 * axes], &[])
            .unwrap()
            .tangent;
        symmetric(&k);
        for axis in 0..axes {
            let mut u = vec![0.0; 2 * axes];
            u[axis] = 1.0;
            u[axes + axis] = 1.0;
            for f in multiply(&k, &u) {
                close(f, 0.0);
            }
        }
        for axis in if dim == Dim::D2 { 2..3 } else { 0..3 } {
            let u: Vec<_> = coordinates
                .iter()
                .flat_map(|p| {
                    let mut t = [0.0; 3];
                    t[(axis + 1) % 3] = -p[(axis + 2) % 3];
                    t[(axis + 2) % 3] = p[(axis + 1) % 3];
                    t[..axes].to_vec()
                })
                .collect();
            for f in multiply(&k, &u) {
                close(f, 0.0);
            }
            for f in element.respond(&c, &u, &[]).unwrap().internal_force {
                close(f, 0.0);
            }
        }
    }
}

#[test]
fn mass_entries_total_per_translation_and_no_mass() {
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let coordinates = [[0.0; 3], [2.0, 0.0, 0.0]];
        let c = ctx(dim, &coordinates, &p);
        let element = bar(dim);
        let axes = element.dofs_per_node().len();
        let n = 2 * axes;
        for kind in [MassKind::Lumped, MassKind::Consistent] {
            let m = element.mass(&c, kind).unwrap().unwrap();
            symmetric(&m);
            let (diag, off) = if kind == MassKind::Lumped {
                (6.0, 0.0)
            } else {
                (4.0, 2.0)
            };
            for i in 0..n {
                for j in 0..n {
                    let expected = if i == j {
                        diag
                    } else if i % axes == j % axes {
                        off
                    } else {
                        0.0
                    };
                    close(m.get(i, j).unwrap(), expected);
                }
            }
            for axis in 0..axes {
                let mut velocity = vec![0.0; n];
                velocity[axis] = 1.0;
                velocity[axes + axis] = 1.0;
                let mv = multiply(&m, &velocity);
                close(velocity.iter().zip(mv).map(|(a, b)| a * b).sum(), 12.0);
            }
            let zero = LinearTruss::new(dim, 200.0, 2.0, 0.0).unwrap();
            assert!(zero.mass(&c, kind).unwrap().is_none());
        }
    }
}

#[test]
fn geometric_stiffness_spectrum_sign_rank_and_nullspace() {
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let coordinates = [[0.0; 3], [2.0, 0.0, 0.0]];
        let c = ctx(dim, &coordinates, &p);
        let element = bar(dim);
        let axes = element.dofs_per_node().len();
        let order = 2 * axes;
        for extension in [0.06, -0.06, 0.0] {
            let mut u = vec![0.0; order];
            u[axes] = extension;
            // N=200δ, N/L=100δ; each transverse eigenvalue is 2N/L.
            let kg = element.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
            symmetric(&kg);
            for axis in 0..axes {
                let mut rigid = vec![0.0; order];
                rigid[axis] = 1.0;
                rigid[axes + axis] = 1.0;
                for f in multiply(&kg, &rigid) {
                    close(f, 0.0);
                }
                let mut relative = vec![0.0; order];
                relative[axis] = -1.0;
                relative[axes + axis] = 1.0;
                let lambda = if axis == 0 { 0.0 } else { 200.0 * extension };
                for (f, v) in multiply(&kg, &relative).iter().zip(relative) {
                    close(*f, lambda * v);
                }
            }
            let identity = DMat::new(
                order,
                order,
                (0..order * order)
                    .map(|i| if i / order == i % order { 1.0 } else { 0.0 })
                    .collect(),
            )
            .unwrap();
            let eigen = ReferenceBackend
                .generalized_eigen(
                    &SymMatrix::new(kg).unwrap(),
                    &SymMatrix::new(identity).unwrap(),
                )
                .unwrap();
            let rank = eigen.eigenvalues.iter().filter(|v| v.abs() > 1e-9).count();
            assert_eq!(rank, if extension == 0.0 { 0 } else { axes - 1 });
            for lambda in eigen.eigenvalues {
                close(
                    lambda,
                    if lambda.abs() < 1e-9 {
                        0.0
                    } else {
                        200.0 * extension
                    },
                );
            }
        }
    }
}

#[test]
fn uniform_axial_load_recovery_and_vertical_self_weight() {
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let coordinates = [[0.0; 3], [2.0, 0.0, 0.0]];
        let c = ctx(dim, &coordinates, &p);
        let element = bar(dim);
        let axes = element.dofs_per_node().len();
        let load = MemberLoad {
            force_per_length: [4.0, 0.0, 0.0],
            moment_per_length: [0.0; 3],
        };
        let v = element.load_vector(&c, &load).unwrap();
        for (i, f) in v.iter().enumerate() {
            close(*f, if i % axes == 0 { 4.0 } else { 0.0 });
        }
        let mut u = vec![0.0; 2 * axes];
        u[axes] = 0.06;
        let out = element
            .recover(&c, &u, &[], &[load.clone(), load], &[1.0, 0.0, 0.5])
            .unwrap();
        for (s, expected) in out.stations.iter().zip([4.0, 20.0, 12.0]) {
            close(s.forces_local[0], expected);
        }
        close(out.end_forces_local[0], -20.0);
        close(out.end_forces_local[axes], 4.0);
        close(
            out.end_forces_local[0] + out.end_forces_local[axes] + 16.0,
            0.0,
        );
    }
    // rho A g=3*2*(-10)=-60 N/m, L=2: -60 N per nodo.
    let element = bar(Dim::D3);
    let coordinates = [[0.0; 3], [0.0, 0.0, 2.0]];
    let c = ctx(Dim::D3, &coordinates, &p);
    let weight = MemberLoad {
        force_per_length: [0.0, 0.0, -60.0],
        moment_per_length: [0.0; 3],
    };
    assert_eq!(
        element.load_vector(&c, &weight).unwrap(),
        vec![0.0, 0.0, -60.0, 0.0, 0.0, -60.0]
    );
    let out = element
        .recover(&c, &[0.0; 6], &[], &[weight], &[0.0, 0.5, 1.0])
        .unwrap();
    for (s, n) in out.stations.iter().zip([-60.0, 0.0, 60.0]) {
        close(s.forces_local[0], n);
    }
}

#[test]
fn invalid_properties_dimensions_coordinates_states_and_stations() {
    for dim in [Dim::D2, Dim::D3] {
        for (e, a, rho, expected) in [
            (0.0, 2.0, 3.0, "ENG-E-0010"),
            (-1.0, 2.0, 3.0, "ENG-E-0010"),
            (200.0, 0.0, 3.0, "ENG-E-0010"),
            (200.0, -1.0, 3.0, "ENG-E-0010"),
            (200.0, 2.0, -1.0, "ENG-E-0010"),
            (f64::MAX, 2.0, 3.0, "ENG-E-0008"),
            (f64::MIN_POSITIVE, f64::MIN_POSITIVE, 3.0, "ENG-E-0010"),
        ] {
            code(LinearTruss::new(dim, e, a, rho), expected);
        }
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            code(LinearTruss::new(dim, bad, 2.0, 3.0), "ENG-E-0008");
            code(LinearTruss::new(dim, 200.0, bad, 3.0), "ENG-E-0008");
            code(LinearTruss::new(dim, 200.0, 2.0, bad), "ENG-E-0008");
        }
        let p = ElementParams::default();
        let element = bar(dim);
        let n = 2 * element.dofs_per_node().len();
        let coordinates = [[0.0; 3], [2.0, 0.0, 0.0]];
        let c = ctx(dim, &coordinates, &p);
        for u in [vec![0.0; n - 1], vec![0.0; n + 1]] {
            code(element.respond(&c, &u, &[]), "ENG-E-0010");
            code(element.geometric_stiffness(&c, &u, &[]), "ENG-E-0010");
            code(element.recover(&c, &u, &[], &[], &[]), "ENG-E-0010");
        }
        code(element.respond(&c, &vec![0.0; n], &[0.0]), "ENG-E-0010");
        code(
            element.geometric_stiffness(&c, &vec![0.0; n], &[0.0]),
            "ENG-E-0010",
        );
        code(
            element.recover(&c, &vec![0.0; n], &[0.0], &[], &[]),
            "ENG-E-0010",
        );
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut coords = coordinates;
                coords[1][axis] = bad;
                let badctx = ctx(dim, &coords, &p);
                code(element.mass(&badctx, MassKind::Lumped), "ENG-E-0008");
                code(element.respond(&badctx, &vec![0.0; n], &[]), "ENG-E-0008");
            }
            for axis in 0..n {
                let mut u = vec![0.0; n];
                u[axis] = bad;
                code(element.respond(&c, &u, &[]), "ENG-E-0008");
                code(element.geometric_stiffness(&c, &u, &[]), "ENG-E-0008");
                code(element.recover(&c, &u, &[], &[], &[]), "ENG-E-0008");
            }
            code(
                element.recover(&c, &vec![0.0; n], &[], &[], &[bad]),
                "ENG-E-0008",
            );
        }
        for station in [-0.01, 1.01] {
            code(
                element.recover(&c, &vec![0.0; n], &[], &[], &[station]),
                "ENG-E-0010",
            );
        }
        for length in [0.0, 1e-13, 1e-12] {
            let coords = [[0.0; 3], [length, 0.0, 0.0]];
            let badctx = ctx(dim, &coords, &p);
            code(element.respond(&badctx, &vec![0.0; n], &[]), "ENG-E-0005");
            code(element.mass(&badctx, MassKind::Lumped), "ENG-E-0005");
        }
        code(
            element.respond(&ctx(dim, &coordinates[..1], &p), &vec![0.0; n], &[]),
            "ENG-E-0010",
        );
        let other = if dim == Dim::D2 { Dim::D3 } else { Dim::D2 };
        code(
            element.mass(&ctx(other, &coordinates, &p), MassKind::Consistent),
            "ENG-E-0010",
        );
    }
    let p = ElementParams::default();
    let coords = [[0.0; 3], [1.0, 0.0, 0.1]];
    code(
        bar(Dim::D2).respond(&ctx(Dim::D2, &coords, &p), &[0.0; 4], &[]),
        "ENG-E-0010",
    );
}

#[test]
fn loads_reject_unsupported_components_even_small_and_nonfinite() {
    let p = ElementParams::default();
    let coordinates = [[0.0; 3], [2.0, 0.0, 0.0]];
    for dim in [Dim::D2, Dim::D3] {
        let element = bar(dim);
        let c = ctx(dim, &coordinates, &p);
        let n = 2 * element.dofs_per_node().len();
        for axis in 1..3 {
            for magnitude in [1.0, 1e-30] {
                let mut load = MemberLoad {
                    force_per_length: [0.0; 3],
                    moment_per_length: [0.0; 3],
                };
                load.force_per_length[axis] = magnitude;
                code(element.load_vector(&c, &load), "ENG-E-0010");
                code(
                    element.recover(&c, &vec![0.0; n], &[], &[load], &[]),
                    "ENG-E-0010",
                );
            }
        }
        for axis in 0..3 {
            let mut load = MemberLoad {
                force_per_length: [0.0; 3],
                moment_per_length: [0.0; 3],
            };
            load.moment_per_length[axis] = 1e-30;
            code(element.load_vector(&c, &load), "ENG-E-0010");
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                load.moment_per_length[axis] = bad;
                code(element.load_vector(&c, &load), "ENG-E-0008");
                load.moment_per_length = [0.0; 3];
                load.force_per_length[axis] = bad;
                code(element.load_vector(&c, &load), "ENG-E-0008");
                load.force_per_length = [0.0; 3];
            }
        }
    }
}

#[test]
fn overflowing_geometry_stiffness_mass_force_and_recovery_fail() {
    let p = ElementParams::default();
    let coords = [[-f64::MAX, 0.0, 0.0], [f64::MAX, 0.0, 0.0]];
    code(
        bar(Dim::D2).mass(&ctx(Dim::D2, &coords, &p), MassKind::Lumped),
        "ENG-E-0008",
    );
    let coords = [[0.0; 3], [1.0, 0.0, 0.0]];
    let c = ctx(Dim::D2, &coords, &p);
    code(
        bar(Dim::D2).respond(&c, &[0.0, 0.0, f64::MAX, 0.0], &[]),
        "ENG-E-0008",
    );
    code(
        bar(Dim::D2).respond(&c, &[-f64::MAX, 0.0, f64::MAX, 0.0], &[]),
        "ENG-E-0008",
    );
    code(
        LinearTruss::new(Dim::D2, 200.0, 2.0, f64::MAX)
            .unwrap()
            .mass(&c, MassKind::Consistent),
        "ENG-E-0008",
    );
    let short = [[0.0; 3], [1e-11, 0.0, 0.0]];
    let shortctx = ctx(Dim::D2, &short, &p);
    code(
        LinearTruss::new(Dim::D2, f64::MAX, 1.0, 0.0)
            .unwrap()
            .respond(&shortctx, &[0.0; 4], &[]),
        "ENG-E-0008",
    );
    let long = [[0.0; 3], [4.0, 0.0, 0.0]];
    let longctx = ctx(Dim::D2, &long, &p);
    let load = MemberLoad {
        force_per_length: [f64::MAX, 0.0, 0.0],
        moment_per_length: [0.0; 3],
    };
    code(bar(Dim::D2).load_vector(&longctx, &load), "ENG-E-0008");
    code(
        bar(Dim::D2).recover(&c, &[0.0; 4], &[], &[load.clone(), load], &[]),
        "ENG-E-0008",
    );
}

#[test]
fn factory_schema_and_negative_inputs() {
    for dim in [Dim::D2, Dim::D3] {
        let factory = TrussFactory::new(dim);
        let model = ModelView { dim, nodes: &[] };
        assert!(factory
            .params_schema()
            .contains("additionalProperties\":false"));
        assert_eq!(
            factory.build(&params(), &model).unwrap().kind(),
            factory.kind()
        );
        code(factory.build(&JsonValue::Null, &model), "ENG-E-0010");
        let JsonValue::Object(fields) = params() else {
            unreachable!()
        };
        for key in ["E", "A", "rho"] {
            let mut missing = fields.clone();
            missing.remove(key);
            code(
                factory.build(&JsonValue::Object(missing), &model),
                "ENG-E-0010",
            );
            for value in [
                JsonValue::String("1".into()),
                JsonValue::Bool(true),
                JsonValue::Null,
            ] {
                let mut wrong = fields.clone();
                wrong.insert(key.into(), value);
                code(
                    factory.build(&JsonValue::Object(wrong), &model),
                    "ENG-E-0010",
                );
            }
            let mut wrong = fields.clone();
            wrong.insert(key.into(), JsonValue::Number(f64::NAN));
            code(
                factory.build(&JsonValue::Object(wrong), &model),
                "ENG-E-0008",
            );
        }
        let mut extra = fields.clone();
        extra.insert("extra".into(), JsonValue::Null);
        code(
            factory.build(&JsonValue::Object(extra), &model),
            "ENG-E-0010",
        );
        let other = if dim == Dim::D2 { Dim::D3 } else { Dim::D2 };
        code(
            factory.build(
                &params(),
                &ModelView {
                    dim: other,
                    nodes: &[],
                },
            ),
            "ENG-E-0010",
        );
        code(
            factory.build(
                &params(),
                &ModelView {
                    dim,
                    nodes: &[(NodeId(1), [0.0; 3]), (NodeId(1), [0.0; 3])],
                },
            ),
            "ENG-E-0002",
        );
        code(
            factory.build(
                &params(),
                &ModelView {
                    dim,
                    nodes: &[(NodeId(1), [f64::NAN, 0.0, 0.0])],
                },
            ),
            "ENG-E-0008",
        );
    }
    code(
        TrussFactory::new(Dim::D2).build(
            &params(),
            &ModelView {
                dim: Dim::D2,
                nodes: &[(NodeId(1), [0.0, 0.0, 1.0])],
            },
        ),
        "ENG-E-0010",
    );
}

#[test]
fn geometric_and_load_covariance_and_endpoint_reversal() {
    // source/citation: SOURCE y CITATION; derivación cerrada P=I-ddᵀ.
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let element = bar(dim);
        let axes = element.dofs_per_node().len();
        let n = 2 * axes;
        let coords = if dim == Dim::D2 {
            [[0.1, -0.3, 0.0], [2.1, 1.7, 0.0]]
        } else {
            [[0.1, -0.3, 0.2], [2.1, 1.7, 1.2]]
        };
        let c = ctx(dim, &coords, &p);
        let axis = if dim == Dim::D2 { 2 } else { 1 };
        let (s, co) = 0.61_f64.sin_cos();
        let mut r = [[0.0; 3]; 3];
        r[axis][axis] = 1.0;
        let a = (axis + 1) % 3;
        let b = (axis + 2) % 3;
        r[a][a] = co;
        r[a][b] = -s;
        r[b][a] = s;
        r[b][b] = co;
        let rotate = |p: [f64; 3]| -> [f64; 3] {
            std::array::from_fn(|i| r[i].iter().zip(p).map(|(a, b)| a * b).sum())
        };
        let rotated_coords = coords.map(rotate);
        let rc = ctx(dim, &rotated_coords, &p);
        let mut values = vec![0.0; n * n];
        for node in 0..2 {
            for i in 0..axes {
                for j in 0..axes {
                    values[(node * axes + i) * n + node * axes + j] = r[i][j];
                }
            }
        }
        let q = DMat::new(n, n, values).unwrap();
        let u: Vec<_> = (0..n).map(|i| (i as f64 - 2.0) * 0.01).collect();
        let ru = multiply(&q, &u);
        let kg = element.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
        let rkg = element.geometric_stiffness(&rc, &ru, &[]).unwrap().unwrap();
        let mut backend = ReferenceBackend;
        let qt = backend.transpose(&q).unwrap();
        let left = backend.matmul(&q, &kg).unwrap();
        let expected = backend.matmul(&left, &qt).unwrap();
        for (v, e) in rkg.values().iter().zip(expected.values()) {
            close(*v, *e);
        }
        // q paralelo a la diferencia geométrica; no hace falta calcular d.
        let force = std::array::from_fn(|a| 3.0 * (coords[1][a] - coords[0][a]));
        let load = MemberLoad {
            force_per_length: force,
            moment_per_length: [0.0; 3],
        };
        let rload = MemberLoad {
            force_per_length: rotate(force),
            moment_per_length: [0.0; 3],
        };
        let v = element.load_vector(&c, &load).unwrap();
        let rv = element.load_vector(&rc, &rload).unwrap();
        for (v, e) in rv.iter().zip(multiply(&q, &v)) {
            close(*v, e);
        }
        let output = element
            .recover(&c, &u, &[], std::slice::from_ref(&load), &[0.0, 0.3, 1.0])
            .unwrap();
        let rout = element
            .recover(&rc, &ru, &[], &[rload], &[0.0, 0.3, 1.0])
            .unwrap();
        for (s, t) in output.stations.iter().zip(rout.stations) {
            close(s.forces_local[0], t.forces_local[0]);
        }
        // Invertir i/j invierte el eje y q axial, pero conserva N de la misma
        // sección física y permuta los extremos globales de fuerza y rigidez.
        let reverse_coords = [coords[1], coords[0]];
        let reversectx = ctx(dim, &reverse_coords, &p);
        let reverseu: Vec<_> = u[axes..].iter().chain(&u[..axes]).copied().collect();
        let original = element.respond(&c, &u, &[]).unwrap();
        let reversed = element.respond(&reversectx, &reverseu, &[]).unwrap();
        for i in 0..n {
            close(
                reversed.internal_force[i],
                original.internal_force[(i + axes) % n],
            );
            for j in 0..n {
                close(
                    reversed.tangent.get(i, j).unwrap(),
                    original
                        .tangent
                        .get((i + axes) % n, (j + axes) % n)
                        .unwrap(),
                );
            }
        }
        let out = element
            .recover(&reversectx, &reverseu, &[], &[load], &[1.0, 0.7, 0.0])
            .unwrap();
        for (s, t) in output.stations.iter().zip(out.stations) {
            close(s.forces_local[0], t.forces_local[0]);
        }
    }
}
