//! Structural results in SI and queries for a linear static case.
//!
//! Owned data with public fields, without element references in the output.
//! Prepared for future serialization: the `results/1` schema, JSON,
//! hashes, and serialized provenance belong to ENG-CORE-005, still pending.
//! This module builds on ENG-ANL-001 and does not implement that boundary.
//!
//! Stations are normalized from −0.0 to +0.0, sorted, and deduplicated,
//! always including the ends.
//! The default is `[0, 0.25, 0.5, 0.75, 1]`. For `frame2d.euler_elastic`,
//! exact roots of shear and of the moment derivative under uniform loads
//! are added; for other elements, extrema are those of the recovered
//! stations (without promising interior extrema of arbitrary fields).
//! Local signs are those of `Element::recover`: ends as actions
//! on the member, stations on the positive face `[N,Vy,Vz,T,My,Mz]`.

use crate::{finite, StaticCaseResult};
use engine_assembly::LinearSystem;
use engine_contracts::{CaseRef, Element, ElementCtx, ElementOutput, ElementParams, MemberLoad};
use engine_core::math;
use engine_core::{Dim, DofIndex, EResult, EngineError, EntityRef, MemberId, NodeId, Real};
use engine_linalg::LinAlgBackend;
use std::collections::{BTreeMap, BTreeSet};

/// Dimensionless stations used when no list is provided.
pub const DEFAULT_STATIONS: [Real; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// View of the solved member; must match the same model/case as the system.
/// Does not assemble, accept, or modify history. Loads are scaled once
/// by `StaticCaseResult::factor`, including zero or negative factors.
/// The collection must include all members to check nodal equilibrium.
pub struct MemberRecovery<'a> {
    /// Unique member identifier; not interpreted as a vector index.
    pub id: MemberId,
    /// Nodes in the order required by the element.
    pub nodes: &'a [NodeId],
    /// Element that produced the assembled contribution.
    pub element: &'a dyn Element,
    /// Element context dimension.
    pub dim: Dim,
    /// Resolved SI parameters of the element.
    pub params: &'a ElementParams,
    /// Accepted history, never trial history.
    pub committed: &'a [Real],
    /// Uniform loads of this case and member, UNSCALED.
    pub loads: &'a [MemberLoad],
    /// Stations x/L; `None` uses defaults, an empty list uses only the ends.
    pub stations: Option<&'a [Real]>,
}

/// Selectable field; local resultants retain both D3 directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// Global nodal vector of translations in m and rotations in rad.
    Displacements,
    /// Global nodal vector of forces in N and couples in N·m.
    Reactions,
    /// Local end actions in the element DOF order.
    EndForcesLocal,
    /// Axial force N, positive in tension.
    N,
    /// Local y shear, N (V in frame2d).
    Vy,
    /// Local z shear, N.
    Vz,
    /// Local x torsion, N·m.
    T,
    /// Local y moment, N·m.
    My,
    /// Local z moment, N·m (M in frame2d).
    Mz,
}

const FORCE_FIELDS: [Field; 6] = [
    Field::N,
    Field::Vy,
    Field::Vz,
    Field::T,
    Field::My,
    Field::Mz,
];

/// Displacements and reactions grouped by node.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeResult {
    /// Node identifier.
    pub id: NodeId,
    /// `[Ux,Uy,Uz,Rx,Ry,Rz]` in m/rad; missing components are zero.
    pub displacements: [Real; 6],
    /// Prescribed values `[Ux,Uy,Uz,Rx,Ry,Rz]` in m/rad, without load factor.
    /// `None` indicates a free, dependent, or missing DOF. When constructing results
    /// manually, these data must be retained to combine settlements.
    pub prescribed_displacements: [Option<Real>; 6],
    /// `[Fx,Fy,Fz,Mx,My,Mz]` in N/N·m; free or missing DOFs are zero.
    pub reactions: [Real; 6],
}

/// Exact local resultants returned by the element at a station.
#[derive(Clone, Debug, PartialEq)]
pub struct StationResult {
    /// Dimensionless position x/L in `[0,1]`.
    pub position: Real,
    /// `[N,Vy,Vz,T,My,Mz]` in N/N·m, on the local positive face.
    pub forces_local: [Real; 6],
}

/// Algebraic extreme value and location (not an absolute-value extremum).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extreme {
    /// SI value of the field.
    pub value: Real,
    /// Station x/L where it occurs; ties retain the first (smallest x/L).
    pub at_station: Real,
}

/// Algebraic maximum and minimum of a member field.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldExtremes {
    /// One of N, Vy, Vz, T, My, Mz.
    pub field: Field,
    /// Maximum and its location.
    pub max: Extreme,
    /// Minimum and its location.
    pub min: Extreme,
}

/// Actions, diagrams, and extrema of a member.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberResult {
    /// Member identifier.
    pub id: MemberId,
    /// Local actions on ends i/j, node then element DOF.
    /// Frame2d: `[Ni,Vi,Mi,Nj,Vj,Mj]`; truss: local translations per node.
    pub end_forces_local: Vec<Real>,
    /// Requested stations plus ends and critical stations, in increasing order.
    pub stations: Vec<StationResult>,
    /// Six maximum/minimum pairs, in the order N, Vy, Vz, T, My, Mz.
    pub extremes: Vec<FieldExtremes>,
}

/// Owned postprocessing data model; `results/1` serialization pending.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseResults {
    /// Nominal system case.
    pub case: CaseRef,
    /// Factor applied to loads, not settlements.
    pub factor: Real,
    /// All map nodes, by ascending identifier.
    pub nodes: Vec<NodeResult>,
    /// Recovered members, by ascending identifier.
    pub members: Vec<MemberResult>,
}

/// Nodal projection of a query; `None` means an excluded field.
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedNode {
    /// Selected node.
    pub id: NodeId,
    /// Global displacements in [`NodeResult`] order.
    pub displacements: Option<[Real; 6]>,
    /// Global reactions in [`NodeResult`] order.
    pub reactions: Option<[Real; 6]>,
}

/// Value of a selected local field at a station.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldValue {
    /// Local field.
    pub field: Field,
    /// Value in N or N·m according to the field.
    pub value: Real,
}

/// Station projected onto the selected local fields.
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedStation {
    /// Station x/L.
    pub position: Real,
    /// Only selected local fields, in the order N, Vy, Vz, T, My, Mz.
    pub values: Vec<FieldValue>,
}

/// Projection of a member, without values of excluded fields.
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedMember {
    /// Selected member.
    pub id: MemberId,
    /// End actions, or `None` if that field was excluded.
    pub end_forces_local: Option<Vec<Real>>,
    /// Empty if no local field was selected.
    pub stations: Vec<QueriedStation>,
    /// Only extrema of selected local fields.
    pub extremes: Vec<FieldExtremes>,
}

/// Owned query for future serialization, without a JSON schema in this ticket.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryResults {
    /// Queried case.
    pub case: CaseRef,
    /// Case factor.
    pub factor: Real,
    /// Selected nodes.
    pub nodes: Vec<QueriedNode>,
    /// Selected members.
    pub members: Vec<QueriedMember>,
}

impl CaseResults {
    /// Recovers a case using its global displacements (including prescribed values).
    /// Recalculates reactions with K and the system's physical loads; does not depend
    /// on reduced copies or reactions stored in the analysis result.
    /// The caller supplies all system members and their loads for the case.
    /// Checks per global DOF `−Σ(f_int,e − f_carga,e) + F_nodal + R = 0`:
    /// end actions ON the member are computed in global axes through
    /// `Element::respond` and `Element::load_vector`, without inferring local axes.
    /// Includes diaphragm actions when present. Requires relative residual
    /// ≤ 1e-9 with respect to the sum of term magnitudes for that DOF, with a
    /// floor of 1 N or 1 N·m, as in the SI analysis checks. The tolerance
    /// allows recovery/assembly rounding, not load differences.
    /// An inconsistency returns ENG-E-0010 with case, node, DOF, and incident
    /// members. This checks equilibrium, not identity: erroneous contributions
    /// that cancel nodally may be indistinguishable.
    /// ENG-E-0002 for duplicate ids/connectivity/DOFs, ENG-E-0003 for missing
    /// references, ENG-E-0010 for inconsistent sizes/stations/prescribed values,
    /// ENG-E-0008 for nonfinite values, and ENG-E-0012 for inconsistent element output.
    /// Propagates element diagnostics with the member in entities.
    pub fn recover<B: LinAlgBackend>(
        solution: &StaticCaseResult,
        system: &LinearSystem<B>,
        members: &[MemberRecovery<'_>],
    ) -> EResult<Self> {
        Self::recover_with_check(solution, system, members, &mut || Ok(()))
    }

    /// Cooperative recovery, checked between nodes and members.
    pub fn recover_with_check<B: LinAlgBackend>(
        solution: &StaticCaseResult,
        system: &LinearSystem<B>,
        members: &[MemberRecovery<'_>],
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Self> {
        check()?;
        let map = system.dof_map();
        let global = &solution.global_displacements;
        size(global.len(), map.len())?;
        finite(global)?;
        finite(&[solution.factor])?;
        for (&value, prescribed) in global.iter().zip(system.constraints().prescribed_values()) {
            if prescribed.is_some_and(|p| p != value) {
                return Err(error("ENG-E-0010", "Inconsistent prescribed displacement."));
            }
        }
        let free: Vec<_> = system
            .constraints()
            .free_dofs()
            .iter()
            .map(|i| global[i.0 as usize])
            .collect();
        let reactions = system.reactions(&free, &solution.case, solution.factor)?;
        let mut balance = NodalBalance::new(map.len());
        for (i, value) in system
            .global_nodal_force(&solution.case, solution.factor)?
            .into_iter()
            .enumerate()
        {
            balance.add(i, value)?;
        }
        for r in &reactions {
            balance.add(map.global_index(r.node, r.dof)?.0 as usize, r.value)?;
        }
        for diaphragm in system.diaphragm_forces(&free, &solution.case, solution.factor)? {
            for action in std::iter::once(diaphragm.master).chain(diaphragm.slaves) {
                for (dof, value) in [
                    engine_core::Dof::Ux,
                    engine_core::Dof::Uy,
                    engine_core::Dof::Rz,
                ]
                .into_iter()
                .zip(action.values)
                {
                    balance.add(map.global_index(action.node, dof)?.0 as usize, value)?;
                }
            }
        }
        let mut nodes = BTreeMap::new();
        for (i, &value) in global.iter().enumerate() {
            check()?;
            let index =
                u32::try_from(i).map_err(|_| error("ENG-E-0010", "Global index exceeds u32."))?;
            let (id, dof) = map.label(DofIndex(index))?;
            let node = nodes.entry(id).or_insert(NodeResult {
                id,
                displacements: [0.0; 6],
                prescribed_displacements: [None; 6],
                reactions: [0.0; 6],
            });
            node.displacements[dof.index()] = value;
            node.prescribed_displacements[dof.index()] =
                system.constraints().prescribed_values()[i];
        }
        for r in reactions {
            let node = nodes
                .get_mut(&r.node)
                .ok_or_else(|| error("ENG-E-0012", "Reaction has no node."))?;
            node.reactions[r.dof.index()] = r.value;
        }
        let mut recovered = BTreeMap::new();
        for member in members {
            check()?;
            if recovered.contains_key(&member.id) {
                return Err(with_member(
                    error("ENG-E-0002", "Duplicate member."),
                    member.id,
                ));
            }
            let result = recover_member(member, map, global, solution.factor, &mut balance)
                .map_err(|e| with_member(e, member.id))?;
            recovered.insert(member.id, result);
        }
        balance.verify(map, &solution.case, members)?;
        Ok(Self {
            case: solution.case.clone(),
            factor: solution.factor,
            nodes: nodes.into_values().collect(),
            members: recovered.into_values().collect(),
        })
    }

    /// Filters by ids and fields. `None` selects all; `Some(&[])`, none.
    /// Retains original order and does not duplicate repeatedly requested entities/fields.
    /// ENG-E-0003 if a requested node or member is not in this result.
    /// Fields that do not apply to a block add no values to that block.
    pub fn query(
        &self,
        nodes: Option<&[NodeId]>,
        members: Option<&[MemberId]>,
        fields: Option<&[Field]>,
    ) -> EResult<QueryResults> {
        if let Some(ids) = nodes {
            for &id in ids {
                if !self.nodes.iter().any(|n| n.id == id) {
                    let mut e = error("ENG-E-0003", "Missing node in query.");
                    e.entities.push(EntityRef::Node(id));
                    return Err(e);
                }
            }
        }
        if let Some(ids) = members {
            for &id in ids {
                if !self.members.iter().any(|m| m.id == id) {
                    return Err(with_member(
                        error("ENG-E-0003", "Missing member in query."),
                        id,
                    ));
                }
            }
        }
        let selected = |f| fields.is_none_or(|fs| fs.contains(&f));
        let local: Vec<_> = FORCE_FIELDS
            .iter()
            .enumerate()
            .filter(|(_, f)| selected(**f))
            .collect();
        Ok(QueryResults {
            case: self.case.clone(),
            factor: self.factor,
            nodes: self
                .nodes
                .iter()
                .filter(|n| nodes.is_none_or(|ids| ids.contains(&n.id)))
                .map(|n| QueriedNode {
                    id: n.id,
                    displacements: selected(Field::Displacements).then_some(n.displacements),
                    reactions: selected(Field::Reactions).then_some(n.reactions),
                })
                .collect(),
            members: self
                .members
                .iter()
                .filter(|m| members.is_none_or(|ids| ids.contains(&m.id)))
                .map(|m| QueriedMember {
                    id: m.id,
                    end_forces_local: selected(Field::EndForcesLocal)
                        .then(|| m.end_forces_local.clone()),
                    stations: if local.is_empty() {
                        vec![]
                    } else {
                        m.stations
                            .iter()
                            .map(|s| QueriedStation {
                                position: s.position,
                                values: local
                                    .iter()
                                    .map(|&(i, &field)| FieldValue {
                                        field,
                                        value: s.forces_local[i],
                                    })
                                    .collect(),
                            })
                            .collect()
                    },
                    extremes: m
                        .extremes
                        .iter()
                        .filter(|e| selected(e.field))
                        .cloned()
                        .collect(),
                })
                .collect(),
        })
    }
}

fn recover_member(
    m: &MemberRecovery<'_>,
    map: &engine_assembly::DofMap,
    global: &[Real],
    factor: Real,
    balance: &mut NodalBalance,
) -> EResult<MemberResult> {
    size(m.nodes.len(), m.element.n_nodes())?;
    size(m.committed.len(), m.element.state_len())?;
    finite(m.committed)?;
    if m.nodes.is_empty() || m.element.dofs_per_node().is_empty() {
        return Err(error("ENG-E-0010", "Member has no nodes or DOFs."));
    }
    let mut coords = Vec::new();
    let mut u = Vec::new();
    let mut seen = BTreeSet::new();
    for &node in m.nodes {
        coords.push(map.coordinates(node)?);
        for &dof in m.element.dofs_per_node() {
            let index = map.global_index(node, dof)?;
            if !seen.insert(index) {
                return Err(error(
                    "ENG-E-0002",
                    "Duplicate connectivity or element DOF.",
                ));
            }
            let value = global
                .get(index.0 as usize)
                .ok_or_else(|| error("ENG-E-0010", "Element DOF is outside the global vector."))?;
            u.push(*value);
        }
    }
    let ctx = ElementCtx {
        dim: m.dim,
        coords: &coords,
        params: m.params,
    };
    let mut loads = m.loads.to_vec();
    for load in &mut loads {
        for value in load
            .force_per_length
            .iter_mut()
            .chain(&mut load.moment_per_length)
        {
            finite(&[*value])?;
            *value *= factor;
            finite(&[*value])?;
        }
    }
    let mut stations = m.stations.unwrap_or(&DEFAULT_STATIONS).to_vec();
    finite(&stations)?;
    if stations.iter().any(|s| !(0.0..=1.0).contains(s)) {
        return Err(error("ENG-E-0010", "Station is outside [0,1]."));
    }
    stations.extend([0.0, 1.0]);
    sort_stations(&mut stations);
    let mut output = recover_output(m, &ctx, &u, &loads, &stations)?;
    if m.element.kind() == "frame2d.euler_elastic" {
        let first = output
            .stations
            .first()
            .ok_or_else(|| error("ENG-E-0012", "Recovery has no ends."))?;
        let v_i = first.forces_local[1];
        let [a, b] = coords.as_slice() else {
            return Err(error("ENG-E-0012", "Frame2d must have two nodes."));
        };
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let length = math::hypot(dx, dy);
        finite(&[dx, dy, length])?;
        if length <= 0.0 {
            return Err(error("ENG-E-0005", "Zero-length member."));
        }
        let mut qy = 0.0;
        let mut mz = 0.0;
        for load in &loads {
            qy += -(dy / length) * load.force_per_length[0]
                + (dx / length) * load.force_per_length[1];
            mz += load.moment_per_length[2];
            finite(&[qy, mz])?;
        }
        if qy != 0.0 {
            // V(x)=Vi−qy*x. M′=−V−mz: with mz=0 the two roots coincide.
            let denominator = qy * length;
            finite(&[denominator, v_i + mz])?;
            if denominator == 0.0 {
                return Err(error("ENG-E-0010", "Transverse load per length underflow."));
            }
            for numerator in [v_i, v_i + mz] {
                // Checking the interior before dividing avoids overflow of an exterior root.
                if (denominator > 0.0 && numerator > 0.0 && numerator < denominator)
                    || (denominator < 0.0 && numerator < 0.0 && numerator > denominator)
                {
                    let position = numerator / denominator;
                    finite(&[position])?;
                    if position > 0.0 && position < 1.0 {
                        stations.push(position);
                    }
                }
            }
            sort_stations(&mut stations);
            output = recover_output(m, &ctx, &u, &loads, &stations)?;
        }
    }
    let stations: Vec<_> = output
        .stations
        .into_iter()
        .map(|s| StationResult {
            position: canonical_station(s.position),
            forces_local: s.forces_local,
        })
        .collect();
    let internal = m.element.respond(&ctx, &u, m.committed)?.internal_force;
    output_size(internal.len(), u.len())?;
    finite(&internal)?;
    let mut global_loads = Vec::new();
    for load in &loads {
        let values = m.element.load_vector(&ctx, load)?;
        output_size(values.len(), u.len())?;
        finite(&values)?;
        global_loads.push(values);
    }
    for (i, (&node, &dof)) in m
        .nodes
        .iter()
        .flat_map(|node| m.element.dofs_per_node().iter().map(move |dof| (node, dof)))
        .enumerate()
    {
        let index = map.global_index(node, dof)?.0 as usize;
        balance.add(index, -internal[i])?;
        for load in &global_loads {
            balance.add(index, load[i])?;
        }
    }
    Ok(MemberResult {
        id: m.id,
        end_forces_local: output.end_forces_local,
        extremes: station_extremes(&stations)?,
        stations,
    })
}

// Scale per DOF: does not mix forces with moments; retains the terms before
// canceling f_int − f_carga, to allow rounding at ends with zero moment.
struct NodalBalance {
    residual: Vec<Real>,
    scale: Vec<Real>,
}

impl NodalBalance {
    fn new(size: usize) -> Self {
        Self {
            residual: vec![0.0; size],
            scale: vec![0.0; size],
        }
    }

    fn add(&mut self, index: usize, value: Real) -> EResult<()> {
        self.residual[index] += value;
        self.scale[index] += value.abs();
        finite(&[self.residual[index], self.scale[index]])
    }

    fn verify(
        &self,
        map: &engine_assembly::DofMap,
        case: &CaseRef,
        members: &[MemberRecovery<'_>],
    ) -> EResult<()> {
        for (i, (&residual, &scale)) in self.residual.iter().zip(&self.scale).enumerate() {
            let relative = residual.abs() / scale.max(1.0);
            if relative > 1e-9 {
                let index = DofIndex(
                    u32::try_from(i)
                        .map_err(|_| error("ENG-E-0010", "Global index exceeds u32."))?,
                );
                let (node, dof) = map.label(index)?;
                let mut e = error(
                    "ENG-E-0010",
                    &format!(
                        "Inconsistent recovery at node {} / {dof:?}: relative residual {relative:e} > 1e-9.",
                        node.0
                    ),
                );
                e.entities.extend([
                    EntityRef::Case(case.0.clone()),
                    EntityRef::Node(node),
                    EntityRef::Dof(index),
                ]);
                for member in members {
                    if member.nodes.contains(&node) && member.element.dofs_per_node().contains(&dof)
                    {
                        e.entities.push(EntityRef::Member(member.id));
                    }
                }
                return Err(e);
            }
        }
        Ok(())
    }
}

/// Algebraic extrema of the stations, retaining the first on ties.
pub(crate) fn station_extremes(stations: &[StationResult]) -> EResult<Vec<FieldExtremes>> {
    let first = stations
        .first()
        .ok_or_else(|| error("ENG-E-0010", "Member has no stations."))?;
    Ok(FORCE_FIELDS
        .iter()
        .enumerate()
        .map(|(i, &field)| {
            let start = Extreme {
                value: first.forces_local[i],
                at_station: first.position,
            };
            let mut max = start;
            let mut min = start;
            for s in stations {
                let candidate = Extreme {
                    value: s.forces_local[i],
                    at_station: s.position,
                };
                if candidate.value > max.value {
                    max = candidate;
                }
                if candidate.value < min.value {
                    min = candidate;
                }
            }
            FieldExtremes { field, max, min }
        })
        .collect())
}

fn recover_output(
    m: &MemberRecovery<'_>,
    ctx: &ElementCtx<'_>,
    u: &[Real],
    loads: &[MemberLoad],
    stations: &[Real],
) -> EResult<ElementOutput> {
    let output = m.element.recover(ctx, u, m.committed, loads, stations)?;
    if output.end_forces_local.len() != u.len() || output.stations.len() != stations.len() {
        return Err(error("ENG-E-0012", "Inconsistent element recovery size."));
    }
    finite(&output.end_forces_local)?;
    for (s, &position) in output.stations.iter().zip(stations) {
        finite(&[s.position])?;
        finite(&s.forces_local)?;
        if s.position != position {
            return Err(error(
                "ENG-E-0012",
                "Recovered station differs from the requested station.",
            ));
        }
    }
    Ok(output)
}

fn sort_stations(stations: &mut Vec<Real>) {
    for position in stations.iter_mut() {
        *position = canonical_station(*position);
    }
    stations.sort_by(Real::total_cmp);
    stations.dedup_by(|a, b| *a == *b);
}

fn canonical_station(position: Real) -> Real {
    if position == 0.0 {
        0.0
    } else {
        position
    }
}

fn output_size(actual: usize, expected: usize) -> EResult<()> {
    if actual != expected {
        return Err(error("ENG-E-0012", "Inconsistent element output size."));
    }
    Ok(())
}

fn size(actual: usize, expected: usize) -> EResult<()> {
    if actual != expected {
        return Err(error("ENG-E-0010", "Incompatible size in postprocessing."));
    }
    Ok(())
}

fn error(code: &'static str, message: &str) -> EngineError {
    EngineError::new(code, message)
}

fn with_member(mut e: EngineError, id: MemberId) -> EngineError {
    e.entities.push(EntityRef::Member(id));
    e
}
