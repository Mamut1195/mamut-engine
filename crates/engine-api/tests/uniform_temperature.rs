//! Public uniform temperature on a restrained truss.
//!
//! The axial −48 N is the hand product 8·4·0.5·3 from
//! provenance/ENG-LOAD-TEMP-972.md. This file does not recompute it
//! through the element and then compare the element with itself.
use engine_api::{validate, CancelToken, Status};
use serde_json::{json, Value};

fn restrained() -> Value {
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [2.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 3, "dim": "D2", "coordinates": [4.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [
                {"id": 1, "nodes": [1, 2], "section": "s", "law": "elastic", "element": "truss2d.linear"},
                {"id": 2, "nodes": [2, 3], "section": "s", "law": "elastic", "element": "truss2d.linear"}
            ],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 4.0}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 8.0},
                {"name": "rho", "value": 0.0}
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Ux", "Uy"]},
                {"id": "c", "node": 3, "restrained": ["Uy"]}
            ],
            "masses": []
        },
        "cases": [
            {
                "name": "T",
                "loads": [],
                "temperature_loads": [
                    {"member": 1, "alpha_per_kelvin": 0.5, "delta_t_kelvin": 3.0},
                    {"member": 2, "alpha_per_kelvin": 0.5, "delta_t_kelvin": 0.0}
                ]
            },
            {"name": "D", "loads": []}
        ],
        "combinations": [],
        "analyses": [{"method": "linearStatic"}]
    })
}

fn document_of(model: &Value) -> (engine_api::Outcome, Value) {
    let outcome = engine_api::run(&model.to_string(), &CancelToken::new(), |_| {});
    let document = serde_json::from_str(&outcome.document).expect("results json");
    (outcome, document)
}

fn axial(case: &Value, member: usize) -> f64 {
    case["members"][member]["stations"][0]["forces_local"][0]
        .as_f64()
        .unwrap()
}

#[test]
fn restrained_truss_recovers_the_existing_fixed_end_axial() {
    let (outcome, document) = document_of(&restrained());
    assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
    let heated = &document["cases"][0];
    let cold = &document["cases"][1];
    assert_eq!(heated["case"], "T");
    assert_eq!(cold["case"], "D");
    assert_eq!(axial(heated, 0), -48.0);
    assert_eq!(axial(heated, 1), 0.0);
    assert_eq!(heated["nodes"][0]["reactions"][0], 48.0);
    assert_eq!(heated["nodes"][1]["reactions"][0], -48.0);
    assert_eq!(heated["nodes"][2]["reactions"][0], 0.0);
    assert_eq!(heated["nodes"][2]["displacements"][0], 0.0);
    assert_eq!(axial(cold, 0), 0.0);
    assert_eq!(axial(cold, 1), 0.0);
    assert_eq!(cold["nodes"][0]["reactions"][0], 0.0);
    assert_eq!(cold["nodes"][1]["reactions"][0], 0.0);
}

#[test]
fn combination_keeps_the_recovered_thermal_axial() {
    let mut model = restrained();
    model["combinations"] = json!([{"id": "C", "terms": [{"case": "T", "factor": 1.0}]}]);
    let (outcome, document) = document_of(&model);
    assert!(
        matches!(outcome.status, Status::Ok | Status::Warning),
        "{}",
        outcome.document
    );
    assert_eq!(axial(&document["cases"][0], 0), -48.0);
    let combined = &document["combinations"][0]["results"];
    assert_eq!(axial(combined, 0), -48.0);
}

#[test]
fn elastic_frame_rejects_uniform_temperature_without_a_formula() {
    for kind in ["frame2d.euler_elastic", ""] {
        let mut model = restrained();
        model["model"]["members"][0]["element"] = json!(kind);
        model["model"]["members"][1]["element"] = json!(kind);
        model["model"]["sections"][0]["properties"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "I", "value": 1.0}));
        model["model"]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .for_each(|node| {
                node["dofs"] = json!(["Ux", "Uy", "Rz"]);
            });
        for support in model["model"]["supports"].as_array_mut().unwrap() {
            if support["node"] != 3 {
                support["restrained"] = json!(["Ux", "Uy", "Rz"]);
            }
        }
        let diagnostics = validate(&model.to_string()).unwrap();
        assert!(
            diagnostics.iter().any(|d| d.code == "ENG-E-0013"),
            "{diagnostics:?}"
        );
        let (outcome, document) = document_of(&model);
        assert_eq!(outcome.status, Status::Unsupported, "{}", outcome.document);
        assert!(document["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "ENG-E-0013"));
        assert!(document["cases"].as_array().unwrap().is_empty());
    }
}

#[test]
fn repeated_temperature_on_one_member_is_rejected() {
    let mut model = restrained();
    model["cases"][0]["temperature_loads"]
        .as_array_mut()
        .unwrap()
        .push(json!({"member": 1, "alpha_per_kelvin": 0.1, "delta_t_kelvin": 1.0}));
    let diagnostics = validate(&model.to_string()).unwrap();
    assert!(
        diagnostics.iter().any(|d| d.code == "ENG-E-0002"),
        "{diagnostics:?}"
    );
    let (outcome, _) = document_of(&model);
    assert_eq!(outcome.status, Status::Invalid);
}

#[test]
fn missing_temperature_member_is_rejected() {
    let mut model = restrained();
    model["cases"][0]["temperature_loads"][0]["member"] = json!(99);
    let error = validate(&model.to_string()).unwrap_err();
    assert!(error.to_string().contains("missing member 99"), "{error}");
}
