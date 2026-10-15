//! Benchmarks compartidos con el runtime wasm, con oráculos analíticos propios.
#[path = "support/consolidation_harness.rs"]
mod harness;
#[path = "support/consolidation_table.rs"]
mod table;
#[test]
fn analytical_consolidation_benchmarks() {
    assert_eq!(harness::run_all().expect("benchmarks consolidación"), 7);
}
