//! Elasticidad cúbica conservativa; derivación propia en docs/GUIA_METODOS.md §5.

use super::UniaxialFactory;
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Descriptor de la elasticidad cúbica, método 0.1.0.
pub const CUBIC_ELASTIC: LawDescriptor = LawDescriptor {
    kind: "elastic.cubic",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Parámetros SI de [`CubicElasticFactory`], validados al construir.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicElasticParams {
    /// Módulo inicial E en Pa, finito y estrictamente positivo.
    pub e: Real,
    /// Coeficiente cúbico C en Pa, finito y no negativo.
    pub c: Real,
}

/// Fábrica tipada de [`CubicElastic`], sin estado ni registro global.
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

/// Ley elástica suave σ = Eε + Cε³, sin historia, precarga, daño ni plasticidad.
///
/// ε es total, adimensional y positiva en tracción; σ, E, C y la tangente
/// están en Pa. La energía W = Eε²/2 + Cε⁴/4 es estrictamente convexa:
/// σ = dW/dε y K = E + 3Cε² > 0. La descarga recorre la misma curva.
/// C = 0 recupera la elasticidad lineal. Es una idealización local 1D;
/// no garantiza validez física para grandes deformaciones.
///
/// La historia inicial, aceptada y de prueba tiene longitud cero. El dominio
/// numérico exige ε y todas las operaciones q = (Cε)ε, σ = (E+q)ε y K = E+3q
/// finitas. Se admite el redondeo/subdesbordamiento habitual de f64. No se forma
/// ε² por separado, para conservar el límite C = 0 a deformaciones grandes.
/// Derivación y benchmark propios: docs/GUIA_METODOS.md §5, (1)–(3), ENG-CORE-011.
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
    /// Construye con E > 0 y C ≥ 0 en Pa, ambos finitos.
    ///
    /// # Errores
    /// `ENG-E-0008` por parámetros no finitos; `ENG-E-0010` por E ≤ 0 o C < 0.
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

    /// Parámetros originales validados, inmutables.
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

    /// Evalúa sin mutar el objeto ni aceptar historia.
    ///
    /// `ENG-E-0010` por historia no vacía; `ENG-E-0008` por ε no finita
    /// o desbordamiento en productos/sumas de tensión o tangente.
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
