// SPDX-License-Identifier: AGPL-3.0-only
//! ENG-SFC-004: exercise the registered argv and stdin transport as a CLI host.

#[path = "support/schema.rs"]
mod error_schema;
#[path = "support/stdin.rs"]
mod stdin;

use error_schema::schema_validator as schema;
use serde_json::{json, Value};
use std::{
    io::Read,
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

const PROGRAM: &str = include_str!("../harness/mamut-engine.cli-program.json");
const HARNESS_SCHEMA: &str = include_str!("fixtures/harness/cli-program-1.0.0.schema.json");
const INPUT: &str = include_str!("../../engine-api/tests/fixtures/determinism/cantilever.json");

fn value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

// Keep the upstream fixture intact. The shared validator supports const, but
// not these patterns or string lengths. Only those keywords are checked below;
// object closure, required fields, types, enums, refs and bounds stay shared.
fn structural_schema(spec: &mut Value) {
    match spec {
        Value::Object(fields) => {
            for keyword in ["pattern", "minLength", "maxLength"] {
                fields.remove(keyword);
            }
            for child in fields.values_mut() {
                structural_schema(child);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(structural_schema),
        _ => {}
    }
}

fn string_length(text: &str, rules: &Value) -> bool {
    let count = text.chars().count() as u64;
    rules["minLength"].as_u64().is_none_or(|min| count >= min)
        && rules["maxLength"].as_u64().is_none_or(|max| count <= max)
}

fn identifier(text: &str) -> bool {
    text.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && text.split(['.', '_', '-']).all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn semver(text: &str) -> bool {
    let identifiers = |suffix: &str| {
        suffix.split('.').all(|part| {
            !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    };
    let (base, build) = text
        .split_once('+')
        .map_or((text, None), |(a, b)| (a, Some(b)));
    let (core, pre) = base
        .split_once('-')
        .map_or((base, None), |(a, b)| (a, Some(b)));
    let parts: Vec<_> = core.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|b| b.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0'))
        })
        && pre.is_none_or(identifiers)
        && build.is_none_or(identifiers)
}

fn relative_path(text: &str) -> bool {
    let bytes = text.as_bytes();
    !text.starts_with(['/', '\\'])
        && !(bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        && !text.split(['/', '\\']).any(|part| part == "..")
        && !text.contains(['\r', '\n'])
}

fn program_conforms(program: &Value) -> bool {
    let spec = value(HARNESS_SCHEMA.as_bytes());
    let mut structural = spec.clone();
    structural_schema(&mut structural);
    if !schema::conforms(program, &structural) {
        return false;
    }
    let defs = &spec["$defs"];
    let id = |v: &Value| {
        let text = v.as_str().unwrap();
        identifier(text) && string_length(text, &defs["identifier"])
    };
    let path = program["launcher"]["relative_path"].as_str().unwrap();
    id(&program["program_id"])
        && semver(program["version"].as_str().unwrap())
        && string_length(
            program["display_name"].as_str().unwrap(),
            &spec["properties"]["display_name"],
        )
        && string_length(path, &defs["relativePath"])
        && relative_path(path)
        && program["operations"].as_array().unwrap().iter().all(|op| {
            id(&op["capability_id"])
                && op["fixed_arguments"].as_array().unwrap().iter().all(|arg| {
                    let text = arg.as_str().unwrap();
                    string_length(text, &defs["argument"]) && !text.contains(['\r', '\n', '\0'])
                })
        })
}

#[test]
fn registry_entry_matches_schema_and_rejects_contract_violations() {
    let program = value(PROGRAM.as_bytes());
    assert!(program_conforms(&program));
    assert_eq!(program["program_id"], "mamut-engine");
    assert_eq!(program["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(program["launcher"]["kind"], "executable");
    assert_eq!(program["launcher"]["relative_path"], "bin/mamut-engine.exe");
    for (pointer, replacement) in [
        ("/schema_version", json!("2.0.0")),
        ("/launcher/platform", json!("linux")),
        ("/program_id", json!("Mamut")),
        ("/program_id", json!("mamut..engine")),
        ("/program_id", json!("m".repeat(129))),
        ("/display_name", json!("")),
        ("/display_name", json!("á".repeat(101))),
        ("/version", json!("01.0.0")),
        ("/version", json!("1.0")),
        ("/version", json!("1.0.0+")),
        ("/launcher/relative_path", json!("")),
        ("/launcher/relative_path", json!("x".repeat(261))),
        ("/launcher/relative_path", json!("../bin/engine.exe")),
        ("/launcher/relative_path", json!("bin\\..\\engine.exe")),
        ("/launcher/relative_path", json!("C:engine.exe")),
        ("/launcher/relative_path", json!("/engine.exe")),
        ("/launcher/relative_path", json!("\\engine.exe")),
        ("/launcher/relative_path", json!("bin/engine\n.exe")),
        ("/operations/0/capability_id", json!("bad-")),
        ("/operations/0/fixed_arguments/0", json!("run\r")),
        ("/operations/0/fixed_arguments/0", json!("run\n")),
        ("/operations/0/fixed_arguments/0", json!("run\0")),
        ("/operations/0/fixed_arguments/0", json!("x".repeat(257))),
        ("/operations/0/timeout_ms", json!(99)),
        ("/operations/0/max_output_bytes", json!(67108865)),
    ] {
        let mut bad = program.clone();
        *bad.pointer_mut(pointer).unwrap() = replacement;
        assert!(!program_conforms(&bad), "accepted mutation: {pointer}");
    }
    let mut extra = program.clone();
    extra["extra"] = json!(true);
    assert!(!program_conforms(&extra));
    for pointer in ["", "/launcher", "/operations/0"] {
        for key in program
            .pointer(pointer)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
        {
            let mut missing = program.clone();
            missing
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(!program_conforms(&missing), "missing: {pointer}/{key}");
        }
    }
}

fn read_bounded(stream: impl Read, limit: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    stream.take(limit + 1).read_to_end(&mut bytes).unwrap();
    bytes
}

fn invoke(op: &Value, input: Option<&str>) -> Output {
    let args: Vec<_> = op["fixed_arguments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    assert_eq!(input.is_some(), op["input"] == "json");
    let limit = op["max_output_bytes"].as_u64().unwrap();
    let timeout = Duration::from_millis(op["timeout_ms"].as_u64().unwrap());
    let mut command = Command::new(env!("CARGO_BIN_EXE_mamut-engine"));
    command.env_clear();
    // Windows needs its system directory; run materializes embedded workflow
    // documents in a private temporary directory. No PATH or MAMUT_* is inherited.
    #[cfg(windows)]
    {
        if let Some(root) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", root);
        }
        command.env("TMP", env!("CARGO_TARGET_TMPDIR"));
        command.env("TEMP", env!("CARGO_TARGET_TMPDIR"));
    }
    #[cfg(not(windows))]
    command.env("TMPDIR", env!("CARGO_TARGET_TMPDIR"));
    let started = Instant::now();
    let mut child = command
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || read_bounded(stdout, limit));
    let err = thread::spawn(move || read_bounded(stderr, limit));
    // These small documents cannot fill a pipe; the shared writer also closes
    // stdin, delivering EOF before waiting, and accepts only BrokenPipe errors.
    if let Some(input) = input {
        stdin::write_child_stdin(&mut child, input);
    }
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break (status, false);
        }
        if started.elapsed() >= timeout {
            child.kill().unwrap();
            break (child.wait().unwrap(), true);
        }
        thread::sleep(Duration::from_millis(5));
    };
    let stdout = out.join().unwrap();
    let stderr = err.join().unwrap();
    assert!(!timed_out, "{} exceeded {timeout:?}", op["capability_id"]);
    assert!(
        (stdout.len() + stderr.len()) as u64 <= limit,
        "output exceeds registry limit"
    );
    Output {
        status,
        stdout,
        stderr,
    }
}

fn checked_output(op: &Value, input: Option<&str>, code: i32) -> (Output, Value) {
    let output = invoke(op, input);
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    let document = if op["capability_id"] == "mamut-engine.run" && code > 1 {
        assert!(output.stdout.is_empty());
        error_schema::assert_error_stream(&output.stderr, code)
    } else {
        assert!(output.stderr.is_empty(), "{output:?}");
        let document = value(&output.stdout);
        let spec = match op["capability_id"].as_str().unwrap() {
            "mamut-engine.run" => include_str!("../schemas/cli-run-1.json"),
            "mamut-engine.validate" => {
                include_str!("../../engine-api/schemas/validate-report-1.json")
            }
            "mamut-engine.describe" => include_str!("../schemas/cli-describe-1.json"),
            other => panic!("unmapped capability: {other}"),
        };
        assert!(schema::conforms(&document, &value(spec.as_bytes())));
        document
    };
    if op["capability_id"] == "mamut-engine.run" {
        assert_eq!(document["exitCode"], code);
        let results = document["engineResults"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["schema"], "results/1");
        assert_eq!(
            results[0]["input_sha256"],
            engine_api::input_sha256(input.unwrap()).unwrap()
        );
        if code <= 1 {
            assert_eq!(
                document["outcome"],
                if code == 0 { "ok" } else { "warning" }
            );
        }
    } else if op["capability_id"] == "mamut-engine.validate" {
        assert_eq!(
            document["status"],
            match code {
                0 => "ok",
                1 => "warning",
                2 => "invalid",
                _ => panic!("invalid report exit"),
            }
        );
    }
    (output, document)
}

#[test]
fn registered_operations_run_in_subprocesses_with_minimal_environment() {
    // Engine execution is exclusively in CARGO_BIN_EXE_mamut-engine. This test
    // does not call/link an execution entry point (EngineCli, run or validate).
    // The only direct engine_api call computes the expected RFC 8785 input hash;
    // the existing shared schema helper uses API schema/catalog metadata only.
    let program = value(PROGRAM.as_bytes());
    assert!(program_conforms(&program));
    let operations = program["operations"].as_array().unwrap();
    assert_eq!(operations.len(), 3);
    let parsed = value(INPUT.as_bytes());
    // Serialize top-level keys in reverse order explicitly: Value's map would
    // sort them again. Array order and every input value are preserved.
    let reordered = format!(
        "{{{}}}",
        parsed
            .as_object()
            .unwrap()
            .iter()
            .rev()
            .map(|(key, val)| format!("{}:{val}", serde_json::to_string(key).unwrap()))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_ne!(reordered, INPUT);
    assert_eq!(value(reordered.as_bytes()), parsed);
    assert_eq!(
        engine_api::input_sha256(&reordered).unwrap(),
        engine_api::input_sha256(INPUT).unwrap()
    );
    for (op, expected_args) in operations.iter().zip([
        vec!["run", "--input", "-"],
        vec!["validate", "--input", "-"],
        vec!["describe", "--json"],
    ]) {
        assert_eq!(op["fixed_arguments"], json!(expected_args));
        assert_eq!(op["output"], "json");
        assert_eq!(op["mutates_external_state"], false);
        let input = (op["input"] == "json").then_some(INPUT);
        let (first, document) = checked_output(op, input, 0);
        let (second, repeated) = checked_output(op, input, 0);
        // Nothing is removed: metadata, manifests and fingerprints must repeat.
        assert_eq!(first.stdout, second.stdout);
        assert_eq!(document, repeated);
        if input.is_some() {
            let (permuted, _) = checked_output(op, Some(&reordered), 0);
            assert_eq!(first.stdout, permuted.stdout);
        } else {
            assert_eq!(document["engineVersion"], program["version"]);
            assert_eq!(document["engineId"], program["program_id"]);
            let max_input = document["limits"]["MAX_INPUT_BYTES"].as_u64().unwrap();
            assert_eq!(max_input, 8 * 1024 * 1024);
            for model_op in &operations[..2] {
                assert_eq!(
                    model_op["max_output_bytes"].as_u64().unwrap(),
                    8 * max_input
                );
            }
        }
    }
}

#[test]
fn stdin_model_outcomes_preserve_exit_table_and_deterministic_streams() {
    let program = value(PROGRAM.as_bytes());
    let mut warning = value(INPUT.as_bytes());
    warning["combinations"] = json!([{"id":"scaled","terms":[{"case":"D","factor":2.0}]}]);
    for (op, warning_code) in program["operations"].as_array().unwrap()[..2]
        .iter()
        .zip([1, 0])
    {
        // Same existing own warning fixture construction as cli.rs/contract.rs.
        // Direct validation checks topology and does not produce the run warning.
        for (input, code) in [(warning.to_string(), warning_code), ("{}".to_owned(), 2)] {
            let (first, _) = checked_output(op, Some(&input), code);
            let (second, _) = checked_output(op, Some(&input), code);
            assert_eq!(first.stdout, second.stdout);
            assert_eq!(first.stderr, second.stderr);
        }
    }
}
