//! MAM-835: la función exportada WASM se ejecuta también de forma nativa.
#[path = "../../engine-api/tests/support/sfc010.rs"]
mod fixtures;
#[test]
fn exported_wasm_run_preserves_api_bytes_for_every_sfc010_fixture() {
    for &(name, input) in fixtures::FIXTURES {
        let expected = engine_api::run(input, &engine_api::CancelToken::new(), |_| {});
        assert_eq!(
            expected.status,
            engine_api::Status::Ok,
            "{name}: {}",
            expected.document
        );
        let actual = engine_wasm::run(input, &engine_wasm::CancelToken::new(), None).unwrap();
        assert_eq!(actual.as_bytes(), expected.document.as_bytes(), "{name}");
    }
}

#[test]
fn exported_wasm_discovery_and_run_preserve_experimental_policy() {
    let capabilities: serde_json::Value =
        serde_json::from_str(&engine_wasm::capabilities().unwrap()).unwrap();
    for id in ["nonlinearStatic", "soilProfile", "consolidation1d"] {
        let descriptor = capabilities["method_descriptors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["workflow"]["method_id"] == id)
            .unwrap();
        assert_eq!(descriptor["status"], "Experimental");
    }
    assert!(engine_wasm::explain("API-E-SFC-OPTIONS")
        .unwrap()
        .contains("ENG-VAL-006"));
    for &(name, source) in fixtures::FIXTURES {
        let mut input: serde_json::Value = serde_json::from_str(source).unwrap();
        if input["options"]["allow_experimental"] != true {
            continue;
        }
        for options in [
            serde_json::json!({}),
            serde_json::json!({"allow_experimental": false}),
        ] {
            input["options"] = options;
            let actual =
                engine_wasm::run(&input.to_string(), &engine_wasm::CancelToken::new(), None)
                    .unwrap();
            let document: serde_json::Value = serde_json::from_str(&actual).unwrap();
            assert_eq!(document["status"], "invalid", "{name}: {actual}");
            assert_eq!(document["diagnostics"][0]["code"], "API-E-INPUT");
            assert!(actual.contains("options.allow_experimental = true"));
        }
    }
}
