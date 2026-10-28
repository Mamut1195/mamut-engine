//! MAM-860: E3 P-Δ+P-δ, independent reference with 1/8/16 elements per column.
use engine_api::{CancelToken, Status};
use serde_json::Value;

const FIXTURE: &str =
    include_str!("../../../docs/validation-report/fixtures/val032/E3-v1.1.3.json");
const REFERENCE: &str =
    include_str!("../../../docs/validation-report/fixtures/val032/_ref/e3_pdelta_reference.json");
const REFERENCE_SCRIPT: &str =
    include_str!("../../../docs/validation-report/fixtures/val032/_ref/e3_pdelta_reference.py");

#[test]
fn e3_pdelta_converges_and_matches_consistent_geometric_reference() {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let runs: Vec<_> = fixture["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run["id"] == "E3-pdelta")
        .collect();
    assert_eq!(runs.len(), 1, "A single E3-pdelta run is required");
    let input = &runs[0]["engine_input"];
    let outcome = engine_api::run(
        &serde_json::to_string(input).unwrap(),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(outcome.status, Status::Warning, "{}", outcome.document);
    let result: Value = serde_json::from_str(&outcome.document).unwrap();
    let diagnostics = result["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0]["code"], "API-W-DIAPHRAGM-RECOVERY");
    assert_eq!(diagnostics[0]["severity"], "warning");
    let convergence = &result["convergence"];
    assert_eq!(convergence["status"], "ok");
    let iterations = convergence["iterations"].as_array().unwrap();
    assert!(!iterations.is_empty() && iterations.len() <= 100);
    let last = iterations.last().unwrap();
    assert!(last["residual_convergence_ratio"].as_f64().unwrap() <= 1.0);
    assert!(last["relative_displacement_change"].as_f64().unwrap() <= 1e-10);

    let reference: Value = serde_json::from_str(REFERENCE).unwrap();
    assert_eq!(reference["run_id"], runs[0]["id"]);
    assert_eq!(
        reference["fixture_sha256"],
        mamut_canon::sha256_hex(FIXTURE.as_bytes()),
        "Regenerate the reference when changing the fixture"
    );
    assert_eq!(
        reference["script_sha256"],
        mamut_canon::sha256_hex(REFERENCE_SCRIPT.as_bytes()),
        "Regenerate the reference when changing the independent script"
    );
    // EX also runs in first order: amp does not use an oracle denominator.
    let linear_input = &fixture["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|run| run["id"] == "E3-lineal")
        .unwrap()["engine_input"];
    let linear_outcome = engine_api::run(
        &serde_json::to_string(linear_input).unwrap(),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(linear_outcome.status, Status::Warning);
    let linear_result: Value = serde_json::from_str(&linear_outcome.document).unwrap();
    let linear_case = linear_result["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["case"] == "EX")
        .unwrap();
    let meshes = reference["meshes"].as_array().unwrap();
    assert_eq!(meshes.len(), 3);
    let mut previous_ux = 0.0;
    let mut previous_linear = 0.0;
    eprintln!("piso | err_amp_8 | err_deriva_8 | err_amp_16 | err_deriva_16 | err_amp_1 | err_deriva_1 | amp*(1-theta)-1");
    for (i, floor) in meshes[0]["floors"].as_array().unwrap().iter().enumerate() {
        let master = &floor["master"];
        let model_node = input["model"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| &n["id"] == master)
            .unwrap();
        let arm_y =
            floor["cm_m"][1].as_f64().unwrap() - model_node["coordinates"][1].as_f64().unwrap();
        let cm_ux = |case: &Value| {
            let node = case["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| &n["id"] == master)
                .unwrap();
            node["displacements"][0].as_f64().unwrap()
                - arm_y * node["displacements"][5].as_f64().unwrap()
        };
        let ux = cm_ux(&result["cases"][0]);
        let linear_ux = cm_ux(linear_case);
        let drift = ux - previous_ux;
        let linear_drift = linear_ux - previous_linear;
        previous_ux = ux;
        previous_linear = linear_ux;
        let amp = drift / linear_drift;
        let theta = floor["gravity_above_N"].as_f64().unwrap() * linear_drift
            / (floor["shear_above_N"].as_f64().unwrap() * floor["height_m"].as_f64().unwrap());
        let relative = |actual: f64, expected: f64| ((actual - expected) / expected).abs();
        // Preserves and strengthens the existing theta and drift checks.
        assert!(relative(theta, floor["theta_linear"].as_f64().unwrap()) <= 1e-9);
        let theta_pd = theta * amp;
        assert!(relative(theta_pd, floor["theta_pdelta"].as_f64().unwrap()) <= 1e-9);
        let mut errors = Vec::new();
        for (mesh, count) in meshes.iter().zip([1, 8, 16]) {
            assert_eq!(mesh["elements_per_column"], count);
            let floors = mesh["floors"].as_array().unwrap();
            assert_eq!(floors.len(), 5);
            let row = &floors[i];
            assert_eq!(row["master"], *master);
            let amp_error = relative(amp, row["amplification"].as_f64().unwrap());
            let drift_error = relative(drift, row["pdelta_drift_m"].as_f64().unwrap());
            let tolerance = if count == 1 { 1e-9 } else { 1e-3 };
            assert!(
                amp_error <= tolerance,
                "floor {}: {count} elem, amp {amp_error:e}",
                i + 1
            );
            assert!(
                drift_error <= tolerance,
                "floor {}: {count} elem, drift {drift_error:e}",
                i + 1
            );
            assert!(relative(linear_drift, row["linear_drift_m"].as_f64().unwrap()) <= 1e-9);
            errors.push((amp_error, drift_error));
        }
        // Reference convergence evidence, stricter than 1e-3.
        for metric in ["amplification", "pdelta_drift_m"] {
            assert!(
                relative(
                    meshes[1]["floors"][i][metric].as_f64().unwrap(),
                    meshes[2]["floors"][i][metric].as_f64().unwrap()
                ) <= 2e-6
            );
        }
        let chord_excess = amp * (1.0 - theta) - 1.0;
        assert!(
            (0.0..=5e-2).contains(&chord_excess),
            "floor {}: {chord_excess:e}",
            i + 1
        );
        eprintln!(
            "{} | {:.9e} | {:.9e} | {:.9e} | {:.9e} | {:.9e} | {:.9e} | {:.9e}",
            i + 1,
            errors[1].0,
            errors[1].1,
            errors[2].0,
            errors[2].1,
            errors[0].0,
            errors[0].1,
            chord_excess
        );
    }
    assert_eq!(meshes[0]["floors"].as_array().unwrap().len(), 5);
    eprintln!("E3-pdelta converge: {} iteraciones", iterations.len());
}
