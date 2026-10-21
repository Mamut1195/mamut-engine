//! Follow-up to J4's clean-room review of ENG-ASM-001.
use engine_assembly::{
    assemble::{assemble_matrix, assemble_matrix_with_tolerance, EXACT_SYMMETRY},
    ElementContribution, LinearSystem,
};
use engine_contracts::SystemView;
use engine_core::{Dim, Dof, DofIndex, EResult, NodeId};
use engine_linalg::{DMat, ReferenceBackend, SymMatrix};
use engine_model::{DofValue, Model, NodalMass, Node, Support};

fn code<T>(r: EResult<T>, expected: &str) {
    match r {
        Ok(_) => panic!("Se esperaba {expected}"),
        Err(e) => assert_eq!(e.code, expected, "{e}"),
    }
}
fn model() -> Model {
    let mut m = Model::new();
    for i in 0..2u32 {
        let mut n = Node::new(NodeId(i + 1), Dim::D2, [f64::from(i), 0.0, 0.0]);
        n.dofs = vec![Dof::Ux];
        m.nodes.push(n);
    }
    m.supports.push(Support {
        id: "a".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux],
    });
    m
}
fn element(k: f64) -> ElementContribution {
    ElementContribution {
        location: vec![DofIndex(0), DofIndex(1)],
        stiffness: DMat::new(2, 2, vec![k, -k, -k, k]).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    }
}

#[test]
fn geometric_rejects_states_other_than_reference() {
    let mut e = element(10.0);
    e.geometric = Some(DMat::new(2, 2, vec![3.0, -3.0, -3.0, 3.0]).unwrap());
    let mut s = LinearSystem::from_model(ReferenceBackend, &model(), &[e], &[]).unwrap();
    let mut out = SymMatrix::zeros(1).unwrap();
    // Default reference: u = 0, without calling set_geometric_reference.
    s.geometric(&[0.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[3.0]);
    out = SymMatrix::zeros(1).unwrap();
    code(s.geometric(&[0.5], &mut out), "ENG-E-0013");
    assert_eq!(out.dense().values(), &[0.0], "salida intacta ante error");
    s.set_geometric_reference(&[0.5]).unwrap();
    s.geometric(&[0.5], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[3.0]);
    code(s.geometric(&[0.0], &mut out), "ENG-E-0013");
    code(s.set_geometric_reference(&[0.0, 0.0]), "ENG-E-0010");
    code(s.set_geometric_reference(&[f64::NAN]), "ENG-E-0008");
}

#[test]
fn geometric_without_contributions_is_zero_for_any_state() {
    let mut s =
        LinearSystem::from_model(ReferenceBackend, &model(), &[element(10.0)], &[]).unwrap();
    let mut out = SymMatrix::zeros(1).unwrap();
    s.geometric(&[7.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[0.0]);
}

#[test]
fn dof_label_out_of_range_does_not_panic_via_try() {
    let s = LinearSystem::from_model(ReferenceBackend, &model(), &[element(10.0)], &[]).unwrap();
    assert_eq!(s.try_dof_label(DofIndex(0)).unwrap(), (NodeId(2), Dof::Ux));
    assert_eq!(s.dof_label(DofIndex(0)), (NodeId(2), Dof::Ux));
    code(s.try_dof_label(DofIndex(1)), "ENG-E-0010");
    code(s.try_dof_label(DofIndex(u32::MAX)), "ENG-E-0010");
}

#[test]
fn negative_nodal_mass_and_element_stiffness_are_rejected() {
    let mut m = model();
    m.masses.push(NodalMass {
        id: "m".into(),
        node: NodeId(2),
        values: vec![DofValue {
            dof: Dof::Ux,
            value: -1.0,
        }],
    });
    code(
        LinearSystem::from_model(ReferenceBackend, &m, &[element(10.0)], &[]),
        "ENG-E-0010",
    );
    m.masses[0].values[0].value = 0.0;
    LinearSystem::from_model(ReferenceBackend, &m, &[element(10.0)], &[]).unwrap();
    code(
        LinearSystem::from_model(ReferenceBackend, &model(), &[element(-10.0)], &[]),
        "ENG-E-0010",
    );
    let negative = DMat::new(2, 2, vec![-1.0, 0.0, 0.0, 1.0]).unwrap();
    for lumped in [true, false] {
        let mut e = element(10.0);
        if lumped {
            e.mass_lumped = Some(negative.clone());
        } else {
            e.mass_consistent = Some(negative.clone());
        }
        code(
            LinearSystem::from_model(ReferenceBackend, &model(), &[e], &[]),
            "ENG-E-0010",
        );
    }
}

#[test]
fn symmetry_tolerance_is_explicit_and_relative() {
    let loc = [DofIndex(0), DofIndex(1)];
    let skew = DMat::new(2, 2, vec![1.0, 2.0, 2.0 + 1e-9, 1.0]).unwrap();
    assert_eq!(EXACT_SYMMETRY, 0.0);
    code(
        assemble_matrix(&mut ReferenceBackend, 2, &[(&loc, &skew)]),
        "ENG-E-0010",
    );
    code(
        assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &skew)], 1e-12),
        "ENG-E-0010",
    );
    let k =
        assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &skew)], 1e-6).unwrap();
    let d = k.to_dense().unwrap();
    assert_eq!(d.get(0, 1), d.get(1, 0), "se simetriza con la media");
    assert_eq!(d.get(0, 1), Some(2.0 + 0.5e-9));
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        code(
            assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &skew)], bad),
            "ENG-E-0010",
        );
    }
}

#[test]
fn symmetry_check_and_mean_do_not_overflow() {
    // J4 (#23): assemble.rs:113/124. Finite values whose sum or difference
    // overflows f64 must not change the verdict or produce ∞.
    let loc = [DofIndex(0), DofIndex(1)];
    let big = 1.7e308;
    let same = DMat::new(2, 2, vec![1.0, big, big, 1.0]).unwrap();
    // a == b: the mean (a+b)/2 overflowed to ∞ with tolerance > 0.
    let k = assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &same)], 1e-6)
        .unwrap()
        .to_dense()
        .unwrap();
    assert_eq!(k.get(0, 1), Some(big));
    assert_eq!(k.get(1, 0), Some(big));
    let opposite = DMat::new(2, 2, vec![1.0, big, -big, 1.0]).unwrap();
    // |a−b| = 2·big overflows: never accepted through ∞ ≤ ∞, and rejected with
    // insufficient tolerances.
    for tol in [0.0, 1e-6, 1.0, 1.9] {
        code(
            assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &opposite)], tol),
            "ENG-E-0010",
        );
    }
    // With tol = 2 (|a−b| = 2·máx) it is mathematically symmetric within the
    // tolerance: accepted, and the mean is 0.
    let k = assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &opposite)], 2.0)
        .unwrap()
        .to_dense()
        .unwrap();
    assert_eq!(k.get(0, 1), Some(0.0));
    assert_eq!(k.get(1, 0), Some(0.0));
}

#[test]
fn zero_reference_geometric_still_rejects_other_states() {
    // T1 (J4): Kg(ref)=0 with geometric contributions must not return Kg=0 for u≠ref.
    let mut e = element(10.0);
    e.geometric = Some(DMat::new(2, 2, vec![0.0; 4]).unwrap());
    let mut s = LinearSystem::from_model(ReferenceBackend, &model(), &[e], &[]).unwrap();
    let mut out = SymMatrix::zeros(1).unwrap();
    s.geometric(&[0.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[0.0]);
    code(s.geometric(&[0.5], &mut out), "ENG-E-0013");
    s.set_geometric_reference(&[0.5]).unwrap();
    s.geometric(&[0.5], &mut out).unwrap();
    code(s.geometric(&[0.0], &mut out), "ENG-E-0013");
}

#[test]
fn subnormal_asymmetry_and_means_are_exact() {
    // J4 (#35): 5e-324 versus 0 cannot be accepted with tol=0; two equal 5e-324
    // values do not lose their value with tol>0.
    let loc = [DofIndex(0), DofIndex(1)];
    let tiny = 5e-324;
    let skew = DMat::new(2, 2, vec![1.0, tiny, 0.0, 1.0]).unwrap();
    code(
        assemble_matrix(&mut ReferenceBackend, 2, &[(&loc, &skew)]),
        "ENG-E-0010",
    );
    let same = DMat::new(2, 2, vec![1.0, tiny, tiny, 1.0]).unwrap();
    for tol in [0.0, 1e-6] {
        let k = assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &same)], tol)
            .unwrap()
            .to_dense()
            .unwrap();
        assert_eq!(k.get(0, 1), Some(tiny), "tol {tol}");
        assert_eq!(k.get(1, 0), Some(tiny), "tol {tol}");
    }
    // With sufficient tolerance the mean of different values is exact.
    let k = assemble_matrix_with_tolerance(&mut ReferenceBackend, 2, &[(&loc, &skew)], 1e-6)
        .unwrap()
        .to_dense()
        .unwrap();
    assert_eq!(k.get(0, 1), k.get(1, 0));
}
