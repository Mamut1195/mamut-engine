//! Adapters for domain methods, without P-Delta member recovery.
use super::*;
use crate::analysis_input::{Analysis, MassSource, Table};
use engine_analysis::buckling::BucklingLinear;
use engine_analysis::{modal::Modal, pdelta, spectrum};
use engine_contracts::MassKind;
use engine_core::DofIndex;

fn invalid(message: impl Into<String>) -> Failure {
    ApiError::InvalidInput(message.into()).into()
}
fn mass_kind(kind: &str) -> Result<MassKind, Failure> {
    match kind {
        "lumped" => Ok(MassKind::Lumped),
        "consistent" => Ok(MassKind::Consistent),
        _ => Err(invalid("mass_kind requires lumped or consistent")),
    }
}
fn bounded_steps(n: usize) -> Result<(), Failure> {
    if n == 0 || n > crate::MAX_JSON_ARRAY_ITEMS {
        return Err(invalid(format!(
            "Steps/modes require 1..={}",
            crate::MAX_JSON_ARRAY_ITEMS
        )));
    }
    Ok(())
}
// Domain callbacks use EResult; the facade translates cancellation
// before propagating its internal marker, preserving its own status/code.
fn checkpoint(
    cancel: &CancelToken,
    progress: &mut dyn FnMut(Progress),
    stage: &'static str,
    completed: usize,
    total: usize,
) -> engine_core::EResult<()> {
    step(cancel, progress, stage, completed, total)
        .map_err(|_| EngineError::new("ENG-E-0012", "Cancellation requested by the facade"))
}
fn check_cancel(cancel: &CancelToken) -> engine_core::EResult<()> {
    cancel
        .check()
        .map_err(|_| EngineError::new("ENG-E-0012", "Cancellation requested by the facade"))
}
fn prepare_mass(
    execution: &input::Execution,
    contributions: &[ElementContribution],
    system: &mut LinearSystem<ReferenceBackend>,
    source: Option<&MassSource>,
    document: &mut Value,
) -> Result<(), Failure> {
    let Some(source) = source else { return Ok(()) };
    if !execution.model.masses.is_empty() {
        return Err(invalid(
            "Declare nodal masses or mass_source, without mixing sources",
        ));
    }
    push_warning(document, "API-W-MASS-PROJECTION", "#86: mass from loads has no tolerance for negative projection; gravity not aligned with the axes may cause rejection due to rounding. Suction is rejected with ENG-E-0010.".into());
    let mut model = execution.model.clone();
    model.masses = match source.method.as_str() {
        "masses_from_case" => {
            let case = model
                .cases
                .iter()
                .find(|c| c.name == source.case)
                .ok_or_else(|| EngineError::new("ENG-E-0003", "Missing mass case"))?;
            engine_loads::mass::masses_from_case(
                system.dof_map(),
                case,
                source.factor,
                source.direction,
                source.gravity,
            )?
        }
        "system_case" | "masses_from_system_case" => engine_loads::mass::masses_from_system_case(
            system,
            &CaseRef(source.case.clone()),
            source.factor,
            source.direction,
            source.gravity,
        )?,
        _ => {
            return Err(invalid(
                "mass_source.method requires masses_from_case or system_case",
            ))
        }
    };
    *system = LinearSystem::from_model_with_diaphragms(
        ReferenceBackend,
        &model,
        contributions,
        &execution.prescribed,
        &execution.diaphragms,
    )?;
    Ok(())
}

fn reject_combinations(execution: &input::Execution) -> Result<(), Failure> {
    if !execution.model.combinations.is_empty() {
        return Err(invalid("Load combinations require linearStatic"));
    }
    Ok(())
}

pub(crate) fn pdelta_budget(execution: &input::Execution) -> usize {
    if let Analysis::PDelta {
        options: input::Object(o),
    } = &execution.analyses[0]
    {
        o.max_iterations.min(crate::MAX_JSON_ARRAY_ITEMS) + 5
    } else {
        5
    }
}
pub(crate) fn buckling_budget(execution: &input::Execution) -> usize {
    if let Analysis::Buckling {
        options: input::Object(o),
    } = &execution.analyses[0]
    {
        o.mode_count.min(crate::MAX_JSON_ARRAY_ITEMS) + 6
    } else {
        6
    }
}
pub(crate) fn modal_budget(execution: &input::Execution) -> usize {
    if let Analysis::Modal {
        options: input::Object(o),
    } = &execution.analyses[0]
    {
        o.mode_count.min(crate::MAX_JSON_ARRAY_ITEMS) + 5
    } else {
        5
    }
}
pub(crate) fn spectrum_budget(execution: &input::Execution) -> usize {
    if let Analysis::Spectrum {
        options: input::Object(o),
    } = &execution.analyses[0]
    {
        let n = o.mode_count.min(crate::MAX_JSON_ARRAY_ITEMS);
        let d = o.directions.len();
        n + n.saturating_mul(d).min(crate::MAX_JSON_ARRAY_ITEMS) + d + 5
    } else {
        5
    }
}

pub(crate) fn pdelta(context: MethodContext<'_>) -> Result<(), Failure> {
    let MethodContext {
        execution,
        members,
        contributions,
        system,
        cancel,
        progress,
        document,
        total,
        ..
    } = context;
    reject_combinations(execution)?;
    let Analysis::PDelta {
        options: input::Object(o),
    } = &execution.analyses[0]
    else {
        return Err(invalid("Inconsistent analysis adapter"));
    };

    bounded_steps(o.max_iterations)?;
    // geometric_producer recovers N from deformations; it does not receive member
    // loads to correct end actions for the gravity case.
    if execution
        .model
        .cases
        .iter()
        .zip(&execution.loads)
        .any(|(c, loads)| c.name == o.gravity.0.case && !loads.is_empty())
    {
        return Err(EngineError::new(
            "ENG-E-0013",
            "P-Delta: gravity requires nodal loads; Kg does not support member load correction",
        )
        .into());
    }
    let mut producers = Vec::new();
    for (m, contribution) in members.iter().zip(contributions) {
        cancel.check()?;
        let coords = m
            .nodes
            .iter()
            .map(|n| system.dof_map().coordinates(*n))
            .collect::<Result<Vec<_>, _>>()?;
        producers.push(pdelta::geometric_producer(
            m.element.clone(),
            &ElementCtx {
                dim: m.dim,
                coords: &coords,
                params: &m.params,
            },
            contribution.location.clone(),
            vec![0.0; m.element.state_len()],
        )?);
    }
    system.set_geometric_producers(producers)?;
    let result = pdelta::PDelta::run_with_check(
        system,
        &(CaseRef(o.gravity.0.case.clone()), o.gravity.0.factor),
        &(CaseRef(o.lateral.0.case.clone()), o.lateral.0.factor),
        pdelta::PDeltaOptions {
            tolerance: o.tolerance,
            max_iterations: o.max_iterations,
            displacement_scale: o.displacement_scale,
            rotation_scale: o.rotation_scale,
        },
        |n| {
            checkpoint(
                cancel,
                progress,
                if n == 0 { "gravity" } else { "pdeltaIteration" },
                n + 4,
                total,
            )
        },
        || check_cancel(cancel),
    );
    cancel.check()?;
    let result = result?;
    if result.inertia_uses_2x2 != Some(false) {
        push_warning(document, "API-W-PDELTA-INERTIA", "#78: 2×2 inertia uses copysign without a confirmed test (or backend has no metadata); review stability.".into());
    }
    let status = match result.outcome {
        pdelta::PDeltaOutcome::Completed => Status::Ok,
        pdelta::PDeltaOutcome::NotConverged => Status::NotConverged,
        pdelta::PDeltaOutcome::Unstable { .. } => Status::Unstable,
    };
    document["convergence"] = json!({"method":"pDelta", "status":status,
        "gravity": {"case":o.gravity.0.case,"factor":o.gravity.0.factor},
        "lateral": {"case":o.lateral.0.case,"factor":o.lateral.0.factor},
        "tolerance":o.tolerance,"max_iterations":o.max_iterations,
        "displacement_scale":o.displacement_scale,"rotation_scale":o.rotation_scale,
        "inertia": [result.inertia.0,result.inertia.1,result.inertia.2],
        "inertia_uses_2x2":result.inertia_uses_2x2,
        "iterations":result.iterations.iter().map(|i| json!({"iteration":i.iteration,
            "relative_displacement_change":i.relative_displacement_change,
            "free_residual_norm":i.free_residual_norm,"relative_free_residual":i.relative_free_residual,
                    "residual_convergence_ratio":i.residual_convergence_ratio})).collect::<Vec<_>>()});
    if status != Status::Ok {
        return Err(Failure(
            status,
            vec![diagnostic(
                if status == Status::Unstable {
                    "ENG-E-0006"
                } else {
                    "ENG-E-0007"
                },
                "error",
                "P-Delta: consulte convergence; no se emiten respuestas sin convergencia estable"
                    .into(),
            )],
        ));
    }
    let global = result
        .global_displacements
        .as_ref()
        .ok_or_else(|| EngineError::new("ENG-E-0012", "P-Delta has no displacements"))?;
    document["cases"] = json!([{"case":o.lateral.0.case,"factor":o.lateral.0.factor,
        "nodes":nodes(system, global, &result.reactions, false)?,"members":[]}]);
    Ok(())
}

pub(crate) fn buckling(context: MethodContext<'_>) -> Result<(), Failure> {
    let MethodContext {
        execution,
        members,
        contributions,
        system,
        cancel,
        progress,
        document,
        total,
        ..
    } = context;
    reject_combinations(execution)?;
    let Analysis::Buckling {
        options: input::Object(o),
    } = &execution.analyses[0]
    else {
        return Err(invalid("Inconsistent analysis adapter"));
    };
    if o.mode_count > crate::MAX_JSON_ARRAY_ITEMS {
        return Err(invalid(format!(
            "Steps/modes require 0..={}",
            crate::MAX_JSON_ARRAY_ITEMS
        )));
    }
    // geometric_producer recovers N from the reference displacement. A member
    // load would change the end actions without appearing in that displacement.
    if execution
        .model
        .cases
        .iter()
        .zip(&execution.loads)
        .any(|(c, loads)| c.name == o.reference.0.case && !loads.is_empty())
    {
        return Err(EngineError::new(
            "ENG-E-0013",
            "Buckling: the reference case requires nodal loads; Kg does not include member-load correction",
        )
        .into());
    }
    let mut producers = Vec::new();
    for (m, contribution) in members.iter().zip(contributions) {
        cancel.check()?;
        let coords = m
            .nodes
            .iter()
            .map(|n| system.dof_map().coordinates(*n))
            .collect::<Result<Vec<_>, _>>()?;
        producers.push(pdelta::geometric_producer(
            m.element.clone(),
            &ElementCtx {
                dim: m.dim,
                coords: &coords,
                params: &m.params,
            },
            contribution.location.clone(),
            vec![0.0; m.element.state_len()],
        )?);
    }
    system.set_geometric_producers(producers)?;
    let solved = StaticLinear::run_about_with_check(
        system,
        &[(CaseRef(o.reference.0.case.clone()), o.reference.0.factor)],
        [0.0; 3],
        &mut || check_cancel(cancel),
    )?;
    let reference = solved.cases.into_iter().next().ok_or_else(|| {
        EngineError::new("ENG-E-0012", "Buckling: the reference solution is missing.")
    })?;
    system.set_geometric_reference(&reference.free_displacements)?;
    let mut reported = 0usize;
    let result = BucklingLinear::run_with_check(system, o.mode_count, || {
        reported += 1;
        let completed = (3 + reported).min(total.saturating_sub(1));
        checkpoint(cancel, progress, "buckling", completed, total)
    })?;
    let modes = result
        .modes
        .iter()
        .enumerate()
        .map(|(index, mode)| {
            Ok(json!({
                "mode": index + 1,
                "eigenvalue": mode.eigenvalue,
                "normalization": "negative_kg",
                "shape": nodes(system, &mode.global_shape, &[], true)?,
            }))
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    document["buckling"] = json!({
        "rigid_mode_count": result.rigid_mode_count,
        "modes": modes,
    });
    Ok(())
}

pub(crate) fn modal(context: MethodContext<'_>) -> Result<(), Failure> {
    let MethodContext {
        execution,
        members: _,
        contributions,
        system,
        cancel,
        progress,
        document,
        total,
        ..
    } = context;
    reject_combinations(execution)?;
    let Analysis::Modal {
        options: input::Object(o),
    } = &execution.analyses[0]
    else {
        return Err(invalid("Inconsistent analysis adapter"));
    };

    bounded_steps(o.mode_count)?;
    let kind = mass_kind(&o.mass_kind)?;
    prepare_mass(
        execution,
        contributions,
        system,
        o.mass_source.as_ref().map(|s| &s.0),
        document,
    )?;
    cancel.check()?;
    let result = Modal::run_with_check(
        system,
        o.mode_count,
        kind,
        &o.directions,
        |n| checkpoint(cancel, progress, "modal", n + 4, total),
        || check_cancel(cancel),
    );
    cancel.check()?;
    document["modes"] = json!(modes_json(system, &result?)?);
    Ok(())
}

pub(crate) fn spectrum(context: MethodContext<'_>) -> Result<(), Failure> {
    let MethodContext {
        execution,
        members: _,
        contributions,
        system,
        cancel,
        progress,
        document,
        total,
        ..
    } = context;
    reject_combinations(execution)?;
    let Analysis::Spectrum {
        options: input::Object(o),
    } = &execution.analyses[0]
    else {
        return Err(invalid("Inconsistent analysis adapter"));
    };

    bounded_steps(o.mode_count)?;
    bounded_steps(o.mode_count.saturating_mul(o.directions.len()))?;
    let kind = mass_kind(&o.mass_kind)?;
    let options = spectrum::SpectrumOptions {
        mode_count: o.mode_count,
        mass_kind: kind,
        modal_combination: match o.modal_combination.as_str() {
            "srss" => spectrum::ModalCombination::Srss,
            "cqc" => spectrum::ModalCombination::Cqc,
            _ => return Err(invalid("modal_combination requires srss or cqc")),
        },
        directional_combination: match o.directional_combination.as_str() {
            "srss" => spectrum::DirectionalCombination::Srss,
            "weightedSum" => spectrum::DirectionalCombination::WeightedSum,
            _ => {
                return Err(invalid(
                    "directional_combination requires srss or weightedSum",
                ))
            }
        },
        directional_factors: o.directional_factors.clone(),
    };
    let tables = o
        .directions
        .iter()
        .map(|d| {
            d.0.spectra
                .iter()
                .map(|t| table(&t.0))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let directions: Vec<_> = o
        .directions
        .iter()
        .zip(&tables)
        .map(|(d, t)| spectrum::SpectrumDirection {
            direction: d.0.direction,
            spectra: t,
        })
        .collect();
    prepare_mass(
        execution,
        contributions,
        system,
        o.mass_source.as_ref().map(|s| &s.0),
        document,
    )?;
    cancel.check()?;
    let mut completed = 3;
    let result = spectrum::Spectrum::run_with_check(
        system,
        &directions,
        &options,
        |stage, _| {
            completed += 1;
            checkpoint(cancel, progress, stage, completed, total)
        },
        || check_cancel(cancel),
    );
    cancel.check()?;
    let result = result?;
    let mut modes = modes_json(system, &result.modal)?;
    for (n, mode) in modes.iter_mut().enumerate() {
        mode["spectrum_responses"] = json!(result.directions.iter().map(|d| {
            let m = &d.modes[n];
            Ok(json!({"direction":d.unit_direction,"damping":m.damping,
                "spectral_acceleration":m.spectral_acceleration,"modal_displacement":m.modal_displacement,
                "equivalent_forces":m.equivalent_forces,"nodes":nodes(system,&m.response.global_displacements,&m.response.reactions,true)?,"base_shear":m.response.base_shear}))
        }).collect::<Result<Vec<Value>, Failure>>()?);
    }
    document["modes"] = json!(modes);
    let mut envelopes = Vec::new();
    for (index, d) in result.directions.iter().enumerate() {
        envelopes.push(json!({"direction":d.unit_direction,"factor":o.directional_factors[index],
            "nodes":nodes(system,&d.combined.global_displacements,&d.combined.reactions,true)?,"base_shear":d.combined.base_shear}));
    }
    envelopes.push(json!({"direction":null,"factor":null,"nodes":nodes(system,&result.combined.global_displacements,&result.combined.reactions,true)?,"base_shear":result.combined.base_shear}));
    document["envelopes"] = json!(envelopes);
    Ok(())
}

fn table(t: &Table) -> Result<spectrum::TabulatedSpectrum, Failure> {
    Ok(spectrum::TabulatedSpectrum::new(
        &t.pairs.iter().map(|p| (p[0], p[1])).collect::<Vec<_>>(),
        match t.units.as_str() {
            "periodSecondsAccelerationMetersPerSecondSquared" => {
                spectrum::SpectrumUnits::PeriodSecondsAccelerationMetersPerSecondSquared
            }
            "frequencyHertzAccelerationMetersPerSecondSquared" => {
                spectrum::SpectrumUnits::FrequencyHertzAccelerationMetersPerSecondSquared
            }
            _ => return Err(invalid("Invalid SI spectral units")),
        },
        t.damping,
        match t.interpolation.as_str() {
            "linear" => spectrum::Interpolation::Linear,
            "logLog" => spectrum::Interpolation::LogLog,
            _ => return Err(invalid("Invalid interpolation")),
        },
        match t.extrapolation.as_str() {
            "error" => spectrum::Extrapolation::Error,
            "plateau" => spectrum::Extrapolation::Plateau,
            _ => return Err(invalid("Invalid extrapolation")),
        },
    )?)
}
fn nodes(
    system: &LinearSystem<ReferenceBackend>,
    global: &[f64],
    reactions: &[engine_assembly::Reaction],
    perturbation: bool,
) -> Result<Vec<Value>, Failure> {
    if global.len() != system.dof_map().len() {
        return Err(EngineError::new("ENG-E-0012", "Invalid global size").into());
    }
    let mut by_node = BTreeMap::new();
    for (i, &value) in global.iter().enumerate() {
        let (node, dof) = system.dof_map().label(DofIndex(i as u32))?;
        let entry = by_node
            .entry(node)
            .or_insert(([0.0; 6], [None; 6], [0.0; 6]));
        entry.0[dof.index()] = value;
        entry.1[dof.index()] =
            system.constraints().prescribed_values()[i].map(|v| if perturbation { 0.0 } else { v });
    }
    for r in reactions {
        let entry = by_node
            .get_mut(&r.node)
            .ok_or_else(|| EngineError::new("ENG-E-0012", "Missing reaction node"))?;
        entry.2[r.dof.index()] = r.value;
    }
    Ok(by_node.into_iter().map(|(id,(u,p,r))| json!({"id":id.0,"displacements":u,"prescribed_displacements":p,"reactions":r})).collect())
}
fn modes_json(
    system: &LinearSystem<ReferenceBackend>,
    result: &engine_analysis::modal::ModalResults,
) -> Result<Vec<Value>, Failure> {
    result.modes.iter().enumerate().map(|(index,m)| Ok(json!({"mode":index+1,
        "eigenvalue":m.eigenvalue,"angular_frequency":m.angular_frequency,"frequency_hz":m.frequency_hz,"period":m.period,
        "normalization":"mass","shape":nodes(system,&m.global_shape,&[],true)?,
        "participation":m.participation.iter().zip(&result.directions).map(|(p,d)| json!({"direction":d.direction,
            "total_mass":d.total_mass,"free_mass":d.free_mass,"factor":p.factor,"effective_mass":p.effective_mass,
            "cumulative_effective_mass":p.cumulative_effective_mass,"mass_fraction":p.mass_fraction,"cumulative_mass_fraction":p.cumulative_mass_fraction})).collect::<Vec<_>>(),
        "spectrum_responses":[]}))).collect()
}
