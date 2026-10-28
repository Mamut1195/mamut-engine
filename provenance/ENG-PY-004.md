# Clean-room log · ENG-PY-004

- Ticket / PR: MAM-82 · ENG-PY-004 · PR #156; local follow-up without commit or publication.
- Author(s): Codex gpt-6.1-sol high · **2026-10-17** (America/Guayaquil).
- Role: implementer.
- Implemented contract: tabular projection of `results/1` in the
  `mamut_engine` package over PyO3; lists and optional NumPy, without modifying the engine.
- Permitted sources used: exclusively in-house sources from the checkout:
  `bindings/python`, `crates/engine-api` (schemas, fixtures and serialization),
  `crates/engine-analysis/src/results.rs` (order, units and stations),
  DOF contracts of the in-house frame/truss elements, in-house CLI,
  `docs/adr/ADR-003-python-v1.md`, `provenance/ENG-PY-003.md`,
  `docs/DETERMINISMO.md`, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md` and
  local build/artifact scripts. No external sources.
- Own derivations: no new physics. The benchmark only replicates and
  renumbers the in-house analytical cantilever fixture. Rust references for
  the 16 existing fixtures are independent of the binding. The mixed data
  imported in the tabular test verify representation, not physics.
- Third-party code copied: none into the repository. No changes to Cargo.toml,
  Cargo.lock, vendor, clean-room, workflows or CODEOWNERS; no new Rust
  dependencies, unsafe or new GIL releases.
- Validation: exact comparison of documents with Rust engine-api and tables
  against their original rows; existing in-house analytical oracles.
- Oracle comparison (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult code or results of CalculiX or LAPACK,
  or outputs/reports of other engines. I work exclusively offline.
- Clean-room reviewer: **PENDING (J4)**.

## Decisions and scope

[ADR-004](../docs/adr/ADR-004-numpy-python.md) records the optional extra
`numpy>=1.23`, lazy import and `MissingNumPyError` (`PY-E-NUMPY`).
`Results.to_arrays()` and `Results.to_numpy()` return `ArrayResults` with
an independent complete document, status and blocks per case, combination,
mode, spectral response and envelope. Each matrix carries shape, integer IDs,
station index, DOF, unit per column and axes. End forces are
separated by element type. The document's own orders are preserved.

NumPy receives lists with `asarray(dtype=float64, order="C")`, explicit reshape
and `.copy(order="C")`, including empty `(0,k)`. Owned memory, without views
or backend borrows; they outlive Results and do not share storage
between exports. Textual status, nullable values, extrema and convergence
remain in the complete document. NaN is not used as a null marker.
Only representations are transformed and shapes/finiteness/identity checked;
all physics and analysis arithmetic remains in Rust. No numerical
results are changed and the GIL is not optimized.

## Offline build and tests

Rust 1.99.0 stable, maturin 1.15.0, CPython 3.13.5/Linux x86_64,
NumPy 2.5.4 and pytest 9.1.1. `CARGO_NET_OFFLINE=true`, Cargo `--offline`,
`PIP_NO_INDEX=1`, no PyPI publication. `CARGO_TARGET_DIR` points to the
authorized directory, without versioned binary artifacts.

```bash
export PATH="$RUST_TOOLCHAIN_BIN:$PATH"
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1
MATURIN="$MATURIN" bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/mam82-dist"
python3 -m venv --system-site-packages "$CARGO_TARGET_DIR/mam82-consumer"
"$CARGO_TARGET_DIR/mam82-consumer/bin/python" -I -m pip install \
  --no-index --no-deps --no-cache-dir "$CARGO_TARGET_DIR/mam82-dist/"*.whl
```

Pytest was provisioned in another local environment, outside the system
Python's site. It was exposed through a `.pth` only inside the temporary
venv, pointing to `$PYTEST_SITE_PACKAGES`; without downloading or modifying
external packages. NumPy comes from the system site. The
16 in-house references were generated in `$CARGO_TARGET_DIR/mam82-references` with the
Rust `facade_reference` binary. Installed wheel execution:

```bash
export MAMUT_RUST_REFERENCES="$CARGO_TARGET_DIR/mam82-references"
export MAMUT_RUST_REFERENCE="$MAMUT_RUST_REFERENCES/cantilever-linear-static.json"
export PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 PYTHONDONTWRITEBYTECODE=1
"$CARGO_TARGET_DIR/mam82-consumer/bin/python" -I -B -m pytest \
  -q -p no:cacheprovider bindings/python/tests
```

- Artifacts: **25 passed / 0 failed** in `scripts/tests`, run by the
  builder and `scripts/python-wheel-artifacts.py` for inspection and comparison.
- Isolated stdlib consumer without NumPy: **38 passed / 0 failed** (unittest).
- Temporary consumer with NumPy: **59 passed / 0 failed**, plus
  **241 subtests passed** (pytest, includes the 38 basic ones).
- Simulated NumPy absence in a new process: **6 passed / 0 failed,
  1 skipped**, plus 14 subtests; `pytest.importorskip` cleanly skips the
  optional module. The basic API and typed error are verified without NumPy.
- Boundary cases: NaN/±Inf, values unrepresentable as f64, incorrect/empty
  shapes, input ndarray outside the JSON contract, nonconsecutive maximum
  IDs, duplicates/unknowns, mixed element types,
  units/labels, stale, failed status, mutation and copy lifetime.

Wheel `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`:
**two independent release builds, same SHA-256**:
`bf353bdd5e1c2a2e3caa5f9bf7006d70cd59b990b1268db4291cb41bdb3f6b8f`.
Evidence: `$CARGO_TARGET_DIR/mam82-dist/REPRODUCIBILITY.json`, `SHA256SUMS`,
SBOM/compatibility/smoke and `$CARGO_TARGET_DIR/mam82-dist.log`.

## Performance

Plan and reproduction: [boundary benchmark](../docs/PYTHON_BOUNDARY_BENCHMARKS.md).
`scripts/benchmark-python-boundary.py` uses the installed wheel and Rust example
`boundary_benchmark`; CLI and baselines are built in release offline before measuring.
Three complete runs with sizes 1/16/64, warm-up 2 and 7 samples per
metric. Each run checks Rust/CLI/facade equality. There are no CI thresholds.

Medians of the third run, **ms**, shared machine; exploratory local
evidence, without performance promises or CPU isolation:

| Replicas | Rust API | CLI/process | PyO3/JSON | Facade run | Parse Results | Lists | NumPy |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.1334 | 5.0433 | 0.2247 | 1.2682 | 0.5406 | 0.5031 | 0.5117 |
| 16 | 1.3964 | 4.4718 | 1.7600 | 8.4416 | 5.0583 | 2.8345 | 2.9979 |
| 64 | 14.3334 | 22.2011 | 14.1618 | 41.9114 | 21.9639 | 10.9600 | 11.8592 |

Differences of medians may be negative due to noise. CLI includes startup,
I/O and envelope; exporting includes stale/fingerprint, document copy and tables. That
time is not identified with solve and overlapping phases are not added. Reports with
samples/versions/bytes/hashes: `$CARGO_TARGET_DIR/mam82-benchmark.json`,
`$CARGO_TARGET_DIR/mam82-benchmark-2.json` and
`$CARGO_TARGET_DIR/mam82-benchmark-3.json`. Neither RSS nor cold import is measured.

## Mandatory checks and determinism

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: **658 passed / 0 failed / 0 ignored**,
  including doctests (104 suites).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `cargo check --workspace --offline --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK.
- `git diff --check`: OK.

Without setting `MAMUT_ENGINE_GIT_COMMIT`, the examples
`determinism_native`/`determinism_wasm` were run in debug and release, with
`scripts/check-determinism.mjs`: **16/16 hashes nativo=wasm**, ten runs
per case/target/profile; native debug/release `cmp` OK. Explicit comparison
with the `docs/DETERMINISMO.md` table: **16/16 identical** in both profiles.
Logs and outputs in `$CARGO_TARGET_DIR/mam82-determinism-*`; workspace
checks in `$CARGO_TARGET_DIR/mam82-{fmt,clippy,test,doc,wasm}.log`.

Pending: J4 clean-room review and platforms/interpreters other than those
tested. No need to modify protected paths or publish artifacts.

## OpenCode review follow-up · PR #156

- Author: Codex gpt-6.1-sol high · **2026-10-17** (America/Guayaquil).
- Role: implementer. Clean-room reviewer: **PENDING (J4)**.
- Sources: exclusively the in-house code, contracts, fixtures and scripts
  cited above, plus review comments supplied by the user.
  No external sources, new dependencies or physical derivations.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult code or results of CalculiX or
  LAPACK or results of other engines. I work offline; third-party code
  copied: none. External oracle comparison: no.

The original `ModuleNotFoundError` is preserved as `__cause__` through
`raise MissingNumPyError(...) from exc`, with a regression in a new process.
Tabular validation explicitly rejects `isinstance(v, bool)`; True and
False are tested in displacements, reactions, end forces, station
forces and position directly in the exporter, preventing the JSON
schema from hiding this check.

Regressions mutate all exported matrices and check the document,
snapshot (`to_dict` and `to_json`), a previous second export and a subsequent
call. They check distinct objects/lists/rows and, with NumPy, absence of
shared memory and preservation of copies after releasing Results.
`numpy_test.py` becomes `test_numpy.py`, with `unittest.TestCase`, subtests and
`unittest.skipUnless`; it preserves the 16 Rust fixtures, the four invalid input
ndarray, empty shapes, cancellation, IDs, labels, units,
float64, C order, owned memory and finiteness. No pytest dependency or
changes to the official builder.

The following evidence corresponds to the follow-up and supersedes the historical
pytest counts and wheel hash in the preceding sections.

### Follow-up reproduction and results

Rust 1.99.0, maturin 1.15.0, CPython 3.13.5 and NumPy 2.5.4. All offline:

```bash
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:?define un directorio de target}"
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1 PIP_DISABLE_PIP_VERSION_CHECK=1
export MATURIN="${MATURIN:-maturin}"
bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/mam82-followup-dist"

mkdir "$CARGO_TARGET_DIR/mam82-followup-references"
for fixture in crates/engine-api/fixtures/input/valid/*.json \
    crates/engine-api/tests/fixtures/determinism/*.json; do
  cargo run --offline --locked --quiet -p mamut-engine-py \
    --example facade_reference -- "$fixture" \
    > "$CARGO_TARGET_DIR/mam82-followup-references/${fixture##*/}"
done
export MAMUT_RUST_REFERENCES="$CARGO_TARGET_DIR/mam82-followup-references"
export MAMUT_RUST_REFERENCE="$MAMUT_RUST_REFERENCES/cantilever-linear-static.json"

python3 -I -m venv --system-site-packages "$CARGO_TARGET_DIR/mam82-followup-with-numpy"
python3 -I -m venv "$CARGO_TARGET_DIR/mam82-followup-without-numpy"
for variant in with-numpy without-numpy; do
  consumer="$CARGO_TARGET_DIR/mam82-followup-$variant"
  "$consumer/bin/python" -I -m pip install --no-index --no-deps --no-cache-dir \
    "$CARGO_TARGET_DIR/mam82-followup-dist/"*.whl
  "$consumer/bin/python" -I -B -m unittest discover -s bindings/python/tests -v
done
```

- Official builder: **25 passed / 0 failed** in scripts/tests and **40 passed /
  0 failed / 3 skipped** out of 43 tests discovered in the consumer without NumPy.
- Venv with NumPy 2.5.4: **43 passed / 0 failed / 0 skipped**; all three NumPy
  methods run through the same `unittest discover` as the pipeline.
- Venv without NumPy (actual absence verified): **40 passed / 0 failed / 3 skipped**.
  The typed error and its original cause are checked in both environments.
- Wheel: two independent release builds, identical content and SHA-256:
  `56f8d9680ccf0615dba67aea2fc702f432ad5d987ff21a7096d26fdd8f567ab5`.
  Evidence: `$CARGO_TARGET_DIR/mam82-followup-dist/REPRODUCIBILITY.json`,
  `SHA256SUMS`, SBOM/compatibility/smoke and `mam82-followup-wheel.log`.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: **658 passed / 0 failed / 0 ignored**,
  including doctests (104 suites).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `cargo check --workspace --offline --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main` and `git diff --check`: OK.
- Determinism: in-house examples `determinism_native`/`determinism_wasm` and
  `scripts/check-determinism.mjs`, debug and release, ten runs per case:
  **16/16 hashes nativo=wasm**, debug/release `cmp` OK and **16/16 identical** to
  `docs/DETERMINISMO.md` in both profiles, without `MAMUT_ENGINE_GIT_COMMIT`.

Persistent logs and references under `$CARGO_TARGET_DIR/mam82-followup-*`.
Local changes without commit; no changes to Rust, Cargo.toml/Cargo.lock, vendor,
clean-room, docs/CLEAN_ROOM.md, workflows or CODEOWNERS. Only clean-room
review **J4** and the already pending matrix of other platforms remain pending.
