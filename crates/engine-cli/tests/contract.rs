// SPDX-License-Identifier: AGPL-3.0-only
//! ENG-CONT-002: actual stdout, published schema, and byte golden, without oracles.

#[path = "../../engine-api/tests/support/schema.rs"]
mod schema;

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

const INPUT: &str = include_str!("../../engine-api/tests/fixtures/determinism/cantilever.json");
const RUN: &str = include_str!("../run.workflow.json");
const VALIDATE: &str = include_str!("../validate.workflow.json");
const SCHEMA: &str = include_str!("../schemas/cli-run-1.json");
const GOLDEN: &str = "tests/golden/stdout-sha256.json";

fn value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}
fn hash(value: &Value) -> String {
    // Public JCS/SHA-256 API, without running physics or implementing another hash.
    engine_api::input_sha256(&value.to_string()).unwrap()
}

// MAM-836: fingerprints cover codes and data, never message text. The
// projection removes only `cause` and `hint` from each diagnostic and is computed here
// independently of the adapter; the rest of the result remains intact.
fn fingerprint_view(result: &Value) -> Value {
    let mut view = result.clone();
    if let Some(diagnostics) = view.get_mut("diagnostics").and_then(Value::as_array_mut) {
        for diagnostic in diagnostics {
            let fields = diagnostic.as_object_mut().unwrap();
            fields.remove("cause");
            fields.remove("hint");
        }
    }
    view
}

fn fixtures() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut fixtures = Vec::new();
    for (group, directory) in [
        ("determinism", "../engine-api/tests/fixtures/determinism"),
        ("input", "../engine-api/fixtures/input/valid"),
    ] {
        let mut paths: Vec<_> = fs::read_dir(root.join(directory))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        assert!(!paths.is_empty(), "empty fixture group: {group}");
        for path in paths {
            let source = fs::read_to_string(&path).unwrap();
            assert_eq!(value(source.as_bytes())["schema"], "engine-input/1");
            fixtures.push((
                format!("{group}-{}", path.file_stem().unwrap().to_str().unwrap()),
                source,
            ));
        }
    }
    // Only engine_input is extracted: no numerical expectations or external references.
    let e1: Value = value(
        &fs::read(root.join("../../docs/validation-report/fixtures/val032/E1-v1.1.2.json"))
            .unwrap(),
    );
    for run in e1["runs"].as_array().unwrap() {
        assert_eq!(run["engine_input"]["schema"], "engine-input/1");
        fixtures.push((
            run["id"].as_str().unwrap().to_owned(),
            run["engine_input"].to_string(),
        ));
    }
    let mut warning = value(INPUT.as_bytes());
    warning["combinations"] = json!([{"id":"scaled","terms":[{"case":"D","factor":2.0}]}]);
    fixtures.push(("warning-combination".into(), warning.to_string()));
    fixtures
}

fn binary(input: &Path, workflow: &str, catalog: &Path) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mamut-engine"))
        .env_remove("MAMUT_ENGINE_CANCEL_FILE")
        .env("MAMUT_ENGINE_INPUT", input)
        .args(["run", "--catalog"])
        .arg(catalog)
        .args(["--workflow", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(workflow.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

// stdout is carried into the manifest provenance without reserializing it. First,
// metadata is checked against the compiled engine-api and all preimages are checked.
// Only target/git_commit and their dependent fingerprints change; the rest are exact bytes.
fn materialize(stdout: &str, compiled: &Value, reference: &Value) -> String {
    let original = value(stdout.as_bytes());
    let mut expected = original.clone();
    let results = expected["engineResults"].as_array_mut().unwrap();
    let mut replacements = Vec::new();
    for result in results.iter_mut().filter(|r| r["schema"] == "results/1") {
        for key in ["target", "git_commit"] {
            assert_eq!(result.get(key), compiled.get(key), "metadata: {key}");
            replacements.push((
                format!("\"{key}\":{}", result[key]),
                format!("\"{key}\":{}", reference[key]),
            ));
            result[key] = reference[key].clone();
        }
    }
    for index in 0..results.len() {
        let old = &original["data"]["steps"][index]["outputs"][0]["fingerprint"]["value"];
        assert_eq!(
            old,
            &hash(&fingerprint_view(&original["engineResults"][index]))
        );
        let new = hash(&fingerprint_view(&expected["engineResults"][index]));
        replacements.push((old.to_string(), json!(new).to_string()));
        expected["data"]["steps"][index]["outputs"][0]["fingerprint"]["value"] = json!(new);
    }
    let old = &original["data"]["recordFingerprint"]["value"];
    let mut manifest = original["data"].clone();
    manifest
        .as_object_mut()
        .unwrap()
        .remove("recordFingerprint");
    assert_eq!(old, &hash(&manifest));
    expected["data"]
        .as_object_mut()
        .unwrap()
        .remove("recordFingerprint");
    replacements.push((old.to_string(), json!(hash(&expected["data"])).to_string()));
    let mut bytes = stdout.to_owned();
    for (old, new) in replacements {
        assert!(bytes.contains(&old), "missing stdout token: {old}");
        bytes = bytes.replace(&old, &new);
    }
    bytes
}

fn assert_lf(bytes: &[u8]) {
    assert_eq!(bytes.last(), Some(&b'\n'), "stdout must end with LF");
    assert_eq!(bytes.iter().filter(|&&b| b == b'\n').count(), 1);
    assert!(!bytes.contains(&b'\r'), "stdout does not allow CRLF");
}

// hashlib computes SHA-256 of raw bytes, including LF; never JCS. Python 3 is already
// part of the parity environment. No Cargo dependencies change.
fn stdout_sha256(bytes: &[u8]) -> String {
    let mut child = Command::new(if cfg!(windows) { "python" } else { "python3" })
        .args([
            "-c",
            "import hashlib, sys; print(hashlib.sha256(sys.stdin.buffer.read()).hexdigest())",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("SHA-256 of stdout requires Python 3 (see README)");
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    let digest = String::from_utf8(output.stdout).unwrap();
    let digest = digest.trim();
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|b| b.is_ascii_hexdigit()));
    digest.to_owned()
}

fn dump_failure(name: &str, stdout: &[u8], comparable: &[u8]) {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("cli-contract-failure-{}", std::process::id()));
    let save = || -> std::io::Result<()> {
        fs::create_dir_all(&directory)?;
        fs::write(directory.join(format!("{name}.stdout.json")), stdout)?;
        fs::write(directory.join(format!("{name}.reference.json")), comparable)
    };
    match save() {
        Ok(()) => eprintln!("stdout de diagnóstico: {}", directory.display()),
        Err(error) => eprintln!("No se pudo guardar stdout de diagnóstico: {error}"),
    }
}

struct Temp(std::path::PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn all_accepted_fixtures_match_schema_and_stdout_golden() {
    let regenerate = match std::env::var("MAMUT_UPDATE_CLI_GOLDENS") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "1" => {
            assert!(
                std::env::var_os("CI").is_none() && std::env::var_os("GITHUB_ACTIONS").is_none(),
                "Golden regeneration is prohibited in CI"
            );
            true
        }
        _ => panic!("MAMUT_UPDATE_CLI_GOLDENS only allows 1 or absence"),
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let temp = Temp(
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-contract-{}", std::process::id())),
    );
    fs::create_dir_all(&temp.0).unwrap();
    let input_path = temp.0.join("input.json");
    let catalog_path = temp.0.join("catalog.json");
    fs::write(&catalog_path, engine_cli::CATALOG).unwrap();
    let spec = value(SCHEMA.as_bytes());
    // Metadata independent of the stdout under test, from the same compiled engine-api.
    let metadata = value(
        engine_api::run(INPUT, &engine_api::CancelToken::new(), |_| {})
            .document
            .as_bytes(),
    );
    let manifest_path = root.join(GOLDEN);
    let mut manifest = if regenerate {
        json!({"version":1, "buildMetadata": {
            "target":metadata["target"], "git_commit":metadata["git_commit"]
        }, "cases":[]})
    } else {
        value(&fs::read(&manifest_path).unwrap_or_else(|error| {
            panic!(
                "{}: {error}; see README to regenerate",
                manifest_path.display()
            )
        }))
    };
    assert_eq!(manifest["version"], 1, "unsupported manifest version");
    assert_eq!(manifest["buildMetadata"].as_object().unwrap().len(), 2);
    assert!(manifest["buildMetadata"]["target"].is_string());
    assert!(manifest["buildMetadata"].get("git_commit").is_some());
    let mut codes = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (name, input) in fixtures() {
        fs::write(&input_path, &input).unwrap();
        for (operation, workflow) in [("run", RUN), ("validate", VALIDATE)] {
            let output = binary(&input_path, workflow, &catalog_path);
            let code = output.status.code().unwrap();
            assert!(
                matches!(code, 0 | 1),
                "{name}/{operation}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty(), "{name}/{operation}");
            assert_lf(&output.stdout);
            let document = value(&output.stdout);
            assert!(
                schema::conforms(&document, &spec),
                "{name}/{operation}: outside cli-run/1"
            );
            assert_eq!(document["exitCode"], code);
            assert_eq!(
                document["outcome"],
                if code == 0 { "ok" } else { "warning" }
            );
            assert_eq!(
                document["engineResults"].as_array().unwrap().len(),
                document["data"]["steps"].as_array().unwrap().len()
            );
            codes.insert(code);
            let name = format!("{name}-{operation}");
            assert!(names.insert(name.clone()));
            let comparable = materialize(
                std::str::from_utf8(&output.stdout).unwrap(),
                &metadata,
                &manifest["buildMetadata"],
            );
            assert_lf(comparable.as_bytes());
            let digest = stdout_sha256(comparable.as_bytes());
            let actual = json!({"name":name, "bytes":comparable.len(), "sha256":digest});
            if regenerate {
                manifest["cases"].as_array_mut().unwrap().push(actual);
            } else {
                let expected = manifest["cases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|entry| entry["name"] == name)
                    .unwrap_or_else(|| panic!("missing case: {name}; see README to regenerate"));
                if expected != &actual {
                    dump_failure(&name, &output.stdout, comparable.as_bytes());
                    panic!(
                        "{name}: expected SHA-256 {}, got {}; expected bytes {}, got {}",
                        expected["sha256"],
                        digest,
                        expected["bytes"],
                        comparable.len()
                    );
                }
            }
            // Actual repeat: covers complete bytes, including provenance and fingerprints.
            let repeated = binary(&input_path, workflow, &catalog_path);
            assert_eq!(repeated.status.code(), Some(code));
            assert_eq!(repeated.stdout, output.stdout, "{name}: nondeterministic");
        }
    }
    assert_eq!(codes, [0, 1].into_iter().collect());
    if regenerate {
        manifest["cases"]
            .as_array_mut()
            .unwrap()
            .sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        let mut bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        bytes.push(b'\n');
        fs::write(&manifest_path, bytes).unwrap();
    }
    let entries = manifest["cases"].as_array().unwrap();
    let stored: BTreeSet<_> = entries
        .iter()
        .map(|entry| entry["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(stored.len(), entries.len(), "duplicate cases in manifest");
    assert_eq!(names, stored, "missing or obsolete golden");
    assert_eq!(
        fs::read_dir(root.join("tests/golden")).unwrap().count(),
        1,
        "only stdout-sha256.json is versioned"
    );
}

#[test]
fn schema_rejects_breaking_changes_and_accepts_sdk_extensions() {
    let original = value(&sample_stdout());
    let spec = value(SCHEMA.as_bytes());
    assert!(schema::conforms(&original, &spec));
    for key in ["data", "engineResults", "engineId", "exitCode"] {
        let mut bad = original.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(!schema::conforms(&bad, &spec), "required field: {key}");
    }
    for (pointer, replacement) in [
        ("/exitCode", json!(1)),
        ("/outcome", json!("unsupported")),
        ("/engineId", json!("other")),
        ("/engineResults", json!({})),
        ("/engineResults/0/schema", json!("results/2")),
        ("/engineResults/0/status", json!("invalid")),
        ("/engineResults/0/cases/0/nodes/0/displacements", json!([0])),
        ("/data/recordFingerprint", json!("bad-type")),
        ("/data/steps/0/outputs/0/fingerprint", json!(null)),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            !schema::conforms(&bad, &spec),
            "mutation accepted: {pointer}"
        );
    }
    let mut extended = original.clone();
    extended["future"] = json!(true);
    assert!(!schema::conforms(&extended, &spec));
    extended.as_object_mut().unwrap().remove("future");
    extended["data"]["sdkExtension"] = json!(true);
    assert!(schema::conforms(&extended, &spec));
    extended["data"]["recordFingerprint"] = json!({"sdkExtension":true});
    extended["data"]["steps"][0]["outputs"][0]["fingerprint"] = json!({});
    assert!(schema::conforms(&extended, &spec));
    let mut validate = original;
    validate["engineResults"] = json!([{"operation":"validate", "status":"warning", "diagnostics":[{
        "code":"API-W-TEST", "severity":"warning", "entities":[], "cause":"fixture de contrato", "hint":null
    }]}]);
    assert!(!schema::conforms(&validate, &spec));
    validate["outcome"] = json!("warning");
    validate["exitCode"] = json!(1);
    assert!(schema::conforms(&validate, &spec));
    validate["engineResults"][0]["diagnostics"][0]["severity"] = json!("unknown");
    assert!(!schema::conforms(&validate, &spec));
}

fn sample_stdout() -> Vec<u8> {
    let temp = Temp(Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "cli-contract-sample-{:?}",
        std::thread::current().id()
    )));
    fs::create_dir_all(&temp.0).unwrap();
    let input = temp.0.join("input.json");
    let catalog = temp.0.join("catalog.json");
    fs::write(&input, INPUT).unwrap();
    fs::write(&catalog, engine_cli::CATALOG).unwrap();
    let output = binary(&input, RUN, &catalog);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_lf(&output.stdout);
    output.stdout
}

#[test]
fn golden_materialization_preserves_bytes_across_build_metadata() {
    let stdout = String::from_utf8(sample_stdout()).unwrap();
    let original = value(stdout.as_bytes());
    let compiled = &original["engineResults"][0];
    for target in [
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
    ] {
        let metadata =
            json!({"target":target, "git_commit":"0123456789abcdef0123456789abcdef01234567"});
        let expected = materialize(&stdout, compiled, &metadata);
        assert_lf(expected.as_bytes());
        let document = value(expected.as_bytes());
        assert_eq!(document["engineResults"][0]["target"], target);
        assert_eq!(
            document["engineResults"][0]["git_commit"],
            metadata["git_commit"]
        );
        assert_eq!(materialize(&expected, &metadata, compiled), stdout);
    }
}
