//! Ordered catalog of stable codes and explanations in Spanish.
//!
//! ```
//! use engine_core::catalog::{explain, lookup};
//! assert_eq!(lookup("ENG-E-0001").unwrap().title, "Unknown unit");
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
        title: "Unknown unit",
        message: "The specified unit is not recognized.",
        explanation: "The input uses a unit the engine does not recognize. Check its name and convert the quantity to a supported unit.",
    },
    CodeInfo {
        code: "ENG-E-0002",
        severity: Severity::Error,
        title: "Duplicate identifier",
        message: "The identifier is already assigned to another entity.",
        explanation: "Two entities of the same class share an identifier. Assign unique identifiers and update the corresponding references.",
    },
    CodeInfo {
        code: "ENG-E-0003",
        severity: Severity::Error,
        title: "Broken reference",
        message: "The referenced entity does not exist.",
        explanation: "A relationship points to an entity missing from the model. Check the identifier and create the required entity or correct the reference.",
    },
    CodeInfo {
        code: "ENG-E-0004",
        severity: Severity::Error,
        title: "Orphan node",
        message: "The node is not connected to any member.",
        explanation: "The node is isolated from the model members. Check connectivity and connect the node or remove the entity that does not participate in the calculation.",
    },
    CodeInfo {
        code: "ENG-E-0005",
        severity: Severity::Error,
        title: "Zero length",
        message: "The member length is zero.",
        explanation: "The member ends coincide and do not define a usable length. Check the coordinates and references to its end nodes.",
    },
    CodeInfo {
        code: "ENG-E-0006",
        severity: Severity::Error,
        title: "Singular matrix or mechanism",
        message: "The system has a singular matrix or a mechanism.",
        explanation: "The equations do not determine a unique solution. Check supports, connectivity, releases, and stiffness for unresisted movement.",
    },
    CodeInfo {
        code: "ENG-E-0007",
        severity: Severity::Error,
        title: "Failure to converge",
        message: "The calculation did not reach the required convergence.",
        explanation: "The procedure exhausted its steps without meeting the tolerances. Check the data, tolerances, and calculation increment sizes.",
    },
    CodeInfo {
        code: "ENG-E-0008",
        severity: Severity::Error,
        title: "Non-finite input",
        message: "An input quantity is not a finite number.",
        explanation: "An undefined or infinite value was received. Check conversions, prior operations, and original data before starting the calculation.",
    },
    CodeInfo {
        code: "ENG-E-0009",
        severity: Severity::Error,
        title: "Incompatible contract",
        message: "The contracts have incompatible names or major versions.",
        explanation: "The parties require contracts with different names or major versions. Use components that share both values; the minor version does not affect this check.",
    },
    CodeInfo {
        code: "ENG-E-0010",
        severity: Severity::Error,
        title: "Invalid dimension",
        message: "The specified dimension is not valid for the operation.",
        explanation: "The operation received an unsupported dimension or one incompatible with the model. Check whether the model is planar or spatial and adapt the data to that dimension.",
    },
    CodeInfo {
        code: "ENG-E-0011",
        severity: Severity::Error,
        title: "Invalid degree of freedom",
        message: "The degree of freedom is not available in the specified context.",
        explanation: "The component or its global position is outside the available set. Check the model dimension and degree of freedom numbering.",
    },
    CodeInfo {
        code: "ENG-E-0012",
        severity: Severity::Error,
        title: "Internal error",
        message: "The engine encountered an unexpected internal state.",
        explanation: "An internal condition prevents completion of the operation. Keep the input data and diagnostic code to reproduce the situation and investigate its cause.",
    },
    CodeInfo {
        code: "ENG-E-0013",
        severity: Severity::Error,
        title: "Unsupported capability",
        message: "The requested operation is not supported by this component.",
        explanation: "The input is valid, but the component does not implement the requested capability (for example, evaluating geometric stiffness in a state other than the reference state). Use a component that supports it or limit the request to supported operations; do not retry with the same data.",
    },
    CodeInfo {
        code: "ENG-E-0014",
        severity: Severity::Error,
        title: "Massless mechanism",
        message: "The stiffness of the massless degrees of freedom is not positive definite.",
        explanation: "Guyan condensation requires positive definite Kss. A singular pivot or a negative stiffness direction prevents recovery of the massless DOFs. Check supports, connectivity, and releases at the specified node/DOF; no artificial mass or stiffness is added.",
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
        assert_eq!(info.title, "Unsupported capability");
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
