//! Benchmarks lineales de derivación propia, embebidos y sin E/S para wasm.
//! Fórmulas, hipótesis y tolerancias: provenance/ENG-ANL-001.md.
#[path = "../../engine-assembly/tests/fixtures/counting.rs"]
mod counting;
use counting::CountingBackend;
use engine_analysis::{StaticLinear, StaticResults};
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem, PrescribedDof};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MemberLoad, SystemView};
use engine_core::math;
use engine_core::{Dim, Dof, DofIndex, EntityRef, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_elements_truss::LinearTruss;
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const E: f64 = 200.0;
const A: f64 = 2.0;
const I: f64 = 3.0;
const P: f64 = 12.0;
const L: f64 = 4.0;

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}
fn model(coords: &[[f64; 3]], dim: Dim, dofs: &[Dof]) -> Model {
    let mut m = Model::new();
    for (i, &xyz) in coords.iter().enumerate() {
        let mut n = Node::new(NodeId(i as u32 + 1), dim, xyz);
        n.dofs = dofs.to_vec();
        m.nodes.push(n);
    }
    m
}
fn support(m: &mut Model, node: u32, dofs: &[Dof]) {
    m.supports.push(Support {
        id: format!("s{node}"),
        node: NodeId(node),
        restrained: dofs.to_vec(),
    });
}
fn load_case(m: &mut Model, name: &str, loads: &[(u32, Dof, f64)]) {
    m.cases.push(LoadCase {
        name: name.into(),
        loads: loads
            .iter()
            .map(|&(n, dof, value)| NodalLoad {
                node: NodeId(n),
                values: vec![DofValue { dof, value }],
            })
            .collect(),
    });
}
fn cases(names: &[&str]) -> Vec<(CaseRef, f64)> {
    names.iter().map(|n| (CaseRef((*n).into()), 1.0)).collect()
}
fn contribution(
    m: &Model,
    element: &dyn Element,
    a: u32,
    b: u32,
    loads: &[(CaseRef, MemberLoad)],
) -> ElementContribution {
    let map = DofMap::new(&m.nodes).unwrap();
    let coords = [
        map.coordinates(NodeId(a)).unwrap(),
        map.coordinates(NodeId(b)).unwrap(),
    ];
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: m.nodes[0].dim,
        coords: &coords,
        params: &params,
    };
    let location: Vec<_> = [a, b]
        .into_iter()
        .flat_map(|n| {
            element
                .dofs_per_node()
                .iter()
                .map(|&d| map.global_index(NodeId(n), d).unwrap())
                .collect::<Vec<_>>()
        })
        .collect();
    ElementContribution {
        stiffness: element
            .respond(&ctx, &vec![0.0; location.len()], &[])
            .unwrap()
            .tangent,
        location,
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: loads
            .iter()
            .map(|(case, load)| ElementLoad {
                case: case.clone(),
                values: element.load_vector(&ctx, load).unwrap(),
            })
            .collect(),
    }
}
fn reaction(result: &engine_analysis::StaticCaseResult, node: u32, dof: Dof) -> f64 {
    result
        .reactions
        .iter()
        .find(|r| r.node == NodeId(node) && r.dof == dof)
        .unwrap()
        .value
}
fn balanced(results: &StaticResults) {
    for r in &results.cases {
        assert!(r.relative_free_residual < 1e-12);
        assert!(r.free_residual_norm < 1e-9);
        assert!(r.equilibrium.relative_residual < 1e-12);
        for v in &r.equilibrium.residual_force[..3] {
            close(*v, 0.0, 1e-9);
        }
        for v in r.equilibrium.residual_moment {
            close(v, 0.0, 1e-9);
        }
    }
}

#[test]
fn symmetric_two_bar_truss_cases_superposition_one_factorization() {
    // V=EA/(2L) Σ(d·u)². Para c=4/5,s=3/5:
    // Kxx=2(EA/5)c²=102.4; Kyy=2(EA/5)s²=57.6; Kxy=0.
    let mut m = model(
        &[[-4.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
        Dim::D2,
        &[Dof::Ux, Dof::Uy],
    );
    support(&mut m, 1, &[Dof::Ux, Dof::Uy]);
    support(&mut m, 2, &[Dof::Ux, Dof::Uy]);
    load_case(&mut m, "A", &[(3, Dof::Ux, 10.0)]);
    load_case(&mut m, "B", &[(3, Dof::Uy, -P)]);
    load_case(&mut m, "AB", &[(3, Dof::Ux, 10.0), (3, Dof::Uy, -P)]);
    let bar = LinearTruss::new(Dim::D2, E, A, 0.0).unwrap();
    let elements = [
        contribution(&m, &bar, 1, 3, &[]),
        contribution(&m, &bar, 2, 3, &[]),
    ];
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let solves = backend.solves.clone();
    let mut s = LinearSystem::from_model(backend, &m, &elements, &[]).unwrap();
    s.accept(&[0.7, 0.8]).unwrap();
    let r = StaticLinear::run(&mut s, &cases(&["A", "B", "AB"])).unwrap();
    assert_eq!(factors.get(), 1);
    assert_eq!(solves.get(), 3);
    assert_eq!(s.accepted_displacement(), &[0.7, 0.8]);
    assert_eq!(s.trial_displacement(), &[0.7, 0.8]);
    close(r.cases[0].free_displacements[0], 10.0 / 102.4, 1e-12);
    close(r.cases[1].free_displacements[1], -P / 57.6, 1e-12);
    for i in 0..6 {
        close(
            r.cases[0].global_displacements[i] + r.cases[1].global_displacements[i],
            r.cases[2].global_displacements[i],
            1e-12,
        );
    }
    for i in 0..4 {
        close(
            r.cases[0].reactions[i].value + r.cases[1].reactions[i].value,
            r.cases[2].reactions[i].value,
            1e-9,
        );
    }
    close(reaction(&r.cases[1], 1, Dof::Uy), P / 2.0, 1e-9);
    close(
        reaction(&r.cases[1], 1, Dof::Ux),
        P * 4.0 / (2.0 * 3.0),
        1e-9,
    );
    balanced(&r);
}

#[test]
fn bar_settlement_physical_support_loads_and_affine_superposition() {
    // EA/L=100,200; 300u2=200s+factor P. s=.03 da u2=.02.
    let mut m = model(
        &[[0.0; 3], [4.0, 0.0, 0.0], [6.0, 0.0, 0.0]],
        Dim::D2,
        &[Dof::Ux, Dof::Uy],
    );
    support(&mut m, 1, &[Dof::Ux, Dof::Uy]);
    support(&mut m, 2, &[Dof::Uy]);
    support(&mut m, 3, &[Dof::Ux, Dof::Uy]);
    load_case(&mut m, "S", &[]);
    load_case(&mut m, "A", &[(2, Dof::Ux, P), (1, Dof::Ux, 7.0)]);
    load_case(&mut m, "B", &[(2, Dof::Ux, 5.0)]);
    load_case(&mut m, "AB", &[(2, Dof::Ux, P + 5.0), (1, Dof::Ux, 7.0)]);
    let bar = LinearTruss::new(Dim::D2, E, A, 0.0).unwrap();
    let elements = [
        contribution(&m, &bar, 1, 2, &[]),
        contribution(&m, &bar, 2, 3, &[]),
    ];
    let prescribed = [PrescribedDof {
        node: NodeId(3),
        dof: Dof::Ux,
        value: 0.03,
    }];
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &elements, &prescribed).unwrap();
    let r = StaticLinear::run(&mut s, &cases(&["S", "A", "B", "AB"])).unwrap();
    close(r.cases[0].free_displacements[0], 0.02, 1e-12);
    close(reaction(&r.cases[0], 1, Dof::Ux), -2.0, 1e-9);
    close(reaction(&r.cases[0], 3, Dof::Ux), 2.0, 1e-9);
    close(
        reaction(&r.cases[1], 1, Dof::Ux),
        -100.0 * (P + 6.0) / 300.0 - 7.0,
        1e-9,
    );
    for i in 0..6 {
        close(
            r.cases[1].global_displacements[i] + r.cases[2].global_displacements[i]
                - r.cases[0].global_displacements[i],
            r.cases[3].global_displacements[i],
            1e-12,
        );
    }
    let scaled = StaticLinear::run(&mut s, &[(CaseRef("A".into()), 2.0)]).unwrap();
    close(scaled.cases[0].free_displacements[0], 0.1, 1e-12);
    close(scaled.cases[0].global_displacements[4], 0.03, 1e-12);
    close(scaled.cases[0].equilibrium.applied_force_sum[0], 38.0, 1e-9);
    balanced(&r);
    balanced(&scaled);
}

fn beam_model() -> (Model, Vec<ElementContribution>) {
    let mut m = model(&[[2.0, 3.0, 0.0], [6.0, 3.0, 0.0]], Dim::D2, Dim::D2.dofs());
    support(&mut m, 1, Dim::D2.dofs());
    load_case(&mut m, "P", &[(2, Dof::Uy, -P)]);
    load_case(&mut m, "q", &[]);
    let beam = EulerFrame2D::new(E, A, I, 0.0, MomentReleases::default()).unwrap();
    let e = contribution(
        &m,
        &beam,
        1,
        2,
        &[(
            CaseRef("q".into()),
            MemberLoad {
                force_per_length: [0.0, -6.0, 0.0],
                moment_per_length: [0.0; 3],
            },
        )],
    );
    (m, vec![e])
}

#[test]
fn frame_cantilever_tip_deflection_rotation_reaction_and_global_moments() {
    // EI v''=−P(L−x); integración con v(0)=v'(0)=0.
    let (m, e) = beam_model();
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &e, &[]).unwrap();
    let r = StaticLinear::run(&mut s, &cases(&["P", "q"])).unwrap();
    close(
        r.cases[0].global_displacements[4],
        -P * math::powi(L, 3) / (3.0 * E * I),
        1e-12,
    );
    close(
        r.cases[0].global_displacements[5],
        -P * math::powi(L, 2) / (2.0 * E * I),
        1e-12,
    );
    close(reaction(&r.cases[0], 1, Dof::Rz), P * L, 1e-9);
    close(reaction(&r.cases[0], 1, Dof::Uy), P, 1e-9);
    // Los pares solos no equilibran: el brazo de la fuerza cancela PL.
    close(r.cases[0].equilibrium.residual_force[5], P * L, 1e-9);
    close(r.cases[0].equilibrium.residual_moment[2], 0.0, 1e-9);
    close(
        r.cases[1].global_displacements[4],
        -6.0 * math::powi(L, 4) / (8.0 * E * I),
        1e-12,
    );
    close(
        r.cases[1].global_displacements[5],
        -6.0 * math::powi(L, 3) / (6.0 * E * I),
        1e-12,
    );
    close(reaction(&r.cases[1], 1, Dof::Rz), 6.0 * L * L / 2.0, 1e-9);
    close(r.cases[1].equilibrium.applied_force_sum[1], -6.0 * L, 1e-9);
    let shifted = StaticLinear::run_about(&mut s, &cases(&["P", "q"]), [-2.0, 1.0, 0.0]).unwrap();
    assert_eq!(shifted.cases[0].equilibrium.moment_point, [-2.0, 1.0, 0.0]);
    balanced(&r);
    balanced(&shifted);
}

#[test]
fn symmetric_portal_closed_form_lateral_equilibrium() {
    // La energía en Δ,θ,v se deriva en la procedencia; EA es finito.
    let h: f64 = 3.0;
    let span: f64 = 4.0;
    let ei = E * I;
    let beta = (12.0 * ei / math::powi(span, 2)) / (E * A / h + 24.0 * ei / math::powi(span, 3));
    let d = 4.0 * ei / h + 6.0 * ei / span - 12.0 * ei * beta / math::powi(span, 2);
    let delta =
        P / (24.0 * ei / math::powi(h, 3) - 72.0 * math::powi(ei, 2) / (math::powi(h, 4) * d));
    let theta = -6.0 * ei * delta / (math::powi(h, 2) * d);
    let v = -beta * theta;
    let mut m = model(
        &[[0.0; 3], [span, 0.0, 0.0], [0.0, h, 0.0], [span, h, 0.0]],
        Dim::D2,
        Dim::D2.dofs(),
    );
    support(&mut m, 1, Dim::D2.dofs());
    support(&mut m, 2, Dim::D2.dofs());
    load_case(&mut m, "H", &[(3, Dof::Ux, P / 2.0), (4, Dof::Ux, P / 2.0)]);
    let beam = EulerFrame2D::new(E, A, I, 0.0, MomentReleases::default()).unwrap();
    let elements = [(1, 3), (2, 4), (3, 4)].map(|(a, b)| contribution(&m, &beam, a, b, &[]));
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &elements, &[]).unwrap();
    let r = StaticLinear::run(&mut s, &cases(&["H"])).unwrap();
    for (n, y) in [(2, v), (3, -v)] {
        close(r.cases[0].global_displacements[3 * n], delta, 1e-12);
        close(r.cases[0].global_displacements[3 * n + 1], y, 1e-12);
        close(r.cases[0].global_displacements[3 * n + 2], theta, 1e-12);
    }
    close(reaction(&r.cases[0], 1, Dof::Ux), -P / 2.0, 1e-9);
    close(reaction(&r.cases[0], 2, Dof::Ux), -P / 2.0, 1e-9);
    close(reaction(&r.cases[0], 1, Dof::Uy), -E * A * v / h, 1e-9);
    close(r.cases[0].equilibrium.applied_force_sum[0], P, 1e-9);
    close(r.cases[0].equilibrium.reaction_sum[0], -P, 1e-9);
    balanced(&r);
}

#[test]
fn spatial_bar_all_components_and_nodal_couples() {
    let mut m = model(&[[1.0, 2.0, 3.0], [3.0, 5.0, 7.0]], Dim::D3, Dim::D3.dofs());
    support(&mut m, 1, Dim::D3.dofs());
    support(&mut m, 2, &[Dof::Ux, Dof::Uy, Dof::Rx, Dof::Ry, Dof::Rz]);
    load_case(
        &mut m,
        "P",
        &[
            (2, Dof::Uz, P),
            (1, Dof::Rx, 7.0),
            (1, Dof::Ry, -8.0),
            (2, Dof::Rz, 9.0),
        ],
    );
    m.nodes.reverse(); // Numeración por id, independiente del orden de entrada.
    let bar = LinearTruss::new(Dim::D3, E, A, 0.0).unwrap();
    let element = contribution(&m, &bar, 1, 2, &[]);
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &[element], &[]).unwrap();
    let r = StaticLinear::run_about(&mut s, &cases(&["P"]), [4.0, -2.0, 1.0]).unwrap();
    // L=√29, nz=4/L => Kzz=EA*16/L³.
    close(
        r.cases[0].free_displacements[0],
        P * math::powf(29.0_f64, 1.5) / (E * A * 16.0),
        1e-12,
    );
    for (axis, expected) in [(3, 7.0), (4, -8.0), (5, 9.0)] {
        close(
            r.cases[0].equilibrium.applied_force_sum[axis],
            expected,
            1e-9,
        );
        close(r.cases[0].equilibrium.reaction_sum[axis], -expected, 1e-9);
    }
    balanced(&r);
}

#[test]
fn unstable_truss_reports_node_and_global_dof() {
    let mut m = model(&[[0.0; 3], [L, 0.0, 0.0]], Dim::D2, &[Dof::Ux, Dof::Uy]);
    support(&mut m, 1, &[Dof::Ux, Dof::Uy]);
    load_case(&mut m, "P", &[(2, Dof::Uy, -P)]);
    let bar = LinearTruss::new(Dim::D2, E, A, 0.0).unwrap();
    let e = contribution(&m, &bar, 1, 2, &[]);
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &[e], &[]).unwrap();
    let err = StaticLinear::run(&mut s, &cases(&["P"])).unwrap_err();
    assert_eq!(err.code, "ENG-E-0006");
    assert!(err.message.contains("Nodo 2") && err.message.contains("GDL Uy"));
    assert!(err
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Node(NodeId(2)))));
    assert!(err
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Dof(DofIndex(3)))));
}

#[test]
fn invalid_inputs_fail_before_factorization_and_preserve_state() {
    let (m, e) = beam_model();
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let mut s = LinearSystem::from_model(backend, &m, &e, &[]).unwrap();
    s.accept(&[0.1, 0.2, 0.3]).unwrap();
    for (cases, code) in [
        (vec![], "ENG-E-0010"),
        (cases(&["P", "P"]), "ENG-E-0010"),
        (cases(&["P", "missing"]), "ENG-E-0003"),
        (vec![(CaseRef("P".into()), f64::NAN)], "ENG-E-0008"),
        (vec![(CaseRef("P".into()), f64::INFINITY)], "ENG-E-0008"),
        (vec![(CaseRef("P".into()), f64::NEG_INFINITY)], "ENG-E-0008"),
        (vec![(CaseRef("P".into()), f64::MAX)], "ENG-E-0008"),
    ] {
        assert_eq!(StaticLinear::run(&mut s, &cases).unwrap_err().code, code);
    }
    assert_eq!(
        StaticLinear::run_about(&mut s, &cases(&["P"]), [f64::NAN, 0.0, 0.0])
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    assert_eq!(factors.get(), 0);
    assert_eq!(s.accepted_displacement(), &[0.1, 0.2, 0.3]);
    assert_eq!(s.trial_displacement(), &[0.1, 0.2, 0.3]);
    let zero = StaticLinear::run(&mut s, &[(CaseRef("P".into()), 0.0)]).unwrap();
    assert_eq!(zero.cases[0].relative_free_residual, 0.0);
    assert_eq!(zero.cases[0].equilibrium.relative_residual, 0.0);
    let negative = StaticLinear::run(&mut s, &[(CaseRef("P".into()), -1.0)]).unwrap();
    close(
        negative.cases[0].global_displacements[4],
        P * math::powi(L, 3) / (3.0 * E * I),
        1e-12,
    );
    balanced(&negative);
}

#[test]
fn reports_detect_solver_error_and_moment_reference_translation() {
    let (m, e) = beam_model();
    let backend = CountingBackend {
        solution_scale: 0.5,
        ..Default::default()
    };
    let mut s = LinearSystem::from_model(backend, &m, &e, &[]).unwrap();
    let r = StaticLinear::run(&mut s, &cases(&["P"])).unwrap();
    let shifted = StaticLinear::run_about(&mut s, &cases(&["P"]), [2.0, 1.0, 0.0]).unwrap();
    // u/2 => R_y=P/2, R_m=PL/2; residual físico Fy=−P/2.
    close(r.cases[0].free_residual_norm, P / 2.0, 1e-9);
    close(r.cases[0].relative_free_residual, 0.5, 1e-12);
    close(r.cases[0].equilibrium.residual_force[1], -P / 2.0, 1e-9);
    close(r.cases[0].equilibrium.relative_residual, 1.0 / 3.0, 1e-12);
    let e0 = &r.cases[0].equilibrium;
    let e1 = &shifted.cases[0].equilibrium;
    // M(p)=M(0)−p×ΣF; el desplazamiento del punto sí cambia el residual.
    close(
        e1.residual_moment[2],
        e0.residual_moment[2] - 2.0 * e0.residual_force[1] + e0.residual_force[0],
        1e-9,
    );
}
