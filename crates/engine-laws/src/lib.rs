#![forbid(unsafe_code)]
//! Pure constitutive laws, without external dependencies.
//!
//! [`uniaxial::Elastic`] and [`uniaxial::MultilinearElastic`] implement
//! `uniaxial/1` (kinds `elastic` and `multilinear_elastic`), and
//! [`section::ElasticFrame`] implements `section/1` (kind `elastic_frame`).
//! [`uniaxial::SteelBilinearKinematic`] implements `uniaxial/1`, with plasticity
//! history and kind `steel.bilinear_kinematic`.
//! [`uniaxial::CubicElastic`] implements `uniaxial/1`, kind `elastic.cubic`,
//! with a smooth elastic curve derived from an original quartic energy.
//! Parameters and results use SI; there is no unit conversion, geometric
//! calculation, or global registry. Typed factories construct
//! `engine-contracts` trait objects without coupling the consumer to an
//! implementation. Descriptors distinguish method and contract versions;
//! API changes follow semver, initially 0.1.0.
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

/// Static metadata without a registry, I/O, or implicit construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LawDescriptor {
    /// Stable internal identifier, equal to `kind()` of the constructed object.
    pub kind: &'static str,
    /// Implemented public contract; its version is independent of the method.
    pub contract: ContractId,
    /// Method semantic version: major, minor, patch.
    pub version: (u16, u16, u16),
}

fn finite(value: Real, name: &str) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            format!("{name} must be finite; also check for overflow."),
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
            format!("{name} must be strictly positive."),
        ))
    }
}

fn vector(values: &[Real], len: usize, name: &str) -> EResult<()> {
    if values.len() != len {
        return Err(EngineError::new(
            "ENG-E-0010",
            format!("{name}: length {}; expected {len}.", values.len()),
        ));
    }
    for &value in values {
        finite(value, name)?;
    }
    Ok(())
}
