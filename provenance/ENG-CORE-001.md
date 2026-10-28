# Clean-room log · ENG-CORE-001

- Ticket / PR: ENG-CORE-001 (base types, errors and diagnostics) · Draft PR from branch `j2/eng-core-001-engine-core`
- Author(s): Codex (gpt-6.1-sol, medium effort) as code author, under the direction of J2 (Grok Bot agent); log and verification by J2 · 2026-10-10 (America/Guayaquil)
- Role: implementer
- Contract implemented: base types of `engine-core` (section 4.1 of the project's internal architecture, the Engine's own design document: `Real`, ids, `Dof`, `Dim`, `ContractId`, `EntityRef`, `Severity`, `Diagnostic`, `EngineError`, `EResult`) and `ENG-E-xxxx` code catalog. Does not implement any element `kind`.
- Permitted sources used: no external sources. The types are generic data structures (numeric identifiers, degree-of-freedom enumerations, error codes) defined by the project's own design. The degree-of-freedom names (Ux, Uy, Uz, Rx, Ry, Rz) are standard structural mechanics notation, independent of any particular tool. Message texts and explanations are original writing.
- Own derivations: not applicable (there are no equations or numerical methods in this ticket).
- Third-party code copied: none. No external dependencies (Rust standard library only).
- Validation: own unit tests and doctests (catalog uniqueness and `ENG-E-NNNN` format, nonempty message and explanation, `explain` for an unknown code, `Display`, DOFs per dimension); `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo doc` locally (remote CI does not run because of GitHub Actions billing).
- Comparison with oracle (sibling repo, informational): no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: pending (OpenCode comments on the verdict in the PR; formal approval belongs to the responsible person).
