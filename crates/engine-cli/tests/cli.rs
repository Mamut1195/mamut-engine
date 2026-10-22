// SPDX-License-Identifier: AGPL-3.0-only

use std::{
    cell::RefCell,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use engine_cli::{Control, EngineCli, CATALOG};
use mamut_workflow_cli::{Cancellation, ExitCodes};
use serde_json::{json, Value};

#[path = "../../engine-api/tests/support/schema.rs"]
mod schema_validator;

#[path = "support/validate_report.rs"]
mod validate_report_schema;
use validate_report_schema::assert_validate_report;

const RUN: &str = include_str!("../run.workflow.json");
const VALIDATE: &str = include_str!("../validate.workflow.json");
const INPUT: &str = include_str!("../../engine-api/tests/fixtures/determinism/cantilever.json");
#[path = "support/schema.rs"]
mod error_schema;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "engine-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        let files = Self(path);
        files.write("catalog.json", CATALOG);
        files.write("input.json", INPUT);
        files
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn write(&self, name: &str, contents: &str) {
        fs::write(self.path(name), contents).unwrap();
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mamut-engine"));
        command.env_remove("MAMUT_ENGINE_CANCEL_FILE");
        command.env_remove("MAMUT_ENGINE_TIMEOUT");
        command.env("MAMUT_ENGINE_INPUT", self.path("input.json"));
        command
    }
    fn run(&self, workflow: &str, progress: bool) -> Output {
        let mut command = self.command();
        command.args(["run", "--catalog"]);
        command.arg(self.path("catalog.json"));
        command.args(["--workflow", "-"]);
        if progress {
            command.args(["--progress", "ndjson"]);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        write_child_stdin(&mut child, workflow);
        child.wait_with_output().unwrap()
    }
    fn args(&self) -> Vec<String> {
        vec![
            "run".into(),
            "--catalog".into(),
            self.path("catalog.json").to_str().unwrap().into(),
            "--workflow".into(),
            "-".into(),
        ]
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn write_child_stdin(child: &mut Child, input: &str) {
    let mut stdin = child.stdin.take().unwrap();
    // El hijo puede terminar sin leer stdin; los asserts comprobarán su salida.
    if let Err(error) = stdin.write_all(input.as_bytes()) {
        assert_eq!(
            error.kind(),
            io::ErrorKind::BrokenPipe,
            "Error al escribir stdin del hijo: {error}"
        );
    }
    // Al salir se cierra stdin antes de esperar al hijo con wait_with_output.
}
fn value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}
fn assert_code(output: &Output, code: i32) {
    if code == 2 && !output.stdout.is_empty() {
        let report = value(&output.stdout);
        assert_validate_report(&report);
        assert_eq!(report["status"], "invalid");
        assert!(output.stderr.is_empty());
    } else if code > 1 {
        error_schema::assert_error_stream(&output.stderr, code);
        assert!(output.stdout.is_empty(), "stdout de fallo debe estar vacío");
    }
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// Derivada del fixture «consolidation» de engine-api/tests/support/sfc010.rs.
// 80 elementos, 64 intervalos y 72000 subpasos; permanece bajo las cotas
// públicas de trabajo y coste denso. Sin datos ni solvers externos.
fn heavy_consolidation() -> String {
    let mut input: Value = serde_json::from_str(include_str!(
        "../../engine-api/fixtures/input/valid/sfc010-consolidation.json"
    ))
    .unwrap();
    let options = &mut input["analyses"][0]["options"];
    let mut layer = options["layers"][0].clone();
    layer["elements"] = json!(20);
    options["layers"] = json!([layer, layer, layer, layer]);
    let mut time = 0.0;
    options["history"] = json!((0..=64)
        .map(|i| {
            if i > 0 {
                time += if i % 2 == 0 { 1.0 } else { 1.25 };
            }
            json!({"time_s":time,"delta_stress_kpa":{"value":20 + i % 3,"provenance":"measured"}})
        })
        .collect::<Vec<_>>());
    options["output_times_s"] = json!([0.0, time]);
    options["max_step_s"] = json!(0.001);
    input.to_string()
}

fn timed_run(files: &Files, timeout: &str, from_env: bool, progress: bool) -> Output {
    let start = Instant::now();
    let limit = Duration::from_secs_f64(timeout.parse::<f64>().unwrap() + 1.0);
    let mut command = files.command();
    command.args(files.args());
    if progress {
        command.args(["--progress", "ndjson"]);
    }
    if from_env {
        command.env("MAMUT_ENGINE_TIMEOUT", timeout);
    } else {
        command.args(["--timeout", timeout]);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    write_child_stdin(&mut child, RUN);
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() >= limit {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!("El timeout no terminó en {limit:?}: {output:?}");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        start.elapsed() < limit,
        "Plazo excedido: {:?}",
        start.elapsed()
    );
    output
}

#[test]
fn timeout_flag_and_environment_cancel_heavy_consolidation_with_six() {
    let files = Files::new();
    files.write("input.json", &heavy_consolidation());
    // Confirma que la entrada es ejecutable y no dispara las cotas de trabajo.
    let start = Instant::now();
    let baseline = files.run(RUN, false);
    assert_code(&baseline, 0);
    eprintln!("Consolidación pesada sin timeout: {:?}", start.elapsed());
    for from_env in [false, true] {
        let output = timed_run(&files, "0.15", from_env, false);
        assert_code(&output, 6);
        assert!(output.stdout.is_empty());
        let envelope = value(&output.stderr);
        assert_eq!(envelope["exitCode"], 6);
        assert_eq!(envelope["outcome"], "timeout");
        assert_eq!(envelope["code"], "CLI-E-TIMEOUT");
        assert_eq!(envelope["operation"], "run");
        assert_eq!(envelope["engineId"], "mamut-engine");
    }
}

#[cfg(unix)]
fn interrupted_run(signal: &str) {
    use std::{
        io::{BufRead, BufReader, Read},
        sync::mpsc,
        thread,
    };

    let files = Files::new();
    let directory = files.0.clone();
    files.write("input.json", &heavy_consolidation());
    let temporary = files.path("tmp");
    fs::create_dir(&temporary).unwrap();
    // A long deadline must not reclassify signal cancellation as timeout.
    for timeout in [false, true] {
        let mut command = files.command();
        command
            .args(files.args())
            .args(["--progress", "ndjson"])
            .env("TMPDIR", &temporary)
            .env("TMP", &temporary)
            .env("TEMP", &temporary);
        if timeout {
            command.args(["--timeout", "60"]);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stderr = BufReader::new(child.stderr.take().unwrap());
        let (ready, started) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut first = String::new();
            stderr.read_line(&mut first).unwrap();
            let _ = ready.send(first.clone());
            let mut bytes = first.into_bytes();
            stderr.read_to_end(&mut bytes).unwrap();
            bytes
        });
        write_child_stdin(&mut child, RUN);
        // Both handlers are installed before started. Wait for the heavy method
        // to begin rather than sending the signal during process startup.
        let first = match started.recv_timeout(Duration::from_secs(5)) {
            Ok(first) => first,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("No started event: {error}; stderr: {:?}", reader.join());
            }
        };
        assert_eq!(value(first.as_bytes())["phase"], "started");
        thread::sleep(Duration::from_millis(100));
        assert!(child.try_wait().unwrap().is_none());
        let sent = Command::new("kill")
            .arg(signal)
            .arg(child.id().to_string())
            .output()
            .unwrap();
        if !sent.status.success() {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Could not send {signal}: {sent:?}");
        }
        let start = Instant::now();
        while child.try_wait().unwrap().is_none() {
            if start.elapsed() >= Duration::from_secs(1) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("{signal} did not cancel computation within 1 s");
            }
            thread::sleep(Duration::from_millis(5));
        }
        let mut output = child.wait_with_output().unwrap();
        output.stderr = reader.join().unwrap();
        assert_code(&output, 5);
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr.last(), Some(&b'\n'));
        let lines: Vec<Value> = std::str::from_utf8(&output.stderr)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).expect("Complete JSON line"))
            .collect();
        let (envelope, events) = lines.split_last().unwrap();
        assert_eq!(events[0]["phase"], "started");
        for event in &events[1..] {
            assert!(progress::conforms(event));
            assert_eq!(event["index"], 0);
        }
        assert_eq!(envelope["schema"], "cli-error/1");
        assert_eq!(envelope["code"], "CLI-E-CANCELLED");
        assert_eq!(envelope["exitCode"], 5);
        assert_eq!(envelope["outcome"], "cancelled");
        assert_eq!(envelope["engineId"], "mamut-engine");
        assert_eq!(envelope["type"], "result");
        assert_eq!(envelope["operation"], "run");
        assert_eq!(envelope["error"]["code"], "CLI-E-CANCELLED");
        assert_eq!(envelope["error"]["message"], "Execution was cancelled.");
        assert!(envelope.get("data").is_none());
        // Match external cancellation: retain only the failed API document,
        // if produced, with no prior results or partial workflow manifest.
        if let Some(results) = envelope.get("engineResults") {
            let documents = results.as_array().unwrap();
            assert_eq!(documents.len(), 1);
            assert_eq!(documents[0]["schema"], "results/1");
            assert_eq!(documents[0]["status"], "cancelled");
        }
        assert_eq!(fs::read_dir(&temporary).unwrap().count(), 0);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 3);
    }
    drop(files);
    assert!(
        !directory.exists(),
        "Temporary directory was not cleaned up"
    );
}

#[cfg(unix)]
#[test]
fn sigint_cancels_heavy_consolidation_with_five() {
    interrupted_run("-INT");
}

#[cfg(unix)]
#[test]
fn sigterm_cancels_heavy_consolidation_with_five() {
    interrupted_run("-TERM");
}

#[test]
fn timeout_with_ndjson_emits_only_complete_json_lines_and_timeout_last() {
    let files = Files::new();
    files.write("input.json", &heavy_consolidation());
    let mut saw_started = false;
    // Incluye vencimientos durante el arranque y durante el método pesado.
    for from_env in [false, true] {
        for timeout in ["0.000000001", "0.0001", "0.001", "0.15"] {
            let output = timed_run(&files, timeout, from_env, true);
            assert_code(&output, 6);
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr.last(), Some(&b'\n'));
            let stderr = std::str::from_utf8(&output.stderr).unwrap();
            let lines: Vec<Value> = stderr
                .lines()
                .map(|line| {
                    serde_json::from_str(line).expect("Línea stderr debe ser JSON completo")
                })
                .collect();
            let (envelope, events) = lines.split_last().unwrap();
            let mut started = false;
            for event in events {
                if event["schema"] == "progress/1" {
                    assert!(started);
                    assert!(progress::conforms(event));
                    assert_eq!(event["index"], 0);
                } else {
                    assert_eq!(event["phase"], "started");
                    assert_eq!(event["method_id"], "engine.run");
                    assert!(!started);
                    started = true;
                    saw_started = true;
                }
            }
            assert_eq!(envelope["exitCode"], 6);
            assert_eq!(envelope["outcome"], "timeout");
            assert_eq!(envelope["code"], "CLI-E-TIMEOUT");
            assert_eq!(envelope["operation"], "run");
            assert_eq!(envelope["engineId"], "mamut-engine");
            assert!(envelope.get("engineResults").is_none());
        }
    }
    assert!(
        saw_started,
        "Debe comprobarse también un evento de progreso"
    );
}

#[test]
fn invalid_timeout_is_usage_and_flag_overrides_environment() {
    let files = Files::new();
    for timeout in [
        "",
        "0",
        "-1",
        "NaN",
        "inf",
        "-inf",
        "abc",
        "31536000.001",
        "1e300",
    ] {
        for from_env in [false, true] {
            let mut command = files.command();
            command.args(files.args());
            if from_env {
                command.env("MAMUT_ENGINE_TIMEOUT", timeout);
            } else {
                command.args(["--timeout", timeout]);
            }
            let output = command.output().unwrap();
            assert_code(&output, 2);
            assert!(output.stdout.is_empty());
            let envelope = value(&output.stderr);
            assert_eq!(envelope["outcome"], "usage");
            assert_eq!(envelope["exitCode"], 2);
            assert_eq!(envelope["code"], "CLI-E-USAGE");
            assert_eq!(envelope["operation"], "run");
            assert_eq!(envelope["engineId"], "mamut-engine");
        }
    }
    for extra in [vec!["--timeout"], vec!["--timeout", "1", "--timeout", "2"]] {
        let output = files
            .command()
            .args(files.args())
            .args(extra)
            .output()
            .unwrap();
        assert_code(&output, 2);
    }
    files.write("workflow.json", RUN);
    let baseline = files.run(RUN, false);
    for variable in ["0.000000001", "invalid", "31536000.001", "1e300"] {
        let output = files
            .command()
            .env("MAMUT_ENGINE_TIMEOUT", variable)
            .args(["--timeout", "60", "run", "--catalog"])
            .arg(files.path("catalog.json"))
            .arg("--workflow")
            .arg(files.path("workflow.json"))
            .output()
            .unwrap();
        assert_code(&output, 0);
        assert_eq!(output.stdout, baseline.stdout);
        assert_eq!(output.stderr, baseline.stderr);
    }
}

#[test]
fn leading_timeout_preserves_command_identity_on_usage_and_transport_failures() {
    let files = Files::new();
    for command in ["run", "validate", "methods", "compile", "mcp", "describe"] {
        let output = files
            .command()
            .args(["--timeout", "31536001", command])
            .output()
            .unwrap();
        assert_code(&output, 2);
        let envelope = value(&output.stderr);
        assert_eq!(envelope["operation"], command);
        assert_eq!(envelope["code"], "CLI-E-USAGE");
    }
    for command in ["run", "validate"] {
        let output = files
            .command()
            .args(["--timeout", "60", command, "--input"])
            .arg(files.path("missing-private-path.json"))
            .output()
            .unwrap();
        assert_code(&output, 2);
        let envelope = value(&output.stderr);
        assert_eq!(envelope["operation"], command);
        assert_eq!(envelope["code"], "CLI-E-DOCUMENT");
    }
}

#[test]
fn generous_timeout_preserves_every_output_byte_and_does_not_delay_exit() {
    let files = Files::new();
    let baseline = files.run(RUN, false);
    for (timeout, from_env) in [
        ("60", false),
        ("60", true),
        ("31536000", false),
        ("31536000", true),
    ] {
        let start = Instant::now();
        let output = timed_run(&files, timeout, from_env, false);
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_code(&output, 0);
        assert_eq!(output.stdout, baseline.stdout);
        assert_eq!(output.stderr, baseline.stderr);
    }
}

#[test]
fn timeout_is_not_extracted_from_a_base_flag_value() {
    let files = Files::new();
    files.write("workflow.json", RUN);
    for flag in ["--catalog", "--workflow", "--progress", "--input"] {
        let mut command = files.command();
        command.arg("run");
        for (option, argument) in [
            ("--catalog", files.path("catalog.json")),
            ("--workflow", files.path("workflow.json")),
            ("--progress", PathBuf::from("ndjson")),
            ("--input", files.path("input.json")),
        ] {
            command.arg(option);
            if option == flag {
                command.args(["--timeout", "5"]);
            }
            command.arg(argument);
        }
        let output = command.output().unwrap();
        assert_code(&output, 2);
        assert!(output.stdout.is_empty());
        let envelope = value(&output.stderr);
        assert_eq!(envelope["exitCode"], 2);
        assert_eq!(envelope["outcome"], "usage");
    }
}

fn with_input_stdin(files: &Files, args: &[&str], input: &[u8]) -> Output {
    let mut child = files
        .command()
        .current_dir(&files.0)
        .env_remove("MAMUT_ENGINE_INPUT")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    if let Err(error) = stdin.write_all(input) {
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }
    drop(stdin);
    child.wait_with_output().unwrap()
}

#[test]
fn timeout_and_input_share_embedded_documents_and_stdin() {
    let files = Files::new();
    for command in ["run", "validate"] {
        let baseline = with_input_stdin(&files, &[command, "--input", "-"], INPUT.as_bytes());
        assert_code(&baseline, 0);
        let stdin = with_input_stdin(
            &files,
            &["--timeout", "60", command, "--input", "-"],
            INPUT.as_bytes(),
        );
        let path = files
            .command()
            .args([command, "--input"])
            .arg(files.path("input.json"))
            .args(["--timeout", "60"])
            .output()
            .unwrap();
        for output in [stdin, path] {
            assert_code(&output, 0);
            assert_eq!(output.stdout, baseline.stdout);
            assert_eq!(output.stderr, baseline.stderr);
        }
    }

    let output = with_input_stdin(
        &files,
        &["run", "--input", "-", "--timeout", "0.001"],
        heavy_consolidation().as_bytes(),
    );
    assert_code(&output, 6);
    assert!(output.stdout.is_empty());
    let envelope = value(&output.stderr);
    assert_eq!(envelope["outcome"], "timeout");
    assert!(envelope.get("engineResults").is_none());
}

#[test]
fn input_stdin_and_path_match_legacy_bytes_with_embedded_and_explicit_documents() {
    let files = Files::new();
    files.write("workflow.json", RUN);
    let legacy = files.run(RUN, false);
    assert_code(&legacy, 0);
    let stdin = with_input_stdin(&files, &["run", "--input", "-"], INPUT.as_bytes());
    assert_code(&stdin, 0);
    assert_eq!(
        value(&stdin.stdout)["engineResults"][0]["schema"],
        "results/1"
    );
    assert_eq!(stdin.stdout, legacy.stdout);
    assert_eq!(stdin.stderr, legacy.stderr);
    for explicit in [false, true] {
        let mut command = files.command();
        command.current_dir(&files.0);
        // Una variable inválida comprueba que --input tiene prioridad.
        command.env("MAMUT_ENGINE_INPUT", files.path("missing.json"));
        command
            .args(["run", "--input"])
            .arg(files.path("input.json"));
        if explicit {
            command.arg("--catalog").arg(files.path("catalog.json"));
            command.arg("--workflow").arg(files.path("workflow.json"));
        }
        let output = command.output().unwrap();
        assert_code(&output, 0);
        assert_eq!(output.stdout, legacy.stdout);
        assert_eq!(output.stderr, legacy.stderr);
    }
    let workflow_stdin = with_input_stdin(
        &files,
        &[
            "run",
            "--input",
            files.path("input.json").to_str().unwrap(),
            "--catalog",
            files.path("catalog.json").to_str().unwrap(),
            "--workflow",
            "-",
        ],
        RUN.as_bytes(),
    );
    assert_code(&workflow_stdin, 0);
    assert_eq!(workflow_stdin.stdout, legacy.stdout);
    assert_eq!(workflow_stdin.stderr, legacy.stderr);

    files.write("-", INPUT);
    let literal_dash = files
        .command()
        .current_dir(&files.0)
        .env("MAMUT_ENGINE_INPUT", "-")
        .args(["run", "--catalog"])
        .arg(files.path("catalog.json"))
        .arg("--workflow")
        .arg(files.path("workflow.json"))
        .output()
        .unwrap();
    assert_code(&literal_dash, 0);
    assert_eq!(literal_dash.stdout, legacy.stdout);
    assert_eq!(literal_dash.stderr, legacy.stderr);
}

#[test]
fn input_stdin_e1_preserves_existing_warning_and_results_contract() {
    let files = Files::new();
    let fixture = value(include_bytes!(
        "../../../docs/validation-report/fixtures/val032/E1-v1.1.2.json"
    ));
    let input = fixture["runs"][0]["engine_input"].to_string();
    let output = with_input_stdin(&files, &["run", "--input", "-"], input.as_bytes());
    assert_code(&output, 1);
    let result = &value(&output.stdout)["engineResults"][0];
    assert_eq!(result["schema"], "results/1");
    assert_eq!(result["status"], "warning");
    assert!(result["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| { diagnostic["code"] == "API-W-DIAPHRAGM-RECOVERY" }));
}

#[test]
fn validate_input_e1_with_timeout_preserves_report_bytes() {
    let files = Files::new();
    let fixture = value(include_bytes!(
        "../../../docs/validation-report/fixtures/val032/E1-v1.1.2.json"
    ));
    files.write("e1.json", &fixture["runs"][0]["engine_input"].to_string());
    let mut command = files.command();
    command
        .args(["validate", "--input"])
        .arg(files.path("e1.json"));
    let baseline = command.output().unwrap();
    assert_code(&baseline, 0);
    assert_validate_report(&value(&baseline.stdout));
    assert!(baseline.stderr.is_empty());

    let timed = command.args(["--timeout", "60"]).output().unwrap();
    assert_code(&timed, 0);
    assert_eq!(timed.stdout, baseline.stdout);
    assert_eq!(timed.stderr, baseline.stderr);
}

#[test]
fn validate_input_reports_directly_and_preserves_explicit_workflow_validation() {
    let files = Files::new();
    let stdin = with_input_stdin(&files, &["validate", "--input", "-"], INPUT.as_bytes());
    assert_code(&stdin, 0);
    let report = value(&stdin.stdout);
    assert_eq!(report["schema"], "validate-report/1");
    assert_eq!(report["operation"], "validate");
    assert_eq!(report["status"], "ok");
    assert_eq!(report["diagnostics"], json!([]));
    assert!(stdin.stderr.is_empty());
    assert_validate_report(&report);
    let path = files
        .command()
        .args(["validate", "--input"])
        .arg(files.path("input.json"))
        .output()
        .unwrap();
    assert_code(&path, 0);
    assert_eq!(path.stdout, stdin.stdout);

    let invalid = with_input_stdin(&files, &["validate", "--input", "-"], b"{}");
    assert_code(&invalid, 2);
    assert_eq!(value(&invalid.stdout)["status"], "invalid");
    assert!(invalid.stderr.is_empty());
    assert_validate_report(&value(&invalid.stdout));
    files.write("workflow.json", RUN);
    let explicit = files
        .command()
        .args(["validate", "--input"])
        .arg(files.path("input.json"))
        .arg("--catalog")
        .arg(files.path("catalog.json"))
        .arg("--workflow")
        .arg(files.path("workflow.json"))
        .output()
        .unwrap();
    assert_code(&explicit, 0);
    assert_eq!(value(&explicit.stdout)["valid"], true);
    assert!(value(&explicit.stdout).get("engineResults").is_none());
    let previous = files
        .command()
        .args(["validate", "--catalog"])
        .arg(files.path("catalog.json"))
        .arg("--workflow")
        .arg(files.path("workflow.json"))
        .output()
        .unwrap();
    assert_eq!(explicit.stdout, previous.stdout);
    assert_eq!(explicit.stderr, previous.stderr);
}

#[test]
fn validate_model_errors_have_deterministic_reports_and_eng_diagnostics() {
    let files = Files::new();
    let mut wrong_schema = value(INPUT.as_bytes());
    wrong_schema["schema"] = json!("engine-input/2");
    let mut wrong_field = value(INPUT.as_bytes());
    wrong_field["units"] = json!("kN-mm");
    let mut too_many_nodes = value(INPUT.as_bytes());
    too_many_nodes["model"]["nodes"] = json!(vec![
        too_many_nodes["model"]["nodes"][0].clone();
        engine_api::MAX_NODES + 1
    ]);
    let mut bad_topology = value(INPUT.as_bytes());
    bad_topology["model"]["members"][0]["nodes"][1] = json!(99);
    for (input, expected, cause) in [
        ("{}".into(), "ENG-E-0009", "schema"),
        ("{".into(), "ENG-E-0009", ""),
        (wrong_schema.to_string(), "ENG-E-0009", "engine-input/2"),
        (wrong_field.to_string(), "ENG-E-0009", ""),
        (too_many_nodes.to_string(), "ENG-E-0009", "MAX_NODES"),
        (bad_topology.to_string(), "ENG-E-0003", ""),
    ] {
        let output = with_input_stdin(&files, &["validate", "--input", "-"], input.as_bytes());
        assert_code(&output, 2);
        assert!(output.stderr.is_empty());
        let report = value(&output.stdout); // Rechaza documentos JSON adicionales.
        assert_validate_report(&report);
        assert_eq!(report["status"], "invalid");
        let diagnostics = report["diagnostics"].as_array().unwrap();
        assert!(diagnostics
            .iter()
            .all(|d| d["code"].as_str().unwrap().starts_with("ENG-")));
        assert!(diagnostics
            .iter()
            .any(|d| d["code"] == expected && d["cause"].as_str().unwrap().contains(cause)));
        files.write("invalid.json", &input);
        let path = files
            .command()
            .args(["validate", "--input"])
            .arg(files.path("invalid.json"))
            .output()
            .unwrap();
        assert_code(&path, 2);
        assert_eq!(output.stdout, path.stdout);
        assert_eq!(output.stderr, path.stderr);
    }
}

#[test]
fn validate_does_not_solve_or_check_stability_and_rejects_workflow_flags() {
    let files = Files::new();
    let mut input = value(INPUT.as_bytes());
    input["model"]["supports"] = json!([]);
    let output = with_input_stdin(
        &files,
        &["validate", "--input", "-"],
        input.to_string().as_bytes(),
    );
    assert_code(&output, 0);
    assert_validate_report(&value(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_eq!(value(&output.stdout)["diagnostics"], json!([]));
    for suffix in [
        vec!["--progress", "ndjson"],
        vec!["--unknown"],
        vec!["--help"],
    ] {
        let output = files
            .command()
            .args(["validate", "--input"])
            .arg(files.path("input.json"))
            .args(suffix)
            .output()
            .unwrap();
        assert_code(&output, 2);
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn direct_validation_preserves_cooperative_cancellation() {
    let files = Files::new();
    files.write("cancel", "");
    let output = files
        .command()
        .env("MAMUT_ENGINE_CANCEL_FILE", files.path("cancel"))
        .args(["validate", "--input"])
        .arg(files.path("input.json"))
        .output()
        .unwrap();
    assert_code(&output, 5);
    assert!(output.stdout.is_empty());
    for boundary in 1..=3 {
        let control = Control::default();
        let cancel = control.clone();
        let cli = EngineCli::new(None)
            .with_control(control)
            .with_engine_progress(move |p| {
                if p.completed == boundary {
                    cancel.cancel();
                }
            });
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            cli.run(
                &["validate".into(), "--input".into(), "-".into()],
                &mut INPUT.as_bytes(),
                &mut stdout,
                &mut stderr
            ),
            5
        );
        assert!(stdout.is_empty());
        let envelope = error_schema::assert_error_stream(&stderr, 5);
        assert_eq!(envelope["operation"], "validate");
        assert_eq!(envelope["code"], "CLI-E-CANCELLED");
    }
}

#[test]
fn input_size_boundary_and_transport_errors_are_bounded_and_sanitized() {
    let files = Files::new();
    let mut input = INPUT.as_bytes().to_vec();
    for size in [engine_api::MAX_INPUT_BYTES - 1, engine_api::MAX_INPUT_BYTES] {
        input.resize(size, b' ');
        for command in ["run", "validate"] {
            let exact = with_input_stdin(&files, &[command, "--input", "-"], &input);
            assert_code(&exact, 0);
            assert!(!exact.stdout.is_empty());
            assert!(exact.stderr.is_empty());
        }
    }
    input.push(b' ');
    for command in ["run", "validate"] {
        for bytes in [input.as_slice(), &[0xff]] {
            let output = with_input_stdin(&files, &[command, "--input", "-"], bytes);
            assert_code(&output, 2);
            assert!(output.stdout.is_empty());
            let error = value(&output.stderr);
            assert_eq!(error["code"], "CLI-E-DOCUMENT");
            assert_eq!(error["operation"], command);
            assert!(error.get("engineResults").is_none());
            if command == "validate" {
                let previous = with_input_stdin(&files, &["run", "--input", "-"], bytes);
                let mut previous_error = value(&previous.stderr);
                previous_error["operation"] = json!("validate");
                assert_eq!(value(&output.stderr), previous_error);
            }
        }
        for (name, bytes) in [
            ("too-big.json", input.as_slice()),
            ("invalid-utf8.json", &[0xff][..]),
        ] {
            fs::write(files.path(name), bytes).unwrap();
            let output = files
                .command()
                .args([command, "--input"])
                .arg(files.path(name))
                .output()
                .unwrap();
            assert_code(&output, 2);
            assert!(output.stdout.is_empty());
            assert!(value(&output.stderr).get("engineResults").is_none());
        }
        let output = files
            .command()
            .args([command, "--input"])
            .arg(files.path("missing-private-path.json"))
            .output()
            .unwrap();
        assert_code(&output, 2);
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("missing-private-path"));
        if command == "validate" {
            let previous = files
                .command()
                .args(["run", "--input"])
                .arg(files.path("missing-private-path.json"))
                .output()
                .unwrap();
            let mut previous_error = value(&previous.stderr);
            previous_error["operation"] = json!("validate");
            assert_eq!(value(&output.stderr), previous_error);
        }
    }
}

#[test]
fn input_usage_errors_reject_stdin_conflicts_duplicates_and_missing_values() {
    let files = Files::new();
    for command in ["run", "validate"] {
        for suffix in [
            vec!["--input", "-", "--workflow", "-"],
            vec!["--catalog", "-", "--input", "-"],
            vec!["--input", "-", "--input", "-"],
            vec!["--input", "modelo.json", "--input", "otro.json"],
            vec!["--input"],
            vec!["--input", ""],
            vec!["--input", "--catalog", "catalog.json"],
            vec!["--input", "--workflow", "workflow.json"],
            vec!["--input", "--progress", "ndjson"],
            vec!["--input", "--help"],
            vec!["--input", "-h"],
            vec!["--input", "--unknown"],
            vec!["--input", "-x"],
            vec!["--"],
            vec!["--", "--input", "-"],
            vec!["--input", "--", "modelo.json"],
            vec!["--input", "-", "--"],
        ] {
            let output = files.command().arg(command).args(suffix).output().unwrap();
            assert_code(&output, 2);
            assert!(output.stdout.is_empty());
            assert!(value(&output.stderr).get("engineResults").is_none());
            if command == "run" {
                assert_eq!(value(&output.stderr)["outcome"], "usage");
            }
        }
    }
}

#[test]
fn equals_syntax_is_rejected_like_the_base_parser() {
    let files = Files::new();
    for command in ["run", "validate"] {
        for option in ["--input", "--catalog", "--workflow"] {
            for source in ["input.json", "-", ""] {
                let args = vec![command.to_string(), format!("{option}={source}")];
                assert!(mamut_workflow_cli::parse_args(&args).is_err());
                let output = files.command().args(&args).output().unwrap();
                assert_code(&output, 2);
                assert!(output.stdout.is_empty());
                if command == "run" {
                    assert_eq!(value(&output.stderr)["outcome"], "usage");
                }
            }
        }
    }
}

#[test]
fn embedded_documents_are_isolated_across_parallel_invocations() {
    let barrier = std::sync::Barrier::new(16);
    let outputs = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    let cli = EngineCli::new(None);
                    let args = ["run".into(), "--input".into(), "-".into()];
                    let mut stdout = Vec::new();
                    let mut stderr = Vec::new();
                    barrier.wait();
                    assert_eq!(
                        cli.run(&args, &mut INPUT.as_bytes(), &mut stdout, &mut stderr),
                        0
                    );
                    assert!(stderr.is_empty());
                    stdout
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        value(&outputs[0])["engineResults"][0]["schema"],
        "results/1"
    );
    assert!(outputs.iter().all(|output| output == &outputs[0]));
}

#[test]
fn input_stdin_is_reused_across_workflow_steps() {
    let files = Files::new();
    let mut workflow = value(RUN.as_bytes());
    workflow["steps"] = json!([
        value(VALIDATE.as_bytes())["steps"][0],
        value(RUN.as_bytes())["steps"][0]
    ]);
    let mut catalog = value(CATALOG.as_bytes());
    let slot = catalog["profile"]["slots"][0].clone();
    catalog["profile"]["slots"] = json!([slot, slot]);
    files.write("catalog.json", &catalog.to_string());
    files.write("workflow.json", &workflow.to_string());
    let output = with_input_stdin(
        &files,
        &[
            "run",
            "--input",
            "-",
            "--catalog",
            files.path("catalog.json").to_str().unwrap(),
            "--workflow",
            files.path("workflow.json").to_str().unwrap(),
        ],
        INPUT.as_bytes(),
    );
    assert_code(&output, 0);
    let envelope = value(&output.stdout);
    assert_eq!(envelope["engineResults"].as_array().unwrap().len(), 2);
    assert_eq!(envelope["engineResults"][0]["operation"], "validate");
    assert_eq!(envelope["engineResults"][1]["schema"], "results/1");

    for explicit_input in [false, true] {
        files.write("input.json", INPUT);
        let path = files.path("input.json");
        let mut removed = false;
        let cli = EngineCli::new(Some(path.clone())).with_engine_progress(move |_| {
            if !removed {
                fs::remove_file(&path).unwrap();
                removed = true;
            }
        });
        let mut args = files.args();
        if explicit_input {
            args.extend([
                "--input".into(),
                files.path("input.json").to_str().unwrap().into(),
            ]);
        }
        let mut stdout = Vec::new();
        assert_eq!(
            cli.run(
                &args,
                &mut workflow.to_string().as_bytes(),
                &mut stdout,
                &mut Vec::new()
            ),
            0
        );
        assert_eq!(value(&stdout)["engineResults"], envelope["engineResults"]);
        assert!(!files.path("input.json").exists());
    }
}

#[test]
fn describe_identifies_engine_and_publishes_engine_v1_table() {
    let output = Files::new().command().arg("describe").output().unwrap();
    assert_code(&output, 0);
    assert!(output.stderr.is_empty());
    let description = value(&output.stdout);
    assert_eq!(description["name"], "mamut-engine");
    assert_eq!(
        description["version"],
        engine_api::version()["engine_version"]
    );
    let codes = ExitCodes::engine_v1().capabilities();
    for key in [
        "exit_codes",
        "exit_code_categories",
        "reserved_exit_code_categories",
    ] {
        assert_eq!(description[key], codes[key]);
    }
    assert_eq!(
        description["exit_codes"],
        json!({"0":"ok","1":"warning","2":"invalid","3":"unsupported","4":"not_converged","5":"cancelled","6":"timeout","70":"internal"})
    );
    assert_eq!(
        description["reserved_exit_code_categories"],
        json!({"timeout":6})
    );
}

#[test]
fn stdin_workflow_ndjson_and_complete_api_result_preserve_manifest() {
    let files = Files::new();
    let output = files.run(RUN, true);
    assert_code(&output, 0);
    let envelope = value(&output.stdout);
    assert_eq!(envelope["engineId"], "mamut-engine");
    assert_eq!(
        envelope["engineVersion"],
        engine_api::version()["engine_version"]
    );
    assert_eq!(envelope["operation"], "run");
    assert_eq!(envelope["exitCode"], 0);
    let api = engine_api::run(INPUT, &engine_api::CancelToken::new(), |_| {});
    assert_eq!(envelope["engineResults"][0], value(api.document.as_bytes()));
    assert!(envelope["engineResults"][0]["cases"][0]["nodes"].is_array());
    // La extensión no modifica el manifiesto ni la huella calculada por la base.
    let mut base_stdout = Vec::new();
    let config = mamut_workflow_cli::CliConfig::new().with_provider(ApiProvider);
    assert_eq!(
        mamut_workflow_cli::run(
            &files.args(),
            &mut RUN.as_bytes(),
            &mut base_stdout,
            &mut Vec::new(),
            &config
        ),
        0
    );
    let mut expected = value(&base_stdout);
    assert_eq!(envelope["data"], expected["data"]);
    // Reconstruye los bytes del adaptador anterior a ENG-CONT-001 a partir del SDK.
    expected["engineId"] = json!("mamut-engine");
    expected["engineVersion"] = engine_api::version()["engine_version"].clone();
    expected["operation"] = json!("run");
    expected["engineResults"] = json!([value(api.document.as_bytes())]);
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
    let events: Vec<Value> = String::from_utf8(output.stderr)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let sdk_events: Vec<_> = events
        .iter()
        .filter(|event| event["schema"] != "progress/1")
        .collect();
    assert_eq!(sdk_events.len(), 2);
    for (event, phase) in sdk_events.iter().zip(["started", "finished"]) {
        assert_eq!(event["phase"], phase);
        assert_eq!(event["method_id"], "engine.run");
        assert_eq!(event["index"], 0);
    }
}

#[path = "support/progress.rs"]
mod progress;

struct ApiProvider;
impl mamut_workflow_cli::MethodProvider for ApiProvider {
    fn create(
        &self,
        _: &mamut_workflow::MethodDescriptor,
        _: &mamut_workflow::CompiledStep<mamut_workflow::RegisteredMethod>,
    ) -> Result<Box<dyn mamut_workflow_cli::Method>, String> {
        Ok(Box::new(ApiMethod))
    }
}
struct ApiMethod;
impl mamut_workflow_cli::Method for ApiMethod {
    fn run(&mut self) -> Result<Value, String> {
        serde_json::from_str(
            &engine_api::run(INPUT, &engine_api::CancelToken::new(), |_| {}).document,
        )
        .map_err(|e| e.to_string())
    }
}

#[test]
fn methods_reads_catalog_from_stdin_and_lists_engine_methods() {
    let files = Files::new();
    let mut child = files
        .command()
        .args(["methods", "--catalog", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    write_child_stdin(&mut child, CATALOG);
    let output = child.wait_with_output().unwrap();
    assert_code(&output, 0);
    let methods = value(&output.stdout);
    let ids: Vec<_> = methods
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["method_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["engine.run", "engine.validate"]);
}

#[test]
fn usage_document_workflow_and_engine_input_errors_exit_two() {
    let files = Files::new();
    for args in [
        vec!["version"],
        vec!["capabilities"],
        vec!["schema"],
        vec!["explain"],
        vec!["--version"],
        vec!["run", "--catalog", "missing.json"],
        vec!["run", "--catalog", "-", "--workflow", "-"],
    ] {
        let output = files.command().args(args).output().unwrap();
        assert_code(&output, 2);
        assert!(output.stdout.is_empty());
    }
    for workflow in ["{", &RUN.replace("mamut-engine.v1", "unknown")] {
        assert_code(&files.run(workflow, false), 2);
    }
    for input in ["{", r#"{"schema":"engine-input/2"}"#] {
        files.write("input.json", input);
        let output = files.run(RUN, false);
        assert_code(&output, 2);
        assert_eq!(value(&output.stderr)["outcome"], "invalid_input");
        assert!(output.stdout.is_empty());
    }
    fs::remove_file(files.path("input.json")).unwrap();
    assert_code(&files.run(RUN, false), 2);
    files.write("catalog.json", "{");
    assert_code(&files.run(RUN, false), 2);
}

#[test]
fn actual_engine_warning_unsupported_and_unstable_exit_one_three_four() {
    let files = Files::new();
    let original: Value = serde_json::from_str(INPUT).unwrap();
    let mut warning = original.clone();
    warning["combinations"] = json!([{"id":"scaled","terms":[{"case":"D","factor":2.0}]}]);
    let mut unsupported = original.clone();
    unsupported["model"]["members"][0]["element"] = json!("unknown");
    let mut unstable = original;
    unstable["model"]["supports"] = json!([]);
    for (input, code, outcome, status) in [
        (warning, 1, "warning", "warning"),
        (unsupported, 3, "unsupported", "unsupported"),
        (unstable, 4, "not_converged", "unstable"),
    ] {
        files.write("input.json", &input.to_string());
        let output = files.run(RUN, true);
        assert_code(&output, code);
        let envelope = if code == 1 {
            value(&output.stdout)
        } else {
            assert!(output.stdout.is_empty());
            let lines = String::from_utf8(output.stderr.clone()).unwrap();
            let lines: Vec<Value> = lines
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .filter(|event: &Value| event["schema"] != "progress/1")
                .collect();
            assert_eq!(lines.len(), 2); // started + fallo; sin finished.
            lines[1].clone()
        };
        assert_eq!(envelope["exitCode"], code);
        assert_eq!(envelope["outcome"], outcome);
        assert_eq!(envelope["engineResults"][0]["status"], status);
        assert!(!envelope["engineResults"][0]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}

#[test]
fn nonconverged_pdelta_exit_four() {
    let files = Files::new();
    let mut input: Value = serde_json::from_str(include_str!(
        "../../engine-api/tests/fixtures/determinism/pdelta.json"
    ))
    .unwrap();
    input["analyses"][0]["options"]["max_iterations"] = json!(1);
    input["analyses"][0]["options"]["tolerance"] = json!(1e-15);
    files.write("input.json", &input.to_string());
    let output = files.run(RUN, false);
    assert_code(&output, 4);
    assert_eq!(
        value(&output.stderr)["engineResults"][0]["status"],
        "notConverged"
    );
}

#[test]
fn engine_validate_is_a_method_while_standard_validate_only_compiles() {
    let files = Files::new();
    let output = files.run(VALIDATE, false);
    assert_code(&output, 0);
    assert_eq!(
        value(&output.stdout)["engineResults"][0]["diagnostics"],
        json!([])
    );
    let mut input: Value = serde_json::from_str(INPUT).unwrap();
    input["model"]["members"][0]["nodes"] = json!([1, 99]);
    files.write("input.json", &input.to_string());
    assert_code(&files.run(VALIDATE, false), 2);
    files.write("workflow.json", RUN);
    let output = files
        .command()
        .arg("validate")
        .arg("--catalog")
        .arg(files.path("catalog.json"))
        .arg("--workflow")
        .arg(files.path("workflow.json"))
        .output()
        .unwrap();
    assert_code(&output, 0);
    assert_eq!(value(&output.stdout)["valid"], true);
}

#[test]
fn cancellation_before_workflow_and_during_engine_stages_returns_five() {
    let files = Files::new();
    for via_token in [false, true] {
        let control = Control::default();
        if via_token {
            control.token().cancel();
        } else {
            control.flag().cancel();
        }
        let cli = EngineCli::new(None).with_control(control.clone());
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            cli.run(&files.args(), &mut RUN.as_bytes(), &mut stdout, &mut stderr),
            5
        );
        assert!(stdout.is_empty());
        assert_eq!(value(&stderr)["outcome"], "cancelled");
        assert_eq!(value(&stderr)["code"], "CLI-E-CANCELLED");
        assert!(control.flag().is_cancelled());
        assert!(control.token().is_cancelled());
    }
    for (validation, via_file) in [(false, false), (false, true), (true, false), (true, true)] {
        let control = Control::default();
        let control = if via_file {
            control.with_cancel_file(files.path("cancel"))
        } else {
            control
        };
        let callback_control = control.clone();
        let cancel_path = files.path("cancel");
        let stages = Rc::new(RefCell::new(Vec::new()));
        let stages_seen = Rc::clone(&stages);
        let cli = EngineCli::new(Some(files.path("input.json")))
            .with_control(control.clone())
            .with_engine_progress(move |p| {
                stages_seen.borrow_mut().push(p.stage);
                if via_file {
                    fs::write(&cancel_path, "").unwrap();
                } else {
                    callback_control.flag().cancel();
                }
            });
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let workflow = if validation { VALIDATE } else { RUN };
        assert_eq!(
            cli.run(
                &files.args(),
                &mut workflow.as_bytes(),
                &mut stdout,
                &mut stderr
            ),
            5
        );
        assert_eq!(*stages.borrow(), ["parse"]);
        assert!(control.token().is_cancelled());
        assert!(stdout.is_empty());
        assert_eq!(value(&stderr)["outcome"], "cancelled");
        assert_eq!(value(&stderr)["code"], "CLI-E-CANCELLED");
        if via_file {
            fs::remove_file(files.path("cancel")).unwrap();
            // Quitar el marcador no revoca una cancelación ya sincronizada.
            assert!(control.is_cancelled());
        }
    }
}

#[test]
fn multiple_steps_preserve_order_and_aggregate_warnings_without_reusing_state() {
    let files = Files::new();
    let mut input: Value = serde_json::from_str(INPUT).unwrap();
    input["combinations"] = json!([{"id":"scaled","terms":[{"case":"D","factor":2.0}]}]);
    files.write("input.json", &input.to_string());
    let mut workflow = value(RUN.as_bytes());
    workflow["steps"] = json!([
        value(VALIDATE.as_bytes())["steps"][0],
        value(RUN.as_bytes())["steps"][0]
    ]);
    let mut catalog = value(CATALOG.as_bytes());
    let slot = catalog["profile"]["slots"][0].clone();
    catalog["profile"]["slots"] = json!([slot, slot]);
    files.write("catalog.json", &catalog.to_string());
    let output = files.run(&workflow.to_string(), false);
    assert_code(&output, 1);
    let envelope = value(&output.stdout);
    let results = envelope["engineResults"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["operation"], "validate");
    assert_eq!(results[1]["schema"], "results/1");
    assert_eq!(envelope["outcome"], "warning");
    assert!(output.stderr.is_empty());

    let mut unstable = value(INPUT.as_bytes());
    unstable["model"]["supports"] = json!([]);
    files.write("input.json", &unstable.to_string());
    let output = files.run(&workflow.to_string(), false);
    assert_code(&output, 4);
    assert!(output.stdout.is_empty());
    let failure = value(&output.stderr);
    assert_eq!(failure["engineResults"].as_array().unwrap().len(), 1);
    assert_eq!(failure["engineResults"][0]["status"], "unstable");
    assert!(failure.get("data").is_none());

    files.write("catalog.json", CATALOG);
    let cli = EngineCli::new(Some(files.path("input.json")));
    for (input, code) in [(input.to_string(), 1), (INPUT.into(), 0)] {
        files.write("input.json", &input);
        let mut stdout = Vec::new();
        assert_eq!(
            cli.run(
                &files.args(),
                &mut RUN.as_bytes(),
                &mut stdout,
                &mut Vec::new()
            ),
            code
        );
        assert_eq!(value(&stdout)["engineResults"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn cancellation_marker_returns_five_from_binary_before_and_after_spawn() {
    let files = Files::new();
    for before_spawn in [true, false] {
        if before_spawn {
            files.write("cancel", "");
        }
        let mut child = files
            .command()
            .env("MAMUT_ENGINE_CANCEL_FILE", files.path("cancel"))
            // La cancelación debe preceder incluso al fallo de lectura de entrada.
            .env_remove("MAMUT_ENGINE_INPUT")
            .args(["run", "--catalog"])
            .arg(files.path("catalog.json"))
            .args(["--workflow", "-", "--progress", "ndjson"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // El proceso espera EOF del workflow: no depende de sleeps ni carreras.
        if !before_spawn {
            files.write("cancel", "");
        }
        write_child_stdin(&mut child, RUN);
        let output = child.wait_with_output().unwrap();
        assert_code(&output, 5);
        assert!(output.stdout.is_empty());
        let envelope = value(&output.stderr);
        assert_eq!(envelope["exitCode"], 5);
        assert_eq!(envelope["outcome"], "cancelled");
        assert_eq!(envelope["engineId"], "mamut-engine");
        assert!(envelope.get("engineResults").is_none());
        assert!(files.path("cancel").exists());
        fs::remove_file(files.path("cancel")).unwrap();
    }
}

#[test]
fn closed_stdout_pipe_returns_internal_seventy_from_binary() {
    let files = Files::new();
    let mut child = files
        .command()
        .args(["run", "--catalog"])
        .arg(files.path("catalog.json"))
        .args(["--workflow", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // El workflow por stdin impide escribir antes de cerrar el lector de stdout.
    drop(child.stdout.take());
    write_child_stdin(&mut child, RUN);
    let output = child.wait_with_output().unwrap();
    assert_code(&output, 70);
    assert_eq!(value(&output.stderr)["outcome"], "internal");
}

struct BrokenWriter;
impl Write for BrokenWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("private detail"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn progress_and_error_io_failures_return_seventy() {
    let files = Files::new();
    let mut args = files.args();
    args.extend(["--progress".into(), "ndjson".into()]);
    let cli = EngineCli::new(Some(files.path("input.json")));
    let mut stdout = Vec::new();
    assert_eq!(
        cli.run(&args, &mut RUN.as_bytes(), &mut stdout, &mut BrokenWriter),
        70
    );
    assert!(stdout.is_empty());
    assert_eq!(
        cli.run(
            &["unknown".into()],
            &mut io::empty(),
            &mut stdout,
            &mut BrokenWriter
        ),
        70
    );
}

#[test]
fn provider_rejects_unknown_methods_and_numeric_parameters_without_identity_fallback() {
    let files = Files::new();
    let mut catalog: Value = serde_json::from_str(CATALOG).unwrap();
    catalog["methods"][0]["method_id"] = json!("not-an-engine-method");
    files.write("catalog.json", &catalog.to_string());
    assert_code(
        &files.run(&RUN.replace("engine.run", "not-an-engine-method"), false),
        3,
    );
    catalog["methods"][0]["method_id"] = json!("engine.run");
    catalog["methods"][0]["parameters"] =
        json!([{"name":"n","required":true,"dimension":"1","unit":"1"}]);
    files.write("catalog.json", &catalog.to_string());
    let mut workflow: Value = serde_json::from_str(RUN).unwrap();
    workflow["steps"][0]["parameters"] = json!({"n":2.0});
    assert_code(&files.run(&workflow.to_string(), false), 2);
}

#[path = "../../engine-api/tests/support/sfc010.rs"]
mod sfc010;
#[test]
fn sfc010_cli_envelope_contains_identical_canonical_api_bytes_for_every_fixture() {
    let files = Files::new();
    for &(name, input) in sfc010::FIXTURES {
        files.write("input.json", input);
        let output = files.run(RUN, false);
        assert_code(&output, 0);
        let envelope = value(&output.stdout);
        let actual = engine_api::canonical_json(&envelope["engineResults"][0].to_string()).unwrap();
        let expected = engine_api::run(input, &engine_api::CancelToken::new(), |_| {});
        assert_eq!(actual.as_bytes(), expected.document.as_bytes(), "{name}");
    }
}

#[test]
fn sfc010_cli_rejects_experimental_methods_without_job_opt_in() {
    let files = Files::new();
    for &(name, source) in sfc010::FIXTURES {
        let mut input: Value = serde_json::from_str(source).unwrap();
        if input["options"]["allow_experimental"] != true {
            continue;
        }
        for options in [json!({}), json!({"allow_experimental": false})] {
            input["options"] = options;
            files.write("input.json", &input.to_string());
            let output = files.run(RUN, false);
            assert_code(&output, 2);
            assert!(output.stdout.is_empty());
            let envelope = value(&output.stderr);
            let document = &envelope["engineResults"][0];
            assert_eq!(document["status"], "invalid", "{name}: {envelope}");
            assert_eq!(document["diagnostics"][0]["code"], "API-E-INPUT");
            assert!(document
                .to_string()
                .contains("options.allow_experimental = true"));
        }
    }
}

#[test]
fn binary_failures_without_progress_have_exactly_one_error_line() {
    let files = Files::new();
    let output = files.command().arg("unknown").output().unwrap();
    assert_code(&output, 2);
    assert_eq!(value(&output.stderr)["code"], "CLI-E-USAGE");
    assert_eq!(value(&output.stderr)["operation"], "cli");
    let cases = [
        ("{private-invalid-json".into(), 2, "API-E-INPUT"),
        (
            {
                let mut input = value(INPUT.as_bytes());
                input["model"]["members"][0]["element"] = json!("unknown");
                input.to_string()
            },
            3,
            "ENG-E-0013",
        ),
        (
            {
                let mut input = value(INPUT.as_bytes());
                input["model"]["supports"] = json!([]);
                input.to_string()
            },
            4,
            "ENG-E-0006",
        ),
    ];
    for (input, exit_code, code) in cases {
        files.write("input.json", &input);
        let output = files.run(RUN, false);
        assert_code(&output, exit_code);
        let envelope = value(&output.stderr); // falla también si hay varias líneas
        assert_eq!(envelope["code"], code);
        assert_eq!(envelope["operation"], "run");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-invalid-json"));
    }
    for command in ["methods", "mcp"] {
        let output = files
            .command()
            .args([command, "--catalog"])
            .arg(files.path("private-missing.json"))
            .output()
            .unwrap();
        assert_code(&output, 2);
        let envelope = value(&output.stderr);
        assert_eq!(envelope["code"], "CLI-E-DOCUMENT");
        assert_eq!(envelope["operation"], command);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-missing.json"));
    }
}

#[test]
fn mcp_sdk_protocol_errors_stay_in_rpc_but_terminal_transport_errors_use_cli_error() {
    let files = Files::new();
    for (input, exit_code) in [("{\n", 0), ("Content-Length: 10\r\n\r\nx", 70)] {
        let mut child = files
            .command()
            .args(["mcp", "--catalog"])
            .arg(files.path("catalog.json"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        write_child_stdin(&mut child, input);
        let output = child.wait_with_output().unwrap();
        assert_code(&output, exit_code);
        if exit_code == 0 {
            assert!(output.stderr.is_empty());
            assert_eq!(value(&output.stdout)["error"]["code"], -32700);
        } else {
            assert_eq!(value(&output.stderr)["operation"], "mcp");
        }
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_a_sanitized_usage_error() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let output = Files::new()
        .command()
        .arg(OsString::from_vec(vec![0xff]))
        .output()
        .unwrap();
    assert_code(&output, 2);
    assert_eq!(value(&output.stderr)["code"], "CLI-E-USAGE");
}
