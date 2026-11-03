#![forbid(unsafe_code)]
//! Geotechnics: layered profiles and incremental consolidation, from scratch.
//!
//! [`consolidation`] solves multilayer 1D Terzaghi consolidation by finite elements;
//! [`winkler`] provides the elastic foundation. The following assumptions describe
//! only the geostatic stress profile.
//! [`py`] generates static lateral curves in unprefixed SI (N, m, Pa).
//! Axial curves [`tz`] and [`qz`] use unprefixed SI: m, Pa, and N; their z is
//! local displacement.
//!
//! # Units, axes, and assumptions
//!
//! Thicknesses and depths in m, unit weights in kN/m³, and stresses in kPa:
//! `(kN/m³) × m = kN/m² = kPa`. These are SI units with explicit prefixes.
//! The profile API uses kPa; to integrate with structural contracts in Pa, the
//! caller must convert through `engine_core::units::convert`. The core unit
//! catalog does not represent unit weights; it is not extended here.
//!
//! In the profile, depth `z` is positive downward: with global Z
//! upward, `z = Z_superficie − Z`. Geotechnical compression is positive;
//! when representing a normal stress with positive tension, its sign must be
//! reversed. The horizontal D2 plane is not interpreted as a vertical section.
//!
//! Each layer contributes γ above groundwater level `zw` and γsat below it.
//! Integrates `σv = Σ γi hi`, splitting layers that cross `zw`.
//! `u = γw max(z − zw, 0)` and `σ'v = σv − u`. Water weight is not added
//! again to γsat. No cohesion, capillarity, suction, surcharge, flow,
//! excess pore pressure, consolidation, or horizontal stresses.
//! Rejects `zw < 0` (water above ground); `zw >= espesor_total` produces
//! a dry profile for this model. At groundwater level, `u = 0`.
//!
//! # Explicit data and diagnostics
//!
//! Each γ, γsat, γw, and zw has required [`Provenance`], without default
//! values. [`Layer`] can represent missing data; the profile constructor
//! requires γ in all layers and γsat in each layer with a portion below `zw`.
//! Also requires zw and γw, even in a dry profile. Validates the entire profile
//! before any query and never substitutes γ for γsat.
//! All supplied unit weights must be positive and finite, without imposing
//! `γsat >= γ` or `γsat >= γw`; effective stress may be negative and is not
//! clipped. Results report all parameters declared
//! [`Provenance::Assumed`], even if unused at the queried depth.
//!
//! The catalog has no specific geotechnical codes. Reuses
//! `ENG-E-0003` for missing data, `ENG-E-0010` for invalid ranges/geometry,
//! and `ENG-E-0008` for NaN, infinities, or overflow. Each [`Diagnostic`]
//! retains the catalog message/severity and details the datum, cause, and correction in
//! `hint`. `entities` is empty because the core has no soil entities.
//!
//! # Compatibility
//!
//! Crate 0.1.0: an incompatible public API change increments the
//! minor version under semver before 1.0. The internal `stress-profile/1` contract
//! increments its major version if units, signs, or trait requirements change;
//! its minor version allows compatible extensions.
//!
//! ```
//! use engine_geo::{Layer, Parameter, Provenance, StratifiedProfile, StressProfile};
//! let measured = |value| Parameter::new(value, Provenance::Measured);
//! let profile = StratifiedProfile::new(
//!     vec![Layer::new(10.0, Some(measured(18.0)), Some(measured(20.0)))],
//!     Some(measured(3.0)), // zw, m bajo el terreno
//!     Some(Parameter::new(10.0, Provenance::Assumed)), // γw, kN/m³
//! )?;
//! let interchangeable: &dyn StressProfile = &profile;
//! let stress = interchangeable.stresses_at(5.0)?;
//! assert_eq!(stress.total_vertical_kpa, 18.0 * 3.0 + 20.0 * 2.0);
//! assert_eq!(stress.pore_pressure_kpa, 20.0);
//! assert_eq!(stress.effective_vertical_kpa, 74.0);
//! assert_eq!(stress.assumed_parameters, [engine_geo::ParameterId::WaterUnitWeight]);
//! # Ok::<(), engine_core::Diagnostic>(())
//! ```

#![deny(missing_docs)]

pub use engine_core::{ContractId, Diagnostic, Real};

pub mod pile;
pub mod winkler;

pub mod py;
pub mod tz;
pub use tz::qz;

pub mod consolidation;

mod profile;
pub use profile::StratifiedProfile;

/// Geotechnical result with a catalog diagnostic and contextual `hint`.
pub type GeoResult<T> = Result<T, Diagnostic>;

/// Declared provenance of a datum, without implicit classification or default value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Measured datum, as declared by the caller.
    Measured,
    /// Assumed datum, explicitly reported in the result.
    Assumed,
}

/// Inseparable value and provenance; the unit depends on the receiving field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameter {
    /// Value in the explicit units of the receiving field.
    pub value: Real,
    /// Explicit provenance of the value; does not certify measurement quality.
    pub provenance: Provenance,
}

impl Parameter {
    /// Declares a value and its provenance; validation occurs when constructing the profile.
    pub const fn new(value: Real, provenance: Provenance) -> Self {
        Self { value, provenance }
    }
}

/// Horizontal layer of constant thickness; layers are ordered from ground level.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    /// Thickness in m; must be finite and strictly positive.
    pub thickness_m: Real,
    /// γ in kN/m³ above groundwater level, required in all layers.
    pub gamma_kn_m3: Option<Parameter>,
    /// γsat in kN/m³, required if part of the layer is below groundwater level.
    pub gamma_sat_kn_m3: Option<Parameter>,
}

impl Layer {
    /// Declares a layer without filling missing data or validating them yet.
    pub const fn new(
        thickness_m: Real,
        gamma_kn_m3: Option<Parameter>,
        gamma_sat_kn_m3: Option<Parameter>,
    ) -> Self {
        Self {
            thickness_m,
            gamma_kn_m3,
            gamma_sat_kn_m3,
        }
    }
}

/// Identity of an assumed parameter; zero-based layer indices, without duplicates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterId {
    /// γ of the indicated layer.
    LayerGamma {
        /// Layer position from ground level, starting at zero.
        layer_index: usize,
    },
    /// γsat of the indicated layer.
    LayerGammaSat {
        /// Layer position from ground level, starting at zero.
        layer_index: usize,
    },
    /// Groundwater depth zw.
    GroundwaterDepth,
    /// Water unit weight γw.
    WaterUnitWeight,
}

/// Stresses at a depth, with explicit provenance of the entire profile.
#[derive(Clone, Debug, PartialEq)]
pub struct StressResult {
    /// Queried depth in m from ground level, positive downward.
    pub depth_m: Real,
    /// Total vertical stress σv in kPa, positive in compression.
    pub total_vertical_kpa: Real,
    /// Pore pressure u in kPa, hydrostatic below zw and zero above.
    pub pore_pressure_kpa: Real,
    /// Effective stress σ'v = σv − u in kPa, without clipping negative values.
    pub effective_vertical_kpa: Real,
    /// All parameters supplied as Assumed, including unused ones.
    ///
    /// Order: γ, γsat for each layer from the top, then zw and γw.
    pub assumed_parameters: Vec<ParameterId>,
}

/// Interchangeable, pure, thread-safe profile, internal contract `stress-profile/1`.
///
/// Input: finite depth in m in `[0, total_depth_m()]`, without extrapolation.
/// Output: finite stresses in kPa with positive compression and Assumed report.
/// Errors as documented for the crate; never omits a required datum.
/// Scalar methods are shortcuts; use `stresses_at` to obtain provenance.
pub trait StressProfile: Send + Sync {
    /// Contract identity; different names or major versions are incompatible.
    fn contract(&self) -> ContractId {
        ContractId::new("stress-profile", 1, 0)
    }

    /// Maximum allowed depth in m, finite and positive.
    fn total_depth_m(&self) -> Real;

    /// Computes σv, u, and σ'v, together with Assumed parameters of the entire profile.
    fn stresses_at(&self, depth_m: Real) -> GeoResult<StressResult>;

    /// Computes σv in kPa; propagates diagnostics from `stresses_at`.
    fn total_vertical_stress_at(&self, depth_m: Real) -> GeoResult<Real> {
        Ok(self.stresses_at(depth_m)?.total_vertical_kpa)
    }

    /// Computes u in kPa; propagates diagnostics from `stresses_at`.
    fn pore_pressure_at(&self, depth_m: Real) -> GeoResult<Real> {
        Ok(self.stresses_at(depth_m)?.pore_pressure_kpa)
    }

    /// Computes σ'v in kPa; propagates diagnostics from `stresses_at`.
    fn effective_vertical_stress_at(&self, depth_m: Real) -> GeoResult<Real> {
        Ok(self.stresses_at(depth_m)?.effective_vertical_kpa)
    }
}

#[cfg(test)]
mod tests;
