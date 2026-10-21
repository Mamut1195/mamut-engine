# Rust method review checklist

- [ ] Method sheet predates code, with permitted local sources and full citations;
  original derivation identified, without copied tables/figures/listings.
- [ ] Roles and clean session recorded; no consultation of prohibited code, data or
  reports; J4 review distinguished from author tests.
- [ ] Method distinct from existing ones; explicit assumptions, SI, signs, physical
  and numerical domain, history and nondifferentiable points.
- [ ] Current trait respected; tangent derived at fixed history, correct origin and
  acceptance/discard; finite outputs and caller state intact.
- [ ] Constructor validates parameters and length before indexing; catalog
  errors with Spanish messages; no new unsafe or unwrap/expect/panic
  in the library; all public API documented.
- [ ] Descriptor, version, factory and exports match the object;
  Send + Sync, without global state, I/O or threads in the law.
- [ ] Conformance without diagnostics; samples/h justified, boundaries
  checked separately; negatives for ranges, sizes, nonfinite values and overflow.
- [ ] Native benchmark with source/citation, independent analytical expected values
  established before running and justified tolerances; nonempty table.
- [ ] If fixture/1 applies: actual API input, valid source, JSON Pointers and
  execution with the ENG-CORE-009 runner; otherwise, limitation documented.
- [ ] fmt, Clippy -D warnings, workspace tests, rustdoc -D warnings, wasm
  check and clean-room origin/main pass, including new files.
- [ ] 16 hashes identical to the reference; native/wasm debug/release comparison
  recorded without updating hashes to hide differences.
- [ ] No changes to protected paths or unjustified dependencies/manifests;
  no numerical change to existing methods.
- [ ] CONTRIBUTING/guide, CHANGELOG and provenance complete in Spanish, without absolute
  local paths; logs with the `$CARGO_TARGET_DIR` marker.
- [ ] Passed/failed counts and execution limits reported; clean-room reviewer
  **PENDING (J4)** until actual review. Merge requires Jonathan's approval
  according to the protocol; this checklist does not grant it.
