//! Fixture tipado para una ley todavía no accesible desde engine-input/1.
use engine_contracts::UniaxialLaw;
use engine_core::{EResult, Real};
use engine_laws::uniaxial::Metodo;

struct Fixture {
    source: &'static str,
    citation: &'static str,
    // Filas: ε total, σ en Pa, tangente en Pa. Orden = trayectoria aceptada.
    rows: &'static [(Real, Real, Real)],
    absolute: Real,
    relative: Real,
}

// Completar desde ecuaciones de la ficha ANTES de ejecutar la ley.
// No calcular esperados llamando a respond ni copiando otro motor.
const FIXTURE: Fixture = Fixture {
    source: "analytical",
    citation: "", // Autor, título, fecha, ficha y ecuaciones: obligatorio.
    rows: &[],   // Evaluaciones propias: obligatorio, no dejar vacío.
    absolute: 1e-12, // Pa; justificar ambos límites en la ficha.
    relative: 1e-12,
};

#[test]
fn benchmark_analitico_nativo() -> EResult<()> {
    let f = FIXTURE;
    assert!(!f.source.is_empty() && !f.citation.is_empty() && !f.rows.is_empty());
    let law = Metodo::new(200.0)?; // Sustituir parámetros SI de la ficha.
    let mut committed = law.initial_state();
    for &(strain, stress, tangent) in f.rows {
        let trial = law.respond(strain, &committed)?;
        for (actual, expected) in [(trial.stress, stress), (trial.tangent, tangent)] {
            assert!(actual.is_finite() && expected.is_finite());
            let limit = f.absolute.max(f.relative * expected.abs());
            assert!(
                (actual - expected).abs() <= limit,
                "ε={strain}: {actual} frente a {expected}, límite={limit}; {}: {}",
                f.source, f.citation
            );
        }
        committed = trial.trial_state;
    }
    Ok(())
}
