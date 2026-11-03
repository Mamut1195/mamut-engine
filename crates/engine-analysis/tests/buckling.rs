//! SOURCE: own derivation of EI w'''' + P w'' = 0.
//! CITATION: provenance/ENG-ANL-006.md. Timoshenko–Gere citation, page, and table: UNKNOWN.
//! source.kind = analytical. No published table and no external engine.
use engine_analysis::buckling::{BucklingLinear, BucklingResults};
use engine_assembly::{DofMap, ElementContribution, LinearSystem};
use engine_contracts::{Element, ElementCtx, ElementParams};
use engine_core::math;
use engine_core::{Dim, Dof, EResult, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::{ReferenceBackend, SymMatrix};
use engine_model::{Model, Node, Support};
use std::f64::consts::PI;

const E: f64 = 200.0;
const A: f64 = 2.0;
const I: f64 = 3.0;
const L: f64 = 4.0;

struct AnalyticalSource {
    kind: &'static str,
    citation: &'static str,
    derivation: &'static str,
}

const EULER: AnalyticalSource = AnalyticalSource {
    kind: "analytical",
    citation: "DESCONOCIDO. Timoshenko–Gere no fue consultado: sin página ni tabla. Derivación propia en provenance/ENG-ANL-006.md.",
    derivation: "EI w'''' + P w'' = 0, P>0 en compresión. Pcr = C π² EI / L². Articulada-articulada: C=1. Empotrada-libre: C=1/4. Empotrada-empotrada: C=4. Empotrada-articulada: tan θ = θ, θ en (π, 3π/2), C=(θ/π)².",
};

#[derive(Clone, Copy)]
enum End {
    Fixed,
    Pinned,
    Free,
}

fn close(actual: f64, expected: f64, tol: f64) {
    assert!(
        (actual - expected).abs() <= tol,
        "{actual} != {expected}, tol={tol}; source.kind={}, {}",
        EULER.kind,
        EULER.citation
    );
}

fn fixed_pinned_theta() -> f64 {
    // f(θ) = sin θ − θ cos θ. f(π) = π > 0 and f(3π/2) = −1 < 0.
    // f'(θ) = θ sin θ < 0 on (π, 3π/2), so the root in that interval is unique.
    let mut lo = PI;
    let mut hi = 1.5 * PI;
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let value = math::sin(mid) - mid * math::cos(mid);
        if value > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

fn euler_constant(end_i: End, end_j: End) -> f64 {
    match (end_i, end_j) {
        (End::Pinned, End::Pinned) => 1.0,
        (End::Fixed, End::Free) => 0.25,
        (End::Fixed, End::Fixed) => 4.0,
        (End::Fixed, End::Pinned) => {
            let theta = fixed_pinned_theta();
            (theta / PI) * (theta / PI)
        }
        _ => panic!("this pair is not one of the four Euler cases"),
    }
}

fn restraints(end: End, at_start: bool) -> Vec<Dof> {
    match (end, at_start) {
        (End::Fixed, true) => vec![Dof::Ux, Dof::Uy, Dof::Rz],
        (End::Fixed, false) => vec![Dof::Uy, Dof::Rz],
        (End::Pinned, true) => vec![Dof::Ux, Dof::Uy],
        (End::Pinned, false) => vec![Dof::Uy],
        (End::Free, _) => Vec::new(),
    }
}

fn column(segments: usize, start: End, end: End, axial: f64) -> LinearSystem<ReferenceBackend> {
    let mut model = Model::new();
    for index in 0..=segments {
        let x = L * index as f64 / segments as f64;
        let mut node = Node::new(NodeId(index as u32 + 1), Dim::D2, [x, 0.0, 0.0]);
        node.dofs = vec![Dof::Ux, Dof::Uy, Dof::Rz];
        model.nodes.push(node);
    }
    for (index, end, at_start) in [(0, start, true), (segments, end, false)] {
        let restrained = restraints(end, at_start);
        if restrained.is_empty() {
            continue;
        }
        model.supports.push(Support {
            id: format!("s{index}"),
            node: NodeId(index as u32 + 1),
            restrained,
        });
    }
    let map = DofMap::new(&model.nodes).unwrap();
    let beam = EulerFrame2D::new(E, A, I, 0.0, MomentReleases::default()).unwrap();
    let params = ElementParams::default();
    let mut elements = Vec::new();
    for segment in 0..segments {
        let length = L / segments as f64;
        let x0 = segment as f64 * length;
        let coords = [[x0, 0.0, 0.0], [x0 + length, 0.0, 0.0]];
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        let zeros = [0.0; 6];
        let stiffness = beam.respond(&ctx, &zeros, &[]).unwrap().tangent;
        // N = EA Δ / L. axial is the reference tension; compression is negative.
        let mut shortening = zeros;
        shortening[3] = axial * length / (E * A);
        let geometric = beam
            .geometric_stiffness(&ctx, &shortening, &[])
            .unwrap()
            .unwrap();
        let mut location = Vec::new();
        for node in [segment as u32 + 1, segment as u32 + 2] {
            for dof in [Dof::Ux, Dof::Uy, Dof::Rz] {
                location.push(map.global_index(NodeId(node), dof).unwrap());
            }
        }
        elements.push(ElementContribution {
            location,
            stiffness,
            mass_lumped: None,
            mass_consistent: None,
            geometric: Some(geometric),
            loads: vec![],
        });
    }
    LinearSystem::from_model(ReferenceBackend, &model, &elements, &[]).unwrap()
}

fn factor(segments: usize, start: End, end: End) -> f64 {
    let mut system = column(segments, start, end, -1.0);
    let accepted = system.accepted_displacement().to_vec();
    let trial = system.trial_displacement().to_vec();
    let result = BucklingLinear::run(&mut system, 1).unwrap();
    check_mode(&mut system, &result);
    assert_eq!(system.accepted_displacement(), accepted.as_slice());
    assert_eq!(system.trial_displacement(), trial.as_slice());
    result.modes[0].eigenvalue
}

fn check_mode(system: &mut LinearSystem<ReferenceBackend>, result: &BucklingResults) {
    assert_eq!(result.rigid_mode_count, 0);
    let mode = &result.modes[0];
    let k = system.reduced_stiffness().clone();
    let kg = system.reduced_geometric_reference().unwrap();
    let phi = &mode.free_shape;
    close(quadratic(&kg, phi, -1.0), 1.0, 1e-8);
    close(
        quadratic(&k, phi, 1.0),
        mode.eigenvalue,
        1e-6 * mode.eigenvalue.abs().max(1.0),
    );
    let mut residual: f64 = 0.0;
    let mut scale: f64 = 1.0;
    let n = phi.len();
    for row in 0..n {
        let mut elastic = 0.0;
        let mut geometric = 0.0;
        for column in 0..n {
            elastic += k.dense().values()[row * n + column] * phi[column];
            geometric += kg.dense().values()[row * n + column] * phi[column];
        }
        let component = elastic + mode.eigenvalue * geometric;
        residual = residual.max(component.abs());
        scale = scale
            .max(elastic.abs())
            .max((mode.eigenvalue * geometric).abs());
    }
    assert!(
        residual / scale < 1e-8,
        "residual {residual} / {scale}; {}",
        EULER.derivation
    );
}

fn quadratic(matrix: &SymMatrix, vector: &[f64], scale: f64) -> f64 {
    let n = vector.len();
    let mut sum = 0.0;
    for row in 0..n {
        for column in 0..n {
            sum += vector[row] * scale * matrix.dense().values()[row * n + column] * vector[column];
        }
    }
    sum
}

fn pcr(constant: f64) -> f64 {
    constant * PI * PI * E * I / (L * L)
}

#[test]
fn source_is_analytical_and_book_citation_is_unknown() {
    assert_eq!(EULER.kind, "analytical");
    assert!(EULER.citation.contains("DESCONOCIDO"));
    assert!(!EULER.derivation.is_empty());
    assert_eq!(BucklingLinear::KIND, "buckling.linear");
}

#[test]
fn fixed_pinned_root_is_the_first_positive_solution() {
    let theta = fixed_pinned_theta();
    let residual = math::sin(theta) - theta * math::cos(theta);
    assert!(theta > PI && theta < 1.5 * PI, "theta={theta}");
    assert!(residual.abs() < 1e-12, "f(theta)={residual}");
    let constant = (theta / PI) * (theta / PI);
    assert!(constant > 2.0 && constant < 2.1, "C={constant}");
    // No root of tan θ = θ on (0, π]: f stays positive up to π, and θ=0 is the trivial field.
    assert!(math::sin(1.0) - 1.0 * math::cos(1.0) > 0.0);
    assert!(math::sin(PI) - PI * math::cos(PI) > 0.0);
}

#[test]
fn one_element_pinned_column_matches_the_cubic_projection() {
    // Rotational block, N = −1 N. K = (EI/L) [[4, 2], [2, 4]].
    // Kg = −(L/30) [[4, −1], [−1, 4]]. Mode [1, −1] gives λ = 12 EI / L².
    let exact = 12.0 * E * I / (L * L);
    let mut system = column(1, End::Pinned, End::Pinned, -1.0);
    let result = BucklingLinear::run(&mut system, 1).unwrap();
    close(result.modes[0].eigenvalue, exact, 1e-8);
    check_mode(&mut system, &result);
    let mut doubled = column(1, End::Pinned, End::Pinned, -2.0);
    let half = BucklingLinear::run(&mut doubled, 1).unwrap();
    close(half.modes[0].eigenvalue * 2.0, exact, 1e-7);
}

#[test]
fn four_euler_cases_converge_to_the_beam_equation() {
    let cases = [
        (End::Pinned, End::Pinned, "articulada-articulada"),
        (End::Fixed, End::Free, "empotrada-libre"),
        (End::Fixed, End::Pinned, "empotrada-articulada"),
        (End::Fixed, End::Fixed, "empotrada-empotrada"),
    ];
    for (start, end, name) in cases {
        let constant = euler_constant(start, end);
        let exact = pcr(constant);
        let coarse = factor(4, start, end);
        let fine = factor(8, start, end);
        assert!(
            coarse + 1e-8 >= exact,
            "{name}: coarse {coarse} is below {exact}"
        );
        assert!(fine + 1e-8 >= exact, "{name}: fine {fine} is below {exact}");
        assert!(fine < coarse, "{name}: refinement did not decrease λ");
        let ratio = fine / exact;
        assert!(
            (ratio - 1.0).abs() < 1e-3,
            "{name}: λ={fine}, Pcr={exact}, C={constant}, ratio={ratio}; {}",
            EULER.derivation
        );
    }
}

#[test]
fn zero_geometric_stiffness_and_tension_are_rejected() -> EResult<()> {
    let mut model = Model::new();
    for (index, x) in [(1, 0.0), (2, L)] {
        let mut node = Node::new(NodeId(index), Dim::D2, [x, 0.0, 0.0]);
        node.dofs = vec![Dof::Ux, Dof::Uy, Dof::Rz];
        model.nodes.push(node);
    }
    model.supports.push(Support {
        id: "base".into(),
        node: NodeId(1),
        restrained: vec![Dof::Ux, Dof::Uy, Dof::Rz],
    });
    let map = DofMap::new(&model.nodes)?;
    let beam = EulerFrame2D::new(E, A, I, 0.0, MomentReleases::default())?;
    let params = ElementParams::default();
    let coords = [[0.0, 0.0, 0.0], [L, 0.0, 0.0]];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let stiffness = beam.respond(&ctx, &[0.0; 6], &[])?.tangent;
    let mut location = Vec::new();
    for node in [1u32, 2] {
        for dof in [Dof::Ux, Dof::Uy, Dof::Rz] {
            location.push(map.global_index(NodeId(node), dof)?);
        }
    }
    let mut bare = LinearSystem::from_model(
        ReferenceBackend,
        &model,
        &[ElementContribution {
            location: location.clone(),
            stiffness: stiffness.clone(),
            mass_lumped: None,
            mass_consistent: None,
            geometric: None,
            loads: vec![],
        }],
        &[],
    )?;
    assert_eq!(
        BucklingLinear::run(&mut bare, 1).unwrap_err().code,
        "ENG-E-0010"
    );
    assert!(BucklingLinear::run(&mut bare, 0)?.modes.is_empty());
    let mut tension_u = [0.0; 6];
    tension_u[3] = L / (E * A);
    let geometric = beam.geometric_stiffness(&ctx, &tension_u, &[])?.unwrap();
    let mut tension = LinearSystem::from_model(
        ReferenceBackend,
        &model,
        &[ElementContribution {
            location,
            stiffness,
            mass_lumped: None,
            mass_consistent: None,
            geometric: Some(geometric),
            loads: vec![],
        }],
        &[],
    )?;
    assert_eq!(
        BucklingLinear::run(&mut tension, 1).unwrap_err().code,
        "ENG-E-0010"
    );
    Ok(())
}
