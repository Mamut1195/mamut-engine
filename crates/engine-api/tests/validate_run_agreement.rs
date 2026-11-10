//! `validate` and `run` share the diagnostics for member loads and for a job
//! that does not carry exactly one analysis.
//!
//! The registry stays at 32 codes. These rejections reuse `ENG-E-0013` and
//! `API-E-INPUT`.
use engine_api::{canonical_json, run, validate, CancelToken, Status};
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
    assert_eq!(validated.len(), 1, "{validated:?}");
    assert_eq!(document["diagnostics"].as_array().unwrap().len(), 1);
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

fn truss(method: Value, gravity_member_load: bool, other_member_load: bool) -> Value {
    let mut gravity = json!({
        "name": "G",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -1.0}]}]
    });
    let mut other = json!({
        "name": "H",
        "loads": [{"node": 2, "values": [{"dof": "Uy", "value": 1.0}]}]
    });
    let load = json!([{
        "member": 1,
        "force_per_length": [0.0, -1.0, 0.0],
        "moment_per_length": [0.0, 0.0, 0.0]
    }]);
    if gravity_member_load {
        gravity["member_loads"] = load.clone();
    }
    if other_member_load {
        other["member_loads"] = load;
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
        "cases": [gravity, other],
        "combinations": [],
        "analyses": [method]
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

fn cause(model: &Value) -> String {
    validate(&model.to_string()).unwrap()[0].cause.clone()
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
        assert_eq!(cause(&model), expected);
        assert_same_diagnostics(&model, Status::Invalid);
        assert_eq!(validate(&model.to_string()).unwrap()[0].code, "API-E-INPUT");
    }
}

#[test]
fn member_loads_on_the_geometric_reference_match_run_byte_for_byte() {
    let gravity =
        "P-Delta: gravity requires nodal loads; Kg does not support member load correction";
    let reference = "Buckling: the reference case requires nodal loads; Kg does not include member-load correction";
    let pdelta_model = truss(pdelta(), true, false);
    let buckling_model = truss(buckling(), true, false);
    assert_eq!(cause(&pdelta_model), gravity);
    assert_eq!(cause(&buckling_model), reference);
    assert_same_diagnostics(&pdelta_model, Status::Unsupported);
    assert_same_diagnostics(&buckling_model, Status::Unsupported);
    assert_eq!(
        validate(&pdelta_model.to_string()).unwrap()[0].code,
        "ENG-E-0013"
    );
    assert_eq!(
        validate(&buckling_model.to_string()).unwrap()[0].code,
        "ENG-E-0013"
    );
}

#[test]
fn member_loads_off_the_geometric_reference_are_not_this_rejection() {
    for model in [truss(pdelta(), false, true), truss(buckling(), false, true)] {
        let diagnostics = validate(&model.to_string()).unwrap();
        assert!(
            diagnostics.iter().all(|item| item.code != "ENG-E-0013"),
            "{diagnostics:?}"
        );
    }
}
