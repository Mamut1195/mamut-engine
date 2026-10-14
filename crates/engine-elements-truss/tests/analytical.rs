//! Integración de elementos reales con ensamblaje y backend; solo soluciones cerradas.
mod fixtures;
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem};
use engine_contracts::{
    CaseRef, Element, ElementCtx, ElementParams, MassKind, MemberLoad, SystemView,
};
use engine_core::math;
use engine_core::{Dim, Dof, NodeId};
use engine_elements_truss::LinearTruss;
use engine_linalg::{ReferenceBackend, SymMatrix};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const FORCE_TOL: f64 = 1e-9; // N absoluta, incluido ΣF por dirección.
const DISP_TOL: f64 = 1e-12; // m absoluta en estos fixtures pequeños.

fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}; tol={tol}");
}
fn rotation(p: [f64; 3], spatial: bool) -> [f64; 3] {
    if spatial {
        // Rotación no trivial alrededor de Y; todas las bases del trípode fijas.
        let (s, c) = math::sin_cos(0.73_f64);
        [c * p[0] + s * p[2], p[1], -s * p[0] + c * p[2]]
    } else {
        [-p[1], p[0], p[2]]
    } // 90° en XY: rodillo Uy pasa a Ux.
}

fn solve_fixture(
    f: &fixtures::Fixture,
    dim: Dim,
    rotated: bool,
) -> (Vec<f64>, Vec<f64>, Vec<[f64; 3]>) {
    fixtures::cited(f);
    let bar = LinearTruss::new(dim, 200.0, 2.0, 3.0).unwrap();
    let dofs = bar.dofs_per_node();
    let coords: Vec<_> = f
        .coords
        .iter()
        .map(|&p| {
            if rotated {
                rotation(p, dim == Dim::D3)
            } else {
                p
            }
        })
        .collect();
    let mut m = Model::new();
    for (i, &p) in coords.iter().enumerate() {
        let mut node = Node::new(NodeId(i as u32 + 1), dim, p);
        node.dofs = dofs.to_vec();
        m.nodes.push(node);
    }
    let n = coords.len();
    let prescribed: Vec<Vec<Dof>> = match n {
        2 => vec![dofs.to_vec(), dofs[1..].to_vec()],
        3 => vec![dofs.to_vec(), vec![if rotated { Dof::Ux } else { Dof::Uy }]],
        4 => vec![dofs.to_vec(); 3],
        _ => unreachable!(),
    };
    for (i, restrained) in prescribed.into_iter().enumerate() {
        m.supports.push(Support {
            id: format!("base{i}"),
            node: NodeId(i as u32 + 1),
            restrained,
        });
    }
    let force = if rotated {
        rotation(f.force, dim == Dim::D3)
    } else {
        f.force
    };
    m.cases.push(LoadCase {
        name: "P".into(),
        loads: vec![NodalLoad {
            node: NodeId(n as u32),
            values: dofs
                .iter()
                .enumerate()
                .map(|(i, &dof)| DofValue {
                    dof,
                    value: force[i],
                })
                .collect(),
        }],
    });
    let map = DofMap::new(&m.nodes).unwrap();
    let params = ElementParams::default();
    let contributions: Vec<_> = f
        .members
        .iter()
        .map(|&[a, b]| {
            let ec = [coords[a], coords[b]];
            let ctx = ElementCtx {
                dim,
                coords: &ec,
                params: &params,
            };
            let location = [a, b]
                .iter()
                .flat_map(|&node| {
                    let map = &map;
                    dofs.iter()
                        .map(move |&dof| map.global_index(NodeId(node as u32 + 1), dof).unwrap())
                })
                .collect();
            ElementContribution {
                location,
                stiffness: bar
                    .respond(&ctx, &vec![0.0; 2 * dofs.len()], &[])
                    .unwrap()
                    .tangent,
                mass_lumped: bar.mass(&ctx, MassKind::Lumped).unwrap(),
                mass_consistent: bar.mass(&ctx, MassKind::Consistent).unwrap(),
                geometric: None,
                loads: vec![],
            }
        })
        .collect();
    let mut system = LinearSystem::from_model(ReferenceBackend, &m, &contributions, &[]).unwrap();
    let case = CaseRef("P".into());
    let u = system.solve(&case, 1.0).unwrap();
    let full = system.constraints().expand(&u).unwrap();
    let mut reactions = vec![[0.0; 3]; n];
    for r in system.reactions(&u, &case, 1.0).unwrap() {
        reactions[r.node.0 as usize - 1][r.dof.index()] = r.value;
    }
    for axis in 0..dofs.len() {
        close(
            reactions.iter().map(|r| r[axis]).sum::<f64>() + force[axis],
            0.0,
            FORCE_TOL,
        );
    }
    // Equilibrio de momentos en los tres ejes; producto vectorial r×F.
    let mut moment = [0.0; 3];
    for (i, p) in coords.iter().enumerate() {
        let f: [f64; 3] =
            std::array::from_fn(|a| reactions[i][a] + if i + 1 == n { force[a] } else { 0.0 });
        for a in 0..3 {
            moment[a] += p[(a + 1) % 3] * f[(a + 2) % 3] - p[(a + 2) % 3] * f[(a + 1) % 3];
        }
    }
    for m in moment {
        close(m, 0.0, FORCE_TOL);
    }
    let axial = f
        .members
        .iter()
        .map(|&[a, b]| {
            let ec = [coords[a], coords[b]];
            let ctx = ElementCtx {
                dim,
                coords: &ec,
                params: &params,
            };
            let ue: Vec<_> = [a, b]
                .iter()
                .flat_map(|&i| full[i * dofs.len()..(i + 1) * dofs.len()].iter().copied())
                .collect();
            let out = bar.recover(&ctx, &ue, &[], &[], &[1.0, 0.0, 0.5]).unwrap();
            let axial = out.stations[0].forces_local[0];
            for station in &out.stations {
                close(station.forces_local[0], axial, FORCE_TOL);
            }
            close(out.end_forces_local[0], -axial, FORCE_TOL);
            close(out.end_forces_local[dofs.len()], axial, FORCE_TOL);
            axial
        })
        .collect();
    (full, axial, reactions)
}

#[test]
fn axial_extension_pl_over_ea_2d_and_3d() {
    for dim in [Dim::D2, Dim::D3] {
        let (u, n, r) = solve_fixture(&fixtures::AXIAL, dim, false);
        let axes = if dim == Dim::D2 { 2 } else { 3 };
        close(u[axes], 0.06, DISP_TOL);
        close(n[0], 12.0, FORCE_TOL);
        close(r[0][0], -12.0, FORCE_TOL);
    }
}

#[test]
fn three_bar_isostatic_triangle_and_rotated_model() {
    let root2 = math::sqrt(2.0_f64);
    for rotated in [false, true] {
        let (u, n, r) = solve_fixture(&fixtures::TRIANGLE, Dim::D2, rotated);
        for (actual, expected) in n.iter().zip([9.0, -3.0 * root2, -9.0 * root2]) {
            close(*actual, expected, FORCE_TOL);
        }
        for (actual, expected) in r.iter().zip([[-6.0, 3.0, 0.0], [0.0, 9.0, 0.0], [0.0; 3]]) {
            let expected = if rotated {
                rotation(expected, false)
            } else {
                expected
            };
            for axis in 0..3 {
                close(actual[axis], expected[axis], FORCE_TOL);
            }
        }
        let expected = [
            [0.0; 3],
            [0.045, 0.0, 0.0],
            [0.0225 + 0.015 * root2, -0.0225 - 0.03 * root2, 0.0],
        ];
        for (node, p) in expected.into_iter().enumerate() {
            let p = if rotated { rotation(p, false) } else { p };
            for axis in 0..2 {
                close(u[2 * node + axis], p[axis], DISP_TOL);
            }
        }
    }
}

#[test]
fn isostatic_tripod_and_rotated_model() {
    for rotated in [false, true] {
        let (u, n, r) = solve_fixture(&fixtures::TRIPOD, Dim::D3, rotated);
        for (actual, expected) in
            n.iter()
                .zip([-9.0 * math::sqrt(2.0_f64), 0.0, -3.0 * math::sqrt(3.0_f64)])
        {
            close(*actual, expected, FORCE_TOL);
        }
        for (actual, expected) in
            r.iter()
                .zip([[-9.0, 0.0, 9.0], [0.0; 3], [3.0, 3.0, 3.0], [0.0; 3]])
        {
            let expected = if rotated {
                rotation(expected, true)
            } else {
                expected
            };
            for axis in 0..3 {
                close(actual[axis], expected[axis], FORCE_TOL);
            }
        }
        let z = -3.0 * math::sqrt(3.0_f64) / 400.0 - 3.0 * math::sqrt(2.0_f64) / 200.0;
        let expected = [z + 9.0 * math::sqrt(2.0_f64) / 200.0, z, z];
        let expected = if rotated {
            rotation(expected, true)
        } else {
            expected
        };
        for axis in 0..3 {
            close(u[9 + axis], expected[axis], DISP_TOL);
        }
    }
}

#[test]
fn distributed_axial_load_assembly_reactions_and_geometric_snapshot() {
    // source: equilibrio propio con carga terminal P=12 N y q=4 N/m.
    // citation: provenance/ENG-ELM-001.md, Derivaciones propias; bibliografía PENDIENTE.
    // L=2 m: F_j=12+4=16 N, u_j=16/200=0.08 m, R_i=-20 N.
    // N(x/L)=16+4(1-2x/L); N(0)=20, N(0.5)=16, N(1)=12 N.
    fixtures::cited(&fixtures::AXIAL);
    for dim in [Dim::D2, Dim::D3] {
        let bar = LinearTruss::new(dim, 200.0, 2.0, 3.0).unwrap();
        let dofs = bar.dofs_per_node();
        let axes = dofs.len();
        let params = ElementParams::default();
        let ctx = ElementCtx {
            dim,
            coords: fixtures::AXIAL.coords,
            params: &params,
        };
        let load = MemberLoad {
            force_per_length: [4.0, 0.0, 0.0],
            moment_per_length: [0.0; 3],
        };
        let case = CaseRef("P".into());
        let mut model = Model::new();
        for (i, &coordinates) in fixtures::AXIAL.coords.iter().enumerate() {
            let mut node = Node::new(NodeId(i as u32 + 1), dim, coordinates);
            node.dofs = dofs.to_vec();
            model.nodes.push(node);
            model.supports.push(Support {
                id: format!("s{i}"),
                node: NodeId(i as u32 + 1),
                restrained: if i == 0 {
                    dofs.to_vec()
                } else {
                    dofs[1..].to_vec()
                },
            });
        }
        model.cases.push(LoadCase {
            name: "P".into(),
            loads: vec![NodalLoad {
                node: NodeId(2),
                values: vec![DofValue {
                    dof: Dof::Ux,
                    value: 12.0,
                }],
            }],
        });
        let map = DofMap::new(&model.nodes).unwrap();
        let mut ue = vec![0.0; 2 * axes];
        ue[axes] = 0.08;
        let kg = bar.geometric_stiffness(&ctx, &ue, &[]).unwrap().unwrap();
        let element = ElementContribution {
            location: [NodeId(1), NodeId(2)]
                .iter()
                .flat_map(|&node| {
                    let map = &map;
                    dofs.iter()
                        .map(move |&dof| map.global_index(node, dof).unwrap())
                })
                .collect(),
            stiffness: bar
                .respond(&ctx, &vec![0.0; 2 * axes], &[])
                .unwrap()
                .tangent,
            mass_lumped: bar.mass(&ctx, MassKind::Lumped).unwrap(),
            mass_consistent: bar.mass(&ctx, MassKind::Consistent).unwrap(),
            geometric: Some(kg),
            loads: vec![ElementLoad {
                case: case.clone(),
                values: bar.load_vector(&ctx, &load).unwrap(),
            }],
        };
        let mut system =
            LinearSystem::from_model(ReferenceBackend, &model, &[element], &[]).unwrap();
        let u = system.solve(&case, 1.0).unwrap();
        close(u[0], 0.08, DISP_TOL);
        let reactions = system.reactions(&u, &case, 1.0).unwrap();
        close(
            reactions
                .iter()
                .find(|r| r.node == NodeId(1) && r.dof == Dof::Ux)
                .unwrap()
                .value,
            -20.0,
            FORCE_TOL,
        );
        close(
            reactions
                .iter()
                .filter(|r| r.dof == Dof::Ux)
                .map(|r| r.value)
                .sum::<f64>()
                + 20.0,
            0.0,
            FORCE_TOL,
        );
        for r in reactions.iter().filter(|r| r.dof != Dof::Ux) {
            close(r.value, 0.0, FORCE_TOL);
        }
        let full = system.constraints().expand(&u).unwrap();
        let out = bar
            .recover(&ctx, &full, &[], &[load], &[0.0, 0.5, 1.0])
            .unwrap();
        for (s, expected) in out.stations.iter().zip([20.0, 16.0, 12.0]) {
            close(s.forces_local[0], expected, FORCE_TOL);
        }
        let mut reduced = SymMatrix::zeros(1).unwrap();
        system.mass(MassKind::Lumped, &mut reduced).unwrap();
        close(reduced.dense().get(0, 0).unwrap(), 6.0, FORCE_TOL);
        system.mass(MassKind::Consistent, &mut reduced).unwrap();
        close(reduced.dense().get(0, 0).unwrap(), 4.0, FORCE_TOL);
        // Kg no se consulta vía SystemView::geometric(u): ver bitácora (ASM-001 la ignora u).
    }
}
