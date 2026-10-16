//! Contratos de resolución SDK, política del job y evidencia de descubrimiento.
use engine_api::{
    capabilities, method_descriptor, method_descriptors, run, schema, CancelToken, MethodStatus,
    Status,
};
use serde_json::{json, Value};
use std::path::Path;

const INPUT: &str = "urn:mamut:engine:engine-input:1.0.0";
const RESULTS: &str = "urn:mamut:engine:results:1.0.0";
const PDELTA: &str = include_str!("../fixtures/input/valid/p-delta.json");

#[test]
fn dispatch_resolves_id_and_contract_with_sdk_major_compatibility() {
    for id in ["linearStatic", "pDelta", "modal", "responseSpectrum"] {
        let d = method_descriptor(id, INPUT, true).unwrap();
        assert_eq!(d.workflow.method_id, id);
        assert_eq!(d.workflow.method_version, 1);
        assert_eq!(d.workflow.consumes[0].urn(), INPUT);
        assert_eq!(d.workflow.produces[0].urn(), RESULTS);
        assert_eq!(
            method_descriptor(id, "urn:mamut:engine:engine-input:1.7.3", true).unwrap(),
            d
        );
        for wrong in [
            RESULTS,
            "urn:mamut:engine:engine-input:2.0.0",
            "urn:mamut:other:engine-input:1.0.0",
            "engine-input/1",
        ] {
            assert!(method_descriptor(id, wrong, true).is_err(), "{id}: {wrong}");
        }
    }
    let d = method_descriptor("combinations", RESULTS, false).unwrap();
    assert_eq!(d.workflow.consumes[0].urn(), RESULTS);
    assert!(method_descriptor("combinations", INPUT, false).is_err());
    let error = method_descriptor("unknown", INPUT, true).unwrap_err();
    assert!(error.to_string().contains("unknown"));
}

#[test]
fn pdelta_is_stable_and_runs_without_experimental_opt_in() {
    for allow_experimental in [false, true] {
        assert_eq!(
            method_descriptor("pDelta", INPUT, allow_experimental)
                .unwrap()
                .status,
            MethodStatus::Stable
        );
    }
    for options in [
        json!({}),
        json!({"allow_experimental": false}),
        json!({"allow_experimental": true}),
    ] {
        let mut input: Value = serde_json::from_str(PDELTA).unwrap();
        input["options"] = options;
        let mut progress = Vec::new();
        let o = run(&input.to_string(), &CancelToken::new(), |p| {
            progress.push(p)
        });
        assert_eq!(o.status, Status::Ok, "{}", o.document);
        assert_eq!(o.exit_code(), 0);
        assert!(!progress.is_empty());
        let d: Value = serde_json::from_str(&o.document).unwrap();
        assert_eq!(d["convergence"]["method"], "pDelta");
        assert!(!d["cases"].as_array().unwrap().is_empty());
    }
    let allowed = run(PDELTA, &CancelToken::new(), |_| {});
    assert_eq!(allowed.status, Status::Ok, "{}", allowed.document);
    let d: Value = serde_json::from_str(&allowed.document).unwrap();
    assert_eq!(d["convergence"]["method"], "pDelta");
    assert!(!d["cases"].as_array().unwrap().is_empty());
}

fn ascii_keys(value: &Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let mut bytes = key.bytes();
                assert!(bytes.next().is_some_and(|b| b.is_ascii_lowercase()));
                assert!(bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'));
                ascii_keys(value);
            }
        }
        Value::Array(values) => values.iter().for_each(ascii_keys),
        _ => {}
    }
}

#[test]
fn capabilities_are_deterministic_compatible_and_backed_by_evidence() {
    let a = capabilities();
    let descriptors = method_descriptors().unwrap();
    assert_eq!(descriptors.len(), 5);
    let ids: Vec<_> = descriptors
        .iter()
        .map(|d| d.workflow.method_id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "combinations",
            "linearStatic",
            "modal",
            "pDelta",
            "responseSpectrum"
        ]
    );
    assert_eq!(a["method_descriptors"], json!(descriptors));
    let pdelta = descriptors
        .iter()
        .find(|d| d.workflow.method_id == "pDelta")
        .unwrap();
    assert_eq!(pdelta.status, MethodStatus::Stable);
    assert!(pdelta
        .citations
        .iter()
        .any(|citation| citation.contains("ENG-BENCH-002 (PR #132)")));
    let bytes = serde_json::to_vec(&a).unwrap();
    for _ in 0..8 {
        assert_eq!(serde_json::to_vec(&capabilities()).unwrap(), bytes);
    }
    for id in ["linearStatic", "modal", "pDelta", "responseSpectrum"] {
        assert_eq!(a["methods"][id], json!({"engine": true, "run": true}));
    }
    assert_eq!(a["combinations"], json!({"run": true}));
    // Las claves históricas camelCase de methods se conservan; toda extensión
    // nueva usa la gramática de nombres aprobada, sin ampliar LEGACY_DEFS.
    ascii_keys(&a["method_descriptors"]);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for d in descriptors {
        assert_eq!(d.schema, "method-descriptor/1");
        assert!(!d.fixtures.is_empty());
        assert!(!d.citations.is_empty());
        assert!(!d.provenance.is_empty());
        for path in d.fixtures.iter().chain(d.provenance.iter()) {
            assert!(root.join(path).is_file(), "{path}");
        }
    }
    let input_schema: Value = serde_json::from_str(schema("engine-input/1").unwrap()).unwrap();
    assert_eq!(
        input_schema["properties"]["options"]["properties"]["allow_experimental"]["type"],
        "boolean"
    );
    for name in ["method-descriptor/1", "capabilities/1"] {
        let document: Value = serde_json::from_str(schema(name).unwrap()).unwrap();
        assert_eq!(document["additionalProperties"], false);
    }
    let descriptor_schema: Value =
        serde_json::from_str(schema("method-descriptor/1").unwrap()).unwrap();
    assert!(descriptor_schema["properties"]["status"]["enum"]
        .as_array()
        .unwrap()
        .contains(&json!("Stable")));
}
