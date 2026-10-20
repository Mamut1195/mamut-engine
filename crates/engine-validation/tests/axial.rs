//! Fixtures API RP 2GEO compartidos con engine-geo y el runtime wasm.
#[path = "../../engine-geo/tests/support/axial_benchmarks.rs"]
mod harness;

#[test]
fn analytical_axial_benchmarks() {
    assert_eq!(harness::run_all().expect("Fixtures axiales"), 5);
}
