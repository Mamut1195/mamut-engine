# Clean-room log · ENG-SFC-001

- Ticket / PR: ENG-SFC-001 (MAM-90), «WebAssembly bindings»; branch
  `j2/eng-sfc-001`, based on `origin/main`. Local commit requested; no push.
- Author(s): Codex agent under user instructions, 2026-10-15
  (America/Guayaquil).
- Role: implementer.
- Implemented contract: adaptation of engine-api (`validate`, `run`,
  `capabilities`, `schema`, `version`, `explain`), `engine-input/1` and
  `results/1`, using JSON strings; without redefining contracts or mathematics.
- Permitted sources used: ticket instructions in this session;
  `crates/engine-api/src/{lib,run,json,limits}.rs`, manifest, examples and tests;
  published fixtures `crates/engine-api/fixtures/input/{valid,invalid}`;
  `crates/engine-linalg/tests/determinism_guard.rs`, local manifests and
  determinism/clean-room scripts; local registry metadata for
  wasm-bindgen 0.2.129. No sibling repositories or sources
  from other engines were consulted. No network, fetch or downloads.
- Own derivations: none mathematical. Facade, worker protocol and
  parity harnesses written for this ticket. No transcendentals are added;
  all mathematics remains in engine-api and its dependencies, including
  engine_core::math.
- Third-party code copied: none. Local registry dependencies
  referenced through Cargo; wasm-bindgen glue generated only in ignored
  pkg. No binaries or generated artifacts are included in the commit.
- Validation: byte-for-byte comparison with native engine-api of all
  published V1 fixtures (5 valid and 9 invalid), not an external oracle.
  The real path is `engine-api/fixtures/input`; `src/fixtures/input` does not exist.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Implementation and limits

`crates/engine-wasm` contains a thin wasm-bindgen facade, compilable
natively. `run` returns the canonical engine-api document without
rewriting. Validation and callbacks use engine-api type serialization
and its canonical normalizer. Errors preserve `ApiError`
as a JSON string; errors thrown by callbacks are propagated after
cancellation. `schema` preserves the published document and `explain` returns a
JSON string literal. serde-wasm-bindgen is not used because transport is
JSON text, without mapping contracts to JS objects.

The library retains `#![forbid(unsafe_code)]`, without panic, unwrap or
expect in its source code. wasm-bindgen 0.2.129 compiles with that policy;
it was not necessary to modify or use the
`UNSAFE_LIBRARY_EXCEPTIONS` list (currently empty) of the determinism
guard. The workspace suite includes that guard and passes.

`js/worker.js` is an ES module; `js/main.js` sends requests and receives
progress/result strings through postMessage. With origin isolation, the
client shares a 4-byte JS flag, checked in callbacks, and calls the
original engine-api CancelToken. The cancel message is also posted,
but remains queued while Rust executes synchronously: **active cancellation
is not attributed to that message**. Engine-api steps are
indivisible, including factorization; cancellation between steps/modes/
iterations, without a maximum latency promise. If the signal arrives after
completion, the full result may be retained. Without SharedArrayBuffer,
the worker is terminated and rejected with AbortError; no cancelled
results/1 is invented. The client may create another worker on the next
request. HTML example, loopback server with COOP/COEP and browser harness
included; no npm dependencies.

## Cargo changes

- New member `crates/engine-wasm` in `members` and `default-members` because
  it compiles natively. Types `cdylib` and `rlib`; the release profile is not modified.
- Direct dependencies: local engine-api; serde `=1.0.229` and serde_json
  `=1.0.154` already present; wasm-bindgen **`=0.2.129`** and js-sys **`=0.3.106`**.
- Cargo.lock adds engine-wasm and 11 external packages resolved offline:
  wasm-bindgen and its three macro/shared crates, js-sys, bumpalo,
  futures-core/task/util, pin-project-lite and slab. Existing versions are not
  updated. CLI 0.2.129 checked by the script before compilation.

## Measured sizes

`scripts/build-wasm.sh` generates `pkg/size-report.md`. Release measurements with
rustc **1.99.0 (b940084d7 2026-09-28)**, wasm-bindgen CLI **0.2.129**, target
`wasm32-unknown-unknown`; gzip **`-9 -n`** (without name or timestamp). No
wasm-opt, additional LTO or workspace profile changes.

| Artifact | Raw bytes | gzip -9 bytes | Against proposed ≤ 1.000.000 gzip bytes |
| --- | ---: | ---: | --- |
| Cargo release, before wasm-bindgen | 1.908.323 | 496.172 | PASS |
| wasm-bindgen `--target web`, engine_wasm_bg.wasm | 1.455.396 | 383.805 | PASS |
| wasm-bindgen `--target nodejs`, engine_wasm_bg.wasm | 1.455.396 | 383.805 | PASS |

**Budget of ≤ 1 MB gzip: PROPOSED, PENDING confirmation.**
The stricter decimal interpretation (1.000.000 bytes) is used; these
measurements also remain below 1 MiB. These are .wasm sizes;
the web JS glue adds 13.533 raw bytes, and the client/worker are additional.
They are not presented as total application size. Web wasm SHA-256:
`b255bdbc4b6328a89fc5fd937d1c316ad489c360bf525ac1f913507ce15f6d36`.

## Local verification

All Cargo commands use
`export CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine` and
`PATH=/home/box/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin:$PATH`.
Compilation, tests and docs with `--offline --locked`; there was no network.
Node **v22.19.0**, binary `/home/box/.local/bin/node`; CLI
`/workspace/tools-bin/wbg/bin/wasm-bindgen`.

| Command / evidence | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --offline --locked --workspace --no-fail-fast` | PASS: 545 tests/doctests; zero failures |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace` | PASS |
| `cargo check --offline --locked --workspace --target wasm32-unknown-unknown` | PASS |
| `bash clean-room/check.sh origin/main` | PASS on staged changes |
| `bash scripts/build-wasm.sh` | PASS: release build, web+nodejs CLI, native oracle, Node suite and report |
| Node, nodejs target | PASS: 14 fixtures, byte-for-byte identical results/1, progress, validation, metadata, errors and cancellation |
| Node, web ES module target | PASS: the same 14 fixtures and direct tests |
| worker_threads, test-only adapter | PASS: real worker.js and main.js, 14 fixtures, all operations, progress, active cancellation with shared flag, terminate fallback and reuse |
| `git diff --check` and JS/Bash/Python syntax | PASS |
| Chrome headless | **PENDING**: installed, but the sandbox prevents sockets and startup |
| Firefox | **PENDING**: no browser engine was run |
| Safari | **PENDING**: no browser engine was run |

An attempt was made to start the offline server with `python3 scripts/serve-wasm.py`:
`PermissionError: [Errno 1] Operation not permitted` when creating the socket.
Chrome with profile in `/tmp/eng-sfc-chrome`, `--headless --no-sandbox
--disable-gpu --disable-dev-shm-usage --dump-dom about:blank` terminated with
exit 133 and `setsockopt: Operation not permitted`. No permissions were requested
and no attempt was made to execute outside the sandbox. The harness
`crates/engine-wasm/js/test-browser.html` remains ready to run in
Chrome/Firefox/Safari with an authorized local server; it publishes PASS/FAIL.
Execution of the Node adapter **does not certify browsers**. Multibrowser
acceptance remains pending, along with budget confirmation.

No changes to `docs/CLEAN_ROOM.md`, `.github/**`, `clean-room/**` or
`vendor/**`. No subagents, push or committed .wasm/pkg artifacts.
