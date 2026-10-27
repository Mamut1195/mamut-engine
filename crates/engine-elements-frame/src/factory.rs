//! Internal SI schema, without a parser or references; product independent of ModelView.
use crate::{finite, invalid, EulerFrame2D, MomentReleases};
use engine_contracts::{Element, ElementFactory, JsonValue, ModelView};
use engine_core::{Dim, EResult, EngineError};
use std::collections::BTreeSet;

/// frame2d.euler_elastic factory; E,A,I,rho required, release_i/j optional.
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
        return Err(invalid("Beam factory: requires D2."));
    }
    let mut ids = BTreeSet::new();
    for (id, p) in model.nodes {
        if !ids.insert(*id) {
            return Err(EngineError::new("ENG-E-0002", "ModelView: duplicate id."));
        }
        for &x in p {
            finite(x, "ModelView coordinate")?;
        }
        if p[2] != 0.0 {
            return Err(invalid("ModelView D2 requires Z=0."));
        }
    }
    let JsonValue::Object(fields) = params else {
        return Err(invalid("Beam: expected an SI object."));
    };
    if fields
        .keys()
        .any(|k| !["E", "A", "I", "rho", "release_i", "release_j"].contains(&k.as_str()))
    {
        return Err(invalid("Beam: unknown property."));
    }
    let number = |key| match fields.get(key) {
        Some(JsonValue::Number(v)) => Ok(*v),
        _ => Err(invalid(&format!("Beam: numeric {key} is required."))),
    };
    let boolean = |key| match fields.get(key) {
        None => Ok(false),
        Some(JsonValue::Bool(v)) => Ok(*v),
        _ => Err(invalid("Release must be boolean.")),
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

/// `frame2d.winkler` factory: Euler properties and k≥0 in N/m².
/// No end releases; orientation and axes as in FrameFactory.
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
            return Err(invalid("Winkler: expected an SI object."));
        };
        if fields.contains_key("release_i") || fields.contains_key("release_j") {
            return Err(invalid("Winkler: end releases are not allowed."));
        }
        let number = |key| match fields.get(key) {
            Some(JsonValue::Number(v)) => Ok(*v),
            _ => Err(invalid("Winkler: numeric stiffness is required.")),
        };
        let mut beam_fields = fields.clone();
        beam_fields.remove("k");
        Ok(Box::new(crate::WinklerFrame2D::new(
            build_frame(&JsonValue::Object(beam_fields), model)?,
            number("k")?,
        )?))
    }
}
