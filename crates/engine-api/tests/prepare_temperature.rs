//! Paridad de `prepare` y `run` con temperatura uniforme.
//!
//! Barra recta, homogénea y elástica, de longitud L, con el extremo i impedido
//! y el extremo j libre a lo largo de la barra. La deformación axial se parte
//! en parte mecánica y parte térmica, ε = ε_mecánica + α·ΔT, como en
//! `provenance/ENG-LOAD-TEMP-972.md`. Sin fuerza exterior el axil es nulo, así
//! que ε_mecánica = 0 y el alargamiento del extremo libre es
//!
//! δ = α·ΔT·L.
//!
//! Con α = 0.5 1/K, ΔT = 3 K y L = 2 m, δ = 0.5·3·2 = 3.0 m. Ese producto es
//! el Ux esperado del nudo libre. No procede de otro programa.
//!
//! Una fuerza axial F en el nudo libre añade δ_mec = F·L/(E·A). Con F = 16 N,
//! E = 8 Pa y A = 4 m², δ_mec = 16·2/(8·4) = 1.0 m. En régimen lineal, fuerza
//! y temperatura en el mismo caso dan Ux = δ_mec + δ = 4.0 m.
//!
//! `prepare` no acepta combinaciones: ese trabajo sigue en `run`. El factor 2
//! sobre el caso térmico da Ux = 2·δ = 6.0 m.
use engine_api::{CancelToken, PreparedLinear, Status};
use serde_json::{json, Value};

const ALPHA_PER_KELVIN: f64 = 0.5;
const DELTA_T_KELVIN: f64 = 3.0;
const LENGTH_M: f64 = 2.0;
const YOUNG_PA: f64 = 8.0;
const AREA_M2: f64 = 4.0;
const AXIAL_FORCE_N: f64 = 16.0;
/// Cota del solver en binary64. El producto α·ΔT·L de estos datos es exacto.
const ABSOLUTE_TOLERANCE_M: f64 = 1e-12;

fn free_elongation_m() -> f64 {
    ALPHA_PER_KELVIN * DELTA_T_KELVIN * LENGTH_M
}

fn mechanical_elongation_m() -> f64 {
    AXIAL_FORCE_N * LENGTH_M / (YOUNG_PA * AREA_M2)
}

fn bar(cases: Value, combinations: Value) -> String {
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [LENGTH_M, 0.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [{
                "id": 1,
                "nodes": [1, 2],
                "section": "s",
                "law": "elastic",
                "element": "truss2d.linear"
            }],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": AREA_M2}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": YOUNG_PA},
                {"name": "rho", "value": 0.0}
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]}
            ],
            "masses": []
        },
        "cases": cases,
        "combinations": combinations,
        "analyses": [{"method": "linearStatic"}]
    })
    .to_string()
}

fn thermal(name: &str, loads: Value) -> Value {
    json!({
        "name": name,
        "loads": loads,
        "temperature_loads": [{
            "member": 1,
            "alpha_per_kelvin": ALPHA_PER_KELVIN,
            "delta_t_kelvin": DELTA_T_KELVIN
        }]
    })
}

fn force_at_free_end() -> Value {
    json!([{"node": 2, "values": [{"dof": "Ux", "value": AXIAL_FORCE_N}]}])
}

fn within(got: f64, want: f64, label: &str) {
    let delta = (got - want).abs();
    assert!(
        delta <= ABSOLUTE_TOLERANCE_M,
        "{label}: got {got}, want {want}, |delta| {delta}, tolerance {ABSOLUTE_TOLERANCE_M}"
    );
}

fn run_document(input: &str) -> (engine_api::Outcome, Value) {
    let outcome = engine_api::run(input, &CancelToken::new(), |_| {});
    let document = serde_json::from_str(&outcome.document).expect("results json");
    (outcome, document)
}

fn ux(case: &Value, node: u64) -> f64 {
    case["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"].as_u64() == Some(node))
        .unwrap()["displacements"][0]
        .as_f64()
        .unwrap()
}

fn prepared_ux(cases: &[engine_api::CaseDisplacements], name: &str, node: u32) -> f64 {
    cases
        .iter()
        .find(|case| case.case == name)
        .unwrap()
        .nodes
        .iter()
        .find(|item| item.id == node)
        .unwrap()
        .displacements[0]
}

fn assert_prepare_matches_run(input: &str) {
    let before = engine_api::run(input, &CancelToken::new(), |_| {});
    assert!(
        matches!(before.status, Status::Ok | Status::Warning),
        "{}",
        before.document
    );
    let prepared = PreparedLinear::prepare(input, &CancelToken::new()).unwrap();
    let first = prepared.displacements().unwrap();
    let second = prepared.displacements().unwrap();
    assert_eq!(first, second);
    let after = engine_api::run(input, &CancelToken::new(), |_| {});
    assert_eq!(before.document, after.document);
    let document: Value = serde_json::from_str(&before.document).unwrap();
    let expected = document["cases"].as_array().unwrap();
    assert_eq!(first.len(), expected.len());
    for (actual, expected) in first.iter().zip(expected) {
        assert_eq!(actual.case, expected["case"].as_str().unwrap());
        let nodes = expected["nodes"].as_array().unwrap();
        assert_eq!(actual.nodes.len(), nodes.len());
        for (node, expected_node) in actual.nodes.iter().zip(nodes) {
            assert_eq!(u64::from(node.id), expected_node["id"].as_u64().unwrap());
            let values = expected_node["displacements"].as_array().unwrap();
            assert_eq!(values.len(), 6);
            for (component, (got, want)) in node.displacements.iter().zip(values).enumerate() {
                within(
                    *got,
                    want.as_f64().unwrap(),
                    &format!(
                        "case {} node {} component {component}",
                        actual.case, node.id
                    ),
                );
            }
        }
    }
}

#[test]
fn free_bar_elongation_matches_the_closed_form_and_run() {
    let elongation = free_elongation_m();
    assert_eq!(elongation, 3.0);
    let input = bar(json!([thermal("T", json!([]))]), json!([]));
    let (outcome, document) = run_document(&input);
    assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
    within(ux(&document["cases"][0], 2), elongation, "run Ux");
    assert_prepare_matches_run(&input);
    let prepared = PreparedLinear::prepare(&input, &CancelToken::new())
        .unwrap()
        .displacements()
        .unwrap();
    within(prepared_ux(&prepared, "T", 2), elongation, "prepare Ux");
    for node in &prepared[0].nodes {
        for (component, value) in node.displacements.iter().enumerate() {
            if node.id == 2 && component == 0 {
                continue;
            }
            within(
                *value,
                0.0,
                &format!("node {} component {component}", node.id),
            );
        }
    }
}

#[test]
fn mechanical_load_plus_temperature_matches_run() {
    let elongation = free_elongation_m();
    let mechanical = mechanical_elongation_m();
    assert_eq!(mechanical, 1.0);
    let combined = mechanical + elongation;
    assert_eq!(combined, 4.0);
    let input = bar(
        json!([
            {"name": "F", "loads": force_at_free_end()},
            thermal("FT", force_at_free_end())
        ]),
        json!([]),
    );
    let (outcome, document) = run_document(&input);
    assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
    within(
        ux(&document["cases"][0], 2),
        mechanical,
        "run mechanical Ux",
    );
    within(ux(&document["cases"][1], 2), combined, "run combined Ux");
    assert_prepare_matches_run(&input);
    let prepared = PreparedLinear::prepare(&input, &CancelToken::new())
        .unwrap()
        .displacements()
        .unwrap();
    within(
        prepared_ux(&prepared, "F", 2),
        mechanical,
        "prepare mechanical Ux",
    );
    within(
        prepared_ux(&prepared, "FT", 2),
        combined,
        "prepare combined Ux",
    );
}

#[test]
fn combination_stays_on_run() {
    let scaled = 2.0 * free_elongation_m();
    assert_eq!(scaled, 6.0);
    let input = bar(
        json!([thermal("T", json!([]))]),
        json!([{"id": "C", "terms": [{"case": "T", "factor": 2.0}]}]),
    );
    let rejected = PreparedLinear::prepare(&input, &CancelToken::new()).unwrap_err();
    assert_eq!(rejected.status, Status::Invalid);
    assert!(rejected.diagnostics[0].cause.contains("combinations"));
    let (outcome, document) = run_document(&input);
    assert!(
        matches!(outcome.status, Status::Ok | Status::Warning),
        "{}",
        outcome.document
    );
    within(
        ux(&document["combinations"][0]["results"], 2),
        scaled,
        "run combination Ux",
    );
}
