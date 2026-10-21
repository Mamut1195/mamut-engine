//! <Method and relative link to its sheet with equations/citations>.
//! Template for engine-laws/src/uniaxial/mi_metodo.rs.
//! Replace Metodo, Params, descriptor, parameters and pending response.

use super::UniaxialFactory;
use crate::{finite, positive, vector, LawDescriptor};
use engine_contracts::{UniaxialLaw, UniaxialResponse};
use engine_core::{ContractId, EResult, EngineError, Real};

/// Own method identity and version, independent of the contract.
pub const METODO: LawDescriptor = LawDescriptor {
    kind: "metodo.pendiente",
    contract: ContractId::new("uniaxial", 1, 0),
    version: (0, 1, 0),
};

/// Replace with SI parameters from the method sheet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetodoParams {
    /// Example initial modulus, in Pa, finite and positive.
    pub e: Real,
}

/// Typed factory without global registration.
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

/// Complete assumptions, equations, signs, units, domain and history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metodo {
    params: MetodoParams,
}

impl Metodo {
    /// Validates parameters; complete ranges and diagnostics from the method sheet.
    ///
    /// # Errors
    /// ENG-E-0008 for nonfinite values; ENG-E-0010 for invalid range.
    pub fn new(e: Real) -> EResult<Self> {
        Ok(Self {
            params: MetodoParams {
                e: positive(e, "E (Pa)")?,
            },
        })
    }

    /// Original validated, immutable parameters.
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
        0 // Replace if there is history; document components and units.
    }

    fn initial_state(&self) -> Vec<Real> {
        Vec::new()
    }

    fn initial_tangent(&self) -> Real {
        self.params.e // Derive from the method sheet and check at the origin.
    }

    fn respond(&self, strain: Real, committed: &[Real]) -> EResult<UniaxialResponse> {
        vector(committed, self.state_len(), "Historia aceptada")?;
        finite(strain, "ε total")?;
        // Replace with method sheet equations, validating intermediates,
        // stress, tangent and history. Never accept or mutate committed.
        Err(EngineError::new(
            "ENG-E-0013",
            "Plantilla pendiente de implementar desde su ficha matemática.",
        ))
    }
}
