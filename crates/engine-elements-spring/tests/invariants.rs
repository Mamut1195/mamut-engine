//! Conformidad, energía, covarianza explícita, extremos numéricos y diagnósticos.
use engine_contracts::{
    conformance::{check_element, check_element_factory},
    Element, ElementCtx, ElementFactory, ElementParams, JsonValue, MassKind, MemberLoad, ModelView,
};
use engine_core::{Dim, Dof, EResult, NodeId};
use engine_elements_spring::{LinearSpring, SpringAxes, SpringConnection, SpringFactory};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use std::collections::BTreeMap;

const TOL: f64 = 1e-10; // Absoluta en unidades SI de fuerzas/matrices de estos casos.
const I: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= TOL, "{a} != {b}");
}
fn code<T>(r: EResult<T>, expected: &str) {
    assert_eq!(r.err().expect("debe fallar").code, expected);
}
fn spring(dim: Dim, ground: bool, k: f64, dofs: &[Dof], axes: SpringAxes) -> LinearSpring {
    LinearSpring::new(
        dim,
        if ground {
            SpringConnection::Ground
        } else {
            SpringConnection::BetweenNodes
        },
        k,
        dofs,
        axes,
    )
    .unwrap()
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
fn params(dofs: &[Dof]) -> BTreeMap<String, JsonValue> {
    BTreeMap::from([
        ("k".into(), JsonValue::Number(120.0)),
        (
            "dofs".into(),
            JsonValue::Array(
                dofs.iter()
                    .map(|d| JsonValue::String(d.label().into()))
                    .collect(),
            ),
        ),
    ])
}
fn array(v: [f64; 3]) -> JsonValue {
    JsonValue::Array(v.into_iter().map(JsonValue::Number).collect())
}
fn zero_load() -> MemberLoad {
    MemberLoad {
        force_per_length: [0.0; 3],
        moment_per_length: [0.0; 3],
    }
}

#[test]
fn element_and_factory_conformance_compatible_rigid_modes() {
    let p = ElementParams::default();
    for dim in [Dim::D2, Dim::D3] {
        let coords = [[0.0; 3], [1.0, 0.0, 0.0]];
        let c = ctx(dim, &coords, &p);
        let nodes = [(NodeId(1), coords[0]), (NodeId(2), coords[1])];
        let m = ModelView { dim, nodes: &nodes };
        // Subconjunto axial: el kit no infiere una rotación de ejes propios.
        for dofs in [&[Dof::Ux][..], &[Dof::Rz][..]] {
            let s = spring(dim, false, 120.0, dofs, SpringAxes::Global);
            let samples = [vec![0.0; 2], vec![0.002, -0.003], vec![-0.004, 0.001]];
            let issues = check_element(&s, &c, &samples, 1e-7);
            assert!(issues.is_empty(), "{issues:?}");
            let issues = check_element_factory(
                &SpringFactory::new(dim),
                &JsonValue::Object(params(dofs)),
                &m,
                &c,
                &samples,
                1e-7,
            );
            assert!(issues.is_empty(), "{issues:?}");
            assert_eq!(s.state_len(), 0);
            assert!(s.initial_state().is_empty());
            assert_eq!(s.dim(), dim);
            assert_eq!(s.connection(), SpringConnection::BetweenNodes);
            let r = s.respond(&c, &samples[1], &[]).unwrap();
            for _ in 0..10 {
                assert_eq!(r, s.respond(&c, &samples[1], &[]).unwrap());
            }
        }
    }
}

#[test]
fn coupled_dofs_rank_one_energy_force_recovery_and_rigid_translation() {
    let p = ElementParams::default();
    let coords = [[0.0; 3]; 2]; // Se admiten nodos coincidentes.
    for (dim, dofs) in [
        (Dim::D2, vec![Dof::Uy, Dof::Ux]),
        (Dim::D3, vec![Dof::Ry, Dof::Rx]),
    ] {
        let s = spring(dim, false, 120.0, &dofs, SpringAxes::Global);
        let c = ctx(dim, &coords, &p);
        let u = [0.1, 0.2, 0.4, 0.6]; // δ=(.4−.1)+(.6−.2)=.7.
        let r = s.respond(&c, &u, &[]).unwrap();
        SymMatrix::new(r.tangent.clone()).unwrap();
        for (a, b) in r.internal_force.iter().zip([-84.0, -84.0, 84.0, 84.0]) {
            close(*a, b);
        }
        for (a, b) in multiply(&r.tangent, &u).iter().zip(&r.internal_force) {
            close(*a, *b);
        }
        let energy = 0.5
            * u.iter()
                .zip(&r.internal_force)
                .map(|(u, f)| u * f)
                .sum::<f64>();
        close(energy, 0.5 * 120.0 * 0.7_f64.powi(2));
        for rigid in [
            [1.0, 0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, -1.0],
        ] {
            for f in multiply(&r.tangent, &rigid) {
                close(f, 0.0);
            }
            for f in s.respond(&c, &rigid, &[]).unwrap().internal_force {
                close(f, 0.0);
            }
        }
        let out = s.recover(&c, &u, &[], &[], &[1.0, 0.0, 0.5, 0.5]).unwrap();
        assert_eq!(out.end_forces_local, r.internal_force);
        for station in out.stations {
            for dof in &dofs {
                close(station.forces_local[dof.index()], 84.0);
            }
        }
    }
}

fn finite_difference(s: &dyn Element, c: &ElementCtx<'_>, u: &[f64]) {
    let base = s.respond(c, u, &[]).unwrap();
    SymMatrix::new(base.tangent.clone()).unwrap();
    for j in 0..u.len() {
        let mut plus = u.to_vec();
        let mut minus = u.to_vec();
        plus[j] += 1e-7;
        minus[j] -= 1e-7;
        let fp = s.respond(c, &plus, &[]).unwrap().internal_force;
        let fm = s.respond(c, &minus, &[]).unwrap().internal_force;
        for i in 0..u.len() {
            assert!(((fp[i] - fm[i]) / 2e-7 - base.tangent.get(i, j).unwrap()).abs() < 1e-6);
        }
    }
}

#[test]
fn ground_and_oblique_variants_consistent_tangent_and_null_mass_kg() {
    let p = ElementParams::default();
    let coords = [[0.0; 3]; 2];
    for dim in [Dim::D2, Dim::D3] {
        let direction = if dim == Dim::D2 {
            [3.0, 4.0, 0.0]
        } else {
            [2.0, -3.0, 6.0]
        };
        for ground in [false, true] {
            let s = spring(
                dim,
                ground,
                120.0,
                &[Dof::Ux],
                SpringAxes::Direction(direction),
            );
            let c = ctx(dim, &coords[..s.n_nodes()], &p);
            let u: Vec<_> = (0..s.n_nodes() * s.dofs_per_node().len())
                .map(|i| i as f64 * 0.003)
                .collect();
            finite_difference(&s, &c, &u);
            for kind in [MassKind::Lumped, MassKind::Consistent] {
                assert!(s.mass(&c, kind).unwrap().is_none());
            }
            assert!(s.geometric_stiffness(&c, &u, &[]).unwrap().is_none());
            assert_eq!(s.load_vector(&c, &zero_load()).unwrap(), vec![0.0; u.len()]);
            let out = s.recover(&c, &u, &[], &[zero_load()], &[0.0, 1.0]).unwrap();
            assert_eq!(out.stations[0].forces_local, out.stations[1].forces_local);
            for f in &out.stations[0].forces_local[1..] {
                close(*f, 0.0);
            }
        }
    }
    // En apoyo k=0 sí aplica el kit completo, incluidos sus modos rígidos.
    let s = spring(Dim::D2, true, 0.0, &[Dof::Ux], SpringAxes::Global);
    let issues = check_element(
        &s,
        &ctx(Dim::D2, &coords[..1], &p),
        &[vec![0.0], vec![0.03]],
        1e-7,
    );
    assert!(issues.is_empty(), "{issues:?}");
}

fn rotate(q: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| (0..3).map(|j| q[i][j] * v[j]).sum())
}
#[test]
fn oblique_axis_reversal_scaling_and_spatial_rotation_covariance() {
    let p = ElementParams::default();
    let coords = [[-1.0, 2.0, 3.0], [4.0, -2.0, 5.0]];
    let c = ctx(Dim::D3, &coords, &p);
    let d = [2.0, -3.0, 6.0];
    let s = spring(Dim::D3, false, 120.0, &[Dof::Ux], SpringAxes::Direction(d));
    let u = [0.001, -0.002, 0.004, -0.003, 0.005, 0.006];
    let base = s.respond(&c, &u, &[]).unwrap();
    for factor in [-2.0, 1e-200, 1e200] {
        let other = spring(
            Dim::D3,
            false,
            120.0,
            &[Dof::Ux],
            SpringAxes::Direction(d.map(|v| v * factor)),
        )
        .respond(&c, &u, &[])
        .unwrap();
        for (a, b) in base.tangent.values().iter().zip(other.tangent.values()) {
            close(*a, *b);
        }
        for (a, b) in base.internal_force.iter().zip(other.internal_force) {
            close(*a, b);
        }
    }
    let (sn, cs) = 0.73_f64.sin_cos();
    let q = [[cs, 0.0, sn], [0.0, 1.0, 0.0], [-sn, 0.0, cs]];
    let rotated_coords = coords.map(|v| rotate(&q, v));
    let rotated_u: Vec<_> = u
        .chunks_exact(3)
        .flat_map(|v| rotate(&q, [v[0], v[1], v[2]]))
        .collect();
    let rotated_force: Vec<_> = base
        .internal_force
        .chunks_exact(3)
        .flat_map(|v| rotate(&q, [v[0], v[1], v[2]]))
        .collect();
    let sr = spring(
        Dim::D3,
        false,
        120.0,
        &[Dof::Ux],
        SpringAxes::Direction(rotate(&q, d)),
    );
    let r = sr
        .respond(&ctx(Dim::D3, &rotated_coords, &p), &rotated_u, &[])
        .unwrap();
    for (a, b) in r.internal_force.iter().zip(rotated_force) {
        close(*a, b);
    }
    // Congruencia independiente Q K Qᵀ, bloque por nodo.
    for i in 0..6 {
        for j in 0..6 {
            let mut expected = 0.0;
            for a in 0..3 {
                for b in 0..3 {
                    expected += q[i % 3][a]
                        * base.tangent.get(i / 3 * 3 + a, j / 3 * 3 + b).unwrap()
                        * q[j % 3][b];
                }
            }
            close(r.tangent.get(i, j).unwrap(), expected);
        }
    }
    for rigid in [
        [1.0, 2.0, 3.0, 1.0, 2.0, 3.0],
        [-2.0, 0.0, 4.0, -2.0, 0.0, 4.0],
    ] {
        for f in multiply(&r.tangent, &rigid) {
            close(f, 0.0);
        }
    }
}

#[test]
fn rotation_matrix_matches_direction_and_coupled_local_components() {
    let p = ElementParams::default();
    let coords = [[0.0; 3]; 2];
    let c = ctx(Dim::D2, &coords, &p);
    let q = [[0.6, 0.8, 0.0], [-0.8, 0.6, 0.0], [0.0, 0.0, 1.0]];
    let u = [0.0, 0.0, 0.003, 0.004];
    let a = spring(Dim::D2, false, 120.0, &[Dof::Ux], SpringAxes::Rotation(q));
    let b = spring(
        Dim::D2,
        false,
        120.0,
        &[Dof::Ux],
        SpringAxes::Direction([3.0, 4.0, 0.0]),
    );
    assert_eq!(
        a.respond(&c, &u, &[]).unwrap(),
        b.respond(&c, &u, &[]).unwrap()
    );
    let coupled = spring(
        Dim::D2,
        false,
        120.0,
        &[Dof::Ux, Dof::Uy],
        SpringAxes::Rotation(q),
    );
    finite_difference(&coupled, &c, &u);
    // δ=(.6−.8)*.003+(.8+.6)*.004=.005.
    let out = coupled.recover(&c, &u, &[], &[], &[0.5]).unwrap();
    for f in out.end_forces_local {
        close(f.abs(), 0.6);
    }
    close(out.stations[0].forces_local[0], 0.6);
    close(out.stations[0].forces_local[1], 0.6);
    // Giro local Rx oblicuo en D3 usa la misma congruencia que traslaciones.
    let cs = 0.6;
    let sn = 0.8;
    let q3 = [[cs, 0.0, sn], [0.0, 1.0, 0.0], [-sn, 0.0, cs]];
    let rotational = spring(Dim::D3, false, 120.0, &[Dof::Rx], SpringAxes::Rotation(q3));
    let r = rotational
        .respond(
            &ctx(Dim::D3, &coords, &p),
            &[0.0, 0.0, 0.0, 0.003, 0.0, 0.004],
            &[],
        )
        .unwrap();
    close(r.internal_force[3], 0.36);
    close(r.internal_force[5], 0.48);
}

#[test]
fn huge_tiny_zero_stiffness_and_extreme_direction_without_panics() {
    let p = ElementParams::default();
    let c = ctx(Dim::D2, &[[0.0; 3]; 2], &p);
    for k in [0.0, f64::MAX, 1e-300, f64::from_bits(1)] {
        let s = spring(Dim::D2, false, k, &[Dof::Ux], SpringAxes::Global);
        let r = s.respond(&c, &[0.0, 1.0], &[]).unwrap();
        assert_eq!(r.tangent.values(), &[k, -k, -k, k]);
        assert_eq!(r.internal_force, &[-k, k]);
    }
    let s = spring(Dim::D2, false, f64::MAX, &[Dof::Ux], SpringAxes::Global);
    code(s.respond(&c, &[0.0, 2.0], &[]), "ENG-E-0008");
    code(s.respond(&c, &[-f64::MAX, f64::MAX], &[]), "ENG-E-0008");
    let zero = spring(Dim::D2, false, 0.0, &[Dof::Ux], SpringAxes::Global);
    assert_eq!(
        zero.respond(&c, &[-f64::MAX, f64::MAX], &[])
            .unwrap()
            .internal_force,
        vec![0.0; 2]
    );
    for d in [
        [f64::MAX, f64::MAX, 0.0],
        [f64::from_bits(1), f64::from_bits(1), 0.0],
    ] {
        let s = spring(Dim::D2, false, 120.0, &[Dof::Ux], SpringAxes::Direction(d));
        let r = s.respond(&c, &[0.0, 0.0, 1.0, 1.0], &[]).unwrap();
        close(r.internal_force[2], 120.0);
        close(r.tangent.get(0, 0).unwrap(), 60.0);
    }
    let angle = std::f64::consts::FRAC_PI_4;
    let (sn, cs) = angle.sin_cos();
    let q = [[cs, sn, 0.0], [-sn, cs, 0.0], [0.0, 0.0, 1.0]];
    let coupled = spring(
        Dim::D2,
        false,
        f64::MAX,
        &[Dof::Ux, Dof::Uy],
        SpringAxes::Rotation(q),
    );
    code(coupled.respond(&c, &[0.0; 4], &[]), "ENG-E-0008"); // 2k no representable.
}

#[test]
fn invalid_properties_dofs_directions_and_rotations_have_catalog_diagnostics() {
    let make = |k, dofs: &[Dof], axes| {
        LinearSpring::new(Dim::D2, SpringConnection::BetweenNodes, k, dofs, axes)
    };
    code(make(-1.0, &[Dof::Ux], SpringAxes::Global), "ENG-E-0010");
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        code(make(value, &[Dof::Ux], SpringAxes::Global), "ENG-E-0008");
        code(
            make(1.0, &[Dof::Ux], SpringAxes::Direction([value, 0.0, 0.0])),
            "ENG-E-0008",
        );
        let mut q = I;
        q[0][0] = value;
        code(make(1.0, &[Dof::Ux], SpringAxes::Rotation(q)), "ENG-E-0008");
    }
    for dofs in [
        &[][..],
        &[Dof::Ux, Dof::Ux][..],
        &[Dof::Uz][..],
        &[Dof::Rx][..],
        &[Dof::Ux, Dof::Rz][..],
    ] {
        code(make(1.0, dofs, SpringAxes::Global), "ENG-E-0010");
    }
    for d in [[0.0; 3], [1.0, 0.0, 1e-30]] {
        code(
            make(1.0, &[Dof::Ux], SpringAxes::Direction(d)),
            "ENG-E-0010",
        );
    }
    code(
        make(1.0, &[Dof::Uy], SpringAxes::Direction([1.0, 0.0, 0.0])),
        "ENG-E-0010",
    );
    for q in [
        [[0.0; 3]; 3],
        [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]],
        [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
    ] {
        code(make(1.0, &[Dof::Ux], SpringAxes::Rotation(q)), "ENG-E-0010");
    }
    let mut huge = I;
    huge[0][0] = f64::MAX;
    code(
        make(1.0, &[Dof::Ux], SpringAxes::Rotation(huge)),
        "ENG-E-0008",
    );
}

#[test]
fn invalid_context_state_stations_and_loads_are_never_silently_ignored() {
    let p = ElementParams::default();
    let s = spring(Dim::D2, false, 120.0, &[Dof::Ux], SpringAxes::Global);
    let coords = [[0.0; 3]; 2];
    let c = ctx(Dim::D2, &coords, &p);
    for bad in [
        ctx(Dim::D3, &coords, &p),
        ctx(Dim::D2, &coords[..1], &p),
        ctx(Dim::D2, &[[0.0, 0.0, 1.0]; 2], &p),
    ] {
        code(s.respond(&bad, &[0.0; 2], &[]), "ENG-E-0010");
        code(s.mass(&bad, MassKind::Lumped), "ENG-E-0010");
        code(s.geometric_stiffness(&bad, &[0.0; 2], &[]), "ENG-E-0010");
        code(s.load_vector(&bad, &zero_load()), "ENG-E-0010");
        code(s.recover(&bad, &[0.0; 2], &[], &[], &[]), "ENG-E-0010");
    }
    let bad = ctx(Dim::D2, &[[f64::NAN, 0.0, 0.0]; 2], &p);
    code(s.mass(&bad, MassKind::Consistent), "ENG-E-0008");
    code(s.respond(&bad, &[0.0; 2], &[]), "ENG-E-0008");
    code(s.respond(&c, &[0.0], &[]), "ENG-E-0010");
    code(s.respond(&c, &[0.0; 2], &[0.0]), "ENG-E-0010");
    code(s.geometric_stiffness(&c, &[0.0], &[]), "ENG-E-0010");
    code(s.geometric_stiffness(&c, &[0.0; 2], &[0.0]), "ENG-E-0010");
    for value in [f64::NAN, f64::INFINITY] {
        code(s.respond(&c, &[0.0, value], &[]), "ENG-E-0008");
        code(s.geometric_stiffness(&c, &[0.0, value], &[]), "ENG-E-0008");
        code(s.recover(&c, &[0.0; 2], &[], &[], &[value]), "ENG-E-0008");
        let mut load = zero_load();
        load.force_per_length[0] = value;
        code(s.load_vector(&c, &load), "ENG-E-0008");
        code(s.recover(&c, &[0.0; 2], &[], &[load], &[]), "ENG-E-0008");
    }
    for station in [-0.001, 1.001] {
        code(s.recover(&c, &[0.0; 2], &[], &[], &[station]), "ENG-E-0010");
    }
    for axis in 0..3 {
        let mut load = zero_load();
        load.force_per_length[axis] = 1e-300;
        code(s.load_vector(&c, &load), "ENG-E-0010");
        code(s.recover(&c, &[0.0; 2], &[], &[load], &[]), "ENG-E-0010");
        let mut load = zero_load();
        load.moment_per_length[axis] = 1e-300;
        code(s.load_vector(&c, &load), "ENG-E-0010");
    }
}

#[test]
fn factory_orientations_ground_and_owned_properties() {
    let nodes = [(NodeId(1), [0.0; 3])];
    let m = ModelView {
        dim: Dim::D2,
        nodes: &nodes,
    };
    let f = SpringFactory::new(Dim::D2);
    let p = ElementParams::default();
    let c = ctx(Dim::D2, &[[0.0; 3]], &p);
    for orientation in [
        None,
        Some(("direction", array([3.0, 4.0, 0.0]))),
        Some((
            "rotation",
            JsonValue::Array(I.into_iter().map(array).collect()),
        )),
    ] {
        let mut fields = params(&[Dof::Ux]);
        fields.insert("ground".into(), JsonValue::Bool(true));
        if let Some((key, value)) = orientation {
            fields.insert(key.into(), value);
        }
        let s = f.build(&JsonValue::Object(fields), &m).unwrap();
        let u = vec![0.03; s.dofs_per_node().len()];
        finite_difference(s.as_ref(), &c, &u);
        let base = s.respond(&c, &u, &[]).unwrap();
        let mut other = ElementParams::default();
        other.scalars.insert("k".into(), f64::NAN);
        assert_eq!(
            base,
            s.respond(&ctx(Dim::D2, c.coords, &other), &u, &[]).unwrap()
        );
    }
}

#[test]
fn factory_rejects_missing_extra_wrong_type_and_invalid_orientation_fields() {
    let f = SpringFactory::new(Dim::D2);
    let nodes = [(NodeId(1), [0.0; 3])];
    let m = ModelView {
        dim: Dim::D2,
        nodes: &nodes,
    };
    code(f.build(&JsonValue::Null, &m), "ENG-E-0010");
    for key in ["k", "dofs"] {
        let mut fields = params(&[Dof::Ux]);
        fields.remove(key);
        code(f.build(&JsonValue::Object(fields), &m), "ENG-E-0010");
    }
    for (key, value, expected) in [
        ("extra", JsonValue::Number(1.0), "ENG-E-0010"),
        ("k", JsonValue::String("120".into()), "ENG-E-0010"),
        ("k", JsonValue::Number(-1.0), "ENG-E-0010"),
        ("k", JsonValue::Number(f64::NAN), "ENG-E-0008"),
        ("ground", JsonValue::Number(1.0), "ENG-E-0010"),
        ("dofs", JsonValue::Null, "ENG-E-0010"),
        (
            "dofs",
            JsonValue::Array(vec![JsonValue::Number(0.0)]),
            "ENG-E-0010",
        ),
        (
            "dofs",
            JsonValue::Array(vec![JsonValue::String("Uz".into())]),
            "ENG-E-0010",
        ),
        (
            "dofs",
            JsonValue::Array(vec![JsonValue::String("Ux".into()); 2]),
            "ENG-E-0010",
        ),
        (
            "dofs",
            JsonValue::Array(vec![JsonValue::String("Ux".into()); 7]),
            "ENG-E-0010",
        ),
        ("direction", JsonValue::Bool(true), "ENG-E-0010"),
        (
            "direction",
            JsonValue::Array(vec![JsonValue::Number(1.0); 2]),
            "ENG-E-0010",
        ),
        (
            "direction",
            JsonValue::Array(vec![JsonValue::Bool(false); 3]),
            "ENG-E-0010",
        ),
        ("direction", array([0.0; 3]), "ENG-E-0010"),
        ("direction", array([f64::INFINITY, 0.0, 0.0]), "ENG-E-0008"),
        (
            "rotation",
            JsonValue::Array(vec![array([1.0, 0.0, 0.0]); 2]),
            "ENG-E-0010",
        ),
        (
            "rotation",
            JsonValue::Array(vec![array([1.0, 0.0, 0.0]); 3]),
            "ENG-E-0010",
        ),
        (
            "rotation",
            JsonValue::Array(vec![JsonValue::Null; 3]),
            "ENG-E-0010",
        ),
    ] {
        let mut fields = params(&[Dof::Ux]);
        fields.insert(key.into(), value);
        code(f.build(&JsonValue::Object(fields), &m), expected);
    }
    let mut fields = params(&[Dof::Ux]);
    fields.insert("direction".into(), array([1.0, 0.0, 0.0]));
    fields.insert(
        "rotation".into(),
        JsonValue::Array(I.into_iter().map(array).collect()),
    );
    code(f.build(&JsonValue::Object(fields), &m), "ENG-E-0010");
    for (dim, nodes, expected) in [
        (Dim::D3, vec![], "ENG-E-0010"),
        (Dim::D2, vec![(NodeId(1), [0.0; 3]); 2], "ENG-E-0002"),
        (Dim::D2, vec![(NodeId(1), [0.0, 0.0, 1.0])], "ENG-E-0010"),
        (
            Dim::D2,
            vec![(NodeId(1), [f64::NAN, 0.0, 0.0])],
            "ENG-E-0008",
        ),
    ] {
        code(
            f.build(
                &JsonValue::Object(params(&[Dof::Ux])),
                &ModelView { dim, nodes: &nodes },
            ),
            expected,
        );
    }
}

#[test]
fn factory_empty_dofs_reports_exactly_one_dof_required() {
    let nodes = [(NodeId(1), [0.0; 3])];
    for dim in [Dim::D2, Dim::D3] {
        let model = ModelView { dim, nodes: &nodes };
        let error = SpringFactory::new(dim)
            .build(&JsonValue::Object(params(&[])), &model)
            .err()
            .expect("la fábrica debe rechazar una lista vacía de GDL");
        assert_eq!(error.code, "ENG-E-0010");
        assert_eq!(
            error.message,
            "Resorte: la fábrica requiere exactamente un GDL de la dimensión seleccionada."
        );
    }
}

#[test]
fn factory_rejects_multi_dof_springs_that_would_couple_dofs() {
    // J4 r7: con varios GDL el resorte es de rango uno (K=k·bbᵀ) y acopla.
    let model_nodes = [(NodeId(1), [0.0; 3]), (NodeId(2), [1.0, 0.0, 0.0])];
    for dim in [Dim::D2, Dim::D3] {
        let model = ModelView {
            dim,
            nodes: &model_nodes,
        };
        let f = SpringFactory::new(dim);
        let one = JsonValue::Object(params(&[Dof::Ux]));
        assert!(f.build(&one, &model).is_ok());
        // Se conserva el diagnóstico para 2..3 GDL en D2 y 2..6 en D3,
        // incluso si la lista mezcla traslaciones y giros.
        for count in 2..=dim.dofs().len() {
            let many = JsonValue::Object(params(&dim.dofs()[..count]));
            code(f.build(&many, &model), "ENG-E-0013");
        }
    }
}
