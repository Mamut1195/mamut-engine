//! Queries without recomputing the analysis or modifying the original document.
use crate::{ApiError, Outcome};
use engine_core::{MemberId, NodeId};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Entity selection and field projection for an `Outcome`.
///
/// `None` selects everything; `Some(&[])` selects nothing. IDs apply to
/// identities, cases, combinations, mode shapes, spectrum responses, and
/// envelopes. Preserves order and geometric references, diagnostics,
/// provenance, and aggregate quantities (e.g. base shear/equivalent forces).
/// Does not recompute those quantities for the selected subset.
#[derive(Clone, Copy, Debug, Default)]
pub struct ResultQuery<'a> {
    /// Node IDs, also applied to `modes[].shape`.
    pub node_ids: Option<&'a [NodeId]>,
    /// Member IDs, independent of node selection.
    pub member_ids: Option<&'a [MemberId]>,
    /// Dot-separated paths, for example `cases.nodes.displacements` or
    /// `cases.members.stations.forces_local`. They traverse arrays without indices;
    /// selecting a branch includes its descendants. Preserves `id` in projected
    /// objects. Absent paths contribute no fields; malformed paths cause an error.
    /// A projection is a JSON fragment, without any guarantee of satisfying results/1.
    pub fields: Option<&'a [&'a str]>,
}

impl Outcome {
    /// Queries the already computed document; the original and its fingerprint do not change.
    /// Without field projection, returns the full envelope with filtered entities.
    /// Does not apply input limits to the output, which may be larger.
    pub fn query(&self, query: &ResultQuery<'_>) -> Result<Value, ApiError> {
        let mut value: Value = serde_json::from_str(&self.document)
            .map_err(|e| ApiError::InvalidInput(format!("results/1: {e}")))?;
        if value.get("schema").and_then(Value::as_str) != Some("results/1") {
            return Err(ApiError::UnsupportedSchema("results/1 is required".into()));
        }
        let node_ids = query
            .node_ids
            .map(|ids| ids.iter().map(|id| u64::from(id.0)).collect());
        let member_ids = query
            .member_ids
            .map(|ids| ids.iter().map(|id| u64::from(id.0)).collect());
        filter(&mut value, &node_ids, &member_ids);
        let Some(fields) = query.fields else {
            return Ok(value);
        };
        let mut selection = Selection::default();
        for field in fields {
            let mut branch = &mut selection;
            for part in field.split('.') {
                let mut bytes = part.bytes();
                if !bytes.next().is_some_and(|b| b.is_ascii_lowercase())
                    || !bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
                {
                    return Err(ApiError::InvalidInput(format!(
                        "Invalid field path: {field}"
                    )));
                }
                branch = branch.children.entry(part.to_owned()).or_default();
            }
            branch.complete = true;
        }
        Ok(project(&value, &selection))
    }
}

fn filter(value: &mut Value, node_ids: &Option<BTreeSet<u64>>, member_ids: &Option<BTreeSet<u64>>) {
    match value {
        Value::Object(object) => {
            for (name, child) in object {
                if let Value::Array(items) = child {
                    match name.as_str() {
                        "nodes" | "shape" => {
                            // members[].nodes are endpoint IDs, not response rows.
                            if let Some(ids) = node_ids
                                .as_ref()
                                .filter(|_| items.iter().all(Value::is_object))
                            {
                                items.retain(|item| {
                                    item.get("id")
                                        .and_then(Value::as_u64)
                                        .is_some_and(|id| ids.contains(&id))
                                });
                            }
                        }
                        "members" => {
                            if let Some(ids) = member_ids {
                                items.retain(|item| {
                                    item.get("id")
                                        .and_then(Value::as_u64)
                                        .is_some_and(|id| ids.contains(&id))
                                });
                            }
                        }
                        _ => {}
                    }
                }
                filter(child, node_ids, member_ids);
            }
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| filter(item, node_ids, member_ids)),
        _ => {}
    }
}

#[derive(Default)]
struct Selection {
    complete: bool,
    children: BTreeMap<String, Selection>,
}

fn project(value: &Value, selection: &Selection) -> Value {
    if selection.complete {
        return value.clone();
    }
    match value {
        Value::Object(object) => {
            let mut result = Map::new();
            for (name, branch) in &selection.children {
                if let Some(child) = object.get(name) {
                    result.insert(name.clone(), project(child, branch));
                }
            }
            if let Some(id) = object.get("id") {
                result.insert("id".into(), id.clone());
            }
            Value::Object(result)
        }
        Value::Array(items) => {
            Value::Array(items.iter().map(|item| project(item, selection)).collect())
        }
        _ => Value::Null,
    }
}
