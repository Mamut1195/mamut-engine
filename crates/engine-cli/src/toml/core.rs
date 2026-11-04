// SPDX-License-Identifier: AGPL-3.0-only

//! Map mamut-toml/1 to engine-input/1 and back for the structural core,
//! the load collections, and the analysis branches.
//!
//! U5 stays open: textual aliases of node, member, and spring ids are rejected.
//! U7 stays open: an identifier longer than 256 bytes is rejected.
//! U9 stays open: the emitter is deterministic, not a unique textual form,
//! and it does not extract `[defaults]`.
//! U10 stays open: `type` is rejected instead of expanded into a DOF list.
//! U11 stays open: nesting uses a provisional fence, and errors omit line
//! and column. `spring.nonlinear` is accepted only with `nonlinearStatic`.

mod analysis;

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
const MAX_CASES: usize = 64;
const MAX_COMBINATIONS: usize = 128;
const MAX_LOADS: usize = 16_384;
const NEST_LIMIT: usize = 64;
const DOFS: [&str; 6] = ["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"];
const LOAD_PAIRS: [(&str, &str); 6] = [
    ("Fx", "Ux"),
    ("Fy", "Uy"),
    ("Fz", "Uz"),
    ("Mx", "Rx"),
    ("My", "Ry"),
    ("Mz", "Rz"),
];
const KINDS: [&str; 4] = [
    "elastic",
    "steel.bilinear_kinematic",
    "multilinear_elastic",
    "elastic.cubic",
];
const MODES: [&str; 3] = ["both", "compression_only", "tension_only"];
const ROOT_KEYS: [&str; 16] = [
    "format",
    "units",
    "defaults",
    "nodes",
    "sections",
    "laws",
    "members",
    "supports",
    "masses",
    "springs",
    "prescribed",
    "diaphragms",
    "cases",
    "combinations",
    "analysis",
    "options",
];

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
    export_root(&value)
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
    let defaults = parse_defaults(table)?;
    let nodes = parse_nodes(optional_array(table, "nodes")?, &defaults)?;
    let sections = parse_sections(optional_table(table, "sections")?)?;
    let laws = parse_laws(optional_table(table, "laws")?)?;
    let members = parse_members(optional_array(table, "members")?, &nodes, &sections, &laws)?;
    let supports = parse_supports(optional_array(table, "supports")?, &nodes)?;
    let masses = parse_masses(optional_array(table, "masses")?, &nodes)?;
    let springs_empty = match optional_array(table, "springs")? {
        Some(items) => items.is_empty(),
        None => true,
    };
    let prescribed = parse_prescribed(optional_array(table, "prescribed")?, &nodes)?;
    let diaphragms = parse_diaphragms(optional_array(table, "diaphragms")?, &nodes)?;
    let cases = parse_cases(optional_array(table, "cases")?, &nodes, &members)?;
    let combinations = parse_combinations(optional_table(table, "combinations")?, &cases.names)?;
    let analysis = analysis::from_toml(
        table,
        &cases.names,
        &analysis::BranchContext {
            experimental,
            structural_empty: nodes.is_empty()
                && members.is_empty()
                && sections.is_empty()
                && laws.is_empty()
                && supports.is_empty()
                && masses.is_empty()
                && springs_empty
                && prescribed.is_empty()
                && diaphragms.is_empty()
                && cases.bodies.is_empty()
                && combinations.is_empty(),
            combinations_empty: combinations.is_empty(),
            diaphragms_empty: diaphragms.is_empty(),
        },
    )?;
    let law_names = laws.iter().map(|law| law.name.clone()).collect();
    let springs = parse_springs(
        optional_array(table, "springs")?,
        &nodes,
        &members,
        analysis.nonlinear,
        &law_names,
    )?;
    Ok(assemble(ModelParts {
        nodes,
        members,
        sections,
        laws,
        supports,
        masses,
        springs,
        prescribed,
        diaphragms,
        cases: cases.bodies,
        combinations,
        experimental,
        analysis: analysis.body,
    }))
}

fn screen_root(table: &toml::Table) -> Result<(), Failure> {
    table.values().try_for_each(screen)
}

fn check_root_keys(table: &toml::Table) -> Result<(), Failure> {
    for key in table.keys() {
        if !ROOT_KEYS.contains(&key.as_str()) {
            return Err(Failure::validation(
                Some(String::new()),
                "The document contains an unknown key.",
            ));
        }
    }
    Ok(())
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

struct MassRec {
    id: String,
    body: Json,
}

fn parse_masses(items: Option<&Vec<Toml>>, nodes: &[NodeRec]) -> Result<Vec<MassRec>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    let node_ids = node_id_set(nodes);
    let mut masses = Vec::with_capacity(items.len());
    let mut seen = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/masses/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(
            table,
            &["id", "node", "Ux", "Uy", "Uz", "Rx", "Ry", "Rz"],
            &pointer,
        )?;
        let id = required_text(table, "id", &pointer)?;
        if !seen.insert(id.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let node = required_u32(table, "node", &pointer)?;
        if !node_ids.contains(&node) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let values = toml_components(table, &MASS_PAIRS, &pointer)?;
        let body = json!({"id": id.clone(), "node": node, "values": values});
        masses.push(MassRec { id, body });
    }
    masses.sort_by_cached_key(|mass| order::natural_key(mass.id.as_bytes()));
    Ok(masses)
}

const MASS_PAIRS: [(&str, &str); 6] = [
    ("Ux", "Ux"),
    ("Uy", "Uy"),
    ("Uz", "Uz"),
    ("Rx", "Rx"),
    ("Ry", "Ry"),
    ("Rz", "Rz"),
];

struct SpringRec {
    id: u64,
    body: Json,
}

fn parse_springs(
    items: Option<&Vec<Toml>>,
    nodes: &[NodeRec],
    members: &[MemberRec],
    nonlinear: bool,
    laws: &BTreeSet<String>,
) -> Result<Vec<SpringRec>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    if members.len() + items.len() > MAX_MEMBERS {
        return Err(Failure::validation(
            Some("/springs".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let node_ids = node_id_set(nodes);
    let mut seen: BTreeSet<u64> = members.iter().map(|member| member.id).collect();
    let mut springs = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/springs/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(
            table,
            &[
                "id",
                "nodes",
                "kind",
                "dof",
                "k",
                "law",
                "direction",
                "rotation",
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
        let endpoints = spring_nodes(table, &pointer, &node_ids)?;
        let kind = spring_kind(table, &pointer)?;
        let dof = required_dof(table, "dof", &pointer)?;
        let (direction, rotation) = spring_axes(table, &pointer)?;
        let mut body = json!({
            "id": id,
            "nodes": endpoints,
            "kind": kind,
            "dof": dof,
        });
        if kind == "spring.linear" {
            if table.contains_key("law") {
                return Err(Failure::validation(
                    Some(pointer),
                    "A linear spring forbids a law.",
                ));
            }
            let k = match table.get("k") {
                Some(value) => nonnegative(value, &pointer)?,
                None => {
                    return Err(Failure::validation(
                        Some(pointer),
                        "A required field is missing.",
                    ))
                }
            };
            body["k"] = k;
        } else if !nonlinear {
            return Err(Failure::validation(
                Some(pointer),
                "A nonlinear spring requires nonlinearStatic.",
            ));
        } else if table.contains_key("k") {
            return Err(Failure::validation(
                Some(pointer),
                "A nonlinear spring forbids a stiffness.",
            ));
        } else {
            let law = match table.get("law") {
                Some(value) => expect_text(value, &pointer)?,
                None => {
                    return Err(Failure::validation(
                        Some(pointer),
                        "A required field is missing.",
                    ))
                }
            };
            if !laws.contains(&law) {
                return Err(Failure::validation(
                    Some(pointer),
                    "A reference does not exist.",
                ));
            }
            body["law"] = json!(law);
        }
        if let Some(direction) = direction {
            body["direction"] = direction;
        }
        if let Some(rotation) = rotation {
            body["rotation"] = rotation;
        }
        springs.push(SpringRec { id, body });
    }
    springs.sort_by_cached_key(|spring| order::natural_key(spring.id.to_string().as_bytes()));
    Ok(springs)
}

fn parse_prescribed(items: Option<&Vec<Toml>>, nodes: &[NodeRec]) -> Result<Vec<Json>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    let node_ids = node_id_set(nodes);
    let mut prescribed = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/prescribed/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(table, &["node", "dof", "value"], &pointer)?;
        let node = required_u32(table, "node", &pointer)?;
        if !node_ids.contains(&node) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let dof = required_dof(table, "dof", &pointer)?;
        let value = json_scalar(required_scalar(table, "value", &pointer)?)?;
        prescribed.push(json!({"node": node, "dof": dof, "value": value}));
    }
    Ok(prescribed)
}

fn parse_diaphragms(items: Option<&Vec<Toml>>, nodes: &[NodeRec]) -> Result<Vec<Json>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    let node_ids = node_id_set(nodes);
    let mut diaphragms = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/diaphragms/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(table, &["master", "slaves"], &pointer)?;
        let master = required_u32(table, "master", &pointer)?;
        if !node_ids.contains(&master) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let slaves = id_list(table.get("slaves"), &pointer, &node_ids)?;
        diaphragms.push(json!({"master": master, "slaves": slaves}));
    }
    Ok(diaphragms)
}

struct ParsedCases {
    bodies: Vec<Json>,
    names: BTreeSet<String>,
}

fn parse_cases(
    items: Option<&Vec<Toml>>,
    nodes: &[NodeRec],
    members: &[MemberRec],
) -> Result<ParsedCases, Failure> {
    let Some(items) = items else {
        return Ok(ParsedCases {
            bodies: Vec::new(),
            names: BTreeSet::new(),
        });
    };
    if items.len() > MAX_CASES {
        return Err(Failure::validation(
            Some("/cases".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let node_ids = node_id_set(nodes);
    let member_ids: BTreeSet<u64> = members.iter().map(|member| member.id).collect();
    let mut bodies = Vec::with_capacity(items.len());
    let mut names = BTreeSet::new();
    let mut load_count = 0usize;
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/cases/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(table, &["name", "loads", "member_loads"], &pointer)?;
        let name = required_text(table, "name", &pointer)?;
        if !names.insert(name.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let loads = parse_loads(
            array_at(table, "loads", &format!("{pointer}/loads"))?,
            &pointer,
            &node_ids,
        )?;
        let member_loads = parse_member_loads(
            array_at(table, "member_loads", &format!("{pointer}/member_loads"))?,
            &pointer,
            &member_ids,
        )?;
        load_count = load_count
            .saturating_add(loads.len())
            .saturating_add(member_loads.len());
        if load_count > MAX_LOADS {
            return Err(Failure::validation(
                Some(pointer),
                "A schema limit was exceeded.",
            ));
        }
        let mut body = json!({"name": name, "loads": loads});
        if !member_loads.is_empty() {
            body["member_loads"] = Json::Array(member_loads);
        }
        bodies.push(body);
    }
    Ok(ParsedCases { bodies, names })
}

fn parse_loads(
    items: Option<&Vec<Toml>>,
    parent: &str,
    node_ids: &BTreeSet<u64>,
) -> Result<Vec<Json>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    let mut loads = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("{parent}/loads/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(
            table,
            &["node", "Fx", "Fy", "Fz", "Mx", "My", "Mz"],
            &pointer,
        )?;
        let node = required_u32(table, "node", &pointer)?;
        if !node_ids.contains(&node) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let values = toml_components(table, &LOAD_PAIRS, &pointer)?;
        loads.push(json!({"node": node, "values": values}));
    }
    Ok(loads)
}

fn parse_member_loads(
    items: Option<&Vec<Toml>>,
    parent: &str,
    member_ids: &BTreeSet<u64>,
) -> Result<Vec<Json>, Failure> {
    let Some(items) = items else {
        return Ok(Vec::new());
    };
    let mut loads = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("{parent}/member_loads/{index}");
        let table = as_table(item, &pointer)?;
        reject_toml_keys(
            table,
            &["member", "force_per_length", "moment_per_length"],
            &pointer,
        )?;
        let member = required_u32(table, "member", &pointer)?;
        if !member_ids.contains(&member) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let force = optional_vec3(table, "force_per_length", &pointer)?;
        let moment = optional_vec3(table, "moment_per_length", &pointer)?;
        loads.push(json!({
            "member": member,
            "force_per_length": force,
            "moment_per_length": moment,
        }));
    }
    Ok(loads)
}

struct CombinationRec {
    id: String,
    body: Json,
}

fn parse_combinations(
    table: Option<&toml::Table>,
    cases: &BTreeSet<String>,
) -> Result<Vec<CombinationRec>, Failure> {
    let Some(table) = table else {
        return Ok(Vec::new());
    };
    if table.len() > MAX_COMBINATIONS {
        return Err(Failure::validation(
            Some("/combinations".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut combinations = Vec::with_capacity(table.len());
    for (index, (id, value)) in table.iter().enumerate() {
        let pointer = format!("/combinations/{index}");
        check_text_len(id, &pointer)?;
        let terms_table = as_table(value, &pointer)?;
        let mut terms = Vec::with_capacity(terms_table.len());
        for (case, factor) in terms_table {
            check_text_len(case, &pointer)?;
            if !cases.contains(case) {
                return Err(Failure::validation(
                    Some(pointer),
                    "A reference does not exist.",
                ));
            }
            let factor = json_scalar(expect_scalar(factor, &pointer)?)?;
            terms.push(json!({"case": case, "factor": factor}));
        }
        terms.sort_by(|left, right| {
            left["case"]
                .as_str()
                .unwrap_or("")
                .as_bytes()
                .cmp(right["case"].as_str().unwrap_or("").as_bytes())
        });
        combinations.push(CombinationRec {
            id: id.clone(),
            body: json!({"id": id, "terms": terms}),
        });
    }
    combinations.sort_by_cached_key(|combination| order::natural_key(combination.id.as_bytes()));
    Ok(combinations)
}

struct ModelParts {
    nodes: Vec<NodeRec>,
    members: Vec<MemberRec>,
    sections: Vec<Named>,
    laws: Vec<Named>,
    supports: Vec<SupportRec>,
    masses: Vec<MassRec>,
    springs: Vec<SpringRec>,
    prescribed: Vec<Json>,
    diaphragms: Vec<Json>,
    cases: Vec<Json>,
    combinations: Vec<CombinationRec>,
    experimental: bool,
    analysis: Json,
}

fn assemble(parts: ModelParts) -> Json {
    let ModelParts {
        nodes,
        members,
        sections,
        laws,
        supports,
        masses,
        springs,
        prescribed,
        diaphragms,
        cases,
        combinations,
        experimental,
        analysis,
    } = parts;
    let options = if experimental {
        json!({"allow_experimental": true})
    } else {
        json!({})
    };
    let mut model = json!({
        "axes": "Z-up",
        "nodes": nodes.into_iter().map(|node| node.body).collect::<Vec<_>>(),
        "members": members.into_iter().map(|member| member.body).collect::<Vec<_>>(),
        "sections": sections.into_iter().map(|section| section.body).collect::<Vec<_>>(),
        "laws": laws.into_iter().map(|law| law.body).collect::<Vec<_>>(),
        "supports": supports.into_iter().map(|support| support.body).collect::<Vec<_>>(),
        "masses": masses.into_iter().map(|mass| mass.body).collect::<Vec<_>>(),
    });
    if !springs.is_empty() {
        model["springs"] = Json::Array(springs.into_iter().map(|spring| spring.body).collect());
    }
    if !prescribed.is_empty() {
        model["prescribed"] = Json::Array(prescribed);
    }
    if !diaphragms.is_empty() {
        model["diaphragms"] = Json::Array(diaphragms);
    }
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "model": model,
        "cases": cases,
        "combinations": combinations.into_iter().map(|combination| combination.body).collect::<Vec<_>>(),
        "analyses": [analysis],
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
        Some(value) => expect_u32(value, pointer),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn expect_u32(value: &Toml, pointer: &str) -> Result<u64, Failure> {
    match value {
        Toml::Integer(number) if (0..=4_294_967_295).contains(number) => Ok(*number as u64),
        Toml::Integer(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier is outside the integer range.",
        )),
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "An identifier must be an integer.",
        )),
    }
}

fn node_id_set(nodes: &[NodeRec]) -> BTreeSet<u64> {
    nodes.iter().map(|node| node.id).collect()
}

fn toml_components(
    table: &toml::Table,
    pairs: &[(&str, &str)],
    pointer: &str,
) -> Result<Vec<Json>, Failure> {
    let mut values = Vec::new();
    for (key, dof) in pairs {
        if let Some(value) = table.get(*key) {
            let number = json_scalar(expect_scalar(value, pointer)?)?;
            values.push(json!({"dof": *dof, "value": number}));
        }
    }
    Ok(values)
}

fn array_at<'a>(
    table: &'a toml::Table,
    key: &str,
    pointer: &str,
) -> Result<Option<&'a Vec<Toml>>, Failure> {
    match table.get(key) {
        None => Ok(None),
        Some(Toml::Array(items)) => Ok(Some(items)),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A collection has the wrong shape.",
        )),
    }
}

fn required_dof(table: &toml::Table, key: &str, pointer: &str) -> Result<String, Failure> {
    match table.get(key) {
        Some(Toml::String(name)) if DOFS.contains(&name.as_str()) => Ok(name.clone()),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A degree of freedom is not recognized.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn spring_kind(table: &toml::Table, pointer: &str) -> Result<String, Failure> {
    match table.get("kind") {
        Some(Toml::String(value)) if value == "spring.linear" || value == "spring.nonlinear" => {
            Ok(value.clone())
        }
        Some(Toml::String(_)) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A spring kind is not allowed.",
        )),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn spring_nodes(
    table: &toml::Table,
    pointer: &str,
    node_ids: &BTreeSet<u64>,
) -> Result<Vec<u64>, Failure> {
    let Some(value) = table.get("nodes") else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        ));
    };
    let Toml::Array(items) = value else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if items.len() != 1 && items.len() != 2 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A spring needs one or two distinct nodes.",
        ));
    }
    let mut ids = Vec::with_capacity(items.len());
    for item in items {
        let id = expect_u32(item, pointer)?;
        if !node_ids.contains(&id) {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A reference does not exist.",
            ));
        }
        ids.push(id);
    }
    if ids.len() == 2 && ids[0] == ids[1] {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A spring needs one or two distinct nodes.",
        ));
    }
    Ok(ids)
}

fn spring_axes(
    table: &toml::Table,
    pointer: &str,
) -> Result<(Option<Json>, Option<Json>), Failure> {
    let direction = match table.get("direction") {
        None => None,
        Some(value) => Some(expect_vec3(value, pointer)?),
    };
    let rotation = match table.get("rotation") {
        None => None,
        Some(value) => Some(expect_rotation(value, pointer)?),
    };
    if direction.is_some() && rotation.is_some() {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "Direction and rotation are mutually exclusive.",
        ));
    }
    Ok((direction, rotation))
}

fn expect_rotation(value: &Toml, pointer: &str) -> Result<Json, Failure> {
    let Toml::Array(rows) = value else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if rows.len() != 3 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    }
    let mut matrix = Vec::with_capacity(3);
    for row in rows {
        matrix.push(expect_vec3(row, pointer)?);
    }
    Ok(Json::Array(matrix))
}

fn optional_vec3(table: &toml::Table, key: &str, pointer: &str) -> Result<Json, Failure> {
    match table.get(key) {
        None => Ok(json!([0, 0, 0])),
        Some(value) => expect_vec3(value, pointer),
    }
}

fn id_list(
    value: Option<&Toml>,
    pointer: &str,
    node_ids: &BTreeSet<u64>,
) -> Result<Vec<u64>, Failure> {
    let Some(value) = value else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        ));
    };
    let Toml::Array(items) = value else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    let mut ids = Vec::with_capacity(items.len());
    for item in items {
        let id = expect_u32(item, pointer)?;
        if !node_ids.contains(&id) {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A reference does not exist.",
            ));
        }
        ids.push(id);
    }
    Ok(ids)
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
    let nodes = required_array(model, "nodes", "/model/nodes")?;
    let members = required_array(model, "members", "/model/members")?;
    let sections = required_array(model, "sections", "/model/sections")?;
    let laws = required_array(model, "laws", "/model/laws")?;
    let supports = required_array(model, "supports", "/model/supports")?;
    let masses = json_items(model, "masses", "/model/masses")?;
    let springs = json_items(model, "springs", "/model/springs")?;
    let prescribed = json_items(model, "prescribed", "/model/prescribed")?;
    let diaphragms = json_items(model, "diaphragms", "/model/diaphragms")?;
    let cases = json_items(root, "cases", "/cases")?;
    let combinations = json_items(root, "combinations", "/combinations")?;
    let case_names = case_name_set(cases)?;
    let analysis = analysis::from_json(
        root,
        &case_names,
        &analysis::BranchContext {
            experimental,
            structural_empty: nodes.is_empty()
                && members.is_empty()
                && sections.is_empty()
                && laws.is_empty()
                && supports.is_empty()
                && masses.is_empty()
                && springs.is_empty()
                && prescribed.is_empty()
                && diaphragms.is_empty()
                && cases.is_empty()
                && combinations.is_empty(),
            combinations_empty: combinations.is_empty(),
            diaphragms_empty: diaphragms.is_empty(),
        },
    )?;
    let mut out = Out::default();
    out.entry("format", "\"mamut-toml/1\"");
    out.entry("units", "\"SI\"");
    write_nodes(&mut out, nodes)?;
    write_sections(&mut out, sections)?;
    write_laws(&mut out, laws)?;
    write_members(&mut out, members, nodes, sections, laws)?;
    write_supports(&mut out, supports, nodes)?;
    write_masses(&mut out, masses, nodes)?;
    write_springs(&mut out, springs, nodes, members, analysis.nonlinear, laws)?;
    write_prescribed(&mut out, prescribed, nodes)?;
    write_diaphragms(&mut out, diaphragms, nodes)?;
    let case_names = write_cases(&mut out, cases, nodes, members)?;
    write_combinations(&mut out, combinations, &case_names)?;
    analysis::write(&mut out, &analysis)?;
    if experimental {
        out.header("[options]");
        out.entry("allow_experimental", "true");
    }
    Ok(out.finish())
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

fn json_items<'a>(
    object: &'a serde_json::Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<&'a [Json], Failure> {
    match object.get(key) {
        None => Ok(&[]),
        Some(Json::Array(items)) => Ok(items),
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

fn write_masses(out: &mut Out, items: &[Json], nodes: &[Json]) -> Result<(), Failure> {
    let node_ids = node_dims(nodes)?;
    let masses = sorted_objects(items, "/model/masses", |object, pointer| {
        Ok(order::natural_key(
            json_text_field(object, "id", pointer)?.as_bytes(),
        ))
    })?;
    let mut seen = BTreeSet::new();
    for (index, mass) in masses.iter().enumerate() {
        let pointer = format!("/model/masses/{index}");
        reject_json_keys(mass, &["id", "node", "values"], &pointer)?;
        let id = json_text_field(mass, "id", &pointer)?;
        if !seen.insert(id.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let node = json_u32_field(mass, "node", &pointer)?;
        if !node_ids.contains_key(&node) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let values = json_components(
            mass.get("values").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        out.header("[[masses]]");
        out.entry("id", &quoted(&id));
        out.entry("node", &node.to_string());
        for (dof, value) in values {
            out.entry(&dof, &format_f64(value)?);
        }
    }
    Ok(())
}

fn write_springs(
    out: &mut Out,
    items: &[Json],
    nodes: &[Json],
    members: &[Json],
    nonlinear: bool,
    laws: &[Json],
) -> Result<(), Failure> {
    if members.len() + items.len() > MAX_MEMBERS {
        return Err(Failure::validation(
            Some("/model/springs".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let node_ids = node_dims(nodes)?;
    let mut seen = member_id_set(members)?;
    let law_names = if nonlinear {
        law_records(laws)?
            .into_iter()
            .map(|law| law.name)
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let springs = sorted_objects(items, "/model/springs", |object, pointer| {
        Ok(order::natural_key(json_u32_id(object, pointer)?.as_bytes()))
    })?;
    for (index, spring) in springs.iter().enumerate() {
        let pointer = format!("/model/springs/{index}");
        reject_json_keys(
            spring,
            &[
                "id",
                "nodes",
                "kind",
                "dof",
                "k",
                "law",
                "direction",
                "rotation",
            ],
            &pointer,
        )?;
        let id = json_u32_field(spring, "id", &pointer)?;
        if !seen.insert(id) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let endpoints = json_spring_nodes(spring, &pointer, &node_ids)?;
        let kind = json_spring_kind(spring, &pointer)?;
        let dof = json_one_dof(
            spring.get("dof").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        let direction = optional_json_vec3(spring, "direction", &pointer)?;
        let rotation = optional_json_rotation(spring, "rotation", &pointer)?;
        if direction.is_some() && rotation.is_some() {
            return Err(Failure::validation(
                Some(pointer),
                "Direction and rotation are mutually exclusive.",
            ));
        }
        let nonlinear_law = if kind == "spring.nonlinear" {
            if !nonlinear {
                return Err(Failure::validation(
                    Some(pointer.clone()),
                    "A nonlinear spring requires nonlinearStatic.",
                ));
            }
            if spring.contains_key("k") {
                return Err(Failure::validation(
                    Some(pointer.clone()),
                    "A nonlinear spring forbids a stiffness.",
                ));
            }
            let law = json_text_field(spring, "law", &pointer)?;
            if !law_names.contains(&law) {
                return Err(Failure::validation(
                    Some(pointer.clone()),
                    "A reference does not exist.",
                ));
            }
            Some(law)
        } else {
            if spring.contains_key("law") {
                return Err(Failure::validation(
                    Some(pointer.clone()),
                    "A linear spring forbids a law.",
                ));
            }
            None
        };
        let k = if nonlinear_law.is_none() {
            Some(match spring.get("k") {
                Some(value) => json_nonnegative(value, &pointer)?,
                None => {
                    return Err(Failure::validation(
                        Some(pointer),
                        "A required field is missing.",
                    ))
                }
            })
        } else {
            None
        };
        out.header("[[springs]]");
        out.entry("id", &id.to_string());
        out.entry("nodes", &format_id_list(&endpoints));
        out.entry("kind", &quoted(&kind));
        out.entry("dof", &quoted(&dof));
        if let Some(k) = k {
            out.entry("k", &format_f64(k)?);
        }
        if let Some(law) = nonlinear_law {
            out.entry("law", &quoted(&law));
        }
        if let Some(direction) = direction {
            out.entry("direction", &format_vec3_text(direction)?);
        }
        if let Some(rotation) = rotation {
            out.entry("rotation", &format_rotation(rotation)?);
        }
    }
    Ok(())
}

fn write_prescribed(out: &mut Out, items: &[Json], nodes: &[Json]) -> Result<(), Failure> {
    let node_ids = node_dims(nodes)?;
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/model/prescribed/{index}");
        let record = as_object(item, &pointer)?;
        reject_json_keys(record, &["node", "dof", "value"], &pointer)?;
        let node = json_u32_field(record, "node", &pointer)?;
        if !node_ids.contains_key(&node) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let dof = json_one_dof(
            record.get("dof").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
        )?;
        let value = json_f64(record.get("value").ok_or_else(|| {
            Failure::validation(Some(pointer.clone()), "A required field is missing.")
        })?)?;
        out.header("[[prescribed]]");
        out.entry("node", &node.to_string());
        out.entry("dof", &quoted(&dof));
        out.entry("value", &format_f64(value)?);
    }
    Ok(())
}

fn write_diaphragms(out: &mut Out, items: &[Json], nodes: &[Json]) -> Result<(), Failure> {
    let node_ids = node_dims(nodes)?;
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/model/diaphragms/{index}");
        let record = as_object(item, &pointer)?;
        reject_json_keys(record, &["master", "slaves"], &pointer)?;
        let master = json_u32_field(record, "master", &pointer)?;
        if !node_ids.contains_key(&master) {
            return Err(Failure::validation(
                Some(pointer),
                "A reference does not exist.",
            ));
        }
        let slaves = json_id_list(
            record.get("slaves").ok_or_else(|| {
                Failure::validation(Some(pointer.clone()), "A required field is missing.")
            })?,
            &pointer,
            &node_ids,
        )?;
        out.header("[[diaphragms]]");
        out.entry("master", &master.to_string());
        out.entry("slaves", &format_id_list(&slaves));
    }
    Ok(())
}

fn case_name_set(items: &[Json]) -> Result<BTreeSet<String>, Failure> {
    if items.len() > MAX_CASES {
        return Err(Failure::validation(
            Some("/cases".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut names = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/cases/{index}");
        let case = as_object(item, &pointer)?;
        let name = json_text_field(case, "name", &pointer)?;
        if !names.insert(name) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
    }
    Ok(names)
}

fn write_cases(
    out: &mut Out,
    items: &[Json],
    nodes: &[Json],
    members: &[Json],
) -> Result<BTreeSet<String>, Failure> {
    if items.len() > MAX_CASES {
        return Err(Failure::validation(
            Some("/cases".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let node_ids = node_dims(nodes)?;
    let member_ids = member_id_set(members)?;
    let mut names = BTreeSet::new();
    let mut load_count = 0usize;
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/cases/{index}");
        let case = as_object(item, &pointer)?;
        reject_json_keys(case, &["name", "loads", "member_loads"], &pointer)?;
        let name = json_text_field(case, "name", &pointer)?;
        if !names.insert(name.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let loads = required_array(case, "loads", &format!("{pointer}/loads"))?;
        let member_loads = json_items(case, "member_loads", &format!("{pointer}/member_loads"))?;
        load_count = load_count
            .saturating_add(loads.len())
            .saturating_add(member_loads.len());
        if load_count > MAX_LOADS {
            return Err(Failure::validation(
                Some(pointer),
                "A schema limit was exceeded.",
            ));
        }
        out.header("[[cases]]");
        out.entry("name", &quoted(&name));
        for (load_index, load) in loads.iter().enumerate() {
            let load_pointer = format!("{pointer}/loads/{load_index}");
            let load = as_object(load, &load_pointer)?;
            reject_json_keys(load, &["node", "values"], &load_pointer)?;
            let node = json_u32_field(load, "node", &load_pointer)?;
            if !node_ids.contains_key(&node) {
                return Err(Failure::validation(
                    Some(load_pointer),
                    "A reference does not exist.",
                ));
            }
            let values = json_components(
                load.get("values").ok_or_else(|| {
                    Failure::validation(Some(load_pointer.clone()), "A required field is missing.")
                })?,
                &load_pointer,
            )?;
            out.header("[[cases.loads]]");
            out.entry("node", &node.to_string());
            for (dof, value) in values {
                out.entry(force_key(&dof)?, &format_f64(value)?);
            }
        }
        for (load_index, load) in member_loads.iter().enumerate() {
            let load_pointer = format!("{pointer}/member_loads/{load_index}");
            let load = as_object(load, &load_pointer)?;
            reject_json_keys(
                load,
                &["member", "force_per_length", "moment_per_length"],
                &load_pointer,
            )?;
            let member = json_u32_field(load, "member", &load_pointer)?;
            if !member_ids.contains(&member) {
                return Err(Failure::validation(
                    Some(load_pointer),
                    "A reference does not exist.",
                ));
            }
            let force = json_vec3(
                load.get("force_per_length").ok_or_else(|| {
                    Failure::validation(Some(load_pointer.clone()), "A required field is missing.")
                })?,
                &load_pointer,
            )?;
            let moment = json_vec3(
                load.get("moment_per_length").ok_or_else(|| {
                    Failure::validation(Some(load_pointer.clone()), "A required field is missing.")
                })?,
                &load_pointer,
            )?;
            out.header("[[cases.member_loads]]");
            out.entry("member", &member.to_string());
            if !is_zero_vec(force) {
                out.entry("force_per_length", &format_vec3_text(force)?);
            }
            if !is_zero_vec(moment) {
                out.entry("moment_per_length", &format_vec3_text(moment)?);
            }
        }
    }
    Ok(names)
}

fn write_combinations(
    out: &mut Out,
    items: &[Json],
    cases: &BTreeSet<String>,
) -> Result<(), Failure> {
    if items.len() > MAX_COMBINATIONS {
        return Err(Failure::validation(
            Some("/combinations".to_string()),
            "A schema limit was exceeded.",
        ));
    }
    let mut records = Vec::with_capacity(items.len());
    let mut seen_ids = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let pointer = format!("/combinations/{index}");
        let combination = as_object(item, &pointer)?;
        reject_json_keys(combination, &["id", "terms"], &pointer)?;
        let id = json_text_field(combination, "id", &pointer)?;
        if !seen_ids.insert(id.clone()) {
            return Err(Failure::validation(
                Some(pointer),
                "An identifier is duplicated.",
            ));
        }
        let term_items = required_array(combination, "terms", &format!("{pointer}/terms"))?;
        let mut terms = Vec::with_capacity(term_items.len());
        let mut seen_cases = BTreeSet::new();
        for (term_index, term) in term_items.iter().enumerate() {
            let term_pointer = format!("{pointer}/terms/{term_index}");
            let term = as_object(term, &term_pointer)?;
            reject_json_keys(term, &["case", "factor"], &term_pointer)?;
            let case = json_text_field(term, "case", &term_pointer)?;
            if !cases.contains(&case) {
                return Err(Failure::validation(
                    Some(term_pointer),
                    "A reference does not exist.",
                ));
            }
            if !seen_cases.insert(case.clone()) {
                return Err(Failure::validation(
                    Some(term_pointer),
                    "A term is repeated.",
                ));
            }
            let factor = json_f64(term.get("factor").ok_or_else(|| {
                Failure::validation(Some(term_pointer.clone()), "A required field is missing.")
            })?)?;
            terms.push((case, factor));
        }
        terms.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        records.push((id, terms));
    }
    records.sort_by_cached_key(|(id, _)| order::natural_key(id.as_bytes()));
    for (id, terms) in records {
        out.header_named("combinations", &id);
        for (case, factor) in terms {
            out.entry(&case, &format_f64(factor)?);
        }
    }
    Ok(())
}

type SectionProperty = (String, f64);
type SectionRecord = (String, Vec<SectionProperty>);

fn section_records(items: &[Json]) -> Result<Vec<SectionRecord>, Failure> {
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

fn json_components(value: &Json, pointer: &str) -> Result<Vec<(String, f64)>, Failure> {
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A collection has the wrong shape.",
        ));
    };
    let mut present = [None; 6];
    for item in items {
        let object = as_object(item, pointer)?;
        reject_json_keys(object, &["dof", "value"], pointer)?;
        let dof = match object.get("dof") {
            Some(value) => json_one_dof(value, pointer)?,
            None => {
                return Err(Failure::validation(
                    Some(pointer.to_string()),
                    "A required field is missing.",
                ))
            }
        };
        let Some(index) = DOFS.iter().position(|name| *name == dof) else {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is not recognized.",
            ));
        };
        if present[index].is_some() {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A degree of freedom is repeated.",
            ));
        }
        let number = match object.get("value") {
            Some(value) => json_f64(value)?,
            None => {
                return Err(Failure::validation(
                    Some(pointer.to_string()),
                    "A required field is missing.",
                ))
            }
        };
        present[index] = Some(number);
    }
    Ok(DOFS
        .iter()
        .enumerate()
        .filter_map(|(index, dof)| present[index].map(|number| ((*dof).to_string(), number)))
        .collect())
}

fn json_one_dof(value: &Json, pointer: &str) -> Result<String, Failure> {
    match value.as_str() {
        Some(name) if DOFS.contains(&name) => Ok(name.to_string()),
        _ => Err(Failure::validation(
            Some(pointer.to_string()),
            "A degree of freedom is not recognized.",
        )),
    }
}

fn json_spring_kind(
    object: &serde_json::Map<String, Json>,
    pointer: &str,
) -> Result<String, Failure> {
    match object.get("kind") {
        Some(Json::String(value)) if value == "spring.linear" || value == "spring.nonlinear" => {
            Ok(value.clone())
        }
        Some(Json::String(_)) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A spring kind is not allowed.",
        )),
        Some(_) => Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        )),
        None => Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        )),
    }
}

fn json_spring_nodes(
    object: &serde_json::Map<String, Json>,
    pointer: &str,
    node_ids: &BTreeMap<u64, String>,
) -> Result<Vec<u64>, Failure> {
    let Some(value) = object.get("nodes") else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A required field is missing.",
        ));
    };
    let ids = json_id_list(value, pointer, node_ids)?;
    if (ids.len() != 1 && ids.len() != 2) || (ids.len() == 2 && ids[0] == ids[1]) {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A spring needs one or two distinct nodes.",
        ));
    }
    Ok(ids)
}

fn json_id_list(
    value: &Json,
    pointer: &str,
    node_ids: &BTreeMap<u64, String>,
) -> Result<Vec<u64>, Failure> {
    let Some(items) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    let mut ids = Vec::with_capacity(items.len());
    for item in items {
        let id = json_u32(item, pointer)?;
        if !node_ids.contains_key(&id) {
            return Err(Failure::validation(
                Some(pointer.to_string()),
                "A reference does not exist.",
            ));
        }
        ids.push(id);
    }
    Ok(ids)
}

fn optional_json_vec3(
    object: &serde_json::Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<Option<[f64; 3]>, Failure> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => Ok(Some(json_vec3(value, pointer)?)),
    }
}

fn optional_json_rotation(
    object: &serde_json::Map<String, Json>,
    key: &str,
    pointer: &str,
) -> Result<Option<[[f64; 3]; 3]>, Failure> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let Some(rows) = value.as_array() else {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    };
    if rows.len() != 3 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value has the wrong type.",
        ));
    }
    Ok(Some([
        json_vec3(&rows[0], pointer)?,
        json_vec3(&rows[1], pointer)?,
        json_vec3(&rows[2], pointer)?,
    ]))
}

fn json_nonnegative(value: &Json, pointer: &str) -> Result<f64, Failure> {
    let number = json_f64(value)?;
    if number < 0.0 {
        return Err(Failure::validation(
            Some(pointer.to_string()),
            "A value is outside the allowed range.",
        ));
    }
    Ok(number)
}

fn member_id_set(members: &[Json]) -> Result<BTreeSet<u64>, Failure> {
    let mut ids = BTreeSet::new();
    for (index, item) in members.iter().enumerate() {
        let pointer = format!("/model/members/{index}");
        let object = as_object(item, &pointer)?;
        ids.insert(json_u32_field(object, "id", &pointer)?);
    }
    Ok(ids)
}

fn force_key(dof: &str) -> Result<&'static str, Failure> {
    LOAD_PAIRS
        .iter()
        .find(|(_, mapped)| *mapped == dof)
        .map(|(key, _)| *key)
        .ok_or_else(|| Failure::validation(None, "A degree of freedom is not recognized."))
}

fn is_zero_vec(vector: [f64; 3]) -> bool {
    vector.iter().all(|item| *item == 0.0)
}

fn format_vec3_text(vector: [f64; 3]) -> Result<String, Failure> {
    Ok(format!(
        "[{}, {}, {}]",
        format_f64(vector[0])?,
        format_f64(vector[1])?,
        format_f64(vector[2])?,
    ))
}

fn format_rotation(matrix: [[f64; 3]; 3]) -> Result<String, Failure> {
    Ok(format!(
        "[{}, {}, {}]",
        format_vec3_text(matrix[0])?,
        format_vec3_text(matrix[1])?,
        format_vec3_text(matrix[2])?,
    ))
}

fn format_id_list(ids: &[u64]) -> String {
    let text = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{text}]")
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
        let accepted = convert("[[cases]]\nname = \"DL\"\n");
        assert_eq!(accepted["cases"][0]["name"], "DL");
        assert_eq!(accepted["cases"][0]["loads"], json!([]));
        assert_eq!(
            toml_to_json(
                "format = \"mamut-toml/1\"\nunits = \"SI\"\n[analysis]\nmethod = \"buckling\"\n"
            )
            .unwrap_err()
            .cause,
            "This analysis method is outside the implemented subset."
        );
        assert_eq!(
            toml_to_json(
                "format = \"mamut-toml/1\"\nunits = \"SI\"\n[analysis]\nmethod = \"pDelta\"\n"
            )
            .unwrap_err()
            .cause,
            "A required field is missing."
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
    fn missing_mass_node_does_not_echo_its_identifier() {
        let json = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[],"members":[],"sections":[],"laws":[],"supports":[],"masses":[{"id":"SENTINEL_MASS_9f3a","node":1,"values":[]}]},"cases":[],"combinations":[],"analyses":[{"method":"linearStatic"}]}"#;
        let failure = json_to_toml(json).unwrap_err();
        assert_eq!(failure.cause, "A reference does not exist.");
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
    fn empty_combinations_are_accepted_and_missing_cases_are_not() {
        assert!(toml_to_json(&minimal("[combinations]\n")).is_ok());
        let failure = failure_of("[combinations.SENTINEL_COMB_9f3a]\nDL = 1\n");
        assert_eq!(failure.cause, "A reference does not exist.");
        hidden(&failure, "SENTINEL_COMB_9f3a");
    }

    #[test]
    fn application_collections_round_trip() {
        let source = minimal(
            r#"
            [[nodes]]
            id = 2
            x = 1
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux", "Uy", "Rz"]

            [[nodes]]
            id = 1
            x = 0
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux", "Uy", "Rz"]

            [sections.S]
            A = 1

            [laws.L]
            E = 1

            [[members]]
            id = 7
            from = 1
            to = 2
            section = "S"
            law = "L"

            [[masses]]
            id = "10"
            node = 2
            Rz = 3
            Ux = 4

            [[masses]]
            id = "2"
            node = 1

            [[springs]]
            id = 10
            nodes = [2]
            kind = "spring.linear"
            dof = "Uy"
            k = 5
            direction = [0, 1, 0]

            [[springs]]
            id = 2
            nodes = [1, 2]
            kind = "spring.linear"
            dof = "Ux"
            k = 8

            [[prescribed]]
            node = 2
            dof = "Uy"
            value = 0.25

            [[prescribed]]
            node = 1
            dof = "Ux"
            value = 0

            [[diaphragms]]
            master = 2
            slaves = [1, 2]

            [[cases]]
            name = "L"

            [[cases.loads]]
            node = 2
            My = 1.5
            Fx = 9

            [[cases]]
            name = "D"

            [[cases.member_loads]]
            member = 7
            force_per_length = [0, -6, 0]

            [combinations.10]
            L = 1
            D = 2

            [combinations.2]
            D = 0.5
            "#,
        );
        let json = toml_to_json(&source).unwrap();
        let value: Json = serde_json::from_str(json.trim()).unwrap();
        assert_eq!(value["model"]["masses"][0]["id"], "2");
        assert_eq!(value["model"]["masses"][1]["id"], "10");
        assert_eq!(value["model"]["masses"][0]["values"], json!([]));
        assert_eq!(value["model"]["masses"][1]["values"][0]["dof"], "Ux");
        assert_eq!(
            value["model"]["masses"][1]["values"][1],
            json!({"dof": "Rz", "value": 3})
        );
        assert_eq!(value["model"]["springs"][0]["id"], 2);
        assert_eq!(value["model"]["springs"][0]["nodes"], json!([1, 2]));
        assert_eq!(value["model"]["springs"][1]["direction"], json!([0, 1, 0]));
        assert_eq!(value["model"]["prescribed"][0]["node"], 2);
        assert_eq!(value["model"]["prescribed"][1]["value"], 0);
        assert_eq!(value["model"]["diaphragms"][0]["slaves"], json!([1, 2]));
        assert_eq!(value["cases"][0]["name"], "L");
        assert_eq!(
            value["cases"][0]["loads"][0]["values"][0],
            json!({"dof": "Ux", "value": 9})
        );
        assert_eq!(value["cases"][0]["loads"][0]["values"][1]["dof"], "Ry");
        assert!(value["cases"][0].get("member_loads").is_none());
        assert_eq!(value["cases"][1]["name"], "D");
        assert_eq!(
            value["cases"][1]["member_loads"][0]["moment_per_length"],
            json!([0, 0, 0])
        );
        assert_eq!(value["combinations"][0]["id"], "2");
        assert_eq!(value["combinations"][1]["id"], "10");
        assert_eq!(value["combinations"][1]["terms"][0]["case"], "D");
        assert_eq!(value["combinations"][1]["terms"][1]["case"], "L");

        let toml_text = json_to_toml(&json).unwrap();
        assert_eq!(toml_text, json_to_toml(&json).unwrap());
        assert!(!toml_text.contains("moment_per_length"));
        assert!(toml_text.contains("force_per_length = [0, -6, 0]"));
        let masses_at = toml_text.find("[[masses]]").unwrap();
        let mass_2 = toml_text[masses_at..].find("id = \"2\"").unwrap();
        let mass_10 = toml_text[masses_at..].find("id = \"10\"").unwrap();
        assert!(mass_2 < mass_10);
        let springs_at = toml_text.find("[[springs]]").unwrap();
        let spring_2 = toml_text[springs_at..].find("id = 2\n").unwrap();
        let spring_10 = toml_text[springs_at..].find("id = 10\n").unwrap();
        assert!(spring_2 < spring_10);
        assert!(toml_text.find("name = \"L\"").unwrap() < toml_text.find("name = \"D\"").unwrap());
        assert!(
            toml_text.find("[combinations.2]").unwrap()
                < toml_text.find("[combinations.10]").unwrap()
        );
        let again = toml_to_json(&toml_text).unwrap();
        assert_eq!(
            engine_api::input_sha256(json.trim()).unwrap(),
            engine_api::input_sha256(again.trim()).unwrap()
        );
    }

    #[test]
    fn application_collection_boundaries_stay_closed() {
        let node = "[[nodes]]\nid = 1\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n";
        let text_spring = failure_of(&format!(
            "{node}[[springs]]\nid = \"S1\"\nnodes = [1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = 1\n"
        ));
        assert_eq!(text_spring.cause, "An identifier must be an integer.");

        let long_id = "9".repeat(4096);
        let long = failure_of(&format!("{node}[[masses]]\nid = \"{long_id}\"\nnode = 1\n"));
        assert_eq!(long.cause, "An identifier exceeds the length limit.");
        assert_eq!(long.pointer.as_deref(), Some("/masses/0"));
        assert!(toml_to_json(&minimal(&format!(
            "{node}[[masses]]\nid = {long_id}\nnode = 1\n"
        )))
        .is_err());

        let nonlinear = failure_of(&format!(
            "{node}[[springs]]\nid = 3\nnodes = [1]\nkind = \"spring.nonlinear\"\ndof = \"Ux\"\nlaw = \"L\"\n"
        ));
        assert_eq!(
            nonlinear.cause,
            "A nonlinear spring requires nonlinearStatic."
        );

        let collision = failure_of(&format!(
            "{node}[sections.S]\nA = 1\n[laws.L]\nE = 1\n[[members]]\nid = 4\nfrom = 1\nto = 1\nsection = \"S\"\nlaw = \"L\"\n[[springs]]\nid = 4\nnodes = [1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = 1\n"
        ));
        assert_eq!(collision.cause, "An identifier is duplicated.");

        let both_axes = failure_of(&format!(
            "{node}[[springs]]\nid = 3\nnodes = [1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = 1\ndirection = [1, 0, 0]\nrotation = [[1, 0, 0], [0, 1, 0], [0, 0, 1]]\n"
        ));
        assert_eq!(
            both_axes.cause,
            "Direction and rotation are mutually exclusive."
        );

        let with_law = failure_of(&format!(
            "{node}[[springs]]\nid = 3\nnodes = [1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = 1\nlaw = \"L\"\n"
        ));
        assert_eq!(with_law.cause, "A linear spring forbids a law.");

        let unknown = failure_of(&format!(
            "{node}[[masses]]\nid = \"M\"\nnode = 1\nSENTINEL_MASSKEY_9f3a = 1\n"
        ));
        assert_eq!(unknown.cause, "The document contains an unknown key.");
        hidden(&unknown, "SENTINEL_MASSKEY_9f3a");

        let repeated = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[{"id":1,"dim":"D2","coordinates":[0,0,0],"dofs":["Ux"]}],"members":[],"sections":[],"laws":[],"supports":[],"masses":[]},"cases":[{"name":"D","loads":[]}],"combinations":[{"id":"C","terms":[{"case":"D","factor":1},{"case":"D","factor":2}]}],"analyses":[{"method":"linearStatic"}]}"#;
        assert_eq!(
            json_to_toml(repeated).unwrap_err().cause,
            "A term is repeated."
        );

        let support = failure_of(&format!("{node}[[supports]]\nnode = 1\ntype = \"pin\"\n"));
        assert_eq!(support.cause, "Support type expansion is not decided.");

        let same = failure_of(&format!(
            "{node}[[springs]]\nid = 3\nnodes = [1, 1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = 1\n"
        ));
        assert_eq!(same.cause, "A spring needs one or two distinct nodes.");

        let negative = failure_of(&format!(
            "{node}[[springs]]\nid = 3\nnodes = [1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = -1\n"
        ));
        assert_eq!(negative.cause, "A value is outside the allowed range.");

        let name = "m".repeat(256);
        let accepted = convert(&format!(
            "{node}[[masses]]\nid = \"{name}\"\nnode = 1\nUx = 1\n"
        ));
        assert_eq!(accepted["model"]["masses"][0]["id"], name);
        assert_eq!(
            accepted["model"]["masses"][0]["values"],
            json!([{"dof": "Ux", "value": 1}])
        );

        let rotation = convert(&format!(
            "{node}[[springs]]\nid = 3\nnodes = [1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = 2\nrotation = [[1, 0, 0], [0, 1, 0], [0, 0, 1]]\n"
        ));
        assert_eq!(
            rotation["model"]["springs"][0]["rotation"],
            json!([[1, 0, 0], [0, 1, 0], [0, 0, 1]])
        );
        let exported = json_to_toml(&serde_json::to_string(&rotation).unwrap()).unwrap();
        assert!(exported.contains("rotation = [[1, 0, 0], [0, 1, 0], [0, 0, 1]]"));
        assert!(!exported.contains("direction"));
    }

    fn round_trip(source: &str) -> (Json, String) {
        let json = toml_to_json(source).unwrap_or_else(|error| panic!("{}", error.cause));
        let toml = json_to_toml(&json).unwrap_or_else(|error| panic!("{}", error.cause));
        assert_eq!(json_to_toml(&json).unwrap(), toml);
        let again = toml_to_json(&toml).unwrap();
        assert_eq!(
            engine_api::input_sha256(json.trim()).unwrap(),
            engine_api::input_sha256(again.trim()).unwrap()
        );
        (serde_json::from_str(json.trim()).unwrap(), toml)
    }

    #[test]
    fn analysis_branches_round_trip_without_invented_values() {
        let (p_delta, p_delta_toml) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [[nodes]]
            id = 1
            x = 0
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux"]
            [[cases]]
            name = "G"
            [[cases]]
            name = "H"
            [analysis]
            method = "pDelta"
            tolerance = 1e-10
            max_iterations = 100
            displacement_scale = 1
            rotation_scale = 2
            [analysis.gravity]
            case = "G"
            factor = 1
            [analysis.lateral]
            case = "H"
            factor = -0.5
            "#,
        );
        assert_eq!(p_delta["analyses"][0]["method"], "pDelta");
        assert_eq!(p_delta["analyses"][0]["options"]["tolerance"], 1e-10);
        assert_eq!(p_delta["analyses"][0]["options"]["lateral"]["factor"], -0.5);
        assert!(p_delta_toml.contains("[analysis.gravity]"));
        assert!(!p_delta_toml.contains("mass_source"));

        let (modal, modal_toml) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [[cases]]
            name = "D"
            [analysis]
            method = "modal"
            mode_count = 2
            mass_kind = "consistent"
            directions = [[1, 0, 0], [0, 1, 0]]
            [analysis.mass_source]
            method = "masses_from_case"
            case = "D"
            factor = 1
            direction = [0, 0, -1]
            gravity = 9.80665
            "#,
        );
        assert_eq!(modal["analyses"][0]["options"]["directions"][1][1], 1);
        assert_eq!(
            modal["analyses"][0]["options"]["mass_source"]["gravity"],
            9.80665
        );
        assert!(modal_toml.find("directions").unwrap() < modal_toml.find("mass_source").unwrap());
        let (bare_modal, bare_modal_toml) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [analysis]
            method = "modal"
            mode_count = 1
            mass_kind = "lumped"
            directions = [[1, 0, 0]]
            "#,
        );
        assert!(bare_modal["analyses"][0]["options"]
            .get("mass_source")
            .is_none());
        assert!(!bare_modal_toml.contains("mass_source"));

        let (spectrum, spectrum_toml) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [analysis]
            method = "responseSpectrum"
            mode_count = 1
            mass_kind = "lumped"
            modal_combination = "cqc"
            directional_combination = "weightedSum"
            directional_factors = [1, 0]
            [[analysis.directions]]
            direction = [0, 1, 0]
            [[analysis.directions.spectra]]
            pairs = [[0, 3], [10, 3]]
            units = "periodSecondsAccelerationMetersPerSecondSquared"
            damping = 0.05
            interpolation = "logLog"
            extrapolation = "plateau"
            [[analysis.directions]]
            direction = [1, 0, 0]
            [[analysis.directions.spectra]]
            pairs = [[1, 2], [2, 1]]
            units = "frequencyHertzAccelerationMetersPerSecondSquared"
            damping = 0
            interpolation = "linear"
            extrapolation = "error"
            "#,
        );
        assert_eq!(
            spectrum["analyses"][0]["options"]["directions"][0]["direction"][1],
            1
        );
        assert_eq!(
            spectrum["analyses"][0]["options"]["directions"][1]["spectra"][0]["units"],
            "frequencyHertzAccelerationMetersPerSecondSquared"
        );
        let first = spectrum_toml.find("direction = [0, 1, 0]").unwrap();
        let second = spectrum_toml.find("direction = [1, 0, 0]").unwrap();
        assert!(first < second);

        let (newton, newton_toml) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [options]
            allow_experimental = true
            [[nodes]]
            id = 1
            x = 0
            y = 0
            z = 0
            dim = "D2"
            dofs = ["Ux"]
            [laws.L]
            kind = "elastic"
            E = 1
            [[cases]]
            name = "P"
            [[springs]]
            id = 3
            nodes = [1]
            kind = "spring.nonlinear"
            dof = "Ux"
            law = "L"
            [analysis]
            method = "nonlinearStatic"
            case = "P"
            load_factors = [1, -0.5, 1]
            tolerance = 1e-8
            max_iterations = 30
            force_scale = 16
            displacement_scale = 2
            displacement_tolerance = 1e-6
            "#,
        );
        assert_eq!(newton["model"]["springs"][0]["kind"], "spring.nonlinear");
        assert_eq!(newton["model"]["springs"][0]["law"], "L");
        assert!(newton["model"]["springs"][0].get("k").is_none());
        assert_eq!(
            newton["analyses"][0]["options"]["load_factors"],
            json!([1, -0.5, 1])
        );
        assert!(newton["analyses"][0]["options"]
            .get("energy_scale")
            .is_none());
        assert!(newton_toml.contains("law = \"L\""));
        assert!(!newton_toml.contains("\nk = "));

        let (soil, soil_toml) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [options]
            allow_experimental = true
            [analysis]
            method = "soilProfile"
            depths_m = [0, 1]
            [[analysis.layers]]
            thickness_m = 2
            "#,
        );
        assert!(soil["analyses"][0]["options"]["layers"][0]
            .get("gamma_kn_m3")
            .is_none());
        assert!(soil["analyses"][0]["options"]
            .get("groundwater_depth_m")
            .is_none());
        assert!(!soil_toml.contains("gamma_"));
        let (soil_full, _) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [options]
            allow_experimental = true
            [analysis]
            method = "soilProfile"
            depths_m = [0, 5]
            groundwater_depth_m = { value = 1, provenance = "measured" }
            water_unit_weight_kn_m3 = { value = 10, provenance = "assumed" }
            [[analysis.layers]]
            thickness_m = 2
            gamma_kn_m3 = { value = 18, provenance = "measured" }
            gamma_sat_kn_m3 = { value = 20, provenance = "measured" }
            [[analysis.layers]]
            thickness_m = 3
            gamma_kn_m3 = { value = 19, provenance = "assumed" }
            "#,
        );
        assert_eq!(
            soil_full["analyses"][0]["options"]["layers"][1]["thickness_m"],
            3
        );
        assert_eq!(
            soil_full["analyses"][0]["options"]["layers"][1]["gamma_kn_m3"]["provenance"],
            "assumed"
        );

        let (consolidation, _) = round_trip(
            r#"
            format = "mamut-toml/1"
            units = "SI"
            [options]
            allow_experimental = true
            [analysis]
            method = "consolidation1d"
            drainage = "double"
            output_times_s = [0, 10]
            theta = 1
            max_step_s = 0.05
            [[analysis.layers]]
            thickness_m = { value = 0.5, provenance = "measured" }
            elements = 4
            [[analysis.history]]
            time_s = 0
            [[analysis.history]]
            time_s = 10
            delta_stress_kpa = { value = 20, provenance = "assumed" }
            "#,
        );
        let layer = &consolidation["analyses"][0]["options"]["layers"][0];
        assert!(layer.get("cv_m2_s").is_none());
        assert!(consolidation["analyses"][0]["options"]
            .get("water_unit_weight_kn_m3")
            .is_none());
        assert!(consolidation["analyses"][0]["options"]["history"][0]
            .get("delta_stress_kpa")
            .is_none());
        assert_eq!(
            consolidation["analyses"][0]["options"]["history"][1]["time_s"],
            10
        );
    }

    #[test]
    fn analysis_branch_rules_stay_closed() {
        let base = "format = \"mamut-toml/1\"\nunits = \"SI\"\n";
        let cause = |body: &str| toml_to_json(&format!("{base}{body}")).unwrap_err().cause;
        assert_eq!(
            cause("[analysis]\nmethod = \"SENTINEL_METHOD_9f3a\"\n"),
            "This analysis method is outside the implemented subset."
        );
        let sentinel = toml_to_json(&format!(
            "{base}[analysis]\nmethod = \"SENTINEL_METHOD_9f3a\"\n"
        ))
        .unwrap_err();
        hidden(&sentinel, "SENTINEL_METHOD_9f3a");
        assert_eq!(
            cause("[analysis]\nmethod = \"linearStatic\"\ntolerance = 1e-6\n"),
            "The document contains an unknown key."
        );
        assert_eq!(
            cause("[analysis]\nmethod = \"pDelta\"\noptions = { tolerance = 1e-6 }\n"),
            "The document contains an unknown key."
        );
        assert_eq!(
            cause("[analysis]\nmethod = \"modal\"\nmode_count = 1\nmass_kind = \"lumped\"\ndirections = [[1, 0, 0]]\nmodal_combination = \"srss\"\n"),
            "The document contains an unknown key."
        );
        assert_eq!(
            cause("[options]\nallow_experimental = false\n[analysis]\nmethod = \"soilProfile\"\ndepths_m = [0]\n[[analysis.layers]]\nthickness_m = 1\n"),
            "An experimental method requires allow_experimental."
        );
        assert_eq!(
            cause("[options]\nallow_experimental = true\n[[nodes]]\nid = 1\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n[analysis]\nmethod = \"soilProfile\"\ndepths_m = [0]\n[[analysis.layers]]\nthickness_m = 1\n"),
            "A geotechnical analysis requires an empty structural model."
        );
        assert_eq!(
            cause("[options]\nallow_experimental = true\n[[cases]]\nname = \"P\"\n[combinations.C]\nP = 1\n[analysis]\nmethod = \"nonlinearStatic\"\ncase = \"P\"\nload_factors = [1]\ntolerance = 1e-8\nmax_iterations = 2\nforce_scale = 1\n"),
            "Nonlinear static does not allow combinations or diaphragms."
        );
        assert_eq!(
            cause("[[nodes]]\nid = 1\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n[[diaphragms]]\nmaster = 1\nslaves = [1]\n[options]\nallow_experimental = true\n[[cases]]\nname = \"P\"\n[analysis]\nmethod = \"nonlinearStatic\"\ncase = \"P\"\nload_factors = [1]\ntolerance = 1e-8\nmax_iterations = 2\nforce_scale = 1\n"),
            "Nonlinear static does not allow combinations or diaphragms."
        );
        assert_eq!(
            cause("[options]\nallow_experimental = true\n[[nodes]]\nid = 1\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n[[cases]]\nname = \"P\"\n[[springs]]\nid = 3\nnodes = [1]\nkind = \"spring.nonlinear\"\ndof = \"Ux\"\nk = 1\nlaw = \"L\"\n[analysis]\nmethod = \"nonlinearStatic\"\ncase = \"P\"\nload_factors = [1]\ntolerance = 1e-8\nmax_iterations = 2\nforce_scale = 1\n"),
            "A nonlinear spring forbids a stiffness."
        );
        assert_eq!(
            cause("[analysis]\nmethod = \"pDelta\"\ngravity = { case = \"G\", factor = 1 }\nlateral = { case = \"G\", factor = 1 }\ntolerance = 1e-6\nmax_iterations = 1\ndisplacement_scale = 1\nrotation_scale = 1\n"),
            "A reference does not exist."
        );
        assert_eq!(
            cause("[[cases]]\nname = \"G\"\n[analysis]\nmethod = \"pDelta\"\ngravity = { case = \"G\" }\nlateral = { case = \"G\", factor = 1 }\ntolerance = 1e-6\nmax_iterations = 1\ndisplacement_scale = 1\nrotation_scale = 1\n"),
            "A required field is missing."
        );
        assert_eq!(
            cause("[options]\nallow_experimental = true\n[analysis]\nmethod = \"consolidation1d\"\ndrainage = \"top\"\noutput_times_s = [0]\ntheta = 1\nmax_step_s = 1\n[[analysis.layers]]\nthickness_m = { value = 1, provenance = \"measured\" }\nelements = 1\n[[analysis.history]]\ntime_s = 1\n[[analysis.history]]\ntime_s = 2\n"),
            "A value is outside the allowed range."
        );
        assert_eq!(
            cause("[options]\nallow_experimental = true\n[analysis]\nmethod = \"soilProfile\"\ndepths_m = [0]\n[[analysis.layers]]\nthickness_m = 1\ngamma_kn_m3 = 18\n"),
            "A value must be a table."
        );
        let two = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[],"members":[],"sections":[],"laws":[],"supports":[],"masses":[]},"cases":[],"combinations":[],"analyses":[{"method":"linearStatic"},{"method":"linearStatic"}]}"#;
        assert_eq!(
            json_to_toml(two).unwrap_err().cause,
            "A required field is missing."
        );
        let other = r#"{"schema":"engine-input/1","units":"SI","options":{},"model":{"axes":"Z-up","nodes":[],"members":[],"sections":[],"laws":[],"supports":[],"masses":[]},"cases":[{"name":"G","loads":[]}],"combinations":[],"analyses":[{"method":"pDelta","options":{"gravity":{"case":"G","factor":1},"lateral":{"case":"G","factor":1},"tolerance":1e-6,"max_iterations":1,"displacement_scale":1,"rotation_scale":1,"mode_count":1}}]}"#;
        assert_eq!(
            json_to_toml(other).unwrap_err().cause,
            "The document contains an unknown key."
        );
    }

    #[test]
    fn published_analysis_fixtures_keep_their_branch() {
        for name in [
            "p-delta.json",
            "modal-single-dof.json",
            "response-spectrum.json",
            "sfc010-soil.json",
            "sfc010-consolidation.json",
            "sfc010-cubic.json",
            "sfc010-spring3d.json",
        ] {
            let path = format!(
                "{}/../engine-api/fixtures/input/valid/{name}",
                env!("CARGO_MANIFEST_DIR")
            );
            let original =
                std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{name}: {error}"));
            let toml =
                json_to_toml(&original).unwrap_or_else(|error| panic!("{name}: {}", error.cause));
            let json =
                toml_to_json(&toml).unwrap_or_else(|error| panic!("{name} back: {}", error.cause));
            assert_eq!(json_to_toml(&json).unwrap(), toml, "{name}");
            let source: Json = serde_json::from_str(&original).unwrap();
            let back: Json = serde_json::from_str(json.trim()).unwrap();
            assert_eq!(back["analyses"], source["analyses"], "{name}");
            assert_eq!(back["options"], source["options"], "{name}");
        }
    }
}
