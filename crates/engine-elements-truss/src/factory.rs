//! element/1 factory without a parser, material references, or global registry.
use crate::{finite, invalid, LinearTruss};
use engine_contracts::{Element, ElementFactory, JsonValue, ModelView};
use engine_core::{Dim, EResult, EngineError};
use std::collections::BTreeSet;

/// Factory for a truss kind; validates the SI E, A, and rho schema.
///
/// Receives no ids or connections: each truss geometry arrives in ElementCtx.
/// ModelView is checked for dimension, unique ids, and finite/planar coordinates;
/// its lifetime does not constrain the product's. Does not modify or assemble the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrussFactory {
    dim: Dim,
}
impl TrussFactory {
    /// Selects truss2d.linear for D2 or truss3d.linear for D3.
    pub fn new(dim: Dim) -> Self {
        Self { dim }
    }
}
impl ElementFactory for TrussFactory {
    fn kind(&self) -> &'static str {
        match self.dim {
            Dim::D2 => "truss2d.linear",
            Dim::D3 => "truss3d.linear",
        }
    }
    fn params_schema(&self) -> String {
        r#"{"type":"object","properties":{"E":{"type":"number","exclusiveMinimum":0,"description":"Módulo de Young (Pa)"},"A":{"type":"number","exclusiveMinimum":0,"description":"Área (m²)"},"rho":{"type":"number","minimum":0,"description":"Densidad (kg/m³)"}},"required":["E","A","rho"],"additionalProperties":false}"#.into()
    }
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>> {
        if self.dim != model.dim {
            return Err(invalid("Truss factory: incompatible dimension."));
        }
        let mut ids = BTreeSet::new();
        for (id, p) in model.nodes {
            if !ids.insert(*id) {
                return Err(EngineError::new(
                    "ENG-E-0002",
                    "ModelView: id nodal duplicado.",
                ));
            }
            for &v in p {
                finite(v, "ModelView coordinate (m)")?;
            }
            if model.dim == Dim::D2 && p[2] != 0.0 {
                return Err(invalid("ModelView D2 requires Z=0."));
            }
        }
        let JsonValue::Object(fields) = params else {
            return Err(invalid("Truss: expected an E, A, rho object."));
        };
        if fields.len() != 3 {
            return Err(invalid("Truss: only E, A, rho are required."));
        }
        let number = |key| match fields.get(key) {
            Some(JsonValue::Number(value)) => Ok(*value),
            _ => Err(invalid(&format!(
                "Truss: {key} must be an explicit SI number."
            ))),
        };
        Ok(Box::new(LinearTruss::new(
            self.dim,
            number("E")?,
            number("A")?,
            number("rho")?,
        )?))
    }
}
