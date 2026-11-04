//! Public uniform temperature on a linear truss.
//!
//! The closed form stays in `engine-elements-truss`. This module rejects any
//! other kind, turns the equivalent joint forces into nodal loads on a cloned
//! model, and builds a per-case bar so recovery can report the thermal axial.
//! A gradient is not accepted. Elastic beam-column kinds have no αΔT formula.
//!
//! `respond` and geometric stiffness stay mechanical. The joint forces are
//! nodal loads of the assembled model, not a `MemberLoad`, so nodal
//! equilibrium still closes. A later modal rebuild from the original model
//! drops those injected loads; this module does not invent a thermal mass.

use super::*;
use engine_elements_truss::LinearTruss;

/// One finite uniform change stored on a member for a single case.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Change {
    /// Linear expansion coefficient, 1/K.
    pub alpha_per_kelvin: f64,
    /// Uniform temperature change, K.
    pub delta_t_kelvin: f64,
}

/// Per-case change for `id`. A repeated member in one case is ENG-E-0002.
///
/// Any record on a kind other than `truss2d.linear` or `truss3d.linear` is
/// ENG-E-0013. An empty element kind has already been resolved by the caller.
pub(crate) fn for_member(
    id: engine_core::MemberId,
    kind: &str,
    cases: &[Vec<input::TemperatureLoad>],
) -> Result<Vec<Option<Change>>, EngineError> {
    let mut changes = Vec::with_capacity(cases.len());
    for case in cases {
        let mut found = None;
        for record in case.iter().filter(|record| record.member == id) {
            if found.is_some() {
                return Err(EngineError::new(
                    "ENG-E-0002",
                    format!("Uniform temperature is repeated on member {}.", id.0),
                ));
            }
            found = Some(Change {
                alpha_per_kelvin: record.alpha_per_kelvin,
                delta_t_kelvin: record.delta_t_kelvin,
            });
        }
        if found.is_some() && !matches!(kind, "truss2d.linear" | "truss3d.linear") {
            return Err(EngineError::new(
                "ENG-E-0013",
                format!(
                    "Uniform temperature has no formula on {kind}. Only truss2d.linear and truss3d.linear accept it."
                ),
            ));
        }
        changes.push(found);
    }
    Ok(changes)
}

/// Clones `model` and appends equivalent joint forces as nodal loads.
///
/// The original model is left unchanged. Zero components are omitted. The
/// numeric value comes from [`LinearTruss::fixed_end_uniform_temperature`].
pub(crate) fn with_joint_forces(
    model: &engine_model::Model,
    members: &[Member],
) -> Result<engine_model::Model, EngineError> {
    let mut loaded = model.clone();
    let map = DofMap::new(&model.nodes)?;
    for member in members {
        if member.temperature.iter().all(Option::is_none) {
            continue;
        }
        if member.temperature.len() != model.cases.len() {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Temperature cases do not match the model.",
            ));
        }
        let bar = truss_from(member)?;
        let coords = member
            .nodes
            .iter()
            .map(|node| map.coordinates(*node))
            .collect::<Result<Vec<_>, _>>()?;
        let ctx = ElementCtx {
            dim: member.dim,
            coords: &coords,
            params: &member.params,
        };
        let dofs = member.element.dofs_per_node();
        let width = dofs.len();
        for (case_index, change) in member.temperature.iter().enumerate() {
            let Some(change) = change else { continue };
            let thermal = bar.fixed_end_uniform_temperature(
                &ctx,
                change.alpha_per_kelvin,
                change.delta_t_kelvin,
                &[],
            )?;
            if thermal.joint_forces.len() != width * member.nodes.len() {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Temperature joint forces do not match the member.",
                ));
            }
            for (node_index, node) in member.nodes.iter().enumerate() {
                let start = node_index * width;
                let mut values = Vec::new();
                for (offset, dof) in dofs.iter().enumerate() {
                    let value = thermal.joint_forces[start + offset];
                    if value != 0.0 {
                        values.push(engine_model::DofValue { dof: *dof, value });
                    }
                }
                if !values.is_empty() {
                    loaded.cases[case_index]
                        .loads
                        .push(engine_model::NodalLoad {
                            node: *node,
                            values,
                        });
                }
            }
        }
    }
    Ok(loaded)
}

/// Heated bar for recovery of one case. `scale` multiplies ΔT.
///
/// Linear static passes 1. A nonlinear step passes the accepted load factor
/// so the reported axial matches the joint forces applied by the solver.
/// `None` keeps the assembled element.
pub(crate) fn recovery_bar(
    member: &Member,
    case: usize,
    scale: f64,
) -> Result<Option<LinearTruss>, EngineError> {
    if case >= member.temperature.len() {
        return Err(EngineError::new(
            "ENG-E-0012",
            "Temperature case is missing.",
        ));
    }
    let Some(change) = member.temperature[case] else {
        return Ok(None);
    };
    let bar = truss_from(member)?;
    Ok(Some(bar.with_fixed_end_uniform_temperature(
        change.alpha_per_kelvin,
        change.delta_t_kelvin * scale,
    )?))
}

/// One optional heated bar per member, at load factor 1.
pub(crate) fn recovery_bars(
    members: &[Member],
    case: usize,
) -> Result<Vec<Option<LinearTruss>>, EngineError> {
    members
        .iter()
        .map(|member| recovery_bar(member, case, 1.0))
        .collect()
}

fn truss_from(member: &Member) -> Result<LinearTruss, EngineError> {
    let scalar = |name: &str| {
        member.params.scalars.get(name).copied().ok_or_else(|| {
            EngineError::new(
                "ENG-E-0012",
                format!("Missing {name} on member {}.", member.id.0),
            )
        })
    };
    LinearTruss::new(member.dim, scalar("E")?, scalar("A")?, scalar("rho")?)
}
