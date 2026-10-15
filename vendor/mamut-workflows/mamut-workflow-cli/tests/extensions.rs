// SPDX-License-Identifier: Apache-2.0

use std::{
    cell::Cell,
    io::{self, Read, Write},
    rc::Rc,
};

use mamut_workflow::{
    CompiledStep, MethodCatalog, MethodDescriptor, RegisteredCatalog, RegisteredMethod,
};
use mamut_workflow_cli::{
    dispatch, parse_args, run, run_one_method_with, CancelFlag, CliConfig, CliIdentity, Command,
    ExitCodes, FailureKind, Method, MethodProvider, ProgressEvent, ProgressListener, ProgressPhase,
    EXIT_CANCELLED, MAX_DOCUMENT_BYTES,
};
use serde_json::{json, Value};

const CATALOG: &str = include_str!("../../../examples/open-local/catalog.json");
const WORKFLOW: &str = include_str!("../../../examples/open-local/workflow.json");

fn catalog_path() -> String {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/open-local/catalog.json"
    )
    .into()
}
fn workflow_path() -> String {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/open-local/workflow.json"
    )
    .into()
}
fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}
fn run_args() -> Vec<String> {
    args(&[
        "run",
        "--catalog",
        &catalog_path(),
        "--workflow",
        &workflow_path(),
    ])
}
fn capture(arguments: &[String], input: &str, config: &CliConfig<'_>) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(arguments, &mut input.as_bytes(), &mut out, &mut err, config);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}
fn value(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

struct EngineProvider {
    calls: Rc<Cell<usize>>,
}
struct EngineMethod {
    output: Value,
    calls: Rc<Cell<usize>>,
}
impl Method for EngineMethod {
    fn run(&mut self) -> Result<Value, String> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.output.clone())
    }
}
impl MethodProvider for EngineProvider {
    fn create(
        &self,
        descriptor: &MethodDescriptor,
        step: &CompiledStep<RegisteredMethod>,
    ) -> Result<Box<dyn Method>, String> {
        assert_eq!(descriptor.method_id, step.method_id);
        Ok(Box::new(EngineMethod {
            output: json!({"engine_result":step.parameters["n"] * 3.0}),
            calls: self.calls.clone(),
        }))
    }
}

#[test]
fn real_provider_output_is_recorded_in_the_manifest() {
    let calls = Rc::new(Cell::new(0));
    let config = CliConfig::new().with_provider(EngineProvider {
        calls: calls.clone(),
    });
    let (code, out, err) = capture(&run_args(), "", &config);
    assert_eq!(code, 0);
    assert!(err.is_empty());
    assert_eq!(calls.get(), 1);
    let fingerprint = mamut_canon::fingerprint_value(&json!({"engine_result":6.0})).unwrap();
    assert_eq!(
        value(&out)["data"]["steps"][0]["outputs"][0]["fingerprint"],
        serde_json::to_value(fingerprint).unwrap()
    );

    let catalog = RegisteredCatalog::from_json(CATALOG).unwrap();
    let descriptor = catalog.descriptors()[0];
    let provider = EngineProvider { calls };
    let manifest =
        run_one_method_with(&catalog, descriptor, [("n".into(), 2.0)].into(), &provider).unwrap();
    assert_eq!(serde_json::to_value(manifest).unwrap(), value(&out)["data"]);
}

#[test]
fn a_foreign_binary_uses_public_parse_dispatch_and_its_own_identity() {
    let config = CliConfig::new()
        .with_identity(CliIdentity {
            engine_id: "foreign-engine".into(),
            engine_version: "9.1".into(),
            operation: "simulate".into(),
        })
        .with_describe("foreign-cli", vec!["run".into(), "describe".into()]);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    dispatch(
        parse_args(&run_args()).unwrap(),
        &mut io::empty(),
        &mut out,
        &mut err,
        &config,
    )
    .unwrap();
    let result: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(result["engineId"], "foreign-engine");
    assert_eq!(result["engineVersion"], "9.1");
    assert_eq!(result["operation"], "simulate");
    let (_, out, _) = capture(&args(&["describe"]), "", &config);
    assert_eq!(value(&out)["name"], "foreign-cli");
    assert_eq!(value(&out)["version"], "9.1");
    assert_eq!(value(&out)["commands"], json!(["run", "describe"]));
    let (code, out, err) = capture(&args(&["run"]), "", &config);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert_eq!(value(&err)["engineId"], "foreign-engine");
    assert_eq!(value(&err)["operation"], "simulate");
}

#[test]
fn each_document_can_come_from_stdin_but_not_both() {
    let config = CliConfig::default();
    let expected = capture(&run_args(), "", &config);
    assert_eq!(
        capture(
            &args(&["run", "--catalog", "-", "--workflow", &workflow_path()]),
            CATALOG,
            &config
        ),
        expected
    );
    assert_eq!(
        capture(
            &args(&["run", "--catalog", &catalog_path(), "--workflow", "-"]),
            WORKFLOW,
            &config
        ),
        expected
    );
    let (code, out, err) = capture(&args(&["methods", "--catalog", "-"]), CATALOG, &config);
    assert_eq!(code, 0);
    assert!(!out.is_empty());
    assert!(err.is_empty());
    let (code, out, err) = capture(
        &args(&["run", "--catalog", "-", "--workflow", "-"]),
        "{}",
        &config,
    );
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert_eq!(value(&err)["outcome"], "usage");
    let error = dispatch(
        Command::Run {
            catalog: "-".into(),
            workflow: "-".into(),
            progress_ndjson: false,
        },
        &mut Unreadable,
        &mut Vec::new(),
        &mut Vec::new(),
        &config,
    )
    .unwrap_err();
    assert_eq!(error.kind, FailureKind::Usage);
    let (code, _, err) = capture(
        &args(&["run", "--catalog", &catalog_path(), "--workflow", "-"]),
        "{",
        &config,
    );
    assert_eq!(code, 3);
    assert_eq!(value(&err)["outcome"], "invalid_input");
}
struct Unreadable;
impl Read for Unreadable {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("must not consume ambiguous stdin")
    }
}

fn two_steps() -> (Value, Value) {
    let (mut catalog, mut workflow) = (value(CATALOG), value(WORKFLOW));
    let mut slot = catalog["profile"]["slots"][0].clone();
    slot["capability_slot"] = json!("second");
    catalog["profile"]["slots"]
        .as_array_mut()
        .unwrap()
        .push(slot);
    let mut method = catalog["methods"][0].clone();
    method["capability_slot"] = json!("second");
    method["method_id"] = json!("sample-two");
    catalog["methods"].as_array_mut().unwrap().push(method);
    let mut step = workflow["steps"][0].clone();
    step["capability_slot"] = json!("second");
    step["method_id"] = json!("sample-two");
    workflow["steps"].as_array_mut().unwrap().push(step);
    (catalog, workflow)
}
fn temp_document(name: &str, value: &Value) -> String {
    temp_text_document(name, &value.to_string())
}
fn temp_text_document(name: &str, text: &str) -> String {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-extensions");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path.to_str().unwrap().into()
}

#[test]
fn ndjson_events_are_valid_ordered_and_do_not_contaminate_stdout() {
    let (catalog, workflow) = two_steps();
    let path = temp_document("ndjson-catalog.json", &catalog);
    let arguments = args(&[
        "run",
        "--catalog",
        &path,
        "--workflow",
        "-",
        "--progress",
        "ndjson",
    ]);
    let (code, out, err) = capture(&arguments, &workflow.to_string(), &CliConfig::default());
    assert_eq!(code, 0);
    assert_eq!(value(&out)["outcome"], "ok");
    let events: Vec<Value> = err.lines().map(value).collect();
    assert_eq!(events.len(), 4);
    for (i, event) in events.iter().enumerate() {
        assert_eq!(event["index"], i / 2);
        assert_eq!(
            event["phase"],
            if i % 2 == 0 { "started" } else { "finished" }
        );
        assert_eq!(
            event["method_id"],
            if i < 2 { "sample" } else { "sample-two" }
        );
        assert_eq!(event["method_version"], 1);
    }
    let (quiet_code, quiet_out, quiet_err) = capture(
        &arguments[..arguments.len() - 2],
        &workflow.to_string(),
        &CliConfig::default(),
    );
    assert_eq!(
        (quiet_code, quiet_out, quiet_err),
        (0, out.clone(), String::new())
    );
    let mut progress = Vec::new();
    {
        let config = CliConfig::new()
            .with_ndjson(true)
            .with_progress_writer(&mut progress);
        assert_eq!(
            capture(
                &arguments[..arguments.len() - 2],
                &workflow.to_string(),
                &config
            ),
            (0, out, String::new())
        );
    }
    assert_eq!(String::from_utf8(progress).unwrap(), err);
}

struct CancelAfterFirst {
    flag: CancelFlag,
}
impl ProgressListener for CancelAfterFirst {
    fn on_progress(&mut self, event: ProgressEvent) {
        if event.index == 0 && event.phase == ProgressPhase::Finished {
            self.flag.cancel();
        }
    }
}
#[test]
fn cancellation_before_step_n_returns_published_code_and_no_manifest() {
    let (catalog, workflow) = two_steps();
    let path = temp_document("cancel-catalog.json", &catalog);
    for codes in [ExitCodes::default(), ExitCodes::engine_v1()] {
        let calls = Rc::new(Cell::new(0));
        let flag = CancelFlag::new();
        let expected = codes.cancelled;
        let config = CliConfig::new()
            .with_exit_codes(codes)
            .with_provider(EngineProvider {
                calls: calls.clone(),
            })
            .with_progress(CancelAfterFirst { flag: flag.clone() })
            .with_cancellation(flag);
        let (code, out, err) = capture(
            &args(&["run", "--catalog", &path, "--workflow", "-"]),
            &workflow.to_string(),
            &config,
        );
        assert_eq!(code, expected);
        assert!(out.is_empty());
        assert_eq!(calls.get(), 1);
        let report = value(&err);
        assert_eq!(report["outcome"], "cancelled");
        assert_eq!(report["exitCode"], expected);
        assert!(report.get("data").is_none());
        assert!(!err.contains("execution-manifest"));
    }
    assert_eq!(EXIT_CANCELLED, 6);
    let flag = CancelFlag::new();
    flag.cancel();
    let (code, out, err) = capture(&run_args(), "", &CliConfig::new().with_cancellation(flag));
    assert_eq!(code, EXIT_CANCELLED);
    assert!(out.is_empty());
    assert_eq!(value(&err)["outcome"], "cancelled");
}

#[test]
fn engine_exit_table_remaps_semantics_instead_of_matching_numbers() {
    let codes = ExitCodes::engine_v1();
    assert_eq!(
        (
            codes.ok,
            codes.warning,
            codes.usage,
            codes.document,
            codes.validation
        ),
        (0, 1, 2, 2, 2)
    );
    assert_eq!(
        (
            codes.unsupported,
            codes.not_converged,
            codes.cancelled,
            codes.timeout,
            codes.internal
        ),
        (3, 4, 5, 6, 70)
    );
    let config = CliConfig::new().with_exit_codes(codes);
    for arguments in [
        args(&["run"]),
        args(&[
            "run",
            "--catalog",
            "/absent-sdk-wf-012",
            "--workflow",
            &workflow_path(),
        ]),
    ] {
        let (code, _, err) = capture(&arguments, "", &config);
        assert_eq!(code, 2);
        assert_eq!(value(&err)["exitCode"], 2);
    }
    let (code, _, err) = capture(
        &args(&["run", "--catalog", &catalog_path(), "--workflow", "-"]),
        &WORKFLOW.replace("sample", "missing"),
        &config,
    );
    assert_eq!(code, 2);
    assert_eq!(value(&err)["outcome"], "invalid_input");
    let (_, out, _) = capture(&args(&["describe"]), "", &config);
    assert_eq!(
        value(&out)["exit_codes"],
        json!({"0":"ok","1":"warning","2":"invalid","3":"unsupported","4":"not_converged","5":"cancelled","6":"timeout","70":"internal"})
    );
    let custom = ExitCodes {
        ok: 12,
        usage: 42,
        ..ExitCodes::default()
    };
    let config = CliConfig::new().with_exit_codes(custom);
    let (code, out, _) = capture(&run_args(), "", &config);
    assert_eq!(code, 12);
    assert_eq!(value(&out)["exitCode"], 12);
    assert_eq!(capture(&args(&["run"]), "", &config).0, 42);
}

#[test]
fn default_output_matches_compatibility_snapshots() {
    let config = CliConfig::default();
    for (name, expected) in [
        ("describe", include_str!("fixtures/describe.txt")),
        ("help", include_str!("fixtures/help.txt")),
        ("methods", include_str!("fixtures/methods.txt")),
        ("compile", include_str!("fixtures/compile.txt")),
        ("validate", include_str!("fixtures/validate.txt")),
        ("run", include_str!("fixtures/run.txt")),
    ] {
        let arguments = match name {
            "describe" | "help" => args(&[name]),
            "methods" => args(&[name, "--catalog", &catalog_path()]),
            _ => args(&[
                name,
                "--catalog",
                &catalog_path(),
                "--workflow",
                &workflow_path(),
            ]),
        };
        assert_eq!(
            capture(&arguments, "", &config),
            (0, expected.into(), String::new()),
            "{name}"
        );
    }
    for (arguments, code, expected) in [
        (args(&[]), 2, include_str!("fixtures/missing.txt")),
        (
            args(&["no-such-command"]),
            2,
            include_str!("fixtures/unknown.txt"),
        ),
        (args(&["run"]), 2, include_str!("fixtures/run-usage.txt")),
        (
            args(&[
                "run",
                "--catalog",
                "/absent-sdk-wf-012",
                "--workflow",
                &workflow_path(),
            ]),
            3,
            include_str!("fixtures/run-document.txt"),
        ),
    ] {
        assert_eq!(
            capture(&arguments, "", &config),
            (code, String::new(), expected.into())
        );
    }
    let mut catalog = value(CATALOG);
    catalog["methods"][0]["failure"] = json!("private engine detail");
    assert_eq!(
        capture(
            &args(&["run", "--catalog", "-", "--workflow", &workflow_path()]),
            &catalog.to_string(),
            &config
        ),
        (
            5,
            String::new(),
            include_str!("fixtures/run-method.txt").into()
        )
    );
    assert_eq!(
        capture(
            &args(&["run", "--catalog", &catalog_path(), "--workflow", "-"]),
            &WORKFLOW.replace("sample", "missing"),
            &config
        ),
        (
            4,
            String::new(),
            include_str!("fixtures/run-validation.txt").into()
        )
    );
}

struct BrokenWriter;
impl Write for BrokenWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("private io detail"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn output_and_progress_io_errors_use_the_configured_internal_code() {
    let config = CliConfig::new().with_exit_codes(ExitCodes::engine_v1());
    let mut err = Vec::new();
    assert_eq!(
        run(
            &run_args(),
            &mut io::empty(),
            &mut BrokenWriter,
            &mut err,
            &config
        ),
        70
    );
    assert_eq!(
        value(std::str::from_utf8(&err).unwrap())["outcome"],
        "internal"
    );
    let config = config.with_ndjson(true).with_progress_writer(BrokenWriter);
    let (code, out, err) = capture(&run_args(), "", &config);
    assert_eq!(code, 70);
    assert!(out.is_empty());
    assert_eq!(value(&err)["outcome"], "internal");
    assert!(!err.contains("private io detail"));
}

#[test]
fn invalid_progress_arguments_are_usage_errors() {
    for arguments in [
        args(&["run", "--progress", "text"]),
        args(&["describe", "--progress", "ndjson"]),
        args(&["run", "--progress", "ndjson", "--progress", "ndjson"]),
    ] {
        assert_eq!(parse_args(&arguments).unwrap_err().kind, FailureKind::Usage);
    }
}

struct RejectingProvider;
impl MethodProvider for RejectingProvider {
    fn create(
        &self,
        _: &MethodDescriptor,
        _: &CompiledStep<RegisteredMethod>,
    ) -> Result<Box<dyn Method>, String> {
        Err("private factory detail".into())
    }
}

#[test]
fn cancellation_prevents_factories_and_factory_failures_remain_sanitized() {
    let flag = CancelFlag::new();
    flag.cancel();
    let provider = RejectingProvider;
    let config = CliConfig::new()
        .with_provider(&provider)
        .with_cancellation(flag);
    let (code, out, err) = capture(&run_args(), "", &config);
    assert_eq!(code, EXIT_CANCELLED);
    assert!(out.is_empty());
    assert_eq!(value(&err)["outcome"], "cancelled");
    let config = CliConfig::new()
        .with_provider(&provider)
        .with_exit_codes(ExitCodes::engine_v1())
        .with_ndjson(true);
    let (code, out, err) = capture(&run_args(), "", &config);
    assert_eq!(code, 70);
    assert!(out.is_empty());
    assert!(!err.contains("private factory detail"));
    let lines: Vec<Value> = err.lines().map(value).collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["phase"], "started");
    assert_eq!(lines[1]["outcome"], "unknown");
    assert!(lines[1].get("data").is_none());
}

#[test]
fn mcp_dispatch_applies_the_same_engine_configuration_to_method_tools() {
    let calls = Rc::new(Cell::new(0));
    let flag = CancelFlag::new();
    let config = CliConfig::new()
        .with_provider(EngineProvider {
            calls: calls.clone(),
        })
        .with_identity(CliIdentity {
            engine_id: "mcp-engine".into(),
            engine_version: "9.1".into(),
            operation: "simulate".into(),
        })
        .with_exit_codes(ExitCodes::engine_v1())
        .with_cancellation(flag.clone())
        .with_ndjson(true);
    let mut input = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"only.sample.1","arguments":{"n":2}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"only.sample.1","arguments":{"n":"private invalid value"}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"only.sample.1","arguments":{}}}),
    ].map(|message| message.to_string()).join("\n");
    input.push('\n');
    let arguments = args(&["mcp", "--catalog", &catalog_path()]);
    let (code, out, err) = capture(&arguments, &input, &config);
    assert_eq!(code, 0);
    assert_eq!(calls.get(), 1);
    let lines: Vec<Value> = out.lines().map(value).collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(
        lines[0]["result"]["serverInfo"],
        json!({"name":"mcp-engine","version":"9.1"})
    );
    let envelope = &lines[1]["result"]["structuredContent"];
    assert_eq!(envelope["engineId"], "mcp-engine");
    assert_eq!(envelope["operation"], "simulate");
    assert_eq!(envelope["outcome"], "ok");
    assert_eq!(
        envelope["data"]["steps"][0]["outputs"][0]["fingerprint"],
        serde_json::to_value(
            mamut_canon::fingerprint_value(&json!({"engine_result":6.0})).unwrap()
        )
        .unwrap()
    );
    assert_eq!(
        value(lines[1]["result"]["content"][0]["text"].as_str().unwrap()),
        *envelope
    );
    for line in &lines[2..] {
        assert_eq!(line["result"]["isError"], true);
        assert_eq!(line["result"]["structuredContent"]["exitCode"], 2);
        assert_eq!(
            line["result"]["structuredContent"]["outcome"],
            "invalid_input"
        );
    }
    assert!(!out.contains("private invalid value"));
    assert_eq!(
        err.lines()
            .map(value)
            .map(|event| event["phase"].clone())
            .collect::<Vec<_>>(),
        vec![json!("started"), json!("finished")]
    );
    flag.cancel();
    let (code, out, err) = capture(&arguments, &input, &config);
    assert_eq!(code, 0);
    assert!(err.is_empty());
    assert_eq!(calls.get(), 1);
    let lines: Vec<Value> = out.lines().map(value).collect();
    let envelope = &lines[1]["result"]["structuredContent"];
    assert_eq!(envelope["outcome"], "cancelled");
    assert_eq!(envelope["exitCode"], 5);
    assert!(envelope.get("data").is_none());
}

#[test]
fn document_limit_applies_to_stdin_and_files_for_both_document_options() {
    assert_eq!(MAX_DOCUMENT_BYTES, 16 * 1024 * 1024);
    for (option, document) in [("catalog", CATALOG), ("workflow", WORKFLOW)] {
        let mut input = document.to_owned();
        input.extend(std::iter::repeat_n(' ', MAX_DOCUMENT_BYTES - input.len()));
        let path = temp_text_document(&format!("limit-{option}.json"), &input);
        for (source, label) in [("-", "stdin"), (path.as_str(), "file")] {
            let arguments = if option == "catalog" {
                args(&["methods", "--catalog", source])
            } else {
                args(&[
                    "validate",
                    "--catalog",
                    &catalog_path(),
                    "--workflow",
                    source,
                ])
            };
            let (code, _, err) = capture(&arguments, &input, &CliConfig::default());
            assert_eq!(code, 0, "{option} {label} at limit: {err}");
            let oversized = format!("{input} ");
            std::fs::write(&path, &oversized).unwrap();
            for (codes, expected) in [
                (ExitCodes::default(), 3),
                (ExitCodes::engine_v1(), 2),
                (
                    ExitCodes {
                        document: 42,
                        ..ExitCodes::default()
                    },
                    42,
                ),
            ] {
                let config = CliConfig::new().with_exit_codes(codes);
                let (code, out, err) = capture(&arguments, &oversized, &config);
                assert_eq!(code, expected, "{option} {label}: {err}");
                assert!(out.is_empty());
                assert_eq!(
                    value(&err)["message"],
                    "Input document exceeds the 16 MiB limit"
                );
                assert!(!err.contains(&path));
            }
            std::fs::write(&path, &input).unwrap();
        }
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn oversized_stdin_stops_reading_after_limit_plus_one() {
    let mut input = io::repeat(b' ').take((MAX_DOCUMENT_BYTES + 4096) as u64);
    let error = dispatch(
        Command::Methods {
            catalog: "-".into(),
        },
        &mut input,
        &mut Vec::new(),
        &mut Vec::new(),
        &CliConfig::default(),
    )
    .unwrap_err();
    assert_eq!(error.kind, FailureKind::Document);
    assert_eq!(error.code, 3);
    assert_eq!(input.limit(), 4095);
}

#[test]
fn unreadable_documents_do_not_disclose_paths_or_reader_errors() {
    struct PrivateError;
    impl Read for PrivateError {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("private reader detail /private/path"))
        }
    }
    for (path, mut reader) in [
        ("-", PrivateError),
        ("/absent-sdk-wf-012/private-document.json", PrivateError),
    ] {
        let error = dispatch(
            Command::Methods {
                catalog: path.into(),
            },
            &mut reader,
            &mut Vec::new(),
            &mut Vec::new(),
            &CliConfig::default(),
        )
        .unwrap_err();
        assert_eq!(error.kind, FailureKind::Document);
        assert_eq!(error.message, "Input document could not be read");
    }
}

#[test]
fn generic_cli_advertises_only_exit_categories_it_emits() {
    let (_, out, err) = capture(&args(&["describe"]), "", &CliConfig::default());
    assert!(err.is_empty());
    let description = value(&out);
    assert_eq!(
        description["exit_code_categories"],
        json!({
            "ok": 0, "usage": 2, "document": 3, "validation": 4,
            "method": 5, "cancelled": 6, "internal": 10,
        })
    );
    assert_eq!(
        description["reserved_exit_code_categories"],
        json!({"warning": 1, "unsupported": 5, "not_converged": 5, "timeout": 10})
    );
}

#[test]
fn describe_and_mcp_initialize_separate_emitted_and_reserved_exit_categories() {
    for codes in [
        ExitCodes::default(),
        ExitCodes::engine_v1(),
        ExitCodes {
            ok: 11,
            warning: 12,
            usage: 13,
            document: 14,
            validation: 15,
            unsupported: 16,
            not_converged: 17,
            method: 18,
            cancelled: 19,
            timeout: 20,
            internal: 21,
        },
    ] {
        let mut expected = json!({
            "ok": codes.ok, "usage": codes.usage,
            "document": codes.document, "validation": codes.validation,
            "method": codes.method, "cancelled": codes.cancelled,
            "internal": codes.internal,
        });
        let reserved = if codes == ExitCodes::default() {
            json!({
                "warning": codes.warning, "unsupported": codes.unsupported,
                "not_converged": codes.not_converged, "timeout": codes.timeout,
            })
        } else {
            expected["warning"] = json!(codes.warning);
            expected["unsupported"] = json!(codes.unsupported);
            expected["not_converged"] = json!(codes.not_converged);
            json!({"timeout": codes.timeout})
        };
        let config = CliConfig::new().with_exit_codes(codes.clone());
        let (_, out, err) = capture(&args(&["describe"]), "", &config);
        assert!(err.is_empty());
        let description = value(&out);
        assert_eq!(description["exit_code_categories"], expected);
        assert_eq!(description["reserved_exit_code_categories"], reserved);
        if codes == ExitCodes::default() {
            assert_eq!(
                description["exit_codes"],
                json!({
                    "0":"success", "2":"usage", "3":"document", "4":"validation",
                    "5":"method", "10":"internal"
                })
            );
        } else if codes == ExitCodes::engine_v1() {
            assert_eq!(
                description["exit_codes"],
                json!({
                    "0":"ok", "1":"warning", "2":"invalid", "3":"unsupported",
                    "4":"not_converged", "5":"cancelled", "6":"timeout", "70":"internal"
                })
            );
        } else {
            assert_eq!(description["exit_codes"]["11"], "success");
        }
        let request = json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{}});
        let (_, out, err) = capture(
            &args(&["mcp", "--catalog", &catalog_path()]),
            &format!("{request}\n"),
            &config,
        );
        assert!(err.is_empty());
        let response = value(&out);
        assert_eq!(
            response["result"]["capabilities"],
            json!({"tools":{"listChanged":false}})
        );
        let capability = &response["result"]["_meta"]["mamut/cli-run"];
        assert_eq!(capability["exit_codes"], description["exit_codes"]);
        assert_eq!(capability["exit_code_categories"], expected);
        assert_eq!(capability["reserved_exit_code_categories"], reserved);
    }
}
