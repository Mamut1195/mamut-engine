//! API RP 2GEO fixtures shared with engine-geo and the wasm runtime.
#[path = "../../engine-geo/tests/support/axial_benchmarks.rs"]
mod harness;

#[test]
fn analytical_axial_benchmarks() {
    assert_eq!(harness::run_all().expect("Axial fixtures"), 5);
}
