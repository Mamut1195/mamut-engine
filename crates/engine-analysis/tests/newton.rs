//! source: own derivations; citation: provenance/ENG-SOL-001.md.
//! Closed-form SI solutions, without external sources or oracles.
#[path = "../../engine-assembly/tests/fixtures/counting.rs"]
mod counting;
use counting::CountingBackend;
use engine_analysis::{
    newton::{NewtonElement, NewtonOptions, NewtonSystem, StaticNewton},
    StaticLinear,
};
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem, PrescribedDof};
use engine_contracts::{
    CaseRef, ElementCtx, ElementParams, Status, SystemView, UniaxialLaw, UniaxialResponse,
};
use engine_core::{Dim, Dof, DofIndex, EResult, EntityRef, NodeId, Real};
use engine_elements_spring::{NonlinearSpring, SpringAxes, SpringConnection};
use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};
use engine_linalg::{DMat, SymMatrix};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
use std::sync::Arc;

fn close(a: Real, b: Real) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}
fn case() -> CaseRef {
    CaseRef("P".into())
}
fn model(n: u32, p: Real) -> Model {
    let mut m = Model::new();
    for id in 1..=n {
        let mut node = Node::new(NodeId(id), Dim::D2, [id as Real, 0.0, 0.0]);
        node.dofs = vec![Dof::Ux];
        m.nodes.push(node);
    }
    m.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![NodalLoad {
            node: NodeId(n),
            values: vec![DofValue {
                dof: Dof::Ux,
                value: p,
            }],
        }],
    });
    m
}
fn bilinear(k: Real) -> Arc<dyn UniaxialLaw> {
    Arc::new(
        MultilinearElastic::new(
            vec![
                (-0.5, -0.18 * k),
                (-0.1, -0.1 * k),
                (0.0, 0.0),
                (0.1, 0.1 * k),
                (0.5, 0.18 * k),
            ],
            MultilinearMode::Both,
        )
        .unwrap(),
    )
}
fn binding(m: &Model, nodes: &[u32], law: Arc<dyn UniaxialLaw>) -> NewtonElement {
    let s = NonlinearSpring::new(
        Dim::D2,
        if nodes.len() == 1 {
            SpringConnection::Ground
        } else {
            SpringConnection::BetweenNodes
        },
        law,
        &[Dof::Ux],
        SpringAxes::Global,
    )
    .unwrap();
    let map = DofMap::new(&m.nodes).unwrap();
    let location = nodes
        .iter()
        .map(|&n| map.global_index(NodeId(n), Dof::Ux).unwrap())
        .collect();
    let coords = nodes
        .iter()
        .map(|&n| map.coordinates(NodeId(n)).unwrap())
        .collect::<Vec<_>>();
    NewtonElement::new(
        Box::new(s),
        location,
        &ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &ElementParams::default(),
        },
    )
    .unwrap()
}
fn contribution(location: &[u32], values: Vec<Real>) -> ElementContribution {
    ElementContribution {
        location: location.iter().map(|&i| DofIndex(i)).collect(),
        stiffness: DMat::new(location.len(), location.len(), values).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: Vec::new(),
    }
}
fn ground(p: Real) -> NewtonSystem<CountingBackend> {
    let m = model(1, p);
    NewtonSystem::new(
        LinearSystem::from_model(CountingBackend::default(), &m, &[], &[]).unwrap(),
        vec![binding(&m, &[1], bilinear(100.0))],
    )
    .unwrap()
}
fn run(
    s: &mut NewtonSystem<CountingBackend>,
    factors: &[Real],
    options: NewtonOptions,
) -> engine_analysis::newton::NewtonResults {
    StaticNewton::run(s, &case(), factors, options).unwrap()
}

#[test]
fn ground_bilinear_crosses_corner_in_exactly_two_corrections() {
    let m = model(1, 16.0);
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let solves = backend.solves.clone();
    let linear = LinearSystem::from_model(backend, &m, &[], &[]).unwrap();
    let mut s = NewtonSystem::new(linear, vec![binding(&m, &[1], bilinear(100.0))]).unwrap();
    let r = run(&mut s, &[1.0], NewtonOptions::default());
    assert_eq!(StaticNewton::KIND, "static.newton");
    assert_eq!(r.status, Status::Completed);
    close(r.free_displacements[0], 0.4);
    assert_eq!(r.steps[0].iterations, 2);
    assert_eq!(factors.get(), 2);
    assert_eq!(solves.get(), 2);
    assert!(r.steps[0].force_norm < 1e-10);
    assert_eq!(
        s.linear_system().trial_displacement(),
        s.linear_system().accepted_displacement()
    );
}

#[test]
fn cancellation_inside_newton_iteration_keeps_last_accepted_step_and_rolls_back() {
    let m = model(1, 16.0);
    let backend = CountingBackend::default();
    let solves = backend.solves.clone();
    let linear = LinearSystem::from_model(backend, &m, &[], &[]).unwrap();
    let mut system = NewtonSystem::new(linear, vec![binding(&m, &[1], bilinear(100.0))]).unwrap();
    let previous = run(&mut system, &[0.25], NewtonOptions::default());
    let previous_solves = solves.get();
    let error = StaticNewton::run_with(
        &mut system,
        &case(),
        &[1.0],
        NewtonOptions::default(),
        || {
            if solves.get() > previous_solves {
                Err(engine_core::EngineError::new("TEST-CANCEL", "Cancelado"))
            } else {
                Ok(())
            }
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "TEST-CANCEL");
    assert_eq!(solves.get(), previous_solves + 1);
    assert_eq!(system.accepted_factor(), previous.accepted_factor);
    assert_eq!(system.accepted_element_states(), previous.element_states);
    assert_eq!(
        system.linear_system().accepted_displacement(),
        previous.free_displacements
    );
    assert_eq!(
        system.linear_system().trial_displacement(),
        previous.free_displacements
    );
}

#[test]
fn two_nonlinear_springs_in_series_closed_form_each_segment() {
    let m = model(2, 24.0);
    let mut s = NewtonSystem::new(
        LinearSystem::from_model(CountingBackend::default(), &m, &[], &[]).unwrap(),
        vec![
            binding(&m, &[1], bilinear(100.0)),
            binding(&m, &[1, 2], bilinear(200.0)),
        ],
    )
    .unwrap();
    // δ1=P/100 (P≤10), or .1+(P−10)/20; δ2=P/200 (P≤20), or .1+(P−20)/40.
    for (lambda, delta1, delta2) in [(1.0 / 6.0, 0.04, 0.02), (0.5, 0.2, 0.06), (1.0, 0.8, 0.2)] {
        let r = run(&mut s, &[lambda], NewtonOptions::default());
        assert_eq!(r.status, Status::Completed);
        close(r.free_displacements[0], delta1);
        close(r.free_displacements[1] - r.free_displacements[0], delta2);
        let f1 = if delta1 <= 0.1 {
            100.0 * delta1
        } else {
            10.0 + 20.0 * (delta1 - 0.1)
        };
        let f2 = if delta2 <= 0.1 {
            200.0 * delta2
        } else {
            20.0 + 40.0 * (delta2 - 0.1)
        };
        close(f1, f2);
        close(f1, 24.0 * lambda);
    }
}

#[test]
fn compression_gap_closes_from_open_state_with_physical_parallel_spring() {
    let m = model(1, -20.0);
    let gap = Arc::new(
        MultilinearElastic::new(
            vec![(-1.25, -100.0), (-0.25, 0.0), (0.0, 0.0)],
            MultilinearMode::CompressionOnly,
        )
        .unwrap(),
    );
    let mut s = NewtonSystem::new(
        LinearSystem::from_model(
            CountingBackend::default(),
            &m,
            &[contribution(&[0], vec![10.0])],
            &[],
        )
        .unwrap(),
        vec![binding(&m, &[1], gap)],
    )
    .unwrap();
    // Open: K=10, u1=−2. Closed: 10u+100(u+.25)=−20.
    let r = run(&mut s, &[1.0], NewtonOptions::default());
    assert_eq!(r.status, Status::Completed);
    assert_eq!(r.steps[0].iterations, 2);
    close(r.free_displacements[0], -45.0 / 110.0);
}

#[test]
fn linear_matches_static_linear_one_iteration_with_prescribed_and_equivalent_loads() {
    let mut m = model(2, 12.0);
    m.supports.push(Support {
        id: "s1".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux],
    });
    let mut e = contribution(&[0, 1], vec![100.0, -100.0, -100.0, 100.0]);
    e.loads.push(ElementLoad {
        case: case(),
        values: vec![7.0, 3.0],
    });
    let prescribed = [PrescribedDof {
        node: NodeId(1),
        dof: Dof::Ux,
        value: 0.03,
    }];
    let mut linear =
        LinearSystem::from_model(CountingBackend::default(), &m, &[e.clone()], &prescribed)
            .unwrap();
    let exact = StaticLinear::run(&mut linear, &[(case(), 2.0)]).unwrap();
    let mut s = NewtonSystem::new(
        LinearSystem::from_model(CountingBackend::default(), &m, &[e], &prescribed).unwrap(),
        vec![],
    )
    .unwrap();
    let r = run(&mut s, &[2.0], NewtonOptions::default());
    assert_eq!(r.status, Status::Completed);
    assert_eq!(r.steps[0].iterations, 1);
    close(r.free_displacements[0], 0.33);
    assert_eq!(r.global_displacements, exact.cases[0].global_displacements);
    close(r.global_displacements[0], 0.03);
}

#[test]
fn nonlinear_prescribed_displacement_is_not_scaled() {
    let mut m = model(2, 16.0);
    m.supports.push(Support {
        id: "s1".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux],
    });
    let linear = LinearSystem::from_model(
        CountingBackend::default(),
        &m,
        &[],
        &[PrescribedDof {
            node: NodeId(1),
            dof: Dof::Ux,
            value: 0.03,
        }],
    )
    .unwrap();
    let mut s = NewtonSystem::new(linear, vec![binding(&m, &[1, 2], bilinear(100.0))]).unwrap();
    let r = run(&mut s, &[0.25, 1.0], NewtonOptions::default());
    assert_eq!(r.status, Status::Completed);
    close(r.global_displacements[0], 0.03);
    close(r.global_displacements[1], 0.43);
}

// In-house test law: plasticity with kinematic hardening H=25,
// k=100 and limit |F−Hp|=10; elastoplastic tangent = kH/(k+H)=20.
struct Hardening;
impl UniaxialLaw for Hardening {
    fn kind(&self) -> &'static str {
        "test.hardening"
    }
    fn state_len(&self) -> usize {
        1
    }
    fn initial_state(&self) -> Vec<Real> {
        vec![0.0]
    }
    fn initial_tangent(&self) -> Real {
        100.0
    }
    fn respond(&self, u: Real, state: &[Real]) -> EResult<UniaxialResponse> {
        let xi = 100.0 * (u - state[0]) - 25.0 * state[0];
        let phi = xi.abs() - 10.0;
        let (p, k) = if phi > 1e-12 {
            (state[0] + xi.signum() * phi / 125.0, 20.0)
        } else {
            (state[0], 100.0)
        };
        Ok(UniaxialResponse {
            stress: 100.0 * (u - p),
            tangent: k,
            trial_state: vec![p],
        })
    }
}

#[test]
fn max_one_rolls_back_last_step_history_and_stops_without_hidden_retries() {
    let m = model(1, 16.0);
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let solves = backend.solves.clone();
    let mut s = NewtonSystem::new(
        LinearSystem::from_model(backend, &m, &[], &[]).unwrap(),
        vec![binding(&m, &[1], Arc::new(Hardening))],
    )
    .unwrap();
    let options = NewtonOptions {
        max_iterations: 1,
        ..NewtonOptions::default()
    };
    let r = run(&mut s, &[0.25, 1.0, 2.0], options);
    assert_eq!(r.status, Status::NotConverged);
    assert_eq!(r.steps.len(), 2);
    assert!(r.steps[0].converged);
    assert!(!r.steps[1].converged);
    assert_eq!(r.accepted_factor, Some(0.25));
    close(r.free_displacements[0], 0.04);
    assert_eq!(r.element_states, vec![vec![0.0]]);
    assert_eq!(s.accepted_element_states(), r.element_states);
    assert_eq!(factors.get(), 2);
    assert_eq!(solves.get(), 2);
    assert_eq!(
        r.steps.iter().map(|r| r.iterations as usize).sum::<usize>(),
        factors.get()
    );
    close(r.steps[1].force_norm, 4.8);
    close(r.steps[1].displacement_norm, 0.12);
    close(r.steps[1].energy_norm, 0.576);
    assert_eq!(s.linear_system().trial_displacement(), r.free_displacements);
    // First accepts observable plasticity; a later attempt preserves it.
    let accepted = run(&mut s, &[1.0], NewtonOptions::default());
    close(accepted.free_displacements[0], 0.4);
    close(accepted.element_states[0][0], 0.24);
    let failed = run(&mut s, &[1.25, 2.0], options);
    assert_eq!(failed.status, Status::NotConverged);
    assert_eq!(failed.steps.len(), 1);
    assert_eq!(failed.free_displacements, accepted.free_displacements);
    assert_eq!(failed.element_states, accepted.element_states);
    assert_eq!(failed.accepted_factor, Some(1.0));
}

#[test]
fn failure_before_any_accepted_step_preserves_initial_displacement() {
    let mut s = ground(16.0);
    let r = run(
        &mut s,
        &[1.0],
        NewtonOptions {
            max_iterations: 1,
            ..NewtonOptions::default()
        },
    );
    assert_eq!(r.status, Status::NotConverged);
    assert_eq!(r.accepted_factor, None);
    assert_eq!(r.free_displacements, vec![0.0]);
    assert_eq!(r.element_states, vec![vec![]]);
}

#[test]
fn multiple_absolute_factors_finish_at_total_load_and_allow_explicit_unloading() {
    let mut s = ground(16.0);
    let r = run(&mut s, &[0.25, 0.5, 1.0], NewtonOptions::default());
    assert_eq!(r.status, Status::Completed);
    assert_eq!(r.steps.len(), 3);
    assert_eq!(
        r.steps.iter().map(|r| r.iterations).collect::<Vec<_>>(),
        vec![1, 1, 2]
    );
    close(r.free_displacements[0], 0.4);
    assert_eq!(r.accepted_factor, Some(1.0));
    let r = run(
        &mut s,
        &[1.0, 0.5, 0.0, -0.5, -1.0],
        NewtonOptions::default(),
    );
    assert_eq!(r.status, Status::Completed);
    close(r.free_displacements[0], -0.4);
}

#[test]
fn force_displacement_energy_independent_with_explicit_scales() {
    // First correction u=.16: F=11.2, R=4.8, Δu=.16, |Δu R|=.768.
    for (force, disp, energy, scale, tolerance, norm) in [
        (true, false, false, 2.0, 2.40000000001, 4.8),
        (false, true, false, 2.0, 0.08000000001, 0.16),
        (false, false, true, 2.0, 0.38400000001, 0.768),
    ] {
        let options = NewtonOptions {
            max_iterations: 1,
            force_tolerance: force.then_some(tolerance),
            displacement_tolerance: disp.then_some(tolerance),
            energy_tolerance: energy.then_some(tolerance),
            force_scale: scale,
            displacement_scale: scale,
            energy_scale: scale,
        };
        let r = run(&mut ground(16.0), &[1.0], options);
        assert_eq!(r.status, Status::Completed);
        let measured = if force {
            r.steps[0].force_norm
        } else if disp {
            r.steps[0].displacement_norm
        } else {
            r.steps[0].energy_norm
        };
        close(measured, norm);
        let strict = NewtonOptions {
            force_tolerance: force.then_some(tolerance * 0.99),
            displacement_tolerance: disp.then_some(tolerance * 0.99),
            energy_tolerance: energy.then_some(tolerance * 0.99),
            ..options
        };
        assert_eq!(
            run(&mut ground(16.0), &[1.0], strict).status,
            Status::NotConverged
        );
    }
}

#[test]
fn all_active_tolerances_must_pass_and_reports_use_last_correction() {
    let base = NewtonOptions {
        max_iterations: 1,
        force_tolerance: Some(10.0),
        displacement_tolerance: Some(1.0),
        energy_tolerance: Some(1.0),
        ..NewtonOptions::default()
    };
    for options in [
        NewtonOptions {
            force_tolerance: Some(1e-10),
            ..base
        },
        NewtonOptions {
            displacement_tolerance: Some(1e-10),
            ..base
        },
        NewtonOptions {
            energy_tolerance: Some(1e-10),
            ..base
        },
    ] {
        assert_eq!(
            run(&mut ground(16.0), &[1.0], options).status,
            Status::NotConverged
        );
    }
    let r = run(
        &mut ground(16.0),
        &[1.0],
        NewtonOptions {
            max_iterations: 3,
            force_tolerance: Some(1e-10),
            displacement_tolerance: Some(1e-10),
            energy_tolerance: Some(1e-10),
            ..base
        },
    );
    assert_eq!(r.status, Status::Completed);
    assert_eq!(r.steps[0].iterations, 3);
}

#[test]
fn singular_tangent_names_node_and_dof_and_restores_trial() {
    let m = model(1, -20.0);
    let gap = Arc::new(
        MultilinearElastic::new(
            vec![(-1.25, -100.0), (-0.25, 0.0), (0.0, 0.0)],
            MultilinearMode::CompressionOnly,
        )
        .unwrap(),
    );
    let mut linear = LinearSystem::from_model(CountingBackend::default(), &m, &[], &[]).unwrap();
    let mut out = SymMatrix::zeros(1).unwrap();
    linear.tangent(&[0.3], &mut out).unwrap();
    let mut s = NewtonSystem::new(linear, vec![binding(&m, &[1], gap)]).unwrap();
    let e = StaticNewton::run(&mut s, &case(), &[1.0], NewtonOptions::default()).unwrap_err();
    assert_eq!(e.code, "ENG-E-0006");
    assert!(e.message.contains("Node 1, DOF Ux"));
    assert!(e
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Node(NodeId(1)))));
    assert!(e
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Dof(DofIndex(0)))));
    assert_eq!(s.linear_system().accepted_displacement(), &[0.0]);
    assert_eq!(s.linear_system().trial_displacement(), &[0.0]);
}

#[test]
fn invalid_options_factors_missing_case_and_overflows_are_catalog_errors() {
    let default = NewtonOptions::default();
    for options in [
        NewtonOptions {
            max_iterations: 0,
            ..default
        },
        NewtonOptions {
            force_tolerance: None,
            ..default
        },
        NewtonOptions {
            force_tolerance: Some(0.0),
            ..default
        },
        NewtonOptions {
            displacement_tolerance: Some(-1.0),
            ..default
        },
        NewtonOptions {
            energy_scale: 0.0,
            ..default
        },
    ] {
        assert_eq!(
            StaticNewton::run(&mut ground(16.0), &case(), &[1.0], options)
                .unwrap_err()
                .code,
            "ENG-E-0010"
        );
    }
    for options in [
        NewtonOptions {
            force_tolerance: Some(Real::NAN),
            ..default
        },
        NewtonOptions {
            displacement_scale: Real::INFINITY,
            ..default
        },
    ] {
        assert_eq!(
            StaticNewton::run(&mut ground(16.0), &case(), &[1.0], options)
                .unwrap_err()
                .code,
            "ENG-E-0008"
        );
    }
    assert_eq!(
        StaticNewton::run(&mut ground(16.0), &case(), &[], default)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        StaticNewton::run(
            &mut ground(16.0),
            &CaseRef("absent".into()),
            &[1.0],
            default
        )
        .unwrap_err()
        .code,
        "ENG-E-0003"
    );
    for factor in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY, Real::MAX] {
        let mut s = ground(16.0);
        assert_eq!(
            StaticNewton::run(&mut s, &case(), &[0.25, factor], default)
                .unwrap_err()
                .code,
            "ENG-E-0008"
        );
        assert_eq!(s.accepted_factor(), None);
        assert_eq!(s.linear_system().accepted_displacement(), &[0.0]);
    }
    // An inaccurate backend returning infinity cannot accept a step.
    let m = model(1, 16.0);
    let backend = CountingBackend {
        solution_scale: Real::INFINITY,
        ..CountingBackend::default()
    };
    let mut s = NewtonSystem::new(
        LinearSystem::from_model(backend, &m, &[], &[]).unwrap(),
        vec![binding(&m, &[1], bilinear(100.0))],
    )
    .unwrap();
    assert_eq!(
        StaticNewton::run(&mut s, &case(), &[1.0], default)
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    assert_eq!(s.linear_system().accepted_displacement(), &[0.0]);
}

#[test]
fn full_newton_cycle_exhausts_exact_budget_without_changing_load_factor() {
    let m = model(1, 16.0);
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let solves = backend.solves.clone();
    let mut s = NewtonSystem::new(
        LinearSystem::from_model(backend, &m, &[], &[]).unwrap(),
        vec![binding(&m, &[1], bilinear(100.0))],
    )
    .unwrap();
    let accepted = run(&mut s, &[1.0], NewtonOptions::default());
    // From u=.4, unloading directly to zero oscillates between ±.4 with K=20.
    // The user's list contains no intermediate steps: failure is reported.
    let before = factors.get();
    let r = run(
        &mut s,
        &[0.0, -1.0],
        NewtonOptions {
            max_iterations: 5,
            ..NewtonOptions::default()
        },
    );
    assert_eq!(r.status, Status::NotConverged);
    assert_eq!(r.steps.len(), 1);
    assert_eq!(r.steps[0].iterations, 5);
    assert_eq!(r.steps[0].load_factor, 0.0);
    assert_eq!(factors.get() - before, 5);
    assert_eq!(solves.get(), factors.get());
    assert_eq!(r.free_displacements, accepted.free_displacements);
    assert_eq!(r.accepted_factor, Some(1.0));
}

struct NonfiniteHistory;
impl UniaxialLaw for NonfiniteHistory {
    fn kind(&self) -> &'static str {
        "test.nonfinite_history"
    }
    fn state_len(&self) -> usize {
        1
    }
    fn initial_state(&self) -> Vec<Real> {
        vec![0.0]
    }
    fn initial_tangent(&self) -> Real {
        100.0
    }
    fn respond(&self, u: Real, _state: &[Real]) -> EResult<UniaxialResponse> {
        Ok(UniaxialResponse {
            stress: 100.0 * u,
            tangent: 100.0,
            trial_state: vec![if u > 0.1 { Real::NAN } else { u }],
        })
    }
}

#[test]
fn element_error_after_accepted_step_preserves_nonempty_history() {
    let m = model(1, 16.0);
    let mut s = NewtonSystem::new(
        LinearSystem::from_model(CountingBackend::default(), &m, &[], &[]).unwrap(),
        vec![binding(&m, &[1], Arc::new(NonfiniteHistory))],
    )
    .unwrap();
    let e = StaticNewton::run(&mut s, &case(), &[0.25, 1.0], NewtonOptions::default()).unwrap_err();
    assert_eq!(e.code, "ENG-E-0008");
    assert_eq!(s.accepted_factor(), Some(0.25));
    close(s.linear_system().accepted_displacement()[0], 0.04);
    close(s.accepted_element_states()[0][0], 0.04);
    assert_eq!(
        s.linear_system().trial_displacement(),
        s.linear_system().accepted_displacement()
    );
}

#[test]
fn tolerance_equality_is_inclusive_for_each_criterion() {
    // Backend at 1/2: load 8, K=2, Δu=2, R=4 and |ΔuR|=8, all exact.
    for (force, disp, energy) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let m = model(1, 8.0);
        let backend = CountingBackend {
            solution_scale: 0.5,
            ..CountingBackend::default()
        };
        let mut s = NewtonSystem::new(
            LinearSystem::from_model(backend, &m, &[contribution(&[0], vec![2.0])], &[]).unwrap(),
            vec![],
        )
        .unwrap();
        let r = run(
            &mut s,
            &[1.0],
            NewtonOptions {
                max_iterations: 1,
                force_tolerance: force.then_some(2.0),
                displacement_tolerance: disp.then_some(1.0),
                energy_tolerance: energy.then_some(4.0),
                force_scale: 2.0,
                displacement_scale: 2.0,
                energy_scale: 2.0,
            },
        );
        assert_eq!(r.status, Status::Completed);
    }
}

#[test]
fn element_binding_rejects_invalid_locations_and_context_before_solving() {
    let m = model(2, 16.0);
    let make = |location: Vec<DofIndex>, coords: &[[Real; 3]]| {
        let spring = NonlinearSpring::new(
            Dim::D2,
            SpringConnection::BetweenNodes,
            bilinear(100.0),
            &[Dof::Ux],
            SpringAxes::Global,
        )
        .unwrap();
        NewtonElement::new(
            Box::new(spring),
            location,
            &ElementCtx {
                dim: Dim::D2,
                coords,
                params: &ElementParams::default(),
            },
        )
    };
    let coords = [[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    assert_eq!(
        make(vec![DofIndex(0)], &coords).err().unwrap().code,
        "ENG-E-0010"
    );
    assert_eq!(
        make(vec![DofIndex(0), DofIndex(1)], &[[Real::NAN; 3]; 2])
            .err()
            .unwrap()
            .code,
        "ENG-E-0008"
    );
    for (indices, coords) in [
        (vec![DofIndex(0), DofIndex(2)], coords),
        (vec![DofIndex(1), DofIndex(0)], coords),
    ] {
        let element = make(indices, &coords).unwrap();
        let linear = LinearSystem::from_model(CountingBackend::default(), &m, &[], &[]).unwrap();
        assert_eq!(
            NewtonSystem::new(linear, vec![element]).err().unwrap().code,
            "ENG-E-0010"
        );
    }
}
