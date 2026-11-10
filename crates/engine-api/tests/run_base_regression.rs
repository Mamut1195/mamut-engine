//! Representative `run` cases from the review corpus.
//!
//! Each case is generated here. The expected status, exit code, diagnostic
//! text, and progress stages are the ones the base already produces. `run`
//! must not move those rejections earlier.
use engine_api::{run, CancelToken, Progress, Status};
use serde_json::{json, Value};

fn truss(member_load: Option<[f64; 3]>, temperature: bool, experimental: bool) -> Value {
    let mut gravity = json!({
        "name": "G",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -16.0}]}]
    });
    if let Some(force) = member_load {
        gravity["member_loads"] = json!([{
            "member": 1,
            "force_per_length": force,
            "moment_per_length": [0.0, 0.0, 0.0]
        }]);
    }
    if temperature {
        gravity["temperature_loads"] = json!([{
            "member": 1,
            "alpha_per_kelvin": 0.5,
            "delta_t_kelvin": 3.0
        }]);
    }
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

fn frame_buckling(member_load: bool, temperature: bool, experimental: bool) -> Value {
    let mut gravity = json!({
        "name": "G",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -1.0}]}]
    });
    if member_load {
        gravity["member_loads"] = json!([{
            "member": 1,
            "force_per_length": [0.0, -1.0, 0.0],
            "moment_per_length": [0.0, 0.0, 0.0]
        }]);
    }
    if temperature {
        gravity["temperature_loads"] = json!([{
            "member": 1,
            "alpha_per_kelvin": 0.5,
            "delta_t_kelvin": 3.0
        }]);
    }
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": experimental},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]},
                {"id": 2, "dim": "D2", "coordinates": [4.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]}
            ],
            "members": [{
                "id": 1, "nodes": [1, 2], "section": "s", "law": "elastic",
                "element": "frame2d.euler_elastic"
            }],
            "sections": [{"id": "s", "properties": [
                {"name": "A", "value": 2.0},
                {"name": "I", "value": 3.0}
            ]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 200.0},
                {"name": "rho", "value": 0.0}
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy", "Rz"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]}
            ],
            "masses": []
        },
        "cases": [gravity],
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {"mode_count": 1, "reference": {"case": "G", "factor": 1.0}}
        }]
    })
}

struct Expectation {
    name: &'static str,
    model: Value,
    status: Status,
    exit_code: i32,
    cause_has: &'static str,
    code: &'static str,
    extra_codes: &'static [&'static str],
    stages_include: &'static [&'static str],
    stages_exclude: &'static [&'static str],
}

fn check(expectation: Expectation) {
    let mut stages = Vec::new();
    let outcome = run(
        &expectation.model.to_string(),
        &CancelToken::new(),
        |event: Progress| stages.push(event.stage),
    );
    assert_eq!(
        outcome.status, expectation.status,
        "{}: {}",
        expectation.name, outcome.document
    );
    assert_eq!(
        outcome.exit_code(),
        expectation.exit_code,
        "{}",
        expectation.name
    );
    let document: Value = serde_json::from_str(&outcome.document).unwrap();
    let diagnostics = document["diagnostics"].as_array().unwrap();
    assert_eq!(
        diagnostics.len(),
        1,
        "{}: {}",
        expectation.name,
        outcome.document
    );
    assert!(
        diagnostics.iter().any(|item| {
            item["code"] == expectation.code
                && item["cause"]
                    .as_str()
                    .unwrap()
                    .contains(expectation.cause_has)
        }),
        "{}: {}",
        expectation.name,
        outcome.document
    );
    for code in expectation.extra_codes {
        assert!(
            diagnostics.iter().all(|item| item["code"] != *code),
            "{} still has {code}: {}",
            expectation.name,
            outcome.document
        );
    }
    for stage in expectation.stages_include {
        assert!(
            stages.contains(stage),
            "{} missing {stage} in {stages:?}",
            expectation.name
        );
    }
    for stage in expectation.stages_exclude {
        assert!(
            !stages.contains(stage),
            "{} still has {stage} in {stages:?}",
            expectation.name
        );
    }
}

#[test]
fn representative_run_cases_keep_the_base_result() {
    let mut disconnected = truss(None, true, false);
    disconnected["model"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "id": 9, "dim": "D2", "coordinates": [0.0, 1.0, 0.0], "dofs": ["Ux", "Uy"]
        }));
    let mut bad_element = truss(Some([0.0, -1.0, 0.0]), false, false);
    bad_element["model"]["members"][0]["element"] = json!("nope.element");
    let mut combined = frame_buckling(true, false, true);
    combined["combinations"] = json!([{"id": "c", "terms": [{"case": "G", "factor": 1.0}]}]);
    let mut wide = frame_buckling(true, false, true);
    wide["analyses"][0]["options"]["mode_count"] = json!(10_000_000);
    let mut two = truss(None, false, false);
    two["analyses"]
        .as_array_mut()
        .unwrap()
        .push(json!({"method": "linearStatic"}));
    let mut none = truss(None, false, false);
    none["analyses"] = json!([]);
    let mut no_flag = frame_buckling(true, false, false);
    no_flag["options"] = json!({});

    let cases = [
        Expectation {
            name: "axial member load on pDelta gravity",
            model: truss(Some([-1.0, 0.0, 0.0]), false, false),
            status: Status::Unsupported,
            exit_code: 3,
            cause_has: "gravity requires nodal loads",
            code: "ENG-E-0013",
            extra_codes: &[],
            stages_include: &["model", "assembly"],
            stages_exclude: &[],
        },
        Expectation {
            name: "transverse truss member load",
            model: truss(Some([0.0, -1.0, 0.0]), false, false),
            status: Status::Invalid,
            exit_code: 2,
            cause_has: "parallel to the axis",
            code: "ENG-E-0010",
            extra_codes: &["ENG-E-0013"],
            stages_include: &["model"],
            stages_exclude: &["assembly"],
        },
        Expectation {
            name: "unknown element before member loads",
            model: bad_element,
            status: Status::Unsupported,
            exit_code: 3,
            cause_has: "Unsupported element nope.element",
            code: "ENG-E-0013",
            extra_codes: &[],
            stages_include: &["model"],
            stages_exclude: &["assembly"],
        },
        Expectation {
            name: "combinations before member loads",
            model: combined,
            status: Status::Invalid,
            exit_code: 2,
            cause_has: "Load combinations require linearStatic",
            code: "API-E-INPUT",
            extra_codes: &["ENG-E-0013"],
            stages_include: &["model", "assembly"],
            stages_exclude: &[],
        },
        Expectation {
            name: "mode count before member loads",
            model: wide,
            status: Status::Invalid,
            exit_code: 2,
            cause_has: "Steps/modes require 0..=16384",
            code: "API-E-INPUT",
            extra_codes: &["ENG-E-0013"],
            stages_include: &["model", "assembly"],
            stages_exclude: &[],
        },
        Expectation {
            name: "truss reference temperature",
            model: truss(None, true, false),
            status: Status::Unsupported,
            exit_code: 3,
            cause_has: "gravity case has uniform temperature",
            code: "ENG-E-0013",
            extra_codes: &[],
            stages_include: &["parse"],
            stages_exclude: &["model", "assembly"],
        },
        Expectation {
            name: "disconnected node with reference temperature",
            model: disconnected,
            status: Status::Invalid,
            exit_code: 2,
            cause_has: "not connected",
            code: "ENG-E-0004",
            extra_codes: &["ENG-E-0013"],
            stages_include: &["parse"],
            stages_exclude: &["model", "assembly"],
        },
        Expectation {
            name: "frame temperature has no second geometric diagnostic",
            model: frame_buckling(false, true, true),
            status: Status::Unsupported,
            exit_code: 3,
            cause_has: "has no formula",
            code: "ENG-E-0013",
            extra_codes: &[],
            stages_include: &["parse"],
            stages_exclude: &["model", "assembly"],
        },
        Expectation {
            name: "buckling without opt-in precedes member loads",
            model: no_flag,
            status: Status::Invalid,
            exit_code: 2,
            cause_has: "Experimental method buckling.linear",
            code: "API-E-INPUT",
            extra_codes: &["ENG-E-0013"],
            stages_include: &[],
            stages_exclude: &["parse", "model", "assembly"],
        },
        Expectation {
            name: "two analyses",
            model: two,
            status: Status::Invalid,
            exit_code: 2,
            cause_has: "Exactly one analysis is required",
            code: "API-E-INPUT",
            extra_codes: &[],
            stages_include: &[],
            stages_exclude: &["parse", "model", "assembly"],
        },
        Expectation {
            name: "zero analyses",
            model: none,
            status: Status::Invalid,
            exit_code: 2,
            cause_has: "Exactly one analysis is required",
            code: "API-E-INPUT",
            extra_codes: &[],
            stages_include: &[],
            stages_exclude: &["parse"],
        },
        Expectation {
            name: "temperature and member loads on a truss",
            model: truss(Some([-1.0, 0.0, 0.0]), true, false),
            status: Status::Unsupported,
            exit_code: 3,
            cause_has: "gravity case has uniform temperature",
            code: "ENG-E-0013",
            extra_codes: &[],
            stages_include: &["parse"],
            stages_exclude: &["model", "assembly"],
        },
    ];
    assert!(cases.len() >= 10 && cases.len() <= 15);
    for expectation in cases {
        check(expectation);
    }
}
