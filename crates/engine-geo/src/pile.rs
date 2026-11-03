//! Pile-axis discretization and tributary lengths; see provenance/ENG-GEO-005.md.
//!
//! Depths are measured downward in m. This module does not assemble a pile.
use engine_core::{EResult, EngineError, Real};

/// Analytical lateral response on a semi-infinite linear foundation.
pub mod lateral;

/// Ordered pile-axis nodes and their trapezoidal tributary lengths, in m.
#[derive(Clone, Debug, PartialEq)]
pub struct PileDiscretization {
    depths_m: Vec<Real>,
    tributaries_m: Vec<Real>,
}

impl PileDiscretization {
    /// Requires at least two finite, nonnegative, strictly increasing depths.
    /// Returns ENG-E-0008 for nonfinite values, ENG-E-0010 for invalid geometry.
    pub fn new(depths_m: Vec<Real>) -> EResult<Self> {
        if depths_m.len() < 2 {
            return Err(invalid("at least two depths are required"));
        }
        for &depth in &depths_m {
            if !depth.is_finite() {
                return Err(nonfinite());
            }
        }
        let mut spans = Vec::with_capacity(depths_m.len() - 1);
        for pair in depths_m.windows(2) {
            let span = pair[1] - pair[0];
            if !span.is_finite() {
                return Err(nonfinite());
            }
            if pair[0] < 0.0 || span <= 0.0 {
                return Err(invalid(
                    "depths must be nonnegative and strictly increasing",
                ));
            }
            let half = span * 0.5;
            if half == 0.0 {
                return Err(invalid("half-span underflows"));
            }
            spans.push(half);
        }
        let mut tributaries_m = Vec::with_capacity(depths_m.len());
        tributaries_m.push(spans[0]);
        for pair in spans.windows(2) {
            let weight = pair[0] + pair[1];
            if !weight.is_finite() {
                return Err(nonfinite());
            }
            tributaries_m.push(weight);
        }
        tributaries_m.push(spans[spans.len() - 1]);
        Ok(Self {
            depths_m,
            tributaries_m,
        })
    }

    /// Node depths in m, in the supplied order.
    pub fn depths_m(&self) -> &[Real] {
        &self.depths_m
    }
    /// Nodal tributary lengths in m, in the same order as the depths.
    pub fn tributaries_m(&self) -> &[Real] {
        &self.tributaries_m
    }
}

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", format!("Pile discretization: {message} (m)."))
}
fn nonfinite() -> EngineError {
    EngineError::new(
        "ENG-E-0008",
        "Pile discretization: non-finite depth or result (m).",
    )
}
