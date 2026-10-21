//! Adaptadores de datos geotécnicos; las unidades conservan sus prefijos explícitos.
use crate::{
    analysis_input::Analysis,
    extensions::optional,
    input::{Execution, Object},
};
use engine_geo::{
    consolidation as c, GeoResult, Layer, Parameter, Provenance, StratifiedProfile, StressProfile,
};
use serde::Deserialize;

/// Trabajo máximo de consolidation1d: suma de subpasos de toda la historia
/// (partida también en tiempos de salida) por número total de elementos.
/// La validación rechaza valores mayores con API-E-CONSOLIDATION-WORK.
pub const MAX_CONSOLIDATION_WORK: u128 = 50_000_000;

/// Coste denso máximo: Σ intervalos n³ + Σ subpasos n², con n nodos libres.
/// Se cuentan todos los quiebres de carga y salidas de toda la historia, sin
/// descontar factores reutilizados. u128 saturante; API-E-CONSOLIDATION-COST.
/// Calibrado en release con margen respecto a 5 s; véase ENG-SFC-010, r3.
pub const MAX_CONSOLIDATION_COST: u128 = 500_000_000;
/// Máximo de intervalos al unir historia y salidas (también limita la historia
/// a 257 puntos). Superarlo produce API-E-CONSOLIDATION-COST.
pub const MAX_CONSOLIDATION_SEGMENTS: u128 = 256;
/// Máximo agregado de subpasos, incluso con una sola incógnita; limita el
/// coste fijo por paso. Superarlo produce API-E-CONSOLIDATION-COST.
pub const MAX_CONSOLIDATION_SUBSTEPS: u128 = 100_000;

#[derive(Deserialize)]
#[serde(try_from = "String")]
enum Origin {
    Measured,
    Assumed,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Datum {
    value: f64,
    provenance: Origin,
}
impl Datum {
    fn domain(&self) -> Parameter {
        Parameter::new(
            self.value,
            match self.provenance {
                Origin::Measured => Provenance::Measured,
                Origin::Assumed => Provenance::Assumed,
            },
        )
    }
}
fn parameter(value: &Option<Object<Datum>>) -> Option<Parameter> {
    value.as_ref().map(|Object(p)| p.domain())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SoilLayer {
    thickness_m: f64,
    #[serde(default, deserialize_with = "optional")]
    gamma_kn_m3: Option<Object<Datum>>,
    #[serde(default, deserialize_with = "optional")]
    gamma_sat_kn_m3: Option<Object<Datum>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Soil {
    layers: Vec<Object<SoilLayer>>,
    #[serde(default, deserialize_with = "optional")]
    groundwater_depth_m: Option<Object<Datum>>,
    #[serde(default, deserialize_with = "optional")]
    water_unit_weight_kn_m3: Option<Object<Datum>>,
    pub depths_m: Vec<f64>,
}
impl Soil {
    pub fn profile(&self) -> GeoResult<StratifiedProfile> {
        StratifiedProfile::new(
            self.layers
                .iter()
                .map(|Object(l)| {
                    Layer::new(
                        l.thickness_m,
                        parameter(&l.gamma_kn_m3),
                        parameter(&l.gamma_sat_kn_m3),
                    )
                })
                .collect(),
            parameter(&self.groundwater_depth_m),
            parameter(&self.water_unit_weight_kn_m3),
        )
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConsolidationLayer {
    thickness_m: Object<Datum>,
    #[serde(default, deserialize_with = "optional")]
    cv_m2_s: Option<Object<Datum>>,
    #[serde(default, deserialize_with = "optional")]
    mv_per_kpa: Option<Object<Datum>>,
    #[serde(default, deserialize_with = "optional")]
    k_m_s: Option<Object<Datum>>,
    elements: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoadPoint {
    time_s: f64,
    #[serde(default, deserialize_with = "optional")]
    delta_stress_kpa: Option<Object<Datum>>,
}
#[derive(Deserialize)]
#[serde(try_from = "String")]
enum Drainage {
    Top,
    Bottom,
    Double,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Consolidation {
    layers: Vec<Object<ConsolidationLayer>>,
    #[serde(default, deserialize_with = "optional")]
    water_unit_weight_kn_m3: Option<Object<Datum>>,
    drainage: Drainage,
    history: Vec<Object<LoadPoint>>,
    pub output_times_s: Vec<f64>,
    theta: f64,
    max_step_s: f64,
}
impl Consolidation {
    pub fn profile(&self) -> GeoResult<c::ConsolidationProfile> {
        c::ConsolidationProfile::new(
            self.layers
                .iter()
                .map(|Object(l)| c::ConsolidationLayer {
                    thickness_m: l.thickness_m.0.domain(),
                    cv_m2_s: parameter(&l.cv_m2_s),
                    mv_per_kpa: parameter(&l.mv_per_kpa),
                    k_m_s: parameter(&l.k_m_s),
                    elements: l.elements,
                })
                .collect(),
            parameter(&self.water_unit_weight_kn_m3),
            match self.drainage {
                Drainage::Top => c::Drainage::Top,
                Drainage::Bottom => c::Drainage::Bottom,
                Drainage::Double => c::Drainage::Double,
            },
        )
    }
    pub fn run_with_cancel(
        &self,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> GeoResult<c::ConsolidationResult> {
        self.profile()?.solve_with_cancel(
            &self
                .history
                .iter()
                .map(|Object(p)| c::LoadPoint {
                    time_s: p.time_s,
                    delta_stress_kpa: parameter(&p.delta_stress_kpa),
                })
                .collect::<Vec<_>>(),
            &self.output_times_s,
            c::ConsolidationOptions {
                theta: self.theta,
                max_step_s: self.max_step_s,
            },
            cancelled,
        )
    }
    fn validate(&self) -> GeoResult<()> {
        self.profile()?;
        if !(0.0..=1.0).contains(&self.theta)
            || self.max_step_s <= 0.0
            || !self.max_step_s.is_finite()
            || self.history.len() < 2
            || self.output_times_s.is_empty()
        {
            return Err(error(
                "ENG-E-0010",
                "Consolidación: controles, historia o salidas inválidos.",
            ));
        }
        let mut previous = -1.0;
        for (index, Object(p)) in self.history.iter().enumerate() {
            if !p.time_s.is_finite() || (index == 0 && p.time_s != 0.0) || p.time_s <= previous {
                return Err(error(
                    "ENG-E-0010",
                    "Historia creciente desde t=0 requerida.",
                ));
            }
            let stress = parameter(&p.delta_stress_kpa)
                .ok_or_else(|| error("ENG-E-0003", "Falta delta_stress_kpa y su procedencia."))?;
            if !stress.value.is_finite() {
                return Err(error("ENG-E-0008", "Carga no finita."));
            }
            previous = p.time_s;
        }
        let end = previous;
        previous = -1.0;
        for &time in &self.output_times_s {
            if !time.is_finite() || time < 0.0 || time > end || time <= previous {
                return Err(error(
                    "ENG-E-0010",
                    "Salidas crecientes dentro de la historia requeridas.",
                ));
            }
            previous = time;
        }
        self.validate_work()?;
        Ok(())
    }

    fn validate_work(&self) -> GeoResult<()> {
        let elements = self.layers.iter().fold(0u128, |total, Object(l)| {
            total.saturating_add(l.elements as u128)
        });
        let mut substeps = 0u128;
        let mut segments = 0u128;
        let mut outputs = self.output_times_s.iter().copied().peekable();
        for pair in self.history.windows(2) {
            let mut time = pair[0].0.time_s;
            let end = pair[1].0.time_s;
            while time < end {
                while outputs.peek().is_some_and(|&t| t <= time) {
                    outputs.next();
                }
                let stop = outputs.peek().copied().unwrap_or(end).min(end);
                // La conversión satura incluso si el cociente llega a infinito.
                let steps = ((stop - time) / self.max_step_s).ceil().max(1.0) as u128;
                substeps = substeps.saturating_add(steps);
                segments = segments.saturating_add(1);
                if substeps.saturating_mul(elements) > MAX_CONSOLIDATION_WORK {
                    return Err(error(
                        "API-E-CONSOLIDATION-WORK",
                        "Consolidación: el trabajo total supera 50000000 subpasos por elementos; reduzca la historia/malla o aumente max_step_s.",
                    ));
                }
                time = stop;
            }
        }
        // free_range del dominio elimina un dren o dos de elements+1 nodos.
        let n = elements.saturating_sub(u128::from(matches!(self.drainage, Drainage::Double)));
        let square = n.saturating_mul(n);
        let cost = segments
            .saturating_mul(square.saturating_mul(n))
            .saturating_add(substeps.saturating_mul(square));
        if cost > MAX_CONSOLIDATION_COST
            || segments > MAX_CONSOLIDATION_SEGMENTS
            || substeps > MAX_CONSOLIDATION_SUBSTEPS
        {
            return Err(error(
                "API-E-CONSOLIDATION-COST",
                "Consolidación: supera el coste denso 500000000 (Σ intervalos n³ + Σ subpasos n², n nodos libres), 256 intervalos de historia/salida o 100000 subpasos totales; reduzca historia/malla/salidas o aumente max_step_s.",
            ));
        }
        Ok(())
    }
}
fn error(code: &'static str, message: &str) -> engine_core::Diagnostic {
    engine_core::Diagnostic::new(code, engine_core::Severity::Error, message)
}
pub(crate) fn validate(e: &Execution) -> GeoResult<()> {
    let analysis = e.analyses.first();
    if matches!(
        analysis,
        Some(Analysis::SoilProfile { .. } | Analysis::Consolidation { .. })
    ) {
        let m = &e.model;
        if !m.nodes.is_empty()
            || !m.members.is_empty()
            || !m.sections.is_empty()
            || !m.laws.is_empty()
            || !m.supports.is_empty()
            || !m.masses.is_empty()
            || !m.cases.is_empty()
            || !m.combinations.is_empty()
            || !e.prescribed.is_empty()
            || !e.diaphragms.is_empty()
            || !e.springs.is_empty()
        {
            return Err(error(
                "ENG-E-0010",
                "Análisis de suelo requiere modelo estructural, casos y combinaciones vacíos.",
            ));
        }
    }
    match analysis {
        Some(Analysis::SoilProfile { options: Object(o) }) => {
            if o.depths_m.is_empty() {
                return Err(error("ENG-E-0010", "Aporte profundidades de consulta."));
            }
            let profile = o.profile()?;
            for &z in &o.depths_m {
                profile.stresses_at(z)?;
            }
        }
        Some(Analysis::Consolidation { options: Object(o) }) => o.validate()?,
        _ => {}
    }
    Ok(())
}

impl TryFrom<String> for Origin {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "measured" => Ok(Self::Measured),
            "assumed" => Ok(Self::Assumed),
            _ => Err("Procedencia: measured o assumed requeridos."),
        }
    }
}
impl TryFrom<String> for Drainage {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "top" => Ok(Self::Top),
            "bottom" => Ok(Self::Bottom),
            "double" => Ok(Self::Double),
            _ => Err("Drenaje: top, bottom o double requeridos."),
        }
    }
}
