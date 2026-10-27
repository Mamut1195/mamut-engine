//! Run the same five axial fixtures on native and wasm targets, without JS imports.
#[path = "../../engine-geo/tests/support/axial_benchmarks.rs"]
mod harness;

fn main() -> engine_core::EResult<()> {
    harness::run_all().map(|_| ())
}

/// Five fixtures passed; zero if any comparison fails.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn axial_benchmarks() -> u32 {
    match harness::run_all() {
        Ok(n) => n as u32,
        Err(_) => 0,
    }
}
