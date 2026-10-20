//! <Método y enlace relativo a su ficha con ecuaciones/citas>.
//! Plantilla para engine-laws/src/uniaxial/mi_metodo.rs.
//! Sustituir Metodo, Params, descriptor, parámetros y respuesta pendiente.

use super::UniaxialFactory;
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Identidad propia y versión del método, independiente del contrato.
pub const METODO: LawDescriptor = LawDescriptor {
    kind: "metodo.pendiente",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Sustituir por los parámetros SI de la ficha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetodoParams {
    /// Ejemplo de módulo inicial, en Pa, finito y positivo.
    pub e: Real,
}

/// Fábrica tipada sin registro global.
#[derive(Clone, Copy, Debug, Default)]
pub struct MetodoFactory;

impl UniaxialFactory for MetodoFactory {
    type Params = MetodoParams;

    fn descriptor(&self) -> LawDescriptor {
        METODO
    }

    fn build(&self, params: MetodoParams) -> EResult<Box<dyn UniaxialLaw>> {
        Ok(Box::new(Metodo::new(params.e)?))
    }
}

/// Completar hipótesis, ecuaciones, signos, unidades, dominio e historia.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metodo {
    params: MetodoParams,
}

impl Metodo {
    /// Valida parámetros; completar rangos y diagnósticos de la ficha.
    ///
    /// # Errores
    /// ENG-E-0008 por no finitos; ENG-E-0010 por rango inválido.
    pub fn new(e: Real) -> EResult<Self> {
        Ok(Self {
            params: MetodoParams {
                e: positive(e, "E (Pa)")?,
            },
        })
    }

    /// Parámetros originales validados, inmutables.
    pub fn params(&self) -> &MetodoParams {
        &self.params
    }
}

impl UniaxialLaw for Metodo {
    fn kind(&self) -> &'static str {
        METODO.kind
    }

    fn contract(&self) -> ContractId {
        METODO.contract
    }

    fn state_len(&self) -> usize {
        0 // Sustituir si hay historia; documentar componentes y unidades.
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    fn initial_tangent(&self) -> Real {
        self.params.e // Derivar desde la ficha y comprobar en el origen.
    }

    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, self.state_len(), "Historia aceptada")?;
        finite(strain, "ε total")?;
        // Reemplazar por las ecuaciones de la ficha, validando intermedios,
        // tensión, tangente e historia. Nunca aceptar ni mutar committed.
        Err(EngineError::new(
            "ENG-E-0013",
            "Plantilla pendiente de implementar desde su ficha matemática.",
        ))
    }
}
