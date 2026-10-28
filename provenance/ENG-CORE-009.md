# Clean-room log · ENG-CORE-009

- Ticket / PR: MAM-22 · ENG-CORE-009 · fixture/1 format and benchmark runner.
- Author(s): Codex gpt-6.1-sol high, 2026-10-17 (America/Guayaquil).
- Role: specifier and implementer.
- Contract implemented: fixture/1, fixture-report/1 report and pluggable Target
  on engine-input/1 → results/1; without modifying the engine's numerical methods.
- Permitted sources used: docs/CLEAN_ROOM.md §§4, 7, 8 and 10;
  local engine-api contracts, tests and examples; local ABI patterns
  in engine-api/examples/determinism_wasm.rs and scripts/check-determinism.mjs.
  Own format and analytical derivations in this note, without external sources.
- Citation of the own note: Codex. *Casos estructurales cerrados, derivación propia
  para ENG-CORE-009 (MAM-22)*, 17 de octubre de 2026, cases identified
  below and in crates/engine-validation/fixtures/*.json. Session author:
  Codex gpt-6.1-sol high.
- Own derivations: this note and source.derivation of each fixture.
- Third-party code copied: none. Already present dependencies are used:
  engine-api, serde 1.0.229 and serde_json 1.0.154. Cargo.lock changes only
  the three engine-validation edges; it introduces no packages or versions.
- Validation: own analytical; no result calculated by another engine.
- Comparison with oracle (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." I also did not consult CalculiX or LAPACK code or results,
  oracle reports, sibling repo content or external resources. Work without network.
- Clean-room reviewer: **PENDING (J4)**.

## Derivaciones, unidades y signos

Artificial SI quantities to check the algebra of the linear elastic model;
they do not represent a physical design or large-displacement validation.
Original data, without copying published tables, figures or listings. JSON
numbers are obtained by evaluating these expressions, never by copying engine output.

1. **axial-bar**. L=4 m, E=200 Pa, A=2 m², P=12 N in +X. By equilibrium
   N=P. Since ε=u′ and σ=Eε=N/A, integrating u′=P/(EA), u(0)=0, gives
   u(L)=PL/(EA)=0,12 m. The support reacts −12 N; Uy=0 due to the guides.
2. **axial-series**. E=200 Pa, P=12 N, L₁=2 m, A₁=2 m², L₂=3 m,
   A₂=1 m². The unloaded intermediate node imposes N₁=N₂=P. Each segment
   has kᵢ=EAᵢ/Lᵢ. Compatibility: u₂=P/k₁=0,06 m;
   u₃=P(1/k₁+1/k₂)=0,24 m. Reaction −12 N and N₂=12 N. The spring
   series is realized with bars, supported by the current API.
3. **cantilever-point**. L=4 m, E=200 Pa, I=3 m⁴, P=12 N toward −Y.
   Clamped-end moment PL and shear P by equilibrium. Curvature
   v″=−P(L−x)/(EI), integrated with v(0)=v′(0)=0, gives
   v′(L)=−PL²/(2EI)=−0,16 rad and v(L)=−PL³/(3EI)=−32/75 m.
   Ry=12 N, Mz=48 N·m and zero axial displacement.
4. **simply-supported-uniform**. L=4 m, E=200 Pa, I=3 m⁴, w=6 N/m
   toward −Y. Equilibrium and symmetry give R₁=R₃=wL/2=12 N.
   M(x)=wx(L−x)/2; M′=0 at L/2, Mmáx=wL²/8=12 N·m.
   Integrating EI v″=M with v(0)=v(L)=0 yields
   v(x)=−wx(L³−2Lx²+x³)/(24EI), and v(L/2)=−5wL⁴/(384EI)=−1/30 m;
   v′(L/2)=0 by symmetry. Two elements of length 2 m with consistent uniform
   load and a central node allow comparison of the exact nodal deflection;
   both recovered moment maxima are also compared.
5. **spatial-cantilever**. L=4 m, E=200 Pa, Iy=3 m⁴, Iz=5 m⁴, P=12 N
   toward −Z. Integrating EI_y z″=−P(L−x) with a clamped end yields
   z(L)=−PL³/(3EI_y)=−32/75 m. r×F=+PL e_y requires My=−48 N·m at
   the support, with Rz=12 N. Zero Y displacement due to decoupling.

## Tolerances and provenance

For each value, |actual−esperado| ≤ max(absolute, relative·|esperado|).
Defaults relative=absolute=1e-10 for linearStatic; 1e-7 for iterative
methods (pDelta, modal, responseSpectrum). The absolute floor is in the
quantity's SI units and protects zeros; there is no division by the expected value.
Finite overrides require explicit own justification. The five closed-form cases
use 1e-10, without calibrating tolerances against external results.

source is required and its fields must not be blank. The contract
accepts an analytical note, a book with author/title/edition/page and a published example
with citation and locator. source and provenance are inspected recursively,
including keys and capitalization, to reject mentions of OpenSees,
Stabileo and CalculiX before invoking targets. Mentions in code and negative
tests express only that rule; they contain no external results.

## Design and scope

The library compares against the declared oracle; targets only
execute the input. In-process native Target via engine-api and
Node/WebAssembly adapter without npm, wasm-bindgen, WASI, imports or network. The ABI transports
bytes through scalar functions, safe storage in Mutex and
engine-api input limits; without unsafe, pointers or new external dependencies.
An **exact** exception for fixture_wasm.rs is added to the source guard,
limited to no_mangle and extern C functions, like the existing examples.
It does not relax unsafe permissions or affect engine hash calculations.

All existing models, methods and results are preserved. No changes
to root Cargo.toml, docs/DETERMINISMO.md, clean-room/**, docs/CLEAN_ROOM.md,
.github/workflows, CODEOWNERS or vendor/. The mamut-engine CLI is not a dependency:
its adapter will be added after the ENG-SFC-002 merge. The runner measures
accuracy; it introduces no performance measurements or nondeterministic timings.

## Local evidence

Offline execution on 2026-10-17: rustc 1.99.0 (b940084d7 2026-09-28),
Node v22.19.0, one CARGO_TARGET_DIR per worktree and usual Cargo
flags. Checks finished with code 0:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK, no warnings. |
| `cargo test --workspace --offline` | **639 passed / 0 failed / 0 ignored**, including the 9 new tests. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK. |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK. |
| `bash clean-room/check.sh origin/main` | OK, including new files through a temporary index under CARGO_TARGET_DIR; real index intact. |

`cargo test -p engine-validation --test fixtures --offline`: 9 passed / 0 failed.
Coverage includes the complete JSON fixture inventory, missing/empty source,
incomplete variants, recursive veto before targets, defaults, absolute floor,
justified override, discrepancies/missing paths/incorrect status, invalid
formats, continuation after failures and report repeatability/order.

fixture_wasm was compiled as wasm32-unknown-unknown cdylib and the runner executed
with Node: **5 fixtures × 2 targets = 10 passed rows**, with 23 checks
per target (46 total). Text and JSON outputs were exercised; two complete
JSON runs were byte-for-byte identical (`cmp`). Node verified zero
imports and ABI range errors. Cargo tests do not require Node.

The determinism_native and determinism_wasm examples and
scripts/check-determinism.mjs were run in **debug and release**, with ten runs per
case/target/profile: **16 native=wasm hashes**, debug=release native outputs by
`cmp` and the 16 hashes identical to the current docs/DETERMINISMO.md table.
MAMUT_ENGINE_GIT_COMMIT undefined, as required by that reproducible reference.

Logs and reports retained under CARGO_TARGET_DIR:
`eng-core-009-{clippy,tests,doc,wasm-check}.log`,
`fixture-report-009-{a,b}.json`, `fixture-report-009.txt`,
`eng-core-009-native-{debug,release}.txt` and
`eng-core-009-determinism-{debug,release}.log`.

## Scope incident and pending items

Agents must remain within their worktree, including when searching for instructions.
During ENG-CORE-009, Codex ran `find .. -name AGENTS.md -print` and traversed
sibling directories, including opensees, calculix and stabileo directories, outside the
indicated scope. It only listed names, without opening content: none of
those files or external code, data, reports or results was opened. The scope
of subsequent searches was corrected to the worktree; there was no network access or
reading of forbidden sources.

External pending items: **J4** clean-room review and future CLI adapter after
the **ENG-SFC-002** merge. No technical checks remain pending
for this deliverable.

## MAM-22 follow-up · issue #142

Codex gpt-6.1-sol high, 2026-10-17 (America/Guayaquil), implementer role.
Exactly five JSON fixtures are required and the local paths in
this log and the engine-validation README are replaced by one CARGO_TARGET_DIR per
worktree. The scope rule and previous incident are made explicit above.
Sources: ticket request, local files and provenance/TEMPLATE.md;
no new derivations, numerical data or third-party code copied.
Validation: existing own analytical oracles, without external comparison.
Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para
este ticket." I also did not consult CalculiX or LAPACK code or results.
Clean-room reviewer: **PENDING (J4)**.

Offline follow-up verification: fmt, Clippy with `-D warnings`, workspace
tests (**655 passed / 0 failed / 0 ignored**, including the 9 fixture tests),
rustdoc with `-D warnings`, wasm32-unknown-unknown check and
`bash clean-room/check.sh origin/main`: all OK. The 16 hashes match
docs/DETERMINISMO.md and between native and wasm in debug, ten runs per case/target.
Follow-up logs under CARGO_TARGET_DIR/mam-22-142/.
