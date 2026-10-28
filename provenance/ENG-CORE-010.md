# Clean-room log · ENG-CORE-010

- Ticket / PR: MAM-23 · ENG-CORE-010 «Determinism policy»; branch j2/eng-core-010 stacked on j2/eng-api-001 (#88), base b270626.
- Author(s): Codex gpt-6.1-sol high under J2, 2026-10-14 America/Guayaquil.
- Role: implementer.
- Contract implemented: V1 execution policy, run(engine-input/1) → canonical results/1; library guard, repeatability and native/wasm cross-check.
- Permitted sources used: ticket instructions received from J2; own sources and documentation from main eed45dd and #88 b270626; own engine-api analytical tests; installed std/Rust API, offline Cargo and Node WebAssembly. No network or external engine queries.
- Own derivations: cantilever EI y''=−P(L−x), y(0)=y'(0)=0 → y(L)=−PL³/(3EI); equilibrium R=P, M=PL. Oscillator EA/L=k=100 and m=5 → ω²=k/m=20. Column Pcr=π²EI/(4L²), P=0,2Pcr set as an input decimal. Modal beam with Uy/Rz DOFs and masses 5/2: the element's own bending stiffness; CQC and log-log use repository APIs. The new tests compare repeated results, without an external oracle or new mechanical algorithms.
- Third-party code copied or vendored: none. The first delivery retained std because of the absence of offline libm; that decision is superseded by the approval and dependency recorded below. serde/serde_json/sha2 are preserved.
- Validation: lexical guard on crates/*/src, with alias/macro/comment/string cases; six valid inputs over ten runs with results/1 hash; native and C-ABI wasm cdylib examples executed with Node without wasm-bindgen or network, in debug and release.
- Comparison with oracle (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

The current scalar policy and migrated inventory appear in docs/DETERMINISMO.md.
The solver, its tolerances and Sturm's explicit FMA are unchanged. No changes
to docs/CLEAN_ROOM.md, .github/** or clean-room/**. Native/wasm equality is limited
to measured evidence; not every input/architecture is certified.

## libm approved by Jonathan 2026-10-14

- Authorization: Jonathan approves libm for native/wasm determinism; J2's
  instruction received on 2026-10-14. PR #98 / MAM-23 / ENG-CORE-010, branch
  j2/eng-core-010 integrated with origin/main d0031f8 (initial HEAD 7fcc5c3).
- Implementation: engine_core::math centralizes transcendental functions with libm
  0.2.16; sqrt retains std for correct IEEE-754 rounding. powi converts
  i32 exactly to f64 and uses libm::pow. Libraries, examples and
  tests of all crates are migrated, without modifying mechanical algorithms, validations,
  tolerances, reductions or Sturm's explicit FMA.
- Dependency/license: metadata from the Cargo.toml of the package installed in the
  local registry, version 0.2.16, license MIT. Workspace Cargo.toml adds
  exact version =0.2.16 and default-features=false (no arch kernels); the
  engine-core manifest inherits libm.workspace=true. Cargo.lock adds libm, the edge from
  engine-core and checksum
  b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981.
  No download, additional dependency, copied code or external engine
  sources; the public API of the permitted dependency is used, without consulting
  OpenSees, Stabileo or derivative algorithms.
- Guard: lexical test on all Rust in crates/**, including tests and
  examples; only excepts engine-core/src/math.rs. Detects std methods,
  UFCS/function references, f64 aliases and direct libm access;
  includes macro/whitespace/comment/string/char tests. Preserves the
  previous hash collection and concurrency guard in libraries.
- Own suite: nine results/1 inputs (six original and inclined
  modal, P-Delta and spectrum variants) and four f64 bit hashes (trig,
  exp/log, powers/norms and load rotation through Beam2D). The variants
  use 0,8x/0,6x coordinates and rotated global forces in P-Delta; they incorporate
  no new mechanical algorithms or external data. Winkler does not exist in main
  d0031f8 or this branch; coverage of that path is not claimed.
- ABI/script: case identity and order by name SHA-256, states and
  index bounds, complete SHA-256 hashes, zero imports and thirteen mandatory cases.
  Scalar bits are hashed as to_bits().to_be_bytes(), without JSON or
  tolerances; ten runs per case/target, without global mutable state.

Offline checks with CARGO_TARGET_DIR=/workspace/.cargo-target/me45,
Rust 1.99.0 (LLVM 23.1.1), without special RUSTFLAGS:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK; 507 tests including doctests,
  0 failures/ignored. The shared suite was checked again after adding
  the explicit sqrt(-0.0) case: 2 tests OK.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- Offline build/execution with Node v22.19.0: OK in debug and release,
  thirteen equal hashes between targets, ten runs per case/target.
  `cmp` of debug/release native outputs: OK, also equal between profiles.
  Commands, hashes and limits in docs/DETERMINISMO.md.
- Script negative control: an altered native hash (equal in its ten
  runs) is rejected by bit-for-bit equality in the first wasm comparison.
- `git diff --cached --check`: OK.
- `bash clean-room/check.sh origin/main`: OK on staged changes,
  including new files. Human review PENDING (J4).
- No changes to docs/CLEAN_ROOM.md, .github/** or clean-room/**; no network or push.

## J4 r25 extension · 2026-10-14

- Request: MAM-23 / PR #98, on fccda4d; explicit sin_cos case and
  inclined frame3d cross-check with local axis rotation, linearStatic and modal.
- Sources: user instructions, own checkout code and fixtures,
  installed Rust toolchain and Node. No network or consulting sources from other
  engines. No third-party code copied or vendored; no new dependencies
  or library changes.
- sin_cos: the engine_core::math::sin_cos wrapper over libm::sincos already
  existed and was already used to calculate both values of the same angle in
  engine-loads::Beam2D and engine-contracts::conformance. It is preserved and
  scalar-sin-cos is added: direct hash of the (sine, cosine) pair bits for sixteen
  angles, including signed zeros, subnormals, neighbors of π/2 and ±1e300.
- Frame3d: two inputs for the same three-node spatial frame with two members
  inclined in XYZ; distinct local orientations unaligned with
  global axes, Iy different from Iz, clamped support and two free nodes with translational
  and rotational masses. One linearStatic input with six load
  components and another modal with six modes and X/Y/Z participation; the API only
  accepts one analysis per input. No mechanical algorithms are added.
- Native test: ten runs of the eleven results/1 inputs and the five
  bit cases; requires the three new cases. Checks twelve nonzero local forces
  per member, six positive modes and three participations per mode;
  changing orientations must alter mechanical results, not only
  input_sha256. The native/wasm examples include the new cases through
  shared support; the Node script requires them as mandatory coverage.
- Actual offline cross-check: Rust 1.99.0 (LLVM 23.1.1), Node v22.19.0,
  x86_64-unknown-linux-gnu / wasm32-unknown-unknown, without special RUSTFLAGS,
  retaining CARGO_TARGET_DIR=/workspace/.cargo-target/me45. Debug and release:
  16 identical complete hashes, ten runs per case/target; cmp of the
  160 debug/release native rows also OK. The thirteen fccda4d hashes
  remain equal. Complete table and reproduction in docs/DETERMINISMO.md.
- New SHA-256 hashes (equal in both targets and profiles):
  - scalar-sin-cos: b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a.
  - frame3d-inclined-static: 152dc9c3a40210634053378061c25668b93bf2eb018223e90b8b18712d9e754f.
  - frame3d-inclined-modal: 58e6c5f1733a576a969d323164bf7dbee19155974688a33c73036b2462d4f060.
- Verification with exported CARGO_TARGET_DIR and CARGO_NET_OFFLINE=true:
  cargo fmt --all -- --check; cargo clippy --offline --workspace --all-targets
  -- -D warnings; cargo test --offline --workspace (508 tests including
  doctests, zero failures/ignored); RUSTDOCFLAGS="-D warnings" cargo doc --offline
  --no-deps --workspace; cargo check --offline --workspace --target
  wasm32-unknown-unknown: all OK.
- git diff --cached --check and bash clean-room/check.sh origin/main: OK on
  staged changes, including the two new fixtures. Human review
  PENDING (J4). No changes to docs/CLEAN_ROOM.md, .github/** or clean-room/**;
  no network or push. No panic/unwrap/expect added to the library.


## FU #105 · MAM-23 · J4 r26 · 2026-10-14

- Request: close the five gaps in the #98 scalar guard; branch
  j2/eng-core-010-fu on main abe7453, no network or push.
- Sources: user instructions, own checkout guard and sources,
  std/Rust APIs and local toolchain. No third-party code copied or
  vendored, no new dependency or consulting other engines.
- Guard: distinguishes r#identificador from raw strings and normalizes the name;
  requires the engine_core::math:: path or exact module import without alias
  for the math:: abbreviation. Rejects primitive aliases, as math and
  alternative math redefinitions/imports, as well as paths with another leading segment.
  Unqualified scalar arguments inside macros are rejected;
  data receivers such as log.acceleration(...) remain allowed.
- FMA: exception limited to engine-linalg/src/modes.rs, without excepting
  tests or other files named modes.rs; even math.rs does not allow mul_add.
- include!: only literal .rs paths targeting the inspected Rust
  inventory of crates/**, checked after path canonicalization. Rejects
  files with other extensions and dynamic paths; data include_str!/include_bytes!
  remain allowed. The math.rs scalar exception does not exempt
  the includes check.
- Own fixtures: five pairs of .txt texts, positive and negative,
  consumed as data by five specific tests. Additional tests for
  every function in the list with raw identifiers and three macro
  delimiters, complete paths, equivalent aliases, comments and raw strings.
- Integration: main contains std sqrt/sin_cos uses in engine-validation Winkler
  tests that the previous guard also rejected. They are centralized
  in engine_core::math in winkler_elements.rs and support/winkler_harness.rs;
  sqrt retains its IEEE exception and sin_cos uses the already pinned libm provider.
  No library, mechanical algorithm, tolerance or Sturm FMA changes;
  no panic/unwrap/expect added to the library.
- Documentation: guard scope is made explicit and attribution is corrected
  to «extension after J4 r25, reproduced by J4 r26» in docs/DETERMINISMO.md.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket."
- Offline verification with exported CARGO_TARGET_DIR
  (/workspace/.cargo-target/me45), CARGO_NET_OFFLINE=true:
  - cargo fmt --all -- --check: OK.
  - cargo clippy --offline --workspace --all-targets -- -D warnings: OK.
  - cargo test --offline --workspace: OK, 520 tests including doctests,
    zero failures/ignored; all nine guard tests pass.
  - RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK.
  - cargo check --offline --workspace --target wasm32-unknown-unknown: OK.
  - git diff --cached --check and bash clean-room/check.sh origin/main: OK,
    on staged changes, including the ten new fixtures.
- Human clean-room review: PENDING (J4). No changes to docs/CLEAN_ROOM.md,
  .github/** or clean-room/**.


## FU2 #107 · MAM-23 · J4 r27 · 2026-10-14

- Request: close uninspected code paths and FFI; branch
  j2/eng-core-010-fu2 stacked on j2/eng-core-010-fu, PR #106,
  base commit 101c4d1; local origin/main abe7453. No network or push.
- Sources: user instructions, own checkout sources and
  documentation, std/Rust APIs and installed toolchain. No third-party code
  copied or vendored, no new dependency or consulting other engines.
- Paths: #[path = "..."] shares include! validation: .rs literal without
  escapes and canonicalized target within the crates/** Rust inventory.
  Whitespace, raw identifiers, raw strings and cfg_attr
  attributes with multiple attributes or nested cfg_attr are inspected, regardless of their
  condition. cfg(path = "...") predicates are not module attributes.
  Nonexistent targets and actual files outside the inventory are rejected;
  math.rs and examples/ are not excepted from this rule.
- FFI: #[link_name = ...], also in cfg_attr, and
  extern "..." { ... } / extern { ... } blocks, including raw ABI and macros, are rejected in
  crates/** outside crates/*/examples/. extern crate,
  extern functions and function pointer types are not confused with import blocks;
  extern crate retains the guard's previous prohibitions.
- Library: #![forbid(unsafe_code)] is added to all fifteen crates/*/src/lib.rs
  and also to bindings/python/src/lib.rs: PyO3 0.25.1 compiles with the prohibition,
  so no binding exception is needed. The guard requires the inner,
  unconditional attribute at each library root, and checks the binding
  separately. Explicit UNSAFE_LIBRARY_EXCEPTIONS list empty, without library
  exceptions or globs. Attributes in comments, strings, submodules,
  macros or cfg_attr, and deny(unsafe_code), do not satisfy the check.
- Existing unsafe: engine-linalg/tests/heap_peak.rs implements GlobalAlloc
  and delegates to System through unsafe to measure memory; it is an integration
  test, outside the library. It is preserved without imposing forbid and
  does not justify an engine-linalg exception. vendor/** is untouched.
- Tests: five specific negatives for non-.rs path, path outside the
  inventory (includes an actual binding .rs), link_name, extern block and
  lib.rs without forbid. Positives for inspected paths, raw strings,
  nested attributes, comments, strings, cfg predicates and examples/;
  the examples/ exception does not relax paths or previous prohibitions.
  Each of the five checks was temporarily removed: its specific
  test failed by assertion in all five cases. With code restored,
  all sixteen guard tests pass.
- Documentation: docs/DETERMINISMO.md records the new scope, the list without
  exceptions and the distinction between libraries and the GlobalAlloc test.
  Mechanical algorithms, tolerances and Sturm FMA are unchanged; no
  panic/unwrap/expect added to the library.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket."
- Offline verification with CARGO_TARGET_DIR=/workspace/.cargo-target/me45
  already exported, CARGO_NET_OFFLINE=true and Rust/Cargo 1.99.0:
  - cargo fmt --all -- --check: OK.
  - cargo clippy --offline --workspace --all-targets -- -D warnings: OK.
  - cargo test --offline --workspace --no-fail-fast: OK, 527 tests including
    doctests, zero failures/ignored; sixteen guard tests.
  - RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK.
  - cargo check --offline --workspace --target wasm32-unknown-unknown: OK.
  - git diff --cached --check and bash clean-room/check.sh origin/main: OK,
    on all staged changes.
- Human clean-room review: PENDING (J4). No changes to docs/CLEAN_ROOM.md,
  .github/**, clean-room/** or vendor/**.

## FU3 · MAM-23 · J4 r29/r30 · 2026-10-15

- Request: close attributes/extern generated by macro_rules, export_name,
  assembly and the implicit tests/examples exception; branch
  j2/eng-core-010-fu3 based on main 800ef3f, which includes #106 and #108.
- Sources: user instructions, own checkout guard, sources and documentation,
  std/Rust APIs and local toolchain. Offline work, without consulting
  other engine sources, copying third-party code or adding dependencies.
- Guard: rejects attributes with metavariables, including repetitions and
  cfg_attr, and extern with variable ABI/body/tokens inside macro_rules.
  Rejects export_name and asm!/global_asm!/naked_asm! in crates/**, including
  core::arch/std::arch paths, raw identifiers and macro delimiters.
  Nested cfg_attr attributes and unsafe(...) wrappers are inspected
  independently of compilation conditions; path/link_name,
  includes inventory and forbid(unsafe_code) in libraries are maintained.
- Tests/examples: SOURCE_POLICY_EXCEPTIONS replaces the unsafe convention
  and the general examples/ FFI exception with three exact, justified paths
  with minimal permissions: heap_peak.rs only unsafe (GlobalAlloc delegated to
  System); determinism_wasm.rs and winkler_wasm.rs only no_mangle and public
  extern "C" functions. Current examples do not use unsafe and do not allow it.
  No extern blocks, link_name, export_name or assembly are allowed in
  any exception. Permissions are not inherited by name or subdirectory.
  The guard fails if an excepted file or any of its authorized
  constructs disappears, even if it still uses the others.
- Tests: twenty new tests, thirty-six in the guard. Negatives per
  construct in src/tests/examples and inside macros with three delimiters;
  extra permissions, unlisted paths, stale permissions and missing file.
  Positives for minimal exceptions, nested comments, normal/raw
  strings, characters, harmless attributes and macros with constant attributes.
  Temporary mutation: 61 negative runs confirmed by assertion failure
  of the selected test, excluding compilation errors and empty tests.
  Each detector was removed; macro bodies were also skipped, a general
  tests/ and examples/ exception was restored separately, and
  minimal/stale permission and inventory checks were removed. Code restored.
- Documentation: docs/DETERMINISMO.md distinguishes the previous convention from the
  new encoded policy and details the exact list and its validity checks.
  No library, mechanical algorithm, tolerance or Sturm FMA changes;
  no panic/unwrap/expect added to the library.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket."
- Offline verification with CARGO_TARGET_DIR=/workspace/.cargo-target/me45
  retained from the environment and CARGO_NET_OFFLINE=true:
  - Initial guard and 61 mutation runs: Rust/Cargo 1.85.1 from the initial
    PATH; all negatives failed at their selected assertion.
  - Complete verification with Rust/Cargo 1.99.0 already installed locally,
    including its wasm32 target; without downloading or installing components:
    cargo fmt --all -- --check: OK;
    cargo clippy --offline --workspace --all-targets -- -D warnings: OK;
    cargo test --offline --workspace --no-fail-fast: OK, 565 tests including
    doctests, zero failures/ignored, thirty-six guard tests;
    RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace: OK;
    cargo check --offline --workspace --target wasm32-unknown-unknown: OK.
  - git diff --cached --check and bash clean-room/check.sh origin/main: OK
    on the three files staged for commit.
- Human clean-room review: PENDING (J4). No changes to docs/CLEAN_ROOM.md,
  .github/**, clean-room/** or vendor/**. No network or push.
