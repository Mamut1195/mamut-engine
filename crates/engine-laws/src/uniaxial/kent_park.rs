//! Kent and Park monotonic compression envelope.
//!
//! Equations (A.9)–(A.15) of BARC/2009/E/026, Appendix A, section A.1.1.3,
//! printed pages 135–138. The sheet is `docs/fichas/concrete.kent_park.md`.
//! The 1971 article cited by that report was not opened.

use super::UniaxialFactory;
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::math;
use engine_core::{ContractId, EResult, EngineError, Real};

/// Kent–Park descriptor, method 0.1.0.
pub const KENT_PARK: LawDescriptor = LawDescriptor {
    kind: "concrete.kent_park",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Peak compressive strain of (A.9) and (A.10), dimensionless.
const PEAK_STRAIN: Real = 0.002;
/// Plateau ratio of (A.15).
const RESIDUAL_RATIO: Real = 0.2;
/// (A.12) is undefined at and below this cylinder strength, in psi.
const MIN_FC_PSI: Real = 1000.0;
/// 1 lbf in newtons. Unit factor for (A.12) only.
const LBF_IN_NEWTONS: Real = 4.4482216152605;
/// 1 inch in metres. Unit factor for (A.12) only.
const INCH_IN_METRES: Real = 0.0254;
/// 1 psi in pascals: lbf/in² with the two factors above.
const PSI_IN_PA: Real = LBF_IN_NEWTONS / (INCH_IN_METRES * INCH_IN_METRES);

/// SI parameters of [`KentParkFactory`], validated on construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KentParkParams {
    /// Cylinder strength f'_c in Pa, finite and strictly positive.
    /// (A.12) also requires this value to exceed 1000 psi.
    pub fc: Real,
    /// Volumetric hoop ratio ρ_s, dimensionless, finite and nonnegative.
    /// The law does not evaluate (A.14).
    pub rho_s: Real,
    /// Confined-core width b'' in m, finite and strictly positive.
    pub b_double_prime: Real,
    /// Hoop centre spacing s_h in m, finite and strictly positive.
    pub s_h: Real,
}

/// Typed factory for [`KentPark`], without state or a global registry.
#[derive(Clone, Copy, Debug, Default)]
pub struct KentParkFactory;

impl UniaxialFactory for KentParkFactory {
    type Params = KentParkParams;

    fn descriptor(&self) -> LawDescriptor {
        KENT_PARK
    }

    fn build(&self, params: KentParkParams) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(KentPark::new(
            params.fc,
            params.rho_s,
            params.b_double_prime,
            params.s_h,
        )?))
    }
}

/// Monotonic concrete envelope (A.9), (A.10) and (A.15), tension-positive.
///
/// ε is total and dimensionless. σ and the tangent are in Pa. Compression in
/// the report is positive; this law uses the engine sign, so engine strain,
/// stress and tangent are −ε_c, −f_c and df_c/dε_c. History length is zero:
/// the envelope does not remember a cycle.
///
/// At ε_c = 0.002 the parabola (A.9) is selected, so the tangent is 0.
/// At ε_20c the falling line (A.10) is selected. Tensile strain is outside
/// the cited curve and returns zero stress and zero tangent. The derivative
/// used on the parabola is f'_c · (2/0.002) · (1 − ε_c/0.002).
///
/// ```
/// use engine_contracts::UniaxialLaw;
/// use engine_laws::uniaxial::KentPark;
/// let psi = 4.4482216152605 / (0.0254 * 0.0254);
/// let law = KentPark::new(3000.0 * psi, 0.0, 0.2, 0.1)?;
/// let response = law.respond(-0.001, &[])?;
/// assert_eq!(response.stress, -0.75 * law.params().fc);
/// assert_eq!(response.tangent, 500.0 * law.params().fc);
/// # Ok::<(), engine_core::EngineError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KentPark {
    params: KentParkParams,
    /// Slope parameter Z of (A.11), dimensionless.
    z: Real,
    /// Compressive strain where (A.10) meets (A.15).
    eps20: Real,
    /// Parabola tangent at zero strain, in Pa.
    k0: Real,
}

impl KentPark {
    /// Constructs from f'_c (Pa), ρ_s, b'' (m) and s_h (m).
    ///
    /// # Errors
    /// `ENG-E-0008` for a nonfinite parameter or intermediate.
    /// `ENG-E-0010` when f'_c, b'' or s_h is not strictly positive, ρ_s is
    /// negative, f'_c does not exceed 1000 psi, or the denominator of Z is
    /// not strictly positive.
    pub fn new(fc: Real, rho_s: Real, b_double_prime: Real, s_h: Real) -> EResult<Self> {
        let fc = positive(fc, "fc (Pa)")?;
        finite(rho_s, "rho_s")?;
        if rho_s < 0.0 {
            return Err(EngineError::new("ENG-E-0010", "rho_s must be nonnegative."));
        }
        let b_double_prime = positive(b_double_prime, "b_double_prime (m)")?;
        let s_h = positive(s_h, "s_h (m)")?;
        let fc_psi = finite(fc / PSI_IN_PA, "fc (psi)")?;
        if fc_psi <= MIN_FC_PSI {
            return Err(EngineError::new(
                "ENG-E-0010",
                "fc (Pa) must be greater than 1000 psi.",
            ));
        }
        // (A.12) gives ε_50u − 0.002 = 5 / (f'_c,psi − 1000). Substituting into
        // (A.11) avoids subtracting two quantities that both round to 0.002.
        let strength_gap = finite(fc_psi - MIN_FC_PSI, "epsilon_50u denominator (psi)")?;
        let eps50h = epsilon_50h(rho_s, b_double_prime, s_h)?;
        let confined = finite(eps50h * strength_gap, "epsilon_50h*(fc_psi-1000)")?;
        let z_scale = finite(5.0 + confined, "Z denominator scale")?;
        if z_scale <= 0.0 {
            return Err(EngineError::new(
                "ENG-E-0010",
                "Z denominator must be strictly positive.",
            ));
        }
        let z = finite(0.5 * strength_gap / z_scale, "Z")?;
        let eps20 = finite(PEAK_STRAIN + 0.8 / z, "epsilon_20c")?;
        let k0 = finite(fc * (2.0 / PEAK_STRAIN), "initial tangent (Pa)")?;
        Ok(Self {
            params: KentParkParams {
                fc,
                rho_s,
                b_double_prime,
                s_h,
            },
            z,
            eps20,
            k0,
        })
    }

    /// Original validated parameters, immutable.
    pub fn params(&self) -> &KentParkParams {
        &self.params
    }
}

/// (A.13). A zero hoop ratio makes the product zero without dividing the core sizes.
fn epsilon_50h(rho_s: Real, b_double_prime: Real, s_h: Real) -> EResult<Real> {
    if rho_s == 0.0 {
        return Ok(0.0);
    }
    let ratio = finite(b_double_prime / s_h, "b_double_prime/s_h")?;
    let root = finite(math::sqrt(ratio), "sqrt(b_double_prime/s_h)")?;
    finite(0.75 * rho_s * root, "epsilon_50h")
}

impl UniaxialLaw for KentPark {
    fn kind(&self) -> &'static str {
        KENT_PARK.kind
    }

    fn contract(&self) -> ContractId {
        KENT_PARK.contract
    }

    fn state_len(&self) -> usize {
        0
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    fn initial_tangent(&self) -> Real {
        self.k0
    }

    /// Evaluates the envelope without mutating the object or accepting history.
    ///
    /// `ENG-E-0010` for nonempty history. `ENG-E-0008` for nonfinite strain or
    /// a nonfinite stress or tangent.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0, "Accepted history")?;
        finite(strain, "ε total")?;
        let trial_state = Vec::new();
        if strain > 0.0 {
            return Ok(UniaxialResponse {
                stress: 0.0,
                tangent: 0.0,
                trial_state,
            });
        }
        let eps_c = -strain;
        let (paper, tangent) = if eps_c <= PEAK_STRAIN {
            let x = finite(eps_c / PEAK_STRAIN, "epsilon_c/0.002")?;
            let ratio = finite(x * (2.0 - x), "parabola ratio")?;
            let paper = finite(self.params.fc * ratio, "f_c (Pa)")?;
            let tangent = finite(self.k0 * (1.0 - x), "K (Pa)")?;
            (paper, tangent)
        } else if eps_c <= self.eps20 {
            let delta = finite(eps_c - PEAK_STRAIN, "epsilon_c-0.002")?;
            let drop = finite(self.z * delta, "Z*(epsilon_c-0.002)")?;
            let ratio = finite(1.0 - drop, "descending ratio")?;
            let paper = finite(self.params.fc * ratio, "f_c (Pa)")?;
            let tangent = finite(-self.z * self.params.fc, "K (Pa)")?;
            (paper, tangent)
        } else {
            let paper = finite(RESIDUAL_RATIO * self.params.fc, "0.2 f_c (Pa)")?;
            (paper, 0.0)
        };
        Ok(UniaxialResponse {
            stress: finite(-paper, "σ (Pa)")?,
            tangent,
            trial_state,
        })
    }
}
