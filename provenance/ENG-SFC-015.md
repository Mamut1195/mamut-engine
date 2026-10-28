# Clean-room log · ENG-SFC-015 · MAM-866

- Date: 2026-10-21 (America/Guayaquil).
- Author: Codex agent, GPT-6; role: implementer.
- Local base: origin/main, revision 7375fa25a34a3cb004bcaec84f645743b864ff89.
- Contract implemented: progress/1, NDJSON transport of engine stages.
- Permitted sources: ticket MAM-866; own engine-api and engine-cli code;
  docs/DETERMINISMO.md; changelog.d/README.md; local interfaces of the vendored
  SDK (parser, configuration, observers, provider and execution).
- Own derivation: adaptation of the API callback to an event with a step
  index, without domain calculations or new mechanical results.
- Third-party code copied: none.
- Declaration: I did not consult external structural analysis implementations,
  their reference inputs/outputs, restricted paths or logs from other
  sessions. The restricted log indicated in the ticket was not consulted.
- Clean-room reviewer: pending human review.

## Contract and decisions

The actual engine_api::Progress type contains operation, stage, completed and total;
it contains no iteration or residual. Those four fields and the zero-based SDK
index are transported. Iteration is not inferred from the stage counter.
The schema allows no additional fields or non-finite numbers. For a future
extension with a residual, a non-finite value will be represented as null and the schema
must be reviewed; today the field does not exist.

The existing --progress ndjson flag includes progress/1 by default, preserving
the original SDK events. The README documents selection by schema or
phase for consumers. No syntax is added to the vendored parser.
The same public canonicalizer as results/1 is used, without timings, to maintain
deterministic key order and numeric formatting.

Transport logic is concentrated in crates/engine-cli/src/progress.rs.
The shared writer groups SDK writes up to the newline and
sends each complete event through write_all; each stage is written and flushed
during the callback, before the host observer. I/O errors are
remembered to return 70 and prevent a success output on stdout.

The SDK requires Method to contain no borrows, but invokes the factory within
the started/finished interval. The factory executes the local method with the
borrowed writer and returns a method that delivers that result only once.
This preserves the visible sequence, including failures and cancellation, without
adding threads, global state, Send requirements or accumulation of stages per step.
The public host API and its callback retain their signatures.

## Validation

The binary tests traverse the own linear, pDelta and modal fixtures
from engine-api/tests/fixtures/determinism and the eleven shared sfc010 fixtures,
including nonlinearStatic/Newton and consolidation. Each input uses three steps
(validation and two executions) to check indices and context reset.
Each event is compared with the actual API callbacks and validated against
the schema with the existing test validator, extended with a generic schema
input; no dependencies are added.

Equality of stderr between two executions, equality of stdout against the
execution without progress, valid JSON on every line and
started/stages/finished order per step are required. Convergence failure
without finished, mutations rejected by the schema, complete-line writes,
immediate delivery to the host and write and flush errors are also tested.
No SDK documents are saved as fixtures or reference captures.

Final offline checks, with CARGO_TARGET_DIR=/workspace/.cargo-target/sfc15:

| Check | Result |
| --- | --- |
| cargo fmt --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | 753 passed, 0 failed, 0 ignored; 115 suites, including documentation tests |
| bash clean-room/check.sh origin/main | OK, including new files through a temporary index and objects in /tmp; real index intact |
| Review of identifiers in the full content of each new or modified file | OK |
| git diff --check | OK |

The 19 CLI integration tests and its 4 unit tests pass. Progress coverage
comprises 14 own inputs, three steps per input and two executions
with progress against one without it. The SDK event bytes
are also compared with their original format; empty lines are rejected.

No changes are made to vendor, Cargo manifests, CI configuration, clean-room
rules, domain methods, axial bar or truss elements.
No network or commits. CHANGELOG.md is not modified: the per-ticket fragment is added.

## Local merge of origin/main · 2026-10-22

Integration without commit of origin/main e4002e16b13720f297b1f3c10b46efd0d2233496
onto the #210 branch, HEAD 884eef2bc3a8d0b3bca19dddc1c693b62dc02db0.
The merge remains in progress; neither rebase nor abort is run.

Resolution of the three conflicts:

- `crates/engine-cli/src/lib.rs`: the single read and `--input` precedence from
  #197, the direct validate-report/1 path from #200, the timeout module
  from #202 and optional signal installation with signal-hook from #214 are retained. On top,
  the stage observer from #210 is connected to the SDK step context.
- `crates/engine-api/tests/support/mod.rs`: the main version is retained
  in full, delegating to the shared `support/schema.rs` validator. The CLI tests
  import that validator only once for validate-report/1 and progress/1.
- `crates/engine-cli/README.md`: the English translation from #216 is retained and
  progress/1 is documented in English, along with timeout, signals and validation.

`progress.rs` reuses main's EventWriter for stage events and
SDK events; the duplicate line buffer from #210 is removed. write_all,
flush, delivery during the callback, canonical keys and absence of
timings are retained. Write/flush errors return 70 and do not publish success.
Timeout and signal tests accept and validate intermediate stages,
retaining the terminal envelope and codes 6/5 respectively. An additional
regression compares stdout/stderr bytes between the previous path and --input
via file/stdin, with and without a generous timeout.

M1 from J4: progress/1 is registered in `scripts/schema-sources.json`, through
additional_schemas, and `schemas/progress/1.json` is regenerated along with the index.
The URN/URL table is updated and the separation of registries is explained in
`docs/PUBLICACION_ESQUEMAS.md`. It is not added to SCHEMAS: it is a CLI event and
the #212 convention publishes those contracts from additional_schemas, without
requiring engine_api::schema() to expose them or creating a dependency on the CLI.

The MAM-866 entry remains in `changelog.d/MAM-866.added.md`, updated
with integration and publication. CHANGELOG.md contains no #210 entry to
move and is not edited. The manifests, lockfile and other prohibited
paths retain the changes already brought by the merge, without further editing.
The full-content review of the merge detects a banned term in prose
inherited from `provenance/ENG-SFC-002.md`; it is replaced with «correction» without altering
its meaning. No restricted logs or other people's logs are consulted.

Verification environment: no network, Cargo offline, PATH with $HOME/.cargo/bin and
CARGO_TARGET_DIR=/workspace/.cargo-target/sfc15. TMPDIR, TMP and TEMP point to the
same target directory during checks; no temporary files are used
in /tmp. The logs of this execution are also saved in that target.

Results of this integration:

| Check | Result |
| --- | --- |
| cargo fmt --check | OK |
| cargo clippy --workspace --all-targets --offline -- -D warnings | OK |
| cargo test --workspace --offline | 786 passed, 0 failed, 0 ignored; 117 suites, including documentation tests |
| python3 scripts/generate-schemas.py --write | OK, local generation |
| python3 scripts/generate-schemas.py --check | OK |
| python3 -m unittest scripts/tests/test_schema_publication.py | 7 passed |
| Full content of the 124 new/modified files in the merge | No prohibited identifiers or markers |
| git diff --check | OK |

The CLI has 9 passing unit tests and 41 passing integration tests. EventWriter is
retained without duplicating its complete-line logic. The review detected
a repeated validator import in the tests, corrected by sharing
a single module declaration before running full Clippy and the full suite.

Environment limitation when recording the resolution: `git add` could not create
`/workspace/mamut-engine-r/.git/worktrees/mamut-engine-sfc15/index.lock`, because
the actual Git directory is mounted read-only, outside the writable
targets. The three files have no markers and pass the tests, but
the real index retains its unmerged entries (UU). The index is not altered
through alternative paths and no permission escalation is attempted. MERGE_HEAD remains
at e4002e16b13720f297b1f3c10b46efd0d2233496 and HEAD does not change.

The exact git add command, without commit, is left in
`/workspace/.cargo-target/sfc15/MAM-866-stage-resolutions.sh`, to run
from an environment that allows writing the real index. All content
changes and verification are complete; only recording the resolution
in that index remains.

## Merge of main with cli-error/1 · 2026-10-22

- Author: Codex gpt-6.1-sol high.
- The additions from main (#205 and #183) are retained and progress/1 from
  #210 is integrated. The five conflicting files have no markers;
  the Git index retains its previous state because git add is not run.
- Progress write and flush failures go through the central cli-error/1
  emitter with CLI-E-INTERNAL, already registered, and exit 70. The tests
  validate the intermediate stages and the single terminal envelope against their schemas,
  including timeout, signals and method failures.
- cli-error/1 and progress/1 are joined in scripts/schema-sources.json and
  schemas/index.json is regenerated exclusively with scripts/generate-schemas.py --write.
- Offline verification: cargo fmt --check, workspace Clippy with all
  targets and warnings as errors, cargo test --workspace --offline
  (818 passed, 0 failed, 0 ignored; 124 suites), generation --check
  and the 7 Python publication tests: OK. Coverage of all workspace
  code emitters passes.
- CHANGELOG.md and changelog.d/MAM-866.added.md are retained. The prohibited
  paths are not edited; temporary files and logs remain under CARGO_TARGET_DIR.
  No network, commit, git add, rebase or abort.
