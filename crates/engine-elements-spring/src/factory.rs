//! element/1 factory with an internal SI schema, without a parser or global registry.
use crate::{finite, invalid, LinearSpring, SpringAxes, SpringConnection};
use engine_contracts::{Element, ElementFactory, JsonValue, ModelView};
use engine_core::{Dim, EResult, EngineError, Real};
use std::collections::BTreeSet;

/// spring.linear factory for the selected dimension.
/// k and dofs are required; ground=false, without direction/rotation uses global axes.
/// dofs must contain exactly one DOF of the selected dimension.
/// Empty lists or lists with more DOFs than the dimension are rejected; multiple distinct
/// valid DOFs are rejected with ENG-E-0013 to avoid coupling.
/// Direction requires dofs=["Ux"]; dofs expresses LOCAL components.
/// Validates ModelView as in truss/frame; the product retains no references.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpringFactory {
    dim: Dim,
}
impl SpringFactory {
    /// Selects D2 (XY) or D3 to validate model, DOFs, and orientation.
    pub fn new(dim: Dim) -> Self {
        Self { dim }
    }
}
impl ElementFactory for SpringFactory {
    fn kind(&self) -> &'static str {
        "spring.linear"
    }
    fn params_schema(&self) -> String {
        let dofs = match self.dim {
            Dim::D2 => r#"["Ux","Uy","Rz"]"#,
            Dim::D3 => r#"["Ux","Uy","Uz","Rx","Ry","Rz"]"#,
        };
        // Additional physical constraints (family, plane, orthonormality)
        // are documented and validated on construction, as geometry is in frame.
        format!(
            r##"{{"type":"object","properties":{{"k":{{"type":"number","minimum":0,"description":"Rigidez SI: N/m para traslaciones, N·m/rad para giros"}},"dofs":{{"type":"array","minItems":1,"maxItems":1,"uniqueItems":true,"items":{{"enum":{dofs}}},"description":"Un único GDL local (varios GDL en un elemento serían un resorte de rango uno que los acopla; la fábrica lo rechaza)"}},"ground":{{"type":"boolean","default":false}},"direction":{{"$ref":"#/$defs/vector","description":"Dirección global finita no nula, solo Ux local"}},"rotation":{{"type":"array","minItems":3,"maxItems":3,"items":{{"$ref":"#/$defs/vector"}},"description":"Filas: ejes locales ortonormales, determinante +1; D2 conserva XY y +Z"}}}},"required":["k","dofs"],"additionalProperties":false,"not":{{"required":["direction","rotation"]}},"$defs":{{"vector":{{"type":"array","minItems":3,"maxItems":3,"items":{{"type":"number"}}}}}}}}"##
        )
    }
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>> {
        if self.dim != model.dim {
            return Err(invalid("Spring factory: incompatible dimension."));
        }
        let mut ids = BTreeSet::new();
        for (id, p) in model.nodes {
            if !ids.insert(*id) {
                return Err(EngineError::new(
                    "ENG-E-0002",
                    "ModelView: id nodal duplicado.",
                ));
            }
            for &value in p {
                finite(value, "ModelView coordinate")?;
            }
            if self.dim == Dim::D2 && p[2] != 0.0 {
                return Err(invalid("ModelView D2 requires Z=0."));
            }
        }
        let JsonValue::Object(fields) = params else {
            return Err(invalid("Spring: expected an SI object."));
        };
        if fields
            .keys()
            .any(|key| !["k", "dofs", "ground", "direction", "rotation"].contains(&key.as_str()))
        {
            return Err(invalid("Spring: unknown property."));
        }
        let k = match fields.get("k") {
            Some(JsonValue::Number(v)) => *v,
            _ => return Err(invalid("Spring: numeric k is required.")),
        };
        let JsonValue::Array(values) = fields
            .get("dofs")
            .ok_or_else(|| invalid("Spring: dofs is required."))?
        else {
            return Err(invalid("Spring: dofs must be a list."));
        };
        if values.is_empty() || values.len() > 6 {
            return Err(invalid(
                "Spring: factory requires exactly one DOF of the selected dimension.",
            ));
        }
        let mut dofs = Vec::with_capacity(values.len());
        for value in values {
            let JsonValue::String(label) = value else {
                return Err(invalid("DOF must be text."));
            };
            let dof = self
                .dim
                .dofs()
                .iter()
                .find(|dof| dof.label() == label)
                .copied()
                .ok_or_else(|| invalid("Unknown DOF or DOF outside the dimension."))?;
            dofs.push(dof);
        }
        if dofs.iter().enumerate().any(|(i, d)| dofs[..i].contains(d)) {
            return Err(invalid("Spring: duplicate DOF."));
        }
        if dofs.len() > 1 {
            // Multiple DOFs in one element form a rank-one spring that
            // COUPLES the DOFs (k·bbᵀ), not independent springs: the factory
            // rejects this to avoid unrequested coupled stiffnesses (J4 r7).
            return Err(EngineError::new(
                "ENG-E-0013",
                "Spring: an element supports a single DOF; for independent springs, use one element per DOF.",
            ));
        }
        let connection = match fields.get("ground") {
            None | Some(JsonValue::Bool(false)) => SpringConnection::BetweenNodes,
            Some(JsonValue::Bool(true)) => SpringConnection::Ground,
            _ => return Err(invalid("ground must be boolean.")),
        };
        let axes = match (fields.get("direction"), fields.get("rotation")) {
            (None, None) => SpringAxes::Global,
            (Some(value), None) => SpringAxes::Direction(array(value)?),
            (None, Some(JsonValue::Array(rows))) if rows.len() == 3 => {
                SpringAxes::Rotation([array(&rows[0])?, array(&rows[1])?, array(&rows[2])?])
            }
            _ => {
                return Err(invalid(
                    "Spring: incompatible orientation or invalid matrix size.",
                ))
            }
        };
        Ok(Box::new(LinearSpring::new(
            self.dim, connection, k, &dofs, axes,
        )?))
    }
}

fn array(value: &JsonValue) -> EResult<[Real; 3]> {
    let JsonValue::Array(values) = value else {
        return Err(invalid("Orientation requires a numeric list."));
    };
    if values.len() != 3 {
        return Err(invalid("Orientation requires three components."));
    }
    let mut out = [0.0; 3];
    for (target, value) in out.iter_mut().zip(values) {
        let JsonValue::Number(v) = value else {
            return Err(invalid("Orientation requires numbers."));
        };
        *target = finite(*v, "Orientation")?;
    }
    Ok(out)
}
