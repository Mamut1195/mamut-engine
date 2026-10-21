# Determinism policy · V1 · ENG-CORE-010 (MAM-23)

The same input, engine version, backend, options and build environment
must produce the same canonical `results/1` bytes, including its external
SHA-256. This includes `git_commit` and `target`: changing the target changes
provenance and may change the hash of the complete document. The
`input_sha256` field identifies the canonical input: it is not the result hash. There are no clocks, paths or random state in `run`.

MAM-79 / #143: input parsing rejects integer literals with
`|n| > 2^53-1` before converting them to binary64 or computing JCS; Python applies
the same symmetric range. Thus `2^63-1` and `2^63` cannot be published with an
indistinguishable fingerprint or hide changes in stale detection. Tokens with a decimal point or
exponent retain binary64 semantics; the fingerprint identifies their canonical
value, rather than their lexical spelling. The SDK and computations are unchanged, and the
inputs of the 16 published cases satisfy the range: their hashes are preserved.

## Initial audit of main and #88

Local offline audit, 2026-10-14: `origin/main` = `eed45dd`;
`origin/j2/eng-api-001` / base of this branch = `b270626`, PR #88.
The remote PR status was not queried. The base incorporates main.

| Aspect | main | Base #88 |
| --- | --- | --- |
| Numerical collections | `BTreeMap/BTreeSet`, `Vec`; no hash collections | Same; engine-api also uses BTreeMap and explicitly sorts JSON keys |
| Threads | No `std::thread` or rayon in libraries | Same; cooperative cancellation, indivisible solves |
| ENG-LA-006 backend | Working `ReferenceBackend`; empty faer feature, Unsupported adapter | Same; `run` selects ReferenceBackend |
| Reductions | Sequential loops/iterators; CSR sums duplicates in input order | Same |
| SIMD/FMA | No explicit SIMD or per-CPU numerical dispatch; one explicit `mul_add` in Sturm | Same |
| Seeds | No RNG or seeds in libraries | Same |
| Scalars | std `sqrt`, `hypot`, `sin_cos`, `ln`, `ln_1p`, `exp` | Same; no direct sin/cos/powf calls in libraries |
| Canonical facade | engine-api is not yet in main | `run`, results/1, input_sha256 and two-run tests |

Threads in `engine-linalg/tests/heap_peak.rs` are test instrumentation,
outside `src`; they are not backend numerical execution.

## V1 configuration and operation order

- One execution thread per analysis. Internal threads, pools, rayon,
  atomics used to accumulate results and parallel reductions are prohibited. `Arc` and the
  cancellation token do not alter numerical order; repeatability uses
  an uncancelled token and callbacks with no effects on the model.
- `ReferenceBackend` without optional features is the V1 configuration. `faer`
  remains a stub, rather than a certified provider. Any new backend
  must declare reduction order, tie-breaking, SIMD/FMA and pass this suite.
- `HashMap/HashSet` are prohibited in numerical paths: indices and traversals
  are ordered with `BTreeMap/BTreeSet` or vectors. Input arrays are not reordered:
  permuting loads or members may change rounding and is not the “same input”.
- There is no randomness today. If a procedure using an RNG is added, its procedure,
  version and seed must be explicitly pinned (V1 seed `0_u64`), without
  system entropy, time or implicit seeds; this requires extending the tests.
- IEEE-754 binary64, ordinary round-to-nearest, ties-to-even; no
  fast-math, reassociation, flush-to-zero or CPU-dependent options such as
  `target-cpu=native`. The contract is not promised under flags that change this.
- No explicit SIMD or target-based kernel selection. Native uses its
  scalar f64 instructions (e.g. SSE2); wasm uses WebAssembly f64, without
  enabling simd128. Autovectorization that preserves semantics may operate
  on independent components; it cannot regroup an f64 reduction.
  Verified builds use ordinary Cargo flags, without special RUSTFLAGS,
  in both debug and release.
- No implicit FMA: separately written multiplication and addition retain
  their rounding steps. **Existing explicit exception:** `modes.rs`, formation of
  `K−σM` for Sturm with `(-sigma).mul_add(mi, ki)`. This is a single rounding by
  contract, in hardware or software, including wasm; it is not replaced with `a*b+c`.

Backend order, determined by the project's own code:

| Operation | Order and tie-breaking |
| --- | --- |
| Dense product | `i` row, `j` column, increasing `k`; left-to-right sum from +0 |
| CSR | Triplets in input order; duplicates summed sequentially; output in increasing `(fila,columna)` order through BTreeMap |
| General solve | Increasing pivot `k`; absolute maximum in `k..n`, ties take the last index (`max_by`); increasing `i,j` elimination; substitution in decreasing row order, increasing `k` terms |
| Equilibration | Row sweeps and maxima in increasing index order; up to 40 deterministic steps |
| LDLᵀ | Increasing active block; maximum diagonal, ties take the last; off-diagonal by increasing `i` and `j<i`, first strict maximum; lower-triangle update in increasing `i,j` |
| LDLᵀ solve | Increasing RHS; forward substitution with increasing rows and terms; D blocks in factorization order; back substitution with decreasing rows and increasing terms |
| Cholesky | Increasing `i`, increasing `j≤i`, subtraction of products with increasing `k<j` |
| Jacobi | Search with increasing `i`, increasing `j<i`, first strict maximum; rotations in that order; eigenvalues with stable ascending sort (`total_cmp`), ties preserve index |
| Modal normalization | Increasing columns; norm with increasing `i,j`; dominant component with max_by (last on a tie), positive sign |

Assembly traverses contributions in received order, with DOFs ordered by
NodeId and component. Combinations traverse terms in the given order;
SRSS/CQC traverse modes and pair indices in increasing order. No sum is distributed
among workers. The policy does not change tolerances, pivots or the solver.

## Scalar functions: libm approved by Jonathan 2026-10-14

The V1 policy is **libm 0.2.16, MIT license**, approved by Jonathan on
2026-10-14. The previous decision to retain std because libm was absent from the
offline registry is superseded. The local registry now contains this version;
it was incorporated without network access or code from other engines.

`Cargo.toml` pins `libm = { version = "=0.2.16", default-features = false }` in
workspace.dependencies; engine-core inherits it. `Cargo.lock` adds only
libm 0.2.16 (checksum `b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981`)
and its edge from engine-core. Disabling the default `arch` feature
selects generic Rust implementations, without architecture-specific kernels.
No unstable features or optional libm intrinsics are enabled.

`engine_core::math` is the sole entry point for scalar functions. It centralizes
sin/cos/sin_cos, tan and inverse trigonometric functions, hyperbolic functions and their inverses, exp,
exp2, exp_m1, ln, ln_1p, log/log2/log10, powf/powi, atan2, hypot and cbrt. Uses
in libraries, examples and tests of engine crates go through that module.
`sin_cos` uses libm::sincos and preserves the (sine, cosine) pair. `powi(x,n)` uses
libm::pow(x,f64::from(n)): every i32 converts exactly, including MIN/MAX;
it may round differently from std::powi multiplications. `log(x,base)`
specifies two natural logarithms and a division. Domains, finiteness
validation, tolerances, reduction order and the explicit Sturm FMA are unchanged.

**Documented exception:** math::sqrt retains std `f64::sqrt`. IEEE-754
requires correctly rounded square roots (nearest, ties-to-even) for
finite nonnegative operands. This does not mean an irrational root is
exactly representable. Subnormals and signed zeros are included in the
bit comparison across targets. libm does not guarantee correct rounding of all its transcendentals;
its implementation is pinned for repeatability. NaN payload identity is not
promised; analysis paths reject nonfinite results.

hypot is not replaced with sqrt(x*x+y*y): math::hypot retains libm::hypot and its
protection against overflow/underflow.

| Migrated path | Functions |
| --- | --- |
| model geometry, frame/frame3d/truss/spring, results, masses/directions | hypot |
| ReferenceBackend: LDLᵀ, Jacobi, Cholesky, equilibration and normalization | hypot and sqrt (IEEE exception) |
| Modal and SRSS/CQC | sqrt, hypot, powi |
| Log-log spectrum interpolation and distance between periods | ln, ln_1p, exp |
| Load rotation/integration and contract conformance | sin_cos, sqrt |
| Analytical references and examples/tests | Also sin/cos/tan, cosh/tanh, powf/powi; same scalar policy |

## Tests and reproducible offline comparison across targets

`engine-linalg/tests/determinism_guard.rs` maintains the collection and thread
guard over `crates/*/src/**/*.rs`: it rejects HashMap, HashSet, thread, rayon and
rayon_*. It adds a scalar guard over **all Rust files in crates/**,
including tests and examples and future crates. Only
engine-core/src/math.rs is exempted for transcendentals and libm. It rejects methods such as
`x.sin()` and `x.r#sin()`, UFCS/references such as `f64::sin` or `Real::sin`,
and direct libm access/imports outside the module. It also requires centralizing
sqrt. It permits `engine_core::math::función` and the shorthand `math::función`
only with unaliased `use engine_core::math;`: it rejects `as math`, primitive
aliases, individual function imports and redefinitions of the reserved name
math. It checks the complete path, rejecting `other::math::función`.
Inside macros it rejects scalar identifiers without that path, for example
`call!(x, exp)`; it preserves data receivers such as `log.acceleration(...)`.
`mul_add` is permitted only in engine-linalg/src/modes.rs, including checks for
raw identifiers and macro arguments.

`include!` and `#[path = "..."]` accept only literal paths ending in .rs,
whose target, canonicalized relative to the source directory, must belong to
the inspected Rust inventory in crates/**. Other extensions,
computed paths (concat!/env!, etc.) and missing targets or targets outside the
inventory are rejected. It also checks `path` inside `cfg_attr`, including nested forms,
with whitespace and raw strings, without exempting math.rs or examples/.
`include_str!` and `include_bytes!` may include data with other extensions.

FU2 #107 (J4 r27) added the prohibition of `#[link_name = ...]` and
`extern` blocks, with a blanket exception for examples/. FU3 (J4 r29/r30) removes
that blanket exception: in **all Rust files in crates/** it rejects `link_name`,
`export_name`, `extern "..." { ... }` / `extern { ... }` blocks and the
`asm!`, `global_asm!`, `naked_asm!` macros, including `core::arch::` and
`std::arch::` paths, whitespace and raw identifiers. It visits nested `cfg_attr`
attributes and `#[unsafe(...)]` wrappers regardless of the condition.
It also rejects attributes with metavariables (`#[$attr]`, `#[$($a)*]`, etc.)
and `extern` with variable ABI/tokens (`extern $abi {}`, `extern $($t)*`,
`extern "C" $body`), including inside `macro_rules!`. It does not need to expand
macros to close those uninspected-code paths.

Until this change, the unsafe exception for tests/examples was conventional:
FU2 required `forbid(unsafe_code)` in libraries, without encoding permissions for
those directories, and exempted examples/ wholesale from its FFI check.
The guard now prohibits unsafe, assembly, export_name, no_mangle, link_name
and extern blocks or functions/types in crates/*/tests/** and crates/*/examples/**,
except for the explicit `SOURCE_POLICY_EXCEPTIONS` list, by exact path and
construct:

- `engine-linalg/tests/heap_peak.rs`: unsafe only to instrument GlobalAlloc.
- `engine-api/examples/determinism_wasm.rs`: only no_mangle and public
  extern "C" functions for its wasm ABI; no unsafe permission.
- `engine-validation/examples/winkler_wasm.rs`: the same two permissions for
  its wasm ABI; no unsafe permission.

The guard fails if a listed file disappears or stops using
**any** authorized construct, even if it retains another.
No exception permits assembly, export_name, link_name, extern blocks,
variable attributes/extern or relaxes include!/path or scalar rules.
`extern crate` retains the previous prohibitions. In src/, functions and
types with extern ABI remain distinct from import blocks.

Each crates/*/src/lib.rs library declares `#![forbid(unsafe_code)]` at the
root, unconditionally, and the guard checks its presence. The explicit
`UNSAFE_LIBRARY_EXCEPTIONS` list is empty: all fifteen libraries
compile without unsafe. The Python binding also compiles with the prohibition
despite its PyO3 macros; it is added there and checked separately. vendor/**
is not inspected. The `unsafe impl GlobalAlloc` and its System calls
in engine-linalg/tests/heap_peak.rs instrument an integration test;
they are not library code and do not require exempting engine-linalg.

The guard ignores comments, ordinary/raw strings and char literals. Five
pairs of positive/negative text fixtures cover FU #105; FU2 adds
five specific negative tests (non-.rs path, target outside the inventory,
link_name, extern block and missing forbid), checked by removing each
rule separately, plus positives for comments, strings,
nested attributes and inspected Rust targets. FU3 adds negatives per
construct, directly and inside macros with three delimiters, in src/,
tests/ and examples/; it also checks minimal permissions, exact paths and
obsolete exceptions by permission or file. They are checked by removing their
rules individually; positives remain without false positives for
comments, ordinary/raw strings and unrelated attributes. It retains tests
for aliases, whitespace, macros and every listed scalar function.
It is a conservative lexical guard, rather than type resolution; it does not inspect
transitive dependencies or code generated by build scripts. Changing providers
requires review.

`engine-api/tests/determinism.rs` executes eleven valid inputs ten times:
Status::Ok, idempotent canonicalization, SHA-256 of the complete document and also
direct byte equality. It retains cantilever, inclined cantilever, one-DOF
modal, P-Delta, two-coupled-DOF modal and log-log/CQC spectrum; it adds
P-Delta, modal and spectrum with inclined geometry (coordinates 0,8x/0,6x),
including rotated P-Delta loads. Both P-Delta fixtures use
`options: {}` since MAM-582: pDelta is `Stable` according to
[ENG-BENCH-002 (PR #132)](../provenance/ENG-BENCH-002.md),
AISC 360-16 Comm. C2 cases 1 and 2, error ≤0.27% with tolerances 3% M / 5% Δ.
This justification comes from evidence supplied for the ticket;
the determinism suite checks repeatability rather than repeating those benchmarks.
`allow_experimental` remains in the schema for future Experimental methods.
Two frame3d inputs from the
same spatial frame are added: `frame3d-inclined-static` (linearStatic) and
`frame3d-inclined-modal` (six modes, nodal masses in all six DOFs of each free
node and X/Y/Z participation). The API requires one analysis per input.
The frame has three nodes (0,0,0), (2.4,1.8,1.3), (4.1,0.7,3.2), two members
inclined along all three axes and local z references (0.3,−0.4,1), (−0.6,1,0.2).
Iy=3 and Iz=5 avoid bending symmetry. The native test checks recovery
of the twelve local forces per member, six positive modes and that changing
orientations changes the mechanical results of both analyses.
They are variants of the project's own fixtures, without data from other engines.
Winkler was integrated in #99 and is present after the merge; these sixteen cases
do not cover that path. It has its own suite in engine-validation. The harness uses
the central scalar module and the guard passes in the 2026-10-15 rerun.

The native and cdylib wasm examples share those eleven inputs and five
additional cases: scalar-trig, scalar-exp-log, scalar-powers-norm, load-rotation-bits
and scalar-sin-cos.
The latter compute SHA-256 directly from `f64::to_bits().to_be_bytes()`:
there is no JSON, tolerance or normalization hiding signed zeros or rounding.
They exercise all module functions, subnormals, small/large values,
trigonometric arguments up to 1e300, atan2 with signed zeros, extreme integer
exponents, and sin_cos through Beam2D::forces_to_global. The independent
scalar-sin-cos case directly uses engine_core::math::sin_cos and
hashes the (sine, cosine) pair for sixteen angles: signed zeros,
subnormals, binary64 neighbors of π/2 and arguments with magnitude up to 1e300.
The libm::sincos wrapper and its use when computing sine/cosine of the same
angle in the engine were already present in fccda4d; they are retained.
They include only finite results and check that the lists are nonempty.

Since ENG-CORE-005, hashes for comparison across targets are computed over `results/1` after
checking that `target` matches the Cargo target and removing only
that field. It is serialized again with `engine_api::canonical_json`, preserving
the output contract; JCS is not applied to results. `git_commit`,
`input_sha256` and all numerical fields remain in the comparison.
The ten-run native test still compares the bytes and hash of the
**complete document**, including `target`. The projection test rejects missing/incorrect
target and checks that all other fields are preserved.
`mamut_canon::sha256_hex` hashes received bytes without canonicalization;
the helpers decode its hex to the same 32 bytes as the previous ABI. Scalar cases
concatenate exactly the same big-endian bytes, without JSON.

C ABI: `fixture_count()`, `fixture_name_hash(index,word)`, `run_status(index)` and
`run_hash(index,word)`. Four big-endian u64 values form each SHA-256; each word
recomputes the case, without global mutable state. Node receives u64 as BigInt
and normalizes its sign. The script checks names/order, required coverage,
out-of-range ABI, zero imports, ten stable hashes and equality across targets;
it fails on the first difference. No wasm-bindgen, WASI, npm or network.

Local reproduction with Node (retain the already exported CARGO_TARGET_DIR):

```bash
export PATH=/home/box/.cargo/bin:$PATH
export CARGO_NET_OFFLINE=true
unset MAMUT_ENGINE_GIT_COMMIT # tabla reproducida con git_commit: null
cargo test --offline -p engine-linalg --test determinism_guard
cargo test --offline -p engine-api --test determinism
cargo build --offline -p engine-api --example determinism_wasm --target wasm32-unknown-unknown
cargo run --offline -p engine-api --example determinism_native > /tmp/eng-core-006-native-debug.txt
node scripts/check-determinism.mjs \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/examples/determinism_wasm.wasm" \
  /tmp/eng-core-006-native-debug.txt
cargo build --offline --release -p engine-api --example determinism_wasm --target wasm32-unknown-unknown
cargo run --offline --release -p engine-api --example determinism_native > /tmp/eng-core-006-native-release.txt
node scripts/check-determinism.mjs \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/examples/determinism_wasm.wasm" \
  /tmp/eng-core-006-native-release.txt
cmp /tmp/eng-core-006-native-debug.txt /tmp/eng-core-006-native-release.txt
```

The scalar guard and all four engine-api determinism tests pass
in the reconciled tree; no checks are skipped to run the comparison across targets.

A toolchain with wasm32-unknown-unknown, provisioned dependencies and
Node with WebAssembly/BigInt is required. The comparison across targets supplements cargo test/check:
`cargo check` for wasm only compiles; it does not execute the analyses.

## Evidence and limits

Actual offline comparison across targets on 2026-10-15 for MAM-582, with the ongoing merge of
origin/main `00e2b48` (#134/MAM-581 and #132): Rust 1.99.0 (LLVM 23.1.1),
x86_64-unknown-linux-gnu versus wasm32-unknown-unknown in Node v22.19.0,
ordinary Cargo flags and unset MAMUT_ENGINE_GIT_COMMIT
(`git_commit: null`). CARGO_TARGET_DIR=/tmp/mam253-target is used.
**Debug and release: 16 matching native/wasm hashes, ten runs per case/target**;
for analyses only `target` is validated and excluded. `cmp` of native
outputs confirms the same 16 hashes across profiles. Local outputs and logs:
`/tmp/mam582-merge-evidence/`.

MAM-582 removes `options.allow_experimental: true` from the two P-Delta fixtures;
this changes their JCS `input_sha256` fingerprint, retained in the hashes used for comparison across targets.
The audit before this merge, against #103, checked that the three modified
fixtures (including the valid example) changed only `input_sha256` in
results/1 and recovered the #104 hashes. Current combined hashes
also include the MAM-581 (#134) report and convergence criterion.
The other fourteen hashes remain identical. The registry publishes `Stable`
and the justification from [ENG-BENCH-002 (PR #132)](../provenance/ENG-BENCH-002.md)
in capabilities/1, outside this suite.
The previous/new table and P-Delta fingerprints are in the
[MAM-582 note in ENG-CORE-006](../provenance/ENG-CORE-006.md#mam-582--pdelta-estable--2026-10-15).
Earlier #98/#103 and #104 evidence is retained in the
ENG-CORE-006 and ENG-CORE-005 logs, respectively.

The evidence covers eleven V1 analyses and five scalar bit cases; it does not
certify all inputs/architectures or intermediate solver bits.
Pinned libm removes the unspecified-precision std alternative for
transcendentals, but any new backend, libm version or flags require
rerunning the comparison across targets. On differences, record the exact input,
versions/flags, hashes and affected field/values; a numerical tolerance does not
replace bitwise equality.

MAM-581 (#134, after #103 in main 5d0139c): P-Delta results add
`residual_convergence_ratio` per iteration and stopping uses the per-DOF criterion with
a rounding floor. Only `pdelta` (980bf21b…56ca → f2cc535e…6740) and
`pdelta-inclined` (9b5a4584…a16b → 70825cf7…978b) change; the other 14 hashes remain identical.
Offline comparison across targets 2026-10-15 with stable, Node: 16 native=wasm cases in debug and release,
ten runs per case; debug/release `cmp` OK.

MAM-582 combined with MAM-581, before → after from main `00e2b48`:

| Case | Main (MAM-581) | Merge (MAM-581 + MAM-582) |
| --- | --- | --- |
| pdelta | `f2cc535ed32fb2ed071a3ed8048ebf53c2cd81083efe8ba528e5c95d8cf46740` | `d0f5475fbd1123cb33699ae7f6fc41828907629c0bb0ac55fe29561e8b9b2b53` |
| pdelta-inclined | `70825cf724a03f9b82f1fd21c76175645a78bc4fcaea648504a7bfc6fd3b978b` | `8ca08c230d5f8888a46bd60d7e5acf988267dab57d3da86a6ea3763304c3df06` |

The other 14 hashes are unchanged relative to main.

Near or above buckling, P-Delta may terminate with `unstable` or
`notConverged`: exit 4, without mechanical results; the convergence
report is retained. The test
[`pdelta_nonconvergence_and_instability_return_four_with_iteration_report`](../crates/engine-api/tests/dynamic.rs)
checks `notConverged` with P/Pcr=0,2 when limiting `max_iterations=1`
(one reported iteration and `cases: []`) and `unstable` with P/Pcr=1,2
(no iterations and `modes: []`), both with exit 4. The first case tests
the iteration limit rather than a threshold near buckling.

SHA-256 for comparison across targets (analyses without `target`; complete scalar bits), identical in
native/wasm and debug/release:

<!-- | Entrada / caso de bits | Hash | -->
| Input / bit case | Hash |
| --- | --- |
| cantilever | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `d0f5475fbd1123cb33699ae7f6fc41828907629c0bb0ac55fe29561e8b9b2b53` |
| modal-coupled | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `8ca08c230d5f8888a46bd60d7e5acf988267dab57d3da86a6ea3763304c3df06` |
| modal-inclined | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |
| scalar-trig | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` |
| scalar-exp-log | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` |
| scalar-powers-norm | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` |
| load-rotation-bits | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` |
| scalar-sin-cos | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` |

The hashes record this version of the contract, engine and cases; legitimate changes
may alter them. The table is the reproducible reference; tests compare
executions, without freezing hashes as goldens in code.
