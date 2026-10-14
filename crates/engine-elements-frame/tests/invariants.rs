//! Conformidad element/1, derivaciones energéticas y entradas negativas.
use engine_contracts::{
    conformance::{check_element, check_element_factory},
    Element, ElementCtx, ElementFactory, ElementParams, JsonValue, MassKind, MemberLoad, ModelView,
};
use engine_core::math;
use engine_core::{Dim, EResult, NodeId};
use engine_elements_frame::{EulerFrame2D, FrameFactory, MomentReleases};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use std::collections::BTreeMap;

const SOURCE: &str = "analítico: energía e integración polinómica propias";
const CITATION: &str =
    "provenance/ENG-ELM-002.md, Derivaciones propias; bibliografía exacta PENDIENTE";
const TOL: f64 = 1e-10; // absoluto, unidades SI de cada entrada de matrices/fuerzas.
const RELEASES: [MomentReleases; 4] = [
    MomentReleases { i: false, j: false },
    MomentReleases { i: true, j: false },
    MomentReleases { i: false, j: true },
    MomentReleases { i: true, j: true },
];
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= TOL, "{a} != {b}, tol={TOL}");
}
fn code<T>(r: EResult<T>, expected: &str) {
    assert_eq!(r.err().expect("debe fallar").code, expected);
}
fn beam(releases: MomentReleases) -> EulerFrame2D {
    EulerFrame2D::new(200.0, 2.0, 3.0, 3.0, releases).unwrap()
}
fn ctx<'a>(coords: &'a [[f64; 3]], params: &'a ElementParams) -> ElementCtx<'a> {
    ElementCtx {
        dim: Dim::D2,
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
fn params(r: MomentReleases) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("E".into(), JsonValue::Number(200.0)),
        ("A".into(), JsonValue::Number(2.0)),
        ("I".into(), JsonValue::Number(3.0)),
        ("rho".into(), JsonValue::Number(3.0)),
        ("release_i".into(), JsonValue::Bool(r.i)),
        ("release_j".into(), JsonValue::Bool(r.j)),
    ]))
}
fn uniform(q: [f64; 3], m: f64) -> MemberLoad {
    MemberLoad {
        force_per_length: q,
        moment_per_length: [0.0, 0.0, m],
    }
}

#[test]
fn element_and_factory_conformance_all_releases_and_repeatability() {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
    let p = ElementParams::default();
    let coords = [[0.2, -0.4, 0.0], [2.0, 1.3, 0.0]];
    let c = ctx(&coords, &p);
    let nodes = [(NodeId(1), coords[0]), (NodeId(2), coords[1])];
    let model = ModelView {
        dim: Dim::D2,
        nodes: &nodes,
    };
    let samples = vec![
        vec![0.0; 6],
        vec![0.002, -0.003, 0.005, -0.001, 0.004, -0.007],
        vec![-0.002, 0.003, -0.005, 0.001, -0.004, 0.007],
    ];
    for r in RELEASES {
        let b = beam(r);
        let issues = check_element(&b, &c, &samples, 1e-7);
        assert!(issues.is_empty(), "{r:?}: {issues:?}");
        let issues = check_element_factory(&FrameFactory, &params(r), &model, &c, &samples, 1e-7);
        assert!(issues.is_empty(), "{r:?}: {issues:?}");
        assert_eq!(b.state_len(), 0);
        assert!(b.initial_state().is_empty());
        assert_eq!(b.material().young_modulus(), 200.0);
        assert_eq!(b.area(), 2.0);
        assert_eq!(b.inertia(), 3.0);
        assert_eq!(b.density(), 3.0);
        assert_eq!(b.releases(), r);
        let response = b.respond(&c, &samples[1], &[]).unwrap();
        for _ in 0..10 {
            assert_eq!(response, b.respond(&c, &samples[1], &[]).unwrap());
        }
    }
}
#[test]
fn rigid_modes_symmetry_and_mass_total_per_global_direction() {
    let p = ElementParams::default();
    let coords = [[1.0, 2.0, 0.0], [4.0, 6.0, 0.0]];
    let c = ctx(&coords, &p);
    for r in RELEASES {
        let b = beam(r);
        let k = b.respond(&c, &[0.0; 6], &[]).unwrap().tangent;
        SymMatrix::new(k.clone()).unwrap();
        for u in [
            [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            [-2.0, 1.0, 1.0, -6.0, 4.0, 1.0],
        ] {
            for f in multiply(&k, &u) {
                close(f, 0.0);
            }
        }
        for kind in [MassKind::Lumped, MassKind::Consistent] {
            let m = b.mass(&c, kind).unwrap().unwrap();
            SymMatrix::new(m.clone()).unwrap();
            for v in [
                [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            ] {
                close(
                    v.iter().zip(multiply(&m, &v)).map(|(a, b)| a * b).sum(),
                    30.0,
                ); // rho A L=3*2*5.
            }
        }
        let zero = EulerFrame2D::new(200.0, 2.0, 3.0, 0.0, r).unwrap();
        assert!(zero.mass(&c, MassKind::Consistent).unwrap().is_none());
    }
}

// Exact integration on s∈[0,1], independent polynomial derivation of Hermite
// kinetic/strain/prestress energies. Coefficients low to high power.
fn integral_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .enumerate()
        .flat_map(|(i, &x)| {
            b.iter()
                .enumerate()
                .map(move |(j, &y)| x * y / (i + j + 1) as f64)
        })
        .sum()
}
fn derivative(a: &[f64]) -> Vec<f64> {
    a.iter()
        .enumerate()
        .skip(1)
        .map(|(i, x)| i as f64 * x)
        .collect()
}
#[test]
fn matrices_from_independent_polynomial_energy_integrals() {
    let p = ElementParams::default();
    let l = 2.0;
    let coords = [[0.0; 3], [l, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let b = beam(RELEASES[0]);
    let forms = [
        vec![1.0, 0.0, -3.0, 2.0],
        vec![0.0, l, -2.0 * l, l],
        vec![0.0, 0.0, 3.0, -2.0],
        vec![0.0, 0.0, -l, l],
    ];
    let first: Vec<_> = forms.iter().map(|v| derivative(v)).collect();
    let second: Vec<_> = first.iter().map(|v| derivative(v)).collect();
    let k = b.respond(&c, &[0.0; 6], &[]).unwrap().tangent;
    let m = b.mass(&c, MassKind::Consistent).unwrap().unwrap();
    // δu=0.06 => N=12 N, tracción.
    let g = b
        .geometric_stiffness(&c, &[0.0, 0.0, 0.0, 0.06, 0.0, 0.0], &[])
        .unwrap()
        .unwrap();
    SymMatrix::new(g.clone()).unwrap();
    let ids = [1, 2, 4, 5];
    for (i, &r) in ids.iter().enumerate() {
        for (j, &s) in ids.iter().enumerate() {
            close(
                k.get(r, s).unwrap(),
                600.0 / math::powi(l, 3) * integral_product(&second[i], &second[j]),
            );
            close(
                m.get(r, s).unwrap(),
                6.0 * l * integral_product(&forms[i], &forms[j]),
            );
            close(
                g.get(r, s).unwrap(),
                12.0 / l * integral_product(&first[i], &first[j]),
            );
        }
    }
    close(k.get(0, 0).unwrap(), 200.0);
    close(m.get(0, 0).unwrap(), 4.0);
    close(m.get(0, 3).unwrap(), 2.0);
    let lumped = b.mass(&c, MassKind::Lumped).unwrap().unwrap();
    for i in 0..6 {
        close(
            lumped.get(i, i).unwrap(),
            if i % 3 == 2 { 0.0 } else { 6.0 },
        );
    }
}
#[test]
fn releases_match_backend_schur_and_static_energy_projection() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let base = beam(RELEASES[0]);
    let k = base.respond(&c, &[0.0; 6], &[]).unwrap().tangent;
    let load = uniform([4.0, -6.0, 0.0], 2.0);
    let f = base.load_vector(&c, &load).unwrap();
    let u = [0.002, -0.003, 0.7, 0.062, 0.004, -0.9];
    let kg = base.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
    for r in &RELEASES[1..] {
        let b = beam(*r);
        let released: Vec<_> = [(r.i, 2), (r.j, 5)]
            .into_iter()
            .filter_map(|(yes, i)| yes.then_some(i))
            .collect();
        let active: Vec<_> = (0..6).filter(|i| !released.contains(i)).collect();
        let n = released.len();
        let mut backend = ReferenceBackend;
        let rr = DMat::new(
            n,
            n,
            released
                .iter()
                .flat_map(|&i| {
                    let k = &k;
                    released.iter().map(move |&j| k.get(i, j).unwrap())
                })
                .collect(),
        )
        .unwrap();
        let ra = DMat::new(
            n,
            active.len(),
            released
                .iter()
                .flat_map(|&i| {
                    active
                        .iter()
                        .map(|&j| -k.get(i, j).unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
        )
        .unwrap();
        let map = backend.solve_dense(&rr, &ra).unwrap();
        let mut h = vec![0.0; 36];
        for &i in &active {
            h[6 * i + i] = 1.0;
        }
        for (i, &row) in released.iter().enumerate() {
            for (j, &col) in active.iter().enumerate() {
                h[6 * row + col] = map.get(i, j).unwrap();
            }
        }
        let h = DMat::new(6, 6, h).unwrap();
        let ht = backend.transpose(&h).unwrap();
        let mut project = |a: &DMat| {
            let ah = backend.matmul(a, &h).unwrap();
            backend.matmul(&ht, &ah).unwrap()
        };
        let kc = b.respond(&c, &u, &[]).unwrap().tangent;
        let expected = project(&k);
        for (v, e) in kc.values().iter().zip(expected.values()) {
            close(*v, *e);
        }
        let expected = project(&kg);
        let gc = b.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
        for (v, e) in gc.values().iter().zip(expected.values()) {
            close(*v, *e);
        }
        for kind in [MassKind::Lumped, MassKind::Consistent] {
            let original = base.mass(&c, kind).unwrap().unwrap();
            let expected = project(&original);
            let mc = b.mass(&c, kind).unwrap().unwrap();
            for (v, e) in mc.values().iter().zip(expected.values()) {
                close(*v, *e);
            }
        }
        let fc = b.load_vector(&c, &load).unwrap();
        for (v, e) in fc.iter().zip(multiply(&ht, &f)) {
            close(*v, e);
        }
        let recovered = b
            .local_displacements(&c, &u, std::slice::from_ref(&load))
            .unwrap();
        let out = b
            .recover(
                &c,
                &u,
                &[],
                std::slice::from_ref(&load),
                &[1.0, 0.0, 0.5, 0.5],
            )
            .unwrap();
        let ku = multiply(&k, &recovered);
        let kcu = multiply(&kc, &u);
        for i in 0..6 {
            close(ku[i] - f[i], out.end_forces_local[i]);
            close(kcu[i] - fc[i], out.end_forces_local[i]);
        }
        for &i in &released {
            close(out.end_forces_local[i], 0.0);
        }
        assert_eq!(out.stations[2], out.stations[3]);
        for (station, node) in [(&out.stations[1], 0), (&out.stations[0], 3)] {
            let sign = if node == 0 { -1.0 } else { 1.0 };
            for (result, idx) in [(0, 0), (1, 1), (5, 2)] {
                close(
                    station.forces_local[result],
                    sign * out.end_forces_local[node + idx],
                );
            }
        }
    }
}
#[test]
fn double_release_has_zero_bending_linear_mass_and_load_recovery() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let b = beam(RELEASES[3]);
    let k = b.respond(&c, &[0.0; 6], &[]).unwrap().tangent;
    for i in [1, 2, 4, 5] {
        for j in 0..6 {
            assert_eq!(k.get(i, j).unwrap(), 0.0);
        }
    }
    let kg = b
        .geometric_stiffness(&c, &[0.0, 0.0, 0.0, 0.06, 0.0, 0.0], &[])
        .unwrap()
        .unwrap();
    close(kg.get(1, 1).unwrap(), 6.0);
    close(kg.get(1, 4).unwrap(), -6.0);
    let m = b.mass(&c, MassKind::Consistent).unwrap().unwrap();
    close(m.get(1, 1).unwrap(), 4.0);
    close(m.get(1, 4).unwrap(), 2.0);
    let load = uniform([0.0, -6.0, 0.0], 0.0);
    let internal = b
        .local_displacements(
            &c,
            &[0.0, 0.0, 999.0, 0.0, 0.0, -999.0],
            std::slice::from_ref(&load),
        )
        .unwrap();
    close(internal[2], -6.0 * 8.0 / (24.0 * 600.0));
    close(internal[5], 6.0 * 8.0 / (24.0 * 600.0));
    let out = b
        .recover(&c, &[0.0; 6], &[], &[load], &[0.0, 0.5, 1.0])
        .unwrap();
    for (station, m) in out.stations.iter().zip([0.0, 3.0, 0.0]) {
        close(station.forces_local[5], m);
    }
    close(out.end_forces_local[1], 6.0);
    close(out.end_forces_local[4], 6.0);
}
#[test]
fn rotated_geometric_load_and_recovery_and_self_weight() {
    let p = ElementParams::default();
    let coords = [[0.1, -0.3, 0.0], [2.1, 1.7, 0.0]];
    let c = ctx(&coords, &p);
    let (s, co) = math::sin_cos(0.61_f64);
    let rotate = |p: [f64; 3]| [co * p[0] - s * p[1], s * p[0] + co * p[1], p[2]];
    let rc = coords.map(rotate);
    let rotated = ctx(&rc, &p);
    let u = [0.01, -0.02, 0.03, 0.05, 0.02, -0.04];
    let ru: Vec<_> = u
        .chunks_exact(3)
        .flat_map(|v| rotate([v[0], v[1], v[2]]))
        .collect();
    let mut q = vec![0.0; 36];
    for a in [0, 3] {
        q[6 * a + a] = co;
        q[6 * a + a + 1] = -s;
        q[6 * (a + 1) + a] = s;
        q[6 * (a + 1) + a + 1] = co;
        q[6 * (a + 2) + a + 2] = 1.0;
    }
    let q = DMat::new(6, 6, q).unwrap();
    let mut backend = ReferenceBackend;
    let qt = backend.transpose(&q).unwrap();
    for r in RELEASES {
        let b = beam(r);
        let kg = b.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
        let rkg = b.geometric_stiffness(&rotated, &ru, &[]).unwrap().unwrap();
        let left = backend.matmul(&q, &kg).unwrap();
        let expected = backend.matmul(&left, &qt).unwrap();
        for (v, e) in rkg.values().iter().zip(expected.values()) {
            close(*v, *e);
        }
        let load = uniform([4.0, -6.0, 0.0], 2.0);
        let rload = uniform(rotate(load.force_per_length), 2.0);
        let f = b.load_vector(&c, &load).unwrap();
        let rf = b.load_vector(&rotated, &rload).unwrap();
        for (v, e) in rf.iter().zip(multiply(&q, &f)) {
            close(*v, e);
        }
        let out = b.recover(&c, &u, &[], &[load], &[0.0, 0.3, 1.0]).unwrap();
        let rout = b
            .recover(&rotated, &ru, &[], &[rload], &[0.0, 0.3, 1.0])
            .unwrap();
        for (v, e) in out.stations.iter().zip(rout.stations) {
            for (v, e) in v.forces_local.iter().zip(e.forces_local) {
                close(*v, e);
            }
        }
        let weight = b.self_weight([0.0, -10.0, 0.0]).unwrap();
        assert_eq!(weight.force_per_length, [0.0, -60.0, 0.0]);
        let f = b.load_vector(&c, &weight).unwrap();
        close(f[0] + f[3], 0.0);
        close(f[1] + f[4], -60.0 * math::sqrt(2.0_f64) * 2.0);
        code(b.self_weight([0.0, 0.0, -10.0]), "ENG-E-0013");
    }
}
#[test]
fn axial_and_distributed_moment_equilibrium_and_superposition() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let b = beam(RELEASES[0]);
    let load = uniform([4.0, -6.0, 0.0], 2.0);
    assert_eq!(
        b.load_vector(&c, &load).unwrap(),
        vec![4.0, -8.0, -2.0, 4.0, -4.0, 2.0]
    );
    let out = b
        .recover(
            &c,
            &[0.0, 0.0, 0.0, 0.06, 0.0, 0.0],
            &[],
            &[load.clone(), load],
            &[0.0, 0.5, 1.0],
        )
        .unwrap();
    for (station, n) in out.stations.iter().zip([20.0, 12.0, 4.0]) {
        close(station.forces_local[0], n);
    }
    // Pure distributed couple m: endpoints receive [-m,+m] transverse forces,
    // with constant V=-m and M=0 in a fixed straight member.
    let out = b
        .recover(
            &c,
            &[0.0; 6],
            &[],
            &[uniform([0.0; 3], 2.0)],
            &[0.0, 0.5, 1.0],
        )
        .unwrap();
    for station in out.stations {
        close(station.forces_local[1], -2.0);
        close(station.forces_local[5], 0.0);
    }
}

#[test]
fn invalid_properties_geometry_states_and_stations() {
    for r in RELEASES {
        for (e, a, i, rho, expected) in [
            (0.0, 2.0, 3.0, 3.0, "ENG-E-0010"),
            (-1.0, 2.0, 3.0, 3.0, "ENG-E-0010"),
            (200.0, 0.0, 3.0, 3.0, "ENG-E-0010"),
            (200.0, -1.0, 3.0, 3.0, "ENG-E-0010"),
            (200.0, 2.0, 0.0, 3.0, "ENG-E-0010"),
            (200.0, 2.0, -1.0, 3.0, "ENG-E-0010"),
            (200.0, 2.0, 3.0, -1.0, "ENG-E-0010"),
            (f64::MAX, 2.0, 3.0, 0.0, "ENG-E-0008"),
            (f64::MIN_POSITIVE, f64::MIN_POSITIVE, 1.0, 0.0, "ENG-E-0010"),
            (f64::MIN_POSITIVE, 1.0, f64::MIN_POSITIVE, 0.0, "ENG-E-0010"),
        ] {
            code(EulerFrame2D::new(e, a, i, rho, r), expected);
        }
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for index in 0..4 {
                let mut v = [200.0, 2.0, 3.0, 3.0];
                v[index] = bad;
                code(EulerFrame2D::new(v[0], v[1], v[2], v[3], r), "ENG-E-0008");
            }
        }
        let p = ElementParams::default();
        let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
        let c = ctx(&coords, &p);
        let b = beam(r);
        for u in [vec![0.0; 5], vec![0.0; 7]] {
            code(b.respond(&c, &u, &[]), "ENG-E-0010");
            code(b.geometric_stiffness(&c, &u, &[]), "ENG-E-0010");
            code(b.recover(&c, &u, &[], &[], &[]), "ENG-E-0010");
            code(b.local_displacements(&c, &u, &[]), "ENG-E-0010");
        }
        code(b.respond(&c, &[0.0; 6], &[0.0]), "ENG-E-0010");
        code(b.geometric_stiffness(&c, &[0.0; 6], &[0.0]), "ENG-E-0010");
        code(b.recover(&c, &[0.0; 6], &[0.0], &[], &[]), "ENG-E-0010");
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for node in 0..2 {
                for axis in 0..3 {
                    let mut ec = coords;
                    ec[node][axis] = bad;
                    let c = ctx(&ec, &p);
                    code(b.respond(&c, &[0.0; 6], &[]), "ENG-E-0008");
                    code(b.mass(&c, MassKind::Consistent), "ENG-E-0008");
                    code(b.load_vector(&c, &uniform([0.0; 3], 0.0)), "ENG-E-0008");
                }
            }
            for axis in 0..6 {
                let mut u = [0.0; 6];
                u[axis] = bad;
                code(b.respond(&c, &u, &[]), "ENG-E-0008");
                code(b.geometric_stiffness(&c, &u, &[]), "ENG-E-0008");
                code(b.recover(&c, &u, &[], &[], &[]), "ENG-E-0008");
                code(b.local_displacements(&c, &u, &[]), "ENG-E-0008");
            }
            code(b.recover(&c, &[0.0; 6], &[], &[], &[bad]), "ENG-E-0008");
        }
        for station in [-0.01, 1.01] {
            code(b.recover(&c, &[0.0; 6], &[], &[], &[station]), "ENG-E-0010");
        }
        for l in [0.0, 1e-13, 1e-12] {
            let ec = [[0.0; 3], [l, 0.0, 0.0]];
            let c = ctx(&ec, &p);
            code(b.respond(&c, &[0.0; 6], &[]), "ENG-E-0005");
            code(b.mass(&c, MassKind::Lumped), "ENG-E-0005");
        }
        code(
            b.respond(&ctx(&coords[..1], &p), &[0.0; 6], &[]),
            "ENG-E-0010",
        );
        let ec = [[0.0; 3], [2.0, 0.0, 0.1]];
        code(b.mass(&ctx(&ec, &p), MassKind::Consistent), "ENG-E-0010");
        code(
            b.mass(
                &ElementCtx {
                    dim: Dim::D3,
                    coords: &coords,
                    params: &p,
                },
                MassKind::Consistent,
            ),
            "ENG-E-0010",
        );
    }
}
#[test]
fn unsupported_loads_nonfinite_loads_and_numeric_overflows() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let tiny_mass = EulerFrame2D::new(1e300, 1e-300, 1.0, 1e-300, RELEASES[0]).unwrap();
    code(tiny_mass.mass(&c, MassKind::Consistent), "ENG-E-0010");
    code(tiny_mass.self_weight([0.0, -10.0, 0.0]), "ENG-E-0010");
    for r in RELEASES {
        let b = beam(r);
        for v in [1.0, 1e-30] {
            for load in [
                uniform([0.0, 0.0, v], 0.0),
                MemberLoad {
                    force_per_length: [0.0; 3],
                    moment_per_length: [v, 0.0, 0.0],
                },
                MemberLoad {
                    force_per_length: [0.0; 3],
                    moment_per_length: [0.0, v, 0.0],
                },
            ] {
                code(b.load_vector(&c, &load), "ENG-E-0013");
                code(
                    b.recover(&c, &[0.0; 6], &[], std::slice::from_ref(&load), &[]),
                    "ENG-E-0013",
                );
                code(b.local_displacements(&c, &[0.0; 6], &[load]), "ENG-E-0013");
            }
        }
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut load = uniform([0.0; 3], 0.0);
                load.force_per_length[axis] = bad;
                code(b.load_vector(&c, &load), "ENG-E-0008");
                code(b.recover(&c, &[0.0; 6], &[], &[load], &[]), "ENG-E-0008");
                let mut load = uniform([0.0; 3], 0.0);
                load.moment_per_length[axis] = bad;
                code(b.load_vector(&c, &load), "ENG-E-0008");
                let mut g = [0.0; 3];
                g[axis] = bad;
                code(b.self_weight(g), "ENG-E-0008");
            }
        }
        let ec = [[-f64::MAX, 0.0, 0.0], [f64::MAX, 0.0, 0.0]];
        code(b.mass(&ctx(&ec, &p), MassKind::Lumped), "ENG-E-0008");
        code(
            b.respond(&c, &[0.0, 0.0, 0.0, f64::MAX, 0.0, 0.0], &[]),
            "ENG-E-0008",
        );
        let load = uniform([f64::MAX, 0.0, 0.0], 0.0);
        code(
            b.recover(&c, &[0.0; 6], &[], &[load.clone(), load], &[]),
            "ENG-E-0008",
        );
        let heavy = EulerFrame2D::new(200.0, 2.0, 3.0, f64::MAX, r).unwrap();
        code(heavy.mass(&c, MassKind::Consistent), "ENG-E-0008");
        code(heavy.self_weight([1.0, 0.0, 0.0]), "ENG-E-0008");
        let ec = [[0.0; 3], [4.0, 0.0, 0.0]];
        code(
            b.load_vector(&ctx(&ec, &p), &uniform([f64::MAX, 0.0, 0.0], 0.0)),
            "ENG-E-0008",
        );
    }
    let b = EulerFrame2D::new(1e300, 1.0, 1.0, 0.0, RELEASES[0]).unwrap();
    let ec = [[0.0; 3], [1e-11, 0.0, 0.0]];
    code(b.respond(&ctx(&ec, &p), &[0.0; 6], &[]), "ENG-E-0008");
    // Giros aislados de un miembro doblemente liberado: mecanismo explícito,
    // nunca una regularización artificial dentro del elemento.
    let k = beam(RELEASES[3])
        .respond(&c, &[0.0; 6], &[])
        .unwrap()
        .tangent;
    let rotational = DMat::new(
        2,
        2,
        [2, 5]
            .into_iter()
            .flat_map(|i| [2, 5].map(|j| k.get(i, j).unwrap()))
            .collect(),
    )
    .unwrap();
    code(
        ReferenceBackend.factorize(&SymMatrix::new(rotational).unwrap()),
        "ENG-E-0006",
    );
}
#[test]
fn factory_validates_required_fields_optional_releases_and_model() {
    let factory = FrameFactory;
    let model = ModelView {
        dim: Dim::D2,
        nodes: &[],
    };
    assert!(factory
        .params_schema()
        .contains("additionalProperties\":false"));
    let JsonValue::Object(fields) = params(RELEASES[0]) else {
        unreachable!()
    };
    code(factory.build(&JsonValue::Null, &model), "ENG-E-0010");
    for key in ["E", "A", "I", "rho"] {
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
            JsonValue::Array(vec![]),
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
    for key in ["release_i", "release_j"] {
        let mut wrong = fields.clone();
        wrong.insert(key.into(), JsonValue::Number(1.0));
        code(
            factory.build(&JsonValue::Object(wrong), &model),
            "ENG-E-0010",
        );
    }
    let mut no_releases = fields.clone();
    no_releases.remove("release_i");
    no_releases.remove("release_j");
    assert_eq!(
        factory
            .build(&JsonValue::Object(no_releases), &model)
            .unwrap()
            .kind(),
        factory.kind()
    );
    let mut extra = fields;
    extra.insert("unknown".into(), JsonValue::Null);
    code(
        factory.build(&JsonValue::Object(extra), &model),
        "ENG-E-0010",
    );
    for (dim, nodes, expected) in [
        (Dim::D3, vec![], "ENG-E-0010"),
        (
            Dim::D2,
            vec![(NodeId(1), [0.0; 3]), (NodeId(1), [0.0; 3])],
            "ENG-E-0002",
        ),
        (
            Dim::D2,
            vec![(NodeId(1), [f64::NAN, 0.0, 0.0])],
            "ENG-E-0008",
        ),
        (Dim::D2, vec![(NodeId(1), [0.0, 0.0, 0.1])], "ENG-E-0010"),
    ] {
        code(
            factory.build(&params(RELEASES[0]), &ModelView { dim, nodes: &nodes }),
            expected,
        );
    }
}

#[test]
fn double_release_with_huge_ei_has_no_spurious_shear() {
    // J4 (#31): EI = 1e20, doble liberación, L = 3.1: el cortante y el momento
    // deben ser exactamente los de la carga (sin ruido de redondeo ~64 N / 198 N·m).
    let p = ElementParams::default();
    let coords = [[0.0; 3], [3.1, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let b = EulerFrame2D::new(1e20, 2.0, 1.0, 3.0, RELEASES[3]).unwrap();
    let u = [0.0, 0.37, 0.0, 0.0, -0.21, 0.0];
    let out = b.recover(&c, &u, &[], &[], &[0.0, 0.5, 1.0]).unwrap();
    for v in [out.end_forces_local[1], out.end_forces_local[4]] {
        assert_eq!(v, 0.0);
    }
    for s in &out.stations {
        assert_eq!(s.forces_local[1], 0.0);
        assert_eq!(s.forces_local[5], 0.0);
    }
    // Con carga uniforme q: V extremos = ±qL/2 (equilibrio de viga articulada).
    let load = uniform([0.0, -6.0, 0.0], 0.0);
    let out = b
        .recover(&c, &u, &[], std::slice::from_ref(&load), &[0.5])
        .unwrap();
    assert!((out.end_forces_local[1] - 6.0 * 3.1 / 2.0).abs() <= 1e-12);
    assert!((out.end_forces_local[4] - 6.0 * 3.1 / 2.0).abs() <= 1e-12);
    assert!((out.stations[0].forces_local[5] - 6.0 * 3.1 * 3.1 / 8.0).abs() <= 1e-12);
}

#[test]
fn double_release_shear_is_exact_for_ei_1e12_and_1e30() {
    // Seguimiento de #41 (J4): el test con EI=1e20 pasaba también sin el arreglo
    // en algunos redondeos. Aquí se barren EI y longitudes y estados u arbitrarios:
    // sin poner a cero la flexión elástica condensada aparece cortante espurio
    // (~EI/L³·ε), con el arreglo es exactamente cero.
    let p = ElementParams::default();
    let mut spurious_without_fix_possible = 0;
    for ei in [1e12, 1e20, 1e30] {
        for length in [3.1, 0.7, 11.3, 2.0 / 3.0] {
            let coords = [[0.0; 3], [length, 0.0, 0.0]];
            let c = ctx(&coords, &p);
            let b = EulerFrame2D::new(ei, 2.0, 1.0, 3.0, RELEASES[3]).unwrap();
            for u in [
                [0.0, 0.37, 0.11, 0.0, -0.21, 0.07],
                [0.013, -0.9, 0.5, -0.4, 0.3, -0.2],
                [0.0, 1e-3, 1.0 / 3.0, 0.0, 1e-3, -1.0 / 7.0],
            ] {
                let out = b.recover(&c, &u, &[], &[], &[0.0, 0.25, 0.5, 1.0]).unwrap();
                assert_eq!(out.end_forces_local[1], 0.0, "EI={ei} L={length}");
                assert_eq!(out.end_forces_local[4], 0.0, "EI={ei} L={length}");
                for s in &out.stations {
                    assert_eq!(s.forces_local[1], 0.0, "EI={ei} L={length}");
                    assert_eq!(s.forces_local[5], 0.0, "EI={ei} L={length}");
                }
                spurious_without_fix_possible += 1;
            }
        }
    }
    assert_eq!(spurious_without_fix_possible, 36);
}
