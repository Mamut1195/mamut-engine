//! Conformidad fixture/1 y oráculos propios; sin salidas de otros motores.
use engine_validation::fixtures::{self, Fixture, NativeTarget, Target, REPOSITORY_FIXTURES};
use serde_json::{json, Value};
use std::{cell::Cell, fs, path::Path};

fn base() -> Value {
    serde_json::from_str(REPOSITORY_FIXTURES[0].1).unwrap()
}

struct Stub<'a> {
    name: &'a str,
    output: Result<String, String>,
    calls: Cell<usize>,
}
impl Target for Stub<'_> {
    fn name(&self) -> &str {
        self.name
    }
    fn execute(&self, _: &str) -> Result<String, String> {
        self.calls.set(self.calls.get() + 1);
        self.output.clone()
    }
}
fn stub(output: Value) -> Stub<'static> {
    Stub {
        name: "stub",
        output: Ok(output.to_string()),
        calls: Cell::new(0),
    }
}

#[test]
fn native_runner_passes_every_repository_fixture() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let mut paths: Vec<_> = fs::read_dir(directory)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 5, "Se requieren exactamente 5 fixtures JSON");
    let documents: Vec<_> = paths
        .iter()
        .map(|p| {
            (
                p.file_stem().unwrap().to_str().unwrap().to_owned(),
                fs::read_to_string(p).unwrap(),
            )
        })
        .collect();
    // El inventario embebido tampoco puede omitir ni congelar un archivo cambiado.
    let embedded: Vec<_> = REPOSITORY_FIXTURES
        .iter()
        .map(|(n, d)| (n.to_string(), d.to_string()))
        .collect();
    assert_eq!(documents, embedded);
    let documents: Vec<_> = documents
        .iter()
        .map(|(n, d)| (n.as_str(), d.as_str()))
        .collect();
    let report = fixtures::run(&documents, &[&NativeTarget]);
    assert!(report.passed(), "{}", report.text());
    assert_eq!(report.rows.len(), paths.len());
    assert!(report.rows.iter().all(|r| r.comparisons.len() >= 4));
}

#[test]
fn missing_empty_and_incomplete_source_fail_before_execution() {
    let target = stub(json!({}));
    for source in [
        None,
        Some(json!(null)),
        Some(json!("")),
        Some(json!("  ")),
        Some(json!({})),
        Some(json!({"kind":"analytical","citation":" ","derivation":"propia"})),
        Some(json!({"kind":"analytical","citation":"propia","derivation":"\n\t"})),
        Some(
            json!({"kind":"book","citation":"cita","author":"autor","title":"título","edition":"1"}),
        ),
        Some(
            json!({"kind":"published","citation":"cita","author":"autor","title":"título","publication":"revista","locator":" "}),
        ),
    ] {
        let mut value = base();
        value.as_object_mut().unwrap().remove("source");
        if let Some(source) = source {
            value["source"] = source;
        }
        let text = value.to_string();
        let report = fixtures::run(&[("invalid", &text)], &[&target]);
        assert!(!report.passed(), "{text}");
        assert!(
            report.rows[0].error.as_ref().unwrap().contains("source"),
            "{}",
            report.text()
        );
        assert!(serde_json::from_str::<Value>(&report.json().unwrap()).is_ok());
    }
    assert_eq!(target.calls.get(), 0);
}

#[test]
fn forbidden_source_or_nested_provenance_never_reaches_any_target() {
    let target = stub(json!({}));
    for name in [
        "OpenSees", "Stabileo", "CalculiX", "oPeNsEeS", "STABILEO", "calculix",
    ] {
        for field in ["citation", "derivation", "provenance", "key"] {
            let mut value = base();
            match field {
                "provenance" => {
                    value["provenance"] =
                        json!({"history":[{"origin":format!("derivado de {name}")}]})
                }
                "key" => value["provenance"] = json!({name: "resultado"}),
                _ => value["source"][field] = json!(format!("Datos: {name}")),
            }
            let text = value.to_string();
            let report = fixtures::run(&[("invalid", &text)], &[&target]);
            assert!(!report.passed(), "{text}");
            assert!(report.rows[0].error.as_ref().unwrap().contains("prohibida"));
        }
    }
    // Tampoco se admite una cadena de procedencia contaminada en la cita de libro.
    let mut value = base();
    value["source"] = json!({"kind":"book","citation":"cita OpenSees","author":"a","title":"t","edition":"1","page":"2"});
    assert!(Fixture::parse(&value.to_string()).is_err());
    assert_eq!(target.calls.get(), 0);
}

#[test]
fn defaults_and_justified_overrides_use_relative_error_with_absolute_floor() {
    let mut value = base();
    let linear = Fixture::parse(&value.to_string()).unwrap().tolerance();
    assert_eq!((linear.relative, linear.absolute), (1e-10, 1e-10));
    value["input"]["analyses"][0]["method"] = json!("pDelta");
    let iterative = Fixture::parse(&value.to_string()).unwrap().tolerance();
    assert_eq!((iterative.relative, iterative.absolute), (1e-7, 1e-7));
    value["tolerance"] = json!({"relative":1e-3,"absolute":1e-6,"justification":"Cota artificial para probar el comparador."});
    value["expected"] = json!([
        {"pointer":"/zero","value":0.0}, {"pointer":"/large","value":100.0},
        {"pointer":"/negative","value":-100.0}, {"pointer":"/tiny","value":1e-15}]);
    let target = stub(
        json!({"schema":"results/1","status":"ok","zero":1e-6,"large":100.05,"negative":-100.05,"tiny":-0.5e-6}),
    );
    let text = value.to_string();
    let report = fixtures::run(&[("bounds", &text)], &[&target]);
    assert!(report.passed(), "{}", report.text());
    assert_eq!(report.rows[0].comparisons[0].allowed_error, 1e-6);
    assert_eq!(report.rows[0].comparisons[1].allowed_error, 0.1);
    assert!(report.rows[0].tolerance_justification.is_some());
    let outside = stub(
        json!({"schema":"results/1","status":"ok","zero":1.01e-6,"large":100.11,"negative":-100.11,"tiny":1.01e-6}),
    );
    let report = fixtures::run(&[("bounds", &text)], &[&outside]);
    assert!(!report.passed());
    assert!(report.rows[0].comparisons.iter().all(|c| !c.passed));
}

#[test]
fn invalid_tolerances_versions_expected_and_duplicate_keys_fail() {
    for tolerance in [
        json!({"relative":-1,"absolute":1e-10,"justification":"x"}),
        json!({"relative":1e-10,"absolute":0,"justification":"x"}),
        json!({"relative":1e-10,"absolute":-1,"justification":"x"}),
        json!({"relative":1e-10,"absolute":1e-10,"justification":" "}),
        json!({"relative":1e-10,"absolute":1e-10}),
    ] {
        let mut value = base();
        value["tolerance"] = tolerance;
        assert!(Fixture::parse(&value.to_string()).is_err());
    }
    for (pointer, replacement) in [
        ("/schema", json!("fixture/2")),
        ("/input/schema", json!("engine-input/2")),
        ("/id", json!(" ")),
        ("/expected", json!([])),
        ("/expected/0/pointer", json!("no-slash")),
        ("/expected/0/pointer", json!("/bad~2escape")),
        ("/expected/0/pointer", json!("/bad~")),
    ] {
        let mut value = base();
        *value.pointer_mut(pointer).unwrap() = replacement;
        assert!(Fixture::parse(&value.to_string()).is_err(), "{pointer}");
    }
    let mut value = base();
    let duplicate = value["expected"][0].clone();
    value["expected"].as_array_mut().unwrap().push(duplicate);
    assert!(Fixture::parse(&value.to_string()).is_err());
    let text = REPOSITORY_FIXTURES[0].1;
    assert!(Fixture::parse(&text.replacen("{", "{\"schema\":\"fixture/1\",", 1)).is_err());
    assert!(Fixture::parse(&text.replace("0.12", "1e999")).is_err());
    let mut value = base();
    value["unknown"] = json!(true);
    assert!(Fixture::parse(&value.to_string()).is_err());
}

#[test]
fn reports_continue_after_failures_and_are_deterministic_for_each_target() {
    let failure = Stub {
        name: "a-failure",
        output: Err("fallo controlado".into()),
        calls: Cell::new(0),
    };
    let mut reversed = REPOSITORY_FIXTURES.to_vec();
    reversed.reverse();
    let first = fixtures::run(REPOSITORY_FIXTURES, &[&NativeTarget, &failure]);
    let second = fixtures::run(&reversed, &[&failure, &NativeTarget]);
    assert!(!first.passed());
    assert_eq!(first.rows.len(), 2 * REPOSITORY_FIXTURES.len());
    assert_eq!(failure.calls.get(), 2 * REPOSITORY_FIXTURES.len());
    assert_eq!(first.json().unwrap(), second.json().unwrap());
    assert_eq!(first.text(), second.text());
    for pair in first.rows.chunks_exact(2) {
        assert!(!pair[0].passed);
        assert!(pair[1].passed);
        assert_eq!(pair[0].fixture, pair[1].fixture);
    }
    assert!(first.text().contains("FALLO"));
    assert!(first.text().contains("esperado="));
}

#[test]
fn absent_nonnumeric_mismatched_and_nonfinite_results_fail() {
    let mut value = base();
    value["expected"] = json!([{"pointer":"/answer","value":1.0}]);
    let text = value.to_string();
    for output in [
        json!({"schema":"results/1","status":"ok"}),
        json!({"schema":"results/1","status":"ok","answer":"1.0"}),
        json!({"schema":"results/1","status":"ok","answer":2.0}),
        json!({"schema":"results/1","status":"warning","answer":1.0}),
        json!({"schema":"results/2","status":"ok","answer":1.0}),
    ] {
        let target = stub(output);
        let report = fixtures::run(&[("bad-output", &text)], &[&target]);
        assert!(!report.passed());
        assert!(report.json().is_ok());
    }
    for output in [
        "invalid JSON",
        "{\"schema\":\"results/1\",\"status\":\"ok\",\"answer\":1e999}",
    ] {
        let target = Stub {
            name: "invalid",
            output: Ok(output.into()),
            calls: Cell::new(0),
        };
        assert!(!fixtures::run(&[("bad-output", &text)], &[&target]).passed());
    }
}

#[test]
fn schema_and_all_three_source_variants_have_required_citations() {
    let schema: Value = serde_json::from_str(fixtures::SCHEMA).unwrap();
    assert_eq!(schema["properties"]["schema"]["const"], "fixture/1");
    assert!(schema["required"]
        .as_array()
        .unwrap()
        .contains(&json!("source")));
    for source in [
        json!({"kind":"analytical","citation":"Autor, Nota propia, 2026, §1.","derivation":"EA du/dx=P; u(0)=0, u(L)=PL/EA."}),
        json!({"kind":"book","citation":"Autor, Título, 2ª edición, 2020, p. 34.","author":"Autor","title":"Título","edition":"2ª, 2020","page":"34"}),
        json!({"kind":"published","citation":"Autor, Ejemplo, Revista 3 (2020), p. 4, ejemplo 2.","author":"Autor","title":"Ejemplo","publication":"Revista 3 (2020)","locator":"p. 4, ejemplo 2"}),
    ] {
        let mut value = base();
        value["source"] = source;
        assert!(Fixture::parse(&value.to_string()).is_ok());
    }
}

#[test]
fn empty_runs_and_duplicate_identities_cannot_pass() {
    assert!(!fixtures::run(&[], &[&NativeTarget]).passed());
    assert!(!fixtures::run(REPOSITORY_FIXTURES, &[]).passed());
    assert!(!fixtures::run(
        &[REPOSITORY_FIXTURES[0], REPOSITORY_FIXTURES[0]],
        &[&NativeTarget]
    )
    .passed());
    assert!(!fixtures::run(REPOSITORY_FIXTURES, &[&NativeTarget, &NativeTarget]).passed());
}
