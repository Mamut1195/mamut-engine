//! Minimal in-house models shared by the VAL-039 API and CLI tests.
use serde_json::{json, Value};

pub fn cantilever() -> Value {
    serde_json::from_str(include_str!("../fixtures/val039/cantilever.json")).unwrap()
}

pub fn truss() -> Value {
    let mut input = cantilever();
    for node in input["model"]["nodes"].as_array_mut().unwrap() {
        node["dofs"] = json!(["Ux", "Uy"]);
    }
    input["model"]["members"][0]["element"] = json!("truss2d.linear");
    input["model"]["sections"][0]["properties"] = json!([{"name":"A","value":2.0}]);
    input["model"]["supports"] = json!([
        {"id":"base","node":1,"restrained":["Ux","Uy"]},
        {"id":"guide","node":2,"restrained":["Uy"]}]);
    input["cases"][0]["loads"][0]["values"] = json!([{"dof":"Ux","value":1.0}]);
    input
}

pub fn modal(input: &mut Value, mode_count: usize) {
    input["cases"] = json!([]);
    input["analyses"] = json!([{"method":"modal","options":{
        "mode_count":mode_count,"mass_kind":"lumped",
        "directions":[[1.0,0.0,0.0],[0.0,1.0,0.0]]}}]);
}

pub fn supercritical_column(ratio: f64) -> Value {
    let mut input = cantilever();
    let mesh = 4;
    input["model"]["nodes"] = json!((0..=mesh)
        .map(|i| json!({
        "id":i+1,"dim":"D2","coordinates":[i as f64,0.0,0.0],
        "dofs":["Ux","Uy","Rz"]}))
        .collect::<Vec<_>>());
    input["model"]["members"] = json!((0..mesh)
        .map(|i| json!({
        "id":i+1,"nodes":[i+1,i+2],"section":"s","law":"elastic"}))
        .collect::<Vec<_>>());
    // EI v'''' + P v'' = 0, clamped/free: sqrt(P/EI)L = pi/2.
    // Thus K=2, Pcr=pi² EI/(2L)²; E=200, I=3, L=4, all chosen here.
    // Products instead of powi: this file is also included by engine-cli tests.
    let pi = std::f64::consts::PI;
    let pcr = pi * pi * 200.0 * 3.0 / ((2.0 * 4.0) * (2.0 * 4.0));
    input["cases"] = json!([
        {"name":"G","loads":[{"node":5,"values":[{"dof":"Ux","value":-ratio*pcr}]}]},
        {"name":"H","loads":[{"node":5,"values":[{"dof":"Uy","value":1.0}]}]}]);
    input["analyses"] = json!([{"method":"pDelta","options":{
        "gravity":{"case":"G","factor":1.0},"lateral":{"case":"H","factor":1.0},
        "tolerance":1e-10,"max_iterations":100,"displacement_scale":1.0,"rotation_scale":1.0}}]);
    input
}

pub fn topology_errors() -> Vec<(String, Value, &'static str)> {
    let mut cases = Vec::new();
    for field in ["nodes", "members", "sections", "laws"] {
        let mut input = cantilever();
        let duplicate = input["model"][field][0].clone();
        input["model"][field]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        cases.push((format!("duplicate {field}"), input, "ENG-E-0002"));
    }
    for field in ["section", "law"] {
        let mut input = cantilever();
        input["model"]["members"][0][field] = json!("absent");
        cases.push((format!("missing {field}"), input, "ENG-E-0003"));
    }
    let mut input = cantilever();
    input["model"]["supports"][0]["node"] = json!(99);
    cases.push(("missing support node".into(), input, "ENG-E-0003"));
    let mut input = cantilever();
    input["model"]["nodes"].as_array_mut().unwrap().push(json!({
        "id":3,"dim":"D2","coordinates":[8.0,0.0,0.0],"dofs":["Ux","Uy","Rz"]}));
    cases.push(("disconnected node".into(), input, "ENG-E-0004"));
    let mut input = cantilever();
    input["model"]["nodes"][1]["coordinates"] = json!([0.0, 0.0, 0.0]);
    cases.push(("zero length".into(), input, "ENG-E-0005"));
    let mut input = cantilever();
    input["schema"] = json!("engine-input/9");
    cases.push(("incompatible major".into(), input, "ENG-E-0009"));
    cases
}

pub fn unstable_models() -> Vec<(String, Value, &'static str)> {
    let mut beam = cantilever();
    beam["model"]["supports"] = json!([]);
    let mut bar = truss();
    bar["model"]["supports"].as_array_mut().unwrap().pop();
    let mut massless = bar.clone();
    massless["model"]["masses"] = json!([
        {"id":"m","node":2,"values":[{"dof":"Ux","value":5.0}]}]);
    modal(&mut massless, 1);
    vec![
        ("unsupported beam".into(), beam, "ENG-E-0006"),
        ("hypostatic truss".into(), bar, "ENG-E-0006"),
        ("massless mechanism".into(), massless, "ENG-E-0014"),
        (
            "supercritical column".into(),
            supercritical_column(1.2),
            "ENG-E-0006",
        ),
    ]
}
