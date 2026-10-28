# Clean-room log · ENG-PY-007

- Ticket / PR: MAM-85 · ENG-PY-007 «Wheels, CI, SBOM and Python V1 distribution»; issue #94. Branch `j2/eng-py-007`, based on `origin/main`.
- Author(s): Codex (GPT-6), under user direction, 2026-10-15 (America/Guayaquil).
- Role: packaging implementer, without changes to binding methods or API.
- Implemented contract: offline build/consumption and local distribution artifacts; references to the engine's `engine-input/1` and `results/1`, without implementing them in the Python scaffold (PY-003 PENDING).
- Permitted sources used: user request; own files `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `bindings/python/**`, `engine-core` manifest, `engine-api`/`engine-contracts` contracts, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md` and ENG-PY-002-FU log; local maturin 1.15.0 help and local Cargo metadata (including license expressions). Without consulting external sources or using network.
- Own derivations: Bash/Python scripts with the standard library; component identities independent of Cargo paths; regression tests and rollback procedure. No numerical derivations.
- Third-party code copied: none. Existing `LICENSE`/`NOTICE` are checked byte for byte, not modified. Artifacts contain declarative dependency metadata, not copied code.
- Validation: binary reproducibility, native import/call, ZIP/legal inspection, SBOM portability regressions, Rust checks and local rollback; this is not solver validation.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: **PENDING (J4)**.
- Licenses, ownership, final notice, commercial license and CLA: **PENDING review (ENG-GOV-004 / ENG-GOV-007)**. The inventory does not constitute authorization to distribute.

## Implementation

`scripts/build-python-wheel.sh` accepts `MATURIN` and `PYTHON`, requires
exported `CARGO_TARGET_DIR` and a new output. It does not depend on paths on this
machine. It checks Rust against `rust-toolchain.toml` (1.99.0), blocks
automatic rustup installation, pins maturin 1.15.0 and
`SOURCE_DATE_EPOCH=1767225600` (2026-01-01 UTC). It runs Cargo with
`--offline --locked`; the two release/stripped builds do not share a target and
remap Rust paths. It does not install tools or publish.

The consumer venv is new; only its `bin` appears in PATH. pip uses
`--no-index --no-deps --no-cache-dir`. Checks cover the absence of
`cargo`/`rustc`/`rustup`, the distribution version and real calls to
`mamut_engine.version()` and `_native.version()`. That is the only current function;
no analysis or new API is added.

`scripts/python-wheel-artifacts.py` generates `SBOM.cdx.json` (CycloneDX 1.5),
`THIRD_PARTY_NOTICES`, `COMPATIBILITY.json`, `REPRODUCIBILITY.json`, `SMOKE.json`
and `SHA256SUMS` alongside the wheel. The SBOM uses the transitive closure from
`mamut-engine-py`, not the entire workspace; it conservatively includes
Cargo build and runtime dependencies for all targets. It does not inventory system
libraries or tools. That limitation is documented. No external SBOM generator
was used: own generation requires only Cargo and the standard library.

**Closes #94:** the Python README no longer contains workspace paths;
SBOM identities never reuse Cargo's absolute package IDs. A recursive check
rejects system paths and eight tests cover absolute references,
metadata relocation and order, identity collisions, serialization,
SBOM/notices generation, legal file integrity and packaged README.
The real inventory has 22 components; `engine-core` lacks a license
expression in its metadata and remains explicitly PENDING, without inferring it.

For #93, checks cover the presence of `dist-info/licenses/LICENSE` and `NOTICE`,
their byte-for-byte equality with the binding and `License-File` fields in METADATA.
Existing legal notices remain unchanged.

## Local verification

Actual environment: Linux x86_64, CPython 3.13.5, rustc
`1.99.0 (b940084d7 2026-09-28)`, maturin 1.15.0. To select the already
installed toolchain, its local `bin` directory was prepended to PATH. The exported
`CARGO_TARGET_DIR` was preserved: `/workspace/.cargo-target/mamut-engine`.
All Cargo operations used offline mode; none required network.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK, exit 0 |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | OK, exit 0 |
| `cargo test --offline --locked --workspace --no-fail-fast` | OK, exit 0; 545 tests/doctests, 0 failed, 0 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | OK, exit 0 |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | OK, exit 0 |
| `bash -n scripts/build-python-wheel.sh` | OK |
| Full script, including binding unittest and artifact regressions | OK, exit 0 |
| `sha256sum --check SHA256SUMS` | OK for wheel and all artifacts |
| `bash clean-room/check.sh origin/main` | OK with new files staged |
| `git diff --cached --check` | OK |

Wheel: `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.
Two builds from scratch produced exactly the same SHA-256:
`92913abb592f3dfeeae8ceaef6f7611f58ea72318cbe74e41d4bf4bb24d263b6`.
The demonstrated reproducibility is with the indicated local tools, not
across different OS/toolchains. The script fails and reports hashes and ZIP
differences if that equality no longer holds.

Final local output: `/tmp/eng-py-007-dist-final/`. Local logs:
`/tmp/eng-py-007-{fmt,clippy,test,doc,wasm}.log`,
`/tmp/eng-py-007-wheel-final.log` and `/tmp/eng-py-007-rollback.log`.
Wheels, artifacts and logs are not added to the repository.

## Rollback rehearsal

Own HEAD sources were archived in `/tmp/eng-py-007-rollback-source` and
only that copy's binding version was changed to **synthetic 0.1.1**;
`engine-core` remained at 0.1.0. Its lockfile was updated with `cargo metadata
--offline` and it was built with maturin `--offline --locked`, without touching the
versions or lockfile of the working checkout.

In another clean venv, with PATH limited to its `bin`, the following was run:

1. Install wheel 0.1.0 and check distribution 0.1.0 / engine 0.1.0.
2. Install synthetic wheel 0.1.1 with `--force-reinstall`, check distribution
   0.1.1 / engine 0.1.0.
3. Verify previous `SHA256SUMS` and reinstall wheel 0.1.0 with
   `--no-index --no-deps --no-cache-dir --force-reinstall`.
4. In a new Python process, check distribution 0.1.0 / engine 0.1.0,
   native call and absence of Rust in PATH.

Result: **OK**, `0.1.0 → 0.1.1 → 0.1.0`. The synthetic version only rehearses the
replacement mechanism; it does not test rollback of real API/data changes or
represent a published version. Operating procedure in the Python README.

## Pending items and limits

- Only Linux x64 and CPython 3.13.5 demonstrated here. Windows x64, macOS arm64 and
  CPython 3.11/3.12/3.14: PENDING. Candidate matrix in Python README.
- `abi3-py310` was already configured; tag inspected and loading on 3.13.5 tested.
  Cross-interpreter abi3 compatibility PENDING testing of each cell.
- Actions CI PENDING due to budget; VPS runner PENDING migration by J0.
- Clean-room review, licenses and CLA PENDING. Authorized distribution is not claimed.
- No changes to `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` or `vendor/**`.
  The untracked `crates/engine-wasm/` directory, predating this work, remains intact.
- No publication, uploads or push. Only the local commit requested by the user.

## J4 r37 — Embedded SBOM and portability of the entire wheel

Review on 2026-10-15, MAM-85 / PR #117. The previous wheel included
`dist-info/sboms/mamut-engine-py.cyclonedx.json` automatically generated by
maturin, with `path+file:///workspace/...` references that the external SBOM
check did not reach.

`maturin --version`, `maturin --help` and
`maturin build --help` from the local 1.15.0 executable were consulted offline. Help offers
`--sbom-include`, but no CLI option to disable generation.
Local configuration was checked with a temporary project: the
maturin parser recognizes `sbom.rust` and `sbom.auditwheel` and rejects text values
because it requires booleans. In `[tool.maturin]`,
`sbom = { rust = false, auditwheel = false }` was configured; real builds confirm
that there are no `dist-info/sboms/**` members. Neither ZIP nor RECORD is rewritten.
The own portable SBOM continues to be generated alongside the wheel.

`check_wheel_portable` traverses `ZipFile.infolist()` and checks the name and
decompressed content of **all** members, including RECORD,
all `dist-info/**`, legal files and `_native.abi3.so`. It directly searches for
bytes of `/workspace`, `/home`, `/tmp`, `/root`, `file://`, `path+` and Windows
drives with slash or backslash; it does not decode binaries or discard bytes.
The Windows pattern distinguishes a drive from URLs such as `https://`.
A match aborts with member, position and path fragment.
The script runs `check-wheel` for each build before comparing hashes,
installing or exposing output; `artifacts` also runs it at startup.

The already present reproducible remapping for Rust panic/debuginfo paths
is preserved and documented: `--remap-path-prefix` transforms the checkout into `.`,
Cargo home into `cargo-home` and each build's directory into `build`.
The script passes these RUSTFLAGS through `CARGO_ENCODED_RUSTFLAGS`, the Cargo
equivalent that preserves arguments with spaces and avoids depending on flags
inherited from the environment. The real binary passes the indicated byte search.

The 11 artifact regressions pass. The new synthetic wheel checks
48 combinations: eight paths/patterns in METADATA, RECORD, nested SBOM,
another nested dist-info member, `.so` binary and package data.
It includes NUL and non-UTF-8 bytes; every combination fails on the injected path.
Rejection of absolute names and acceptance of portable content/URLs
are also tested. The CLI returns exit 1 when injecting a path in RECORD and
in `.so`. Both real wheels pass the same check within the script.

Verification r37, all offline, with
`CARGO_TARGET_DIR=/workspace/.cargo-target/me45` exported and PATH pointing
to already installed Rust 1.99.0:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | OK |
| `cargo test --offline --locked --workspace --no-fail-fast` | OK; 545 tests/doctests, 0 failed |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | OK |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | OK |
| `bash -n scripts/build-python-wheel.sh` | OK |
| Full script: two independent builds, ZIP checks, venv and binding | OK |
| `sha256sum --check SHA256SUMS` | OK for all artifacts |
| Real wheel inspection | 9 members; no embedded SBOM; `License-Expression: AGPL-3.0-only` |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --cached --check` | OK |

Wheel `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`;
identical SHA-256 in the two independent builds:
`3ab15573289adb3dd3109c9cbf0d1f2d51705b820bbd2fa5fe0db5cd6470841f`.
Local output: `/tmp/eng-py-007-r37-dist/`; logs
`/tmp/eng-py-007-r37-{wheel,fmt,clippy,test,doc,wasm}.log`.
The reproducibility scope of the previous verification is retained:
indicated local tools and platform, without claiming equality across
different toolchains. No license changes or new legal texts;
no changes to `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` or `vendor/**`.
No network, publication or push.

## J4 r39 — More local paths in the portability check (#121)

Follow-up MAM-85 / ENG-PY-007-FU, 2026-10-15, branch
`j2/eng-py-007-fu` based on `origin/main`. The byte pattern shared
by `check_portable` and `check_wheel_portable` is expanded to reject
`/opt`, `/Users`, `/var`, UNC with backslashes (`\\servidor\recurso`)
or slashes (`//servidor/recurso`) and `~/`. The Bash script already applies that
check to both builds and artifact generation; it does not require
duplicating the pattern. Rejections of `file://` and `path+` are retained.

The left UNC boundary excludes URL scheme separators
such as `https://` and `http://`. A server name with letters,
digits, hyphens, periods or underscores is required: an initially more
permissive alternative matched `_native.abi3.so` instructions, not a
path. The final pattern passes over all real wheel members,
without decoding binaries or omitting bytes. Directory word boundaries
are retained to accept `optional`, `UsersGuide` and `variant`.

The **18 artifact tests** pass. There is an individual regression for
each of the six new forms, with recursive text values and
non-UTF-8/NUL bytes, and another for Cargo `path+file:///build/…` references
in the `purl` and `bom-ref` fields. The ZIP matrix expands to **84 cases**
(14 patterns across six members), including METADATA, RECORD, nested SBOM,
another dist-info member, `.so` and data. Member names with
the six new forms are also rejected and URLs, portable references
and the binary sequence that produced the spurious match are accepted. Before
expanding the pattern, the new regressions produced 48 expected failures.

Offline verification with `CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine`
exported, Rust 1.99.0 installed, maturin 1.15.0 and CPython 3.13.5 on Linux x64:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | OK |
| `cargo test --offline --locked --workspace --no-fail-fast` | OK; 569 tests/doctests, 0 failed, 0 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | OK |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | OK |
| `bash -n scripts/build-python-wheel.sh` | OK |
| Artifact tests, also within the full script | OK; 18 tests |
| Full script: two independent builds, ZIP checks, venv and binding | OK; native call and binding test |
| `sha256sum --check SHA256SUMS` | OK for wheel and all artifacts |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --cached --check` | OK |

Wheel `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`;
the two independent builds produce the same SHA-256:
`3ab15573289adb3dd3109c9cbf0d1f2d51705b820bbd2fa5fe0db5cd6470841f`.
Local artifacts in `/tmp/eng-py-007-r39-dist/`; logs
`/tmp/eng-py-007-r39-{wheel,artifacts,fmt,clippy,test,doc,wasm}.log`
and `/tmp/eng-py-007-r39-regressions-before.log`.
The local reproducibility scope described in r37 is preserved.
No license changes or changes to `docs/CLEAN_ROOM.md`, `.github/**`,
`clean-room/**` or `vendor/**`. No network, publication or push.

## J4 r4x — `~/` in binary text context and isolated `path+` (#121)

Follow-up MAM-85 / PR #124, 2026-10-15, branch `j2/eng-py-007-fu`.
The strict text pattern confused the `7e 2f` bytes of the real `.so` from
#119 with a path. That pattern is retained for Python `str` values and for
valid UTF-8 content without NUL, regardless of member extension.
Bytes containing NUL or invalid UTF-8 are considered binary; they are
inspected in full, without discarding bytes or using lossy decoding.

In binaries, **only `~/`** now requires these simultaneous conditions:

- Left: start of data, ASCII space, single/double quote, `=` or `:`,
  or a byte outside the printable ASCII range `0x20..0x7e`.
- After `~/`: a sequence of **at least two** bytes from
  `[A-Za-z0-9._/-]`.
- That sequence must end at end of data, NUL, space, tab, CR,
  LF, single/double quote or one of `= : ; , ( ) [ ] { }`.

The right delimiter avoids another real match from #119: `~/xs` followed
by `0xc8`, at offsets 238255 and 270823. Requiring only two path characters
was not enough. The other patterns retain their boundaries and strict rejection;
they did not need relaxing for the real wheel to pass.

The **23 tests** of `python3 -m unittest discover -s scripts/tests` pass.
Regressions include the real 32-byte windows `[96871:96903]` and
`[238239:238271]` from `mamut_engine/_native.abi3.so`, embedded as hex in the
test, without adding binaries. The first contains `~/` at 96887 followed by two
NULs; the previous check rejects that window at relative byte 16.
Both now pass, both directly and inside a synthetic wheel.
`b"\x00~/secreto/x\x00"` in a synthetic `.so` still fails at byte 1.
Left/right boundaries, the two-byte minimum and strict rejection
in text are also covered. Cases `path+/ruta/local` and `path+C:\x`
fail on `path+` without `file://` or another pattern that could mask its absence;
they are included in the ZIP matrix expanded to **96 combinations**.

The own script was run from `/workspace/me-geo6`, detached at
`ffb8a6775d3660cd1d3f9fe6334ac430221e7d8b` (#119), without modifying or making
a commit in that worktree. **Environment limitation:** the attempt with
`CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6` exported failed before
compilation: `mktemp: Read-only file system`. That directory is outside the
roots permitted by the sandbox. To complete the check,
the same script was repeated, unchanged, with
`CARGO_TARGET_DIR=/tmp/eng-py-007-r4x-me-geo6-target` exported and output
`/tmp/eng-py-007-r4x-geo6-dist/`. The full script passed: two builds with identical
SHA and 12 installed binding tests. The wheel has SHA-256
`0841090a11bc0030644bdd95188bcbe19b1b6089f1f96d25ee4095e6a80bc0e2`.

For that wheel,
`/workspace/mamut-engine/scripts/python-wheel-artifacts.py` from **this branch** was imported directly
and `check_wheel_portable` was run: **OK on all 9 members**, including the
`.so` of 2016360 bytes. Both windows were extracted from that build and their
exact equality with the fixtures was checked. The #119 worktree remained clean.

Verification of this branch, offline, with local Rust 1.99.0, maturin 1.15.0,
CPython 3.13.5 and `CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine` exported:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | OK |
| `cargo test --offline --locked --workspace --no-fail-fast` | OK; 569 tests/doctests, 0 failed, 0 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | OK |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | OK |
| `bash -n scripts/build-python-wheel.sh` | OK |
| `python3 -m unittest discover -s scripts/tests` | OK; 23 tests |
| Full script with the final rule: two independent builds, ZIP, venv and binding | OK |
| `sha256sum --check SHA256SUMS`, both outputs | OK |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --cached --check` | OK |

Wheel of this branch: `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`;
identical SHA-256 in the two independent builds:
`3ab15573289adb3dd3109c9cbf0d1f2d51705b820bbd2fa5fe0db5cd6470841f`.
Final output: `/tmp/eng-py-007-r4x-dist-final/`. Logs:
`/tmp/eng-py-007-r4x-{wheel-final,artifacts,fmt,clippy,test,doc,wasm,clean-room}.log`
and `/tmp/eng-py-007-r4x-geo6-{wheel,cross-check}.log`.
The local reproducibility scope described in r37 is preserved.
No network, license changes or changes to `docs/CLEAN_ROOM.md`, `.github/**`,
`clean-room/**` or `vendor/**`. No publication or push; only the local commit.
