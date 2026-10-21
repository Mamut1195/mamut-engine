#![forbid(unsafe_code)]
//! Basic types and stable diagnostics for the computation core.
//!
//! Scalar functions are centralized in [`math`] with pinned libm. The
//! messages are in Spanish and may evolve; the codes are stable.
//!
//! ```
//! use engine_core::{ContractId, Diagnostic, Dim, Dof, EngineError, EResult};
//!
//! assert_eq!(Dim::D2.dofs(), &[Dof::Ux, Dof::Uy, Dof::Rz]);
//! let contrato = ContractId::new("modelo", 1, 2);
//! assert_eq!(contrato.to_string(), "modelo/1");
//! assert!(contrato.is_compatible_with(&ContractId::new("modelo", 1, 8)));
//! let aviso = Diagnostic::from_code("ENG-E-0001").unwrap();
//! let resultado: EResult<()> = Err(EngineError::from_code(aviso.code).unwrap());
//! assert!(resultado.is_err());
//! ```

#![deny(missing_docs)]

use std::fmt;

pub mod catalog;
pub mod math;
pub mod units;

/// Engine core version, derived from the `engine-core` manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Real number used for model quantities and results.
pub type Real = f64;

/// Node identifier, without requiring contiguous numbering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(
    /// Numeric value of the identifier.
    pub u32,
);

impl NodeId {
    /// Constructs a node identifier from its value.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Member identifier, without requiring contiguous numbering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberId(
    /// Numeric value of the identifier.
    pub u32,
);

impl MemberId {
    /// Constructs a member identifier from its value.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Global position of a degree of freedom in the system of equations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DofIndex(
    /// Numeric position, counted from zero.
    pub u32,
);

impl DofIndex {
    /// Constructs a global degree-of-freedom position.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Displacement or rotation component of a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dof {
    /// Displacement along the X axis.
    Ux,
    /// Displacement along the Y axis.
    Uy,
    /// Displacement along the Z axis.
    Uz,
    /// Rotation about the X axis.
    Rx,
    /// Rotation about the Y axis.
    Ry,
    /// Rotation about the Z axis.
    Rz,
}

impl Dof {
    /// Returns the component index in spatial order, starting at zero.
    ///
    /// This index is independent of dimension: Rz always occupies index
    /// 5, even though it is the third component in a planar model.
    ///
    /// ```
    /// use engine_core::Dof;
    /// assert_eq!(Dof::Ux.index(), 0);
    /// assert_eq!(Dof::Rz.index(), 5);
    /// ```
    pub const fn index(self) -> usize {
        match self {
            Self::Ux => 0,
            Self::Uy => 1,
            Self::Uz => 2,
            Self::Rx => 3,
            Self::Ry => 4,
            Self::Rz => 5,
        }
    }

    /// Returns the stable component label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ux => "Ux",
            Self::Uy => "Uy",
            Self::Uz => "Uz",
            Self::Rx => "Rx",
            Self::Ry => "Ry",
            Self::Rz => "Rz",
        }
    }
}

impl fmt::Display for Dof {
    /// Writes the stable component label.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Geometric dimension of the model and set of available components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dim {
    /// Planar model with two displacements and one rotation.
    D2,
    /// Spatial model with three displacements and three rotations.
    D3,
}

impl Dim {
    /// Returns the dimension's components in their local order.
    ///
    /// ```
    /// use engine_core::{Dim, Dof};
    /// assert_eq!(Dim::D2.dofs(), &[Dof::Ux, Dof::Uy, Dof::Rz]);
    /// assert_eq!(Dim::D3.dofs().len(), 6);
    /// ```
    pub const fn dofs(self) -> &'static [Dof] {
        match self {
            Self::D2 => &[Dof::Ux, Dof::Uy, Dof::Rz],
            Self::D3 => &[Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz],
        }
    }
}

/// Contract identified by a name and a major and minor version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContractId {
    /// Stable contract name.
    pub name: &'static str,
    /// Major version; a change indicates incompatibility.
    pub major: u16,
    /// Minor version; changes preserve compatibility.
    pub minor: u16,
}

impl ContractId {
    /// Constructs a contract with its name and both parts of its version.
    pub const fn new(name: &'static str, major: u16, minor: u16) -> Self {
        Self { name, major, minor }
    }

    /// Checks that both contracts have the same name and major version.
    ///
    /// The check is symmetric and does not depend on the minor version.
    ///
    /// ```
    /// use engine_core::ContractId;
    /// let contrato = ContractId::new("modelo", 1, 0);
    /// assert!(contrato.is_compatible_with(&ContractId::new("modelo", 1, 3)));
    /// assert!(!contrato.is_compatible_with(&ContractId::new("modelo", 2, 0)));
    /// assert!(!contrato.is_compatible_with(&ContractId::new("resultado", 1, 0)));
    /// ```
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.name == other.name && self.major == other.major
    }
}

impl fmt::Display for ContractId {
    /// Writes the name and major version separated by a slash.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.name, self.major)
    }
}

/// Reference to an entity involved in a diagnostic or error.
#[derive(Clone, Debug)]
pub enum EntityRef {
    /// Reference to a node.
    Node(
        /// Node identifier.
        NodeId,
    ),
    /// Reference to a member.
    Member(
        /// Member identifier.
        MemberId,
    ),
    /// Reference to a global degree of freedom.
    Dof(
        /// Global degree-of-freedom position.
        DofIndex,
    ),
    /// Reference to an analysis case.
    Case(
        /// Case name.
        String,
    ),
    /// Reference to a constitutive law.
    Law(
        /// Law name.
        String,
    ),
}

/// Diagnostic severity level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Information that does not require correction.
    Info,
    /// Warning requiring review, without necessarily preventing computation.
    Warning,
    /// Error that prevents completion of the requested operation.
    Error,
}

/// Diagnostic with a stable code, message in Spanish, and optional context.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// Stable situation code, in ENG-E-NNNN format in the catalog.
    pub code: &'static str,
    /// Severity of the described situation.
    pub severity: Severity,
    /// Message intended for the person using the engine.
    pub message: String,
    /// Entities involved, in the order supplied by the diagnostic producer.
    pub entities: Vec<EntityRef>,
    /// Optional suggestion for resolving or investigating the situation.
    pub hint: Option<String>,
}

impl Diagnostic {
    /// Constructs a diagnostic with no entities or hint.
    ///
    /// Allows custom codes; for catalog codes, the recommended method is
    /// [`Self::from_code`], which provides the registered severity and message.
    pub fn new(code: &'static str, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            entities: Vec::new(),
            hint: None,
        }
    }

    /// Constructs a diagnostic from catalog data, or returns `None`.
    ///
    /// The diagnostic starts with no entities or hint.
    ///
    /// ```
    /// use engine_core::{Diagnostic, Severity};
    /// let diagnostico = Diagnostic::from_code("ENG-E-0002").unwrap();
    /// assert_eq!(diagnostico.severity, Severity::Error);
    /// assert!(Diagnostic::from_code("desconocido").is_none());
    /// ```
    pub fn from_code(code: &str) -> Option<Self> {
        catalog::lookup(code).map(|info| Self::new(info.code, info.severity, info.message))
    }
}

/// Error from an engine operation, with a stable code and entities involved.
#[derive(Clone, Debug)]
pub struct EngineError {
    /// Stable situation code.
    pub code: &'static str,
    /// Message in Spanish describing the failure.
    pub message: String,
    /// Entities related to the failure.
    pub entities: Vec<EntityRef>,
}

impl EngineError {
    /// Constructs an error with a custom message and no related entities.
    ///
    /// To obtain a registered message, [`Self::from_code`] is recommended.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            entities: Vec::new(),
        }
    }

    /// Constructs an error with the catalog code and message, or returns `None`.
    ///
    /// ```
    /// use engine_core::EngineError;
    /// let error = EngineError::from_code("ENG-E-0005").unwrap();
    /// assert_eq!(error.to_string(), "ENG-E-0005: La longitud del miembro es nula.");
    /// assert!(EngineError::from_code("desconocido").is_none());
    /// ```
    pub fn from_code(code: &str) -> Option<Self> {
        catalog::lookup(code).map(|info| Self::new(info.code, info.message))
    }
}

impl fmt::Display for EngineError {
    /// Writes the code and message separated by a colon.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for EngineError {}

/// Result of an engine operation that may produce an [`EngineError`].
pub type EResult<T> = Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks all component indices and labels and their local order.
    #[test]
    fn dimensions_and_components() {
        assert_eq!(Dim::D2.dofs(), &[Dof::Ux, Dof::Uy, Dof::Rz]);
        let spatial = [Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz];
        assert_eq!(Dim::D3.dofs(), &spatial);
        for (index, (dof, label)) in spatial
            .iter()
            .zip(["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"])
            .enumerate()
        {
            assert_eq!(dof.index(), index);
            assert_eq!(dof.label(), label);
            assert_eq!(dof.to_string(), label);
        }
    }

    /// Checks contract representation and compatibility boundaries.
    #[test]
    fn contract_display_and_compatibility() {
        let contract = ContractId::new("modelo", 1, 2);
        assert_eq!(contract.to_string(), "modelo/1");
        let newer = ContractId::new("modelo", 1, 9);
        assert!(contract.is_compatible_with(&newer));
        assert!(newer.is_compatible_with(&contract));
        assert!(!contract.is_compatible_with(&ContractId::new("modelo", 2, 2)));
        assert!(!contract.is_compatible_with(&ContractId::new("otro", 1, 2)));
    }

    /// Checks that constructors preserve values and empty context.
    #[test]
    fn constructors_and_error_display() {
        assert_eq!(NodeId::new(7), NodeId(7));
        assert_eq!(MemberId::new(8), MemberId(8));
        assert_eq!(DofIndex::new(9), DofIndex(9));
        let diagnostic = Diagnostic::new("ENG-E-0001", Severity::Warning, "Revisar unidad.");
        assert_eq!(diagnostic.code, "ENG-E-0001");
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(diagnostic.message, "Revisar unidad.");
        assert!(diagnostic.entities.is_empty());
        assert!(diagnostic.hint.is_none());
        let error = EngineError::new("ENG-E-0012", "Fallo interno.");
        assert_eq!(error.to_string(), "ENG-E-0012: Fallo interno.");
        assert!(error.entities.is_empty());
        let standard_error: &dyn std::error::Error = &error;
        assert!(standard_error.source().is_none());
    }

    /// Checks that all constructors by code use the catalog.
    #[test]
    fn catalog_constructors() {
        for info in catalog::CODES {
            let diagnostic = Diagnostic::from_code(info.code).unwrap();
            assert_eq!(diagnostic.code, info.code);
            assert_eq!(diagnostic.severity, info.severity);
            assert_eq!(diagnostic.message, info.message);
            assert!(diagnostic.entities.is_empty());
            assert!(diagnostic.hint.is_none());
            let error = EngineError::from_code(info.code).unwrap();
            assert_eq!(error.code, info.code);
            assert_eq!(error.message, info.message);
            assert!(error.entities.is_empty());
        }
        assert!(Diagnostic::from_code("ENG-E-9999").is_none());
        assert!(EngineError::from_code("ENG-E-9999").is_none());
    }
}
