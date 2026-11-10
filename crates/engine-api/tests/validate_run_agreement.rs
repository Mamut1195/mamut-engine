//! `validate` repeats the diagnostics `run` already emits, in the same order.
//! `run` itself stays on the base path: it does not reject earlier.
use engine_api::{canonical_json, run, validate, CancelToken, Progress, Status};
use serde_json::{json, Value};

fn diagnostics_bytes(value: &Value) -> String {
    canonical_json(&value.to_string()).expect("canonical diagnostics")
}

fn assert_same_diagnostics(model: &Value, status: Status) {
    let input = model.to_string();
    let validated = validate(&input).expect("validate");
    let outcome = run(&input, &CancelToken::new(), |_| {});
    assert_eq!(outcome.status, status, "{}", outcome.document);
    let document: Value = serde_json::from_str(&outcome.document).expect("results json");
    let from_validate = diagnostics_bytes(&serde_json::to_value(&validated).unwrap());
    let from_run = diagnostics_bytes(&document["diagnostics"]);
    assert_eq!(from_validate, from_run, "{}", outcome.document);
}

fn cause_list(model: &Value) -> Vec<String> {
    validate(&model.to_string())
        .unwrap()
        .into_iter()
        .map(|item| item.cause)
        .collect()
}

fn shell(analyses: Value) -> Value {
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [{
                "id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]
            }],
            "members": [],
            "sections": [],
            "laws": [],
            "supports": [],
            "masses": []
        },
        "cases": [{"name": "D", "loads": []}],
        "combinations": [],
        "analyses": analyses
    })
}

fn truss(method: Value, axial_member_load: bool, transverse_member_load: bool) -> Value {
    let mut gravity = json!({
        "name": "G",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -1.0}]}]
    });
    let load = if transverse_member_load {
        json!([{
            "member": 1,
            "force_per_length": [0.0, -1.0, 0.0],
            "moment_per_length": [0.0, 0.0, 0.0]
        }])
    } else {
        json!([{
            "member": 1,
            "force_per_length": [-1.0, 0.0, 0.0],
            "moment_per_length": [0.0, 0.0, 0.0]
        }])
    };
    if axial_member_load || transverse_member_load {
        gravity["member_loads"] = load;
    }
    let experimental = method["method"] == json!("buckling.linear");
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": experimental},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [2.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [{
                "id": 1, "nodes": [1, 2], "section": "s", "law": "elastic",
                "element": "truss2d.linear"
            }],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 4.0}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 8.0},
                {"name": "rho", "value": 0.0}
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]}
            ],
            "masses": []
        },
        "cases": [gravity, {"name": "H", "loads": []}],
        "combinations": [],
        "analyses": [method]
    })
}

fn frame() -> Value {
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": true},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]},
                {"id": 2, "dim": "D2", "coordinates": [2.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]}
            ],
            "members": [{
                "id": 1, "nodes": [1, 2], "section": "s", "law": "elastic",
                "element": "frame2d.euler_elastic"
            }],
            "sections": [{"id": "s", "properties": [
                {"name": "A", "value": 4.0},
                {"name": "I", "value": 1.0}
            ]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 8.0},
                {"name": "rho", "value": 0.0}
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy", "Rz"]},
                {"id": "b", "node": 2, "restrained": ["Uy", "Rz"]}
            ],
            "masses": []
        },
        "cases": [
            {
                "name": "G",
                "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -1.0}]}],
                "member_loads": [{
                    "member": 1,
                    "force_per_length": [0.0, -1.0, 0.0],
                    "moment_per_length": [0.0, 0.0, 0.0]
                }]
            },
            {"name": "H", "loads": []}
        ],
        "combinations": [],
        "analyses": [{
            "method": "pDelta",
            "options": {
                "gravity": {"case": "G", "factor": 1.0},
                "lateral": {"case": "H", "factor": 1.0},
                "tolerance": 1e-8,
                "max_iterations": 20,
                "displacement_scale": 1.0,
                "rotation_scale": 1.0
            }
        }]
    })
}

fn pdelta() -> Value {
    json!({
        "method": "pDelta",
        "options": {
            "gravity": {"case": "G", "factor": 1.0},
            "lateral": {"case": "H", "factor": 1.0},
            "tolerance": 1e-8,
            "max_iterations": 20,
            "displacement_scale": 1.0,
            "rotation_scale": 1.0
        }
    })
}

fn buckling() -> Value {
    json!({
        "method": "buckling.linear",
        "options": {"mode_count": 1, "reference": {"case": "G", "factor": 1.0}}
    })
}

fn with_temperature(mut model: Value) -> Value {
    model["cases"][0]["temperature_loads"] = json!([{
        "member": 1,
        "alpha_per_kelvin": 0.5,
        "delta_t_kelvin": 3.0
    }]);
    model
}

fn disconnect(mut model: Value) -> Value {
    model["model"]["nodes"].as_array_mut().unwrap().push(json!({
        "id": 9,
        "dim": "D2",
        "coordinates": [0.0, 1.0, 0.0],
        "dofs": ["Ux", "Uy"]
    }));
    model
}

#[test]
fn error_registry_keeps_thirty_two_codes() {
    let registry: Value =
        serde_json::from_str(include_str!("../../../docs/contracts/error-codes-1.json"))
            .expect("error-codes-1");
    assert_eq!(registry["codes"].as_array().unwrap().len(), 32);
}

#[test]
fn one_analysis_rejection_matches_run_byte_for_byte() {
    let expected = "InvalidInput: Exactly one analysis is required";
    for analyses in [
        json!([]),
        json!([{"method": "linearStatic"}, {"method": "linearStatic"}]),
    ] {
        let model = shell(analyses);
        assert_eq!(cause_list(&model), vec![expected.to_string()]);
        assert_same_diagnostics(&model, Status::Invalid);
        assert_eq!(validate(&model.to_string()).unwrap()[0].code, "API-E-INPUT");
    }
}

#[test]
fn axial_member_loads_on_the_reference_match_run_byte_for_byte() {
    let gravity =
        "P-Delta: gravity requires nodal loads; Kg does not support member load correction";
    let reference =
        "Buckling: the reference case requires nodal loads; Kg does not include member-load correction";
    let pdelta_model = truss(pdelta(), true, false);
    let buckling_model = truss(buckling(), true, false);
    assert_eq!(cause_list(&pdelta_model), vec![gravity.to_string()]);
    assert_eq!(cause_list(&buckling_model), vec![reference.to_string()]);
    assert_same_diagnostics(&pdelta_model, Status::Unsupported);
    assert_same_diagnostics(&buckling_model, Status::Unsupported);
}

#[test]
fn transverse_truss_member_load_matches_the_assembly_diagnostic() {
    let model = truss(pdelta(), false, true);
    let causes = cause_list(&model);
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert!(causes[0].contains("parallel to the axis"), "{causes:?}");
    assert_same_diagnostics(&model, Status::Invalid);
    assert_eq!(validate(&model.to_string()).unwrap()[0].code, "ENG-E-0010");
}

#[test]
fn member_loads_off_the_geometric_reference_are_not_this_rejection() {
    let mut model = truss(pdelta(), false, false);
    model["cases"][1]["member_loads"] = json!([{
        "member": 1,
        "force_per_length": [-1.0, 0.0, 0.0],
        "moment_per_length": [0.0, 0.0, 0.0]
    }]);
    let diagnostics = validate(&model.to_string()).unwrap();
    assert!(
        diagnostics.iter().all(|item| item.code != "ENG-E-0013"),
        "{diagnostics:?}"
    );
}

#[test]
fn combinations_and_step_bounds_precede_member_loads() {
    let mut combined = frame();
    combined["combinations"] = json!([{
        "id": "c",
        "terms": [{"case": "G", "factor": 1.0}]
    }]);
    assert_eq!(
        cause_list(&combined),
        vec!["InvalidInput: Load combinations require linearStatic".to_string()]
    );
    assert_same_diagnostics(&combined, Status::Invalid);

    let mut wide = frame();
    wide["analyses"][0]["options"]["max_iterations"] = json!(1_000_000_000);
    assert_eq!(
        cause_list(&wide),
        vec!["InvalidInput: Steps/modes require 1..=16384".to_string()]
    );
    assert_same_diagnostics(&wide, Status::Invalid);
}

#[test]
fn unknown_element_precedes_member_loads() {
    let mut model = frame();
    model["model"]["members"][0]["element"] = json!("nope.element");
    let causes = cause_list(&model);
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert!(causes[0].contains("Unsupported element"), "{causes:?}");
    assert_same_diagnostics(&model, Status::Unsupported);
}

#[test]
fn experimental_opt_in_precedes_member_loads() {
    let mut model = truss(buckling(), true, false);
    model["options"] = json!({});
    let causes = cause_list(&model);
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert!(
        causes[0].contains("Experimental method buckling.linear"),
        "{causes:?}"
    );
    assert!(causes[0].contains("options.allow_experimental = true"));
    assert_same_diagnostics(&model, Status::Invalid);
    assert_eq!(validate(&model.to_string()).unwrap()[0].code, "API-E-INPUT");
}

#[test]
fn extension_error_is_not_followed_by_reference_temperature() {
    let mut model = with_temperature(truss(pdelta(), false, false));
    model["model"]["members"][0]["winkler"] = json!({"k": 1.0});
    let causes = cause_list(&model);
    assert_eq!(
        causes,
        vec!["winkler requires a Winkler beam kind.".to_string()]
    );
    assert_same_diagnostics(&model, Status::Invalid);
}

#[test]
fn geo_input_error_is_not_followed_by_another_diagnostic() {
    let model = json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": true},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [1.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [{
                "id": 1, "nodes": [1, 2], "section": "s", "law": "elastic",
                "element": "truss2d.linear"
            }],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 1.0}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 1.0}, {"name": "rho", "value": 0.0}
            ]}],
            "supports": [{"id": "a", "node": 1, "restrained": ["Ux", "Uy"]}],
            "masses": []
        },
        "cases": [],
        "combinations": [],
        "analyses": [{
            "method": "soilProfile",
            "options": {
                "depths_m": [1.0],
                "layers": [{
                    "thickness_m": 2.0,
                    "gamma_kn_m3": {"value": 18.0, "provenance": "measured"},
                    "gamma_sat_kn_m3": {"value": 20.0, "provenance": "measured"}
                }]
            }
        }]
    });
    let causes = cause_list(&model);
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert!(causes[0].contains("empty structural model"), "{causes:?}");
    assert_same_diagnostics(&model, Status::Invalid);
}

#[test]
fn model_error_blocks_member_loads_and_reference_temperature() {
    let model = disconnect(with_temperature(frame()));
    let validated = validate(&model.to_string()).unwrap();
    assert!(validated.iter().any(|item| item.code == "ENG-E-0004"));
    assert!(validated.iter().all(|item| item.code != "ENG-E-0013"));
    assert!(validated.iter().all(|item| {
        !item.cause.contains("nodal loads") && !item.cause.contains("uniform temperature")
    }));
    assert_same_diagnostics(&model, Status::Invalid);
}

#[test]
fn reference_temperature_precedes_member_loads() {
    let heated = with_temperature(truss(pdelta(), true, false));
    let causes = cause_list(&heated);
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert!(
        causes[0].contains("gravity case has uniform temperature"),
        "{causes:?}"
    );
    assert_same_diagnostics(&heated, Status::Unsupported);

    let framed = with_temperature(frame());
    let framed_causes = cause_list(&framed);
    assert_eq!(framed_causes.len(), 1, "{framed_causes:?}");
    assert!(
        framed_causes[0].contains("has no formula"),
        "{framed_causes:?}"
    );
    assert_same_diagnostics(&framed, Status::Unsupported);
}

#[test]
fn plain_member_loads_still_report_model_and_assembly_progress() {
    let input = frame().to_string();
    let mut stages = Vec::new();
    let outcome = run(&input, &CancelToken::new(), |event: Progress| {
        stages.push(event.stage);
    });
    assert_eq!(outcome.status, Status::Unsupported, "{}", outcome.document);
    assert!(stages.contains(&"model"), "{stages:?}");
    assert!(stages.contains(&"assembly"), "{stages:?}");
}
