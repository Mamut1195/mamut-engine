# Clean-room log · ENG-PY-005

- Ticket / PR: ENG-PY-005 · MAM-80 · Python experimentation and workflows outside the core.
- Author(s): Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Role: implementer.
- Implemented contract: pure Python `mamut_engine.experiments` module, profile
  `mamut.experiment/1`; reuses engine-input/1, results/1 and SDK contracts
  for workflow/plan/manifest. **It is not a numerical method.**
- Permitted sources used: own contracts and code from the checkout:
  `bindings/python` (ENG-PY-003 facade and ENG-PY-004 arrays),
  `crates/engine-api` (ENG-CORE-006 registry), `crates/engine-cli`
  (ENG-SFC-002) and the vendored mamut-workflows interfaces
  (SDK-WF-003/SDK-WF-012). No external sources or new numerical bibliography.
- Own derivations: none; field selection, copies and document
  transport. No equations, solver or new numerical computation.
- Third-party code copied: none.
- Validation: contracts and integration on existing own fixtures
  `cantilever-linear-static.json` and `p-delta.json`, without new reference
  values or tolerances. Tests compare documents/fingerprints and statuses
  of the same Rust facade; they do not validate another numerical method.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult CalculiX or LAPACK code or results,
  or use network, tables or results from external engines.
- Clean-room reviewer: **PENDING (J4)**.

## Decisions and scope

- No Rust, Cargo, pyproject, vendor, clean-room or CI changes. Existing mixed
  packaging automatically includes the new Python modules.
- The Cartesian product and comparison live in the user's Python
  process. Each run receives an independent model copy. The
  existing facade preserves Rust validation, typed errors and JCS fingerprint;
  Engine failures are identified by variant and do not stop the batch.
  `raise_for_error` is offered to rethrow the original instance.
- Profile, optional seed (metadata only), declared units, original
  and selected analyses, effective methods, version, commit and target accompany the
  result. The output fingerprint covers the exact results/1 bytes; it is not
  confused with the input JCS fingerprint. A missing commit remains null.
- Metrics are scalar selections by path; they do not compute, aggregate or
  convert units. The unit label is declared by the user according to SI.
- No callbacks or registrable Python functions are offered. Python is not loaded
  within Rust, solver, WASM, MCP or CLI. Python errors outside the Engine
  contract are propagated.
- Workflows use the Rust CLI installed by the user. The compiled plan
  and catalog/definition snapshots are retained; the CLI recompiles those same
  documents when running because its current contract does not import serialized
  plans. Python does not interpret or execute steps. The full SDK envelope
  preserves official fingerprints and method overrides.
- The CLI binary is installed separately from the wheel. The CLI's nested JSON
  representation preserves the semantic document and SDK fingerprints, not the original
  results/1 bytes. Failures without an Engine document preserve the envelope and
  an `SDK-EXIT-*` code; no numerical diagnostic is invented.

## Offline verification

Stable Rust 1.99.0 toolchain; CARGO_TARGET_DIR dedicated to the worktree,
CARGO_NET_OFFLINE=true and PIP_NO_INDEX=1. CPython 3.13/Linux x86_64,
maturin 1.15.0, pytest 9.1.1 and NumPy 2.5.3 available locally for the
second run. Changes are integrated into commits on the PR branch;
no publication or network access.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **672 passed / 0 failed / 0 ignored**, including doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| pytest on final wheel, without NumPy | **58 passed / 0 failed / 3 skipped**, 266 subtests; skips only for the NumPy extra |
| pytest on final wheel, with NumPy | **61 passed / 0 failed**, 389 subtests |
| New ENG-PY-005 tests | **18 passed / 0 failed**, including four SDK tests with the real Rust CLI |
| `scripts/build-python-wheel.sh` | OK: 25 tooling tests; wheel installed with unittest: 58 passed / 0 failed / 3 skipped |
| Wheel contents | Both new modules present and byte-for-byte equal to the working tree; without editing pyproject |
| Wheel reproducibility | Two identical independent builds; SHA-256 `ff94fd973048917574455043f1a5f78641a55454ba534412056b3dc5a42ea23a` |
| Python examples on installed wheel | OK: three load variants and two cantilever section scenarios |
| Native/wasm determinism, debug/release | **16/16 hashes identical to docs/DETERMINISMO.md**, ten runs per case/target; debug=release |
| `git diff --check` | OK |

The analysis options test checks that `base_analyses` retains the
original list even when sweeping its tolerance; `requested_analyses` records
the applied option. SDK path profiles reject sweep,
metric and analysis override fields to avoid silently ignoring them.
Transport errors preserve the I/O cause and engine failures
preserve their class, code, entities, document and diagnostics.

## Round 2 · PR #170 · 2026-10-20

- Author: Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Role: implementer. Scope limited to comments #5, #4 and #6.
- Sources: own Python contracts and code from the checkout; no external
  source, third-party code copied or new numerical derivation.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult CalculiX/LAPACK code or results,
  or use network, external oracles, third-party tables or figures.
- Clean-room reviewer: **PENDING (J4)**.

### Corrections and evidence

- #5: `compile_workflow` and `run_workflow` accept `timeout` only by keyword,
  default 300 s; zero, negatives, NaN/Inf, bool, incompatible
  types and integers not representable as finite time are rejected. Reuses
  `AnalysisError` with code `PY-E-WORKFLOW-TIMEOUT`, preserving
  `subprocess.TimeoutExpired` as cause. `subprocess.run` kills and waits for the
  CLI on expiration; the temporary context removes its inputs before raising
  the public error. An own Python executable that sleeps 60 s checks
  both operations with timeout of 0,2 s, terminated process and removed
  directory. Other tests check prior validation and propagation of the
  default/custom value to transport.
- #4: without `Results`, the failure status of `error.document` takes priority if it is
  readable JSON; then the public category is used: `InvalidInputError` →
  `invalid`, `Unstable` → `unstable`, `NotConverged` → `notConverged`,
  `Unsupported` → `unsupported`, `Cancelled` → `cancelled` and
  `InternalEngineError` → `internalError`. Generic `AnalysisError` →
  `analysisError`; another `EngineError` → `error` (Python variant statuses).
  The original error's instance, code, document and fingerprint are preserved.
  Tests cover all eight categories, batch continuity, document
  precedence and fallback for unreadable documents or those without failure status.
  The README documents the mapping and timeout.
- #6: deep copies of `methods()` and `capabilities()` are captured, the
  returned descriptors and containers are mutated and subsequent calls are compared
  by deep equality with the original full snapshots.

### Offline verification of round 2

Rust 1.99.0, CPython 3.13.5, `CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1` and
`CARGO_TARGET_DIR=$CARGO_TARGET_DIR`. Artifacts,
test temporary files and venv are inside that authorized directory.
Maturin is not available in this session: the extension was compiled through
Cargo and the package manually installed in the test venv, with copied
sources and metadata obtained from existing manifests. This round verifies
code and its integration; it does not rebuild or demonstrate a new wheel.
Regression references were generated directly from engine-api,
through the own `facade_reference` example.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **672 passed / 0 failed / 0 ignored**, including doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| Full Python unittest, with NumPy and real Rust CLI | **66 passed / 0 failed / 0 skipped** |
| ENG-PY-005 subsuite (included in the 66) | **23 passed / 0 failed**, five new tests and one strengthened |
| Native/wasm determinism, debug/release | **16/16 hashes equal to docs/DETERMINISMO.md**, ten runs per case/target; debug=release |
| `git diff --check` | OK |

No Rust, Cargo, pyproject, workflows, vendor, clean-room or determinism document
changes. No numerical results are modified. Changes are integrated
into commits on the PR branch.

## Round 3 · PR #170 · J4 H1 · 2026-10-20

- Author: Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Role: implementer. Exclusive scope: H1 (CLI grandchildren after timeout).
  H2–H5 and N1–N3 are deferred to issue #177.
- Permitted sources: own Python contracts and code from the checkout; own
  POSIX process test, without numerical oracle or external sources.
- Own derivations: none. Third-party code copied into the product: none.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult CalculiX/LAPACK code or results,
  or use network, tables, figures or external oracles.
- Clean-room reviewer: **PENDING (J4)**.

### Correction and evidence

- Transport uses `Popen` with `start_new_session=True` on POSIX with
  `os.killpg`. The initial PID identifies its group even if the wrapper
  terminates before its descendants. On expiration, SIGTERM reaches the group;
  its existence is checked for up to 0,2 s with a monotonic clock and
  polling of up to 0,01 s. If it persists, it receives SIGKILL.
  `ProcessLookupError` is ignored in signals; `poll`/`communicate` reap the child and
  close the pipes. Group checking does not depend on the child
  still being alive or its descendants keeping pipes open.
- `AnalysisError` / `PY-E-WORKFLOW-TIMEOUT`, the original instance
  of `TimeoutExpired` as cause and temporary input cleanup are preserved.
  The normal path preserves UTF-8 text stdout/stderr, newline
  normalization and returncode handling. Without `killpg` (Windows),
  direct process kill is retained; this limitation appears in the README.
- The POSIX regression, skipped on other platforms, uses a `sh` script with
  a background `sleep`, PID written to a file and a long wait
  interrupted by a 0,2 s timeout. It covers compile and run, cooperative exit
  and a wrapper that persists after SIGTERM with stdout/stderr closed
  (SIGKILL termination verified). A trap reaps the shell's children
  to avoid depending on init reaping zombies; `os.kill(pid, 0)` must raise
  `ProcessLookupError`, with 0,01 s polling and 1 s limit. Checks
  also cover the direct process reaped, typed error and removed temporary file.
  There are no actual long waits in the test. Another test covers UTF-8,
  CRLF and exit codes 0, 1 and 3.

### Offline verification of round 3

Rust 1.99.0, CPython 3.13.5, pytest 9.1.1, `CARGO_NET_OFFLINE=true`,
`PIP_NO_INDEX=1` and `CARGO_TARGET_DIR=$CARGO_TARGET_DIR`.
Venv, test packages copied from local cache and artifacts in that
authorized directory. No maturin: the extension is compiled with Cargo and
the package manually installed in the venv with existing manifest
metadata; no new wheel is demonstrated. Own references
generated directly by Rust with `facade_reference`; real Rust CLI and
NumPy available in the full suite.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **672 passed / 0 failed / 0 ignored**, including doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| Full Python pytest, with NumPy and real Rust CLI | **68 passed / 0 failed / 0 skipped**, 433 subtests |
| Timeout/transport subsuite (included in the 68) | **5 passed / 0 failed**, 31 subtests |
| Repeated POSIX regression | **5/5 runs passed**, four scenarios per run; no failures |
| Native/wasm determinism, debug/release | **16/16 hashes equal to docs/DETERMINISMO.md**, ten runs per case/target; debug=release |
| `git diff --check` | OK |

No Rust, Cargo, pyproject, workflows, vendor, clean-room or numerical result
changes. Changes are integrated into commits on the PR branch; no network access.

## Round 4 · PR #170 · 2026-10-20

- Scope: Python timeout transport and tests, this log and CHANGELOG.
  No Rust, Cargo, pyproject, workflows or vendor changes; no network or commits
  created in this session. The PR delivery is integrated into branch commits.
- Sources: own code and contracts from the checkout; no external sources,
  third-party code copied or new numerical derivations.
- Clean-room reviewer: **PENDING (J4)**.
- The shell test polls the PID file for up to 5 s, in steps of up to
  0,01 s, and requires a positive integer ending with a newline. If the file
  is missing or remains incomplete, it fails with an explicit message.
- The final cleanup `communicate` has a 5 s limit. If it expires,
  SIGKILL to the group is retried where applicable and the direct process is killed,
  stdout/stderr are explicitly closed and the child is reaped with `wait`
  limited to 5 s. Errors in that cleanup do not replace the original
  timeout: `AnalysisError` / `PY-E-WORKFLOW-TIMEOUT` and its cause are preserved.
- New POSIX regression for compile and run: a descendant creates a session
  through `setsid` and retains the pipes while running `sleep`.
  Both executables are resolved to absolute paths with `os.defpath`,
  independently of the wheel consumer's limited PATH; if missing,
  there is an explicit skip. Checks cover bounded time, original error and cause,
  direct process reaped, closed pipes and removed temporary inputs.
  The descendant outside the group is always killed in `finally`.

### Offline verification of round 4

Rust 1.99.0, CPython 3.13.5 and local maturin 1.15.0;
`CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1` and
`CARGO_TARGET_DIR=$CARGO_TARGET_DIR`.
`MATURIN=/workspace/venv-maturin/bin/maturin` and the local stable toolchain
at the start of PATH were used, without downloading tools or dependencies.

| Check | Result |
| --- | --- |
| `bash scripts/build-python-wheel.sh $CARGO_TARGET_DIR/pyv-check` | **exit 0**; two identical independent builds and offline consumption with PATH limited to the venv |
| Script tooling unittest | **25 passed / 0 failed** |
| Installed wheel unittest | **62 passed / 0 failed / 7 skipped** (69 tests; four require Rust CLI and three optional NumPy) |
| POSIX shell/grandchild and descendant-outside-group regressions | **Both passed**, compile and run; run inside the script, without skip |
| `cargo fmt --all -- --check` | **exit 0** |
| `git diff --check` | OK |

## Round 5 · PR #170 · J4 r2 review · 2026-10-20

- Role: implementer. Scope: H6, H7, N4 and N5, limited to transport,
  `WorkflowTimeoutTest`, README, CHANGELOG and this log.
- Sources: own code and contracts from the checkout and local test
  artifacts; no network, external sources, third-party code copied or new
  numerical derivations. No commits were created or excluded paths consulted.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult CalculiX/LAPACK code or results.
- Clean-room reviewer: **PENDING (J4)**.
- H6: after catching `TimeoutExpired`, `_invoke` catches
  `BaseException`, terminates the process/group and rethrows the original exception.
  `KeyboardInterrupt` preserves its class and instance; timeout remains
  `AnalysisError` / `PY-E-WORKFLOW-TIMEOUT`, with `TimeoutExpired` as cause.
  The new POSIX regression uses a `sh` wrapper with a `sleep` grandchild, PID written
  to a file and a trap to reap it. A `communicate` mock waits for the PID,
  checks it belongs to the CLI group and raises `KeyboardInterrupt` only
  once; cleanup uses the real `communicate`. It covers compile and run,
  requires disappearance of both PIDs with polling bounded to 1 s, removed inputs
  and absence of new `mamut-experiment-*` directories. Cleanup in
  `finally` also occurs if an assertion fails. Skipped without POSIX groups/`killpg`.
- H7: the shell timeout regression measures with a monotonic clock and requires
  `elapsed < 2` in both subcases (cooperative and persistent), for compile and
  run. This detects resorting to the final 5 s wait if the main killpg
  is missing.
- N4: an `OSError` other than `ProcessLookupError` in the
  `os.killpg(pid, 0)` probe immediately advances to group SIGKILL. The new
  test denies only that probe with `PermissionError`, allows real signals
  through and checks SIGKILL and the original public timeout in compile and run.
- N5: README documents survival of descendants that leave the group
  with `setsid`, the additional delay of up to about 5 s if they retain pipes and
  group termination also on Ctrl-C or other exceptions.

### Offline verification of round 5

CPython 3.13.5 and pytest 9.1.1. The local PR #170 wheel is reused and
extracted to an authorized test directory, replacing only
`_experiment_workflows.py` with the modified file. The wheel's other `.py`
modules are compared byte for byte with the checkout and are identical. No
new wheel is rebuilt or demonstrated. CLI, Rust references and NumPy are
existing local artifacts; no downloads.

Invocation with `/workspace/venv-j2/bin/python -m pytest -q
bindings/python/tests` and `PYTHONPATH` initially obtains 70 passed and an environment
failure: the import test's isolated `python -I` subprocess does not
see `PYTHONPATH`, and that venv does not have `mamut_engine` installed. A test
venv is prepared within the authorized directory, using the same interpreter
and local packages through `.pth`, so that `python -I` can also see it.

Full suite command in that environment, with `MAMUT_EXPERIMENT_CLI` and
Rust reference variables pointing to local artifacts:

```sh
$CARGO_TARGET_DIR/r5-venv/bin/python -m pytest -q bindings/python/tests
```

| Check | Result |
| --- | --- |
| Full Python suite with local Rust CLI and NumPy | **71 passed / 0 failed / 0 skipped**, 433 subtests |
| `WorkflowTimeoutTest`, full class in 20 successive pytest processes | **20/20 runs passed**; 8 tests and 31 subtests per run, without skips or failures |
| `git diff --check` | OK |

Each repetition takes between 12,61 and 12,65 s, including cleanup waits
for the descendant with `setsid`. Individual logs remain in
`$CARGO_TARGET_DIR/r5-repetitions/01.log` through `20.log`.
The new `KeyboardInterrupt` regression covers both operations in every
run: 40 interruption scenarios in these 20 repetitions. The tested transport
module remains identical to the checkout file.

## Round 6 · PR #170 · J4 r3 review · S1 · 2026-10-20

- Role: implementer. Scope: `WorkflowTimeoutTest`, CHANGELOG and this
  log; no production transport, Rust, Cargo, pyproject,
  workflows or vendor changes. No network or commits; excluded paths were not consulted.
- Sources: own code and tests from the checkout, installed Python standard
  library and local artifacts. No external sources, third-party code
  copied or new numerical derivations.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Nor did I consult CalculiX/LAPACK code or results.
- Clean-room reviewer: **PENDING (J4)**.

### Reproduced root cause and diagnostic limits

- The test scenarios started the 0,2 s timeout without confirming that the
  wrapper had created the grandchild and written its PID. Under load the wrapper may
  die first: waiting for the file afterward cannot recover a PID
  that was never published. In the `setsid` case, the parent published `Popen.pid`:
  that also did not confirm that the grandchild had executed `setsid` or `sleep`.
  In the shell, `$!` confirmed fork, but not exec or resetting the
  grandchild's signals. The interruption test had this latter race even
  though it waited for the file.
- Evidence: with instrumentation and without correcting setup, the first
  round with 16 `yes` and pytest restricted to the same CPU failed in
  `escaped/compile`. `_invoke` had already returned in about 0,301 s; the
  `escaped.pid` file remained empty/missing. The first wait failed at 5,301 s
  total and `finally` waited another 5 s (10,304 s total), hiding
  the first failure. The class gave **1 failed / 7 passed**, 31 subtests, in 15,19 s;
  the full pytest process took 23,264 s under that load. The 16 load
  processes were killed and reaped in `finally`.
- This demonstrates a cause of test instability, but does not identify with
  certainty the historical failure of 1/62 rounds without diagnostics. An excess of about
  5 s also fits the shell's PID wait or the final
  `communicate` limit. No `_terminate` failure was observed or that code changed.
- Transport reading: `start_new_session=True` establishes the session/group
  before exec of the direct child; `Popen` waits for the exec result, so
  sending `killpg` before creating that group does not explain this case. Probes
  may see groups containing zombies, but the grace wait ends
  at 0,2 s; zombies do not retain pipes. A descendant that leaves
  the group with `setsid` and retains stdout/stderr does legitimately trigger the
  fallback `communicate(timeout=5)`, twice in the class (compile/run).
  That is why normal duration remains around 13 s. Failure to schedule
  the shell during grace or a process slow to respond to SIGKILL
  remain residual hypotheses; the new messages allow distinguishing them.

### Correction and preservation of checks

- Each operation is named `compile`/`run`; all time,
  PID/PGID, signal and reaping assertions identify the scenario and elapsed
  monotonic time through `msg=f"..."`. Failed PID waits
  indicate subcase, known CLI PID, file, partial content, wait
  time and total time. If a PID disappears before querying its PGID,
  the failure also identifies it.
- A small Python test program publishes its own PID after exec
  and sleeps with default SIGTERM. The shell installs the trap, creates that grandchild
  and also publishes its PID once it has closed its pipes in the
  persistent case. In the external scenario, `setsid` is executed at an absolute path
  and then the same program; PID = PGID is checked before timeout.
- The first call to `communicate` waits for those readiness signals
  (up to 5 s, with diagnostics), and then calls the real `communicate` with the
  original timeout of **0,2 s**. Timeout + termination measurement begins
  after setup; failures also include its separate duration.
  **elapsed < 2 s** is retained for cooperative and persistent shell, in both
  operations. SIGKILL for the persistent one, actual grandchild disappearance
  (zombies are not accepted), direct child reaping, original error and
  cause, pipe closure and temporary input removal remain required.
- Both shell descendants are retained: both now publish their PID
  after exec and disappearance of both is required. In the persistent case,
  the first retains the original pipes and the second is born after
  closing them in the shell. The trap reaps both and executes `sleep` after
  SIGTERM; the wrapper must terminate by SIGKILL even with pipes closed.
- The `finally` blocks use already recorded PIDs, do not wait again for a missing
  file, and also clean up if a prior assertion fails. The
  interruption case likewise uses the PID published by the grandchild itself.

### Offline verification of round 6

CPython 3.13.5 and pytest 9.1.1 from local venv `v-con`; installed transport
module compared with the checkout through `cmp` (identical). No wheel is rebuilt
or demonstrated. Rust CLI and references are existing local artifacts.

```sh
MAMUT_EXPERIMENT_CLI=$CARGO_TARGET_DIR/release/mamut-engine \
$CARGO_TARGET_DIR/v-con/bin/python -I -B -m pytest -q -p no:cacheprovider bindings/python/tests/test_experiments.py -k WorkflowTimeoutTest
```

| Check | Result |
| --- | --- |
| Instrumentation on original test, without load | **8 passed**, 31 subtests, 12,64 s |
| Original test instrumented with 16 `yes` on one CPU | **1 failed / 7 passed** in the first round: unpublished PID; diagnostic reproduced above |
| Development series under load | **10/10 rounds passed**; completed before the final series |
| Final file, full class, without load | **10/10 rounds passed**, 8 tests and 31 subtests per round, without skips; 12,80–13,43 s per class |
| Final file, full class, 16 `yes` and pytest on one CPU | **10/10 rounds passed**, 8 tests and 31 subtests per round, without skips; 17,11–17,73 s per class; all 16 `yes` killed and reaped |
| Full Python suite, local Rust CLI and NumPy, existing Rust references | **71 passed / 0 failed / 0 skipped**, 433 subtests, 14,04 s |
| Cap check through delay added only in memory | Adding 2,05 s before real `_terminate` produces **4/4 expected failures** of `elapsed < 2`, with operation, persistence, PID, elapsed and setup in the message |
| AST review of time/PID assertions | All contain `msg` with f-string |
| `git diff --check` | OK |

The **20 final rounds** used the same test (SHA-256
`3628b5a6984ea18e303eee32d3d0724229817747da0b9a2a7504c100839752ae`,
checked each round); they total 160 test runs and 620 subtests. They are not
mixed with the ten development rounds. The installed transport and the
checkout transport match (SHA-256
`c0b58f41eacbc65589a4cf9268ea5d0ff51b804039cfe65c62865be047873b3c`).

Local logs and runners in
`$CARGO_TARGET_DIR/r6-repetitions/`:
`instrumented-load-01.log` retains the failure; `final-load-01.log` through `10.log`
and `final-normal-01.log` through `10.log` retain the final rounds. Files
`final-load-summary.log` and `final-normal-summary.log` record SHA, whole process
duration and load cleanup; `full-suite-summary.log` records
the full suite. `time-bound.log` contains the four deliberate failures and
`time-bound-summary.log` confirms they were exactly the expected ones. The
delay was injected through a mock in a separate process, without editing transport.

## Pending items

Clean-room review **PENDING (J4)**. The CLI must be installed separately to
use workflows; direct reimport of serialized plans is outside
its current contract. No protected path or existing numerical result
was modified. PR changes are integrated into branch commits.
