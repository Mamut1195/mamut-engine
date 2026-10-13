//! Esquema SI propio; parámetros poseídos e independientes de ModelView.
use crate::{finite, invalid, EulerFrame3D, MomentReleases};
use engine_contracts::{Element, ElementFactory, JsonValue, ModelView};
use engine_core::{Dim, EResult, EngineError};
use std::collections::BTreeSet;

/// Fábrica frame3d.euler_elastic; E,G,A,J,Iy,Iz,rho y orientation requeridos.
/// release_i/j opcionales: arrays de tres booleanos `[T,My,Mz]` locales.
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
        if model.dim != Dim::D3 {
            return Err(invalid("Fábrica de viga espacial: requiere D3."));
        }
        let mut ids = BTreeSet::new();
        for (id, p) in model.nodes {
            if !ids.insert(*id) {
                return Err(EngineError::new("ENG-E-0002", "ModelView: id duplicado."));
            }
            for &x in p {
                finite(x, "Coordenada ModelView")?;
            }
        }
        let JsonValue::Object(fields) = params else {
            return Err(invalid("Viga: se esperaba objeto SI."));
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
            return Err(invalid("Viga: propiedad desconocida."));
        }
        let number = |key| match fields.get(key) {
            Some(JsonValue::Number(v)) => Ok(*v),
            _ => Err(invalid(&format!("Viga: {key} numérico requerido."))),
        };
        let Some(JsonValue::Array(values)) = fields.get("orientation") else {
            return Err(invalid("orientation requiere array de tres números."));
        };
        if values.len() != 3 {
            return Err(invalid("orientation requiere tres componentes."));
        }
        let mut orientation = [0.0; 3];
        for (r, v) in orientation.iter_mut().zip(values) {
            let JsonValue::Number(v) = v else {
                return Err(invalid("orientation requiere números."));
            };
            *r = *v;
        }
        let releases = |key| -> EResult<[bool; 3]> {
            let mut out = [false; 3];
            if let Some(v) = fields.get(key) {
                let JsonValue::Array(v) = v else {
                    return Err(invalid("Liberación requiere array booleano."));
                };
                if v.len() != 3 {
                    return Err(invalid("Liberación requiere tres componentes."));
                }
                for (r, v) in out.iter_mut().zip(v) {
                    let JsonValue::Bool(v) = v else {
                        return Err(invalid("Liberación debe ser booleana."));
                    };
                    *r = *v;
                }
            }
            Ok(out)
        };
        Ok(Box::new(EulerFrame3D::new(
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
        )?))
    }
}
