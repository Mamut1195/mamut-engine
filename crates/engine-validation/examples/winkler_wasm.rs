//! Run the same native/wasm benchmarks, without a framework or JS imports.
#[path = "../tests/support/winkler_harness.rs"]
mod harness;
#[path = "../tests/support/winkler_table.rs"]
mod table;
fn main() -> engine_core::EResult<()> {
    harness::run_all().map(|_| ())
}
/// Return six passed fixtures, or zero if any benchmark fails.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn winkler_benchmarks() -> u32 {
    match harness::run_all() {
        Ok(n) => n as u32,
        Err(_) => 0,
    }
}
