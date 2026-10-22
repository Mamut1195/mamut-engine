// SPDX-License-Identifier: AGPL-3.0-only

#[path = "support/schema.rs"]
mod schema;

use engine_cli::EngineCli;
use serde_json::{json, Value};

#[test]
fn schema_rejects_missing_fields_wrong_types_unknown_codes_and_invalid_pointers() {
    let mut stderr = Vec::new();
    let code = EngineCli::new(None).run(
        &["unknown".into()],
        &mut std::io::empty(),
        &mut Vec::new(),
        &mut stderr,
    );
    let envelope = schema::assert_error_stream(&stderr, code.into());
    let definition: Value =
        serde_json::from_str(include_str!("../schemas/cli-error-1.json")).unwrap();
    for field in definition["required"].as_array().unwrap() {
        let mut missing = envelope.clone();
        missing
            .as_object_mut()
            .unwrap()
            .remove(field.as_str().unwrap());
        assert!(
            !schema::conforms(&missing),
            "Campo obligatorio no comprobado: {field}"
        );
    }
    for (field, value) in [
        ("schema", json!("cli-run/1")),
        ("exitCode", json!(0)),
        ("exitCode", json!("2")),
        ("exitCode", json!(3)), // incompatibilidad outcome/exit
        ("outcome", json!("unknown")),
        ("code", json!("WF-E-0002")),
        ("diagnostics", Value::Null),
        ("diagnostics", json!([{}])),
        (
            "diagnostics",
            json!([{"code":"ENG-E-0003","severity":"fatal","entities":[],"cause":"Missing","hint":null}]),
        ),
        ("pointer", json!("nodes/0")),
        ("pointer", json!("/bad~2escape")),
        ("pointer", json!("/bad~")),
        ("hint", json!(42)),
        ("operation", json!("private/path")),
        ("error", json!({"code":"CLI-E-USAGE"})),
        ("unexpected", json!(true)),
    ] {
        let mut invalid = envelope.clone();
        invalid[field] = value;
        assert!(
            !schema::conforms(&invalid),
            "Mutación no detectada: {invalid}"
        );
    }
    for pointer in ["", "/", "/nodes/0", "/a~1b/~0/ñ"] {
        let mut valid = envelope.clone();
        valid["pointer"] = json!(pointer);
        assert!(schema::conforms(&valid));
    }
}
