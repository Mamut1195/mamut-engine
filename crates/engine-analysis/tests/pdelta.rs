//! Soluciones analíticas propias; fórmulas y tolerancias en ENG-ANL-003.md.
#[path = "../../engine-assembly/tests/fixtures/counting.rs"]
mod counting;
use counting::CountingBackend;
use engine_analysis::{
    pdelta::{geometric_producer, PDelta, PDeltaOptions, PDeltaOutcome, PDeltaResult},
    StaticLinear,
};
use engine_assembly::{
    DofMap, ElementContribution, GeometricProducer, LinearSystem, PrescribedDof,
};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, SystemView};
use engine_core::{Dim, Dof, DofIndex, EResult, EngineError, EntityRef, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_elements_truss::LinearTruss;
use engine_linalg::{
    DMat, FactorInfo, GeneralizedEigen, LinAlgBackend, ReferenceBackend, SymFactorization,
    SymMatrix,
};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
use std::{cell::Cell, rc::Rc, sync::Arc};

const E: f64 = 200.0;
const A: f64 = 2.0;
const I: f64 = 3.0;
const L: f64 = 4.0;
const H: f64 = 0.1;

fn options() -> PDeltaOptions {
    PDeltaOptions {
        tolerance: 1e-10,
        max_iterations: 2000,
        displacement_scale: 0.01,
        rotation_scale: 0.01,
    }
}
fn case(name: &str) -> CaseRef {
    CaseRef(name.into())
}
fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}; tol={tolerance}"
    );
}
fn load(m: &mut Model, name: &str, node: u32, dof: Dof, value: f64) {
    m.cases.push(LoadCase {
        name: name.into(),
        loads: vec![NodalLoad {
            node: NodeId(node),
            values: vec![DofValue { dof, value }],
        }],
    });
}
fn support(m: &mut Model, id: u32, dofs: &[Dof]) {
    m.supports.push(Support {
        id: format!("s{id}"),
        node: NodeId(id),
        restrained: dofs.to_vec(),
    });
}
fn member(
    m: &Model,
    element: Arc<dyn Element>,
    a: u32,
    b: u32,
) -> (ElementContribution, GeometricProducer) {
    let map = DofMap::new(&m.nodes).unwrap();
    let coords = [
        map.coordinates(NodeId(a)).unwrap(),
        map.coordinates(NodeId(b)).unwrap(),
    ];
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let location: Vec<_> = [a, b]
        .into_iter()
        .flat_map(|id| {
            element
                .dofs_per_node()
                .iter()
                .map(|&d| map.global_index(NodeId(id), d).unwrap())
                .collect::<Vec<_>>()
        })
        .collect();
    let stiffness = element
        .respond(&ctx, &vec![0.0; location.len()], &[])
        .unwrap()
        .tangent;
    let producer = geometric_producer(element, &ctx, location.clone(), vec![]).unwrap();
    (
        ElementContribution {
            location,
            stiffness,
            mass_lumped: None,
            mass_consistent: None,
            geometric: None,
            loads: vec![],
        },
        producer,
    )
}
fn cantilever<B: LinAlgBackend>(
    backend: B,
    mesh: u32,
    p: f64,
    prescribed: &[PrescribedDof],
) -> LinearSystem<B> {
    let mut m = Model::new();
    for i in 0..=mesh {
        m.nodes.push(Node::new(
            NodeId(i + 1),
            Dim::D2,
            [L * i as f64 / mesh as f64, 0.0, 0.0],
        ));
    }
    support(&mut m, 1, Dim::D2.dofs());
    load(&mut m, "G", mesh + 1, Dof::Ux, -p);
    load(&mut m, "H", mesh + 1, Dof::Uy, H);
    let element: Arc<dyn Element> =
        Arc::new(EulerFrame2D::new(E, A, I, 0.0, MomentReleases::default()).unwrap());
    let (elements, producers): (Vec<_>, Vec<_>) = (1..=mesh)
        .map(|i| member(&m, element.clone(), i, i + 1))
        .unzip();
    let mut s = LinearSystem::from_model(backend, &m, &elements, prescribed).unwrap();
    s.set_geometric_producers(producers).unwrap();
    s
}
fn run<B: LinAlgBackend>(s: &mut LinearSystem<B>) -> PDeltaResult {
    PDelta::run(s, &(case("G"), 1.0), &(case("H"), 1.0), options()).unwrap()
}
fn tip(r: &PDeltaResult) -> f64 {
    let u = r.global_displacements.as_ref().unwrap();
    u[u.len() - 2]
}
fn pcr() -> f64 {
    std::f64::consts::PI.powi(2) * E * I / (4.0 * L * L)
}
fn linear_tip() -> f64 {
    H * L.powi(3) / (3.0 * E * I)
}
fn exact_compression(ratio: f64) -> f64 {
    let z = std::f64::consts::PI * ratio.sqrt() / 2.0;
    linear_tip() * 3.0 * (z.tan() - z) / z.powi(3)
}

#[test]
fn near_euler_exact_solution_mesh_refinement_and_declared_amplification_tolerance() {
    // EI v''''+P v''=0, v(0)=v'(0)=0, v''(L)=0,
    // EI v'''(L)+P v'(L)=-H => v(L)=H/P [tan(kL)/k-L].
    // 1/(1-r) es aproximado; el coeficiente del polo exacto es 96/π⁴.
    for ratio in [0.9, 0.95] {
        let exact = exact_compression(ratio);
        let mut previous_error = f64::INFINITY;
        for mesh in [1, 2, 4, 8, 16, 32] {
            let mut s = cantilever(ReferenceBackend, mesh, ratio * pcr(), &[]);
            let r = run(&mut s);
            assert_eq!(r.outcome, PDeltaOutcome::Completed);
            assert_eq!(r.inertia, (3 * mesh as usize, 0, 0));
            let error = (tip(&r) / exact - 1.0).abs();
            assert!(
                error < previous_error,
                "r={ratio}, mesh={mesh}, error={error}, previous={previous_error}"
            );
            previous_error = error;
            if mesh == 32 {
                assert!(error < 5e-7, "{error}");
                let approximate = linear_tip() / (1.0 - ratio);
                assert!((tip(&r) / approximate - 1.0).abs() < 0.02);
            }
            let last = r.iterations.last().unwrap();
            assert!(last.relative_free_residual <= options().tolerance);
            assert!(last.relative_displacement_change <= options().tolerance);
            // Cada miembro recibe el mismo axial N=-P de la gravedad recuperada.
            let dx = L / mesh as f64;
            for i in 0..mesh as usize {
                let u = &r.gravity.global_displacements;
                close(
                    E * A / dx * (u[(i + 1) * 3] - u[i * 3]),
                    -ratio * pcr(),
                    1e-10,
                );
            }
        }
    }
}

#[test]
fn supercritical_returns_unstable_with_inertia_counts() {
    for (ratio, count) in [(1.01, 1), (1.2, 1), (10.0, 2)] {
        let mut s = cantilever(ReferenceBackend, 16, ratio * pcr(), &[]);
        let r = run(&mut s);
        assert_eq!(
            r.outcome,
            PDeltaOutcome::Unstable {
                negative_eigenvalues: count
            }
        );
        assert_eq!(r.inertia.1, count);
        assert!(r.iterations.is_empty());
        assert!(r.global_displacements.is_none());
        assert!(r.reactions.is_empty());
    }
    // El elemento cúbico único sobrestima Pcr; el refinamiento resuelve el cruce.
    assert_eq!(
        run(&mut cantilever(ReferenceBackend, 1, 1.001 * pcr(), &[])).outcome,
        PDeltaOutcome::NotConverged
    );
    assert!(matches!(
        run(&mut cantilever(ReferenceBackend, 8, 1.001 * pcr(), &[])).outcome,
        PDeltaOutcome::Unstable {
            negative_eigenvalues: 1
        }
    ));
}

#[test]
fn zero_axial_reproduces_static_linear_including_prescribed_values_and_reactions() {
    // Traslación rígida de la base: no genera axiales de gravedad; prueba Kg_fp
    // y prescritos no escalados al comparar con static.linear lateral.
    let prescribed = [PrescribedDof {
        node: NodeId(1),
        dof: Dof::Uy,
        value: 0.003,
    }];
    let mut s = cantilever(ReferenceBackend, 4, 0.0, &prescribed);
    let linear = StaticLinear::run(&mut s, &[(case("H"), 2.0)])
        .unwrap()
        .cases
        .remove(0);
    let r = PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), 2.0), options()).unwrap();
    assert_eq!(r.outcome, PDeltaOutcome::Completed);
    for (a, b) in r
        .global_displacements
        .as_ref()
        .unwrap()
        .iter()
        .zip(&linear.global_displacements)
    {
        close(*a, *b, 1e-12);
    }
    for (a, b) in r.reactions.iter().zip(&linear.reactions) {
        assert_eq!((a.node, a.dof), (b.node, b.dof));
        close(a.value, b.value, 1e-10);
    }
}

#[test]
fn tension_positive_kg_stiffens_and_matches_closed_form() {
    // Continuación hiperbólica: v(L)=H/T [L-tanh(kL)/k], k²=T/EI.
    let ratio = 0.5;
    let t = ratio * pcr();
    let z = (t / (E * I)).sqrt() * L;
    let exact = H / t * (L - z.tanh() * L / z);
    let mut s = cantilever(ReferenceBackend, 16, -t, &[]);
    let r = run(&mut s);
    assert_eq!(r.outcome, PDeltaOutcome::Completed);
    assert!(tip(&r) < linear_tip());
    assert!((tip(&r) / exact - 1.0).abs() < 2e-7);
    let mut kg = SymMatrix::zeros(s.n_free_dofs()).unwrap();
    s.geometric(&r.gravity.free_displacements, &mut kg).unwrap();
    let u = r.iterations.last().unwrap().free_displacements.as_slice();
    let energy: f64 = (0..u.len())
        .flat_map(|i| (0..u.len()).map(move |j| (i, j)))
        .map(|(i, j)| u[i] * kg.dense().get(i, j).unwrap() * u[j])
        .sum();
    assert!(energy > 0.0);
}

#[test]
fn existing_truss_producers_recover_gravity_axials_analytically() {
    let mut m = Model::new();
    for (i, xyz) in [[-4.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 3.0, 0.0]]
        .into_iter()
        .enumerate()
    {
        let mut n = Node::new(NodeId(i as u32 + 1), Dim::D2, xyz);
        n.dofs = vec![Dof::Ux, Dof::Uy];
        m.nodes.push(n);
    }
    support(&mut m, 1, &[Dof::Ux, Dof::Uy]);
    support(&mut m, 2, &[Dof::Ux, Dof::Uy]);
    let p = 40.0;
    let h = 10.0;
    load(&mut m, "G", 3, Dof::Uy, -p);
    load(&mut m, "H", 3, Dof::Ux, h);
    let bar = LinearTruss::new(Dim::D2, E, A, 0.0).unwrap();
    let element: Arc<dyn Element> = Arc::new(bar);
    let map = DofMap::new(&m.nodes).unwrap();
    let mut elements = Vec::new();
    let mut producers = Vec::new();
    for a in [1, 2] {
        let (contribution, _) = member(&m, element.clone(), a, 3);
        let coords = [
            map.coordinates(NodeId(a)).unwrap(),
            map.coordinates(NodeId(3)).unwrap(),
        ];
        let params = ElementParams::default();
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        producers.push(
            engine_elements_truss::geometric_producer(bar, &ctx, contribution.location.clone())
                .unwrap(),
        );
        elements.push(contribution);
    }
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &elements, &[]).unwrap();
    s.set_geometric_producers(producers).unwrap();
    let r = run(&mut s);
    assert_eq!(r.outcome, PDeltaOutcome::Completed);
    // Kxx=2(EA/5)(4/5)², N=-P/(2*3/5), Kgxx=2N/5*(3/5)².
    let kxx = 2.0 * E * A / 5.0 * 0.8_f64.powi(2);
    let kgxx = -p * 0.6 / 5.0;
    close(
        r.global_displacements.as_ref().unwrap()[4],
        h / (kxx + kgxx),
        1e-9,
    );
    close(r.global_displacements.as_ref().unwrap()[5], 0.0, 1e-12);
}

fn scalar<B: LinAlgBackend>(backend: B, geometric: f64, prescribed: f64) -> LinearSystem<B> {
    let mut m = Model::new();
    for i in 1..=2 {
        let mut n = Node::new(NodeId(i), Dim::D2, [i as f64, 0.0, 0.0]);
        n.dofs = vec![Dof::Ux];
        m.nodes.push(n);
    }
    support(&mut m, 1, &[Dof::Ux]);
    load(&mut m, "G", 2, Dof::Ux, -25.0);
    load(&mut m, "H", 2, Dof::Ux, 10.0);
    let location = vec![DofIndex(0), DofIndex(1)];
    let element = ElementContribution {
        location: location.clone(),
        stiffness: DMat::new(2, 2, vec![100.0, -100.0, -100.0, 100.0]).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    };
    let mut s = LinearSystem::from_model(
        backend,
        &m,
        &[element],
        &[PrescribedDof {
            node: NodeId(1),
            dof: Dof::Ux,
            value: prescribed,
        }],
    )
    .unwrap();
    s.set_geometric_producers(vec![GeometricProducer {
        location,
        eval: Box::new(move |_| {
            DMat::new(2, 2, vec![geometric, -geometric, -geometric, geometric])
        }),
    }])
    .unwrap();
    s
}

#[test]
fn fixed_point_iteration_count_history_residual_and_no_hidden_retry() {
    // k=100, kg=-25: u_i=(10/75)(1-(1/4)^i). Error/r0=(1/4)^i.
    let backend = CountingBackend::default();
    let factors = backend.factorizations.clone();
    let solves = backend.solves.clone();
    let mut s = scalar(backend, -25.0, 0.0);
    s.accept(&[0.7]).unwrap();
    s.internal_force(&[0.8], &mut [0.0]).unwrap();
    let settings = PDeltaOptions {
        tolerance: 1e-6,
        max_iterations: 20,
        displacement_scale: 1.0,
        rotation_scale: 1.0,
    };
    let r = PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), 1.0), settings).unwrap();
    assert_eq!(r.outcome, PDeltaOutcome::Completed);
    assert_eq!(r.iterations.len(), 10);
    assert_eq!(factors.get(), 3); // gravedad, inercia total, K del punto fijo
    assert_eq!(solves.get(), 1 + r.iterations.len());
    assert_eq!(s.accepted_displacement(), &[0.7]);
    assert_eq!(s.trial_displacement(), &[0.8]);
    for (i, report) in r.iterations.iter().enumerate() {
        let step = i + 1;
        assert_eq!(report.iteration, step);
        close(
            report.free_displacements[0],
            10.0 / 75.0 * (1.0 - 0.25_f64.powi(step as i32)),
            1e-15,
        );
        close(
            report.relative_free_residual,
            0.25_f64.powi(step as i32),
            1e-15,
        );
        close(
            report.relative_displacement_change,
            0.1 * 0.25_f64.powi(i as i32),
            1e-15,
        );
    }
    let limited = PDelta::run(
        &mut s,
        &(case("G"), 1.0),
        &(case("H"), 1.0),
        PDeltaOptions {
            max_iterations: 3,
            ..settings
        },
    )
    .unwrap();
    assert_eq!(limited.outcome, PDeltaOutcome::NotConverged);
    assert_eq!(limited.iterations.len(), 3);
    assert_eq!(solves.get(), 15);
    assert_eq!(factors.get(), 6);
    assert_eq!(s.accepted_displacement(), &[0.7]);
    assert_eq!(s.trial_displacement(), &[0.8]);
}

#[test]
fn geometric_prescribed_shift_and_reactions_use_total_stiffness() {
    // (100-25)(u2-s)=10 => u2=s+10/75, R1=-10.
    let mut s = scalar(ReferenceBackend, -25.0, 0.03);
    let r = run(&mut s);
    assert_eq!(r.outcome, PDeltaOutcome::Completed);
    let u = r.global_displacements.as_ref().unwrap();
    close(u[0], 0.03, 1e-15);
    close(u[1], 0.03 + 10.0 / 75.0, 1e-10);
    close(r.reactions[0].value, -10.0, 1e-8);
}

#[test]
fn stable_tension_can_reach_limit_without_retrying_or_accepting() {
    // Punto fijo u_new=0.1-2u: radio espectral 2, aunque K+Kg=300>0.
    let mut s = scalar(ReferenceBackend, 200.0, 0.0);
    let r = PDelta::run(
        &mut s,
        &(case("G"), 1.0),
        &(case("H"), 1.0),
        PDeltaOptions {
            max_iterations: 4,
            ..options()
        },
    )
    .unwrap();
    assert_eq!(r.inertia, (1, 0, 0));
    assert_eq!(r.outcome, PDeltaOutcome::NotConverged);
    assert_eq!(r.iterations.len(), 4);
    assert_eq!(s.accepted_displacement(), &[0.0]);
}

fn code<T: std::fmt::Debug>(r: EResult<T>, expected: &str) {
    assert_eq!(r.unwrap_err().code, expected);
}
#[test]
fn invalid_parameters_cases_finiteness_and_mechanism_use_catalog() {
    let mut s = scalar(ReferenceBackend, -25.0, 0.0);
    for bad in [
        PDeltaOptions {
            tolerance: 0.0,
            ..options()
        },
        PDeltaOptions {
            tolerance: -1.0,
            ..options()
        },
        PDeltaOptions {
            tolerance: 1.0,
            ..options()
        },
        PDeltaOptions {
            max_iterations: 0,
            ..options()
        },
        PDeltaOptions {
            displacement_scale: 0.0,
            ..options()
        },
        PDeltaOptions {
            rotation_scale: -1.0,
            ..options()
        },
    ] {
        code(
            PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), 1.0), bad),
            "ENG-E-0010",
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for bad in [
            PDeltaOptions {
                tolerance: value,
                ..options()
            },
            PDeltaOptions {
                displacement_scale: value,
                ..options()
            },
            PDeltaOptions {
                rotation_scale: value,
                ..options()
            },
        ] {
            code(
                PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), 1.0), bad),
                "ENG-E-0008",
            );
        }
        code(
            PDelta::run(&mut s, &(case("G"), value), &(case("H"), 1.0), options()),
            "ENG-E-0008",
        );
        code(
            PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), value), options()),
            "ENG-E-0008",
        );
    }
    code(
        PDelta::run(&mut s, &(case("absent"), 1.0), &(case("H"), 1.0), options()),
        "ENG-E-0003",
    );
    code(
        PDelta::run(&mut s, &(case("G"), 1.0), &(case("absent"), 1.0), options()),
        "ENG-E-0003",
    );
    code(
        PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), f64::MAX), options()),
        "ENG-E-0008",
    );
    code(
        PDelta::run(
            &mut s,
            &(case("G"), 1.0),
            &(case("H"), 1.0),
            PDeltaOptions {
                displacement_scale: f64::MIN_POSITIVE * 1e-10,
                ..options()
            },
        ),
        "ENG-E-0008",
    );
    let mut singular = scalar(ReferenceBackend, -100.0, 0.0);
    let error = PDelta::run(
        &mut singular,
        &(case("G"), 1.0),
        &(case("H"), 1.0),
        options(),
    )
    .unwrap_err();
    assert_eq!(error.code, "ENG-E-0006");
    assert!(error.message.contains("Ux"));
    assert!(error
        .entities
        .iter()
        .any(|e| matches!(e, EntityRef::Node(NodeId(2)))));
    // El mecanismo elástico de gravedad se detecta antes de evaluar Kg.
    let mut m = Model::new();
    let mut n = Node::new(NodeId(1), Dim::D2, [0.0; 3]);
    n.dofs = vec![Dof::Ux];
    m.nodes.push(n);
    load(&mut m, "G", 1, Dof::Ux, 0.0);
    load(&mut m, "H", 1, Dof::Ux, 1.0);
    let mut mechanism = LinearSystem::from_model(ReferenceBackend, &m, &[], &[]).unwrap();
    code(
        PDelta::run(
            &mut mechanism,
            &(case("G"), 1.0),
            &(case("H"), 1.0),
            options(),
        ),
        "ENG-E-0006",
    );
}

#[test]
fn nonfinite_geometric_evaluation_and_total_overflow_are_errors_without_state_changes() {
    let mut s = scalar(ReferenceBackend, 0.0, 0.0);
    s.set_geometric_producers(vec![GeometricProducer {
        location: vec![DofIndex(0), DofIndex(1)],
        eval: Box::new(|u| DMat::new(2, 2, vec![if u[1] == 0.0 { 0.0 } else { f64::NAN }; 4])),
    }])
    .unwrap();
    code(
        PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), 1.0), options()),
        "ENG-E-0008",
    );
    assert_eq!(s.accepted_displacement(), &[0.0]);
    assert_eq!(s.trial_displacement(), &[0.0]);
}

// El camino de estabilidad debe usar FactorInfo y nunca generalized_eigen.
struct InfoBackend {
    mode: u8,
    calls: Rc<Cell<usize>>,
}
struct InfoFactor {
    inner: Box<dyn SymFactorization>,
    info: FactorInfo,
}
impl SymFactorization for InfoFactor {
    fn info(&self) -> &FactorInfo {
        &self.info
    }
    fn solve_in_place(&self, b: &mut DMat) -> EResult<()> {
        self.inner.solve_in_place(b)
    }
}
impl LinAlgBackend for InfoBackend {
    fn name(&self) -> &'static str {
        "test.inertia"
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
    fn factorize(&mut self, a: &SymMatrix) -> EResult<Box<dyn SymFactorization>> {
        self.calls.set(self.calls.get() + 1);
        let inner = ReferenceBackend.factorize(a)?;
        let mut info = inner.info().clone();
        if self.calls.get() == 2 {
            info.inertia = match self.mode {
                0 => None,
                1 => Some((usize::MAX, 1, 0)),
                _ => Some((0, 1, 0)),
            };
        }
        Ok(Box::new(InfoFactor { inner, info }))
    }
    fn generalized_eigen(&mut self, _: &SymMatrix, _: &SymMatrix) -> EResult<GeneralizedEigen> {
        Err(EngineError::new(
            "ENG-E-0012",
            "No debe calcular modos para estabilidad.",
        ))
    }
}
#[test]
fn missing_inertia_is_explicit_and_invalid_inertia_is_rejected() {
    for (mode, expected) in [(0, "ENG-E-0013"), (1, "ENG-E-0012")] {
        let mut s = scalar(
            InfoBackend {
                mode,
                calls: Rc::new(Cell::new(0)),
            },
            -25.0,
            0.0,
        );
        code(
            PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), 1.0), options()),
            expected,
        );
    }
    let calls = Rc::new(Cell::new(0));
    let mut s = scalar(
        InfoBackend {
            mode: 2,
            calls: calls.clone(),
        },
        -25.0,
        0.0,
    );
    let r = run(&mut s);
    assert_eq!(
        r.outcome,
        PDeltaOutcome::Unstable {
            negative_eigenvalues: 1
        }
    );
    assert_eq!(calls.get(), 2);
}

#[test]
fn ldlt_two_by_two_block_inertia_reaches_unstable_result() {
    // K_ff=I, Kg_ff=[-1,2;2,-1] => total=[0,2;2,0], espectro ±2.
    // Ambos pivotes diagonales son cero: contar solo diagonales daría mal.
    let mut m = Model::new();
    let mut n = Node::new(NodeId(1), Dim::D2, [0.0; 3]);
    n.dofs = vec![Dof::Ux, Dof::Uy];
    m.nodes.push(n);
    load(&mut m, "G", 1, Dof::Ux, 1.0);
    load(&mut m, "H", 1, Dof::Uy, 1.0);
    let location = vec![DofIndex(0), DofIndex(1)];
    let c = ElementContribution {
        location: location.clone(),
        stiffness: DMat::new(2, 2, vec![1.0, 0.0, 0.0, 1.0]).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: vec![],
    };
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &[c], &[]).unwrap();
    s.set_geometric_producers(vec![GeometricProducer {
        location,
        eval: Box::new(|_| DMat::new(2, 2, vec![-1.0, 2.0, 2.0, -1.0])),
    }])
    .unwrap();
    let r = run(&mut s);
    assert_eq!(r.inertia, (1, 1, 0));
    assert_eq!(
        r.outcome,
        PDeltaOutcome::Unstable {
            negative_eigenvalues: 1
        }
    );
}

#[test]
fn producer_adapter_invalid_inputs_and_static_reference_guard() {
    let element: Arc<dyn Element> =
        Arc::new(EulerFrame2D::new(E, A, I, 0.0, MomentReleases::default()).unwrap());
    let params = ElementParams::default();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    assert_eq!(
        geometric_producer(element.clone(), &ctx, vec![], vec![])
            .err()
            .unwrap()
            .code,
        "ENG-E-0010"
    );
    assert_eq!(
        geometric_producer(element.clone(), &ctx, vec![DofIndex(0); 6], vec![])
            .err()
            .unwrap()
            .code,
        "ENG-E-0002"
    );
    assert_eq!(
        geometric_producer(element, &ctx, (0..6).map(DofIndex).collect(), vec![1.0])
            .err()
            .unwrap()
            .code,
        "ENG-E-0010"
    );
    let mut s = cantilever(ReferenceBackend, 1, 0.2 * pcr(), &[]);
    let g = s.solve(&case("G"), 1.0).unwrap();
    let mut reduced = SymMatrix::zeros(s.n_free_dofs()).unwrap();
    s.geometric(&g, &mut reduced).unwrap();
    let global = s.geometric_at(&g).unwrap();
    assert_eq!(global.rows(), 6);
    assert_eq!(
        global.to_dense().unwrap().get(4, 4),
        reduced.dense().get(1, 1)
    );
    // Una Kg estática no se puede reutilizar silenciosamente fuera de referencia.
    let mut m = Model::new();
    let mut n = Node::new(NodeId(1), Dim::D2, [0.0; 3]);
    n.dofs = vec![Dof::Ux];
    m.nodes.push(n);
    load(&mut m, "G", 1, Dof::Ux, 1.0);
    load(&mut m, "H", 1, Dof::Ux, 1.0);
    let c = ElementContribution {
        location: vec![DofIndex(0)],
        stiffness: DMat::new(1, 1, vec![1.0]).unwrap(),
        mass_lumped: None,
        mass_consistent: None,
        geometric: Some(DMat::new(1, 1, vec![0.0]).unwrap()),
        loads: vec![],
    };
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &[c], &[]).unwrap();
    code(
        PDelta::run(&mut s, &(case("G"), 1.0), &(case("H"), 1.0), options()),
        "ENG-E-0013",
    );
}
