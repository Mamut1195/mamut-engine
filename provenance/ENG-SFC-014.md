# ENG-SFC-014 · MAM-865 · CLI timeout and signals

Date: **2026-10-21**. Base: `origin/main`; worktree
`/workspace/mamut-engine-sfc14`. Complements #174 H1. No network or sources from
other engines; restricted ticket materials were not consulted.

## Implementation and decisions

- `timeout::parse_timeout_args`, in its own `engine-cli` module, removes
  exclusively `--timeout <s>` and leaves the rest to the SDK parser. The call in
  `lib.rs` is small to minimize conflicts with #197 preparsing.
  Finite seconds >0, fractional/scientific; invalid or omitted value or duplicate
  flag: 2. The flag takes precedence over `MAMUT_ENGINE_TIMEOUT`, even if invalid.
  Positive values below clock resolution expire immediately;
  finite values outside the `Instant` range are waited for in bounded segments.
- A std thread waits for a monotonic deadline and marks the existing `Arc<CancelToken>`.
  The SDK's `CancelFlag` is not `Send`; it remains on the calling thread
  and `Control::sync` synchronizes both boundaries. A channel wakes the timer
  on completion and `Drop` joins the thread. Without timeout no thread is created or output
  changed. The clock never intervenes in arithmetic, progress or manifests.
- The vendored SDK already publishes `FailureKind::Timeout` and code 6. The adapter
  maps cancellation caused by expiration to that failure: empty stdout;
  stderr ends with the
  cli-run/1 envelope with `exitCode: 6`, `outcome: "timeout"` and sanitized message.
  Partial documents are omitted; previously observed cancellation
  preserves code 5. Vendor is not modified. The historical reserved category
  metadata of `describe` is preserved to avoid altering bytes without timeout.
- Round 2, terminal precedence: `timeout::resolve` converts only
  `Cancelled` into `Timeout` when the timer expired. A dispatch already
  completed with success/warning or an error other than cancellation preserves its
  result, even if the deadline expires before the subsequent check.
  The deadline remains cooperative during I/O: its mere expiration does not invalidate
  a terminal result. A deterministic unit test injects the expired
  state and checks success, terminal errors and both kinds of cancellation.
- Round 2, NDJSON: `EventWriter::write_fmt` materializes each SDK event emitted
  with `writeln!`, including the newline, before passing it to stderr with
  `write_all`. The error envelope uses the same adapter. Atomicity applies to the
  buffer of each line; real I/O failures preserve code 70.
  The timer only marks the token and does not write to stderr.
- Loop review: Newton checks during factor validation, steps,
  iterations, element evaluations and before accepting state, with rollback
  of trial state on cancellation. Static, P-Delta and modal pass
  control to factorization/solve and eigensolver; spectrum passes it to modal and CQC.
  Recovery and combinations check between members/nodes/terms.
  Consolidation retains the check per substep and LDLᵀ blocks from SFC-010;
  it adds checks in validation, assembly, outputs, integration and substitution.
  The reference checks in dense products/solve, Cholesky, Jacobi,
  normalization, Sturm inertia and Guyan condensation/recovery. The cooperative
  variants are additive; progress callbacks retain their events.
  No numerical expression, reduction, pivot, tolerance or result of a
  completed run is modified. See `docs/DETERMINISMO.md`.
- The determinism guard retains the prohibition on concurrency in numerical
  sources and allows only the `thread` token in the exact path
  `crates/engine-cli/src/timeout.rs`. A regression checks minimal scope:
  other paths, hash collections and Rayon remain prohibited. The exception only
  allows the expressly requested timer, without parallel computation.
- The `val032_runs` guard requires current source hashes: only
  the two SHA-256 values of `pdelta.rs` and `modal.rs` in the validation index are synchronized. No
  fixtures, references, models or tolerances are regenerated.

## C: signals blocked by policy

**SIGINT/SIGTERM not implemented.** `std` does not provide an API for registering
handlers for these signals. Changes to Cargo.toml/lock are prohibited
and the crates retain `forbid(unsafe_code)`; installing handlers through own
FFI would violate the policy. Ctrl-C and SIGTERM retain OS
behavior, without a guarantee of cooperative exit 5.

**Decision pending Jonathan:** authorize a `signal-hook` dependency
in `engine-cli`, with signal registration and bridging to the token outside the
handler; alternatively `libc`, which would also require resolving the
unsafe/FFI policy. No implicit exceptions are requested or dependency added
in this ticket. The available external cancellation mechanism remains
`MAMUT_ENGINE_CANCEL_FILE` (5).

The timeout is cooperative, not a forceful terminator: blocking reads
of stdin/files and SDK I/O are not interrupted. Other backends that
do not implement the new hooks retain their indivisible kernels. The terminal
envelope applies to `run`; MCP preserves its frames and SDK semantics.

## Tests and reproduction

Binary tests in `crates/engine-cli/tests/cli.rs`:

- Consolidation derived from the `consolidation` fixture of
  `crates/engine-api/tests/support/sfc010.rs`: four layers, 80 elements,
  64 alternating intervals of 1/1,25 s, 72000 substeps and two outputs. Declared
  cost `64·80³ + 72000·80² = 493568000`, below 500000000; it also
  meets interval, substep and element-work bounds. The full run
  without timeout checks valid input and code 0; initial local
  measurement: **8,48 s** in debug before adding checks to substitution.
- Flag and variable with timeout **0,15 s**: code 6, empty stdout and timeout envelope;
  the process must finish in **<1,15 s** from spawn. A watchdog kills and reaps
  the child on regression, to avoid blocking the suite.
- Invalid values, duplicate/valueless flag, precedence over an invalid
  or tiny variable, huge finite values and generous timeout of 60 s.
  Direct stdout/stderr byte comparison and quick exit without waiting 60 s.
- The existing cancel file test before/after spawn is retained: 5.
- NDJSON + timeout by flag and variable: deadlines of 1 ns, 0,1 ms, 1 ms and 150 ms
  on heavy consolidation. All stderr lines are parsed as
  valid JSON, end with a newline and the last is the timeout 6 envelope
  without partial documents; observing `started` progress is also required.
  A unit test with a strict writer deterministically checks that
  each event and the envelope arrive as a complete JSON line, without fragments.

Domain regressions: cancellation within a Newton correction without losing
the last accepted step; interruption at every checkpoint of the reference modal
path, bit equality of modes/vectors; consolidation with control without
cancellation versus the uncached backend preserves all bits.

Verification environment, repository Rust 1.99.0 toolchain:

```sh
export PATH=$HOME/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/workspace/.cargo-target/sfc14
cargo fmt --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
bash clean-room/check.sh origin/main
```

Final results, repeated in round 2: **all four checks pass**.
`cargo fmt --check`
without differences; Clippy across the entire workspace/all-targets without warnings;
`cargo test --workspace --offline`: **758 tests pass, 0 failures, 0 ignored**
in 115 suites (including empty suites and doctests). The CLI integration suite
has 19 tests and the unit suite, 6. Round 2 adds three regressions: deterministic terminal
precedence, complete line writing and NDJSON + timeout binary.
`git diff --check` also passes.

`clean-room/check.sh` only inspects files known to Git. To include
the two new files without modifying the real index or writing to `.git`,
the same command was run with a temporary index and object store:

```sh
mam865_git_objects=$(git rev-parse --path-format=absolute --git-path objects)
export GIT_INDEX_FILE=/tmp/mam865-verification.index
export GIT_OBJECT_DIRECTORY=/tmp/mam865-verification.objects
export GIT_ALTERNATE_OBJECT_DIRECTORIES="$mam865_git_objects"
mkdir -p "$GIT_OBJECT_DIRECTORY"
git read-tree HEAD
git -c core.hooksPath=/dev/null add -N -- \
  crates/engine-cli/src/timeout.rs provenance/ENG-SFC-014.md
bash clean-room/check.sh origin/main
```

Result: **Clean-room: OK**, including both new files. The temporary
environment is limited to that command; the real index remains unchanged.

No changes to Cargo.toml/lock, vendor, CI, clean-room policies, axial bar
or truss. No commit is created.

## SIGINT/SIGTERM signals (decision #203, PR #214) — 2026-10-21

Own implementation on the public and safe API of `signal-hook =0.4.5`
(`flag::register`, `low_level::unregister`; MIT OR Apache-2.0, crates.io).
Dependency only `cfg(unix)` with `default-features = false`; new transitive dependencies in
Cargo.lock: signal-hook-registry 1.4.8, errno 0.3.14, windows-sys 0.61.2 and
windows-link 0.2.1 (the last two only on Windows through errno). No own `unsafe`
code: `#![forbid(unsafe_code)]` is retained in lib.rs and main.rs.

`crates/engine-cli/src/signals.rs` registers an `Arc<AtomicBool>` per signal; a
`std` thread checks it every 5 ms and cancels the same deadline `CancelToken`, so
that `run` ends with code 5, empty stdout and `outcome: "cancelled"` envelope.
The determinism guard accepts `thread` only in the exact CLI paths
`timeout.rs` and `signals.rs`. No third-party code beyond signal-hook's public
documentation was consulted.

Limitations documented in the README: signal before handler installation
→ default disposition (without code 5); a SIGINT ignored by the
parent is replaced by the handler.

Tests: `sigint_cancels_heavy_consolidation_with_five`,
`sigterm_cancels_heavy_consolidation_with_five` and
`cli_cancellation_exceptions_only_allow_threads_in_their_exact_sources`.
