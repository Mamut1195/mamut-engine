// SPDX-License-Identifier: AGPL-3.0-only

//! Helpers for cli-error/1 using the shared validator in engine-api/tests/support.
//! No new dependencies or remote resolution; unknown keywords fail.
use serde_json::Value;

pub fn assert_error_stream(bytes: &[u8], exit_code: i32) -> Value {
    let text = std::str::from_utf8(bytes).unwrap();
    assert!(
        text.ends_with('\n'),
        "Envelope without NDJSON terminator: {text}"
    );
    let mut errors = Vec::new();
    for line in text.lines() {
        let value: Value = serde_json::from_str(line).unwrap();
        if value["schema"] == "cli-error/1" {
            assert!(conforms(&value), "cli-error/1 outside schema: {value}");
            assert_registered(&value);
            assert_eq!(value["exitCode"], exit_code);
            errors.push(value);
        } else {
            assert!(errors.is_empty(), "Progress after the terminal envelope");
            if value["schema"] == "progress/1" {
                let root: Value =
                    serde_json::from_str(include_str!("../../schemas/progress-1.json")).unwrap();
                assert!(
                    schema_validator::conforms(&value, &root),
                    "progress/1 outside schema: {value}"
                );
            } else {
                assert!(
                    matches!(value["phase"].as_str(), Some("started" | "finished")),
                    "Emitter outside contract: {value}"
                );
            }
        }
    }
    assert_eq!(
        errors.len(),
        1,
        "There must be a single terminal error: {text}"
    );
    errors.remove(0)
}

fn assert_registered(envelope: &Value) {
    let registry: Value = serde_json::from_str(include_str!(
        "../../../../docs/contracts/error-codes-1.json"
    ))
    .unwrap();
    let codes = registry["codes"].as_array().unwrap();
    let definition = codes
        .iter()
        .find(|c| c["id"] == envelope["code"])
        .expect("Unregistered code");
    assert_eq!(definition["exitCode"], envelope["exitCode"]);
    assert_eq!(definition["outcome"], envelope["outcome"]);
    assert_eq!(definition["messageKey"], envelope["messageKey"]);
    assert_eq!(envelope["error"]["code"], envelope["code"]);

    assert_eq!(
        envelope["error"]["message"],
        engine_api::message(
            engine_api::Lang::En,
            definition["messageKey"].as_str().unwrap()
        )
        .unwrap()
    );
    for diagnostic in envelope["diagnostics"].as_array().unwrap() {
        assert!(
            codes.iter().any(|c| c["id"] == diagnostic["code"]),
            "Unregistered diagnostic: {diagnostic}"
        );
    }
}

#[path = "../../../engine-api/tests/support/schema.rs"]
pub(crate) mod schema_validator;

pub fn conforms(value: &Value) -> bool {
    let root: Value = serde_json::from_str(include_str!("../../schemas/cli-error-1.json")).unwrap();
    schema_validator::conforms(value, &root)
}
