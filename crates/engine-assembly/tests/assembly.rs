//! Aceptación analítica y conformidad; solo datos en memoria, también en wasm.
mod fixtures;

use engine_assembly::{
    assemble::{assemble_matrix, assemble_vector},
    Constraints, DofMap, ElementContribution, ElementLoad, LinearSystem, PrescribedDof,
};
use engine_contracts::{conformance::check_system_view, CaseRef, MassKind, SystemView};
use engine_core::{Dim, Dof, DofIndex, EResult, EngineError, EntityRef, NodeId};
use engine_linalg::{
    DMat, GeneralizedEigen, LinAlgBackend, ReferenceBackend, SymFactorization, SymMatrix,
};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, NodalMass, Node, Support};

fn case() -> CaseRef {
    CaseRef("P".into())
}
fn code<T>(result: EResult<T>, expected: &str) {
    match result {
        Ok(_) => panic!("Se esperaba {expected}"),
        Err(e) => assert_eq!(e.code, expected, "{e}"),
    }
}
fn close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= 1e-10 * e.abs().max(1.0), "{a} != {e}");
    }
}
fn model(count: usize) -> Model {
    let mut m = Model::new();
    // Subconjunto axial Ux; el ensamblador no inventa elementos de resorte/barra.
    for i in 0..count {
        let mut n = Node::new(NodeId(i as u32 + 1), Dim::D2, [i as f64, 0.0, 0.0]);
        n.dofs = vec![Dof::Ux];
        m.nodes.push(n);
    }
    m.supports.push(Support {
        id: "izquierdo".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux],
    });
    m.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![NodalLoad {
            node: NodeId(count as u32),
            values: vec![DofValue {
                dof: Dof::Ux,
                value: 12.0,
            }],
        }],
    });
    m
}
fn contribution(i: usize, k: f64) -> ElementContribution {
    ElementContribution {
        location: vec![DofIndex(i as u32), DofIndex(i as u32 + 1)],
        stiffness: DMat::new(2, 2, vec![k, -k, -k, k]).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    }
}
fn elements(ks: &[f64]) -> Vec<ElementContribution> {
    ks.iter()
        .enumerate()
        .map(|(i, k)| contribution(i, *k))
        .collect()
}
fn system(
    m: &Model,
    e: &[ElementContribution],
    p: &[PrescribedDof],
) -> LinearSystem<ReferenceBackend> {
    LinearSystem::from_model(ReferenceBackend, m, e, p).unwrap()
}

#[test]
fn hand_derived_matrices_displacements_and_global_equilibrium() {
    for fixture in [
        &fixtures::TWO_SPRINGS,
        &fixtures::THREE_SPRINGS,
        &fixtures::AXIAL_BAR,
    ] {
        assert!(!fixture.source.is_empty() && !fixture.citation.is_empty());
        let mut s = system(
            &model(fixture.stiffnesses.len() + 1),
            &elements(fixture.stiffnesses),
            &[],
        );
        assert_eq!(
            s.stiffness().to_dense().unwrap().values(),
            fixture.expected_k
        );
        let u = s.solve(&case(), 1.0).unwrap();
        close(&u, fixture.expected_u);
        let reactions = s.reactions(&u, &case(), 1.0).unwrap();
        close(&[reactions[0].value], &[-12.0]);
        let balance = reactions.iter().map(|r| r.value).sum::<f64>()
            + s.global_force(&case(), 1.0).unwrap().iter().sum::<f64>();
        assert!(balance.abs() < 1e-9);
        let mut internal = vec![f64::NAN; u.len()];
        s.internal_force(&u, &mut internal).unwrap();
        let mut external = vec![f64::NAN; u.len()];
        s.external_force(&case(), 1.0, &mut external).unwrap();
        close(&internal, &external);
    }
}

#[test]
fn settlement_and_load_on_support_are_not_scaled_twice() {
    assert!(!fixtures::SETTLEMENT_SOURCE.is_empty() && !fixtures::SETTLEMENT_CITATION.is_empty());
    let mut m = model(3);
    m.cases[0].loads.clear();
    m.supports.push(Support {
        id: "derecho".into(),
        node: NodeId(3),
        restrained: vec![Dof::Ux],
    });
    let p = [PrescribedDof {
        node: NodeId(3),
        dof: Dof::Ux,
        value: 0.03,
    }];
    let mut s = system(&m, &elements(&[100.0, 200.0]), &p);
    close(&s.reduced_rhs(&case(), 0.0).unwrap(), &[6.0]);
    let u = s.solve(&case(), 0.0).unwrap();
    close(&u, &[0.02]);
    close(&s.constraints().expand(&u).unwrap(), &[0.0, 0.02, 0.03]);
    close(
        &s.reactions(&u, &case(), 0.0)
            .unwrap()
            .iter()
            .map(|r| r.value)
            .collect::<Vec<_>>(),
        &[-2.0, 2.0],
    );
    let mut f = [f64::NAN];
    s.internal_force(&[0.0], &mut f).unwrap();
    close(&f, &[-6.0]);
    s.external_force(&case(), 2.0, &mut f).unwrap();
    close(&f, &[0.0]);
    // Con P_2=12 N y P_3=4 N, u_2=(12+6)/300=0.06 m.
    // R_1=−100*0.06=−6; R_3=200*(0.03−0.06)−4=−10 N.
    m.cases[0].loads = vec![
        NodalLoad {
            node: NodeId(2),
            values: vec![DofValue {
                dof: Dof::Ux,
                value: 12.0,
            }],
        },
        NodalLoad {
            node: NodeId(3),
            values: vec![DofValue {
                dof: Dof::Ux,
                value: 4.0,
            }],
        },
    ];
    let mut s = system(&m, &elements(&[100.0, 200.0]), &p);
    close(&s.reduced_rhs(&case(), 1.0).unwrap(), &[18.0]);
    close(&s.reduced_rhs(&case(), 2.0).unwrap(), &[30.0]);
    close(&s.solve(&case(), 1.0).unwrap(), &[0.06]);
    close(
        &s.reactions(&[0.06], &case(), 1.0)
            .unwrap()
            .iter()
            .map(|r| r.value)
            .collect::<Vec<_>>(),
        &[-6.0, -10.0],
    );
    for factor in [0.0, 1.0, 2.0, -1.0] {
        let u = s.solve(&case(), factor).unwrap();
        let balance = s
            .reactions(&u, &case(), factor)
            .unwrap()
            .iter()
            .map(|r| r.value)
            .sum::<f64>()
            + s.global_force(&case(), factor).unwrap().iter().sum::<f64>();
        assert!(balance.abs() <= 1e-9);
    }
    assert!(check_system_view(&mut s, &[vec![0.0], vec![0.06]], &case(), 1e-5).is_empty());
}

#[test]
fn ordering_2d_3d_and_reduced_labels() {
    let nodes = vec![
        Node::new(NodeId(20), Dim::D3, [0.0; 3]),
        Node::new(NodeId(10), Dim::D2, [0.0; 3]),
    ];
    let map = DofMap::new(&nodes).unwrap();
    assert_eq!(map.len(), 9);
    for (i, d) in Dim::D2.dofs().iter().enumerate() {
        assert_eq!(map.label(DofIndex(i as u32)).unwrap(), (NodeId(10), *d));
    }
    for (i, d) in Dim::D3.dofs().iter().enumerate() {
        assert_eq!(
            map.global_index(NodeId(20), *d).unwrap(),
            DofIndex(i as u32 + 3)
        );
    }
    let mut reversed = nodes.clone();
    reversed.reverse();
    reversed[0].dofs.reverse();
    let other = DofMap::new(&reversed).unwrap();
    for i in 0..9 {
        assert_eq!(
            map.label(DofIndex(i)).unwrap(),
            other.label(DofIndex(i)).unwrap()
        );
    }
    let supports = [Support {
        id: "s".into(),
        node: NodeId(10),
        restrained: vec![Dof::Ux, Dof::Rz],
    }];
    let c = Constraints::new(&map, &supports, &[]).unwrap();
    assert_eq!(c.reduced_index(DofIndex(0)).unwrap(), None);
    assert_eq!(c.reduced_index(DofIndex(1)).unwrap(), Some(DofIndex(0)));
    let m = Model {
        nodes,
        supports: supports.to_vec(),
        cases: vec![LoadCase {
            name: "P".into(),
            loads: vec![],
        }],
        ..Model::default()
    };
    let s = system(&m, &[], &[]);
    assert_eq!(s.dof_label(DofIndex(0)), (NodeId(10), Dof::Uy));
    assert_eq!(s.dof_label(DofIndex(6)), (NodeId(20), Dof::Rz));
    code(map.label(DofIndex(9)), "ENG-E-0010");
    code(c.reduced_index(DofIndex(9)), "ENG-E-0010");
}

#[test]
fn mass_geometric_forces_pattern_and_overwrite() {
    let mut m = model(3);
    m.cases.push(LoadCase {
        name: "Q".into(),
        loads: vec![],
    });
    m.masses.push(NodalMass {
        id: "masa".into(),
        node: NodeId(2),
        values: vec![DofValue {
            dof: Dof::Ux,
            value: 5.0,
        }],
    });
    let mut e = elements(&[100.0, 200.0]);
    // Datos arbitrarios de interfaz: M_l=diag(2,2), M_c=[[2,1],[1,2]],
    // Kg=[[3,−3],[−3,3]], sumados en conectividad (1,2) y (2,3).
    for el in &mut e {
        el.mass_lumped = Some(DMat::new(2, 2, vec![2.0, 0.0, 0.0, 2.0]).unwrap());
        el.mass_consistent = Some(DMat::new(2, 2, vec![2.0, 1.0, 1.0, 2.0]).unwrap());
        el.geometric = Some(DMat::new(2, 2, vec![3.0, -3.0, -3.0, 3.0]).unwrap());
        el.loads.push(ElementLoad {
            case: CaseRef("Q".into()),
            values: vec![1.0, 2.0],
        });
    }
    let mut s = system(&m, &e, &[]);
    assert_eq!(
        s.global_mass(MassKind::Consistent)
            .to_dense()
            .unwrap()
            .values(),
        &[2.0, 1.0, 0.0, 1.0, 9.0, 1.0, 0.0, 1.0, 2.0]
    );
    let mut out = SymMatrix::zeros(2).unwrap();
    s.mass(MassKind::Lumped, &mut out).unwrap();
    assert_eq!(out.dense().values(), &[9.0, 0.0, 0.0, 2.0]);
    s.mass(MassKind::Consistent, &mut out).unwrap();
    assert_eq!(out.dense().values(), &[9.0, 1.0, 1.0, 2.0]);
    s.geometric(&[0.0, 0.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[6.0, -3.0, -3.0, 3.0]);
    assert_eq!(s.pattern().coordinates(), &[(0, 0), (0, 1), (1, 0), (1, 1)]);
    let mut force = [f64::NAN; 2];
    s.external_force(&CaseRef("Q".into()), 2.0, &mut force)
        .unwrap();
    close(&force, &[6.0, 4.0]);
    assert!(check_system_view(&mut s, &[vec![0.0, 0.0]], &case(), 1e-5).is_empty());
    // Con Kg precalculado el kit solo puede muestrear el estado de referencia;
    // otro u se rechaza (ver j4_followup.rs). Muestras fuera de él: sistema sin Kg.
    let mut no_kg = system(&model(3), &elements(&[100.0, 200.0]), &[]);
    assert!(
        check_system_view(&mut no_kg, &[vec![0.0, 0.0], vec![0.1, 0.2]], &case(), 1e-5).is_empty()
    );
    // None sobrescribe una salida no nula con cero, sin fabricar masa ni Kg.
    let mut zero = system(&model(3), &elements(&[100.0, 200.0]), &[]);
    zero.mass(MassKind::Consistent, &mut out).unwrap();
    assert_eq!(out.dense().values(), &[0.0; 4]);
    zero.geometric(&[0.0; 2], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[0.0; 4]);
}

#[test]
fn accept_rollback_and_failed_operations_are_atomic() {
    let mut s = system(&model(3), &elements(&[100.0, 200.0]), &[]);
    s.accept(&[0.12, 0.18]).unwrap();
    let mut f = [0.0; 2];
    s.internal_force(&[0.3, 0.4], &mut f).unwrap();
    assert_eq!(s.accepted_displacement(), &[0.12, 0.18]);
    assert_eq!(s.trial_displacement(), &[0.3, 0.4]);
    code(s.accept(&[f64::NAN, 0.0]), "ENG-E-0008");
    code(s.accept(&[1.0]), "ENG-E-0010");
    s.rollback();
    s.rollback();
    assert_eq!(s.trial_displacement(), s.accepted_displacement());
    let old = f;
    code(
        s.internal_force(&[f64::MAX, f64::MAX], &mut f),
        "ENG-E-0008",
    );
    assert_eq!(f, old);
    code(
        s.external_force(&CaseRef("ausente".into()), 1.0, &mut f),
        "ENG-E-0003",
    );
    assert_eq!(f, old);
    code(
        s.external_force(&case(), f64::INFINITY, &mut f),
        "ENG-E-0008",
    );
    assert_eq!(f, old);
    let mut wrong = SymMatrix::zeros(3).unwrap();
    let original = wrong.clone();
    code(s.tangent(&[0.0; 2], &mut wrong), "ENG-E-0010");
    assert_eq!(wrong, original);
    code(s.mass(MassKind::Lumped, &mut wrong), "ENG-E-0010");
    code(s.geometric(&[0.0; 2], &mut wrong), "ENG-E-0010");
    code(s.reactions(&[0.0], &case(), 1.0), "ENG-E-0010");
}

#[test]
fn negative_topology_and_element_data() {
    let mut m = model(2);
    let e = elements(&[100.0]);
    m.nodes[0].dofs.push(Dof::Ux);
    code(DofMap::new(&m.nodes), "ENG-E-0002");
    m = model(2);
    m.nodes.push(m.nodes[0].clone());
    code(DofMap::new(&m.nodes), "ENG-E-0002");
    m = model(2);
    m.nodes[0].dofs = vec![Dof::Uz];
    code(DofMap::new(&m.nodes), "ENG-E-0011");
    m = model(2);
    m.supports[0].node = NodeId(99);
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &e, &[]),
        "ENG-E-0003",
    );
    m = model(2);
    m.supports.push(Support {
        id: "todo".into(),
        node: NodeId(2),
        restrained: vec![Dof::Ux],
    });
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &e, &[]),
        "ENG-E-0010",
    );
    m = model(2);
    m.supports[0].restrained.push(Dof::Ux);
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &e, &[]),
        "ENG-E-0002",
    );
    m = model(2);
    m.cases.push(m.cases[0].clone());
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &e, &[]),
        "ENG-E-0002",
    );
    m = model(2);
    let p = PrescribedDof {
        node: NodeId(1),
        dof: Dof::Ux,
        value: 0.0,
    };
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &e, &[p, p]),
        "ENG-E-0002",
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        code(DMat::new(1, 1, vec![value]), "ENG-E-0008");
        code(
            LinearSystem::from_model(ReferenceBackend, &m, &e, &[PrescribedDof { value, ..p }]),
            "ENG-E-0008",
        );
        let mut bad = m.clone();
        bad.cases[0].loads[0].values[0].value = value;
        code(
            LinearSystem::from_model(ReferenceBackend, &bad, &e, &[]),
            "ENG-E-0008",
        );
        let mut bad = e.clone();
        bad[0].loads.push(ElementLoad {
            case: case(),
            values: vec![value, 0.0],
        });
        code(
            LinearSystem::from_model(ReferenceBackend, &m, &bad, &[]),
            "ENG-E-0008",
        );
    }
    let mut bad = e.clone();
    bad[0].location[1] = DofIndex(99);
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &bad, &[]),
        "ENG-E-0010",
    );
    bad = e.clone();
    bad[0].location[1] = DofIndex(0);
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &bad, &[]),
        "ENG-E-0002",
    );
    bad = e.clone();
    bad[0].mass_lumped = Some(DMat::zeros(3, 3).unwrap());
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &bad, &[]),
        "ENG-E-0010",
    );
    bad = e.clone();
    bad[0].stiffness = DMat::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]).unwrap();
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &bad, &[]),
        "ENG-E-0010",
    );
    bad = e.clone();
    bad[0].loads.push(ElementLoad {
        case: CaseRef("ausente".into()),
        values: vec![0.0; 2],
    });
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &bad, &[]),
        "ENG-E-0003",
    );
    bad[0].loads[0].case = case();
    bad[0].loads[0].values.pop();
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &bad, &[]),
        "ENG-E-0010",
    );
    code(
        assemble_matrix(
            &mut ReferenceBackend,
            2,
            &[(&[DofIndex(0)], &e[0].stiffness)],
        ),
        "ENG-E-0010",
    );
    let mut f = [f64::MAX, 0.0];
    let original = f;
    code(
        assemble_vector(&mut f, &[DofIndex(0)], &[f64::MAX]),
        "ENG-E-0008",
    );
    assert_eq!(f, original);
    let huge = DMat::new(1, 1, vec![f64::MAX]).unwrap();
    code(
        assemble_matrix(
            &mut ReferenceBackend,
            1,
            &[(&[DofIndex(0)], &huge), (&[DofIndex(0)], &huge)],
        ),
        "ENG-E-0008",
    );
}

#[test]
fn singular_mechanism_is_translated_to_global_node_and_dof() {
    let mut m = model(3);
    m.supports.clear();
    let mut s = system(&m, &elements(&[100.0, 200.0]), &[]);
    let err = s.solve(&case(), 1.0).unwrap_err();
    assert_eq!(err.code, "ENG-E-0006");
    assert!(err.message.contains("Nodo") && err.message.contains("Ux"));
    assert!(err.entities.iter().any(|e| matches!(e, EntityRef::Node(_))));
    let mut m = model(3);
    m.supports[0].restrained = vec![Dof::Ux];
    // Solo se conecta 1–2; 3 queda libre sin rigidez. Índice reducido 1 → global 2.
    let mut s = system(&m, &elements(&[100.0]), &[]);
    let err = s.solve(&case(), 1.0).unwrap_err();
    assert_eq!(err.code, "ENG-E-0006");
    assert!(err
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Node(NodeId(3)))));
    assert!(err
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Dof(DofIndex(2)))));
}

// Backend de prueba: incumple únicamente el contrato de índice del diagnóstico.
struct InvalidDiagnosticBackend(Vec<EntityRef>);
impl LinAlgBackend for InvalidDiagnosticBackend {
    fn name(&self) -> &'static str {
        "test.invalid_diagnostic"
    }
    fn matmul(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        ReferenceBackend.matmul(a, b)
    }
    fn transpose(&mut self, a: &DMat) -> EResult<DMat> {
        ReferenceBackend.transpose(a)
    }
    fn solve_dense(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        ReferenceBackend.solve_dense(a, b)
    }
    fn factorize(&mut self, _: &SymMatrix) -> EResult<Box<dyn SymFactorization>> {
        let mut e = EngineError::from_code("ENG-E-0006").unwrap();
        e.entities.clone_from(&self.0);
        Err(e)
    }
    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen> {
        ReferenceBackend.generalized_eigen(k, m)
    }
}
#[test]
fn inconsistent_backend_diagnostic_uses_catalog_error() {
    for entities in [
        vec![EntityRef::Dof(DofIndex(99))],
        vec![],
        vec![EntityRef::Node(NodeId(1))],
    ] {
        let mut s = LinearSystem::from_model(
            InvalidDiagnosticBackend(entities),
            &model(2),
            &elements(&[100.0]),
            &[],
        )
        .unwrap();
        code(s.solve(&case(), 1.0), "ENG-E-0012");
        code(s.solve_cases(&[(case(), 1.0)]), "ENG-E-0012");
    }
}

#[test]
fn repeatability_of_assembly_and_solve() {
    let m = model(4);
    let e = elements(&[100.0, 200.0, 300.0]);
    let mut base = system(&m, &e, &[]);
    let expected = base.solve(&case(), 1.0).unwrap();
    for _ in 0..10 {
        let mut s = system(&m, &e, &[]);
        assert_eq!(s.stiffness(), base.stiffness());
        assert_eq!(s.pattern(), base.pattern());
        let u = s.solve(&case(), 1.0).unwrap();
        assert_eq!(
            u.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            expected.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn arbitrary_location_order_and_middle_constraint_preserve_coupling() {
    // source: analítico, sustitución directa de índices en energía cuadrática.
    // citation: provenance/ENG-ASM-001.md, Derivaciones propias; bibliografía PENDIENTE.
    // A en [2,0] aporta K_22=30, K_00=10, K_02=K_20=4;
    // B en [1,2] aporta K_11=20, K_22+=6, K_12=K_21=5.
    let mut m = model(3);
    m.supports[0].node = NodeId(2);
    let mut a = contribution(0, 1.0);
    a.location = vec![DofIndex(2), DofIndex(0)];
    a.stiffness = DMat::new(2, 2, vec![30.0, 4.0, 4.0, 10.0]).unwrap();
    a.loads = vec![ElementLoad {
        case: case(),
        values: vec![3.0, 1.0],
    }];
    let mut b = contribution(1, 1.0);
    b.stiffness = DMat::new(2, 2, vec![20.0, 5.0, 5.0, 6.0]).unwrap();
    let mut s = system(&m, &[a, b], &[]);
    assert_eq!(
        s.stiffness().to_dense().unwrap().values(),
        &[10.0, 0.0, 4.0, 0.0, 20.0, 5.0, 4.0, 5.0, 36.0]
    );
    let mut reduced = SymMatrix::zeros(2).unwrap();
    s.tangent(&[0.0; 2], &mut reduced).unwrap();
    assert_eq!(reduced.dense().values(), &[10.0, 4.0, 4.0, 36.0]);
    close(&s.global_force(&case(), 1.0).unwrap(), &[1.0, 0.0, 15.0]);
    assert_eq!(
        s.reduced_index(NodeId(1), Dof::Ux).unwrap(),
        Some(DofIndex(0))
    );
    assert_eq!(s.reduced_index(NodeId(2), Dof::Ux).unwrap(), None);
    assert_eq!(
        s.reduced_index(NodeId(3), Dof::Ux).unwrap(),
        Some(DofIndex(1))
    );
    code(s.reduced_index(NodeId(99), Dof::Ux), "ENG-E-0003");
    code(s.reduced_index(NodeId(1), Dof::Uy), "ENG-E-0011");
}
