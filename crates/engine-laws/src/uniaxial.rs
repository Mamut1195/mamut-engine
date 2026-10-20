//! Leyes uniaxiales `uniaxial/1`: elásticas (`elastic`, `elastic.cubic`, `multilinear_elastic`) y
//! de plasticidad con historia (`steel.bilinear_kinematic`).

mod cubic_elastic;
pub use cubic_elastic::{CubicElastic, CubicElasticFactory, CubicElasticParams, CUBIC_ELASTIC};
mod multilinear;
pub use multilinear::{
    MultilinearElastic, MultilinearElasticFactory, MultilinearElasticParams, MultilinearMode,
    MULTILINEAR_ELASTIC,
};
mod steel_bilinear;
pub use steel_bilinear::{
    SteelBilinearKinematic, SteelBilinearKinematicFactory, SteelBilinearKinematicParams,
    STEEL_BILINEAR_KINEMATIC,
};

use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, Real};

/// Descriptor de la ley elástica lineal, método 0.1.0.
pub const ELASTIC: LawDescriptor = LawDescriptor {
    kind: "elastic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Parámetros SI de [`ElasticFactory`]; se validan al construir la ley.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElasticParams {
    /// Módulo de Young E, en Pa, finito y estrictamente positivo.
    pub e: Real,
}

/// Fábrica sustituible de leyes uniaxiales, sin estado ni registro global.
///
/// Implementaciones que comparten `Params` se pueden intercambiar como objetos
/// del trait. Cada fábrica valida las unidades/rangos documentados por sus
/// parámetros y devuelve una ley propietaria, `Send + Sync`.
pub trait UniaxialFactory: Send + Sync {
    /// Parámetros tipados SI de la implementación.
    type Params;
    /// Kind, contrato y versión semántica del método construido.
    fn descriptor(&self) -> LawDescriptor;
    /// Construye una ley validada; errores según la implementación.
    fn build(&self, params: Self::Params) -> EResult<Box<dyn UniaxialLaw>>;
}

/// Fábrica de [`Elastic`]; no añade parámetros ni estado.
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

/// Material elástico lineal: σ = E ε, sin precarga ni historia.
///
/// E y σ en Pa; ε total adimensional, positiva en tracción. La tangente
/// dσ/dε = E es constante. El rango admitido es cualquier ε finita cuyo
/// producto E ε sea finito; esta idealización no representa fluencia ni daño.
/// La historia aceptada y la historia de prueba tienen longitud cero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Elastic {
    e: Real,
}

impl Elastic {
    /// Construye con E en Pa, finito y > 0.
    ///
    /// # Errores
    /// `ENG-E-0008` por NaN/infinito; `ENG-E-0010` por E ≤ 0.
    pub fn new(e: Real) -> EResult<Self> {
        Ok(Self {
            e: positive(e, "E (Pa)")?,
        })
    }

    /// Módulo de Young validado, en Pa.
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

    /// Evalúa ε total sin mutar la ley ni aceptar historia.
    ///
    /// Devuelve σ y tangente en Pa, e historia vacía. `ENG-E-0010` por
    /// historia no vacía; `ENG-E-0008` por ε no finita o producto desbordado.
    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, 0, "Historia aceptada")?;
        finite(strain, "ε")?;
        Ok(UniaxialResponse {
            stress: finite(self.e * strain, "σ (Pa)")?,
            tangent: self.e,
            trial_state: Vec::new(),
        })
    }
}
