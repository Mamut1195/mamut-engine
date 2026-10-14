//! Soluciones cerradas propias y ensamblaje real; sin oráculos ni fuentes externas.
mod fixtures;
use engine_assembly::{
    assemble::assemble_matrix, DofMap, ElementContribution, ElementLoad, LinearSystem,
};
use engine_contracts::{CaseRef, Element, ElementParams, MassKind, SystemView};
use engine_core::math;
use engine_core::{Dim, Dof, DofIndex, NodeId};
use engine_elements_frame3d::{geometric_producer, MomentReleases};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
use fixtures::*;

#[test]
fn cantilever_biaxial_and_torsion_closed_forms_with_real_assembly() {
    cited();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let p = ElementParams::default();
    let c = ctx(&coords, &p);
    let b = beam(MomentReleases::default());
    let (py, pz, t) = (12.0, -15.0, 9.0);
    let mut model = Model::new();
    model.nodes = vec![
        Node::new(NodeId(1), Dim::D3, coords[0]),
        Node::new(NodeId(2), Dim::D3, coords[1]),
    ];
    model.supports.push(Support {
        id: "base".into(),
        node: NodeId(1),
        restrained: b.dofs_per_node().to_vec(),
    });
    model.cases.push(LoadCase {
        name: "Q".into(),
        loads: vec![NodalLoad {
            node: NodeId(2),
            values: vec![
                DofValue {
                    dof: Dof::Uy,
                    value: py,
                },
                DofValue {
                    dof: Dof::Uz,
                    value: pz,
                },
                DofValue {
                    dof: Dof::Rx,
                    value: t,
                },
            ],
        }],
    });
    let map = DofMap::new(&model.nodes).unwrap();
    let location: Vec<_> = [NodeId(1), NodeId(2)]
        .into_iter()
        .flat_map(|node| {
            b.dofs_per_node()
                .iter()
                .map(|&dof| map.global_index(node, dof).unwrap())
                .collect::<Vec<_>>()
        })
        .collect();
    let contribution = ElementContribution {
        location: location.clone(),
        stiffness: b.respond(&c, &[0.0; 12], &[]).unwrap().tangent,
        mass_lumped: b.mass(&c, MassKind::Lumped).unwrap(),
        mass_consistent: b.mass(&c, MassKind::Consistent).unwrap(),
        geometric: None,
        loads: vec![],
    };
    let mut system =
        LinearSystem::from_model(ReferenceBackend, &model, &[contribution], &[]).unwrap();
    let case = CaseRef("Q".into());
    let solution = system.solve(&case, 1.0).unwrap();
    let u = system.constraints().expand(&solution).unwrap();
    close(u[7], py * math::powi(L, 3) / (3.0 * E * IZ));
    close(u[8], pz * math::powi(L, 3) / (3.0 * E * IY));
    close(u[9], t * L / (G * J));
    close(u[10], -pz * math::powi(L, 2) / (2.0 * E * IY));
    close(u[11], py * math::powi(L, 2) / (2.0 * E * IZ));
    for reaction in system.reactions(&solution, &case, 1.0).unwrap() {
        let expected = match reaction.dof {
            Dof::Ux => 0.0,
            Dof::Uy => -py,
            Dof::Uz => -pz,
            Dof::Rx => -t,
            Dof::Ry => pz * L,
            Dof::Rz => -py * L,
        };
        close(reaction.value, expected);
    }
    let out = b.recover(&c, &u, &[], &[], &[0.0, 0.25, 1.0]).unwrap();
    for s in out.stations {
        for (value, expected) in s.forces_local.into_iter().zip([
            0.0,
            py,
            pz,
            t,
            -pz * L * (1.0 - s.position),
            py * L * (1.0 - s.position),
        ]) {
            close(value, expected);
        }
    }
    // El productor se registra y el sistema evalúa el axial del estado total.
    system
        .set_geometric_producers(vec![geometric_producer(b, &c, location).unwrap()])
        .unwrap();
    let mut reduced = solution;
    reduced[0] = 0.06;
    let mut kg = SymMatrix::zeros(6).unwrap();
    system.geometric(&reduced, &mut kg).unwrap();
    close(
        kg.dense().get(1, 1).unwrap(),
        (E * A * 0.06 / L) * 36.0 / (30.0 * L),
    );
}

#[test]
fn uniform_fixed_end_reactions_and_section_equilibrium() {
    cited();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let p = ElementParams::default();
    let c = ctx(&coords, &p);
    let b = beam(MomentReleases::default());
    let q = [6.0, -12.0, 15.0];
    let m = [3.0, 2.0, -4.0];
    let loading = load(q, m);
    let f = b.load_vector(&c, &loading).unwrap();
    let expected = [
        q[0] * L / 2.0,
        q[1] * L / 2.0 - m[2],
        q[2] * L / 2.0 + m[1],
        m[0] * L / 2.0,
        -q[2] * L * L / 12.0,
        q[1] * L * L / 12.0,
        q[0] * L / 2.0,
        q[1] * L / 2.0 + m[2],
        q[2] * L / 2.0 - m[1],
        m[0] * L / 2.0,
        q[2] * L * L / 12.0,
        -q[1] * L * L / 12.0,
    ];
    for (v, e) in f.iter().zip(expected) {
        close(*v, e);
    }
    let out = b
        .recover(
            &c,
            &[0.0; 12],
            &[],
            std::slice::from_ref(&loading),
            &[1.0, 0.0, 0.5, 0.5],
        )
        .unwrap();
    for (v, e) in out.end_forces_local.iter().zip(&f) {
        close(*v, -e);
    }
    for a in 0..6 {
        close(out.stations[0].forces_local[a], out.end_forces_local[6 + a]);
        close(out.stations[1].forces_local[a], -out.end_forces_local[a]);
    }
    assert_eq!(out.stations[2], out.stations[3]);
    // Dos elementos para dejar GDL interiores libres en el ensamblador.
    let nodes = [[0.0; 3], [L / 2.0, 0.0, 0.0], [L, 0.0, 0.0]];
    let mut model = Model::new();
    for (i, &coord) in nodes.iter().enumerate() {
        let node = NodeId(i as u32 + 1);
        model.nodes.push(Node::new(node, Dim::D3, coord));
        if i != 1 {
            model.supports.push(Support {
                id: format!("s{i}"),
                node,
                restrained: b.dofs_per_node().to_vec(),
            });
        }
    }
    model.cases.push(LoadCase {
        name: "Q".into(),
        loads: vec![],
    });
    let case = CaseRef("Q".into());
    let qload = load(q, [0.0; 3]);
    let contributions: Vec<_> = (0..2)
        .map(|e| {
            let ec = [nodes[e], nodes[e + 1]];
            let c = ctx(&ec, &p);
            ElementContribution {
                location: (6 * e..6 * e + 12).map(|i| DofIndex(i as u32)).collect(),
                stiffness: b.respond(&c, &[0.0; 12], &[]).unwrap().tangent,
                mass_lumped: None,
                mass_consistent: None,
                geometric: None,
                loads: vec![ElementLoad {
                    case: case.clone(),
                    values: b.load_vector(&c, &qload).unwrap(),
                }],
            }
        })
        .collect();
    let mut system =
        LinearSystem::from_model(ReferenceBackend, &model, &contributions, &[]).unwrap();
    let u = system.solve(&case, 1.0).unwrap();
    for r in system.reactions(&u, &case, 1.0).unwrap() {
        let s = if r.node == NodeId(1) { 1.0 } else { -1.0 };
        let expected = match r.dof {
            Dof::Ux => -q[0] * L / 2.0,
            Dof::Uy => -q[1] * L / 2.0,
            Dof::Uz => -q[2] * L / 2.0,
            Dof::Rx => 0.0,
            Dof::Ry => s * q[2] * L * L / 12.0,
            Dof::Rz => -s * q[1] * L * L / 12.0,
        };
        close(r.value, expected);
    }
}

#[test]
fn releases_uniform_pinned_and_propped_in_both_planes() {
    cited();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let p = ElementParams::default();
    let c = ctx(&coords, &p);
    for (axis, v, sign) in [(2, 1, 1.0), (1, 2, -1.0)] {
        let mut q = [0.0; 3];
        q[v] = -6.0;
        let loading = load(q, [0.0; 3]);
        for (ri, rj) in [(true, false), (false, true), (true, true)] {
            let mut r = MomentReleases::default();
            r.i[axis] = ri;
            r.j[axis] = rj;
            let b = beam(r);
            let out = b
                .recover(
                    &c,
                    &[0.0; 12],
                    &[],
                    std::slice::from_ref(&loading),
                    &[0.0, 0.5, 1.0],
                )
                .unwrap();
            let (ai, aj) = if ri && rj {
                (0.5, 0.5)
            } else if ri {
                (0.375, 0.625)
            } else {
                (0.625, 0.375)
            };
            close(out.end_forces_local[v], 6.0 * L * ai);
            close(out.end_forces_local[v + 6], 6.0 * L * aj);
            let mi = if ri { 0.0 } else { sign * 6.0 * L * L / 8.0 };
            let mj = if rj { 0.0 } else { -sign * 6.0 * L * L / 8.0 };
            close(out.end_forces_local[axis + 3], mi);
            close(out.end_forces_local[axis + 9], mj);
            if ri && rj {
                close(
                    out.stations[1].forces_local[axis + 3],
                    sign * 6.0 * L * L / 8.0,
                );
            }
        }
    }
    // Torsión uniforme con un extremo libre: todo el par llega al conservado.
    for end in [0, 1] {
        let mut r = MomentReleases::default();
        if end == 0 {
            r.i[0] = true;
        } else {
            r.j[0] = true;
        }
        let b = beam(r);
        let loading = load([0.0; 3], [6.0, 0.0, 0.0]);
        let out = b
            .recover(
                &c,
                &[0.0; 12],
                &[],
                std::slice::from_ref(&loading),
                &[0.0, 1.0],
            )
            .unwrap();
        close(out.end_forces_local[3 + 6 * end], 0.0);
        close(out.end_forces_local[9 - 6 * end], -6.0 * L);
        let d = b.local_displacements(&c, &[0.0; 12], &[loading]).unwrap();
        close(d[3 + 6 * end], 6.0 * L * L / (2.0 * G * J));
    }
}

#[test]
fn euler_critical_both_planes_mesh_convergence_and_mode_residual() {
    cited();
    for (count, tolerance) in [(1, 0.22), (2, 0.008), (4, 0.0006)] {
        let order = 6 * (count + 1);
        let p = ElementParams::default();
        let mut kb = vec![];
        let mut gb = vec![];
        for e in 0..count {
            let l = L / count as f64;
            let coords = [[e as f64 * l, 0.0, 0.0], [(e + 1) as f64 * l, 0.0, 0.0]];
            let c = ctx(&coords, &p);
            let b = beam(MomentReleases::default());
            let ids: Vec<_> = (6 * e..6 * e + 12).map(|i| DofIndex(i as u32)).collect();
            kb.push((ids.clone(), b.respond(&c, &[0.0; 12], &[]).unwrap().tangent));
            let mut u = [0.0; 12];
            u[6] = -l / (E * A);
            let kg = b.geometric_stiffness(&c, &u, &[]).unwrap().unwrap();
            gb.push((
                ids,
                DMat::new(12, 12, kg.values().iter().map(|g| -g).collect()).unwrap(),
            ));
        }
        let mut backend = ReferenceBackend;
        let kr: Vec<_> = kb.iter().map(|(ids, k)| (ids.as_slice(), k)).collect();
        let gr: Vec<_> = gb.iter().map(|(ids, k)| (ids.as_slice(), k)).collect();
        let k = assemble_matrix(&mut backend, order, &kr)
            .unwrap()
            .to_dense()
            .unwrap();
        let g = assemble_matrix(&mut backend, order, &gr)
            .unwrap()
            .to_dense()
            .unwrap();
        for (v, r, inertia) in [(1, 5, IZ), (2, 4, IY)] {
            let ids: Vec<_> = (0..order)
                .filter(|i| i % 6 == r || (i % 6 == v && *i != v && *i != order - 6 + v))
                .collect();
            let k = SymMatrix::new(submatrix(&k, &ids, &ids)).unwrap();
            let g = SymMatrix::new(submatrix(&g, &ids, &ids)).unwrap();
            let modes = backend.generalized_eigen(&k, &g).unwrap();
            let pcr = modes.eigenvalues[0];
            let exact = math::powi(std::f64::consts::PI, 2) * E * inertia / (L * L);
            assert!(
                (pcr / exact - 1.0).abs() < tolerance,
                "n={count}: {pcr} vs {exact}"
            );
            if count == 1 {
                close(pcr, 12.0 * E * inertia / (L * L));
            }
            let phi: Vec<_> = (0..ids.len())
                .map(|i| modes.vectors.get(i, 0).unwrap())
                .collect();
            for (kv, gv) in multiply(k.dense(), &phi)
                .into_iter()
                .zip(multiply(g.dense(), &phi))
            {
                close(kv, pcr * gv);
            }
        }
    }
}
