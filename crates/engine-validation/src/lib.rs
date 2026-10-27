#![forbid(unsafe_code)]
//! Structural benchmark suite based on independent analytical derivations.
//!
//! Development crate: `fixtures` exposes the fixture/1 contract and its
//! portable runner through engine-api. Historical suites live in tests. Scope: linear statics,
//! modal analysis, elastic P-Delta, and spectrum analysis (73 independent analytical fixtures).
//! ENG-GEO-002 adds six 2D/3D Winkler foundation fixtures, run
//! on native and wasm targets; data in `tests/support/winkler_table.rs`.
//! Data and tolerances in `tests/support/table.rs`, `dynamic_table.rs`, and
//! the repository file `provenance/ENG-VAL-001.md`. No I/O, threads, or external sources.

pub mod fixtures;
