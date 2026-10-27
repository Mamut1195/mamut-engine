use super::{invalid, validate_damping};
use crate::finite;
use engine_core::math;
use engine_core::{EResult, EngineError, Real};

/// Componentwise modal combination, preserving signs before summation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalCombination {
    /// √Σ R_n²; statistically independent modes.
    Srss,
    /// √Σ_iΣ_j ρ_ij R_i R_j with ζ from each modal table.
    Cqc,
}

/// Combination of directional envelopes with explicit user factors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionalCombination {
    /// √Σ_d (a_d R_d)².
    Srss,
    /// Σ_d a_d R_d. A weighted envelope, not an equilibrated state.
    WeightedSum,
}

/// Wilson–Der Kiureghian–Bayo (1981) CQC correlation, formula from the request.
///
/// r=ω_j/ω_i; numerator=8√(ζ_iζ_j)(ζ_i+rζ_j)r^(3/2);
/// denominator=(1−r²)²+4ζ_iζ_j r(1+r²)+4(ζ_i²+ζ_j²)r².
/// Reduces to the common-ζ formula. Swaps i,j to use r≤1 and avoid
/// overflow; the expression is symmetric. Returns 0≤ρ≤1.
/// For equal frequencies and ζ_i=ζ_j (including zero), ρ=1 by convention;
/// with both ζ=0 and distinct frequencies, ρ=0. For equal frequencies and
/// distinct ζ, ρ=2√(ζ_iζ_j)/(ζ_i+ζ_j), zero if only one has ζ=0.
/// Nonfinite values: ENG-E-0008; ω≤0 or ζ outside [0,1): ENG-E-0010.
pub fn cqc_correlation(omega_i: Real, omega_j: Real, zi: Real, zj: Real) -> EResult<Real> {
    validate_frequency(omega_i)?;
    validate_frequency(omega_j)?;
    validate_damping(zi)?;
    validate_damping(zj)?;
    if omega_i == omega_j {
        if zi == zj {
            return Ok(1.0);
        }
        let scale = zi.max(zj);
        let (a, b) = (zi / scale, zj / scale);
        return Ok(2.0 * math::sqrt(a) * math::sqrt(b) / (a + b));
    }
    let (high, low, zh, zl) = if omega_i > omega_j {
        (omega_i, omega_j, zi, zj)
    } else {
        (omega_j, omega_i, zj, zi)
    };
    let r = low / high;
    let root_product = math::sqrt(zh) * math::sqrt(zl);
    let denominator_root = math::hypot(
        math::hypot(
            1.0 - r * r,
            2.0 * root_product * math::sqrt(r * (1.0 + r * r)),
        ),
        2.0 * r * math::hypot(zh, zl),
    );
    let rho = 8.0
        * (root_product / denominator_root)
        * ((zh + r * zl) / denominator_root)
        * r
        * math::sqrt(r);
    finite(&[rho])?;
    // Only rounding at the mathematical bounds of a correlation.
    Ok(rho.clamp(0.0, 1.0))
}

pub(super) fn validate_frequency(omega: Real) -> EResult<()> {
    finite(&[omega])?;
    if omega <= 0.0 {
        return Err(invalid(
            "Positive modal frequencies are required; rigid modes are not allowed.",
        ));
    }
    Ok(())
}

pub(super) fn correlations(
    frequencies: &[Real],
    dampings: &[Real],
    method: ModalCombination,
) -> EResult<Vec<Vec<Real>>> {
    correlations_with_check(frequencies, dampings, method, &mut || Ok(()))
}

pub(super) fn correlations_with_check(
    frequencies: &[Real],
    dampings: &[Real],
    method: ModalCombination,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<Vec<Vec<Real>>> {
    if frequencies.is_empty() || frequencies.len() != dampings.len() {
        return Err(invalid("Empty modes or incompatible damping dimensions."));
    }
    for (&omega, &zeta) in frequencies.iter().zip(dampings) {
        validate_frequency(omega)?;
        validate_damping(zeta)?;
    }
    let mut rows = Vec::new();
    for (i, &omega) in frequencies.iter().enumerate() {
        check()?;
        let mut row = Vec::new();
        for (j, &other) in frequencies.iter().enumerate() {
            row.push(if i == j {
                1.0
            } else if method == ModalCombination::Srss {
                0.0
            } else {
                cqc_correlation(omega, other, dampings[i], dampings[j])?
            });
        }
        rows.push(row);
    }
    Ok(rows)
}

/// Combines one signed scalar per mode (e.g. a recovered member action).
/// Equal nonempty dimensions; frequencies in rad/s, ζ∈[0,1). Returns a
/// nonnegative magnitude. Actions are recovered with `Element::recover` at
/// EACH modal displacement before combining; recovery at the combined
/// envelope would lose the signs and correlations of the modes.
pub fn combine_modal(
    values: &[Real],
    frequencies: &[Real],
    dampings: &[Real],
    method: ModalCombination,
) -> EResult<Real> {
    let rho = correlations(frequencies, dampings, method)?;
    if values.len() != frequencies.len() {
        return Err(invalid("Incompatible response and mode dimensions."));
    }
    quadratic(values, &rho)
}

pub(super) fn quadratic(values: &[Real], rho: &[Vec<Real>]) -> EResult<Real> {
    quadratic_with_check(values, rho, &mut || Ok(()))
}

pub(super) fn quadratic_with_check(
    values: &[Real],
    rho: &[Vec<Real>],
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<Real> {
    check()?;
    finite(values)?;
    let scale = values.iter().fold(0.0_f64, |s, v| s.max(v.abs()));
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    for (i, &a) in values.iter().enumerate() {
        check()?;
        for (j, &b) in values.iter().enumerate() {
            sum += rho[i][j] * (a / scale) * (b / scale);
        }
    }
    finite(&[sum])?;
    let tolerance = 64.0 * Real::EPSILON * math::powi(values.len() as Real, 2);
    if sum < -tolerance {
        return Err(EngineError::new(
            "ENG-E-0012",
            "CQC: negative quadratic form.",
        ));
    }
    let result = scale * math::sqrt(sum.max(0.0));
    finite(&[result])?;
    Ok(result)
}

/// Combines directional scalars with finite nonnegative factors.
/// ENG-E-0010 for empty/incompatible lists or negative factors;
/// ENG-E-0008 for nonfinite values/overflow. No factor is built in.
pub fn combine_directions(
    values: &[Real],
    factors: &[Real],
    method: DirectionalCombination,
) -> EResult<Real> {
    if values.is_empty() || values.len() != factors.len() {
        return Err(invalid("Empty directions or incompatible factors."));
    }
    finite(values)?;
    finite(factors)?;
    if factors.iter().any(|&v| v < 0.0) {
        return Err(invalid("Directional factors must be nonnegative."));
    }
    let mut result = 0.0_f64;
    for (&value, &factor) in values.iter().zip(factors) {
        let term = value * factor;
        finite(&[term])?;
        result = match method {
            DirectionalCombination::Srss => math::hypot(result, term),
            DirectionalCombination::WeightedSum => result + term,
        };
        finite(&[result])?;
    }
    Ok(result)
}
