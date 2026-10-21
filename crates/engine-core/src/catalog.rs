//! Ordered catalog of stable codes and explanations in Spanish.
//!
//! ```
//! use engine_core::catalog::{explain, lookup};
//! assert_eq!(lookup("ENG-E-0001").unwrap().title, "Unidad desconocida");
//! assert!(explain("ENG-E-0001").is_some());
//! assert_eq!(explain("ENG-E-9999"), None);
//! ```

use crate::Severity;

/// Public information about a situation recognized by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeInfo {
    /// Stable identifier in ENG-E-NNNN format.
    pub code: &'static str,
    /// Default diagnostic severity.
    pub severity: Severity,
    /// Short title in Spanish.
    pub title: &'static str,
    /// Default message in Spanish.
    pub message: &'static str,
    /// Explanation in Spanish of the cause and possible review actions.
    pub explanation: &'static str,
}

/// Static list ordered by code, with no duplicate entries.
///
/// Existing identifiers retain their meaning; new entries are added
/// without reusing codes. All initial situations are errors.
pub static CODES: &[CodeInfo] = &[
    CodeInfo {
        code: "ENG-E-0001",
        severity: Severity::Error,
        title: "Unidad desconocida",
        message: "La unidad indicada no está reconocida.",
        explanation: "La entrada utiliza una unidad que el motor no conoce. Revise su nombre y convierta la magnitud a una unidad admitida.",
    },
    CodeInfo {
        code: "ENG-E-0002",
        severity: Severity::Error,
        title: "Identificador duplicado",
        message: "El identificador ya está asignado a otra entidad.",
        explanation: "Dos entidades de la misma clase comparten un identificador. Asigne identificadores únicos y actualice las referencias correspondientes.",
    },
    CodeInfo {
        code: "ENG-E-0003",
        severity: Severity::Error,
        title: "Referencia rota",
        message: "La entidad referenciada no existe.",
        explanation: "Una relación apunta a una entidad ausente del modelo. Revise el identificador y cree la entidad necesaria o corrija la referencia.",
    },
    CodeInfo {
        code: "ENG-E-0004",
        severity: Severity::Error,
        title: "Nodo huérfano",
        message: "El nodo no está conectado a ningún miembro.",
        explanation: "El nodo queda aislado de los miembros del modelo. Revise la conectividad y conecte el nodo o retire la entidad que no participa en el cálculo.",
    },
    CodeInfo {
        code: "ENG-E-0005",
        severity: Severity::Error,
        title: "Longitud nula",
        message: "La longitud del miembro es nula.",
        explanation: "Los extremos del miembro coinciden y no definen una longitud utilizable. Revise las coordenadas y las referencias a sus nodos extremos.",
    },
    CodeInfo {
        code: "ENG-E-0006",
        severity: Severity::Error,
        title: "Matriz singular o mecanismo",
        message: "El sistema presenta una matriz singular o un mecanismo.",
        explanation: "Las ecuaciones no determinan una solución única. Revise los apoyos, la conectividad, las liberaciones y la rigidez para detectar movimientos sin resistencia.",
    },
    CodeInfo {
        code: "ENG-E-0007",
        severity: Severity::Error,
        title: "Falta de convergencia",
        message: "El cálculo no alcanzó la convergencia requerida.",
        explanation: "El procedimiento agotó sus pasos sin satisfacer las tolerancias. Revise los datos, las tolerancias y el tamaño de los incrementos del cálculo.",
    },
    CodeInfo {
        code: "ENG-E-0008",
        severity: Severity::Error,
        title: "Entrada no finita",
        message: "Una magnitud de entrada no es un número finito.",
        explanation: "Se recibió un valor indefinido o infinito. Revise las conversiones, las operaciones previas y los datos originales antes de iniciar el cálculo.",
    },
    CodeInfo {
        code: "ENG-E-0009",
        severity: Severity::Error,
        title: "Contrato incompatible",
        message: "Los contratos tienen nombres o versiones mayores incompatibles.",
        explanation: "Las partes requieren contratos con distinto nombre o distinta versión mayor. Utilice componentes que compartan ambos valores; la versión menor no afecta esta comprobación.",
    },
    CodeInfo {
        code: "ENG-E-0010",
        severity: Severity::Error,
        title: "Dimensión inválida",
        message: "La dimensión indicada no es válida para la operación.",
        explanation: "La operación recibió una dimensión no admitida o incompatible con el modelo. Revise si el modelo es plano o espacial y adapte los datos a esa dimensión.",
    },
    CodeInfo {
        code: "ENG-E-0011",
        severity: Severity::Error,
        title: "Grado de libertad inválido",
        message: "El grado de libertad no está disponible en el contexto indicado.",
        explanation: "La componente o su posición global no pertenece al conjunto disponible. Revise la dimensión del modelo y la numeración de los grados de libertad.",
    },
    CodeInfo {
        code: "ENG-E-0012",
        severity: Severity::Error,
        title: "Error interno",
        message: "El motor encontró un estado interno inesperado.",
        explanation: "Una condición interna impide completar la operación. Conserve los datos de entrada y el código del diagnóstico para reproducir la situación y revisar su causa.",
    },
    CodeInfo {
        code: "ENG-E-0013",
        severity: Severity::Error,
        title: "Capacidad no soportada",
        message: "La operación solicitada no está soportada por este componente.",
        explanation: "La entrada es válida, pero el componente no implementa la capacidad pedida (por ejemplo, evaluar la rigidez geométrica en un estado distinto del de referencia). Use un componente que la soporte o limite la solicitud a lo admitido; no reintente con los mismos datos.",
    },
    CodeInfo {
        code: "ENG-E-0014",
        severity: Severity::Error,
        title: "Mecanismo sin masa",
        message: "La rigidez de los grados de libertad sin masa no es definida positiva.",
        explanation: "La condensación de Guyan requiere Kss definida positiva. Un pivote singular o una dirección de rigidez negativa impide recuperar los GDL sin masa. Revise los apoyos, la conectividad y las liberaciones del nodo/GDL indicado; no se añade masa ni rigidez artificial.",
    },
];

/// Looks up an exact code and returns its static information, or `None`.
///
/// The lookup is case-sensitive and does not modify the input.
pub fn lookup(code: &str) -> Option<&'static CodeInfo> {
    CODES
        .binary_search_by_key(&code, |info| info.code)
        .ok()
        .map(|index| &CODES[index])
}

/// Returns the explanation in Spanish for a known code, or `None`.
///
/// ```
/// use engine_core::catalog::explain;
/// assert!(!explain("ENG-E-0006").unwrap().is_empty());
/// assert_eq!(explain("desconocido"), None);
/// ```
pub fn explain(code: &str) -> Option<&'static str> {
    lookup(code).map(|info| info.explanation)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks the format, strict ordering, and absence of repeated codes.
    #[test]
    fn unique_ordered_codes_with_valid_format() {
        // Historical minimum; adding new codes must not break the test.
        assert!(CODES.len() >= 13);
        for info in CODES {
            let number = info.code.strip_prefix("ENG-E-").unwrap();
            assert_eq!(number.len(), 4);
            assert!(number.bytes().all(|byte| byte.is_ascii_digit()));
        }
        assert!(CODES.windows(2).all(|pair| pair[0].code < pair[1].code));
    }

    /// Pins the stable meaning of ENG-E-0013 (do not reuse or renumber).
    #[test]
    fn unsupported_capability_code_is_stable() {
        let info = lookup("ENG-E-0013").unwrap();
        assert_eq!(info.title, "Capacidad no soportada");
        assert_eq!(info.severity, Severity::Error);
        // Ordering and uniqueness are required by `unique_ordered_codes_with_valid_format`; the
        // "last" code is not pinned: adding ENG-E-0014 must not break this test.
        assert!(CODES.iter().any(|c| c.code == "ENG-E-0013"));
    }

    /// Checks that texts are nonempty and lookup preserves the registered data.
    #[test]
    fn complete_catalog_text_and_lookup() {
        for info in CODES {
            assert!(!info.title.trim().is_empty());
            assert!(!info.message.trim().is_empty());
            assert!(!info.explanation.trim().is_empty());
            assert_eq!(lookup(info.code), Some(info));
            assert_eq!(explain(info.code), Some(info.explanation));
        }
    }

    /// Checks that missing or misspelled codes have no explanation.
    #[test]
    fn unknown_code_has_no_explanation() {
        for code in ["ENG-E-0000", "ENG-E-9999", "eng-e-0001", "", "ENG-E-0001 "] {
            assert_eq!(lookup(code), None);
            assert_eq!(explain(code), None);
        }
    }
}
