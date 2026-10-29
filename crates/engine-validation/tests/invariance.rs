//! Engine self-comparisons and elementary closed forms; see ENG-VAL-040.
//! Uses the same public assembly/analysis/recovery path as support/harness.rs.
use engine_analysis::{
    results::{CaseResults, MemberRecovery, NodeResult},
    StaticCaseResult, StaticLinear,
};
use engine_assembly::{DofMap, ElementContribution, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams};
use engine_core::math;
use engine_core::{Dim, Dof, MemberId, NodeId};
use engine_elements_frame::EulerFrame2D;
use engine_elements_frame3d::EulerFrame3D;
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const RTOL: f64 = 1e-10;
const E: f64 = 200e9;
const G: f64 = 80e9;
const A: f64 = 0.02;
const J: f64 = 3e-5;
const IY: f64 = 5e-5;
const IZ: f64 = 8e-5;
const DOFS: [Dof; 6] = [Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz];
type Rotation = [[f64; 3]; 3];
const IDENTITY: Rotation = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// No absolute floor: a zero reference series requires exact zero.
fn compare(label: &str, actual: &[f64], expected: &[f64]) {
    let scale = expected.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    compare_scaled(label, actual, expected, scale);
}

fn compare_scaled(label: &str, actual: &[f64], expected: &[f64], scale: f64) {
    assert_eq!(actual.len(), expected.len(), "{label}: series length");
    assert!(!actual.is_empty(), "{label}: empty comparison");
    assert!(scale.is_finite() && scale >= 0.0, "{label}: invalid scale");
    let mut max_error = 0.0_f64;
    let mut worst = 0;
    for (i, (&a, &b)) in actual.iter().zip(expected).enumerate() {
        assert!(
            a.is_finite() && b.is_finite(),
            "{label}[{i}]: nonfinite {a}, {b}"
        );
        if (a - b).abs() > max_error {
            max_error = (a - b).abs();
            worst = i;
        }
    }
    let tolerance = RTOL * scale;
    assert!(
        max_error <= tolerance,
        "{label}: maximum error={max_error:.16e}, tolerance={tolerance:.16e} \
         (rtol={RTOL:.1e}, scale={scale:.16e}), index={worst}, \
         actual={:.16e}, expected={:.16e}",
        actual[worst],
        expected[worst]
    );
}

fn rotate(q: Rotation, v: [f64; 3]) -> [f64; 3] {
    q.map(|row| row.iter().zip(v).map(|(a, b)| a * b).sum())
}

fn rotate_six(q: Rotation, v: [f64; 6]) -> [f64; 6] {
    let f = rotate(q, [v[0], v[1], v[2]]);
    let m = rotate(q, [v[3], v[4], v[5]]);
    [f[0], f[1], f[2], m[0], m[1], m[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn rotation(dim: Dim, angle: f64) -> Rotation {
    // Rodrigues' formula; D3 deliberately mixes all three global axes.
    let axis = if dim == Dim::D2 {
        [0.0, 0.0, 1.0]
    } else {
        let norm = math::sqrt(14.0);
        [1.0 / norm, 2.0 / norm, 3.0 / norm]
    };
    let (s, c) = math::sin_cos(angle);
    let skew = [
        [0.0, -axis[2], axis[1]],
        [axis[2], 0.0, -axis[0]],
        [-axis[1], axis[0], 0.0],
    ];
    std::array::from_fn(|i| {
        std::array::from_fn(|j| c * IDENTITY[i][j] + (1.0 - c) * axis[i] * axis[j] + s * skew[i][j])
    })
}

struct Edge {
    id: MemberId,
    nodes: [NodeId; 2],
}

struct Problem {
    dim: Dim,
    model: Model,
    edges: Vec<Edge>,
    orientation: [f64; 3],
}

impl Problem {
    fn new(dim: Dim, coordinates: &[[f64; 3]], edges: &[[u32; 2]], fixed: &[u32]) -> Self {
        let mut model = Model::new();
        model.nodes = coordinates
            .iter()
            .enumerate()
            .map(|(i, &xyz)| Node::new(NodeId(i as u32 + 1), dim, xyz))
            .collect();
        model.supports = fixed
            .iter()
            .map(|&id| Support {
                id: format!("fixed-{id}"),
                node: NodeId(id),
                restrained: dim.dofs().to_vec(),
            })
            .collect();
        model.cases.push(LoadCase {
            name: "Q".into(),
            loads: vec![],
        });
        Self {
            dim,
            model,
            orientation: [0.0, 0.0, 1.0],
            edges: edges
                .iter()
                .enumerate()
                .map(|(i, &[a, b])| Edge {
                    id: MemberId(i as u32 + 1),
                    nodes: [NodeId(a), NodeId(b)],
                })
                .collect(),
        }
    }

    fn load(&mut self, node: u32, values: [f64; 6]) {
        self.model.cases[0].loads.push(NodalLoad {
            node: NodeId(node),
            values: DOFS
                .iter()
                .zip(values)
                .filter(|(_, v)| *v != 0.0)
                .map(|(&dof, value)| {
                    assert!(self.dim.dofs().contains(&dof), "load on inactive DOF");
                    DofValue { dof, value }
                })
                .collect(),
        });
    }

    fn transform(&mut self, q: Rotation, offset: [f64; 3]) {
        for node in &mut self.model.nodes {
            let xyz = rotate(q, node.coordinates);
            node.coordinates = std::array::from_fn(|i| xyz[i] + offset[i]);
        }
        self.orientation = rotate(q, self.orientation);
        for load in &mut self.model.cases[0].loads {
            let mut values = [0.0; 6];
            for v in &load.values {
                values[DOFS.iter().position(|d| *d == v.dof).unwrap()] += v.value;
            }
            let values = rotate_six(q, values);
            load.values = DOFS
                .iter()
                .zip(values)
                .filter(|(_, v)| *v != 0.0)
                .map(|(&dof, value)| DofValue { dof, value })
                .collect();
        }
        // All supports are fully clamped: their restrained subspace is rotation invariant.
        assert!(self
            .model
            .supports
            .iter()
            .all(|s| s.restrained == self.dim.dofs()));
    }

    fn element(&self) -> Box<dyn Element> {
        if self.dim == Dim::D3 {
            Box::new(
                EulerFrame3D::new(
                    E,
                    G,
                    A,
                    J,
                    IY,
                    IZ,
                    0.0,
                    self.orientation,
                    Default::default(),
                )
                .unwrap(),
            )
        } else {
            Box::new(EulerFrame2D::new(E, A, IZ, 0.0, Default::default()).unwrap())
        }
    }

    fn solve(&self) -> CaseResults {
        self.run(false).2.unwrap()
    }

    /// Raw global displacements straight from the solver, without
    /// `CaseResults::recover` or the equilibrium check.
    fn solve_raw(&self) -> (DofMap, StaticCaseResult) {
        let (map, raw, _) = self.run(true);
        (map, raw.unwrap())
    }

    fn run(&self, raw_only: bool) -> (DofMap, Option<StaticCaseResult>, Option<CaseResults>) {
        let map = DofMap::new(&self.model.nodes).unwrap();
        let params = ElementParams::default();
        let elements: Vec<_> = self.edges.iter().map(|_| self.element()).collect();
        let contributions: Vec<_> = self
            .edges
            .iter()
            .zip(&elements)
            .map(|(edge, element)| {
                let coords = edge.nodes.map(|n| map.coordinates(n).unwrap());
                let ctx = ElementCtx {
                    dim: self.dim,
                    coords: &coords,
                    params: &params,
                };
                let location: Vec<_> = edge
                    .nodes
                    .iter()
                    .flat_map(|&n| {
                        element
                            .dofs_per_node()
                            .iter()
                            .map(|&d| map.global_index(n, d).unwrap())
                            .collect::<Vec<_>>()
                    })
                    .collect();
                let tangent = element
                    .respond(&ctx, &vec![0.0; location.len()], &[])
                    .unwrap()
                    .tangent;
                ElementContribution {
                    location,
                    stiffness: tangent,
                    mass_lumped: None,
                    mass_consistent: None,
                    geometric: None,
                    loads: vec![],
                }
            })
            .collect();
        let mut system =
            LinearSystem::from_model(ReferenceBackend, &self.model, &contributions, &[]).unwrap();
        let solved = StaticLinear::run(&mut system, &[(CaseRef("Q".into()), 1.0)]).unwrap();
        assert_eq!(solved.cases.len(), 1);
        if raw_only {
            return (map, solved.cases.into_iter().next(), None);
        }
        let members: Vec<_> = self
            .edges
            .iter()
            .zip(&elements)
            .map(|(e, element)| MemberRecovery {
                id: e.id,
                nodes: &e.nodes,
                element: element.as_ref(),
                dim: self.dim,
                params: &params,
                committed: &[],
                loads: &[],
                stations: None,
            })
            .collect();
        let result = CaseResults::recover(&solved.cases[0], &system, &members).unwrap();
        self.equilibrium(&result);
        (map, None, Some(result))
    }

    fn equilibrium(&self, result: &CaseResults) {
        // Independent physical balance about the global origin, including r x F.
        // Normalize by absolute physical contributions, with couples converted
        // using a characteristic length. A pure couple still defines a force
        // error scale; a pure force similarly defines a moment error scale.
        let mut sum = [0.0; 6];
        let mut scale = [0.0; 6];
        for node in &self.model.nodes {
            let reaction = node_result(result, node.id.0).reactions;
            let mut terms = vec![reaction];
            for load in self.model.cases[0]
                .loads
                .iter()
                .filter(|l| l.node == node.id)
            {
                let mut values = [0.0; 6];
                for v in &load.values {
                    values[DOFS.iter().position(|d| *d == v.dof).unwrap()] += v.value;
                }
                terms.push(values);
            }
            for values in terms {
                let arm = cross(node.coordinates, [values[0], values[1], values[2]]);
                for i in 0..6 {
                    sum[i] += values[i];
                    scale[i] += values[i].abs();
                    if i >= 3 {
                        sum[i] += arm[i - 3];
                        scale[i] += arm[i - 3].abs();
                    }
                }
            }
        }
        let length = self
            .edges
            .iter()
            .map(|edge| {
                let xyz = edge.nodes.map(|id| {
                    self.model
                        .nodes
                        .iter()
                        .find(|n| n.id == id)
                        .unwrap()
                        .coordinates
                });
                math::sqrt(
                    (0..3)
                        .map(|i| math::powi(xyz[1][i] - xyz[0][i], 2))
                        .sum::<f64>(),
                )
            })
            .fold(0.0_f64, f64::max);
        assert!(
            length.is_finite() && length > 0.0,
            "equilibrium characteristic length"
        );
        let force_scale = scale[..3].iter().copied().fold(0.0_f64, f64::max);
        let moment_scale = scale[3..].iter().copied().fold(0.0_f64, f64::max);
        for (label, range, normalization) in [
            (
                "global force equilibrium",
                0..3,
                force_scale.max(moment_scale / length),
            ),
            (
                "global moment equilibrium",
                3..6,
                moment_scale.max(force_scale * length),
            ),
        ] {
            compare_scaled(label, &sum[range], &[0.0; 3], normalization);
        }
    }
}

fn node_result(r: &CaseResults, id: u32) -> &NodeResult {
    r.nodes
        .iter()
        .find(|n| n.id == NodeId(id))
        .expect("mapped node result")
}

fn reference(dim: Dim) -> Problem {
    let z = if dim == Dim::D3 { 1.5 } else { 0.0 };
    let mut p = Problem::new(
        dim,
        &[
            [0.0, 0.0, 0.0],
            [2.5, 0.5, z],
            [4.0, 2.0, 0.0],
            [3.5, -1.0, -z],
        ],
        &[[1, 2], [2, 3], [2, 4]],
        &[1],
    );
    p.load(3, [350.0, -1200.0, 0.0, 0.0, 0.0, 170.0]);
    p.load(4, [-210.0, -450.0, 0.0, 0.0, 0.0, -80.0]);
    if dim == Dim::D3 {
        p.load(3, [0.0, 0.0, 620.0, 95.0, -130.0, 0.0]);
        p.load(4, [0.0, 0.0, -270.0, -40.0, 60.0, 0.0]);
    }
    p
}

fn compare_results(
    label: &str,
    reference: &CaseResults,
    actual: &CaseResults,
    nodes: &[(u32, u32)],
    members: &[(u32, u32)],
    q: Rotation,
    exact: bool,
) {
    assert_eq!(reference.nodes.len(), nodes.len());
    assert_eq!(actual.nodes.len(), nodes.len());
    assert_eq!(reference.members.len(), members.len());
    assert_eq!(actual.members.len(), members.len());
    for &(old, new) in nodes {
        let a = node_result(actual, new);
        let r = node_result(reference, old);
        for (field, got, expected) in [
            (
                "displacements",
                a.displacements,
                rotate_six(q, r.displacements),
            ),
            ("reactions", a.reactions, rotate_six(q, r.reactions)),
        ] {
            let context = format!("{label}, node {old}->{new}, {field}");
            // Keep translations/forces separate from rotations/couples.
            compare(&context, &got[..3], &expected[..3]);
            compare(&context, &got[3..], &expected[3..]);
            if exact {
                assert_eq!(got, expected, "{context}: exact translation comparison");
            }
        }
    }
    for &(old, new) in members {
        let a = actual
            .members
            .iter()
            .find(|m| m.id == MemberId(new))
            .unwrap();
        let r = reference
            .members
            .iter()
            .find(|m| m.id == MemberId(old))
            .unwrap();
        let context = format!("{label}, member {old}->{new}, local end forces");
        compare(&context, &a.end_forces_local, &r.end_forces_local);
        if exact {
            assert_eq!(a.end_forces_local, r.end_forces_local, "{context}");
        }
    }
}

const NODE_IDENTITY: [(u32, u32); 4] = [(1, 1), (2, 2), (3, 3), (4, 4)];
const MEMBER_IDENTITY: [(u32, u32); 3] = [(1, 1), (2, 2), (3, 3)];

fn translation(dim: Dim) {
    let mut p = reference(dim);
    let baseline = p.solve();
    // Dyadic coordinates and offsets preserve every member difference exactly.
    let z = if dim == Dim::D3 { 4.75 } else { 0.0 };
    p.transform(IDENTITY, [13.25, -8.5, z]);
    compare_results(
        "translation",
        &baseline,
        &p.solve(),
        &NODE_IDENTITY,
        &MEMBER_IDENTITY,
        IDENTITY,
        true,
    );
}

fn rotations(dim: Dim) {
    let baseline = reference(dim).solve();
    for degrees in [30.0_f64, 90.0] {
        let q = rotation(dim, degrees.to_radians());
        let mut p = reference(dim);
        p.transform(q, [0.0; 3]);
        compare_results(
            &format!("rotation {degrees} degrees"),
            &baseline,
            &p.solve(),
            &NODE_IDENTITY,
            &MEMBER_IDENTITY,
            q,
            false,
        );
    }
}

fn permutation(dim: Dim) {
    let mut p = reference(dim);
    let baseline = p.solve();
    let nodes = [(1, 41), (2, 7), (3, 93), (4, 12)];
    let members = [(1, 73), (2, 9), (3, 51)];
    let mapped_node = |id: NodeId| NodeId(nodes.iter().find(|&&(old, _)| old == id.0).unwrap().1);
    for n in &mut p.model.nodes {
        n.id = mapped_node(n.id);
    }
    p.model.nodes.rotate_left(2);
    p.model.nodes.reverse();
    for s in &mut p.model.supports {
        s.node = mapped_node(s.node);
    }
    for l in &mut p.model.cases[0].loads {
        l.node = mapped_node(l.node);
    }
    for e in &mut p.edges {
        e.nodes = e.nodes.map(mapped_node);
        e.id = MemberId(members.iter().find(|&&(old, _)| old == e.id.0).unwrap().1);
    }
    p.edges.reverse();
    compare_results(
        "renumbered/reordered nodes and members",
        &baseline,
        &p.solve(),
        &nodes,
        &members,
        IDENTITY,
        false,
    );
}

fn reciprocity(dim: Dim) {
    // Unit generalized loads: C_(j,b),(i,a) = C_(i,a),(j,b).
    // The force/moment pair compares displacement under moment with rotation under force.
    let mut pairs = vec![((3, 1), (4, 0)), ((3, 1), (4, 5))];
    if dim == Dim::D3 {
        pairs.extend([((3, 2), (4, 1)), ((3, 2), (4, 4)), ((3, 3), (4, 5))]);
    }
    for ((i, a), (j, b)) in pairs {
        let mut first = reference(dim);
        first.model.cases[0].loads.clear();
        let mut load = [0.0; 6];
        load[a] = 1.0;
        first.load(i, load);
        let first_result = first.solve();
        let mut second = reference(dim);
        second.model.cases[0].loads.clear();
        let mut load = [0.0; 6];
        load[b] = 1.0;
        second.load(j, load);
        let second_result = second.solve();
        let cij = node_result(&first_result, j).displacements[b];
        let cji = node_result(&second_result, i).displacements[a];
        assert!(
            cij.abs() > 1e-20 && cji.abs() > 1e-20,
            "reciprocity pair must be nontrivial"
        );
        compare(
            &format!("Maxwell-Betti ({i},{a}) <-> ({j},{b})"),
            &[cij],
            &[cji],
        );
    }
}

fn reciprocity_raw(dim: Dim) {
    // Same pairs as `reciprocity`, read from the solver's global displacement
    // vector, so an asymmetric stiffness reaches this comparison instead of
    // stopping in result recovery.
    let mut pairs = vec![((3, 1), (4, 0)), ((3, 1), (4, 5))];
    if dim == Dim::D3 {
        pairs.extend([((3, 2), (4, 1)), ((3, 2), (4, 4)), ((3, 3), (4, 5))]);
    }
    let raw = |node: u32, comp: usize| -> (DofMap, Vec<f64>) {
        let mut p = reference(dim);
        p.model.cases[0].loads.clear();
        let mut load = [0.0; 6];
        load[comp] = 1.0;
        p.load(node, load);
        let (map, solved) = p.solve_raw();
        (map, solved.global_displacements)
    };
    for ((i, a), (j, b)) in pairs {
        let (first_map, first) = raw(i, a);
        let (second_map, second) = raw(j, b);
        let at = |map: &DofMap, n: u32, c: usize| {
            map.global_index(NodeId(n), DOFS[c]).unwrap().0 as usize
        };
        let cij = first[at(&first_map, j, b)];
        let cji = second[at(&second_map, i, a)];
        assert!(
            cij.abs() > 1e-20 && cji.abs() > 1e-20,
            "raw reciprocity pair must be nontrivial"
        );
        compare(
            &format!("raw Maxwell-Betti ({i},{a}) <-> ({j},{b})"),
            &[cij],
            &[cji],
        );
    }
}

fn symmetry(dim: Dim) {
    let mut p = Problem::new(
        dim,
        &[
            [-3.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [-3.0, 4.0, 0.0],
            [3.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
        ],
        &[[1, 3], [2, 4], [3, 5], [5, 4]],
        &[1, 2],
    );
    let fz = if dim == Dim::D3 { 230.0 } else { 0.0 };
    p.load(3, [150.0, -700.0, fz, 0.0, 0.0, 0.0]);
    p.load(4, [-150.0, -700.0, fz, 0.0, 0.0, 0.0]);
    p.load(5, [0.0, -900.0, 2.0 * fz, 0.0, 0.0, 0.0]);
    let r = p.solve();
    // Reflection x -> -x: polar vectors (-x,y,z), axial vectors (x,-y,-z).
    let signs = [-1.0, 1.0, 1.0, 1.0, -1.0, -1.0];
    // Include the center node to enforce zero odd components on the symmetry plane.
    for field in ["displacements", "reactions"] {
        for block in [0..3, 3..6] {
            let mut actual = Vec::new();
            let mut expected = Vec::new();
            for (left, right) in [(1, 2), (3, 4), (5, 5)] {
                let values = |id| {
                    let n = node_result(&r, id);
                    if field == "displacements" {
                        n.displacements
                    } else {
                        n.reactions
                    }
                };
                let a = values(right);
                let b = values(left);
                for i in block.clone() {
                    actual.push(a[i]);
                    expected.push(signs[i] * b[i]);
                }
            }
            // One series over all mirror pairs avoids normalizing a center-plane
            // numerical zero by that zero alone. There is no absolute floor.
            compare(
                &format!("mirror pairs (1,2),(3,4),(5,5), {field}, {block:?}"),
                &actual,
                &expected,
            );
        }
    }
    let base = node_result(&r, 1).reactions;
    compare(
        "equal vertical support forces",
        &[base[1], node_result(&r, 2).reactions[1]],
        &[1150.0; 2],
    );
    if dim == Dim::D3 {
        compare(
            "equal out-of-plane support forces",
            &[base[2], node_result(&r, 2).reactions[2]],
            &[-2.0 * fz; 2],
        );
    }
}

fn cantilever_closed_form(dim: Dim) {
    let l = 4.0_f64;
    let force = 1200.0;
    let mut p = Problem::new(
        dim,
        &[[0.0; 3], [l / 2.0, 0.0, 0.0], [l, 0.0, 0.0]],
        &[[1, 2], [2, 3]],
        &[1],
    );
    p.load(3, [0.0, -force, 0.0, 0.0, 0.0, 0.0]);
    let r = p.solve();
    let tip = node_result(&r, 3).displacements;
    // Integrating EI v'' = -P(L-x), with v(0)=v'(0)=0.
    compare(
        "cantilever PL^3/(3EI)",
        &[tip[1]],
        &[-force * math::powi(l, 3) / (3.0 * E * IZ)],
    );
    compare(
        "cantilever PL^2/(2EI)",
        &[tip[5]],
        &[-force * math::powi(l, 2) / (2.0 * E * IZ)],
    );
    if dim == Dim::D3 {
        p.model.cases[0].loads.clear();
        p.load(3, [0.0, 0.0, force, 75.0, 0.0, 0.0]);
        let r = p.solve();
        let tip = node_result(&r, 3).displacements;
        compare(
            "D3 bending about y",
            &[tip[2], tip[4]],
            &[
                force * math::powi(l, 3) / (3.0 * E * IY),
                -force * math::powi(l, 2) / (2.0 * E * IY),
            ],
        );
        compare("D3 torsion ML/(GJ)", &[tip[3]], &[75.0 * l / (G * J)]);
    }
}

#[test]
fn translation_2d() {
    translation(Dim::D2);
}
#[test]
fn translation_3d() {
    translation(Dim::D3);
}
#[test]
fn rotation_2d() {
    rotations(Dim::D2);
}
#[test]
fn rotation_3d() {
    rotations(Dim::D3);
}
#[test]
fn permutation_2d() {
    permutation(Dim::D2);
}
#[test]
fn permutation_3d() {
    permutation(Dim::D3);
}
#[test]
fn maxwell_betti_2d() {
    reciprocity(Dim::D2);
}
#[test]
fn maxwell_betti_3d() {
    reciprocity(Dim::D3);
}
#[test]
fn maxwell_betti_raw_2d() {
    reciprocity_raw(Dim::D2);
}
#[test]
fn maxwell_betti_raw_3d() {
    reciprocity_raw(Dim::D3);
}
#[test]
fn symmetry_2d() {
    symmetry(Dim::D2);
}
#[test]
fn symmetry_3d() {
    symmetry(Dim::D3);
}
#[test]
fn cantilever_analytical_2d() {
    cantilever_closed_form(Dim::D2);
}
#[test]
fn cantilever_analytical_3d() {
    cantilever_closed_form(Dim::D3);
}
