//! Explicit units and conversions at the input and output boundary.
//!
//! Internal computation uses SI: meters, newtons, newton meters, pascals, and
//! radians. Units are converted when receiving data or presenting results;
//! they must not accompany or alter the engine's internal operations.
//!
//! [`convert`] validates quantities and values. [`to_si`] and [`from_si`] are the
//! arithmetic operations for boundaries whose values have already been validated.
//!
//! ```
//! use engine_core::units::{convert, Unit};
//! let momento_si = convert(2.5, Unit::KilonewtonMeter, Unit::NewtonMeter)?;
//! assert_eq!(momento_si, 2500.0);
//! assert_eq!(Unit::parse("kN*m")?, Unit::KilonewtonMeter);
//! assert!(convert(1.0, Unit::Meter, Unit::Newton).is_err());
//! # Ok::<(), engine_core::EngineError>(())
//! ```

use std::collections::BTreeMap;

use crate::{EResult, EngineError, Real};

// Explicit decimal factors, without runtime power calculations.
const CENTI: Real = 1e-2;
const MILLI: Real = 1e-3;
const KILO: Real = 1e3;
const MEGA: Real = 1e6;
const GIGA: Real = 1e9;
const DEGREE_TO_RADIAN: Real = std::f64::consts::PI / 180.0;

/// Physical quantity of a unit; pressure and stress share a dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quantity {
    /// Length.
    Length,
    /// Force.
    Force,
    /// Moment of a force.
    Moment,
    /// Pressure or mechanical stress.
    Pressure,
    /// Angle, distinguished from dimensionless quantities.
    Angle,
    /// Quantity without a physical dimension.
    Dimensionless,
}

/// Alternative name for the physical quantity of a unit.
pub type UnitKind = Quantity;

/// Closed list of units accepted at the engine boundaries.
///
/// There is no implicit unit or variant for unknown text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unit {
    /// Meter (m), SI unit of length.
    Meter,
    /// Centimeter (cm).
    Centimeter,
    /// Millimeter (mm).
    Millimeter,
    /// Newton (N), SI unit of force.
    Newton,
    /// Kilonewton (kN).
    Kilonewton,
    /// Meganewton (MN).
    Meganewton,
    /// Newton meter (N·m), SI unit of moment.
    NewtonMeter,
    /// Kilonewton meter (kN·m).
    KilonewtonMeter,
    /// Meganewton meter (MN·m).
    MeganewtonMeter,
    /// Pascal (Pa), SI unit of pressure and stress.
    Pascal,
    /// Kilopascal (kPa).
    Kilopascal,
    /// Megapascal (MPa).
    Megapascal,
    /// Gigapascal (GPa).
    Gigapascal,
    /// Radian (rad), SI unit of angle.
    Radian,
    /// Degree (deg).
    Degree,
    /// Dimensionless unit, explicitly represented by "1".
    Dimensionless,
}

impl Unit {
    /// Recognizes an exact symbol, with case sensitivity.
    ///
    /// Accepts the symbols from [`Self::symbol`] and the ASCII aliases `N.m`, `N*m`,
    /// `Nm`, `kN.m`, `kN*m`, `kNm`, `MN*m`, and `MNm`. Does not trim spaces or
    /// accept empty text.
    ///
    /// # Errors
    ///
    /// Returns `ENG-E-0001` if the text is not recognized. The message includes
    /// the received string, even if it is empty.
    ///
    /// ```
    /// use engine_core::units::Unit;
    /// assert_eq!(Unit::parse("kN·m")?, Unit::parse("kNm")?);
    /// assert_eq!(Unit::parse("1")?, Unit::Dimensionless);
    /// let error = Unit::parse("tonelada").unwrap_err();
    /// assert_eq!(error.code, "ENG-E-0001");
    /// assert!(error.message.contains("tonelada"));
    /// # Ok::<(), engine_core::EngineError>(())
    /// ```
    pub fn parse(text: &str) -> EResult<Self> {
        match text {
            "m" => Ok(Self::Meter),
            "cm" => Ok(Self::Centimeter),
            "mm" => Ok(Self::Millimeter),
            "N" => Ok(Self::Newton),
            "kN" => Ok(Self::Kilonewton),
            "MN" => Ok(Self::Meganewton),
            "N·m" | "N.m" | "N*m" | "Nm" => Ok(Self::NewtonMeter),
            "kN·m" | "kN.m" | "kN*m" | "kNm" => Ok(Self::KilonewtonMeter),
            "MN·m" | "MN*m" | "MNm" => Ok(Self::MeganewtonMeter),
            "Pa" => Ok(Self::Pascal),
            "kPa" => Ok(Self::Kilopascal),
            "MPa" => Ok(Self::Megapascal),
            "GPa" => Ok(Self::Gigapascal),
            "rad" => Ok(Self::Radian),
            "deg" => Ok(Self::Degree),
            "1" => Ok(Self::Dimensionless),
            _ => Err(EngineError::new(
                "ENG-E-0001",
                format!("La unidad indicada no está reconocida: «{text}»."),
            )),
        }
    }

    /// Returns the canonical symbol; the dimensionless unit uses "1".
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Meter => "m",
            Self::Centimeter => "cm",
            Self::Millimeter => "mm",
            Self::Newton => "N",
            Self::Kilonewton => "kN",
            Self::Meganewton => "MN",
            Self::NewtonMeter => "N·m",
            Self::KilonewtonMeter => "kN·m",
            Self::MeganewtonMeter => "MN·m",
            Self::Pascal => "Pa",
            Self::Kilopascal => "kPa",
            Self::Megapascal => "MPa",
            Self::Gigapascal => "GPa",
            Self::Radian => "rad",
            Self::Degree => "deg",
            Self::Dimensionless => "1",
        }
    }

    /// Returns the physical quantity to check dimensional compatibility.
    pub const fn kind(self) -> Quantity {
        match self {
            Self::Meter | Self::Centimeter | Self::Millimeter => Quantity::Length,
            Self::Newton | Self::Kilonewton | Self::Meganewton => Quantity::Force,
            Self::NewtonMeter | Self::KilonewtonMeter | Self::MeganewtonMeter => Quantity::Moment,
            Self::Pascal | Self::Kilopascal | Self::Megapascal | Self::Gigapascal => {
                Quantity::Pressure
            }
            Self::Radian | Self::Degree => Quantity::Angle,
            Self::Dimensionless => Quantity::Dimensionless,
        }
    }

    /// Returns the SI unit chosen by the engine for this quantity.
    pub const fn si_unit(self) -> Self {
        match self.kind() {
            Quantity::Length => Self::Meter,
            Quantity::Force => Self::Newton,
            Quantity::Moment => Self::NewtonMeter,
            Quantity::Pressure => Self::Pascal,
            Quantity::Angle => Self::Radian,
            Quantity::Dimensionless => Self::Dimensionless,
        }
    }

    const fn si_factor(self) -> Real {
        match self {
            Self::Centimeter => CENTI,
            Self::Millimeter => MILLI,
            Self::Kilonewton | Self::KilonewtonMeter | Self::Kilopascal => KILO,
            Self::Meganewton | Self::MeganewtonMeter | Self::Megapascal => MEGA,
            Self::Gigapascal => GIGA,
            Self::Degree => DEGREE_TO_RADIAN,
            _ => 1.0,
        }
    }
}

/// Multiplies a value in the specified unit by its SI factor.
///
/// Boundary arithmetic operation: the caller must validate that the value and
/// result are finite. The signature returns a number and preserves IEEE 754
/// arithmetic; to reject NaN, infinities, and overflows with an
/// `ENG-E-0008` error, use [`convert`] with [`Unit::si_unit`] as the target.
///
/// ```
/// use engine_core::units::{to_si, Unit};
/// assert_eq!(to_si(2.0, Unit::Megapascal), 2_000_000.0);
/// ```
pub fn to_si(value: Real, unit: Unit) -> Real {
    value * unit.si_factor()
}

/// Divides an SI value by the output unit factor.
///
/// The caller must validate that the value and result are finite. For a
/// validated boundary returning `ENG-E-0008` for NaN, infinity, or overflow,
/// use [`convert`] with [`Unit::si_unit`] as the source.
///
/// ```
/// use engine_core::units::{from_si, Unit};
/// assert_eq!(from_si(2500.0, Unit::KilonewtonMeter), 2.5);
/// ```
pub fn from_si(value: Real, unit: Unit) -> Real {
    value / unit.si_factor()
}

/// Converts between units of the same quantity at a validated boundary.
///
/// Uses the ratio of factors to avoid intermediate SI overflow.
/// A conversion to the same unit preserves the received value exactly.
/// Rounding and underflow follow `f64` arithmetic.
///
/// # Errors
///
/// Returns `ENG-E-0008` for a NaN or infinite input, or if the result
/// overflows. Returns `ENG-E-0010` if physical dimensions are incompatible;
/// angles and dimensionless quantities are considered different.
///
/// ```
/// use engine_core::units::{convert, Unit};
/// assert_eq!(convert(1.0, Unit::Megapascal, Unit::Pascal)?, 1e6);
/// assert_eq!(convert(f64::NAN, Unit::Meter, Unit::Meter).unwrap_err().code,
///            "ENG-E-0008");
/// # Ok::<(), engine_core::EngineError>(())
/// ```
pub fn convert(value: Real, from: Unit, to: Unit) -> EResult<Real> {
    if !value.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            format!("La magnitud de entrada no es finita: {value}."),
        ));
    }
    if from.kind() != to.kind() {
        return Err(EngineError::new(
            "ENG-E-0010",
            format!(
                "Las dimensiones físicas de las unidades «{}» y «{}» son incompatibles.",
                from.symbol(),
                to.symbol()
            ),
        ));
    }
    let result = value * (from.si_factor() / to.si_factor());
    if !result.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            format!(
                "La conversión de «{}» a «{}» produce una magnitud no finita.",
                from.symbol(),
                to.symbol()
            ),
        ));
    }
    Ok(result)
}

/// Ordered declaration of SI units for result fields.
///
/// Each field is registered with its SI unit, even if an equivalent input
/// unit is supplied. This structure declares metadata; it does not convert
/// result values. The caller must supply those values in SI.
///
/// ```
/// use engine_core::units::{Unit, UnitsOut};
/// let mut units_out = UnitsOut::new();
/// units_out.insert("desplazamiento", Unit::Meter);
/// units_out.insert("momento", Unit::NewtonMeter);
/// units_out.insert("tension", Unit::Pascal);
/// assert_eq!(units_out.get("momento"), Some(Unit::NewtonMeter));
/// let campos: Vec<_> = units_out.fields().keys().map(String::as_str).collect();
/// assert_eq!(campos, ["desplazamiento", "momento", "tension"]);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitsOut {
    fields: BTreeMap<String, Unit>,
}

impl UnitsOut {
    /// Creates an empty declaration of output units.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares a field's SI unit, taking the quantity from `unit`.
    ///
    /// If the field already existed, replaces its declaration and returns the
    /// previous SI unit. For example, supplying MPa registers Pa; the associated value
    /// must have been converted to Pa before emitting the result.
    pub fn insert(&mut self, field: impl Into<String>, unit: Unit) -> Option<Unit> {
        self.fields.insert(field.into(), unit.si_unit())
    }

    /// Looks up the SI unit of a declared field.
    pub fn get(&self, field: &str) -> Option<Unit> {
        self.fields.get(field).copied()
    }

    /// Returns SI declarations ordered by field name.
    pub fn fields(&self) -> &BTreeMap<String, Unit> {
        &self.fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNITS: [Unit; 16] = [
        Unit::Meter,
        Unit::Centimeter,
        Unit::Millimeter,
        Unit::Newton,
        Unit::Kilonewton,
        Unit::Meganewton,
        Unit::NewtonMeter,
        Unit::KilonewtonMeter,
        Unit::MeganewtonMeter,
        Unit::Pascal,
        Unit::Kilopascal,
        Unit::Megapascal,
        Unit::Gigapascal,
        Unit::Radian,
        Unit::Degree,
        Unit::Dimensionless,
    ];

    #[test]
    fn simbolos_alias_y_texto_desconocido() {
        for unit in UNITS {
            assert_eq!(Unit::parse(unit.symbol()).unwrap(), unit);
        }
        for (aliases, unit) in [
            (["N*m", "Nm"], Unit::NewtonMeter),
            (["kN*m", "kNm"], Unit::KilonewtonMeter),
            (["MN*m", "MNm"], Unit::MeganewtonMeter),
        ] {
            for alias in aliases {
                assert_eq!(Unit::parse(alias).unwrap(), unit);
            }
        }
        for text in ["", " ", " m", "m ", "kn", "mpa", "kg", "desconocida"] {
            let error = Unit::parse(text).unwrap_err();
            assert_eq!(error.code, "ENG-E-0001");
            assert!(error.message.contains(&format!("«{text}»")));
        }
    }

    #[test]
    fn codigos_ascii_de_units_1() {
        assert_eq!(Unit::parse("N.m").unwrap(), Unit::NewtonMeter);
        assert_eq!(Unit::parse("kN.m").unwrap(), Unit::KilonewtonMeter);
        assert_eq!(Unit::parse("N.m").unwrap(), Unit::parse("N·m").unwrap());
        for text in ["MN.m", "N.mm", "n.m"] {
            assert_eq!(Unit::parse(text).unwrap_err().code, "ENG-E-0001");
        }
    }

    #[test]
    fn factores_y_unidades_si_por_magnitud() {
        let expected = [
            (Quantity::Length, Unit::Meter, 1.0),
            (Quantity::Length, Unit::Meter, 0.01),
            (Quantity::Length, Unit::Meter, 0.001),
            (Quantity::Force, Unit::Newton, 1.0),
            (Quantity::Force, Unit::Newton, 1000.0),
            (Quantity::Force, Unit::Newton, 1_000_000.0),
            (Quantity::Moment, Unit::NewtonMeter, 1.0),
            (Quantity::Moment, Unit::NewtonMeter, 1000.0),
            (Quantity::Moment, Unit::NewtonMeter, 1_000_000.0),
            (Quantity::Pressure, Unit::Pascal, 1.0),
            (Quantity::Pressure, Unit::Pascal, 1000.0),
            (Quantity::Pressure, Unit::Pascal, 1_000_000.0),
            (Quantity::Pressure, Unit::Pascal, 1_000_000_000.0),
            (Quantity::Angle, Unit::Radian, 1.0),
            (Quantity::Angle, Unit::Radian, std::f64::consts::PI / 180.0),
            (Quantity::Dimensionless, Unit::Dimensionless, 1.0),
        ];
        for (unit, (kind, si, factor)) in UNITS.into_iter().zip(expected) {
            assert_eq!(unit.kind(), kind);
            assert_eq!(unit.si_unit(), si);
            assert_eq!(to_si(1.0, unit), factor);
            assert_eq!(from_si(factor, unit), 1.0);
            assert_eq!(si.si_unit(), si);
        }
        assert_eq!(
            convert(180.0, Unit::Degree, Unit::Radian).unwrap(),
            std::f64::consts::PI
        );
        assert_eq!(
            convert(1.0, Unit::Centimeter, Unit::Millimeter).unwrap(),
            10.0
        );
        assert_eq!(
            convert(1.0, Unit::Gigapascal, Unit::Megapascal).unwrap(),
            1000.0
        );
    }

    fn assert_relative(actual: Real, expected: Real) {
        if expected == 0.0 {
            assert_eq!(actual.to_bits(), expected.to_bits());
        } else {
            let relative = ((actual - expected) / expected).abs();
            assert!(
                relative <= 1e-15,
                "Error relativo {relative}: {actual} frente a {expected}"
            );
        }
    }

    #[test]
    fn ida_y_vuelta_momento_y_presion() {
        for (scaled, si, factor) in [
            (Unit::KilonewtonMeter, Unit::NewtonMeter, 1000.0),
            (Unit::Megapascal, Unit::Pascal, 1_000_000.0),
        ] {
            for value in [
                0.0,
                -0.0,
                0.1,
                123.456789,
                1e-9,
                1e9,
                -0.1,
                -123.456789,
                -1e-9,
                -1e9,
            ] {
                assert_relative(to_si(value, scaled), value * factor);
                assert_relative(from_si(to_si(value, scaled), scaled), value);
                assert_relative(to_si(from_si(value, scaled), scaled), value);
                assert_relative(convert(value, scaled, si).unwrap(), value * factor);
                assert_relative(
                    convert(convert(value, scaled, si).unwrap(), si, scaled).unwrap(),
                    value,
                );
                assert_relative(
                    convert(convert(value, si, scaled).unwrap(), scaled, si).unwrap(),
                    value,
                );
            }
        }
    }

    #[test]
    fn compatibilidad_de_todas_las_magnitudes() {
        for from in UNITS {
            for to in UNITS {
                let result = convert(1.0, from, to);
                if from.kind() == to.kind() {
                    assert!(result.is_ok());
                } else {
                    let error = result.unwrap_err();
                    assert_eq!(error.code, "ENG-E-0010");
                    assert!(error.message.contains(from.symbol()));
                    assert!(error.message.contains(to.symbol()));
                }
            }
        }
    }

    #[test]
    fn no_finitos_desbordamiento_e_identidad() {
        for unit in UNITS {
            for value in [Real::NAN, Real::INFINITY, Real::NEG_INFINITY] {
                assert_eq!(
                    convert(value, unit, unit.si_unit()).unwrap_err().code,
                    "ENG-E-0008"
                );
                assert_eq!(
                    convert(value, unit.si_unit(), unit).unwrap_err().code,
                    "ENG-E-0008"
                );
            }
            for value in [Real::MAX, -Real::MAX, Real::MIN_POSITIVE, 0.0, -0.0] {
                assert_eq!(
                    convert(value, unit, unit).unwrap().to_bits(),
                    value.to_bits()
                );
            }
        }
        for value in [Real::MAX, -Real::MAX] {
            assert_eq!(
                convert(value, Unit::Megapascal, Unit::Pascal)
                    .unwrap_err()
                    .code,
                "ENG-E-0008"
            );
            assert_eq!(
                convert(value, Unit::Meter, Unit::Millimeter)
                    .unwrap_err()
                    .code,
                "ENG-E-0008"
            );
        }
    }

    #[test]
    fn declaraciones_si_ordenadas_y_reemplazo() {
        let mut output = UnitsOut::new();
        assert!(output.fields().is_empty());
        assert_eq!(output.get("ausente"), None);
        assert_eq!(output.insert("tension", Unit::Megapascal), None);
        assert_eq!(output.insert("longitud", Unit::Millimeter), None);
        assert_eq!(output.get("tension"), Some(Unit::Pascal));
        assert_eq!(output.get("longitud"), Some(Unit::Meter));
        assert_eq!(
            output.insert("tension", Unit::Gigapascal),
            Some(Unit::Pascal)
        );
        let names: Vec<_> = output.fields().keys().map(String::as_str).collect();
        assert_eq!(names, ["longitud", "tension"]);
        for unit in UNITS {
            output.insert(unit.symbol(), unit);
            assert_eq!(output.get(unit.symbol()), Some(unit.si_unit()));
        }
    }
}
