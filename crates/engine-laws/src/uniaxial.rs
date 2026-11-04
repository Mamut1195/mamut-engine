//! `uniaxial/1` laws: elastic (`elastic`, `elastic.cubic`, `multilinear_elastic`),
//! plasticity with history (`steel.bilinear_kinematic`), and the monotonic
//! concrete envelope `concrete.kent_park`.

mod cubic_elastic;
pub use cubic_elastic::{CubicElastic, CubicElasticFactory, CubicElasticParams, CUBIC_ELASTIC};
mod multilinear;
pub use multilinear::{
    MultilinearElastic, MultilinearElasticFactory, MultilinearElasticParams, MultilinearMode,
    MULTILINEAR_ELASTIC,
};
mod kent_park;
pub use kent_park::{KentPark, KentParkFactory, KentParkParams, KENT_PARK};
mod steel_bilinear;
pub use steel_bilinear::{
    SteelBilinearKinematic, SteelBilinearKinematicFactory, SteelBilinearKinematicParams,
    STEEL_BILINEAR_KINEMATIC,
};

use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, Real};

/// Linear elastic law descriptor, method 0.1.0.
pub const ELASTIC: LawDescriptor = LawDescriptor {
    kind: "elastic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// SI parameters of [`ElasticFactory`]; validated when constructing the law.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElasticParams {
    /// Young's modulus E, in Pa, finite and strictly positive.
    pub e: Real,
}

/// Interchangeable uniaxial law factory, without state or a global registry.
///
/// Implementations sharing `Params` can be interchanged as trait
/// objects. Each factory validates the units/ranges documented by its
/// parameters and returns an owned `Send + Sync` law.
pub trait UniaxialFactory: Send + Sync {
    /// Typed SI parameters of the implementation.
    type Params;
    /// Kind, contract, and semantic version of the constructed method.
    fn descriptor(&self) -> LawDescriptor;
    /// Constructs a validated law; errors depend on the implementation.
    fn build(&self, params: Self::Params) -> EResult<Box<dyn UniaxialLaw>>;
}

/// Factory for [`Elastic`]; adds no parameters or state.
#[derive(Clone, Copy, Debug, Default)]
pub struct ElasticFactory;

impl UniaxialFactory for ElasticFactory {
    type Params = ElasticParams;

    fn descriptor(&self) -> LawDescriptor {
        ELASTIC
    }

    fn build(&self, params: ElasticParams) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(Elastic::new(params.e)?))
    }
}

/// Linear elastic material: σ = E ε, without preload or history.
///
/// E and σ in Pa; total dimensionless ε, positive in tension. The tangent
/// dσ/dε = E is constant. The allowed range is any finite ε whose
/// product E ε is finite; this idealization does not represent yielding or damage.
/// Accepted and trial histories have zero length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Elastic {
    e: Real,
}

impl Elastic {
    /// Constructs with E in Pa, finite and > 0.
    ///
    /// # Errors
    /// `ENG-E-0008` for NaN/infinity; `ENG-E-0010` for E ≤ 0.
    pub fn new(e: Real) -> EResult<Self> {
        Ok(Self {
            e: positive(e, "E (Pa)")?,
        })
    }

    /// Validated Young's modulus, in Pa.
    pub fn young_modulus(&self) -> Real {
        self.e
    }
}

impl UniaxialLaw for Elastic {
    fn kind(&self) -> &'static str {
        ELASTIC.kind
    }

    fn contract(&self) -> ContractId {
        ELASTIC.contract
    }

    fn state_len(&self) -> usize {
        0
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    fn initial_tangent(&self) -> Real {
        self.e
    }

    /// Evaluates total ε without mutating the law or accepting history.
    ///
    /// Returns σ and tangent in Pa, and empty history. `ENG-E-0010` for
    /// nonempty history; `ENG-E-0008` for nonfinite ε or overflowed product.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0, "Accepted history")?;
        finite(strain, "ε")?;
        Ok(UniaxialResponse {
            stress: finite(self.e * strain, "σ (Pa)")?,
            tangent: self.e,
            trial_state: Vec::new(),
        })
    }
}
