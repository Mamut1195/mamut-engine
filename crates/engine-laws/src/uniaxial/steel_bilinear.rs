//! 1D plasticity with linear kinematic hardening; original derivation.

use super::UniaxialFactory;
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Bilinear kinematic steel descriptor, method 0.1.0.
pub const STEEL_BILINEAR_KINEMATIC: LawDescriptor = LawDescriptor {
    kind: "steel.bilinear_kinematic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// SI parameters of [`SteelBilinearKinematicFactory`], validated on construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteelBilinearKinematicParams {
    /// Elastic modulus E in Pa, finite and strictly positive.
    pub e: Real,
    /// Initial yield stress fy in Pa, finite and strictly positive.
    pub fy: Real,
    /// Dimensionless ratio of post-yield slope to E: 0 ≤ b < 1.
    /// The kinematic modulus is H = bE/(1−b); b = 0 allows perfect plasticity.
    pub b: Real,
}

/// Typed factory without a global registry, conforming to [`UniaxialFactory`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SteelBilinearKinematicFactory;

impl UniaxialFactory for SteelBilinearKinematicFactory {
    type Params = SteelBilinearKinematicParams;

    fn descriptor(&self) -> LawDescriptor {
        STEEL_BILINEAR_KINEMATIC
    }

    fn build(&self, params: Self::Params) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(SteelBilinearKinematic::new(
            params.e, params.fy, params.b,
        )?))
    }
}

/// Bilinear steel: exact implicit return in 1D with kinematic hardening.
///
/// ε is TOTAL dimensionless strain, positive in tension; E, fy, σ,
/// α, and tangent are in Pa. The flat history is `[ε_p, α]`, initially
/// `[0, 0]`: plastic strain and back-stress. The caller owns
/// the accepted state; `respond` returns an independent trial and neither accepts
/// nor modifies history. There is no damage, isotropic hardening, or accumulated quantities.
///
/// H = bE/(1−b) is defined so that the post-yield slope is bE.
/// With accepted history: σ_tr = E(ε−ε_p), ξ = σ_tr−α, and f = |ξ|−fy.
/// If f ≤ 0, the response is elastic and its tangent is E. If f > 0, s = sign(ξ),
/// Δγ = f/(E+H), ε_p' = ε_p + sΔγ, α' = α + sHΔγ, and
/// σ = σ_tr−sEΔγ = α'+s fy. The consistent tangent is EH/(E+H) = bE.
/// The surface |σ−α| ≤ fy translates: after reversal, the elastic stress
/// range is always 2fy (in strain, 2fy/E), the Bauschinger effect.
///
/// Evaluates EΔγ = (1−b)f and HΔγ = bf, without forming H or E+H. Obtains
/// Δγ by dividing EΔγ by E and σ from the returned surface, avoiding
/// subtraction of large stresses to obtain a small plateau. The numerical range
/// requires finite differences, increments, and results; their overflow is
/// diagnosed. History precision is limited by `Real` (f64).
/// At f = 0 the elastic branch is chosen; no unique two-sided derivative exists there.
///
/// ```
/// use engine_contracts::UniaxialLaw;
/// use engine_laws::uniaxial::SteelBilinearKinematic;
/// let law = SteelBilinearKinematic::new(200.0, 1.0, 0.1)?;
/// let mut committed = law.initial_state();
/// let trial = law.respond(0.02, &committed)?;
/// assert!((trial.stress - 1.3).abs() < 1e-12);
/// committed = trial.trial_state; // explicit acceptance by the caller
/// let unload = law.respond(0.015, &committed)?;
/// assert!((unload.stress - 0.3).abs() < 1e-12);
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteelBilinearKinematic {
    params: SteelBilinearKinematicParams,
}

impl SteelBilinearKinematic {
    /// Constructs with E and fy in Pa and b = post-yield slope/E.
    ///
    /// # Errors
    /// `ENG-E-0008` for nonfinite parameters; `ENG-E-0010` for E ≤ 0,
    /// fy ≤ 0 or b outside [0, 1). H need not be representable.
    pub fn new(e: Real, fy: Real, b: Real) -> EResult<Self> {
        positive(e, "E (Pa)")?;
        positive(fy, "fy (Pa)")?;
        finite(b, "b")?;
        if !(0.0..1.0).contains(&b) {
            return Err(EngineError::new("ENG-E-0010", "b must be in [0, 1)."));
        }
        Ok(Self {
            params: SteelBilinearKinematicParams { e, fy, b },
        })
    }

    /// Original validated parameters, immutable.
    pub fn params(&self) -> &SteelBilinearKinematicParams {
        &self.params
    }
}

impl UniaxialLaw for SteelBilinearKinematic {
    fn kind(&self) -> &'static str {
        STEEL_BILINEAR_KINEMATIC.kind
    }

    fn contract(&self) -> ContractId {
        STEEL_BILINEAR_KINEMATIC.contract
    }

    fn state_len(&self) -> usize {
        2
    }

    fn initial_state(&self) -> Vec<Real> {
        vec![0.0, 0.0]
    }

    fn initial_tangent(&self) -> Real {
        self.params.e
    }

    /// Evaluates without mutating the law or accepting history.
    ///
    /// `ENG-E-0010` for length other than two; `ENG-E-0008` for nonfinite ε or
    /// history, or overflow of differences, increments, or results.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 2, "Accepted history [ε_p, α]")?;
        finite(strain, "ε")?;
        let SteelBilinearKinematicParams { e, fy, b } = self.params;
        // Length was validated before accessing both components.
        let (plastic_strain, alpha) = (committed[0], committed[1]);
        let elastic_strain = finite(strain - plastic_strain, "ε−ε_p")?;
        let stress_trial = finite(e * elastic_strain, "σ_tr (Pa)")?;
        let xi = finite(stress_trial - alpha, "ξ (Pa)")?;
        let f = xi.abs() - fy;
        if f <= 0.0 {
            return Ok(UniaxialResponse {
                stress: stress_trial,
                tangent: e,
                trial_state: committed.to_vec(),
            });
        }
        let sign = xi.signum();
        let stress_correction = (1.0 - b) * f;
        let gamma = finite(stress_correction / e, "Δγ")?;
        let plastic_trial = finite(plastic_strain + sign * gamma, "Trial ε_p")?;
        let alpha_trial = finite(alpha + sign * (b * f), "Trial α (Pa)")?;
        let stress = finite(alpha_trial + sign * fy, "σ (Pa)")?;
        Ok(UniaxialResponse {
            stress,
            tangent: b * e,
            trial_state: vec![plastic_trial, alpha_trial],
        })
    }
}
