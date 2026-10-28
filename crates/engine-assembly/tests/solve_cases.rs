//! One reusable factorization per batch; solve preserves its previous semantics.
#[path = "fixtures/counting.rs"]
mod counting;
#[path = "fixtures/rigid_cases.rs"]
mod rigid_cases;
use counting::CountingBackend;
use engine_assembly::{ElementContribution, LinearSystem, PrescribedDof};
use engine_contracts::{CaseRef, SystemView};
use engine_core::{Dim, Dof, DofIndex, EResult, NodeId};
use engine_linalg::DMat;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

fn system(backend: CountingBackend) -> LinearSystem<CountingBackend> {
    let mut model = Model::new();
    for id in 1..=2 {
        let mut n = Node::new(NodeId(id), Dim::D2, [f64::from(id), 0.0, 0.0]);
        n.dofs = vec![Dof::Ux];
        model.nodes.push(n);
    }
    model.supports.push(Support {
        id: "base".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux],
    });
    for (name, p) in [("A", 10.0), ("B", 20.0), ("AB", 30.0)] {
        model.cases.push(LoadCase {
            name: name.into(),
            loads: vec![NodalLoad {
                node: NodeId(2),
                values: vec![DofValue {
                    dof: Dof::Ux,
                    value: p,
                }],
            }],
        });
    }
    let e = ElementContribution {
        location: vec![DofIndex(0), DofIndex(1)],
        stiffness: DMat::new(2, 2, vec![100.0, -100.0, -100.0, 100.0]).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    };
    LinearSystem::from_model(
        backend,
        &model,
        &[e],
        &[PrescribedDof {
            node: NodeId(1),
            dof: Dof::Ux,
            value: 0.03,
        }],
    )
    .unwrap()
}

#[test]
fn one_factorization_all_rhs_and_unchanged_solve() {
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let solves = backend.solves.clone();
    let mut s = system(backend);
    s.accept(&[0.7]).unwrap();
    let cases: Vec<_> = ["B", "A", "AB"]
        .iter()
        .map(|n| (CaseRef((*n).into()), 2.0))
        .collect();
    let u = s.solve_cases(&cases).unwrap();
    assert_eq!(factors.get(), 1);
    assert_eq!(solves.get(), 3);
    for (actual, expected) in u.iter().zip([0.43, 0.23, 0.63]) {
        assert!((actual[0] - expected).abs() < 1e-12);
    }
    assert_eq!(s.accepted_displacement(), &[0.7]);
    assert_eq!(s.trial_displacement(), &[0.7]);
    for ((case, factor), expected) in cases.iter().zip(&u) {
        assert_eq!(&s.solve(case, *factor).unwrap(), expected);
    }
    assert_eq!(factors.get(), 4, "solve still factorizes on every call");
}

#[test]
fn invalid_lists_and_values() {
    let mut s = system(CountingBackend::default());
    for (cases, code) in [
        (vec![], "ENG-E-0010"),
        (
            vec![(CaseRef("A".into()), 1.0), (CaseRef("A".into()), 2.0)],
            "ENG-E-0010",
        ),
        (vec![(CaseRef("missing".into()), 1.0)], "ENG-E-0003"),
        (vec![(CaseRef("A".into()), f64::NAN)], "ENG-E-0008"),
        (vec![(CaseRef("A".into()), f64::INFINITY)], "ENG-E-0008"),
    ] {
        assert_eq!(s.solve_cases(&cases).unwrap_err().code, code);
    }
}

#[test]
fn rigid_diaphragm_cases_match_single_solves_and_analytic_solution() -> EResult<()> {
    for settlement in [None, Some(0.03)] {
        let backend = CountingBackend::default();
        let factors = backend.factorizations.clone();
        let solves = backend.solves.clone();
        let mut s = rigid_cases::system(backend, settlement)?;
        let accepted = vec![0.7; s.n_free_dofs()];
        let trial = vec![-0.2; s.n_free_dofs()];
        s.accept(&accepted)?;
        s.internal_force(&trial, &mut vec![0.0; s.n_free_dofs()])?;
        let cases = rigid_cases::cases();
        let solutions = s.solve_cases(&cases)?;
        assert_eq!(solutions.len(), cases.len());
        assert_eq!(factors.get(), 1);
        assert_eq!(solves.get(), cases.len());
        for ((case, factor), free) in cases.iter().zip(&solutions) {
            assert_eq!(*free, s.solve(case, *factor)?);
            rigid_cases::verify_solution(&s, free, case, *factor, settlement)?;
        }
        assert_eq!(factors.get(), 1 + cases.len());
        assert_eq!(s.accepted_displacement(), accepted);
        assert_eq!(s.trial_displacement(), trial);
    }
    Ok(())
}

#[test]
fn rigid_cases_validate_entire_batch_before_factorization() -> EResult<()> {
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let solves = backend.solves.clone();
    let mut s = rigid_cases::system(backend, Some(0.03))?;
    let accepted = vec![0.7; s.n_free_dofs()];
    let trial = vec![-0.2; s.n_free_dofs()];
    s.accept(&accepted)?;
    s.internal_force(&trial, &mut vec![0.0; s.n_free_dofs()])?;
    for (name, factor, code) in [
        ("A", 2.0, "ENG-E-0010"),
        ("ausente", 1.0, "ENG-E-0003"),
        ("B", f64::NAN, "ENG-E-0008"),
        ("B", f64::INFINITY, "ENG-E-0008"),
        // Physical forces are finite; the arm x=3 overflows during projection.
        ("B", f64::MAX / 32.0, "ENG-E-0008"),
    ] {
        let cases = [(CaseRef("A".into()), 1.0), (CaseRef(name.into()), factor)];
        assert_eq!(s.solve_cases(&cases).unwrap_err().code, code);
        assert_eq!(factors.get(), 0);
        assert_eq!(solves.get(), 0);
        assert_eq!(s.accepted_displacement(), accepted);
        assert_eq!(s.trial_displacement(), trial);
    }
    Ok(())
}
