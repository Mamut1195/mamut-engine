//! Local P-Delta: own derivations, provenance/ENG-ASM-001-KG.md.
//! External sources: none. Precise bibliographic citation: PENDING.
use engine_assembly::{DofMap, ElementContribution, LinearSystem, PrescribedDof};
use engine_contracts::{
    conformance::check_system_view, CaseRef, Element, ElementCtx, ElementParams, SystemView,
};
use engine_core::math;
use engine_core::{Dim, Dof, DofIndex, NodeId};
use engine_elements_truss::{geometric_producer, LinearTruss};
use engine_linalg::{DMat, ReferenceBackend, SymMatrix};
use engine_model::{LoadCase, Model, Node, Support};

const ABS_TOL: f64 = 1e-12; // N/m: closed-form formula against assembly.
const FD_REL_TOL: f64 = 1e-6; // Relative maximum norm, incremental derivative.
const FD_STEP: f64 = 1e-5; // m, central differences of analytical force.

fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= ABS_TOL, "{a} != {b}; tol={ABS_TOL}");
}

fn model(dim: Dim, coords: &[[f64; 3]]) -> Model {
    let bar = LinearTruss::new(dim, 200.0, 2.0, 0.0).unwrap();
    let mut model = Model::new();
    for (i, &coord) in coords.iter().enumerate() {
        let mut node = Node::new(NodeId(i as u32 + 1), dim, coord);
        node.dofs = bar.dofs_per_node().to_vec();
        model.nodes.push(node);
        if i + 1 < coords.len() {
            model.supports.push(Support {
                id: format!("base{i}"),
                node: NodeId(i as u32 + 1),
                restrained: bar.dofs_per_node().to_vec(),
            });
        }
    }
    model.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![],
    });
    model
}

fn contribution(
    bar: LinearTruss,
    ctx: &ElementCtx<'_>,
    location: Vec<DofIndex>,
) -> ElementContribution {
    let zero = vec![0.0; location.len()];
    ElementContribution {
        location,
        stiffness: bar.respond(ctx, &zero, &[]).unwrap().tangent,
        mass_lumped: None,
        mass_consistent: None,
        geometric: bar.geometric_stiffness(ctx, &zero, &[]).unwrap(),
        loads: vec![],
    }
}

#[test]
fn axial_state_changes_transverse_kg_in_2d_and_3d_and_passes_kit() {
    // EA=400 N, L=2 m => N=200δ N, transverse Kg=100δ N/m.
    for dim in [Dim::D2, Dim::D3] {
        let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
        let params = ElementParams::default();
        let ctx = ElementCtx {
            dim,
            coords: &coords,
            params: &params,
        };
        let bar = LinearTruss::new(dim, 200.0, 2.0, 0.0).unwrap();
        let axes = bar.dofs_per_node().len();
        let location: Vec<_> = (0..2 * axes).map(|i| DofIndex(i as u32)).collect();
        let element = contribution(bar, &ctx, location.clone());
        let mut s =
            LinearSystem::from_model(ReferenceBackend, &model(dim, &coords), &[element], &[])
                .unwrap();
        let mut out = SymMatrix::zeros(axes).unwrap();
        s.geometric(&vec![0.0; axes], &mut out).unwrap();
        let static_zero = out.clone();
        let mut nonzero = vec![0.0; axes];
        nonzero[0] = 0.06;
        assert_eq!(
            s.geometric(&nonzero, &mut out).unwrap_err().code,
            "ENG-E-0013"
        );
        s.set_geometric_producers(vec![geometric_producer(bar, &ctx, location).unwrap()])
            .unwrap();
        s.geometric(&vec![0.0; axes], &mut out).unwrap();
        assert_eq!(out, static_zero, "Kg(0) matches the zero static matrix");
        for delta in [0.06, -0.04, 0.0] {
            let mut u = vec![0.0; axes];
            u[0] = delta;
            // A transverse displacement does not change the projected extension.
            u[1] = 0.3;
            s.geometric(&u, &mut out).unwrap();
            for i in 0..axes {
                for j in 0..axes {
                    close(
                        out.dense().get(i, j).unwrap(),
                        if i == j && i > 0 { 100.0 * delta } else { 0.0 },
                    );
                    assert_eq!(out.dense().get(i, j), out.dense().get(j, i));
                }
            }
        }
        let diagnostics = check_system_view(
            &mut s,
            &[vec![0.0; axes], nonzero, vec![-0.02; axes]],
            &CaseRef("P".into()),
            FD_STEP,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }
}

#[test]
fn prescribed_extension_is_seen_even_at_reduced_zero() {
    // Node i moves -0.06 m; u_j=0 => N=12 N => Kg_yy=6 N/m.
    let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let bar = LinearTruss::new(Dim::D2, 200.0, 2.0, 0.0).unwrap();
    let location = vec![DofIndex(0), DofIndex(1), DofIndex(2), DofIndex(3)];
    let mut element = contribution(bar, &ctx, location.clone());
    element.geometric = bar
        .geometric_stiffness(&ctx, &[-0.06, 0.0, 0.0, 0.0], &[])
        .unwrap();
    let mut s = LinearSystem::from_model(
        ReferenceBackend,
        &model(Dim::D2, &coords),
        &[element],
        &[PrescribedDof {
            node: NodeId(1),
            dof: Dof::Ux,
            value: -0.06,
        }],
    )
    .unwrap();
    let mut out = SymMatrix::zeros(2).unwrap();
    s.geometric(&[0.0, 0.0], &mut out).unwrap();
    let static_reference = out.clone();
    s.set_geometric_producers(vec![geometric_producer(bar, &ctx, location).unwrap()])
        .unwrap();
    s.geometric(&[0.0, 0.0], &mut out).unwrap();
    assert_eq!(
        out, static_reference,
        "Kg(0) also matches the nonzero static matrix"
    );
    close(out.dense().get(1, 1).unwrap(), 6.0);
}

#[test]
fn oblique_3d_producer_returns_global_projector() {
    let coords = [[0.0; 3], [2.0, 3.0, 6.0]]; // L=7 m.
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D3,
        coords: &coords,
        params: &params,
    };
    let bar = LinearTruss::new(Dim::D3, 200.0, 2.0, 0.0).unwrap();
    let location = (0..6).map(DofIndex).collect::<Vec<_>>();
    let element = contribution(bar, &ctx, location.clone());
    let mut s =
        LinearSystem::from_model(ReferenceBackend, &model(Dim::D3, &coords), &[element], &[])
            .unwrap();
    s.set_geometric_producers(vec![geometric_producer(bar, &ctx, location).unwrap()])
        .unwrap();
    let d = [2.0 / 7.0, 3.0 / 7.0, 6.0 / 7.0];
    let mut out = SymMatrix::zeros(3).unwrap();
    s.geometric(&d.map(|x| 0.07 * x), &mut out).unwrap(); // N=4 N.
    for i in 0..3 {
        for j in 0..3 {
            let identity = if i == j { 1.0 } else { 0.0 };
            close(
                out.dense().get(i, j).unwrap(),
                (4.0 / 7.0) * (identity - d[i] * d[j]),
            );
        }
    }
}

// Exact force in prestressed members under incremental perturbation v of the
// apex. Constant k=EA/L, l0=L−N/k; f(v)=Σ k(l(v)−l0) r(v)/l(v).
// At v=0 the axial force is N obtained from base u, with the crate's constant geometry.
// This independent analytical force has derivative K+Kg; it does NOT differentiate
// linear respond, whose derivative is only K. See scope in the log.
fn arch_force(v: [f64; 2], a: f64, h: f64, ea: f64, axial: f64) -> [f64; 2] {
    let length = math::hypot(a, h);
    let k = ea / length;
    let natural_length = length - axial / k;
    let mut force = [0.0; 2];
    for base_x in [-a, a] {
        let r = [v[0] - base_x, h + v[1]];
        let ell = math::hypot(r[0], r[1]);
        let n = k * (ell - natural_length);
        for i in 0..2 {
            force[i] += n * r[i] / ell;
        }
    }
    force
}

#[test]
fn symmetric_two_bar_arch_tangent_and_analytic_critical_multiplier() {
    // Bases (-a,0),(a,0), apex (0,h): s=h/L,c=a/L,k=EA/L.
    // N=k s w<0; P=-2Ns>0 along -Y (fixture local vertical).
    // K=2k diag(c²,s²), Kg=2N/L diag(s²,c²).
    // λ_cr,y=-EA s²/(N c²); λ_cr,x=-EA c²/(N s²).
    let (a, h, ea, w): (f64, f64, f64, f64) = (2.0, 1.0, 400.0, -0.1);
    let length = math::hypot(a, h);
    let (c, s) = (a / length, h / length);
    let k = ea / length;
    let axial = k * s * w; // -8 N.
    let p = -2.0 * axial * s;
    let coords = [[-a, 0.0, 0.0], [a, 0.0, 0.0], [0.0, h, 0.0]];
    let model = model(Dim::D2, &coords);
    let map = DofMap::new(&model.nodes).unwrap();
    let params = ElementParams::default();
    let bar = LinearTruss::new(Dim::D2, 200.0, 2.0, 0.0).unwrap();
    let mut elements = Vec::new();
    let mut producers = Vec::new();
    for base in 0..2 {
        let ec = [coords[base], coords[2]];
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &ec,
            params: &params,
        };
        let location: Vec<_> = [NodeId(base as u32 + 1), NodeId(3)]
            .into_iter()
            .flat_map(|node| [Dof::Ux, Dof::Uy].map(|dof| map.global_index(node, dof).unwrap()))
            .collect();
        elements.push(contribution(bar, &ctx, location.clone()));
        producers.push(geometric_producer(bar, &ctx, location).unwrap());
    }
    let mut system = LinearSystem::from_model(ReferenceBackend, &model, &elements, &[]).unwrap();
    system.set_geometric_producers(producers).unwrap();
    let mut material = SymMatrix::zeros(2).unwrap();
    let mut geometric = SymMatrix::zeros(2).unwrap();
    let u = [0.0, w]; // differs from the zero reduced reference.
    system.tangent(&u, &mut material).unwrap();
    system.geometric(&u, &mut geometric).unwrap();
    close(axial, -8.0);
    close(material.dense().get(0, 0).unwrap(), 2.0 * k * c * c);
    close(material.dense().get(1, 1).unwrap(), 2.0 * k * s * s);
    close(
        geometric.dense().get(0, 0).unwrap(),
        2.0 * axial / length * s * s,
    );
    close(
        geometric.dense().get(1, 1).unwrap(),
        2.0 * axial / length * c * c,
    );
    close(arch_force([0.0, 0.0], a, h, ea, axial)[0], 0.0);
    close(arch_force([0.0, 0.0], a, h, ea, axial)[1], -p);

    let tangent: Vec<_> = material
        .dense()
        .values()
        .iter()
        .zip(geometric.dense().values())
        .map(|(k, g)| k + g)
        .collect();
    let mut fd = [0.0; 4];
    for j in 0..2 {
        let mut plus = [0.0; 2];
        let mut minus = [0.0; 2];
        plus[j] = FD_STEP;
        minus[j] = -FD_STEP;
        let fp = arch_force(plus, a, h, ea, axial);
        let fm = arch_force(minus, a, h, ea, axial);
        for i in 0..2 {
            fd[2 * i + j] = (fp[i] - fm[i]) / (2.0 * FD_STEP);
        }
    }
    let norm = tangent.iter().fold(0.0_f64, |m, t| m.max(t.abs()));
    let error = tangent
        .iter()
        .zip(fd)
        .fold(0.0_f64, |m, (t, f)| m.max((t - f).abs()))
        / norm;
    assert!(
        error < FD_REL_TOL,
        "relative error={error}, tol={FD_REL_TOL}"
    );
    let transverse_error = (tangent[3] - fd[3]).abs() / tangent[3].abs();
    assert!(
        transverse_error < FD_REL_TOL,
        "transverse error={transverse_error}"
    );

    let lambda_cr = -ea * s * s / (axial * c * c);
    close(lambda_cr, 12.5);
    assert!(lambda_cr < -ea * c * c / (axial * s * s));
    let combined = |lambda: f64| {
        SymMatrix::new(
            DMat::new(
                2,
                2,
                material
                    .dense()
                    .values()
                    .iter()
                    .zip(geometric.dense().values())
                    .map(|(k, g)| k + lambda * g)
                    .collect(),
            )
            .unwrap(),
        )
        .unwrap()
    };
    let critical = combined(lambda_cr);
    close(critical.dense().get(1, 1).unwrap(), 0.0); // mode [0,1], residual N/m.
    close(critical.dense().get(0, 1).unwrap(), 0.0);
    assert!(critical.dense().get(0, 0).unwrap() > 0.0);
    assert!(combined(0.99 * lambda_cr).dense().get(1, 1).unwrap() > 0.0);
    assert!(combined(1.01 * lambda_cr).dense().get(1, 1).unwrap() < 0.0);
    close(lambda_cr * p, -2.0 * (lambda_cr * axial) * s);
}

#[test]
fn adapter_owns_context_and_validates_location_geometry_and_displacements() {
    let bar = LinearTruss::new(Dim::D2, 200.0, 2.0, 0.0).unwrap();
    let producer = {
        let params = ElementParams::default();
        let coords = [[0.0; 3], [2.0, 0.0, 0.0]];
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        let location = vec![DofIndex(3), DofIndex(1), DofIndex(4), DofIndex(0)];
        assert_eq!(
            geometric_producer(bar, &ctx, vec![DofIndex(0)])
                .err()
                .unwrap()
                .code,
            "ENG-E-0010"
        );
        assert_eq!(
            geometric_producer(bar, &ctx, vec![DofIndex(0); 4])
                .err()
                .unwrap()
                .code,
            "ENG-E-0002"
        );
        geometric_producer(bar, &ctx, location).unwrap()
    };
    let matrix = (producer.eval)(&[0.0, 0.0, 0.06, 0.0]).unwrap();
    close(matrix.get(3, 3).unwrap(), 6.0);
    assert_eq!((producer.eval)(&[0.0]).unwrap_err().code, "ENG-E-0010");
    assert_eq!(
        (producer.eval)(&[0.0, 0.0, f64::NAN, 0.0])
            .unwrap_err()
            .code,
        "ENG-E-0008"
    );
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &[[0.0; 3]; 2],
        params: &params,
    };
    assert_eq!(
        geometric_producer(
            bar,
            &ctx,
            vec![DofIndex(0), DofIndex(1), DofIndex(2), DofIndex(3)]
        )
        .err()
        .unwrap()
        .code,
        "ENG-E-0005"
    );
}
