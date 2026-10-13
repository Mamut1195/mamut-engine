//! Conformidad element/1, covariancia espacial, energía y diagnósticos.
mod fixtures;
use engine_contracts::{
    conformance::{check_element, check_element_factory},
    Element, ElementFactory, ElementParams, JsonValue, MassKind, ModelView,
};
use engine_core::{Dim, DofIndex, EResult, NodeId};
use engine_elements_frame3d::{geometric_producer, EulerFrame3D, FrameFactory, MomentReleases};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use fixtures::*;

fn code<T>(result: EResult<T>, expected: &str) {
    assert_eq!(result.err().expect("debe fallar").code, expected);
}
fn samples() -> Vec<Vec<f64>> {
    vec![
        vec![0.0; 12],
        vec![
            0.002, -0.003, 0.005, 0.001, -0.002, 0.004, -0.001, 0.004, -0.007, 0.003, 0.005, -0.006,
        ],
    ]
}
#[test]
fn element_and_factory_pass_existing_kit_isotropic_sections_and_releases() {
    cited();
    let p = ElementParams::default();
    let coords = [[0.2, -0.4, 0.3], [2.0, 1.3, 3.4]];
    let c = ctx(&coords, &p);
    let nodes = [(NodeId(1), coords[0]), (NodeId(2), coords[1])];
    let model = ModelView {
        dim: Dim::D3,
        nodes: &nodes,
    };
    // El kit rota ctx.coords pero no self.orientation. La sección isotrópica
    // y las liberaciones biaxiales pareadas son independientes del roll local.
    // Se verifica toda la anisotropía aparte rotando TAMBIÉN la orientación.
    for mask in 0..16 {
        let r = MomentReleases {
            i: [mask & 1 != 0, mask & 2 != 0, mask & 2 != 0],
            j: [mask & 4 != 0, mask & 8 != 0, mask & 8 != 0],
        };
        let b = EulerFrame3D::new(E, G, A, J, IY, IY, RHO, [0.0, 0.0, 1.0], r).unwrap();
        let issues = check_element(&b, &c, &samples(), 1e-7);
        assert!(issues.is_empty(), "mask={mask}: {issues:?}");
        let JsonValue::Object(mut fields) = params(r, b.orientation()) else {
            unreachable!()
        };
        fields.insert("Iz".into(), JsonValue::Number(IY));
        let issues = check_element_factory(
            &FrameFactory,
            &JsonValue::Object(fields),
            &model,
            &c,
            &samples(),
            1e-7,
        );
        assert!(issues.is_empty(), "mask={mask}: {issues:?}");
        assert_eq!(b.state_len(), 0);
        assert!(b.initial_state().is_empty());
    }
}
#[test]
fn anisotropic_tangent_finite_differences_and_repeatability_all_64_releases() {
    let p = ElementParams::default();
    let coords = [[0.2, -0.4, 0.3], [2.0, 1.3, 3.4]];
    let c = ctx(&coords, &p);
    let u = samples().remove(1);
    let h = 1e-7;
    for mask in 0..64 {
        let b = beam(releases(mask));
        let response = b.respond(&c, &u, &[]).unwrap();
        assert_eq!(response, b.respond(&c, &u, &[]).unwrap());
        assert!(response.trial_state.is_empty());
        for j in 0..12 {
            let mut plus = u.clone();
            let mut minus = u.clone();
            plus[j] += h;
            minus[j] -= h;
            let fp = b.respond(&c, &plus, &[]).unwrap().internal_force;
            let fm = b.respond(&c, &minus, &[]).unwrap().internal_force;
            for i in 0..12 {
                let fd = (fp[i] - fm[i]) / (2.0 * h);
                let exact = response.tangent.get(i, j).unwrap();
                assert!(
                    (fd - exact).abs() < 1e-7 + 1e-6 * exact.abs(),
                    "mask={mask}, ({i},{j}), fd={fd}, K={exact}"
                );
            }
        }
    }
}
#[test]
fn stiffness_symmetry_six_rigid_modes_and_total_mass() {
    cited();
    let p = ElementParams::default();
    let coords = [[1.0, 2.0, 3.0], [4.0, 6.0, 8.0]];
    let c = ctx(&coords, &p);
    let length = 3.0_f64.hypot(4.0).hypot(5.0);
    for mask in 0..64 {
        let b = beam(releases(mask));
        let k = b.respond(&c, &[0.0; 12], &[]).unwrap().tangent;
        SymMatrix::new(k.clone()).unwrap();
        for axis in 0..3 {
            let direction = std::array::from_fn(|i| if i == axis { 1.0 } else { 0.0 });
            let mut translation = [0.0; 12];
            translation[..3].copy_from_slice(&direction);
            translation[6..9].copy_from_slice(&direction);
            let mut rotation = [0.0; 12];
            for (n, coord) in coords.into_iter().enumerate() {
                rotation[n * 6..n * 6 + 3].copy_from_slice(&cross(direction, coord));
                rotation[n * 6 + 3..n * 6 + 6].copy_from_slice(&direction);
            }
            for mode in [translation, rotation] {
                for f in multiply(&k, &mode) {
                    close(f, 0.0);
                }
            }
            for kind in [MassKind::Lumped, MassKind::Consistent] {
                let m = b.mass(&c, kind).unwrap().unwrap();
                SymMatrix::new(m.clone()).unwrap();
                let energy: f64 = translation
                    .iter()
                    .zip(multiply(&m, &translation))
                    .map(|(u, f)| u * f)
                    .sum();
                close(energy, RHO * A * length);
            }
        }
    }
}
// Rodrigues derivado de exp(ángulo [n]×); eje genérico fuera de planos coordenados.
fn rotation() -> [[f64; 3]; 3] {
    let n = [
        1.0 / 14.0_f64.sqrt(),
        2.0 / 14.0_f64.sqrt(),
        3.0 / 14.0_f64.sqrt(),
    ];
    let (s, c) = 0.73_f64.sin_cos();
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            let mut e = [0.0; 3];
            e[j] = 1.0;
            c * e[i] + (1.0 - c) * n[i] * n[j] + s * cross(n, e)[i]
        })
    })
}
fn rotate(r: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    r.map(|row| row.iter().zip(v).map(|(a, b)| a * b).sum())
}
fn rotate_vector(r: [[f64; 3]; 3], v: &[f64]) -> Vec<f64> {
    v.chunks_exact(3)
        .flat_map(|v| rotate(r, [v[0], v[1], v[2]]))
        .collect()
}
#[test]
fn rigid_rotation_of_complete_model_k_force_load_mass_kg_and_recovery() {
    cited();
    let p = ElementParams::default();
    let coords = [[0.2, -0.4, 0.3], [2.0, 1.3, 3.4]];
    let c = ctx(&coords, &p);
    let r = rotation();
    let rc = coords.map(|v| {
        let q = rotate(r, v);
        [q[0] + 7.0, q[1] - 3.0, q[2] + 5.0]
    });
    let cr = ctx(&rc, &p);
    let u = samples().remove(1);
    let ru = rotate_vector(r, &u);
    let mut q = vec![0.0; 144];
    for start in [0, 3, 6, 9] {
        for i in 0..3 {
            for j in 0..3 {
                q[12 * (start + i) + start + j] = r[i][j];
            }
        }
    }
    let q = DMat::new(12, 12, q).unwrap();
    let mut backend = ReferenceBackend;
    let qt = backend.transpose(&q).unwrap();
    for mask in 0..64 {
        let releases = releases(mask);
        let b = beam(releases);
        let br = oriented(releases, rotate(r, b.orientation()));
        let a = b.respond(&c, &u, &[]).unwrap();
        let ar = br.respond(&cr, &ru, &[]).unwrap();
        let mut matrices = vec![
            (a.tangent, ar.tangent),
            (
                b.geometric_stiffness(&c, &u, &[]).unwrap().unwrap(),
                br.geometric_stiffness(&cr, &ru, &[]).unwrap().unwrap(),
            ),
        ];
        for kind in [MassKind::Consistent, MassKind::Lumped] {
            matrices.push((
                b.mass(&c, kind).unwrap().unwrap(),
                br.mass(&cr, kind).unwrap().unwrap(),
            ));
        }
        for (m, mr) in matrices {
            let qm = backend.matmul(&q, &m).unwrap();
            let expected = backend.matmul(&qm, &qt).unwrap();
            for (a, b) in mr.values().iter().zip(expected.values()) {
                close(*a, *b);
            }
        }
        for (a, b) in ar
            .internal_force
            .into_iter()
            .zip(rotate_vector(r, &a.internal_force))
        {
            close(a, b);
        }
        // En doble torsión se usa m=0: no hay carga neta en su mecanismo libre.
        let moments = if releases.i[0] && releases.j[0] {
            [0.0; 3]
        } else {
            [1.0, 2.0, -3.0]
        };
        let loading = load([4.0, -6.0, 7.0], moments);
        let lr = load(rotate(r, loading.force_per_length), rotate(r, moments));
        let f = b.load_vector(&c, &loading).unwrap();
        let fr = br.load_vector(&cr, &lr).unwrap();
        for (a, b) in fr.into_iter().zip(rotate_vector(r, &f)) {
            close(a, b);
        }
        let out = b
            .recover(
                &c,
                &u,
                &[],
                std::slice::from_ref(&loading),
                &[0.0, 0.3, 1.0],
            )
            .unwrap();
        let outr = br
            .recover(&cr, &ru, &[], std::slice::from_ref(&lr), &[0.0, 0.3, 1.0])
            .unwrap();
        for (a, b) in out.end_forces_local.iter().zip(outr.end_forces_local) {
            close(*a, b);
        }
        for (a, b) in out.stations.iter().zip(outr.stations) {
            for (a, b) in a.forces_local.iter().zip(b.forces_local) {
                close(*a, b);
            }
        }
        let d = b.local_displacements(&c, &u, &[loading]).unwrap();
        let dr = br.local_displacements(&cr, &ru, &[lr]).unwrap();
        for (a, b) in d.into_iter().zip(dr) {
            close(a, b);
        }
    }
}

#[test]
fn local_load_conversion_and_self_weight() {
    cited();
    let p = ElementParams::default();
    let coords = [[1.0, 2.0, 3.0], [1.0, 2.0, 7.0]];
    let c = ctx(&coords, &p);
    let b = oriented(MomentReleases::default(), [1.0, 0.0, 0.0]); // x=Z, y=−Y, z=X.
    let local = load([2.0, 3.0, 4.0], [5.0, 6.0, 7.0]);
    let global = b.global_load(&c, &local).unwrap();
    assert_eq!(global, load([4.0, -3.0, 2.0], [7.0, -6.0, 5.0]));
    let base = [[0.0; 3], [4.0, 0.0, 0.0]];
    let base = ctx(&base, &p);
    let reference = beam(MomentReleases::default());
    let f = reference.load_vector(&base, &local).unwrap();
    let fg = b.load_vector(&c, &global).unwrap();
    for chunk in 0..4 {
        for (actual, expected) in fg[3 * chunk..3 * chunk + 3].iter().zip([
            f[3 * chunk + 2],
            -f[3 * chunk + 1],
            f[3 * chunk],
        ]) {
            close(*actual, expected);
        }
    }
    let weight = b.self_weight([0.0, 0.0, -10.0]).unwrap();
    assert_eq!(weight.force_per_length, [0.0, 0.0, -60.0]);
    let f = b.load_vector(&c, &weight).unwrap();
    close(f[2] + f[8], -240.0);
}

// Integra productos de polinomios en t=x/L, independiente de matrices de producción.
fn integral(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .enumerate()
        .flat_map(|(i, a)| {
            b.iter()
                .enumerate()
                .map(move |(j, b)| a * b / (i + j + 1) as f64)
        })
        .sum()
}
fn derivative(p: &[f64]) -> Vec<f64> {
    p.iter()
        .enumerate()
        .skip(1)
        .map(|(i, v)| *v * i as f64 / L)
        .collect()
}
#[test]
fn independent_polynomial_energy_integrals_k_mass_and_kg() {
    cited();
    let p = ElementParams::default();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let b = beam(MomentReleases::default());
    let k = b.respond(&c, &[0.0; 12], &[]).unwrap().tangent;
    let m = b.mass(&c, MassKind::Consistent).unwrap().unwrap();
    let mut u = [0.0; 12];
    u[6] = L / (E * A);
    let kg = b.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
    let h = [
        vec![1.0, 0.0, -3.0, 2.0],
        vec![0.0, L, -2.0 * L, L],
        vec![0.0, 0.0, 3.0, -2.0],
        vec![0.0, 0.0, -L, L],
    ];
    let hp = h.each_ref().map(|v| derivative(v));
    let hpp = hp.each_ref().map(|v| derivative(v));
    let mut ke = vec![0.0; 144];
    let mut me = ke.clone();
    let mut ge = ke.clone();
    for (ids, sign, ei) in [([1, 5, 7, 11], 1.0, E * IZ), ([2, 4, 8, 10], -1.0, E * IY)] {
        let signs = [1.0, sign, 1.0, sign];
        for i in 0..4 {
            for j in 0..4 {
                let index = 12 * ids[i] + ids[j];
                let factor = signs[i] * signs[j];
                ke[index] = factor * ei * L * integral(&hpp[i], &hpp[j]);
                me[index] = factor * RHO * A * L * integral(&h[i], &h[j]);
                ge[index] = factor * L * integral(&hp[i], &hp[j]);
            }
        }
    }
    let h = [vec![1.0, -1.0], vec![0.0, 1.0]];
    let hp = h.each_ref().map(|v| derivative(v));
    for (ids, rigidity, density) in [([0, 6], E * A, RHO * A), ([3, 9], G * J, RHO * (IY + IZ))] {
        for i in 0..2 {
            for j in 0..2 {
                let index = 12 * ids[i] + ids[j];
                ke[index] = rigidity * L * integral(&hp[i], &hp[j]);
                me[index] = density * L * integral(&h[i], &h[j]);
            }
        }
    }
    for (matrix, expected) in [(&k, ke), (&m, me), (&kg, ge)] {
        for (actual, expected) in matrix.values().iter().zip(expected) {
            close(*actual, expected);
        }
    }
}

#[test]
fn independent_schur_projection_and_loaded_internal_rotations() {
    cited();
    let p = ElementParams::default();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let full = beam(MomentReleases::default());
    let k = full.respond(&c, &[0.0; 12], &[]).unwrap().tangent;
    let u = samples().remove(1);
    let loading = load([2.0, -3.0, 4.0], [0.0, 5.0, -6.0]);
    let f = full.load_vector(&c, &loading).unwrap();
    let mut axial_u = [0.0; 12];
    axial_u[6] = 0.06;
    let mass = full.mass(&c, MassKind::Consistent).unwrap().unwrap();
    let kg = full
        .geometric_stiffness(&c, &axial_u, &[])
        .unwrap()
        .unwrap();
    for mask in 1..64 {
        let release = releases(mask);
        let both_t = release.i[0] && release.j[0];
        let b = beam(release);
        let is_released = |i: usize| {
            i % 6 >= 3
                && if i < 6 {
                    release.i[i - 3]
                } else {
                    release.j[i - 9]
                }
        };
        // Doble torsión: fija gauge rx_i=0 y resuelve rx_j; Kc y fc no dependen del gauge.
        let rr: Vec<_> = (0..12)
            .filter(|&i| is_released(i) && !(both_t && i == 3))
            .collect();
        let aa: Vec<_> = (0..12).filter(|&i| !is_released(i)).collect();
        let kra = submatrix(&k, &rr, &aa);
        let krr = submatrix(&k, &rr, &rr);
        let inverse_product = ReferenceBackend.solve_dense(&krr, &kra).unwrap();
        let mut h = vec![0.0; 144];
        for &i in &aa {
            h[12 * i + i] = 1.0;
        }
        for (r, &i) in rr.iter().enumerate() {
            for (a, &j) in aa.iter().enumerate() {
                h[12 * i + j] = -inverse_product.get(r, a).unwrap();
            }
        }
        let h = DMat::new(12, 12, h).unwrap();
        let mut backend = ReferenceBackend;
        let ht = backend.transpose(&h).unwrap();
        let hk = backend.matmul(&ht, &k).unwrap();
        let kc = backend.matmul(&hk, &h).unwrap();
        let actual = b.respond(&c, &u, &[]).unwrap();
        for (a, e) in actual.tangent.values().iter().zip(kc.values()) {
            close(*a, *e);
        }
        let fc = multiply(&ht, &f);
        for (a, e) in b.load_vector(&c, &loading).unwrap().into_iter().zip(&fc) {
            close(a, *e);
        }
        let fr: DMat = DMat::new(rr.len(), 1, rr.iter().map(|&i| f[i]).collect()).unwrap();
        let dc = backend.solve_dense(&krr, &fr).unwrap();
        let mut d = multiply(&h, &u);
        for (r, &i) in rr.iter().enumerate() {
            d[i] += dc.get(r, 0).unwrap();
        }
        let recovered = b
            .local_displacements(&c, &u, std::slice::from_ref(&loading))
            .unwrap();
        for (i, (a, e)) in recovered.into_iter().zip(&d).enumerate() {
            if !(both_t && (i == 3 || i == 9)) {
                close(a, *e);
            }
        }
        let ends: Vec<_> = multiply(&k, &d)
            .into_iter()
            .zip(&f)
            .map(|(k, f)| k - f)
            .collect();
        let actual = b
            .recover(&c, &u, &[], std::slice::from_ref(&loading), &[])
            .unwrap();
        for (a, e) in actual.end_forces_local.into_iter().zip(ends) {
            close(a, e);
        }
        for (matrix, actual) in [
            (
                &kg,
                b.geometric_stiffness(&c, &axial_u, &[]).unwrap().unwrap(),
            ),
            (&mass, b.mass(&c, MassKind::Consistent).unwrap().unwrap()),
        ] {
            if both_t && std::ptr::eq(matrix, &mass) {
                continue;
            } // Masa depende del gauge torsional declarado.
            let left = backend.matmul(&ht, matrix).unwrap();
            let projected = backend.matmul(&left, &h).unwrap();
            for (a, e) in actual.values().iter().zip(projected.values()) {
                close(*a, *e);
            }
        }
    }
}

#[test]
fn invalid_properties_geometry_orientation_vectors_history_and_loads() {
    let make = |v: [f64; 7], z| {
        EulerFrame3D::new(
            v[0],
            v[1],
            v[2],
            v[3],
            v[4],
            v[5],
            v[6],
            z,
            MomentReleases::default(),
        )
    };
    let values = [E, G, A, J, IY, IZ, RHO];
    for axis in 0..7 {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut v = values;
            v[axis] = bad;
            code(make(v, [0.0, 0.0, 1.0]), "ENG-E-0008");
        }
        let mut v = values;
        v[axis] = -1.0;
        code(make(v, [0.0, 0.0, 1.0]), "ENG-E-0010");
        if axis < 6 {
            v[axis] = 0.0;
            code(make(v, [0.0, 0.0, 1.0]), "ENG-E-0010");
        }
    }
    code(
        make([f64::MAX, G, A, J, IY, IZ, 0.0], [0.0, 0.0, 1.0]),
        "ENG-E-0008",
    );
    code(
        make([E, f64::MAX, A, J, IY, IZ, 0.0], [0.0, 0.0, 1.0]),
        "ENG-E-0008",
    );
    code(
        make([1e-300, G, 1e-300, J, IY, IZ, 0.0], [0.0, 0.0, 1.0]),
        "ENG-E-0010",
    );
    code(
        make([E, 1e-300, A, 1e-300, IY, IZ, 0.0], [0.0, 0.0, 1.0]),
        "ENG-E-0010",
    );
    code(make(values, [0.0; 3]), "ENG-E-0010");
    code(make(values, [0.0, f64::NAN, 1.0]), "ENG-E-0008");
    let p = ElementParams::default();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    for z in [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 1e-14, 0.0]] {
        let b = make(values, z).unwrap();
        code(b.respond(&c, &[0.0; 12], &[]), "ENG-E-0010");
        code(b.mass(&c, MassKind::Consistent), "ENG-E-0010");
    }
    let b = beam(MomentReleases::default());
    for z in [[0.0, 0.0, f64::MAX], [0.0, 0.0, 1e-310], [2.0, 0.0, 1.0]] {
        let k = make(values, z)
            .unwrap()
            .respond(&c, &[0.0; 12], &[])
            .unwrap()
            .tangent;
        assert_eq!(k, b.respond(&c, &[0.0; 12], &[]).unwrap().tangent);
    }
    for length in [0.0, 1e-13, 1e-12] {
        code(
            b.respond(&ctx(&[[0.0; 3], [length, 0.0, 0.0]], &p), &[0.0; 12], &[]),
            "ENG-E-0005",
        );
    }
    code(
        b.respond(&ctx(&coords[..1], &p), &[0.0; 12], &[]),
        "ENG-E-0010",
    );
    let mut wrong = ctx(&coords, &p);
    wrong.dim = Dim::D2;
    code(b.respond(&wrong, &[0.0; 12], &[]), "ENG-E-0010");
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for n in 0..2 {
            for axis in 0..3 {
                let mut ec = coords;
                ec[n][axis] = bad;
                code(b.mass(&ctx(&ec, &p), MassKind::Consistent), "ENG-E-0008");
            }
        }
        for axis in 0..12 {
            let mut u = [0.0; 12];
            u[axis] = bad;
            code(b.respond(&c, &u, &[]), "ENG-E-0008");
            code(b.geometric_stiffness(&c, &u, &[]), "ENG-E-0008");
            code(b.recover(&c, &u, &[], &[], &[]), "ENG-E-0008");
            code(b.local_displacements(&c, &u, &[]), "ENG-E-0008");
        }
        for axis in 0..6 {
            let mut q = load([0.0; 3], [0.0; 3]);
            if axis < 3 {
                q.force_per_length[axis] = bad;
            } else {
                q.moment_per_length[axis - 3] = bad;
            }
            code(b.load_vector(&c, &q), "ENG-E-0008");
            code(b.global_load(&c, &q), "ENG-E-0008");
            code(
                b.recover(&c, &[0.0; 12], &[], std::slice::from_ref(&q), &[]),
                "ENG-E-0008",
            );
        }
        code(b.self_weight([0.0, bad, 0.0]), "ENG-E-0008");
        code(b.recover(&c, &[0.0; 12], &[], &[], &[bad]), "ENG-E-0008");
    }
    for size in [0, 11, 13] {
        let u = vec![0.0; size];
        code(b.respond(&c, &u, &[]), "ENG-E-0010");
        code(b.geometric_stiffness(&c, &u, &[]), "ENG-E-0010");
        code(b.recover(&c, &u, &[], &[], &[]), "ENG-E-0010");
        code(b.local_displacements(&c, &u, &[]), "ENG-E-0010");
    }
    code(b.respond(&c, &[0.0; 12], &[0.0]), "ENG-E-0010");
    code(b.geometric_stiffness(&c, &[0.0; 12], &[0.0]), "ENG-E-0010");
    code(b.recover(&c, &[0.0; 12], &[0.0], &[], &[]), "ENG-E-0010");
    for station in [-0.001, 1.001] {
        code(
            b.recover(&c, &[0.0; 12], &[], &[], &[station]),
            "ENG-E-0010",
        );
    }
}

#[test]
fn enormous_ei_gj_double_releases_and_overflows_return_diagnostics() {
    let p = ElementParams::default();
    let coords = [[0.0; 3], [3.1, 0.0, 0.0]];
    let c = ctx(&coords, &p);
    let u = [
        0.0, 0.37, 0.27, 0.02, 0.31, -0.21, 0.0, -0.21, 0.38, -0.04, -0.18, 0.23,
    ];
    for mask in 0..64 {
        let b = EulerFrame3D::new(
            1e200,
            1e200,
            2.0,
            1.0,
            1.0,
            1.0,
            3.0,
            [0.0, 0.0, 1.0],
            releases(mask),
        )
        .unwrap();
        assert!(b
            .respond(&c, &u, &[])
            .unwrap()
            .internal_force
            .iter()
            .all(|x| x.is_finite()));
        assert!(b
            .geometric_stiffness(&c, &u, &[])
            .unwrap()
            .unwrap()
            .values()
            .iter()
            .all(|x| x.is_finite()));
        let q = load([0.0, -6.0, 8.0], [0.0; 3]);
        let out = b
            .recover(&c, &u, &[], std::slice::from_ref(&q), &[0.0, 0.5, 1.0])
            .unwrap();
        assert!(out.end_forces_local.iter().all(|x| x.is_finite()));
        b.local_displacements(&c, &u, &[q]).unwrap();
        let r = b.releases();
        for (axis, v) in [(1, 2), (2, 1)] {
            if r.i[axis] && r.j[axis] {
                assert_eq!(out.end_forces_local[axis + 3], 0.0);
                assert_eq!(out.end_forces_local[axis + 9], 0.0);
                let value = if v == 1 { -6.0 } else { 8.0 };
                close(out.end_forces_local[v], -value * 3.1 / 2.0);
                close(out.end_forces_local[v + 6], -value * 3.1 / 2.0);
            }
        }
    }
    let b = beam(MomentReleases::default());
    let ec = [[-f64::MAX, 0.0, 0.0], [f64::MAX, 0.0, 0.0]];
    code(b.mass(&ctx(&ec, &p), MassKind::Lumped), "ENG-E-0008");
    let mut huge = [0.0; 12];
    huge[6] = f64::MAX;
    code(b.respond(&c, &huge, &[]), "ENG-E-0008");
    code(b.geometric_stiffness(&c, &huge, &[]), "ENG-E-0008");
    let q = load([f64::MAX, 0.0, 0.0], [0.0; 3]);
    code(b.load_vector(&c, &q), "ENG-E-0008");
    code(
        b.recover(&c, &[0.0; 12], &[], &[q.clone(), q], &[]),
        "ENG-E-0008",
    );
    let heavy = EulerFrame3D::new(
        E,
        G,
        A,
        J,
        IY,
        IZ,
        f64::MAX,
        [0.0, 0.0, 1.0],
        MomentReleases::default(),
    )
    .unwrap();
    code(heavy.mass(&c, MassKind::Consistent), "ENG-E-0008");
    code(heavy.self_weight([0.0, 0.0, -10.0]), "ENG-E-0008");
    let stiff = EulerFrame3D::new(
        1e300,
        1e300,
        1.0,
        1.0,
        1.0,
        1.0,
        0.0,
        [0.0, 0.0, 1.0],
        MomentReleases::default(),
    )
    .unwrap();
    code(
        stiff.respond(&ctx(&[[0.0; 3], [1e-11, 0.0, 0.0]], &p), &[0.0; 12], &[]),
        "ENG-E-0008",
    );
    assert!(stiff.mass(&c, MassKind::Consistent).unwrap().is_none());
    let b = beam(releases(63));
    let out = b.recover(&c, &u, &[], &[], &[0.0, 0.5, 1.0]).unwrap();
    assert!(out.end_forces_local.iter().all(|&x| x == 0.0));
    let f = load([0.0; 3], [1e-30, 0.0, 0.0]);
    code(b.load_vector(&c, &f), "ENG-E-0006");
    code(
        b.local_displacements(&c, &u, std::slice::from_ref(&f)),
        "ENG-E-0006",
    );
    code(b.recover(&c, &u, &[], &[f], &[]), "ENG-E-0006");
    let k = b.respond(&c, &u, &[]).unwrap().tangent;
    code(
        ReferenceBackend.factorize(&SymMatrix::new(submatrix(&k, &[3, 9], &[3, 9])).unwrap()),
        "ENG-E-0006",
    );
}

#[test]
fn factory_schema_and_invalid_fields_and_model() {
    let factory = FrameFactory;
    let model = ModelView {
        dim: Dim::D3,
        nodes: &[],
    };
    assert!(factory
        .params_schema()
        .contains("additionalProperties\":false"));
    let JsonValue::Object(fields) = params(MomentReleases::default(), [0.0, 0.0, 1.0]) else {
        unreachable!()
    };
    code(factory.build(&JsonValue::Null, &model), "ENG-E-0010");
    for key in ["E", "G", "A", "J", "Iy", "Iz", "rho", "orientation"] {
        let mut f = fields.clone();
        f.remove(key);
        code(factory.build(&JsonValue::Object(f), &model), "ENG-E-0010");
        let mut f = fields.clone();
        f.insert(key.into(), JsonValue::Bool(true));
        code(factory.build(&JsonValue::Object(f), &model), "ENG-E-0010");
    }
    for key in ["E", "G", "A", "J", "Iy", "Iz", "rho"] {
        let mut f = fields.clone();
        f.insert(key.into(), JsonValue::Number(f64::NAN));
        code(factory.build(&JsonValue::Object(f), &model), "ENG-E-0008");
    }
    for key in ["release_i", "release_j", "orientation"] {
        for bad in [
            JsonValue::Array(vec![]),
            JsonValue::Array(vec![JsonValue::Null; 3]),
            JsonValue::String("0".into()),
        ] {
            let mut f = fields.clone();
            f.insert(key.into(), bad);
            code(factory.build(&JsonValue::Object(f), &model), "ENG-E-0010");
        }
    }
    let mut f = fields.clone();
    f.insert(
        "orientation".into(),
        JsonValue::Array(vec![JsonValue::Number(0.0); 3]),
    );
    code(factory.build(&JsonValue::Object(f), &model), "ENG-E-0010");
    let mut f = fields.clone();
    f.remove("release_i");
    f.remove("release_j");
    assert_eq!(
        factory.build(&JsonValue::Object(f), &model).unwrap().kind(),
        factory.kind()
    );
    let mut f = fields;
    f.insert("extra".into(), JsonValue::Null);
    code(factory.build(&JsonValue::Object(f), &model), "ENG-E-0010");
    for (dim, nodes, expected) in [
        (Dim::D2, vec![], "ENG-E-0010"),
        (
            Dim::D3,
            vec![(NodeId(1), [0.0; 3]), (NodeId(1), [1.0; 3])],
            "ENG-E-0002",
        ),
        (
            Dim::D3,
            vec![(NodeId(1), [f64::NAN, 0.0, 0.0])],
            "ENG-E-0008",
        ),
    ] {
        code(
            factory.build(
                &params(MomentReleases::default(), [0.0, 0.0, 1.0]),
                &ModelView { dim, nodes: &nodes },
            ),
            expected,
        );
    }
}

#[test]
fn producer_owns_context_and_validates_locations_and_axial_states() {
    let b = beam(MomentReleases::default());
    let producer = {
        let p = ElementParams::default();
        let coords = [[0.0; 3], [L, 0.0, 0.0]];
        let c = ctx(&coords, &p);
        code(geometric_producer(b, &c, vec![DofIndex(0)]), "ENG-E-0010");
        code(
            geometric_producer(b, &c, vec![DofIndex(0); 12]),
            "ENG-E-0002",
        );
        geometric_producer(b, &c, (10..22).map(DofIndex).collect()).unwrap()
    };
    assert_eq!(
        producer.location,
        (10..22).map(DofIndex).collect::<Vec<_>>()
    );
    for delta in [-0.06, 0.0, 0.06] {
        let mut u = [0.0; 12];
        u[6] = delta;
        u[7] = 0.3;
        let m = (producer.eval)(&u).unwrap();
        close(
            m.get(1, 1).unwrap(),
            (E * A * delta / L) * 36.0 / (30.0 * L),
        );
        close(m.get(2, 2).unwrap(), m.get(1, 1).unwrap());
    }
    code((producer.eval)(&[0.0]), "ENG-E-0010");
    let mut u = [0.0; 12];
    u[6] = f64::NAN;
    code((producer.eval)(&u), "ENG-E-0008");
    let p = ElementParams::default();
    code(
        geometric_producer(b, &ctx(&[[0.0; 3]; 2], &p), (0..12).map(DofIndex).collect()),
        "ENG-E-0005",
    );
}
