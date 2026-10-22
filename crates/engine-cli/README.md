# ENG-SFC-002 · MAM-91 · mamut-engine

Native Engine CLI, package `engine-cli`, binary `mamut-engine`, a workspace
member and default member. Rust 1.99 as specified in `rust-toolchain.toml`;
`license.workspace = true` inherits **AGPL-3.0-only**. On Unix it uses
`signal-hook =0.4.5` (decision #203); it uses no network, telemetry or helper processes.

The previous blocker is resolved by SDK-WF-012 / MAM-493. It uses the
vendored `mamut-workflow-cli` and `mamut-workflow-execution` libraries from
commit `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`. `EngineCli::run` calls the
public `parse_args` and a single base `dispatch` for workflows with `CliConfig`,
`MethodProvider`, `CliIdentity` and `ExitCodes::engine_v1()`. The adapter extracts
`--timeout <s>` in `timeout::parse_timeout_args` and then pre-parses
`--input` and the default documents. The SDK parser, compilation and
workflow execution remain in the base. `validate --input` without workflow
documents calls `engine_api::validate_with` directly. Vendor is unchanged.

## Usage and input

From the repository root, with Cargo offline:

```sh
cargo build -p engine-cli --offline
cargo run -p engine-cli --offline -- describe
cargo run -p engine-cli --offline -- methods --catalog crates/engine-cli/catalog.json
cargo run -p engine-cli --offline -- run \
  --input crates/engine-api/tests/fixtures/determinism/cantilever.json --progress ndjson
cargo run -p engine-cli --offline -- run --input - < modelo.json
cargo run -p engine-cli --offline -- validate --input modelo.json
cargo run -p engine-cli --offline -- validate --input - < modelo.json
```

`catalog.json` is the CLI's own catalog, also embedded as `engine_cli::CATALOG`.
It registers `engine.run` v1 and `engine.validate` v1. The former calls
`engine_api::run`; it supports the analyses exposed by that facade (currently
linearStatic, pDelta, modal and responseSpectrum). The latter calls
`engine_api::validate_with`. It can also be selected from `run` with
`validate.workflow.json`. Both methods receive `{}` parameters; the provider
rejects numeric parameters and other methods/versions, without using IdentityProvider.

If both `--catalog` and `--workflow` are omitted, `run` uses `engine_cli::CATALOG`
and `engine_cli::RUN_WORKFLOW`. `validate --input` validates the model directly,
without a workflow or temporary files. For compatibility, `validate` without `--input`
still uses the catalog and `engine_cli::VALIDATE_WORKFLOW`. The documents are embedded with
`include_str!`. The adapter materializes them in a private temporary directory
for base dispatch and removes it on completion; their paths do not appear in
the output. With both arguments explicit, the previous behavior is preserved:
`run` executes the selected workflow and `validate` only validates/compiles that
workflow, without executing engine-api. Specifying only one of them remains
invalid usage (2).

**Open transport decision (SDK adoption):** `--input <path|->` is
now the adapter's primary path for engine-input/1. `-` reads stdin until
EOF. Stdin is read in full once before dispatch and all steps
reuse the document in memory; there is no per-step streaming. Reading
stops when the `engine_api::MAX_INPUT_BYTES` limit is exceeded
(8 MiB, 8 388 608 bytes); exactly that size is accepted. Exceeding the limit, a
nonexistent path, a read error or invalid UTF-8 produces code 2, empty stdout and
a sanitized error envelope on stderr. Repeating `--input` also produces 2.

A space is required between flag and value: `--input=value` is rejected with 2,
as are `--catalog=value` and `--workflow=value` in the base parser. A
single option syntax is preserved. The `--` token does not act as a separator:
it is rejected with 2, including before or after `--input`. `--input` without a value,
with an empty value or followed by another flag (including `-h`) produces 2; `-` is the only
value that may start with a hyphen. For a path whose name starts with a hyphen,
use a prefix such as `./-modelo.json`.

`MAMUT_ENGINE_INPUT=<local-path>` is retained for compatibility (in Rust,
`EngineCli::new(Some(path))`), with the same output bytes for commands
with explicit catalog/workflow arguments. **If both are specified, `--input` wins** and
the environment variable's path is not opened. Without `--input`, the path is read once at the start
of `run`, but its errors are reported when executing the method, preserving the
existing cancellation priority and progress. In the environment variable, `-` remains
a literal path. MCP retains the old path's per-call reading.
SDK adoption of document transport and independent per-step input
remain open; engine-input/1 does not fit in `NumericParameters`.

`--workflow -` **or** `--catalog -` consumes stdin until EOF, with the base's 16 MiB
limit. Using both at once is invalid usage (2), as is `--input -` together
with either of them. Input from a path allows workflow or
catalog input from stdin. For example:

```sh
mamut-engine run --input modelo.json \
  --catalog crates/engine-cli/catalog.json --workflow - < crates/engine-cli/run.workflow.json
```

Descriptors use dimensionless quantities for opaque input/output
references, rather than to represent model quantities. The local reference
`urn:mamut:engine:engine-input:1.0.0` refers to engine-input/1;
`urn:mamut:engine:operation-result:1.0.0` identifies the method result:
results/1 for run, `{operation:"validate",status,diagnostics}` for validate.
Units and physical quantities are validated inside engine-api (SI).

## Output, codes and cancellation

The configured identity is `engineId: "mamut-engine"`, `engineVersion` obtained
from `engine_api::version().engine_version`, `operation: "run"`. The envelope's
operation represents workflow execution; the validation method reports
`operation: "validate"` in its own result.

The base returns a cli-run/1 envelope whose `data` is the execution manifest.
This stores **fingerprints**, rather than computed values. The adapter preserves the
entire manifest and adds `engineResults` to the terminal envelope: an array of
complete engine-api documents, in step order. Thus `run` stdout contains the
actual results/1, including displacements, reactions and diagnostics. Run warnings
preserve data and the manifest with `outcome: "warning"` and code 1.

Since MAM-863 / ENG-SFC-012, `validate --input <path|->` without catalog/workflow
arguments directly emits a single compact JSON document, with sorted keys and a final newline:

```json
{"diagnostics":[],"operation":"validate","schema":"validate-report/1","status":"ok"}
```

This decision replaces the cli-run/1 envelope of the shorthand introduced in
#197 (MAM-862). Its published contract is
[`validate-report/1`](../engine-api/schemas/validate-report-1.json), also available
through `engine_api::schema("validate-report/1")` and registered in capabilities.
`status` is `ok` (0), `warning` (1) or `invalid` (2). This is a dry run of the **model**:
it checks strict JSON parsing, DTO/schema, conventions, limits and topology
through `validate_with`. It does not assemble/solve the system or certify stability,
convergence or results. The optional alias `run --dry-run` is not added.

In all three states the report goes to stdout and stderr remains empty. Parsing,
DTO, contract or internal limit errors are normalized to `ENG-E-0009`
(incompatible contract), with the API cause and empty entities. Model diagnostics
are preserved in full and in their original order; any `error` severity
produces `invalid`/2. Transport failures (including exceeding 8 MiB),
usage errors and cancellation retain empty stdout and a sanitized error on stderr as before;
cancellation produces 5 and a write failure produces 70.

Currently `validate_with` does not generate warnings and no validation fixture
produces 1. The warning/1 mapping is tested with a synthetic diagnostic; warnings
are not invented for the binary. E1's diaphragm recovery warning is generated
only during run, so E1 produces 0 when validating. `validate --catalog … --workflow …`
retains exactly its workflow validation/compilation, including with `--input`.
The direct form rejects workflow flags such as `--progress ndjson` with usage/2.

During workflow execution, input or method failures or cancellation leave stdout empty; stderr ends
with the error envelope. An I/O failure may leave an incomplete write.
If engine-api produced a failed document, `engineResults` contains only that
document, without previous steps' results or a partial manifest. Transport errors
or prior cancellation have no documents. I/O causes and paths
are not included in adapter messages. Engine diagnostics are
preserved as returned by its API.

**Open output decision:** approve the cli-run/1 `engineResults` extension
and the validate method's report as published contracts, or add to the
SDK a standard channel for complete results and typed failures. The adapter
retains results/semantics in shared provider state because the
vendored `Method::run` only returns `Result<Value, String>`; it does not infer statuses
by parsing messages. The progress callback and manifest remain part of the SDK.

| Status/category | Code | Mapping |
| --- | --- | --- |
| OK | 0 | `ExitCodes.ok` |
| Warning | 1 | `ExitCodes.warning`; the base has no warning FailureKind |
| Invalid usage, document, workflow or Engine input | 2 | Usage / Document / Validation |
| Unsupported | 3 | FailureKind::Unsupported |
| NotConverged or Unstable | 4 | FailureKind::NotConverged |
| Cancelled | 5 | FailureKind::Cancelled |
| Timeout | 6 | FailureKind::Timeout; cooperative deadline expiration |
| InternalError, unclassified failure or output/progress I/O | 70 | Internal / Method |

The `Outcome` status determines the run mapping. In a workflow's engine.validate
method, contract errors
are Invalid; error diagnostics from catalog 0006/0007/0012/0013 are mapped
as in engine-api, and the rest are Invalid. A warning diagnostic without errors
produces Warning. An unknown input contract is invalid (2), as in engine-api.

`--progress ndjson` uses base events on stderr: `started` and `finished`
per step, without timestamps, written and flushed immediately. The adapter
serializes each event together with its newline into a complete buffer before
passing it to stderr via `write_all`; it does the same with the error envelope.
A timeout does not interleave the envelope with event fragments. A method failure
emits started and the final envelope, without finished. stdout remains a single JSON document.
Internal `engine_api::Progress` stages are available in the library through
`with_engine_progress`; they do not yet form another NDJSON dialect.

`Control` connects `CancelFlag` to an `Arc<engine_api::CancelToken>`. They are synchronized
before execution and after each engine callback; the token is also
checked by engine-api between stages/iterations. `Control::cancel()` cancels
both; `flag()` and `token()` allow connecting host observers.
The library API can cancel before starting or during a completed stage.

The binary supports `MAMUT_ENGINE_CANCEL_FILE=<local-path>`: the presence of that
path requests cooperative cancellation (5). It may exist before starting or be
created during execution; it is checked before each step and after
engine callbacks. In the library it is configured with `Control::with_cancel_file`.
The marker is neither created nor removed; use a separate path for each execution. A path
that is absent or whose existence cannot be checked does not request cancellation. Once
observed, cancellation remains active even if the marker is removed.

`--timeout <s>` and `MAMUT_ENGINE_TIMEOUT=<s>` set a deadline in seconds,
allowing fractional values or scientific notation, finite and strictly positive,
with an inclusive maximum of **31536000 seconds (1 year)** for both forms.
The flag takes precedence even if the environment variable is invalid; invalid values, a value
above the maximum, a missing value or repeated flags result in invalid usage (2),
with the usual error envelope and empty stdout. `--timeout` is not extracted when
it occupies the value position of `--catalog`, `--workflow`, `--progress` or `--input`;
the input pre-parser and the base validate those arguments. For example:

```sh
MAMUT_ENGINE_INPUT=modelo.json mamut-engine run --timeout 2.5 \
  --catalog crates/engine-cli/catalog.json --workflow crates/engine-cli/run.workflow.json
```

The timeout is parsed and validated for all commands, before pre-parsing
`--input`. Direct validation with `validate --input` accepts the flag and the
environment variable, but does not start a timer or enforce the deadline; it retains its
`validate-report/1` report and external cooperative cancellation. For example,
`validate --input modelo.json --timeout 60` produces the same bytes as without a timeout.

For commands dispatched by the base, the deadline starts after argument
parsing, before dispatch. A
`std` thread, with a monotonic `Instant` deadline, cancels the existing `CancelToken`;
`Control` synchronizes the SDK flag on subsequent checks. It does not run computations
or callbacks on another thread. If dispatch observes cancellation from the deadline,
`run` leaves stdout empty and emits the same type of error envelope as cancellation,
with `exitCode: 6`, `outcome: "timeout"`
and the message `Execution timed out`. The adapter identifies expiration without
changing vendor or engine-api statuses; it omits partial documents.
A terminal success result (including warning) or an error other than cancellation
already obtained from dispatch retains its code, even if the deadline has expired when
checked afterward: the timeout only converts cancellation caused by the
timer to 6. Cancellation already observed retains code 5.
The timer is woken and joined on completion, without waiting for the deadline;
without a timeout, no thread is created
and the previous bytes are preserved. `describe` retains the entire table from the
vendored SDK, including its historical `reserved_exit_code_categories` metadata.

This is cooperative cancellation: analysis loops check the token between
iterations, rows, members and substeps; LDLᵀ also checks it during
factorization and substitution. Kernels in third-party backends that do not override
the cooperative variants only check on entry. Blocking reads
from stdin/files and SDK I/O cannot be interrupted with this token;
the deadline is not a hard limit on those operations. MCP retains its
own frames and errors; the terminal semantics described here apply to `run`.

On **Unix**, the binary installs SIGINT (Ctrl-C) and SIGTERM handlers after
argument parsing, before dispatch. The safe `signal_hook::flag` API
only sets an `Arc<AtomicBool>` in the handler. A `std` thread polls it
every 5 ms and cancels the same `CancelToken` used by the deadline; `Control` synchronizes
the SDK flag on its next check. It performs no I/O or callbacks in the handler.
When `run` observes this cancellation, it leaves stdout empty and exits with **5** and
a cli-run/1 envelope on stderr with `exitCode: 5`, `outcome: "cancelled"` and the message
`Execution was cancelled`, distinguishable from timeout (6). With NDJSON the
terminal envelope follows complete events. A timeout that has already caused
cancellation retains code 6; a terminal result already obtained retains its
semantics. The cooperative computation and I/O limits described above apply;
these signals do not force blocking reads to stop.

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

## Actual mapping to engine-api

| engine-api operation | Standard command / actual coverage | Pending |
| --- | --- | --- |
| version | `describe.version` is the engine-api version; run includes engineVersion | Does not publish `version().contracts`; no version command or --version flag |
| capabilities | `describe` publishes identity, commands and engine_v1 codes; `methods --catalog …` lists engine.run and engine.validate | Does not publish the complete capabilities document (elements, analysis methods, schemas) |
| schema | Method descriptors reference contracts and parameters; they are not engine-api JSON Schemas | No standard command for `schema("engine-input/1")` / `schema("results/1")` |
| explain | Results preserve engine diagnostics | No standard command for `engine_api::explain(code)` |
| validate | `validate --input …` validates the model directly and emits validate-report/1; `validate --catalog … --workflow …` retains **workflow** validation/compilation | SDK adoption of transport |
| run | `run --input …` executes engine.run with embedded documents; accepts explicit catalog/workflow arguments | Approval of the transport/output extension above |

The commands/flags `capabilities`, `schema`, `explain`, `version` and `--version`
are not added: the base parser rejects them with 2. `compile` retains the standard
workflow plan. The table does not claim coverage for operations without an equivalent.

The base also retains `mcp --catalog …`: it injects the provider, identity,
cancellation and engine_v1 table, but its tools still return fingerprint
manifests and unclassified method errors (70). The run terminal extension
does not apply to MCP frames; warnings there retain generic behavior.
MCP parity for complete data and semantic codes is an open decision
outside this ticket's describe/methods/validate/run mapping.

## Verification and platform

The CLI's tests create local files and execute its binary through
`env!("CARGO_BIN_EXE_mamut-engine")`; on Unix they also use `Command::new("kill")`
with `-INT` and `-TERM`, without unsafe code. Cancellation between stages and writer
failures are also tested in the library, without signals, shells or network access.
They cover **0/1/2/3/4/5/6/70 with actual binary executions**, identity/table in describe,
input/catalog/workflow from stdin, 8 MiB limits, invalid UTF-8, stdin
conflicts, precedence over the environment variable and identical bytes across transports,
NDJSON, result equality with engine-api,
manifest preservation, result ordering across multiple steps and the distinction
between the two forms of validation. Code 4 includes
instability and P-Delta non-convergence; the binary's code 70 uses a stdout pipe
closed before providing the workflow through stdin, without depending on a race.
Code 5 uses a marker before and after launching the binary, while stdin
holds the workflow; it does not depend on sleeps or signals.
Additionally, two `cfg(unix)` tests reuse the heavy consolidation case
for SIGINT and SIGTERM, with and without a long timeout. They wait for `started` before
sending the signal, require exit code 5 in less than 1 s, empty stdout, complete JSON
lines and a final `cancelled` envelope; they check temporary directories
and remove fixtures on completion. Code 6 is verified through
the flag and environment variable with a heavy consolidation case of four layers, 80 elements and
72000 substeps, valid within the public bounds; completion is required in less than
timeout + 1 s. Tests also cover invalid values, flag precedence,
byte-identical output and immediate thread joining with a long timeout.
A unit test without a clock establishes terminal result precedence over
expiration. Another checks that events and errors reach the writer as
complete JSON lines. The binary combines NDJSON with several deadlines via the flag and
environment variable: each stderr line must be valid JSON and the last must be the timeout envelope.

The MAM-865 verification from 2026-10-21 and the decision on signals are in
[`provenance/ENG-SFC-014.md`](../../provenance/ENG-SFC-014.md).

MAM-863 adds binary tests for valid/invalid reports involving schemas,
internal limits and topology, stdin/path determinism, cancellation and absence of
solving. It reuses engine-api's test schema validator, with
negative mutations at each level, without new dependencies or changes to the
other schemas. The warning branch is tested in the report classifier.

Reproducible performance (in process, median of 11 alternating repetitions after
warming up both paths; only E1's `runs[0].engine_input`):

```sh
CARGO_TARGET_DIR=/workspace/.cargo-target/sfc12 cargo test -p engine-api \
  --test validate_performance --offline -- --nocapture
```

Debug measurement with Rust 1.99.0 on 2026-10-21: validate **5.44 ms**, run
**151.46 ms**, ratio **27.84×**. The test requires ≥10× and remains active in CI. Deserialization and
input extraction are excluded from the measurement; fixture reference values
are not consulted. See [provenance ENG-SFC-012](../../provenance/ENG-SFC-012.md).

Offline verification and the test count from 2026-10-17 are recorded in
`provenance/ENG-SFC-002.md`, with Rust 1.99.0 and
`CARGO_TARGET_DIR=/workspace/.cargo-target/me-sfc2`.

The `wasm32-unknown-unknown` check verifies compilation of the entire workspace,
including this crate; no exclusions are required. This does not imply CLI execution:
the binary needs stdio, files and a native host environment.

**Pending:** Windows/Linux/macOS binaries in CI (**Jonathan decides**),
approval of contract decisions and clean-room review
**PENDING (J4)**. `.github/`, `clean-room/` and vendor are unchanged.
