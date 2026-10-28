# Clean-room log · ENG-LA-008-FU

- Ticket / PR: issue #77 · MAM-451; branch `j2/fix-77-heap-peak`, from
  `origin/main` (`d1249f1`).
- Author(s): Codex gpt-6.1-sol high under J2's direction, 2026-10-14
  America/Guayaquil.
- Role: implementer.
- Implemented contract: heap bound regression for `engine-linalg` for
  `MemoryBudget::factorize` and `MemoryBudget::generalized_eigen`.
- Permitted sources used: only the repository's own code,
  `crates/engine-linalg/tests/heap_peak.rs`, `provenance/ENG-LA-008-WIRE.md` and
  `provenance/TEMPLATE.md`. No network or consultation of external sources.
- Own derivations: isolation and accounting described below.
- Third-party code copied: none.
- Validation: own measurement of Layout bytes, regression for isolation between
  threads and concurrent stress; checks detailed below.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Isolation and meaning of the measurement

The previous GlobalAlloc counted all libtest threads. The Mutex only
serialized measurements; it did not prevent allocations or deallocations by the harness on
other threads, which could alter the peak or make LIVE fall below the baseline.

The standard harness is retained: the thread-local flag
`Cell<bool>` with a `const` initializer enables only the measuring thread.
Thus `cargo test --workspace` still discovers and runs the test under its
name and propagates its assertion failures through libtest. There is no need for
`harness = false` or an alternative main.

The flag requires no allocations or destructor. The allocator queries TLS with
`try_with`, ignores access if TLS is unavailable and temporarily disables
the flag while updating counters to protect against reentrancy.
These updates use only Cell and atomics, without allocating memory. An RAII
guard disables the flag even during panic unwinding. The Mutex retains
exclusion between tests that use the same global counters.

The counter is enabled throughout the test, including construction and
deallocation of inputs and results; each operation retains its live-byte
baseline and resets its peak. Existing inputs are excluded from the
measurement by taking the difference from that baseline. All counter subtractions use
`saturating_sub`, including deallocations and realloc size changes.
The atomic subtraction of LIVE uses compare_exchange to retain compatibility with
MSRV 1.85 without using APIs deprecated by toolchain 1.99.0.

No bound, dimension, budget formula or assertion of the
original regression is changed: orders 0, 1, 3, 8, 9, 64, 65 and 256 remain, as do the
peak and retained memory checks, and the requirement that the old bound
underestimate at least one peak. The additional regression creates another thread before
enabling the counter, exercises allocations, growing/shrinking realloc and
deallocations, and requires LIVE and PEAK to remain exactly unchanged. It resets
PEAK to LIVE so that a peak from a previous test does not hide interference.
Another case deallocates a Vec constructed before measuring: it checks that, even if LIVE
legitimately falls below the baseline, the measured peak and retained memory saturate at zero.

## Offline verification

Installed toolchain: Rust/Cargo 1.99.0;
`/home/box/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin` is prepended to PATH because
`/usr/bin` contains 1.85.1. All Cargo invocations use
`CARGO_TARGET_DIR=/workspace/.cargo-target` and `CARGO_NET_OFFLINE=true`; those that
accept that flag also use `--offline`. No component is installed.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline -p engine-linalg --test heap_peak -- --nocapture`: OK,
  all three tests pass.
- `cargo test --offline --workspace`: OK, including doctests and the three counter
  tests; 19 complete runs in parallel with the final stress test.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK.

## Concurrent stress

With the same offline environment and CARGO_TARGET_DIR, a loop of
`cargo test --offline --workspace` is launched in the background in another process. In the foreground:

```bash
for attempt in {1..50}; do
    cargo test --offline -p engine-linalg --test heap_peak -- \
        --exact charged_bounds_cover_actual_factorize_and_eigen_heap_peaks \
        --nocapture || exit 1
done
```

The workspace loop remains active during all 50 runs, not just
the first. At the end, it is signaled to stop and allowed to complete
its current run, also checking its exit code. Each invocation
has a separate log; it is checked that the filtered test ran and that there
were no failures. Local script: `/tmp/me45-issue77-stress/run.sh`; logs of the final
version: `/tmp/me45-issue77-final/`.

Final stress result: **50/50 OK**, with **19/19 complete workspace
runs OK**. The three counter tests also ran in every
workspace invocation. The six reported peaks are identical across all 50
runs; the assertions remain for all orders, peaks,
retained memory and the insufficiency of the old bound.

Reported peaks (Layout bytes), without changing bounds:

| n | factorize peak | factorize charge | Old bound | eigen peak | eigen charge |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 1856 | 2088 | 1344 | 4224 | 5800 |
| 64 | 100864 | 101096 | 68096 | 263168 | 331496 |
| 256 | 1583104 | 1583336 | 1058816 | 4198400 | 5257448 |
