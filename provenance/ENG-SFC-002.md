# Clean-room log · ENG-SFC-002

- Ticket / PR: ENG-SFC-002 · MAM-91; PR PENDING.
- Author(s): Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: CLI adapter implementer; no new numerical implementation.
- Implemented contract: native `mamut-engine` CLI on SDK-WF-012;
  cli-run/1 + `engineResults` extension, engine-input/1, results/1;
  Engine v1 operations catalog and `ExitCodes::engine_v1()` codes.
- Permitted sources used: own contracts, implementation and fixtures from
  `crates/engine-api/src`, `crates/engine-api/tests/fixtures/determinism/`;
  workspace configuration and `bindings/python` conventions;
  `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md` and previous state of this ticket;
  `vendor/mamut-workflows/{README.md,mamut-workflow-cli/README.md}` and local
  sources of mamut-workflow, mamut-workflow-cli and mamut-workflow-execution,
  commit `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d` (SDK-WF-012 / MAM-493).
  No applicable AGENTS.md was found in the repo or its ancestors.
- Own derivations: no new equation; integration decisions
  documented in `crates/engine-cli/README.md`.
- Third-party code copied: none new. MAMPRO's own libraries already
  vendored under Apache-2.0 are linked by path, with their existing LICENSE/NOTICE
  files. No vendor file is modified.
- Validation: integration tests of the binary itself, direct comparison with
  engine-api and SDK manifest, existing own fixtures; offline workspace
  checks and integrity/clean-room guards.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni código de derivados o rutas prohibidas para este ticket."
- Clean-room reviewer: **PENDING (J4)**.

## Status and changes

The historical block documented here for SDK-WF-007 is resolved by the
base already pinned in the repo, SDK-WF-012 / MAM-493. It is not re-vendored and its
code is not altered. `crates/engine-cli` is delivered as package `engine-cli`, binary
`mamut-engine`, library `engine_cli`, workspace member and default-member.
The license is inherited with `license.workspace = true`: AGPL-3.0-only after #120.
Cargo.lock only incorporates the local package, without new external dependencies.

Parser and dispatch are exclusively `mamut_workflow_cli::parse_args` and
`dispatch`. CliConfig injects mamut-engine identity, version from
`engine_api::version()`, run operation, engine_v1 table, cancellation and an
own MethodProvider. The provider builds real engine.run and
engine.validate methods, which call engine-api with local input. No executor,
argument parser, numerical validator or parallel compiler is implemented.
Input JSON is validated by engine-api.

An own embedded catalog and two example workflows are included. Since SDK
parameters are numerical, structured input is obtained from the
`MAMUT_ENGINE_INPUT` path (or PathBuf supplied through the library); no flag is added.
Reading applies the engine's MAX_INPUT_BYTES and classifies failures as Document/2.
Stdin `-` remains the base's standard catalog or workflow transport.
The README expressly distinguishes that path from a future engine-input/1 stdin.

Method returns only Result<Value,String> and the manifest stores only fingerprints.
The provider retains semantic state in memory shared with the
host; the adapter transforms the terminal run envelope, without changing the
sealed manifest, to add `engineResults`. Outcome/diagnostics are
explicitly mapped to FailureKind; Warning uses ExitCodes.warning because
that variant does not exist in FailureKind. Arbitrary messages are not classified.
1/3/4 are checked with real engine-api runs, as well as 0/2/5/70.

NDJSON is written and flushed to stderr by the base; only the stdout envelope is
retained until results are attached. Control connects CancelFlag and CancelToken
between steps and stages, with the synchronous engine-api Progress callback; it also
accepts token cancellation from a host thread, without creating threads.
The binary also connects MAMUT_ENGINE_CANCEL_FILE to Control::with_cancel_file:
the presence of the path cancels before a step or after an engine callback.
No signal handler or thread is installed; Ctrl-C retains OS
semantics. The marker is optional, is neither created nor removed and, once observed,
cancellation is latched. Code 5 is checked by running the binary,
both with a prior marker and one created after spawn while it awaits the workflow.
The library also checks cancellation between run/validate stages.
MCP does not retain full documents of successive calls in State; only
the run command needs to capture them to enrich its terminal envelope.
No unsafe or unwrap/expect/panic is added in the own library/binary.

## Mapping and open decisions

The full table and usage examples are in `crates/engine-cli/README.md`.
Describe publishes version/identity/commands/codes; methods lists the two
registered operations. They do not equal the entire capabilities document or
JSON schemas. Standard validate validates workflows; the engine.validate method
validates engine input. There are no standard schema, explain,
version or capabilities commands: they remain rejected as invalid usage (2).

The following remain open, without inventing own commands:

- Per-step document transport, replacement of the environment bridge and
  direct engine-input/1 input through stdin.
- Publication/approval of the engineResults extension and engine.validate
  report, or a channel for full results and typed errors in the SDK.
- Standard exposure of full capabilities, version contracts, schema
  and explain. Descriptors only reference contracts.
- Cooperative cancellation by process signals and possible NDJSON for internal
  engine stages; today NDJSON is the workflow's started/finished contract.
- MCP parity: standard dispatch retains its fingerprint manifests and
  generic method error semantics (70), without terminal engineResults
  extension or code 1 for warnings. Provider/identity/table are injected,
  but MCP is not declared equivalent to this ticket's enriched run.
- **Windows/Linux/macOS binaries in CI: PENDING; Jonathan decides**.
  No changes to `.github/workflows`, `clean-room/` or vendor.
- Human and clean-room review **PENDING (J4)**.

## Offline verification · 2026-10-17

Toolchain selected with
`export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH`
(rustc 1.99.0) and
`CARGO_TARGET_DIR` specific to the worktree.
`CARGO_NET_OFFLINE=true`, `PIP_NO_INDEX=1`; no access to external sources.

Results of the current review, all green:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **643 passed / 0 failed / 0 ignored**, including doctests; **13 own CLI tests** |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK, without exclusions |
| `bash clean-room/check.sh origin/main` | OK |
| `bash scripts/check-vendor.sh` | OK, intact inventory and hashes |
| `bash clean-room/tests/run.sh` | **73 tests / 0 failures** |
| `git diff --check` | OK |

Determinism: the existing engine-api determinism_native and
determinism_wasm examples were run in debug and release, with
`MAMUT_ENGINE_GIT_COMMIT` unset. `scripts/check-determinism.mjs` confirmed
**16 native=wasm cases, ten runs per case/target/profile**. `cmp` confirmed
debug/release equality and a comparison by case name against the final
table in `docs/DETERMINISMO.md` confirmed **16/16 unchanged hashes**.
No fixtures, numerical implementation or reference table were modified.

Local evidence: `$CARGO_TARGET_DIR/mam91-verificacion/` (logs for each command,
native outputs and wasm cross-check). Operational incident corrected: logs were
initially written in `$TMPDIR/mam91-verificacion`, outside the ticket's path
restriction; they were moved to the authorized target and `TMPDIR` was set
under that target for final checks. No changes to vendor/,
clean-room/, docs/CLEAN_ROOM.md, .github/workflows or CODEOWNERS. Cargo.toml and
Cargo.lock retain the engine-cli integration already present in the branch.

Binary coverage includes 0/1/2/3/4/5/70: real combination with warnings,
unsupported element, mechanism, exhausted P-Delta iteration, cancellation by
marker and closed stdout pipe before delivering the workflow. Results are compared
with engine-api and the manifest with the base. Library tests also
cover flag/token/file cancellation between stages, I/O failures
and state isolation between invocations. No numerical logic is added.

Clean-room review **PENDING (J4)**.

## Follow-up #141

- Ticket: MAM-91 · GitHub #141, J4 follow-up to #138.
- Author: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: test implementer; no new numerical logic.
- Sources: own contracts and implementation from `crates/engine-cli/src/lib.rs`,
  `crates/engine-api/src/{lib.rs,limits.rs}`, `crates/engine-model/src/lib.rs`,
  `docs/CLEAN_ROOM.md` and `provenance/TEMPLATE.md`. No external sources,
  numerical derivations, third-party code copied or comparison with oracles.
- Declaration: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni derivados o rutas prohibidas para este ticket."
- Clean-room reviewer: **PENDING (J4)**.

Two unit tests are added. Reading is extracted without behavioral changes
to private `read_engine_input` (without expanding API or visibility). A generator of
spaces without EOF counts consumed bytes; a fuse returns an I/O error
after MAX+2 to bound memory and time even when removing the limit.
Consuming exactly MAX+1 and returning Document/2 (`invalid_input`) is required.

The second test injects an ENG-E-0013 error diagnostic into `diagnostic_status`,
the private classifier used by the validate method; it traverses `failure_kind` and
`report_failure` and requires Unsupported/3 and the `unsupported` envelope. The current API
`validate_with` only checks topology and does not generate ENG-E-0013: a test with
an unknown element returns 0 in validate (run does reject it). Therefore this
regression is tested in the library with the explicit diagnostic, without extending
validation or inventing a JSON input activating a currently unreachable path.
No bug was detected in the two behaviors targeted by this follow-up;
no functional correction is introduced.

Temporary mutations, compiled and run separately:

```bash
cargo test -p engine-cli --lib --offline tests::input_read_stops_at_max_plus_one_and_returns_document_two -- --exact
cargo test -p engine-cli --lib --offline tests::validate_eng_e_0013_maps_to_exit_three -- --exact
```

- Without `.take((engine_api::MAX_INPUT_BYTES + 1) as u64)` (adding `mut` to the
  parameter only to compile the mutation): **0 passed / 1 failed**, Cargo
  return 101; assertion 8388610 bytes consumed versus 8388609 expected.
- With `"ENG-E-0013" => Status::Invalid`: **0 passed / 1 failed**, Cargo return
  101; assertion exit 2 versus 3.
- After each mutation the file is restored byte for byte and exactly the same
  command is repeated: **1 passed / 0 failed**, return 0 in both cases. No
  mutations remain in the tree.

Offline verification with Rust 1.99.0, stable PATH indicated in the ticket,
`CARGO_TARGET_DIR` pointing to a build directory outside the worktree,
`CARGO_NET_OFFLINE=true` and `PIP_NO_INDEX=1`; temporary files and logs in
`$CARGO_TARGET_DIR/mam91-141/`.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **654 passed / 0 failed / 0 ignored**, including doctests; CLI: 2 new unit + 13 integration |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |

Determinism: `determinism_native` and `determinism_wasm` examples, in debug and
release, with `MAMUT_ENGINE_GIT_COMMIT` unset. The existing script
`scripts/check-determinism.mjs` confirms **16 native=wasm cases, ten runs
per case/target/profile**. Native debug/release outputs are byte-for-byte
identical and their 16 hashes match by name the final table in
`docs/DETERMINISMO.md`. No fixtures or numerical results are modified.

Evidence: `mutation-{reading-limit,validate-0013}.log`,
`restored-{reading-limit,validate-0013}.log`, individual check logs,
`native-{debug,release}.txt` and `determinism-cross-{debug,release}.log`, all in
`$CARGO_TARGET_DIR/mam91-141/`. Only tests/private reading extraction,
this log and CHANGELOG.md change. No protected paths or Cargo.toml/lock are touched.
Only the **PENDING (J4)** clean-room review of this follow-up remains pending.

## Follow-up #146

- Ticket: MAM-91 · GitHub #146, follow-up to ENG-SFC-002 and #141.
- Author: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Role: test implementer; contract: CLI reading with inclusive limit
  `engine_api::MAX_INPUT_BYTES` and Document/2 rejection when exceeding it.
- Permitted sources: own implementation and contracts from
  `crates/engine-cli/src/lib.rs`, `docs/CLEAN_ROOM.md` and `provenance/TEMPLATE.md`.
  Validation through own synthetic space inputs with EOF;
  no numerical derivations, third-party code copied or comparison with oracles.
- Declaration: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni derivados o rutas prohibidas para este ticket."
- Clean-room reviewer: **PENDING (J4)**.

Two tests are added: exactly MAX bytes with EOF returns Ok and length MAX;
MAX+1 with real EOF returns Document and maps to exit 2. No functional
bug is detected: only tests and documentation change. The local
build path from #141 is replaced by a generic description.

Temporary mutation `input.len() > engine_api::MAX_INPUT_BYTES` → `>=`, run with
`cargo test -p engine-cli --lib --offline tests::input_read_accepts_exact_max_with_eof -- --exact`:
**0 passed / 1 failed**, Document error, Cargo exit 101. After restoring the file
byte for byte (`cmp`), the same command gives **1 passed / 0 failed**, exit 0.
Logs: `mutation-inclusive-limit.log` and `restored-inclusive-limit.log`, in
`$CARGO_TARGET_DIR/mam91-146/`.

Offline verification with Rust 1.99.0, stable PATH indicated in the ticket,
`CARGO_TARGET_DIR` outside the worktree, `CARGO_NET_OFFLINE=true` and `PIP_NO_INDEX=1`:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **657 passed / 0 failed / 0 ignored**, including doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |

Determinism: **16/16 unchanged hashes** against `docs/DETERMINISMO.md`,
with native/wasm cross-check in debug and release, ten runs per case/target/profile
through `scripts/check-determinism.mjs`. Native outputs of both profiles
byte-for-byte identical; `MAMUT_ENGINE_GIT_COMMIT` unset.
Check logs, `native-{debug,release}.txt`,
`determinism-cross-{debug,release}.log` and `determinism-table.log` in
`$CARGO_TARGET_DIR/mam91-146/`. Production code identical to HEAD;
no changes in protected paths or Cargo.toml/lock.
Only the **PENDING (J4)** clean-room review remains pending.

## BrokenPipe correction in tests

- Ticket: MAM-91; author: Codex gpt-6.1-sol high · 2026-10-17
  (America/Guayaquil); role: test implementer.
- Contract: CLI tests preserve exit codes and diagnostics when
  the child terminates before reading stdin. Sources: own tests from
  `crates/engine-cli/tests/cli.rs`, `docs/CLEAN_ROOM.md` and `provenance/TEMPLATE.md`.
- Own derivation of I/O handling; no external sources, third-party code
  copied, numerical derivations or comparison with oracles.
- Declaration: "No consulté código fuente ni resultados de OpenSees, Stabileo,
  CalculiX ni LAPACK, ni derivados o rutas prohibidas para este ticket."
- Clean-room reviewer: **PENDING (J4)**.

Option (a) is chosen: `write_child_stdin` tolerates exclusively `io::ErrorKind::BrokenPipe`;
any other write error fails the test. It is the simplest option:
it adds no threads or synchronization for a small input whose reader may
legitimately terminate with exit 2. The taken handle is closed on leaving the
helper, before `wait_with_output`. It applies to the four child stdin writes:
`Files::run`, catalog, cancellation and closed stdout. All existing assertions
of exit code, stdout and stderr are retained unchanged.
Only tests and documentation are modified; no changes to production or calculations.

Offline validation with Rust 1.99.0, indicated stable PATH,
`CARGO_TARGET_DIR` specific to the worktree, `CARGO_NET_OFFLINE=true`
and `PIP_NO_INDEX=1`: the affected test passes **400/400 runs**, with eight
concurrent processes (`--exact` on the Cargo-compiled binary).

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | **658 passed / 0 failed / 0 ignored**, including doctests; CLI: **13/13** integration |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` and `git diff --check` | OK |

The **16/16 hashes** in `docs/DETERMINISMO.md` match ten native debug runs
per case of `determinism_native`, without `MAMUT_ENGINE_GIT_COMMIT`.
Logs in `$CARGO_TARGET_DIR/mam91-brokenpipe/`. Pending: clean-room review
**PENDING (J4)**; uncommitted changes.

## MAM-862 / ENG-SFC-011 · `--input` transport · 2026-10-21

- Author: Codex (GPT-6), implementer, America/Guayaquil.
- Scope: CLI adapter preparsing for `--input <ruta|->`, default embedded
  documents and input reuse between steps; no numerical,
  engine contract, dependency or vendored source changes.
- Sources used: own code and documentation from `crates/engine-cli`,
  `engine-api` limit, `docs/DETERMINISMO.md`, `crates/engine-api/tests/val032_runs.rs`,
  deterministic `cantilever.json` fixture and own VAL-032 fixture
  `docs/validation-report/fixtures/val032/E1-v1.1.2.json`; local vendored SDK parser and dispatch
  to learn their public extension points;
  `clean-room/check.sh` and `provenance/TEMPLATE.md` to record verification.
- External code copied, numerical derivations and comparison with oracles:
  none. No external sources, results from other engines,
  restricted paths or records from other sessions were consulted. Work without network.
- Compatibility: full bytes are compared between `--input`, embedded
  documents and MAMUT_ENGINE_INPUT with explicit catalog/workflow. Explicit workflow
  validation preserves its behavior; shorthand validation runs
  `engine.validate`. Variable input preserves its error
  classification and cancellation priority. MCP retains per-call reading.
- E1 retains its known diaphragm recovery warning and code 1;
  the success test with code 0 uses the existing deterministic cantilever. No
  warnings are suppressed to satisfy an expectation incompatible with the fixture.
- Initial verification: fmt, diff and clean-room OK; Clippy and tests remained
  blocked by the absence of `/workspace/.cargo-target/sfc11`. In round 2
  the target already exists; the offline checks indicated below are run there.
- Clean-room reviewer: **PENDING (J4)**.

### Round 2 · OpenCode review · 2026-10-21

- Author: Codex (GPT-6), implementer; same sources and scope as MAM-862.
  The vendored SDK's local `mamut-workflow-cli/src/parser.rs` parser is consulted
  only to verify its syntax; no code is modified or copied.
  No network, external sources, numerical derivations or comparison with oracles.
- Decision: the base recognizes exact `--catalog` and `--workflow` tokens, without
  `=valor` or `--` separator. `--input=valor` is explicitly rejected with
  Usage/2 to preserve a single syntax. `--` is also Usage/2 before,
  after or as a supposed `--input` value. A missing, empty value or one that
  starts with a hyphen is Usage/2, except `-` (stdin); paths with a hyphen may
  be written with `./`. Binary tests cover both commands, known and
  unknown flags and help, and compare `=` forms with the base parser.
- Stdin limits: valid documents padded with spaces to MAX-1 and
  MAX bytes yield 0, stdout with results and empty stderr in both run and
  validate; MAX+1 yields 2 and empty stdout. UTF-8 and path coverage is retained.
- README clarifies full stdin reading before dispatch, with an inclusive
  8 MiB limit and no per-step streaming.
- Temporary files: PID + global atomic counter is retained, without resets,
  exclusive creation and up to 128 retries on AlreadyExists. Relaxed suffices
  to reserve unique names; directory creation arbitrates collisions
  with remnants of processes whose PID was reused. A private function is extracted
  to allow testing with local root and counter: eight previous
  collisions and 16 creations, unique paths with PID, correct content,
  own cleanup and preservation of the directories that caused collisions.
  The test does not alter the global counter, avoiding interference between tests.
  Another integration test synchronizes 16 concurrent EngineCli invocations with
  a barrier, using the real global counter, and requires identical results.

Verification with Rust 1.99.0, `CARGO_TARGET_DIR=/workspace/.cargo-target/sfc11`,
`CARGO_NET_OFFLINE=true` and `PIP_NO_INDEX=1`. Check logs and temporary files
in `$CARGO_TARGET_DIR/mam862-r2/`. No commits or changes to Cargo, vendor,
`.github` or `clean-room`; human clean-room review **PENDING (J4)**.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 758 passed / 0 failed / 0 ignored; CLI: 5 unit and 23 integration |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |
