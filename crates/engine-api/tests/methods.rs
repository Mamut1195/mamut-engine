//! SDK solve contracts, job policy, and discovery evidence.
use engine_api::{
    capabilities, explain, method_descriptor, method_descriptors, run, schema, CancelToken,
    MethodStatus, Status,
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
    assert_eq!(descriptors.len(), 9);
    let ids: Vec<_> = descriptors
        .iter()
        .map(|d| d.workflow.method_id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "buckling.linear",
            "combinations",
            "consolidation1d",
            "linearStatic",
            "modal",
            "nonlinearStatic",
            "pDelta",
            "responseSpectrum",
            "soilProfile"
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
    assert!(a["laws"]
        .as_array()
        .expect("laws")
        .iter()
        .any(|law| law == "concrete.kent_park"));
    // Historical camelCase method keys are preserved; every new extension
    // uses the approved naming grammar, without expanding LEGACY_DEFS.
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

#[path = "support/sfc010.rs"]
mod sfc010;

#[test]
fn sfc010_experimental_methods_require_explicit_job_opt_in() {
    let capabilities = capabilities();
    for (id, expected) in [
        ("buckling.linear", MethodStatus::Experimental),
        ("combinations", MethodStatus::Validated),
        ("consolidation1d", MethodStatus::Experimental),
        ("linearStatic", MethodStatus::Validated),
        ("modal", MethodStatus::Validated),
        ("nonlinearStatic", MethodStatus::Experimental),
        ("pDelta", MethodStatus::Stable),
        ("responseSpectrum", MethodStatus::Validated),
        ("soilProfile", MethodStatus::Experimental),
    ] {
        let descriptor = method_descriptors()
            .unwrap()
            .into_iter()
            .find(|d| d.workflow.method_id == id)
            .unwrap();
        assert_eq!(descriptor.status, expected, "{id}");
        let published = capabilities["method_descriptors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["workflow"]["method_id"] == id)
            .unwrap();
        assert_eq!(published["status"], json!(expected), "{id}");
    }
    for &(name, source) in sfc010::FIXTURES {
        let mut input: Value = serde_json::from_str(source).unwrap();
        let id = input["analyses"][0]["method"].as_str().unwrap().to_owned();
        if !matches!(
            id.as_str(),
            "nonlinearStatic" | "soilProfile" | "consolidation1d"
        ) {
            continue;
        }
        assert_eq!(input["options"]["allow_experimental"], true, "{name}");
        assert_eq!(
            method_descriptor(&id, INPUT, true).unwrap().status,
            MethodStatus::Experimental
        );
        let error = method_descriptor(&id, INPUT, false)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("options.allow_experimental = true"),
            "{error}"
        );
        for options in [json!({}), json!({"allow_experimental": false})] {
            input["options"] = options;
            let outcome = run(&input.to_string(), &CancelToken::new(), |_| {});
            assert_eq!(
                outcome.status,
                Status::Invalid,
                "{name}: {}",
                outcome.document
            );
            assert_eq!(outcome.exit_code(), 2);
            let document: Value = serde_json::from_str(&outcome.document).unwrap();
            assert_eq!(document["diagnostics"][0]["code"], "API-E-INPUT");
            assert!(outcome.document.contains(&id));
            assert!(outcome.document.contains("Experimental"));
            assert!(outcome
                .document
                .contains("options.allow_experimental = true"));
            assert_eq!(document["cases"], json!([]));
            assert!(document.get("geotechnical").is_none());
            assert!(document["convergence"].is_null());
        }
    }
    for code in ["API-E-SFC-OPTIONS", "API-E-INPUT"] {
        let explanation = explain(code).unwrap();
        for expected in [
            "nonlinearStatic",
            "soilProfile",
            "consolidation1d",
            "Experimental",
            "ENG-VAL-006",
            "options.allow_experimental = true",
        ] {
            assert!(explanation.contains(expected), "{code}: {explanation}");
        }
    }
}
