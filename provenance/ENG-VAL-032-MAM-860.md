# Clean-room log · MAM-860

- Ticket / PR: MAM-860 · VAL-032 E3-pdelta: change internal threshold;
  request and round 2 of 2026-10-20. No commit or push.
- Author(s): Codex gpt-6.1-sol high, date 2026-10-17 (America/Guayaquil),
  author metadata requested for this ticket.
- Role: specifier and implementer of analytical reference and tests;
  no delegation or solver changes.
- Contract implemented: `val032-fixture/1` fixture, E3 v1.1.3;
  `e3-pdelta-reference/1` reference, unchanged `engine-input/1` input.
- Permitted sources used: ticket request; `docs/CLEAN_ROOM.md`;
  `provenance/TEMPLATE.md`; E3 fixture and own VAL-032 documentation;
  `crates/engine-api/tests/fixtures/pdelta_diaphragm/reference.py` and its
  previous own analytical reference; public contracts and repo tests.
  Standard consistent geometric matrix: McGuire, W., Gallagher, R. H. y
  Ziemian, R. D., *Matrix Structural Analysis*, 2.ª ed., Wiley, 2000, cap. 9,
  análisis de segundo orden. Bibliographic citation; no external consultation or
  copying of listings, tables or figures. The full derivation is our own.
- Own derivations: 3D Euler energy, algebraic Hermite integration,
  XY constraints and condensation of unloaded interior nodes, in
  [`e3_pdelta_reference.py`](../docs/validation-report/fixtures/val032/_ref/e3_pdelta_reference.py)
  and [README, converged reference](../docs/validation-report/fixtures/val032/README.md#referencia-convergida-propia-de-e3--mam-860).
- Third-party code copied: none. NumPy 2.5.4 preinstalled,
  BSD-3-Clause, numerical dependency only; neither its implementation
  nor that of its dependencies was consulted. No package is installed or Cargo changed.
- Validation: own analytical validation. Compressed cantilever closed-form solution and
  exact Gauss integration as self-tests; E3 with 1/8/16 elements per column.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni
  derivados para este ticket." Nor CalculiX or LAPACK code/results,
  oracle outputs, `provenance/ENG-BENCH-001-SMOKE-VPS.md` or
  `/workspace/audit/restricted/`. Offline session, no web tools.
- Clean-room reviewer: **PENDING (J4)**.

## Method, criteria and reproducibility

An own independent script is written from
`U=1/2∫[EAu′²+GJφ′²+EIy w″²+EIz v″²+N(v′²+w′²)]dx`, without using Rust code
or Engine responses to calculate the reference. Linear axial/torsional
and cubic transverse Hermite interpolation; `Kg=N∫H′ᵀH′dx`, standard
coefficients `N/(30L)`. The tension-positive axial force is frozen at the linear
gravity solution: `N=EA/L·(ug_bx−ug_ax)`. Both bending planes receive Kg,
also in beams; no geometric torsion terms are added.

Each column between stories is subdivided into 1, 8 and 16 elements. Interior
nodes have no loads or constraints; the Schur complement
preserves the refined system exactly. Elastic condensation reproduces
the original matrix to 1e-10; linear gravity and N are invariant under refinement.
`Tᵀ(K+Kg)T q=TᵀF_EX` is solved directly, without adding G deformation.
Positive stability and projected residual <1e-10 are checked per mesh.
The node counts are 72/492/972 and element counts 145/565/1045.

The JSON stores axial forces per member, the three meshes, linear/P-Δ drift,
amplification, linear/P-Δ θ, residual and stability. It includes a citation and
SHA-256 fingerprints of the fixture and script. One BLAS thread pins reductions in this
environment. `gen.py` regenerates the reference and records both hashes in the index;
the previous `reference.py` command delegates to the canonical script, without duplicating data.
Rust does not need Python to run the checks.

The tests run E3-pdelta and linear EX, reconstruct CM Ux from the
master and Rz, and subtract consecutive stories (zero base). The amplification
denominator is the Engine's own linear response. Per story:

- P-Δ amplification and drift vs both 8/16 references: ≤1e-3 relative.
- Both quantities vs independent one-element reference: ≤1e-9 relative.
- Internal convergence 8 vs 16: ≤2e-6 relative in both quantities.
- One-sided diagnostic: `0 ≤ amp·(1−θ_lineal)−1 ≤ 5e-2`.
- The previous θ and drift checks at 1e-4 are tightened to 1e-9.

Only the internal amplification threshold against `1/(1−θ)` is replaced;
the 5e-3 against Frame3DD and all other tolerances unrelated to the replacement
are preserved exactly. No external information is used to choose values.
E3 rises from v1.1.1 to v1.1.2; the threshold wrapper changes, with no numerical
changes to the model or runs. Historical r3 evidence does not change.
No library sources, dependencies, clean-room, workflows,
CODEOWNERS, vendor or `docs/DETERMINISMO.md` are modified.

## Round 2: current statuses and hash correction · 2026-10-20

Work is carried out on `de9d984`, with MAM-860 `fa917d5` and the merge of
`origin/main` `c6dd075` (#188). The mandatory target
`CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine-e3crit` exists and is
used for all Rust checks with stable 1.99.0, always offline.

The eight modal/spectrum runs are executable after Guyan #188 (#184);
E3-pdelta is executable after #185/#186. All nine return `warning` and only
`API-W-DIAPHRAGM-RECOVERY` (#84), without errors. `gen.py` updates their
`runs[].info.estado/status_engine/causa/evidencia_engine` and the
`info.estado_engine` summary. `bench_translate.py` removes resolved gaps
ENG-E-0006/ENG-E-0007; the index declares seven compatible models without gaps,
with the actual warnings. `validate_bench.py` uses the same documentation convention.

The version of fixtures with corrected metadata or prose is incremented:
E1/E2/E4 1.1.1 → 1.1.2, E2b 1.0.1 → 1.0.2 and E3 1.1.2 → 1.1.3.
E5 retains bytes and version; E5b goes from 1.0.1 to 1.0.2 due to format prose.
The `include_str!`, the path
read by the E3 script and the READMEs are updated. The E3 reference changes only
fixture name and hashes; all its numerical values are retained.

Regeneration also corrects `index.json.engine_verification_r3.fuentes`.
The **three** hashes inherited from `origin/main` (`val032_runs.rs`,
`pdelta.rs` and `modal.rs`) were outdated with respect to the executable
sources (J4 follow-up on #192): MAM-860 refreshed the first two in
`fa917d5` and this round recalculates them after the merge and new
assertions, including `modal.rs`, whose fingerprint in main did not match either.

| Source | Current checked SHA-256 |
| --- | --- |
| `crates/engine-api/tests/val032_runs.rs` | `79e146a84320e50ba9f2c8ba6a0cdb99f2443016ee6cc6ad1ae5de36e664bb1d` |
| `crates/engine-analysis/src/pdelta.rs` | `6c6cd48ed3b6dbc579f811ad79554dce265c8b2a4be35e019926d1389701b673` |
| `crates/engine-analysis/src/modal.rs` | `f787fec25fb3e06b5315e1edfd295a373af6826f1e47e231f912d81fdc273b0a` |

The obsolete main fingerprints were `d0089414…d6503` for `val032_runs.rs`,
`d869fd4b…3a2` for `pdelta.rs` and `994113d7…69be` for `modal.rs`.
The `fuentes` fingerprints identify the
current checked sources; they do not turn `engine_r3.json` into a new
execution: that file and `calibrate_r3.json` retain their historical bytes.

`val032_runs` now compares `status_engine` with `results/1.status`, requires
`warning` without errors in the twelve runs, validates executable statuses of the
nine corrected runs, warning codes/counts, fixture summary, statuses
and absence of gaps in the index. It also verifies hashes of fixtures and
Rust sources: an outdated fingerprint fails again in Rust, without Python.

New E5/E5b warnings: with current main E5-combinaciones returns `warning`
with `API-W-DIAPHRAGM-RECOVERY` (#84) and ten `API-W-COMBINED-PRESCRIBED` (#85),
and E5b-combinaciones `warning` with ten `API-W-COMBINED-PRESCRIBED` (#85).
These are documentation warnings about coverage (#84) and weighting of prescribed values in
combinations (#85), without errors; `val032_runs` pins their codes and counts.

### Compatibility with the oracle output guard

The first run of `clean-room/check.sh origin/main` rejected inherited
prose: it interpreted the requirement of ten significant digits and configuration
options as numerical assignments from an oracle, and flagged historical
diagnostics in E3. Precision instructions are restated in words,
with the same requirement, and J4 r2 diagnostics are referred to the existing README.
In the generator, a comment about an own area and configuration options
are separated from program names. No option, model,
input or numerical threshold/tolerance changes; the historical diagnostic is not used
to implement or adjust anything either. E5b requires a bump due to reproduced prose.
In E5b the justification formula references the `tol` field instead of
repeating its literal: the same threshold applies to both relative/absolute branches.
The guard and its rules remain intact.

## Actual session checks

- Two analytical self-tests: **2 passed, 0 failed**. For the compressed cantilever
  at P/Pcr=0.2, relative errors for 1/8/16 elements:
  3.339274063e-4 / 1.026667527e-7 / 6.431808530e-9.
- Exact regeneration: **28 identical files in two generations**,
  excluding Python caches; hashes of fixtures, tools, evidence,
  seven bench-model and three Rust sources correct.
- **12 exact translations** except for bijections; the twelve original and
  translated inputs pass the local Engine schema, without network retrieval.
  The seven bench-model retain their bytes against HEAD. Validation
  of external schema #183, outside the authorized worktree, is not repeated.
- Structural comparison against HEAD: **7 models, 12 inputs and all
  numerical thresholds identical**: metrics, comparisons, thresholds, classes and
  tolerance statuses retained. Only format justification prose in
  E3/E5b and the reference to the historical diagnostic in E3 change. Numerical
  E3 reference values identical; E5 and both r3 evidence files identical
  byte for byte.
- `val032_runs` and `pdelta_diaphragm`: **2 passed, 0 failed**, explicit
  rerun with `--nocapture`; also included in the workspace suite.
- VAL-032: **12/12 runs warning, 0 errors, 0 invalid**, with **31 reviewed
  warnings** (11 from #84 and 20 from #85). **120/120 periods** compared;
  maximum relative error ≈5.928590951498e-14 (original tolerance 1e-5).
- E3-pdelta: **14 iterations**, convergence `ok`; maximum relative error
  of amplification/drift ≈4.733811682e-4 against 8 elements,
  ≈4.745757265e-4 against 16 and ≈3.599931695e-11 against one element.
  One-sided diagnostic per story between ≈1.242178060e-2 and ≈3.010418287e-2.
  Reference convergence 8 vs 16: maximum ≈1.195124056e-6.
- `cargo fmt --all -- --check`: **OK**, exit 0.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`:
  **OK**, exit 0.
- `cargo test --workspace --offline`: **718 passed, 0 failed, 0 ignored**,
  including doctests, in 113 suites (including empty suites).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`:
  **OK**, exit 0.
- `cargo check --workspace --offline --target wasm32-unknown-unknown`:
  **OK**, exit 0.

- Determinism: **16/16 hashes from `docs/DETERMINISMO.md` identical**, without
  editing that table, in native/wasm and debug/release; ten runs per
  case/target/profile, `git_commit: null`. `cmp` debug/release: **OK**.

- `bash clean-room/check.sh origin/main`: **Clean-room: OK**, exit 0,
  with temporary index and objects inside the worktree to include the six
  renamed fixtures through `git add -N`. Real index unchanged.
- `git diff --check origin/main`, with those same files included: **OK**.
- Final diff: **18 files**, including six renamed by version;
  no changes to Rust libraries, dependencies, clean-room, workflows,
  CODEOWNERS, vendor or determinism table. Everything remains uncommitted.

Actual logs and documents exported in `target/mam860-r2/` (ignored by Git).
