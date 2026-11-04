//! SOURCE: own derivation of EI w'''' + P w'' = 0.
//! CITATION: provenance/ENG-ANL-006.md. Timoshenko–Gere citation, page, and table: UNKNOWN.
//! The page stays DESCONOCIDO. These expected values are the analytical cases
//! already fixed by crates/engine-analysis/tests/buckling.rs.
mod support;

use engine_api::{capabilities, CancelToken, Status};
use serde_json::{json, Value};
use support::checked_run as run;

const CITATION: &str = "DESCONOCIDO. Timoshenko–Gere no fue consultado: sin página ni tabla. UNKNOWN. Derivación propia en provenance/ENG-ANL-006.md.";

fn euler(c_factor: f64) -> f64 {
    // Pcr = C π² EI / L² with the library scales E=200 Pa, I=3 m⁴, L=4 m.
    let elastic: f64 = 200.0;
    let inertia: f64 = 3.0;
    let length: f64 = 4.0;
    c_factor * std::f64::consts::PI * std::f64::consts::PI * elastic * inertia / (length * length)
}

fn fixed_pinned_factor() -> f64 {
    // f(θ) = sin θ − θ cos θ on (π, 3π/2). Same 80 bisections as the library test.
    let mut lo = std::f64::consts::PI;
    let mut hi = 1.5 * std::f64::consts::PI;
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let value = mid.sin() - mid * mid.cos();
        if value > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let theta = 0.5 * (lo + hi);
    (theta / std::f64::consts::PI).powi(2)
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

fn document(
    segments: usize,
    start: &str,
    end: &str,
    factor: f64,
    mode_count: usize,
    member_load: bool,
    experimental: bool,
) -> Value {
    let mut nodes = Vec::new();
    for index in 0..=segments {
        let x = 4.0 * index as f64 / segments as f64;
        nodes.push(json!({
            "id": index + 1,
            "dim": "D2",
            "coordinates": [x, 0.0, 0.0],
            "dofs": ["Ux", "Uy", "Rz"]
        }));
    }
    let members: Vec<_> = (0..segments)
        .map(|index| {
            json!({"id": index + 1, "nodes": [index + 1, index + 2], "section": "s", "law": "elastic"})
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
    let mut case = json!({
        "name": "N",
        "loads": [{"node": segments + 1, "values": [{"dof": "Ux", "value": -1.0}]}]
    });
    if member_load {
        case["member_loads"] = json!([{
            "member": 1,
            "force_per_length": [0.0, -1.0, 0.0],
            "moment_per_length": [0.0, 0.0, 0.0]
        }]);
    }
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "model": {
            "axes": "Z-up",
            "nodes": nodes,
            "members": members,
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 2.0}, {"name": "I", "value": 3.0}]}],
            "laws": [{"name": "elastic", "parameters": [{"name": "E", "value": 200.0}, {"name": "rho", "value": 0.0}]}],
            "supports": supports,
            "masses": []
        },
        "cases": [case],
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {
                "mode_count": mode_count,
                "reference": {"case": "N", "factor": factor}
            }
        }],
        "options": {"allow_experimental": experimental}
    })
}

fn eigenvalue(segments: usize, start: &str, end: &str, factor: f64) -> f64 {
    let outcome = run(
        &document(segments, start, end, factor, 1, false, true).to_string(),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
    let value: Value = serde_json::from_str(&outcome.document).unwrap();
    assert_eq!(value["methods"], json!(["buckling.linear"]));
    assert_eq!(value["modes"], json!([]));
    assert_eq!(value["cases"], json!([]));
    assert!(value["convergence"].is_null());
    let buckling = &value["buckling"];
    assert_eq!(buckling["rigid_mode_count"], 0);
    assert_eq!(buckling["modes"][0]["mode"], 1);
    assert_eq!(buckling["modes"][0]["normalization"], "negative_kg");
    assert_eq!(
        buckling["modes"][0]["shape"].as_array().unwrap().len(),
        segments + 1
    );
    let amplitude: f64 = buckling["modes"][0]["shape"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|node| node["displacements"].as_array().unwrap().iter())
        .map(|component| component.as_f64().unwrap().abs())
        .sum();
    assert!(amplitude > 0.0, "{CITATION}");
    buckling["modes"][0]["eigenvalue"].as_f64().unwrap()
}

fn close_relative(actual: f64, expected: f64) {
    let tolerance = 1e-3 * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected} within {tolerance}; {CITATION}"
    );
}

#[test]
fn page_citation_stays_unknown() {
    assert!(CITATION.contains("DESCONOCIDO"));
    assert!(CITATION.contains("UNKNOWN"));
    assert!(CITATION.contains("provenance/ENG-ANL-006.md"));
}

#[test]
fn one_element_pinned_column_matches_the_library_projection() {
    let factor = eigenvalue(1, "pinned", "pinned", 1.0);
    assert!(
        (factor - 450.0).abs() <= 1e-8,
        "{factor} != 450; {CITATION}"
    );
    let doubled = eigenvalue(1, "pinned", "pinned", 2.0);
    assert!((doubled - 225.0).abs() <= 1e-8, "{doubled}; {CITATION}");
}

#[test]
fn eight_element_euler_cases_match_the_library_factors() {
    close_relative(eigenvalue(8, "pinned", "pinned", 1.0), euler(1.0));
    close_relative(eigenvalue(8, "fixed", "free", 1.0), euler(0.25));
    close_relative(
        eigenvalue(8, "fixed", "pinned", 1.0),
        euler(fixed_pinned_factor()),
    );
    close_relative(eigenvalue(8, "fixed", "fixed", 1.0), euler(4.0));
}

#[test]
fn zero_mode_count_returns_an_empty_spectrum() {
    let outcome = run(
        &document(1, "pinned", "pinned", 1.0, 0, false, true).to_string(),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
    let value: Value = serde_json::from_str(&outcome.document).unwrap();
    assert_eq!(value["buckling"]["rigid_mode_count"], 0);
    assert_eq!(value["buckling"]["modes"], json!([]));
    assert_eq!(value["modes"], json!([]));
}

#[test]
fn experimental_opt_in_is_required_and_drops_partial_buckling() {
    assert_eq!(
        capabilities()["methods"]["buckling.linear"],
        json!({"engine": true, "run": true})
    );
    let outcome = run(
        &document(1, "pinned", "pinned", 1.0, 1, false, false).to_string(),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(outcome.status, Status::Invalid, "{}", outcome.document);
    let value: Value = serde_json::from_str(&outcome.document).unwrap();
    assert_eq!(value["diagnostics"][0]["code"], "API-E-INPUT");
    assert!(outcome.document.contains("buckling.linear"));
    assert!(outcome.document.contains("Experimental"));
    assert!(outcome
        .document
        .contains("options.allow_experimental = true"));
    assert!(value.get("buckling").is_none());
    assert_eq!(value["modes"], json!([]));
}

#[test]
fn member_loads_on_the_reference_case_are_rejected() {
    let outcome = run(
        &document(1, "pinned", "pinned", 1.0, 1, true, true).to_string(),
        &CancelToken::new(),
        |_| {},
    );
    assert_eq!(outcome.status, Status::Unsupported, "{}", outcome.document);
    let value: Value = serde_json::from_str(&outcome.document).unwrap();
    assert_eq!(value["diagnostics"][0]["code"], "ENG-E-0013");
    assert!(value.get("buckling").is_none());
}
