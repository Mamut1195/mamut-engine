//! Typed fixture for a law not yet accessible through engine-input/1.
use engine_contracts::UniaxialLaw;
use engine_core::{EResult, Real};
use engine_laws::uniaxial::Metodo;

struct Fixture {
    source: &'static str,
    citation: &'static str,
    // Rows: total ε, σ in Pa, tangent in Pa. Order = accepted path.
    rows: &'static [(Real, Real, Real)],
    absolute: Real,
    relative: Real,
}

// Complete from method sheet equations BEFORE executing the law.
// Do not compute expected values by calling respond or copying another engine.
const FIXTURE: Fixture = Fixture {
    source: "analytical",
    citation: "", // Author, title, date, method sheet and equations: required.
    rows: &[],   // Own evaluations: required, do not leave empty.
    absolute: 1e-12, // Pa; justify both limits in the method sheet.
    relative: 1e-12,
};

#[test]
fn benchmark_analitico_nativo() -> EResult<()> {
    let f = FIXTURE;
    assert!(!f.source.is_empty() && !f.citation.is_empty() && !f.rows.is_empty());
    let law = Metodo::new(200.0)?; // Substitute SI parameters from the method sheet.
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
