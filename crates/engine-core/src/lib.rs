#![forbid(unsafe_code)]
//! Tipos básicos y diagnósticos estables del núcleo de cálculo.
//!
//! Las funciones escalares se centralizan en [`math`] con libm fijada. Los
//! mensajes están en español y pueden evolucionar; los códigos son estables.
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

/// Versión del núcleo del motor, derivada del manifiesto de `engine-core`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Número real utilizado en las magnitudes del modelo y sus resultados.
pub type Real = f64;

/// Identificador de un nodo, sin imponer una numeración contigua.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(
    /// Valor numérico del identificador.
    pub u32,
);

impl NodeId {
    /// Construye un identificador de nodo a partir de su valor.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Identificador de un miembro, sin imponer una numeración contigua.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberId(
    /// Valor numérico del identificador.
    pub u32,
);

impl MemberId {
    /// Construye un identificador de miembro a partir de su valor.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Posición global de un grado de libertad en el sistema de ecuaciones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DofIndex(
    /// Posición numérica, contada desde cero.
    pub u32,
);

impl DofIndex {
    /// Construye una posición global de grado de libertad.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Componente de desplazamiento o giro de un nodo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dof {
    /// Desplazamiento en el eje X.
    Ux,
    /// Desplazamiento en el eje Y.
    Uy,
    /// Desplazamiento en el eje Z.
    Uz,
    /// Giro alrededor del eje X.
    Rx,
    /// Giro alrededor del eje Y.
    Ry,
    /// Giro alrededor del eje Z.
    Rz,
}

impl Dof {
    /// Devuelve el índice de la componente en el orden espacial, desde cero.
    ///
    /// Este índice es independiente de la dimensión: Rz siempre ocupa el índice
    /// 5, aunque sea la tercera componente de un modelo plano.
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

    /// Devuelve la etiqueta estable de la componente.
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
    /// Escribe la etiqueta estable de la componente.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Dimensión geométrica del modelo y conjunto de componentes disponibles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dim {
    /// Modelo plano con dos desplazamientos y un giro.
    D2,
    /// Modelo espacial con tres desplazamientos y tres giros.
    D3,
}

impl Dim {
    /// Devuelve las componentes de la dimensión en su orden local.
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

/// Contrato identificado por un nombre y una versión mayor y menor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContractId {
    /// Nombre estable del contrato.
    pub name: &'static str,
    /// Versión mayor; un cambio indica incompatibilidad.
    pub major: u16,
    /// Versión menor; sus cambios conservan la compatibilidad.
    pub minor: u16,
}

impl ContractId {
    /// Construye un contrato con el nombre y las dos partes de su versión.
    pub const fn new(name: &'static str, major: u16, minor: u16) -> Self {
        Self { name, major, minor }
    }

    /// Comprueba que ambos contratos tengan el mismo nombre y versión mayor.
    ///
    /// La comprobación es simétrica y no depende de la versión menor.
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
    /// Escribe el nombre y la versión mayor separados por una barra.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.name, self.major)
    }
}

/// Referencia a una entidad implicada en un diagnóstico o error.
#[derive(Clone, Debug)]
pub enum EntityRef {
    /// Referencia a un nodo.
    Node(
        /// Identificador del nodo.
        NodeId,
    ),
    /// Referencia a un miembro.
    Member(
        /// Identificador del miembro.
        MemberId,
    ),
    /// Referencia a un grado de libertad global.
    Dof(
        /// Posición global del grado de libertad.
        DofIndex,
    ),
    /// Referencia a un caso de cálculo.
    Case(
        /// Nombre del caso.
        String,
    ),
    /// Referencia a una ley constitutiva.
    Law(
        /// Nombre de la ley.
        String,
    ),
}

/// Nivel de gravedad de un diagnóstico.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Información que no requiere una corrección.
    Info,
    /// Advertencia que requiere revisión, sin impedir necesariamente el cálculo.
    Warning,
    /// Error que impide completar la operación solicitada.
    Error,
}

/// Diagnóstico con código estable, mensaje en español y contexto opcional.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// Código estable de la situación, con formato ENG-E-NNNN en el catálogo.
    pub code: &'static str,
    /// Gravedad de la situación descrita.
    pub severity: Severity,
    /// Mensaje destinado a la persona que utiliza el motor.
    pub message: String,
    /// Entidades implicadas, en el orden aportado por quien produce el diagnóstico.
    pub entities: Vec<EntityRef>,
    /// Sugerencia opcional para resolver o investigar la situación.
    pub hint: Option<String>,
}

impl Diagnostic {
    /// Construye un diagnóstico sin entidades ni sugerencia.
    ///
    /// Permite códigos propios; para códigos del catálogo se recomienda
    /// [`Self::from_code`], que proporciona la gravedad y el mensaje registrados.
    pub fn new(code: &'static str, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            entities: Vec::new(),
            hint: None,
        }
    }

    /// Construye un diagnóstico con los datos del catálogo, o devuelve `None`.
    ///
    /// El diagnóstico comienza sin entidades ni sugerencia.
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

/// Error de una operación del motor, con código estable y entidades implicadas.
#[derive(Clone, Debug)]
pub struct EngineError {
    /// Código estable de la situación.
    pub code: &'static str,
    /// Mensaje en español que describe el fallo.
    pub message: String,
    /// Entidades relacionadas con el fallo.
    pub entities: Vec<EntityRef>,
}

impl EngineError {
    /// Construye un error con mensaje propio y sin entidades relacionadas.
    ///
    /// Para obtener un mensaje registrado se recomienda [`Self::from_code`].
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            entities: Vec::new(),
        }
    }

    /// Construye un error con el código y mensaje del catálogo, o devuelve `None`.
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
    /// Escribe el código y el mensaje separados por dos puntos.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for EngineError {}

/// Resultado de una operación del motor que puede producir un [`EngineError`].
pub type EResult<T> = Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Comprueba los índices y etiquetas de todas las componentes y su orden local.
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

    /// Comprueba la representación de contratos y los límites de compatibilidad.
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

    /// Comprueba que los constructores conservan los valores y el contexto vacío.
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

    /// Comprueba que todos los constructores por código utilizan el catálogo.
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
