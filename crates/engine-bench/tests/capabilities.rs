//! Cada archivo de `fixtures/capabilities` cubre una capacidad que los rc3d no usan.
use engine_api::{run, CancelToken, Status};
use engine_bench::{translate, translate_variant, AnalysisKind, BenchModel};
use serde_json::Value;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(name: &str) -> BenchModel {
    let path = repo_root().join(format!(
        "crates/engine-bench/fixtures/capabilities/{name}.json"
    ));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{name}: {error}"))
}

fn document(model: &BenchModel, kind: AnalysisKind) -> Value {
    serde_json::from_str(&translate(model, kind).unwrap()).unwrap()
}

fn execute(input: &str) -> (Status, Value) {
    let outcome = run(input, &CancelToken::new(), |_| {});
    let document = serde_json::from_str(&outcome.document).unwrap();
    (outcome.status, document)
}

#[test]
fn declared_mass_is_not_expanded() {
    let model = load("declared-mass");
    let input = translate(&model, AnalysisKind::Modal).unwrap();
    let value: Value = serde_json::from_str(&input).unwrap();
    let masses = value["model"]["masses"].as_array().unwrap();
    assert_eq!(masses.len(), 2);
    assert_eq!(masses[0]["node"], 2);
    let slave: Vec<_> = masses[0]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["dof"].as_str().unwrap())
        .collect();
    assert_eq!(slave, ["Uz", "Rx", "Ry"]);
    assert_eq!(masses[0]["values"][0]["value"], 10.0);
    assert_eq!(masses[1]["node"], 3);
    let master: Vec<_> = masses[1]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["dof"].as_str().unwrap())
        .collect();
    assert_eq!(master, ["Ux", "Uy", "Rz"]);
    assert_eq!(masses[1]["values"][0]["value"], 1000.0);
    assert_eq!(masses[1]["values"][2]["value"], 2000.0);

    let (status, result) = execute(&input);
    assert!(matches!(status, Status::Ok | Status::Warning), "{result}");
    let period = result["modes"][0]["period"].as_f64().unwrap();
    assert!(period.is_finite() && period > 0.0);
}

#[test]
fn massless_free_dof_stays_declared_and_guyan_condenses_it() {
    let mut model = load("declared-mass");
    model
        .nodal_masses
        .as_mut()
        .unwrap()
        .retain(|item| item.node != 2);
    let input = translate(&model, AnalysisKind::Modal).unwrap();
    let value: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(value["model"]["masses"].as_array().unwrap().len(), 1);
    let (status, result) = execute(&input);
    assert!(
        matches!(status, Status::Ok | Status::Warning),
        "{status:?} {result}"
    );
    assert!(!result.to_string().contains("ENG-E-0006"), "{result}");
    let period = result["modes"][0]["period"].as_f64().unwrap();
    assert!(period.is_finite() && period > 0.0, "{result}");
}

#[test]
fn modal_count_is_the_declared_integer() {
    let model = load("modes-15");
    let value = document(&model, AnalysisKind::Modal);
    assert_eq!(value["analyses"][0]["options"]["mode_count"], 15);
}

#[test]
fn spectrum_accepts_srss_two_directions_and_a_variant() {
    let model = load("spectrum-srss-xy");
    let value = document(&model, AnalysisKind::Spectrum);
    let options = &value["analyses"][0]["options"];
    assert_eq!(options["modal_combination"], "srss");
    assert_eq!(options["directional_combination"], "srss");
    assert_eq!(
        options["directional_factors"],
        serde_json::json!([1.0, 1.0])
    );
    assert_eq!(
        options["directions"][0]["direction"],
        serde_json::json!([1.0, 0.0, 0.0])
    );
    assert_eq!(
        options["directions"][1]["direction"],
        serde_json::json!([0.0, 1.0, 0.0])
    );
    assert_eq!(options["directions"][0]["spectra"][0]["pairs"][0][1], 1.0);

    let variant = translate_variant(&model, AnalysisKind::Spectrum, Some("cqc-X")).unwrap();
    let variant: Value = serde_json::from_str(&variant).unwrap();
    let options = &variant["analyses"][0]["options"];
    assert_eq!(options["modal_combination"], "cqc");
    assert_eq!(options["directions"].as_array().unwrap().len(), 1);
    assert_eq!(options["directional_factors"], serde_json::json!([1.0]));
}

#[test]
fn pdelta_passes_the_declared_options_through() {
    let model = load("pdelta");
    let input = translate(&model, AnalysisKind::PDelta).unwrap();
    let value: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(value["analyses"][0]["method"], "pDelta");
    assert_eq!(value["analyses"][0]["options"]["gravity"]["case"], "G");
    assert_eq!(value["analyses"][0]["options"]["lateral"]["case"], "EX");
    assert_eq!(value["analyses"][0]["options"]["tolerance"], 1e-8);
    assert_eq!(value["analyses"][0]["options"]["max_iterations"], 30);
    assert_eq!(value["analyses"][0]["options"]["displacement_scale"], 0.01);
    assert_eq!(value["analyses"][0]["options"]["rotation_scale"], 0.001);
    assert_eq!(value["combinations"].as_array().unwrap().len(), 0);
    let names: Vec<_> = value["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["G", "EX"]);

    let (status, result) = execute(&input);
    assert!(matches!(status, Status::Ok | Status::Warning), "{result}");
    let ux = result["cases"][0]["nodes"][1]["displacements"][0]
        .as_f64()
        .unwrap();
    assert!(ux.is_finite() && ux > 0.0);

    let mut without = model;
    without.p_delta = None;
    let error = translate(&without, AnalysisKind::PDelta)
        .unwrap_err()
        .to_string();
    assert!(error.contains("no tiene opciones por defecto"), "{error}");
}

#[test]
fn named_patterns_and_static_combinations_keep_their_factors() {
    let model = load("static-combinations");
    let input = translate(&model, AnalysisKind::Static).unwrap();
    let value: Value = serde_json::from_str(&input).unwrap();
    let names: Vec<_> = value["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["D", "L", "EX"]);
    assert_eq!(value["combinations"][1]["id"], "C5_1.2D+L-EX");
    assert_eq!(value["combinations"][1]["terms"][2]["factor"], -1.0);
    let (status, result) = execute(&input);
    assert!(matches!(status, Status::Ok | Status::Warning), "{result}");
    assert_eq!(result["combinations"][0]["id"], "C1_1.4D");
    assert_eq!(result["combinations"][1]["id"], "C5_1.2D+L-EX");
}

#[test]
fn truss_without_diaphragm_or_mass_is_a_truss() {
    let model = load("truss-no-diaphragm");
    let input = translate(&model, AnalysisKind::Static).unwrap();
    let value: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(value["model"]["diaphragms"].as_array().unwrap().len(), 0);
    assert_eq!(value["model"]["masses"].as_array().unwrap().len(), 0);
    assert_eq!(value["model"]["members"][0]["element"], "truss3d.linear");
    assert!(value["model"]["members"][0].get("orientation").is_none());
    let properties = value["model"]["sections"][0]["properties"]
        .as_array()
        .unwrap();
    assert_eq!(properties.len(), 1);
    assert_eq!(properties[0]["name"], "A");
    let parameters = value["model"]["laws"][0]["parameters"].as_array().unwrap();
    let names: Vec<_> = parameters
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["E", "rho"]);
    let (status, result) = execute(&input);
    assert!(matches!(status, Status::Ok | Status::Warning), "{result}");
    let ux = result["cases"][1]["nodes"][1]["displacements"][0]
        .as_f64()
        .unwrap();
    assert!(ux.is_finite() && ux > 0.0);
}
