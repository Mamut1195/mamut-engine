//! Opciones de análisis explícitas, sin factores ni espectros predeterminados.
use crate::input::Object;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub(crate) enum Analysis {
    #[serde(rename = "linearStatic")]
    LinearStatic {},
    #[serde(rename = "nonlinearStatic")]
    Nonlinear {
        options: Object<crate::extensions::Newton>,
    },
    #[serde(rename = "soilProfile")]
    SoilProfile {
        options: Object<crate::geo_input::Soil>,
    },
    #[serde(rename = "consolidation1d")]
    Consolidation {
        options: Object<crate::geo_input::Consolidation>,
    },
    #[serde(rename = "pDelta")]
    PDelta { options: Object<PDelta> },
    #[serde(rename = "modal")]
    Modal { options: Object<Modal> },
    #[serde(rename = "responseSpectrum")]
    Spectrum { options: Object<Spectrum> },
}
impl Analysis {
    pub fn method(&self) -> &'static str {
        match self {
            Self::LinearStatic { .. } => "linearStatic",
            Self::Nonlinear { .. } => "nonlinearStatic",
            Self::SoilProfile { .. } => "soilProfile",
            Self::Consolidation { .. } => "consolidation1d",
            Self::PDelta { .. } => "pDelta",
            Self::Modal { .. } => "modal",
            Self::Spectrum { .. } => "responseSpectrum",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaseFactor {
    pub case: String,
    pub factor: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PDelta {
    pub gravity: Object<CaseFactor>,
    pub lateral: Object<CaseFactor>,
    pub tolerance: f64,
    pub max_iterations: usize,
    pub displacement_scale: f64,
    pub rotation_scale: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MassSource {
    pub method: String,
    pub case: String,
    pub factor: f64,
    pub direction: [f64; 3],
    pub gravity: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Modal {
    pub mode_count: usize,
    pub mass_kind: String,
    pub directions: Vec<[f64; 3]>,
    #[serde(default, deserialize_with = "mass_source")]
    pub mass_source: Option<Object<MassSource>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spectrum {
    pub mode_count: usize,
    pub mass_kind: String,
    pub directions: Vec<Object<SpectrumDirection>>,
    pub modal_combination: String,
    pub directional_combination: String,
    pub directional_factors: Vec<f64>,
    #[serde(default, deserialize_with = "mass_source")]
    pub mass_source: Option<Object<MassSource>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpectrumDirection {
    pub direction: [f64; 3],
    pub spectra: Vec<Object<Table>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Table {
    pub pairs: Vec<[f64; 2]>,
    pub units: String,
    pub damping: f64,
    pub interpolation: String,
    pub extrapolation: String,
}

fn mass_source<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Object<MassSource>>, D::Error> {
    Object::<MassSource>::deserialize(d).map(Some)
}
