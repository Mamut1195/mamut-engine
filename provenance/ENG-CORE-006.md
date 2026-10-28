# Clean-room log · ENG-CORE-006

- Ticket: MAM-20 · ENG-CORE-006 «Method and capability registry (on mamut-workflows)».
- Branch: j2/eng-core-006, stacked on j2/vnd-mamut-workflows / PR #100.
- Author: Codex agent, GPT-6, 2026-10-14 (America/Guayaquil).
- Role: implementer.
- Contracts: method-descriptor/1, capabilities/1 discovery and job option
  engine-input/1 `options.allow_experimental`; results/1 remains compatible.
- Permitted sources: ticket request; own engine-api and
  engine-analysis APIs, their tests and logs; authorized vendored interfaces
  of mamut-workflow v0.1.0 (registry.rs, descriptor.rs, catalog.rs, compiler.rs,
  contract.rs, workflow.rs and lib.rs). No network.
- Own derivations: adaptation of complete documents to dimensionless SDK
  `document` ports; engine-input/results URNs with major-version
  compatibility. No mechanical formulas are added and solvers are not changed.
- Third-party code copied: none in this ticket. The Apache-2.0 SDK previously
  vendored in #100 is used through a path dependency, without modifying vendor.
- Validation: id/contract resolution through the SDK compiler, domain/name/major
  incompatibility, minor/patch compatibility, Experimental policy
  before assembly, capability determinism and existing evidence.
- Comparison with external oracle: no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Comparison and scope

The SDK already provides MethodRegistry, factories, MethodDescriptor, CatalogDocument
for processes that only have JSON and WorkflowCompiler with contract resolution.
MethodRegistry<EngineMethod> is used because the engine needs executable factories;
no other registry, index, map or MethodCatalog implementation is created.

The facade already executed linearStatic, combinations, pDelta, modal and
responseSpectrum, with analytical tests, cancellation, progress, SHA-256 and
canonical output. Those calculations and warnings #78/#84/#85/#86 are preserved.
Descriptors, states, evidence in capabilities and job consent
were missing. Adapters are separated from the orchestrator: `run`
invokes the SDK-compiled instance, without branches by method. Combinations
are resolved in the same registry as postprocessing of retrieved linear cases.

Each MethodDescriptor contains the generic SDK descriptor under `workflow`,
with method version 1, plus state, JSON schemas, fixtures, citations to own
derivations and provenance paths. No external bibliographic citations are invented:
the equations and their validation are in ENG-API-001 (increments 2a/2c) and the
original logs for each method. The URNs only adapt engine-input/1 and
results/1 to SDK contract syntax; they do not publish another model/job format.
Mechanical options are validated in the document DTO, not duplicated as
workflow NumericParameters. No general workflow plan is implemented.

P-Delta is Experimental because of documented limitations #78/#84;
`options.allow_experimental: true` is required and its fixture is published with
explicit consent. Absent or false produces invalid / API-E-INPUT / exit 2 with the method
name and required option, without assembly. The other four methods are
Validated within the published analytical scope. Deprecated is defined for
future compatibility; no current method is deprecated. Validated does not certify
arbitrary models or close open follow-ups.

capabilities retains the previous fields and adds `method_descriptors` in
registry order. Schemas are published for method-descriptor/1 and capabilities/1;
engine-input/1 accepts the boolean job option. All new keys and
new definitions use ASCII snake_case. LEGACY_DEFS remains frozen.

## Cargo

engine-api adds only a path dependency on mamut-workflow =0.1.0.
Cargo.lock adds that edge; versions, members and external dependencies
are unchanged. CARGO_TARGET_DIR retains the exported value /workspace/.cargo-target/me45.

## Checks before the merge

All Cargo checks use `--offline` and the exported CARGO_TARGET_DIR
`/workspace/.cargo-target/me45`; no network was accessed and no packages were installed.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 521 tests and doctests, no failures or
  ignored tests; includes three new resolution/policy/discovery tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps`: OK.
- `PATH=/home/box/.cargo/bin:$PATH cargo +stable check --offline --workspace
  --target wasm32-unknown-unknown`: OK, with the target already installed. Checks
  compilation; no execution in a wasm runtime.
- `git diff --check` and review of modified files: OK. No changes to
  docs/CLEAN_ROOM.md, .github/**, clean-room/** or vendor/**; LEGACY_DEFS intact.
- No panic/unwrap/expect in crates/engine-api/src.
- `bash clean-room/check.sh origin/main`: exit 1 solely due to the four
  preexisting occurrences of `algorithm` in
  vendor/mamut-workflows/mamut-canon/src/lib.rs, known #100 blockage.
  There are no other diagnostics. Only that authorized failure is excepted;
  the filter, lists and vendored SDK are unchanged.

Human clean-room review pending (J4). Local commit, no push.

## Merge reconciliation and determinism · 2026-10-14

Reviewed base: merge e53cf0d of j2/vnd-mamut-workflows, with #98 (libm and
determinism), #99 (Winkler) and mamut-canon 585f32d. No unresolved entries
remain in the index or conflict markers. Resolution of
crates/engine-api/Cargo.toml preserves mamut-workflow and the determinism_wasm
cdylib example; the registry and adapters pass their existing tests.
No other merge breakages were found within the checks below.

The shared_native_wasm_suite_hashes_are_repeatable and
v1_results_are_identical_in_ten_runs failures were reproduced: the #98 pdelta and pdelta-inclined
fixtures still had empty options. Both now declare
`options.allow_experimental: true` in the job. By Jonathan's decision,
P-Delta retains Experimental status until BENCH-001; the registry is not changed
and its policy is not skipped to make tests pass.

SHA-256 hashes of complete results/1, #98 reference → regeneration:

| Case | Previous (#98) | Current (ENG-CORE-006) |
| --- | --- | --- |
| pdelta | `5bcea296b5fd2652f75aac339b80bffa82e2076b06279f4020db817644a31101` | `61462a610d4b66c1ac13f0e5e3a37a4194052ca519cb0d3c2cd76bc2d3970ec4` |
| pdelta-inclined | `efd49f53009742c13ff5dbc66fcbf43e063cbfb707cc61dbe23ee043aaf4614f` | `01510cb66b56a52eeecdfa275d0c8035a775522c2f81dc59456497cbb0b5a31d` |

Consent changes the canonical input and its input_sha256, included in
results/1. Additional offline audit: removing allow_experimental from input
only to calculate its previous hash, replacing only input_sha256 in
the current result and canonicalizing recovers exactly both #98 SHA-256 hashes.
This checks that no other result field or mechanical calculation
changed in these two cases. The other fourteen hashes remain equal.
method_descriptors does extend capabilities/1, but that output is not included
in results/1 or scalar cases: it does not cause changes in these hashes.

Reproducible regeneration through docs/DETERMINISMO.md commands:
determinism_native and determinism_wasm examples, plus
scripts/check-determinism.mjs, without npm, WASI or wasm-bindgen. Rust 1.99.0
(LLVM 23.1.1), x86_64-unknown-linux-gnu and wasm32-unknown-unknown, Node
v22.19.0, usual flags. Debug and release: sixteen cases with ten runs
per case/target, complete hashes equal; cmp of the native outputs of the
two profiles passes. The updated docs/DETERMINISMO.md table matches
the sixteen hashes of both outputs. There were no frozen goldens in the
Rust tests; their comparison of hashes and bytes between executions is preserved.
The outdated note on the absence of Winkler in that document is also corrected,
without attributing coverage in this suite to it.

Post-merge checks, all without network and with exported CARGO_TARGET_DIR
`/workspace/.cargo-target/me45`:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: 535 tests and doctests
  pass, one fails, none ignored. Sole failure:
  engine_crates_use_only_the_central_scalar_module, due to direct sqrt in
  crates/engine-validation/tests/support/winkler_harness.rs. The file is
  identical to origin/main (abe7453); it remains intact as requested,
  since its correction belongs to PR #106. The three engine-api determinism
  tests pass.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- Actual cross-check with Node debug and release and cmp between profiles: OK.
- `bash clean-room/check.sh origin/main`: OK; the integrated vendoring exception
  validates the 585f32d manifest and no longer produces the blockage
  documented in the earlier checks.
- `git diff --check`: OK. Only the two P-Delta fixtures,
  docs/DETERMINISMO.md and this log are modified. No changes to vendor/**,
  clean-room/**, .github/** or docs/CLEAN_ROOM.md.

Local commit, no push. Human clean-room review pending (J4).


## Reconciliation of #103 onto #104 · 2026-10-15

Local branch `wip/core-006-sobre-005`: HEAD `67c54d3`, merge in progress of
`origin/j2/eng-core-005` (`6a4b58d`), with `origin/main` = `19c3ae5`.
This section records the new evidence; earlier sections describe
the historical checks, including their blockages already corrected in the base.
No abort or rebase is performed; git add/commit are not run and the index
remains intact. J2 must mark the resolution and create the merge commit.

Cargo.lock, crates/engine-api/README.md, src/lib.rs, src/run.rs and
docs/DETERMINISMO.md are resolved. Cargo.toml preserves both path dependencies
mamut-canon and mamut-workflow; Cargo.lock is regenerated with `cargo metadata
--offline --format-version 1` and checked with `cargo check --offline
--workspace`. The direct engine-api sha2 dependency is removed: run uses
`mamut_canon::fingerprint_value` (RFC 8785/JCS), and tests and examples use
`mamut_canon::sha256_hex`. sha2 remains only as a transitive
mamut-canon dependency; removing it would require modifying the forbidden vendor.
Package versions do not change and vendor is not modified.

lib.rs includes methods and query, with their exports. run retains #103 method
dispatch and progress budget, without duplicating the parse
checkpoint; it emits git_commit and target also in the #104 internal error
envelope. The README preserves discovery/registry and fingerprint/queries.

### Previous/new table

#103 hashes: complete results/1, before incorporating metadata and JCS
from #104. New hashes: results/1 with git_commit null, removing only
target after checking it; the five scalar cases still hash the same
big-endian bits without JSON. The input hash is not compared with the result hash.

| Case | Previous (#103) | New (#103 + #104) |
| --- | --- | --- |
| cantilever | `d993e74910a942cdd27f727e635e3f109d7eac65ad8c0da69cec63a37afd1861` | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `3378d324bd96923bab07640a17c9afd1fff18f69842a1f21bbc06152c8d76eed` | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `2ba08c590e219f982b9b33f0955d9c790b2183b5d1af6309f09b579d43c92a71` | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `61462a610d4b66c1ac13f0e5e3a37a4194052ca519cb0d3c2cd76bc2d3970ec4` | `980bf21b60a5b931a0f0c137ebc782a4a878abb95f7e8bce880c91c4374f56ca` |
| modal-coupled | `59499893099017bbb13ffcea8bed513537a059d929cddebc5511bee9a2de2f4e` | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `a403028d3dc23b3d1e3312492866c7eacd9dd0431c2b50f5563bdefd83c25eb1` | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `01510cb66b56a52eeecdfa275d0c8035a775522c2f81dc59456497cbb0b5a31d` | `9b5a4584e2151bb4d72c34933f564e8642e85600d44662e78ea376a61541a16b` |
| modal-inclined | `b3c4b92d66bc3894b2400354bb3518d78a8c224a6870cfc7aadd2861e2c0e480` | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `a03dea7b4d9dd115cbef6b74cdf68e739e0d4655197957064d03a58a11bd0ac8` | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `152dc9c3a40210634053378061c25668b93bf2eb018223e90b8b18712d9e754f` | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `58e6c5f1733a576a969d323164bf7dbee19155974688a33c73036b2462d4f060` | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |
| scalar-trig | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` |
| scalar-exp-log | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` |
| scalar-powers-norm | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` |
| load-rotation-bits | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` |
| scalar-sin-cos | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` |

The eleven analyses change relative to #103; the five scalar cases remain
identical. Additional offline audit: removing git_commit and target from the
new result and replacing input_sha256 with the SHA-256 of canonical_json of the
same input reproduces exactly the eleven previous #103 hashes. Thus the
change is attributed to those fields, without mechanical differences.

Nine analyses and the five scalars match the #104 table. For
pdelta and pdelta-inclined the inputs are checked to differ exclusively
in `options.allow_experimental: true`, required by the #103 Experimental policy.
Replacing only input_sha256 in the new result with the JCS fingerprint
of the #104 fixture reproduces its hashes exactly:

| Case | Hash #104 | Hash new |
| --- | --- | --- |
| pdelta | `426db50197873a7a8cafb586233ac7af70d8953c22d3a718fe6dbbb51b4637f0` | `980bf21b60a5b931a0f0c137ebc782a4a878abb95f7e8bce880c91c4374f56ca` |
| pdelta-inclined | `f08ff9c7264cb64c8209956d97da0b2e99343479854f11d39d75773b6ec7c71c` | `9b5a4584e2151bb4d72c34933f564e8642e85600d44662e78ea376a61541a16b` |

The sole results/1 field explaining that difference relative to #104 is:

| Case | input_sha256 #104 | input_sha256 new |
| --- | --- | --- |
| pdelta | `93887960a5cc262e58eefbe32a1942dc3910570c389c950d74472dc5108765de` | `06203cc969a0da17c9289014b81de32aabab113d3e5d98796571c2efcedec88e` |
| pdelta-inclined | `ea61ee74c64f263cda128ad360208d3c1ce846f1380b0b3bf46a8626329ad422` | `c10e197c32b64a35b9a9a4880f7564d3480ea95ebf6d5d7abf698794b6ed86e8` |

method_descriptors extends capabilities/1 and does not participate in results/1 or
scalar cases. The Experimental policy is not changed to obtain hashes.

### Execution and checks

Without network, without installing dependencies, with PATH=/home/box/.cargo/bin:$PATH,
CARGO_NET_OFFLINE=true, CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6
exported and MAMUT_ENGINE_GIT_COMMIT undefined. Rust 1.99.0
(LLVM 23.1.1), Node v22.19.0, x86_64-unknown-linux-gnu and
wasm32-unknown-unknown; usual Cargo flags.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: OK, 621 tests and
  doctests, zero failed and zero ignored. Includes scalar guard (36 tests),
  method registry, JCS fingerprints, results/1 contract and queries.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --offline -p engine-api --tests --target wasm32-unknown-unknown`:
  OK (test compilation, no wasm execution).
- Cross-check commands from docs/DETERMINISMO.md: build/run of the
  determinism_wasm and determinism_native examples in debug and release, Node script in both
  profiles and cmp of native outputs: OK. Sixteen cases, ten runs per
  case/target/profile, all hashes equal native/wasm and debug/release.
- `bash clean-room/check.sh origin/main`: OK.
- `git diff --check`: OK; no conflict markers in the working tree.

Local outputs and logs: `/tmp/me-geo6-merge-evidence/`, including
native-debug.txt, native-release.txt, node-debug.log, node-release.log,
status.txt and fingerprint-audit.json; temporary audit outside the repo.
No changes are made to docs/CLEAN_ROOM.md, .github/**, clean-room/** or vendor/**.
The index retains its unmerged entries until J2 marks the resolution.
Human clean-room review pending (J4).

## MAM-582 · pDelta estable · 2026-10-15

This note supersedes the Experimental P-Delta policy described in the
historical #103 sections. Prepared on HEAD `78c4f00` and reconciled
with origin/main `00e2b48` (#134/MAM-581 and #132), pDelta is published as
`Stable` and enabled by default, with `options: {}` or
`options.allow_experimental: false`. The boolean and its false default value
in engine-input/1, the resolution argument and the gate for
future Experimental methods are preserved. The other methods retain Validated.
Stable does not certify arbitrary models or close limitations #78/#84.

Justification supplied in MAM-582:
[ENG-BENCH-002 (PR #132)](ENG-BENCH-002.md), AISC 360-16 Comm. C2,
cases 1 and 2 pass with error ≤0.27%, within tolerances of 3% in moments M
and 5% in displacements Δ. This ticket evidence is recorded;
the merge incorporates the #132 benchmark in
`crates/engine-analysis/tests/bench_pdelta.rs`; it is verified with the workspace
suite, without consulting the network. No external engine code was consulted.
Promotion to Stable changes no formulas, solvers, tolerances, dependencies
or scalar functions; the merge incorporates the MAM-581 convergence threshold and
engine_core::math is preserved as the sole access to scalar mathematics.

The integration test accepts pDelta without the option, with false and with true,
verifies Stable in resolution and capabilities and its publication in the schema.
Since no Experimental methods remain in the production registry, a unit
test applies the same policy function used by resolve to a test Experimental
descriptor: rejects false with InvalidInput, name and required
option; accepts true. It also checks that other states do not require the
option. No fictitious method is registered in the public API.

allow_experimental is removed from the valid example and the two P-Delta determinism
fixtures. Temporary audit before the merge, against #103,
through engine_api::run, with git_commit null and target x86_64-unknown-linux-gnu: all result fields except
input_sha256 are identical in the three cases. The new state and
citation only affect capabilities/1, outside results/1 hashes.

### Previous → new hashes

Cross-check SHA-256 from docs/DETERMINISMO.md: canonical results/1 after validating and
removing only target; git_commit null, input_sha256 preserved.

| Case | Main `00e2b48` (MAM-581) | Merge (MAM-581 + MAM-582) |
| --- | --- | --- |
| pdelta | `f2cc535ed32fb2ed071a3ed8048ebf53c2cd81083efe8ba528e5c95d8cf46740` | `d0f5475fbd1123cb33699ae7f6fc41828907629c0bb0ac55fe29561e8b9b2b53` |
| pdelta-inclined | `70825cf724a03f9b82f1fd21c76175645a78bc4fcaea648504a7bfc6fd3b978b` | `8ca08c230d5f8888a46bd60d7e5acf988267dab57d3da86a6ea3763304c3df06` |

The other fourteen cross-check hashes do not change relative to main. The combined
hashes include `residual_convergence_ratio` and the MAM-581 stopping
threshold, as well as the `input_sha256` of the fixtures with empty options; therefore
they no longer recover the historical #104 hashes.

Input JCS fingerprints (results/1 input_sha256; they are not output hashes):

| Fixture (relative to crates/engine-api/) | Previous | New |
| --- | --- | --- |
| tests/fixtures/determinism/pdelta.json | `06203cc969a0da17c9289014b81de32aabab113d3e5d98796571c2efcedec88e` | `93887960a5cc262e58eefbe32a1942dc3910570c389c950d74472dc5108765de` |
| tests/fixtures/determinism/pdelta-inclined.json | `c10e197c32b64a35b9a9a4880f7564d3480ea95ebf6d5d7abf698794b6ed86e8` | `ea61ee74c64f263cda128ad360208d3c1ce846f1380b0b3bf46a8626329ad422` |
| fixtures/input/valid/p-delta.json | `78d598e6c7d855a9534f8ff23cbc9406a991c01d867c17ed1ed57b59ed37fc84` | `a76ae5636f2a24be92fa7e6a02c6a148ae2e3736d891888d90f455e111aecbea` |

Historical record before the merge, against #103: SHA-256 of the complete
results/1 document, including native target. These values are not the current
combined hashes or the previous cross-check projection:

| Fixture (relative to crates/engine-api/) | Previous | New |
| --- | --- | --- |
| tests/fixtures/determinism/pdelta.json | `94f0264c274abaae1209f382f260cbcf63d5abe5c525c6d92dbed952cc6eeb8d` | `44ec010eefbe28c5b05159fd134a318523cd38db449afdfaa3d2e8bb56d3b3f9` |
| tests/fixtures/determinism/pdelta-inclined.json | `3310bd8dda26da74974a43cd4e81912972eb9337d90189092e0e99f787891bd8` | `cd2c88b45ba6121012e4a9f44a7e57e1f75268ecd7f92e1110a7510d07a62070` |
| fixtures/input/valid/p-delta.json | `880375ccbe0db091b3f4a5009a539efd9882022b234eb7b5cea66a77889b3422` | `fbc577746fde54e1f0af140bd698417ea24e51156f9909dc3d0a8003efe95a2d` |

### Local MAM-582 verification before the merge

All without network, with PATH=/home/box/.cargo/bin:$PATH, CARGO_NET_OFFLINE=true,
MAMUT_ENGINE_GIT_COMMIT undefined and CARGO_TARGET_DIR=/tmp/mam253-target
(the inherited value was outside writable paths). Rust 1.99.0
(LLVM 23.1.1), Node v22.19.0, without special numeric flags.

- `cargo fmt --all`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, 622 tests and doctests, zero failures and ignored;
  includes scalar guard, unit gate, registry and determinism.
- docs/DETERMINISMO.md procedure: offline build/run of the
  determinism_wasm and determinism_native examples, debug and release, execution of
  scripts/check-determinism.mjs in both profiles and cmp of native outputs:
  OK, 16 cases with ten runs per case/target/profile. Table verified against
  the 16 executed hashes; only the two P-Delta hashes change.
- Audit of the three fixtures before/after: OK, without mechanical changes.
- `git diff --check`: OK.

Temporary evidence in /tmp/mam253-evidence/: before.jsonl, after.jsonl,
clippy.log, test.log, native-debug.txt, native-release.txt, node-debug.log and
node-release.log. The temporary audit example was deleted. No commits
or push are performed. No changes are made to vendor/**, clean-room/**, .github/** or
the docs/CLEAN_ROOM.md policy.

### MAM-581 + MAM-582 merge verification · 2026-10-15

Offline rerun with origin/main `00e2b48` in the merge tree,
MAMUT_ENGINE_GIT_COMMIT undefined (`git_commit: null`),
CARGO_NET_OFFLINE=true and CARGO_TARGET_DIR=/tmp/mam253-target;
Rust 1.99.0 (LLVM 23.1.1), Node v22.19.0, without special numeric flags.

- `cargo fmt --all`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, 630 tests and doctests, zero failures and ignored;
  includes the published #132 benchmarks and
  `pdelta_nonconvergence_and_instability_return_four_with_iteration_report`
  from `crates/engine-api/tests/dynamic.rs`.
- `cargo run --offline -p engine-api --example determinism_native` and
  `cargo build --offline -p engine-api --example determinism_wasm --target wasm32-unknown-unknown`,
  repeated with `--release`, and `node scripts/check-determinism.mjs` per profile:
  OK, 16 equal native/wasm hashes, ten runs per case/target/profile.
  `cmp` of debug/release native outputs: OK. Complete hashes are in
  the combined table above and in docs/DETERMINISMO.md; relative to main only
  `pdelta` and `pdelta-inclined` change, the other 14 remain equal.
- The #132 benchmark moves to [ENG-BENCH-002](ENG-BENCH-002.md) to avoid
  collision with ENG-BENCH-001, the CLEAN_ROOM §5.1 exception.

Temporary evidence of this rerun: `/tmp/mam582-merge-evidence/`
(native-debug.txt, native-release.txt, node-debug.log, node-release.log,
build/run logs, fmt.log, clippy.log and test.log). No network, commit, rebase or push.
