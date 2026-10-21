//! Despacho y serialización geotécnica, sin modificar el cálculo del dominio.
use super::*;
use crate::{analysis_input::Analysis, input::Object};
use engine_geo::{ParameterId, StressProfile};

pub(crate) fn budget(_: &input::Execution) -> usize {
    4
}

pub(crate) fn execute(
    e: &input::Execution,
    cancel: &CancelToken,
    progress: &mut dyn FnMut(Progress),
    document: &mut Value,
    total: usize,
) -> Result<(), Failure> {
    cancel.check()?;
    let result = match e.analyses.first() {
        Some(Analysis::SoilProfile { options: Object(o) }) => {
            let profile = o.profile()?;
            let mut states = Vec::new();
            for &depth in &o.depths_m {
                cancel.check()?;
                let s = profile.stresses_at(depth)?;
                let assumed: Vec<_> = s
                    .assumed_parameters
                    .iter()
                    .map(|p| match p {
                        ParameterId::LayerGamma { layer_index } => {
                            format!("layers[{layer_index}].gamma_kn_m3")
                        }
                        ParameterId::LayerGammaSat { layer_index } => {
                            format!("layers[{layer_index}].gamma_sat_kn_m3")
                        }
                        ParameterId::GroundwaterDepth => "groundwater_depth_m".into(),
                        ParameterId::WaterUnitWeight => "water_unit_weight_kn_m3".into(),
                    })
                    .collect();
                states.push(json!({"depth_m": s.depth_m, "total_vertical_kpa": s.total_vertical_kpa, "pore_pressure_kpa": s.pore_pressure_kpa, "effective_vertical_kpa": s.effective_vertical_kpa, "assumed_parameters": assumed}));
            }
            json!({"method": "soilProfile", "states": states})
        }
        Some(Analysis::Consolidation { options: Object(o) }) => {
            let result = o
                .run_with_cancel(&mut || cancel.is_cancelled())
                .map_err(|d| {
                    if d.code == "API-I-CANCELLED" {
                        Failure::from(ApiError::Cancelled)
                    } else {
                        Failure::from(d)
                    }
                })?;
            json!({"method": "consolidation1d", "depths_m": result.depths_m, "assumed_parameters": result.assumed_parameters,
                "states": result.states.iter().map(|s| json!({"time_s": s.time_s, "delta_stress_kpa": s.delta_stress_kpa,
                    "excess_pore_pressure_kpa": s.excess_pore_pressure_kpa, "settlement_m": s.settlement_m, "mean_degree": s.mean_degree})).collect::<Vec<_>>()})
        }
        _ => return Err(EngineError::new("ENG-E-0012", "Despacho geotécnico incoherente.").into()),
    };
    step(cancel, progress, "geotechnical", total - 1, total)?;
    document["geotechnical"] = result;
    Ok(())
}
