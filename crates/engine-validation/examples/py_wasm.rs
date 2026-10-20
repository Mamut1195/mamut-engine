//! Ejecuta los mismos fixtures p-y en nativo y wasm sin imports JS.
#[path = "../tests/support/py_harness.rs"]
mod py_harness;
#[path = "../tests/support/py_table.rs"]
mod py_table;
fn main() -> engine_core::EResult<()> {
    py_harness::run_all().map(|_| ())
}
/// Tres fixtures analíticos aprobados; cero indica error.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn py_benchmarks() -> u32 {
    match py_harness::run_all() {
        Ok(n) => n as u32,
        Err(_) => 0,
    }
}
