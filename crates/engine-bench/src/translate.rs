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
            "Límite MAX_NODES de engine-input/1 superado: {} nudos, máximo {MAX_NODES}.",
            model.nodes.len()
        )));
    }
    if model.members.len() > MAX_MEMBERS {
        return Err(BenchError::new(format!(
            "Límite MAX_MEMBERS de engine-input/1 superado: {} barras, máximo {MAX_MEMBERS}.",
            model.members.len()
        )));
    }
    Ok(())
}

fn check_model(model: &BenchModel) -> Result<(), BenchError> {
    if model.schema != SCHEMA || model.units != "SI" {
        return Err(BenchError::new(
            "El documento debe declarar schema bench-model/1 y units SI.",
        ));
    }
    if !(1..=16_384).contains(&model.modal.n) {
        return Err(BenchError::new(
            "modal.n debe estar entre 1 y 16384. El traductor no lo recorta.",
        ));
    }
    if model.combination.method != "cqc" && model.combination.method != "srss" {
        return Err(BenchError::new("combination.method debe ser cqc o srss."));
    }
    if model.combination.zeta != model.spectrum.zeta {
        return Err(BenchError::new(
            "combination.zeta y spectrum.zeta deben ser el mismo ζ.",
        ));
    }
    if !(0.0..1.0).contains(&model.spectrum.zeta) {
        return Err(BenchError::new("ζ debe pertenecer a [0, 1)."));
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
                "Cada spectrum.variants[].id debe existir y no repetirse.",
            ));
        }
        seen.push(variant.id.clone());
        if variant.method != "cqc" && variant.method != "srss" {
            return Err(BenchError::new(
                "spectrum.variants[].method debe ser cqc o srss.",
            ));
        }
        if variant.directions.is_empty() {
            return Err(BenchError::new(
                "spectrum.variants[].directions necesita al menos una dirección.",
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
            "spectrum.direction y spectrum.directions no se usan juntos. Declare uno.",
        )),
        (None, true) => Err(BenchError::new(
            "Falta spectrum.direction o spectrum.directions.",
        )),
    }
}

fn check_axis_directions(directions: &[SpectrumDirection]) -> Result<(), BenchError> {
    for direction in directions {
        if direction.vector != direction.axis.vector() {
            return Err(BenchError::new(
                "El vector de cada dirección espectral debe ser el unitario de su eje global.",
            ));
        }
    }
    Ok(())
}

fn check_directional(method: &str, factors: usize, directions: usize) -> Result<(), BenchError> {
    if method != "srss" && method != "weightedSum" {
        return Err(BenchError::new(
            "directional_combination.method debe ser srss o weightedSum.",
        ));
    }
    if factors != directions {
        return Err(BenchError::new(
            "directional_combination.factors debe tener un factor por dirección, en el mismo orden.",
        ));
    }
    Ok(())
}

fn check_sa(spectrum: &Spectrum) -> Result<(), BenchError> {
    if spectrum.sa.unit != SA_UNIT {
        return Err(BenchError::new(
            "spectrum.Sa.unit debe ser m/s^2. La ordenada no se acepta en g.",
        ));
    }
    if spectrum.sa.pairs.len() < 2 {
        return Err(BenchError::new("La tabla Sa necesita al menos dos pares."));
    }
    for window in spectrum.sa.pairs.windows(2) {
        if window[0][0] >= window[1][0] {
            return Err(BenchError::new(
                "Los periodos de Sa deben ser estrictamente crecientes.",
            ));
        }
    }
    if spectrum
        .sa
        .pairs
        .iter()
        .any(|pair| pair[0] < 0.0 || pair[1] < 0.0)
    {
        return Err(BenchError::new("T y Sa deben ser no negativos."));
    }
    Ok(())
}

fn check_ids(model: &BenchModel) -> Result<(), BenchError> {
    for (expected, node) in (1_u32..).zip(model.nodes.iter()) {
        if node.id != expected {
            return Err(BenchError::new(
                "Los nudos oficiales y traducibles usan ids contiguos desde 1. \
                 results/1 los lista en ese orden y el puntero del índice depende de ello.",
            ));
        }
    }
    for (member_id, member) in (1_u32..).zip(model.members.iter()) {
        if member.id != member_id {
            return Err(BenchError::new(
                "Las barras deben usar ids contiguos desde 1.",
            ));
        }
        if model.nodes.iter().all(|node| node.id != member.nodes[0])
            || model.nodes.iter().all(|node| node.id != member.nodes[1])
        {
            return Err(BenchError::new(format!(
                "La barra {} cita un nudo ausente.",
                member.id
            )));
        }
        let section = model
            .sections
            .iter()
            .find(|section| section.id == member.section)
            .ok_or_else(|| {
                BenchError::new(format!("La barra {} cita una sección ausente.", member.id))
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
                    "Una columna o una viga exige orientation.kind local_z_vector. No es un ángulo.",
                )
            })?;
            if orientation.kind != crate::model::ORIENTATION_KIND {
                return Err(BenchError::new(
                    "orientation.kind debe ser local_z_vector. No es un ángulo.",
                ));
            }
            if !flexure {
                return Err(BenchError::new(
                    "Una columna o una viga exige G, Iy, Iz y J en la sección.",
                ));
            }
        }
        MemberKind::Truss => {
            if orientation.is_some() {
                return Err(BenchError::new(
                    "truss3d.linear no admite orientation. El Engine la mezcla con los parámetros y la fábrica solo acepta E, A y rho.",
                ));
            }
            if flexure {
                return Err(BenchError::new(
                    "Una celosía solo admite E y A. G, Iy, Iz y J no caben en truss3d.linear.",
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
            "La sección declara G, Iy, Iz y J todos juntos, o ninguno. No se completa una sección a medias.",
        )),
    }
}

fn check_stories(model: &BenchModel) -> Result<(), BenchError> {
    if model.stories.is_empty() {
        return Err(BenchError::new("El modelo necesita al menos un piso."));
    }
    if model.nodal_masses.is_none() && !model.diaphragms.is_empty() {
        if model.stories.len() != model.diaphragms.len() {
            return Err(BenchError::new(
                "Sin nodal_masses, cada piso tiene un diafragma y la base no tiene diafragma.",
            ));
        }
        let mut elevation = 0.0;
        for (story, diaphragm) in model.stories.iter().zip(&model.diaphragms) {
            if story.level != diaphragm.level || story.mass.m <= 0.0 || story.mass.izz <= 0.0 {
                return Err(BenchError::new(
                    "Cada piso necesita level coincidente, m > 0 e Izz > 0.",
                ));
            }
            elevation += story.h;
            if (diaphragm.z - elevation).abs() > 1e-9 {
                return Err(BenchError::new(
                    "La cota del diafragma debe ser la suma de las alturas hasta ese piso.",
                ));
            }
            if diaphragm.slaves.is_empty() || diaphragm.slaves.contains(&diaphragm.master) {
                return Err(BenchError::new(
                    "El diafragma necesita esclavos distintos del maestro.",
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
            "Hay masa de piso y no hay diafragma ni nodal_masses. Declare la masa nodal o un diafragma. El traductor no reparte esa masa por su cuenta.",
        ));
    }
    for story in &model.stories {
        if story.h <= 0.0 || story.mass.m < 0.0 || story.mass.izz < 0.0 {
            return Err(BenchError::new(
                "Cada piso necesita h > 0, m >= 0 e Izz >= 0.",
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
                    "El diafragma necesita la cota acumulada del piso y esclavos distintos del maestro.",
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
        return Err(BenchError::new("Hay dos diafragmas en el mismo piso."));
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
            .ok_or_else(|| BenchError::new("Diafragma sin piso."))?;
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
                "La masa nodal del nudo {} está repetida.",
                mass.node
            )));
        }
        if model.nodes.iter().all(|node| node.id != mass.node) {
            return Err(BenchError::new(format!(
                "La masa nodal cita el nudo ausente {}.",
                mass.node
            )));
        }
        if mass.values.is_empty() {
            return Err(BenchError::new(
                "Cada masa nodal declara al menos un componente. No se inventan los demás.",
            ));
        }
        let mut dofs = [false; 6];
        for value in &mass.values {
            let index = value.dof.index();
            if dofs[index] || !value.value.is_finite() || value.value < 0.0 {
                return Err(BenchError::new(
                    "Cada componente de masa nodal es único, finito y no negativo.",
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
            "load_patterns.patterns reemplaza a gravity y lateral. Deje esos dos vacíos; el traductor no los mezcla.",
        ));
    }
    let mut names = BTreeSet::new();
    let mut cases = Vec::new();
    for pattern in &model.load_patterns.patterns {
        if pattern.name.is_empty() || !names.insert(pattern.name.clone()) {
            return Err(BenchError::new(
                "Cada patrón de carga necesita un nombre propio, sin repetirlo.",
            ));
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
                        "Cada combinación estática necesita un id propio.",
                    ));
                }
                if combination.terms.is_empty() {
                    return Err(BenchError::new(
                        "Una combinación estática necesita al menos un término.",
                    ));
                }
                for term in &combination.terms {
                    if !names.contains(&term.case) || !term.factor.is_finite() {
                        return Err(BenchError::new(format!(
                            "La combinación {} cita un caso ausente o un factor no finito.",
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
                    "pDelta no tiene opciones por defecto. Declare p_delta.gravity, p_delta.lateral, tolerance, max_iterations, displacement_scale y rotation_scale. El traductor no elige esos valores.",
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
                    "p_delta exige tolerance en (0, 1), max_iterations entre 1 y 16384, y escalas positivas finitas. No se sustituyen.",
                ));
            }
            let names = load_cases(model)?
                .iter()
                .filter_map(|item| item.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect::<BTreeSet<_>>();
            if !names.contains(&options.gravity.case) || !names.contains(&options.lateral.case) {
                return Err(BenchError::new(
                    "p_delta.gravity.case y p_delta.lateral.case deben ser casos declarados en el modelo.",
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
            .ok_or_else(|| BenchError::new(format!("No existe la variante espectral {name}.")))?;
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
        return Err(BenchError::new(
            "Cada eje de la retícula necesita dos cotas.",
        ));
    }
    let mut spacing = f64::MAX;
    for window in axis.windows(2) {
        let step = window[1] - window[0];
        if step <= 0.0 {
            return Err(BenchError::new("Las cotas de la retícula deben crecer."));
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
