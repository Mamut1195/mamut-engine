# Clean-room log · ENG-CORE-003

- Ticket / PR: ENG-CORE-003 (domain model and cumulative validation) · branch `j2/eng-core-003-model`; no commit, push or PR created in this session.
- Author(s): Codex (gpt-6.1-sol, medium effort) as author of code, documentation, log and verification, under the direction of J2 · 2026-10-10 (America/Guayaquil).
- Role: specifier and implementer of the project's own domain model; no oracle operator or similarity auditor activity.
- Contract implemented: crate `engine-model`, registered in the workspace and depending only on `engine-core` by path. `Model` preserves nodes with coordinates, dimension and DOFs; two-node members with section and law references; sections with generic properties; laws with name and parameters; supports; nodal load cases; case combinations with factors; nodal masses and inertias. Constructor and insertions without implicit validation. `validate(&Model) -> Vec<Diagnostic>` accumulates duplicates (`ENG-E-0002`), broken references (`ENG-E-0003`), orphans (`ENG-E-0004`), zero length (`ENG-E-0005`), non-finite inputs (`ENG-E-0008`) and invalid DOFs by dimension (`ENG-E-0011`). Exclusively reuses the existing catalog without modifying `engine-core`. Does not implement any element `kind` or structural calculation method.
- Permitted sources used: No external sources; only internal project documents: `README.md`, `docs/CLEAN_ROOM.md`, `clean-room/forbidden_identifiers.txt`, `crates/engine-core/src/lib.rs`, `crates/engine-core/src/catalog.rs`, `crates/engine-core/Cargo.toml`, workspace configuration, `provenance/TEMPLATE.md` and `provenance/ENG-CORE-001.md`. Standard structural mechanics and own design (Cartesian coordinates, right-handed triad, DOFs, connectivity, Euclidean distance, face resultants, linear combination of actions). Context messages, data design, documentation and tests are original; no code from structural-kernel or motor-geotecnico was reused.
- Own derivations: [axis and sign convention](../docs/CONVENCION_EJES.md) and [module rustdoc](../crates/engine-model/src/lib.rs). Local x axis from i to j; in D2, y = (-x_y, x_x, 0) and z = x × y. The horizontal X-Y convention with Z vertically upward is approved (see below); the 2D elevation convention is still pending. Own sign table by face and end, with positive tension. **APPROVED by Jonathan 2026-10-10 22:52: X-Y horizontal, Z vertically upward (the Engine governs).** Distance is obtained through `hypot` accumulation; length less than or equal to `1e-12` in the coordinate unit is considered zero, without relative scaling. That check is skipped if ends are missing or coordinates are non-finite. No stiffness formulation is derived and no law is evaluated.
- Third-party code copied: none. Rust standard library and the internal `engine-core` contract only; no other dependencies.
- Validation: N/A (analytical: N/A · book: N/A · published example: N/A)
- Justification: this ticket implements no numerical method, law or stiffness formulation (see «Own derivations»), so there is no result to compare with an analytical solution, book or published example and no external source is cited (`docs/CLEAN_ROOM.md` §4 and §7). What is validated are the topological invariants of the domain model (identifier uniqueness, references, orphans, zero length, finite values, DOFs by dimension) through own unit tests and doctests. Details: own unit tests and doctests, without external datasets. Ten unit tests in the new crate: empty model and valid D2/D3 models; duplicates preserved in all eight collections; all broken references (including nodal actions); orphans even with support/load/mass; identical ends and inclusive tolerance bounds; NaN and both infinities in all numeric fields; invalid DOFs in nodes, supports, loads and masses; nominal references for entities without a connected representative; deterministic order and first declaration for duplicates; large and small finite distances. Each error is checked with code, responsible entity and exact context; reuse of catalog messages and severity is verified. Seven doctests: valid model and one for each of the six required codes, checking entities. Local verification with `~/.cargo/bin/cargo fmt --check`, `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`, `~/.cargo/bin/cargo test` and `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`: all successful. Full workspace: 17 unit tests and 15 doctests successful. `#![deny(missing_docs)]` mandatory in the new crate. Additional local check for absence of forbidden identifiers in ticket files and diff without modifications to protected paths. No remote CI execution.
- Comparison with oracle (sibling repo, informational): no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: pending; human review and Jonathan's approval pending.

## Scope decisions and traceability

Identifiers are compared within each class, with exact equality and without
normalizing names. Collections are lists to avoid losing duplicates during
construction. Diagnostic order follows input classes and positions;
queries resolve a duplicate node to its first declaration. Parameter,
DOF and term lists are not entities with their own identifier, so
no additional uniqueness is imposed on them.

A node is orphaned if it does not appear at any member end, even when
that member has another invalid reference. DOFs are checked against the corresponding
node's `Dim`. Stability, physical positivity, D2 Z = 0
geometry and member dimensional compatibility are not checked in this ticket.

Without extending `EntityRef`, sections point to the first member using them;
sections without a member use `Law(id_sección)` as a nominal reference.
Supports and masses point to their node, even if absent. Combinations point to
the case of the responsible term; duplicates point to the first case and empty
combinations use `Case(id_combinación)` as a nominal reference. A nominal
reference does not declare the existence of another entity. All diagnostics include
the class and exact name/id of the original entity in `hint`. These rules
are documented and tested so the consumer can interpret the context.
