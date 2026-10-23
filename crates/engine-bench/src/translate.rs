//! Traducción de `bench-model/1` a un documento `engine-input/1`.
//!
//! Cada archivo lleva un solo análisis: el contrato admite como máximo uno.
//! Si `nodal_masses` está ausente, la masa `{m, Izz}` se reparte así, con N
//! nudos en el diafragma y `bay` la menor luz de la retícula:
//! - maestro: Ux = m, Uy = m, Uz = m/N, Rx = Ry = m·bay²/(12·N), Rz = Izz;
//! - cada esclavo: Uz = m/N, Rx = Ry = m·bay²/(12·N).
//!
//! Los esclavos no reciben Ux, Uy ni Rz: eso duplicaría la inercia del diafragma.
//! Si `nodal_masses` está presente, aunque sea vacío, esa lista es la única masa
//! nodal: no se añade Uz, Rx ni Ry. La densidad de barra es 0.

use serde_json::{json, Value};
use std::collections::BTreeSet;

use crate::error::BenchError;
use crate::model::{
    AnalysisKind, BenchModel, Dof, MemberKind, Section, Spectrum, SpectrumDirection, SA_UNIT,
    SCHEMA,
};
use engine_api::{validate, MAX_MEMBERS, MAX_NODES};

/// Traduce y comprueba el contrato `engine-input/1`, incluidos los límites de nudos y barras.
pub fn translate(model: &BenchModel, kind: AnalysisKind) -> Result<String, BenchError> {
    translate_variant(model, kind, None)
}

/// Como [`translate`], eligiendo una entrada de `spectrum.variants` por su `id`.
pub fn translate_variant(
    model: &BenchModel,
    kind: AnalysisKind,
    variant: Option<&str>,
) -> Result<String, BenchError> {
    check_limits(model)?;
    check_model(model)?;
    let value = engine_document(model, kind, variant)?;
    let text = render(&value)?;
    let diagnostics = validate(&text).map_err(|error| BenchError::new(error.to_string()))?;
    if let Some(diagnostic) = diagnostics.iter().find(|item| item.severity == "error") {
        return Err(BenchError::new(format!(
            "{}: {}",
            diagnostic.code, diagnostic.cause
        )));
    }
    Ok(text)
}

fn check_limits(model: &BenchModel) -> Result<(), BenchError> {
    if model.nodes.len() > MAX_NODES {
        return Err(BenchError::new(format!(
            "engine-input/1 MAX_NODES limit exceeded: {} nodes, maximum {MAX_NODES}.",
            model.nodes.len()
        )));
    }
    if model.members.len() > MAX_MEMBERS {
        return Err(BenchError::new(format!(
            "engine-input/1 MAX_MEMBERS limit exceeded: {} members, maximum {MAX_MEMBERS}.",
            model.members.len()
        )));
    }
    Ok(())
}

fn check_model(model: &BenchModel) -> Result<(), BenchError> {
    if model.schema != SCHEMA || model.units != "SI" {
        return Err(BenchError::new(
            "Document must declare schema bench-model/1 and units SI.",
        ));
    }
    if !(1..=16_384).contains(&model.modal.n) {
        return Err(BenchError::new(
            "modal.n must be between 1 and 16384. The translator does not truncate it.",
        ));
    }
    if model.combination.method != "cqc" && model.combination.method != "srss" {
        return Err(BenchError::new("combination.method must be cqc or srss."));
    }
    if model.combination.zeta != model.spectrum.zeta {
        return Err(BenchError::new(
            "combination.zeta and spectrum.zeta must be the same ζ.",
        ));
    }
    if !(0.0..1.0).contains(&model.spectrum.zeta) {
        return Err(BenchError::new("ζ must be in [0, 1)."));
    }
    check_direction(&model.spectrum)?;
    check_sa(&model.spectrum)?;
    check_ids(model)?;
    check_stories(model)?;
    Ok(())
}

fn check_direction(spectrum: &Spectrum) -> Result<(), BenchError> {
    let directions = active_directions(spectrum)?;
    check_axis_directions(&directions)?;
    check_directional(
        &spectrum.directional_combination.method,
        spectrum.directional_combination.factors.len(),
        directions.len(),
    )?;
    let mut seen = Vec::new();
    for variant in &spectrum.variants {
        if variant.id.is_empty() || !seen.iter().all(|id: &String| id != &variant.id) {
            return Err(BenchError::new(
                "Each spectrum.variants[].id must exist and be unique.",
            ));
        }
        seen.push(variant.id.clone());
        if variant.method != "cqc" && variant.method != "srss" {
            return Err(BenchError::new(
                "spectrum.variants[].method must be cqc or srss.",
            ));
        }
        if variant.directions.is_empty() {
            return Err(BenchError::new(
                "spectrum.variants[].directions requires at least one direction.",
            ));
        }
        check_axis_directions(&variant.directions)?;
        check_directional(
            &variant.directional_combination.method,
            variant.directional_combination.factors.len(),
            variant.directions.len(),
        )?;
    }
    Ok(())
}

fn active_directions(spectrum: &Spectrum) -> Result<Vec<SpectrumDirection>, BenchError> {
    match (spectrum.direction.clone(), spectrum.directions.is_empty()) {
        (Some(direction), true) => Ok(vec![direction]),
        (None, false) => Ok(spectrum.directions.clone()),
        (Some(_), false) => Err(BenchError::new(
            "spectrum.direction and spectrum.directions cannot be used together. Declare one.",
        )),
        (None, true) => Err(BenchError::new(
            "Missing spectrum.direction or spectrum.directions.",
        )),
    }
}

fn check_axis_directions(directions: &[SpectrumDirection]) -> Result<(), BenchError> {
    for direction in directions {
        if direction.vector != direction.axis.vector() {
            return Err(BenchError::new(
                "Each spectral direction vector must be the unit vector of its global axis.",
            ));
        }
    }
    Ok(())
}

fn check_directional(method: &str, factors: usize, directions: usize) -> Result<(), BenchError> {
    if method != "srss" && method != "weightedSum" {
        return Err(BenchError::new(
            "directional_combination.method must be srss or weightedSum.",
        ));
    }
    if factors != directions {
        return Err(BenchError::new(
            "directional_combination.factors must have one factor per direction, in the same order.",
        ));
    }
    Ok(())
}

fn check_sa(spectrum: &Spectrum) -> Result<(), BenchError> {
    if spectrum.sa.unit != SA_UNIT {
        return Err(BenchError::new(
            "spectrum.Sa.unit must be m/s^2. The ordinate is not accepted in g.",
        ));
    }
    if spectrum.sa.pairs.len() < 2 {
        return Err(BenchError::new("Sa table requires at least two pairs."));
    }
    for window in spectrum.sa.pairs.windows(2) {
        if window[0][0] >= window[1][0] {
            return Err(BenchError::new("Sa periods must be strictly increasing."));
        }
    }
    if spectrum
        .sa
        .pairs
        .iter()
        .any(|pair| pair[0] < 0.0 || pair[1] < 0.0)
    {
        return Err(BenchError::new("T and Sa must be nonnegative."));
    }
    Ok(())
}

fn check_ids(model: &BenchModel) -> Result<(), BenchError> {
    for (expected, node) in (1_u32..).zip(model.nodes.iter()) {
        if node.id != expected {
            return Err(BenchError::new(
                "Official and translatable nodes use contiguous ids starting at 1. \
                 results/1 lists them in that order and the index pointer depends on it.",
            ));
        }
    }
    for (member_id, member) in (1_u32..).zip(model.members.iter()) {
        if member.id != member_id {
            return Err(BenchError::new(
                "Members must use contiguous ids starting at 1.",
            ));
        }
        if model.nodes.iter().all(|node| node.id != member.nodes[0])
            || model.nodes.iter().all(|node| node.id != member.nodes[1])
        {
            return Err(BenchError::new(format!(
                "Member {} references a missing node.",
                member.id
            )));
        }
        let section = model
            .sections
            .iter()
            .find(|section| section.id == member.section)
            .ok_or_else(|| {
                BenchError::new(format!(
                    "Member {} references a missing section.",
                    member.id
                ))
            })?;
        check_member_section(member.kind, member.orientation.as_ref(), section)?;
    }
    Ok(())
}

fn check_member_section(
    kind: MemberKind,
    orientation: Option<&crate::model::Orientation>,
    section: &Section,
) -> Result<(), BenchError> {
    let flexure = section_flexure(section)?;
    match kind {
        MemberKind::Column | MemberKind::Beam => {
            let orientation = orientation.ok_or_else(|| {
                BenchError::new(
                    "A column or beam requires orientation.kind local_z_vector. It is not an angle.",
                )
            })?;
            if orientation.kind != crate::model::ORIENTATION_KIND {
                return Err(BenchError::new(
                    "orientation.kind must be local_z_vector. It is not an angle.",
                ));
            }
            if !flexure {
                return Err(BenchError::new(
                    "A column or beam requires G, Iy, Iz, and J in the section.",
                ));
            }
        }
        MemberKind::Truss => {
            if orientation.is_some() {
                return Err(BenchError::new(
                    "truss3d.linear does not support orientation. The Engine mixes it with the parameters and the factory accepts only E, A, and rho.",
                ));
            }
            if flexure {
                return Err(BenchError::new(
                    "A truss supports only E and A. G, Iy, Iz, and J are not supported in truss3d.linear.",
                ));
            }
        }
    }
    Ok(())
}

fn section_flexure(section: &Section) -> Result<bool, BenchError> {
    let present = [
        section.shear.is_some(),
        section.iy.is_some(),
        section.iz.is_some(),
        section.j.is_some(),
    ]
    .into_iter()
    .filter(|item| *item)
    .count();
    match present {
        4 => Ok(true),
        0 => Ok(false),
        _ => Err(BenchError::new(
            "Section must declare all of G, Iy, Iz, and J, or none of them. A partially specified section is not completed.",
        )),
    }
}

fn check_stories(model: &BenchModel) -> Result<(), BenchError> {
    if model.stories.is_empty() {
        return Err(BenchError::new("Model requires at least one floor."));
    }
    if model.nodal_masses.is_none() && !model.diaphragms.is_empty() {
        if model.stories.len() != model.diaphragms.len() {
            return Err(BenchError::new(
                "Without nodal_masses, each floor has one diaphragm and the base has none.",
            ));
        }
        let mut elevation = 0.0;
        for (story, diaphragm) in model.stories.iter().zip(&model.diaphragms) {
            if story.level != diaphragm.level || story.mass.m <= 0.0 || story.mass.izz <= 0.0 {
                return Err(BenchError::new(
                    "Each floor requires a matching level, m > 0, and Izz > 0.",
                ));
            }
            elevation += story.h;
            if (diaphragm.z - elevation).abs() > 1e-9 {
                return Err(BenchError::new(
                    "Diaphragm elevation must be the sum of heights up to that floor.",
                ));
            }
            if diaphragm.slaves.is_empty() || diaphragm.slaves.contains(&diaphragm.master) {
                return Err(BenchError::new(
                    "Diaphragm requires slaves distinct from the master.",
                ));
            }
        }
        return Ok(());
    }
    if model.nodal_masses.is_none()
        && model
            .stories
            .iter()
            .any(|story| story.mass.m != 0.0 || story.mass.izz != 0.0)
    {
        return Err(BenchError::new(
            "Floor mass is present without a diaphragm or nodal_masses. Declare nodal mass or a diaphragm. The translator does not distribute that mass on its own.",
        ));
    }
    for story in &model.stories {
        if story.h <= 0.0 || story.mass.m < 0.0 || story.mass.izz < 0.0 {
            return Err(BenchError::new(
                "Each floor requires h > 0, m >= 0, and Izz >= 0.",
            ));
        }
    }
    let mut elevation = 0.0;
    for story in &model.stories {
        elevation += story.h;
        if let Some(diaphragm) = model
            .diaphragms
            .iter()
            .find(|diaphragm| diaphragm.level == story.level)
        {
            if (diaphragm.z - elevation).abs() > 1e-9
                || diaphragm.slaves.is_empty()
                || diaphragm.slaves.contains(&diaphragm.master)
            {
                return Err(BenchError::new(
                    "Diaphragm requires the accumulated floor elevation and slaves distinct from the master.",
                ));
            }
        }
    }
    if model.diaphragms.len()
        != model
            .diaphragms
            .iter()
            .map(|diaphragm| diaphragm.level)
            .collect::<BTreeSet<_>>()
            .len()
    {
        return Err(BenchError::new(
            "There are two diaphragms on the same floor.",
        ));
    }
    Ok(())
}

fn engine_document(
    model: &BenchModel,
    kind: AnalysisKind,
    variant: Option<&str>,
) -> Result<Value, BenchError> {
    let bay = min_bay(&model.grid)?;
    let nodes = model
        .nodes
        .iter()
        .map(|node| {
            json!({
                "id": node.id,
                "dim": "D3",
                "coordinates": node.coordinates,
                "dofs": ["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"]
            })
        })
        .collect::<Vec<_>>();
    let mut members = Vec::new();
    for member in &model.members {
        let law = law_name(&member.section, model)?;
        let section = model
            .sections
            .iter()
            .find(|item| item.id == member.section)
            .expect("sección comprobada");
        let flexure = section_flexure(section)?;
        members.push(if flexure {
            json!({
                "id": member.id,
                "nodes": member.nodes,
                "section": member.section,
                "law": law,
                "element": "frame3d.euler_elastic",
                "orientation": member.orientation.as_ref().expect("orientación comprobada").vector
            })
        } else {
            json!({
                "id": member.id,
                "nodes": member.nodes,
                "section": member.section,
                "law": law,
                "element": "truss3d.linear"
            })
        });
    }
    let document = json!({
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": nodes,
            "members": members,
            "sections": sections(model)?,
            "laws": laws(model)?,
            "supports": supports(model),
            "masses": masses(model, bay)?,
            "diaphragms": model.diaphragms.iter().map(|diaphragm| json!({
                "master": diaphragm.master,
                "slaves": diaphragm.slaves
            })).collect::<Vec<_>>()
        },
        "cases": cases(model, kind)?,
        "combinations": static_combinations(model, kind)?,
        "analyses": [analysis(model, kind, variant)?],
        "options": {}
    });
    Ok(document)
}

fn law_name(section: &str, model: &BenchModel) -> Result<String, BenchError> {
    let current = model
        .sections
        .iter()
        .find(|item| item.id == section)
        .expect("sección comprobada");
    if !section_flexure(current)? {
        return Ok(format!("elastic-{}", current.id));
    }
    let shear = current.shear.expect("sección de pórtico");
    let first = model
        .sections
        .iter()
        .find(|item| item.young == current.young && item.shear == Some(shear))
        .expect("sección comprobada");
    Ok(format!("elastic-{}", first.id))
}

fn sections(model: &BenchModel) -> Result<Vec<Value>, BenchError> {
    let mut sections = Vec::new();
    for section in &model.sections {
        let properties = if section_flexure(section)? {
            vec![
                json!({"name": "A", "value": section.area}),
                json!({"name": "Iy", "value": section.iy}),
                json!({"name": "Iz", "value": section.iz}),
                json!({"name": "J", "value": section.j}),
            ]
        } else {
            vec![json!({"name": "A", "value": section.area})]
        };
        sections.push(json!({"id": section.id, "properties": properties}));
    }
    Ok(sections)
}

fn laws(model: &BenchModel) -> Result<Vec<Value>, BenchError> {
    let mut laws = Vec::new();
    let mut seen = Vec::new();
    for section in &model.sections {
        if !section_flexure(section)? {
            laws.push(json!({
                "name": format!("elastic-{}", section.id),
                "parameters": [
                    {"name": "E", "value": section.young},
                    {"name": "rho", "value": 0.0}
                ]
            }));
            continue;
        }
        let shear = section.shear.expect("sección de pórtico");
        let key = (section.young.to_bits(), shear.to_bits());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        laws.push(json!({
            "name": format!("elastic-{}", section.id),
            "parameters": [
                {"name": "E", "value": section.young},
                {"name": "G", "value": shear},
                {"name": "rho", "value": 0.0}
            ]
        }));
    }
    Ok(laws)
}

fn supports(model: &BenchModel) -> Vec<Value> {
    model
        .restraints
        .iter()
        .map(|restraint| {
            json!({
                "id": format!("support-{}", restraint.node),
                "node": restraint.node,
                "restrained": restraint.dofs
            })
        })
        .collect()
}

fn masses(model: &BenchModel, bay: f64) -> Result<Vec<Value>, BenchError> {
    if let Some(declared) = &model.nodal_masses {
        return declared_masses(model, declared);
    }
    let mut masses = Vec::new();
    for diaphragm in &model.diaphragms {
        let story = model
            .stories
            .iter()
            .find(|story| story.level == diaphragm.level)
            .ok_or_else(|| BenchError::new("Diaphragm has no floor."))?;
        let count = diaphragm.slaves.len() + 1;
        let share = story.mass.m / count as f64;
        let rotary = story.mass.m * bay * bay / (12.0 * count as f64);
        masses.push(mass_entry(
            diaphragm.master,
            vec![
                (Dof::Ux, story.mass.m),
                (Dof::Uy, story.mass.m),
                (Dof::Uz, share),
                (Dof::Rx, rotary),
                (Dof::Ry, rotary),
                (Dof::Rz, story.mass.izz),
            ],
        ));
        for slave in &diaphragm.slaves {
            masses.push(mass_entry(
                *slave,
                vec![(Dof::Uz, share), (Dof::Rx, rotary), (Dof::Ry, rotary)],
            ));
        }
    }
    Ok(masses)
}

fn declared_masses(
    model: &BenchModel,
    declared: &[crate::model::NodalMass],
) -> Result<Vec<Value>, BenchError> {
    let mut seen = BTreeSet::new();
    let mut masses = Vec::new();
    for mass in declared {
        if !seen.insert(mass.node) {
            return Err(BenchError::new(format!(
                "Duplicate nodal mass at node {}.",
                mass.node
            )));
        }
        if model.nodes.iter().all(|node| node.id != mass.node) {
            return Err(BenchError::new(format!(
                "Nodal mass references missing node {}.",
                mass.node
            )));
        }
        if mass.values.is_empty() {
            return Err(BenchError::new(
                "Each nodal mass declares at least one component. The others are not invented.",
            ));
        }
        let mut dofs = [false; 6];
        for value in &mass.values {
            let index = value.dof.index();
            if dofs[index] || !value.value.is_finite() || value.value < 0.0 {
                return Err(BenchError::new(
                    "Each nodal mass component must be unique, finite, and nonnegative.",
                ));
            }
            dofs[index] = true;
        }
        masses.push(json!({
            "id": format!("mass-{}", mass.node),
            "node": mass.node,
            "values": mass.values
        }));
    }
    Ok(masses)
}

fn mass_entry(node: u32, values: Vec<(Dof, f64)>) -> Value {
    json!({
        "id": format!("mass-{node}"),
        "node": node,
        "values": values.into_iter().map(|(dof, value)| json!({"dof": dof, "value": value})).collect::<Vec<_>>()
    })
}

fn cases(model: &BenchModel, kind: AnalysisKind) -> Result<Vec<Value>, BenchError> {
    match kind {
        AnalysisKind::Static | AnalysisKind::PDelta => load_cases(model),
        AnalysisKind::Modal | AnalysisKind::Spectrum => Ok(Vec::new()),
    }
}

fn load_cases(model: &BenchModel) -> Result<Vec<Value>, BenchError> {
    if model.load_patterns.patterns.is_empty() {
        return Ok(vec![
            case("gravity", &model.load_patterns.gravity),
            case("lateral", &model.load_patterns.lateral),
        ]);
    }
    if !model.load_patterns.gravity.is_empty() || !model.load_patterns.lateral.is_empty() {
        return Err(BenchError::new(
            "load_patterns.patterns replaces gravity and lateral. Leave both empty; the translator does not mix them.",
        ));
    }
    let mut names = BTreeSet::new();
    let mut cases = Vec::new();
    for pattern in &model.load_patterns.patterns {
        if pattern.name.is_empty() || !names.insert(pattern.name.clone()) {
            return Err(BenchError::new("Each load pattern requires a unique name."));
        }
        cases.push(case(&pattern.name, &pattern.actions));
    }
    Ok(cases)
}

fn static_combinations(model: &BenchModel, kind: AnalysisKind) -> Result<Vec<Value>, BenchError> {
    match kind {
        AnalysisKind::Modal | AnalysisKind::Spectrum | AnalysisKind::PDelta => Ok(Vec::new()),
        AnalysisKind::Static => {
            let names = load_cases(model)?
                .iter()
                .filter_map(|item| item.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect::<BTreeSet<_>>();
            let mut ids = BTreeSet::new();
            let mut combinations = Vec::new();
            for combination in &model.static_combinations {
                if combination.id.is_empty() || !ids.insert(combination.id.clone()) {
                    return Err(BenchError::new(
                        "Each static combination requires a unique id.",
                    ));
                }
                if combination.terms.is_empty() {
                    return Err(BenchError::new(
                        "A static combination requires at least one term.",
                    ));
                }
                for term in &combination.terms {
                    if !names.contains(&term.case) || !term.factor.is_finite() {
                        return Err(BenchError::new(format!(
                            "Combination {} references a missing case or a non-finite factor.",
                            combination.id
                        )));
                    }
                }
                combinations.push(json!({
                    "id": combination.id,
                    "terms": combination.terms
                }));
            }
            Ok(combinations)
        }
    }
}

fn case(name: &str, actions: &[crate::model::NodalAction]) -> Value {
    json!({
        "name": name,
        "loads": actions.iter().map(|action| json!({
            "node": action.node,
            "values": action.values
        })).collect::<Vec<_>>()
    })
}

fn analysis(
    model: &BenchModel,
    kind: AnalysisKind,
    variant: Option<&str>,
) -> Result<Value, BenchError> {
    Ok(match kind {
        AnalysisKind::Static => json!({"method": "linearStatic"}),
        AnalysisKind::Modal => json!({
            "method": "modal",
            "options": {
                "mode_count": model.modal.n,
                "mass_kind": model.modal.mass_kind,
                "directions": model.modal.directions
            }
        }),
        AnalysisKind::Spectrum => {
            let choice = spectrum_choice(model, variant)?;
            let directions = choice
                .directions
                .iter()
                .map(|direction| {
                    json!({
                        "direction": direction.vector,
                        "spectra": [{
                            "pairs": model.spectrum.sa.pairs,
                            "units": "periodSecondsAccelerationMetersPerSecondSquared",
                            "damping": model.spectrum.zeta,
                            "interpolation": model.spectrum.interpolation,
                            "extrapolation": model.spectrum.extrapolation
                        }]
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "method": "responseSpectrum",
                "options": {
                    "mode_count": model.modal.n,
                    "mass_kind": model.modal.mass_kind,
                    "directions": directions,
                    "modal_combination": choice.method,
                    "directional_combination": choice.directional_method,
                    "directional_factors": choice.factors
                }
            })
        }
        AnalysisKind::PDelta => {
            let options = model.p_delta.as_ref().ok_or_else(|| {
                BenchError::new(
                    "pDelta has no default options. Declare p_delta.gravity, p_delta.lateral, tolerance, max_iterations, displacement_scale, and rotation_scale. The translator does not choose these values.",
                )
            })?;
            if !(0.0..1.0).contains(&options.tolerance)
                || options.max_iterations == 0
                || options.max_iterations > 16_384
                || options.displacement_scale <= 0.0
                || options.rotation_scale <= 0.0
                || !options.displacement_scale.is_finite()
                || !options.rotation_scale.is_finite()
                || !options.gravity.factor.is_finite()
                || !options.lateral.factor.is_finite()
            {
                return Err(BenchError::new(
                    "p_delta requires tolerance in (0, 1), max_iterations between 1 and 16384, and positive finite scales. They are not substituted.",
                ));
            }
            let names = load_cases(model)?
                .iter()
                .filter_map(|item| item.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect::<BTreeSet<_>>();
            if !names.contains(&options.gravity.case) || !names.contains(&options.lateral.case) {
                return Err(BenchError::new(
                    "p_delta.gravity.case and p_delta.lateral.case must be cases declared in the model.",
                ));
            }
            json!({
                "method": "pDelta",
                "options": {
                    "gravity": {"case": options.gravity.case, "factor": options.gravity.factor},
                    "lateral": {"case": options.lateral.case, "factor": options.lateral.factor},
                    "tolerance": options.tolerance,
                    "max_iterations": options.max_iterations,
                    "displacement_scale": options.displacement_scale,
                    "rotation_scale": options.rotation_scale
                }
            })
        }
    })
}

struct SpectrumChoice {
    method: String,
    directions: Vec<SpectrumDirection>,
    directional_method: String,
    factors: Vec<f64>,
}

fn spectrum_choice(
    model: &BenchModel,
    variant: Option<&str>,
) -> Result<SpectrumChoice, BenchError> {
    if let Some(name) = variant {
        let found = model
            .spectrum
            .variants
            .iter()
            .find(|item| item.id == name)
            .ok_or_else(|| BenchError::new(format!("Spectral variant {name} does not exist.")))?;
        return Ok(SpectrumChoice {
            method: found.method.clone(),
            directions: found.directions.clone(),
            directional_method: found.directional_combination.method.clone(),
            factors: found.directional_combination.factors.clone(),
        });
    }
    Ok(SpectrumChoice {
        method: model.combination.method.clone(),
        directions: active_directions(&model.spectrum)?,
        directional_method: model.spectrum.directional_combination.method.clone(),
        factors: model.spectrum.directional_combination.factors.clone(),
    })
}

fn min_bay(grid: &crate::model::Grid) -> Result<f64, BenchError> {
    let dx = min_step(&grid.x)?;
    let dy = min_step(&grid.y)?;
    Ok(dx.min(dy))
}

fn min_step(axis: &[f64]) -> Result<f64, BenchError> {
    if axis.len() < 2 {
        return Err(BenchError::new("Each grid axis requires two coordinates."));
    }
    let mut spacing = f64::MAX;
    for window in axis.windows(2) {
        let step = window[1] - window[0];
        if step <= 0.0 {
            return Err(BenchError::new("Grid coordinates must increase."));
        }
        spacing = spacing.min(step);
    }
    Ok(spacing)
}

pub(crate) fn render(value: &impl serde::Serialize) -> Result<String, BenchError> {
    let mut text =
        serde_json::to_string_pretty(value).map_err(|error| BenchError::new(error.to_string()))?;
    text.push('\n');
    Ok(text)
}
