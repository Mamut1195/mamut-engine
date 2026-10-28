// SPDX-License-Identifier: AGPL-3.0-only

use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, Output, Stdio},
};

use engine_cli::EngineCli;
use mamut_workflow_cli::ExitCodes;
use serde_json::{json, Value};

#[path = "support/schema.rs"]
mod error_schema;
use error_schema::schema_validator;

const DESCRIBE_SCHEMA: &str = include_str!("../schemas/cli-describe-1.json");
const VERSION_SCHEMA: &str = include_str!("../schemas/version-1.json");
const EXPLAIN_SCHEMA: &str = include_str!("../schemas/explain-1.json");
const INDEX: &str = include_str!("../../../schemas/index.json");

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mamut-engine"));
    for name in [
        "MAMUT_ENGINE_INPUT",
        "MAMUT_ENGINE_TIMEOUT",
        "MAMUT_ENGINE_CANCEL_FILE",
    ] {
        command.env_remove(name);
    }
    command.stdin(Stdio::null());
    command
}

fn run(args: &[&str], code: i32) -> Output {
    let output = command().args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
    if code == 0 {
        assert!(output.stderr.is_empty(), "{args:?}");
    } else {
        assert!(output.stdout.is_empty(), "{args:?}");
        error_schema::assert_error_stream(&output.stderr, code);
    }
    output
}

fn checked_json(output: &Output, schema: &str) -> Value {
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let schema: Value = serde_json::from_str(schema).unwrap();
    assert!(schema_validator::conforms(&value, &schema));
    value
}

fn sha256(bytes: &[u8]) -> String {
    // Python is already required by the publication tests; standard library only.
    let mut child = Command::new("python3")
        .args([
            "-c",
            "import hashlib,sys; print(hashlib.sha256(sys.stdin.buffer.read()).hexdigest())",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().into()
}

#[test]
fn description_is_valid_deterministic_and_has_a_versioned_digest_snapshot() {
    let output = run(&["describe", "--json"], 0);
    let description = checked_json(&output, DESCRIBE_SCHEMA);
    let snapshot = include_str!("snapshots/describe-1.stdout.sha256").trim();
    assert_eq!(sha256(&output.stdout), snapshot);
    for args in [
        vec!["describe", "--json"],
        vec!["--help", "--json"],
        vec!["help", "--json"],
        vec!["-h", "--json"],
        vec!["describe", "--json", "--timeout", "60"],
    ] {
        assert_eq!(run(&args, 0).stdout, output.stdout);
    }
    assert_eq!(description["engineId"], "mamut-engine");
    assert_eq!(
        description["engineVersion"],
        engine_api::version()["engine_version"]
    );
    let codes = ExitCodes::engine_v1();
    assert_eq!(description["exitCodes"], codes.capabilities()["exit_codes"]);
    assert_eq!(
        description["exitCodeCategories"],
        json!({
            "ok":codes.ok,"warning":codes.warning,"usage":codes.usage,
            "document":codes.document,"validation":codes.validation,"unsupported":codes.unsupported,
            "not_converged":codes.not_converged,"method":codes.method,
            "cancelled":codes.cancelled,"timeout":codes.timeout,"internal":codes.internal
        })
    );
    let names: Vec<_> = description["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|command| command["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "describe",
            "help",
            "--version",
            "capabilities",
            "schema",
            "explain",
            "methods",
            "compile",
            "validate",
            "run",
            "mcp"
        ]
    );
    for flag in [
        "--input",
        "--catalog",
        "--workflow",
        "--progress",
        "--timeout",
    ] {
        let commands = description["commands"].as_array().unwrap();
        let flags = commands
            .iter()
            .flat_map(|command| command["flags"].as_array().unwrap())
            .chain(description["globalFlags"].as_array().unwrap());
        assert!(flags.into_iter().any(|entry| entry["name"] == flag));
    }
    for name in [
        "MAMUT_ENGINE_INPUT",
        "MAMUT_ENGINE_TIMEOUT",
        "MAMUT_ENGINE_CANCEL_FILE",
    ] {
        assert!(description["environment"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == name));
    }
    let limits = &description["limits"];
    for (name, expected) in [
        ("MAX_INPUT_BYTES", engine_api::MAX_INPUT_BYTES),
        ("MAX_NODES", engine_api::MAX_NODES),
        ("MAX_MEMBERS", engine_api::MAX_MEMBERS),
        ("MAX_CASES", engine_api::MAX_CASES),
        ("MAX_COMBINATIONS", engine_api::MAX_COMBINATIONS),
        ("MAX_LOADS", engine_api::MAX_LOADS),
        ("MAX_JSON_DEPTH", engine_api::MAX_JSON_DEPTH),
        ("MAX_JSON_ARRAY_ITEMS", engine_api::MAX_JSON_ARRAY_ITEMS),
        ("MAX_DOCUMENT_BYTES", mamut_workflow_cli::MAX_DOCUMENT_BYTES),
        ("MAX_TIMEOUT_SECONDS", 31_536_000),
    ] {
        assert_eq!(limits[name], expected);
    }
    for (name, expected) in [
        ("MAX_CONSOLIDATION_WORK", engine_api::MAX_CONSOLIDATION_WORK),
        ("MAX_CONSOLIDATION_COST", engine_api::MAX_CONSOLIDATION_COST),
        (
            "MAX_CONSOLIDATION_SEGMENTS",
            engine_api::MAX_CONSOLIDATION_SEGMENTS,
        ),
        (
            "MAX_CONSOLIDATION_SUBSTEPS",
            engine_api::MAX_CONSOLIDATION_SUBSTEPS,
        ),
    ] {
        assert_eq!(limits[name], json!(expected));
    }
}

#[test]
fn schema_command_preserves_every_api_byte_and_inventory_hashes_match_publication() {
    let description: Value =
        serde_json::from_slice(&run(&["describe", "--json"], 0).stdout).unwrap();
    let index: Value = serde_json::from_str(INDEX).unwrap();
    let schemas = description["schemas"].as_array().unwrap();
    assert_eq!(schemas.len(), index["schemas"].as_array().unwrap().len());
    assert!(schemas
        .iter()
        .any(|entry| entry["id"] == "cli-error/1" && entry["availableViaSchema"] == false));
    assert!(description["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["id"] == "cli-error/1" && entry["channel"] == "stderr"));
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    for published in index["schemas"].as_array().unwrap() {
        let kind = format!(
            "{}/{}",
            published["id"].as_str().unwrap(),
            published["version"]
        );
        let entry = schemas.iter().find(|entry| entry["id"] == kind).unwrap();
        assert_eq!(entry["$id"], published["$id"]);
        let bytes = fs::read(root.join(published["path"].as_str().unwrap())).unwrap();
        assert_eq!(entry["sha256"], sha256(&bytes));
        let from_api = engine_api::schema_kinds().any(|id| id == kind);
        assert_eq!(entry["availableViaSchema"], from_api);
        if from_api {
            let output = run(&["schema", &kind], 0);
            assert_eq!(output.stdout, engine_api::schema(&kind).unwrap().as_bytes());
            assert_eq!(output.stdout, bytes);
            let document: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(document["$id"], entry["$id"]);
        } else {
            run(&["schema", &kind], 2);
        }
    }
    for kind in engine_api::schema_kinds() {
        assert!(schemas
            .iter()
            .any(|entry| entry["id"] == kind && entry["availableViaSchema"] == true));
    }
}

#[test]
fn version_and_explanation_match_the_api_in_text_and_json() {
    let version = checked_json(&run(&["--version", "--json"], 0), VERSION_SCHEMA);
    assert_eq!(version, engine_api::version());
    assert_eq!(
        run(&["--version"], 0).stdout,
        format!(
            "mamut-engine {}\n",
            version["engine_version"].as_str().unwrap()
        )
        .as_bytes()
    );
    for code in ["ENG-E-0009", "API-E-INPUT"] {
        let explanation = engine_api::explain(code).unwrap();
        assert_eq!(
            run(&["explain", code], 0).stdout,
            format!("{explanation}\n").as_bytes()
        );
        let document = checked_json(&run(&["explain", code, "--json"], 0), EXPLAIN_SCHEMA);
        assert_eq!(document, json!({"code":code,"explanation":explanation}));
    }
}

#[test]
fn capabilities_match_the_api_and_its_existing_schema() {
    let output = run(&["capabilities"], 0);
    let mut schema: Value =
        serde_json::from_str(engine_api::schema("capabilities/1").unwrap()).unwrap();
    // Resolves the sole external reference with the actual document, without network access.
    let mut descriptor: Value =
        serde_json::from_str(engine_api::schema("method-descriptor/1").unwrap()).unwrap();
    schema["$defs"] = descriptor.as_object_mut().unwrap().remove("$defs").unwrap();
    schema["$defs"]["descriptor"] = descriptor;
    schema["properties"]["method_descriptors"]["items"]["$ref"] = json!("#/$defs/descriptor");
    let document = checked_json(&output, &schema.to_string());
    assert_eq!(document, engine_api::capabilities());
}

#[test]
fn invalid_queries_and_extra_arguments_exit_two_with_empty_stdout() {
    for args in [
        vec!["schema"],
        vec!["schema", "unknown/1"],
        vec!["schema", "engine-input/1", "--json"],
        vec!["explain"],
        vec!["explain", "unknown"],
        vec!["explain", "unknown", "--json"],
        vec!["explain", "ENG-E-0009", "--json", "extra"],
        vec!["capabilities", "extra"],
        vec!["--version", "extra"],
        vec!["--version", "--json", "--json"],
        vec!["describe", "--json", "extra"],
        vec!["--help", "--json", "extra"],
        vec!["describe", "--json", "--json"],
        vec!["describe", "--json", "--timeout", "0"],
    ] {
        let output = run(&args, 2);
        let error = error_schema::assert_error_stream(&output.stderr, 2);
        assert_eq!(error["code"], "CLI-E-USAGE");
        assert_eq!(
            error["operation"],
            match args[0] {
                "describe" => "describe",
                "--help" => "help",
                _ => "cli",
            }
        );
    }
    let output = command()
        .args(["--version", "--json"])
        .env("MAMUT_ENGINE_TIMEOUT", "invalid")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    error_schema::assert_error_stream(&output.stderr, 2);
    let output = command()
        .args(["--version", "--json", "--timeout", "60"])
        .env("MAMUT_ENGINE_TIMEOUT", "invalid")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(checked_json(&output, VERSION_SCHEMA), engine_api::version());
}

#[test]
fn query_output_failures_use_the_registered_terminal_error_contract() {
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    for args in [
        vec!["describe", "--json"],
        vec!["--help", "--json"],
        vec!["--version"],
        vec!["--version", "--json"],
        vec!["capabilities"],
        vec!["schema", "engine-input/1"],
        vec!["explain", "ENG-E-0009"],
        vec!["explain", "ENG-E-0009", "--json"],
    ] {
        let args: Vec<String> = args.into_iter().map(String::from).collect();
        let mut stderr = Vec::new();
        let code =
            EngineCli::new(None).run(&args, &mut io::empty(), &mut BrokenWriter, &mut stderr);
        assert_eq!(code, 70);
        let error = error_schema::assert_error_stream(&stderr, 70);
        assert_eq!(error["code"], "CLI-E-INTERNAL");
    }
}

#[test]
fn text_help_uses_the_engine_name_including_nested_help() {
    for args in [
        vec!["--help"],
        vec!["help"],
        vec!["-h"],
        vec!["run", "--help"],
        vec!["describe", "-h"],
    ] {
        let output = run(&args, 0);
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.lines().all(|line| line.starts_with("mamut-engine ")));
    }
}

#[test]
fn new_schemas_reject_missing_fields_extra_fields_and_invalid_nested_types() {
    for (args, schema) in [
        (vec!["describe", "--json"], DESCRIBE_SCHEMA),
        (vec!["--version", "--json"], VERSION_SCHEMA),
        (vec!["explain", "ENG-E-0009", "--json"], EXPLAIN_SCHEMA),
    ] {
        let document = checked_json(&run(&args, 0), schema);
        let schema: Value = serde_json::from_str(schema).unwrap();
        for key in document.as_object().unwrap().keys() {
            let mut mutated = document.clone();
            mutated.as_object_mut().unwrap().remove(key);
            assert!(!schema_validator::conforms(&mutated, &schema), "{key}");
        }
        let mut mutated = document.clone();
        mutated["extra"] = json!(true);
        assert!(!schema_validator::conforms(&mutated, &schema));
    }
    let document = checked_json(&run(&["describe", "--json"], 0), DESCRIBE_SCHEMA);
    let schema: Value = serde_json::from_str(DESCRIBE_SCHEMA).unwrap();
    for (pointer, value) in [
        ("/commands/0/flags/0/required", json!("false")),
        ("/commands/0/subcommands", json!(["unknown"])),
        ("/environment/0/commands", json!(null)),
        ("/inputs/0/maxBytes", json!(-1)),
        ("/outputs/0/channel", json!("file")),
        ("/exitCodeCategories/timeout", json!("6")),
        ("/limits/MAX_INPUT_BYTES", json!(null)),
        ("/schemas/0/sha256", json!("invalid")),
        ("/schemas/0/availableViaSchema", json!(1)),
    ] {
        let mut mutated = document.clone();
        *mutated.pointer_mut(pointer).unwrap() = value;
        assert!(!schema_validator::conforms(&mutated, &schema), "{pointer}");
    }
}
