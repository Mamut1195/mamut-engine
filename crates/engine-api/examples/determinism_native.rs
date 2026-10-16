//! Emite SHA-256 de results/1 sin target y de bits escalares para cruce con wasm.
#[path = "../tests/support/determinism.rs"]
mod fixtures;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for index in 0..fixtures::case_count() {
        let name = fixtures::case_name(index).ok_or("Índice de caso inválido")?;
        let mut first = None;
        for _ in 0..10 {
            let hash = fixtures::case_hash(index)?;
            if first.as_ref().is_some_and(|previous| previous != &hash) {
                return Err(format!("{name}: hash varía entre corridas").into());
            }
            first = Some(hash);
            let hex: String = hash.iter().map(|byte| format!("{byte:02x}")).collect();
            println!("{name} {hex}");
        }
    }
    Ok(())
}
