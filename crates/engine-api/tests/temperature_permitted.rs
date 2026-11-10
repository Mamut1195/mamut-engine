//! Casos de temperatura que sí entran en pDelta y en buckling.linear.
//!
//! Barra horizontal de longitud L, extremo i impedido y extremo j libre en el
//! eje. La fuerza de nudo equivalente en j es E·A·α·ΔT a lo largo de la barra
//! (provenance/ENG-LOAD-TEMP-972.md) y la rigidez axial es E·A/L, así que el
//! alargamiento es
//!
//! δ = α·ΔT·L.
//!
//! Con α = 0.5 1/K, ΔT = 3 K y L = 2 m, δ = 0.5·3·2 = 3.0 m. En una barra
//! alineada con X, la proyección transversal de Kg es I−ddᵀ y la fila axial
//! es nula, así que la compresión de gravedad no cambia ese Ux.
//!
//! El pandeo de la misma barra, arriostrada en Y por otra cercha de longitud
//! Lv, tiene K_y = E·A/Lv y −Kg_y = P/L. El factor es
//!
//! λ = (E·A/Lv) / (P/L) = E·A·L / (Lv·P).
//!
//! Con E = 8 Pa, A = 4 m², L = 2 m, Lv = 1 m y P = 16 N, λ = 32·2/(1·16) = 4.
//! Una temperatura en un caso que no es la referencia no entra en ese Kg.
use engine_api::{run, validate, CancelToken, Status};
use serde_json::{json, Value};

const ALPHA_PER_KELVIN: f64 = 0.5;
const DELTA_T_KELVIN: f64 = 3.0;
const LENGTH_M: f64 = 2.0;
const BRACE_M: f64 = 1.0;
const YOUNG_PA: f64 = 8.0;
const AREA_M2: f64 = 4.0;
const COMPRESSION_N: f64 = 16.0;
const ABSOLUTE_TOLERANCE: f64 = 1e-9;

fn thermal_elongation_m() -> f64 {
    ALPHA_PER_KELVIN * DELTA_T_KELVIN * LENGTH_M
}

fn buckling_factor() -> f64 {
    YOUNG_PA * AREA_M2 * LENGTH_M / (BRACE_M * COMPRESSION_N)
}

fn within(got: f64, want: f64, label: &str) {
    let delta = (got - want).abs();
    assert!(
        delta <= ABSOLUTE_TOLERANCE,
        "{label}: got {got}, want {want}, |delta| {delta}, tolerance {ABSOLUTE_TOLERANCE}"
    );
}

fn section_and_law() -> (Value, Value) {
    (
        json!([{"id": "s", "properties": [{"name": "A", "value": AREA_M2}]}]),
        json!([{"name": "elastic", "parameters": [
            {"name": "E", "value": YOUNG_PA},
            {"name": "rho", "value": 0.0}
        ]}]),
    )
}

fn temperature(member: u32) -> Value {
    json!([{
        "member": member,
        "alpha_per_kelvin": ALPHA_PER_KELVIN,
        "delta_t_kelvin": DELTA_T_KELVIN
    }])
}

fn pdelta(gravity_temperature: bool, lateral_temperature: bool) -> Value {
    let (sections, laws) = section_and_law();
    let mut gravity = json!({
        "name": "G",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -COMPRESSION_N}]}]
    });
    let mut lateral = json!({"name": "H", "loads": []});
    if gravity_temperature {
        gravity["temperature_loads"] = temperature(1);
    }
    if lateral_temperature {
        lateral["temperature_loads"] = temperature(1);
    }
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
                "id": 1, "nodes": [1, 2], "section": "s", "law": "elastic",
                "element": "truss2d.linear"
            }],
            "sections": sections,
            "laws": laws,
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "b", "node": 2, "restrained": ["Uy"]}
            ],
            "masses": []
        },
        "cases": [gravity, lateral],
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

fn buckling(reference_temperature: bool, other_temperature: bool) -> Value {
    let (sections, laws) = section_and_law();
    let mut reference = json!({
        "name": "R",
        "loads": [{"node": 2, "values": [{"dof": "Ux", "value": -COMPRESSION_N}]}]
    });
    if reference_temperature {
        reference["temperature_loads"] = temperature(1);
    }
    let mut cases = vec![reference];
    if other_temperature {
        cases.push(json!({
            "name": "T",
            "loads": [],
            "temperature_loads": temperature(2)
        }));
    }
    json!({
        "schema": "engine-input/1",
        "units": "SI",
        "options": {"allow_experimental": true},
        "model": {
            "axes": "Z-up",
            "nodes": [
                {"id": 1, "dim": "D2", "coordinates": [0.0, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 2, "dim": "D2", "coordinates": [LENGTH_M, 0.0, 0.0], "dofs": ["Ux", "Uy"]},
                {"id": 3, "dim": "D2", "coordinates": [LENGTH_M, BRACE_M, 0.0], "dofs": ["Ux", "Uy"]}
            ],
            "members": [
                {"id": 1, "nodes": [1, 2], "section": "s", "law": "elastic", "element": "truss2d.linear"},
                {"id": 2, "nodes": [2, 3], "section": "s", "law": "elastic", "element": "truss2d.linear"}
            ],
            "sections": sections,
            "laws": laws,
            "supports": [
                {"id": "a", "node": 1, "restrained": ["Ux", "Uy"]},
                {"id": "c", "node": 3, "restrained": ["Ux", "Uy"]}
            ],
            "masses": []
        },
        "cases": cases,
        "combinations": [],
        "analyses": [{
            "method": "buckling.linear",
            "options": {"mode_count": 1, "reference": {"case": "R", "factor": 1.0}}
        }]
    })
}

fn document(model: &Value) -> (engine_api::Outcome, Value) {
    let outcome = run(&model.to_string(), &CancelToken::new(), |_| {});
    let parsed = serde_json::from_str(&outcome.document).expect("results json");
    (outcome, parsed)
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

fn eigenvalue(model: &Value) -> f64 {
    let (outcome, parsed) = document(model);
    assert_eq!(outcome.status, Status::Ok, "{}", outcome.document);
    parsed["buckling"]["modes"][0]["eigenvalue"]
        .as_f64()
        .unwrap()
}

fn assert_validate_code(model: &Value, code: Option<&str>) {
    let diagnostics = validate(&model.to_string()).unwrap();
    match code {
        None => assert!(
            diagnostics.iter().all(|item| item.severity != "error"),
            "{diagnostics:?}"
        ),
        Some(code) => assert!(
            diagnostics
                .iter()
                .any(|item| item.code == code && item.severity == "error"),
            "{diagnostics:?}"
        ),
    }
}

#[test]
fn pdelta_lateral_temperature_elongates_by_alpha_delta_t_length() {
    let elongation = thermal_elongation_m();
    assert_eq!(elongation, 3.0);
    let model = pdelta(false, true);
    assert_validate_code(&model, None);
    let (outcome, parsed) = document(&model);
    assert!(
        matches!(outcome.status, Status::Ok | Status::Warning),
        "{}",
        outcome.document
    );
    within(ux(&parsed["cases"][0], 2), elongation, "lateral Ux");
    let node = parsed["cases"][0]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"].as_u64() == Some(2))
        .unwrap();
    for (component, value) in node["displacements"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .skip(1)
    {
        within(
            value.as_f64().unwrap(),
            0.0,
            &format!("component {component}"),
        );
    }
}

#[test]
fn buckling_temperature_outside_the_reference_keeps_the_factor() {
    let expected = buckling_factor();
    assert_eq!(expected, 4.0);
    let plain = buckling(false, false);
    let heated = buckling(false, true);
    assert_validate_code(&plain, None);
    assert_validate_code(&heated, None);
    let plain_factor = eigenvalue(&plain);
    let heated_factor = eigenvalue(&heated);
    within(plain_factor, expected, "reference factor");
    within(
        heated_factor,
        plain_factor,
        "factor with temperature off the reference",
    );
}

#[test]
fn validate_rejects_reference_and_gravity_temperature() {
    assert_validate_code(&pdelta(true, false), Some("ENG-E-0013"));
    assert_validate_code(&buckling(true, false), Some("ENG-E-0013"));
}
