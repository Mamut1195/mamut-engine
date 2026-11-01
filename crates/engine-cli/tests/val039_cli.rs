//! VAL-039: process exit codes and exact Engine diagnostics through the CLI.
use serde_json::{json, Value};
use std::{
    io::Write,
    process::{Command, Stdio},
};
#[path = "../../engine-api/tests/support/val039.rs"]
mod models;

fn rejected(raw: &str, expected: &str, exit: i32) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mamut-engine"))
        .args(["run", "--input", "-"])
        .env_remove("MAMUT_ENGINE_INPUT")
        .env_remove("MAMUT_ENGINE_CANCEL_FILE")
        .env_remove("MAMUT_ENGINE_TIMEOUT")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(raw.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["schema"], "cli-error/1");
    assert_eq!(envelope["exitCode"], exit);
    assert_eq!(envelope["code"], expected);
    assert_eq!(envelope["error"]["code"], expected);
    let results = envelope["engineResults"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["diagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(results[0]["diagnostics"][0]["code"], expected);
    for field in ["cases", "combinations", "modes", "envelopes"] {
        assert_eq!(results[0][field], json!([]));
    }
    let registry: Value =
        serde_json::from_str(include_str!("../../../docs/contracts/error-codes-1.json")).unwrap();
    let entry = registry["codes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == expected)
        .unwrap();
    assert_eq!(entry["exitCode"], exit);
    assert_eq!(envelope["outcome"], entry["outcome"]);
}

#[test]
fn topology_and_contract_errors_exit_two_with_exact_engine_codes() {
    for (_, input, code) in models::topology_errors() {
        rejected(&input.to_string(), code, 2);
    }
}

#[test]
fn mechanisms_massless_mechanism_and_supercritical_column_exit_four() {
    for (_, input, code) in models::unstable_models() {
        rejected(&input.to_string(), code, 4);
    }
}

#[test]
fn h1_unknown_units_report_eng_e_0001() {
    let mut input = models::cantilever();
    input["units"] = json!("kN-mm");
    rejected(&input.to_string(), "ENG-E-0001", 2);
}

#[test]
fn malformed_json_and_missing_model_exit_two_with_input_structure_code() {
    rejected("{", "API-E-INPUT", 2);
    let mut input = models::cantilever();
    input.as_object_mut().unwrap().remove("model");
    rejected(&input.to_string(), "API-E-INPUT", 2);
}
