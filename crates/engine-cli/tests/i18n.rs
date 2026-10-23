use engine_api::{Lang, Status};
use serde_json::{json, Value};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

const INVALID: &str =
    include_str!("../../engine-api/fixtures/input/invalid/missing-reference.json");
const VALID: &str = include_str!("../../engine-api/tests/fixtures/determinism/cantilever.json");

fn invoke(args: &[&str], input: &str, locale: Option<(&str, &str)>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mamut-engine"));
    for name in [
        "MAMUT_ENGINE_INPUT",
        "MAMUT_ENGINE_TIMEOUT",
        "MAMUT_ENGINE_CANCEL_FILE",
        "LANG",
        "LC_ALL",
    ] {
        command.env_remove(name);
    }
    if let Some((name, value)) = locale {
        command.env(name, value);
    }
    let mut child = command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

#[test]
fn run_errors_and_validation_use_the_shared_catalog_and_preserve_english_bytes() {
    let diagnostics = engine_api::validate(INVALID).unwrap();
    for lang in Lang::ALL {
        let expected = json!(engine_api::localize_diagnostics(diagnostics.clone(), lang));
        let validation = invoke(
            &["--lang", lang.code(), "validate", "--input", "-"],
            INVALID,
            None,
        );
        assert_eq!(validation.status.code(), Some(2));
        assert!(validation.stderr.is_empty());
        assert_eq!(value(&validation.stdout)["diagnostics"], expected);
        let run = invoke(
            &["run", "--input", "-", "--lang", lang.code()],
            INVALID,
            None,
        );
        assert_eq!(run.status.code(), Some(2));
        assert!(run.stdout.is_empty());
        let error = value(&run.stderr);
        assert_eq!(error["diagnostics"], expected);
        let expected_document =
            engine_api::run(INVALID, &engine_api::CancelToken::new(), |_| {}).localized(lang);
        assert_eq!(expected_document.status, Status::Invalid);
        assert_eq!(
            error["engineResults"][0],
            value(expected_document.document.as_bytes())
        );
        let key = error["messageKey"].as_str().unwrap();
        assert_eq!(
            error["error"]["message"],
            engine_api::message(lang, key).unwrap()
        );
        if lang == Lang::En {
            for (command, output) in [("validate", validation), ("run", run)] {
                let default = invoke(&[command, "--input", "-"], INVALID, None);
                assert_eq!(output.stdout, default.stdout);
                assert_eq!(output.stderr, default.stderr);
            }
        } else {
            assert_ne!(expected, json!(diagnostics));
        }
    }
}

#[test]
fn valid_run_and_manifest_with_a_real_warning_are_language_independent() {
    // A successful warning run emits a sealed manifest. Invalid models return
    // cli-error/1 instead. This API warning is specific text left in English
    // in phase 1; the unit test separately proves all message text is excluded.
    let mut input: Value = serde_json::from_str(VALID).unwrap();
    input["combinations"] = json!([{"id":"scaled","terms":[{"case":"D","factor":2.0}]}]);
    let input = input.to_string();
    let default = invoke(&["run", "--input", "-"], &input, None);
    assert_eq!(default.status.code(), Some(1));
    let baseline = value(&default.stdout);
    assert!(!baseline["engineResults"][0]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
    let fingerprints = |value: &Value| {
        fn collect(value: &Value, output: &mut Vec<String>) {
            match value {
                Value::Object(fields) => {
                    for (key, value) in fields {
                        if key == "recordFingerprint" {
                            output.push(value.to_string());
                        }
                        collect(value, output);
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        collect(value, output);
                    }
                }
                _ => {}
            }
        }
        let mut result = Vec::new();
        collect(value, &mut result);
        result
    };
    let expected = fingerprints(&baseline);
    assert!(!expected.is_empty());
    for lang in Lang::ALL {
        let output = invoke(
            &["run", "--lang", lang.code(), "--input", "-"],
            &input,
            None,
        );
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        assert_eq!(fingerprints(&value(&output.stdout)), expected);
        assert_eq!(output.stdout, default.stdout);
    }
}

#[test]
fn explicit_language_ignores_host_locale_for_run_validate_and_errors() {
    for lang in Lang::ALL {
        for input in [VALID, INVALID] {
            for operation in ["run", "validate"] {
                let args = [operation, "--lang", lang.code(), "--input", "-"];
                let baseline = invoke(&args, input, None);
                for locale in [
                    ("LANG", "C"),
                    ("LANG", "es_EC.UTF-8"),
                    ("LC_ALL", "pt_BR.UTF-8"),
                ] {
                    let output = invoke(&args, input, Some(locale));
                    assert_eq!(output.status.code(), baseline.status.code());
                    assert_eq!(output.stdout, baseline.stdout);
                    assert_eq!(output.stderr, baseline.stderr);
                }
            }
        }
    }
}

#[test]
fn language_flag_requires_one_exact_separate_value() {
    for args in [
        vec!["--lang"],
        vec!["run", "--lang"],
        vec!["run", "--lang", "es", "--lang", "pt"],
        vec!["--lang", "en", "run", "--lang", "en"],
        vec!["run", "--lang=es"],
        vec!["--lang", "ES", "run"],
        vec!["--lang", "es_EC", "run"],
        vec!["--lang", "pt-BR", "run"],
        vec!["--lang", " en", "run"],
        vec!["--lang", "", "run"],
    ] {
        let output = invoke(&args, "", None);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(value(&output.stderr)["code"], "CLI-E-USAGE");
    }
    let output = invoke(
        &["--lang", "es", "--timeout", "10", "run", "--unknown"],
        "",
        None,
    );
    let error = value(&output.stderr);
    assert_eq!(error["operation"], "run");
    assert_eq!(error["messageKey"], "errors.CLI-E-USAGE");
    assert_eq!(
        error["error"]["message"],
        engine_api::message(Lang::Es, "errors.CLI-E-USAGE").unwrap()
    );
}

#[test]
fn explain_uses_shared_translations_and_queries_keep_default_bytes() {
    for lang in Lang::ALL {
        for code in ["ENG-E-0004", "API-E-CONSOLIDATION-COST"] {
            let output = invoke(
                &["--lang", lang.code(), "explain", code, "--json"],
                "",
                None,
            );
            assert_eq!(output.status.code(), Some(0));
            assert_eq!(
                value(&output.stdout)["explanation"],
                engine_api::explain_in(code, lang).unwrap()
            );
            let text = invoke(&["explain", code, "--lang", lang.code()], "", None);
            assert_eq!(
                text.stdout,
                format!("{}\n", engine_api::explain_in(code, lang).unwrap()).as_bytes()
            );
        }
    }
    for args in [
        vec!["describe", "--json"],
        vec!["explain", "ENG-E-0004"],
        vec!["explain", "ENG-E-0004", "--json"],
    ] {
        let default = invoke(&args, "", None);
        let mut explicit = args.clone();
        explicit.extend(["--lang", "en"]);
        assert_eq!(invoke(&explicit, "", None).stdout, default.stdout);
    }
}

#[test]
fn embedded_validate_workflow_retains_localized_engine_results_on_failure() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engine-api/fixtures/input/invalid/missing-reference.json");
    for lang in Lang::ALL {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = engine_cli::EngineCli::new(Some(path.clone())).run(
            &["validate".into(), "--lang".into(), lang.code().into()],
            &mut std::io::empty(),
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(code, 2);
        assert!(stdout.is_empty());
        let envelope = value(&stderr);
        let expected = json!(engine_api::localize_diagnostics(
            engine_api::validate(INVALID).unwrap(),
            lang
        ));
        assert_eq!(envelope["diagnostics"], expected);
        assert_eq!(envelope["engineResults"][0]["diagnostics"], expected);
    }
}
