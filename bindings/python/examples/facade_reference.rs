//! Emite exactamente results/1 de Rust para contrastar la wheel instalada.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let path = std::env::args()
        .nth(1)
        .ok_or("Falta ruta de engine-input/1")?;
    let input = std::fs::read_to_string(path)?;
    let result = engine_api::run(&input, &engine_api::CancelToken::new(), |_| {});
    if !matches!(
        result.status,
        engine_api::Status::Ok | engine_api::Status::Warning
    ) {
        return Err(result.document.into());
    }
    std::io::stdout().write_all(result.document.as_bytes())?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
