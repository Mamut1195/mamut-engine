//! Registered adapters for static analysis and its linear postprocessing.
use super::*;

pub(crate) fn budget(execution: &input::Execution) -> usize {
    execution.model.cases.len() + 5
}

pub(crate) fn execute(context: MethodContext<'_>) -> Result<(), Failure> {
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
    let model = &execution.model;
    if model.cases.is_empty() {
        return Err(
            ApiError::InvalidInput("linearStatic requires at least one case".into()).into(),
        );
    }
    let mut cases = Vec::new();
    let mut solutions = Vec::new();
    for (index, case) in model.cases.iter().enumerate() {
        cancel.check()?;
        let solution = StaticLinear::run_about_with_check(
            system,
            &[(CaseRef(case.name.clone()), 1.0)],
            [0.0; 3],
            &mut || {
                cancel.check().map_err(|_| {
                    EngineError::new("ENG-E-0012", "Cancellation requested by the facade")
                })
            },
        );
        cancel.check()?;
        let solution = solution?
            .cases
            .into_iter()
            .next()
            .ok_or_else(|| EngineError::new("ENG-E-0012", "Solver has no case result"))?;
        let views: Vec<_> = members
            .iter()
            .map(|m| MemberRecovery {
                id: m.id,
                nodes: &m.nodes,
                element: m.element.as_ref(),
                dim: m.dim,
                params: &m.params,
                committed: &[],
                loads: &m.loads[index],
                stations: None,
            })
            .collect();
        let recovered = CaseResults::recover_with_check(&solution, system, &views, &mut || {
            check_cancel(cancel)
        });
        cancel.check()?;
        cases.push(recovered?);
        solutions.push(solution);
        step(cancel, progress, "case", index + 4, total)?;
    }
    if !model.combinations.is_empty() {
        let stations: Vec<Vec<f64>> = members
            .iter()
            .map(|member| {
                let mut positions: Vec<_> = cases
                    .iter()
                    .flat_map(|case| {
                        case.members
                            .iter()
                            .filter(|m| m.id == member.id)
                            .flat_map(|m| m.stations.iter().map(|s| s.position))
                    })
                    .collect();
                positions.sort_by(f64::total_cmp);
                positions.dedup();
                positions
            })
            .collect();
        for (index, solution) in solutions.iter().enumerate() {
            cancel.check()?;
            let views: Vec<_> = members
                .iter()
                .zip(&stations)
                .map(|(m, stations)| MemberRecovery {
                    id: m.id,
                    nodes: &m.nodes,
                    element: m.element.as_ref(),
                    dim: m.dim,
                    params: &m.params,
                    committed: &[],
                    loads: &m.loads[index],
                    stations: Some(stations),
                })
                .collect();
            let recovered = CaseResults::recover_with_check(solution, system, &views, &mut || {
                check_cancel(cancel)
            });
            cancel.check()?;
            cases[index] = recovered?;
        }
    }
    document["cases"] = json!(cases.iter().map(case_json).collect::<Vec<_>>());
    if !model.combinations.is_empty() {
        let method = crate::methods::resolve(
            "combinations",
            crate::methods::RESULTS_CONTRACT,
            execution.allow_experimental,
        )?;
        method.run(MethodContext {
            execution,
            members,
            contributions,
            system,
            cancel,
            progress,
            document,
            total,
            cases: &cases,
        })?;
    } else {
        step(cancel, progress, "combinations", total - 1, total)?;
    }
    Ok(())
}

pub(crate) fn combinations(context: MethodContext<'_>) -> Result<(), Failure> {
    let MethodContext {
        execution,
        cancel,
        progress,
        document,
        total,
        cases,
        ..
    } = context;
    let model = &execution.model;
    if cases.len() != model.cases.len() {
        return Err(ApiError::InvalidInput(
            "combinations requires results from all linear cases".into(),
        )
        .into());
    }
    let mut combinations = Vec::new();
    for combination in &model.combinations {
        cancel.check()?;
        let combined =
            engine_analysis::combination::combine_with_check(combination, cases, &mut || {
                check_cancel(cancel)
            });
        cancel.check()?;
        let combined = combined?;
        if combined
            .results
            .nodes
            .iter()
            .any(|n| n.prescribed_displacements.iter().any(Option::is_some))
        {
            let mut warning = diagnostic("API-W-COMBINED-PRESCRIBED", "warning", format!("#85: combination {}: prescribed displacements = sum of factor·δ; a single settlement is weighted by its factor. Repeated settlements are rejected.", combination.id));
            warning
                .entities
                .push(json!({"kind": "case", "id": combination.id}));
            push_diagnostic(document, warning);
        }
        combinations.push(json!({"id": combined.id, "results": case_json(&combined.results)}));
    }
    document["combinations"] = json!(combinations);
    step(cancel, progress, "combinations", total - 1, total)
}

fn check_cancel(cancel: &CancelToken) -> engine_core::EResult<()> {
    cancel
        .check()
        .map_err(|_| EngineError::new("ENG-E-0012", "Cancellation requested by the facade"))
}
