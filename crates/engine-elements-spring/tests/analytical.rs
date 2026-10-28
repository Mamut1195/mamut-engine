//! Actual assembly and backend solution against own closed forms.
mod fixtures;
use engine_assembly::{DofMap, ElementContribution, LinearSystem};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MassKind};
use engine_core::math;
use engine_core::{Dim, Dof, NodeId};
use engine_elements_spring::{LinearSpring, SpringAxes, SpringConnection};
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const DISP_TOL: f64 = 1e-12; // absolute m.
const FORCE_TOL: f64 = 1e-9; // absolute N.
fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}; tol={tol}");
}
fn scalar(k: f64, ground: bool) -> LinearSpring {
    LinearSpring::new(
        Dim::D2,
        if ground {
            SpringConnection::Ground
        } else {
            SpringConnection::BetweenNodes
        },
        k,
        &[Dof::Ux],
        SpringAxes::Global,
    )
    .unwrap()
}
fn model(count: u32, dofs: &[Dof], force: &[f64], fixed_first: bool) -> Model {
    let mut m = Model::new();
    for i in 1..=count {
        let mut node = Node::new(NodeId(i), Dim::D2, [i as f64, 0.0, 0.0]);
        node.dofs = dofs.to_vec();
        m.nodes.push(node);
    }
    if fixed_first {
        m.supports.push(Support {
            id: "base".into(),
            node: NodeId(1),
            restrained: dofs.to_vec(),
        });
    }
    m.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![NodalLoad {
            node: NodeId(count),
            values: dofs
                .iter()
                .zip(force)
                .map(|(&dof, &value)| DofValue { dof, value })
                .collect(),
        }],
    });
    m
}
fn contribution(s: &LinearSpring, nodes: &[u32], m: &Model, map: &DofMap) -> ElementContribution {
    let coords: Vec<_> = nodes
        .iter()
        .map(|id| m.nodes[*id as usize - 1].coordinates)
        .collect();
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    ElementContribution {
        location: nodes
            .iter()
            .flat_map(|&id| {
                s.dofs_per_node()
                    .iter()
                    .map(move |&dof| map.global_index(NodeId(id), dof).unwrap())
            })
            .collect(),
        stiffness: s
            .respond(&ctx, &vec![0.0; s.n_nodes() * s.dofs_per_node().len()], &[])
            .unwrap()
            .tangent,
        mass_lumped: s.mass(&ctx, MassKind::Lumped).unwrap(),
        mass_consistent: s.mass(&ctx, MassKind::Consistent).unwrap(),
        geometric: s
            .geometric_stiffness(&ctx, &vec![0.0; s.n_nodes() * s.dofs_per_node().len()], &[])
            .unwrap(),
        loads: vec![],
    }
}

#[test]
fn series_equivalent_stiffness_and_equal_internal_forces() {
    fixtures::cited();
    let m = model(3, &[Dof::Ux], &[fixtures::FORCE], true);
    let map = DofMap::new(&m.nodes).unwrap();
    let s1 = scalar(fixtures::K1, false);
    let s2 = scalar(fixtures::K2, false);
    let contributions = [
        contribution(&s1, &[1, 2], &m, &map),
        contribution(&s2, &[2, 3], &m, &map),
    ];
    let mut sys = LinearSystem::from_model(ReferenceBackend, &m, &contributions, &[]).unwrap();
    let case = CaseRef("P".into());
    let reduced = sys.solve(&case, 1.0).unwrap();
    let u = sys.constraints().expand(&reduced).unwrap();
    let equivalent = 1.0 / (1.0 / fixtures::K1 + 1.0 / fixtures::K2);
    close(u[1], fixtures::FORCE / fixtures::K1, DISP_TOL);
    close(u[2], fixtures::FORCE / equivalent, DISP_TOL);
    close(
        sys.reactions(&reduced, &case, 1.0).unwrap()[0].value,
        -fixtures::FORCE,
        FORCE_TOL,
    );
    let params = ElementParams::default();
    for (s, i) in [(s1, 0), (s2, 1)] {
        let coords = [m.nodes[i].coordinates, m.nodes[i + 1].coordinates];
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        let out = s
            .recover(&ctx, &u[i..i + 2], &[], &[], &[1.0, 0.0, 0.5])
            .unwrap();
        close(out.end_forces_local[0], -fixtures::FORCE, FORCE_TOL);
        close(out.end_forces_local[1], fixtures::FORCE, FORCE_TOL);
        for station in out.stations {
            close(station.forces_local[0], fixtures::FORCE, FORCE_TOL);
        }
    }
}

#[test]
fn parallel_equivalent_stiffness_and_force_partition() {
    fixtures::cited();
    let m = model(2, &[Dof::Ux], &[fixtures::FORCE], true);
    let map = DofMap::new(&m.nodes).unwrap();
    let s1 = scalar(fixtures::K1, false);
    let s2 = scalar(fixtures::K2, false);
    let contributions = [
        contribution(&s1, &[1, 2], &m, &map),
        contribution(&s2, &[1, 2], &m, &map),
    ];
    let mut sys = LinearSystem::from_model(ReferenceBackend, &m, &contributions, &[]).unwrap();
    let case = CaseRef("P".into());
    let u = sys.solve(&case, 1.0).unwrap();
    close(
        u[0],
        fixtures::FORCE / (fixtures::K1 + fixtures::K2),
        DISP_TOL,
    );
    close(
        sys.reactions(&u, &case, 1.0).unwrap()[0].value,
        -fixtures::FORCE,
        FORCE_TOL,
    );
    let params = ElementParams::default();
    let coords = [m.nodes[0].coordinates, m.nodes[1].coordinates];
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    for s in [s1, s2] {
        let f = s.respond(&ctx, &[0.0, u[0]], &[]).unwrap().internal_force;
        close(
            f[1],
            s.stiffness() * fixtures::FORCE / (fixtures::K1 + fixtures::K2),
            FORCE_TOL,
        );
    }
}

#[test]
fn elastic_ground_support_u_equals_f_over_k() {
    fixtures::cited();
    let m = model(1, &[Dof::Ux], &[fixtures::FORCE], false);
    let map = DofMap::new(&m.nodes).unwrap();
    let s = scalar(fixtures::K1, true);
    let contributions = [contribution(&s, &[1], &m, &map)];
    let mut sys = LinearSystem::from_model(ReferenceBackend, &m, &contributions, &[]).unwrap();
    let u = sys.solve(&CaseRef("P".into()), 1.0).unwrap();
    close(u[0], fixtures::FORCE / fixtures::K1, DISP_TOL);
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: Dim::D2,
        coords: &[[1.0, 0.0, 0.0]],
        params: &params,
    };
    let out = s.recover(&ctx, &u, &[], &[], &[0.0, 1.0]).unwrap();
    close(out.end_forces_local[0], fixtures::FORCE, FORCE_TOL);
}

#[test]
fn oblique_ground_support_covariance_of_assembled_solution() {
    fixtures::cited();
    // The perpendicular spring removes the transverse mechanism without changing
    // the requested spring's solution u=(F/k1)d.
    for angle in [0.43_f64, 1.16, -0.71] {
        let (sn, cs) = math::sin_cos(angle);
        let d = [cs, sn, 0.0];
        let m = model(
            1,
            &[Dof::Ux, Dof::Uy],
            &[fixtures::FORCE * cs, fixtures::FORCE * sn],
            false,
        );
        let map = DofMap::new(&m.nodes).unwrap();
        let s1 = LinearSpring::new(
            Dim::D2,
            SpringConnection::Ground,
            fixtures::K1,
            &[Dof::Ux],
            SpringAxes::Direction(d),
        )
        .unwrap();
        let s2 = LinearSpring::new(
            Dim::D2,
            SpringConnection::Ground,
            fixtures::K2,
            &[Dof::Ux],
            SpringAxes::Direction([-sn, cs, 0.0]),
        )
        .unwrap();
        let contributions = [
            contribution(&s1, &[1], &m, &map),
            contribution(&s2, &[1], &m, &map),
        ];
        let mut sys = LinearSystem::from_model(ReferenceBackend, &m, &contributions, &[]).unwrap();
        let u = sys.solve(&CaseRef("P".into()), 1.0).unwrap();
        close(u[0], fixtures::FORCE / fixtures::K1 * cs, DISP_TOL);
        close(u[1], fixtures::FORCE / fixtures::K1 * sn, DISP_TOL);
    }
}
