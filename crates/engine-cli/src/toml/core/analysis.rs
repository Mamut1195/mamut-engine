// SPDX-License-Identifier: AGPL-3.0-only

//! Analysis branches of mamut-toml/1.
//!
//! Options stay flattened in TOML and nested under `options` in JSON.
//! `linearStatic` has no options object. No tolerance, factor, mass source,
//! or geotechnical value is invented. A key from another branch is rejected.

use std::collections::BTreeSet;

use serde_json::{json, Map, Value as Json};
use toml::Value as Toml;

use super::Failure;

const MASS_METHODS: [&str; 3] = [
    "masses_from_case",
    "system_case",
    "masses_from_system_case",
];
const MASS_KINDS: [&str; 2] = ["lumped", "consistent"];
const MODAL_RULES: [&str; 2] = ["srss", "cqc"];
const DIRECTIONAL_RULES: [&str; 2] = ["srss", "weightedSum"];
const SPECTRUM_UNITS: [&str; 2] = [
    "periodSecondsAccelerationMetersPerSecondSquared",
    "frequencyHertzAccelerationMetersPerSecondSquared",
];
const INTERPOLATION: [&str; 2] = ["linear", "logLog"];
const EXTRAPOLATION: [&str; 2] = ["error", "plateau"];
const DRAINAGE: [&str; 3] = ["top", "bottom", "double"];
const PROVENANCE: [&str; 2] = ["measured", "assumed"];
const P_DELTA: [&str; 6] = [
    "gravity",
    "lateral",
    "tolerance",
    "max_iterations",
    "displacement_scale",
    "rotation_scale",
];
const MODAL: [&str; 4] = ["mode_count", "mass_kind", "directions", "mass_source"];
const SPECTRUM: [&str; 7] = [
    "mode_count",
    "mass_kind",
    "directions",
    "modal_combination",
    "directional_combination",
    "directional_factors",
    "mass_source",
];
const NEWTON: [&str; 9] = [
    "case",
    "load_factors",
    "tolerance",
    "max_iterations",
    "force_scale",
    "displacement_scale",
    "displacement_tolerance",
    "energy_scale",
    "energy_tolerance",
];
const SOIL: [&str; 4] = [
    "layers",
    "depths_m",
    "groundwater_depth_m",
    "water_unit_weight_kn_m3",
];
const CONSOLIDATION: [&str; 7] = [
    "layers",
    "drainage",
    "history",
    "output_times_s",
    "theta",
    "max_step_s",
    "water_unit_weight_kn_m3",
];

/// Facts outside `[analysis]` that still select a legal branch.
pub(super) struct BranchContext {
    /// Root `options.allow_experimental`.
    pub experimental: bool,
    /// Structural collections, cases, and combinations are all empty.
    pub structural_empty: bool,
    /// No combination is present.
    pub combinations_empty: bool,
    /// No diaphragm is present.
    pub diaphragms_empty: bool,
}

/// One `analyses[0]` object.
pub(super) struct MappedAnalysis {
    /// engine-input/1 analysis object.
    pub body: Json,
    /// True only when `method` is `nonlinearStatic`.
    pub nonlinear: bool,
}

/// Map `[analysis]` into one engine-input/1 analysis object.
pub(super) fn from_toml(
    root: &toml::Table,
    cases: &BTreeSet<String>,
    side: &BranchContext,
) -> Result<MappedAnalysis, Failure> {
    let Some(value) = root.get("analysis") else {
        return Err(Failure::validation(
            Some(String::new()),
            "The analysis field is missing.",
        ));
    };
    let table = super::as_table(value, "/analysis")?;
    let method = match table.get("method") {
        Some(Toml::String(value)) => value.as_str(),
        Some(_) => {
            return Err(invalid(
                "/analysis",
                "A value has the wrong type.",
            ))
        }
        None => {
            return Err(invalid(
                "/analysis",
                "A required field is missing.",
            ))
        }
    };
    let body = match method {
        "linearStatic" => linear_toml(table)?,
        "pDelta" => p_delta_toml(table, cases)?,
        "modal" => modal_toml(table, cases)?,
        "responseSpectrum" => spectrum_toml(table, cases)?,
        "nonlinearStatic" => newton_toml(table, cases)?,
        "soilProfile" => soil_toml(table)?,
        "consolidation1d" => consolidation_toml(table)?,
        _ => {
            return Err(invalid(
                "/analysis",
                "This analysis method is outside the implemented subset.",
            ))
        }
    };
    finish(method, body, side, "/analysis")
}

/// Validate `analyses`, which must contain exactly one analysis.
pub(super) fn from_json(
    root: &Map<String, Json>,
    cases: &BTreeSet<String>,
    side: &BranchContext,
) -> Result<MappedAnalysis, Failure> {
    let Some(value) = root.get("analyses") else {
        return Err(Failure::validation(
            Some(String::new()),
            "The analysis field is missing.",
        ));
    };
    let Some(items) = value.as_array() else {
        return Err(invalid("/analyses", "A collection has the wrong shape."));
    };
    if items.len() != 1 {
        return Err(invalid("/analyses", "A required field is missing."));
    }
    let analysis = super::as_object(&items[0], "/analyses/0")?;
    let method = match analysis.get("method") {
        Some(Json::String(value)) => value.as_str(),
        Some(_) => {
            return Err(invalid(
                "/analyses/0",
                "A value has the wrong type.",
            ))
        }
        None => {
            return Err(invalid(
                "/analyses/0",
                "A required field is missing.",
            ))
        }
    };
    match method {
        "linearStatic" => linear_json(analysis)?,
        "pDelta" => p_delta_json(analysis, cases)?,
        "modal" => modal_json(analysis, cases)?,
        "responseSpectrum" => spectrum_json(analysis, cases)?,
        "nonlinearStatic" => newton_json(analysis, cases)?,
        "soilProfile" => soil_json(analysis)?,
        "consolidation1d" => consolidation_json(analysis)?,
        _ => {
            return Err(invalid(
                "/analyses/0",
                "This analysis method is outside the implemented subset.",
            ))
        }
    }
    finish(method, items[0].clone(), side, "/analyses/0")
}

/// Emit the flattened `[analysis]` section.
pub(super) fn write(out: &mut super::Out, mapped: &MappedAnalysis) -> Result<(), Failure> {
    let analysis = super::as_object(&mapped.body, "/analyses/0")?;
    let method = match analysis.get("method").and_then(Json::as_str) {
        Some(method) => method,
        None => {
            return Err(invalid(
                "/analyses/0",
                "A required field is missing.",
            ))
        }
    };
    out.header("[analysis]");
    out.entry("method", &super::quoted(method));
    match method {
        "linearStatic" => Ok(()),
        "pDelta" => write_p_delta(out, analysis),
        "modal" => write_modal(out, analysis),
        "responseSpectrum" => write_spectrum(out, analysis),
        "nonlinearStatic" => write_newton(out, analysis),
        "soilProfile" => write_soil(out, analysis),
        "consolidation1d" => write_consolidation(out, analysis),
        _ => Err(invalid(
            "/analyses/0",
            "This analysis method is outside the implemented subset.",
        )),
    }
}

fn finish(
    method: &str,
    body: Json,
    side: &BranchContext,
    pointer: &str,
) -> Result<MappedAnalysis, Failure> {
    let experimental = matches!(
        method,
        "nonlinearStatic" | "soilProfile" | "consolidation1d"
    );
    if experimental && !side.experimental {
        return Err(invalid(
            pointer,
            "An experimental method requires allow_experimental.",
        ));
    }
    if method == "nonlinearStatic" && (!side.combinations_empty || !side.diaphragms_empty) {
        return Err(invalid(
            pointer,
            "Nonlinear static does not allow combinations or diaphragms.",
        ));
    }
    if matches!(method, "soilProfile" | "consolidation1d") && !side.structural_empty {
        return Err(invalid(
            pointer,
            "A geotechnical analysis requires an empty structural model.",
        ));
    }
    Ok(MappedAnalysis {
        body,
        nonlinear: method == "nonlinearStatic",
    })
}

fn linear_toml(table: &toml::Table) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &["method"], "/analysis")?;
    Ok(json!({"method": "linearStatic"}))
}

fn p_delta_toml(table: &toml::Table, cases: &BTreeSet<String>) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &with_method(&P_DELTA), "/analysis")?;
    let gravity = term_toml(nested(table, "gravity", "/analysis", "/analysis/gravity")?, cases, "/analysis/gravity")?;
    let lateral = term_toml(nested(table, "lateral", "/analysis", "/analysis/lateral")?, cases, "/analysis/lateral")?;
    Ok(json!({
        "method": "pDelta",
        "options": {
            "gravity": gravity,
            "lateral": lateral,
            "tolerance": open_unit_toml(table, "tolerance", "/analysis")?,
            "max_iterations": int_toml(table, "max_iterations", 1, 16_384, "/analysis")?,
            "displacement_scale": positive_toml(table, "displacement_scale", "/analysis")?,
            "rotation_scale": positive_toml(table, "rotation_scale", "/analysis")?,
        }
    }))
}

fn modal_toml(table: &toml::Table, cases: &BTreeSet<String>) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &with_method(&MODAL), "/analysis")?;
    let mut options = Map::new();
    options.insert(
        "mode_count".to_string(),
        int_toml(table, "mode_count", 1, 16_384, "/analysis")?,
    );
    options.insert(
        "mass_kind".to_string(),
        json!(choice_toml(table, "mass_kind", &MASS_KINDS, "/analysis")?),
    );
    options.insert(
        "directions".to_string(),
        vector_list_toml(required(table, "directions", "/analysis")?, "/analysis")?,
    );
    if let Some(source) = optional_mass_source(table, cases)? {
        options.insert("mass_source".to_string(), source);
    }
    Ok(json!({"method": "modal", "options": options}))
}

fn spectrum_toml(table: &toml::Table, cases: &BTreeSet<String>) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &with_method(&SPECTRUM), "/analysis")?;
    let directions = array_of(table, "directions", "/analysis")?;
    let mut options = Map::new();
    options.insert(
        "mode_count".to_string(),
        int_toml(table, "mode_count", 1, 16_384, "/analysis")?,
    );
    options.insert(
        "mass_kind".to_string(),
        json!(choice_toml(table, "mass_kind", &MASS_KINDS, "/analysis")?),
    );
    options.insert(
        "directions".to_string(),
        spectrum_directions_toml(directions, "/analysis/directions")?,
    );
    options.insert(
        "modal_combination".to_string(),
        json!(choice_toml(table, "modal_combination", &MODAL_RULES, "/analysis")?),
    );
    options.insert(
        "directional_combination".to_string(),
        json!(choice_toml(
            table,
            "directional_combination",
            &DIRECTIONAL_RULES,
            "/analysis",
        )?),
    );
    options.insert(
        "directional_factors".to_string(),
        number_list_toml(
            required(table, "directional_factors", "/analysis")?,
            1,
            None,
            |value| value >= 0.0,
            "/analysis",
        )?,
    );
    if let Some(source) = optional_mass_source(table, cases)? {
        options.insert("mass_source".to_string(), source);
    }
    Ok(json!({"method": "responseSpectrum", "options": options}))
}

fn newton_toml(table: &toml::Table, cases: &BTreeSet<String>) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &with_method(&NEWTON), "/analysis")?;
    let case_name = text_toml(table, "case", "/analysis")?;
    if !cases.contains(&case_name) {
        return Err(invalid("/analysis", "A reference does not exist."));
    }
    let mut options = Map::new();
    options.insert("case".to_string(), json!(case_name));
    options.insert(
        "load_factors".to_string(),
        number_list_toml(
            required(table, "load_factors", "/analysis")?,
            1,
            Some(16_384),
            |_| true,
            "/analysis",
        )?,
    );
    options.insert(
        "tolerance".to_string(),
        positive_toml(table, "tolerance", "/analysis")?,
    );
    options.insert(
        "max_iterations".to_string(),
        int_toml(table, "max_iterations", 1, 16_384, "/analysis")?,
    );
    options.insert(
        "force_scale".to_string(),
        positive_toml(table, "force_scale", "/analysis")?,
    );
    criterion_toml(
        table,
        "displacement_tolerance",
        "displacement_scale",
        &mut options,
    )?;
    criterion_toml(table, "energy_tolerance", "energy_scale", &mut options)?;
    Ok(json!({"method": "nonlinearStatic", "options": options}))
}

fn soil_toml(table: &toml::Table) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &with_method(&SOIL), "/analysis")?;
    let mut options = Map::new();
    options.insert(
        "layers".to_string(),
        soil_layers_toml(array_of(table, "layers", "/analysis")?, "/analysis/layers")?,
    );
    options.insert(
        "depths_m".to_string(),
        number_list_toml(
            required(table, "depths_m", "/analysis")?,
            1,
            Some(16_384),
            |value| value >= 0.0,
            "/analysis",
        )?,
    );
    optional_geo_toml(table, "groundwater_depth_m", &mut options, "/analysis")?;
    optional_geo_toml(table, "water_unit_weight_kn_m3", &mut options, "/analysis")?;
    Ok(json!({"method": "soilProfile", "options": options}))
}

fn consolidation_toml(table: &toml::Table) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &with_method(&CONSOLIDATION), "/analysis")?;
    let end = history_end_toml(array_of(table, "history", "/analysis")?, "/analysis/history")?;
    let mut options = Map::new();
    options.insert(
        "layers".to_string(),
        consolidation_layers_toml(array_of(table, "layers", "/analysis")?, "/analysis/layers")?,
    );
    options.insert(
        "drainage".to_string(),
        json!(choice_toml(table, "drainage", &DRAINAGE, "/analysis")?),
    );
    options.insert(
        "history".to_string(),
        history_toml(array_of(table, "history", "/analysis")?, "/analysis/history")?,
    );
    options.insert(
        "output_times_s".to_string(),
        output_times_toml(
            required(table, "output_times_s", "/analysis")?,
            end,
            "/analysis",
        )?,
    );
    options.insert(
        "theta".to_string(),
        closed_unit_toml(table, "theta", "/analysis")?,
    );
    options.insert(
        "max_step_s".to_string(),
        positive_toml(table, "max_step_s", "/analysis")?,
    );
    optional_geo_toml(table, "water_unit_weight_kn_m3", &mut options, "/analysis")?;
    Ok(json!({"method": "consolidation1d", "options": options}))
}

fn linear_json(analysis: &Map<String, Json>) -> Result<(), Failure> {
    super::reject_json_keys(analysis, &["method"], "/analyses/0")
}

fn p_delta_json(analysis: &Map<String, Json>, cases: &BTreeSet<String>) -> Result<(), Failure> {
    let options = options_json(analysis, &P_DELTA)?;
    json_term(require_json(options, "gravity", "/analyses/0/options")?, cases, "/analyses/0/options/gravity")?;
    json_term(require_json(options, "lateral", "/analyses/0/options")?, cases, "/analyses/0/options/lateral")?;
    json_open_unit(require_json(options, "tolerance", "/analyses/0/options")?, "/analyses/0/options")?;
    json_int(require_json(options, "max_iterations", "/analyses/0/options")?, 1, 16_384, "/analyses/0/options")?;
    json_positive(require_json(options, "displacement_scale", "/analyses/0/options")?, "/analyses/0/options")?;
    json_positive(require_json(options, "rotation_scale", "/analyses/0/options")?, "/analyses/0/options")?;
    Ok(())
}

fn modal_json(analysis: &Map<String, Json>, cases: &BTreeSet<String>) -> Result<(), Failure> {
    let options = options_json(analysis, &MODAL)?;
    let pointer = "/analyses/0/options";
    json_int(require_json(options, "mode_count", pointer)?, 1, 16_384, pointer)?;
    json_choice(require_json(options, "mass_kind", pointer)?, &MASS_KINDS, pointer)?;
    json_vector_list(require_json(options, "directions", pointer)?, pointer)?;
    optional_mass_source_json(options, cases, pointer)?;
    Ok(())
}

fn spectrum_json(analysis: &Map<String, Json>, cases: &BTreeSet<String>) -> Result<(), Failure> {
    let options = options_json(analysis, &SPECTRUM)?;
    let pointer = "/analyses/0/options";
    json_int(require_json(options, "mode_count", pointer)?, 1, 16_384, pointer)?;
    json_choice(require_json(options, "mass_kind", pointer)?, &MASS_KINDS, pointer)?;
    json_spectrum_directions(require_json(options, "directions", pointer)?, &format!("{pointer}/directions"))?;
    json_choice(require_json(options, "modal_combination", pointer)?, &MODAL_RULES, pointer)?;
    json_choice(
        require_json(options, "directional_combination", pointer)?,
        &DIRECTIONAL_RULES,
        pointer,
    )?;
    json_number_list(
        require_json(options, "directional_factors", pointer)?,
        1,
        None,
        |value| value >= 0.0,
        pointer,
    )?;
    optional_mass_source_json(options, cases, pointer)?;
    Ok(())
}

fn newton_json(analysis: &Map<String, Json>, cases: &BTreeSet<String>) -> Result<(), Failure> {
    let options = options_json(analysis, &NEWTON)?;
    let pointer = "/analyses/0/options";
    let case_name = super::json_text_field(options, "case", pointer)?;
    if !cases.contains(&case_name) {
        return Err(invalid(pointer, "A reference does not exist."));
    }
    json_number_list(
        require_json(options, "load_factors", pointer)?,
        1,
        Some(16_384),
        |_| true,
        pointer,
    )?;
    json_positive(require_json(options, "tolerance", pointer)?, pointer)?;
    json_int(require_json(options, "max_iterations", pointer)?, 1, 16_384, pointer)?;
    json_positive(require_json(options, "force_scale", pointer)?, pointer)?;
    json_criterion(options, "displacement_tolerance", "displacement_scale", pointer)?;
    json_criterion(options, "energy_tolerance", "energy_scale", pointer)?;
    Ok(())
}

fn soil_json(analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = options_json(analysis, &SOIL)?;
    let pointer = "/analyses/0/options";
    json_soil_layers(require_json(options, "layers", pointer)?, &format!("{pointer}/layers"))?;
    json_number_list(
        require_json(options, "depths_m", pointer)?,
        1,
        Some(16_384),
        |value| value >= 0.0,
        pointer,
    )?;
    optional_geo_json(options, "groundwater_depth_m", pointer)?;
    optional_geo_json(options, "water_unit_weight_kn_m3", pointer)?;
    Ok(())
}

fn consolidation_json(analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = options_json(analysis, &CONSOLIDATION)?;
    let pointer = "/analyses/0/options";
    json_consolidation_layers(require_json(options, "layers", pointer)?, &format!("{pointer}/layers"))?;
    json_choice(require_json(options, "drainage", pointer)?, &DRAINAGE, pointer)?;
    let end = json_history(require_json(options, "history", pointer)?, &format!("{pointer}/history"))?;
    json_output_times(require_json(options, "output_times_s", pointer)?, end, pointer)?;
    json_closed_unit(require_json(options, "theta", pointer)?, pointer)?;
    json_positive(require_json(options, "max_step_s", pointer)?, pointer)?;
    optional_geo_json(options, "water_unit_weight_kn_m3", pointer)?;
    Ok(())
}

fn write_p_delta(out: &mut super::Out, analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = super::as_object(require_json(analysis, "options", "/analyses/0")?, "/analyses/0/options")?;
    emit_f64(out, "tolerance", require_json(options, "tolerance", "/analyses/0/options")?)?;
    emit_int(out, "max_iterations", require_json(options, "max_iterations", "/analyses/0/options")?)?;
    emit_f64(out, "displacement_scale", require_json(options, "displacement_scale", "/analyses/0/options")?)?;
    emit_f64(out, "rotation_scale", require_json(options, "rotation_scale", "/analyses/0/options")?)?;
    write_term(out, "[analysis.gravity]", require_json(options, "gravity", "/analyses/0/options")?, "/analyses/0/options/gravity")?;
    write_term(out, "[analysis.lateral]", require_json(options, "lateral", "/analyses/0/options")?, "/analyses/0/options/lateral")?;
    Ok(())
}

fn write_modal(out: &mut super::Out, analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = super::as_object(require_json(analysis, "options", "/analyses/0")?, "/analyses/0/options")?;
    emit_int(out, "mode_count", require_json(options, "mode_count", "/analyses/0/options")?)?;
    emit_text(out, "mass_kind", require_json(options, "mass_kind", "/analyses/0/options")?)?;
    out.entry(
        "directions",
        &format_vector_list(require_json(options, "directions", "/analyses/0/options")?, "/analyses/0/options")?,
    );
    if let Some(source) = options.get("mass_source") {
        write_mass_source(out, source)?;
    }
    Ok(())
}

fn write_spectrum(out: &mut super::Out, analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = super::as_object(require_json(analysis, "options", "/analyses/0")?, "/analyses/0/options")?;
    let pointer = "/analyses/0/options";
    emit_int(out, "mode_count", require_json(options, "mode_count", pointer)?)?;
    emit_text(out, "mass_kind", require_json(options, "mass_kind", pointer)?)?;
    emit_text(out, "modal_combination", require_json(options, "modal_combination", pointer)?)?;
    emit_text(
        out,
        "directional_combination",
        require_json(options, "directional_combination", pointer)?,
    )?;
    out.entry(
        "directional_factors",
        &format_number_list(require_json(options, "directional_factors", pointer)?)?,
    );
    if let Some(source) = options.get("mass_source") {
        write_mass_source(out, source)?;
    }
    write_spectrum_directions(out, require_json(options, "directions", pointer)?)?;
    Ok(())
}

fn write_newton(out: &mut super::Out, analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = super::as_object(require_json(analysis, "options", "/analyses/0")?, "/analyses/0/options")?;
    let pointer = "/analyses/0/options";
    emit_text(out, "case", require_json(options, "case", pointer)?)?;
    out.entry(
        "load_factors",
        &format_number_list(require_json(options, "load_factors", pointer)?)?,
    );
    emit_f64(out, "tolerance", require_json(options, "tolerance", pointer)?)?;
    emit_int(out, "max_iterations", require_json(options, "max_iterations", pointer)?)?;
    emit_f64(out, "force_scale", require_json(options, "force_scale", pointer)?)?;
    for key in [
        "displacement_scale",
        "displacement_tolerance",
        "energy_scale",
        "energy_tolerance",
    ] {
        if let Some(value) = options.get(key) {
            emit_f64(out, key, value)?;
        }
    }
    Ok(())
}

fn write_soil(out: &mut super::Out, analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = super::as_object(require_json(analysis, "options", "/analyses/0")?, "/analyses/0/options")?;
    let pointer = "/analyses/0/options";
    out.entry(
        "depths_m",
        &format_number_list(require_json(options, "depths_m", pointer)?)?,
    );
    if let Some(value) = options.get("groundwater_depth_m") {
        out.entry("groundwater_depth_m", &format_geo(value, pointer)?);
    }
    if let Some(value) = options.get("water_unit_weight_kn_m3") {
        out.entry("water_unit_weight_kn_m3", &format_geo(value, pointer)?);
    }
    let layers = require_json(options, "layers", pointer)?.as_array().ok_or_else(|| {
        invalid(pointer, "A collection has the wrong shape.")
    })?;
    for layer in layers {
        let record = super::as_object(layer, pointer)?;
        out.header("[[analysis.layers]]");
        emit_f64(out, "thickness_m", require_json(record, "thickness_m", pointer)?)?;
        if let Some(value) = record.get("gamma_kn_m3") {
            out.entry("gamma_kn_m3", &format_geo(value, pointer)?);
        }
        if let Some(value) = record.get("gamma_sat_kn_m3") {
            out.entry("gamma_sat_kn_m3", &format_geo(value, pointer)?);
        }
    }
    Ok(())
}

fn write_consolidation(out: &mut super::Out, analysis: &Map<String, Json>) -> Result<(), Failure> {
    let options = super::as_object(require_json(analysis, "options", "/analyses/0")?, "/analyses/0/options")?;
    let pointer = "/analyses/0/options";
    emit_text(out, "drainage", require_json(options, "drainage", pointer)?)?;
    out.entry(
        "output_times_s",
        &format_number_list(require_json(options, "output_times_s", pointer)?)?,
    );
    emit_f64(out, "theta", require_json(options, "theta", pointer)?)?;
    emit_f64(out, "max_step_s", require_json(options, "max_step_s", pointer)?)?;
    if let Some(value) = options.get("water_unit_weight_kn_m3") {
        out.entry("water_unit_weight_kn_m3", &format_geo(value, pointer)?);
    }
    let layers = require_json(options, "layers", pointer)?.as_array().ok_or_else(|| {
        invalid(pointer, "A collection has the wrong shape.")
    })?;
    for layer in layers {
        let record = super::as_object(layer, pointer)?;
        out.header("[[analysis.layers]]");
        out.entry(
            "thickness_m",
            &format_geo(require_json(record, "thickness_m", pointer)?, pointer)?,
        );
        for key in ["cv_m2_s", "mv_per_kpa", "k_m_s"] {
            if let Some(value) = record.get(key) {
                out.entry(key, &format_geo(value, pointer)?);
            }
        }
        emit_int(out, "elements", require_json(record, "elements", pointer)?)?;
    }
    let history = require_json(options, "history", pointer)?.as_array().ok_or_else(|| {
        invalid(pointer, "A collection has the wrong shape.")
    })?;
    for item in history {
        let record = super::as_object(item, pointer)?;
        out.header("[[analysis.history]]");
        emit_f64(out, "time_s", require_json(record, "time_s", pointer)?)?;
        if let Some(value) = record.get("delta_stress_kpa") {
            out.entry("delta_stress_kpa", &format_geo(value, pointer)?);
        }
    }
    Ok(())
}

fn with_method<'a>(keys: &'a [&'a str]) -> Vec<&'a str> {
    let mut allowed = vec!["method"];
    allowed.extend_from_slice(keys);
    allowed
}

fn invalid(pointer: &str, cause: &'static str) -> Failure {
    Failure::validation(Some(pointer.to_string()), cause)
}

fn required<'a>(table: &'a toml::Table, key: &str, pointer: &str) -> Result<&'a Toml, Failure> {
    table
        .get(key)
        .ok_or_else(|| invalid(pointer, "A required field is missing."))
}

fn nested<'a>(
    table: &'a toml::Table,
    key: &str,
    parent: &str,
    child: &str,
) -> Result<&'a toml::Table, Failure> {
    match table.get(key) {
        Some(value) => super::as_table(value, child),
        None => Err(invalid(parent, "A required field is missing.")),
    }
}

fn array_of<'a>(
    table: &'a toml::Table,
    key: &str,
    pointer: &str,
) -> Result<&'a Vec<Toml>, Failure> {
    match required(table, key, pointer)? {
        Toml::Array(items) => Ok(items),
        _ => Err(invalid(pointer, "A collection has the wrong shape.")),
    }
}

fn text_toml(table: &toml::Table, key: &str, pointer: &str) -> Result<String, Failure> {
    match table.get(key) {
        Some(value) => super::expect_text(value, pointer),
        None => Err(invalid(pointer, "A required field is missing.")),
    }
}

fn choice_toml<'a>(
    table: &'a toml::Table,
    key: &str,
    choices: &[&str],
    pointer: &str,
) -> Result<&'a str, Failure> {
    match table.get(key) {
        Some(Toml::String(value)) if choices.contains(&value.as_str()) => Ok(value.as_str()),
        Some(Toml::String(_)) => Err(invalid(pointer, "A value is not an allowed choice.")),
        Some(_) => Err(invalid(pointer, "A value has the wrong type.")),
        None => Err(invalid(pointer, "A required field is missing.")),
    }
}

fn owned_number(value: &Toml, pointer: &str) -> Result<(f64, Json), Failure> {
    let scalar = super::expect_scalar(value, pointer)?;
    let number = super::scalar_f64(scalar);
    Ok((number, super::json_scalar(scalar)?))
}

fn positive_toml(table: &toml::Table, key: &str, pointer: &str) -> Result<Json, Failure> {
    let (number, json) = owned_number(required(table, key, pointer)?, pointer)?;
    if number <= 0.0 {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(json)
}

fn open_unit_toml(table: &toml::Table, key: &str, pointer: &str) -> Result<Json, Failure> {
    let (number, json) = owned_number(required(table, key, pointer)?, pointer)?;
    if number <= 0.0 || number >= 1.0 {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(json)
}

fn closed_unit_toml(table: &toml::Table, key: &str, pointer: &str) -> Result<Json, Failure> {
    let (number, json) = owned_number(required(table, key, pointer)?, pointer)?;
    if !(0.0..=1.0).contains(&number) {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(json)
}

fn optional_positive(table: &toml::Table, key: &str, pointer: &str) -> Result<Option<Json>, Failure> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };
    let (number, json) = owned_number(value, pointer)?;
    if number <= 0.0 {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(Some(json))
}

fn int_toml(
    table: &toml::Table,
    key: &str,
    min: i64,
    max: i64,
    pointer: &str,
) -> Result<Json, Failure> {
    match required(table, key, pointer)? {
        Toml::Integer(value) if (min..=max).contains(value) => Ok(json!(*value)),
        Toml::Integer(_) => Err(invalid(pointer, "A value is outside the allowed range.")),
        _ => Err(invalid(pointer, "A value has the wrong type.")),
    }
}

fn bounded(len: usize, min: usize, max: usize, pointer: &str) -> Result<(), Failure> {
    if (min..=max).contains(&len) {
        Ok(())
    } else {
        Err(invalid(pointer, "A schema limit was exceeded."))
    }
}

fn at_least(len: usize, min: usize, pointer: &str) -> Result<(), Failure> {
    if len >= min {
        Ok(())
    } else {
        Err(invalid(pointer, "A schema limit was exceeded."))
    }
}

fn term_toml(table: &toml::Table, cases: &BTreeSet<String>, pointer: &str) -> Result<Json, Failure> {
    super::reject_toml_keys(table, &["case", "factor"], pointer)?;
    let case_name = text_toml(table, "case", pointer)?;
    if !cases.contains(&case_name) {
        return Err(invalid(pointer, "A reference does not exist."));
    }
    let factor = match table.get("factor") {
        Some(value) => super::json_scalar(super::expect_scalar(value, pointer)?)?,
        None => return Err(invalid(pointer, "A required field is missing.")),
    };
    Ok(json!({"case": case_name, "factor": factor}))
}

fn vector_list_toml(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    let Toml::Array(items) = value else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    at_least(items.len(), 1, pointer)?;
    let mut directions = Vec::with_capacity(items.len());
    for item in items {
        directions.push(super::expect_vec3(item, pointer)?);
    }
    Ok(Json::Array(directions))
}

fn number_list_toml(
    value: &Toml,
    min: usize,
    max: Option<usize>,
    accept: impl Fn(f64) -> bool,
    pointer: &str,
) -> Result<Json, Failure> {
    let Toml::Array(items) = value else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    match max {
        Some(max) => bounded(items.len(), min, max, pointer)?,
        None => at_least(items.len(), min, pointer)?,
    }
    let mut numbers = Vec::with_capacity(items.len());
    for item in items {
        let (number, json) = owned_number(item, pointer)?;
        if !accept(number) {
            return Err(invalid(pointer, "A value is outside the allowed range."));
        }
        numbers.push(json);
    }
    Ok(Json::Array(numbers))
}

fn optional_mass_source(
    table: &toml::Table,
    cases: &BTreeSet<String>,
) -> Result<Option<Json>, Failure> {
    match table.get("mass_source") {
        None => Ok(None),
        Some(value) => Ok(Some(mass_source_toml(
            super::as_table(value, "/analysis/mass_source")?,
            cases,
        )?)),
    }
}

fn mass_source_toml(table: &toml::Table, cases: &BTreeSet<String>) -> Result<Json, Failure> {
    let pointer = "/analysis/mass_source";
    super::reject_toml_keys(
        table,
        &["method", "case", "factor", "direction", "gravity"],
        pointer,
    )?;
    let method = choice_toml(table, "method", &MASS_METHODS, pointer)?;
    let case_name = text_toml(table, "case", pointer)?;
    if !cases.contains(&case_name) {
        return Err(invalid(pointer, "A reference does not exist."));
    }
    let factor = match table.get("factor") {
        Some(value) => super::nonnegative(value, pointer)?,
        None => return Err(invalid(pointer, "A required field is missing.")),
    };
    let direction = match table.get("direction") {
        Some(value) => super::expect_vec3(value, pointer)?,
        None => return Err(invalid(pointer, "A required field is missing.")),
    };
    Ok(json!({
        "method": method,
        "case": case_name,
        "factor": factor,
        "direction": direction,
        "gravity": positive_toml(table, "gravity", pointer)?,
    }))
}

fn spectrum_directions_toml(items: &[Toml], pointer: &str) -> Result<Json, Failure> {
    at_least(items.len(), 1, pointer)?;
    let mut directions = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_table(item, &child)?;
        super::reject_toml_keys(table, &["direction", "spectra"], &child)?;
        let direction = match table.get("direction") {
            Some(value) => super::expect_vec3(value, &child)?,
            None => return Err(invalid(&child, "A required field is missing.")),
        };
        let spectra = match table.get("spectra") {
            Some(Toml::Array(items)) => spectra_toml(items, &format!("{child}/spectra"))?,
            Some(_) => return Err(invalid(&child, "A collection has the wrong shape.")),
            None => return Err(invalid(&child, "A required field is missing.")),
        };
        directions.push(json!({"direction": direction, "spectra": spectra}));
    }
    Ok(Json::Array(directions))
}

fn spectra_toml(items: &[Toml], pointer: &str) -> Result<Json, Failure> {
    at_least(items.len(), 1, pointer)?;
    let mut spectra = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        spectra.push(spectrum_table_toml(super::as_table(item, &child)?, &child)?);
    }
    Ok(Json::Array(spectra))
}

fn spectrum_table_toml(table: &toml::Table, pointer: &str) -> Result<Json, Failure> {
    super::reject_toml_keys(
        table,
        &["pairs", "units", "damping", "interpolation", "extrapolation"],
        pointer,
    )?;
    let pairs = pairs_toml(required(table, "pairs", pointer)?, pointer)?;
    let units = choice_toml(table, "units", &SPECTRUM_UNITS, pointer)?;
    let (damping, damping_json) = owned_number(required(table, "damping", pointer)?, pointer)?;
    if !(0.0..1.0).contains(&damping) {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(json!({
        "pairs": pairs,
        "units": units,
        "damping": damping_json,
        "interpolation": choice_toml(table, "interpolation", &INTERPOLATION, pointer)?,
        "extrapolation": choice_toml(table, "extrapolation", &EXTRAPOLATION, pointer)?,
    }))
}

fn pairs_toml(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    let Toml::Array(items) = value else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    at_least(items.len(), 2, pointer)?;
    let mut pairs = Vec::with_capacity(items.len());
    for item in items {
        let Toml::Array(pair) = item else {
            return Err(invalid(pointer, "A value has the wrong type."));
        };
        if pair.len() != 2 {
            return Err(invalid(pointer, "A value has the wrong type."));
        }
        pairs.push(json!([
            super::json_scalar(super::expect_scalar(&pair[0], pointer)?)?,
            super::json_scalar(super::expect_scalar(&pair[1], pointer)?)?,
        ]));
    }
    Ok(Json::Array(pairs))
}

fn criterion_toml(
    table: &toml::Table,
    tolerance: &str,
    scale: &str,
    options: &mut Map<String, Json>,
) -> Result<(), Failure> {
    let tolerance_value = optional_positive(table, tolerance, "/analysis")?;
    let scale_value = optional_positive(table, scale, "/analysis")?;
    if tolerance_value.is_some() && scale_value.is_none() {
        return Err(invalid("/analysis", "A required field is missing."));
    }
    if let Some(value) = scale_value {
        options.insert(scale.to_string(), value);
    }
    if let Some(value) = tolerance_value {
        options.insert(tolerance.to_string(), value);
    }
    Ok(())
}

fn geo_toml(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    let table = super::as_table(value, pointer)?;
    super::reject_toml_keys(table, &["value", "provenance"], pointer)?;
    let number = match table.get("value") {
        Some(value) => super::json_scalar(super::expect_scalar(value, pointer)?)?,
        None => return Err(invalid(pointer, "A required field is missing.")),
    };
    let provenance = choice_toml(table, "provenance", &PROVENANCE, pointer)?;
    Ok(json!({"value": number, "provenance": provenance}))
}

fn optional_geo_toml(
    table: &toml::Table,
    key: &str,
    options: &mut Map<String, Json>,
    pointer: &str,
) -> Result<(), Failure> {
    if let Some(value) = table.get(key) {
        options.insert(key.to_string(), geo_toml(value, pointer)?);
    }
    Ok(())
}

fn soil_layers_toml(items: &[Toml], pointer: &str) -> Result<Json, Failure> {
    bounded(items.len(), 1, 16_384, pointer)?;
    let mut layers = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_table(item, &child)?;
        super::reject_toml_keys(
            table,
            &["thickness_m", "gamma_kn_m3", "gamma_sat_kn_m3"],
            &child,
        )?;
        let mut layer = Map::new();
        layer.insert(
            "thickness_m".to_string(),
            positive_toml(table, "thickness_m", &child)?,
        );
        optional_geo_toml(table, "gamma_kn_m3", &mut layer, &child)?;
        optional_geo_toml(table, "gamma_sat_kn_m3", &mut layer, &child)?;
        layers.push(Json::Object(layer));
    }
    Ok(Json::Array(layers))
}

fn consolidation_layers_toml(items: &[Toml], pointer: &str) -> Result<Json, Failure> {
    bounded(items.len(), 1, 2048, pointer)?;
    let mut layers = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_table(item, &child)?;
        super::reject_toml_keys(
            table,
            &["thickness_m", "cv_m2_s", "mv_per_kpa", "k_m_s", "elements"],
            &child,
        )?;
        let mut layer = Map::new();
        layer.insert(
            "thickness_m".to_string(),
            geo_toml(required(table, "thickness_m", &child)?, &child)?,
        );
        for key in ["cv_m2_s", "mv_per_kpa", "k_m_s"] {
            optional_geo_toml(table, key, &mut layer, &child)?;
        }
        layer.insert(
            "elements".to_string(),
            int_toml(table, "elements", 1, 2048, &child)?,
        );
        layers.push(Json::Object(layer));
    }
    Ok(Json::Array(layers))
}

fn history_records(items: &[Toml], pointer: &str) -> Result<(Vec<Json>, Vec<f64>), Failure> {
    bounded(items.len(), 2, 257, pointer)?;
    let mut records = Vec::with_capacity(items.len());
    let mut times = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_table(item, &child)?;
        super::reject_toml_keys(table, &["time_s", "delta_stress_kpa"], &child)?;
        let (time, time_json) = owned_number(required(table, "time_s", &child)?, &child)?;
        if time < 0.0 {
            return Err(invalid(&child, "A value is outside the allowed range."));
        }
        let mut record = Map::new();
        record.insert("time_s".to_string(), time_json);
        optional_geo_toml(table, "delta_stress_kpa", &mut record, &child)?;
        times.push(time);
        records.push(Json::Object(record));
    }
    check_history(&times, pointer)?;
    Ok((records, times))
}

fn history_toml(items: &[Toml], pointer: &str) -> Result<Json, Failure> {
    Ok(Json::Array(history_records(items, pointer)?.0))
}

fn history_end_toml(items: &[Toml], pointer: &str) -> Result<f64, Failure> {
    let times = history_records(items, pointer)?.1;
    Ok(times[times.len() - 1])
}

fn output_times_toml(value: &Toml, end: f64, pointer: &str) -> Result<Json, Failure> {
    let Toml::Array(items) = value else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    bounded(items.len(), 1, 16_384, pointer)?;
    let mut times = Vec::with_capacity(items.len());
    let mut numbers = Vec::with_capacity(items.len());
    for item in items {
        let (time, json) = owned_number(item, pointer)?;
        times.push(time);
        numbers.push(json);
    }
    check_outputs(&times, end, pointer)?;
    Ok(Json::Array(numbers))
}

fn check_history(times: &[f64], pointer: &str) -> Result<(), Failure> {
    if times.first().copied() != Some(0.0) {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    let mut previous = -1.0;
    for time in times {
        if *time <= previous {
            return Err(invalid(pointer, "A value is outside the allowed range."));
        }
        previous = *time;
    }
    Ok(())
}

fn check_outputs(times: &[f64], end: f64, pointer: &str) -> Result<(), Failure> {
    let mut previous = -1.0;
    for time in times {
        if *time < 0.0 || *time > end || *time <= previous {
            return Err(invalid(pointer, "A value is outside the allowed range."));
        }
        previous = *time;
    }
    Ok(())
}

fn options_json<'a>(
    analysis: &'a Map<String, Json>,
    keys: &[&str],
) -> Result<&'a Map<String, Json>, Failure> {
    super::reject_json_keys(analysis, &["method", "options"], "/analyses/0")?;
    let options = super::as_object(
        require_json(analysis, "options", "/analyses/0")?,
        "/analyses/0/options",
    )?;
    super::reject_json_keys(options, keys, "/analyses/0/options")?;
    Ok(options)
}

fn require_json<'a>(
    object: &'a Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<&'a Json, Failure> {
    object
        .get(key)
        .ok_or_else(|| invalid(pointer, "A required field is missing."))
}

fn json_choice(value: &Json, choices: &[&str], pointer: &str) -> Result<(), Failure> {
    match value.as_str() {
        Some(text) if choices.contains(&text) => Ok(()),
        Some(_) => Err(invalid(pointer, "A value is not an allowed choice.")),
        None => Err(invalid(pointer, "A value has the wrong type.")),
    }
}

fn json_positive(value: &Json, pointer: &str) -> Result<(), Failure> {
    let number = super::json_f64(value)?;
    if number <= 0.0 {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(())
}

fn json_open_unit(value: &Json, pointer: &str) -> Result<(), Failure> {
    let number = super::json_f64(value)?;
    if number <= 0.0 || number >= 1.0 {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(())
}

fn json_closed_unit(value: &Json, pointer: &str) -> Result<(), Failure> {
    let number = super::json_f64(value)?;
    if !(0.0..=1.0).contains(&number) {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(())
}

fn json_int(value: &Json, min: i64, max: i64, pointer: &str) -> Result<(), Failure> {
    let Some(number) = value.as_number() else {
        return Err(invalid(pointer, "A value has the wrong type."));
    };
    if number.is_f64() {
        return Err(invalid(pointer, "A value has the wrong type."));
    }
    let Some(value) = number.as_i64() else {
        return Err(invalid(pointer, "An integer is outside the safe range."));
    };
    if !(min..=max).contains(&value) {
        return Err(invalid(pointer, "A value is outside the allowed range."));
    }
    Ok(())
}

fn json_term(value: &Json, cases: &BTreeSet<String>, pointer: &str) -> Result<(), Failure> {
    let term = super::as_object(value, pointer)?;
    super::reject_json_keys(term, &["case", "factor"], pointer)?;
    let case_name = super::json_text_field(term, "case", pointer)?;
    if !cases.contains(&case_name) {
        return Err(invalid(pointer, "A reference does not exist."));
    }
    let _factor = super::json_f64(require_json(term, "factor", pointer)?)?;
    Ok(())
}

fn json_vector_list(value: &Json, pointer: &str) -> Result<(), Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    at_least(items.len(), 1, pointer)?;
    for item in items {
        super::json_vec3(item, pointer)?;
    }
    Ok(())
}

fn json_number_list(
    value: &Json,
    min: usize,
    max: Option<usize>,
    accept: impl Fn(f64) -> bool,
    pointer: &str,
) -> Result<(), Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    match max {
        Some(max) => bounded(items.len(), min, max, pointer)?,
        None => at_least(items.len(), min, pointer)?,
    }
    for item in items {
        let number = super::json_f64(item)?;
        if !accept(number) {
            return Err(invalid(pointer, "A value is outside the allowed range."));
        }
    }
    Ok(())
}

fn optional_mass_source_json(
    options: &Map<String, Json>,
    cases: &BTreeSet<String>,
    pointer: &str,
) -> Result<(), Failure> {
    let Some(value) = options.get("mass_source") else {
        return Ok(());
    };
    let source = super::as_object(value, &format!("{pointer}/mass_source"))?;
    let child = format!("{pointer}/mass_source");
    super::reject_json_keys(
        source,
        &["method", "case", "factor", "direction", "gravity"],
        &child,
    )?;
    json_choice(require_json(source, "method", &child)?, &MASS_METHODS, &child)?;
    let case_name = super::json_text_field(source, "case", &child)?;
    if !cases.contains(&case_name) {
        return Err(invalid(&child, "A reference does not exist."));
    }
    let factor = super::json_f64(require_json(source, "factor", &child)?)?;
    if factor < 0.0 {
        return Err(invalid(&child, "A value is outside the allowed range."));
    }
    super::json_vec3(require_json(source, "direction", &child)?, &child)?;
    json_positive(require_json(source, "gravity", &child)?, &child)?;
    Ok(())
}

fn json_spectrum_directions(value: &Json, pointer: &str) -> Result<(), Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    at_least(items.len(), 1, pointer)?;
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_object(item, &child)?;
        super::reject_json_keys(table, &["direction", "spectra"], &child)?;
        super::json_vec3(require_json(table, "direction", &child)?, &child)?;
        let spectra = match table.get("spectra") {
            Some(Json::Array(items)) => items,
            Some(_) => return Err(invalid(&child, "A collection has the wrong shape.")),
            None => return Err(invalid(&child, "A required field is missing.")),
        };
        json_spectra(spectra, &format!("{child}/spectra"))?;
    }
    Ok(())
}

fn json_spectra(items: &[Json], pointer: &str) -> Result<(), Failure> {
    at_least(items.len(), 1, pointer)?;
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_object(item, &child)?;
        super::reject_json_keys(
            table,
            &["pairs", "units", "damping", "interpolation", "extrapolation"],
            &child,
        )?;
        json_pairs(require_json(table, "pairs", &child)?, &child)?;
        json_choice(require_json(table, "units", &child)?, &SPECTRUM_UNITS, &child)?;
        let damping = super::json_f64(require_json(table, "damping", &child)?)?;
        if !(0.0..1.0).contains(&damping) {
            return Err(invalid(&child, "A value is outside the allowed range."));
        }
        json_choice(require_json(table, "interpolation", &child)?, &INTERPOLATION, &child)?;
        json_choice(require_json(table, "extrapolation", &child)?, &EXTRAPOLATION, &child)?;
    }
    Ok(())
}

fn json_pairs(value: &Json, pointer: &str) -> Result<(), Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    at_least(items.len(), 2, pointer)?;
    for item in items {
        super::json_pair(item, pointer)?;
    }
    Ok(())
}

fn json_criterion(
    options: &Map<String, Json>,
    tolerance: &str,
    scale: &str,
    pointer: &str,
) -> Result<(), Failure> {
    if let Some(value) = options.get(tolerance) {
        json_positive(value, pointer)?;
        if !options.contains_key(scale) {
            return Err(invalid(pointer, "A required field is missing."));
        }
    }
    if let Some(value) = options.get(scale) {
        json_positive(value, pointer)?;
    }
    Ok(())
}

fn json_geo(value: &Json, pointer: &str) -> Result<(), Failure> {
    let table = super::as_object(value, pointer)?;
    super::reject_json_keys(table, &["value", "provenance"], pointer)?;
    super::json_f64(require_json(table, "value", pointer)?)?;
    json_choice(require_json(table, "provenance", pointer)?, &PROVENANCE, pointer)?;
    Ok(())
}

fn optional_geo_json(options: &Map<String, Json>, key: &str, pointer: &str) -> Result<(), Failure> {
    if let Some(value) = options.get(key) {
        json_geo(value, pointer)?;
    }
    Ok(())
}

fn json_soil_layers(value: &Json, pointer: &str) -> Result<(), Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    bounded(items.len(), 1, 16_384, pointer)?;
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_object(item, &child)?;
        super::reject_json_keys(
            table,
            &["thickness_m", "gamma_kn_m3", "gamma_sat_kn_m3"],
            &child,
        )?;
        json_positive(require_json(table, "thickness_m", &child)?, &child)?;
        optional_geo_json(table, "gamma_kn_m3", &child)?;
        optional_geo_json(table, "gamma_sat_kn_m3", &child)?;
    }
    Ok(())
}

fn json_consolidation_layers(value: &Json, pointer: &str) -> Result<(), Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    bounded(items.len(), 1, 2048, pointer)?;
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_object(item, &child)?;
        super::reject_json_keys(
            table,
            &["thickness_m", "cv_m2_s", "mv_per_kpa", "k_m_s", "elements"],
            &child,
        )?;
        json_geo(require_json(table, "thickness_m", &child)?, &child)?;
        for key in ["cv_m2_s", "mv_per_kpa", "k_m_s"] {
            optional_geo_json(table, key, &child)?;
        }
        json_int(require_json(table, "elements", &child)?, 1, 2048, &child)?;
    }
    Ok(())
}

fn json_history(value: &Json, pointer: &str) -> Result<f64, Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    bounded(items.len(), 2, 257, pointer)?;
    let mut times = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let child = format!("{pointer}/{index}");
        let table = super::as_object(item, &child)?;
        super::reject_json_keys(table, &["time_s", "delta_stress_kpa"], &child)?;
        let time = super::json_f64(require_json(table, "time_s", &child)?)?;
        if time < 0.0 {
            return Err(invalid(&child, "A value is outside the allowed range."));
        }
        optional_geo_json(table, "delta_stress_kpa", &child)?;
        times.push(time);
    }
    check_history(&times, pointer)?;
    Ok(times[times.len() - 1])
}

fn json_output_times(value: &Json, end: f64, pointer: &str) -> Result<(), Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    bounded(items.len(), 1, 16_384, pointer)?;
    let mut times = Vec::with_capacity(items.len());
    for item in items {
        times.push(super::json_f64(item)?);
    }
    check_outputs(&times, end, pointer)
}

fn emit_f64(out: &mut super::Out, key: &str, value: &Json) -> Result<(), Failure> {
    out.entry(key, &super::format_f64(super::json_f64(value)?)?);
    Ok(())
}

fn emit_int(out: &mut super::Out, key: &str, value: &Json) -> Result<(), Failure> {
    let Some(number) = value.as_i64() else {
        return Err(invalid("/analyses/0", "A value has the wrong type."));
    };
    out.entry(key, &number.to_string());
    Ok(())
}

fn emit_text(out: &mut super::Out, key: &str, value: &Json) -> Result<(), Failure> {
    let Some(text) = value.as_str() else {
        return Err(invalid("/analyses/0", "A value has the wrong type."));
    };
    out.entry(key, &super::quoted(text));
    Ok(())
}

fn format_number_list(value: &Json) -> Result<String, Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid("/analyses/0", "A collection has the wrong shape."));
    };
    let mut parts = Vec::with_capacity(items.len());
    for item in items {
        parts.push(super::format_f64(super::json_f64(item)?)?);
    }
    Ok(format!("[{}]", parts.join(", ")))
}

fn format_vector_list(value: &Json, pointer: &str) -> Result<String, Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid(pointer, "A collection has the wrong shape."));
    };
    let mut parts = Vec::with_capacity(items.len());
    for item in items {
        parts.push(super::format_vec3_text(super::json_vec3(item, pointer)?)?);
    }
    Ok(format!("[{}]", parts.join(", ")))
}

fn format_geo(value: &Json, pointer: &str) -> Result<String, Failure> {
    let table = super::as_object(value, pointer)?;
    let number = super::format_f64(super::json_f64(require_json(table, "value", pointer)?)?)?;
    let provenance = require_json(table, "provenance", pointer)?
        .as_str()
        .ok_or_else(|| invalid(pointer, "A value has the wrong type."))?;
    Ok(format!(
        "{{ value = {number}, provenance = {} }}",
        super::quoted(provenance)
    ))
}

fn write_term(
    out: &mut super::Out,
    header: &str,
    value: &Json,
    pointer: &str,
) -> Result<(), Failure> {
    let term = super::as_object(value, pointer)?;
    let case_name = term
        .get("case")
        .and_then(Json::as_str)
        .ok_or_else(|| invalid(pointer, "A required field is missing."))?;
    out.header(header);
    out.entry("case", &super::quoted(case_name));
    emit_f64(out, "factor", require_json(term, "factor", pointer)?)?;
    Ok(())
}

fn write_mass_source(out: &mut super::Out, value: &Json) -> Result<(), Failure> {
    let source = super::as_object(value, "/analyses/0/options/mass_source")?;
    out.header("[analysis.mass_source]");
    emit_text(out, "method", require_json(source, "method", "/analyses/0/options/mass_source")?)?;
    emit_text(out, "case", require_json(source, "case", "/analyses/0/options/mass_source")?)?;
    emit_f64(out, "factor", require_json(source, "factor", "/analyses/0/options/mass_source")?)?;
    out.entry(
        "direction",
        &super::format_vec3_text(super::json_vec3(
            require_json(source, "direction", "/analyses/0/options/mass_source")?,
            "/analyses/0/options/mass_source",
        )?)?,
    );
    emit_f64(out, "gravity", require_json(source, "gravity", "/analyses/0/options/mass_source")?)?;
    Ok(())
}

fn write_spectrum_directions(out: &mut super::Out, value: &Json) -> Result<(), Failure> {
    let Some(directions) = value.as_array() else {
        return Err(invalid("/analyses/0/options", "A collection has the wrong shape."));
    };
    for direction in directions {
        let table = super::as_object(direction, "/analyses/0/options")?;
        out.header("[[analysis.directions]]");
        out.entry(
            "direction",
            &super::format_vec3_text(super::json_vec3(
                require_json(table, "direction", "/analyses/0/options")?,
                "/analyses/0/options",
            )?)?,
        );
        let Some(spectra) = table.get("spectra").and_then(Json::as_array) else {
            return Err(invalid("/analyses/0/options", "A collection has the wrong shape."));
        };
        for spectrum in spectra {
            let record = super::as_object(spectrum, "/analyses/0/options")?;
            out.header("[[analysis.directions.spectra]]");
            out.entry(
                "pairs",
                &format_pairs(require_json(record, "pairs", "/analyses/0/options")?)?,
            );
            emit_text(out, "units", require_json(record, "units", "/analyses/0/options")?)?;
            emit_f64(out, "damping", require_json(record, "damping", "/analyses/0/options")?)?;
            emit_text(
                out,
                "interpolation",
                require_json(record, "interpolation", "/analyses/0/options")?,
            )?;
            emit_text(
                out,
                "extrapolation",
                require_json(record, "extrapolation", "/analyses/0/options")?,
            )?;
        }
    }
    Ok(())
}

fn format_pairs(value: &Json) -> Result<String, Failure> {
    let Some(items) = value.as_array() else {
        return Err(invalid("/analyses/0/options", "A collection has the wrong shape."));
    };
    let mut parts = Vec::with_capacity(items.len());
    for item in items {
        let pair = super::json_pair(item, "/analyses/0/options")?;
        parts.push(format!(
            "[{}, {}]",
            super::format_f64(pair[0])?,
            super::format_f64(pair[1])?
        ));
    }
    Ok(format!("[{}]", parts.join(", ")))
}
