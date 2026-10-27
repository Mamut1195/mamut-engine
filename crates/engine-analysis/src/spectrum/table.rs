use super::{invalid, validate_damping};
use crate::finite;
use engine_core::math;
use engine_core::{EResult, Real};

/// Declared SI units for ALL input pairs; no conversion from g.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpectrumUnits {
    /// Abscissa T in s and spectral acceleration Sa in m/s².
    PeriodSecondsAccelerationMetersPerSecondSquared,
    /// Abscissa f in Hz and spectral acceleration Sa in m/s².
    FrequencyHertzAccelerationMetersPerSecondSquared,
}

/// Interpolation always in period T, including frequency inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    /// Straight segments in (T, Sa); accepts T=0 and Sa=0.
    Linear,
    /// Straight segments in (ln T, ln Sa); requires T>0 and Sa>0.
    LogLog,
}

/// Required user decision for periods outside the tabulated interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extrapolation {
    /// Returns ENG-E-0010 outside the closed interval.
    Error,
    /// Explicit plateaus: Sa of the first/last point beyond each end.
    Plateau,
}

/// User table for a single declared ζ; does not adjust Sa for damping.
///
/// At least two finite pairs with strictly increasing abscissas and Sa≥0.
/// Frequencies must be positive; they are inverted and reordered as periods.
/// Contains no spectra, code factors, or default values.
#[derive(Clone, Debug, PartialEq)]
pub struct TabulatedSpectrum {
    points: Vec<(Real, Real)>,
    units: SpectrumUnits,
    damping: Real,
    interpolation: Interpolation,
    extrapolation: Extrapolation,
}

impl TabulatedSpectrum {
    /// Constructs an immutable, validated SI table for ζ∈[0,1).
    /// ENG-E-0008 for nonfinite values/overflow; ENG-E-0010 for size,
    /// order, signs, or values incompatible with log-log. Does not sort an
    /// invalid table or interpolate in frequency: f is converted to T=1/f.
    pub fn new(
        pairs: &[(Real, Real)],
        units: SpectrumUnits,
        damping: Real,
        interpolation: Interpolation,
        extrapolation: Extrapolation,
    ) -> EResult<Self> {
        validate_damping(damping)?;
        if pairs.len() < 2 {
            return Err(invalid("At least two spectral pairs are required."));
        }
        for &(x, sa) in pairs {
            finite(&[x, sa])?;
            if x < 0.0 || sa < 0.0 {
                return Err(invalid("Period/frequency and Sa must be nonnegative."));
            }
        }
        if pairs.windows(2).any(|p| p[0].0 >= p[1].0) {
            return Err(invalid("Abscissas must be strictly increasing."));
        }
        let mut points = pairs.to_vec();
        if units == SpectrumUnits::FrequencyHertzAccelerationMetersPerSecondSquared {
            for (x, _) in &mut points {
                if *x == 0.0 {
                    return Err(invalid("Tabulated frequency must be positive."));
                }
                *x = 1.0 / *x;
                finite(&[*x])?;
            }
            points.reverse();
        }
        if points.windows(2).any(|p| p[0].0 >= p[1].0) {
            return Err(invalid("Converted periods are indistinguishable."));
        }
        if interpolation == Interpolation::LogLog
            && points.iter().any(|&(t, sa)| t <= 0.0 || sa <= 0.0)
        {
            return Err(invalid("Log-log requires positive periods and Sa."));
        }
        Ok(Self {
            points,
            units,
            damping,
            interpolation,
            extrapolation,
        })
    }

    /// Internal pairs (T in s, Sa in m/s²), in increasing period order.
    pub fn period_points(&self) -> &[(Real, Real)] {
        &self.points
    }

    /// Declared units of the original table.
    pub fn input_units(&self) -> SpectrumUnits {
        self.units
    }

    /// Dimensionless damping ratio ζ of this table, without automatic correction.
    pub fn damping(&self) -> Real {
        self.damping
    }

    /// User-selected interpolation.
    pub fn interpolation(&self) -> Interpolation {
        self.interpolation
    }

    /// Explicit out-of-range treatment.
    pub fn extrapolation(&self) -> Extrapolation {
        self.extrapolation
    }

    /// Evaluates Sa(T,ζ_tabla) in m/s²; T in s, finite and nonnegative.
    /// Preserves ordinates exactly at table points. T=0 outside
    /// a log-log table can only be evaluated with an explicit plateau.
    pub fn acceleration(&self, period: Real) -> EResult<Real> {
        finite(&[period])?;
        if period < 0.0 {
            return Err(invalid("Query period must be nonnegative."));
        }
        // The constructor guarantees two points and the table allows no mutations.
        let first = self.points[0];
        let last = self.points[self.points.len() - 1];
        if period < first.0 || period > last.0 {
            return match self.extrapolation {
                Extrapolation::Error => Err(invalid("Period is outside the tabulated spectrum.")),
                Extrapolation::Plateau => Ok(if period < first.0 { first.1 } else { last.1 }),
            };
        }
        let upper = self.points.partition_point(|&(t, _)| t < period);
        let (t1, sa1) = self.points[upper];
        if t1 == period {
            return Ok(sa1);
        }
        let (t0, sa0) = self.points[upper - 1];
        let sa = match self.interpolation {
            Interpolation::Linear => {
                let a = (period - t0) / (t1 - t0);
                (1.0 - a) * sa0 + a * sa1
            }
            Interpolation::LogLog => {
                // ln_1p distinguishes nearby periods; the alternative avoids
                // overflowing the ratio for very wide intervals.
                let a = log_distance(t0, period) / log_distance(t0, t1);
                math::exp((1.0 - a) * math::ln(sa0) + a * math::ln(sa1))
            }
        };
        finite(&[sa])?;
        Ok(sa)
    }
}

fn log_distance(low: Real, high: Real) -> Real {
    let relative = (high - low) / low;
    if relative.is_finite() {
        math::ln_1p(relative)
    } else {
        math::ln(high) - math::ln(low)
    }
}
