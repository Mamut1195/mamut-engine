//! E3 de main/VAL-032; referencia NumPy reproducible, ver fixtures/pdelta_diaphragm/README.md.
use engine_api::{CancelToken, Status};
use serde_json::Value;

const FIXTURE: &str =
    include_str!("../../../docs/validation-report/fixtures/val032/E3-v1.1.1.json");
const REFERENCE: &str = include_str!("fixtures/pdelta_diaphragm/reference.json");

#[test]
fn e3_pdelta_converges_and_matches_consistent_geometric_reference() {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let runs: Vec<_> = fixture["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run["id"] == "E3-pdelta")
        .collect();
    assert_eq!(runs.len(), 1, "se requiere un único run E3-pdelta");
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
        "regenerar referencia al cambiar el fixture"
    );
    let nodes = result["cases"][0]["nodes"].as_array().unwrap();
    let mut previous_ux = 0.0;
    let mut max_theta_error: f64 = 0.0;
    let floors = reference["floors"].as_array().unwrap();
    assert_eq!(floors.len(), 5);
    for floor in floors {
        let master = &floor["master"];
        let node = nodes.iter().find(|n| &n["id"] == master).unwrap();
        let model_node = input["model"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| &n["id"] == master)
            .unwrap();
        let arm_y =
            floor["cm_m"][1].as_f64().unwrap() - model_node["coordinates"][1].as_f64().unwrap();
        let ux = node["displacements"][0].as_f64().unwrap()
            - arm_y * node["displacements"][5].as_f64().unwrap();
        let drift = ux - previous_ux;
        previous_ux = ux;
        let theta = floor["gravity_above_N"].as_f64().unwrap() * drift
            / (floor["shear_above_N"].as_f64().unwrap() * floor["height_m"].as_f64().unwrap());
        let theta_ref = floor["theta_pdelta"].as_f64().unwrap();
        let drift_ref = floor["pdelta_drift_m"].as_f64().unwrap();
        let error = ((theta - theta_ref) / theta_ref).abs();
        max_theta_error = max_theta_error.max(error);
        assert!(
            error <= 1e-4,
            "piso {master}: theta={theta}, ref={theta_ref}"
        );
        assert!(((drift - drift_ref) / drift_ref).abs() <= 1e-4);
        eprintln!("E3 piso {master}: theta={theta:.15e}, error_rel={error:.6e}");
    }
    eprintln!(
        "E3-pdelta converge: iteraciones={}, error relativo max theta={max_theta_error:.9e}",
        iterations.len()
    );
}
