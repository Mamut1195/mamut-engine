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

/// Validated immutable layered profile, defined by thicknesses in m.
///
/// Layers are contiguous from depth zero. No implicit layers,
/// elevation conversion, or extension below the last layer. Validation on construction
/// prevents querying an incomplete profile even if data are missing in a deep layer.
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
        Err(diagnostic("ENG-E-0008", format!("{field} = {value}: a finite number is required; check the data or arithmetic overflow. The catalog does not distinguish non-finite inputs from non-finite results.")))
    }
}

fn invalid(hint: String) -> Diagnostic {
    diagnostic("ENG-E-0010", format!("{hint} The catalog has no geotechnical range/geometry code; Invalid dimension is reused."))
}

fn required<T>(parameter: Option<T>, field: &str) -> GeoResult<T> {
    parameter.ok_or_else(|| diagnostic("ENG-E-0003", format!("Missing {field}: provide an explicit value and Measured/Assumed origin. No other parameter is substituted. The catalog has no code for missing geotechnical data; Broken reference is reused.")))
}

fn positive(value: Real, field: &str) -> GeoResult<()> {
    finite(value, field)?;
    if value <= 0.0 {
        return Err(invalid(format!(
            "{field} = {value}: must be strictly positive; correct the data."
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
    /// Constructs and validates the entire profile without default values.
    ///
    /// `groundwater_depth_m` is zw in m and `water_unit_weight_kn_m3` is γw in
    /// kN/m³, both required even if zw is below the profile. Allows
    /// zw = 0 and zw >= total thickness; zw < 0 is rejected because the model does not
    /// include ponded water above ground or its surcharge.
    ///
    /// # Errors
    ///
    /// Returns the first diagnostic: `ENG-E-0003` for missing required data;
    /// `ENG-E-0010` for an empty profile, nonpositive thicknesses/unit weights, negative zw,
    /// or thickness lost during f64 accumulation; `ENG-E-0008` for nonfinite values or
    /// total thickness overflow. A supplied γsat is validated even if the
    /// layer is dry. γsat >= γ is not required. Stresses are checked on
    /// query: a geometrically valid profile may overflow during a query.
    ///
    /// # Precision limit (f64)
    ///
    /// Elevations are accumulated in f64 (53 mantissa bits). At an accumulated depth
    /// near 1e16 m, the spacing between consecutive f64 values is 2 m: a
    /// thickness below half that spacing (e.g. 1 m) is lost and
    /// rejected with `ENG-E-0010`, but a larger one (e.g. 3 m) is accepted and its bottom
    /// elevation is rounded (1e16 + 3 → 1e16 + 4). This is not a logic error but
    /// a limitation of f64 arithmetic; the API emits no warning in the second
    /// case. Actual physical thicknesses are many orders of magnitude
    /// below this regime.
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
            return Err(invalid(format!("groundwater_depth_m = {} m: zw < 0 (water above ground) is not supported; use zw >= 0 or a model with explicit ponded water.", groundwater.value)));
        }
        let water = required(
            water_unit_weight_kn_m3,
            "water_unit_weight_kn_m3 (γw, kN/m³)",
        )?;
        positive(water.value, "water_unit_weight_kn_m3 (γw, kN/m³)")?;
        if layers.is_empty() {
            return Err(invalid(
                "layers: profile is empty; provide at least one layer.".into(),
            ));
        }

        let mut validated = Vec::with_capacity(layers.len());
        let mut assumed = Vec::new();
        let mut top = 0.0;
        for (layer_index, layer) in layers.into_iter().enumerate() {
            let prefix = format!("layers[{layer_index}]");
            positive(layer.thickness_m, &format!("{prefix}.thickness_m (m)"))?;
            let bottom = top + layer.thickness_m;
            finite(bottom, &format!("{prefix}.bottom_m accumulated (m)"))?;
            if bottom <= top {
                return Err(invalid(format!("{prefix}.thickness_m: thickness is lost when accumulating in f64; check the thickness scale.")));
            }
            let gamma = required(layer.gamma_kn_m3, &format!("{prefix}.gamma_kn_m3 (γ)"))?;
            positive(gamma.value, &format!("{prefix}.gamma_kn_m3 (kN/m³)"))?;
            let gamma_sat = if bottom > groundwater.value {
                Some(required(
                    layer.gamma_sat_kn_m3,
                    &format!("{prefix}.gamma_sat_kn_m3 (γsat, span below zw)"),
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

    /// Integrates portions above and below zw without extrapolating or omitting parameters.
    ///
    /// Accepts the ends z = 0 and z = total thickness. A negative query or
    /// one above the total returns `ENG-E-0010`, without tolerance or clipping;
    /// NaN, infinities, and overflowed results return `ENG-E-0008`.
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
            return Err(invalid(format!("depth_m = {depth_m} m: depth is outside [0, {}] m; query within the profile, without extrapolation.", self.total_depth_m)));
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
                // The constructor requires γsat for each layer with a submerged portion.
                let saturated = required(
                    layer.gamma_sat,
                    &format!("layers[{layer_index}].gamma_sat_kn_m3"),
                )?;
                total += saturated * saturated_height;
            }
            finite(
                total,
                &format!("Accumulated σv after layers[{layer_index}] (kPa)"),
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
