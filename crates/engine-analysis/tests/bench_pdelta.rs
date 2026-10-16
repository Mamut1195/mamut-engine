//! ENG-BENCH-002: benchmarks publicados de P-Delta/P-δ (fuentes en provenance/ENG-BENCH-002.md).
//! Sistema de unidades consistente kip, in (como la referencia).
use engine_analysis::pdelta::{
    geometric_producer, PDelta, PDeltaOptions, PDeltaOutcome, PDeltaResult,
};
use engine_assembly::{DofMap, ElementContribution, GeometricProducer, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams};
use engine_core::math;
use engine_core::{Dim, Dof, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
use std::sync::Arc;

// W14x48 eje fuerte (AISC 360-16 Comm. C2, Figs. C-C2.2/C-C2.3).
const E: f64 = 29_000.0; // ksi
const A: f64 = 14.1; // in²
const I: f64 = 484.0; // in⁴
const L: f64 = 28.0 * 12.0; // in
const W: f64 = 0.200 / 12.0; // kip/in (caso 1)
const H: f64 = 1.0; // kip (caso 2)
/// Tolerancias declaradas antes de correr: AISC 360-16 Comm. C2.1, p. 16.1-291.
const TOL_M: f64 = 0.03;
const TOL_D: f64 = 0.05;
/// Tolerancia frente a la solución cerrada de la ecuación diferencial con la malla declarada.
const TOL_EXACT: f64 = 1e-3;
/// Malla declarada para el benchmark (elementos por miembro).
const MESH: u32 = 8;

fn options() -> PDeltaOptions {
    PDeltaOptions {
        tolerance: 1e-10,
        max_iterations: 5000,
        displacement_scale: 1.0,
        rotation_scale: 0.01,
    }
}
fn ei() -> f64 {
    E * I
}
fn pin(m: &mut Model, id: u32, dofs: &[Dof]) {
    m.supports.push(Support {
        id: format!("s{id}"),
        node: NodeId(id),
        restrained: dofs.to_vec(),
    });
}
fn case(name: &str, loads: Vec<(u32, Dof, f64)>) -> LoadCase {
    LoadCase {
        name: name.into(),
        loads: loads
            .into_iter()
            .map(|(n, dof, value)| NodalLoad {
                node: NodeId(n),
                values: vec![DofValue { dof, value }],
            })
            .collect(),
    }
}
fn member(
    m: &Model,
    el: Arc<dyn Element>,
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
            el.dofs_per_node()
                .iter()
                .map(|&d| map.global_index(NodeId(id), d).unwrap())
                .collect::<Vec<_>>()
        })
        .collect();
    let stiffness = el
        .respond(&ctx, &vec![0.0; location.len()], &[])
        .unwrap()
        .tangent;
    let producer = geometric_producer(el, &ctx, location.clone(), vec![]).unwrap();
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
/// Barra en x de n elementos; devuelve el resultado P-Delta (G = axial, H = lateral).
fn solve(
    n: u32,
    supports: &[(u32, &[Dof])],
    g: Vec<(u32, Dof, f64)>,
    h: Vec<(u32, Dof, f64)>,
) -> PDeltaResult {
    let mut m = Model::new();
    for i in 0..=n {
        m.nodes.push(Node::new(
            NodeId(i + 1),
            Dim::D2,
            [L * i as f64 / n as f64, 0.0, 0.0],
        ));
    }
    for (id, dofs) in supports {
        pin(&mut m, *id, dofs);
    }
    m.cases.push(case("G", g));
    m.cases.push(case("H", h));
    let el: Arc<dyn Element> =
        Arc::new(EulerFrame2D::new(E, A, I, 0.0, MomentReleases::default()).unwrap());
    let (els, prods): (Vec<_>, Vec<_>) = (1..=n).map(|i| member(&m, el.clone(), i, i + 1)).unzip();
    let mut s = LinearSystem::from_model(ReferenceBackend, &m, &els, &[]).unwrap();
    s.set_geometric_producers(prods).unwrap();
    let r = PDelta::run(
        &mut s,
        &(CaseRef("G".into()), 1.0),
        &(CaseRef("H".into()), 1.0),
        options(),
    )
    .unwrap();
    if r.outcome != PDeltaOutcome::Completed {
        let l = r.iterations.last().unwrap();
        panic!(
            "{:?} it={} dchg={:e} res={:e}",
            r.outcome, l.iteration, l.relative_displacement_change, l.relative_free_residual
        );
    }
    r
}
fn uy(r: &PDeltaResult, node: u32) -> f64 {
    r.global_displacements.as_ref().unwrap()[((node - 1) * 3 + 1) as usize]
}

/// Caso 1: simplemente apoyada, carga uniforme w (cargas nodales consistentes), axial P.
fn case1(n: u32, p: f64) -> (f64, f64) {
    let dx = L / n as f64;
    let mut h: Vec<_> = (2..=n).map(|i| (i, Dof::Uy, W * dx)).collect();
    // Las medias cargas en los apoyos completan el vector consistente de w.
    h.push((1, Dof::Uy, W * dx / 2.0));
    h.push((n + 1, Dof::Uy, W * dx / 2.0));
    h.push((1, Dof::Rz, W * dx * dx / 12.0));
    h.push((n + 1, Dof::Rz, -W * dx * dx / 12.0));
    let r = solve(
        n,
        &[(1, &[Dof::Ux, Dof::Uy]), (n + 1, &[Dof::Uy])],
        vec![(n + 1, Dof::Ux, -p)],
        h,
    );
    let d = uy(&r, n / 2 + 1);
    // Estática en la configuración deformada (cuerpo libre de media barra).
    let m = W * L * L / 8.0 + p * d;
    (m, d)
}
fn case1_exact(p: f64) -> (f64, f64) {
    if p == 0.0 {
        return (W * L * L / 8.0, 5.0 * W * math::powi(L, 4) / (384.0 * ei()));
    }
    let k = math::sqrt(p / ei());
    let u = k * L / 2.0;
    let m = W / (k * k) * (1.0 / math::cos(u) - 1.0);
    let d = W / (ei() * math::powi(k, 4)) * (1.0 / math::cos(u) - 1.0) - W * L * L / (8.0 * p);
    (m, d)
}
/// Caso 2: voladizo empotrado, H en la punta, axial P. M = -Rz_base para H > 0.
fn case2(n: u32, p: f64) -> (f64, f64) {
    let r = solve(
        n,
        &[(1, Dim::D2.dofs())],
        vec![(n + 1, Dof::Ux, -p)],
        vec![(n + 1, Dof::Uy, H)],
    );
    let m = -r
        .reactions
        .iter()
        .find(|x| x.node == NodeId(1) && x.dof == Dof::Rz)
        .unwrap()
        .value;
    (m, uy(&r, n + 1))
}
fn case2_exact(p: f64) -> (f64, f64) {
    if p == 0.0 {
        return (H * L, H * math::powi(L, 3) / (3.0 * ei()));
    }
    let k = math::sqrt(p / ei());
    (
        (H * math::tan(k * L) / k),
        H * (math::tan(k * L) - k * L) / (p * k),
    )
}
/// Simplemente apoyada con Q en el centro; derivación y atribución pendiente en provenance.
fn tg(n: u32, p: f64, q: f64) -> (f64, f64) {
    let r = solve(
        n,
        &[(1, &[Dof::Ux, Dof::Uy]), (n + 1, &[Dof::Uy])],
        vec![(n + 1, Dof::Ux, -p)],
        vec![(n / 2 + 1, Dof::Uy, q)],
    );
    let d = uy(&r, n / 2 + 1);
    (q * L / 4.0 + p * d, d)
}
fn tg_exact(p: f64, q: f64) -> (f64, f64) {
    let u = math::sqrt(p / ei()) * L / 2.0;
    let d0 = q * math::powi(L, 3) / (48.0 * ei());
    (
        q / (2.0 * math::sqrt(p / ei())) * math::tan(u),
        d0 * 3.0 * (math::tan(u) - u) / math::powi(u, 3),
    )
}
fn err(a: f64, b: f64) -> f64 {
    (a / b - 1.0).abs()
}

// Valores publicados sin deformación por cortante (entre corchetes en las figuras).
const C1: [(f64, f64, f64); 4] = [
    (0.0, 235.0, 0.197),
    (150.0, 269.0, 0.224),
    (300.0, 313.0, 0.261),
    (450.0, 375.0, 0.311),
];
const C2: [(f64, f64, f64); 4] = [
    (0.0, 336.0, 0.901),
    (100.0, 469.0, 1.33),
    (150.0, 598.0, 1.75),
    (200.0, 848.0, 2.56),
];

#[test]
fn aisc_case1_p_small_delta_within_published_tolerances() {
    for (p, m_ref, d_ref) in C1 {
        let (m, d) = case1(MESH, p);
        let (me, de) = case1_exact(p);
        println!("C1 P={p} M={m:.2} ref={m_ref} err={:.3}% exact={me:.2} | D={d:.4} ref={d_ref} err={:.3}% exact={de:.4}", 100.0 * err(m, m_ref), 100.0 * err(d, d_ref));
        assert!(err(m, m_ref) <= TOL_M && err(d, d_ref) <= TOL_D);
        assert!(err(m, me) <= TOL_EXACT && err(d, de) <= TOL_EXACT);
    }
}

#[test]
fn aisc_case2_p_big_delta_within_published_tolerances() {
    for (p, m_ref, d_ref) in C2 {
        let (m, d) = case2(MESH, p);
        let (me, de) = case2_exact(p);
        println!("C2 P={p} M={m:.2} ref={m_ref} err={:.3}% exact={me:.2} | D={d:.4} ref={d_ref} err={:.3}% exact={de:.4}", 100.0 * err(m, m_ref), 100.0 * err(d, d_ref));
        assert!(err(m, m_ref) <= TOL_M && err(d, d_ref) <= TOL_D);
        assert!(err(m, me) <= TOL_EXACT && err(d, de) <= TOL_EXACT);
    }
}

#[test]
fn timoshenko_gere_midspan_point_load_closed_form() {
    let pe = math::powi(std::f64::consts::PI, 2) * ei() / (L * L);
    for ratio in [0.2, 0.4, 0.6, 0.7] {
        let (m, d) = tg(MESH, ratio * pe, 1.0);
        let (me, de) = tg_exact(ratio * pe, 1.0);
        let amp = 1.0 / (1.0 - ratio);
        let d0 = math::powi(L, 3) / (48.0 * ei());
        println!("TG P/Pe={ratio} M={m:.3} exact={me:.3} err={:.4}% | D={d:.5} exact={de:.5} err={:.4}% | D/D0={:.4} 1/(1-P/Pe)={amp:.4}", 100.0 * err(m, me), 100.0 * err(d, de), d / d0);
        assert!(err(m, me) <= TOL_EXACT && err(d, de) <= TOL_EXACT);
        // Se comprueba la aproximación 1/(1-P/Pe) a menos del 2% en estos cuatro cocientes.
        assert!(err(d / d0, amp) <= 0.02);
    }
}

#[test]
fn mesh_sensitivity_is_monotone_and_reported() {
    let mut previous_errors = [f64::INFINITY; 4];
    for n in [2, 4, 8, 16] {
        let (m1, d1) = case1(n, 450.0);
        let (m2, d2) = case2(n, 200.0);
        let (e1m, e1d) = case1_exact(450.0);
        let (e2m, e2d) = case2_exact(200.0);
        println!(
            "MESH n={n} C1 M err={:.4}% D err={:.4}% | C2 M err={:.4}% D err={:.4}%",
            100.0 * err(m1, e1m),
            100.0 * err(d1, e1d),
            100.0 * err(m2, e2m),
            100.0 * err(d2, e2d)
        );
        let errors = [err(m1, e1m), err(d1, e1d), err(m2, e2m), err(d2, e2d)];
        for (error, previous) in errors.iter().zip(previous_errors) {
            assert!(*error < previous, "n={n}: error {error} >= {previous}");
        }
        previous_errors = errors;
        assert!(err(m2, 848.0) <= TOL_M && err(d2, 2.56) <= TOL_D);
        assert!(err(m1, 375.0) <= TOL_M && err(d1, 0.311) <= TOL_D);
    }
}
