# Clean-room log · ENG-PY-002

- Ticket / PR: ENG-PY-002 · branch `j2/eng-py-002`, from `origin/main`.
- Author(s): Codex gpt-6.1-sol high under J2's direction, 2026-10-14 America/Guayaquil.
- Role: implementer.
- Implemented contract: local Python scaffold; `version() -> str` with the `engine-core` version. No new analysis contracts.
- Permitted sources used: user's request; `/workspace/fu/tickets/ENG-PY-002.txt` and `engine-cli-py-plan.md` as data; repository manifests, Rust core, root README and `provenance/TEMPLATE.md`; local Cargo, PyO3 and maturin tools. No network access.
- Own derivations: adaptation of the core's version constant to the Python module; no numerical algorithms.
- Third-party code copied: none.
- Validation: compilation, static analysis and local installation/import smoke test; no numerical oracle required.
- Oracle comparison (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Scope and base

Local base updated by fast-forward to `origin/main`
`d1249f1d533d923957ead3626fe2f43cf5f6fca1`, without network access. The scaffold only depends
on `engine-core` and, on targets other than `wasm32`, PyO3 0.25.1
(`abi3-py310`; `extension-module` only through maturin). It does not use
`engine-api`. The version comes from the `engine-core` manifest, not from a
binding literal. A single workspace `Cargo.lock`.

The binding belongs to the workspace and is excluded from `default-members`, which
lists the 14 original crates. `--workspace` bypasses that default selection;
since the correction on 2026-10-14, WASM includes the empty, documented Python crate
without compiling PyO3. No changes to
`docs/CLEAN_ROOM.md`, `.github/**` or `clean-room/**` relative to the base.

AGPL-3.0-only metadata and MAMPRO SAS commercial note; legal text pending in
ENG-GOV-004. No PyPI publication; Jonathan's approval is required.
Clean-room review remains pending J4. Local commit, no push.

## Initial verification (before the WASM correction)

All Rust checks used `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH` and
`RUSTUP_TOOLCHAIN=stable`. The local toolchain reports rustc 1.99.0
(`b940084d7 2026-09-28`), matching the version pinned by the repository;
`stable` selects the existing installation without downloading another name.
The `wasm32-unknown-unknown` target was already installed. There was no network access.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK, exit 0 |
| `cargo clippy -p mamut-engine-py --offline -- -D warnings` | OK, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, exit 0 |
| `cargo test --workspace` | OK, 416 tests/doctests, 0 failed, 0 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | OK, exit 0 |
| `cargo check --workspace --target wasm32-unknown-unknown` | Fails, exit 101: `pyo3-ffi` 0.25.1 imports `libc` types absent on that target (`wchar_t`, `size_t`, among others) |
| `cargo check --offline --target wasm32-unknown-unknown` | OK, exit 0: uses the 14 default members |
| `cargo check --offline --workspace --exclude mamut-engine-py --target wasm32-unknown-unknown` | OK, exit 0 |

The failure of the literal WASM command is recorded: `--workspace` includes the
binding despite `default-members`. No WASM stubs are added or core changes made
to hide it. Numerical crates do not depend on the binding; default
WASM compilation preserves their isolation.

`cargo metadata --offline --format-version 1` and DAG checks:
15 members, 14 by default, binding excluded from the latter; direct dependencies
exactly `engine-core` and `pyo3`, no crate depends on the binding,
no `engine-api` package; core/binding versions equal to `0.1.0`.
No `panic!`, `unwrap()` or `expect()` in the added library code.
The cdylib disables the Rust harness and its doctests; the function is tested through
the extension actually installed in Python.

Local build with maturin 1.15.0:

```bash
cd bindings/python
/workspace/venv-maturin/bin/maturin build --offline \
  --out /tmp/eng-py-002-fxn2hdfj/wheels \
  --interpreter /workspace/venv-maturin/bin/python
/workspace/venv-maturin/bin/python -m venv /tmp/eng-py-002-fxn2hdfj/venv
/tmp/eng-py-002-fxn2hdfj/venv/bin/python -m pip install --no-index \
  /tmp/eng-py-002-fxn2hdfj/wheels/mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl
/tmp/eng-py-002-fxn2hdfj/venv/bin/python -m unittest discover \
  -s /workspace/mamut-engine/bindings/python/tests -v
```

Build, installation and test: OK, exit 0. `PIP_NO_INDEX=1` was also
used. Test run from `/tmp` on CPython 3.13.5/Linux, with
import from the temporary venv's `site-packages`. One test verifies
type/value `0.1.0`, equality between native version, `__version__` and distribution
metadata, and that the native module only exposes `version`.
Wheel inspection confirms `__init__.py`, `_native.pyi`, `py.typed`,
`.so` extension, `License-Expression: AGPL-3.0-only`, `Requires-Python: >=3.10`
and MAMPRO SAS note. No Python runtime dependency or pytest.

Windows/macOS, other interpreters and manylinux2014 were not verified. The
abi3 tag expresses the ABI target, not a completed test matrix.
The complete legal text remains pending; this verification is not a release.

Records: `/tmp/eng-py-002-{fmt,clippy,test,doc,wasm-workspace,wasm-default,wasm-exclude,wheel}.log`,
`/tmp/eng-py-002-results.json` and `/tmp/eng-py-002-metadata.json`.
Wheel and temporary venv: `/tmp/eng-py-002-fxn2hdfj/`.

`bash clean-room/check.sh origin/main`: OK, exit 0, with all new
files added to the index. `git diff --cached --check`: OK, exit 0.
This ticket's diff does not modify protected paths.

## WASM correction and workspace CI · 2026-10-14

Author: Codex (GPT-6), as implementer at the user's request.
Sources: current request, binding manifests and code, README,
this log, `docs/CLEAN_ROOM.md` and local tools. No network access or
copied third-party code. The initial clean-room declaration is retained.

The failure in `pyo3-ffi` when running the WASM command with
`--workspace`, used by the Rust CI of PR #83, is corrected. The binding remains
in `[workspace].members` and retains the same `default-members`.
PyO3 moves to `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` and
the module containing its imports and macros carries the same condition
`#[cfg(not(target_arch = "wasm32"))]`. In WASM there is no Python API and the crate
remains empty, with documentation explaining this behavior.

`extension-module` is removed from the Cargo dependency features and
activated through `features = ["pyo3/extension-module"]` in
`[tool.maturin]`. Rust checks do not require that linking
configuration; the wheel retains stable ABI `abi3-py310` and the same API.
The Python test on the installed extension and the cdylib without a Rust
harness or doctests are preserved. The README now documents WASM without exclusions.

The following table supersedes the failed WASM result of the initial
verification. Commands from the root, with `PATH=/home/box/.cargo/bin:$PATH`,
`CARGO_TARGET_DIR=/workspace/.cargo-target` and `CARGO_NET_OFFLINE=true`.
The environment retains `RUSTUP_TOOLCHAIN=stable`; that installation reports
rustc `1.99.0 (b940084d7 2026-09-28)`, the version pinned by the repository.

| Check | Result |
| --- | --- |
| `cargo check --workspace --target wasm32-unknown-unknown` | OK, exit 0, without excluding the binding |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, exit 0 |
| `cargo test --workspace` | OK, 416 tests/doctests, 0 failed, 0 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | OK, exit 0 |
| `cargo fmt --check` | OK, exit 0 |
| `bash clean-room/check.sh origin/main` | OK, exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p mamut-engine-py --target wasm32-unknown-unknown` | OK, exit 0 |
| `cargo tree -p mamut-engine-py --target wasm32-unknown-unknown` | Only `engine-core`; no PyO3 or `pyo3-ffi` |

Wheel rebuilt with maturin 1.15.0 and `--offline`, activating the features
from `pyproject.toml`, and installed with `pip --no-index` and `PIP_NO_INDEX=1`
in a new venv. CPython 3.13.5/Linux: `unittest discover` passes
1 test; `version()` returns `0.1.0` and the module is imported from the
new venv's `site-packages`, running from `/tmp`.

Artifact:
`/tmp/eng-py-002-wasm-fix-1or15acg/wheels/mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.
Venv: `/tmp/eng-py-002-wasm-fix-1or15acg/venv`.
Records: `/tmp/eng-py-002-wasm-fix-*.log` and
`/tmp/eng-py-002-wasm-fix-results.json`.
No expansion of the platform matrix or publication. Local commit,
no push; external clean-room review pending J4.
