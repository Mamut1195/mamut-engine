# mamut-workflow-cli

SDK-WF-012 (MAM-493) exposes the local CLI to external engines. The library has
no network runtime, process spawning, signal handler, or new external dependencies.
It uses the workspace version, Apache-2.0 license, and Rust 1.83 minimum version.

## Embed a CLI

`run(args, stdin, stdout, stderr, &config) -> u8` accepts arguments **without** the
executable name and caller-owned `Read`/`Write` streams. It parses, dispatches,
and prints errors. `parse_args` and `Command` are public for hosts that need to
inspect a command first. `dispatch(command, stdin, stdout, stderr, &config)` writes
successful results but returns `CliFailure` to the caller without printing it.
Parser errors use the default codes; `run` and `dispatch` apply the configured table.

```rust,no_run
use mamut_workflow::{CompiledStep, MethodDescriptor, RegisteredMethod};
use mamut_workflow_cli::{CliConfig, CliIdentity, ExitCodes, Method, MethodProvider};
use serde_json::{json, Value};

struct EngineProvider;
struct EngineMethod { n: f64 }

impl Method for EngineMethod {
    fn run(&mut self) -> Result<Value, String> {
        Ok(json!({"computed": self.n * 3.0}))
    }
}

impl MethodProvider for EngineProvider {
    fn create(
        &self,
        descriptor: &MethodDescriptor,
        step: &CompiledStep<RegisteredMethod>,
    ) -> Result<Box<dyn Method>, String> {
        if descriptor.method_id != "sample" {
            return Err("unsupported engine method".into());
        }
        let n = *step.parameters.get("n").ok_or("missing n")?;
        Ok(Box::new(EngineMethod { n }))
    }
}

fn main() -> std::process::ExitCode {
    let config = CliConfig::new()
        .with_provider(EngineProvider)
        .with_identity(CliIdentity {
            engine_id: "my-engine".into(),
            engine_version: "1.2.0".into(),
            operation: "simulate".into(),
        })
        .with_describe("my-engine-cli", vec!["describe".into(), "run".into()])
        .with_exit_codes(ExitCodes::engine_v1());
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::ExitCode::from(mamut_workflow_cli::run(
        &args,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
        &config,
    ))
}
```

The provider receives the validated descriptor and compiled step, including
numeric parameters. Its `Box<dyn Method>` is executed by `Executor`; the result's
JCS/SHA-256 fingerprint is recorded in the manifest's step `outputs.result` scope.
The existing manifest contract stores fingerprints, not the raw method output.
Factories are invoked at each step after cancellation is checked and the started
event is emitted; cancelled steps never construct an engine instance. Factory and
method errors are unclassified method failures; they do not produce a manifest.

`IdentityProvider` is public and preserves selection identity / descriptor
`failure` behavior. `run_registered_with` accepts a provider, progress listener,
and cancellation token. `run_one_method_with` accepts a provider for a one-step
method call. Existing `run_registered` / `run_one_method` signatures retain their
reference behavior and remain available from `mamut-workflow-run`.

`CliIdentity` controls `engineId`, `engineVersion`, and `operation` in success and
failure envelopes. `with_identity` also initializes the describe/usage name;
`with_describe` overrides the advertised name and commands. It does not register
new parser commands. The additive `cli_run_success_with_identity` and
`cli_run_failure_with_identity` envelope helpers support independent hosts; the
old helpers retain their reference identity and existing output.

## Input, progress, and cancellation

Use `--catalog -` **or** `--workflow -` to read a UTF-8 JSON document from the
supplied stdin reader, through EOF, up to the public `MAX_DOCUMENT_BYTES` limit
(16 MiB, shared with MCP message bodies). Local files have the same limit. Reads consume at most the
limit plus one byte; a document exactly at the limit is accepted for parsing.
Oversized input uses the configured document failure code (3 by default), with
no path or reader diagnostic in the error. Both options at once are a usage
error, rejected before reading input. Other paths use local files. MCP sessions
need a catalog file: reading a catalog from stdin consumes that stream through EOF,
leaving no protocol messages to serve.

`run --catalog catalog.json --workflow workflow.json --progress ndjson` emits
one JSON line per progress event on stderr, with `phase` (`started` / `finished`),
zero-based `index`, `capability_slot`, `method_id`, and numeric `method_version`.
Events follow execution order and have no timestamp. A failing method emits its
`started` event and stops without `finished`, matching `ProgressListener`.
Successful stdout remains the single cli-run result. On failure, the final stderr
line is the cli-run failure, following any progress lines.

`with_ndjson(true)` enables the same stream by configuration, and
`with_progress_writer(writer)` redirects progress alone. Each event is flushed.
`with_progress(listener)` receives the same events alongside NDJSON. Progress I/O
errors produce the configured internal failure code and no success result.

`with_cancellation(token)` accepts any `Cancellation`, including a cloned
`CancelFlag` or a borrowed token. The executor checks it **before each method
call**. A listener can cancel a shared `CancelFlag` after step N finishes; step
N+1 never starts. The flag uses local shared state; no OS signals, threads, or
forced interruption are installed. Cancellation produces a cli-run failure with
`outcome: "cancelled"` and no `data` / manifest, even after earlier steps ran.
The default public constant is `EXIT_CANCELLED = 6`.

## Exit tables

Numeric codes are independent of failure semantics. In particular Engine v1's
code 5 means cancelled, while the reference CLI's code 5 means method error.
`ExitCodes` has public fields and a builder hook (`with_exit_codes`), so engines
can adjust the complete table. `FailureKind` retains the semantic classification
when several categories share a numeric code.

| Meaning | Reference CLI | `ExitCodes::engine_v1()` |
| --- | --- | --- |
| OK | 0 | 0 |
| Warning (reserved for engine adapters) | 1 (reserved) | 1 |
| Usage / invalid arguments | 2 | 2 |
| Unreadable / malformed document | 3 | 2 |
| Invalid workflow | 4 | 2 |
| Unsupported (adapter category) | 5 (reserved; method fallback) | 3 |
| Not converged (adapter category) | 5 (reserved; method fallback) | 4 |
| Unclassified method / factory failure | 5 | 70 |
| Cancelled | **6** (`EXIT_CANCELLED`) | **5** |
| Timeout (reserved; no timer installed) | 10 (reserved; internal fallback) | 6 (reserved) |
| Internal / I/O failure | 10 | 70 |

This executor does not infer warnings, unsupported operations, or convergence
from arbitrary method error strings. Adapters can use `ExitCodes::code` and the
explicit envelope helpers for such classifications. Errors in `run` envelopes
use fixed sanitized messages. Document read failures use sanitized messages for
all commands; other non-run failures retain diagnostic messages.

## Compatibility and dependency direction

The `mamut-workflow` binary stays in the `mamut-workflow-run` package with the
same `open-local` feature and install/build commands. Its entire entry point
calls this library with `CliConfig::default()`.

To avoid a cycle, the execution implementation (including its original unit tests
and MCP server) lives in the internal `mamut-workflow-execution` crate.
`mamut-workflow-run` reexports those exact types, keeping existing public paths;
`mamut-workflow-cli` depends directly on the implementation. There is no duplicated
executor or adapter type conversion. The existing MCP server retains its reference
protocol. The reusable dispatch also applies the configured provider, identity,
exit codes, progress and cancellation to MCP method tools; initialization uses the
configured engine id/version. Tool failures are reported inside MCP results while
the serving session continues. Direct `McpServer::new(...).handle(...)` calls keep
their original defaults; `handle_with` offers an additive method execution hook.
`compile_one_method` exposes the single-slot plan for hosts applying observers.

Keeping the binary target in its original package preserves the integration
tests' `CARGO_BIN_EXE_mamut-workflow`. The conformance tests still locate it using
`CARGO_TARGET_TMPDIR` (and therefore respect `CARGO_TARGET_DIR`, MAM-490).
`describe.exit_codes` preserves the default CLI's legacy numeric object.
The additive `exit_code_categories` object publishes emitted categories without
losing meanings when several categories share one number. With
`ExitCodes::default()`, it contains only `ok`, `usage`, `document`, `validation`,
`method`, `cancelled` and `internal`. The separate
`reserved_exit_code_categories` object contains `warning`, `unsupported`,
`not_converged` and `timeout`: the generic CLI does not emit these categories,
even when their numbers overlap emitted categories.

Nondefault tables advertise adapter categories `warning`, `unsupported` and
`not_converged` as emitted; adapters are responsible for producing them. Timeout
remains in `reserved_exit_code_categories` for all tables. Custom numeric tables
retain one legacy label per number; Engine v1 publishes `0: ok`, `1: warning`,
`2: invalid`, `3: unsupported`, `4: not_converged`, `5: cancelled`, `6: timeout`
(reserved), and `70: internal` in `exit_codes`. `ExitCodes::capabilities()` supplies
all three objects. The execution crate owns the shared exit-code types; their
existing CLI reexports remain available.

MCP `initialize` publishes those same tables under
`result._meta["mamut/cli-run"]`. The legacy `capabilities` object remains unchanged
because existing open-local tests require an exact match. CLI dispatch supplies
its configured codes automatically. Direct hosts can use `McpServer::with_exit_codes` alongside
`handle_with` to advertise the codes emitted by their execution hook.

Default help text and all existing success/failure codes and serialization are
preserved. Compatibility snapshots in `tests/fixtures` are compared byte for byte,
including the trailing LF. Only the new CLI crate's `describe` snapshot was
updated to include the additive emitted and reserved category tables; its legacy
numeric object is unchanged. Consumers requiring byte-for-byte identical
`describe` output must pin the version. Existing run and conformance tests require
no changes.
