//! Contrato results/1 y consultas: fixtures propios, sin oráculos externos.
mod support;
use engine_api::{canonical_json, ApiError, CancelToken, ResultQuery};
use engine_core::{MemberId, NodeId};
use serde_json::{json, Value};
use support::checked_run as run;

const INPUTS: [&str; 5] = [
    include_str!("../fixtures/input/valid/cantilever-linear-static.json"),
    include_str!("../fixtures/input/valid/beam-combination.json"),
    include_str!("../fixtures/input/valid/modal-single-dof.json"),
    include_str!("../fixtures/input/valid/p-delta.json"),
    include_str!("../fixtures/input/valid/response-spectrum.json"),
];

#[test]
fn build_metadata_and_schema_for_every_method() {
    for input in INPUTS {
        let outcome = run(input, &CancelToken::new(), |_| {});
        assert!(matches!(outcome.exit_code(), 0 | 1));
        let value: Value = serde_json::from_str(&outcome.document).unwrap();
        assert_eq!(
            value["engine_version"],
            engine_api::version()["engine_version"]
        );
        assert_eq!(
            value["contracts"],
            json!(["engine-input/1", "results/1", "element/1", "system/1"])
        );
        let input_value: Value = serde_json::from_str(input).unwrap();
        assert_eq!(value["methods"][0], input_value["analyses"][0]["method"]);
        assert!(value["target"].as_str().unwrap().contains('-'));
        if cfg!(target_arch = "wasm32") {
            assert_eq!(value["target"], "wasm32-unknown-unknown");
        }
        assert_eq!(
            value["git_commit"],
            json!(option_env!("MAMUT_ENGINE_BUILD_COMMIT"))
        );
        assert_eq!(
            value["input_sha256"],
            mamut_canon::fingerprint(input).unwrap().value
        );
        assert_eq!(
            value["input_sha256"],
            engine_api::input_sha256(input).unwrap()
        );
        assert_eq!(outcome.document, canonical_json(&outcome.document).unwrap());
    }
}

#[test]
fn fingerprint_regressions_document_jcs_changes() {
    let vectors: Value =
        serde_json::from_str(include_str!("../fixtures/results/fingerprints.json")).unwrap();
    for vector in vectors.as_array().unwrap() {
        let input = vector["input"].as_str().unwrap();
        let outcome = run(input, &CancelToken::new(), |_| {});
        let value: Value = serde_json::from_str(&outcome.document).unwrap();
        if vector["rejected_by_engine"] == true {
            // Vector histórico de JCS: el SDK no cambia; la frontera Engine
            // ahora rechaza el entero antes de publicar una huella ambigua.
            assert_eq!(outcome.exit_code(), 2);
            assert!(value["input_sha256"].is_null());
            assert!(matches!(
                engine_api::input_sha256(input),
                Err(ApiError::InvalidInput(_))
            ));
            assert!(matches!(
                canonical_json(input),
                Err(ApiError::InvalidInput(_))
            ));
            assert_eq!(
                mamut_canon::fingerprint(input).unwrap().value,
                vector["input_sha256"]
            );
            assert_eq!(
                mamut_canon::canonicalize(input).unwrap(),
                vector["canonical"].as_str().unwrap().as_bytes()
            );
            continue;
        }
        assert_eq!(value["input_sha256"], vector["input_sha256"]);
        assert_eq!(
            engine_api::input_sha256(input).unwrap(),
            vector["input_sha256"]
        );
        assert_eq!(
            mamut_canon::canonicalize(input).unwrap(),
            vector["canonical"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            mamut_canon::sha256_hex(canonical_json(input).unwrap().as_bytes()),
            vector["previous_sha256"]
        );
    }
    // Mismas entradas mecánicas con representaciones enteras/decimales.
    let integers: Value = serde_json::from_str(INPUTS[0]).unwrap();
    let mut decimals = integers.clone();
    decimals["model"]["nodes"][1]["coordinates"][0] = json!(4.0);
    assert_eq!(
        run(&integers.to_string(), &CancelToken::new(), |_| {}),
        run(&decimals.to_string(), &CancelToken::new(), |_| {})
    );
}

#[test]
fn input_fingerprint_respects_the_engine_parser_limits() {
    for input in [
        "{",
        r#"{"a":1,"a":2}"#,
        "NaN",
        "1e400",
        "18446744073709551616",
    ] {
        assert!(matches!(
            engine_api::input_sha256(input),
            Err(ApiError::InvalidInput(_))
        ));
    }
    let oversized = " ".repeat(engine_api::MAX_INPUT_BYTES + 1);
    assert!(matches!(
        engine_api::input_sha256(&oversized),
        Err(ApiError::InvalidInput(_))
    ));
}

#[test]
fn queries_filter_nodes_members_everywhere_without_mutating_provenance() {
    for input in INPUTS {
        let outcome = run(input, &CancelToken::new(), |_| {});
        let original = outcome.document.clone();
        let all: Value = serde_json::from_str(&original).unwrap();
        assert_eq!(outcome.query(&ResultQuery::default()).unwrap(), all);
        let query = ResultQuery {
            node_ids: Some(&[NodeId(2)]),
            member_ids: Some(&[]),
            fields: None,
        };
        let filtered = outcome.query(&query).unwrap();
        support::assert_results(&filtered);
        check_entity_arrays(&filtered, "nodes", &[2]);
        check_entity_arrays(&filtered, "shape", &[2]);
        check_entity_arrays(&filtered, "members", &[]);
        assert_eq!(filtered["input_sha256"], all["input_sha256"]);
        assert_eq!(filtered["diagnostics"], all["diagnostics"]);
        assert_eq!(filtered["convergence"], all["convergence"]);
        assert_eq!(outcome.document, original);
        let none = outcome
            .query(&ResultQuery {
                node_ids: Some(&[NodeId(999)]),
                member_ids: Some(&[MemberId(999)]),
                fields: None,
            })
            .unwrap();
        check_entity_arrays(&none, "nodes", &[]);
        check_entity_arrays(&none, "members", &[]);
    }
}

fn check_entity_arrays(value: &Value, field: &str, ids: &[u64]) {
    match value {
        Value::Object(object) => {
            for (name, child) in object {
                if name == field {
                    if let Some(items) = child.as_array() {
                        for item in items {
                            // members[].nodes son referencias geométricas, no filas.
                            if item.is_object() {
                                assert!(ids.contains(&item["id"].as_u64().unwrap()));
                            }
                        }
                    }
                }
                check_entity_arrays(child, field, ids);
            }
        }
        Value::Array(items) => {
            for item in items {
                check_entity_arrays(item, field, ids);
            }
        }
        _ => {}
    }
}

#[test]
fn projection_preserves_ids_and_exact_numeric_fields_in_cases_and_combinations() {
    let outcome = run(INPUTS[1], &CancelToken::new(), |_| {});
    let all: Value = serde_json::from_str(&outcome.document).unwrap();
    let complete = outcome
        .query(&ResultQuery {
            node_ids: Some(&[NodeId(2)]),
            member_ids: Some(&[MemberId(7)]),
            fields: None,
        })
        .unwrap();
    support::assert_results(&complete);
    assert_eq!(complete["members"][0]["nodes"], json!([1, 2]));
    assert_eq!(complete["members"].as_array().unwrap().len(), 1);
    assert_eq!(complete["nodes"].as_array().unwrap().len(), 1);
    let selected = outcome
        .query(&ResultQuery {
            node_ids: Some(&[NodeId(2), NodeId(2)]),
            member_ids: Some(&[MemberId(7)]),
            fields: Some(&[
                "cases.nodes.reactions",
                "cases.members.stations.forces_local",
                "combinations.results.nodes.displacements",
            ]),
        })
        .unwrap();
    assert_eq!(selected.as_object().unwrap().len(), 2);
    assert_eq!(
        selected["cases"][0]["nodes"],
        json!([{"id":2,"reactions":all["cases"][0]["nodes"][1]["reactions"]}])
    );
    assert_eq!(selected["cases"][0]["members"][0]["id"], 7);
    let station = &selected["cases"][0]["members"][0]["stations"][0];
    assert_eq!(station.as_object().unwrap().len(), 1);
    assert_eq!(
        station["forces_local"],
        all["cases"][0]["members"][0]["stations"][0]["forces_local"]
    );
    assert_eq!(
        selected["combinations"][0]["id"],
        all["combinations"][0]["id"]
    );
    assert_eq!(
        selected["combinations"][0]["results"]["nodes"],
        json!([{"id":2,"displacements":all["combinations"][0]["results"]["nodes"][1]["displacements"]}])
    );
    assert_eq!(
        outcome
            .query(&ResultQuery {
                fields: Some(&[]),
                ..Default::default()
            })
            .unwrap(),
        json!({})
    );
    assert_eq!(
        outcome
            .query(&ResultQuery {
                fields: Some(&["absent"]),
                ..Default::default()
            })
            .unwrap(),
        json!({})
    );
}

#[test]
fn projection_of_modal_shape_and_spectrum_nodes() {
    for input in [INPUTS[2], INPUTS[4]] {
        let outcome = run(input, &CancelToken::new(), |_| {});
        let all: Value = serde_json::from_str(&outcome.document).unwrap();
        let selected = outcome
            .query(&ResultQuery {
                node_ids: Some(&[NodeId(2)]),
                member_ids: None,
                fields: Some(&[
                    "modes.shape.displacements",
                    "modes.spectrum_responses.nodes.reactions",
                    "envelopes.nodes.displacements",
                ]),
            })
            .unwrap();
        assert_eq!(
            selected["modes"][0]["shape"],
            json!([{"id":2,"displacements":all["modes"][0]["shape"][1]["displacements"]}])
        );
        if !all["envelopes"].as_array().unwrap().is_empty() {
            assert_eq!(
                selected["envelopes"][0]["nodes"],
                json!([{"id":2,"displacements":all["envelopes"][0]["nodes"][1]["displacements"]}])
            );
            assert_eq!(
                selected["modes"][0]["spectrum_responses"][0]["nodes"],
                json!([{"id":2,"reactions":all["modes"][0]["spectrum_responses"][0]["nodes"][1]["reactions"]}])
            );
        }
    }
}

#[test]
fn queries_reject_malformed_paths_and_work_on_empty_failure_results() {
    let outcome = run("{", &CancelToken::new(), |_| {});
    for path in ["", ".nodes", "nodes.", "cases..nodes", "cases[0]", "Nodes"] {
        assert!(matches!(
            outcome.query(&ResultQuery {
                fields: Some(&[path]),
                ..Default::default()
            }),
            Err(ApiError::InvalidInput(_))
        ));
    }
    let selected = outcome
        .query(&ResultQuery {
            fields: Some(&["nodes.coordinates", "cases.nodes.reactions"]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(selected, json!({"nodes":[],"cases":[]}));
    let mut malformed = outcome;
    malformed.document = "{".into();
    assert!(matches!(
        malformed.query(&ResultQuery::default()),
        Err(ApiError::InvalidInput(_))
    ));
    malformed.document = "{}".into();
    assert!(matches!(
        malformed.query(&ResultQuery::default()),
        Err(ApiError::UnsupportedSchema(_))
    ));
}

#[test]
fn schema_validator_rejects_contract_mutations() {
    let outcome = run("{", &CancelToken::new(), |_| {});
    let original: Value = serde_json::from_str(&outcome.document).unwrap();
    for (field, replacement) in [
        ("schema", serde_json::json!("results/2")),
        ("status", serde_json::json!("unknown")),
        ("input_sha256", serde_json::json!("bad")),
        ("git_commit", serde_json::json!("revision")),
        ("target", serde_json::json!(1)),
        (
            "diagnostics",
            serde_json::json!([{"code":"API-E-bad","severity":"error","entities":[],"cause":"","hint":null}]),
        ),
        (
            "nodes",
            serde_json::json!([{"id":1,"dim":"D2","coordinates":[0,0,1],"dofs":[]}]),
        ),
        (
            "nodes",
            serde_json::json!([{"id":4294967296_u64,"dim":"D3","coordinates":[0,0,0],"dofs":[]}]),
        ),
        (
            "nodes",
            serde_json::json!([{"id":1,"dim":"D3","coordinates":[0,0],"dofs":[]}]),
        ),
        ("convergence", serde_json::json!({})),
        ("unexpected", serde_json::json!(true)),
    ] {
        let mut changed = original.clone();
        changed[field] = replacement;
        assert!(
            !support::conforms(&changed),
            "No detectó mutación de {field}"
        );
    }
    let mut changed = original.clone();
    changed.as_object_mut().unwrap().remove("cases");
    assert!(!support::conforms(&changed));
    // Compatibilidad con documentos #88 sin los metadatos añadidos.
    changed = original;
    changed.as_object_mut().unwrap().remove("git_commit");
    changed.as_object_mut().unwrap().remove("target");
    assert!(support::conforms(&changed));
}
