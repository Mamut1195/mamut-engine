//! source: derivaciones analíticas propias, sin fuentes externas ni oráculos.
//! citation: provenance/ENG-ASM-002.md (energía, torsión y modelo de dos pisos).
use engine_assembly::{
    Constraints, DofMap, ElementContribution, GeometricProducer, LinearSystem, PrescribedDof,
    RigidDiaphragm,
};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MassKind, SystemView};
use engine_core::{Dim, Dof, DofIndex, EResult, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const XY: [Dof; 3] = [Dof::Ux, Dof::Uy, Dof::Rz];
const POINTS: [[f64; 2]; 3] = [[-2.0, -1.0], [2.0, -1.0], [0.0, 2.0]];
fn case() -> CaseRef {
    CaseRef("lateral".into())
}
fn close(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance * b.abs().max(1.0), "{a} != {b}");
}
fn code<T>(result: EResult<T>, expected: &str) {
    match result {
        Ok(_) => panic!("Se esperaba {expected}"),
        Err(e) => assert_eq!(e.code, expected, "{e}"),
    }
}
fn node(id: u32, coordinates: [f64; 3], dofs: &[Dof]) -> Node {
    let mut n = Node::new(NodeId(id), Dim::D3, coordinates);
    n.dofs = dofs.to_vec();
    n
}
fn block(location: Vec<DofIndex>, stiffness: DMat) -> ElementContribution {
    ElementContribution {
        location,
        stiffness,
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    }
}
fn spring(map: &DofMap, a: u32, b: u32, dof: Dof, k: f64) -> ElementContribution {
    block(
        vec![
            map.global_index(NodeId(a), dof).unwrap(),
            map.global_index(NodeId(b), dof).unwrap(),
        ],
        DMat::new(2, 2, vec![k, -k, -k, k]).unwrap(),
    )
}
fn rotate(v: [f64; 2], angle: f64) -> [f64; 2] {
    let (s, c) = angle.sin_cos();
    [c * v[0] - s * v[1], s * v[0] + c * v[1]]
}
fn three_columns(angle: f64) -> (Model, Vec<ElementContribution>, RigidDiaphragm) {
    let mut m = Model::new();
    for (i, p) in POINTS.into_iter().enumerate() {
        let [x, y] = rotate(p, angle);
        let id = i as u32 + 1;
        m.nodes.push(node(id, [x, y, 0.0], &XY[..2]));
        m.nodes.push(node(id + 10, [x, y, 3.0], &XY));
        m.supports.push(Support {
            id: format!("base{id}"),
            node: NodeId(id),
            restrained: XY[..2].to_vec(),
        });
    }
    m.nodes.push(node(20, [0.0, 0.0, 3.0], &XY));
    let [fx, fy] = rotate([30.0, 60.0], angle);
    m.cases.push(LoadCase {
        name: case().0,
        loads: vec![NodalLoad {
            node: NodeId(12),
            values: vec![
                DofValue {
                    dof: Dof::Ux,
                    value: fx,
                },
                DofValue {
                    dof: Dof::Uy,
                    value: fy,
                },
            ],
        }],
    });
    let map = DofMap::new(&m.nodes).unwrap();
    let mut elements = Vec::new();
    for i in 1..=3 {
        for dof in &XY[..2] {
            elements.push(spring(&map, i, i + 10, *dof, 100.0));
        }
    }
    (
        m,
        elements,
        RigidDiaphragm {
            master: NodeId(20),
            slaves: vec![NodeId(11), NodeId(12), NodeId(13)],
        },
    )
}
fn system(
    m: &Model,
    e: &[ElementContribution],
    p: &[PrescribedDof],
    d: &[RigidDiaphragm],
) -> LinearSystem<ReferenceBackend> {
    LinearSystem::from_model_with_diaphragms(ReferenceBackend, m, e, p, d).unwrap()
}
fn value(s: &LinearSystem<ReferenceBackend>, u: &[f64], id: u32, dof: Dof) -> f64 {
    u[s.dof_map().global_index(NodeId(id), dof).unwrap().0 as usize]
}

#[test]
fn eccentric_three_columns_closed_solution_and_force_balance() {
    // Σr=0, ΣR²=14 m², k=100 N/m: U=(Fx,Fy)/(3k), θ=Mz/(k ΣR²).
    // F=(30,60) N en (2,-1) m: Mz=150 N·m, θ=3/28 rad.
    let (m, e, d) = three_columns(0.0);
    let mut s = system(&m, &e, &[], &[d]);
    let q = s.solve(&case(), 1.0).unwrap();
    let u = s.constraints().expand(&q).unwrap();
    assert_eq!(s.n_free_dofs(), 3);
    for (dof, expected) in XY.into_iter().zip([0.1, 0.2, 3.0 / 28.0]) {
        close(value(&s, &u, 20, dof), expected, 1e-12);
    }
    for (i, [rx, ry]) in POINTS.into_iter().enumerate() {
        let id = i as u32 + 11;
        close(value(&s, &u, id, Dof::Ux), 0.1 - ry * 3.0 / 28.0, 1e-12);
        close(value(&s, &u, id, Dof::Uy), 0.2 + rx * 3.0 / 28.0, 1e-12);
        close(value(&s, &u, id, Dof::Rz), 3.0 / 28.0, 1e-12);
        assert_eq!(s.reduced_index(NodeId(id), Dof::Ux).unwrap(), None);
    }
    let mut balance = [30.0, 60.0, 150.0];
    for r in s.reactions(&q, &case(), 1.0).unwrap() {
        let [x, y] = POINTS[(r.node.0 - 1) as usize];
        match r.dof {
            Dof::Ux => {
                balance[0] += r.value;
                balance[2] -= y * r.value;
            }
            Dof::Uy => {
                balance[1] += r.value;
                balance[2] += x * r.value;
            }
            _ => unreachable!(),
        }
    }
    for v in balance {
        close(v, 0.0, 1e-10);
    }
    let forces = s.diaphragm_forces(&q, &case(), 1.0).unwrap();
    let f = &forces[0];
    let mut resultant = f.master.values;
    for (slave, [rx, ry]) in f.slaves.iter().zip(POINTS) {
        let [fx, fy, mz] = slave.values;
        resultant[0] += fx;
        resultant[1] += fy;
        resultant[2] += mz + rx * fy - ry * fx;
        // Acción necesaria en el esclavo = fuerza del resorte menos carga externa.
        let external = if slave.node == NodeId(12) {
            [30.0, 60.0]
        } else {
            [0.0; 2]
        };
        close(
            fx,
            100.0 * value(&s, &u, slave.node.0, Dof::Ux) - external[0],
            1e-12,
        );
        close(
            fy,
            100.0 * value(&s, &u, slave.node.0, Dof::Uy) - external[1],
            1e-12,
        );
    }
    for v in resultant {
        close(v, 0.0, 1e-10);
    }
}

#[test]
fn rotation_in_xy_preserves_solution_and_reactions() {
    let (m, e, d) = three_columns(0.0);
    let mut s = system(&m, &e, &[], &[d]);
    let q = s.solve(&case(), 1.0).unwrap();
    let original = s.constraints().expand(&q).unwrap();
    let reactions = s.reactions(&q, &case(), 1.0).unwrap();
    for angle in [0.37, std::f64::consts::FRAC_PI_2, -1.21] {
        let (m, e, d) = three_columns(angle);
        let mut rotated = system(&m, &e, &[], &[d]);
        let q = rotated.solve(&case(), 1.0).unwrap();
        let u = rotated.constraints().expand(&q).unwrap();
        for id in [11, 12, 13, 20] {
            let expected = rotate(
                [
                    value(&s, &original, id, Dof::Ux),
                    value(&s, &original, id, Dof::Uy),
                ],
                angle,
            );
            close(value(&rotated, &u, id, Dof::Ux), expected[0], 1e-11);
            close(value(&rotated, &u, id, Dof::Uy), expected[1], 1e-11);
            close(
                value(&rotated, &u, id, Dof::Rz),
                value(&s, &original, id, Dof::Rz),
                1e-11,
            );
        }
        let rr = rotated.reactions(&q, &case(), 1.0).unwrap();
        for (a, b) in reactions.chunks_exact(2).zip(rr.chunks_exact(2)) {
            let expected = rotate([a[0].value, a[1].value], angle);
            close(b[0].value, expected[0], 1e-10);
            close(b[1].value, expected[1], 1e-10);
        }
    }
}

#[test]
fn prescribed_master_transfers_forces_and_does_not_scale_settlement() {
    let (mut m, e, d) = three_columns(0.0);
    m.supports.push(Support {
        id: "maestro".into(),
        node: NodeId(20),
        restrained: vec![Dof::Ux],
    });
    let prescribed = [PrescribedDof {
        node: NodeId(20),
        dof: Dof::Ux,
        value: 0.025,
    }];
    let mut s = system(&m, &e, &prescribed, &[d]);
    for factor in [0.0, 1.0, 2.0, -1.0] {
        let q = s.solve(&case(), factor).unwrap();
        let u = s.constraints().expand(&q).unwrap();
        close(value(&s, &u, 20, Dof::Ux), 0.025, 1e-12);
        close(value(&s, &u, 20, Dof::Uy), 0.2 * factor, 1e-12);
        let r = s.reactions(&q, &case(), factor).unwrap();
        let master = r.iter().find(|r| r.node == NodeId(20)).unwrap();
        // 3 k u_m - factor Fx, aunque no existe K ni F nodal en el maestro.
        close(master.value, 7.5 - 30.0 * factor, 1e-11);
        close(
            r.iter()
                .filter(|r| r.dof == Dof::Ux)
                .map(|r| r.value)
                .sum::<f64>()
                + 30.0 * factor,
            0.0,
            1e-10,
        );
        let f = s.diaphragm_forces(&q, &case(), factor).unwrap();
        close(f[0].master.values[0], -master.value, 1e-11);
    }
}

#[test]
fn transformation_preserves_out_of_plane_and_rejects_invalid_graphs() {
    let mut m = Model::new();
    m.nodes.push(Node::new(NodeId(1), Dim::D3, [0.0; 3]));
    m.nodes.push(Node::new(NodeId(2), Dim::D3, [2.0, 3.0, 0.0]));
    m.nodes.push(Node::new(NodeId(3), Dim::D3, [1.0, 1.0, 0.0]));
    let d = RigidDiaphragm {
        master: NodeId(1),
        slaves: vec![NodeId(2)],
    };
    let map = DofMap::new(&m.nodes).unwrap();
    let c = Constraints::with_diaphragms(&map, &[], &[], std::slice::from_ref(&d)).unwrap();
    assert_eq!(c.free_dofs().len(), 15);
    let mut q = vec![0.0; 15];
    for (dof, v) in XY.into_iter().zip([0.1, 0.2, 0.01]) {
        q[c.reduced_index(map.global_index(NodeId(1), dof).unwrap())
            .unwrap()
            .unwrap()
            .0 as usize] = v;
    }
    for (dof, v) in [Dof::Uz, Dof::Rx, Dof::Ry].into_iter().zip([0.7, 0.8, 0.9]) {
        let i = map.global_index(NodeId(2), dof).unwrap();
        q[c.reduced_index(i).unwrap().unwrap().0 as usize] = v;
        assert_eq!(c.free_transformation_row(i).unwrap().len(), 1);
    }
    let u = c.expand(&q).unwrap();
    for (dof, v) in [Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz]
        .into_iter()
        .zip([0.07, 0.22, 0.7, 0.8, 0.9, 0.01])
    {
        close(
            u[map.global_index(NodeId(2), dof).unwrap().0 as usize],
            v,
            1e-12,
        );
    }
    let t = c.transformation().unwrap();
    assert_eq!(t.rows(), 18);
    assert_eq!(t.cols(), 15);
    let result = ReferenceBackend
        .matmul(&t, &DMat::new(15, 1, q.clone()).unwrap())
        .unwrap();
    assert_eq!(result.values(), u);
    code(c.free_transformation_row(DofIndex(99)), "ENG-E-0010");
    code(c.expand(&[0.0]), "ENG-E-0010");
    q[0] = f64::NAN;
    code(c.expand(&q), "ENG-E-0008");
    code(c.project_force(&[0.0]), "ENG-E-0010");
    for bad in [
        RigidDiaphragm {
            master: NodeId(1),
            slaves: vec![NodeId(1)],
        },
        RigidDiaphragm {
            master: NodeId(1),
            slaves: vec![],
        },
        RigidDiaphragm {
            master: NodeId(1),
            slaves: vec![NodeId(2), NodeId(2)],
        },
    ] {
        code(
            Constraints::with_diaphragms(&map, &[], &[], &[bad]),
            "ENG-E-0010",
        );
    }
    for (next, expected) in [
        (
            RigidDiaphragm {
                master: NodeId(2),
                slaves: vec![NodeId(1)],
            },
            "ENG-E-0013",
        ),
        (
            RigidDiaphragm {
                master: NodeId(2),
                slaves: vec![NodeId(3)],
            },
            "ENG-E-0013",
        ),
        (
            RigidDiaphragm {
                master: NodeId(3),
                slaves: vec![NodeId(2)],
            },
            "ENG-E-0010",
        ),
        (
            RigidDiaphragm {
                master: NodeId(1),
                slaves: vec![NodeId(3)],
            },
            "ENG-E-0010",
        ),
    ] {
        code(
            Constraints::with_diaphragms(&map, &[], &[], &[d.clone(), next]),
            expected,
        );
    }
    for dof in [Dof::Ux, Dof::Uy, Dof::Rz, Dof::Uz, Dof::Rx, Dof::Ry] {
        let support = Support {
            id: "esclavo".into(),
            node: NodeId(2),
            restrained: vec![dof],
        };
        code(
            Constraints::with_diaphragms(&map, &[support], &[], std::slice::from_ref(&d)),
            "ENG-E-0013",
        );
        let prescribed = PrescribedDof {
            node: NodeId(2),
            dof,
            value: 0.0,
        };
        code(
            Constraints::with_diaphragms(&map, &[], &[prescribed], std::slice::from_ref(&d)),
            "ENG-E-0013",
        );
    }
    code(
        Constraints::with_diaphragms(
            &map,
            &[],
            &[],
            &[RigidDiaphragm {
                master: NodeId(99),
                ..d.clone()
            }],
        ),
        "ENG-E-0003",
    );
    m.nodes[1].coordinates[2] = 0.01;
    code(
        Constraints::with_diaphragms(
            &DofMap::new(&m.nodes).unwrap(),
            &[],
            &[],
            std::slice::from_ref(&d),
        ),
        "ENG-E-0010",
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        m.nodes[1].coordinates[2] = value;
        code(DofMap::new(&m.nodes), "ENG-E-0008");
    }
    m.nodes[1].coordinates = [f64::MAX, 0.0, 0.0];
    m.nodes[0].coordinates = [-f64::MAX, 0.0, 0.0];
    code(
        Constraints::with_diaphragms(
            &DofMap::new(&m.nodes).unwrap(),
            &[],
            &[],
            std::slice::from_ref(&d),
        ),
        "ENG-E-0008",
    );
    for n in &mut m.nodes {
        n.dim = Dim::D2;
        n.dofs = XY.to_vec();
        n.coordinates = [0.0; 3];
    }
    code(
        Constraints::with_diaphragms(&DofMap::new(&m.nodes).unwrap(), &[], &[], &[d]),
        "ENG-E-0013",
    );
}

fn two_floors() -> (Model, Vec<ElementContribution>, Vec<RigidDiaphragm>) {
    // Modelo de cortante de dos pisos: cada columna aporta un resorte XY
    // entre niveles, sin atribuir flexión 3D a frame2d.
    let mut m = Model::new();
    for floor in 0..=2 {
        for (i, [x, y]) in POINTS.into_iter().enumerate() {
            let id = floor * 10 + i as u32 + 1;
            m.nodes.push(node(
                id,
                [x, y, 3.0 * f64::from(floor)],
                if floor == 0 { &XY[..2] } else { &XY },
            ));
            if floor == 0 {
                m.supports.push(Support {
                    id: format!("base{id}"),
                    node: NodeId(id),
                    restrained: XY[..2].to_vec(),
                });
            }
        }
    }
    m.cases.push(LoadCase {
        name: case().0,
        loads: vec![
            NodalLoad {
                node: NodeId(12),
                values: vec![DofValue {
                    dof: Dof::Ux,
                    value: 15.0,
                }],
            },
            NodalLoad {
                node: NodeId(23),
                values: vec![
                    DofValue {
                        dof: Dof::Ux,
                        value: 30.0,
                    },
                    DofValue {
                        dof: Dof::Uy,
                        value: 10.0,
                    },
                ],
            },
        ],
    });
    let map = DofMap::new(&m.nodes).unwrap();
    let mut e = Vec::new();
    for (floor, k) in [(1, 100.0), (2, 200.0)] {
        for i in 1..=3 {
            for &dof in &XY[..2] {
                e.push(spring(&map, (floor - 1) * 10 + i, floor * 10 + i, dof, k));
            }
        }
    }
    let d = [1, 2]
        .into_iter()
        .map(|floor| RigidDiaphragm {
            master: NodeId(floor * 10 + 1),
            slaves: vec![NodeId(floor * 10 + 2), NodeId(floor * 10 + 3)],
        })
        .collect();
    (m, e, d)
}
fn slab_beams(m: &Model, stiffness: f64) -> Vec<ElementContribution> {
    let map = DofMap::new(&m.nodes).unwrap();
    let beam = EulerFrame2D::new(stiffness, 1.0, 1.0, 0.0, MomentReleases::default()).unwrap();
    let params = ElementParams::default();
    let mut result = Vec::new();
    for floor in [1, 2] {
        for (i, j) in [(0, 1), (1, 2), (2, 0)] {
            // Una viga de planta D2 se evalúa en XY local al piso (Z=0), y
            // sus mismos Ux,Uy,Rz globales se ubican en los nodos D3 a cota h.
            let coords = [
                [POINTS[i][0], POINTS[i][1], 0.0],
                [POINTS[j][0], POINTS[j][1], 0.0],
            ];
            let ctx = ElementCtx {
                dim: Dim::D2,
                coords: &coords,
                params: &params,
            };
            let k = beam.respond(&ctx, &[0.0; 6], &[]).unwrap().tangent;
            let location = [i, j]
                .into_iter()
                .flat_map(|i| {
                    XY.into_iter()
                        .map(move |dof| (floor * 10 + i as u32 + 1, dof))
                })
                .map(|(id, dof)| map.global_index(NodeId(id), dof).unwrap())
                .collect();
            result.push(block(location, k));
        }
    }
    result
}

#[test]
fn two_floors_stiff_slabs_converge_to_rigid_diaphragms() {
    let (m, columns, d) = two_floors();
    let mut rigid = system(&m, &columns, &[], &d);
    let q = rigid.solve(&case(), 1.0).unwrap();
    let ur = rigid.constraints().expand(&q).unwrap();
    let rr = rigid.reactions(&q, &case(), 1.0).unwrap();
    // Solución cerrada en centroide: ΔU_j=ΣF_sobre_j/(3k_j),
    // Δθ_j=ΣM_sobre_j/(14k_j). M1=15, M2=-60 N·m.
    for (floor, center, theta) in [
        (1, [0.15, 1.0 / 30.0], -45.0 / 1400.0),
        (2, [0.20, 0.05], -45.0 / 1400.0 - 60.0 / 2800.0),
    ] {
        for (i, [rx, ry]) in POINTS.into_iter().enumerate() {
            let id = floor * 10 + i as u32 + 1;
            close(
                value(&rigid, &ur, id, Dof::Ux),
                center[0] - ry * theta,
                1e-12,
            );
            close(
                value(&rigid, &ur, id, Dof::Uy),
                center[1] + rx * theta,
                1e-12,
            );
            close(value(&rigid, &ur, id, Dof::Rz), theta, 1e-12);
        }
    }
    let mut errors = Vec::new();
    for stiffness in [1e5, 1e8] {
        let mut e = columns.clone();
        e.extend(slab_beams(&m, stiffness));
        let mut elastic = LinearSystem::from_model(ReferenceBackend, &m, &e, &[]).unwrap();
        let q = elastic.solve(&case(), 1.0).unwrap();
        let ue = elastic.constraints().expand(&q).unwrap();
        let re = elastic.reactions(&q, &case(), 1.0).unwrap();
        let error = ue
            .iter()
            .zip(&ur)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f64, f64::max);
        errors.push(error);
        if stiffness == 1e8 {
            // Tolerancias relativas con piso de escala 1: 2e-6 m/rad y
            // 2e-5 N. E_losa/k_columna=1e6 (A=I=1 en unidades SI).
            for (a, b) in ue.iter().zip(&ur) {
                close(*a, *b, 2e-6);
            }
            for (a, b) in re.iter().zip(&rr) {
                assert_eq!((a.node, a.dof), (b.node, b.dof));
                close(a.value, b.value, 2e-5);
            }
        }
        for (dof, load) in [(Dof::Ux, 45.0), (Dof::Uy, 10.0)] {
            close(
                re.iter()
                    .filter(|r| r.dof == dof)
                    .map(|r| r.value)
                    .sum::<f64>()
                    + load,
                0.0,
                2e-6,
            );
        }
    }
    assert!(errors[1] < errors[0] / 100.0, "convergencia: {errors:?}");
}

#[test]
fn all_matrices_and_system_view_use_the_transformation() {
    let (m, mut e, d) = three_columns(0.0);
    // Bloque algebraico acoplado que comprueba términos cruzados y que
    // T no se reduce a seleccionar índices. No es un modelo constitutivo.
    let map = DofMap::new(&m.nodes).unwrap();
    let location = vec![
        map.global_index(NodeId(11), Dof::Ux).unwrap(),
        map.global_index(NodeId(13), Dof::Uy).unwrap(),
    ];
    let mut extra = block(
        location.clone(),
        DMat::new(2, 2, vec![5.0, 2.0, 2.0, 7.0]).unwrap(),
    );
    extra.mass_lumped = Some(DMat::new(2, 2, vec![3.0, 0.0, 0.0, 4.0]).unwrap());
    extra.mass_consistent = Some(DMat::new(2, 2, vec![3.0, 1.0, 1.0, 4.0]).unwrap());
    extra.geometric = Some(DMat::new(2, 2, vec![2.0, -1.0, -1.0, 6.0]).unwrap());
    e.push(extra);
    let mut s = system(&m, &e, &[], &[d]);
    let t = s.constraints().transformation().unwrap();
    // T completa incluye las bases prescritas: seleccionar columnas libres
    // independientemente de la rutina de reducción de la librería.
    let cols: Vec<_> = s
        .constraints()
        .free_dofs()
        .iter()
        .map(|i| s.constraints().independent_dofs().binary_search(i).unwrap())
        .collect();
    let tf = DMat::new(
        t.rows(),
        cols.len(),
        (0..t.rows())
            .flat_map(|r| cols.iter().map(move |&c| (r, c)))
            .map(|(r, c)| t.get(r, c).unwrap())
            .collect(),
    )
    .unwrap();
    let transpose = ReferenceBackend.transpose(&tf).unwrap();
    let mut out = SymMatrix::zeros(3).unwrap();
    let global = [
        s.stiffness().to_dense().unwrap(),
        s.global_mass(MassKind::Lumped).to_dense().unwrap(),
        s.global_mass(MassKind::Consistent).to_dense().unwrap(),
        s.global_geometric().to_dense().unwrap(),
    ];
    for (i, matrix) in global.into_iter().enumerate() {
        let kt = ReferenceBackend.matmul(&matrix, &tf).unwrap();
        let expected = ReferenceBackend.matmul(&transpose, &kt).unwrap();
        match i {
            0 => s.tangent(&[0.0; 3], &mut out).unwrap(),
            1 => s.mass(MassKind::Lumped, &mut out).unwrap(),
            2 => s.mass(MassKind::Consistent, &mut out).unwrap(),
            _ => s.geometric(&[0.0; 3], &mut out).unwrap(),
        }
        for (a, b) in out.dense().values().iter().zip(expected.values()) {
            close(*a, *b, 1e-12);
        }
    }
    let f = s.global_force(&case(), 1.0).unwrap();
    let expected = ReferenceBackend
        .matmul(&transpose, &DMat::new(f.len(), 1, f).unwrap())
        .unwrap();
    let mut external = vec![0.0; 3];
    s.external_force(&case(), 1.0, &mut external).unwrap();
    assert_eq!(external, expected.values());
    let q = [0.04, -0.03, 0.02];
    let mut internal = [0.0; 3];
    s.internal_force(&q, &mut internal).unwrap();
    s.tangent(&q, &mut out).unwrap();
    let expected = ReferenceBackend
        .matmul(out.dense(), &DMat::new(3, 1, q.to_vec()).unwrap())
        .unwrap();
    for (a, b) in internal.into_iter().zip(expected.values()) {
        close(a, *b, 1e-12);
    }
    s.set_geometric_producers(vec![GeometricProducer {
        location,
        eval: Box::new(|u| DMat::new(2, 2, vec![1.0 + u[0], 0.0, 0.0, 1.0 + u[1]])),
    }])
    .unwrap();
    s.geometric(&q, &mut out).unwrap();
    // Esclavo11 Ux=q_x+q_θ=.06; esclavo13 Uy=q_y (r_x=0).
    close(out.dense().get(0, 0).unwrap(), 1.06, 1e-12);
    close(out.dense().get(1, 1).unwrap(), 0.97, 1e-12);
    close(out.dense().get(0, 2).unwrap(), 1.06, 1e-12);
    close(out.dense().get(2, 2).unwrap(), 1.06, 1e-12);
    assert!(s.pattern().contains(0, 2) && s.pattern().contains(2, 0));
}

#[test]
fn nonfinite_transformed_results_are_diagnostics_and_preserve_outputs() {
    let (mut m, e, d) = three_columns(0.0);
    m.nodes
        .iter_mut()
        .find(|n| n.id == NodeId(11))
        .unwrap()
        .coordinates[1] = -10.0;
    let mut s = system(&m, &e, &[], &[d]);
    let q = [0.0, 0.0, f64::MAX];
    code(s.constraints().expand(&q), "ENG-E-0008");
    let mut out = [12.0; 3];
    code(s.internal_force(&q, &mut out), "ENG-E-0008");
    assert_eq!(out, [12.0; 3]);
    assert_eq!(s.trial_displacement(), &[0.0; 3]);
    let mut f = vec![0.0; s.dof_map().len()];
    f[s.dof_map().global_index(NodeId(11), Dof::Ux).unwrap().0 as usize] = f64::MAX;
    code(s.constraints().project_force(&f), "ENG-E-0008");
    code(s.diaphragm_forces(&q, &case(), 1.0), "ENG-E-0008");
    let mut huge = e.clone();
    huge[0].stiffness = DMat::new(2, 2, vec![f64::MAX, -f64::MAX, -f64::MAX, f64::MAX]).unwrap();
    code(
        LinearSystem::from_model_with_diaphragms(
            ReferenceBackend,
            &m,
            &huge,
            &[],
            s.constraints().diaphragms(),
        ),
        "ENG-E-0008",
    );
}
