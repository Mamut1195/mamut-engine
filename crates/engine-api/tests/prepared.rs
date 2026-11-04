//! Prepared displacements match `run` on this repository's own models.
use engine_api::{CancelToken, PreparedLinear, Status};
use serde_json::{json, Value};

fn document(input: &str) -> Value {
    let outcome = engine_api::run(input, &CancelToken::new(), |_| {});
    assert!(
        matches!(outcome.status, Status::Ok | Status::Warning),
        "{}",
        outcome.document
    );
    serde_json::from_str(&outcome.document).unwrap()
}

fn assert_same_displacements(input: &str) {
    let before = engine_api::run(input, &CancelToken::new(), |_| {});
    let prepared = PreparedLinear::prepare(input, &CancelToken::new()).unwrap();
    let first = prepared.displacements().unwrap();
    let second = prepared.displacements().unwrap();
    assert_eq!(first, second);
    let after = engine_api::run(input, &CancelToken::new(), |_| {});
    assert_eq!(before.document, after.document);
    let parsed = document(input);
    let expected = parsed["cases"].as_array().unwrap();
    assert_eq!(first.len(), expected.len());
    for (actual, expected) in first.iter().zip(expected) {
        assert_eq!(actual.case, expected["case"].as_str().unwrap());
        let nodes = expected["nodes"].as_array().unwrap();
        assert_eq!(actual.nodes.len(), nodes.len());
        for (node, expected_node) in actual.nodes.iter().zip(nodes) {
            assert_eq!(u64::from(node.id), expected_node["id"].as_u64().unwrap());
            let values = expected_node["displacements"].as_array().unwrap();
            assert_eq!(values.len(), 6);
            for (got, want) in node.displacements.iter().zip(values) {
                assert_eq!(*got, want.as_f64().unwrap());
            }
        }
    }
}

#[test]
fn axial_bar_fixture_matches_run() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../engine-validation/fixtures/axial-bar.json"
    ))
    .unwrap();
    assert_same_displacements(&fixture["input"].to_string());
}

#[test]
fn cantilever_fixture_matches_run() {
    assert_same_displacements(include_str!(
        "../fixtures/input/valid/cantilever-linear-static.json"
    ));
}

#[test]
fn unordered_truss_with_two_cases_matches_run() {
    let model = json!({
        "schema": "engine-input/1", "units": "SI", "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 10, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 4, "dim": "D2", "coordinates": [4.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 7, "dim": "D2", "coordinates": [2.0, 3.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [
                {"id": 1, "nodes": [10, 4], "section": "s", "law": "steel", "element": "truss2d.linear"},
                {"id": 2, "nodes": [10, 7], "section": "s", "law": "steel", "element": "truss2d.linear"},
                {"id": 3, "nodes": [4, 7], "section": "s", "law": "steel", "element": "truss2d.linear"}
            ],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 0.01}]}],
            "laws": [{"name": "steel", "parameters": [{"name": "E", "value": 2.0e11}, {"name": "rho", "value": 0.0}]}],
            "supports": [
                {"id": "a", "node": 10, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 4, "restrained": ["Ux", "Uy"]}
            ],
            "masses": []
        },
        "cases": [
            {"name": "P", "loads": [{"node": 7, "values": [{"dof": "Ux", "value": 1000.0}, {"dof": "Uy", "value": -500.0}]}]},
            {"name": "Q", "loads": [{"node": 7, "values": [{"dof": "Uy", "value": -200.0}]}]}
        ],
        "combinations": [],
        "analyses": [{"method": "linearStatic"}]
    });
    assert_same_displacements(&model.to_string());
    assert_eq!(
        PreparedLinear::prepare(&model.to_string(), &CancelToken::new())
            .unwrap()
            .displacements()
            .unwrap()
            .iter()
            .map(|case| case.nodes.iter().map(|node| node.id).collect::<Vec<_>>())
            .next(),
        Some(vec![4, 7, 10])
    );
}

#[test]
fn other_analyses_and_combinations_ask_for_run() {
    let mut model = json!({
        "schema": "engine-input/1", "units": "SI", "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]},
                {"id": 2, "dim": "D2", "coordinates": [4.0, 0.0, 0.0], "dofs": ["Ux", "Uy", "Rz"]}
            ],
            "members": [{"id": 7, "nodes": [1, 2], "section": "s", "law": "elastic"}],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 2.0}, {"name": "I", "value": 3.0}]}],
            "laws": [{"name": "elastic", "parameters": [{"name": "E", "value": 200.0}, {"name": "rho", "value": 0.0}]}],
            "supports": [{"id": "fixed", "node": 1, "restrained": ["Ux", "Uy", "Rz"]}],
            "masses": []
        },
        "cases": [{"name": "D", "loads": [{"node": 2, "values": [{"dof": "Uy", "value": -12.0}]}]}],
        "combinations": [],
        "analyses": [{"method": "linearStatic"}]
    });
    let rejected = PreparedLinear::prepare(
        include_str!("../fixtures/input/valid/p-delta.json"),
        &CancelToken::new(),
    )
    .unwrap_err();
    assert_eq!(rejected.status, Status::Invalid);
    assert!(rejected.diagnostics[0].cause.contains("use run"));
    model["combinations"] = json!([{"id": "uls", "terms": [{"case": "D", "factor": 1.4}]}]);
    let rejected = PreparedLinear::prepare(&model.to_string(), &CancelToken::new()).unwrap_err();
    assert_eq!(rejected.status, Status::Invalid);
    assert!(rejected.diagnostics[0].cause.contains("combinations"));
}

#[test]
fn capabilities_lists_prepare() {
    assert!(engine_api::capabilities()["operations"]
        .as_array()
        .unwrap()
        .contains(&json!("prepare")));
}
