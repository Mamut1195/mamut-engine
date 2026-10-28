# Clean-room log · ENG-SFC-007 · MAM-94

- Author: Codex, 2026-10-15, America/Guayaquil.
- Role: implementer, under the user's request.
- Branch: `j2/eng-sfc-007`, based on main; initial HEAD
  `800ef3f` (ENG-CORE-010-FU2). PY-002/API-001 dependencies present.
- Sources: user request; local engine-api code, contracts, fixtures and
  tests; bindings/python scaffold; existing manifests and provenance;
  PyO3 0.25.1 type and conversion documentation in the local Cargo
  registry. Rust, Python and maturin tools already installed. No network.
- Own derivations: adaptation of the Rust facade to Python, conversion of
  JSON values, exceptions with context, atomic token and deferred progress.
  No numerical implementation or direct access to solvers from the binding.
- Third-party code copied: none. No new registry dependencies;
  engine-api, serde_json 1.0.154 and PyO3 0.25.1 available offline are reused.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- No declared license changes or legal texts added; no changes to
  docs/CLEAN_ROOM.md, .github/**, clean-room/** or vendor/**. No publication.
- External clean-room reviewer: PENDING (J4).

## Contract and decisions

`_native` exposes validate/run/capabilities/schema/explain/version; __init__.py,
_native.pyi and py.typed make up the installed typed API. validate returns an
object with diagnostics; topological defects remain diagnostics,
as in engine-api. run returns exactly the canonical results/1 document
for ok/warning. version returns the facade's engine_version/contracts
object, replacing the scaffold string; __version__ retains the string.
The engine implementation is not duplicated.

Boundary and analysis failures produce EngineError or its subclasses
InvalidInputError, AnalysisError, CancelledError and InternalEngineError. The
first error's code, entities and cause are preserved; run failures also retain
full document and diagnostics. Boundary errors use the same
mapping as engine-api::run (API-E-INPUT or ENG-E-0009). Unpublished names/codes
are also treated as API-E-INPUT, with their original cause.

run and validate use py.allow_threads. CancelToken contains an Arc of the
engine-api atomic token; another Python thread may request cancellation. The engine
checks the existing token between stages/iterations/modes; no Python code is run
in iterations. Cancellation is cooperative: it does not interrupt an indivisible solve,
eigensolver, assembly or parse. The token is not reset.

The Rust progress callback accumulates events only if progress was requested.
After run, the GIL is reacquired and they are delivered to Python in order, including
events preceding a failure. This is deferred progress; cancel() from that callback
arrives after analysis and only affects future token uses. Using another
thread to cancel the current analysis is documented. Callback errors are
propagated, unless there is an engine error, which takes priority.

Operations use a common catch_unwind/AssertUnwindSafe barrier that
maps failures to InternalEngineError (ENG-E-0012, empty entities, explicit
cause). PyO3 also retains its barrier in generated code. The Rust
test injects resume_unwind from an engine-api callback and checks that the
barrier contains it and the next analysis works. No
panic!, unwrap() or expect() is introduced in the library.

The cdylib unit harness remains disabled: this environment has Python but
no libpython3.13 for linking an embedded interpreter (the initial attempt
confirmed it). The Rust integration test directly shares the containment
module without linking PyO3, and Python tests exercise the installed
extension. No failure hooks are added to the public API.

Python remains outside default-members, and all its dependencies are native
only. On wasm32 the crate is empty, without engine-api or PyO3. The DAG
check confirms that no engine crate depends on the binding or PyO3;
neither the workspace nor engine crates are changed.

## Real case and tolerances

Own engine-api fixture:
`crates/engine-api/fixtures/input/valid/cantilever-linear-static.json`.
The Rust `facade_reference` example calls engine_api::run directly and emits
the document without adding newlines. unittest compares the bytes and SHA-256
of the installed wheel against that reference in three runs. Transport
tolerance: zero; floats are not reserialized in Python.

results/1 SHA-256 (Rust and Python):
`9f0674dd8a18a094b99e16c803eefa89b992085632c379432c221105e1282bb2`.

Analytical check independent of transport: D2 cantilever with P=12,
L=4, E=200, I=3. Integrating EI y''=-P(L-x) with zero displacement and rotation at
the restrained end gives y(L)=-PL³/(3EI); equilibrium gives R=P and M=PL. Relative
and absolute tolerances 1e-12 allow f64 rounding of the small well-conditioned
system; they do not compensate for differences between Python and Rust.

Python tests: public contract and distribution, capabilities/schemas/catalog,
all invalid input fixtures, diagnostics with entities, mechanism,
warnings and advanced methods, deferred/reentrant callback delivery, callback
and type errors, permanent precancelled token, cancellation from another
thread with GIL released. The latter uses valid JSON whitespace below the
8 MiB limit to keep work in the engine while the thread acquires the GIL;
it does not alter limits or introduce synchronization sleeps.

## Verification and reproduction

Exported environment: PATH=/home/box/.cargo/bin:$PATH,
RUSTUP_TOOLCHAIN=stable (rustc 1.99.0),
CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine,
CARGO_NET_OFFLINE=true and PIP_NO_INDEX=1. CPython 3.13.5/Linux;
maturin 1.15.0. No network during building, installation or tests.

- cargo fmt --check: OK.
- cargo clippy --offline --workspace --all-targets -- -D warnings: OK.
- cargo clippy --offline -p mamut-engine-py --all-targets -- -D warnings: OK.
- cargo test --offline --workspace: OK, 546 tests, 0 failures/ignored;
  includes the Rust binding containment test.
- RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK.
- cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown: OK.
- cargo metadata and cargo tree for WASM: isolation verified.
- maturin build --offline: OK, cp310-abi3-manylinux_2_34_x86_64 wheel.
- pip --no-index installation in a new venv and unittest: OK, 12 tests.
- git diff --check and bash clean-room/check.sh origin/main: OK.

Full reproduction in `bindings/python/README.md`. Main commands,
from the root with the preceding environment:

```bash
mkdir -p /tmp/eng-sfc-007/wheels
cargo run --offline --quiet -p mamut-engine-py --example facade_reference -- \
  crates/engine-api/fixtures/input/valid/cantilever-linear-static.json \
  > /tmp/eng-sfc-007/cantilever-rust.json
/workspace/venv-maturin/bin/maturin build --offline \
  --manifest-path bindings/python/Cargo.toml --out /tmp/eng-sfc-007/wheels \
  --interpreter /workspace/venv-maturin/bin/python
python3 -m venv /tmp/eng-sfc-007/venv-clean
/tmp/eng-sfc-007/venv-clean/bin/python -m pip install --no-index --no-cache-dir \
  /tmp/eng-sfc-007/wheels/*.whl
MAMUT_RUST_REFERENCE=/tmp/eng-sfc-007/cantilever-rust.json \
  /tmp/eng-sfc-007/venv-clean/bin/python -m unittest discover \
  -s bindings/python/tests -v
```

Validation was run from /tmp using the absolute test path;
the import comes from site-packages of the new venv. Local artifacts, reference
and logs in `/tmp/eng-sfc-007/`. Windows/macOS and other
interpreters were not checked. The stable ABI does not constitute a tested platform matrix.

## r41 · Integration of main (#117) and facade smoke

- J4 r41 request, MAM-94, 2026-10-15. Merge already started in
  `j2/eng-sfc-007`: HEAD `9557d8c`, MERGE_HEAD `b0c39f4` (origin/main).
  The merge is retained, without aborting or rebasing it.
- Sources: both sides of the README conflict, local PY-007 (#117) scripts and
  tests, facade implementation and tests (#119), manifests and
  Rust `facade_reference` example. No network or third-party code copied.
- Integrated README: API, EngineError, CancelToken, GIL and progress on completion,
  Python tests, reproducible building, artifacts, matrix and rollback.
  The README uses variables for local paths. Rollback accepts the old
  wheels' string and the current facade's dict.
- Smoke adapted to `version()["engine_version"]`; it also compares the full native
  dict and contracts with COMPATIBILITY. `SMOKE.json` preserves the
  scalar version and records contracts. COMPATIBILITY and the SBOM's
  `mamut:contract_status` property declare the full facade
  validate/run/capabilities/schema/explain/version.
- The script first generates an independent reference through engine-api and
  exports `MAMUT_RUST_REFERENCE` when running wheel tests in the new
  venv, with PATH limited to its binaries. Regressions check the dict,
  incompatible versions/contracts, visible Rust and contract status of both
  artifacts. No numerical changes specific to this integration.

Environment: Rust 1.99.0, maturin 1.15.0, CPython 3.13.5, Linux x86_64.
`CARGO_TARGET_DIR` exported in all checks; Cargo offline/locked,
`CARGO_NET_OFFLINE=true`, `RUSTUP_AUTO_INSTALL=0` and `PIP_NO_INDEX=1`.
Tools were already installed; no dependencies were downloaded.

Reproduction from the root, selecting local binaries and a new output:

```bash
export PATH="$RUST_TOOLCHAIN_BIN:$PATH"
export RUSTUP_TOOLCHAIN=1.99.0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 PIP_NO_INDEX=1
export MATURIN="$MATURIN_BIN"
bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/python-dist-r41"
(cd "$CARGO_TARGET_DIR/python-dist-r41" && sha256sum --check SHA256SUMS)
python3 -m unittest discover -s scripts/tests
cargo fmt --check
cargo clippy --offline --locked --workspace --all-targets -- -D warnings
cargo clippy --offline --locked -p mamut-engine-py --all-targets -- -D warnings
cargo test --offline --locked --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace
cargo check --offline --locked --workspace --all-targets --target wasm32-unknown-unknown
bash clean-room/check.sh origin/main
```

r41 results:

- Full script: OK. Two release builds with independent targets, identical
  wheel `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.
  SHA-256: `0841090a11bc0030644bdd95188bcbe19b1b6089f1f96d25ee4095e6a80bc0e2`.
- Portability of all ZIP members and metadata, unchanged LICENSE/NOTICE,
  artifacts and SHA256SUMS verification: OK.
- Smoke in a new venv, without rustc/cargo/rustup in PATH: OK; engine/binding 0.1.0,
  engine-input/1 and results/1 contracts. Python tests on that wheel: 12 OK,
  including exact equality with the Rust reference, errors and cancellation/GIL.
- `python3 -m unittest discover -s scripts/tests`: 13 OK.
- fmt, both clippy checks with warnings denied, workspace test (570 tests,
  0 failures/ignored), doc with warnings denied and wasm32 check with all
  targets: OK.
- `git diff --check` and `bash clean-room/check.sh origin/main`: OK.

Local evidence: `$CARGO_TARGET_DIR/python-dist-r41/` and logs
`$CARGO_TARGET_DIR/{python-wheel,fmt,clippy-workspace,clippy-python,test-workspace,doc,wasm,clean-room}-r41.log`.
Artifacts are not versioned. No changes to licenses, docs/CLEAN_ROOM.md,
.github/**, clean-room/** or vendor/**. No publication. Other platforms and
interpreters remain pending; external J4 clean-room review pending.
Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

## ENG-PY-003 follow-up · 2026-10-17

Codex gpt-6.1-sol high adds private transport of engine-api canonicalization and
JCS fingerprinting for the typed Python facade, without physical computation in Python.
Review against current main, wheel tests, contract compatibility
and the clean-room declaration for this extension are recorded in
[ENG-PY-003](ENG-PY-003.md). Clean-room reviewer: **PENDING (J4)**.
