# Clean-room log · ENG-PY-003

- Ticket / PR: MAM-79 · ENG-PY-003; local changes without commit or publication.
- Author(s): Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: implementer.
- Implemented contract: typed Python facade for `engine-input/1`, `results/1`,
  `capabilities/1` and `method-descriptor/1` over engine-api/PyO3 (ENG-SFC-007).
- Permitted sources used: exclusively the current checkout's own contracts, schemas, documentation,
  fixtures and code: `crates/engine-api`,
  `bindings/python`, `docs/CONVENCION_EJES.md`, `docs/DETERMINISMO.md`,
  `docs/adr/ADR-003-python-v1.md`, `docs/CLEAN_ROOM.md` and `provenance/TEMPLATE.md`.
  No external sources or numerical dependency code were consulted.
- Own derivations: no new physics. The cantilever test preserves
  ENG-SFC-007's in-house oracle: integrating EI·y'' = −P(L−x) with a clamped base
  gives y(L)=−PL³/(3EI); equilibrium gives R=P and M=PL. P=12, L=4, E=200, I=3;
  absolute/relative tolerances 1e−12 for f64 rounding of the small system.
- Third-party code copied: none. No new dependencies or changes
  to Cargo.toml/Cargo.lock or the vendored SDK.
- Validation: direct Rust references for engine-api's five valid fixtures
  and its eleven determinism fixtures; comparison of complete bytes
  and SHA-256 with the installed wheel, zero tolerance. These are existing in-house
  analytical examples, not results from other engines.
- Oracle comparison (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult code or results of CalculiX or LAPACK,
  or reports/outputs of external engines. I work exclusively offline.
- Clean-room reviewer: **PENDING (J4)**.

## Critical review of the WIP against current main

Local base: `2adc337` (MAM-582, P-Delta Stable by default). The preexisting WIP
was reviewed without assuming it was validated. The facade retains only
the six operations existing in Rust; `Engine` merely groups them, without state.
It does not calculate physics, convert units or supply physical constants.

The WIP fingerprint was corrected: SHA-256 of the old compact JSON does not match
RFC 8785/JCS of ENG-CORE-005. The new Rust helper `engine_api::input_sha256`
reuses the same bounded parsing and the same mamut-canon call as `run`.
The private adapter carries it to Python for `Model.input_sha256` and stale;
JCS is not reimplemented and no analysis is run to obtain a fingerprint.
`run` shares the internal helper on Value without changing numbers or documents.

The provenance DTOs (`git_commit`, `target`), the optional
`residual_convergence_ratio` criterion and current capability descriptors were completed.
The P-Delta regression runs `options: {}` and checks `Stable`; the option
`allow_experimental` is retained only according to the Rust contract.

Public errors are Python classes preserving the original code, entities, severity,
cause, hints, diagnostics and document. NotConverged, Unstable,
Cancelled and Unsupported always raise an exception. Secondary interpretation
of a failed result cannot hide the original error. Imported results
preserve extensions, optional fields and original bytes;
input limits are not arbitrarily applied to output arrays.

Tests detected and corrected two additional WIP assumptions: a
3D orientation added to a 2D beam is rejected by Rust, and resolution of
recursive annotations needs the JSONValue namespace in consuming modules.
The orientation test uses the actual frame3d fixture. `Results.version` preserves
the producer document's contracts, which include element/1 and system/1;
it is not forced to match the shorter `version()` list.

## Offline execution evidence

Stable Rust 1.99.0 toolchain; worktree-specific `CARGO_TARGET_DIR`.
Build through `MATURIN=$MATURIN` (1.15.0)
and `scripts/build-python-wheel.sh`, with two independent release builds,
`PIP_NO_INDEX=1` installation in a temporary venv inside CARGO_TARGET_DIR and tests
against the installed package. There is no PyPI publication.

Neither mypy nor pyright is provisioned offline. The mandatory alternative
`test_typing.py` checks exports/re-exports and installed stub signatures,
as well as required/optional TypedDict fields against Rust schemas;
`get_type_hints` resolves the public surface. This is not equivalent to a full checker.

Final results:

| Check | Result |
| --- | --- |
| Reproducible wheel, two independent release builds | OK, identical bytes |
| Python tests against installed wheel on CPython 3.13.5 | 29 passed / 0 failed |
| scripts/tests artifact tests | 25 passed / 0 failed |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline (including doctests) | 631 passed / 0 failed / 0 ignored |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | OK |
| Native/wasm cross-check in debug and release, ten runs per case | 16/16 identical hashes |
| Published docs/DETERMINISMO.md table and debug/release cmp | 16/16 unchanged |
| git diff HEAD --check | OK |

Artifacts: `$CARGO_TARGET_DIR/eng-py-003-dist-verified/`, including
REPRODUCIBILITY.json, SMOKE.json, COMPATIBILITY.json, SBOM and SHA256SUMS.
abi3 CPython ≥3.10 wheel, manylinux_2_34_x86_64, validated here with CPython 3.13.5;
verification of other interpreters or platforms is not claimed.
Wheel SHA-256:
`7e741382348caf24b7a7d46a206e80bfec98a66340caa8e27d21272b511e1d3e`.

Logs inside CARGO_TARGET_DIR: `eng-py-003-wheel-verified.log`,
`eng-py-003-{fmt,clippy,test,doc,wasm-check,clean-room}.log`,
`eng-py-003-determinism-{debug,release}.log` and
`eng-py-003-native-{debug,release}.txt`. The first wheel attempts
detected the test regressions described above; the final attempt
ends with all 29 tests green, without skipping any.

The common Git directory is read-only: the original index was not modified.
To also check the new files not yet registered, the index was copied to
`eng-py-003-review.index`, using temporary objects in
`eng-py-003-review-objects` and the original store as a read-only alternate.
With `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY` and
`GIT_ALTERNATE_OBJECT_DIRECTORIES` configured this way, only intents to add
the new log and test_typing.py were registered in that copy, and the same
`bash clean-room/check.sh origin/main` was run: OK, log
`eng-py-003-clean-room-complete.log`. It was also run on the original index;
the ENG-SFC-007 log links this extension of the PyO3 adapter.

## Follow-up #143

- Ticket: MAM-79 · issue #143.
- Author: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: implementer; contracts `engine-input/1`, `results/1.input_sha256`
  and Python facade. Clean-room reviewer: **PENDING (J4)**.
- Sources: only the in-house checkout, particularly `engine-api/src/{input,
  analysis_input,limits,json,lib}.rs`, schema `engine-input-1.json`, the
  in-house fixtures, Python facade and this documentation. The JCS /
  binary64 premise comes from the ticket and the already documented contract; no external
  consultation, copied code or new dependencies; no consultation or modification
  of vendor sources.

**Decision: option A, symmetric rejection during parsing.** All integer
fields of the schema and DTOs were reviewed: node/member ids and references (`nodes`,
`node`, `member`, `master`, `slaves`) are u32, maximum 4 294 967 295;
`max_iterations` and `mode_count` are usize limited to 16 384, as is the
product of modes by directions. Other ids (sections, supports, masses,
combinations) and names are strings; DOFs are labels. No seeds
or additional integer indices are published in this input. The future
seed policy `0_u64` in DETERMINISMO.md is not an engine-input/1 field.
None of these contracts needs integers above the safe range.

Own derivation of the decision: binary64 has 53 bits of integer precision;
JCS converts numbers to that representation. Restricting integer tokens
to `|n| <= 2^53-1` eliminates identity loss of accepted integers before
JCS. Merely documenting the collision would retain a silent failure in
stale. Physical f64 fields may also be written as integers: their lexical
representation outside the safe range is deliberately tightened,
without changing the domain of finite floats. Tokens with a decimal point/exponent
remain binary64; different spellings rounding to the same f64
preserve the same canonical identity, not a lexical identity.

The check resides in `engine-api/src/limits.rs`, common lexical inspection of
strict parsing before serde (`json::parse`), including giant integers
that serde could round to float. `canonical_json`, `input_sha256`,
`validate`, `validate_with` and `run` share the check. The typed error
`ApiError::InvalidInput` is reused, without extending the public enum: in `run` it produces
`API-E-INPUT`, exit 2, null fingerprint and no parsing callback. No
unsafe, panic, unwrap or expect is added to library code.

Python applies the same symmetric interval to input ints / JSON tokens
before FFI and preserves the path in `InvalidInputError.details`. `bool` remains
a JSON boolean and is rejected for ids/counters through exact types.
A model mutation toward unsafe integers marks `Results` as stale.
`bounded=False` remains reserved for imported results: their extensions are not
arbitrarily subjected to input limits, including this restriction.

The historical JCS vector `9007199254740993` preserves its reference hashes
and is marked `rejected_by_engine`; tests now require its rejection in the
facade before fingerprinting. No changes to mamut-canon, vendor, canonicalization
algorithms, equations, mechanical fixtures or the 16 hashes of
DETERMINISMO.md. Numerical tests use exclusively the existing in-house
analytical oracles; no external oracle was used.

Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
para este ticket." Nor code or results of CalculiX or LAPACK, or
reports of external engines. I work offline, without copied third-party sources.

### Offline evidence of the follow-up

Stable Rust 1.99.0, worktree-specific `CARGO_TARGET_DIR`;
`CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1`. The wheel was built with
`MATURIN=$MATURIN bash scripts/build-python-wheel.sh
"$CARGO_TARGET_DIR/mam-79-143-dist"`, with new output and two independent release
builds. Installation and tests against the wheel on CPython 3.13.5, without
Rust in PATH during consumption. No artifact was published.

| Check | Final result |
| --- | --- |
| Offline wheel, two independent builds | OK, identical bytes |
| Python tests against installed wheel | 32 passed / 0 failed |
| scripts/tests artifact tests | 25 passed / 0 failed |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline, including doctests | 656 passed / 0 failed / 0 ignored |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | OK |
| Native/wasm cross-check in debug and release, ten runs per case | 16/16 identical |
| Automatic comparison with DETERMINISMO.md table and debug/release bytes | 16/16 unchanged |
| git diff --check | OK |

The first passes detected an old expectation of acceptance of
u64::MAX in canonical_json (updated to the safe limit) and numerical
cancellation in recovery when testing the limit with an enormous load at the
free end. The test now applies that load to the support: it verifies parsing,
run and fingerprint without large displacements; it changes neither physics nor tolerances.
The complete suite was repeated after the adjustment and ended green.

Artifacts: `$CARGO_TARGET_DIR/mam-79-143-dist/`, including reproducibility,
compatibility, smoke, SBOM and SHA256SUMS. Wheel SHA-256:
`cc46e866bf838dd4c90e991492389cb1478e96d853940f34181f8a3f8c239dac`.
Logs: `$CARGO_TARGET_DIR/mam-79-143-{wheel,fmt,clippy,test,doc,wasm-check,
clean-room,determinism}.log`; summary `mam-79-143-checks.json` and
`mam-79-143-native-{debug,release}.txt` with 160 rows each. Comparison
with the table hashes was done by exact equality, without regenerating it.

## Pending items

- Independent clean-room review by J4.
- Typing with mypy/pyright and additional CPython/platform matrix when those
  tools are provisioned; the delivery includes the offline alternative.
- No changes to protected paths, clean-room policy, workflows or vendor.

## Follow-up #144

- Ticket / PR: MAM-79 · issue #144; local changes without commit or publication.
- Author: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: Python adapter implementer; no numerical changes.
- Permitted sources: the checkout's own contracts, implementation and tests,
  `docs/CLEAN_ROOM.md`, `docs/DETERMINISMO.md`, `provenance/TEMPLATE.md`,
  offline Cargo resolution and RUSTSEC-2026-0176/0177 advisories from the
  provisioned local database. No network sources were consulted.
- Own derivations: no new physics; exclusively existing in-house
  fixtures and analytical oracles are reused.
- Third-party code copied: none.
- External oracle comparison: no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult code or results of CalculiX or LAPACK,
  or reports of external engines. I work exclusively offline.
- Clean-room reviewer: **PENDING (J4)**.

### Change and compatibility

`bindings/python/Cargo.toml` pins `pyo3 = "=0.29.3"` preserving
`features = ["abi3-py310"]`. Exactly
`cargo update --offline -p pyo3 --precise 0.29.3` was run: Cargo jointly updated
pyo3, pyo3-build-config, pyo3-ffi, pyo3-macros and pyo3-macros-backend.
No other version was changed. The lock removes autocfg 1.5.1,
indoc 2.0.8, memoffset 0.9.1 and unindent 0.2.4, no longer needed by PyO3.
All dependencies were provisioned locally.

The four `allow_threads` calls become `detach`, preserving separation
between engine execution and Python callback conversion/delivery.
Existing `IntoPyObject`, `Py<PyAny>` and `Bound` compile without additional
adaptation; there were no `Python::with_gil` calls. The public surface,
stubs and Python tests are preserved. The API name is updated in the README.
No unsafe, unwrap/expect/panic or numerical code is added.

### Offline advisories against main

Cargo-deny 0.20.2 is available. The first attempt
`cargo deny --offline check advisories` found the local database, but could not
acquire its lock in Cargo's read-only path. That database was copied to
`$CARGO_TARGET_DIR/mam-79-144-advisory-dbs/` and a copy of deny.toml was generated
that changes only `advisories.db-path`; the repo file is unchanged.
The provisioned database reports commit
`edb58f5974b83eb8302332821cb6d389777aa351`; the evidence is limited to its
local contents, without updating it or consulting the network.

Command on the updated tree:
`cargo deny --offline --config "$CARGO_TARGET_DIR/mam-79-144-deny.toml" check advisories`.
Result: **advisories ok**, zero vulnerabilities. For comparison,
`git archive origin/main` (`b618a4d7d2be3c779e5ee9369219ce43987a7901`)
was extracted inside CARGO_TARGET_DIR and the same command was run with
`--manifest-path "$CARGO_TARGET_DIR/mam-79-144-main/Cargo.toml"`.
Main fails exclusively due to RUSTSEC-2026-0176 and RUSTSEC-2026-0177 in PyO3
0.25.1; both advisories require >=0.29.0. Both trees preserve the
preexisting `advisory-not-detected` warning for exception RUSTSEC-2024-0436 (paste,
absent from the current graph). No changes to exceptions or deny policy.

### Validation and artifacts

Stable Rust 1.99.0 toolchain; worktree-specific `CARGO_TARGET_DIR`,
`CARGO_NET_OFFLINE=true` and `PIP_NO_INDEX=1`. No network access or publication.
The complete script is run, performing two independent release builds
per invocation, inspection of all wheel members, SBOM, checksums,
smoke and installation without dependencies in a venv without Rust/Cargo in PATH.

A difference between the first invocations was detected and explained:
`mam-79-144-wheel-one` captured the README before the documentation rename
`allow_threads` → `detach`, edited during its build. Comparison
of ZIP members showed only that text in METADATA and the consequent
RECORD entry; the binary was identical. Its 25 artifact tests and 29 Python
tests passed, but that invocation is discarded for the final comparison.
It is repeated in full in a third directory with the sources now held constant.

Commands for the two invocations on the definitive state:

```bash
export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_TARGET_DIR=<target del worktree>
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1
MATURIN=$MATURIN bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/mam-79-144-wheel-two"
MATURIN=$MATURIN bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/mam-79-144-wheel-three"
python3 scripts/python-wheel-artifacts.py compare "$CARGO_TARGET_DIR/mam-79-144-wheel-two/"*.whl "$CARGO_TARGET_DIR/mam-79-144-wheel-three/"*.whl
sha256sum "$CARGO_TARGET_DIR/mam-79-144-wheel-two/"*.whl "$CARGO_TARGET_DIR/mam-79-144-wheel-three/"*.whl
```

| Check | Result |
| --- | --- |
| cargo fmt --all -- --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline, including doctests | 655 passed / 0 failed / 0 ignored |
| RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline | OK |
| cargo check --workspace --offline --target wasm32-unknown-unknown | OK |
| bash clean-room/check.sh origin/main | OK |
| Actual native/wasm cross-check, ten runs per case in debug and release | 16/16 identical hashes |
| Comparison with docs/DETERMINISMO.md table and debug/release cmp | 16/16 intact; 160 native runs per profile |
| Offline Cargo-deny, same local database | Updated tree OK; main fails due to the two corrected vulnerabilities |
| Cargo.lock diff against origin/main | Only five PyO3 updates and four transitives removed; all other entries intact |
| Final wheel, invocation two | 25 artifact tests and 29 Python passed / 0 failed |
| Final wheel, invocation three | 25 artifact tests and 29 Python passed / 0 failed |
| Wheels two/three, four independent builds in total | Identical bytes; common SHA-256 confirmed |
| SBOM of both final wheels | All five PyO3 crates are 0.29.3 |
| SHA256SUMS of both directories and git diff --check | OK |

Common SHA-256 of the final wheels:
`c71ea7083e86a1e400b383df53b44641a6394650381f284477b31937fa42dc00`.
Comparison between invocations preserved in
`$CARGO_TARGET_DIR/mam-79-144-reproducibility.json`.

Logs: `$CARGO_TARGET_DIR/mam-79-144-{fmt,clippy,test,doc,wasm-check,clean-room}.log`,
`mam-79-144-deny{,-main}.log`, `mam-79-144-determinism-{debug,release}.log`,
`mam-79-144-native-{debug,release}.txt` and `mam-79-144-wheel-{one,two,three}.log`.
Each final directory includes REPRODUCIBILITY.json, COMPATIBILITY.json,
SMOKE.json, SBOM.cdx.json, THIRD_PARTY_NOTICES and SHA256SUMS.

CPython 3.13.5 on Linux x86_64 with maturin 1.15.0 is validated;
tag `cp310-abi3-manylinux_2_34_x86_64`, Python >=3.10 declared. The matrix
of other interpreters/platforms and mypy/pyright retains the already documented
pending items; existing stub/annotation regressions are run.
Independent clean-room review **PENDING (J4)**. No changes to protected
paths, workflows, vendor or numerical results.
