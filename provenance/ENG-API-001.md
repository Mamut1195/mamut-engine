# Clean-room log · ENG-API-001

- Ticket / PR: ENG-API-001 · engine-api facade · increment 1 · branch j2/eng-api-001.
- Author(s): Codex gpt-6.1-sol high under the direction of J2, 2026-10-14 America/Guayaquil.
- Role: implementer.
- Implemented contract: engine-input/1 (parsing and topological validation); results/1 (reserved static schema); metadata, catalog, canonical JSON, progress and cancellation.
- Permitted sources used: local ticket /workspace/fu/tickets/ENG-API-001.txt and engine-cli-py-plan.md (data context); docs/CONVENCION_EJES.md, sections «Global axes and dimension» and «Actions, displacements and rotations»; own APIs of engine-core, engine-model and engine-analysis::results; JSON Schema Draft 2020-12 (declarative structure, without network consultation).
- Original derivations: DTO-to-domain mapping and canonicalization described in crates/engine-api/README.md; no mechanical derivations or solver changes.
- Third-party code copied: none. Dependencies serde 1.0.229 (derive) and serde_json 1.0.154 (float_roundtrip) from the local registry, only in engine-api; Cargo offline.
- Validation: contract and binary64 representation oracles; own catalog/domain diagnostics; determinism, -0.0, cancellation before starting and between all stages. No external numerical oracle.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

Scope: run is not implemented in this increment; README details
linearStatic → results/1 recovery, SHA-256, combinations and other pending methods.

Offline checks with `CARGO_TARGET_DIR=/workspace/.cargo-target` and the
installed stable toolchain (`PATH=/home/box/.cargo/bin:$PATH`, Rust 1.99.0):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 431 tests (15 new engine-api tests), 0 failed/ignored.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- Cargo metadata: engine-api in members/default-members, sole consumer of serde/serde_json; engine versions aligned at 0.1.0.
- JSON Schema documents: JSON structure and local references verified, without network.
- `bash clean-room/check.sh origin/main`: OK on the diff prepared for commit (including new files); human clean-room review pending from J4.

## Increment 2a · 2026-10-14

- Ticket / PR: ENG-API-001 · draft PR #88 · branch j2/eng-api-001.
- Author: Codex gpt-6.1-sol high under J2, 2026-10-14 America/Guayaquil.
- Role: implementer. Clean-room reviewer: PENDING (J4).
- Contract: run(engine-input/1, cancel, progress) → canonical results/1;
  linearStatic, nodal/all-member recovery and combinations;
  Outcome/Status and codes 0/1/2/3/4/5/6 reserved/70. SHA-256 of canonical input.
- Permitted sources: ticket ENG-API-001.txt and CLI Interface section of
  engine-cli-py-plan.md as data; previous engine-api contract and README;
  own APIs of engine-model, engine-analysis (StaticLinear, CaseResults::recover,
  combine), engine-assembly and own frame/truss factories. No network.
- Original derivations: cantilever equilibrium R=P, M=PL; integration of
  EI y''=−P(L−x) with y(0)=y'(0)=0 → y(L)=−PL³/(3EI). Uniformly loaded supported beam:
  symmetry and equilibrium → R=wL/2; M(x)=wx(L−x)/2, root M'=0 at L/2 → wL²/8.
  Bar: EAδ/L=P → δ=PL/EA. Combination: component-by-component sum
  1,2D+1,6L; combination prescribed values warned about according to #85, repeated ones rejected.
- Consistency #69: a MemberLoad collection per case/member feeds both
  load_vector and MemberRecovery; all members are recovered. Union of
  critical stations for combination compatibility, without changing the solver.
- Open follow-ups: #85 emits API-W-COMBINED-PRESCRIBED and warning whenever
  there are prescribed values in combined output; #84 emits API-W-DIAPHRAGM-RECOVERY
  and warning. The facade smoke test does not declare domain coverage closed.
- Third-party code copied: none. Existing serde/serde_json dependencies
  and sha2 0.10.9 (local registry); Cargo offline. No code from other engines,
  no external oracles or network access.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Validation: 16 new facade tests, based on the original derivations
  above and contracts; determinism and SHA with permuted keys/-0.0,
  cancellation before starting and at the seven boundaries of two cases, Unsupported
  methods, invalid input, mechanism, warnings, repeated settlements,
  multiple members and combination station compatibility.

## Completion of increment 2a · J4 r22 · 2026-10-14

- Ticket / PR: ENG-API-001 · draft PR #88 · branch j2/eng-api-001;
  review of WIP 19e297f after merge eed09df from origin/main.
- Author: Codex gpt-6.1-sol high under J2, 2026-10-14 America/Guayaquil.
- Role: implementer. Clean-room reviewer: PENDING (J4).
- Permitted sources: /workspace/fu/papi2a.txt, ENG-API-001.txt and
  engine-cli-py-plan.md (scope data); own workspace implementation and contracts,
  engine-api README and schemas. No network queries.
- J4 r22: public constants documented for UTF-8 bytes, nodes, members,
  cases, total nodal/member loads, combinations, depth and
  auxiliary array entries. Own lexical inspection without recursion or input
  tree, before serde parsing and the model/solver; errors name
  the limit and maximum, with InvalidInput / API-E-INPUT / exit 2.
- Numbers: token check before serde; negative integers limited
  to i64 and nonnegative ones to u64, without silent degradation to float. f64 overflow,
  including -1e400, rejected. Output canonicalization over Value, retaining
  -0.0 normalization and determinism, without applying input limits to results/1.
- License: literal AGPL-3.0-only in engine-api, like bindings/python.
- Capabilities: run true for linearStatic and combinations; pDelta, modal and
  responseSpectrum remain Unsupported (3). Warnings from #84/#85
  and rejection of repeated settlements are retained; those follow-ups are not closed.
- Additional original derivations: D3 cantilever along X with force in Z,
  EI_y z''=−P(L−x) and integration with z(0)=z'(0)=0 → z(L)=−PL³/(3EI_y);
  r×F=+PL e_y → reaction −PL e_y. Axial D3 bar: EAδ/L=P → δ=PL/EA,
  equilibrium R=−P. No solver modification or external numerical oracle.
- Additional tests: eight for inclusive boundaries and exceeding limits before
  parsing, escaped keys, strings, UTF-8, accumulated loads, giant
  integers and f64 overflow, prior cancellation; two analytical tests for frame3d and
  truss3d. Numerical rejection is extended in complete run inputs.
- Third-party code copied: none. Already available local registry dependencies
  (serde, serde_json and sha2); all Cargo checks offline.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

Final checks on the full increment, including bindings/python
as a workspace member, with `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true` and Rust 1.99.0 (`PATH=/home/box/.cargo/bin:$PATH`):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 462 tests, 0 failed/ignored;
  engine-api: 41 (15 facade, 8 limits, 18 run).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `git diff --check`: OK. No panic/unwrap/expect in the engine-api library;
  no changes to docs/CLEAN_ROOM.md, .github/** or clean-room/**.
- `bash clean-room/check.sh origin/main`: OK on the diff prepared for
  commit, including new files. Human review PENDING (J4).

## Increment 2c · 2026-10-14

- Ticket / PR: ENG-API-001 · draft PR #88 · branch j2/eng-api-001.
- Author: Codex gpt-6.1-sol high under J2, 2026-10-14 America/Guayaquil.
- Role: implementer. Clean-room reviewer: PENDING (J4).
- Contract: run of linearStatic, pDelta, modal and responseSpectrum → canonical
  results/1 and SHA-256; capabilities publishes run:true for all four methods.
  One analysis per input, explicit mechanical options, without code factors.
- Permitted sources: own APIs and documentation of engine-analysis (pdelta,
  modal, spectrum), engine-assembly, engine-loads::mass and engine-linalg;
  previous engine-api contract and increment requirements received from J2.
  No sibling repositories or external numerical oracles were consulted.
- Original modal derivation: bar equilibrium EAδ/L=F → k=EA/L;
  m u''+ku=0 → ω²=k/m, f=ω/(2π), T=2π/ω. For 1 DOF, φ=1/√m,
  Γ=√m, M*=m and participation=1. Two independent bars: eigenvalues
  20/80 s⁻² with m=5 kg per DOF; participation of 50% per mode.
- Original spectral derivation: q=Γ Sa/ω², u=φq=Sa/ω² and V=m Sa;
  test constants Sa=3 m/s², k=100 N/m, m=5 kg → u=0,15 m, V=15 N.
  In two independent oscillators, each DOF responds to its sole mode;
  weighted directional sum with factors 1/2 multiplies the response by 3.
- Original P-Delta derivation: EI y''''+P y''=0, y(0)=y'(0)=0,
  y''(L)=0 and EI y'''(L)+P y'(L)=−H. Integrating and solving constants:
  y(L)=(H L/P)(tan α/α−1), α=L√(P/EI);
  A=y(L)/(HL³/(3EI))=3(tan α−α)/α³. First pole α=π/2 →
  Pcr=π²EI/(4L²). P/Pcr=0,2: comparison with 1/(1−P/Pcr) at 0,4%
  due to first-pole approximation; mesh error decreases between 4/8 elements
  and with 8 is <1e−5 against the exact solution, iterative tolerance 1e−10.
- Masses: uses explicit nodal masses or masses_from_case/system_case (including
  equivalent member forces), simultaneous sources rejected; user's factor and
  gravity. Does not incorporate rho automatically or condense massless
  DOFs. Suction retains ENG-E-0010 / invalid / 2, without additional tolerance.
- Open follow-ups: #86 always emits API-W-MASS-PROJECTION when
  converting load to mass, also on failure; warns of rejection due to rounding with
  gravity not aligned with axes. #78 emits API-W-PDELTA-INERTIA when reporting
  2×2 blocks in the inertia (or unknown metadata), also in unstable.
  Pivot metadata is published without changing copysign or certifying its test.
  #84(c): does not use CaseResults::recover with P-Delta; only independent lateral
  displacements and reactions, without member forces. Retains
  previous warnings #84/#85. No follow-up is declared closed.
- P-Delta: registers geometric producers per member; gravity with member
  loads is rejected as Unsupported because the producer recovers N from
  deformations without member load correction. Convergence retains
  options, inertia and metrics per iteration; NotConverged/Unstable → code 4.
- Progress/cancellation: cooperative callbacks in the domain after gravity and
  each P-Delta iteration, after the eigensolver and each modal mode, and for each
  spectral mode/direction. Each solve/eigensolver is indivisible. Cancellation
  removes all partial responses; no changes to accepted state.
- Third-party code copied: none. Existing serde/serde_json/sha2 from the local
  registry; own engine-loads dependency added to engine-api. No network.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Tests: analytical oracles above; deterministic SHA and bytes; all
  cancellation boundaries, including two modes/directions; invalid inputs,
  suction, masses from member loads, warning with oblique gravity and
  2×2 warning even on failure. Warning test #78 does not test or close copysign.

Final verification of increment 2c, with `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true` and Rust 1.99.0 (`PATH=/home/box/.cargo/bin:$PATH`):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 474 tests, 0 failed/ignored;
  engine-api: 53 (12 dynamic, 15 facade, 8 limits, 18 run).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- JSON Schema: documents parsed and all local references checked,
  without network or new schema-validation dependencies.
- `git diff --check`: OK; no panic/unwrap/expect in the engine-api library.
- `bash clean-room/check.sh origin/main`: OK on the diff prepared for commit,
  including new files. No changes to docs/CLEAN_ROOM.md, .github/** or
  clean-room/**. Human clean-room review PENDING (J4).
