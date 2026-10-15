//! Ejecuta idénticos fixtures y convergencia en nativo y wasm sin imports JS.
#[path = "../tests/support/consolidation_harness.rs"]
mod harness;
#[path = "../tests/support/consolidation_table.rs"]
mod table;
fn main() -> engine_core::EResult<()> {
    harness::run_all().map(|_| ())
}
/// Siete fixtures aprobados y convergencia verificada; cero si hay un error.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn consolidation_benchmarks() -> u32 {
    match harness::run_all() {
        Ok(n) => n as u32,
        Err(_) => 0,
    }
}
