// SPDX-License-Identifier: AGPL-3.0-only

//! Map the structural subset of mamut-toml/1 to engine-input/1 and back.
//!
//! U5 stays open: textual aliases of node and member ids are rejected.
//! U9 stays open: the emitter is deterministic, not a unique textual form,
//! and it does not extract `[defaults]`.
//! U10 stays open: `type` is rejected instead of expanded into a DOF list.
//! U11 stays open: nesting uses a provisional fence, and errors omit line
//! and column.

use std::collections::{BTreeMap, BTreeSet};

use engine_api::{ApiError, MAX_INPUT_BYTES};
use serde_json::{json, Number, Value as Json};
use toml::Value as Toml;

use super::order;
use super::Failure;

const MAX_SAFE: i64 = (1_i64 << 53) - 1;
const MAX_ID_BYTES: usize = 256;
const MAX_NODES: usize = 256;
const MAX_MEMBERS: usize = 1024;
const NEST_LIMIT: usize = 64;
const DOFS: [&str; 6] = ["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"];
const KINDS: [&str; 4] = [
    "elastic",
    "steel.bilinear_kinematic",
    "multilinear_elastic",
    "elastic.cubic",
];
const MODES: [&str; 3] = ["both", "compression_only", "tension_only"];
const ROOT_KEYS: [&str; 10] = [
    "format", "units", "defaults", "nodes", "sections", "laws", "members", "supports", "analysis",
    "options",
];
const ARRAY_DEFERRED: [&str; 5] = ["masses", "springs", "prescribed", "diaphragms", "cases"];

/// Convert mamut-toml/1 text into engine-input/1 JSON, including a trailing newline.
pub(crate) fn toml_to_json(text: &str) -> Result<String, Failure> {
    ensure_len(text.len())?;
    let table = parse_table(text)?;
    let value = root_from_toml(&table)?;
    Ok(format!("{value}\n"))
}

/// Export an engine-input/1 JSON subset to mamut-toml/1, including a trailing newline.
pub(crate) fn json_to_toml(text: &str) -> Result<String, Failure> {
    ensure_len(text.len())?;
    let value = parse_json(text)?;
    reject_null(&value)?;
    Ok(export_root(&value)?)
}

fn ensure_len(len: usize) -> Result<(), Failure> {
    if len > MAX_INPUT_BYTES {
        Err(Failure::document("The document exceeds the size limit."))
    } else {
        Ok(())
    }
}

fn parse_table(text: &str) -> Result<toml::Table, Failure> {
    match toml::from_str::<toml::Table>(text) {
        Ok(table) => Ok(table),
        Err(error) => {
            // Parser text can quote source tokens. Keep only the fact of failure.
            let _message = error.message();
            let _span = error.span();
            Err(Failure::document("The TOML document is invalid."))
        }
    }
}

fn parse_json(text: &str) -> Result<Json, Failure> {
    let canonical = match engine_api::canonical_json(text) {
        Ok(canonical) => canonical,
        Err(error) => return Err(classify_json(error)),
    };
    serde_json::from_str(&canonical).map_err(|_| Failure::document("The JSON document is invalid."))
}

fn classify_json(error: ApiError) -> Failure {
    match error {
        ApiError::NonFiniteNumber(_) => Failure::nonfinite(),
        ApiError::InvalidInput(message) if message.contains("Duplicate key") => {
            Failure::validation(None, "An identifier is duplicated.")
        }
        ApiError::InvalidInput(message) if message.contains("safe range") => {
            Failure::validation(None, "An integer is outside the safe range.")
        }
        ApiError::InvalidInput(message) if message.contains("Limit ") => {
            Failure::validation(None, "A schema limit was exceeded.")
        }
        _ => Failure::document("The JSON document is invalid."),
    }
}

fn reject_null(value: &Json) -> Result<(), Failure> {
    match value {
        Json::Null => Err(Failure::validation(
            None,
            "The document contains a null value.",
        )),
        Json::Array(items) => items.iter().try_for_each(reject_null),
        Json::Object(map) => map.values().try_for_each(reject_null),
        Json::Number(number) => {
            if number.as_f64().is_some_and(|value| !value.is_finite()) {
                Err(Failure::nonfinite())
            } else {
                Ok(())
            }
        }
        Json::Bool(_) | Json::String(_) => Ok(()),
    }
}

/// Provisional fence. U11 does not decide whether the root counts as a level.
/// This count treats the root table as one level and each nested table or
/// array as one more level. It is not the closed convention.
fn nesting_table(table: &toml::Table) -> usize {
    1 + table.values().map(nesting_value).max().unwrap_or(0)
}

fn nesting_value(value: &Toml) -> usize {
    match value {
        Toml::Array(items) => 1 + items.iter().map(nesting_value).max().unwrap_or(0),
        Toml::Table(table) => nesting_table(table),
        _ => 1,
    }
}

fn screen(value: &Toml) -> Result<(), Failure> {
    match value {
        Toml::Datetime(_) => Err(Failure::validation(
            None,
            "The document contains a datetime.",
        )),
        Toml::Float(number) if !number.is_finite() => Err(Failure::nonfinite()),
        Toml::Integer(number) if !(-MAX_SAFE..=MAX_SAFE).contains(number) => Err(
            Failure::validation(None, "An integer is outside the safe range."),
        ),
        Toml::Array(items) => items.iter().try_for_each(screen),
        Toml::Table(table) => table.values().try_for_each(screen),
        _ => Ok(()),
    }
}

fn root_from_toml(table: &toml::Table) -> Result<Json, Failure> {
    if nesting_table(table) > NEST_LIMIT {
        return Err(Failure::validation(
            Some(String::new()),
            "The nesting exceeds the depth limit.",
        ));
    }
    screen_root(table)?;
    check_root_keys(table)?;
    require_format(table)?;
    require_units(table)?;
    let experimental = allow_experimental(table)?;
    require_linear_static(table)?;
    let defaults = parse_defaults(table)?;
    let nodes = parse_nodes(optional_array(table, "nodes")?, &defaults)?;
    let sections = parse_sections(optional_table(table, "sections")?)?;
    let laws = parse_laws(optional_table(table, "laws")?)?;
    let members = parse_members(optional_array(table, "members")?, &nodes, &sections, &laws)?;
    let supports = parse_supports(optional_array(table, "supports")?, &nodes)?;
    Ok(assemble(
        nodes,
        members,
        sections,
        laws,
        supports,
        experimental,
    ))
}

fn screen_root(table: &toml::Table) -> Result<(), Failure> {
    table.values().try_for_each(screen)
}

fn check_root_keys(table: &toml::Table) -> Result<(), Failure> {
    for (key, value) in table {
        if ARRAY_DEFERRED.contains(&key.as_str()) {
            require_empty_toml_array(value, key)?;
        } else if key == "combinations" {
            require_empty_toml_table(value, key)?;
        } else if !ROOT_KEYS.contains(&key.as_str()) {
            return Err(Failure::validation(
                Some(String::new()),
                "The document contains an unknown key.",
            ));
        }
    }
    Ok(())
}

fn require_empty_toml_array(value: &Toml, key: &str) -> Result<(), Failure> {
    match value {
        Toml::Array(items) if items.is_empty() => Ok(()),
        Toml::Array(_) => Err(Failure::validation(
            Some(format!("/{key}")),
            "This collection is outside the implemented subset.",
        )),
        _ => Err(Failure::validation(
            Some(format!("/{key}")),
            "A collection has the wrong shape.",
        )),
    }
}

fn require_empty_toml_table(value: &Toml, key: &str) -> Result<(), Failure> {
    match value {
        Toml::Table(table) if table.is_empty() => Ok(()),
        Toml::Table(_) => Err(Failure::validation(
            Some(format!("/{key}")),
            "This collection is outside the implemented subset.",
        )),
        _ => Err(Failure::validation(
            Some(format!("/{key}")),
            "A collection has the wrong shape.",
        )),
    }
}

fn require_format(table: &toml::Table) -> Result<(), Failure> {
    match table.get("format") {
        Some(Toml::String(value)) if value == "mamut-toml/1" => Ok(()),
        Some(Toml::String(_)) => Err(Failure::unsupported("The format version is not supported.")),
        Some(_) => Err(Failure::validation(
            Some(String::new()),
            "The format field has the wrong type.",
        )),
        None => Err(Failure::validation(
            Some(String::new()),
            "The format field is missing.",
        )),
    }
}

fn require_units(table: &toml::Table) -> Result<(), Failure> {
    match table.get("units") {
        Some(Toml::String(value)) if value == "SI" => Ok(()),
        Some(Toml::String(_)) => Err(Failure::validation(
            Some(String::new()),
            "The units field is not SI.",
        )),
        Some(_) => Err(Failure::validation(
            Some(String::new()),
            "The units field has the wrong type.",
        )),
        None => Err(Failure::validation(
            Some(String::new()),
            "The units field is missing.",
        )),
    }
}

fn allow_experimental(table: &toml::Table) -> Result<bool, Failure> {
    let Some(value) = table.get("options") else {
        return Ok(false);
    };
    let options = as_table(value, "/options")?;
    reject_toml_keys(options, &["allow_experimental"], "/options")?;
    match options.get("allow_experimental") {
        None => Ok(false),
        Some(Toml::Boolean(value)) => Ok(*value),
        Some(_) => Err(Failure::validation(
            Some("/options".to_string()),
            "A value has the wrong type.",
        )),
    }
}

fn require_linear_static(table: &toml::Table) -> Result<(), Failure> {
    let Some(value) = table.get("analysis") else {
        return Err(Failure::validation(
            Some(String::new()),
            "The analysis field is missing.",
        ));
    };
    let analysis = as_table(value, "/analysis")?;
    reject_toml_keys(analysis, &["method"], "/analysis")?;
    match analysis.get("method") {
        Some(Toml::String(value)) if value == "linearStatic" => Ok(()),
        Some(Toml::String(_)) => Err(Failure::validation(
            Some("/analysis".to_string()),
            "This analysis method is outside the implemented subset.",
        )),
        Some(_) => Err(Failure::validation(
            Some("/analysis".to_string()),
            "A value has the wrong type.",
        )),
        None => Err(Failure::validation(
            Some("/analysis".to_string()),
            "A required field is missing.",
        )),
    }
}

struct Defaults {
    dim: Option<String>,
    dofs: Option<Vec<String>>,
}

fn parse_defaults(table: &toml::Table) -> Result<Defaults, Failure> {
    let Some(value) = table.get("defaults") else {
        return Ok(Defaults {
            dim: None,
            dofs: None,
        });
    };
    let defaults = as_table(value, "/defaults")?;
    reject_toml_keys(defaults, &["dim", "dofs"], "/defaults")?;
    let dim = match defaults.get("dim") {
        None => None,
        Some(value) => Some(expect_dim(value, "/defaults")?),
    };
    let dofs = match defaults.get("dofs") {
        None => None,
        Some(value) => Some(expect_dofs(value, "/defaults")?),
    };
    Ok(Defaults { dim, dofs })
}

struct NodeRec {
    id: u64,
    dim: String,
    body: Json,
}

fn parse_nodes(items: Option<&Vec<Toml>>, defaults: &Defaults) -> Result<Vec<NodeRec>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    if items.len() > MAX_NODES {
        return Err(Failure::validation(
            Some("/nodes".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut nodes = Vec::with_capacity(items.len());
    let mut seen = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/nodes/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(table, &["id", "x", "y", "z", "dim", "dofs"], &pointer)?;
        let id = required_u32(table, "id", &pointer)?;
        if seen.insert(id, index).is_some() {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let x = json_scalar(required_scalar(table, "x", &pointer)?)?;
        let y = json_scalar(required_scalar(table, "y", &pointer)?)?;
        let z_scalar = match table.get("z") {
            Some(value) => expect_scalar(value, &pointer)?,
            None => Scalar::Int(0),
        };
        let dim = match table.get("dim") {
            Some(value) => expect_dim(value, &pointer)?,
            None => defaults.dim.clone().ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
        };
        if dim == "D2" && scalar_f64(z_scalar) != 0.0 {
            return Err(Failure::validation(
                Some(pointer),
                "A D2 node requires z = 0.",
            ));
        }
        let dofs = match table.get("dofs") {
            Some(value) => expect_dofs(value, &pointer)?,
            None => defaults.dofs.clone().ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
        };
        let z = json_scalar(z_scalar)?;
        let body = json!({
            "id": id,
            "dim": &dim,
            "coordinates": [x, y, z],
            "dofs": dofs,
        });
        nodes.push(NodeRec { id, dim, body });
    }
    nodes.sort_by_cached_key(|node| order::natural_key(node.id.to_string().as_bytes()));
    Ok(nodes)
}

struct Named {
    name: String,
    names: BTreeSet<String>,
    body: Json,
}

fn parse_sections(table: Option<&toml::Table>) -> Result<Vec<Named>, Failure> {
    let Some(table) = table else {
        return Ok(Vec::new());
    };
    let mut sections = Vec::new();
    for (index, (name, value)) in table.iter().enumerate() {
        let pointer = format!("/sections/{index}");
        check_text_len(name, &pointer)?;
        let properties = as_table(value, &pointer)?;
        let mut names = BTreeSet::new();
        let mut pairs = Vec::new();
        for (key, property) in properties {
            check_text_len(key, &pointer)?;
            if !names.insert(key.clone()) {
                return Err(Failure::validation(
                    Some(pointer),
                    "An identifier is duplicated.",
                ));
            }
            let scalar = expect_scalar(property, &pointer)?;
            pairs.push((key.clone(), json_scalar(scalar)?));
        }
        pairs.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        let properties = pairs
            .into_iter()
            .map(|(name, value)| json!({"name": name, "value": value}))
            .collect::<Vec<_>>();
        sections.push(Named {
            name: name.clone(),
            names,
            body: json!({"id": name, "properties": properties}),
        });
    }
    sections.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    Ok(sections)
}

fn parse_laws(table: Option<&toml::Table>) -> Result<Vec<Named>, Failure> {
    let Some(table) = table else {
        return Ok(Vec::new());
    };
    let mut laws = Vec::new();
    for (index, (name, value)) in table.iter().enumerate() {
        let pointer = format!("/laws/{index}");
        check_text_len(name, &pointer)?;
        let law = as_table(value, &pointer)?;
        let kind = match law.get("kind") {
            None => None,
            Some(Toml::String(value)) if KINDS.contains(&value.as_str()) => Some(value.clone()),
            Some(Toml::String(_)) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A law field is not an allowed value.",
                ))
            }
            Some(_) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A value has the wrong type.",
                ))
            }
        };
        let mode = match law.get("mode") {
            None => None,
            Some(Toml::String(value)) if MODES.contains(&value.as_str()) => Some(value.clone()),
            Some(Toml::String(_)) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A law field is not an allowed value.",
                ))
            }
            Some(_) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A value has the wrong type.",
                ))
            }
        };
        let points = match law.get("points") {
            None => None,
            Some(value) => Some(expect_points(value, &pointer)?),
        };
        let mut names = BTreeSet::new();
        let mut pairs = Vec::new();
        for (key, parameter) in law {
            if matches!(key.as_str(), "kind" | "mode" | "points") {
                continue;
            }
            check_text_len(key, &pointer)?;
            let scalar = expect_scalar(parameter, &pointer)?;
            names.insert(key.clone());
            pairs.push((key.clone(), json_scalar(scalar)?));
        }
        pairs.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        let parameters = pairs
            .into_iter()
            .map(|(name, value)| json!({"name": name, "value": value}))
            .collect::<Vec<_>>();
        let mut body = json!({"name": name, "parameters": parameters});
        if let Some(kind) = kind {
            body["kind"] = json!(kind);
        }
        if let Some(mode) = mode {
            body["mode"] = json!(mode);
        }
        if let Some(points) = points {
            body["points"] = points;
        }
        laws.push(Named {
            name: name.clone(),
            names,
            body,
        });
    }
    laws.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    Ok(laws)
}

struct MemberRec {
    id: u64,
    body: Json,
}

fn parse_members(
    items: Option<&Vec<Toml>>,
    nodes: &[NodeRec],
    sections: &[Named],
    laws: &[Named],
) -> Result<Vec<MemberRec>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    if items.len() > MAX_MEMBERS {
        return Err(Failure::validation(
            Some("/members".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let dims: BTreeMap<u64, &str> = nodes
        .iter()
        .map(|node| (node.id, node.dim.as_str()))
        .collect();
    let section_names: BTreeMap<&str, &BTreeSet<String>> = sections
        .iter()
        .map(|section| (section.name.as_str(), &section.names))
        .collect();
    let law_names: BTreeMap<&str, &BTreeSet<String>> = laws
        .iter()
        .map(|law| (law.name.as_str(), &law.names))
        .collect();
    let mut members = Vec::with_capacity(items.len());
    let mut seen = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/members/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(
            table,
            &[
                "id",
                "from",
                "to",
                "section",
                "law",
                "element",
                "orientation",
                "release_i",
                "release_j",
                "winkler",
            ],
            &pointer,
        )?;
        let id = required_u32(table, "id", &pointer)?;
        if !seen.insert(id) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let from = required_u32(table, "from", &pointer)?;
        let to = required_u32(table, "to", &pointer)?;
        let section = required_text(table, "section", &pointer)?;
        let law = required_text(table, "law", &pointer)?;
        if !dims.contains_key(&from) || !dims.contains_key(&to) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let Some(properties) = section_names.get(section.as_str()) else {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        };
        let Some(parameters) = law_names.get(law.as_str()) else {
            return Err(Failure::validation(
                Some(pointer.clone()),
                "A reference does not exist.",
            ));
        };
        if properties.intersection(parameters).next().is_some() {
            return Err(Failure::validation(
                Some("/laws".to_string()),
                "A section property collides with a law parameter.",
            ));
        }
        let element = match table.get("element") {
            None => None,
            Some(Toml::String(value)) => Some(value.clone()),
            Some(_) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A value has the wrong type.",
                ))
            }
        };
        let orientation = match table.get("orientation") {
            None => None,
            Some(value) => Some(expect_vec3(value, &pointer)?),
        };
        let release_i = match table.get("release_i") {
            None => None,
            Some(value) => Some(expect_release(value, &pointer)?),
        };
        let release_j = match table.get("release_j") {
            None => None,
            Some(value) => Some(expect_release(value, &pointer)?),
        };
        let winkler = match table.get("winkler") {
            None => None,
            Some(value) => {
                let from_dim = dims[&from];
                let to_dim = dims[&to];
                if from_dim != to_dim {
                    return Err(Failure::validation(
                        Some(pointer),
                        "The member nodes do not share a dimension.",
                    ));
                }
                Some(expect_winkler(value, from_dim, &pointer)?)
            }
        };
        let mut body = json!({
            "id": id,
            "nodes": [from, to],
            "section": section,
            "law": law,
        });
        if let Some(element) = element {
            body["element"] = json!(element);
        }
        if let Some(orientation) = orientation {
            body["orientation"] = orientation;
        }
        if let Some(release_i) = release_i {
            body["release_i"] = release_i;
        }
        if let Some(release_j) = release_j {
            body["release_j"] = release_j;
        }
        if let Some(winkler) = winkler {
            body["winkler"] = winkler;
        }
        members.push(MemberRec { id, body });
    }
    members.sort_by_cached_key(|member| order::natural_key(member.id.to_string().as_bytes()));
    Ok(members)
}

struct SupportRec {
    id: String,
    body: Json,
}

fn parse_supports(
    items: Option<&Vec<Toml>>,
    nodes: &[NodeRec],
) -> Result<Vec<SupportRec>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    let node_ids: BTreeSet<u64> = nodes.iter().map(|node| node.id).collect();
    let mut supports = Vec::with_capacity(items.len());
    let mut seen: BTreeMap<String, (usize, bool)> = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/supports/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(table, &["id", "node", "restrained", "type"], &pointer)?;
        let node = required_u32(table, "node", &pointer)?;
        if !node_ids.contains(&node) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let restrained = match (table.get("restrained"), table.get("type")) {
            (Some(value), None) => expect_dofs(value, &pointer)?,
            (None, Some(_)) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "Support type expansion is not decided.",
                ))
            }
            _ => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A support needs exactly one restraint form.",
                ))
            }
        };
        let (id, written) = match table.get("id") {
            None => (format!("N{node}"), false),
            Some(value) => (expect_text(value, &pointer)?, true),
        };
        if let Some((previous, previous_written)) = seen.get(&id) {
            let at = if *previous_written { *previous } else { index };
            let cause = if written && *previous_written {
                "An identifier is duplicated."
            } else {
                "Support identifiers collide."
            };
            return Err(Failure::validation(Some(format!("/supports/{at}")), cause));
        }
        seen.insert(id.clone(), (index, written));
        let body = json!({"id": &id, "node": node, "restrained": restrained});
        supports.push(SupportRec { id, body });
    }
    supports.sort_by_cached_key(|support| order::natural_key(support.id.as_bytes()));
    Ok(supports)
}

fn assemble(
    nodes: Vec<NodeRec>,
    members: Vec<MemberRec>,
    sections: Vec<Named>,
    laws: Vec<Named>,
    supports: Vec<SupportRec>,
    experimental: bool,
) -> Json {
    let options = if experimental {
        json!({"allow_experimental": true})
    } else {
        json!({})
    };
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": nodes.into_iter().map(|node| node.body).collect::<Vec<_>>(),
            "members": members.into_iter().map(|member| member.body).collect::<Vec<_>>(),
            "sections": sections.into_iter().map(|section| section.body).collect::<Vec<_>>(),
            "laws": laws.into_iter().map(|law| law.body).collect::<Vec<_>>(),
            "supports": supports.into_iter().map(|support| support.body).collect::<Vec<_>>(),
            "masses": [],
        },
        "cases": [],
        "combinations": [],
        "analyses": [{"method": "linearStatic"}],
        "options": options,
    })
}

#[derive(Clone, Copy)]
enum Scalar {
    Int(i64),
    Float(f64),
}

fn scalar_f64(scalar: Scalar) -> f64 {
    match scalar {
        Scalar::Int(value) => value as f64,
        Scalar::Float(value) => value,
    }
}

fn json_scalar(scalar: Scalar) -> Result<Json, Failure> {
    match scalar {
        Scalar::Int(value) => Ok(json!(value)),
        Scalar::Float(value) => {
            if !value.is_finite() {
                return Err(Failure::nonfinite());
            }
            if value == 0.0 {
                return Ok(json!(0));
            }
            if value.abs() <= MAX_SAFE as f64 && value.fract() == 0.0 {
                return Ok(json!(value as i64));
            }
            Number::from_f64(value)
                .map(Json::Number)
                .ok_or_else(Failure::nonfinite)
        }
    }
}

fn expect_scalar(value: &Toml, pointer: &str) -> Result<Scalar, Failure> {
    match value {
        Toml::Integer(number) => Ok(Scalar::Int(*number)),
        Toml::Float(number) if number.is_finite() => Ok(Scalar::Float(*number)),
        Toml::Float(_) => Err(Failure::nonfinite()),
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "A value must be a number.",
        )),
    }
}

fn required_scalar(table: &toml::Table, key: &str, pointer: &str) -> Result<Scalar, Failure> {
    match table.get(key) {
        Some(value) => expect_scalar(value, pointer),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn required_u32(table: &toml::Table, key: &str, pointer: &str) -> Result<u64, Failure> {
    match table.get(key) {
        Some(Toml::Integer(value)) if (0..=4_294_967_295).contains(value) => Ok(*value as u64),
        Some(Toml::Integer(_)) => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier is outside the integer range.",
        )),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier must be an integer.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn required_text(table: &toml::Table, key: &str, pointer: &str) -> Result<String, Failure> {
    match table.get(key) {
        Some(value) => expect_text(value, pointer),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn expect_text(value: &Toml, pointer: &str) -> Result<String, Failure> {
    match value {
        Toml::String(text) => {
            check_text_len(text, pointer)?;
            Ok(text.clone())
        }
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier must be text.",
        )),
    }
}

fn check_text_len(text: &str, pointer: &str) -> Result<(), Failure> {
    if text.len() > MAX_ID_BYTES {
        Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier exceeds the length limit.",
        ))
    } else {
        Ok(())
    }
}

fn expect_dim(value: &Toml, pointer: &str) -> Result<String, Failure> {
    match value {
        Toml::String(value) if value == "D2" || value == "D3" => Ok(value.clone()),
        Toml::String(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A dimension is not allowed.",
        )),
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        )),
    }
}

fn expect_dofs(value: &Toml, pointer: &str) -> Result<Vec<String>, Failure> {
    let Toml::Array(items) = value else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A degree of freedom is not recognized.",
        ));
    };
    let mut seen = [false; 6];
    for item in items {
        let Toml::String(name) = item else {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is not recognized.",
            ));
        };
        let Some(index) = DOFS.iter().position(|dof| *dof == name) else {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is not recognized.",
            ));
        };
        if seen[index] {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is repeated.",
            ));
        }
        seen[index] = true;
    }
    Ok(DOFS
        .iter()
        .enumerate()
        .filter(|(index, _)| seen[*index])
        .map(|(_, dof)| (*dof).to_string())
        .collect())
}

fn expect_points(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    let Toml::Array(items) = value else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if !(2..=16384).contains(&items.len()) {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut points = Vec::with_capacity(items.len());
    for item in items {
        let Toml::Array(pair) = item else {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A value has the wrong type.",
            ));
        };
        if pair.len() != 2 {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A value has the wrong type.",
            ));
        }
        points.push(json!([
            json_scalar(expect_scalar(&pair[0], pointer)?)?,
            json_scalar(expect_scalar(&pair[1], pointer)?)?,
        ]));
    }
    Ok(Json::Array(points))
}

fn expect_vec3(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    let Toml::Array(items) = value else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if items.len() != 3 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    }
    Ok(json!([
        json_scalar(expect_scalar(&items[0], pointer)?)?,
        json_scalar(expect_scalar(&items[1], pointer)?)?,
        json_scalar(expect_scalar(&items[2], pointer)?)?,
    ]))
}

fn expect_release(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    match value {
        Toml::Boolean(value) => Ok(json!(value)),
        Toml::Array(items)
            if items.len() == 3 && items.iter().all(|item| matches!(item, Toml::Boolean(_))) =>
        {
            Ok(Json::Array(
                items
                    .iter()
                    .map(|item| json!(matches!(item, Toml::Boolean(true))))
                    .collect(),
            ))
        }
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "A release has the wrong shape.",
        )),
    }
}

fn expect_winkler(value: &Toml, dim: &str, pointer: &str) -> Result<Json, Failure> {
    let table = as_table(value, pointer)?;
    let keys: BTreeSet<&str> = table.keys().map(String::as_str).collect();
    let d2 = BTreeSet::from(["k"]);
    let d3 = BTreeSet::from(["ky", "kz"]);
    if dim == "D2" && keys == d2 {
        let k = nonnegative(table.get("k").unwrap(), pointer)?;
        return Ok(json!({"k": k}));
    }
    if dim == "D3" && keys == d3 {
        let ky = nonnegative(table.get("ky").unwrap(), pointer)?;
        let kz = nonnegative(table.get("kz").unwrap(), pointer)?;
        return Ok(json!({"ky": ky, "kz": kz}));
    }
    Err(Failure::validation(
        Some(pointer.to_string()),
        "A winkler value does not match the member dimension.",
    ))
}

fn nonnegative(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    let scalar = expect_scalar(value, pointer)?;
    if scalar_f64(scalar) < 0.0 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value is outside the allowed range.",
        ));
    }
    json_scalar(scalar)
}

fn optional_array<'a>(table: &'a toml::Table, key: &str) -> Result<Option<&'a Vec<Toml>>, Failure> {
    match table.get(key) {
        None => Ok(None),
        Some(Toml::Array(items)) => Ok(Some(items)),
        Some(_) => Err(Failure::validation(
            Some(format!("/{key}")),
            "A collection has the wrong shape.",
        )),
    }
}

fn optional_table<'a>(
    table: &'a toml::Table,
    key: &str,
) -> Result<Option<&'a toml::Table>, Failure> {
    match table.get(key) {
        None => Ok(None),
        Some(Toml::Table(nested)) => Ok(Some(nested)),
        Some(_) => Err(Failure::validation(
            Some(format!("/{key}")),
            "A collection has the wrong shape.",
        )),
    }
}

fn as_table<'a>(value: &'a Toml, pointer: &str) -> Result<&'a toml::Table, Failure> {
    match value {
        Toml::Table(table) => Ok(table),
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "A value must be a table.",
        )),
    }
}

fn reject_toml_keys(table: &toml::Table, allowed: &[&str], pointer: &str) -> Result<(), Failure> {
    if table.keys().any(|key| !allowed.contains(&key.as_str())) {
        Err(Failure::validation(
            Some(pointer.to_string()),
            "The document contains an unknown key.",
        ))
    } else {
        Ok(())
    }
}

fn export_root(value: &Json) -> Result<String, Failure> {
    let root = as_object(value, "")?;
    reject_json_keys(
        root,
        &[
            "schema",
            "units",
            "model",
            "cases",
            "combinations",
            "analyses",
            "options",
        ],
        "",
    )?;
    match root.get("schema").and_then(Json::as_str) {
        Some("engine-input/1") => {}
        Some(_) => return Err(Failure::unsupported("The format version is not supported.")),
        None => {
            return Err(Failure::validation(
                Some(String::new()),
                "The format field is missing.",
            ))
        }
    }
    match root.get("units").and_then(Json::as_str) {
        Some("SI") => {}
        Some(_) => {
            return Err(Failure::validation(
                Some(String::new()),
                "The units field is not SI.",
            ))
        }
        None => {
            return Err(Failure::validation(
                Some(String::new()),
                "The units field is missing.",
            ))
        }
    }
    require_empty_json_array(root, "cases", "/cases")?;
    require_empty_json_array(root, "combinations", "/combinations")?;
    require_json_linear_static(root)?;
    let experimental = json_experimental(root)?;
    let model = match root.get("model") {
        Some(model) => as_object(model, "/model")?,
        None => {
            return Err(Failure::validation(
                Some(String::new()),
                "A required field is missing.",
            ))
        }
    };
    reject_json_keys(
        model,
        &[
            "axes",
            "nodes",
            "members",
            "sections",
            "laws",
            "supports",
            "masses",
            "prescribed",
            "diaphragms",
            "springs",
        ],
        "/model",
    )?;
    match model.get("axes").and_then(Json::as_str) {
        Some("Z-up") => {}
        Some(_) => {
            return Err(Failure::validation(
                Some("/model".to_string()),
                "The axes value is not supported.",
            ))
        }
        None => {
            return Err(Failure::validation(
                Some("/model".to_string()),
                "A required field is missing.",
            ))
        }
    }
    require_empty_json_array(model, "masses", "/model/masses")?;
    require_empty_json_array(model, "prescribed", "/model/prescribed")?;
    require_empty_json_array(model, "diaphragms", "/model/diaphragms")?;
    require_empty_json_array(model, "springs", "/model/springs")?;
    let mut out = Out::default();
    out.entry("format", "\"mamut-toml/1\"");
    out.entry("units", "\"SI\"");
    write_nodes(&mut out, required_array(model, "nodes", "/model/nodes")?)?;
    write_sections(
        &mut out,
        required_array(model, "sections", "/model/sections")?,
    )?;
    write_laws(&mut out, required_array(model, "laws", "/model/laws")?)?;
    write_members(
        &mut out,
        required_array(model, "members", "/model/members")?,
        required_array(model, "nodes", "/model/nodes")?,
        required_array(model, "sections", "/model/sections")?,
        required_array(model, "laws", "/model/laws")?,
    )?;
    write_supports(
        &mut out,
        required_array(model, "supports", "/model/supports")?,
        required_array(model, "nodes", "/model/nodes")?,
    )?;
    out.header("[analysis]");
    out.entry("method", "\"linearStatic\"");
    if experimental {
        out.header("[options]");
        out.entry("allow_experimental", "true");
    }
    Ok(out.finish())
}

fn require_json_linear_static(root: &serde_json::Map<String, Json>) -> Result<(), Failure> {
    let Some(value) = root.get("analyses") else {
        return Err(Failure::validation(
            Some(String::new()),
            "The analysis field is missing.",
        ));
    };
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some("/analyses".to_string()),
            "A collection has the wrong shape.",
        ));
    };
    if items.len() != 1 {
        return Err(Failure::validation(
            Some("/analyses".to_string()),
            "A required field is missing.",
        ));
    }
    let analysis = as_object(&items[0], "/analyses/0")?;
    reject_json_keys(analysis, &["method"], "/analyses/0")?;
    match analysis.get("method").and_then(Json::as_str) {
        Some("linearStatic") => Ok(()),
        Some(_) => Err(Failure::validation(
            Some("/analyses/0".to_string()),
            "This analysis method is outside the implemented subset.",
        )),
        None => Err(Failure::validation(
            Some("/analyses/0".to_string()),
            "A required field is missing.",
        )),
    }
}

fn json_experimental(root: &serde_json::Map<String, Json>) -> Result<bool, Failure> {
    let Some(value) = root.get("options") else {
        return Ok(false);
    };
    let options = as_object(value, "/options")?;
    reject_json_keys(options, &["allow_experimental"], "/options")?;
    match options.get("allow_experimental") {
        None => Ok(false),
        Some(Json::Bool(value)) => Ok(*value),
        Some(_) => Err(Failure::validation(
            Some("/options".to_string()),
            "A value has the wrong type.",
        )),
    }
}

fn require_empty_json_array(
    object: &serde_json::Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<(), Failure> {
    match object.get(key) {
        None => Ok(()),
        Some(Json::Array(items)) if items.is_empty() => Ok(()),
        Some(Json::Array(_)) => Err(Failure::validation(
            Some(pointer.to_string()),
            "This collection is outside the implemented subset.",
        )),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A collection has the wrong shape.",
        )),
    }
}

fn write_nodes(out: &mut Out, items: &[Json]) -> Result<(), Failure> {
    let nodes = sorted_objects(items, "/model/nodes", |object, pointer| {
        Ok(order::natural_key(json_u32_id(object, pointer)?.as_bytes()))
    })?;
    if nodes.len() > MAX_NODES {
        return Err(Failure::validation(
            Some("/model/nodes".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut seen = BTreeSet::new();
    for (index, node) in nodes.iter().enumerate() {
        let pointer = format!("/model/nodes/{index}");
        reject_json_keys(node, &["id", "dim", "coordinates", "dofs"], &pointer)?;
        let id = json_u32_field(node, "id", &pointer)?;
        if !seen.insert(id) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let dim = json_dim(node, &pointer)?;
        let coordinates = json_vec3(
            node.get("coordinates").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        if dim == "D2" && coordinates[2] != 0.0 {
            return Err(Failure::validation(
                Some(pointer),
                "A D2 node requires z = 0.",
            ));
        }
        let dofs = json_dofs(
            node.get("dofs").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        out.header("[[nodes]]");
        out.entry("id", &id.to_string());
        out.entry("x", &format_f64(coordinates[0])?);
        out.entry("y", &format_f64(coordinates[1])?);
        out.entry("z", &format_f64(coordinates[2])?);
        out.entry("dim", &quoted(&dim));
        out.entry("dofs", &quoted_array(&dofs));
    }
    Ok(())
}

fn write_sections(out: &mut Out, items: &[Json]) -> Result<(), Failure> {
    let mut sections = section_records(items)?;
    sections.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    for (name, properties) in sections {
        out.header_named("sections", &name);
        for (key, value) in properties {
            out.entry(&key, &format_f64(value)?);
        }
    }
    Ok(())
}

fn write_laws(out: &mut Out, items: &[Json]) -> Result<(), Failure> {
    let mut laws = law_records(items)?;
    laws.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    for law in laws {
        out.header_named("laws", &law.name);
        if let Some(kind) = law.kind {
            out.entry("kind", &quoted(&kind));
        }
        if let Some(mode) = law.mode {
            out.entry("mode", &quoted(&mode));
        }
        if let Some(points) = law.points {
            out.entry("points", &points);
        }
        for (key, value) in law.parameters {
            out.entry(&key, &format_f64(value)?);
        }
    }
    Ok(())
}

fn write_members(
    out: &mut Out,
    items: &[Json],
    nodes: &[Json],
    sections: &[Json],
    laws: &[Json],
) -> Result<(), Failure> {
    let dims = node_dims(nodes)?;
    let section_names = name_sets(sections, "properties", "/model/sections")?;
    let law_names = name_sets(laws, "parameters", "/model/laws")?;
    let members = sorted_objects(items, "/model/members", |object, pointer| {
        Ok(order::natural_key(json_u32_id(object, pointer)?.as_bytes()))
    })?;
    if members.len() > MAX_MEMBERS {
        return Err(Failure::validation(
            Some("/model/members".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut seen = BTreeSet::new();
    for (index, member) in members.iter().enumerate() {
        let pointer = format!("/model/members/{index}");
        reject_json_keys(
            member,
            &[
                "id",
                "nodes",
                "section",
                "law",
                "element",
                "orientation",
                "release_i",
                "release_j",
                "winkler",
            ],
            &pointer,
        )?;
        let id = json_u32_field(member, "id", &pointer)?;
        if !seen.insert(id) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let endpoints = member_nodes(member, &pointer)?;
        let section = json_text_field(member, "section", &pointer)?;
        let law = json_text_field(member, "law", &pointer)?;
        if !dims.contains_key(&endpoints[0]) || !dims.contains_key(&endpoints[1]) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let Some(properties) = section_names.get(section.as_str()) else {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        };
        let Some(parameters) = law_names.get(law.as_str()) else {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        };
        if properties.intersection(parameters).next().is_some() {
            return Err(Failure::validation(
                Some("/model/laws".to_string()),
                "A section property collides with a law parameter.",
            ));
        }
        out.header("[[members]]");
        out.entry("id", &id.to_string());
        out.entry("from", &endpoints[0].to_string());
        out.entry("to", &endpoints[1].to_string());
        out.entry("section", &quoted(&section));
        out.entry("law", &quoted(&law));
        if let Some(element) = member.get("element") {
            let Some(element) = element.as_str() else {
                return Err(Failure::validation(
                    Some(pointer),
                    "A value has the wrong type.",
                ));
            };
            out.entry("element", &quoted(element));
        }
        if let Some(orientation) = member.get("orientation") {
            let orientation = json_vec3(orientation, &pointer)?;
            out.entry(
                "orientation",
                &format!(
                    "[{}, {}, {}]",
                    format_f64(orientation[0])?,
                    format_f64(orientation[1])?,
                    format_f64(orientation[2])?,
                ),
            );
        }
        if let Some(release) = member.get("release_i") {
            out.entry("release_i", &json_release(release, &pointer)?);
        }
        if let Some(release) = member.get("release_j") {
            out.entry("release_j", &json_release(release, &pointer)?);
        }
        if let Some(winkler) = member.get("winkler") {
            let from_dim = &dims[&endpoints[0]];
            let to_dim = &dims[&endpoints[1]];
            if from_dim != to_dim {
                return Err(Failure::validation(
                    Some(pointer),
                    "The member nodes do not share a dimension.",
                ));
            }
            out.entry("winkler", &json_winkler(winkler, from_dim, &pointer)?);
        }
    }
    Ok(())
}

fn write_supports(out: &mut Out, items: &[Json], nodes: &[Json]) -> Result<(), Failure> {
    let node_ids = node_dims(nodes)?;
    let supports = sorted_objects(items, "/model/supports", |object, pointer| {
        Ok(order::natural_key(
            json_text_field(object, "id", pointer)?.as_bytes(),
        ))
    })?;
    let mut seen = BTreeSet::new();
    for (index, support) in supports.iter().enumerate() {
        let pointer = format!("/model/supports/{index}");
        reject_json_keys(support, &["id", "node", "restrained"], &pointer)?;
        let id = json_text_field(support, "id", &pointer)?;
        if !seen.insert(id.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let node = json_u32_field(support, "node", &pointer)?;
        if !node_ids.contains_key(&node) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let dofs = json_dofs(
            support.get("restrained").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        out.header("[[supports]]");
        out.entry("id", &quoted(&id));
        out.entry("node", &node.to_string());
        out.entry("restrained", &quoted_array(&dofs));
    }
    Ok(())
}

fn section_records(items: &[Json]) -> Result<Vec<(String, Vec<(String, f64)>)>, Failure> {
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/model/sections/{index}");
        let section = as_object(item, &pointer)?;
        reject_json_keys(section, &["id", "properties"], &pointer)?;
        let id = json_text_field(section, "id", &pointer)?;
        if !seen.insert(id.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let pairs = property_pairs(
            section.get("properties").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        records.push((id, pairs));
    }
    Ok(records)
}

struct LawOut {
    name: String,
    kind: Option<String>,
    mode: Option<String>,
    points: Option<String>,
    parameters: Vec<(String, f64)>,
}

fn law_records(items: &[Json]) -> Result<Vec<LawOut>, Failure> {
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/model/laws/{index}");
        let law = as_object(item, &pointer)?;
        reject_json_keys(
            law,
            &["name", "parameters", "kind", "points", "mode"],
            &pointer,
        )?;
        let name = json_text_field(law, "name", &pointer)?;
        if !seen.insert(name.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let kind = match law.get("kind") {
            None => None,
            Some(Json::String(value)) if KINDS.contains(&value.as_str()) => Some(value.clone()),
            Some(Json::String(_)) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A law field is not an allowed value.",
                ))
            }
            Some(_) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A value has the wrong type.",
                ))
            }
        };
        let mode = match law.get("mode") {
            None => None,
            Some(Json::String(value)) if MODES.contains(&value.as_str()) => Some(value.clone()),
            Some(Json::String(_)) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A law field is not an allowed value.",
                ))
            }
            Some(_) => {
                return Err(Failure::validation(
                    Some(pointer),
                    "A value has the wrong type.",
                ))
            }
        };
        let points = match law.get("points") {
            None => None,
            Some(value) => Some(format_points(value, &pointer)?),
        };
        let parameters = property_pairs(
            law.get("parameters").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        // P1: kind, points and mode are metadata, never parameter names.
        if parameters
            .iter()
            .any(|(name, _)| matches!(name.as_str(), "kind" | "mode" | "points"))
        {
            return Err(Failure::validation(
                Some(pointer),
                "A law field is not an allowed value.",
            ));
        }
        records.push(LawOut {
            name,
            kind,
            mode,
            points,
            parameters,
        });
    }
    Ok(records)
}

fn property_pairs(value: &Json, pointer: &str) -> Result<Vec<(String, f64)>, Failure> {
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    let mut pairs = Vec::new();
    let mut seen = BTreeSet::new();
    for item in items {
        let pair = as_object(item, pointer)?;
        reject_json_keys(pair, &["name", "value"], pointer)?;
        let name = json_text_field(pair, "name", pointer)?;
        if !seen.insert(name.clone()) {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "An identifier is duplicated.",
            ));
        }
        let number = json_f64(pair.get("value").ok_or_else(|| {
            Failure::validation(Some(pointer.to_string()), "A required field is missing.")
        })?)?;
        pairs.push((name, number));
    }
    pairs.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    Ok(pairs)
}

fn name_sets(
    items: &[Json],
    field: &str,
    pointer: &str,
) -> Result<BTreeMap<String, BTreeSet<String>>, Failure> {
    let mut sets = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        let object = as_object(item, &format!("{pointer}/{index}"))?;
        let id_key = if field == "properties" { "id" } else { "name" };
        let Some(id) = object.get(id_key).and_then(Json::as_str) else {
            continue;
        };
        let names = object
            .get(field)
            .and_then(Json::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("name").and_then(Json::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        sets.insert(id.to_string(), names);
    }
    Ok(sets)
}

fn node_dims(nodes: &[Json]) -> Result<BTreeMap<u64, String>, Failure> {
    let mut dims = BTreeMap::new();
    for (index, node) in nodes.iter().enumerate() {
        let pointer = format!("/model/nodes/{index}");
        let node = as_object(node, &pointer)?;
        let id = json_u32_field(node, "id", &pointer)?;
        let dim = json_dim(node, &pointer)?;
        if dims.insert(id, dim).is_some() {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
    }
    Ok(dims)
}

fn sorted_objects<T: Ord>(
    items: &[Json],
    pointer: &str,
    key_of: impl Fn(&serde_json::Map<String, Json>, &str) -> Result<T, Failure>,
) -> Result<Vec<serde_json::Map<String, Json>>, Failure> {
    let mut objects = Vec::new();
    for (index, item) in items.iter().enumerate() {
        objects.push(as_object(item, &format!("{pointer}/{index}"))?.clone());
    }
    let mut keyed = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        let key = key_of(object, &format!("{pointer}/{index}"))?;
        keyed.push((key, index));
    }
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(keyed
        .into_iter()
        .map(|(_, index)| objects[index].clone())
        .collect())
}

fn json_u32_id(object: &serde_json::Map<String, Json>, pointer: &str) -> Result<String, Failure> {
    Ok(json_u32_field(object, "id", pointer)?.to_string())
}

fn member_nodes(
    member: &serde_json::Map<String, Json>,
    pointer: &str,
) -> Result<[u64; 2], Failure> {
    let Some(nodes) = member.get("nodes").and_then(Json::as_array) else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        ));
    };
    if nodes.len() != 2 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    }
    Ok([json_u32(&nodes[0], pointer)?, json_u32(&nodes[1], pointer)?])
}

fn json_u32_field(
    object: &serde_json::Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<u64, Failure> {
    match object.get(key) {
        Some(value) => json_u32(value, pointer),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn json_u32(value: &Json, pointer: &str) -> Result<u64, Failure> {
    let Some(number) = value.as_number() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier must be an integer.",
        ));
    };
    if number.is_f64() {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier must be an integer.",
        ));
    }
    match number.as_u64() {
        Some(value) if value <= u32::MAX as u64 => Ok(value),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier is outside the integer range.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier must be an integer.",
        )),
    }
}

fn json_text_field(
    object: &serde_json::Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<String, Failure> {
    match object.get(key) {
        Some(Json::String(value)) => {
            check_text_len(value, pointer)?;
            Ok(value.clone())
        }
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier must be text.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn json_dim(object: &serde_json::Map<String, Json>, pointer: &str) -> Result<String, Failure> {
    match object.get("dim").and_then(Json::as_str) {
        Some(value) if value == "D2" || value == "D3" => Ok(value.to_string()),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A dimension is not allowed.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn json_vec3(value: &Json, pointer: &str) -> Result<[f64; 3], Failure> {
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if items.len() != 3 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    }
    Ok([
        json_f64(&items[0])?,
        json_f64(&items[1])?,
        json_f64(&items[2])?,
    ])
}

fn json_f64(value: &Json) -> Result<f64, Failure> {
    let Some(number) = value.as_number() else {
        return Err(Failure::validation(None, "A value must be a number."));
    };
    if let Some(value) = number.as_i64() {
        return Ok(value as f64);
    }
    if let Some(value) = number.as_u64() {
        if value > MAX_SAFE as u64 {
            return Err(Failure::validation(
                None,
                "An integer is outside the safe range.",
            ));
        }
        return Ok(value as f64);
    }
    match number.as_f64() {
        Some(value) if value.is_finite() => Ok(value),
        _ => Err(Failure::nonfinite()),
    }
}

fn json_dofs(value: &Json, pointer: &str) -> Result<Vec<String>, Failure> {
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A degree of freedom is not recognized.",
        ));
    };
    let mut seen = [false; 6];
    for item in items {
        let Some(name) = item.as_str() else {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is not recognized.",
            ));
        };
        let Some(index) = DOFS.iter().position(|dof| *dof == name) else {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is not recognized.",
            ));
        };
        if seen[index] {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is repeated.",
            ));
        }
        seen[index] = true;
    }
    Ok(DOFS
        .iter()
        .enumerate()
        .filter(|(index, _)| seen[*index])
        .map(|(_, dof)| (*dof).to_string())
        .collect())
}

fn json_release(value: &Json, pointer: &str) -> Result<String, Failure> {
    match value {
        Json::Bool(value) => Ok(if *value { "true" } else { "false" }.to_string()),
        Json::Array(items) if items.len() == 3 && items.iter().all(Json::is_boolean) => {
            let text = items
                .iter()
                .map(|item| {
                    if item.as_bool() == Some(true) {
                        "true"
                    } else {
                        "false"
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            Ok(format!("[{text}]"))
        }
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "A release has the wrong shape.",
        )),
    }
}

fn json_winkler(value: &Json, dim: &str, pointer: &str) -> Result<String, Failure> {
    let table = as_object(value, pointer)?;
    let keys: BTreeSet<&str> = table.keys().map(String::as_str).collect();
    if dim == "D2" && keys == BTreeSet::from(["k"]) {
        let k = json_f64(table.get("k").unwrap())?;
        if k < 0.0 {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A value is outside the allowed range.",
            ));
        }
        return Ok(format!("{{ k = {} }}", format_f64(k)?));
    }
    if dim == "D3" && keys == BTreeSet::from(["ky", "kz"]) {
        let ky = json_f64(table.get("ky").unwrap())?;
        let kz = json_f64(table.get("kz").unwrap())?;
        if ky < 0.0 || kz < 0.0 {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A value is outside the allowed range.",
            ));
        }
        return Ok(format!(
            "{{ ky = {}, kz = {} }}",
            format_f64(ky)?,
            format_f64(kz)?
        ));
    }
    Err(Failure::validation(
        Some(pointer.to_string()),
        "A winkler value does not match the member dimension.",
    ))
}

fn format_points(value: &Json, pointer: &str) -> Result<String, Failure> {
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if !(2..=16384).contains(&items.len()) {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut parts = Vec::new();
    for item in items {
        let pair = json_pair(item, pointer)?;
        parts.push(format!(
            "[{}, {}]",
            format_f64(pair[0])?,
            format_f64(pair[1])?
        ));
    }
    Ok(format!("[{}]", parts.join(", ")))
}

fn json_pair(value: &Json, pointer: &str) -> Result<[f64; 2], Failure> {
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if items.len() != 2 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    }
    Ok([json_f64(&items[0])?, json_f64(&items[1])?])
}

fn format_f64(value: f64) -> Result<String, Failure> {
    if !value.is_finite() {
        return Err(Failure::nonfinite());
    }
    if value == 0.0 {
        return Ok("0".to_string());
    }
    if value.abs() <= MAX_SAFE as f64 && value.fract() == 0.0 {
        return Ok(format!("{}", value as i64));
    }
    let mut text = format!("{value}");
    if !text.contains(['.', 'e', 'E']) {
        text.push_str(".0");
    }
    Ok(text)
}

fn quoted(value: &str) -> String {
    let mut out = String::new();
    push_basic(&mut out, value);
    out
}

fn quoted_array(items: &[String]) -> String {
    let mut out = String::from("[");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_basic(&mut out, item);
    }
    out.push(']');
    out
}

fn push_basic(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{000c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            other if (other as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04X}", other as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

fn push_key(out: &mut String, value: &str) {
    let bare = !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    if bare {
        out.push_str(value);
    } else {
        push_basic(out, value);
    }
}

fn as_object<'a>(
    value: &'a Json,
    pointer: &str,
) -> Result<&'a serde_json::Map<String, Json>, Failure> {
    value
        .as_object()
        .ok_or_else(|| Failure::validation(Some(pointer.to_string()), "A value must be a table."))
}

fn required_array<'a>(
    object: &'a serde_json::Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<&'a [Json], Failure> {
    match object.get(key) {
        Some(Json::Array(items)) => Ok(items),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A collection has the wrong shape.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn reject_json_keys(
    object: &serde_json::Map<String, Json>,
    allowed: &[&str],
    pointer: &str,
) -> Result<(), Failure> {
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        Err(Failure::validation(
            Some(pointer.to_string()),
            "The document contains an unknown key.",
        ))
    } else {
        Ok(())
    }
}

#[derive(Default)]
struct Out(String);

impl Out {
    fn line(&mut self, text: &str) {
        self.0.push_str(text);
        self.0.push('\n');
    }

    fn entry(&mut self, key: &str, value: &str) {
        let mut line = String::new();
        push_key(&mut line, key);
        line.push_str(" = ");
        line.push_str(value);
        self.line(&line);
    }

    fn header(&mut self, text: &str) {
        if self.0.ends_with('\n') && !self.0.ends_with("\n\n") {
            self.0.push('\n');
        }
        self.line(text);
    }

    fn header_named(&mut self, family: &str, name: &str) {
        let mut text = format!("[{family}.");
        push_key(&mut text, name);
        text.push(']');
        self.header(&text);
    }

    fn finish(self) -> String {
        if self.0.ends_with('\n') {
            self.0
        } else {
            let mut text = self.0;
            text.push('\n');
            text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal(body: &str) -> String {
        format!(
            "format = \"mamut-toml/1\"\nunits = \"SI\"\n{body}\n[analysis]\nmethod = \"linearStatic\"\n"
        )
    }

    fn convert(body: &str) -> Json {
        let text = toml_to_json(&minimal(body)).unwrap();
        serde_json::from_str(text.trim()).unwrap()
    }

    fn failure_of(body: &str) -> Failure {
        toml_to_json(&minimal(body)).unwrap_err()
    }

    fn hidden(failure: &Failure, sentinel: &str) {
        assert!(!failure.cause.contains(sentinel));
        assert!(!failure.pointer.as_deref().unwrap_or("").contains(sentinel));
    }

    #[test]
    fn structural_round_trip_keeps_bytes_and_hash() {
        let source = minimal(
            r#"
            [defaults]
            dim = "D2"
            dofs = ["Rz", "Ux", "Uy"]

            [[nodes]]
            id = 10
            x = 1
            y = 0

            [[nodes]]
            id = 2
            x = 0
            y = 0

            [sections.S10]
            A = 2
            I = 3

            [sections.S2]
            A = 4

            [sections.S01]
            A = 5

            [laws.elastic]
            E = 200
            kind = "elastic"

            [[members]]
            id = 4
            from = 2
            to = 10
            section = "S2"
            law = "elastic"
            winkler = { k = 1.5 }
            release_i = false

            [[supports]]
            node = 2
            restrained = ["Uy", "Ux"]
            "#,
        );
        let json = toml_to_json(&source).unwrap();
        let value: Json = serde_json::from_str(json.trim()).unwrap();
        assert_eq!(value["schema"], "engine-input/1");
        assert_eq!(value["model"]["axes"], "Z-up");
        assert_eq!(value["model"]["nodes"][0]["id"], 2);
        assert_eq!(value["model"]["nodes"][1]["id"], 10);
        assert_eq!(value["model"]["nodes"][0]["dim"], "D2");
        assert_eq!(value["model"]["nodes"][0]["coordinates"][2], 0);
        assert_eq!(
            value["model"]["nodes"][0]["dofs"],
            json!(["Ux", "Uy", "Rz"])
        );
        assert_eq!(value["model"]["members"][0]["release_i"], false);
        assert_eq!(value["model"]["sections"][0]["id"], "S01");
        assert_eq!(value["model"]["sections"][1]["id"], "S10");
        assert_eq!(value["model"]["sections"][2]["id"], "S2");
        assert_eq!(value["model"]["supports"][0]["id"], "N2");
        assert_eq!(value["model"]["supports"][0]["restrained"][0], "Ux");
        assert_eq!(value["model"]["members"][0]["winkler"]["k"], 1.5);
        assert!(value["model"]["masses"].as_array().unwrap().is_empty());
        assert!(value.get("prescribed").is_none());
        let toml_a = json_to_toml(&json).unwrap();
        let toml_b = json_to_toml(&json).unwrap();
        assert_eq!(toml_a, toml_b);
        assert!(!toml_a.contains("[defaults]"));
        assert!(toml_a.starts_with("format = \"mamut-toml/1\"\nunits = \"SI\"\n"));
        let id2 = toml_a.find("id = 2\n").unwrap();
        let id10 = toml_a.find("id = 10\n").unwrap();
        assert!(id2 < id10);
        let s01 = toml_a.find("[sections.S01]").unwrap();
        let s10 = toml_a.find("[sections.S10]").unwrap();
        let s2 = toml_a.find("[sections.S2]").unwrap();
        assert!(s01 < s10 && s10 < s2);
        let json_again = toml_to_json(&toml_a).unwrap();
        assert_eq!(
            engine_api::input_sha256(json.trim()).unwrap(),
            engine_api::input_sha256(json_again.trim()).unwrap()
        );
    }

    #[test]
    fn float_spellings_share_a_hash_inside_each_u3_class() {
        let hash = |literal: &str| {
            let json = toml_to_json(&minimal(&format!(
                "[[nodes]]\nid = 1\nx = {literal}\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n"
            )))
            .unwrap();
            engine_api::input_sha256(json.trim()).unwrap()
        };
        assert_eq!(hash("4"), hash("4.0"));
        assert_eq!(hash("4.0"), hash("4e0"));
        assert_eq!(hash("0"), hash("0.0"));
        assert_eq!(hash("0.0"), hash("-0.0"));
        assert_ne!(hash("4"), hash("0"));
    }

    #[test]
    fn large_float_stays_a_float_on_export() {
        let json = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[{"id":1,"dim":"D2","coordinates":[1e20,0,0],"dofs":["Ux"]}],"members":[],"sections":[],"laws":[],"supports":[],"masses":[]},"cases":[],"combinations":[],"analyses":[{"method":"linearStatic"}]}"#;
        let toml = json_to_toml(json).unwrap();
        assert!(toml.contains("x = ") && !toml.contains("x = 1\n"));
        let back = toml_to_json(&toml).unwrap();
        let value: Json = serde_json::from_str(back.trim()).unwrap();
        assert_eq!(value["model"]["nodes"][0]["coordinates"][0], 1e20);
    }

    #[test]
    fn sentinels_do_not_appear_in_diagnostics() {
        let cases = [
            (
                "SENTINEL_KEY_9f3a = 1\n",
                "CLI-E-VALIDATION",
                "SENTINEL_KEY_9f3a",
            ),
            (
                "[engine]\nraw = \"SENTINEL_ENGINE_9f3a\"\n",
                "CLI-E-VALIDATION",
                "SENTINEL_ENGINE_9f3a",
            ),
            (
                "[[nodes]]\nid = \"SENTINEL_TYPE_9f3a\"\nx = 0\ny = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n",
                "CLI-E-VALIDATION",
                "SENTINEL_TYPE_9f3a",
            ),
            (
                "[[supports]]\nnode = 1\ntype = \"SENTINEL_PIN_9f3a\"\n[[nodes]]\nid = 1\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n",
                "CLI-E-VALIDATION",
                "SENTINEL_PIN_9f3a",
            ),
        ];
        for (body, code, sentinel) in cases {
            let failure = failure_of(body);
            assert_eq!(failure.code, code);
            hidden(&failure, sentinel);
        }
        let long = format!(
            "[sections.\"{}SENTINEL_LONG_9f3a\"]\nA = 1\n",
            "a".repeat(250)
        );
        let failure = failure_of(&long);
        assert_eq!(failure.cause, "An identifier exceeds the length limit.");
        hidden(&failure, "SENTINEL_LONG_9f3a");
        let collision = minimal(
            r#"
            [[nodes]]
            id = 1
            x = 0
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux"]
            [sections.s]
            SENTINEL_INTERSECT_9f3a = 1
            [laws.L]
            SENTINEL_INTERSECT_9f3a = 2
            [[members]]
            id = 1
            from = 1
            to = 1
            section = "s"
            law = "L"
            "#,
        );
        let failure = toml_to_json(&collision).unwrap_err();
        assert_eq!(
            failure.cause,
            "A section property collides with a law parameter."
        );
        hidden(&failure, "SENTINEL_INTERSECT_9f3a");
    }

    #[test]
    fn kind_metadata_is_not_a_parameter() {
        let value = convert(
            r#"
            [[nodes]]
            id = 1
            x = 0
            y = 0
            dim = "D2"
            dofs = ["Ux"]
            [sections.s]
            kind = 1
            [laws.L]
            kind = "elastic"
            E = 2
            [[members]]
            id = 1
            from = 1
            to = 1
            section = "s"
            law = "L"
            "#,
        );
        assert_eq!(value["model"]["members"][0]["section"], "s");
    }

    #[test]
    fn closed_rejections_use_the_registered_codes() {
        assert_eq!(
            toml_to_json(
                "format = \"mamut-toml/2\"\nunits = \"SI\"\n[analysis]\nmethod = \"linearStatic\"\n"
            )
            .unwrap_err()
            .code,
            "CLI-E-UNSUPPORTED"
        );
        assert_eq!(
            toml_to_json(
                "format = \"mamut-toml/1\"\nunits = \"kip\"\n[analysis]\nmethod = \"linearStatic\"\n"
            )
            .unwrap_err()
            .code,
            "CLI-E-VALIDATION"
        );
        assert_eq!(
            toml_to_json("format = \"mamut-toml/1\"\nunits = \"SI\"\nx = 1979-05-27T07:32:00Z\n")
                .unwrap_err()
                .cause,
            "The document contains a datetime."
        );
        assert_eq!(failure_of("nan_value = nan\n").code, "ENG-E-0008");
        assert_eq!(failure_of("nan_value = inf\n").code, "ENG-E-0008");
        assert_eq!(failure_of("nan_value = -inf\n").code, "ENG-E-0008");
        assert_eq!(
            failure_of("[[cases]]\nname = \"SENTINEL_CASE_9f3a\"\n").cause,
            "This collection is outside the implemented subset."
        );
        assert!(toml_to_json(&minimal("cases = []\n")).is_ok());
        let deep = format!(
            "[{}]\nx = 1\n",
            (0..70).map(|_| "a").collect::<Vec<_>>().join(".")
        );
        assert_eq!(
            toml_to_json(&deep).unwrap_err().cause,
            "The nesting exceeds the depth limit."
        );
        let duplicate = "SENTINEL_DUPKEY_9f3a = 1\nSENTINEL_DUPKEY_9f3a = 2\n";
        let failure = toml_to_json(duplicate).unwrap_err();
        assert_eq!(failure.code, "CLI-E-DOCUMENT");
        hidden(&failure, "SENTINEL_DUPKEY_9f3a");
    }

    #[test]
    fn text_ids_of_256_bytes_are_accepted() {
        let name = "é".repeat(128);
        assert_eq!(name.len(), 256);
        let value = convert(&format!("[sections.\"{name}\"]\nA = 1\n"));
        assert_eq!(value["model"]["sections"][0]["id"], name);
        let longer = format!("[sections.\"{}\"]\nA = 1\n", "é".repeat(129));
        assert_eq!(
            failure_of(&longer).cause,
            "An identifier exceeds the length limit."
        );
    }

    #[test]
    fn non_empty_deferred_json_is_not_dropped() {
        let json = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[],"members":[],"sections":[],"laws":[],"supports":[],"masses":[{"id":"SENTINEL_MASS_9f3a","node":1,"values":[]}]},"cases":[],"combinations":[],"analyses":[{"method":"linearStatic"}]}"#;
        let failure = json_to_toml(json).unwrap_err();
        assert_eq!(
            failure.cause,
            "This collection is outside the implemented subset."
        );
        hidden(&failure, "SENTINEL_MASS_9f3a");
    }

    #[test]
    fn export_uses_natural_order_for_members_and_supports() {
        let json = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[{"id":1,"dim":"D2","coordinates":[0,0,0],"dofs":["Ux"]}],"members":[{"id":10,"nodes":[1,1],"section":"S","law":"L"},{"id":2,"nodes":[1,1],"section":"S","law":"L"}],"sections":[{"id":"S","properties":[]}],"laws":[{"name":"L","parameters":[]}],"supports":[{"id":"S10","node":1,"restrained":["Rz"]},{"id":"S2","node":1,"restrained":["Ux"]},{"id":"S01","node":1,"restrained":["Uy"]},{"id":"","node":1,"restrained":["Ux"]},{"id":"A","node":1,"restrained":["Ux"]}],"masses":[]},"cases":[],"combinations":[],"analyses":[{"method":"linearStatic"}]}"#;
        let toml = json_to_toml(json).unwrap();
        let members = toml.find("[[members]]").unwrap();
        let id2 = toml[members..].find("id = 2\n").unwrap();
        let id10 = toml[members..].find("id = 10\n").unwrap();
        assert!(id2 < id10);
        let s01 = toml.find("id = \"S01\"").unwrap();
        let s2 = toml.find("id = \"S2\"").unwrap();
        let s10 = toml.find("id = \"S10\"").unwrap();
        let empty = toml.find("id = \"\"\n").unwrap();
        let letter = toml.find("id = \"A\"").unwrap();
        assert!(s01 < s2 && s2 < s10 && s10 < empty && empty < letter);
        assert_eq!(json_to_toml(json).unwrap(), toml);
    }

    #[test]
    fn support_identifier_collisions_do_not_echo() {
        let written = failure_of(
            r#"
            [[nodes]]
            id = 7
            x = 0
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux"]
            [[supports]]
            id = "SENTINEL_SUP_9f3a"
            node = 7
            restrained = ["Ux"]
            [[supports]]
            id = "SENTINEL_SUP_9f3a"
            node = 7
            restrained = ["Uy"]
            "#,
        );
        assert_eq!(written.cause, "An identifier is duplicated.");
        assert_eq!(written.pointer.as_deref(), Some("/supports/0"));
        hidden(&written, "SENTINEL_SUP_9f3a");

        let auto_after = failure_of(
            r#"
            [[nodes]]
            id = 7
            x = 0
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux"]
            [[supports]]
            node = 7
            restrained = ["Ux"]
            [[supports]]
            id = "N7"
            node = 7
            restrained = ["Uy"]
            "#,
        );
        assert_eq!(auto_after.cause, "Support identifiers collide.");
        assert_eq!(auto_after.pointer.as_deref(), Some("/supports/1"));

        let auto_before = failure_of(
            r#"
            [[nodes]]
            id = 7
            x = 0
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux"]
            [[supports]]
            id = "N7"
            node = 7
            restrained = ["Ux"]
            [[supports]]
            node = 7
            restrained = ["Uy"]
            "#,
        );
        assert_eq!(auto_before.cause, "Support identifiers collide.");
        assert_eq!(auto_before.pointer.as_deref(), Some("/supports/0"));
    }

    #[test]
    fn dotted_section_names_stay_literal() {
        let value = convert("[sections.\"a.b\"]\nA = 1\n");
        assert_eq!(value["model"]["sections"][0]["id"], "a.b");
        let toml = json_to_toml(&serde_json::to_string(&value).unwrap()).unwrap();
        assert!(toml.contains("[sections.\"a.b\"]"));
        assert!(!toml.contains("[sections.a.b]"));
    }

    #[test]
    fn safe_integer_boundary_and_u32_ids() {
        let value = convert(
            "[[nodes]]\nid = 1\nx = 9007199254740991\ny = -9007199254740991\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n",
        );
        assert_eq!(
            value["model"]["nodes"][0]["coordinates"][0],
            json!(9_007_199_254_740_991_i64)
        );
        assert_eq!(
            failure_of("x = 9007199254740992\n").cause,
            "An integer is outside the safe range."
        );
        assert_eq!(
            failure_of(
                "[[nodes]]\nid = 4294967296\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n"
            )
            .cause,
            "An identifier is outside the integer range."
        );
        assert_eq!(
            failure_of("[[nodes]]\nid = 1.0\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n")
                .cause,
            "An identifier must be an integer."
        );
    }

    #[test]
    fn reserved_law_parameter_names_are_rejected_on_export() {
        let json = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[],"members":[],"sections":[],"laws":[{"name":"L","kind":"elastic","parameters":[{"name":"kind","value":1},{"name":"SENTINEL_P1_9f3a","value":2}]}],"supports":[],"masses":[]},"cases":[],"combinations":[],"analyses":[{"method":"linearStatic"}]}"#;
        let failure = json_to_toml(json).unwrap_err();
        assert_eq!(failure.cause, "A law field is not an allowed value.");
        assert_eq!(failure.pointer.as_deref(), Some("/model/laws/0"));
        hidden(&failure, "SENTINEL_P1_9f3a");
    }

    #[test]
    fn json_null_and_nonfinite_use_fixed_causes() {
        let null = json_to_toml(r#"{"schema":"engine-input/1","units":null}"#).unwrap_err();
        assert_eq!(null.cause, "The document contains a null value.");
        let nonfinite =
            json_to_toml(r#"{"schema":"engine-input/1","units":"SI","x":NaN}"#).unwrap_err();
        assert_eq!(nonfinite.code, "ENG-E-0008");
        hidden(&nonfinite, "NaN");
    }

    #[test]
    fn other_datetime_forms_are_validation_errors() {
        for literal in ["1979-05-27", "07:32:00", "1979-05-27T07:32:00"] {
            let failure = toml_to_json(&format!(
                "format = \"mamut-toml/1\"\nunits = \"SI\"\nx = {literal}\n"
            ))
            .unwrap_err();
            assert_eq!(failure.cause, "The document contains a datetime.");
            assert_eq!(failure.code, "CLI-E-VALIDATION");
        }
    }

    #[test]
    fn empty_deferred_tables_are_accepted_and_nonempty_are_not() {
        assert!(toml_to_json(&minimal("[combinations]\n")).is_ok());
        let failure = failure_of("[combinations.SENTINEL_COMB_9f3a]\nDL = 1\n");
        assert_eq!(
            failure.cause,
            "This collection is outside the implemented subset."
        );
        hidden(&failure, "SENTINEL_COMB_9f3a");
    }
}
