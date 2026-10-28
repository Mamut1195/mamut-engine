# Clean-room log · ENG-CORE-002-FIX

- Ticket / PR: ENG-CORE-002-FIX (compatibility with units/1 ASCII codes).
- Author(s): Codex (gpt-6.1-sol, medium effort), under the direction of J2 · 2026-10-10 (America/Guayaquil).
- Role: implementer
- Contract implemented: `units/1`; `Unit::parse` recognizes `N.m` as `NewtonMeter` and `kN.m` as `KilonewtonMeter`, preserving all previous symbols and aliases. `MN.m`, `N.mm` and `n.m` still return `ENG-E-0001`. See `crates/engine-core/src/units.rs`.
- Permitted sources used: `units/1` ASCII codes from the `mamut-contracts` repository, `schemas/core/units/1.0.0.schema.json`, according to the specification supplied by J2 for this ticket. Rustdoc and tests are original writing.
- Own derivations: none; explicit mapping of the two ASCII codes to the existing variants.
- Third-party code copied: none. No code from other repositories in the program was reused.
- Validation: own unit tests for both ASCII codes, equivalence between `N.m` and `N·m` and rejection of `MN.m`, `N.mm` and `n.m`; existing symbol and alias tests. `~/.cargo/bin/cargo fmt --check`, `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`, `~/.cargo/bin/cargo test` and `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`.
- Comparison with oracle (sibling repo, informational): no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: pending (formal approval belongs to the responsible person).
