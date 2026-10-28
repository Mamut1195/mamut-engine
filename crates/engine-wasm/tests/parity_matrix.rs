// SPDX-License-Identifier: AGPL-3.0-only
//! ENG-PY-006: single analytical matrix; see provenance/ENG-PY-006.md.
//! Hosted here because engine-api has no WASM/CLI dependency. This crate already
//! has API + WASM access; the real CLI runs via offline Cargo without new deps.
//! WASM means the native Rust facade call, as in sfc010.rs, not a JS runtime.

use serde::Deserialize;
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

#[derive(Deserialize)]
struct Matrix {
    schema: String,
    tolerance_factor: f64,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    input: String,
    status: String,
    cli_exit: i32,
    diagnostic: Option<String>,
    free_dofs: u32,
    condition_bound: Option<f64>,
    checks: Vec<Check>,
}

#[derive(Deserialize)]
struct Check {
    pointer: String,
    fraction: [f64; 2],
    scale: f64,
    unit: String,
}

fn cli(input: &str) -> Output {
    // Cargo releases its build lock before running an integration test. Let it
    // locate/build the sibling binary, including when CARGO_TARGET_DIR is set.
    let mut child = Command::new(env!("CARGO"))
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args([
            "run",
            "--offline",
            "--quiet",
            "-p",
            "engine-cli",
            "--bin",
            "mamut-engine",
            "--",
            "run",
            "--input",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("offline Cargo must be available to execute the real CLI");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn jcs_sha256(document: &str) -> String {
    // Existing RFC 8785 / SHA-256 contract, valid for any JSON value.
    engine_api::input_sha256(document).unwrap()
}

fn check_document(document: &str, input: &str, case: &Case, factor: f64) {
    let value: Value = serde_json::from_str(document).unwrap();
    let source: Value = serde_json::from_str(input).unwrap();
    // results/1 defines SI implicitly; it has no units/sign_convention fields.
    // DOF indices and signed physical quantities below check that convention.
    assert_eq!(source["units"], "SI");
    assert_eq!(source["model"]["axes"], "Z-up");
    assert_eq!(value["schema"], "results/1");
    assert_eq!(value["status"], case.status, "{}", case.input);
    assert_eq!(value["input_sha256"], jcs_sha256(input));
    assert_eq!(engine_api::canonical_json(document).unwrap(), document);
    if let Some(code) = &case.diagnostic {
        let diagnostic = value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["severity"] == "error")
            .expect("typed error diagnostic");
        assert_eq!(diagnostic["code"], *code);
        assert!(!diagnostic["entities"].as_array().unwrap().is_empty());
        assert_eq!(value["cases"], serde_json::json!([]));
        assert!(case.checks.is_empty());
    } else {
        assert_eq!(value["diagnostics"], serde_json::json!([]));
        assert_eq!(
            value["nodes"].as_array().unwrap().len(),
            source["model"]["nodes"].as_array().unwrap().len()
        );
        for (actual, expected) in value["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .zip(source["model"]["nodes"].as_array().unwrap())
        {
            assert_eq!(actual["id"], expected["id"]);
            assert_eq!(actual["dofs"], expected["dofs"]);
            assert_eq!(actual["dim"], "D2");
        }
        assert!(!case.checks.is_empty());
        let tolerance =
            factor * f64::EPSILON * f64::from(case.free_dofs) * case.condition_bound.unwrap();
        for check in &case.checks {
            let actual = value.pointer(&check.pointer).unwrap().as_f64().unwrap();
            let expected = check.fraction[0] / check.fraction[1];
            let bound = tolerance * expected.abs().max(check.scale);
            assert!(
                actual.is_finite() && (actual - expected).abs() <= bound,
                "{} {}: {actual} != {expected} {}, tolerance {bound}",
                case.input,
                check.pointer,
                check.unit
            );
        }
    }
}

#[test]
fn api_cli_wasm_share_analytical_results_hashes_and_typed_errors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine-api/tests/fixtures/parity");
    let matrix: Matrix =
        serde_json::from_str(&fs::read_to_string(root.join("matrix.json")).unwrap()).unwrap();
    assert_eq!(matrix.schema, "parity-matrix/1");
    assert_eq!(matrix.cases.len(), 3);
    let references = std::env::var_os("MAMUT_PARITY_REFERENCES");
    if let Some(directory) = &references {
        fs::create_dir_all(directory).unwrap();
    }
    for case in &matrix.cases {
        let input = fs::read_to_string(root.join(&case.input)).unwrap();
        let api = engine_api::run(&input, &engine_api::CancelToken::new(), |_| {});
        assert_eq!(
            api.status,
            if case.diagnostic.is_some() {
                engine_api::Status::Unstable
            } else {
                engine_api::Status::Ok
            }
        );
        let wasm = engine_wasm::run(&input, &engine_wasm::CancelToken::new(), None, None).unwrap();
        let output = cli(&input);
        assert_eq!(
            output.status.code(),
            Some(case.cli_exit),
            "{}: {}",
            case.input,
            String::from_utf8_lossy(&output.stderr)
        );
        let stream = if case.diagnostic.is_some() {
            assert!(output.stdout.is_empty());
            &output.stderr
        } else {
            assert!(output.stderr.is_empty());
            &output.stdout
        };
        assert_eq!(stream.last(), Some(&b'\n'));
        assert_eq!(stream.iter().filter(|&&b| b == b'\n').count(), 1);
        let envelope: Value = serde_json::from_slice(stream).unwrap();
        assert_eq!(envelope["exitCode"], case.cli_exit);
        let results = envelope["engineResults"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        // CLI has its own envelope and LF; compare its canonical results/1 payload.
        let cli_document = engine_api::canonical_json(&results[0].to_string()).unwrap();
        for document in [&api.document, &wasm, &cli_document] {
            check_document(document, &input, case, matrix.tolerance_factor);
            assert_eq!(
                document.as_bytes(),
                api.document.as_bytes(),
                "{}",
                case.input
            );
            assert_eq!(jcs_sha256(document), jcs_sha256(&api.document));
        }
        if let Some(code) = &case.diagnostic {
            assert_eq!(envelope["schema"], "cli-error/1");
            assert_eq!(envelope["outcome"], "not_converged");
            assert_eq!(envelope["code"], *code);
            assert_eq!(envelope["diagnostics"], results[0]["diagnostics"]);
        } else {
            assert_eq!(envelope["type"], "result");
            assert_eq!(envelope["operation"], "run");
            assert_eq!(envelope["outcome"], "ok");
            // The existing CLI contract fingerprints results without cause/hint.
            // Successful cases have no diagnostics, so that projection is identity.
            assert_eq!(
                envelope["data"]["steps"][0]["outputs"][0]["fingerprint"]["value"],
                jcs_sha256(&api.document)
            );
            let mut manifest = envelope["data"].clone();
            let fingerprint = manifest
                .as_object_mut()
                .unwrap()
                .remove("recordFingerprint")
                .unwrap();
            assert_eq!(fingerprint["value"], jcs_sha256(&manifest.to_string()));
        }
        // Optional independent Rust references for the installed Python binding;
        // never generate expected output through the Python surface under test.
        if let Some(directory) = &references {
            fs::write(Path::new(directory).join(&case.input), &api.document).unwrap();
            fs::write(
                Path::new(directory).join(format!("{}.jcs-sha256", case.input)),
                jcs_sha256(&api.document),
            )
            .unwrap();
        }
    }
}
