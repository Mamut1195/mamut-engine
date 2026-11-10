//! Closed SI and US customary catalog, phase 1 of MAM-1043 (ENG-CORE-012).
//!
//! [`CONVERSION_TABLE`] is the single versioned table. Every factor is an exact
//! quotient of the anchors below; the canonical value of a dimension is SI.
//! This module is not consulted by [`super::Unit::parse`] and it is not part of
//! `engine-input/1`, which still accepts only `units: "SI"`.
//!
//! Mass in a force system is a declared unit. Gravity is never inserted when
//! the caller omits it. Temperature intervals use the scale factor alone; the
//! Fahrenheit offset is applied only by [`convert_temperature`] with
//! [`TemperatureReading::Absolute`].
//!
//! ```
//! use engine_core::units::catalog::{convert, CatalogUnit};
//! let metres = convert(1.0, CatalogUnit::Inch, CatalogUnit::Meter)?;
//! assert_eq!(metres, 0.0254);
//! let error = CatalogUnit::parse("lb").unwrap_err();
//! assert_eq!(error.code, "ENG-E-0001");
//! assert!(error.message.contains("«lb»"));
//! # Ok::<(), engine_core::EngineError>(())
//! ```

use crate::{EResult, EngineError, Real};

/// Identifier of the single conversion table. Not an input-schema version.
pub const TABLE_ID: &str = "units-table/1";

/// Version of [`TABLE_ID`]. A factor change requires a new version.
pub const TABLE_VERSION: u32 = 1;

/// Exact positive quotient `numerator / denominator` in canonical SI units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactFactor {
    /// Numerator of the exact quotient.
    pub numerator: u64,
    /// Denominator of the exact quotient. Never zero.
    pub denominator: u64,
}

impl ExactFactor {
    /// Builds a quotient. A zero denominator is a programming failure.
    pub const fn new(numerator: u64, denominator: u64) -> Self {
        assert!(denominator != 0, "exact factor denominator is zero");
        Self {
            numerator,
            denominator,
        }
    }

    /// Quotient 1/1.
    pub const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    /// IEEE-754 division of the two integers.
    ///
    /// Integers above 2^53 are not all representable. Callers that need a
    /// bound use the round trip, which repeats the same `f64` factor.
    pub const fn to_f64(self) -> Real {
        self.numerator as Real / self.denominator as Real
    }

    /// Product of two quotients, reduced by their greatest common divisor.
    pub const fn checked_mul(self, other: Self) -> Option<Self> {
        let numerator = match (self.numerator as u128).checked_mul(other.numerator as u128) {
            Some(value) => value,
            None => return None,
        };
        let denominator = match (self.denominator as u128).checked_mul(other.denominator as u128) {
            Some(value) => value,
            None => return None,
        };
        reduce(numerator, denominator)
    }

    /// Quotient of two quotients. Division by a zero numerator returns `None`.
    pub const fn checked_div(self, other: Self) -> Option<Self> {
        if other.numerator == 0 {
            return None;
        }
        self.checked_mul(Self {
            numerator: other.denominator,
            denominator: other.numerator,
        })
    }

    /// Repeated product. Exponent 0 yields 1/1.
    pub const fn checked_pow(self, exponent: u32) -> Option<Self> {
        let mut acc = Self::ONE;
        let mut left = exponent;
        while left > 0 {
            acc = match acc.checked_mul(self) {
                Some(value) => value,
                None => return None,
            };
            left -= 1;
        }
        Some(acc)
    }

    /// Difference of two quotients. A negative result returns `None`.
    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        let left = match (self.numerator as u128).checked_mul(other.denominator as u128) {
            Some(value) => value,
            None => return None,
        };
        let right = match (other.numerator as u128).checked_mul(self.denominator as u128) {
            Some(value) => value,
            None => return None,
        };
        if left < right {
            return None;
        }
        let denominator = match (self.denominator as u128).checked_mul(other.denominator as u128) {
            Some(value) => value,
            None => return None,
        };
        reduce(left - right, denominator)
    }
}

const fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

const fn reduce(numerator: u128, denominator: u128) -> Option<ExactFactor> {
    if denominator == 0 {
        return None;
    }
    let divisor = gcd(numerator, denominator);
    let numerator = numerator / divisor;
    let denominator = denominator / divisor;
    if numerator > u64::MAX as u128 || denominator > u64::MAX as u128 {
        return None;
    }
    Some(ExactFactor {
        numerator: numerator as u64,
        denominator: denominator as u64,
    })
}

const fn must(factor: Option<ExactFactor>) -> ExactFactor {
    match factor {
        Some(factor) => factor,
        None => panic!("exact factor does not fit in the table"),
    }
}

// Anchors. Every other factor in this module is a product of these values.
/// 1 in = 127/5000 m = 0.0254 m. International inch, yard/36.
const INCH_METRE: ExactFactor = ExactFactor::new(127, 5_000);
/// 1 ft = 12 in.
const FOOT_METRE: ExactFactor = must(INCH_METRE.checked_mul(ExactFactor::new(12, 1)));
/// 1 lbm = 0.45359237 kg. International avoirdupois pound, 1959.
const POUND_MASS_KG: ExactFactor = ExactFactor::new(45_359_237, 100_000_000);
/// Standard gravity, 9.80665 m/s² = 196133/20000.
const STANDARD_GRAVITY_SI: ExactFactor = ExactFactor::new(196_133, 20_000);
/// 1 lbf = 1 lbm × g_n.
const POUND_FORCE_NEWTON: ExactFactor = must(POUND_MASS_KG.checked_mul(STANDARD_GRAVITY_SI));
const KILO: ExactFactor = ExactFactor::new(1_000, 1);
const MEGA: ExactFactor = ExactFactor::new(1_000_000, 1);
const GIGA: ExactFactor = ExactFactor::new(1_000_000_000, 1);
const CENTI: ExactFactor = ExactFactor::new(1, 100);
const MILLI: ExactFactor = ExactFactor::new(1, 1_000);
const KIP_NEWTON: ExactFactor = must(POUND_FORCE_NEWTON.checked_mul(KILO));
const INCH_SQUARED: ExactFactor = must(INCH_METRE.checked_pow(2));
const FOOT_SQUARED: ExactFactor = must(FOOT_METRE.checked_pow(2));
const FOOT_CUBED: ExactFactor = must(FOOT_METRE.checked_pow(3));
const PSI_PASCAL: ExactFactor = must(POUND_FORCE_NEWTON.checked_div(INCH_SQUARED));
const KSI_PASCAL: ExactFactor = must(PSI_PASCAL.checked_mul(KILO));
const PSF_PASCAL: ExactFactor = must(POUND_FORCE_NEWTON.checked_div(FOOT_SQUARED));
const KSF_PASCAL: ExactFactor = must(PSF_PASCAL.checked_mul(KILO));
const PLF_NEWTON_PER_METRE: ExactFactor = must(POUND_FORCE_NEWTON.checked_div(FOOT_METRE));
const KLF_NEWTON_PER_METRE: ExactFactor = must(KIP_NEWTON.checked_div(FOOT_METRE));
const LBF_INCH: ExactFactor = must(POUND_FORCE_NEWTON.checked_mul(INCH_METRE));
const LBF_FOOT: ExactFactor = must(POUND_FORCE_NEWTON.checked_mul(FOOT_METRE));
const KIP_INCH: ExactFactor = must(KIP_NEWTON.checked_mul(INCH_METRE));
const KIP_FOOT: ExactFactor = must(KIP_NEWTON.checked_mul(FOOT_METRE));
/// 1 kip·s²/in in kilograms. Coherent with kip and in/s²; gravity is not used.
const KIP_MASS: ExactFactor = must(KIP_NEWTON.checked_div(INCH_METRE));
/// 1 pcf as lbm/ft³, in kg/m³. Force density is not this unit.
const PCF_KG_PER_M3: ExactFactor = must(POUND_MASS_KG.checked_div(FOOT_CUBED));
/// Interval: Δ°F = 5/9 K.
const FAHRENHEIT_SCALE: ExactFactor = ExactFactor::new(5, 9);
/// 1/°F expressed per kelvin, from ε = α ΔT.
const PER_FAHRENHEIT: ExactFactor = ExactFactor::new(9, 5);
/// 273.15 K, the Celsius zero on the kelvin scale.
const CELSIUS_OFFSET_KELVIN: ExactFactor = ExactFactor::new(5_463, 20);
/// Addend such that K = °F × 5/9 + this offset. Equal to 273.15 − 32 × 5/9.
const FAHRENHEIT_OFFSET_KELVIN: ExactFactor = must(
    CELSIUS_OFFSET_KELVIN.checked_sub(must(ExactFactor::new(32, 1).checked_mul(FAHRENHEIT_SCALE))),
);

/// Physical dimension of a catalog unit.
///
/// Stress and force per area share [`Dimension::ForcePerArea`]. Splitting them
/// into incompatible dimensions is a decision still open for Jonathan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dimension {
    /// Length. Canonical SI: metre.
    Length,
    /// Force. Canonical SI: newton.
    Force,
    /// Moment. Canonical SI: newton metre.
    Moment,
    /// Stress or force per area. Canonical SI: pascal.
    ForcePerArea,
    /// Force per length. Canonical SI: newton per metre.
    ForcePerLength,
    /// Mass. Canonical SI: kilogram.
    Mass,
    /// Mass density. Canonical SI: kilogram per cubic metre.
    Density,
    /// Acceleration. Canonical SI: metre per second squared.
    Acceleration,
    /// Temperature. Intervals are linear; absolute readings use the offset.
    Temperature,
    /// Thermal expansion coefficient. Canonical SI: per kelvin.
    ThermalCoefficient,
}

/// Which closed list a unit belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitSystem {
    /// Système international.
    Si,
    /// United States customary units.
    Us,
}

/// One row of [`CONVERSION_TABLE`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConversionRow {
    /// Catalog unit of this row.
    pub unit: CatalogUnit,
    /// SI or US list.
    pub system: UnitSystem,
    /// Dimension used to accept or reject a conversion.
    pub dimension: Dimension,
    /// Canonical symbol, case-sensitive.
    pub symbol: &'static str,
    /// Exact scale into the canonical SI unit of `dimension`.
    pub factor: ExactFactor,
    /// Absolute-temperature addend in kelvin, after the scale is applied.
    ///
    /// `None` for intervals and for every unit that is not a temperature.
    pub absolute_offset: Option<ExactFactor>,
}

/// Closed unit. Unknown text has no variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CatalogUnit {
    /// Metre (m). Canonical SI length.
    Meter,
    /// Centimetre (cm).
    Centimeter,
    /// Millimetre (mm).
    Millimeter,
    /// Inch (in). 1 in = 0.0254 m.
    Inch,
    /// International foot (ft). 1 ft = 12 in. The survey foot is not this unit.
    Foot,
    /// Newton (N). Canonical SI force.
    Newton,
    /// Kilonewton (kN).
    Kilonewton,
    /// Meganewton (MN).
    Meganewton,
    /// Pound-force (lbf). Not the pound-mass.
    PoundForce,
    /// Kip. 1 kip = 1000 lbf.
    Kip,
    /// Newton metre (N·m). Canonical SI moment. Anchor of the table; not a new input-schema symbol.
    NewtonMeter,
    /// Pound-force inch (lbf·in).
    PoundForceInch,
    /// Pound-force foot (lbf·ft).
    PoundForceFoot,
    /// Kip inch (kip·in).
    KipInch,
    /// Kip foot (kip·ft).
    KipFoot,
    /// Pascal (Pa). Canonical SI stress and force per area.
    Pascal,
    /// Kilopascal (kPa).
    Kilopascal,
    /// Megapascal (MPa).
    Megapascal,
    /// Gigapascal (GPa).
    Gigapascal,
    /// Kilonewton per square metre (kN/m²). Same dimension as the pascal.
    KilonewtonPerSquareMetre,
    /// Pound-force per square inch (psi).
    Psi,
    /// Kip per square inch (ksi).
    Ksi,
    /// Pound-force per square foot (psf).
    Psf,
    /// Kip per square foot (ksf).
    Ksf,
    /// Kilonewton per metre (kN/m).
    KilonewtonPerMetre,
    /// Pound-force per foot (plf).
    PoundForcePerFoot,
    /// Kip per foot (klf).
    KipPerFoot,
    /// Kilogram (kg). Canonical SI mass.
    Kilogram,
    /// Tonne (t). 1 t = 1000 kg. Not the short ton.
    Tonne,
    /// Pound-mass (lbm). Not the pound-force.
    PoundMass,
    /// Kip-second-squared per inch. Coherent mass for kip and in/s².
    KipSecondSquaredPerInch,
    /// Kilogram per cubic metre (kg/m³).
    KilogramPerCubicMetre,
    /// Pound-mass per cubic foot (pcf). Mass density, not a specific weight.
    PoundMassPerCubicFoot,
    /// Metre per second squared (m/s²).
    MeterPerSecondSquared,
    /// Inch per second squared (in/s²).
    InchPerSecondSquared,
    /// Foot per second squared (ft/s²).
    FootPerSecondSquared,
    /// Standard gravity as an acceleration unit: 1 g = 9.80665 m/s². Not the gram, and not an implicit conversion from mass to force.
    StandardGravity,
    /// Degree Celsius (°C). The scale factor converts an interval; the absolute offset is separate.
    Celsius,
    /// Degree Fahrenheit (°F). An interval uses 5/9. An absolute reading also removes the 32 degree offset.
    Fahrenheit,
    /// Per degree Celsius (1/°C). Equal to 1/K on an interval.
    PerCelsius,
    /// Per degree Fahrenheit (1/°F): 9/5 per kelvin, preserving α ΔT.
    PerFahrenheit,
}

impl CatalogUnit {
    /// Every unit, in catalog order.
    pub const ALL: &'static [CatalogUnit] = &[
        CatalogUnit::Meter,
        CatalogUnit::Centimeter,
        CatalogUnit::Millimeter,
        CatalogUnit::Inch,
        CatalogUnit::Foot,
        CatalogUnit::Newton,
        CatalogUnit::Kilonewton,
        CatalogUnit::Meganewton,
        CatalogUnit::PoundForce,
        CatalogUnit::Kip,
        CatalogUnit::NewtonMeter,
        CatalogUnit::PoundForceInch,
        CatalogUnit::PoundForceFoot,
        CatalogUnit::KipInch,
        CatalogUnit::KipFoot,
        CatalogUnit::Pascal,
        CatalogUnit::Kilopascal,
        CatalogUnit::Megapascal,
        CatalogUnit::Gigapascal,
        CatalogUnit::KilonewtonPerSquareMetre,
        CatalogUnit::Psi,
        CatalogUnit::Ksi,
        CatalogUnit::Psf,
        CatalogUnit::Ksf,
        CatalogUnit::KilonewtonPerMetre,
        CatalogUnit::PoundForcePerFoot,
        CatalogUnit::KipPerFoot,
        CatalogUnit::Kilogram,
        CatalogUnit::Tonne,
        CatalogUnit::PoundMass,
        CatalogUnit::KipSecondSquaredPerInch,
        CatalogUnit::KilogramPerCubicMetre,
        CatalogUnit::PoundMassPerCubicFoot,
        CatalogUnit::MeterPerSecondSquared,
        CatalogUnit::InchPerSecondSquared,
        CatalogUnit::FootPerSecondSquared,
        CatalogUnit::StandardGravity,
        CatalogUnit::Celsius,
        CatalogUnit::Fahrenheit,
        CatalogUnit::PerCelsius,
        CatalogUnit::PerFahrenheit,
    ];

    /// Canonical symbol.
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Meter => "m",
            Self::Centimeter => "cm",
            Self::Millimeter => "mm",
            Self::Inch => "in",
            Self::Foot => "ft",
            Self::Newton => "N",
            Self::Kilonewton => "kN",
            Self::Meganewton => "MN",
            Self::PoundForce => "lbf",
            Self::Kip => "kip",
            Self::NewtonMeter => "N·m",
            Self::PoundForceInch => "lbf·in",
            Self::PoundForceFoot => "lbf·ft",
            Self::KipInch => "kip·in",
            Self::KipFoot => "kip·ft",
            Self::Pascal => "Pa",
            Self::Kilopascal => "kPa",
            Self::Megapascal => "MPa",
            Self::Gigapascal => "GPa",
            Self::KilonewtonPerSquareMetre => "kN/m²",
            Self::Psi => "psi",
            Self::Ksi => "ksi",
            Self::Psf => "psf",
            Self::Ksf => "ksf",
            Self::KilonewtonPerMetre => "kN/m",
            Self::PoundForcePerFoot => "plf",
            Self::KipPerFoot => "klf",
            Self::Kilogram => "kg",
            Self::Tonne => "t",
            Self::PoundMass => "lbm",
            Self::KipSecondSquaredPerInch => "kip·s²/in",
            Self::KilogramPerCubicMetre => "kg/m³",
            Self::PoundMassPerCubicFoot => "pcf",
            Self::MeterPerSecondSquared => "m/s²",
            Self::InchPerSecondSquared => "in/s²",
            Self::FootPerSecondSquared => "ft/s²",
            Self::StandardGravity => "g",
            Self::Celsius => "°C",
            Self::Fahrenheit => "°F",
            Self::PerCelsius => "1/°C",
            Self::PerFahrenheit => "1/°F",
        }
    }

    /// ASCII spellings of the same unit. Empty when the symbol is already ASCII.
    pub const fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Meter => &[],
            Self::Centimeter => &[],
            Self::Millimeter => &[],
            Self::Inch => &[],
            Self::Foot => &[],
            Self::Newton => &[],
            Self::Kilonewton => &[],
            Self::Meganewton => &[],
            Self::PoundForce => &[],
            Self::Kip => &[],
            Self::NewtonMeter => &["N*m", "N.m", "Nm"],
            Self::PoundForceInch => &["lbf*in"],
            Self::PoundForceFoot => &["lbf*ft"],
            Self::KipInch => &["kip*in"],
            Self::KipFoot => &["kip*ft"],
            Self::Pascal => &[],
            Self::Kilopascal => &[],
            Self::Megapascal => &[],
            Self::Gigapascal => &[],
            Self::KilonewtonPerSquareMetre => &["kN/m2"],
            Self::Psi => &[],
            Self::Ksi => &[],
            Self::Psf => &[],
            Self::Ksf => &[],
            Self::KilonewtonPerMetre => &[],
            Self::PoundForcePerFoot => &[],
            Self::KipPerFoot => &[],
            Self::Kilogram => &[],
            Self::Tonne => &[],
            Self::PoundMass => &[],
            Self::KipSecondSquaredPerInch => &["kip·s2/in", "kip*s2/in", "kip*s²/in"],
            Self::KilogramPerCubicMetre => &["kg/m3"],
            Self::PoundMassPerCubicFoot => &[],
            Self::MeterPerSecondSquared => &["m/s2"],
            Self::InchPerSecondSquared => &["in/s2"],
            Self::FootPerSecondSquared => &["ft/s2"],
            Self::StandardGravity => &[],
            Self::Celsius => &["degC"],
            Self::Fahrenheit => &["degF"],
            Self::PerCelsius => &["1/degC"],
            Self::PerFahrenheit => &["1/degF"],
        }
    }

    /// SI or US list.
    pub const fn system(self) -> UnitSystem {
        match self {
            Self::Meter => UnitSystem::Si,
            Self::Centimeter => UnitSystem::Si,
            Self::Millimeter => UnitSystem::Si,
            Self::Inch => UnitSystem::Us,
            Self::Foot => UnitSystem::Us,
            Self::Newton => UnitSystem::Si,
            Self::Kilonewton => UnitSystem::Si,
            Self::Meganewton => UnitSystem::Si,
            Self::PoundForce => UnitSystem::Us,
            Self::Kip => UnitSystem::Us,
            Self::NewtonMeter => UnitSystem::Si,
            Self::PoundForceInch => UnitSystem::Us,
            Self::PoundForceFoot => UnitSystem::Us,
            Self::KipInch => UnitSystem::Us,
            Self::KipFoot => UnitSystem::Us,
            Self::Pascal => UnitSystem::Si,
            Self::Kilopascal => UnitSystem::Si,
            Self::Megapascal => UnitSystem::Si,
            Self::Gigapascal => UnitSystem::Si,
            Self::KilonewtonPerSquareMetre => UnitSystem::Si,
            Self::Psi => UnitSystem::Us,
            Self::Ksi => UnitSystem::Us,
            Self::Psf => UnitSystem::Us,
            Self::Ksf => UnitSystem::Us,
            Self::KilonewtonPerMetre => UnitSystem::Si,
            Self::PoundForcePerFoot => UnitSystem::Us,
            Self::KipPerFoot => UnitSystem::Us,
            Self::Kilogram => UnitSystem::Si,
            Self::Tonne => UnitSystem::Si,
            Self::PoundMass => UnitSystem::Us,
            Self::KipSecondSquaredPerInch => UnitSystem::Us,
            Self::KilogramPerCubicMetre => UnitSystem::Si,
            Self::PoundMassPerCubicFoot => UnitSystem::Us,
            Self::MeterPerSecondSquared => UnitSystem::Si,
            Self::InchPerSecondSquared => UnitSystem::Us,
            Self::FootPerSecondSquared => UnitSystem::Us,
            Self::StandardGravity => UnitSystem::Us,
            Self::Celsius => UnitSystem::Si,
            Self::Fahrenheit => UnitSystem::Us,
            Self::PerCelsius => UnitSystem::Si,
            Self::PerFahrenheit => UnitSystem::Us,
        }
    }

    /// Dimension of the unit.
    pub const fn dimension(self) -> Dimension {
        match self {
            Self::Meter => Dimension::Length,
            Self::Centimeter => Dimension::Length,
            Self::Millimeter => Dimension::Length,
            Self::Inch => Dimension::Length,
            Self::Foot => Dimension::Length,
            Self::Newton => Dimension::Force,
            Self::Kilonewton => Dimension::Force,
            Self::Meganewton => Dimension::Force,
            Self::PoundForce => Dimension::Force,
            Self::Kip => Dimension::Force,
            Self::NewtonMeter => Dimension::Moment,
            Self::PoundForceInch => Dimension::Moment,
            Self::PoundForceFoot => Dimension::Moment,
            Self::KipInch => Dimension::Moment,
            Self::KipFoot => Dimension::Moment,
            Self::Pascal => Dimension::ForcePerArea,
            Self::Kilopascal => Dimension::ForcePerArea,
            Self::Megapascal => Dimension::ForcePerArea,
            Self::Gigapascal => Dimension::ForcePerArea,
            Self::KilonewtonPerSquareMetre => Dimension::ForcePerArea,
            Self::Psi => Dimension::ForcePerArea,
            Self::Ksi => Dimension::ForcePerArea,
            Self::Psf => Dimension::ForcePerArea,
            Self::Ksf => Dimension::ForcePerArea,
            Self::KilonewtonPerMetre => Dimension::ForcePerLength,
            Self::PoundForcePerFoot => Dimension::ForcePerLength,
            Self::KipPerFoot => Dimension::ForcePerLength,
            Self::Kilogram => Dimension::Mass,
            Self::Tonne => Dimension::Mass,
            Self::PoundMass => Dimension::Mass,
            Self::KipSecondSquaredPerInch => Dimension::Mass,
            Self::KilogramPerCubicMetre => Dimension::Density,
            Self::PoundMassPerCubicFoot => Dimension::Density,
            Self::MeterPerSecondSquared => Dimension::Acceleration,
            Self::InchPerSecondSquared => Dimension::Acceleration,
            Self::FootPerSecondSquared => Dimension::Acceleration,
            Self::StandardGravity => Dimension::Acceleration,
            Self::Celsius => Dimension::Temperature,
            Self::Fahrenheit => Dimension::Temperature,
            Self::PerCelsius => Dimension::ThermalCoefficient,
            Self::PerFahrenheit => Dimension::ThermalCoefficient,
        }
    }

    /// Exact scale into the canonical SI unit of [`Self::dimension`].
    pub const fn si_factor(self) -> ExactFactor {
        match self {
            Self::Meter => ExactFactor::ONE,
            Self::Centimeter => CENTI,
            Self::Millimeter => MILLI,
            Self::Inch => INCH_METRE,
            Self::Foot => FOOT_METRE,
            Self::Newton => ExactFactor::ONE,
            Self::Kilonewton => KILO,
            Self::Meganewton => MEGA,
            Self::PoundForce => POUND_FORCE_NEWTON,
            Self::Kip => KIP_NEWTON,
            Self::NewtonMeter => ExactFactor::ONE,
            Self::PoundForceInch => LBF_INCH,
            Self::PoundForceFoot => LBF_FOOT,
            Self::KipInch => KIP_INCH,
            Self::KipFoot => KIP_FOOT,
            Self::Pascal => ExactFactor::ONE,
            Self::Kilopascal => KILO,
            Self::Megapascal => MEGA,
            Self::Gigapascal => GIGA,
            Self::KilonewtonPerSquareMetre => KILO,
            Self::Psi => PSI_PASCAL,
            Self::Ksi => KSI_PASCAL,
            Self::Psf => PSF_PASCAL,
            Self::Ksf => KSF_PASCAL,
            Self::KilonewtonPerMetre => KILO,
            Self::PoundForcePerFoot => PLF_NEWTON_PER_METRE,
            Self::KipPerFoot => KLF_NEWTON_PER_METRE,
            Self::Kilogram => ExactFactor::ONE,
            Self::Tonne => KILO,
            Self::PoundMass => POUND_MASS_KG,
            Self::KipSecondSquaredPerInch => KIP_MASS,
            Self::KilogramPerCubicMetre => ExactFactor::ONE,
            Self::PoundMassPerCubicFoot => PCF_KG_PER_M3,
            Self::MeterPerSecondSquared => ExactFactor::ONE,
            Self::InchPerSecondSquared => INCH_METRE,
            Self::FootPerSecondSquared => FOOT_METRE,
            Self::StandardGravity => STANDARD_GRAVITY_SI,
            Self::Celsius => ExactFactor::ONE,
            Self::Fahrenheit => FAHRENHEIT_SCALE,
            Self::PerCelsius => ExactFactor::ONE,
            Self::PerFahrenheit => PER_FAHRENHEIT,
        }
    }

    /// Recognizes a canonical symbol or a listed ASCII alias.
    ///
    /// The match is exact and case-sensitive. Spaces are not trimmed.
    /// `"lb"` is unknown: it is neither [`Self::PoundMass`] nor
    /// [`Self::PoundForce`].
    ///
    /// # Errors
    ///
    /// Returns `ENG-E-0001` when the text is not in the catalog. The
    /// message includes the received text.
    pub fn parse(text: &str) -> EResult<Self> {
        match text {
            "m" => Ok(Self::Meter),
            "cm" => Ok(Self::Centimeter),
            "mm" => Ok(Self::Millimeter),
            "in" => Ok(Self::Inch),
            "ft" => Ok(Self::Foot),
            "N" => Ok(Self::Newton),
            "kN" => Ok(Self::Kilonewton),
            "MN" => Ok(Self::Meganewton),
            "lbf" => Ok(Self::PoundForce),
            "kip" => Ok(Self::Kip),
            "N·m" => Ok(Self::NewtonMeter),
            "N*m" => Ok(Self::NewtonMeter),
            "N.m" => Ok(Self::NewtonMeter),
            "Nm" => Ok(Self::NewtonMeter),
            "lbf·in" => Ok(Self::PoundForceInch),
            "lbf*in" => Ok(Self::PoundForceInch),
            "lbf·ft" => Ok(Self::PoundForceFoot),
            "lbf*ft" => Ok(Self::PoundForceFoot),
            "kip·in" => Ok(Self::KipInch),
            "kip*in" => Ok(Self::KipInch),
            "kip·ft" => Ok(Self::KipFoot),
            "kip*ft" => Ok(Self::KipFoot),
            "Pa" => Ok(Self::Pascal),
            "kPa" => Ok(Self::Kilopascal),
            "MPa" => Ok(Self::Megapascal),
            "GPa" => Ok(Self::Gigapascal),
            "kN/m²" => Ok(Self::KilonewtonPerSquareMetre),
            "kN/m2" => Ok(Self::KilonewtonPerSquareMetre),
            "psi" => Ok(Self::Psi),
            "ksi" => Ok(Self::Ksi),
            "psf" => Ok(Self::Psf),
            "ksf" => Ok(Self::Ksf),
            "kN/m" => Ok(Self::KilonewtonPerMetre),
            "plf" => Ok(Self::PoundForcePerFoot),
            "klf" => Ok(Self::KipPerFoot),
            "kg" => Ok(Self::Kilogram),
            "t" => Ok(Self::Tonne),
            "lbm" => Ok(Self::PoundMass),
            "kip·s²/in" => Ok(Self::KipSecondSquaredPerInch),
            "kip·s2/in" => Ok(Self::KipSecondSquaredPerInch),
            "kip*s2/in" => Ok(Self::KipSecondSquaredPerInch),
            "kip*s²/in" => Ok(Self::KipSecondSquaredPerInch),
            "kg/m³" => Ok(Self::KilogramPerCubicMetre),
            "kg/m3" => Ok(Self::KilogramPerCubicMetre),
            "pcf" => Ok(Self::PoundMassPerCubicFoot),
            "m/s²" => Ok(Self::MeterPerSecondSquared),
            "m/s2" => Ok(Self::MeterPerSecondSquared),
            "in/s²" => Ok(Self::InchPerSecondSquared),
            "in/s2" => Ok(Self::InchPerSecondSquared),
            "ft/s²" => Ok(Self::FootPerSecondSquared),
            "ft/s2" => Ok(Self::FootPerSecondSquared),
            "g" => Ok(Self::StandardGravity),
            "°C" => Ok(Self::Celsius),
            "degC" => Ok(Self::Celsius),
            "°F" => Ok(Self::Fahrenheit),
            "degF" => Ok(Self::Fahrenheit),
            "1/°C" => Ok(Self::PerCelsius),
            "1/degC" => Ok(Self::PerCelsius),
            "1/°F" => Ok(Self::PerFahrenheit),
            "1/degF" => Ok(Self::PerFahrenheit),
            _ => Err(EngineError::new(
                "ENG-E-0001",
                format!("The specified unit is not recognized: «{text}»."),
            )),
        }
    }
}

impl CatalogUnit {
    /// Kelvin addend of an absolute temperature, or `None` for every other unit.
    ///
    /// For Fahrenheit the stored addend is `273.15 − 32 × 5/9`. Absolute
    /// conversion evaluates `(°F − 32) × 5/9 + 273.15` instead of adding the
    /// rounded addend, so that 32 °F lands on 0 °C.
    pub const fn absolute_offset(self) -> Option<ExactFactor> {
        match self {
            Self::Celsius => Some(CELSIUS_OFFSET_KELVIN),
            Self::Fahrenheit => Some(FAHRENHEIT_OFFSET_KELVIN),
            Self::Meter
            | Self::Centimeter
            | Self::Millimeter
            | Self::Inch
            | Self::Foot
            | Self::Newton
            | Self::Kilonewton
            | Self::Meganewton
            | Self::PoundForce
            | Self::Kip
            | Self::NewtonMeter
            | Self::PoundForceInch
            | Self::PoundForceFoot
            | Self::KipInch
            | Self::KipFoot
            | Self::Pascal
            | Self::Kilopascal
            | Self::Megapascal
            | Self::Gigapascal
            | Self::KilonewtonPerSquareMetre
            | Self::Psi
            | Self::Ksi
            | Self::Psf
            | Self::Ksf
            | Self::KilonewtonPerMetre
            | Self::PoundForcePerFoot
            | Self::KipPerFoot
            | Self::Kilogram
            | Self::Tonne
            | Self::PoundMass
            | Self::KipSecondSquaredPerInch
            | Self::KilogramPerCubicMetre
            | Self::PoundMassPerCubicFoot
            | Self::MeterPerSecondSquared
            | Self::InchPerSecondSquared
            | Self::FootPerSecondSquared
            | Self::StandardGravity
            | Self::PerCelsius
            | Self::PerFahrenheit => None,
        }
    }

    /// Row of this unit in [`CONVERSION_TABLE`].
    pub const fn row(self) -> ConversionRow {
        ConversionRow {
            unit: self,
            system: self.system(),
            dimension: self.dimension(),
            symbol: self.symbol(),
            factor: self.si_factor(),
            absolute_offset: self.absolute_offset(),
        }
    }
}

/// Number of rows in [`CONVERSION_TABLE`].
pub const UNIT_COUNT: usize = CatalogUnit::ALL.len();

const fn build_conversion_table() -> [ConversionRow; UNIT_COUNT] {
    let mut rows = [CatalogUnit::Meter.row(); UNIT_COUNT];
    let mut index = 0;
    while index < UNIT_COUNT {
        rows[index] = CatalogUnit::ALL[index].row();
        index += 1;
    }
    rows
}

/// The versioned table, one row per [`CatalogUnit`], in [`CatalogUnit::ALL`] order.
pub const CONVERSION_TABLE: [ConversionRow; UNIT_COUNT] = build_conversion_table();

/// Name of the canonical SI unit of a dimension.
///
/// Newton per metre, kelvin and per kelvin are named here even though the
/// closed symbol list uses kN/m, °C and 1/°C for those dimensions.
pub const fn canonical_si_symbol(dimension: Dimension) -> &'static str {
    match dimension {
        Dimension::Length => "m",
        Dimension::Force => "N",
        Dimension::Moment => "N·m",
        Dimension::ForcePerArea => "Pa",
        Dimension::ForcePerLength => "N/m",
        Dimension::Mass => "kg",
        Dimension::Density => "kg/m³",
        Dimension::Acceleration => "m/s²",
        Dimension::Temperature => "K",
        Dimension::ThermalCoefficient => "1/K",
    }
}

/// Multiplies a value by the unit's SI factor.
///
/// The caller validates finiteness. For `ENG-E-0008`, use [`convert`].
pub fn to_si(value: Real, unit: CatalogUnit) -> Real {
    value * unit.si_factor().to_f64()
}

/// Divides an SI value by the unit's factor.
///
/// The caller validates finiteness. For `ENG-E-0008`, use [`convert`].
pub fn from_si(value: Real, unit: CatalogUnit) -> Real {
    value / unit.si_factor().to_f64()
}

/// Converts two units of the same dimension.
///
/// Temperature uses the interval scale and does not apply the 32 °F offset.
/// Absolute temperatures go through [`convert_temperature`].
///
/// # Errors
///
/// `ENG-E-0008` for a non-finite input or result. `ENG-E-0010` when the
/// dimensions differ. Mass and force are different dimensions; this function
/// does not apply gravity.
pub fn convert(value: Real, from: CatalogUnit, to: CatalogUnit) -> EResult<Real> {
    if !value.is_finite() {
        return Err(nonfinite(value));
    }
    if from.dimension() != to.dimension() {
        return Err(incompatible(from, to));
    }
    let result = value * (from.si_factor().to_f64() / to.si_factor().to_f64());
    if !result.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            format!(
                "Conversion from «{}» to «{}» produces a non-finite quantity.",
                from.symbol(),
                to.symbol()
            ),
        ));
    }
    Ok(result)
}

fn incompatible(from: CatalogUnit, to: CatalogUnit) -> EngineError {
    EngineError::new(
        "ENG-E-0010",
        format!(
            "The physical dimensions of units «{}» and «{}» are incompatible.",
            from.symbol(),
            to.symbol()
        ),
    )
}

fn nonfinite(value: Real) -> EngineError {
    EngineError::new(
        "ENG-E-0008",
        format!("The input quantity is not finite: {value}."),
    )
}

/// Whether a temperature value is an interval or an absolute reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TemperatureReading {
    /// Difference of temperatures. The scale factor is used and the offset is not.
    Interval,
    /// Thermodynamic temperature. The offset is applied around the scale.
    Absolute,
}

/// Converts a temperature interval or an absolute temperature.
///
/// # Errors
///
/// `ENG-E-0010` when either unit is not a temperature. `ENG-E-0008` when the
/// input or the result is not finite.
pub fn convert_temperature(
    value: Real,
    from: CatalogUnit,
    to: CatalogUnit,
    reading: TemperatureReading,
) -> EResult<Real> {
    if from.dimension() != Dimension::Temperature || to.dimension() != Dimension::Temperature {
        return Err(incompatible(from, to));
    }
    match reading {
        TemperatureReading::Interval => convert(value, from, to),
        TemperatureReading::Absolute => {
            let kelvin = absolute_to_kelvin(value, from)?;
            absolute_from_kelvin(kelvin, to)
        }
    }
}

/// Converts an absolute catalog temperature to kelvin.
///
/// # Errors
///
/// `ENG-E-0010` when `unit` is not a temperature. `ENG-E-0008` when the input
/// or the kelvin value is not finite.
pub fn absolute_to_kelvin(value: Real, unit: CatalogUnit) -> EResult<Real> {
    if !value.is_finite() {
        return Err(nonfinite(value));
    }
    let kelvin = match unit {
        CatalogUnit::Celsius => value + CELSIUS_OFFSET_KELVIN.to_f64(),
        CatalogUnit::Fahrenheit => {
            (value - 32.0) * FAHRENHEIT_SCALE.to_f64() + CELSIUS_OFFSET_KELVIN.to_f64()
        }
        other => {
            return Err(EngineError::new(
                "ENG-E-0010",
                format!(
                    "Absolute temperature does not apply to «{}».",
                    other.symbol()
                ),
            ));
        }
    };
    if !kelvin.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Absolute temperature conversion produces a non-finite quantity.".to_string(),
        ));
    }
    Ok(kelvin)
}

/// Converts kelvin to an absolute catalog temperature.
///
/// # Errors
///
/// `ENG-E-0010` when `unit` is not a temperature. `ENG-E-0008` when the input
/// or the result is not finite.
pub fn absolute_from_kelvin(kelvin: Real, unit: CatalogUnit) -> EResult<Real> {
    if !kelvin.is_finite() {
        return Err(nonfinite(kelvin));
    }
    let value = match unit {
        CatalogUnit::Celsius => kelvin - CELSIUS_OFFSET_KELVIN.to_f64(),
        CatalogUnit::Fahrenheit => {
            let celsius = kelvin - CELSIUS_OFFSET_KELVIN.to_f64();
            celsius / FAHRENHEIT_SCALE.to_f64() + 32.0
        }
        other => {
            return Err(EngineError::new(
                "ENG-E-0010",
                format!(
                    "Absolute temperature does not apply to «{}».",
                    other.symbol()
                ),
            ));
        }
    };
    if !value.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Absolute temperature conversion produces a non-finite quantity.".to_string(),
        ));
    }
    Ok(value)
}

/// Acceleration supplied by the caller. Absence means the conversion is refused.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeclaredAcceleration {
    /// Magnitude, expressed in `unit`.
    pub value: Real,
    /// Must be an acceleration unit.
    pub unit: CatalogUnit,
}

/// Converts mass to force only when an acceleration is declared.
///
/// `None` is an error for every mass/force pair, including lbm with lbf and
/// kip·s²/in with kip. The function does not insert 9.80665.
///
/// # Errors
///
/// `ENG-E-0010` when the units are not mass and force, when the acceleration
/// is omitted, or when the declared unit is not an acceleration. The omitted
/// case names both units. `ENG-E-0008` when a quantity is not finite.
pub fn force_from_mass(
    mass: Real,
    mass_unit: CatalogUnit,
    force_unit: CatalogUnit,
    acceleration: Option<DeclaredAcceleration>,
) -> EResult<Real> {
    if !mass.is_finite() {
        return Err(nonfinite(mass));
    }
    if mass_unit.dimension() != Dimension::Mass || force_unit.dimension() != Dimension::Force {
        return Err(incompatible(mass_unit, force_unit));
    }
    let Some(acceleration) = acceleration else {
        return Err(EngineError::new(
            "ENG-E-0010",
            format!(
                "Mass «{}» and force «{}» were mixed without a declared acceleration.",
                mass_unit.symbol(),
                force_unit.symbol()
            ),
        ));
    };
    if acceleration.unit.dimension() != Dimension::Acceleration {
        return Err(EngineError::new(
            "ENG-E-0010",
            format!(
                "Mass «{}» and force «{}» need an acceleration, and «{}» is not one.",
                mass_unit.symbol(),
                force_unit.symbol(),
                acceleration.unit.symbol()
            ),
        ));
    }
    if !acceleration.value.is_finite() {
        return Err(nonfinite(acceleration.value));
    }
    let newton = to_si(mass, mass_unit) * to_si(acceleration.value, acceleration.unit);
    if !newton.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Conversion from mass to force produces a non-finite quantity.".to_string(),
        ));
    }
    convert(newton, CatalogUnit::Newton, force_unit)
}

/// Closed preset. A name for a coherent set, not an input-schema field.
///
/// `"lb-in"` names pound-force and inch. It does not make the bare text `"lb"`
/// a unit. A mass that is not in the catalog stays unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitPreset {
    /// Force N, length m.
    NewtonMeter,
    /// Force kN, length m.
    KilonewtonMeter,
    /// Force N, length mm.
    NewtonMillimeter,
    /// Force kip, length in.
    KipInch,
    /// Force kip, length ft.
    KipFoot,
    /// Force lbf, length in. The preset name says lb; the force unit is lbf.
    PoundInch,
    /// Force lbf, length ft. The preset name says lb; the force unit is lbf.
    PoundFoot,
}

/// Units named by a [`UnitPreset`]. `None` is not filled from gravity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresetDefinition {
    /// Preset that produced this definition.
    pub preset: UnitPreset,
    /// Force unit.
    pub force: CatalogUnit,
    /// Length unit.
    pub length: CatalogUnit,
    /// Moment unit when force × length is itself a catalog symbol.
    pub moment: Option<CatalogUnit>,
    /// Force per area, when the coherent unit is in the catalog.
    pub force_per_area: Option<CatalogUnit>,
    /// Force per length, when the coherent unit is in the catalog.
    pub force_per_length: Option<CatalogUnit>,
    /// Coherent mass, when that unit is in the catalog.
    pub mass: Option<CatalogUnit>,
    /// Coherent mass density, when that unit is in the catalog.
    pub density: Option<CatalogUnit>,
    /// Kinematic acceleration of the length unit, when it is in the catalog.
    pub acceleration: Option<CatalogUnit>,
    /// Temperature symbol of the preset's system.
    pub temperature: CatalogUnit,
    /// Thermal-coefficient symbol of the preset's system.
    pub thermal_coefficient: CatalogUnit,
}

impl UnitPreset {
    /// Every preset, in declaration order.
    pub const ALL: &'static [UnitPreset] = &[
        Self::NewtonMeter,
        Self::KilonewtonMeter,
        Self::NewtonMillimeter,
        Self::KipInch,
        Self::KipFoot,
        Self::PoundInch,
        Self::PoundFoot,
    ];

    /// Closed name. Hyphenated, distinct from a unit symbol.
    pub const fn name(self) -> &'static str {
        match self {
            Self::NewtonMeter => "N-m",
            Self::KilonewtonMeter => "kN-m",
            Self::NewtonMillimeter => "N-mm",
            Self::KipInch => "kip-in",
            Self::KipFoot => "kip-ft",
            Self::PoundInch => "lb-in",
            Self::PoundFoot => "lb-ft",
        }
    }

    /// Recognizes a preset name.
    ///
    /// # Errors
    ///
    /// Returns `ENG-E-0001` when the text is not one of the seven names. The
    /// message includes the received text. `"lb"` and `"SI"` are rejected.
    pub fn parse(text: &str) -> EResult<Self> {
        match text {
            "N-m" => Ok(Self::NewtonMeter),
            "kN-m" => Ok(Self::KilonewtonMeter),
            "N-mm" => Ok(Self::NewtonMillimeter),
            "kip-in" => Ok(Self::KipInch),
            "kip-ft" => Ok(Self::KipFoot),
            "lb-in" => Ok(Self::PoundInch),
            "lb-ft" => Ok(Self::PoundFoot),
            _ => Err(EngineError::new(
                "ENG-E-0001",
                format!("The specified unit preset is not recognized: «{text}»."),
            )),
        }
    }

    /// Units of this preset. Unset fields stay unset.
    pub const fn definition(self) -> PresetDefinition {
        match self {
            Self::NewtonMeter => PresetDefinition {
                preset: self,
                force: CatalogUnit::Newton,
                length: CatalogUnit::Meter,
                moment: Some(CatalogUnit::NewtonMeter),
                force_per_area: Some(CatalogUnit::Pascal),
                force_per_length: None,
                mass: Some(CatalogUnit::Kilogram),
                density: Some(CatalogUnit::KilogramPerCubicMetre),
                acceleration: Some(CatalogUnit::MeterPerSecondSquared),
                temperature: CatalogUnit::Celsius,
                thermal_coefficient: CatalogUnit::PerCelsius,
            },
            Self::KilonewtonMeter => PresetDefinition {
                preset: self,
                force: CatalogUnit::Kilonewton,
                length: CatalogUnit::Meter,
                moment: None,
                force_per_area: Some(CatalogUnit::KilonewtonPerSquareMetre),
                force_per_length: Some(CatalogUnit::KilonewtonPerMetre),
                mass: Some(CatalogUnit::Tonne),
                density: None,
                acceleration: Some(CatalogUnit::MeterPerSecondSquared),
                temperature: CatalogUnit::Celsius,
                thermal_coefficient: CatalogUnit::PerCelsius,
            },
            Self::NewtonMillimeter => PresetDefinition {
                preset: self,
                force: CatalogUnit::Newton,
                length: CatalogUnit::Millimeter,
                moment: None,
                force_per_area: Some(CatalogUnit::Megapascal),
                force_per_length: Some(CatalogUnit::KilonewtonPerMetre),
                mass: None,
                density: None,
                acceleration: None,
                temperature: CatalogUnit::Celsius,
                thermal_coefficient: CatalogUnit::PerCelsius,
            },
            Self::KipInch => PresetDefinition {
                preset: self,
                force: CatalogUnit::Kip,
                length: CatalogUnit::Inch,
                moment: Some(CatalogUnit::KipInch),
                force_per_area: Some(CatalogUnit::Ksi),
                force_per_length: None,
                mass: Some(CatalogUnit::KipSecondSquaredPerInch),
                density: None,
                acceleration: Some(CatalogUnit::InchPerSecondSquared),
                temperature: CatalogUnit::Fahrenheit,
                thermal_coefficient: CatalogUnit::PerFahrenheit,
            },
            Self::KipFoot => PresetDefinition {
                preset: self,
                force: CatalogUnit::Kip,
                length: CatalogUnit::Foot,
                moment: Some(CatalogUnit::KipFoot),
                force_per_area: Some(CatalogUnit::Ksf),
                force_per_length: Some(CatalogUnit::KipPerFoot),
                mass: None,
                density: None,
                acceleration: Some(CatalogUnit::FootPerSecondSquared),
                temperature: CatalogUnit::Fahrenheit,
                thermal_coefficient: CatalogUnit::PerFahrenheit,
            },
            Self::PoundInch => PresetDefinition {
                preset: self,
                force: CatalogUnit::PoundForce,
                length: CatalogUnit::Inch,
                moment: Some(CatalogUnit::PoundForceInch),
                force_per_area: Some(CatalogUnit::Psi),
                force_per_length: None,
                mass: None,
                density: None,
                acceleration: Some(CatalogUnit::InchPerSecondSquared),
                temperature: CatalogUnit::Fahrenheit,
                thermal_coefficient: CatalogUnit::PerFahrenheit,
            },
            Self::PoundFoot => PresetDefinition {
                preset: self,
                force: CatalogUnit::PoundForce,
                length: CatalogUnit::Foot,
                moment: Some(CatalogUnit::PoundForceFoot),
                force_per_area: Some(CatalogUnit::Psf),
                force_per_length: Some(CatalogUnit::PoundForcePerFoot),
                mass: None,
                density: None,
                acceleration: Some(CatalogUnit::FootPerSecondSquared),
                temperature: CatalogUnit::Fahrenheit,
                thermal_coefficient: CatalogUnit::PerFahrenheit,
            },
        }
    }

    /// Exact newton-metre factor of one preset moment: force factor × length factor.
    pub const fn moment_factor(self) -> ExactFactor {
        let definition = self.definition();
        must(
            definition
                .force
                .si_factor()
                .checked_mul(definition.length.si_factor()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned seed for the deterministic magnitude sample.
    const ROUND_TRIP_SEED: u64 = 0x1043_2026_1110;

    fn next_u64(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x
    }

    fn magnitude(state: &mut u64) -> Real {
        let draw = next_u64(state);
        let exponent = (draw % 16) as i32 - 6;
        let mantissa = 1.0 + ((draw >> 4) % 9_000_000) as Real / 1_000_000.0;
        let mut scale = 1.0;
        let mut step = 0;
        if exponent >= 0 {
            while step < exponent {
                scale *= 10.0;
                step += 1;
            }
        } else {
            while step < -exponent {
                scale /= 10.0;
                step += 1;
            }
        }
        mantissa * scale
    }

    fn relative_error(actual: Real, expected: Real) -> Real {
        if expected == 0.0 {
            if actual == 0.0 {
                0.0
            } else {
                Real::INFINITY
            }
        } else {
            ((actual - expected) / expected).abs()
        }
    }

    fn note(worst: &mut Real, label: &mut &'static str, symbol: &'static str, error: Real) {
        if error > *worst {
            *worst = error;
            *label = symbol;
        }
    }

    fn si_partner(dimension: Dimension) -> CatalogUnit {
        match dimension {
            Dimension::Length => CatalogUnit::Meter,
            Dimension::Force => CatalogUnit::Newton,
            Dimension::Moment => CatalogUnit::NewtonMeter,
            Dimension::ForcePerArea => CatalogUnit::Pascal,
            Dimension::ForcePerLength => CatalogUnit::KilonewtonPerMetre,
            Dimension::Mass => CatalogUnit::Kilogram,
            Dimension::Density => CatalogUnit::KilogramPerCubicMetre,
            Dimension::Acceleration => CatalogUnit::MeterPerSecondSquared,
            Dimension::Temperature => CatalogUnit::Celsius,
            Dimension::ThermalCoefficient => CatalogUnit::PerCelsius,
        }
    }

    fn us_partner(dimension: Dimension) -> CatalogUnit {
        match dimension {
            Dimension::Length => CatalogUnit::Inch,
            Dimension::Force => CatalogUnit::PoundForce,
            Dimension::Moment => CatalogUnit::PoundForceInch,
            Dimension::ForcePerArea => CatalogUnit::Psi,
            Dimension::ForcePerLength => CatalogUnit::PoundForcePerFoot,
            Dimension::Mass => CatalogUnit::PoundMass,
            Dimension::Density => CatalogUnit::PoundMassPerCubicFoot,
            Dimension::Acceleration => CatalogUnit::InchPerSecondSquared,
            Dimension::Temperature => CatalogUnit::Fahrenheit,
            Dimension::ThermalCoefficient => CatalogUnit::PerFahrenheit,
        }
    }

    fn partner(unit: CatalogUnit) -> CatalogUnit {
        match unit.system() {
            UnitSystem::Si => us_partner(unit.dimension()),
            UnitSystem::Us => si_partner(unit.dimension()),
        }
    }

    #[test]
    fn table_is_versioned_and_complete() {
        assert_eq!(TABLE_ID, "units-table/1");
        assert_eq!(TABLE_VERSION, 1);
        assert_eq!(CONVERSION_TABLE.len(), CatalogUnit::ALL.len());
        assert_eq!(CONVERSION_TABLE.len(), UNIT_COUNT);
        for (index, unit) in CatalogUnit::ALL.iter().copied().enumerate() {
            let row = CONVERSION_TABLE[index];
            assert_eq!(row, unit.row());
            assert_eq!(row.symbol, unit.symbol());
            assert_eq!(row.factor, unit.si_factor());
            assert_eq!(CatalogUnit::parse(unit.symbol()).unwrap(), unit);
            for alias in unit.aliases() {
                assert_eq!(CatalogUnit::parse(alias).unwrap(), unit);
                assert_ne!(*alias, unit.symbol());
            }
            assert!(!canonical_si_symbol(unit.dimension()).is_empty());
        }
    }

    #[test]
    fn anchors_and_derived_factors_match() {
        assert_eq!(INCH_METRE, ExactFactor::new(127, 5_000));
        assert_eq!(
            INCH_METRE.checked_mul(ExactFactor::new(12, 1)).unwrap(),
            FOOT_METRE
        );
        assert_eq!(FOOT_METRE, ExactFactor::new(381, 1_250));
        assert_eq!(POUND_MASS_KG, ExactFactor::new(45_359_237, 100_000_000));
        assert_eq!(STANDARD_GRAVITY_SI, ExactFactor::new(196_133, 20_000));
        assert_eq!(
            POUND_MASS_KG.checked_mul(STANDARD_GRAVITY_SI).unwrap(),
            POUND_FORCE_NEWTON
        );
        assert_eq!(CatalogUnit::Inch.si_factor().to_f64(), 0.0254);
        assert_eq!(
            CatalogUnit::PoundForce.si_factor().to_f64(),
            4.4482216152605
        );
        assert_eq!(CatalogUnit::Kip.si_factor().to_f64(), 4448.2216152605);
        assert_eq!(
            FAHRENHEIT_OFFSET_KELVIN,
            CELSIUS_OFFSET_KELVIN
                .checked_sub(
                    ExactFactor::new(32, 1)
                        .checked_mul(FAHRENHEIT_SCALE)
                        .unwrap()
                )
                .unwrap()
        );
        assert_eq!(CELSIUS_OFFSET_KELVIN.to_f64(), 273.15);

        let derived = [
            (CatalogUnit::Foot, FOOT_METRE),
            (CatalogUnit::PoundForce, POUND_FORCE_NEWTON),
            (CatalogUnit::Kip, KIP_NEWTON),
            (CatalogUnit::Psi, PSI_PASCAL),
            (CatalogUnit::Ksi, KSI_PASCAL),
            (CatalogUnit::Psf, PSF_PASCAL),
            (CatalogUnit::Ksf, KSF_PASCAL),
            (CatalogUnit::PoundForcePerFoot, PLF_NEWTON_PER_METRE),
            (CatalogUnit::KipPerFoot, KLF_NEWTON_PER_METRE),
            (CatalogUnit::PoundForceInch, LBF_INCH),
            (CatalogUnit::PoundForceFoot, LBF_FOOT),
            (CatalogUnit::KipInch, KIP_INCH),
            (CatalogUnit::KipFoot, KIP_FOOT),
            (CatalogUnit::KipSecondSquaredPerInch, KIP_MASS),
            (CatalogUnit::PoundMassPerCubicFoot, PCF_KG_PER_M3),
            (CatalogUnit::Fahrenheit, FAHRENHEIT_SCALE),
            (CatalogUnit::PerFahrenheit, PER_FAHRENHEIT),
            (CatalogUnit::Tonne, KILO),
            (CatalogUnit::KilonewtonPerMetre, KILO),
            (CatalogUnit::KilonewtonPerSquareMetre, KILO),
        ];
        for (unit, factor) in derived {
            assert_eq!(unit.si_factor(), factor, "{}", unit.symbol());
        }
    }

    #[test]
    fn si_us_round_trip_stays_within_bound() {
        let mut state = ROUND_TRIP_SEED;
        let mut worst = 0.0;
        let mut worst_symbol = "";
        for &unit in CatalogUnit::ALL {
            let mut samples = vec![1e-6, 1.0, 1e9];
            for _ in 0..24 {
                samples.push(magnitude(&mut state));
            }
            let signed: Vec<Real> = samples.iter().flat_map(|value| [*value, -*value]).collect();
            for value in signed {
                let through_si = from_si(to_si(value, unit), unit);
                note(
                    &mut worst,
                    &mut worst_symbol,
                    unit.symbol(),
                    relative_error(through_si, value),
                );
                let other = partner(unit);
                let across = convert(convert(value, unit, other).unwrap(), other, unit).unwrap();
                note(
                    &mut worst,
                    &mut worst_symbol,
                    unit.symbol(),
                    relative_error(across, value),
                );
            }
        }
        println!("MAM-1043 peor error relativo {worst:.16e} unidad {worst_symbol}");
        assert!(
            worst <= 1e-15,
            "peor error relativo {worst:.16e} en {worst_symbol}"
        );
    }

    #[test]
    fn same_dimension_converts_and_others_do_not() {
        for &from in CatalogUnit::ALL {
            for &to in CatalogUnit::ALL {
                let result = convert(12.5, from, to);
                if from.dimension() == to.dimension() {
                    assert!(result.is_ok(), "{} -> {}", from.symbol(), to.symbol());
                } else {
                    let error = result.unwrap_err();
                    assert_eq!(error.code, "ENG-E-0010");
                    assert!(error.message.contains(&format!("«{}»", from.symbol())));
                    assert!(error.message.contains(&format!("«{}»", to.symbol())));
                }
            }
        }
    }

    #[test]
    fn unknown_text_names_what_was_received() {
        for text in [
            "", " ", "lb", "lbs", "pound", "inch", "SI", "si", "kN-mm", "ton", "tonne", "slug",
            "gram", "G", "T", "°", "N·mm", "g ",
        ] {
            let error = CatalogUnit::parse(text).unwrap_err();
            assert_eq!(error.code, "ENG-E-0001");
            assert!(
                error.message.contains(&format!("«{text}»")),
                "message {} does not name {text}",
                error.message
            );
            let preset = UnitPreset::parse(text).unwrap_err();
            assert_eq!(preset.code, "ENG-E-0001");
            assert!(preset.message.contains(&format!("«{text}»")));
        }
        for preset_name in ["N-m", "kN-m", "N-mm", "kip-in", "kip-ft", "lb-in", "lb-ft"] {
            let error = CatalogUnit::parse(preset_name).unwrap_err();
            assert_eq!(error.code, "ENG-E-0001");
            assert!(error.message.contains(&format!("«{preset_name}»")));
            assert_eq!(UnitPreset::parse(preset_name).unwrap().name(), preset_name);
        }
        assert_eq!(
            CatalogUnit::parse("g").unwrap(),
            CatalogUnit::StandardGravity
        );
        assert_eq!(CatalogUnit::parse("lbm").unwrap(), CatalogUnit::PoundMass);
        assert_eq!(CatalogUnit::parse("lbf").unwrap(), CatalogUnit::PoundForce);
        assert_ne!(
            CatalogUnit::parse("lbm").unwrap(),
            CatalogUnit::parse("lbf").unwrap()
        );
    }

    #[test]
    fn legacy_unit_parser_does_not_grow() {
        for text in [
            "in", "ft", "kip", "lbf", "lbm", "psi", "pcf", "°F", "degC", "kg",
        ] {
            let error = super::super::Unit::parse(text).unwrap_err();
            assert_eq!(error.code, "ENG-E-0001");
            assert!(error.message.contains(&format!("«{text}»")));
        }
    }

    #[test]
    fn mass_and_force_require_a_declared_acceleration() {
        let error = force_from_mass(1.0, CatalogUnit::PoundMass, CatalogUnit::PoundForce, None)
            .unwrap_err();
        assert_eq!(error.code, "ENG-E-0010");
        assert!(error.message.contains("«lbm»"));
        assert!(error.message.contains("«lbf»"));
        assert!(error.message.contains("without a declared acceleration"));

        let with_gravity = force_from_mass(
            1.0,
            CatalogUnit::PoundMass,
            CatalogUnit::PoundForce,
            Some(DeclaredAcceleration {
                value: 1.0,
                unit: CatalogUnit::StandardGravity,
            }),
        )
        .unwrap();
        assert!(relative_error(with_gravity, 1.0) <= 1e-15);

        let coherent = force_from_mass(
            1.0,
            CatalogUnit::KipSecondSquaredPerInch,
            CatalogUnit::Kip,
            Some(DeclaredAcceleration {
                value: 1.0,
                unit: CatalogUnit::InchPerSecondSquared,
            }),
        )
        .unwrap();
        assert!(relative_error(coherent, 1.0) <= 1e-15);
        assert_eq!(
            force_from_mass(
                1.0,
                CatalogUnit::KipSecondSquaredPerInch,
                CatalogUnit::Kip,
                None,
            )
            .unwrap_err()
            .code,
            "ENG-E-0010"
        );

        let wrong = force_from_mass(
            1.0,
            CatalogUnit::PoundMass,
            CatalogUnit::PoundForce,
            Some(DeclaredAcceleration {
                value: 1.0,
                unit: CatalogUnit::PoundForce,
            }),
        )
        .unwrap_err();
        assert_eq!(wrong.code, "ENG-E-0010");
        assert!(wrong.message.contains("«lbm»"));
        assert!(wrong.message.contains("«lbf»"));
        assert_eq!(
            force_from_mass(Real::NAN, CatalogUnit::Kilogram, CatalogUnit::Newton, None)
                .unwrap_err()
                .code,
            "ENG-E-0008"
        );
    }

    #[test]
    fn fahrenheit_offset_is_not_part_of_the_interval() {
        let interval = convert(32.0, CatalogUnit::Fahrenheit, CatalogUnit::Celsius).unwrap();
        assert!(relative_error(interval, 32.0 * (5.0 / 9.0)) <= 1e-15);
        assert_ne!(interval, 0.0);
        assert_eq!(
            convert(0.0, CatalogUnit::Fahrenheit, CatalogUnit::Celsius).unwrap(),
            0.0
        );
        assert_eq!(
            convert_temperature(
                32.0,
                CatalogUnit::Fahrenheit,
                CatalogUnit::Celsius,
                TemperatureReading::Absolute,
            )
            .unwrap(),
            0.0
        );
        assert_eq!(
            convert_temperature(
                212.0,
                CatalogUnit::Fahrenheit,
                CatalogUnit::Celsius,
                TemperatureReading::Absolute,
            )
            .unwrap(),
            100.0
        );
        assert_eq!(
            convert_temperature(
                -40.0,
                CatalogUnit::Fahrenheit,
                CatalogUnit::Celsius,
                TemperatureReading::Absolute,
            )
            .unwrap(),
            -40.0
        );
        assert_eq!(
            convert_temperature(
                0.0,
                CatalogUnit::Celsius,
                CatalogUnit::Fahrenheit,
                TemperatureReading::Absolute,
            )
            .unwrap(),
            32.0
        );
        assert_eq!(
            absolute_to_kelvin(0.0, CatalogUnit::Celsius).unwrap(),
            273.15
        );
        for &unit in CatalogUnit::ALL {
            let converted = absolute_to_kelvin(0.0, unit);
            if unit.absolute_offset().is_some() {
                assert!(converted.is_ok(), "{}", unit.symbol());
            } else {
                assert_eq!(converted.unwrap_err().code, "ENG-E-0010");
            }
        }
    }

    #[test]
    fn presets_name_coherent_units_and_leave_mass_unset() {
        assert!(CatalogUnit::parse("kip-in").is_err());
        assert!(CatalogUnit::parse("N-m").is_err());
        for preset in UnitPreset::ALL {
            assert_eq!(UnitPreset::parse(preset.name()).unwrap(), *preset);
            let definition = preset.definition();
            assert_eq!(definition.preset, *preset);
            assert_eq!(definition.force.dimension(), Dimension::Force);
            assert_eq!(definition.length.dimension(), Dimension::Length);
            if let Some(moment) = definition.moment {
                assert_eq!(moment.si_factor(), preset.moment_factor());
            }
            if let Some(per_length) = definition.force_per_length {
                let expected = definition
                    .force
                    .si_factor()
                    .checked_div(definition.length.si_factor())
                    .unwrap();
                assert_eq!(per_length.si_factor(), expected, "{}", preset.name());
            }
            if let Some(per_area) = definition.force_per_area {
                let length_squared = definition.length.si_factor().checked_pow(2).unwrap();
                let expected = definition
                    .force
                    .si_factor()
                    .checked_div(length_squared)
                    .unwrap();
                assert_eq!(per_area.si_factor(), expected, "{}", preset.name());
            }
            if let Some(mass) = definition.mass {
                let acceleration = match definition.acceleration {
                    Some(unit) => unit,
                    None => panic!("{}", preset.name()),
                };
                let expected = definition
                    .force
                    .si_factor()
                    .checked_div(acceleration.si_factor())
                    .unwrap();
                assert_eq!(mass.si_factor(), expected, "{}", preset.name());
            }
        }
        assert_eq!(
            UnitPreset::PoundInch.definition().mass,
            None,
            "lb-in does not infer a mass from lbf"
        );
        assert_eq!(
            UnitPreset::KipInch.definition().mass,
            Some(CatalogUnit::KipSecondSquaredPerInch)
        );
        assert_eq!(
            UnitPreset::PoundInch.definition().force,
            CatalogUnit::PoundForce
        );
        assert_eq!(UnitPreset::NewtonMillimeter.moment_factor(), MILLI);
        assert_eq!(UnitPreset::KilonewtonMeter.moment_factor(), KILO);
        assert_eq!(UnitPreset::NewtonMeter.moment_factor(), ExactFactor::ONE);
    }

    #[test]
    fn identity_conversion_preserves_bits_and_rejects_nonfinite() {
        for &unit in CatalogUnit::ALL {
            for value in [0.0, -0.0, 1.0, -2.5, Real::MAX, -Real::MAX] {
                assert_eq!(
                    convert(value, unit, unit).unwrap().to_bits(),
                    value.to_bits(),
                    "{}",
                    unit.symbol()
                );
            }
            for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
                assert_eq!(
                    convert(value, unit, si_partner(unit.dimension()))
                        .unwrap_err()
                        .code,
                    "ENG-E-0008"
                );
            }
        }
    }
}
