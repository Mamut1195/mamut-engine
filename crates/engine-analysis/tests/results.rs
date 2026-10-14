//! Validación analítica propia del postproceso; sin red, archivos ni oráculos.
//! Derivaciones SI y convenciones en provenance/ENG-RES-001.md.
use engine_analysis::{
    results::{CaseResults, Field, MemberRecovery, MemberResult, DEFAULT_STATIONS},
    StaticCaseResult, StaticLinear,
};
use engine_assembly::{DofMap, ElementContribution, ElementLoad, LinearSystem, PrescribedDof};
use engine_contracts::{
    CaseRef, DMat, Element, ElementCtx, ElementOutput, ElementParams, ElementResponse, MassKind,
    MemberLoad,
};
use engine_core::math;
use engine_core::{ContractId, Dim, Dof, EResult, EngineError, EntityRef, MemberId, NodeId};
use engine_elements_frame::{EulerFrame2D, MomentReleases};
use engine_elements_truss::LinearTruss;
use engine_linalg::ReferenceBackend;
use engine_model::{DofValue, LoadCase, Model, NodalLoad, Node, Support};

const L: f64 = 4.0;
const W: f64 = 6.0;
const P: f64 = 12.0;
const IDS: [NodeId; 2] = [NodeId(10), NodeId(30)];
const MID: MemberId = MemberId(77);
const DOFS: [Dof; 3] = [Dof::Ux, Dof::Uy, Dof::Rz];

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

fn missing() -> EngineError {
    EngineError::new("ENG-E-0012", "Fixture sin resultado esperado.")
}

fn model(coords: &[(NodeId, [f64; 3])], dim: Dim, dofs: &[Dof]) -> Model {
    let mut m = Model::new();
    for &(id, xyz) in coords.iter().rev() {
        let mut node = Node::new(id, dim, xyz);
        node.dofs = dofs.to_vec();
        m.nodes.push(node);
    }
    m.cases.push(LoadCase {
        name: "Q".into(),
        loads: vec![],
    });
    m
}

fn support(m: &mut Model, node: NodeId, dofs: &[Dof]) {
    m.supports.push(Support {
        id: format!("s{}", node.0),
        node,
        restrained: dofs.to_vec(),
    });
}

fn nodal(m: &mut Model, node: NodeId, dof: Dof, value: f64) -> EResult<()> {
    m.cases
        .first_mut()
        .ok_or_else(missing)?
        .loads
        .push(NodalLoad {
            node,
            values: vec![DofValue { dof, value }],
        });
    Ok(())
}

fn contribution(
    m: &Model,
    element: &dyn Element,
    nodes: &[NodeId],
    loads: &[MemberLoad],
) -> EResult<ElementContribution> {
    let map = DofMap::new(&m.nodes)?;
    let coords = nodes
        .iter()
        .map(|&n| map.coordinates(n))
        .collect::<EResult<Vec<_>>>()?;
    let params = ElementParams::default();
    let ctx = ElementCtx {
        dim: m.nodes.first().ok_or_else(missing)?.dim,
        coords: &coords,
        params: &params,
    };
    let mut location = Vec::new();
    for &node in nodes {
        for &dof in element.dofs_per_node() {
            location.push(map.global_index(node, dof)?);
        }
    }
    Ok(ElementContribution {
        stiffness: element
            .respond(&ctx, &vec![0.0; location.len()], &[])?
            .tangent,
        location,
        mass_lumped: None,
        mass_consistent: None,
        geometric: None,
        loads: loads
            .iter()
            .map(|load| {
                Ok(ElementLoad {
                    case: CaseRef("Q".into()),
                    values: element.load_vector(&ctx, load)?,
                })
            })
            .collect::<EResult<Vec<_>>>()?,
    })
}

fn solve(
    m: &Model,
    contributions: &[ElementContribution],
    prescribed: &[PrescribedDof],
    factor: f64,
) -> EResult<(LinearSystem<ReferenceBackend>, StaticCaseResult)> {
    let mut system = LinearSystem::from_model(ReferenceBackend, m, contributions, prescribed)?;
    let result = StaticLinear::run(&mut system, &[(CaseRef("Q".into()), factor)])?
        .cases
        .into_iter()
        .next()
        .ok_or_else(missing)?;
    assert!(result.equilibrium.relative_residual < 1e-12);
    Ok((system, result))
}

fn beam() -> EResult<EulerFrame2D> {
    EulerFrame2D::new(200.0, 2.0, 3.0, 0.0, MomentReleases::default())
}

fn uniform() -> MemberLoad {
    MemberLoad {
        force_per_length: [0.0, -W, 0.0],
        moment_per_length: [0.0; 3],
    }
}

fn view<'a>(
    element: &'a dyn Element,
    params: &'a ElementParams,
    loads: &'a [MemberLoad],
    stations: Option<&'a [f64]>,
) -> MemberRecovery<'a> {
    MemberRecovery {
        id: MID,
        nodes: &IDS,
        element,
        dim: Dim::D2,
        params,
        committed: &[],
        loads,
        stations,
    }
}

fn member(results: &CaseResults) -> EResult<&MemberResult> {
    results.members.first().ok_or_else(missing)
}

fn extremes(m: &MemberResult, field: Field) -> EResult<&engine_analysis::results::FieldExtremes> {
    m.extremes
        .iter()
        .find(|e| e.field == field)
        .ok_or_else(missing)
}

fn station(m: &MemberResult, position: f64) -> EResult<&engine_analysis::results::StationResult> {
    m.stations
        .iter()
        .find(|s| (s.position - position).abs() < 1e-12)
        .ok_or_else(missing)
}

fn simple_model() -> Model {
    let mut m = model(
        &[(IDS[0], [0.0; 3]), (IDS[1], [L, 0.0, 0.0])],
        Dim::D2,
        &DOFS,
    );
    support(&mut m, IDS[0], &[Dof::Ux, Dof::Uy]);
    support(&mut m, IDS[1], &[Dof::Uy]);
    m
}

#[test]
fn simply_supported_uniform_exact_peak_outside_requested_stations() -> EResult<()> {
    let m = simple_model();
    let beam = beam()?;
    let loads = [uniform()];
    let params = ElementParams::default();
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    let requested = [0.8, 0.2, 0.2]; // Sin centro ni extremos, desordenadas y repetidas.
    let r = CaseResults::recover(
        &solution,
        &system,
        &[view(&beam, &params, &loads, Some(&requested))],
    )?;
    let m = member(&r)?;
    assert_eq!(
        m.stations.iter().map(|s| s.position).collect::<Vec<_>>(),
        [0.0, 0.2, 0.5, 0.8, 1.0]
    );
    let moment = extremes(m, Field::Mz)?;
    close(moment.max.value, W * L * L / 8.0);
    close(moment.max.at_station, 0.5);
    close(moment.min.value, 0.0);
    let shear = extremes(m, Field::Vy)?;
    close(shear.min.value, -W * L / 2.0);
    close(shear.min.at_station, 0.0);
    close(shear.max.value, W * L / 2.0);
    close(shear.max.at_station, 1.0);
    // Se comprueba la parábola en una estación solicitada, no interpolación de extremos.
    close(
        station(m, 0.2)?.forces_local[5],
        W * L * L * 0.2 * 0.8 / 2.0,
    );
    for &i in &[0, 2, 3, 5] {
        close(m.end_forces_local[i], 0.0);
    }
    close(m.end_forces_local[1], W * L / 2.0);
    close(m.end_forces_local[4], W * L / 2.0);
    assert_eq!(r.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), IDS);
    for n in &r.nodes {
        close(n.reactions[1], W * L / 2.0);
        close(n.displacements[1], 0.0);
        close(n.displacements[2], 0.0);
        close(n.reactions[5], 0.0);
    }
    close(
        r.nodes[0].displacements[5],
        -W * math::powi(L, 3) / (24.0 * 200.0 * 3.0),
    );
    close(
        r.nodes[1].displacements[5],
        W * math::powi(L, 3) / (24.0 * 200.0 * 3.0),
    );
    close(
        r.nodes.iter().map(|n| n.reactions[1]).sum::<f64>() - W * L,
        0.0,
    );
    close(r.nodes[1].reactions[1] * L - W * L * L / 2.0, 0.0);
    Ok(())
}

#[test]
fn cantilever_nodal_point_load_exact_n_v_m_and_node_reactions() -> EResult<()> {
    let mut m = model(
        &[(IDS[0], [2.0, 3.0, 0.0]), (IDS[1], [2.0 + L, 3.0, 0.0])],
        Dim::D2,
        &DOFS,
    );
    support(&mut m, IDS[0], &DOFS);
    nodal(&mut m, IDS[1], Dof::Ux, 5.0)?;
    nodal(&mut m, IDS[1], Dof::Uy, -P)?;
    nodal(&mut m, IDS[0], Dof::Uy, -7.0)?; // Carga directamente sobre apoyo.
    let beam = beam()?;
    let params = ElementParams::default();
    let (system, mut solution) = solve(&m, &[contribution(&m, &beam, &IDS, &[])?], &[], 1.0)?;
    // Global es la fuente de verdad: no se usan estas dos copias.
    solution.free_displacements.clear();
    solution.reactions.clear();
    let r = CaseResults::recover(&solution, &system, &[view(&beam, &params, &[], None)])?;
    let member = member(&r)?;
    assert_eq!(
        member
            .stations
            .iter()
            .map(|s| s.position)
            .collect::<Vec<_>>(),
        DEFAULT_STATIONS
    );
    for s in &member.stations {
        close(s.forces_local[0], 5.0);
        close(s.forces_local[1], -P);
        close(s.forces_local[5], -P * L * (1.0 - s.position));
        for &i in &[2, 3, 4] {
            close(s.forces_local[i], 0.0);
        }
    }
    for (&actual, expected) in member
        .end_forces_local
        .iter()
        .zip([-5.0, P, P * L, 5.0, -P, 0.0])
    {
        close(actual, expected);
    }
    close(extremes(member, Field::Mz)?.min.value, -P * L);
    close(extremes(member, Field::Mz)?.min.at_station, 0.0);
    close(extremes(member, Field::Mz)?.max.at_station, 1.0);
    close(extremes(member, Field::N)?.min.at_station, 0.0); // Empate determinista.
    close(r.nodes[0].reactions[0], -5.0);
    close(r.nodes[0].reactions[1], P + 7.0);
    close(r.nodes[0].reactions[5], P * L);
    assert_eq!(r.nodes[1].reactions, [0.0; 6]);
    close(r.nodes[1].displacements[0], 5.0 * L / (200.0 * 2.0));
    close(
        r.nodes[1].displacements[1],
        -P * math::powi(L, 3) / (3.0 * 200.0 * 3.0),
    );
    close(
        r.nodes[1].displacements[5],
        -P * math::powi(L, 2) / (2.0 * 200.0 * 3.0),
    );
    close(r.nodes[0].reactions[1] - P - 7.0, 0.0);
    close(r.nodes[0].reactions[5] - P * L, 0.0);
    Ok(())
}

#[test]
fn fixed_fixed_uniform_exact_end_and_center_moments() -> EResult<()> {
    let mut m = simple_model();
    m.supports.clear();
    support(&mut m, IDS[0], &DOFS);
    // Fijo en flexión; Ux_j libre sin carga axial conserva una ecuación.
    support(&mut m, IDS[1], &[Dof::Uy, Dof::Rz]);
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [uniform()];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    let r = CaseResults::recover(
        &solution,
        &system,
        &[view(&beam, &params, &loads, Some(&[]))],
    )?;
    let member = member(&r)?;
    close(member.end_forces_local[2], W * L * L / 12.0);
    close(member.end_forces_local[5], -W * L * L / 12.0);
    close(extremes(member, Field::Mz)?.min.value, -W * L * L / 12.0);
    close(extremes(member, Field::Mz)?.max.value, W * L * L / 24.0);
    close(extremes(member, Field::Mz)?.max.at_station, 0.5);
    close(station(member, 0.5)?.forces_local[1], 0.0);
    for node in &r.nodes {
        assert_eq!(node.displacements, [0.0; 6]);
    }
    close(r.nodes[0].reactions[5], W * L * L / 12.0);
    close(r.nodes[1].reactions[5], -W * L * L / 12.0);
    close(r.nodes.iter().map(|n| n.reactions[1]).sum::<f64>(), W * L);
    close(
        r.nodes[0].reactions[5] + r.nodes[1].reactions[5] + r.nodes[1].reactions[1] * L
            - W * L * L / 2.0,
        0.0,
    );
    Ok(())
}

#[test]
fn two_bar_truss_axial_static_equilibrium_and_queries() -> EResult<()> {
    let apex = NodeId(90);
    let mut m = model(
        &[
            (IDS[0], [-4.0, 0.0, 0.0]),
            (IDS[1], [4.0, 0.0, 0.0]),
            (apex, [0.0, 3.0, 0.0]),
        ],
        Dim::D2,
        &[Dof::Ux, Dof::Uy],
    );
    support(&mut m, IDS[0], &[Dof::Ux, Dof::Uy]);
    support(&mut m, IDS[1], &[Dof::Ux, Dof::Uy]);
    nodal(&mut m, apex, Dof::Uy, -P)?;
    let bar = LinearTruss::new(Dim::D2, 200.0, 2.0, 0.0)?;
    let pairs = [[IDS[0], apex], [IDS[1], apex]];
    let contributions = pairs
        .iter()
        .map(|pair| contribution(&m, &bar, pair, &[]))
        .collect::<EResult<Vec<_>>>()?;
    let (system, solution) = solve(&m, &contributions, &[], 1.0)?;
    let params = ElementParams::default();
    let mut views = [
        view(&bar, &params, &[], None),
        view(&bar, &params, &[], Some(&[0.3])),
    ];
    views[0].nodes = &pairs[0];
    views[1].nodes = &pairs[1];
    views[1].id = MemberId(17);
    let r = CaseResults::recover(&solution, &system, &views)?;
    assert_eq!(
        r.members.iter().map(|m| m.id).collect::<Vec<_>>(),
        [MemberId(17), MID]
    );
    let n = -P * 5.0 / (2.0 * 3.0);
    for member in &r.members {
        for s in &member.stations {
            assert_eq!(s.forces_local[1..], [0.0; 5]);
            close(s.forces_local[0], n);
        }
        close(member.end_forces_local[0], -n);
        close(member.end_forces_local[2], n);
        close(extremes(member, Field::N)?.max.value, n);
        close(extremes(member, Field::N)?.min.value, n);
    }
    close(r.nodes[0].reactions[0], P * 4.0 / 6.0);
    close(r.nodes[1].reactions[0], -P * 4.0 / 6.0);
    close(r.nodes[0].reactions[1] + r.nodes[1].reactions[1], P);
    assert_eq!(r.nodes[2].reactions, [0.0; 6]);
    let q = r.query(
        Some(&[apex, apex]),
        Some(&[MID]),
        Some(&[Field::N, Field::Reactions, Field::N]),
    )?;
    assert_eq!(q.nodes.len(), 1);
    assert_eq!(q.nodes[0].id, apex);
    assert_eq!(q.nodes[0].displacements, None);
    assert_eq!(q.nodes[0].reactions, Some([0.0; 6]));
    assert_eq!(q.members.len(), 1);
    assert_eq!(q.members[0].id, MID);
    assert_eq!(q.members[0].end_forces_local, None);
    assert_eq!(q.members[0].extremes.len(), 1);
    for s in &q.members[0].stations {
        assert_eq!(s.values.len(), 1);
        assert_eq!(s.values[0].field, Field::N);
        close(s.values[0].value, n);
    }
    let all = r.query(None, None, None)?;
    assert_eq!(all.nodes.len(), 3);
    assert_eq!(all.members.len(), 2);
    assert_eq!(all.members[0].extremes.len(), 6);
    assert!(all.members[0].end_forces_local.is_some());
    assert_eq!(all.members[0].stations[0].values.len(), 6);
    let empty = r.query(Some(&[]), Some(&[]), None)?;
    assert!(empty.nodes.is_empty() && empty.members.is_empty());
    let no_fields = r.query(None, None, Some(&[]))?;
    assert!(no_fields
        .nodes
        .iter()
        .all(|n| n.displacements.is_none() && n.reactions.is_none()));
    assert!(no_fields
        .members
        .iter()
        .all(|m| m.end_forces_local.is_none() && m.stations.is_empty() && m.extremes.is_empty()));
    let ends = r.query(Some(&[]), None, Some(&[Field::EndForcesLocal]))?;
    assert!(ends
        .members
        .iter()
        .all(|m| m.end_forces_local.is_some() && m.stations.is_empty()));
    assert_code(r.query(Some(&[NodeId(999)]), None, None), "ENG-E-0003");
    assert_code(r.query(None, Some(&[MemberId(999)]), None), "ENG-E-0003");
    Ok(())
}

fn assert_code<T>(result: EResult<T>, expected: &str) {
    assert!(result.is_err(), "Se esperaba {expected}");
    if let Err(e) = result {
        assert_eq!(e.code, expected);
        assert!(engine_core::catalog::lookup(e.code).is_some());
    }
}

#[test]
fn interior_peak_moves_with_nodal_end_couple() -> EResult<()> {
    let mut m = simple_model();
    nodal(&mut m, IDS[1], Dof::Rz, 8.0)?;
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [uniform()];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    let r = CaseResults::recover(
        &solution,
        &system,
        &[view(&beam, &params, &loads, Some(&[0.1, 0.9]))],
    )?;
    let m = member(&r)?;
    // M(x)=14x−3x²; la raíz no está en la mitad ni en las estaciones pedidas.
    let peak = extremes(m, Field::Mz)?.max;
    close(peak.at_station, 7.0 / 12.0);
    close(peak.value, 14.0 * 14.0 / (2.0 * W));
    close(station(m, 7.0 / 12.0)?.forces_local[1], 0.0);
    close(r.nodes[0].reactions[1], 14.0);
    close(r.nodes[1].reactions[1], 10.0);
    Ok(())
}

#[test]
fn rotated_uniform_beam_sum_loads_and_positive_negative_zero_factors() -> EResult<()> {
    // Viga vertical local y=−X: carga +X => qy=−W. Dos cargas se suman.
    let mut m = model(
        &[(IDS[0], [2.0, 1.0, 0.0]), (IDS[1], [2.0, 1.0 + L, 0.0])],
        Dim::D2,
        &DOFS,
    );
    support(&mut m, IDS[0], &[Dof::Ux, Dof::Uy]);
    support(&mut m, IDS[1], &[Dof::Ux]);
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [
        MemberLoad {
            force_per_length: [W / 3.0, 0.0, 0.0],
            moment_per_length: [0.0; 3],
        },
        MemberLoad {
            force_per_length: [2.0 * W / 3.0, 0.0, 0.0],
            moment_per_length: [0.0; 3],
        },
    ];
    let contributions = [contribution(&m, &beam, &IDS, &loads)?];
    for factor in [2.0, -1.5, 0.0] {
        let (system, solution) = solve(&m, &contributions, &[], factor)?;
        let r = CaseResults::recover(
            &solution,
            &system,
            &[view(&beam, &params, &loads, Some(&[0.1]))],
        )?;
        assert_eq!(r.factor, factor);
        close(
            r.nodes.iter().map(|n| n.reactions[0]).sum::<f64>(),
            -factor * W * L,
        );
        let m = member(&r)?;
        if factor != 0.0 {
            close(station(m, 0.5)?.forces_local[5], factor * W * L * L / 8.0);
            let e = extremes(m, Field::Mz)?;
            let peak = if factor > 0.0 { e.max } else { e.min };
            close(peak.value, factor * W * L * L / 8.0);
            close(peak.at_station, 0.5);
        } else {
            for s in &m.stations {
                assert_eq!(s.forces_local, [0.0; 6]);
            }
        }
    }
    Ok(())
}

#[test]
fn distributed_couple_adds_shear_zero_and_true_moment_extreme() -> EResult<()> {
    let mut m = simple_model();
    m.supports.clear();
    support(&mut m, IDS[0], &DOFS);
    // Fijo en flexión; Ux_j libre sin carga axial conserva una ecuación.
    support(&mut m, IDS[1], &[Dof::Uy, Dof::Rz]);
    let beam = beam()?;
    let params = ElementParams::default();
    let mut load = uniform();
    load.moment_per_length[2] = 3.0;
    let loads = [load];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    let r = CaseResults::recover(
        &solution,
        &system,
        &[view(&beam, &params, &loads, Some(&[]))],
    )?;
    let m = member(&r)?;
    close(station(m, 0.625)?.forces_local[1], 0.0);
    close(station(m, 0.5)?.forces_local[1], -3.0);
    close(extremes(m, Field::Mz)?.max.value, W * L * L / 24.0);
    close(extremes(m, Field::Mz)?.max.at_station, 0.5);
    Ok(())
}

#[test]
fn prescribed_displacement_included_without_scaling() -> EResult<()> {
    let middle = NodeId(20);
    let mut m = model(
        &[
            (IDS[0], [0.0; 3]),
            (middle, [L / 2.0, 0.0, 0.0]),
            (IDS[1], [L, 0.0, 0.0]),
        ],
        Dim::D2,
        &[Dof::Ux, Dof::Uy],
    );
    support(&mut m, IDS[0], &[Dof::Ux, Dof::Uy]);
    support(&mut m, IDS[1], &[Dof::Uy]);
    support(&mut m, middle, &[Dof::Uy]);
    let bar = LinearTruss::new(Dim::D2, 200.0, 2.0, 0.0)?;
    let params = ElementParams::default();
    let prescribed = [PrescribedDof {
        node: IDS[1],
        dof: Dof::Ux,
        value: 0.03,
    }];
    let pairs = [[IDS[0], middle], [middle, IDS[1]]];
    let contributions = pairs
        .iter()
        .map(|p| contribution(&m, &bar, p, &[]))
        .collect::<EResult<Vec<_>>>()?;
    let (system, solution) = solve(&m, &contributions, &prescribed, 2.0)?;
    let mut views = [
        view(&bar, &params, &[], None),
        view(&bar, &params, &[], None),
    ];
    views[0].nodes = &pairs[0];
    views[1].nodes = &pairs[1];
    views[1].id = MemberId(17);
    let r = CaseResults::recover(&solution, &system, &views)?;
    close(r.nodes[1].displacements[0], 0.015);
    close(r.nodes[2].displacements[0], 0.03);
    close(r.nodes[0].reactions[0], -3.0);
    close(r.nodes[2].reactions[0], 3.0);
    close(extremes(member(&r)?, Field::N)?.max.value, 3.0);
    Ok(())
}

#[test]
fn invalid_inputs_catalog_diagnostics_without_panics() -> EResult<()> {
    let m = simple_model();
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [uniform()];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    for value in [-0.01, 1.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let stations = [value];
        let result = CaseResults::recover(
            &solution,
            &system,
            &[view(&beam, &params, &loads, Some(&stations))],
        );
        let code = if value.is_finite() {
            "ENG-E-0010"
        } else {
            "ENG-E-0008"
        };
        if let Err(e) = &result {
            assert!(e
                .entities
                .iter()
                .any(|entity| matches!(entity, EntityRef::Member(id) if *id == MID)));
        }
        assert_code(result, code);
    }
    let duplicated = [
        view(&beam, &params, &loads, None),
        view(&beam, &params, &loads, None),
    ];
    assert_code(
        CaseResults::recover(&solution, &system, &duplicated),
        "ENG-E-0002",
    );
    let mut v = view(&beam, &params, &loads, None);
    v.nodes = &[IDS[0], NodeId(999)];
    assert_code(CaseResults::recover(&solution, &system, &[v]), "ENG-E-0003");
    let mut v = view(&beam, &params, &loads, None);
    v.nodes = &[IDS[0]];
    assert_code(CaseResults::recover(&solution, &system, &[v]), "ENG-E-0010");
    let mut v = view(&beam, &params, &loads, None);
    v.nodes = &[IDS[0], IDS[0]];
    assert_code(CaseResults::recover(&solution, &system, &[v]), "ENG-E-0002");
    let mut v = view(&beam, &params, &loads, None);
    v.committed = &[0.0];
    assert_code(CaseResults::recover(&solution, &system, &[v]), "ENG-E-0010");
    let mut invalid_load = uniform();
    invalid_load.force_per_length[0] = f64::NAN;
    assert_code(
        CaseResults::recover(
            &solution,
            &system,
            &[view(&beam, &params, &[invalid_load], None)],
        ),
        "ENG-E-0008",
    );
    let mut invalid_load = uniform();
    invalid_load.force_per_length[2] = 1.0;
    assert_code(
        CaseResults::recover(
            &solution,
            &system,
            &[view(&beam, &params, &[invalid_load], None)],
        ),
        "ENG-E-0013",
    );
    for change in 0..5 {
        let mut bad = solution.clone();
        match change {
            0 => {
                bad.global_displacements.pop();
            }
            1 => bad.global_displacements[2] = f64::NAN,
            2 => bad.factor = f64::INFINITY,
            3 => bad.global_displacements[0] = 0.1,
            _ => bad.case = CaseRef("ausente".into()),
        }
        let code = match change {
            0 | 3 => "ENG-E-0010",
            1 | 2 => "ENG-E-0008",
            _ => "ENG-E-0003",
        };
        assert_code(
            CaseResults::recover(&bad, &system, &[view(&beam, &params, &loads, None)]),
            code,
        );
    }
    assert_eq!(system.accepted_displacement(), &[0.0; 3]);
    assert_eq!(system.trial_displacement(), &[0.0; 3]);
    Ok(())
}

#[test]
fn recovery_rejects_omitted_or_changed_loads_and_members() -> EResult<()> {
    let mut m = simple_model();
    // Incluye carga nodal directa en apoyo y en un GDL libre (par).
    nodal(&mut m, IDS[0], Dof::Uy, -7.0)?;
    nodal(&mut m, IDS[1], Dof::Rz, 8.0)?;
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [uniform()];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    let changed = [MemberLoad {
        force_per_length: [0.0, -W / 2.0, 0.0],
        moment_per_length: [0.0; 3],
    }];
    for supplied in [&[][..], &changed[..]] {
        let result =
            CaseResults::recover(&solution, &system, &[view(&beam, &params, supplied, None)]);
        let e = result.as_ref().err().ok_or_else(missing)?;
        assert!(e
            .entities
            .iter()
            .any(|e| matches!(e, EntityRef::Case(name) if name == "Q")));
        assert!(e
            .entities
            .iter()
            .any(|e| matches!(e, EntityRef::Node(id) if *id == IDS[0])));
        assert!(e
            .entities
            .iter()
            .any(|e| matches!(e, EntityRef::Member(id) if *id == MID)));
        assert!(e.entities.iter().any(|e| matches!(e, EntityRef::Dof(_))));
        assert_code(result, "ENG-E-0010");
    }
    assert_code(CaseResults::recover(&solution, &system, &[]), "ENG-E-0010");
    // Elemento con rigidez distinta a la ensamblada también rompe equilibrio.
    let wrong_beam = EulerFrame2D::new(400.0, 2.0, 3.0, 0.0, MomentReleases::default())?;
    assert_code(
        CaseResults::recover(
            &solution,
            &system,
            &[view(&wrong_beam, &params, &loads, None)],
        ),
        "ENG-E-0010",
    );
    let coherent = CaseResults::recover(&solution, &system, &[view(&beam, &params, &loads, None)])?;
    close(coherent.nodes[0].reactions[1], 21.0);
    close(
        extremes(member(&coherent)?, Field::Mz)?.max.value,
        math::powi(14.0_f64, 2) / (2.0 * W),
    );
    Ok(())
}

#[test]
fn omitted_load_probe_errors_while_uniform_analytic_peak_is_twelve() -> EResult<()> {
    let m = simple_model();
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [uniform()];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    // Sonda #69: antes, omitir loads producía Mmax=8 en vez de wL²/8=12.
    assert_code(
        CaseResults::recover(&solution, &system, &[view(&beam, &params, &[], None)]),
        "ENG-E-0010",
    );
    let coherent = CaseResults::recover(
        &solution,
        &system,
        &[view(&beam, &params, &loads, Some(&[]))],
    )?;
    let peak = extremes(member(&coherent)?, Field::Mz)?.max;
    close(peak.value, W * math::powi(L, 2) / 8.0);
    close(peak.at_station, 0.5);
    Ok(())
}

#[derive(Clone, Copy)]
enum OutputFault {
    EndSize,
    StationSize,
    Position,
    NegativeZero,
    InternalSize,
    LoadSize,
}

// Elemento propio de prueba: delega física al frame del repo y altera solo
// la frontera de salida indicada; no consulta implementaciones externas.
struct FaultyElement {
    beam: EulerFrame2D,
    fault: OutputFault,
}

impl Element for FaultyElement {
    fn kind(&self) -> &'static str {
        self.beam.kind()
    }
    fn contract(&self) -> ContractId {
        self.beam.contract()
    }
    fn n_nodes(&self) -> usize {
        self.beam.n_nodes()
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        self.beam.dofs_per_node()
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[f64],
        committed: &[f64],
    ) -> EResult<ElementResponse> {
        let mut output = self.beam.respond(ctx, u, committed)?;
        if matches!(self.fault, OutputFault::InternalSize) {
            output.internal_force.pop();
        }
        Ok(output)
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        self.beam.mass(ctx, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[f64],
        committed: &[f64],
    ) -> EResult<Option<DMat>> {
        self.beam.geometric_stiffness(ctx, u, committed)
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<f64>> {
        let mut output = self.beam.load_vector(ctx, load)?;
        if matches!(self.fault, OutputFault::LoadSize) {
            output.pop();
        }
        Ok(output)
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[f64],
        committed: &[f64],
        loads: &[MemberLoad],
        stations: &[f64],
    ) -> EResult<ElementOutput> {
        let mut output = self.beam.recover(ctx, u, committed, loads, stations)?;
        match self.fault {
            OutputFault::EndSize => {
                output.end_forces_local.pop();
            }
            OutputFault::StationSize => {
                output.stations.pop();
            }
            OutputFault::Position => {
                output.stations.first_mut().ok_or_else(missing)?.position = 0.1;
            }
            OutputFault::NegativeZero => {
                output.stations.first_mut().ok_or_else(missing)?.position = -0.0;
            }
            OutputFault::InternalSize | OutputFault::LoadSize => {}
        }
        Ok(output)
    }
}

#[test]
fn negative_zero_input_and_output_are_canonical_and_ties_choose_smallest_station() -> EResult<()> {
    let m = simple_model();
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [uniform()];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    let faulty = FaultyElement {
        beam: self::beam()?,
        fault: OutputFault::NegativeZero,
    };
    for element in [&beam as &dyn Element, &faulty as &dyn Element] {
        let r = CaseResults::recover(
            &solution,
            &system,
            &[view(element, &params, &loads, Some(&[1.0, -0.0, 0.0, 0.8]))],
        )?;
        let m = member(&r)?;
        assert_eq!(m.stations[0].position.to_bits(), 0.0_f64.to_bits());
        assert_eq!(m.stations.iter().filter(|s| s.position == 0.0).count(), 1);
        // N, Vz, T y My son exactamente constantes: empate en todas las estaciones.
        for field in [Field::N, Field::Vz, Field::T, Field::My] {
            let e = extremes(m, field)?;
            assert_eq!(e.min.at_station.to_bits(), 0.0_f64.to_bits());
            assert_eq!(e.max.at_station.to_bits(), 0.0_f64.to_bits());
        }
        let query = r.query(None, None, Some(&[Field::Mz]))?;
        assert_eq!(
            query.members[0].stations[0].position.to_bits(),
            0.0_f64.to_bits()
        );
    }
    Ok(())
}

#[test]
fn malformed_element_outputs_report_internal_error_with_member() -> EResult<()> {
    let m = simple_model();
    let beam = beam()?;
    let params = ElementParams::default();
    let loads = [uniform()];
    let (system, solution) = solve(&m, &[contribution(&m, &beam, &IDS, &loads)?], &[], 1.0)?;
    for fault in [
        OutputFault::EndSize,
        OutputFault::StationSize,
        OutputFault::Position,
        OutputFault::InternalSize,
        OutputFault::LoadSize,
    ] {
        let faulty = FaultyElement {
            beam: self::beam()?,
            fault,
        };
        let result =
            CaseResults::recover(&solution, &system, &[view(&faulty, &params, &loads, None)]);
        let e = result.as_ref().err().ok_or_else(missing)?;
        assert!(e
            .entities
            .iter()
            .any(|e| matches!(e, EntityRef::Member(id) if *id == MID)));
        assert_code(result, "ENG-E-0012");
    }
    Ok(())
}
