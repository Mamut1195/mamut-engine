use crate::{
    Diagnostic, GeoResult, Layer, Parameter, ParameterId, Provenance, Real, StressProfile,
    StressResult,
};

#[derive(Clone, Debug)]
struct ValidatedLayer {
    top_m: Real,
    bottom_m: Real,
    gamma: Real,
    gamma_sat: Option<Real>,
}

/// Perfil estratificado validado e inmutable, definido mediante espesores en m.
///
/// Las capas son contiguas desde profundidad cero. No hay capas implícitas,
/// conversión de cotas ni extensión bajo la última capa. Validar al construir
/// impide consultar un perfil incompleto aunque el dato falte en una capa profunda.
#[derive(Clone, Debug)]
pub struct StratifiedProfile {
    layers: Vec<ValidatedLayer>,
    total_depth_m: Real,
    groundwater_depth_m: Real,
    water_unit_weight_kn_m3: Real,
    assumed_parameters: Vec<ParameterId>,
}

fn diagnostic(code: &'static str, hint: String) -> Diagnostic {
    let mut diagnostic = Diagnostic::from_code(code).expect("Código existente en engine-core");
    diagnostic.hint = Some(hint);
    diagnostic
}

fn finite(value: Real, field: &str) -> GeoResult<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(diagnostic("ENG-E-0008", format!("{field} = {value}: se requiere un número finito; revise los datos o el desbordamiento aritmético. El catálogo no distingue entradas de resultados no finitos.")))
    }
}

fn invalid(hint: String) -> Diagnostic {
    diagnostic("ENG-E-0010", format!("{hint} El catálogo no tiene código geotécnico de rango/geometría; se reutiliza Dimensión inválida."))
}

fn required<T>(parameter: Option<T>, field: &str) -> GeoResult<T> {
    parameter.ok_or_else(|| diagnostic("ENG-E-0003", format!("Falta {field}: aporte valor y origen Measured/Assumed explícitos. No se sustituye por otro parámetro. El catálogo no tiene código de dato geotécnico ausente; se reutiliza Referencia rota.")))
}

fn positive(value: Real, field: &str) -> GeoResult<()> {
    finite(value, field)?;
    if value <= 0.0 {
        return Err(invalid(format!(
            "{field} = {value}: debe ser estrictamente positivo; corrija el dato."
        )));
    }
    Ok(())
}

fn report_assumed(parameters: &mut Vec<ParameterId>, parameter: Parameter, id: ParameterId) {
    if parameter.provenance == Provenance::Assumed {
        parameters.push(id);
    }
}

impl StratifiedProfile {
    /// Construye y valida el perfil completo sin valores por defecto.
    ///
    /// `groundwater_depth_m` es zw en m y `water_unit_weight_kn_m3` es γw en
    /// kN/m³, ambos obligatorios incluso si zw está bajo el perfil. Se admite
    /// zw = 0 y zw >= espesor total; zw < 0 se rechaza porque el modelo no
    /// incluye agua embalsada sobre el terreno ni su sobrecarga.
    ///
    /// # Errores
    ///
    /// Devuelve el primer diagnóstico: `ENG-E-0003` por dato requerido ausente;
    /// `ENG-E-0010` por perfil vacío, espesores/pesos no positivos, zw negativo
    /// o espesor perdido al acumular en f64; `ENG-E-0008` por no finitos o
    /// desbordamiento del espesor total. Un γsat aportado se valida aunque la
    /// capa esté seca. No se exige γsat >= γ. Las tensiones se comprueban al
    /// consultar: un perfil válido geométricamente puede desbordar en una consulta.
    ///
    /// ```
    /// use engine_geo::{Layer, Parameter, Provenance, StratifiedProfile};
    /// let m = |x| Parameter::new(x, Provenance::Measured);
    /// let error = StratifiedProfile::new(
    ///     vec![Layer::new(2.0, Some(m(18.0)), None)],
    ///     Some(m(0.0)), Some(m(10.0)),
    /// ).unwrap_err();
    /// assert_eq!(error.code, "ENG-E-0003");
    /// assert!(error.hint.unwrap().contains("layers[0].gamma_sat_kn_m3"));
    /// ```
    pub fn new(
        layers: Vec<Layer>,
        groundwater_depth_m: Option<Parameter>,
        water_unit_weight_kn_m3: Option<Parameter>,
    ) -> GeoResult<Self> {
        let groundwater = required(groundwater_depth_m, "groundwater_depth_m (zw, m)")?;
        finite(groundwater.value, "groundwater_depth_m (zw, m)")?;
        if groundwater.value < 0.0 {
            return Err(invalid(format!("groundwater_depth_m = {} m: zw < 0 (agua sobre el terreno) no está soportado; use zw >= 0 o un modelo con agua embalsada explícita.", groundwater.value)));
        }
        let water = required(
            water_unit_weight_kn_m3,
            "water_unit_weight_kn_m3 (γw, kN/m³)",
        )?;
        positive(water.value, "water_unit_weight_kn_m3 (γw, kN/m³)")?;
        if layers.is_empty() {
            return Err(invalid(
                "layers: el perfil está vacío; aporte al menos una capa.".into(),
            ));
        }

        let mut validated = Vec::with_capacity(layers.len());
        let mut assumed = Vec::new();
        let mut top = 0.0;
        for (layer_index, layer) in layers.into_iter().enumerate() {
            let prefix = format!("layers[{layer_index}]");
            positive(layer.thickness_m, &format!("{prefix}.thickness_m (m)"))?;
            let bottom = top + layer.thickness_m;
            finite(bottom, &format!("{prefix}.bottom_m acumulado (m)"))?;
            if bottom <= top {
                return Err(invalid(format!("{prefix}.thickness_m: el espesor se pierde al acumular en f64; revise la escala de los espesores.")));
            }
            let gamma = required(layer.gamma_kn_m3, &format!("{prefix}.gamma_kn_m3 (γ)"))?;
            positive(gamma.value, &format!("{prefix}.gamma_kn_m3 (kN/m³)"))?;
            let gamma_sat = if bottom > groundwater.value {
                Some(required(
                    layer.gamma_sat_kn_m3,
                    &format!("{prefix}.gamma_sat_kn_m3 (γsat, tramo bajo zw)"),
                )?)
            } else {
                layer.gamma_sat_kn_m3
            };
            report_assumed(&mut assumed, gamma, ParameterId::LayerGamma { layer_index });
            if let Some(saturated) = gamma_sat {
                positive(
                    saturated.value,
                    &format!("{prefix}.gamma_sat_kn_m3 (kN/m³)"),
                )?;
                report_assumed(
                    &mut assumed,
                    saturated,
                    ParameterId::LayerGammaSat { layer_index },
                );
            }
            validated.push(ValidatedLayer {
                top_m: top,
                bottom_m: bottom,
                gamma: gamma.value,
                gamma_sat: gamma_sat.map(|p| p.value),
            });
            top = bottom;
        }
        report_assumed(&mut assumed, groundwater, ParameterId::GroundwaterDepth);
        report_assumed(&mut assumed, water, ParameterId::WaterUnitWeight);
        Ok(Self {
            layers: validated,
            total_depth_m: top,
            groundwater_depth_m: groundwater.value,
            water_unit_weight_kn_m3: water.value,
            assumed_parameters: assumed,
        })
    }
}

impl StressProfile for StratifiedProfile {
    fn total_depth_m(&self) -> Real {
        self.total_depth_m
    }

    /// Integra los tramos sobre y bajo zw sin extrapolar ni omitir parámetros.
    ///
    /// Admite los extremos z = 0 y z = espesor total. Una consulta negativa o
    /// mayor que el total devuelve `ENG-E-0010`, sin tolerancia ni recorte;
    /// NaN, infinitos y resultados desbordados devuelven `ENG-E-0008`.
    ///
    /// ```
    /// use engine_geo::{Layer, Parameter, Provenance, StratifiedProfile, StressProfile};
    /// let m = |x| Parameter::new(x, Provenance::Measured);
    /// let profile = StratifiedProfile::new(
    ///     vec![Layer::new(2.0, Some(m(18.0)), None)],
    ///     Some(m(3.0)), Some(m(10.0)),
    /// )?;
    /// assert_eq!(profile.stresses_at(0.0)?.total_vertical_kpa, 0.0);
    /// assert_eq!(profile.total_vertical_stress_at(2.0)?, 36.0);
    /// assert_eq!(profile.stresses_at(2.01).unwrap_err().code, "ENG-E-0010");
    /// # Ok::<(), engine_core::Diagnostic>(())
    /// ```
    fn stresses_at(&self, depth_m: Real) -> GeoResult<StressResult> {
        finite(depth_m, "depth_m (m)")?;
        if depth_m < 0.0 || depth_m > self.total_depth_m {
            return Err(invalid(format!("depth_m = {depth_m} m: profundidad fuera de [0, {}] m; consulte dentro del perfil, sin extrapolar.", self.total_depth_m)));
        }
        let mut total = 0.0;
        for (layer_index, layer) in self.layers.iter().enumerate() {
            if depth_m <= layer.top_m {
                break;
            }
            let bottom = depth_m.min(layer.bottom_m);
            let dry_height = (bottom.min(self.groundwater_depth_m) - layer.top_m).max(0.0);
            let saturated_height = (bottom - layer.top_m.max(self.groundwater_depth_m)).max(0.0);
            total += layer.gamma * dry_height;
            if saturated_height > 0.0 {
                // El constructor exige γsat para cada capa con tramo sumergido.
                let saturated = required(
                    layer.gamma_sat,
                    &format!("layers[{layer_index}].gamma_sat_kn_m3"),
                )?;
                total += saturated * saturated_height;
            }
            finite(
                total,
                &format!("σv acumulada tras layers[{layer_index}] (kPa)"),
            )?;
        }
        let pore = self.water_unit_weight_kn_m3 * (depth_m - self.groundwater_depth_m).max(0.0);
        finite(pore, "u (kPa)")?;
        let effective = total - pore;
        finite(effective, "σ'v (kPa)")?;
        Ok(StressResult {
            depth_m,
            total_vertical_kpa: total,
            pore_pressure_kpa: pore,
            effective_vertical_kpa: effective,
            assumed_parameters: self.assumed_parameters.clone(),
        })
    }
}
