# Clean-room log · ENG-VAL-005

- Ticket / PR: MAM-105 · ENG-VAL-005, free oracles as an independent
  baseline. Changes without commit or push.
- Author(s): Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: specifier and implementer of metadata and validation tests.
- Contract implemented: V1 oracle inventory and guard over the existing
  tables; `fixture/1` retains its contract and explicitly declares the
  limits it previously obtained by default.
- Permitted sources used: ticket request; `docs/CLEAN_ROOM.md`, §4/§8;
  `provenance/TEMPLATE.md`; local derivations ENG-VAL-001 (statics and
  D1–D12), ENG-GEO-002 (beam on foundation), ENG-GEO-006 (series, convolution and
  r32 multilayer eigenfunctions), ENG-CORE-009 (`fixture/1`) and ENG-BENCH-002
  (beam-column). No external sources were consulted or network used.
- Own derivations: the previous notes are retained and linked from
  [ORACULOS.md](../crates/engine-validation/ORACULOS.md), with equations,
  conditions and parameters already versioned. No physical method is introduced.
- Third-party code copied: none. No new dependencies or changes
  to Cargo.toml/Cargo.lock, vendor, workflows or clean-room rules.
- Validation: own analytical solutions and existing numerical tests;
  source, citation, justification, tolerance and coverage guard per case.
  Unchecked bibliographic references: PENDING, detailed in the matrix.
- Comparison with oracle (sibling repo, informative): no. No file
  from that repository was opened or engine data or results incorporated.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket. Tampoco consulté ni usé código o resultados de CalculiX o
  LAPACK, ni salidas de OpenSees o Stabileo."
- Clean-room reviewer: PENDING (J4).

## Decisions and scope

91 engine-validation cases are inventoried: 73 from ENG-VAL-001, six Winkler,
seven consolidation (including nontransformable multilayer) and five JSON
`fixture/1`. Each case has an equation or expected values, source and tolerance;
the existing numerical suites continue to traverse all their rows.
There are no artifacts identified as ENG-VAL-002 in the inspected tree.
The independent analytical comparison of the existing published P-Delta
benchmarks is also documented, without copying their tables or figures.

The `tests/oracles.rs` guard checks all tables and fixtures, rejects
missing sources or sources from prohibited engines, verifies finite and positive limits
and detects omitted or duplicate cases or outdated tolerances in the matrix.
Its negative controls remove source, expected values or tolerance from each
JSON and test prohibited sources and invalid limits. The existing fixture test
checks that the embedded inventory matches all JSON in the
directory; no file can be left unexecuted.

The five JSON add absolute=relative=1e-10, identical to the previous default.
Consolidation moves the existing limits to shared metadata used
by the native/wasm runners: settlement/s∞<0.0015, relative U<0.005,
pressure/100<0.004 and relative flow<0.03. No expected value,
mechanical input, formula, acceptance operator or effective tolerance changes.

ENG-REL-001 depends on ENG-VAL-005 and does not require ENG-VAL-003. OpenSees remains
only an additional external comparison, never the sole oracle or a CI
or release requirement. Its files and results do not enter the repository.

## Offline verification

The provisioned stable toolchain, `CARGO_NET_OFFLINE=true`,
`PIP_NO_INDEX=1` and an isolated build directory `$CARGO_TARGET_DIR` are used.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 661 passed, 0 failed, 0 ignored, including doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK, including the three new files |

The targeted engine-validation suite passed 24 tests, including the three
new ones, and traverses the 91 cases. Comparison of the five JSON against HEAD
confirmed equality of `input`, `expected`, `source` and the other previous fields;
only the explicit tolerance identical to the previous default was added.

The real Git index is read-only. The first clean-room invocation did not
count the new provenance because it was still an untracked file.
The same guard was run with `GIT_INDEX_FILE` pointing to a temporary index
under `$CARGO_TARGET_DIR/eng-val-005-evidence/`, initialized from HEAD and with
entries for ORACULOS.md, tests/oracles.rs and this log. Thus the diff
inspects their full contents. The real index was not modified and the guard
was not altered to obtain the result.

Determinism: the 16 hashes published in `docs/DETERMINISMO.md` match
exactly those calculated; ten runs per case in native and wasm,
debug profile, `MAMUT_ENGINE_GIT_COMMIT` unset. The existing
`scripts/check-determinism.mjs` script verified equality between targets, repeatability
and zero external imports. The hash document remains intact.

Additional wasm execution, offline and with the existing runners: six
Winkler cases, seven consolidation plus convergence and five `fixture/1`, the
latter in both runner targets (ten rows passed). This executes the
harnesses and documents whose metadata changed, in addition to compiling them.
Check logs, hashes and reports only in
`$CARGO_TARGET_DIR/eng-val-005-evidence/`; no results are added to the repo.

`git diff --check`: OK. No changes to numerical libraries, determinism
inputs, Cargo.toml/Cargo.lock or protected files.

## Seguimiento de revisión OpenCode · PR #155

- Author: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: specifier and implementer of metadata and guards.
- Sources: only the local derivations cited above and the follow-up
  request; no network or consultation of external bibliography.
- Clean-room declaration: I did not consult or use code or results from OpenSees,
  Stabileo, CalculiX or LAPACK; I did not copy third-party tables, figures or code.
- Clean-room reviewer: PENDING (J4).

Winkler and consolidation citations now point to specific sections of the
own derivations; CQC retains the D11 algebraic evaluation as its source.
Unchecked attributions are gathered in «Optional bibliographic reference
(to be checked)» in ORACULOS.md and are not the acceptance oracle.

The inventory is calculated by summing the lengths of the tables and fixtures,
including the individual multilayer spectral case. The guard compares names
in both directions and reports which cases are missing or extra in the matrix and in the
tables/fixtures, as well as rejecting duplicates. Each row must link a heading
anchor existing in the corresponding provenance file. The five
JSON link the section of their derivations; transformable multilayer links its
specific development.

Negative controls include links without anchors, nonexistent anchors/files,
inventory differences, citations containing the word PENDIENTE (case-insensitive)
and empty justification or justification consisting only of spaces/tabs/newlines
in each fixture/1. «Independiente» remains valid text.
Each JSON now justifies the default according to the scale of the results, stiffnesses,
conditioning and zeros of the case. The library contract is not modified.

Comparison against HEAD: the five JSON retain all their data, including
inputs, results and limits; only citation and tolerance.justification change.
The three modified tables retain all their numeric literals. No libraries,
dependencies, protected files or published hashes are modified.

### Follow-up verification

Stable toolchain rustc 1.99.0, `CARGO_TARGET_DIR=/workspace/.cargo-target/me-val5`,
`CARGO_NET_OFFLINE=true` and `PIP_NO_INDEX=1`. All run without network:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 665 passed, 0 failed, 0 ignored, including doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK, without an alternative index |

Oracle guard: seven tests passed, four added in the follow-up;
the calculated inventory matches the current 91 rows. The 16 recalculated
hashes match `docs/DETERMINISMO.md` exactly. Native/wasm debug
comparison with the existing runners and `scripts/check-determinism.mjs`:
ten runs per case/target, bit-for-bit equality and zero external imports.
`MAMUT_ENGINE_GIT_COMMIT` unset, as in the published table.

Local logs and summaries in `$CARGO_TARGET_DIR/eng-val-005-followup/`.
`git diff --check`: OK. Uncommitted changes in the working tree.

## Pending items

J4 clean-room review; human cross-check of the bibliographic references and
locators marked PENDING in ORACULOS.md. Numerical acceptance uses
the local derivations and does not depend on completing those attributions.
