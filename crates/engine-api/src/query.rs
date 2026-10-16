//! Consultas sin recalcular el análisis ni modificar el documento original.
use crate::{ApiError, Outcome};
use engine_core::{MemberId, NodeId};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Selección de entidades y proyección de campos de un `Outcome`.
///
/// `None` selecciona todo; `Some(&[])` no selecciona nada. Los IDs se aplican a
/// identidades, casos, combinaciones, formas modales, respuestas espectrales y
/// envolventes. Conserva orden y referencias geométricas, diagnósticos,
/// procedencia y magnitudes agregadas (p. ej. cortante basal/fuerzas equivalentes).
/// No recalcula esas magnitudes para el subconjunto seleccionado.
#[derive(Clone, Copy, Debug, Default)]
pub struct ResultQuery<'a> {
    /// IDs de nodos, aplicados también a `modes[].shape`.
    pub node_ids: Option<&'a [NodeId]>,
    /// IDs de miembros, independientes de la selección de nodos.
    pub member_ids: Option<&'a [MemberId]>,
    /// Rutas separadas por puntos, por ejemplo `cases.nodes.displacements` o
    /// `cases.members.stations.forces_local`. Atraviesan arrays sin índices;
    /// seleccionar una rama incluye sus descendientes. Conserva `id` en objetos
    /// proyectados. Rutas ausentes no aportan campos; rutas mal formadas dan error.
    /// Una proyección es un fragmento JSON, sin garantía de satisfacer results/1.
    pub fields: Option<&'a [&'a str]>,
}

impl Outcome {
    /// Consulta el documento ya calculado; el original y su huella no cambian.
    /// Sin proyección de campos devuelve el sobre completo con entidades filtradas.
    /// No aplica los límites de entrada a la salida, que puede ser más grande.
    pub fn query(&self, query: &ResultQuery<'_>) -> Result<Value, ApiError> {
        let mut value: Value = serde_json::from_str(&self.document)
            .map_err(|e| ApiError::InvalidInput(format!("results/1: {e}")))?;
        if value.get("schema").and_then(Value::as_str) != Some("results/1") {
            return Err(ApiError::UnsupportedSchema("Se requiere results/1".into()));
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
                        "Ruta de campo inválida: {field}"
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
                            // members[].nodes son IDs de extremos, no filas de respuesta.
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
