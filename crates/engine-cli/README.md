# ENG-SFC-002 · MAM-91 · mamut-engine

Native Engine CLI: package `engine-cli`, binary `mamut-engine`, a workspace member
and default member. It uses Rust 1.99 as specified by `rust-toolchain.toml` and
inherits **AGPL-3.0-only** through `license.workspace = true`. On Unix it uses
`signal-hook =0.4.5` (decision #203). No network access, telemetry or helper
processes are required.

SDK-WF-012 / MAM-493 resolved the original SDK blocker. The adapter uses the
vendored `mamut-workflow-cli` and `mamut-workflow-execution` libraries at commit
`1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`. `EngineCli::run` extracts `--timeout`
with `timeout::parse_timeout_args`, then prepares `--input` and default documents.
Workflow commands use the SDK's public `parse_args` and one `dispatch`, configured
with `CliConfig`, `MethodProvider`, `CliIdentity` and `ExitCodes::engine_v1()`.
Workflow parsing, compilation and execution remain in the SDK. Direct
`validate --input` calls `engine_api::validate_with`. Vendor is unchanged.

## Usage and input

From the repository root, using Cargo offline:

```sh
cargo build -p engine-cli --offline
cargo run -p engine-cli --offline -- describe
cargo run -p engine-cli --offline -- methods --catalog crates/engine-cli/catalog.json
cargo run -p engine-cli --offline -- run \
  --input crates/engine-api/tests/fixtures/determinism/cantilever.json --progress ndjson
cargo run -p engine-cli --offline -- run --input - < model.json
cargo run -p engine-cli --offline -- validate --input model.json
cargo run -p engine-cli --offline -- validate --input - < model.json
```

`catalog.json`, also embedded as `engine_cli::CATALOG`, registers `engine.run` v1
and `engine.validate` v1. They call `engine_api::run` and
`engine_api::validate_with`, respectively. Run supports the analyses published by
the API, including linearStatic, pDelta, modal and responseSpectrum, plus the
experimental analyses enabled through API options. Use `validate.workflow.json`
to select the validation method from `run`. Both methods take `{}` parameters;
the provider rejects numerical parameters and unknown methods or versions without
using IdentityProvider.

When both `--catalog` and `--workflow` are omitted, `run` uses the embedded
`engine_cli::CATALOG` and `engine_cli::RUN_WORKFLOW`. `validate --input` validates
the model directly, without a workflow or temporary documents. For compatibility,
`validate` without `--input` uses the catalog and `engine_cli::VALIDATE_WORKFLOW`.
The documents use `include_str!`; the adapter materializes workflow documents in
a private temporary directory for SDK dispatch and removes it on completion.
Those paths never appear in the output. With both options explicit, `run`
executes the selected workflow and `validate` only validates/compiles that
workflow without executing engine-api. Providing only one option is usage error 2.

`--input <path|->` is the primary engine-input/1 transport. `-` reads stdin to EOF.
The adapter reads the document once before dispatch and all steps reuse it in
memory. Reading stops on exceeding `engine_api::MAX_INPUT_BYTES`: **8 MiB
(8,388,608 bytes)**; exactly that size is accepted. Oversized input, a missing
path, a read error or invalid UTF-8 produces exit 2, empty stdout and a sanitized
`cli-error/1` on stderr with `code: "CLI-E-DOCUMENT"`.

Options require a space before their value. `--input=value`, `--catalog=value`
and `--workflow=value` are rejected with exit 2, as is the `--` token anywhere in
the input arguments. Missing, empty or repeated `--input` values are usage errors,
as is a following flag, including `-h`. Only `-` may start with a hyphen; prefix
such filenames with `./`, for example `./-model.json`.

`MAMUT_ENGINE_INPUT=<local-path>` remains available for compatibility, equivalent
to `EngineCli::new(Some(path))` in Rust. **`--input` takes precedence** and the
environment path is not opened when the flag is supplied. Without `--input`, run
reads the legacy path once at startup but reports read failures when the method
executes, preserving cancellation precedence and progress behavior. In the
environment variable, `-` is a literal path. MCP retains legacy reads per call.
SDK adoption of document transport and independent input per step remain open;
engine-input/1 cannot be represented by `NumericParameters`.

`--workflow -` or `--catalog -` reads stdin to EOF with the SDK's 16 MiB limit.
Using both, or combining either with `--input -`, is usage error 2. An input path
can be combined with workflow or catalog stdin:

```sh
mamut-engine run --input model.json \
  --catalog crates/engine-cli/catalog.json --workflow - < crates/engine-cli/run.workflow.json
```

Descriptors use dimensionless quantities for opaque input/output references.
`urn:mamut:engine:engine-input:1.0.0` refers to engine-input/1;
`urn:mamut:engine:operation-result:1.0.0` identifies a method result: results/1 for
run or `{operation:"validate",status,diagnostics}` for workflow validation.
Engine-api validates physical quantities and SI units internally.

## Output and exit codes

Identity is `engineId: "mamut-engine"`, with `engineVersion` from
`engine_api::version().engine_version`. Successful workflow execution has
`operation: "run"`; a validation method has `operation: "validate"` inside its
own result. Terminal errors identify the invoked command: `run`, `validate`,
`compile`, `methods`, `describe`, `help` or `mcp`; unknown commands use `cli`.
A leading `--timeout <s>` does not change the command identity.

The SDK's successful cli-run/1 envelope contains the execution manifest in
`data`. The manifest stores **fingerprints**, not calculated values. The adapter
preserves it and adds `engineResults`: complete API documents in step order,
including displacements, reactions and diagnostics. Run warnings retain results
and the manifest with `outcome: "warning"` and exit 1.

Direct `validate --input <path|->`, introduced by MAM-863 / ENG-SFC-012, emits a
compact JSON report with sorted keys and a final newline for every validation
status:

```json
{"diagnostics":[],"operation":"validate","schema":"validate-report/1","status":"ok"}
```

The published [validate-report/1 schema](../engine-api/schemas/validate-report-1.json)
is also available through `engine_api::schema("validate-report/1")` and registered
in capabilities. Validation checks strict JSON parsing, DTO/schema, conventions,
limits and topology. It does not assemble or solve the system or certify
stability, convergence or results. No `run --dry-run` alias is provided.

Reports with `status: "ok"` (0), `"warning"` (1) or `"invalid"` (2) go to stdout
with empty stderr, preserving the published #200 `validate-report/1` contract.
Parse, DTO, contract and internal input-limit errors are normalized to
`ENG-E-0009`, retaining the API cause and empty entities. Model diagnostics retain
their original order; any error severity produces exit 2 for direct validation.
Transport, usage, cancellation and output errors that prevent report delivery use `cli-error/1`
on stderr, with exits 2, 2, 5 and 70 respectively and empty stdout unless an
output write leaves partial bytes.

Currently `validate_with` generates no warnings, so warning/1 is tested with a
synthetic diagnostic rather than a binary fixture. E1's diaphragm recovery
warning occurs only during run; E1 validates with exit 0.
`validate --catalog … --workflow …` preserves workflow validation/compilation,
even with `--input`. Direct validation rejects workflow flags such as
`--progress ndjson` with usage/2.

The adapter retains method results and semantics in shared provider state because
the vendored `Method::run` returns only `Result<Value, String>`. It never infers
status by parsing error messages. Progress callbacks and manifests remain SDK
features; a standard SDK channel for complete results remains outside this work.

| Status/category | Exit | Mapping |
| --- | --- | --- |
| OK | 0 | `ExitCodes.ok` |
| Warning | 1 | `ExitCodes.warning`; no warning FailureKind |
| Invalid usage, document, workflow or Engine input | 2 | Usage / Document / Validation |
| Unsupported | 3 | FailureKind::Unsupported |
| NotConverged or Unstable | 4 | FailureKind::NotConverged |
| Cancelled | 5 | FailureKind::Cancelled |
| Timeout | 6 | FailureKind::Timeout; cooperative deadline expiration |
| InternalError, unclassified failure or output/progress I/O | 70 | Internal / Method |

`Outcome.status` determines run's mapping. For the engine.validate workflow
method, contract errors are Invalid; error diagnostics 0006/0007/0012/0013/0014
map as in engine-api, and other errors are Invalid. Warnings without errors
produce Warning. An unknown input contract is invalid (2).

`--progress ndjson` preserves the base events on stderr: `started` and
`finished` per step, with the same fields and bytes. Between them it emits one
`progress/1` line per `engine_api::Progress` callback, without timestamps or
durations. The published schema is [schemas/progress-1.json](schemas/progress-1.json),
also distributed through the [schema publication tree](../../schemas/progress/1.json).
For example, a stage of the first step:

```json
{"completed":1,"index":0,"operation":"run","schema":"progress/1","stage":"parse","total":7}
```

`index` is the zero-based step index, matching SDK events; `operation`, `stage`,
`completed` and `total` are copied directly from the API. `completed` counts
completed stages rather than iterations. `total` may reserve a maximum number of
stages and finish earlier. The current type exposes neither iteration nor
residual, so those fields are omitted. Non-finite numbers are rejected; a future
residual extension must represent non-finite values as `null` and revise the schema.

**Interface decision:** the existing flag includes `progress/1` by default.
Consumers can distinguish stages by `schema` and SDK events by `phase`.
Consumers that assumed exactly two lines per step must accept the added stages.
This requires no changes to the vendored parser.

The adapter reuses `EventWriter` to buffer each event together with its newline
before `write_all`, followed by `flush`; error envelopes use the same writer.
A timeout does not interleave the envelope with event fragments. Stages are
written during the callback, before the host's `with_engine_progress` observer,
without waiting for step completion. A write or flush failure returns 70 via
`cli-error/1` with `CLI-E-INTERNAL` when stderr remains writable.
A method failure preserves emitted stages and the final envelope, without
`finished`. Numbers use `engine_api::canonical_json`, as results/1 does: sorted
keys, decimal integers and floating zero normalization where applicable.
Without `--progress`, output retains its previous behavior. stdout remains a
single JSON document, byte-identical to execution without progress, as described
in [docs/DETERMINISMO.md](../../docs/DETERMINISMO.md).

### Terminal `cli-error/1` contract · ENG-CONT-001 / MAM-867

Every adapter-controlled terminal process failure without a direct validation
report uses `src/error.rs::report_failure`: usage, document reads/parsing,
workflow validation, method failures, unsupported capabilities,
instability/non-convergence, cancellation, timeout, internal errors and I/O.
It writes **one JSON line to stderr**, ending
in LF. Without progress, that line is all of stderr. With `--progress ndjson`, it
is the last event and the only one with `schema: "cli-error/1"`. No additional
cli-run failure envelope is emitted. Stdout is empty when failure precedes result
writing. Failed output writes can leave partial bytes; broken stderr returns 70
without retrying delivery.

The source schema is [schemas/cli-error-1.json](schemas/cli-error-1.json), Draft
2020-12, `$id: "urn:mamut:cli-error:1"`. It follows engine-api's
`urn:mamut:<name>:<major>` convention. It is registered in
[schema-sources.json](../../scripts/schema-sources.json) and published at
[schemas/cli-error/1.json](../../schemas/cli-error/1.json) with its digest and byte
length in [schemas/index.json](../../schemas/index.json). It does not extend
`engine_api::schema()` or advertise an analysis capability.

| Field | Meaning |
| --- | --- |
| `schema` | Always `cli-error/1` |
| `exitCode`, `outcome` | engine_v1 table; exit 2 distinguishes `usage` and `invalid_input` |
| `code` | `CLI-E-CANCELLED` for cancellation; otherwise the first engine diagnostic with error severity, or a fallback CLI code below |
| `diagnostics` | Original ordered engine diagnostics, or `[]` |
| `pointer` | RFC 6901 JSON Pointer or `null`; currently `null` because engine entities do not identify an unambiguous JSON location |
| `hint` | Primary diagnostic hint, or `null` |
| `engineId`, `engineVersion`, `operation` | Identity and command, without paths or arbitrary arguments |
| `messageKey` | Stable `errors.<code>` key for MAM-836/MAM-841 |
| `type`, `error` | Compatibility fields: `type: "result"`, `error: {code, message}`; `error.code` equals `code` |
| `engineResults` | Optional original failed document only; no previous results |

**Compatibility decision:** cli-run/1 is the shared SDK envelope; the SDK exposes
no replacement hook for all emitters. The adapter emits its own terminal
`cli-error/1` contract, retaining the previous run fields (`type`, identity,
`exitCode`, `outcome`, `error` and optional `engineResults`). Generic `WF-E-<exit>`
codes become stable CLI or engine codes: input transport's former `WF-E-0002`
becomes `CLI-E-DOCUMENT`; timeout's former `WF-E-0006` becomes `CLI-E-TIMEOUT`.
Other commands replace `{exit_code,message}` with the same envelope. Consumers
should inspect `schema` and `code`, without interpreting messages. Successful
stdout bytes, manifests and canonical run documents remain unchanged under
[docs/DETERMINISMO.md](../../docs/DETERMINISMO.md). Direct-validation reports,
including invalid reports (exit 2), retain `validate-report/1` on stdout with
empty stderr.

| Fallback CLI code | exitCode | outcome | Situation |
| --- | --- | --- | --- |
| CLI-E-USAGE | 2 | usage | Invalid command/arguments, including non-UTF-8 arguments or invalid/over-cap timeout |
| CLI-E-DOCUMENT | 2 | invalid_input | Missing, unreadable, oversized, incorrectly encoded or unparseable document |
| CLI-E-VALIDATION | 2 | invalid_input | Invalid workflow contract, parameters or input without an engine diagnostic |
| CLI-E-UNSUPPORTED | 3 | unsupported | Unsupported provider/method without an engine diagnostic |
| CLI-E-NOT-CONVERGED | 4 | not_converged | Non-convergence/instability without an engine diagnostic |
| CLI-E-CANCELLED | 5 | cancelled | Cooperative cancellation, including SIGINT/SIGTERM, markers and host controls |
| CLI-E-TIMEOUT | 6 | timeout | Cooperative deadline cancellation |
| CLI-E-METHOD | 70 | internal | Method failure without semantic classification |
| CLI-E-INTERNAL | 70 | internal | Internal state or output/progress I/O |

ENG/API codes retain their meanings. ENG-E-0006/0007/0014 map to 4, ENG-E-0013 to
3, ENG-E-0012 to 70 and other current ENG errors to 2. API-E-* maps to 2 and
API-I-CANCELLED to 5; it remains in the original diagnostics while terminal
cancellation uses CLI-E-CANCELLED. API-W-* remains warning/1 and never turns a successful
warning result into cli-error. The full catalog (32 initial entries) is
[error-codes-1.json](../../docs/contracts/error-codes-1.json): `id`, `exitCode`,
`outcome`, stable English meaning, `status`, `introduced` and `messageKey`.
`introduced: "1.0.0"` records admission to this registry rather than the historic
implementation date. A code's mapping describes its category; the overall engine
status determines the process exit when multiple diagnostics exist, and the first
error identifies the primary diagnostic. Direct model validation keeps exit 2
for all rejected reports.

[messages/en.json](messages/en.json) holds editable English base messages,
separate from identifiers and frozen meanings. Original engine messages, causes
and hints remain in diagnostics/engineResults; this work does not translate them
or implement language selection.

[error-codes-1.lock](../../docs/contracts/error-codes-1.lock) freezes complete
definitions except `status`. It is **append-only**: never remove or rewrite an
entry; a new meaning requires a new ID and appended definitions in the registry,
lock and independent `FROZEN_DEFINITIONS` anchor in `tests/error_registry.rs`, plus
messages. `status: "deprecated"` retains the ID and definition. Tests read the
real files through `include_str!` and compare the lock to that anchor. They reject
coordinated registry/lock deletions, reordering and frozen-field changes, as well
as duplicate IDs, including deprecated ones. Negative tests mutate each frozen
entry and field. The anchor is reviewed code, not immutable storage; review must
ensure it only grows by appending entries.

Coverage scans `crates/*/src/**/*.rs`, including const/static definitions,
`CodeInfo` catalogs and codes passed indirectly to emitters. The lexical inventory
ignores comments, character literals and `cfg(test)` items. It covers all
`^(ENG|API|CLI|WF)-[A-Z]-` families, including warning codes, and rejects
unregistered prefixes such as `ENG-E-` in `concat!` and `format!`. Adjacent literal
fragments in macros, string additions and array joins are checked for assembled
family prefixes too; dynamic construction cannot reserve a family implicitly.
It does not inspect vendor or evaluate arbitrary Rust expressions. Mutation
tests cover macro fragments, joins and unregistered warning literals.
ENG-E-0014 is registered as 4 / not_converged. The local schema
test validator uses engine-api's Draft 2020-12 subset, fails on unknown keywords
and never resolves network references.

**MCP boundary:** `mamut-workflow-cli/src/lib.rs::serve` uses
`McpServer::handle_with` and `CliFailure::envelope` for tool errors;
`mamut-workflow-execution/src/mcp.rs` constructs JSON-RPC errors. Both are in
`vendor/mamut-workflows/`. `CliConfig` has no dispatch hook to replace these
frames. SDK RPC frames, including WF-E-* tool errors, retain their stdout
contract; no per-call stderr errors are added. Terminal failures opening the
catalog or reading/writing MCP transport use `cli-error/1`. An RPC error can end
a session with exit 0; it is distinct from terminal process failure.

## Progress, cancellation and timeout

`--progress ndjson` uses SDK `started` and `finished` events on stderr per step,
without timestamps, written and flushed immediately. The adapter serializes each
event and its newline into a complete buffer before `write_all`; the error
envelope uses the same mechanism. Timeout cannot interleave an envelope with
event fragments. Method failure emits started and the terminal envelope without
finished. Internal `engine_api::Progress` stages remain available through the
library's `with_engine_progress` callback rather than another NDJSON dialect.

`Control` connects `CancelFlag` to an `Arc<engine_api::CancelToken>`. They are
synchronized before execution and after each engine callback; engine-api also
checks the token between stages/iterations. `Control::cancel()` cancels both;
`flag()` and `token()` let the host attach observers. The library supports
cancellation before startup and at completed-stage boundaries.

`MAMUT_ENGINE_CANCEL_FILE=<local-path>` requests cooperative cancellation (5) when
the path exists. It may exist before launch or appear during execution and is
checked before each step and after engine callbacks. Use
`Control::with_cancel_file` in the library. The adapter never creates or removes
the marker; use a separate path per execution. An absent path or failed existence
check does not request cancellation. Once observed, cancellation remains active
even if the marker disappears.

`--timeout <s>` and `MAMUT_ENGINE_TIMEOUT=<s>` set seconds, allowing fractional
values or scientific notation. Values must be finite, strictly positive and at
most **31536000 seconds (1 year)**. The flag overrides even an invalid environment
variable. Invalid or over-cap values, missing values and repeated flags are
usage/2, with `CLI-E-USAGE` and empty stdout. `--timeout` is not extracted from
the value position of `--catalog`, `--workflow`, `--progress` or `--input`.

```sh
MAMUT_ENGINE_INPUT=model.json mamut-engine run --timeout 2.5 \
  --catalog crates/engine-cli/catalog.json --workflow crates/engine-cli/run.workflow.json
```

Timeout is validated for all commands before input preparation. Direct
`validate --input` accepts the flag/environment value but does not start a timer
or enforce a deadline; it retains external cooperative cancellation. For example,
`validate --input model.json --timeout 60` produces the same bytes as without it.

For SDK-dispatched commands, the deadline starts after argument parsing, before
dispatch. A `std` thread with a monotonic `Instant` deadline cancels the existing
token; `Control` synchronizes the SDK flag on subsequent checks. Computations and
callbacks stay on the calling thread. Deadline cancellation gives empty run
stdout and a terminal `cli-error/1`, `code: "CLI-E-TIMEOUT"`, `exitCode: 6`,
`outcome: "timeout"`, message `Execution timed out.`, without partial documents.
A terminal success (including warning) or a non-cancellation error already
obtained from dispatch keeps its result even if the deadline expires before the
subsequent check. Only cancellation caused by the timer becomes exit 6;
previously observed external cancellation retains 5. The timer is woken and
joined on completion without waiting for its deadline. Without timeout no thread
is created. `describe` retains the SDK's complete table, including historical
`reserved_exit_code_categories` metadata.

Cancellation is cooperative: analysis loops check between iterations, rows,
members and substeps; LDLᵀ also checks during factorization and substitution.
Third-party backends without cooperative overrides only check on entry. Blocking
stdin/file reads and SDK I/O cannot be interrupted by this token, so the deadline
is not a hard bound on those operations. MCP retains its own RPC frames/errors.

On **Unix**, the binary installs SIGINT (Ctrl-C) and SIGTERM handlers after
argument parsing, before dispatch. The safe `signal_hook::flag` API
only sets an `Arc<AtomicBool>` in the handler. A `std` thread polls it
every 5 ms and cancels the same `CancelToken` used by the deadline; `Control` synchronizes
the SDK flag on its next check. It performs no I/O or callbacks in the handler.
When `run` observes this cancellation, it leaves stdout empty and exits with **5** and
a `cli-error/1` envelope on stderr with `code: "CLI-E-CANCELLED"`,
`exitCode: 5`, `outcome: "cancelled"` and the message `Execution was cancelled.`,
distinguishable from timeout (6). With NDJSON the terminal envelope follows
complete events. A timeout that has already caused
cancellation retains code 6; a terminal result already obtained retains its
semantics. The cooperative computation and I/O limits described above apply;
these signals do not force blocking reads to stop. As with other external
cancellation, only the failed API document is retained in `engineResults` when
available; previous steps and partial manifests are omitted.

Known limitations:

- A signal received before handlers are installed (at startup, during
  argument parsing) follows the OS default disposition: the process
  is killed by the signal, without output or code 5.
- `signal_hook::flag::register` replaces the inherited disposition. If the
  parent process left SIGINT ignored (e.g. background jobs in a non-interactive
  bash), the CLI stops ignoring it and SIGINT cancels with code 5.

On completion, signal registrations are removed and the thread is woken and joined,
without creating temporary files or directories. The library does not install signal
handlers by default; on Unix the host can opt in with `EngineCli::with_signals()`.
Handlers affect the entire process and, according to `signal-hook`, removing
registrations does not restore previous OS dispositions.
On **non-Unix platforms**, neither this functionality nor the `signal-hook`
dependency is compiled, and `with_signals()` is unavailable; signals retain the
host's behavior and do not guarantee cooperative code 5. Timeout and the marker remain
available.

## Engine-api coverage

| API operation | CLI coverage | Remaining gap |
| --- | --- | --- |
| version | `describe.version` uses engine-api's version; run includes engineVersion | No version().contracts, version command or --version flag |
| capabilities | `describe` exposes identity, commands and engine_v1 exits; `methods --catalog …` lists engine.run/engine.validate | No complete API capabilities document |
| schema | Method descriptors reference contracts and parameters | No schema command for API JSON Schemas |
| explain | Results retain engine diagnostics | No engine_api::explain command |
| validate | Direct model validation and validate-report/1 with `validate --input …`; workflow compilation with explicit catalog/workflow | SDK adoption of input transport |
| run | Embedded engine.run workflow with `run --input …`; explicit catalog/workflow also supported | SDK adoption of transport/result extensions |

`capabilities`, `schema`, `explain`, `version` and `--version` are rejected by the
SDK parser with 2. `compile` retains the standard workflow plan. MCP injects the
provider, identity, cancellation and engine_v1 exit table, but its tools still
return fingerprint manifests and unclassified method failures (70). Full MCP
results and semantic-code parity remain open; warnings retain generic behavior.

## Verification and platform

Tests use local files and the binary identified by
`env!("CARGO_BIN_EXE_mamut-engine")`. Library tests also cover cancellation between
stages and failing writers, without signals, shells or network access. Binary
tests cover **0/1/2/3/4/5/6/70**, describe identity/table, stdin for input/catalog/
workflow, the 8 MiB boundary, invalid UTF-8, stdin conflicts, flag precedence,
byte-identical successful outputs, NDJSON, engine-api parity, manifest
preservation, ordered multi-step results and the two validation modes. All
terminal failure streams are checked against cli-error/1 and the code registry.

Exit 4 covers instability and P-Delta non-convergence. Binary exit 70 uses stdout
closed before supplying workflow stdin to avoid a race. Exit 5 uses markers before
and after launch while stdin holds the workflow, without sleeps or signals.
On Unix, SIGINT and SIGTERM binary tests wait for `started`, send the signal
with `kill`, and require exit 5 within 1 s, empty stdout, complete JSON lines and
a final `cli-error/1` envelope with `CLI-E-CANCELLED`, with and without a long
timeout.
Exit 6 uses flag/environment deadlines against a valid heavy consolidation model
with four layers, 80 elements and 72000 substeps, requiring completion within
timeout + 1 s. Tests also cover invalid/over-cap timeout values, flag precedence,
unchanged successful bytes and prompt joining for long deadlines. Unit tests
establish terminal-result precedence without a clock and complete-line writes;
NDJSON binary cases check each line and the final timeout envelope. See
[MAM-865 verification and signal decision](../../provenance/ENG-SFC-014.md).

Direct-validation tests cover schema, internal limits, topology, stdin/path
determinism, cancellation and absence of solving. They reuse engine-api's schema
test validator with negative mutations at each level; warning classification
uses a synthetic diagnostic.

Reproduce the in-process benchmark (median of 11 alternating warmed repetitions,
using only `runs[0].engine_input` from E1):

```sh
CARGO_TARGET_DIR=/workspace/.cargo-target/cont1 cargo test -p engine-api \
  --test validate_performance --offline -- --nocapture
```

The 2026-10-21 debug measurement with Rust 1.99.0 was validate **5.44 ms**, run
**151.46 ms**, ratio **27.84×**. The test requires at least 10× and remains active
in CI. Deserialization and input extraction are outside the measurement; no
fixture reference results are read. See [ENG-SFC-012](../../provenance/ENG-SFC-012.md).
Earlier offline verification and test counts are in
[ENG-SFC-002](../../provenance/ENG-SFC-002.md).

The `wasm32-unknown-unknown` check compiles the whole workspace including this
crate. It does not execute the CLI, which requires native stdio, files and an
application environment. **Pending:** Windows/Linux/macOS CI binaries (Jonathan's
decision), contract review and clean-room review (J4).
