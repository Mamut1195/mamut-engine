//! Catálogo ordenado de códigos estables y explicaciones en español.
//!
//! ```
//! use engine_core::catalog::{explain, lookup};
//! assert_eq!(lookup("ENG-E-0001").unwrap().title, "Unidad desconocida");
//! assert!(explain("ENG-E-0001").is_some());
//! assert_eq!(explain("ENG-E-9999"), None);
//! ```

use crate::Severity;

/// Información pública de una situación reconocida por el motor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeInfo {
    /// Identificador estable con formato ENG-E-NNNN.
    pub code: &'static str,
    /// Gravedad predeterminada del diagnóstico.
    pub severity: Severity,
    /// Título breve en español.
    pub title: &'static str,
    /// Mensaje predeterminado en español.
    pub message: &'static str,
    /// Explicación en español de la causa y las posibles acciones de revisión.
    pub explanation: &'static str,
}

/// Lista estática ordenada por código, sin entradas duplicadas.
///
/// Los identificadores existentes conservan su significado; se añaden entradas
/// nuevas sin reutilizar códigos. Todas las situaciones iniciales son errores.
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
];

/// Busca un código exacto y devuelve su información estática, o `None`.
///
/// La búsqueda distingue mayúsculas y minúsculas y no modifica la entrada.
pub fn lookup(code: &str) -> Option<&'static CodeInfo> {
    CODES
        .binary_search_by_key(&code, |info| info.code)
        .ok()
        .map(|index| &CODES[index])
}

/// Devuelve la explicación en español de un código conocido, o `None`.
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

    /// Comprueba el formato, el orden estricto y la ausencia de códigos repetidos.
    #[test]
    fn unique_ordered_codes_with_valid_format() {
        assert_eq!(CODES.len(), 12);
        for info in CODES {
            let number = info.code.strip_prefix("ENG-E-").unwrap();
            assert_eq!(number.len(), 4);
            assert!(number.bytes().all(|byte| byte.is_ascii_digit()));
        }
        assert!(CODES.windows(2).all(|pair| pair[0].code < pair[1].code));
    }

    /// Comprueba textos no vacíos y que la búsqueda conserve los datos registrados.
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

    /// Comprueba que códigos ausentes o mal escritos no tengan una explicación.
    #[test]
    fn unknown_code_has_no_explanation() {
        for code in ["ENG-E-0000", "ENG-E-9999", "eng-e-0001", "", "ENG-E-0001 "] {
            assert_eq!(lookup(code), None);
            assert_eq!(explain(code), None);
        }
    }
}
