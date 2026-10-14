//! Esquema propio SI, sin parser ni referencias; producto independiente de ModelView.
use crate::{finite, invalid, EulerFrame2D, MomentReleases};
use engine_contracts::{Element, ElementFactory, JsonValue, ModelView};
use engine_core::{Dim, EResult, EngineError};
use std::collections::BTreeSet;

/// Fábrica frame2d.euler_elastic; E,A,I,rho requeridos, release_i/j opcionales.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameFactory;
impl ElementFactory for FrameFactory {
    fn kind(&self) -> &'static str {
        "frame2d.euler_elastic"
    }
    fn params_schema(&self) -> String {
        r#"{"type":"object","properties":{"E":{"type":"number","exclusiveMinimum":0,"description":"Young (Pa)"},"A":{"type":"number","exclusiveMinimum":0,"description":"Área (m²)"},"I":{"type":"number","exclusiveMinimum":0,"description":"Iz (m⁴)"},"rho":{"type":"number","minimum":0,"description":"Densidad (kg/m³)"},"release_i":{"type":"boolean","default":false},"release_j":{"type":"boolean","default":false}},"required":["E","A","I","rho"],"additionalProperties":false}"#.into()
    }
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>> {
        Ok(Box::new(build_frame(params, model)?))
    }
}

fn build_frame(params: &JsonValue, model: &ModelView<'_>) -> EResult<EulerFrame2D> {
    if model.dim != Dim::D2 {
        return Err(invalid("Fábrica de viga: requiere D2."));
    }
    let mut ids = BTreeSet::new();
    for (id, p) in model.nodes {
        if !ids.insert(*id) {
            return Err(EngineError::new("ENG-E-0002", "ModelView: id duplicado."));
        }
        for &x in p {
            finite(x, "Coordenada ModelView")?;
        }
        if p[2] != 0.0 {
            return Err(invalid("ModelView D2 requiere Z=0."));
        }
    }
    let JsonValue::Object(fields) = params else {
        return Err(invalid("Viga: se esperaba objeto SI."));
    };
    if fields
        .keys()
        .any(|k| !["E", "A", "I", "rho", "release_i", "release_j"].contains(&k.as_str()))
    {
        return Err(invalid("Viga: propiedad desconocida."));
    }
    let number = |key| match fields.get(key) {
        Some(JsonValue::Number(v)) => Ok(*v),
        _ => Err(invalid(&format!("Viga: {key} numérico requerido."))),
    };
    let boolean = |key| match fields.get(key) {
        None => Ok(false),
        Some(JsonValue::Bool(v)) => Ok(*v),
        _ => Err(invalid("Liberación debe ser booleano.")),
    };
    EulerFrame2D::new(
        number("E")?,
        number("A")?,
        number("I")?,
        number("rho")?,
        MomentReleases {
            i: boolean("release_i")?,
            j: boolean("release_j")?,
        },
    )
}

/// Fábrica `frame2d.winkler`: propiedades Euler y k≥0 en N/m².
/// Sin liberaciones de extremo; orientación y ejes según FrameFactory.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinklerFactory;
impl ElementFactory for WinklerFactory {
    fn kind(&self) -> &'static str {
        "frame2d.winkler"
    }
    fn params_schema(&self) -> String {
        r#"{"type":"object","properties":{"E":{"type":"number","exclusiveMinimum":0,"description":"Young (Pa)"},"A":{"type":"number","exclusiveMinimum":0,"description":"Área (m²)"},"I":{"type":"number","exclusiveMinimum":0,"description":"Iz (m⁴)"},"rho":{"type":"number","minimum":0,"description":"Densidad (kg/m³)"},"k":{"type":"number","minimum":0,"description":"Rigidez distribuida transversal local (N/m²)"}},"required":["E","A","I","rho","k"],"additionalProperties":false}"#.into()
    }
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>> {
        let JsonValue::Object(fields) = params else {
            return Err(invalid("Winkler: se esperaba objeto SI."));
        };
        if fields.contains_key("release_i") || fields.contains_key("release_j") {
            return Err(invalid("Winkler: liberaciones de extremo no admitidas."));
        }
        let number = |key| match fields.get(key) {
            Some(JsonValue::Number(v)) => Ok(*v),
            _ => Err(invalid("Winkler: rigidez numérica requerida.")),
        };
        let mut beam_fields = fields.clone();
        beam_fields.remove("k");
        Ok(Box::new(crate::WinklerFrame2D::new(
            build_frame(&JsonValue::Object(beam_fields), model)?,
            number("k")?,
        )?))
    }
}
