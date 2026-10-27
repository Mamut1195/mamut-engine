//! Benchmarks shared with the wasm runtime, with independent analytical oracles.
#[path = "support/consolidation_harness.rs"]
mod harness;
#[path = "support/consolidation_table.rs"]
mod table;
#[test]
fn analytical_consolidation_benchmarks() {
    assert_eq!(harness::run_all().expect("consolidation benchmarks"), 7);
}
