use engine_api::{
    canonical_json, capabilities, explain, schema, validate, validate_with, version, ApiError,
    CancelToken,
};
use serde_json::{json, Value};

fn input() -> Value {
    json!({
        "schema": "engine-input/1", "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]},
                {"id": 2, "dim": "D2", "coordinates": [2.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]}
            ],
            "members": [{"id": 7, "nodes": [1, 2], "section": "s", "law": "elastic"}],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 0.01}]}],
            "laws": [{"name": "elastic", "parameters": [{"name": "E", "value": 2e11}]}],
            "supports": [{"id": "base", "node": 1, "restrained": ["Ux", "Uy", "Rz"]}],
            "masses": [{"id": "m", "node": 2, "values": [{"dof": "Ux", "value": 5.0}]}]
        },
        "cases": [{"name": "dead", "loads": [{"node": 2, "values": [{"dof": "Uy", "value": -100.0}]}]}],
        "combinations": [{"id": "uls", "terms": [{"case": "dead", "factor": 1.4}]}],
        "analyses": [{"method": "linearStatic"}], "options": {}
    })
}

#[test]
fn valid_domain_and_all_declared_methods() {
    let v = input();
    assert_eq!(capabilities()["run"], true);
    assert!(validate(&v.to_string()).unwrap().is_empty());
    for method in ["linearStatic", "pDelta", "modal", "responseSpectrum"] {
        assert_eq!(capabilities()["methods"][method]["engine"], true);
        assert_eq!(capabilities()["methods"][method]["run"], true);
    }
    assert!(capabilities()["operations"]
        .as_array()
        .unwrap()
        .contains(&json!("run")));
    assert_eq!(version()["engine_version"], "0.1.0");
    assert_eq!(
        version()["contracts"],
        json!(["engine-input/1", "results/1"])
    );
}

#[test]
fn required_schema_and_unknown_contract_or_major() {
    let mut v = input();
    for contract in [
        "other/1",
        "engine-input/2",
        "engine-input/999",
        "engine-input/1.1",
        "results/1",
    ] {
        v["schema"] = json!(contract);
        assert_eq!(
            validate(&v.to_string()),
            Err(ApiError::UnsupportedSchema(contract.into()))
        );
    }
    v.as_object_mut().unwrap().remove("schema");
    assert!(matches!(
        validate(&v.to_string()),
        Err(ApiError::InvalidInput(_))
    ));
    v["schema"] = json!(1);
    assert!(matches!(
        validate(&v.to_string()),
        Err(ApiError::InvalidInput(_))
    ));
}

#[test]
fn strict_json_rejects_nonfinite_duplicates_and_trailing_documents() {
    for bad in [
        "NaN",
        "Infinity",
        "-Infinity",
        "1e999",
        "null {}",
        r#"{"a":1,"a":2}"#,
        r#"{"a":{"b":1,"\u0062":2}}"#,
    ] {
        assert!(
            matches!(canonical_json(bad), Err(ApiError::InvalidInput(_))),
            "{bad}"
        );
    }
    let v = input().to_string();
    for bad in ["NaN", "Infinity", "-Infinity", "1e999", "\"NaN\"", "null"] {
        let invalid = v.replace("2.0", bad);
        assert!(
            matches!(validate(&invalid), Err(ApiError::InvalidInput(_))),
            "{bad}"
        );
    }
    let duplicate = v.replacen(
        "\"schema\":",
        "\"schema\":\"engine-input/1\",\"schema\":",
        1,
    );
    assert!(matches!(
        validate(&duplicate),
        Err(ApiError::InvalidInput(_))
    ));
}

#[test]
fn strict_fields_types_units_and_axes() {
    let baseline = input();
    for pointer in [
        "/model/nodes/0/id",
        "/model/nodes/0/coordinates",
        "/model/members/0/nodes",
        "/cases",
        "/options",
    ] {
        let mut v = baseline.clone();
        *v.pointer_mut(pointer).unwrap() = Value::Null;
        assert!(
            matches!(validate(&v.to_string()), Err(ApiError::InvalidInput(_))),
            "{pointer}"
        );
    }
    for pointer in [
        "",
        "/model",
        "/model/nodes/0",
        "/model/members/0",
        "/model/sections/0",
        "/model/laws/0",
        "/model/supports/0",
        "/model/masses/0",
        "/cases/0/loads/0",
        "/combinations/0/terms/0",
        "/analyses/0",
        "/options",
    ] {
        let mut v = baseline.clone();
        v.pointer_mut(pointer).unwrap()["typo"] = json!(1);
        assert!(
            matches!(validate(&v.to_string()), Err(ApiError::InvalidInput(_))),
            "{pointer}"
        );
    }
    for (pointer, value) in [
        ("/units", json!("kN-mm")),
        ("/model/axes", json!("Y-up")),
        ("/model/nodes/0/dim", json!("D4")),
        ("/model/nodes/0/coordinates/2", json!(1.0)),
        ("/analyses/0/method", json!("newton")),
        ("/model/nodes/0/id", json!(4294967296_u64)),
        ("/model/nodes/0/id", json!(true)),
    ] {
        let mut v = baseline.clone();
        *v.pointer_mut(pointer).unwrap() = value;
        assert!(
            matches!(validate(&v.to_string()), Err(ApiError::InvalidInput(_))),
            "{pointer}"
        );
    }
}

#[test]
fn object_dtos_and_enum_labels_reject_positional_or_external_forms() {
    for (pointer, value) in [
        ("/options", json!([])),
        ("/analyses/0", json!(["linearStatic"])),
        ("/model/sections/0/properties/0", json!(["A", 0.01])),
        ("/units", json!({"SI": null})),
        ("/model/axes", json!({"Z-up": null})),
        ("/model/nodes/0/dim", json!({"D2": null})),
        ("/model/nodes/0/dofs/0", json!({"Ux": null})),
        ("/analyses/0/method", json!({"linearStatic": null})),
    ] {
        let mut v = input();
        *v.pointer_mut(pointer).unwrap() = value;
        assert!(
            matches!(validate(&v.to_string()), Err(ApiError::InvalidInput(_))),
            "{pointer}"
        );
    }
}

#[test]
fn spatial_model_keeps_z_up_and_all_six_components() {
    let mut v = input();
    for n in v["model"]["nodes"].as_array_mut().unwrap() {
        n["dim"] = json!("D3");
        n["dofs"] = json!(["Ux", "Uy", "Uz", "Rx", "Ry", "Rz"]);
    }
    v["model"]["nodes"][1]["coordinates"] = json!([0.0, 0.0, 3.0]);
    assert!(validate(&v.to_string()).unwrap().is_empty());
}

#[test]
fn topology_diagnostics_preserve_codes_entities_and_order() {
    let mut v = input();
    let duplicate = v["model"]["nodes"][0].clone();
    v["model"]["nodes"].as_array_mut().unwrap().push(duplicate);
    v["model"]["nodes"][1]["coordinates"] = json!([0.0, 0.0, 0.0]);
    v["model"]["nodes"][0]["dofs"] = json!(["Uz"]);
    v["model"]["supports"][0]["node"] = json!(99);
    v["model"]["masses"][0]["values"][0]["dof"] = json!("Uz");
    v["cases"][0]["loads"][0]["node"] = json!(88);
    v["combinations"][0]["terms"][0]["case"] = json!("missing");
    let first = validate(&v.to_string()).unwrap();
    assert_eq!(first, validate(&v.to_string()).unwrap());
    assert_eq!(
        first.iter().map(|d| d.code).collect::<Vec<_>>(),
        [
            "ENG-E-0011",
            "ENG-E-0002",
            "ENG-E-0005",
            "ENG-E-0003",
            "ENG-E-0003",
            "ENG-E-0003",
            "ENG-E-0011"
        ]
    );
    assert_eq!(first[0].entities, [json!({"kind":"node", "id":1})]);
    assert_eq!(first[2].entities, [json!({"kind":"member", "id":7})]);
    assert_eq!(first[5].entities, [json!({"kind":"case", "id":"missing"})]);
    assert!(first
        .iter()
        .all(|d| d.severity == "error" && !d.cause.is_empty() && d.hint.is_some()));
    let serialized = serde_json::to_string(&first).unwrap();
    assert_eq!(
        canonical_json(&serialized).unwrap(),
        canonical_json(&serialized).unwrap()
    );
}

#[test]
fn orphan_and_broken_member_references_are_domain_errors() {
    let mut v = input();
    v["model"]["members"][0]["nodes"] = json!([1, 99]);
    v["model"]["members"][0]["section"] = json!("absent");
    v["model"]["members"][0]["law"] = json!("absent");
    let d = validate(&v.to_string()).unwrap();
    assert_eq!(
        d.iter().map(|d| d.code).collect::<Vec<_>>(),
        ["ENG-E-0004", "ENG-E-0003", "ENG-E-0003", "ENG-E-0003"]
    );
}

#[test]
fn explain_uses_every_catalog_entry_and_rejects_unknown() {
    for code in engine_core::catalog::CODES {
        assert_eq!(explain(code.code).unwrap(), code.explanation);
    }
    assert_eq!(
        explain("ENG-E-9999"),
        Err(ApiError::UnknownCode("ENG-E-9999".into()))
    );
}

#[test]
fn static_schemas_are_versioned_and_capabilities_are_honest() {
    for (kind, id) in [
        ("engine-input/1", "urn:mamut:engine-input:1"),
        ("results/1", "urn:mamut:results:1"),
        ("validate-report/1", "urn:mamut:validate-report:1"),
    ] {
        let doc = schema(kind).unwrap();
        let v: Value = serde_json::from_str(doc).unwrap();
        assert_eq!(v["$id"], id);
        assert_eq!(v["properties"]["schema"]["const"], kind);
        assert_eq!(v["additionalProperties"], false);
        assert_eq!(canonical_json(doc).unwrap(), canonical_json(doc).unwrap());
    }
    assert_eq!(
        schema("results/2"),
        Err(ApiError::UnknownSchema("results/2".into()))
    );
    assert_eq!(capabilities()["schemas"]["results/1"]["emit"], true);
    assert_eq!(capabilities()["schemas"]["validate-report/1"]["emit"], true);
}

#[test]
fn canonical_keys_nested_arrays_and_negative_zero() {
    let a = r#"{"z":[-0.0,{"b":-0e10,"a":1}],"a":{"z":3,"b":2}}"#;
    let b = r#"{ "a":{"b":2,"z":3}, "z":[0.0,{"a":1,"b":0.0}]}"#;
    let expected = r#"{"a":{"b":2,"z":3},"z":[0.0,{"a":1,"b":0.0}]}"#;
    assert_eq!(canonical_json(a).unwrap(), expected);
    assert_eq!(canonical_json(b).unwrap(), expected);
    assert_eq!(canonical_json(expected).unwrap(), expected);
    assert_eq!(canonical_json("-0.0").unwrap(), "0.0");
    assert_eq!(
        canonical_json("9007199254740991").unwrap(),
        "9007199254740991"
    );
    let v = input().to_string();
    for _ in 0..20 {
        assert_eq!(
            canonical_json(&v).unwrap().as_bytes(),
            canonical_json(&v).unwrap().as_bytes()
        );
    }
}

#[test]
fn canonical_floats_round_trip_at_binary64_boundaries() {
    for value in [
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::MAX,
        -f64::MAX,
        0.1,
        1.2345678901234567,
        f64::from_bits(0x3fdfffffffffffff),
    ] {
        let input = format!("{value:e}");
        let canonical = canonical_json(&input).unwrap();
        let recovered: f64 = serde_json::from_str(&canonical).unwrap();
        assert_eq!(recovered.to_bits(), value.to_bits());
        assert_eq!(canonical_json(&canonical).unwrap(), canonical);
    }
}

#[test]
fn cancelled_before_start_wins_even_over_invalid_json() {
    let cancel = CancelToken::new();
    assert!(!cancel.is_cancelled());
    cancel.cancel();
    let mut callbacks = 0;
    assert_eq!(
        validate_with("not JSON", &cancel, |_| callbacks += 1),
        Err(ApiError::Cancelled)
    );
    assert_eq!(callbacks, 0);
    assert_eq!(
        serde_json::to_string(&ApiError::Cancelled).unwrap(),
        r#"{"kind":"Cancelled"}"#
    );
}

#[test]
fn cancellation_at_each_boundary_stops_further_steps() {
    for boundary in 1..=3 {
        let cancel = CancelToken::new();
        let mut events = Vec::new();
        let result = validate_with(&input().to_string(), &cancel, |p| {
            events.push(p);
            if p.completed == boundary {
                cancel.cancel();
            }
        });
        assert_eq!(result, Err(ApiError::Cancelled));
        assert_eq!(events.len(), boundary);
    }
    // Cancelar después del parseo impide incluso el mapeo inválido siguiente.
    let cancel = CancelToken::new();
    assert_eq!(
        validate_with(r#"{"schema":"engine-input/1"}"#, &cancel, |_| cancel
            .cancel()),
        Err(ApiError::Cancelled)
    );
}

#[test]
fn successful_progress_is_monotone_and_deterministic() {
    let mut events = Vec::new();
    assert!(
        validate_with(&input().to_string(), &CancelToken::new(), |p| events
            .push(p))
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        events.iter().map(|p| p.stage).collect::<Vec<_>>(),
        ["parse", "model", "topology"]
    );
    assert_eq!(
        events.iter().map(|p| p.completed).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert!(events
        .iter()
        .all(|p| p.operation == "validate" && p.total == 3));
}
