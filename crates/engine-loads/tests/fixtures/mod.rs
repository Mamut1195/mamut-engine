//! In-house fixtures; equations and fractions derived in the ticket log.
pub const SOURCE: &str =
    "analítico: interpolación axial lineal y Hermite cúbica; trabajo virtual y equilibrio";
pub const CITATION: &str =
    "provenance/ENG-LOAD-001.md, Derivaciones propias (1)-(7); sin fuentes externas";

pub fn close(actual: f64, expected: f64) {
    assert!(!SOURCE.is_empty() && !CITATION.is_empty());
    let tol = 2e-12 * expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= tol,
        "{actual} != {expected}, tolerance {tol}"
    );
}
