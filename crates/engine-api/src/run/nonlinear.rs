//! Control de frontera por pasos; Newton y las leyes conservan sus algoritmos.
use super::*;
use crate::{analysis_input::Analysis, input::Object};
use engine_analysis::newton::{NewtonElement, NewtonSystem, StaticNewton};
use engine_core::{DofIndex, EResult};

pub(crate) fn budget(e: &input::Execution) -> usize {
    match e.analyses.first() {
        Some(Analysis::Nonlinear { options: Object(o) }) => o.load_factors.len() + 4,
        _ => 4,
    }
}
pub(crate) fn execute(context: MethodContext<'_>) -> Result<(), Failure> {
    let MethodContext {
        execution: e,
        members,
        contributions,
        cancel,
        progress,
        document,
        total,
        ..
    } = context;
    let Some(Analysis::Nonlinear { options: Object(o) }) = e.analyses.first() else {
        return Err(EngineError::new("ENG-E-0012", "Despacho Newton incoherente.").into());
    };
    let options = o.options()?;
    let linear =
        LinearSystem::from_model(ReferenceBackend, &e.model, contributions, &e.prescribed)?;
    let map = linear.dof_map();
    let mut elements = Vec::new();
    for spring in e.springs.iter().filter(|s| s.kind == "spring.nonlinear") {
        cancel.check()?;
        let (dim, element) = spring.build(e)?;
        let coords = spring
            .nodes
            .iter()
            .map(|&id| map.coordinates(engine_core::NodeId(id)))
            .collect::<EResult<Vec<_>>>()?;
        let mut location = Vec::new();
        for &id in &spring.nodes {
            for &dof in element.dofs_per_node() {
                location.push(map.global_index(engine_core::NodeId(id), dof)?);
            }
        }
        elements.push(NewtonElement::new(
            element,
            location,
            &ElementCtx {
                dim,
                coords: &coords,
                params: &ElementParams::default(),
            },
        )?);
    }
    let mut system = NewtonSystem::new(linear, elements)?;
    let mut steps = Vec::new();
    document["convergence"] = json!({"method": "nonlinearStatic", "case": o.case, "status": "ok",
        "accepted_factor": null, "tolerance": o.tolerance, "max_iterations": o.max_iterations,
        "force_scale": options.force_scale, "displacement_tolerance": options.displacement_tolerance, "energy_tolerance": options.energy_tolerance,
        "displacement_scale": options.displacement_scale, "energy_scale": options.energy_scale, "steps": []});
    // Valida todos los productos λF antes del primer paso aceptado, como run
    // sobre la lista completa; el fraccionamiento solo añade checkpoints.
    for &factor in &o.load_factors {
        cancel.check()?;
        system
            .linear_system()
            .global_force(&CaseRef(o.case.clone()), factor)?;
    }
    for (index, &factor) in o.load_factors.iter().enumerate() {
        cancel.check()?;
        let result = StaticNewton::run_with(
            &mut system,
            &CaseRef(o.case.clone()),
            &[factor],
            options,
            || {
                cancel.check().map_err(|_| {
                    EngineError::new("ENG-E-0012", "Cancelación solicitada por la fachada")
                })
            },
        );
        cancel.check()?;
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                if error.code == "ENG-E-0006" {
                    document["convergence"]["status"] = json!("unstable");
                    document["convergence"]["failed_factor"] = json!(factor);
                }
                return Err(error.into());
            }
        };
        let report = result
            .steps
            .first()
            .ok_or_else(|| EngineError::new("ENG-E-0012", "Newton sin informe de paso."))?;
        let response = if report.converged {
            snapshot(&system, members, e, &result)?
        } else {
            Value::Null
        };
        steps.push(json!({"load_factor": report.load_factor, "iterations": report.iterations, "converged": report.converged,
            "force_norm": report.force_norm, "displacement_norm": report.displacement_norm, "energy_norm": report.energy_norm,
            "relative_force_norm": report.relative_force_norm, "relative_displacement_norm": report.relative_displacement_norm,
            "relative_energy_norm": report.relative_energy_norm, "results": response, "element_states": if report.converged { Some(&result.element_states) } else { None }}));
        document["convergence"]["status"] = json!(if report.converged {
            "ok"
        } else {
            "notConverged"
        });
        document["convergence"]["accepted_factor"] = json!(result.accepted_factor);
        document["convergence"]["steps"] = json!(steps);
        step(cancel, progress, "newtonStep", index + 4, total)?;
        if !report.converged {
            return Err(EngineError::new(
                "ENG-E-0007",
                "Newton: no convergió el paso; no se intentan pasos posteriores.",
            )
            .into());
        }
        document["cases"] = json!([steps
            .last()
            .and_then(|s| s.get("results"))
            .cloned()
            .ok_or_else(|| EngineError::new("ENG-E-0012", "Paso Newton sin resultados."))?]);
    }
    Ok(())
}

fn snapshot(
    system: &NewtonSystem<ReferenceBackend>,
    members: &[Member],
    e: &input::Execution,
    result: &engine_analysis::newton::NewtonResults,
) -> Result<Value, Failure> {
    let linear = system.linear_system();
    let map = linear.dof_map();
    let mut internal = linear.global_internal_force(&result.free_displacements)?;
    let factor = result
        .accepted_factor
        .ok_or_else(|| EngineError::new("ENG-E-0012", "Newton sin factor aceptado."))?;
    let force = linear.global_force(&result.case, factor)?;
    let case_index = e
        .model
        .cases
        .iter()
        .position(|c| c.name == result.case.0)
        .ok_or_else(|| EngineError::new("ENG-E-0003", "Caso Newton ausente."))?;
    let mut states = result.element_states.iter();
    let mut recovered = Vec::new();
    for m in members {
        let state: &[f64] = if m.variable {
            states
                .next()
                .ok_or_else(|| EngineError::new("ENG-E-0012", "Falta historia Newton."))?
        } else {
            &[]
        };
        let coords = m
            .nodes
            .iter()
            .map(|&id| map.coordinates(id))
            .collect::<EResult<Vec<_>>>()?;
        let mut location = Vec::new();
        for &id in &m.nodes {
            for &dof in m.element.dofs_per_node() {
                location.push(map.global_index(id, dof)?);
            }
        }
        let u: Vec<_> = location
            .iter()
            .map(|i| result.global_displacements[i.0 as usize])
            .collect();
        let ctx = ElementCtx {
            dim: m.dim,
            coords: &coords,
            params: &m.params,
        };
        if m.variable {
            let response = m.element.respond(&ctx, &u, state)?;
            engine_assembly::assemble::assemble_vector(
                &mut internal,
                &location,
                &response.internal_force,
            )?;
        }
        let mut loads = m.loads[case_index].clone();
        for l in &mut loads {
            for v in l
                .force_per_length
                .iter_mut()
                .chain(&mut l.moment_per_length)
            {
                *v *= factor;
            }
        }
        let out = m.element.recover(
            &ctx,
            &u,
            state,
            &loads,
            &engine_analysis::results::DEFAULT_STATIONS,
        )?;
        let fields = ["N", "Vy", "Vz", "T", "My", "Mz"];
        let mut extremes = Vec::new();
        for (field, name) in fields.iter().enumerate() {
            let mut min = (f64::INFINITY, 0.0);
            let mut max = (f64::NEG_INFINITY, 0.0);
            for s in &out.stations {
                let v = s.forces_local[field];
                if v < min.0 {
                    min = (v, s.position);
                }
                if v > max.0 {
                    max = (v, s.position);
                }
            }
            extremes.push(json!({"field": name, "min": {"value": min.0, "at_station": min.1}, "max": {"value": max.0, "at_station": max.1}}));
        }
        recovered.push(json!({"id": m.id.0, "end_forces_local": out.end_forces_local,
            "stations": out.stations.iter().map(|s| json!({"position": s.position, "forces_local": s.forces_local})).collect::<Vec<_>>(), "extremes": extremes}));
    }
    recovered.sort_by_key(|m| m["id"].as_u64());
    let mut nodes = BTreeMap::new();
    for index in 0..map.len() {
        let (id, dof) = map.label(DofIndex(index as u32))?;
        let n = nodes.entry(id).or_insert(([0.0; 6], [None; 6], [0.0; 6]));
        n.0[dof.index()] = result.global_displacements[index];
        n.1[dof.index()] = linear.constraints().prescribed_values()[index];
        if n.1[dof.index()].is_some() {
            let reaction = internal[index] - force[index];
            if !reaction.is_finite() {
                return Err(EngineError::new("ENG-E-0008", "Reacción Newton no finita.").into());
            }
            n.2[dof.index()] = reaction;
        }
    }
    Ok(
        json!({"case": result.case.0, "factor": factor, "members": recovered,
        "nodes": nodes.into_iter().map(|(id, (u, p, r))| json!({"id": id.0, "displacements": u, "prescribed_displacements": p, "reactions": r})).collect::<Vec<_>>()}),
    )
}
