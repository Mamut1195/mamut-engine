# Clean-room log · ENG-CORE-002

- Ticket / PR: ENG-CORE-002 (unit policy and boundary conversion) · Draft PR from branch `j2/eng-core-002-units`, stacked on `j2/eng-core-001-engine-core` (#6)
- Author(s): Codex (gpt-6.1-sol, medium effort) as code author, under the direction of J2 (Grok Bot agent); log and verification by J2 · 2026-10-10 (America/Guayaquil)
- Role: implementer
- Contract implemented: closed list of units (`Unit`), quantities (`UnitKind`), unit text parsing (error `ENG-E-0001` if unknown, never assumed), conversion to SI and from SI only at the boundary, `convert` between units of the same quantity and `UnitsOut` to declare `units_out` in results. See `crates/engine-core/src/units.rs`.
- Permitted sources used: no external sources. The conversion factors (powers of ten; degrees to radians with π) are commonly known International System definitions. Texts and rustdoc are original writing.
- Own derivations: exact factors as constants; round trips are verified by tests with relative error <= 1e-15 (values 0.1, 123.456789, 1e-9, 1e9, negatives and zero).
- Third-party code copied: none. Rust standard library only. No code from other repositories in the program was reused.
- Validation: own unit tests and doctests; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo doc --no-deps` with warnings denied and `bash clean-room/check.sh` locally (remote CI does not run because of GitHub Actions billing).
- Comparison with oracle (sibling repo, informational): no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: pending (OpenCode comments on the verdict in the PR; formal approval belongs to the responsible person).
