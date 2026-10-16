//! Oráculos propios: oscilador y ecuación de viga-columna, sin fuentes externas.
use engine_api::{canonical_json, CancelToken, Outcome, Status};
use engine_core::math;
use serde_json::{json, Value};
mod support;
use support::checked_run as run;

fn oscillator() -> Value {
    json!({"schema":"engine-input/1","units":"SI","options":{},
        "model":{"axes":"Z-up","nodes":[
            {"id":1,"dim":"D2","coordinates":[0.0,0.0,0.0],"dofs":["Ux","Uy"]},
            {"id":2,"dim":"D2","coordinates":[4.0,0.0,0.0],"dofs":["Ux","Uy"]}],
            "members":[{"id":1,"nodes":[1,2],"section":"s","law":"elastic","element":"truss2d.linear"}],
            "sections":[{"id":"s","properties":[{"name":"A","value":2.0}]}],
            "laws":[{"name":"elastic","parameters":[{"name":"E","value":200.0},{"name":"rho","value":0.0}]}],
            "supports":[{"id":"fixed","node":1,"restrained":["Ux","Uy"]},{"id":"guide","node":2,"restrained":["Uy"]}],
            "masses":[{"id":"m","node":2,"values":[{"dof":"Ux","value":5.0}]}]},
        "cases":[],"combinations":[],"analyses":[{"method":"modal","options":{
            "mode_count":1,"mass_kind":"lumped","directions":[[1.0,0.0,0.0]]}}]})
}
fn spectrum() -> Value {
    let mut v = oscillator();
    v["analyses"] = json!([{"method":"responseSpectrum","options":{
        "mode_count":1,"mass_kind":"lumped","modal_combination":"srss",
        "directional_combination":"srss","directional_factors":[1.0],
        "directions":[{"direction":[1.0,0.0,0.0],"spectra":[{
            "pairs":[[0.0,3.0],[10.0,3.0]],"units":"periodSecondsAccelerationMetersPerSecondSquared",
            "damping":0.05,"interpolation":"linear","extrapolation":"error"}]}]}}]);
    v
}
fn two_oscillators(mut v: Value) -> Value {
    // Dos barras independientes desde el apoyo, L=4 y L=1, EA=400,
    // masas 5: autovalores 20 y 80; participación 50% por modo.
    v["model"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":3,"dim":"D2","coordinates":[1.0,0.0,0.0],"dofs":["Ux","Uy"]}));
    v["model"]["members"].as_array_mut().unwrap().push(
        json!({"id":2,"nodes":[1,3],"section":"s","law":"elastic","element":"truss2d.linear"}),
    );
    v["model"]["supports"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"guide2","node":3,"restrained":["Uy"]}));
    v["model"]["masses"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"m2","node":3,"values":[{"dof":"Ux","value":5.0}]}));
    v["analyses"][0]["options"]["mode_count"] = json!(2);
    v
}
fn column(mesh: usize, ratio: f64) -> Value {
    let p = ratio * math::powi(std::f64::consts::PI, 2) * 600.0 / (4.0 * 16.0);
    let mut v = oscillator();
    v["model"]["nodes"] = json!((0..=mesh)
        .map(|i| json!({"id":i+1,"dim":"D2",
        "coordinates":[4.0*i as f64/mesh as f64,0.0,0.0],"dofs":["Ux","Uy","Rz"]}))
        .collect::<Vec<_>>());
    v["model"]["members"] = json!((0..mesh)
        .map(|i| json!({"id":i+1,"nodes":[i+1,i+2],"section":"s","law":"elastic"}))
        .collect::<Vec<_>>());
    v["model"]["sections"][0]["properties"] =
        json!([{"name":"A","value":2.0},{"name":"I","value":3.0}]);
    v["model"]["supports"] = json!([{"id":"base","node":1,"restrained":["Ux","Uy","Rz"]}]);
    v["model"]["masses"] = json!([]);
    v["cases"] = json!([
        {"name":"G","loads":[{"node":mesh+1,"values":[{"dof":"Ux","value":-p}]}]},
        {"name":"H","loads":[{"node":mesh+1,"values":[{"dof":"Uy","value":1.0}]}]}]);
    v["analyses"] = json!([{"method":"pDelta","options":{
        "gravity":{"case":"G","factor":1.0},"lateral":{"case":"H","factor":1.0},
        "tolerance":1e-10,"max_iterations":100,"displacement_scale":1.0,"rotation_scale":1.0}}]);
    v
}
fn solve(v: &Value) -> (Outcome, Value) {
    let o = run(&v.to_string(), &CancelToken::new(), |_| {});
    let d = serde_json::from_str(&o.document).unwrap();
    (o, d)
}
fn num(v: &Value) -> f64 {
    v.as_f64().unwrap()
}
fn close(v: &Value, expected: f64) {
    assert!(
        (num(v) - expected).abs() < 1e-9 * expected.abs().max(1.0),
        "{v} != {expected}"
    );
}
#[test]
fn single_dof_modal_frequency_period_mass_normalization_and_full_participation() {
    // EA/L=k=100; m u''+k u=0 → ω²=k/m=20, T=2π/ω.
    for kind in ["lumped", "consistent"] {
        let mut v = oscillator();
        v["analyses"][0]["options"]["mass_kind"] = json!(kind);
        let (o, d) = solve(&v);
        assert_eq!(o.exit_code(), 0, "{}", o.document);
        let m = &d["modes"][0];
        let omega = math::sqrt(20.0_f64);
        close(&m["eigenvalue"], 20.0);
        close(&m["angular_frequency"], omega);
        close(&m["period"], std::f64::consts::TAU / omega);
        close(&m["frequency_hz"], omega / std::f64::consts::TAU);
        close(
            &m["shape"][1]["displacements"][0],
            1.0 / math::sqrt(5.0_f64),
        );
        close(&m["participation"][0]["effective_mass"], 5.0);
        close(&m["participation"][0]["mass_fraction"], 1.0);
        close(&m["participation"][0]["cumulative_mass_fraction"], 1.0);
        assert_eq!(m["normalization"], "mass");
        close(&m["shape"][0]["displacements"][0], 0.0);
    }
}
#[test]
fn constant_spectrum_single_dof_displacement_and_base_shear() {
    // Γ=√m, φ=1/√m → u=Sa/ω²=3/20, V=m Sa=15.
    for method in ["srss", "cqc"] {
        let mut v = spectrum();
        v["analyses"][0]["options"]["modal_combination"] = json!(method);
        let (o, d) = solve(&v);
        assert_eq!(o.exit_code(), 0, "{}", o.document);
        close(
            &d["envelopes"][1]["nodes"][1]["displacements"][0],
            3.0 / 20.0,
        );
        close(&d["envelopes"][1]["base_shear"], 15.0);
        close(
            &d["modes"][0]["spectrum_responses"][0]["spectral_acceleration"],
            3.0,
        );
        close(
            &d["modes"][0]["spectrum_responses"][0]["nodes"][0]["reactions"][0],
            -15.0,
        );
    }
}
#[test]
fn pdelta_cantilever_mesh_convergence_and_euler_amplification() {
    // EI y''''+P y''=0, y(0)=y'(0)=0, y''(L)=0,
    // EI y'''(L)+P y'(L)=−H → A=3(tan α−α)/α³, α=L√(P/EI).
    // Primer polo α=π/2 → Pcr=π²EI/(4L²).
    let ratio = 0.2_f64;
    let alpha = std::f64::consts::FRAC_PI_2 * math::sqrt(ratio);
    let exact = 3.0 * (math::tan(alpha) - alpha) / math::powi(alpha, 3);
    let mut errors = Vec::new();
    for mesh in [4, 8] {
        let (o, d) = solve(&column(mesh, ratio));
        assert_eq!(o.exit_code(), 0, "{}", o.document);
        let amp = num(&d["cases"][0]["nodes"][mesh]["displacements"][1]) / (64.0 / (3.0 * 600.0));
        errors.push((amp - exact).abs() / exact);
        // Aproximación de primer polo difiere <0.4%; malla de 8 limita
        // por separado el error de discretización a 1e−5 frente a solución exacta.
        assert!((amp - 1.0 / (1.0 - ratio)).abs() / (1.0 / (1.0 - ratio)) < 0.004);
        assert_eq!(d["cases"][0]["members"], json!([]));
        assert_eq!(d["convergence"]["status"], "ok");
        close(&d["cases"][0]["nodes"][0]["reactions"][1], -1.0);
        let steps = d["convergence"]["iterations"].as_array().unwrap();
        assert!(steps.len() > 1);
        let last = steps.last().unwrap();
        assert!(num(&last["residual_convergence_ratio"]) <= 1.0);
        assert!(num(&last["relative_displacement_change"]) <= 1e-10);
    }
    assert!(errors[1] < errors[0], "{errors:?}");
    assert!(errors[1] < 1e-5, "{errors:?}");
}
#[test]
fn pdelta_nonconvergence_and_instability_return_four_with_iteration_report() {
    let mut v = column(8, 0.2);
    v["analyses"][0]["options"]["max_iterations"] = json!(1);
    let (o, d) = solve(&v);
    assert_eq!(o.status, Status::NotConverged);
    assert_eq!(o.exit_code(), 4);
    assert_eq!(d["convergence"]["iterations"].as_array().unwrap().len(), 1);
    assert_eq!(d["cases"], json!([]));
    let (o, d) = solve(&column(8, 1.2));
    assert_eq!(o.status, Status::Unstable, "{}", o.document);
    assert_eq!(o.exit_code(), 4);
    assert_eq!(d["convergence"]["iterations"], json!([]));
    assert_eq!(d["modes"], json!([]));
}
#[test]
fn inertia_two_by_two_followup_is_visible_even_on_failure() {
    let mut observed = false;
    for ratio in [1.2, 2.0, 5.0, 10.0, 20.0, 50.0] {
        let (o, d) = solve(&column(2, ratio));
        if d["convergence"]["inertia_uses_2x2"] == true {
            observed = true;
            assert_eq!(o.exit_code(), 4);
            assert!(d["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x["code"] == "API-W-PDELTA-INERTIA"));
        }
    }
    assert!(
        observed,
        "El fixture debe activar bloques 2×2; no valida copysign de #78"
    );
}
#[test]
fn masses_from_nodal_case_respect_factor_gravity_and_reject_suction() {
    let mut v = oscillator();
    v["model"]["masses"] = json!([]);
    v["cases"] =
        json!([{"name":"weight","loads":[{"node":2,"values":[{"dof":"Ux","value":20.0}]}]}]);
    v["analyses"][0]["options"]["mass_source"] = json!({"method":"masses_from_case","case":"weight","factor":0.5,"direction":[1.0,0.0,0.0],"gravity":2.0});
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 1, "{}", o.document);
    close(&d["modes"][0]["eigenvalue"], 20.0);
    assert_eq!(d["diagnostics"][0]["code"], "API-W-MASS-PROJECTION");
    v["cases"][0]["loads"][0]["values"][0]["value"] = json!(-20.0);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 2, "{}", o.document);
    assert!(d["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["code"] == "ENG-E-0010"));
    assert!(d["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["code"] == "API-W-MASS-PROJECTION"));
    assert_eq!(d["modes"], json!([]));
}
#[test]
fn masses_from_system_case_include_member_equivalent_forces() {
    let mut v = oscillator();
    for n in v["model"]["nodes"].as_array_mut().unwrap() {
        n["dofs"] = json!(["Ux", "Uy", "Rz"]);
    }
    v["model"]["members"][0]["element"] = json!("frame2d.euler_elastic");
    v["model"]["sections"][0]["properties"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"I","value":3.0}));
    v["model"]["supports"][0]["restrained"] = json!(["Ux", "Uy", "Rz"]);
    v["model"]["supports"][1]["restrained"] = json!(["Uy", "Rz"]);
    v["model"]["masses"] = json!([]);
    v["cases"] = json!([{"name":"weight","loads":[],"member_loads":[{"member":1,"force_per_length":[5.0,0.0,0.0],"moment_per_length":[0.0,0.0,0.0]}]}]);
    v["analyses"][0]["options"]["mass_source"] = json!({"method":"system_case","case":"weight","factor":1.0,"direction":[1.0,0.0,0.0],"gravity":2.0});
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 1, "{}", o.document);
    // q L/2=10 por nodo, m=10/g=5; apoyo aporta denominador global.
    close(&d["modes"][0]["eigenvalue"], 20.0);
    close(&d["modes"][0]["participation"][0]["mass_fraction"], 0.5);
    v["analyses"][0]["options"]["mass_source"]["method"] = json!("masses_from_case");
    assert_eq!(solve(&v).0.exit_code(), 4); // Caso nodal vacío → M_ff=0.
}
#[test]
fn two_modes_and_directional_factors_preserve_analytical_responses() {
    let (o, d) = solve(&two_oscillators(oscillator()));
    assert_eq!(o.exit_code(), 0, "{}", o.document);
    for (n, eigenvalue) in [20.0, 80.0].iter().enumerate() {
        close(&d["modes"][n]["eigenvalue"], *eigenvalue);
        close(&d["modes"][n]["participation"][0]["mass_fraction"], 0.5);
    }
    close(
        &d["modes"][1]["participation"][0]["cumulative_mass_fraction"],
        1.0,
    );
    let mut v = two_oscillators(spectrum());
    let direction = v["analyses"][0]["options"]["directions"][0].clone();
    v["analyses"][0]["options"]["directions"]
        .as_array_mut()
        .unwrap()
        .push(direction);
    v["analyses"][0]["options"]["directional_combination"] = json!("weightedSum");
    v["analyses"][0]["options"]["directional_factors"] = json!([1.0, 2.0]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 0, "{}", o.document);
    close(
        &d["envelopes"][2]["nodes"][1]["displacements"][0],
        3.0 * 3.0 / 20.0,
    );
    close(
        &d["envelopes"][2]["nodes"][2]["displacements"][0],
        3.0 * 3.0 / 80.0,
    );
}

#[test]
fn mass_projection_off_axis_warns_and_ambiguous_sources_are_invalid() {
    let mut v = oscillator();
    v["cases"] = json!([{"name":"G","loads":[{"node":2,"values":[{"dof":"Ux","value":20.0}]}]}]);
    v["analyses"][0]["options"]["mass_source"] = json!({"method":"masses_from_case","case":"G","factor":0.5,"direction":[1.0,1.0,0.0],"gravity":2.0});
    assert_eq!(solve(&v).0.exit_code(), 2); // No mezcla con masas declaradas.
    v["model"]["masses"] = json!([]);
    let (o, d) = solve(&v);
    assert_eq!(o.exit_code(), 1, "{}", o.document);
    close(
        &d["modes"][0]["eigenvalue"],
        100.0 / (5.0 / math::sqrt(2.0_f64)),
    );
    assert!(d["diagnostics"][0]["cause"]
        .as_str()
        .unwrap()
        .contains("redondeo"));
    v["analyses"][0]["options"]["mass_source"] = Value::Null;
    assert_eq!(solve(&v).0.exit_code(), 2);
    let mut p = column(2, 0.2);
    p["cases"][0]["member_loads"] =
        json!([{"member":1,"force_per_length":[1.0,0.0,0.0],"moment_per_length":[0.0,0.0,0.0]}]);
    assert_eq!(solve(&p).0.exit_code(), 3);
}

#[test]
fn dynamic_determinism_canonical_bytes_and_input_sha() {
    for v in [oscillator(), spectrum(), column(4, 0.2)] {
        let (a, d) = solve(&v);
        let (b, _) = solve(&v);
        assert_eq!(a, b);
        assert_eq!(a.document, canonical_json(&a.document).unwrap());
        assert_eq!(
            d["input_sha256"],
            mamut_canon::fingerprint_value(&v).unwrap().value
        );
        let pretty = serde_json::to_string_pretty(&v).unwrap();
        assert_eq!(a, run(&pretty, &CancelToken::new(), |_| {}));
    }
}
#[test]
fn cancellation_at_every_dynamic_boundary_discards_all_responses() {
    let mut multi = spectrum();
    let direction = multi["analyses"][0]["options"]["directions"][0].clone();
    multi["analyses"][0]["options"]["directions"]
        .as_array_mut()
        .unwrap()
        .push(direction);
    multi["analyses"][0]["options"]["directional_factors"] = json!([1.0, 2.0]);
    for v in [
        oscillator(),
        spectrum(),
        two_oscillators(oscillator()),
        two_oscillators(spectrum()),
        multi,
        column(4, 0.2),
    ] {
        let mut stages = Vec::new();
        let done = run(&v.to_string(), &CancelToken::new(), |p| stages.push(p));
        assert_eq!(done.exit_code(), 0, "{}", done.document);
        assert!(stages.windows(2).all(|p| p[0].completed < p[1].completed));
        assert!(stages.iter().all(|p| p.completed <= p.total));
        for boundary in 0..=stages.len() {
            let c = CancelToken::new();
            if boundary == 0 {
                c.cancel();
            }
            let mut n = 0;
            let o = run(&v.to_string(), &c, |_| {
                n += 1;
                if n == boundary {
                    c.cancel();
                }
            });
            assert_eq!(o.exit_code(), 5, "{boundary}: {}", o.document);
            assert_eq!(n, boundary);
            let d: Value = serde_json::from_str(&o.document).unwrap();
            for k in [
                "nodes",
                "members",
                "cases",
                "combinations",
                "modes",
                "envelopes",
            ] {
                assert_eq!(d[k], json!([]));
            }
            assert!(d["convergence"].is_null());
        }
    }
}
#[test]
fn invalid_dynamic_options_masses_spectra_and_cases() {
    for (mut v, pointer, replacement) in [
        (oscillator(), "/analyses/0/options/mode_count", json!(0)),
        (oscillator(), "/analyses/0/options/mode_count", json!(2)),
        (
            oscillator(),
            "/analyses/0/options/mass_kind",
            json!("unknown"),
        ),
        (
            oscillator(),
            "/analyses/0/options/directions",
            json!([[0.0, 0.0, 0.0]]),
        ),
        (oscillator(), "/model/masses/0/values/0/value", json!(-1.0)),
        (
            spectrum(),
            "/analyses/0/options/directions/0/spectra/0/pairs",
            json!([[1.0, 1.0], [0.0, 1.0]]),
        ),
        (
            spectrum(),
            "/analyses/0/options/directions/0/spectra/0/damping",
            json!(1.0),
        ),
        (
            spectrum(),
            "/analyses/0/options/directions/0/spectra/0/units",
            json!("g"),
        ),
        (
            spectrum(),
            "/analyses/0/options/directions/0/spectra/0/pairs",
            json!([[8.0, 1.0], [10.0, 1.0]]),
        ),
        (
            spectrum(),
            "/analyses/0/options/directional_factors",
            json!([-1.0]),
        ),
        (
            column(2, 0.2),
            "/analyses/0/options/gravity/case",
            json!("missing"),
        ),
        (column(2, 0.2), "/analyses/0/options/tolerance", json!(0.0)),
        (
            column(2, 0.2),
            "/analyses/0/options/max_iterations",
            json!(16385),
        ),
    ] {
        *v.pointer_mut(pointer).unwrap() = replacement;
        let (o, d) = solve(&v);
        assert_eq!(o.exit_code(), 2, "{pointer}: {}", o.document);
        assert_eq!(d["modes"], json!([]));
        assert_eq!(d["cases"], json!([]));
    }
    let mut v = oscillator();
    v["model"]["masses"] = json!([]);
    assert_eq!(solve(&v).0.exit_code(), 4);
    for code in ["API-W-MASS-PROJECTION", "API-W-PDELTA-INERTIA"] {
        assert!(!engine_api::explain(code).unwrap().is_empty());
    }
}
