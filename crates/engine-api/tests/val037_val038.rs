//! VAL-037 and VAL-038.
//!
//! SOURCE: own derivation in docs/validation-report/casos/deriv/VAL-037.tex
//! and provenance/ENG-ANL-006.md.
//! CITATION: Timoshenko–Gere page, section, and equation: DESCONOCIDO.
//! The acceptance band 0.35*(B2-1)+0.005 was committed before this run.
//! source.kind = analytical. No third-program column and no external engine.
mod support;

use engine_api::{CancelToken, Status};
use engine_core::math;
use serde_json::{json, Value};
use std::f64::consts::PI;
use support::checked_run as run;

const CITATION: &str = "DESCONOCIDO. Timoshenko y Gere: cita desconocida. Derivación propia en docs/validation-report/casos/deriv/VAL-037.tex y provenance/ENG-ANL-006.md.";

const STORIES: usize = 3;
const STORY_HEIGHT: f64 = 4.0;
const BAY: f64 = 6.0;
const MODULUS: f64 = 2.0e8;
const AREA: f64 = 0.2;
const INERTIA_COLUMN: f64 = 0.02;
const INERTIA_BEAM: f64 = 6.0;
const FLOOR_GRAVITY: f64 = 1.6e5;
const FLOOR_LATERAL: f64 = 1.0e3;
const B2_SLOPE: f64 = 0.35;
const B2_OFFSET: f64 = 0.005;
const MIN_CLOSED_EXCESS: f64 = 0.02;
const MAX_STORY_RATIO: f64 = 0.25;

const EULER_E: f64 = 200.0;
const EULER_I: f64 = 3.0;
const EULER_L: f64 = 4.0;
const EULER_RELATIVE: f64 = 1.0e-3;

fn completed(status: Status, document: &str) {
    match status {
        Status::Ok | Status::Warning => {}
        Status::Invalid
        | Status::Unsupported
        | Status::NotConverged
        | Status::Unstable
        | Status::Cancelled
        | Status::Timeout
        | Status::InternalError => panic!("analysis stopped: {status:?}; {document}"),
    }
}

fn execute(document: Value) -> Value {
    let outcome = run(&document.to_string(), &CancelToken::new(), |_| {});
    completed(outcome.status, &outcome.document);
    let value: Value = serde_json::from_str(&outcome.document).unwrap();
    if outcome.status == Status::Warning {
        let codes: Vec<_> = value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["code"].as_str().unwrap_or(""))
            .collect();
        assert!(
            codes.iter().all(|code| *code == "API-W-PDELTA-INERTIA"),
            "unexpected warning {codes:?}; {CITATION}"
        );
    }
    value
}

fn section(id: &str, inertia: f64) -> Value {
    json!({
        "id": id,
        "properties": [
            {"name": "A", "value": AREA},
            {"name": "I", "value": inertia}
        ]
    })
}

fn node(id: u32, x: f64, y: f64) -> Value {
    json!({
        "id": id,
        "dim": "D2",
        "coordinates": [x, y, 0.0],
        "dofs": ["Ux", "Uy", "Rz"]
    })
}

fn member(id: u32, i: u32, j: u32, section_id: &str) -> Value {
    json!({"id": id, "nodes": [i, j], "section": section_id, "law": "elastic"})
}

fn load(node_id: u32, dof: &str, value: f64) -> Value {
    json!({"node": node_id, "values": [{"dof": dof, "value": value}]})
}

fn portal(cases: Vec<Value>, analysis: Value) -> Value {
    let mut nodes = Vec::new();
    let mut members = Vec::new();
    let mut supports = Vec::new();
    let mut member_id = 1u32;
    for level in 0..=STORIES {
        let y = STORY_HEIGHT * level as f64;
        let left = (level * 2 + 1) as u32;
        let right = left + 1;
        nodes.push(node(left, 0.0, y));
        nodes.push(node(right, BAY, y));
        if level == 0 {
            for id in [left, right] {
                supports.push(json!({
                    "id": format!("base{id}"),
                    "node": id,
                    "restrained": ["Ux", "Uy", "Rz"]
                }));
            }
        } else {
            let below_left = left - 2;
            let below_right = right - 2;
            members.push(member(member_id, below_left, left, "columna"));
            member_id += 1;
            members.push(member(member_id, below_right, right, "columna"));
            member_id += 1;
            members.push(member(member_id, left, right, "viga"));
            member_id += 1;
        }
    }
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": nodes,
            "members": members,
            "sections": [section("columna", INERTIA_COLUMN), section("viga", INERTIA_BEAM)],
            "laws": [{
                "name": "elastic",
                "parameters": [
                    {"name": "E", "value": MODULUS},
                    {"name": "rho", "value": 0.0}
                ]
            }],
            "supports": supports,
            "masses": []
        },
        "cases": cases,
        "combinations": [],
        "analyses": [analysis],
        "options": {}
    })
}

fn floor_loads(dof: &str, total: f64) -> Vec<Value> {
    let half = total / 2.0;
    let mut loads = Vec::new();
    for level in 1..=STORIES {
        let left = (level * 2 + 1) as u32;
        loads.push(load(left, dof, half));
        loads.push(load(left + 1, dof, half));
    }
    loads
}

fn case_named(name: &str, loads: Vec<Value>) -> Value {
    json!({"name": name, "loads": loads})
}

fn lateral_analysis() -> Value {
    json!({"method": "linearStatic"})
}

fn pdelta_analysis() -> Value {
    json!({
        "method": "pDelta",
        "options": {
            "gravity": {"case": "G", "factor": 1.0},
            "lateral": {"case": "H", "factor": 1.0},
            "tolerance": 1e-10,
            "max_iterations": 40,
            "displacement_scale": 1e-3,
            "rotation_scale": 1e-3
        }
    })
}

fn case_by_name<'a>(document: &'a Value, name: &str) -> &'a Value {
    document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["case"] == name)
        .unwrap_or_else(|| panic!("missing case {name}"))
}

fn ux(case: &Value, id: u32) -> f64 {
    let node = case["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"].as_u64() == Some(u64::from(id)))
        .unwrap_or_else(|| panic!("missing node {id}"));
    let values = node["displacements"].as_array().unwrap();
    assert_eq!(values.len(), 6, "Ux is index 0 of the spatial vector");
    values[0].as_f64().unwrap()
}

fn mean_ux(case: &Value, level: usize) -> f64 {
    let left_id = (level * 2 + 1) as u32;
    let left = ux(case, left_id);
    let right = ux(case, left_id + 1);
    assert!(
        (left - right).abs() <= 1e-9,
        "story level {level} is not symmetric: {left} and {right}"
    );
    0.5 * (left + right)
}

#[test]
fn three_story_b2_stays_inside_the_preregistered_band() {
    let lateral = execute(portal(
        vec![case_named("H", floor_loads("Ux", FLOOR_LATERAL))],
        lateral_analysis(),
    ));
    let second = execute(portal(
        vec![
            case_named("G", floor_loads("Uy", -FLOOR_GRAVITY)),
            case_named("H", floor_loads("Ux", FLOOR_LATERAL)),
        ],
        pdelta_analysis(),
    ));
    assert_eq!(lateral["methods"], json!(["linearStatic"]));
    assert_eq!(second["methods"], json!(["pDelta"]));
    assert_eq!(second["convergence"]["status"], "ok");
    assert_eq!(second["convergence"]["tolerance"], 1e-10);
    assert_eq!(second["convergence"]["max_iterations"], 40);
    let linear = case_by_name(&lateral, "H");
    let geometric = case_by_name(&second, "H");
    for story in 1..=STORIES {
        let carried = (STORIES + 1 - story) as f64;
        let shear = carried * FLOOR_LATERAL;
        let compression = carried * FLOOR_GRAVITY;
        let delta_linear = mean_ux(linear, story) - mean_ux(linear, story - 1);
        let delta_second = mean_ux(geometric, story) - mean_ux(geometric, story - 1);
        assert!(
            delta_linear > 0.0,
            "piso {story}: linear drift {delta_linear}"
        );
        assert!(
            delta_second > delta_linear,
            "piso {story}: P-Delta did not increase the drift"
        );
        let critical = shear * STORY_HEIGHT / delta_linear;
        let ratio = compression / critical;
        let closed = 1.0 / (1.0 - ratio);
        let engine = delta_second / delta_linear;
        let band = B2_SLOPE * (closed - 1.0) + B2_OFFSET;
        println!(
            "VAL037 piso={story} b2_piso={closed:.16e} b2_motor={engine:.16e} ratio={ratio:.16e} band={band:.16e}"
        );
        assert!(
            closed - 1.0 > MIN_CLOSED_EXCESS,
            "piso {story}: closed excess {} is below the declared floor; {CITATION}",
            closed - 1.0
        );
        assert!(
            ratio > 0.0 && ratio < MAX_STORY_RATIO,
            "piso {story}: r={ratio} is outside the declared regime"
        );
        assert!(
            (engine - closed).abs() <= band,
            "piso {story}: motor {engine} vs piso {closed}, band {band}; {CITATION}"
        );
    }
}

fn euler_load(constant: f64) -> f64 {
    constant * PI * PI * EULER_E * EULER_I / (EULER_L * EULER_L)
}

fn fixed_pinned_factor() -> f64 {
    let mut lo = PI;
    let mut hi = 1.5 * PI;
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let value = math::sin(mid) - mid * math::cos(mid);
        if value > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let theta = 0.5 * (lo + hi);
    let ratio = theta / PI;
    ratio * ratio
}

fn restraints(end: &str, at_start: bool) -> Vec<&'static str> {
    match (end, at_start) {
        ("fixed", true) => vec!["Ux", "Uy", "Rz"],
        ("fixed", false) => vec!["Uy", "Rz"],
        ("pinned", true) => vec!["Ux", "Uy"],
        ("pinned", false) => vec!["Uy"],
        ("free", _) => Vec::new(),
        _ => panic!("this pair is not one of the four Euler cases"),
    }
}

fn column_document(segments: usize, start: &str, end: &str) -> Value {
    let mut nodes = Vec::new();
    for index in 0..=segments {
        let x = EULER_L * index as f64 / segments as f64;
        nodes.push(json!({
            "id": index + 1,
            "dim": "D2",
            "coordinates": [x, 0.0, 0.0],
            "dofs": ["Ux", "Uy", "Rz"]
        }));
    }
    let members: Vec<_> = (0..segments)
        .map(|index| {
            json!({
                "id": index + 1,
                "nodes": [index + 1, index + 2],
                "section": "s",
                "law": "elastic"
            })
        })
        .collect();
    let mut supports = Vec::new();
    for (index, condition, at_start) in [(0usize, start, true), (segments, end, false)] {
        let restrained = restraints(condition, at_start);
        if restrained.is_empty() {
            continue;
        }
        supports.push(json!({
            "id": format!("s{index}"),
            "node": index + 1,
            "restrained": restrained
        }));
    }
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": nodes,
            "members": members,
            "sections": [{
                "id": "s",
                "properties": [
                    {"name": "A", "value": 2.0},
                    {"name": "I", "value": EULER_I}
                ]
            }],
            "laws": [{
                "name": "elastic",
                "parameters": [
                    {"name": "E", "value": EULER_E},
                    {"name": "rho", "value": 0.0}
                ]
            }],
            "supports": supports,
            "masses": []
        },
        "cases": [{
            "name": "N",
            "loads": [{"node": segments + 1, "values": [{"dof": "Ux", "value": -1.0}]}]
        }],
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {
                "mode_count": 1,
                "reference": {"case": "N", "factor": 1.0}
            }
        }],
        "options": {"allow_experimental": true}
    })
}

fn lowest_factor(start: &str, end: &str) -> f64 {
    let value = execute(column_document(8, start, end));
    assert_eq!(value["methods"], json!(["buckling.linear"]));
    assert_eq!(value["buckling"]["rigid_mode_count"], 0);
    value["buckling"]["modes"][0]["eigenvalue"]
        .as_f64()
        .unwrap()
}

#[test]
fn four_euler_cases_are_the_val038_series() {
    assert!(CITATION.contains("DESCONOCIDO"));
    assert!(CITATION.contains("cita desconocida"));
    let cases = [
        ("pinned", "pinned", 1.0),
        ("fixed", "free", 0.25),
        ("fixed", "pinned", fixed_pinned_factor()),
        ("fixed", "fixed", 4.0),
    ];
    for (start, end, constant) in cases {
        let expected = euler_load(constant);
        let actual = lowest_factor(start, end);
        println!("VAL038 c={constant:.16e} pcr={expected:.16e} lambda={actual:.16e}");
        let tolerance = EULER_RELATIVE * expected.abs();
        assert!(
            (actual - expected).abs() <= tolerance,
            "{start}-{end}: {actual} vs {expected}; {CITATION}"
        );
    }
}
