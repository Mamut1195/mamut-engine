//! Validador de pruebas del subconjunto Draft 2020-12 usado por results/1.
//! Sin E/S, resolución remota ni dependencias nuevas. Falla ante keywords o
//! expresiones regulares desconocidas; no pretende ser un validador general.
use engine_api::{CancelToken, Outcome, Progress};
use serde_json::Value;

pub fn checked_run(input: &str, cancel: &CancelToken, progress: impl FnMut(Progress)) -> Outcome {
    let outcome = engine_api::run(input, cancel, progress);
    let value: Value = serde_json::from_str(&outcome.document).unwrap();
    assert_results(&value);
    outcome
}

pub fn assert_results(value: &Value) {
    assert!(conforms(value), "results/1 fuera de schema: {value}");
}

mod schema;

pub fn conforms(value: &Value) -> bool {
    let root: Value = serde_json::from_str(engine_api::schema("results/1").unwrap()).unwrap();
    schema::conforms(value, &root)
}
