#![forbid(unsafe_code)]
//! Leyes constitutivas puras, sin dependencias externas.
//!
//! [`uniaxial::Elastic`] y [`uniaxial::MultilinearElastic`] implementan
//! `uniaxial/1` (kinds `elastic` y `multilinear_elastic`), y
//! [`section::ElasticFrame`] implementa `section/1` (kind `elastic_frame`).
//! [`uniaxial::SteelBilinearKinematic`] implementa `uniaxial/1`, con historia
//! de plasticidad y kind `steel.bilinear_kinematic`.
//! Los parámetros y resultados usan SI; no hay conversión de unidades, cálculo
//! geométrico ni registro global. Las fábricas tipadas permiten construir
//! objetos de los traits de `engine-contracts` sin acoplar al consumidor a una
//! implementación. Los descriptores distinguen versión del método y contrato;
//! los cambios de API siguen semver, inicialmente 0.1.0.
//!
//! ```
//! use engine_contracts::UniaxialLaw;
//! use engine_laws::uniaxial::Elastic;
//! let law = Elastic::new(200e9)?;
//! assert_eq!(law.respond(0.001, &[])?.stress, 200e6);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

use engine_core::{ContractId, EResult, EngineError, Real};

pub mod section;
pub mod uniaxial;

/// Metadatos estáticos sin registro, E/S ni construcción implícita.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LawDescriptor {
    /// Identificador propio estable, igual a `kind()` del objeto construido.
    pub kind: &'static str,
    /// Contrato público implementado; su versión es independiente del método.
    pub contract: ContractId,
    /// Versión semántica del método: mayor, menor, parche.
    pub version: (u16, u16, u16),
}

fn finite(value: Real, name: &str) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            format!("{name} debe ser finito; compruebe también desbordamientos."),
        ))
    }
}

fn positive(value: Real, name: &str) -> EResult<Real> {
    finite(value, name)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0010",
            format!("{name} debe ser estrictamente positivo."),
        ))
    }
}

fn vector(values: &[Real], len: usize, name: &str) -> EResult<()> {
    if values.len() != len {
        return Err(EngineError::new(
            "ENG-E-0010",
            format!("{name}: longitud {}; se esperaba {len}.", values.len()),
        ));
    }
    for &value in values {
        finite(value, name)?;
    }
    Ok(())
}
