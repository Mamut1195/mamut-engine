//! Calibración offline del presupuesto denso de consolidación (MAM-835 r3).
//! Uso en release: consolidation_cost <elementos> <intervalos> <subpasos> [same] [outputs].
//! Sin `same`, alterna duraciones 1.25/1 s para impedir la reutilización del
//! factor entre intervalos consecutivos. Subpasos es el máximo por intervalo.
//! Con `same`, todas las duraciones son 1.25 s (reutilización por bits de dt).
//! No usa oráculos externos; mide el recorrido completo de engine_api::run.
use engine_api::{run, CancelToken, Status};
use serde_json::{json, Value};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        return Err(
            "Uso: consolidation_cost <elementos> <intervalos> <subpasos> [same] [outputs]".into(),
        );
    }
    let n: usize = args[1].parse()?;
    let segments: usize = args[2].parse()?;
    let steps: usize = args[3].parse()?;
    if steps == 0 {
        return Err("Subpasos debe ser positivo.".into());
    }
    let same = args[4..].iter().any(|s| s == "same");
    let outputs = args[4..].iter().any(|s| s == "outputs");
    let mut v: Value = serde_json::from_str(include_str!(
        "../fixtures/input/valid/sfc010-consolidation.json"
    ))?;
    let o = &mut v["analyses"][0]["options"];
    let mut layer = o["layers"][0].clone();
    layer["elements"] = json!(n);
    o["layers"] = json!([layer]);
    let mut t = 0.0;
    let history: Vec<_> = (0..=segments)
        .map(|i| {
            if i > 0 {
                t += if same {
                    1.25
                } else {
                    1.0 + (i % 2) as f64 * 0.25
                };
            }
            json!({"time_s":t,"delta_stress_kpa":{"value":20 + i % 3,"provenance":"measured"}})
        })
        .collect();
    o["history"] = json!(history);
    o["output_times_s"] = if outputs {
        json!(history
            .iter()
            .map(|p| p["time_s"].clone())
            .collect::<Vec<_>>())
    } else {
        json!([0.0, t])
    };
    o["max_step_s"] = json!(1.25 / steps as f64);
    let source = v.to_string();
    let start = std::time::Instant::now();
    let result = run(&source, &CancelToken::new(), |_| {});
    println!(
        "n={n} intervalos={segments} subpasos_por_intervalo<={steps} tiempo={:.6}s estado={:?}",
        start.elapsed().as_secs_f64(),
        result.status
    );
    if result.status != Status::Ok {
        return Err(result.document.into());
    }
    Ok(())
}
