//! Conversion of local soil resistance tables to nodal spring tables.
//!
//! A table returned here uses displacement in m and mobilized resistance in N.
//! Positive resistance has the same direction as positive displacement; the
//! soil action on the pile is its negative. A grounded spring's internal force
//! uses the mobilized-resistance sign. Depth and axial settlement are positive
//! downward. The lateral direction is chosen by the caller.
//!
//! These functions only convert tables. They do not construct elements, solve
//! equilibrium, or enforce the tabulated displacement range during a solve.
//! See `provenance/ENG-GEO-005.md`.

use crate::{py::PyCurve, tz::qz::QzCurve, tz::TzCurve};
use engine_core::{EResult, EngineError, Real};

/// Explicit policy for a consumer that extends the table's end segments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndPolicy {
    /// Accept linear extension of both end segments.
    ExtendEndSegments,
    /// Require both end segments to be horizontal before extending them.
    RequirePlateaus,
}

/// Converts p-y ordinates from N/m to nodal resistance in N.
///
/// `tributary_m` is the positive nodal tributary length. The caller chooses
/// the end policy explicitly; the p-y evaluator itself remains bounded.
pub fn lateral(
    curve: &PyCurve,
    tributary_m: Real,
    policy: EndPolicy,
) -> EResult<Vec<(Real, Real)>> {
    convert(
        curve.points(),
        positive(tributary_m, "tributary length")?,
        policy,
    )
}

/// Converts shaft t-z ordinates from Pa to nodal resistance in N.
///
/// `circumference_m * tributary_m` is the represented shaft area in m^2.
/// Both dimensions are explicit and positive. Negative-slope softening is
/// rejected because the existing multilinear elastic law does not support it.
pub fn shaft(
    curve: &TzCurve,
    circumference_m: Real,
    tributary_m: Real,
    policy: EndPolicy,
) -> EResult<Vec<(Real, Real)>> {
    let circumference = positive(circumference_m, "circumference")?;
    let tributary = positive(tributary_m, "tributary length")?;
    let area = product(circumference, tributary)?;
    convert(curve.points(), area, policy)
}

/// Copies tip q-z resistance in N to a nodal spring table.
///
/// Tip resistance is already a force: it is not multiplied by a tributary
/// length, circumference, or tip area. Negative-slope softening is rejected.
pub fn tip(curve: &QzCurve, policy: EndPolicy) -> EResult<Vec<(Real, Real)>> {
    convert(curve.points(), 1.0, policy)
}

fn convert(points: &[(Real, Real)], scale: Real, policy: EndPolicy) -> EResult<Vec<(Real, Real)>> {
    if points.len() < 2 {
        return Err(invalid("at least two curve points are required"));
    }
    if policy == EndPolicy::RequirePlateaus
        && (points[0].1 != points[1].1 || points[points.len() - 2].1 != points[points.len() - 1].1)
    {
        return Err(invalid("both end segments must be plateaus"));
    }
    let mut result = Vec::with_capacity(points.len());
    for &(displacement, resistance) in points {
        finite(displacement)?;
        finite(resistance)?;
        result.push((displacement, product(resistance, scale)?));
    }
    for pair in result.windows(2) {
        let dx = finite(pair[1].0 - pair[0].0)?;
        let df = finite(pair[1].1 - pair[0].1)?;
        if dx <= 0.0 || df < 0.0 {
            return Err(invalid(
                "curve order or slope is incompatible with the elastic law",
            ));
        }
        let slope = finite(df / dx)?;
        if df > 0.0 && slope == 0.0 {
            return Err(invalid("spring slope underflows"));
        }
    }
    Ok(result)
}

fn positive(value: Real, name: &str) -> EResult<Real> {
    finite(value)?;
    if value <= 0.0 {
        return Err(invalid(&format!("{name} must be positive")));
    }
    Ok(value)
}

fn product(a: Real, b: Real) -> EResult<Real> {
    let value = finite(a * b)?;
    if a != 0.0 && b != 0.0 && value == 0.0 {
        return Err(invalid("nonzero conversion product underflows"));
    }
    Ok(value)
}

fn finite(value: Real) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            "Pile spring: nonfinite input or conversion result.",
        ))
    }
}

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", format!("Pile spring: {message}."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{py, tz};

    #[test]
    fn dimensional_conversions_and_tip_force() -> EResult<()> {
        // Own analytical case: p=10 N/m, t=10 Pa, Q=10 N at 1 m.
        let py = py::user(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 10.0)])?;
        let tz = tz::user(
            Some(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 10.0)]),
            tz::UserTraction::Symmetric,
        )?;
        let qz = tz::qz::user(
            Some(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 10.0)]),
            tz::UserTraction::Symmetric,
        )?;
        let p = lateral(&py, 2.0, EndPolicy::RequirePlateaus)?;
        let t = shaft(&tz, 3.0, 2.0, EndPolicy::RequirePlateaus)?;
        let q = tip(&qz, EndPolicy::RequirePlateaus)?;
        assert_eq!(p[3], (1.0, 20.0));
        assert_eq!(t[3], (1.0, 60.0));
        assert_eq!(q[3], (1.0, 10.0));
        assert_eq!(t[1], (-1.0, -60.0));
        Ok(())
    }

    #[test]
    fn rejects_softening_and_requires_explicit_end_policy() -> EResult<()> {
        let soft = tz::user(
            Some(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 5.0)]),
            tz::UserTraction::Symmetric,
        )?;
        assert_eq!(
            shaft(&soft, 1.0, 1.0, EndPolicy::ExtendEndSegments)
                .err()
                .map(|e| e.code),
            Some("ENG-E-0010")
        );
        let py = py::user(vec![(0.0, 0.0), (1.0, 10.0)])?;
        assert_eq!(
            lateral(&py, 1.0, EndPolicy::RequirePlateaus)
                .err()
                .map(|e| e.code),
            Some("ENG-E-0010")
        );
        assert!(lateral(&py, 1.0, EndPolicy::ExtendEndSegments).is_ok());
        assert_eq!(
            lateral(&py, Real::INFINITY, EndPolicy::ExtendEndSegments)
                .err()
                .map(|e| e.code),
            Some("ENG-E-0008")
        );
        assert_eq!(
            lateral(&py, 0.0, EndPolicy::ExtendEndSegments)
                .err()
                .map(|e| e.code),
            Some("ENG-E-0010")
        );
        Ok(())
    }
}
