//! Published contract examples; preserves the codes and warnings of ENG-API-001.
use engine_api::{run, validate, ApiError, CancelToken, Status};
use serde::Deserialize;
use serde_json::Value;
use std::{fs, path::PathBuf};

fn fixtures(category: &str) -> Vec<PathBuf> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/input")
        .join(category);
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => paths.push(path),
            Some("expect") if category == "invalid" => {
                assert!(path.with_extension("json").is_file(), "{path:?}: orphan");
            }
            _ => panic!("Unexpected fixture: {path:?}"),
        }
    }
    paths.sort();
    assert!(!paths.is_empty(), "{category}: missing fixtures");
    paths
}

#[test]
fn valid_fixtures_validate_and_run() {
    for path in fixtures("valid") {
        let input = fs::read_to_string(&path).unwrap();
        let diagnostics = validate(&input).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        assert!(diagnostics.is_empty(), "{path:?}: {diagnostics:?}");
        let outcome = run(&input, &CancelToken::new(), |_| {});
        let document: Value = serde_json::from_str(&outcome.document).unwrap();
        let input: Value = serde_json::from_str(&input).unwrap();
        if input["combinations"].as_array().unwrap().is_empty() {
            assert_eq!(outcome.status, Status::Ok, "{path:?}: {document}");
            assert_eq!(outcome.exit_code(), 0, "{path:?}");
            assert_eq!(document["diagnostics"], serde_json::json!([]), "{path:?}");
        } else {
            // #85: supports are published as zero prescribed displacements. The combination
            // is solved, but ENG-API-001 retains this warning and exit code 1.
            assert_eq!(outcome.status, Status::Warning, "{path:?}: {document}");
            assert_eq!(outcome.exit_code(), 1, "{path:?}");
            let diagnostics = document["diagnostics"].as_array().unwrap();
            assert_eq!(diagnostics.len(), 1, "{path:?}: {document}");
            assert_eq!(diagnostics[0]["code"], "API-W-COMBINED-PRESCRIBED");
            assert_eq!(
                document["combinations"].as_array().unwrap().len(),
                input["combinations"].as_array().unwrap().len(),
                "{path:?}: missing combined results"
            );
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Validation {
    InvalidInput,
    NonFiniteNumber,
    UnsupportedSchema,
    Diagnostics,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    code: String,
    exit_code: i32,
    validation: Validation,
}

#[test]
fn invalid_fixtures_report_expected_code_and_exit() {
    for path in fixtures("invalid") {
        let input = fs::read_to_string(&path).unwrap();
        let expected: Expected =
            serde_json::from_str(&fs::read_to_string(path.with_extension("expect")).unwrap())
                .unwrap();
        match (validate(&input), &expected.validation) {
            (Err(ApiError::InvalidInput(_)), Validation::InvalidInput)
            | (Err(ApiError::NonFiniteNumber(_)), Validation::NonFiniteNumber)
            | (Err(ApiError::UnsupportedSchema(_)), Validation::UnsupportedSchema) => {}
            (Ok(diagnostics), Validation::Diagnostics) => {
                assert!(
                    diagnostics
                        .iter()
                        .any(|d| d.code == expected.code && d.severity == "error"),
                    "{path:?}: {diagnostics:?}"
                );
            }
            (actual, kind) => panic!("{path:?}: validate {actual:?}; expected {kind:?}"),
        }
        let outcome = run(&input, &CancelToken::new(), |_| {});
        assert_eq!(outcome.status, Status::Invalid, "{path:?}");
        assert_eq!(outcome.exit_code(), expected.exit_code, "{path:?}");
        let document: Value = serde_json::from_str(&outcome.document).unwrap();
        assert!(
            document["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == expected.code && d["severity"] == "error"),
            "{path:?}: expected {}, received {document}",
            expected.code
        );
    }
}
