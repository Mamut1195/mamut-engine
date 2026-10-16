//! Native engine-api oracle consumed byte-for-byte by the JS V1 suite.
use engine_api::{CancelToken, Progress};
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, fs, path::PathBuf};

fn canonical(value: &impl Serialize) -> Result<String, Box<dyn Error>> {
    Ok(engine_api::canonical_json(&serde_json::to_string(value)?)?)
}

fn validation(
    input: &str,
    token: &CancelToken,
    events: &mut Vec<Progress>,
) -> Result<Value, Box<dyn Error>> {
    let result = engine_api::validate_with(input, token, |event| {
        events.push(event);
    });
    Ok(match result {
        Ok(diagnostics) => json!({"value": canonical(&diagnostics)?}),
        Err(error) => json!({"error": serde_json::to_string(&error)?}),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../engine-api/fixtures/input");
    let mut cases = Vec::new();
    for category in ["valid", "invalid"] {
        let mut paths = fs::read_dir(root.join(category))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort();
        for path in paths {
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let input = fs::read_to_string(&path)?;
            let mut validate_progress = Vec::new();
            let validate = validation(&input, &CancelToken::new(), &mut validate_progress)?;
            let mut run_progress = Vec::new();
            let outcome = engine_api::run(&input, &CancelToken::new(), |event| {
                run_progress.push(event);
            });
            let validate_progress = validate_progress
                .iter()
                .map(canonical)
                .collect::<Result<Vec<_>, _>>()?;
            let run_progress = run_progress
                .iter()
                .map(canonical)
                .collect::<Result<Vec<_>, _>>()?;
            let expected = if category == "invalid" {
                serde_json::from_str::<Value>(&fs::read_to_string(path.with_extension("expect"))?)?
            } else {
                Value::Null
            };
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("fixture name")?;
            cases.push(json!({"name": format!("{category}/{name}"), "input": input,
                "validate": validate, "validateProgress": validate_progress,
                "run": outcome.document, "runProgress": run_progress,
                "exitCode": outcome.exit_code(), "expected": expected}));
        }
    }
    let input = cases
        .first()
        .and_then(|case| case["input"].as_str())
        .ok_or("no fixtures")?;
    let token = CancelToken::new();
    token.cancel();
    let pre_cancelled = engine_api::run(input, &token, |_| {}).document;
    let token = CancelToken::new();
    let at_checkpoint = engine_api::run(input, &token, |_: Progress| token.cancel()).document;
    let mut unused = Vec::new();
    let validation_cancelled = validation(input, &token, &mut unused)?;
    let output = json!({"cases": cases, "version": canonical(&engine_api::version())?,
        "capabilities": canonical(&engine_api::capabilities())?,
        "schemas": {"engine-input/1": engine_api::schema("engine-input/1")?,
            "results/1": engine_api::schema("results/1")?},
        "explain": canonical(&engine_api::explain("API-I-CANCELLED")?)?,
        "preCancelled": pre_cancelled, "checkpointCancelled": at_checkpoint,
        "validationCancelled": validation_cancelled});
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
