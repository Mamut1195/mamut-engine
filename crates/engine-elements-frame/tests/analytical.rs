//! Soluciones cerradas, ensamblaje real y backend del repo; sin resultados externos.
mod fixtures;
use engine_assembly::{
    assemble::assemble_matrix, DofMap, ElementContribution, ElementLoad, LinearSystem,
};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MassKind, MemberLoad};
use engine_core::{Dim, Dof, DofIndex, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend, SymMatrix};
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};
use fixtures::{A, E, I, L, P, RHO, W};

fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}; tol={tol}");
}
fn beam(releases: MomentReleases) -> EulerFrame2D {
    EulerFrame2D::new(E, A, I, RHO, releases).unwrap()
}
fn uniform(q: f64) -> MemberLoad {
    MemberLoad {
        force_per_length: [0.0, q, 0.0],
        moment_per_length: [0.0; 3],
    }
}
struct Solution {
    u: Vec<f64>,
    r: Vec<[f64; 3]>,
}
fn solve(
    coords: &[[f64; 3]],
    members: &[(usize, usize, MomentReleases)],
    restraints: &[(usize, Vec<Dof>)],
    nodal: &[(usize, Dof, f64)],
    distributed: Option<&MemberLoad>,
) -> Solution {
    fixtures::cited(&fixtures::STATIC);
    let mut model = Model::new();
    for (i, &p) in coords.iter().enumerate() {
        model
            .nodes
            .push(Node::new(NodeId(i as u32 + 1), Dim::D2, p));
    }
    for (i, (node, restrained)) in restraints.iter().enumerate() {
        model.supports.push(Support {
            id: format!("s{i}"),
            node: NodeId(*node as u32 + 1),
            restrained: restrained.clone(),
        });
    }
    model.cases.push(LoadCase {
        name: "Q".into(),
        loads: nodal
            .iter()
            .map(|&(n, dof, value)| NodalLoad {
                node: NodeId(n as u32 + 1),
                values: vec![DofValue { dof, value }],
            })
            .collect(),
    });
    let map = DofMap::new(&model.nodes).unwrap();
    let params = ElementParams::default();
    let case = CaseRef("Q".into());
    let contributions: Vec<_> = members
        .iter()
        .map(|&(a, b, rel)| {
            let ec = [coords[a], coords[b]];
            let ctx = ElementCtx {
                dim: Dim::D2,
                coords: &ec,
                params: &params,
            };
            let element = beam(rel);
            let location = [a, b]
                .into_iter()
                .flat_map(|n| {
                    let map = &map;
                    element
                        .dofs_per_node()
                        .iter()
                        .map(move |&dof| map.global_index(NodeId(n as u32 + 1), dof).unwrap())
                })
                .collect();
            ElementContribution {
                location,
                stiffness: element.respond(&ctx, &[0.0; 6], &[]).unwrap().tangent,
                mass_lumped: element.mass(&ctx, MassKind::Lumped).unwrap(),
                mass_consistent: element.mass(&ctx, MassKind::Consistent).unwrap(),
                geometric: None,
                loads: distributed
                    .into_iter()
                    .map(|load| ElementLoad {
                        case: case.clone(),
                        values: element.load_vector(&ctx, load).unwrap(),
                    })
                    .collect(),
            }
        })
        .collect();
    let mut system =
        LinearSystem::from_model(ReferenceBackend, &model, &contributions, &[]).unwrap();
    let reduced = system.solve(&case, 1.0).unwrap();
    let u = system.constraints().expand(&reduced).unwrap();
    let mut r = vec![[0.0; 3]; coords.len()];
    for reaction in system.reactions(&reduced, &case, 1.0).unwrap() {
        let axis = match reaction.dof {
            Dof::Ux => 0,
            Dof::Uy => 1,
            Dof::Rz => 2,
            _ => unreachable!(),
        };
        r[reaction.node.0 as usize - 1][axis] = reaction.value;
    }
    // ΣFx,ΣFy y ΣMz alrededor del origen con cargas distribuidas integradas.
    let mut equilibrium = [0.0; 3];
    for (&p, &r) in coords.iter().zip(&r) {
        equilibrium[0] += r[0];
        equilibrium[1] += r[1];
        equilibrium[2] += r[2] + p[0] * r[1] - p[1] * r[0];
    }
    for &(n, dof, value) in nodal {
        match dof {
            Dof::Ux => {
                equilibrium[0] += value;
                equilibrium[2] -= coords[n][1] * value;
            }
            Dof::Uy => {
                equilibrium[1] += value;
                equilibrium[2] += coords[n][0] * value;
            }
            Dof::Rz => equilibrium[2] += value,
            _ => unreachable!(),
        }
    }
    if let Some(load) = distributed {
        for &(a, b, _) in members {
            let l = (coords[b][0] - coords[a][0]).hypot(coords[b][1] - coords[a][1]);
            let q = load.force_per_length;
            equilibrium[0] += q[0] * l;
            equilibrium[1] += q[1] * l;
            equilibrium[2] += l
                * ((coords[a][0] + coords[b][0]) * q[1] / 2.0
                    - (coords[a][1] + coords[b][1]) * q[0] / 2.0
                    + load.moment_per_length[2]);
        }
    }
    for residual in equilibrium {
        close(residual, 0.0, fixtures::STATIC.force_tolerance);
    }
    Solution { u, r }
}
fn full() -> Vec<Dof> {
    vec![Dof::Ux, Dof::Uy, Dof::Rz]
}
fn beam_fixture(rel_i: bool, rel_j: bool, q: Option<&MemberLoad>, point: f64) -> Solution {
    let coords = [[0.0; 3], [L / 2.0, 0.0, 0.0], [L, 0.0, 0.0]];
    // Rz exterior liberado no tiene aporte: restringir este GDL desconectado
    // solo evita singularidad; el giro interno se obtiene mediante recuperación.
    solve(
        &coords,
        &[
            (0, 1, MomentReleases { i: rel_i, j: false }),
            (1, 2, MomentReleases { i: false, j: rel_j }),
        ],
        &[(0, full()), (2, full())],
        &[(1, Dof::Uy, point)],
        q,
    )
}
#[test]
fn cantilever_tip_force_and_rotation() {
    let s = solve(
        &[[0.0; 3], [L, 0.0, 0.0]],
        &[(0, 1, MomentReleases::default())],
        &[(0, full())],
        &[(1, Dof::Uy, -P)],
        None,
    );
    close(
        s.u[4],
        -P * L.powi(3) / (3.0 * E * I),
        fixtures::STATIC.displacement_tolerance,
    );
    close(
        s.u[5],
        -P * L.powi(2) / (2.0 * E * I),
        fixtures::STATIC.displacement_tolerance,
    );
    close(s.r[0][1], P, fixtures::STATIC.force_tolerance);
    close(s.r[0][2], P * L, fixtures::STATIC.force_tolerance);
    let params = ElementParams::default();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let out = beam(MomentReleases::default())
        .recover(
            &ElementCtx {
                dim: Dim::D2,
                coords: &coords,
                params: &params,
            },
            &s.u,
            &[],
            &[],
            &[0.0, 0.25, 1.0],
        )
        .unwrap();
    for station in out.stations {
        close(station.forces_local[1], -P, 1e-9);
        close(
            station.forces_local[5],
            -P * L * (1.0 - station.position),
            1e-9,
        );
    }
}
#[test]
fn fixed_fixed_uniform_and_point_load_by_subdivision() {
    let load = uniform(-W);
    let s = beam_fixture(false, false, Some(&load), 0.0);
    close(
        s.u[4],
        -W * L.powi(4) / (384.0 * E * I),
        fixtures::STATIC.displacement_tolerance,
    );
    close(s.r[0][2], W * L.powi(2) / 12.0, 1e-9);
    close(s.r[2][2], -W * L.powi(2) / 12.0, 1e-9);
    // Un único tramo empotrado también recupera M parabólico exacto aunque
    // sus desplazamientos nodales sean cero (no confundir con v interior).
    let params = ElementParams::default();
    let coords = [[0.0; 3], [L, 0.0, 0.0]];
    let out = beam(MomentReleases::default())
        .recover(
            &ElementCtx {
                dim: Dim::D2,
                coords: &coords,
                params: &params,
            },
            &[0.0; 6],
            &[],
            &[load],
            &[0.0, 0.5, 1.0],
        )
        .unwrap();
    for (station, m) in
        out.stations
            .iter()
            .zip([-W * L * L / 12.0, W * L * L / 24.0, -W * L * L / 12.0])
    {
        close(station.forces_local[5], m, 1e-9);
    }
    let s = beam_fixture(false, false, None, -P);
    close(s.u[4], -P * L.powi(3) / (192.0 * E * I), 1e-12);
    close(s.r[0][2], P * L / 8.0, 1e-9);
    close(s.r[2][2], -P * L / 8.0, 1e-9);
}
#[test]
fn pinned_and_propped_uniform_load_with_released_rotations() {
    let q = uniform(-W);
    let s = beam_fixture(true, true, Some(&q), 0.0);
    close(s.u[4], -5.0 * W * L.powi(4) / (384.0 * E * I), 1e-12);
    close(s.r[0][1], W * L / 2.0, 1e-9);
    close(s.r[2][1], W * L / 2.0, 1e-9);
    close(s.r[0][2], 0.0, 1e-9);
    close(s.r[2][2], 0.0, 1e-9);
    let params = ElementParams::default();
    let coords = [[0.0; 3], [L / 2.0, 0.0, 0.0]];
    let element = beam(MomentReleases { i: true, j: false });
    let c = ElementCtx {
        dim: Dim::D2,
        coords: &coords,
        params: &params,
    };
    let internal = element
        .local_displacements(&c, &s.u[..6], std::slice::from_ref(&q))
        .unwrap();
    close(internal[2], -W * L.powi(3) / (24.0 * E * I), 1e-12);
    let output = element
        .recover(&c, &s.u[..6], &[], std::slice::from_ref(&q), &[0.0, 1.0])
        .unwrap();
    close(output.end_forces_local[2], 0.0, 1e-9);
    close(output.stations[1].forces_local[5], W * L * L / 8.0, 1e-9);
    for (ri, rj) in [(false, true), (true, false)] {
        let s = beam_fixture(ri, rj, Some(&q), 0.0);
        close(s.u[4], -W * L.powi(4) / (192.0 * E * I), 1e-12);
        close(s.r[0][1], if ri { 3.0 } else { 5.0 } * W * L / 8.0, 1e-9);
        close(s.r[2][1], if rj { 3.0 } else { 5.0 } * W * L / 8.0, 1e-9);
        close(s.r[0][2], if ri { 0.0 } else { W * L * L / 8.0 }, 1e-9);
        close(s.r[2][2], if rj { 0.0 } else { -W * L * L / 8.0 }, 1e-9);
    }
}
#[test]
fn symmetric_portal_with_finite_axial_stiffness_closed_form() {
    // source/citation: STATIC. Altura h en Y del plano XY, sin gravedad.
    let h: f64 = 3.0;
    let span: f64 = 4.0;
    let ei = E * I;
    let beta = (12.0 * ei / span.powi(2)) / (E * A / h + 24.0 * ei / span.powi(3));
    let d = 4.0 * ei / h + 6.0 * ei / span - 12.0 * ei * beta / span.powi(2);
    let delta = P / (24.0 * ei / h.powi(3) - 72.0 * ei.powi(2) / (h.powi(4) * d));
    let theta = -6.0 * ei * delta / (h.powi(2) * d);
    let v = -beta * theta;
    let s = solve(
        &[[0.0; 3], [span, 0.0, 0.0], [0.0, h, 0.0], [span, h, 0.0]],
        &[
            (0, 2, MomentReleases::default()),
            (1, 3, MomentReleases::default()),
            (2, 3, MomentReleases::default()),
        ],
        &[(0, full()), (1, full())],
        &[(2, Dof::Ux, P / 2.0), (3, Dof::Ux, P / 2.0)],
        None,
    );
    for (n, y) in [(2, v), (3, -v)] {
        close(s.u[3 * n], delta, 1e-12);
        close(s.u[3 * n + 1], y, 1e-12);
        close(s.u[3 * n + 2], theta, 1e-12);
    }
    for r in &s.r[..2] {
        close(r[0], -P / 2.0, 1e-9);
    }
    close(s.r[0][1], -E * A * v / h, 1e-9);
    close(s.r[1][1], E * A * v / h, 1e-9);
}
#[test]
fn euler_pinned_column_one_and_two_elements() {
    fixtures::cited(&fixtures::BUCKLING);
    let exact = std::f64::consts::PI.powi(2) * E * I / L.powi(2);
    for (count, tolerance) in [(1, 0.22), (2, 0.008)] {
        // Hermite Ritz: 1 elemento sobreestima Euler 21.59%; 2, <0.8%.
        // Soportes articulados mantienen rotaciones nodales libres, condición
        // natural M=0; no se condensan los giros que representan el modo.
        let order = 3 * (count + 1);
        let params = ElementParams::default();
        let mut kblocks = vec![];
        let mut gblocks = vec![];
        for e in 0..count {
            let l = L / count as f64;
            let coords = [[e as f64 * l, 0.0, 0.0], [(e + 1) as f64 * l, 0.0, 0.0]];
            let ctx = ElementCtx {
                dim: Dim::D2,
                coords: &coords,
                params: &params,
            };
            let element = beam(MomentReleases::default());
            let ids = (3 * e..3 * e + 6)
                .map(|i| DofIndex(i as u32))
                .collect::<Vec<_>>();
            kblocks.push((
                ids.clone(),
                element.respond(&ctx, &[0.0; 6], &[]).unwrap().tangent,
            ));
            let kg = element
                .geometric_stiffness(&ctx, &[0.0, 0.0, 0.0, -l / (E * A), 0.0, 0.0], &[])
                .unwrap()
                .unwrap();
            gblocks.push((
                ids,
                DMat::new(6, 6, kg.values().iter().map(|x| -x).collect()).unwrap(),
            ));
        }
        let mut backend = ReferenceBackend;
        let kr: Vec<_> = kblocks.iter().map(|(a, b)| (a.as_slice(), b)).collect();
        let gr: Vec<_> = gblocks.iter().map(|(a, b)| (a.as_slice(), b)).collect();
        let k = assemble_matrix(&mut backend, order, &kr)
            .unwrap()
            .to_dense()
            .unwrap();
        let g = assemble_matrix(&mut backend, order, &gr)
            .unwrap()
            .to_dense()
            .unwrap();
        let ids: Vec<_> = (0..order)
            .filter(|i| i % 3 == 2 || (i % 3 == 1 && *i != 1 && *i != order - 2))
            .collect();
        let reduce = |a: &DMat| {
            SymMatrix::new(
                DMat::new(
                    ids.len(),
                    ids.len(),
                    ids.iter()
                        .flat_map(|&i| ids.iter().map(move |&j| a.get(i, j).unwrap()))
                        .collect(),
                )
                .unwrap(),
            )
            .unwrap()
        };
        let modes = backend.generalized_eigen(&reduce(&k), &reduce(&g)).unwrap();
        let pcr = modes.eigenvalues[0];
        assert!(
            (pcr / exact - 1.0).abs() < tolerance,
            "n={count}, Pcr={pcr}, exact={exact}, relative tol={tolerance}"
        );
        assert!(pcr >= exact * (1.0 - 1e-12));
        if count == 1 {
            close(
                pcr,
                12.0 * E * I / L.powi(2),
                fixtures::BUCKLING.force_tolerance,
            );
        }
        // Residual del autovector con tolerancia SI explícita.
        let vector = DMat::new(
            ids.len(),
            1,
            (0..ids.len())
                .map(|i| modes.vectors.get(i, 0).unwrap())
                .collect(),
        )
        .unwrap();
        let kv = backend.matmul(reduce(&k).dense(), &vector).unwrap();
        let gv = backend.matmul(reduce(&g).dense(), &vector).unwrap();
        for (a, b) in kv.values().iter().zip(gv.values()) {
            close(*a, pcr * b, 1e-8);
        }
    }
}
