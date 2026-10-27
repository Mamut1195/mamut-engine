//! Superposition with user factors and algebraic envelopes in SI.
//!
//! No code coefficients are applied. Each combination component is
//! `Σ factor * valor`, with separate f64 multiplication and addition, in
//! term order (without interpolation or additional rounding). `CaseResults::factor`
//! is metadata for already scaled loads: it is not multiplied a second time.
//! With settlements the response is affine: `r(F) = r(0) + L(F)`. Summing complete
//! cases would multiply `r(0)` by the sum of factors (for example, 2.8 in
//! `1.2D + 1.6L`). Results do not separate forces/reactions from `r(0)`;
//! subtracting it automatically would require extending the recovery contract.
//! For safety, terms with more than one case of nonzero prescribed displacements
//! are rejected with ENG-E-0010 and Case entities, even if their factors are zero
//! or their values match. Inconsistent prescribed metadata is also rejected.
//! Solve D and L with zero prescribed displacements and settlement S without loads, keeping
//! the same prescribed DOFs and stiffness; combine `1.2D + 1.6L + αS`, with an
//! explicit factor `α` (1 to count settlement once). A single case with
//! settlement is weighted in full by its factor; it is not silently corrected.
//!
//! Entities are matched by identifier, even if their order differs. Each
//! member requires identical station positions and end-action sizes.
//! Diagrams are the summed stations. Extrema are recalculated over
//! those stations, without using the maxima/minima stored in each input or
//! inferring new critical interior stations from samples.
//!
//! The envelope retains identifier, value, and, for member extrema,
//! position. An exact tie retains the first input; within a member,
//! the first station (smallest x/L). Cases and combinations have distinct
//! identities even with the same name. An envelope is not an equilibrium case.

use crate::results::{station_extremes, CaseResults, Extreme, Field, MemberResult, NodeResult};
use engine_contracts::CaseRef;
use engine_core::{EResult, EngineError, EntityRef, MemberId, NodeId, Real};
use engine_model::Combination;
use std::collections::{BTreeMap, BTreeSet};

/// Result of a user combination, reusing the results model.
#[derive(Clone, Debug, PartialEq)]
pub struct CombinationResults {
    /// Identifier of `engine_model::Combination`.
    pub id: String,
    /// Weighted sum; `case` is the combination id and `factor` is 1.
    pub results: CaseResults,
}

/// Identity of the governing result, with separate namespaces.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResultId {
    /// Load case name.
    Case(String),
    /// Combination identifier.
    Combination(String),
}

/// Borrowed envelope input, retained in user order.
#[derive(Clone, Copy, Debug)]
pub enum EnvelopeInput<'a> {
    /// Solved case, even if a load factor has already been applied.
    Case(&'a CaseResults),
    /// Computed combination.
    Combination(&'a CombinationResults),
}

impl<'a> EnvelopeInput<'a> {
    fn id(self) -> ResultId {
        match self {
            Self::Case(r) => ResultId::Case(r.case.0.clone()),
            Self::Combination(r) => ResultId::Combination(r.id.clone()),
        }
    }

    fn results(self) -> &'a CaseResults {
        match self {
            Self::Case(r) => r,
            Self::Combination(r) => &r.results,
        }
    }
}

/// SI component and its governing identity.
#[derive(Clone, Debug, PartialEq)]
pub struct GoverningValue {
    /// Governing case or combination.
    pub source: ResultId,
    /// Algebraic value, without absolute value or additional scaling.
    pub value: Real,
}

/// Maximum and minimum of a component across inputs.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueEnvelope {
    /// Algebraic maximum; first governing input on ties.
    pub max: GoverningValue,
    /// Algebraic minimum; first governing input on ties.
    pub min: GoverningValue,
}

/// Nodal envelopes in the DOF order of `NodeResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeEnvelope {
    /// Node identifier.
    pub id: NodeId,
    /// Displacements `[Ux,Uy,Uz,Rx,Ry,Rz]`, m/rad.
    pub displacements: [ValueEnvelope; 6],
    /// Reactions `[Fx,Fy,Fz,Mx,My,Mz]`, N/N·m.
    pub reactions: [ValueEnvelope; 6],
}

/// Envelopes of the six section resultants at a common position.
#[derive(Clone, Debug, PartialEq)]
pub struct StationEnvelope {
    /// Dimensionless position x/L.
    pub position: Real,
    /// `[N,Vy,Vz,T,My,Mz]`, N/N·m.
    pub forces_local: [ValueEnvelope; 6],
}

/// Governing member extremum, with location and identity.
#[derive(Clone, Debug, PartialEq)]
pub struct GoverningExtreme {
    /// Governing case or combination.
    pub source: ResultId,
    /// SI value and position of the extremum.
    pub extreme: Extreme,
}

/// Global envelope of a field across inputs and member stations.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldEnvelope {
    /// Field N, Vy, Vz, T, My, or Mz.
    pub field: Field,
    /// Maximum of the recalculated maxima of each input.
    pub max: GoverningExtreme,
    /// Minimum of the recalculated minima of each input.
    pub min: GoverningExtreme,
}

/// Envelopes of end actions, diagrams, and section extrema.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberEnvelope {
    /// Member identifier.
    pub id: MemberId,
    /// In the local DOF order of `MemberResult::end_forces_local`.
    pub end_forces_local: Vec<ValueEnvelope>,
    /// Common stations, in increasing order.
    pub stations: Vec<StationEnvelope>,
    /// Six fields, in the order N, Vy, Vz, T, My, Mz.
    pub extremes: Vec<FieldEnvelope>,
}

/// Algebraic envelope; each component may have a different governing input.
#[derive(Clone, Debug, PartialEq)]
pub struct EnvelopeResults {
    /// Input identities in the order used to resolve ties.
    pub sources: Vec<ResultId>,
    /// Nodes by ascending identifier.
    pub nodes: Vec<NodeEnvelope>,
    /// Members by ascending identifier.
    pub members: Vec<MemberEnvelope>,
}

/// Superposes existing results according to the terms of a combination.
///
/// ENG-E-0003: missing referenced case, even with a zero factor.
/// ENG-E-0008: nonfinite factor/data or overflow of a product or partial sum.
/// ENG-E-0010: empty terms/repository, duplicate cases or entities,
/// incompatible topology/sizes/stations, multiple cases with nonzero prescribed
/// displacements or inconsistent prescribed metadata (Case entities). All terms
/// must share the prescribed DOFs, even if their values differ.
/// Terms with a zero factor
/// are also validated. Unreferenced cases are checked only for duplication.
/// Does not modify inputs. Extrema are computed after summation.
pub fn combine(combination: &Combination, cases: &[CaseResults]) -> EResult<CombinationResults> {
    combine_with_check(combination, cases, &mut || Ok(()))
}

/// Cooperative superposition per term, node, and member, without reordering sums.
pub fn combine_with_check(
    combination: &Combination,
    cases: &[CaseResults],
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<CombinationResults> {
    check()?;
    if combination.terms.is_empty() || cases.is_empty() {
        return Err(invalid("Empty combination or case repository."));
    }
    let mut available = BTreeMap::new();
    for case in cases {
        check()?;
        if available.insert(case.case.0.as_str(), case).is_some() {
            return Err(invalid("Duplicate case in repository."));
        }
    }
    let mut seen = BTreeSet::new();
    let mut terms = Vec::new();
    let mut settlement_case = None;
    for term in &combination.terms {
        check()?;
        number(term.factor)?;
        if !seen.insert(term.case.as_str()) {
            return Err(invalid("Duplicate case in terms."));
        }
        let case = available.get(term.case.as_str()).ok_or_else(|| {
            let mut e = EngineError::new("ENG-E-0003", "Missing case in combination.");
            e.entities.push(EntityRef::Case(term.case.clone()));
            e
        })?;
        let input = layout(case)?;
        if case.nodes.iter().any(|node| {
            node.prescribed_displacements
                .iter()
                .any(|p| p.is_some_and(|value| value != 0.0))
        }) {
            if let Some(previous) = settlement_case {
                let mut e = invalid(
                    "Multiple cases with nonzero prescribed displacements. Solve the loads with zero prescribed displacements and use a separate settlement case with an explicit factor.",
                );
                e.entities.push(EntityRef::Case(previous));
                e.entities.push(EntityRef::Case(term.case.clone()));
                return Err(e);
            }
            settlement_case = Some(term.case.clone());
        }
        terms.push((term.factor, input));
    }
    let (_, first) = terms
        .first()
        .ok_or_else(|| invalid("Combination has no terms."))?;
    for (_, next) in &terms {
        check()?;
        compatible(first, next)?;
        for (a, b) in first.nodes.values().zip(next.nodes.values()) {
            if a.prescribed_displacements.map(|p| p.is_some())
                != b.prescribed_displacements.map(|p| p.is_some())
            {
                let mut e = invalid("Incompatible prescribed DOF sets across cases.");
                e.entities.push(EntityRef::Case(first.case.into()));
                e.entities.push(EntityRef::Case(next.case.into()));
                return Err(e);
            }
        }
    }
    let mut output = CaseResults {
        case: CaseRef(combination.id.clone()),
        factor: 1.0,
        nodes: first.nodes.values().map(|n| (*n).clone()).collect(),
        members: first.members.values().map(|m| (*m).clone()).collect(),
    };
    for n in &mut output.nodes {
        n.displacements.fill(0.0);
        n.reactions.fill(0.0);
    }
    for m in &mut output.members {
        m.end_forces_local.fill(0.0);
        for s in &mut m.stations {
            s.forces_local.fill(0.0);
        }
        m.extremes.clear();
    }
    for (factor, input) in &terms {
        check()?;
        for (n, source) in output.nodes.iter_mut().zip(input.nodes.values()) {
            check()?;
            add(&mut n.displacements, &source.displacements, *factor)?;
            add(&mut n.reactions, &source.reactions, *factor)?;
        }
        for (m, source) in output.members.iter_mut().zip(input.members.values()) {
            check()?;
            add(&mut m.end_forces_local, &source.end_forces_local, *factor)?;
            for (s, source) in m.stations.iter_mut().zip(&source.stations) {
                add(&mut s.forces_local, &source.forces_local, *factor)?;
            }
        }
    }
    for m in &mut output.members {
        m.extremes = station_extremes(&m.stations)?;
    }
    for n in &mut output.nodes {
        for (prescribed, &value) in n.prescribed_displacements.iter_mut().zip(&n.displacements) {
            if prescribed.is_some() {
                *prescribed = Some(value);
            }
        }
    }
    Ok(CombinationResults {
        id: combination.id.clone(),
        results: output,
    })
}

/// Computes maxima/minima per component, station, and member field.
///
/// ENG-E-0010 for empty/duplicate inputs or incompatible results;
/// ENG-E-0008 for nonfinite data. Ties: first input; for equal
/// values within a member: first station. Extrema of each input are
/// recalculated over its stations, without relying on stored extrema.
pub fn envelope(inputs: &[EnvelopeInput<'_>]) -> EResult<EnvelopeResults> {
    let first_input = inputs
        .first()
        .ok_or_else(|| invalid("Envelope has no entries."))?;
    let mut seen = BTreeSet::new();
    let mut layouts = Vec::new();
    for &input in inputs {
        if !seen.insert(input.id()) {
            return Err(invalid("Duplicate entry in envelope."));
        }
        layouts.push(layout(input.results())?);
    }
    let first = layouts
        .first()
        .ok_or_else(|| invalid("Envelope has no results."))?;
    for next in &layouts {
        compatible(first, next)?;
    }
    let id = first_input.id();
    let mut output = EnvelopeResults {
        sources: inputs.iter().map(|i| i.id()).collect(),
        nodes: first
            .nodes
            .values()
            .map(|n| NodeEnvelope {
                id: n.id,
                displacements: n.displacements.map(|v| initial(v, &id)),
                reactions: n.reactions.map(|v| initial(v, &id)),
            })
            .collect(),
        members: Vec::new(),
    };
    for m in first.members.values() {
        output.members.push(MemberEnvelope {
            id: m.id,
            end_forces_local: m
                .end_forces_local
                .iter()
                .map(|&v| initial(v, &id))
                .collect(),
            stations: m
                .stations
                .iter()
                .map(|s| StationEnvelope {
                    position: s.position,
                    forces_local: s.forces_local.map(|v| initial(v, &id)),
                })
                .collect(),
            extremes: station_extremes(&m.stations)?
                .into_iter()
                .map(|e| FieldEnvelope {
                    field: e.field,
                    max: GoverningExtreme {
                        source: id.clone(),
                        extreme: e.max,
                    },
                    min: GoverningExtreme {
                        source: id.clone(),
                        extreme: e.min,
                    },
                })
                .collect(),
        });
    }
    for (&input, data) in inputs.iter().zip(&layouts).skip(1) {
        let id = input.id();
        for (n, source) in output.nodes.iter_mut().zip(data.nodes.values()) {
            update_values(&mut n.displacements, &source.displacements, &id);
            update_values(&mut n.reactions, &source.reactions, &id);
        }
        for (m, source) in output.members.iter_mut().zip(data.members.values()) {
            update_values(&mut m.end_forces_local, &source.end_forces_local, &id);
            for (s, source) in m.stations.iter_mut().zip(&source.stations) {
                update_values(&mut s.forces_local, &source.forces_local, &id);
            }
            for (e, source) in m
                .extremes
                .iter_mut()
                .zip(station_extremes(&source.stations)?)
            {
                if source.max.value > e.max.extreme.value {
                    e.max = GoverningExtreme {
                        source: id.clone(),
                        extreme: source.max,
                    };
                }
                if source.min.value < e.min.extreme.value {
                    e.min = GoverningExtreme {
                        source: id.clone(),
                        extreme: source.min,
                    };
                }
            }
        }
    }
    Ok(output)
}

struct Layout<'a> {
    case: &'a str,
    nodes: BTreeMap<NodeId, &'a NodeResult>,
    members: BTreeMap<MemberId, &'a MemberResult>,
}

fn layout(results: &CaseResults) -> EResult<Layout<'_>> {
    number(results.factor)?;
    if results.nodes.is_empty() {
        return Err(invalid("Result has no nodes."));
    }
    let mut nodes = BTreeMap::new();
    for n in &results.nodes {
        if nodes.insert(n.id, n).is_some() {
            return Err(invalid("Duplicate node."));
        }
        numbers(&n.displacements)?;
        numbers(&n.reactions)?;
        for (prescribed, &value) in n.prescribed_displacements.iter().zip(&n.displacements) {
            if let Some(prescribed) = prescribed {
                number(*prescribed).map_err(|mut e| {
                    e.entities.push(EntityRef::Case(results.case.0.clone()));
                    e
                })?;
                if *prescribed != value {
                    let mut e =
                        invalid("Prescribed displacement is inconsistent with the nodal result.");
                    e.entities.push(EntityRef::Case(results.case.0.clone()));
                    return Err(e);
                }
            }
        }
    }
    let mut members = BTreeMap::new();
    for m in &results.members {
        if members.insert(m.id, m).is_some() {
            return Err(invalid("Duplicate member."));
        }
        if m.end_forces_local.is_empty() || m.stations.len() < 2 {
            return Err(invalid("Member has no end actions or stations."));
        }
        numbers(&m.end_forces_local)?;
        let mut previous = None;
        for s in &m.stations {
            number(s.position)?;
            numbers(&s.forces_local)?;
            if !(0.0..=1.0).contains(&s.position) || previous.is_some_and(|p| p >= s.position) {
                return Err(invalid(
                    "Stations are out of range, unordered, or duplicated.",
                ));
            }
            previous = Some(s.position);
        }
        if m.stations.first().is_none_or(|s| s.position != 0.0)
            || m.stations.last().is_none_or(|s| s.position != 1.0)
        {
            return Err(invalid("Missing end stations 0 and 1."));
        }
        // Public extrema may be stale: they are not used in the calculation.
        for e in &m.extremes {
            numbers(&[e.max.value, e.max.at_station, e.min.value, e.min.at_station])?;
        }
    }
    Ok(Layout {
        case: &results.case.0,
        nodes,
        members,
    })
}

fn compatible(a: &Layout<'_>, b: &Layout<'_>) -> EResult<()> {
    if !a.nodes.keys().eq(b.nodes.keys()) || !a.members.keys().eq(b.members.keys()) {
        return Err(invalid("Incompatible node or member sets."));
    }
    for (a, b) in a.members.values().zip(b.members.values()) {
        if a.end_forces_local.len() != b.end_forces_local.len()
            || a.stations.len() != b.stations.len()
            || !a
                .stations
                .iter()
                .zip(&b.stations)
                .all(|(a, b)| a.position == b.position)
        {
            let mut e = invalid("Incompatible end actions or stations.");
            e.entities.push(EntityRef::Member(a.id));
            return Err(e);
        }
    }
    Ok(())
}

fn add(target: &mut [Real], source: &[Real], factor: Real) -> EResult<()> {
    for (sum, &value) in target.iter_mut().zip(source) {
        let product = factor * value;
        number(product)?;
        *sum += product;
        number(*sum)?;
    }
    Ok(())
}

fn initial(value: Real, id: &ResultId) -> ValueEnvelope {
    let v = GoverningValue {
        source: id.clone(),
        value,
    };
    ValueEnvelope {
        max: v.clone(),
        min: v,
    }
}

fn update_values(target: &mut [ValueEnvelope], source: &[Real], id: &ResultId) {
    for (e, &value) in target.iter_mut().zip(source) {
        if value > e.max.value {
            e.max = GoverningValue {
                source: id.clone(),
                value,
            };
        }
        if value < e.min.value {
            e.min = GoverningValue {
                source: id.clone(),
                value,
            };
        }
    }
}

fn number(value: Real) -> EResult<()> {
    if !value.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Combination/envelope: non-finite data or result.",
        ));
    }
    Ok(())
}

fn numbers(values: &[Real]) -> EResult<()> {
    for &value in values {
        number(value)?;
    }
    Ok(())
}

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
