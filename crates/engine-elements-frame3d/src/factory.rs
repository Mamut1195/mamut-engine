//! Internal SI schema; owned parameters independent of ModelView.
use crate::{finite, invalid, EulerFrame3D, MomentReleases};
use engine_contracts::{Element, ElementFactory, JsonValue, ModelView};
use engine_core::{Dim, EResult, EngineError};
use std::collections::BTreeSet;

/// frame3d.euler_elastic factory; E,G,A,J,Iy,Iz,rho and orientation required.
/// Optional release_i/j: arrays of three local booleans `[T,My,Mz]`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameFactory;
impl ElementFactory for FrameFactory {
    fn kind(&self) -> &'static str {
        "frame3d.euler_elastic"
    }
    fn params_schema(&self) -> String {
        r#"{"type":"object","properties":{"E":{"type":"number","exclusiveMinimum":0,"description":"Young (Pa)"},"G":{"type":"number","exclusiveMinimum":0,"description":"Corte (Pa)"},"A":{"type":"number","exclusiveMinimum":0,"description":"Área (m²)"},"J":{"type":"number","exclusiveMinimum":0,"description":"Saint-Venant (m⁴)"},"Iy":{"type":"number","exclusiveMinimum":0},"Iz":{"type":"number","exclusiveMinimum":0},"rho":{"type":"number","minimum":0},"orientation":{"type":"array","items":{"type":"number"},"minItems":3,"maxItems":3,"description":"Referencia global de z local, no nula ni paralela al miembro"},"release_i":{"type":"array","items":{"type":"boolean"},"minItems":3,"maxItems":3,"default":[false,false,false]},"release_j":{"type":"array","items":{"type":"boolean"},"minItems":3,"maxItems":3,"default":[false,false,false]}},"required":["E","G","A","J","Iy","Iz","rho","orientation"],"additionalProperties":false}"#.into()
    }
    fn build(&self, params: &JsonValue, model: &ModelView<'_>) -> EResult<Box<dyn Element>> {
        Ok(Box::new(build_frame(params, model)?))
    }
}

fn build_frame(params: &JsonValue, model: &ModelView<'_>) -> EResult<EulerFrame3D> {
    if model.dim != Dim::D3 {
        return Err(invalid("Spatial beam factory: requires D3."));
    }
    let mut ids = BTreeSet::new();
    for (id, p) in model.nodes {
        if !ids.insert(*id) {
            return Err(EngineError::new("ENG-E-0002", "ModelView: duplicate id."));
        }
        for &x in p {
            finite(x, "ModelView coordinate")?;
        }
    }
    let JsonValue::Object(fields) = params else {
        return Err(invalid("Beam: expected an SI object."));
    };
    if fields.keys().any(|k| {
        ![
            "E",
            "G",
            "A",
            "J",
            "Iy",
            "Iz",
            "rho",
            "orientation",
            "release_i",
            "release_j",
        ]
        .contains(&k.as_str())
    }) {
        return Err(invalid("Beam: unknown property."));
    }
    let number = |key| match fields.get(key) {
        Some(JsonValue::Number(v)) => Ok(*v),
        _ => Err(invalid(&format!("Beam: numeric {key} is required."))),
    };
    let Some(JsonValue::Array(values)) = fields.get("orientation") else {
        return Err(invalid("orientation requires an array of three numbers."));
    };
    if values.len() != 3 {
        return Err(invalid("orientation requires three components."));
    }
    let mut orientation = [0.0; 3];
    for (r, v) in orientation.iter_mut().zip(values) {
        let JsonValue::Number(v) = v else {
            return Err(invalid("orientation requires numbers."));
        };
        *r = *v;
    }
    let releases = |key| -> EResult<[bool; 3]> {
        let mut out = [false; 3];
        if let Some(v) = fields.get(key) {
            let JsonValue::Array(v) = v else {
                return Err(invalid("Release requires a boolean array."));
            };
            if v.len() != 3 {
                return Err(invalid("Release requires three components."));
            }
            for (r, v) in out.iter_mut().zip(v) {
                let JsonValue::Bool(v) = v else {
                    return Err(invalid("Release must be boolean."));
                };
                *r = *v;
            }
        }
        Ok(out)
    };
    EulerFrame3D::new(
        number("E")?,
        number("G")?,
        number("A")?,
        number("J")?,
        number("Iy")?,
        number("Iz")?,
        number("rho")?,
        orientation,
        MomentReleases {
            i: releases("release_i")?,
            j: releases("release_j")?,
        },
    )
}

/// `frame3d.winkler` factory: Euler properties and ky, kz≥0 in N/m².
/// No end releases; orientation and axes as in FrameFactory.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinklerFactory;
impl ElementFactory for WinklerFactory {
    fn kind(&self) -> &'static str {
        "frame3d.winkler"
    }
    fn params_schema(&self) -> String {
        r#"{"type":"object","properties":{"E":{"type":"number","exclusiveMinimum":0,"description":"Young (Pa)"},"G":{"type":"number","exclusiveMinimum":0,"description":"Corte (Pa)"},"A":{"type":"number","exclusiveMinimum":0,"description":"Área (m²)"},"J":{"type":"number","exclusiveMinimum":0,"description":"Saint-Venant (m⁴)"},"Iy":{"type":"number","exclusiveMinimum":0},"Iz":{"type":"number","exclusiveMinimum":0},"rho":{"type":"number","minimum":0},"orientation":{"type":"array","items":{"type":"number"},"minItems":3,"maxItems":3,"description":"Referencia global de z local, no nula ni paralela al miembro"},"ky":{"type":"number","minimum":0,"description":"Rigidez distribuida transversal local (N/m²)"},"kz":{"type":"number","minimum":0,"description":"Rigidez distribuida transversal local (N/m²)"}},"required":["E","G","A","J","Iy","Iz","rho","orientation","ky","kz"],"additionalProperties":false}"#.into()
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
        beam_fields.remove("ky");
        beam_fields.remove("kz");
        Ok(Box::new(crate::WinklerFrame3D::new(
            build_frame(&JsonValue::Object(beam_fields), model)?,
            number("ky")?,
            number("kz")?,
        )?))
    }
}
