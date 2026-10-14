//! Ejecuta los mismos benchmarks nativos/wasm, sin framework ni imports de JS.
#[path = "../tests/support/winkler_harness.rs"]
mod harness;
#[path = "../tests/support/winkler_table.rs"]
mod table;
fn main() -> engine_core::EResult<()> {
    harness::run_all().map(|_| ())
}
/// Retorna seis fixtures aprobados o cero si algún benchmark falla.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn winkler_benchmarks() -> u32 {
    match harness::run_all() {
        Ok(n) => n as u32,
        Err(_) => 0,
    }
}
