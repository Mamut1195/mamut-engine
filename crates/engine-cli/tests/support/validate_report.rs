//! Uses the existing engine-api validator against the published schema.
use serde_json::{json, Value};

use crate::error_schema::schema_validator::conforms;

fn schema() -> Value {
    serde_json::from_str(engine_api::schema("validate-report/1").unwrap()).unwrap()
}

pub fn assert_validate_report(report: &Value) {
    assert!(conforms(report, &schema()), "Report outside schema");
}

#[test]
fn structural_checker_rejects_contract_mutations_at_every_level() {
    let schema = schema();
    let report = json!({"schema":"validate-report/1", "operation":"validate",
        "status":"warning", "diagnostics":[{"code":"API-W-TEST", "severity":"warning",
            "entities":[{"kind":"node", "id":1}], "cause":"Aviso sintético", "hint":null}]});
    assert!(conforms(&report, &schema));
    for pointer in ["", "/diagnostics/0", "/diagnostics/0/entities/0"] {
        let object = report.pointer(pointer).unwrap().as_object().unwrap();
        for key in object.keys() {
            let mut missing = report.clone();
            missing
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(!conforms(&missing, &schema), "Required: {pointer}/{key}");
        }
        let mut extra = report.clone();
        extra.pointer_mut(pointer).unwrap()["extra"] = json!(true);
        assert!(!conforms(&extra, &schema));
    }
    for (pointer, invalid) in [
        ("/schema", json!("results/1")),
        ("/operation", json!("run")),
        ("/status", json!("unstable")),
        ("/diagnostics", json!({})),
        ("/diagnostics/0", json!(null)),
        ("/diagnostics/0/code", json!("")),
        ("/diagnostics/0/code", json!(12)),
        ("/diagnostics/0/severity", json!("fatal")),
        ("/diagnostics/0/cause", json!(null)),
        ("/diagnostics/0/hint", json!(false)),
        ("/diagnostics/0/entities", json!(null)),
        ("/diagnostics/0/entities/0", json!([])),
        ("/diagnostics/0/entities/0/kind", json!("section")),
        ("/diagnostics/0/entities/0/id", json!(1.5)),
    ] {
        let mut mutated = report.clone();
        *mutated.pointer_mut(pointer).unwrap() = invalid;
        assert!(!conforms(&mutated, &schema), "{pointer}");
    }
}
