//! Dynamic Kg boundary: reduction, replacement, and atomic errors.
use engine_assembly::{ElementContribution, GeometricProducer, LinearSystem, PrescribedDof};
use engine_contracts::{conformance::check_system_view, CaseRef, SystemView};
use engine_core::{Dim, Dof, DofIndex, EResult, EngineError, EntityRef, NodeId};
use engine_linalg::{DMat, ReferenceBackend, SymMatrix};
use engine_model::{LoadCase, Model, Node, Support};
use std::rc::Rc;

fn model() -> Model {
    let mut model = Model::new();
    for id in 1..=3 {
        let mut node = Node::new(NodeId(id), Dim::D2, [f64::from(id), 0.0, 0.0]);
        node.dofs = vec![Dof::Ux];
        model.nodes.push(node);
    }
    model.supports.push(Support {
        id: "base".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux],
    });
    model.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![],
    });
    model
}

fn matrix(value: f64) -> DMat {
    DMat::new(2, 2, vec![value, -value, -value, value]).unwrap()
}

fn element(indices: [u32; 2], kg: Option<f64>) -> ElementContribution {
    ElementContribution {
        location: indices.map(DofIndex).to_vec(),
        stiffness: matrix(100.0),
        mass_lumped: None,
        mass_consistent: None,
        geometric: kg.map(matrix),
        loads: vec![],
    }
}

fn producer(indices: [u32; 2], scale: f64) -> GeometricProducer {
    // Rc proves the API does not impose Send/Sync. The result is deterministic.
    let scale = Rc::new(scale);
    GeometricProducer {
        location: indices.map(DofIndex).to_vec(),
        eval: Box::new(move |u| {
            let value = *scale * (u[1] - u[0]);
            DMat::new(2, 2, vec![value, -value, -value, value])
        }),
    }
}

fn code<T>(result: EResult<T>, expected: &str) {
    match result {
        Ok(_) => panic!("Expected {expected}"),
        Err(e) => assert_eq!(e.code, expected, "{e}"),
    }
}

fn sentinel() -> SymMatrix {
    SymMatrix::new(DMat::new(2, 2, vec![91.0, 13.0, 13.0, 92.0]).unwrap()).unwrap()
}

#[test]
fn expands_prescribed_replaces_static_and_overwrites_in_element_order() {
    let mut s = LinearSystem::from_model(
        ReferenceBackend,
        &model(),
        &[element([0, 1], Some(7.0)), element([1, 2], Some(11.0))],
        &[PrescribedDof {
            node: NodeId(1),
            dof: Dof::Ux,
            value: 0.25,
        }],
    )
    .unwrap();
    let original = s.global_geometric().clone();
    // Registration reverses element order: assembly preserves the latter.
    s.set_geometric_producers(vec![producer([1, 2], 20.0), producer([0, 1], 10.0)])
        .unwrap();
    let mut out = sentinel();
    s.geometric(&[1.0, 3.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[47.5, -40.0, -40.0, 40.0]);
    assert_eq!(s.trial_displacement(), &[1.0, 3.0]);
    assert_eq!(s.accepted_displacement(), &[0.0, 0.0]);
    let first = out.clone();
    s.geometric(&[1.0, 3.0], &mut out).unwrap();
    assert_eq!(out, first, "overwrites, does not accumulate");
    s.geometric(&[0.0, 0.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[-2.5, 0.0, 0.0, 0.0]);
    assert_eq!(
        s.global_geometric(),
        &original,
        "the global API preserves the snapshot"
    );
    s.rollback();
    assert_eq!(s.trial_displacement(), &[0.0, 0.0]);
    s.set_geometric_producers(vec![]).unwrap();
    s.geometric(&[0.0, 0.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[18.0, -11.0, -11.0, 11.0]);
    code(s.geometric(&[1.0, 3.0], &mut out), "ENG-E-0013");
}

#[test]
fn mixed_static_even_zero_retains_reference_rule() {
    let mut s = LinearSystem::from_model(
        ReferenceBackend,
        &model(),
        &[element([0, 1], Some(7.0)), element([1, 2], Some(0.0))],
        &[],
    )
    .unwrap();
    s.set_geometric_producers(vec![producer([0, 1], 10.0)])
        .unwrap();
    let mut out = sentinel();
    let before = out.clone();
    code(s.geometric(&[0.1, 0.2], &mut out), "ENG-E-0013");
    assert_eq!(out, before);
    assert_eq!(s.trial_displacement(), &[0.0, 0.0]);
    s.set_geometric_reference(&[0.1, 0.2]).unwrap();
    s.geometric(&[0.1, 0.2], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn registration_validates_and_preserves_existing_producers_on_error() {
    let mut s = LinearSystem::from_model(ReferenceBackend, &model(), &[element([0, 1], None)], &[])
        .unwrap();
    s.set_geometric_producers(vec![producer([0, 1], 10.0)])
        .unwrap();
    let pattern = s.pattern().clone();
    for (indices, expected) in [
        (vec![], "ENG-E-0010"),
        (vec![DofIndex(3)], "ENG-E-0010"),
        (vec![DofIndex(0), DofIndex(0)], "ENG-E-0002"),
    ] {
        code(
            s.set_geometric_producers(vec![GeometricProducer {
                location: indices,
                eval: Box::new(|_| panic!("Invalid location is not evaluated")),
            }]),
            expected,
        );
    }
    code(
        s.set_geometric_producers(vec![producer([0, 1], 1.0), producer([0, 1], 2.0)]),
        "ENG-E-0002",
    );
    for matrix in [
        DMat::zeros(1, 1).unwrap(),
        DMat::new(2, 2, vec![0.0, 1.0, 2.0, 0.0]).unwrap(),
    ] {
        code(
            s.set_geometric_producers(vec![GeometricProducer {
                location: vec![DofIndex(0), DofIndex(1)],
                eval: Box::new(move |_| Ok(matrix.clone())),
            }]),
            "ENG-E-0010",
        );
    }
    let mut out = sentinel();
    s.geometric(&[0.5, 0.0], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[5.0, 0.0, 0.0, 0.0]);
    assert_eq!(s.pattern(), &pattern);
    let repeated = element([0, 1], Some(1.0));
    let mut ambiguous = LinearSystem::from_model(
        ReferenceBackend,
        &model(),
        &[repeated.clone(), repeated],
        &[],
    )
    .unwrap();
    code(
        ambiguous.set_geometric_producers(vec![producer([0, 1], 1.0)]),
        "ENG-E-0002",
    );
}

#[test]
fn arbitrary_state_errors_preserve_output_and_trial_and_propagate_full_error() {
    let mut s = LinearSystem::from_model(
        ReferenceBackend,
        &model(),
        &[element([0, 1], Some(0.0))],
        &[],
    )
    .unwrap();
    let mut expected = EngineError::new("ENG-E-0005", "fallo propio del productor");
    expected.entities.push(EntityRef::Node(NodeId(3)));
    let propagated = expected.clone();
    s.set_geometric_producers(vec![GeometricProducer {
        location: vec![DofIndex(0), DofIndex(1)],
        eval: Box::new(move |u| match u[1] {
            1.0 => Err(propagated.clone()),
            2.0 => DMat::new(2, 2, vec![0.0, 1.0, 2.0, 0.0]),
            3.0 => DMat::zeros(1, 1),
            4.0 => DMat::new(2, 2, vec![f64::INFINITY; 4]),
            _ => Ok(matrix(u[1])),
        }),
    }])
    .unwrap();
    let mut out = sentinel();
    s.geometric(&[0.5, 0.25], &mut out).unwrap();
    let before = out.clone();
    let err = s.geometric(&[1.0, 0.0], &mut out).unwrap_err();
    assert_eq!(err.code, expected.code);
    assert_eq!(err.message, expected.message);
    assert_eq!(err.entities.len(), 1);
    assert!(matches!(err.entities[0], EntityRef::Node(NodeId(3))));
    for (u, expected) in [
        (vec![2.0, 0.0], "ENG-E-0010"),
        (vec![3.0, 0.0], "ENG-E-0010"),
        (vec![4.0, 0.0], "ENG-E-0008"),
        (vec![0.0], "ENG-E-0010"),
        (vec![f64::NAN, 0.0], "ENG-E-0008"),
        (vec![0.0, f64::INFINITY], "ENG-E-0008"),
    ] {
        code(s.geometric(&u, &mut out), expected);
        assert_eq!(out, before);
        assert_eq!(s.trial_displacement(), &[0.5, 0.25]);
    }
    let mut wrong = SymMatrix::zeros(1).unwrap();
    code(s.geometric(&[0.0, 0.0], &mut wrong), "ENG-E-0010");
    assert_eq!(wrong.dense().values(), &[0.0]);
}

#[test]
fn standalone_and_ordered_locations_extend_pattern_and_sum_shared_dofs() {
    let mut s = LinearSystem::from_model(ReferenceBackend, &model(), &[element([0, 1], None)], &[])
        .unwrap();
    assert!(!s.pattern().contains(0, 1));
    s.set_geometric_producers(vec![producer([0, 1], 10.0), producer([2, 1], 20.0)])
        .unwrap();
    assert!(s.pattern().contains(0, 1));
    assert!(s.pattern().contains(1, 0));
    let mut out = sentinel();
    s.geometric(&[0.5, 0.25], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[10.0, -5.0, -5.0, 5.0]);
    // A finite u is not rejected for overflow of K*u, irrelevant to Kg.
    s.set_geometric_producers(vec![producer([0, 1], 0.0)])
        .unwrap();
    s.geometric(&[f64::MAX, f64::MAX], &mut out).unwrap();
    assert_eq!(out.dense().values(), &[0.0; 4]);
    assert!(
        !s.pattern().contains(0, 1),
        "registration also replaces the dynamic pattern"
    );
}

#[test]
fn conformance_at_states_other_than_zero_reference_is_clean() {
    let mut s = LinearSystem::from_model(
        ReferenceBackend,
        &model(),
        &[element([0, 1], Some(0.0)), element([1, 2], Some(0.0))],
        &[],
    )
    .unwrap();
    s.set_geometric_producers(vec![producer([0, 1], 10.0), producer([1, 2], 20.0)])
        .unwrap();
    let diagnostics = check_system_view(
        &mut s,
        &[vec![0.0, 0.0], vec![0.1, 0.2], vec![-0.2, 0.3]],
        &CaseRef("P".into()),
        1e-5,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn partial_evaluations_and_overflowing_sums_leave_output_and_state_intact() {
    let mut s = LinearSystem::from_model(
        ReferenceBackend,
        &model(),
        &[element([0, 1], None), element([1, 2], None)],
        &[],
    )
    .unwrap();
    s.accept(&[0.1, 0.2]).unwrap();
    s.set_geometric_producers(vec![
        producer([0, 1], 1.0),
        GeometricProducer {
            location: vec![DofIndex(1), DofIndex(2)],
            eval: Box::new(|u| {
                if u[1] == 3.0 {
                    Err(EngineError::new("ENG-E-0005", "Segundo productor falló."))
                } else {
                    Ok(matrix(u[1] - u[0]))
                }
            }),
        },
    ])
    .unwrap();
    let mut out = sentinel();
    let before = out.clone();
    code(s.geometric(&[1.0, 3.0], &mut out), "ENG-E-0005");
    assert_eq!(out, before);
    assert_eq!(s.trial_displacement(), &[0.1, 0.2]);
    assert_eq!(s.accepted_displacement(), &[0.1, 0.2]);
    s.set_geometric_producers(vec![
        producer([0, 1], 0.75 * f64::MAX),
        producer([1, 2], 0.75 * f64::MAX),
    ])
    .unwrap();
    // Each matrix is finite; the shared diagonal sum overflows.
    code(s.geometric(&[1.0, 2.0], &mut out), "ENG-E-0008");
    assert_eq!(out, before);
    assert_eq!(s.trial_displacement(), &[0.1, 0.2]);
    assert_eq!(s.accepted_displacement(), &[0.1, 0.2]);
}

#[test]
fn floating_point_accumulation_follows_original_element_order() {
    let third = ElementContribution {
        location: vec![DofIndex(0), DofIndex(2), DofIndex(1)],
        stiffness: DMat::zeros(3, 3).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    };
    let mut s = LinearSystem::from_model(
        ReferenceBackend,
        &model(),
        &[element([0, 1], None), element([1, 2], None), third],
        &[],
    )
    .unwrap();
    let diagonal = |indices: Vec<DofIndex>, value| {
        let n = indices.len();
        let i = indices.iter().position(|d| *d == DofIndex(1)).unwrap();
        GeometricProducer {
            location: indices,
            eval: Box::new(move |_| {
                let mut values = vec![0.0; n * n];
                values[i * n + i] = value;
                DMat::new(n, n, values)
            }),
        }
    };
    s.set_geometric_producers(vec![
        diagonal(vec![DofIndex(0), DofIndex(2), DofIndex(1)], 1.0),
        diagonal(vec![DofIndex(1), DofIndex(2)], -1e16),
        diagonal(vec![DofIndex(0), DofIndex(1)], 1e16),
    ])
    .unwrap();
    let mut out = sentinel();
    s.geometric(&[0.1, 0.2], &mut out).unwrap();
    // (1e16−1e16)+1 = 1, whereas (1−1e16)+1e16 = 0.
    assert_eq!(out.dense().get(0, 0), Some(1.0));
}
