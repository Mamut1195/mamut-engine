//! Fábrica element/1 con esquema SI propio, sin parser ni registro global.
use crate::{finite, invalid, LinearSpring, SpringAxes, SpringConnection};
use engine_contracts::{Element, ElementFactory, JsonValue, ModelView};
use engine_core::{Dim, EResult, EngineError, Real};
use std::collections::BTreeSet;

/// Fábrica de spring.linear para la dimensión seleccionada.
/// k y dofs son obligatorios; ground=false, sin dirección/rotación usa globales.
/// dofs debe contener exactamente un GDL de la dimensión seleccionada.
/// Las listas vacías o con más GDL que la dimensión se rechazan; varios GDL
/// distintos válidos se rechazan con ENG-E-0013 para evitar acoplamientos.
/// La dirección requiere dofs=["Ux"]; dofs expresa componentes LOCALES.
/// Valida ModelView como truss/frame; el producto no conserva referencias.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpringFactory {
    dim: Dim,
}
impl SpringFactory {
    /// Selecciona D2 (XY) o D3 para validar modelo, GDL y orientación.
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
        // Las restricciones físicas adicionales (familia, plano, ortonormalidad)
        // se documentan y validan al construir, igual que la geometría en frame.
        format!(
            r##"{{"type":"object","properties":{{"k":{{"type":"number","minimum":0,"description":"Rigidez SI: N/m para traslaciones, N·m/rad para giros"}},"dofs":{{"type":"array","minItems":1,"maxItems":1,"uniqueItems":true,"items":{{"enum":{dofs}}},"description":"Un único GDL local (varios GDL en un elemento serían un resorte de rango uno que los acopla; la fábrica lo rechaza)"}},"ground":{{"type":"boolean","default":false}},"direction":{{"$ref":"#/$defs/vector","description":"Dirección global finita no nula, solo Ux local"}},"rotation":{{"type":"array","minItems":3,"maxItems":3,"items":{{"$ref":"#/$defs/vector"}},"description":"Filas: ejes locales ortonormales, determinante +1; D2 conserva XY y +Z"}}}},"required":["k","dofs"],"additionalProperties":false,"not":{{"required":["direction","rotation"]}},"$defs":{{"vector":{{"type":"array","minItems":3,"maxItems":3,"items":{{"type":"number"}}}}}}}}"##
        )
    }
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>> {
        if self.dim != model.dim {
            return Err(invalid("Fábrica de resorte: dimensión incompatible."));
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
                finite(value, "Coordenada ModelView")?;
            }
            if self.dim == Dim::D2 && p[2] != 0.0 {
                return Err(invalid("ModelView D2 requiere Z=0."));
            }
        }
        let JsonValue::Object(fields) = params else {
            return Err(invalid("Resorte: se esperaba objeto SI."));
        };
        if fields
            .keys()
            .any(|key| !["k", "dofs", "ground", "direction", "rotation"].contains(&key.as_str()))
        {
            return Err(invalid("Resorte: propiedad desconocida."));
        }
        let k = match fields.get("k") {
            Some(JsonValue::Number(v)) => *v,
            _ => return Err(invalid("Resorte: k numérica requerida.")),
        };
        let JsonValue::Array(values) = fields
            .get("dofs")
            .ok_or_else(|| invalid("Resorte: dofs requerido."))?
        else {
            return Err(invalid("Resorte: dofs debe ser una lista."));
        };
        if values.is_empty() || values.len() > 6 {
            return Err(invalid(
                "Resorte: la fábrica requiere exactamente un GDL de la dimensión seleccionada.",
            ));
        }
        let mut dofs = Vec::with_capacity(values.len());
        for value in values {
            let JsonValue::String(label) = value else {
                return Err(invalid("GDL debe ser texto."));
            };
            let dof = self
                .dim
                .dofs()
                .iter()
                .find(|dof| dof.label() == label)
                .copied()
                .ok_or_else(|| invalid("GDL desconocido o ajeno a dimensión."))?;
            dofs.push(dof);
        }
        if dofs.iter().enumerate().any(|(i, d)| dofs[..i].contains(d)) {
            return Err(invalid("Resorte: GDL repetido."));
        }
        if dofs.len() > 1 {
            // Varios GDL en un mismo elemento forman un resorte de rango uno que
            // ACOPLA los GDL (k·bbᵀ), no resortes independientes: la fábrica lo
            // rechaza para evitar rigideces acopladas no pedidas (J4 r7).
            return Err(EngineError::new(
                "ENG-E-0013",
                "Resorte: un elemento admite un solo GDL; para resortes independientes use un elemento por GDL.",
            ));
        }
        let connection = match fields.get("ground") {
            None | Some(JsonValue::Bool(false)) => SpringConnection::BetweenNodes,
            Some(JsonValue::Bool(true)) => SpringConnection::Ground,
            _ => return Err(invalid("ground debe ser booleano.")),
        };
        let axes = match (fields.get("direction"), fields.get("rotation")) {
            (None, None) => SpringAxes::Global,
            (Some(value), None) => SpringAxes::Direction(array(value)?),
            (None, Some(JsonValue::Array(rows))) if rows.len() == 3 => {
                SpringAxes::Rotation([array(&rows[0])?, array(&rows[1])?, array(&rows[2])?])
            }
            _ => {
                return Err(invalid(
                    "Resorte: orientación incompatible o matriz de tamaño inválido.",
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
        return Err(invalid("Orientación requiere una lista numérica."));
    };
    if values.len() != 3 {
        return Err(invalid("Orientación requiere tres componentes."));
    }
    let mut out = [0.0; 3];
    for (target, value) in out.iter_mut().zip(values) {
        let JsonValue::Number(v) = value else {
            return Err(invalid("Orientación requiere números."));
        };
        *target = finite(*v, "Orientación")?;
    }
    Ok(out)
}
