# Clean-room log · ENG-CORE-011

- Ticket / PR: MAM-25 · ENG-CORE-011, Rust methods guide and cubic pilot.
- Author(s): Codex gpt-6.1-sol high, **2026-10-20** (America/Guayaquil).
- Role: specifier and implementer; no oracle operator or similarity audit.
- Contract implemented: `uniaxial/1`, kind `elastic.cubic`, method 0.1.0.
- Permitted sources used: docs/CLEAN_ROOM.md §§3–8 and 10; provenance/TEMPLATE.md;
  engine-core (types, catalog, math); engine-contracts (traits, types and
  conformance, ENG-CORE-007); engine-laws (laws, factories and local tests);
  engine-validation (README, schema and axial-bar fixture, ENG-CORE-009);
  provenance/ENG-CORE-007.md, provenance/ENG-CORE-009.md and docs/DETERMINISMO.md.
  clean-room/check.sh was read as a local checking tool, without changes.
- Own derivations: Codex gpt-6.1-sol high. *Elasticidad cúbica conservativa*,
  2026-10-20, [guide §5](../docs/GUIA_METODOS.md#5-piloto-reproducible-elasticidad-cúbica),
  equations (1)–(3) and own analytical table. No external bibliographic references.
- Third-party code copied: none. No new dependencies or manifest/lock changes.
- Validation: own analytical, `check_uniaxial_law` kit, native typed fixture
  with source/citation and ten rows of tension, unloading and compression.
- Comparison with oracle (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
  I also did not consult CalculiX, code_aster,
  LAPACK or derivative code, results or reports; no sibling repos or external sources were opened.
  Work without network, offline Cargo and PIP_NO_INDEX=1.
- Clean-room reviewer: **PENDING (J4)**.

## Pilot decisions and execution

Existing laws were inspected first: linear elasticity,
multilinear (includes bilinear curves) and kinematic bilinear plasticity
(includes b = 0). A smooth cubic curve that none represents
exactly with C > 0 was chosen. The linear limit C = 0 serves as a check.

The guide and its record with expected values were written before the pilot code.
Its steps were then followed: module in engine-laws, validated private
parameters, pure trait without history, descriptor, factory, exports, conformance
tests and native analytical fixture. The pilot exercises the guide in the same
authoring session; it does not constitute an independent test with another implementer.

Derivation: W = Eε²/2 + Cε⁴/4, σ = dW/dε and K = d²W/dε²; E > 0, C ≥ 0
in Pa. q = (Cε)ε, σ = (E+q)ε and K = E+3q are evaluated, requiring finiteness
at every operation. The fixture uses E = 8 Pa, C = 16 Pa and binary fractions
with expected values evaluated directly in the record. Comparison limit
max(1e-12 Pa, 1e-12·|esperado|); FD h = 1e-6, central error C h².
No copied tables, figures or outputs; no calibration against other programs.

Registration replicates the current mechanism of the other laws: `LawDescriptor`,
`UniaxialFactory` and `pub use`, without a global registry or engine-api changes.
The fixture is typed and native because ENG-CORE-009 `fixture/1` requires
engine-input/1 → results/1 and the API does not expose these nonlinear laws. The
JSON template is reserved for methods accessible through that boundary; the five
ENG-CORE-009 JSONs and their closed inventory are not altered.

No changes to clean-room/**, docs/CLEAN_ROOM.md, CODEOWNERS,
.github/workflows, vendor/ or docs/DETERMINISMO.md. No unsafe,
library unwrap/expect/panic, I/O, threads or global state were added.

## Local evidence

Offline execution on **2026-10-20**, Rust 1.99.0
(`b940084d7 2026-09-28`), Node v22.19.0, usual Cargo flags and
`MAMUT_ENGINE_GIT_COMMIT` undefined to reproduce the reference hashes.
All commands finished with exit 0:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK, no warnings. |
| `cargo test --workspace --offline` | **665 passed / 0 failed / 0 ignored**, including doctests. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK. |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK; compilation, no pilot execution in wasm. |
| `bash clean-room/check.sh origin/main` | OK, including new files. |

The pilot has **6 native tests passed / 0 failed** and one new doctest
included in the workspace total. Tests cover the own ten-row
fixture, conformance for C = 0/16/1600 Pa, descriptor/factory/Send + Sync,
state ownership, odd symmetry of σ and even symmetry of K, linear limit, extreme
representable scales and rejection of invalid parameters/inputs/histories.
All six product and sum overflow paths are exercised.

Additional cross-check with the `determinism_native` and `determinism_wasm` examples
and `scripts/check-determinism.mjs`, in **debug and release**, with ten runs
per case/target/profile: **16 native = wasm hashes**, all identical to the
current docs/DETERMINISMO.md table. The 160 native rows of each profile
were also compared with that table and debug/release outputs with each other:
byte-for-byte identical. No reference hash was updated.

The clean-room check used temporary `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY`
under `$CARGO_TARGET_DIR`, with original objects only as alternates:
the environment's `.git` is read-only. New paths were included with
that index, without modifying the real index or creating commits.

Logs retained under `$CARGO_TARGET_DIR/eng-core-011/`:
`fmt.log`, `clippy.log`, `tests.log`, `doc.log`, `wasm-check.log`,
`clean-room.log`, `wasm-build-{debug,release}.log`,
`native-{debug,release}.{log,txt}` and `determinism-{debug,release}.log`.
Also checked: local links in the new documents, absence
of absolute local paths in deliverable documentation, diff without
whitespace errors and absence of changes to protected paths.

## External pending items

Clean-room review **PENDING (J4)**. Pilot wasm execution is not
part of the required native validation; compiling wasm does not establish it.
No mandatory technical checks remain pending. Changes uncommitted.
