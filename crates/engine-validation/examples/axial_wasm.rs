//! Ejecuta los mismos cinco fixtures axiales en nativo y wasm, sin imports JS.
#[path = "../../engine-geo/tests/support/axial_benchmarks.rs"]
mod harness;

fn main() -> engine_core::EResult<()> {
    harness::run_all().map(|_| ())
}

/// Cinco fixtures aprobados; cero si falla alguna comparación.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn axial_benchmarks() -> u32 {
    match harness::run_all() {
        Ok(n) => n as u32,
        Err(_) => 0,
    }
}
