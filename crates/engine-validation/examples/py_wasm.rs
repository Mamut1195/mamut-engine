//! Run the same p-y fixtures on native and wasm targets without JS imports.
#[path = "../tests/support/py_harness.rs"]
mod py_harness;
#[path = "../tests/support/py_table.rs"]
mod py_table;
fn main() -> engine_core::EResult<()> {
    py_harness::run_all().map(|_| ())
}
/// Three analytical fixtures passed; zero indicates an error.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn py_benchmarks() -> u32 {
    match py_harness::run_all() {
        Ok(n) => n as u32,
        Err(_) => 0,
    }
}
