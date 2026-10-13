//! Montaje con APIs públicas; los oráculos están en analytical.rs y la bitácora.
use engine_analysis::{
    combination::combine,
    results::{CaseResults, MemberRecovery},
    StaticLinear,
};
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem, PrescribedDof};
use engine_contracts::{CaseRef, Element, ElementCtx, ElementParams, MemberLoad};
use engine_core::{Dim, Dof, EResult, EngineError, MemberId, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_elements_truss::LinearTruss;
use engine_linalg::ReferenceBackend;
use engine_model::{CaseFactor, Combination, DofValue, LoadCase, Model, NodalLoad, Node, Support};

use super::table::{Fixture, Kind, Parameters, Tolerance};

pub fn missing() -> EngineError {
    EngineError::new("ENG-E-0012", "Fixture: dato o resultado interno ausente.")
}
pub struct Check(pub Tolerance);
impl Check {
    pub fn close(&self, label: &str, actual: f64, expected: f64) -> EResult<()> {
        let limit = self.0.absolute + self.0.relative * expected.abs();
        if !actual.is_finite() || !expected.is_finite() || (actual - expected).abs() > limit {
            return Err(EngineError::new("ENG-E-0012", format!(
                "{label}: obtenido={actual:.15e}, analítico={expected:.15e}, tolerancia={limit:.3e}; {}",
                self.0.justification
            )));
        }
        Ok(())
    }
}
struct Entry {
    element: Box<dyn Element>,
    nodes: [NodeId; 2],
    loads: Vec<(String, MemberLoad)>,
}
pub struct Problem {
    pub model: Model,
    entries: Vec<Entry>,
    prescribed: Vec<PrescribedDof>,
}
impl Problem {
    fn new(coords: &[[f64; 3]], truss: bool) -> Self {
        let mut model = Model::new();
        for (i, &xyz) in coords.iter().enumerate() {
            let mut node = Node::new(NodeId(i as u32 + 1), Dim::D2, xyz);
            node.dofs = if truss {
                vec![Dof::Ux, Dof::Uy]
            } else {
                Dim::D2.dofs().to_vec()
            };
            model.nodes.push(node);
        }
        model.cases.push(LoadCase {
            name: "Q".into(),
            loads: vec![],
        });
        Self {
            model,
            entries: vec![],
            prescribed: vec![],
        }
    }
    fn support(&mut self, n: u32, dofs: &[Dof]) {
        self.model.supports.push(Support {
            id: format!("s{n}"),
            node: NodeId(n),
            restrained: dofs.to_vec(),
        });
    }
    fn nodal(&mut self, case: &str, n: u32, dof: Dof, value: f64) -> EResult<()> {
        self.model
            .cases
            .iter_mut()
            .find(|c| c.name == case)
            .ok_or_else(missing)?
            .loads
            .push(NodalLoad {
                node: NodeId(n),
                values: vec![DofValue { dof, value }],
            });
        Ok(())
    }
    fn frame(
        &mut self,
        a: u32,
        b: u32,
        p: Parameters,
        releases: MomentReleases,
        loads: Vec<(String, MemberLoad)>,
    ) -> EResult<()> {
        self.entries.push(Entry {
            element: Box::new(EulerFrame2D::new(p.e, p.area, p.inertia, 0.0, releases)?),
            nodes: [NodeId(a), NodeId(b)],
            loads,
        });
        Ok(())
    }
    fn bar(&mut self, a: u32, b: u32, p: Parameters) -> EResult<()> {
        self.entries.push(Entry {
            element: Box::new(LinearTruss::new(Dim::D2, p.e, p.area, 0.0)?),
            nodes: [NodeId(a), NodeId(b)],
            loads: vec![],
        });
        Ok(())
    }
}
fn uniform(w: f64, angle: f64) -> MemberLoad {
    MemberLoad {
        force_per_length: [w * angle.sin(), -w * angle.cos(), 0.0],
        moment_per_length: [0.0; 3],
    }
}
fn q(w: f64, angle: f64) -> Vec<(String, MemberLoad)> {
    vec![("Q".into(), uniform(w, angle))]
}

pub fn build(f: &Fixture) -> EResult<Problem> {
    let p = f.parameters;
    let l = p.length;
    let all = Dim::D2.dofs();
    let none = MomentReleases::default();
    match f.kind {
        Kind::TwoBars | Kind::Triangle | Kind::Concurrent => {
            let mut xyz = vec![[-l, 0.0, 0.0], [l, 0.0, 0.0], [0.0, p.height, 0.0]];
            if matches!(f.kind, Kind::Concurrent) {
                xyz.insert(2, [0.0; 3]);
            }
            let mut b = Problem::new(&xyz, true);
            b.support(1, &[Dof::Ux, Dof::Uy]);
            b.support(
                2,
                if matches!(f.kind, Kind::Triangle) {
                    &[Dof::Uy]
                } else {
                    &[Dof::Ux, Dof::Uy]
                },
            );
            let top = if matches!(f.kind, Kind::Concurrent) {
                4
            } else {
                3
            };
            b.bar(1, top, p)?;
            b.bar(2, top, p)?;
            if matches!(f.kind, Kind::Triangle) {
                b.bar(1, 2, p)?;
            }
            if matches!(f.kind, Kind::Concurrent) {
                b.support(3, &[Dof::Ux, Dof::Uy]);
                b.bar(3, 4, p)?;
            }
            b.nodal("Q", top, Dof::Ux, p.horizontal)?;
            b.nodal("Q", top, Dof::Uy, -p.p)?;
            Ok(b)
        }
        Kind::Settlement => {
            let mut b = Problem::new(&[[0.0; 3], [l / 2.0, 0.0, 0.0], [l, 0.0, 0.0]], true);
            b.support(1, &[Dof::Ux, Dof::Uy]);
            b.support(2, &[Dof::Uy]);
            b.support(3, &[Dof::Ux, Dof::Uy]);
            b.prescribed.push(PrescribedDof {
                node: NodeId(3),
                dof: Dof::Ux,
                value: p.settlement,
            });
            b.bar(1, 2, p)?;
            b.bar(2, 3, p)?;
            Ok(b)
        }
        Kind::PortalLateral | Kind::PortalVertical => {
            let h = p.height;
            let mut b = Problem::new(
                &[
                    [0.0; 3],
                    [l, 0.0, 0.0],
                    [0.0, h, 0.0],
                    [l, h, 0.0],
                    [l / 2.0, h, 0.0],
                ],
                false,
            );
            b.support(1, all);
            b.support(2, all);
            let rigid = matches!(f.kind, Kind::PortalLateral);
            let column = Parameters {
                area: p.area * if rigid { p.rigid_ratio } else { 1.0 },
                ..p
            };
            let beam = Parameters {
                area: column.area,
                inertia: p.inertia * if rigid { p.rigid_ratio } else { 1.0 },
                ..p
            };
            b.frame(1, 3, column, none, vec![])?;
            b.frame(2, 4, column, none, vec![])?;
            for (a, z) in [(3, 5), (5, 4)] {
                b.frame(a, z, beam, none, if rigid { vec![] } else { q(p.w, 0.0) })?;
            }
            if rigid {
                b.nodal("Q", 3, Dof::Ux, p.p / 2.0)?;
                b.nodal("Q", 4, Dof::Ux, p.p / 2.0)?;
            }
            Ok(b)
        }
        Kind::Continuous => {
            let mut b = Problem::new(
                &[
                    [0.0; 3],
                    [l / 2.0, 0.0, 0.0],
                    [l, 0.0, 0.0],
                    [1.5 * l, 0.0, 0.0],
                    [2.0 * l, 0.0, 0.0],
                ],
                false,
            );
            b.support(1, &[Dof::Ux, Dof::Uy]);
            b.support(3, &[Dof::Uy]);
            b.support(5, &[Dof::Uy]);
            for n in 1..5 {
                b.frame(n, n + 1, p, none, q(p.w, 0.0))?;
            }
            Ok(b)
        }
        Kind::Release => {
            let mut b = Problem::new(&[[0.0; 3], [l, 0.0, 0.0]], false);
            b.support(1, all);
            // Rz externo desconectado fijado; el giro interno se recupera por condensación.
            b.support(2, &[Dof::Uy, Dof::Rz]);
            b.frame(1, 2, p, MomentReleases { i: false, j: true }, q(p.w, 0.0))?;
            Ok(b)
        }
        _ => {
            let point = matches!(f.kind, Kind::SimplePoint | Kind::FixedPoint);
            let x = if point { p.fraction * l } else { l / 2.0 };
            let coords: Vec<_> = [0.0, x, l]
                .into_iter()
                .map(|s| [2.0 + s * p.angle.cos(), 3.0 + s * p.angle.sin(), 0.0])
                .collect();
            let mut b = Problem::new(&coords, false);
            if matches!(f.kind, Kind::SimplePoint | Kind::SimpleUniform) {
                b.support(1, &[Dof::Ux, Dof::Uy]);
            } else {
                b.support(1, all);
            }
            match f.kind {
                Kind::SimplePoint | Kind::SimpleUniform | Kind::Propped => b.support(3, &[Dof::Uy]),
                Kind::FixedPoint | Kind::FixedUniform => b.support(3, &[Dof::Uy, Dof::Rz]),
                _ => {}
            }
            let distributed = matches!(
                f.kind,
                Kind::CantileverUniform | Kind::SimpleUniform | Kind::FixedUniform | Kind::Propped
            );
            let combo = matches!(f.kind, Kind::Combination);
            if combo {
                for name in ["W", "D"] {
                    b.model.cases.push(LoadCase {
                        name: name.into(),
                        loads: vec![],
                    });
                }
            }
            for (a, z) in [(1, 2), (2, 3)] {
                let loads = if combo {
                    vec![
                        ("W".into(), uniform(p.w, 0.0)),
                        ("D".into(), uniform(p.w * p.factors[1], 0.0)),
                    ]
                } else if distributed {
                    q(p.w, p.angle)
                } else {
                    vec![]
                };
                b.frame(a, z, p, none, loads)?;
            }
            if matches!(
                f.kind,
                Kind::CantileverPoint | Kind::SimplePoint | Kind::FixedPoint | Kind::Combination
            ) {
                let n = if point { 2 } else { 3 };
                b.nodal("Q", n, Dof::Ux, p.p * p.angle.sin())?;
                b.nodal("Q", n, Dof::Uy, -p.p * p.angle.cos())?;
                if combo {
                    b.nodal("D", 3, Dof::Uy, -p.p * p.factors[0])?;
                }
            }
            if matches!(f.kind, Kind::CantileverMoment) {
                b.nodal("Q", 3, Dof::Rz, p.p)?;
            }
            Ok(b)
        }
    }
}

pub fn solve(problem: &Problem, check: &Check) -> EResult<Vec<CaseResults>> {
    let map = DofMap::new(&problem.model.nodes)?;
    let params = ElementParams::default();
    let mut contributions = Vec::new();
    for e in &problem.entries {
        let coords = [map.coordinates(e.nodes[0])?, map.coordinates(e.nodes[1])?];
        let ctx = ElementCtx {
            dim: Dim::D2,
            coords: &coords,
            params: &params,
        };
        let mut location = Vec::new();
        for n in e.nodes {
            for &d in e.element.dofs_per_node() {
                location.push(map.global_index(n, d)?);
            }
        }
        contributions.push(ElementContribution {
            stiffness: e
                .element
                .respond(&ctx, &vec![0.0; location.len()], &[])?
                .tangent,
            location,
            mass_lumped: None,
            mass_consistent: None,
            geometric: None,
            loads: e
                .loads
                .iter()
                .map(|(case, load)| {
                    Ok(ElementLoad {
                        case: CaseRef(case.clone()),
                        values: e.element.load_vector(&ctx, load)?,
                    })
                })
                .collect::<EResult<Vec<_>>>()?,
        });
    }
    let mut system = LinearSystem::from_model(
        ReferenceBackend,
        &problem.model,
        &contributions,
        &problem.prescribed,
    )?;
    let cases: Vec<_> = problem
        .model
        .cases
        .iter()
        .map(|c| (CaseRef(c.name.clone()), 1.0))
        .collect();
    let static_results = StaticLinear::run(&mut system, &cases)?;
    let mut recovered = Vec::new();
    // Rejilla común deliberada para LOAD-002; las raíces adicionales vienen de RES-001.
    let stations = [0.0, 0.2, 0.8, 1.0];
    for result in &static_results.cases {
        check.close(
            "residual libre relativo",
            result.relative_free_residual,
            0.0,
        )?;
        check.close(
            "residual global relativo",
            result.equilibrium.relative_residual,
            0.0,
        )?;
        let loads: Vec<Vec<_>> = problem
            .entries
            .iter()
            .map(|e| {
                e.loads
                    .iter()
                    .filter(|(name, _)| name == &result.case.0)
                    .map(|(_, q)| q.clone())
                    .collect()
            })
            .collect();
        let members: Vec<_> = problem
            .entries
            .iter()
            .zip(&loads)
            .enumerate()
            .map(|(i, (e, q))| MemberRecovery {
                id: MemberId(i as u32 + 1),
                nodes: &e.nodes,
                element: e.element.as_ref(),
                dim: Dim::D2,
                params: &params,
                committed: &[],
                loads: q,
                stations: Some(&stations),
            })
            .collect();
        let r = CaseResults::recover(result, &system, &members)?;
        equilibrium(problem, &r, check)?;
        let identity = combine(
            &Combination {
                id: "identidad".into(),
                terms: vec![CaseFactor {
                    case: result.case.0.clone(),
                    factor: 1.0,
                }],
            },
            std::slice::from_ref(&r),
        )?;
        compare(&identity.results, &r, check)?;
        recovered.push(r);
    }
    Ok(recovered)
}

// Balance independiente sobre cargas físicas y resultados recuperados, incluyendo brazos.
fn equilibrium(problem: &Problem, r: &CaseResults, c: &Check) -> EResult<()> {
    let mut fx = 0.0;
    let mut fy = 0.0;
    let mut mz = 0.0;
    for n in &r.nodes {
        let xyz = problem
            .model
            .nodes
            .iter()
            .find(|v| v.id == n.id)
            .ok_or_else(missing)?
            .coordinates;
        fx += n.reactions[0];
        fy += n.reactions[1];
        mz += n.reactions[5] + xyz[0] * n.reactions[1] - xyz[1] * n.reactions[0];
    }
    let case = problem
        .model
        .cases
        .iter()
        .find(|v| v.name == r.case.0)
        .ok_or_else(missing)?;
    for load in &case.loads {
        let xyz = problem
            .model
            .nodes
            .iter()
            .find(|v| v.id == load.node)
            .ok_or_else(missing)?
            .coordinates;
        for v in &load.values {
            match v.dof {
                Dof::Ux => {
                    fx += v.value;
                    mz -= xyz[1] * v.value;
                }
                Dof::Uy => {
                    fy += v.value;
                    mz += xyz[0] * v.value;
                }
                Dof::Rz => mz += v.value,
                _ => return Err(missing()),
            }
        }
    }
    for e in &problem.entries {
        let a = problem
            .model
            .nodes
            .iter()
            .find(|v| v.id == e.nodes[0])
            .ok_or_else(missing)?
            .coordinates;
        let b = problem
            .model
            .nodes
            .iter()
            .find(|v| v.id == e.nodes[1])
            .ok_or_else(missing)?
            .coordinates;
        let l = (b[0] - a[0]).hypot(b[1] - a[1]);
        for (_, q) in e.loads.iter().filter(|(name, _)| name == &r.case.0) {
            let x = q.force_per_length[0] * l;
            let y = q.force_per_length[1] * l;
            fx += x;
            fy += y;
            mz += (a[0] + b[0]) / 2.0 * y - (a[1] + b[1]) / 2.0 * x + q.moment_per_length[2] * l;
        }
    }
    c.close("ΣFx físico", fx, 0.0)?;
    c.close("ΣFy físico", fy, 0.0)?;
    c.close("ΣMz físico", mz, 0.0)
}

pub fn compare(a: &CaseResults, b: &CaseResults, c: &Check) -> EResult<()> {
    c.close(
        "cantidad de nodos",
        a.nodes.len() as f64,
        b.nodes.len() as f64,
    )?;
    c.close(
        "cantidad de miembros",
        a.members.len() as f64,
        b.members.len() as f64,
    )?;
    for n in &a.nodes {
        let other = b.nodes.iter().find(|v| v.id == n.id).ok_or_else(missing)?;
        for (x, y) in n.displacements.iter().zip(other.displacements) {
            c.close("superposición u", *x, y)?;
        }
        for (x, y) in n.reactions.iter().zip(other.reactions) {
            c.close("superposición R", *x, y)?;
        }
    }
    for m in &a.members {
        let other = b
            .members
            .iter()
            .find(|v| v.id == m.id)
            .ok_or_else(missing)?;
        c.close(
            "tamaño vector de acciones",
            m.end_forces_local.len() as f64,
            other.end_forces_local.len() as f64,
        )?;
        for (x, y) in m.end_forces_local.iter().zip(&other.end_forces_local) {
            c.close("superposición extremo", *x, *y)?;
        }
        for station in &m.stations {
            let other = other
                .stations
                .iter()
                .find(|v| (v.position - station.position).abs() < 1e-12)
                .ok_or_else(missing)?;
            for (x, y) in station.forces_local.iter().zip(other.forces_local) {
                c.close("superposición estación", *x, y)?;
            }
        }
    }
    Ok(())
}
