//! Conservative cubic elasticity; original derivation in docs/METHODS_GUIDE.md §5.

use super::UniaxialFactory;
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Cubic elasticity descriptor, method 0.1.0.
pub const CUBIC_ELASTIC: LawDescriptor = LawDescriptor {
    kind: "elastic.cubic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// SI parameters of [`CubicElasticFactory`], validated on construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicElasticParams {
    /// Initial modulus E in Pa, finite and strictly positive.
    pub e: Real,
    /// Cubic coefficient C in Pa, finite and nonnegative.
    pub c: Real,
}

/// Typed factory for [`CubicElastic`], without state or a global registry.
#[derive(Clone, Copy, Debug, Default)]
pub struct CubicElasticFactory;

impl UniaxialFactory for CubicElasticFactory {
    type Params = CubicElasticParams;

    fn descriptor(&self) -> LawDescriptor {
        CUBIC_ELASTIC
    }

    fn build(&self, params: CubicElasticParams) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(CubicElastic::new(params.e, params.c)?))
    }
}

/// Smooth elastic law σ = Eε + Cε³, without history, preload, damage, or plasticity.
///
/// ε is total, dimensionless, and positive in tension; σ, E, C, and the tangent
/// are in Pa. Energy W = Eε²/2 + Cε⁴/4 is strictly convex:
/// σ = dW/dε and K = E + 3Cε² > 0. Unloading follows the same curve.
/// C = 0 recovers linear elasticity. This is a local 1D idealization;
/// it does not guarantee physical validity at large strains.
///
/// Initial, accepted, and trial histories have zero length. The numerical
/// domain requires finite ε and finite operations q = (Cε)ε, σ = (E+q)ε, and K = E+3q.
/// Ordinary f64 rounding/underflow is allowed. ε² is not formed
/// separately, to preserve the C = 0 limit at large strains.
/// Original derivation and benchmark: docs/METHODS_GUIDE.md §5, (1)–(3), ENG-CORE-011.
///
/// ```
/// use engine_contracts::UniaxialLaw;
/// use engine_laws::uniaxial::CubicElastic;
/// let law = CubicElastic::new(8.0, 16.0)?;
/// let response = law.respond(0.5, &[])?;
/// assert_eq!((response.stress, response.tangent), (6.0, 20.0));
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicElastic {
    params: CubicElasticParams,
}

impl CubicElastic {
    /// Constructs with E > 0 and C ≥ 0 in Pa, both finite.
    ///
    /// # Errors
    /// `ENG-E-0008` for nonfinite parameters; `ENG-E-0010` for E ≤ 0 or C < 0.
    pub fn new(e: Real, c: Real) -> EResult<Self> {
        positive(e, "E (Pa)")?;
        finite(c, "C (Pa)")?;
        if c < 0.0 {
            return Err(EngineError::new(
                "ENG-E-0010",
                "C (Pa) must be nonnegative.",
            ));
        }
        Ok(Self {
            params: CubicElasticParams { e, c },
        })
    }

    /// Original validated parameters, immutable.
    pub fn params(&self) -> &CubicElasticParams {
        &self.params
    }
}

impl UniaxialLaw for CubicElastic {
    fn kind(&self) -> &'static str {
        CUBIC_ELASTIC.kind
    }

    fn contract(&self) -> ContractId {
        CUBIC_ELASTIC.contract
    }

    fn state_len(&self) -> usize {
        0
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    fn initial_tangent(&self) -> Real {
        self.params.e
    }

    /// Evaluates without mutating the object or accepting history.
    ///
    /// `ENG-E-0010` for nonempty history; `ENG-E-0008` for nonfinite ε
    /// or overflow in stress or tangent products/sums.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0, "Accepted history")?;
        finite(strain, "ε total")?;
        let CubicElasticParams { e, c } = self.params;
        let linear_factor = finite(c * strain, "Cε (Pa)")?;
        let q = finite(linear_factor * strain, "Cε² (Pa)")?;
        let secant = finite(e + q, "E+Cε² (Pa)")?;
        let stress = finite(secant * strain, "σ (Pa)")?;
        let cubic_tangent = finite(3.0 * q, "3Cε² (Pa)")?;
        let tangent = finite(e + cubic_tangent, "K (Pa)")?;
        Ok(UniaxialResponse {
            stress,
            tangent,
            trial_state: Vec::new(),
        })
    }
}
