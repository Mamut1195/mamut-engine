//! The same fixtures run in examples/winkler_wasm.rs.
#[path = "support/winkler_harness.rs"]
mod harness;
#[path = "support/winkler_table.rs"]
mod table;
#[test]
fn analytical_winkler_benchmarks() {
    assert_eq!(harness::run_all().expect("benchmarks Winkler"), 6);
}
