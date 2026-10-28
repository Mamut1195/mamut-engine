# Clean-room log · ENG-CONT-001

- Ticket: MAM-867 (ENG-CONT-001).
- Author: Codex agent (GPT-6), 2026-10-21 (America/Guayaquil).
- Role: implementer.
- Local base: `origin/main` = `7375fa25a34a3cb004bcaec84f645743b864ff89`.
- Worktree: `/workspace/mamut-engine-cont1`.
- Contracts: `cli-error/1`, `error-codes/1`, `error-codes-lock/1`.
- Permitted sources used: ticket and user restrictions; own code from
  engine-cli, engine-api and engine-core catalog; CLI README,
  `docs/DETERMINISMO.md`, `changelog.d/README.md`; public API and emission paths
  of the SDK already vendored in the repository (read-only, unchanged).
- Own derivations: terminal envelope design, registry and compatibility
  guards described in `crates/engine-cli/README.md`.
- Third-party code copied: none new. The test validator is adapted from
  the project's own `crates/engine-api/tests/support/mod.rs`, with no new dependencies.
- Comparison with oracle: no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni
  derivados para este ticket."
- No network; no restricted sources or logs from other sessions were consulted.
- No changes to vendor, Cargo.toml/Cargo.lock, .github, clean-room,
  CLEAN_ROOM.md, CODEOWNERS, deny.toml, CHANGELOG.md, axial bar or truss.
- Clean-room reviewer: pending human review.

## Decisions and scope

`src/error.rs` centralizes all observable terminal failures of the adapter.
The schema is published alongside the CLI (`schemas/cli-error-1.json`,
`urn:mamut:cli-error:1`) to keep the process contract separate from the
numerical contracts of engine-api. It does not change the engine schema API.

cli-error/1 is chosen as the sole terminal envelope on stderr, preserving the
compatible cli-run failure fields (`type`, identity, `exitCode`, `outcome`,
`error` and optional failed document). `error.code` matches the new `code`;
WF-E-* is replaced by ENG/API when there is a primary diagnostic or by CLI-E-*.
In other commands the old `{exit_code,message}` is replaced. The actual
command identifies `operation`; unknown arguments and non-UTF-8 bytes are not
reflected. `args_os` prevents std::env::args from aborting on non-UTF-8 arguments.

Engine diagnostics and documents are preserved in full. The first error
identifies the primary code; entities do not imply a unique JSON location,
so pointer is null. Hint comes from the diagnostic or is null. The English
base texts are in messages/en.json and stable keys in the registry;
i18n is not implemented and existing diagnostics are not translated. ENG-E-0014 maps
to instability / 4 also in the validate classifier, as engine-api does.

The initial registry freezes 32 codes (9 CLI, 14 ENG, 9 API). The introduction version
is that of the registry, 1.0.0. The lock contains complete immutable definitions,
except status; deprecations and additive entries are permitted. Tests detect
duplicate ids, disappearance, definition changes and emitted literals without
registration. The lexical inventory traverses src of all crates, skips comments,
chars and cfg(test) items; it does not analyze vendor or dynamic codes. All
current emitters use literals.

MCP: mamut-workflow-cli's `serve` constructs tool errors through
CliFailure::envelope, and the SDK's McpServer constructs JSON-RPC errors. The
CliConfig received by dispatch does not expose replacement of these envelopes. Changing those
frames would require vendor or replacing/intercepting the transport; they remain intact
and the authorized exception is documented. MCP catalog and transport terminal
errors do adopt cli-error/1. The RPC parse error and termination
on a truncated frame are tested to distinguish the two channels.

Timeout (6) remains reserved in origin/main; no timer is added and the work
in PR #202 is untouched. The emitter is tested with FailureKind::Timeout. The
reachable codes 2/3/4/5/70 are tested with the binary, including cancellation
by file and a closed stdout pipe. Broken stderr prevents delivery of an envelope;
70 is returned without retrying. Failed I/O after writing may leave partial
bytes; failures before output leave stdout empty.

## Local validation

Environment: PATH with `$HOME/.cargo/bin`,
`CARGO_TARGET_DIR=/workspace/.cargo-target/cont1`, Cargo always `--offline`.

- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 761 tests, 0 failures, 0 ignored
  (117 suites, including doctests and the vendored SDK already part of the workspace).
- CLI: 31 own tests (7 unit, 18 binary/adapter, 5 registry,
  1 schema mutation). Validation of all FailureKind, including reserved
  timeout, and of each failure observed in the binary tests.
- Exact comparison of successful run stdout with reconstruction of the previous
  envelope from the SDK, and preservation of canonical engine bytes in the
  existing fixtures. Negative schema and registry freeze tests.
- `bash clean-room/check.sh origin/main`: OK, with all new files
  included through `git add -N` in a temporary index.
- `git diff --check`: OK, also with the temporary index.

The real index is in `/workspace/mamut-engine-r/.git/worktrees/mamut-engine-cont1`,
outside the writable roots; the `git add -N` attempt was rejected by the
read-only filesystem. To verify the full diff,
`GIT_INDEX_FILE=/tmp/mam867-review.index`,
`GIT_OBJECT_DIRECTORY=/tmp/mam867-review-objects` and, as
`GIT_ALTERNATE_OBJECT_DIRECTORIES`, the original object directory obtained with
`git rev-parse --git-path objects` were used. After `git read-tree HEAD`, `git add -N` was applied
to the ten new files and the guard was run with that environment. Both the
real index and the original objects remain intact; the new files
still appear as untracked in normal `git status`.
- No commit. `scripts/changelog-check.sh origin/main` does not apply without a commit,
  according to the instructions; `changelog.d/MAM-867.added.md` is added.

## MAM-867 review · round 2

- The actual registry and lock files are validated through `include_str!`.
  The guard requires unique ids in both and compares the complete lock, in order,
  with 32 complete definitions embedded in `tests/error_registry.rs`.
  Comparing exact values avoids introducing a custom hash or dependencies.
- Coordinated editing of registry and lock no longer permits deleting or modifying
  frozen entries. Negative tests mutate each entry and each field,
  remove entries in both files and alter lock order. They also
  test that deprecated retains its definition and reserves the id against duplicate
  additions, both active and deprecated.
- Each addition requires extending registry, lock and test anchor. Explicit limit:
  anyone also modifying the anchor may bypass the guard. Without an immutable
  external reference there is no automatic guarantee against that third edit;
  review must require the lock and anchor to grow only by addition.
- Code definitions in `crates/*/src` were reviewed, including the
  `engine-core::catalog::CODES` catalog, indirect references and local API codes.
  The registry already contains the current 14 ENG and 9 API codes, including ENG-E-0014.
  Coverage traverses all those src and the lexical regression includes const,
  static and CodeInfo definitions used indirectly, as well as direct calls.
- `experiments.py` reexports the transport from `_experiment_workflows.py`.
  That transport is adapted for cli-error/1 without engineResults: it preserves code,
  error.message, diagnostics and the complete envelope in details. results/document
  remain None. It still supports the previous SDK-EXIT-* format and the path
  with a results/1 document preserves its original exceptions and diagnostics.
  New tests exercise compile_workflow and run_workflow without a document, exit
  categories, independent copies and backward compatibility; the
  test with the real CLI also verifies compile failure without engineResults.

Round 2 offline validation:

- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 762 tests, 117 suites,
  0 failures and 0 ignored; 6 registry tests.
- Debug wheel built with local maturin 1.15.0, `build --offline --locked`,
  without new dependencies; installed with pip --no-index --no-deps in a temporary
  venv with NumPy already installed. 27 independent references were generated
  with facade_reference offline and engine-cli was built offline.
- `python -I -B -m unittest discover -s bindings/python/tests -v`: OK,
  77 tests, no skips, with MAMUT_EXPERIMENT_CLI and Rust references.
  Local log: `/tmp/mam867-r2-python-test.log`; Cargo log:
  `/tmp/mam867-r2-cargo-test.log`.
- `bash clean-room/check.sh origin/main`: OK, including the ten new
  files through a temporary index and objects under /tmp, without altering the real
  index. `git diff --check`: OK with the same complete inventory.
- No network or commit; all ticket file restrictions are maintained.
