//! Adaptadores registrados del análisis estático y su postproceso lineal.
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
        return Err(ApiError::InvalidInput("linearStatic requiere al menos un caso".into()).into());
    }
    let mut cases = Vec::new();
    let mut solutions = Vec::new();
    for (index, case) in model.cases.iter().enumerate() {
        cancel.check()?;
        let solution = StaticLinear::run(system, &[(CaseRef(case.name.clone()), 1.0)])?
            .cases
            .into_iter()
            .next()
            .ok_or_else(|| EngineError::new("ENG-E-0012", "Solver sin resultado de caso"))?;
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
        cases.push(CaseResults::recover(&solution, system, &views)?);
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
            cases[index] = CaseResults::recover(solution, system, &views)?;
        }
    }
    document["cases"] = json!(cases.iter().map(case_json).collect::<Vec<_>>());
    if !model.combinations.is_empty() {
        let method = crate::methods::resolve(
            "combinations",
            crate::methods::RESULTS_CONTRACT,
            execution.allow_experimental,
        )?;
        (method.execute)(MethodContext {
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
            "combinations requiere resultados de todos los casos lineales".into(),
        )
        .into());
    }
    let mut combinations = Vec::new();
    for combination in &model.combinations {
        cancel.check()?;
        let combined = combine(combination, cases)?;
        if combined
            .results
            .nodes
            .iter()
            .any(|n| n.prescribed_displacements.iter().any(Option::is_some))
        {
            let mut warning = diagnostic("API-W-COMBINED-PRESCRIBED", "warning", format!("#85: combinación {}: prescritos = suma factor·δ; un asentamiento único se pondera por su factor. Asentamientos repetidos se rechazan.", combination.id));
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
