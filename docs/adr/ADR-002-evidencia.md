# ADR-002: verification evidence (V1, V2, V4, V5, partial V6)

- **Related to:** [ADR-002](ADR-002-algebra-lineal.md) (provisional: faer primary, nalgebra optional, sprs-ldl excluded)
- **Measurement date:** 2026-10-10/11 (America/Guayaquil)
- **Status:** evidence measured on the development machine; does **not** replace Jonathan's approval or CI.
- **Reproducible code:** [`spikes/adr-002-la/`](../../spikes/adr-002-la/) (separate package with its own `[workspace]`; it is not a member of the engine workspace and adds no dependencies to it).
- **`cargo-deny` proposal:** [`deny.toml`](../../deny.toml) at the root.

The validations check self-consistency only (relative residual and comparison between implementations); OpenSees, CalculiX or any other engine was not used as an oracle.

## Environment

| Item | Value |
|---|---|
| OS / CPU | Linux 6.12.94, Intel Xeon (virtualized), 8 threads available |
| Rust | 1.85.1 (workspace MSRV) and stable 1.99.0 (cargo 1.99.0), via rustup 1.29.1 in the machine user's `~/.rustup` |
| wasm target | `wasm32-unknown-unknown` (`rustup target add` on both toolchains) |
| Node | v22.23.2 (wasm runner: `WebAssembly.instantiate`, no imports; no `wasm-bindgen-test`/wasmtime installed) |
| cargo-deny | 0.20.2 |
| Candidates | faer 0.24.4 (`default-features = false`, `features = ["std", "sparse-linalg"]`), nalgebra 0.35.0 (`default-features = false`, `features = ["std"]`) |
| Release profile | `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`; no `-C target-cpu`, no `target-feature=+fma`, no `wasm-opt` |

## 1. wasm compilation

| Check | Command | Result |
|---|---|---|
| Current engine crates (`engine-core`, `engine-model`, `engine-contracts`) | `cargo build --workspace --target wasm32-unknown-unknown` (1.85.1) | **OK** |
| Spike with faer | `cargo +stable build --release --lib --target wasm32-unknown-unknown --no-default-features --features with-faer` | **OK** |
| Spike with nalgebra | same with `--features with-na` | **OK** |
| faer on MSRV 1.85.1 (native) | `cargo +1.85.1 build --release --lib --no-default-features --features with-faer` | **OK** |
| nalgebra on 1.85.1 | `cargo +1.85.1 build` with nalgebra | **Fails** (consistent with the ADR's MSRV of 1.89) |
| `.wasm` imports | `WebAssembly.Module.imports` | **0** imports in both; instantiates in Node without glue |
| Dependencies `rayon`, `*-sys`, `getrandom`, BLAS/LAPACK/ARPACK, `sprs-ldl` in faer's wasm tree | `cargo tree --target wasm32-unknown-unknown -e normal --no-default-features --features with-faer` | **None** (111 tree lines) |

With faer's optional `rayon` feature the tree does include `rayon`/`rayon-core`; the proposed profile does not enable it.

## 2. wasm execution in Node

Problem: factorize A = B·Bᵀ/n + n·I (SPD, B generated with a predetermined LCG, RHS `1 + (i mod 7)`), n = 50, 200, 1000. Relative residual ‖Ax−b‖/‖b‖ (2-norm).

| n | faer residual (Cholesky) | nalgebra residual (Cholesky) |
|---|---|---|
| 50 | 4.31e-16 | 3.70e-16 |
| 200 | 6.20e-16 | 7.10e-16 |
| 1000 | 1.46e-15 | 1.58e-15 |

All < 1e-12 (V2 threshold). Runs in Node: **meets the requirement** for Cholesky. LU also runs in Node, but there only its hash was compared with the native one (§4); its residuals (≤ 1.80e-15) are those from the native benchmark (§3, `bench`), not measured in wasm.

`.wasm` size (release, without `wasm-opt`, includes the spike's data generator, 24 068 bytes in the baseline without a backend):

| Artifact | Bytes | gzip -9 |
|---|---|---|
| baseline (no backend) | 24 068 | not measured |
| faer 0.24.4 (Cholesky+LU+residual) | 217 750 | 63 824 |
| nalgebra 0.35.0 (Cholesky+LU+residual) | 34 638 | 13 247 |

The project's proposed target of ≤ 1 MB gzip is comfortably met in this minimal configuration; the budget remains **PENDING** approval and does not include sparse assembly, LDLᵀ or eigenvalues (**PENDING**: size with `sparse-linalg` actually used and with `wasm-opt`).

Node timings (median of 3–20 repetitions after warm-up; time includes generating the matrix, which is measured separately and subtracted, so **the values are indicative only** and at n ≤ 50 fall below the resolution): n = 200: Cholesky faer ≈ 0.9 ms, nalgebra ≈ 0.4 ms; n = 1000: Cholesky faer ≈ 105 ms, nalgebra ≈ 53 ms; LU n = 1000: faer ≈ 232 ms, nalgebra ≈ 313 ms (noise ±50 ms from subtraction). Without wasm SIMD (`simd128` was not enabled), faer loses its native advantage.

## 3. Native performance (partial V4)

Command: `cargo +stable build --release && ./target/release/bench` (single thread; without `rayon`). Median of 500/50/5 repetitions (n = 50/200/1000) after one warm-up repetition; includes copying to each library's structure and solving with one RHS.

| n | operation | faer (ms) | nalgebra (ms) | nalgebra/faer ratio |
|---|---|---|---|---|
| 50 | Cholesky | 0.0109 | 0.0088 | 0.81 |
| 50 | LU | 0.0133 | 0.0117 | 0.88 |
| 200 | Cholesky | 0.1850 | 0.3446 | 1.86 |
| 200 | LU | 0.2276 | 0.6303 | 2.77 |
| 1000 | Cholesky | 10.36 | 47.75 | 4.61 |
| 1000 | LU | 14.54 | 106.63 | 7.33 |

Variability between repetitions is not published in detail (**PENDING**: deviation, peak memory, sparse matrices/assembly, separate symbolic/numerical timings). A single virtualized machine: indicative, not a budget.

## 4. Determinism (partial V6)

The FNV-1a hash of the solution bits (f64 little-endian) is compared for the 4 combinations (faer/nalgebra × Cholesky/LU) at n = 50, 200, 1000.

| Test | Result |
|---|---|
| 5 native runs, without `rayon`, same binary | **Bitwise identical** (12/12 hashes) |
| Pinning to 1 CPU (`taskset -c 0`) vs. 8 CPUs, without `rayon` | **Identical** (faer without `rayon` is sequential) |
| Build with faer's `rayon` feature and `Par::Seq` | **Identical** to the build without `rayon` |
| `Par::rayon(k)`, k = 1, 2, 4, 8, n = 50 and 200 | Identical to each other |
| `Par::rayon(k)`, n = 1000 (faer, Cholesky and LU) | **Differ with k and from `Par::Seq`**; different hash for each k ∈ {1, 2, 4, 8} |
| Same `k` repeated 10 times (k = 4 and k = 8) | Repeatable: 10/10 equal |
| nalgebra (n = 50, 200, 1000), any configuration | Always identical (does not use threads) |
| Native vs. wasm (Node) | nalgebra **bitwise identical** in all 6 cases. faer identical at n = 50 (both) and LU n = 200; **differs** for Cholesky n = 200 and both at n = 1000 (native SIMD/reduction order vs. wasm without SIMD) |

Conclusions for the ADR: (a) `rayon` must remain disabled in V1 (confirms decision 1); with it, the result depends on thread count for n = 1000; (b) native/wasm bitwise equality **cannot** be promised with faer, although the residuals match in order of magnitude; (c) nalgebra is the only one that, here, gives equality across targets, at the cost of performance. The ENG-CORE-010 criterion should be a justified tolerance for faer, not bits. **PENDING:** `simd128` in wasm, FMA, eigenvectors, LDLᵀ/AMD, sparse matrices.

## 5. Licenses, sources and notices (partial V1, `cargo-deny`)

Commands (from `spikes/adr-002-la/`, with `../../deny.toml` copied or `--config`):

```
cargo-deny check
cargo-deny --all-features check
```

Result: `advisories ok, bans ok, licenses ok, sources ok`, with both candidates and with the `rayon` feature. Licenses in the graph of both (per crate, SPDX expressions): MIT (55), Apache-2.0 (29), Unicode-3.0 (1, `unicode-ident`), Zlib (3: `bytemuck`, `safe_arch`, `wide`); no LGPL/GPL.

Observations:

- The spike's `Cargo.toml` declares `license = "MIT"` only so that `cargo-deny` evaluates the graph; it is not a project licensing decision.
- **RUSTSEC-2024-0436 (`paste` unmaintained):** `paste 1.0.15` comes through `gemm` ← faer; it is a compile-time procedural macro, with no recorded vulnerability. In `deny.toml` it is ignored with an explicit reason; review on each faer update.
- With the proposed `deny.toml` in the engine workspace, `cargo deny check licenses` flags `engine-core`, `engine-model` and `engine-contracts` as **unlicensed** (their manifests do not declare `license`) and warns about path wildcards (resolved with `allow-wildcard-paths = true`). Declaring `license`/`publish = false` is a governance decision (ENG-GOV-004; there is no legal text in the repository) and is not touched here; until then the CI step must run on the spike or await that decision. Today the workspace has no external dependencies.
- The identifiers allowed in `deny.toml` that do not appear in the current graph (BSD-2/3, ISC, CC0, 0BSD, Apache-2.0 WITH LLVM-exception, Unicode-DFS-2016) generate only `license-not-encountered` warnings; they remain as the policy allow-list.
- The RustSec advisory database was downloaded from the network during the run; in CI access must be allowed or `db-path` pinned.

### Proposed CI step (not applied)

`.github/workflows` is a protected file and Actions billing is blocked; this PR does not modify it. Proposal for when it is reactivated:

```yaml
- uses: EmbarkStudios/cargo-deny-action@v2   # fijar a un SHA verificado
  with:
    command: check
    arguments: --all-features
```

plus a `cargo build --workspace --target wasm32-unknown-unknown` matrix with Rust 1.85.1 and the Node test from `run.mjs`.

## 6. Status of ADR-002 outstanding items

| Outstanding item | Status after this evidence |
|---|---|
| V1 graph licenses and `cargo-deny` | **Closed for faer and nalgebra** (spike); effective MSRV: faer compiles with 1.85.1, nalgebra does not; scirs2-sparse/sprs/nalgebra-sparse **PENDING** |
| V2 native/wasm execution in Node | **Closed for dense Cholesky/LU**; triplet assembly, sparse SPD/indefinite, singularity, multiple RHS **PENDING**; CI **PENDING** |
| V3 APIs, inertia, eigenvalues | **PENDING** (not addressed) |
| V4 performance | **Partial** (dense, one thread, native + indicative wasm); sparse/memory **PENDING** |
| V5 wasm size | **Partial** (minimal dense); with `wasm-opt`, sparse and LDLᵀ **PENDING** |
| V6 determinism | **Partial** (see §4); eigenvectors and `simd128`/FMA **PENDING** |
| V7 maintenance | **PENDING** |
| Jonathan's approval | **PENDING** |

## How to reproduce

```
rustup toolchain install 1.85.1 stable && rustup target add wasm32-unknown-unknown --toolchain 1.85.1
rustup target add wasm32-unknown-unknown --toolchain stable
cargo install cargo-deny --locked
cd spikes/adr-002-la
cargo +stable build --release && ./target/release/bench && ./target/release/det
cargo +stable build --release --features rayon --bin det --target-dir target-rayon && ./target-rayon/release/det 4
for f in with-faer with-na; do cargo +stable build --release --lib --target wasm32-unknown-unknown --no-default-features --features $f --target-dir target-w-$f; done
node run.mjs target-w-with-faer/wasm32-unknown-unknown/release/spike_la.wasm faer
node run.mjs target-w-with-na/wasm32-unknown-unknown/release/spike_la.wasm na
cargo-deny check
```
