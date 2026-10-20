//! Baseline offline dentro de Rust: mismo engine-api, sin arranque por muestra.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;
    let args = std::env::args().collect::<Vec<_>>();
    let path = args.get(1).ok_or("Falta input")?;
    let repetitions: usize = args.get(2).ok_or("Faltan repeticiones")?.parse()?;
    let warmups: usize = args.get(3).ok_or("Falta calentamiento")?.parse()?;
    if repetitions == 0 {
        return Err("Se requiere al menos una repetición".into());
    }
    let input = std::fs::read_to_string(path)?;
    let mut samples = Vec::new();
    let mut document = String::new();
    for iteration in 0..warmups
        .checked_add(repetitions)
        .ok_or("Contador excesivo")?
    {
        let start = Instant::now();
        let result = engine_api::run(&input, &engine_api::CancelToken::new(), |_| {});
        let elapsed = u64::try_from(start.elapsed().as_nanos())?;
        if !matches!(
            result.status,
            engine_api::Status::Ok | engine_api::Status::Warning
        ) {
            return Err(result.document.into());
        }
        if iteration >= warmups {
            samples.push(elapsed);
        }
        document = result.document;
    }
    println!(
        "{}",
        serde_json::json!({"nanoseconds":samples,"document":document})
    );
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
