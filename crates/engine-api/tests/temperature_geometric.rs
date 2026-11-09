//! Temperatura uniforme en la referencia geométrica.
//!
//! Kg recupera el axil mecánico y no resta αΔT. `buckling.linear` y pDelta
//! rechazan ese caso con ENG-E-0013, el código de capacidad no soportada,
//! antes de publicar un factor. El modelo de pandeo es una cercha: el pórtico
//! ya rechaza la temperatura por no tener fórmula, y aquí el rechazo es el de Kg.
//!
//! Con temperatura en la referencia, este modelo de pandeo no debe publicar un
//! factor. El factor incorrecto que salía al ignorar αΔT era 300.
use engine_api::{run, CancelToken, Status};
use serde_json::{json, Value};

fn publishes_number(value: &Value, target: f64) -> bool {
    match value {
        Value::Number(number) => number.as_f64() == Some(target),
        Value::Array(items) => items.iter().any(|item| publishes_number(item, target)),
        Value::Object(map) => map.values().any(|item| publishes_number(item, target)),
        Value::Null | Value::Bool(_) | Value::String(_) => false,
    }
}

fn assert_rejected_without_numbers(document: &str) {
    let outcome = run(document, &CancelToken::new(), |_| {});
    assert_eq!(outcome.status, Status::Unsupported, "{}", outcome.document);
    assert_eq!(outcome.exit_code(), 3);
    let value: Value = serde_json::from_str(&outcome.document).expect("results json");
    assert!(
        value["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "ENG-E-0013"),
        "{}",
        outcome.document
    );
    assert_eq!(value["cases"], json!([]));
    assert_eq!(value["modes"], json!([]));
    assert_eq!(value["combinations"], json!([]));
    assert_eq!(value["envelopes"], json!([]));
    assert!(value["convergence"].is_null());
    assert!(value.get("buckling").is_none(), "{}", outcome.document);
    assert!(
        !publishes_number(&value, 300.0),
        "published the incorrect factor 300: {}",
        outcome.document
    );
}

fn probe_buckling() -> Value {
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": true},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [1.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 3, "dim": "D2", "coordinates": [4.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 4, "dim": "D2", "coordinates": [1.0, 1.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [
                {"id": 1, "nodes": [1, 2], "section": "s", "law": "elastic", "element": "truss2d.linear"},
                {"id": 2, "nodes": [2, 3], "section": "s", "law": "elastic", "element": "truss2d.linear"},
                {"id": 3, "nodes": [2, 4], "section": "s", "law": "elastic", "element": "truss2d.linear"}
            ],
            "sections": [{"id": "s", "properties": [{"name": "A", "value": 4.0}]}],
            "laws": [{"name": "elastic", "parameters": [
                {"name": "E", "value": 8.0},
                {"name": "rho", "value": 0.0}
            ]}],
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "c", "node": 3, "restrained": ["Ux", "Uy"]},
                {"id": "d", "node": 4, "restrained": ["Ux", "Uy"]}
            ],
            "masses": []
        },
        "cases": [{
            "name": "R",
            "loads": [],
            "temperature_loads": [{
                "member": 2,
                "alpha_per_kelvin": 0.5,
                "delta_t_kelvin": 0.01
            }]
        }],
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {"mode_count": 1, "reference": {"case": "R", "factor": 1.0}}
        }]
    })
}

fn pdelta_gravity() -> Value {
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [2.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [{
                "id": 1,
                "nodes": [1, 2],
                "section": "s",
                "law": "elastic",
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
        "cases": [
            {
                "name": "G",
                "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -1.0}]}],
                "temperature_loads": [{
                    "member": 1,
                    "alpha_per_kelvin": 0.5,
                    "delta_t_kelvin": 0.01
                }]
            },
            {"name": "H", "loads": [{"node": 2, "values": [{"dof": "Uy", "value": 1.0}]}]}
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

#[test]
fn buckling_reference_temperature_is_unsupported_and_publishes_no_factor() {
    assert_rejected_without_numbers(&probe_buckling().to_string());
}

#[test]
fn pdelta_gravity_temperature_is_unsupported_and_publishes_no_factor() {
    assert_rejected_without_numbers(&pdelta_gravity().to_string());
}
