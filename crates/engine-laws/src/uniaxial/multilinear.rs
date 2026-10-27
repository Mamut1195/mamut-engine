//! Piecewise elastic interpolation, without history or external dependencies.

use super::UniaxialFactory;
use crate::{finite, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Multilinear elastic law descriptor, method 0.1.0.
pub const MULTILINEAR_ELASTIC: LawDescriptor = LawDescriptor {
    kind: "multilinear_elastic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Enabled sides; positive ε is tension and negative ε is compression.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MultilinearMode {
    /// Complete curve, without a gap.
    #[default]
    Both,
    /// Zero σ and tangent for ε > 0.
    CompressionOnly,
    /// Zero σ and tangent for ε < 0.
    TensionOnly,
}

/// SI parameters of [`MultilinearElasticFactory`], validated on construction.
#[derive(Clone, Debug, PartialEq)]
pub struct MultilinearElasticParams {
    /// Points (dimensionless ε, σ in Pa), ordered and including (0, 0).
    pub points: Vec<(Real, Real)>,
    /// Enabled sides; does not change or skip point validation.
    pub mode: MultilinearMode,
}

/// Typed factory without a global registry, conforming to [`UniaxialFactory`].
#[derive(Clone, Copy, Debug, Default)]
pub struct MultilinearElasticFactory;

impl UniaxialFactory for MultilinearElasticFactory {
    type Params = MultilinearElasticParams;

    fn descriptor(&self) -> LawDescriptor {
        MULTILINEAR_ELASTIC
    }

    fn build(&self, params: Self::Params) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(MultilinearElastic::new(
            params.points,
            params.mode,
        )?))
    }
}

/// Continuous, stateless nonlinear elastic σ(ε) curve defined by points.
///
/// Each segment uses mᵢ = (σᵢ₊₁ − σᵢ)/(εᵢ₊₁ − εᵢ) ≥ 0 and
/// σ = σᵢ + mᵢ(ε − εᵢ). Outside the table, the corresponding end segment
/// is extended: first in compression, last in tension. The origin may
/// be at an end (one-sided table); the same rule extends the
/// other side unless the mode disables it. For odd symmetry, supply
/// reflected points (−ε, −σ); independent points allow asymmetry.
///
/// The interior tangent is mᵢ. At a vertex, the right segment is chosen,
/// except at the last point (last segment) and the origin in compression-only mode
/// (left segment if present). `initial_tangent()` matches this choice
/// at ε = 0. In the gap, σ and tangent are zero; the origin retains the tangent
/// of the active side. At vertices with different slopes and at gap opening,
/// there is no two-sided derivative. The `check_uniaxial_law` kit accepts this
/// convention with samples whose ε ± h interval is differentiable, as required
/// by its contract; vertices are checked separately, without central FD.
///
/// σ and tangent are in Pa; ε is total and dimensionless. There is no preload,
/// plasticity, or damage, and both initial and trial histories are empty.
/// The numerical range requires finite intermediate operations and results.
///
/// ```
/// use engine_contracts::UniaxialLaw;
/// use engine_laws::uniaxial::{MultilinearElastic, MultilinearMode};
/// let law = MultilinearElastic::new(
///     vec![(-1.0, -2.0), (0.0, 0.0), (1.0, 4.0), (2.0, 6.0)],
///     MultilinearMode::Both,
/// )?;
/// let response = law.respond(1.5, &[])?;
/// assert_eq!((response.stress, response.tangent), (5.0, 2.0));
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MultilinearElastic {
    params: MultilinearElasticParams,
    slopes: Vec<Real>,
}

impl MultilinearElastic {
    /// Validates at least two finite points, strictly increasing ε, and (0, 0).
    /// Does not sort, reflect, or correct data; the inactive side is also validated.
    ///
    /// # Errors
    /// `ENG-E-0008`: nonfinite values or overflow in differences/slopes.
    /// `ENG-E-0010`: fewer than two points, incorrect order, missing origin,
    /// negative slope, or positive slope reduced to zero by
    /// underflow. Exact plateaus (σᵢ₊₁ = σᵢ) are valid.
    pub fn new(points: Vec<(Real, Real)>, mode: MultilinearMode) -> EResult<Self> {
        if points.len() < 2 {
            return Err(invalid("The curve requires at least two points."));
        }
        let mut has_origin = false;
        for (i, &(strain, stress)) in points.iter().enumerate() {
            finite(strain, &format!("ε at point {i}"))?;
            finite(stress, &format!("σ at point {i} (Pa)"))?;
            has_origin |= strain == 0.0 && stress == 0.0;
        }
        if !has_origin {
            return Err(invalid("The curve must contain the point (0, 0)."));
        }
        let mut slopes = Vec::with_capacity(points.len() - 1);
        for (i, pair) in points.windows(2).enumerate() {
            let ((e0, s0), (e1, s1)) = (pair[0], pair[1]);
            if e1 <= e0 {
                return Err(invalid(format!("Span {i}: ε must be strictly increasing.")));
            }
            let de = finite(e1 - e0, &format!("Δε of span {i}"))?;
            let ds = finite(s1 - s0, &format!("Δσ of span {i} (Pa)"))?;
            if ds < 0.0 {
                return Err(invalid(format!("Span {i}: negative slope is not allowed.")));
            }
            let slope = finite(ds / de, &format!("Slope of span {i} (Pa)"))?;
            if ds > 0.0 && slope == 0.0 {
                return Err(invalid(format!(
                    "Span {i}: positive slope reduced to zero by underflow."
                )));
            }
            slopes.push(slope);
        }
        Ok(Self {
            params: MultilinearElasticParams { points, mode },
            slopes,
        })
    }

    /// Original validated points and mode, immutable.
    pub fn params(&self) -> &MultilinearElasticParams {
        &self.params
    }

    fn segment(&self, strain: Real) -> usize {
        let left_at_origin = strain == 0.0 && self.params.mode == MultilinearMode::CompressionOnly;
        let upper = self.params.points.partition_point(|&(e, _)| {
            if left_at_origin {
                e < strain
            } else {
                e <= strain
            }
        });
        // The constructor guarantees at least two points and one segment.
        upper.saturating_sub(1).min(self.slopes.len() - 1)
    }
}

impl UniaxialLaw for MultilinearElastic {
    fn kind(&self) -> &'static str {
        MULTILINEAR_ELASTIC.kind
    }

    fn contract(&self) -> ContractId {
        MULTILINEAR_ELASTIC.contract
    }

    fn state_len(&self) -> usize {
        0
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    fn initial_tangent(&self) -> Real {
        self.slopes[self.segment(0.0)]
    }

    /// Evaluates without mutating inputs. `ENG-E-0010` for nonempty history;
    /// `ENG-E-0008` for nonfinite ε or overflow in interpolation/extrapolation
    /// subtraction, product, or sum. Uses the nearest end to
    /// reduce cancellation. ε and history are validated even in the gap.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0, "Accepted history")?;
        finite(strain, "ε")?;
        if (self.params.mode == MultilinearMode::CompressionOnly && strain > 0.0)
            || (self.params.mode == MultilinearMode::TensionOnly && strain < 0.0)
        {
            return Ok(UniaxialResponse {
                stress: 0.0,
                tangent: 0.0,
                trial_state: Vec::new(),
            });
        }
        let i = self.segment(strain);
        let (e0, s0) = self.params.points[i];
        let (e1, s1) = self.params.points[i + 1];
        let tangent = self.slopes[i];
        let stress = if strain == e0 || tangent == 0.0 {
            s0
        } else if strain == e1 {
            s1
        } else {
            let midpoint = e0 * 0.5 + e1 * 0.5;
            let (anchor_strain, anchor_stress) = if strain <= midpoint {
                (e0, s0)
            } else {
                (e1, s1)
            };
            let delta = finite(strain - anchor_strain, "Evaluation Δε")?;
            let increment = finite(tangent * delta, "Increment of σ (Pa)")?;
            finite(
                anchor_stress + increment,
                "Interpolated/extrapolated σ (Pa)",
            )?
        };
        Ok(UniaxialResponse {
            stress,
            tangent,
            trial_state: Vec::new(),
        })
    }
}

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
