# Clean-room log · ENG-CORE-005 · MAM-14

- Ticket: ENG-CORE-005 «results/1 results schema and provenance».
- Branch: j2/eng-core-005, stacked on j2/vnd-mamut-workflows / PR #100.
- Author: Codex (GPT-6), under J2's direction, 2026-10-14 America/Guayaquil.
- Role: implementer. Clean-room reviewer: PENDING (J4).
- Permitted sources: user requirements; own crates/engine-api and engine-core
  contracts, documentation, fixtures and APIs; public API and documentation
  of mamut-canon v0.1.0 vendored in #100 (SDK-WF-002); declarative structure
  of JSON Schema Draft 2020-12 and RFC 8785 as described by the local SDK.
  No network, sibling repositories, other engines or external numerical oracles.
- Third-party code copied: none. The SDK is already vendored by #100 and
  is consumed through a path dependency; vendor/** remains verbatim and unchanged.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

## Prior comparison and incremental scope

Local `main` (b744599) is behind and does not yet contain engine-api. It was compared
with origin/main (d0031f8, which includes #88/#95), without fetch or network access.
Before the work, crates/engine-api had no differences from origin/main.
The branch base is 49f4609, merge of origin/main into #100.

The results/1 schema, nodes/members/modes/cases/envelopes/
convergence/diagnostics blocks and combinations, engine_version/contracts/
methods/input_sha256 metadata, canonical output, error/cancellation handling and all
static/P-Delta/modal/spectrum methods already existed. Analysis is not reimplemented.

Changes in this ticket:

1. The fingerprint uses `mamut_canon::fingerprint_value` on the Value from the prior
   strict parse (limits and duplicate/overflow rejection preserved).
   No custom canonicalization for input_sha256.
2. git_commit metadata (40/64-character hexadecimal revision, optional env
   MAMUT_ENGINE_GIT_COMMIT, null if absent/invalid) and target (Cargo TARGET).
   build.rs only reads env, without git/network/clocks. Also emitted in errors and
   in the constant internal error envelope. The schema accepts both as optional
   fields to accept previous results; without changing existing required fields.
3. Rust API `Outcome::query(&ResultQuery)` with typed IDs, ordered sets
   for lookup and field paths. None = all, empty Some = nothing. Filters
   identities and case/combination/mode/spectrum/envelope responses;
   preserves order, referenced geometry, diagnostics and provenance, without
   recalculating aggregates. Dot projection through arrays, preserving
   id; projected fragments do not promise to satisfy the complete schema.
4. Native tests validate outputs against the local schema, including warnings,
   failures, convergence and cancellation. The helper only implements the
   Draft 2020-12 subset used, with exclusively local references and the three
   published patterns; fails on unknown keywords/patterns. Negative mutation
   tests verify its ability to detect violations. The
   same test code is compiled for wasm32; wasm is not executed.

New contract/API fields use ^[a-z][a-z0-9_]*$. No definitions
or exceptions were added to LEGACY_DEFS (frozen by MAM-480).

## Fingerprint change and byte compatibility

RFC 8785/JCS uses ECMAScript/binary64 representation and UTF-16 key order.
The hash may change for the same input relative to #88: `1.0` becomes `1`,
`-0.0` becomes `0`, some exponents change form, integers greater than 2⁵³ are
rounded to binary64 and supplementary keys change order.
fixtures/results/fingerprints.json pins JCS bytes and previous/new hashes:
for {"v":1.0}, the own hash
f3c8be97307d38261061fa08e74365e54a239cddb1d189f3bea18022adbe2f35 becomes
afbf9d0f3560b0fd7795e81c42a0a79ee6b6fc67e064f77826aee642cad28d91.
The explicit bytes of these vectors are own declarative oracles;
SHA-256 of those bytes was calculated locally with hashlib, without network.
SHA expectations in run and dynamics tests were updated to the SDK;
equality is checked across mechanical inputs with integers/decimals,
key permutation, whitespace and negative zeros.

results/1 and the public canonical_json function retain their own published
serializer: keys in scalar order, round-trip floats, exact integers and
-0.0 → 0.0. Using JCS there would change previously published bytes/numeric types;
the SDK is applied to the fingerprint, without migrating that output contract. The
determinism/canonical_json tests on results/1 are preserved.

## Cargo and pending items

- engine-api replaces the direct sha2 =0.10.9 dependency with mamut-canon
  path ../../vendor/mamut-workflows/mamut-canon, version =0.1.0.
- Cargo.lock changes only engine-api dependencies; it adds no registry packages or
  versions. SHA-256 remains available transitively through the SDK.
- No additional changes to workspace Cargo.toml or vendor/**.
- CLI/schema through CLI: PENDING ENG-SFC-002; no CLI exists yet.
- The initial #100 blockage in clean-room/check.sh was resolved in the
  dependency before merge 3d10b33; it is verified again below without modifying
  script/lists or vendored content.

## Initial verification (before merge 3d10b33)

Offline commands with CARGO_TARGET_DIR exported to
/workspace/.cargo-target/mamut-engine and toolchain from /home/box/.cargo/bin.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 525 tests, 0 failures/ignored.
  engine-api: 87 tests, including 7 new ones in tests/results.rs. The 30 existing
  run/dynamic tests now validate each Outcome against the actual schema.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --offline -p engine-api --tests --target wasm32-unknown-unknown`:
  OK; validates portable test compilation, without wasm or CLI execution.
- Metadata test with MAMUT_ENGINE_GIT_COMMIT=0123456789abcdef0123456789abcdef01234567:
  OK; subsequent complete checks with the optional env absent.
- `bash scripts/check-vendor.sh`: OK, SHA manifest intact.
- `git diff --check`: OK; no panic/unwrap/expect in library/build.rs.
  No changes to docs/CLEAN_ROOM.md, .github/**, clean-room/** or vendor/**.
- `bash clean-room/check.sh origin/main`: exit 1 solely due to four
  occurrences of `algorithm` in vendor/mamut-workflows/mamut-canon/src/lib.rs
  (prior #100 blockage). No other findings; no rule is modified.
- Human clean-room review: PENDING (J4).

## Semantic integration of #98/#99 after 3d10b33

Local repair on 2026-10-14 by Codex (GPT-6), implementer role, without network or
consulting sibling repositories. Base of this repair: merge 3d10b33 of
j2/vnd-mamut-workflows; local origin/main = abe7453, SDK pinned to 585f32d.
Additional sources: own #98 tests, examples, cross-check script and documentation;
public API `mamut_canon::sha256_hex` of the local SDK. The initial clean-room
declaration is maintained. No changes are made to vendor/**, clean-room/**,
.github/**, docs/CLEAN_ROOM.md or the Winkler harness.

The #98 helpers and wasm test/example still imported sha2 directly.
They are replaced by `mamut_canon::sha256_hex(bytes)`, which calculates SHA-256 on
the received bytes without passing through JCS. For the wasm ABI the 64
hexadecimal characters are decoded to the same 32 bytes. Scalar cases concatenate
`to_bits().to_be_bytes()` in identical order before hashing. sha2 is not reintroduced,
Cargo.lock is unchanged and no dependencies are added. The Node script retains
its independent name hash verification with node:crypto.

`target` now identifies the actual Cargo target and necessarily differs
between x86_64 and wasm32. The cross-check helper checks its value and removes only that
key before serializing again with `engine_api::canonical_json` and hashing.
The public run output is not modified; the ten-run test on the
complete document still also checks target. An additional test
checks that projection preserves all other fields and rejects incorrect,
null or absent target. `git_commit` remains included in the cross-check hash:
regeneration uses the optional variable absent and, therefore, null.

All eleven analysis hashes change: input_sha256 adopts JCS and
`git_commit: null` is incorporated into the compared bytes. The previous table corresponded to the
complete #98 document, still without git_commit/target; the new one corresponds to the
current document after validating/removing target. The five scalar hashes do not
change, confirming digest equivalence and bit preservation.
The new values are also in docs/DETERMINISMO.md; that table is reproducible
evidence, not a list of goldens hardcoded in the test/script.

| Case | Previous SHA-256 (#98) | New SHA-256 (ENG-CORE-005 cross-check) |
| --- | --- | --- |
| cantilever | `d993e74910a942cdd27f727e635e3f109d7eac65ad8c0da69cec63a37afd1861` | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `3378d324bd96923bab07640a17c9afd1fff18f69842a1f21bbc06152c8d76eed` | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `2ba08c590e219f982b9b33f0955d9c790b2183b5d1af6309f09b579d43c92a71` | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `5bcea296b5fd2652f75aac339b80bffa82e2076b06279f4020db817644a31101` | `426db50197873a7a8cafb586233ac7af70d8953c22d3a718fe6dbbb51b4637f0` |
| modal-coupled | `59499893099017bbb13ffcea8bed513537a059d929cddebc5511bee9a2de2f4e` | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `a403028d3dc23b3d1e3312492866c7eacd9dd0431c2b50f5563bdefd83c25eb1` | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `efd49f53009742c13ff5dbc66fcbf43e063cbfb707cc61dbe23ee043aaf4614f` | `f08ff9c7264cb64c8209956d97da0b2e99343479854f11d39d75773b6ec7c71c` |
| modal-inclined | `b3c4b92d66bc3894b2400354bb3518d78a8c224a6870cfc7aadd2861e2c0e480` | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `a03dea7b4d9dd115cbef6b74cdf68e739e0d4655197957064d03a58a11bd0ac8` | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `152dc9c3a40210634053378061c25668b93bf2eb018223e90b8b18712d9e754f` | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `58e6c5f1733a576a969d323164bf7dbee19155974688a33c73036b2462d4f060` | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |

Additional check of the cause: a temporary own example ran
the eleven current fixtures, removed git_commit/target and replaced
input_sha256 with SHA-256 of `canonical_json(input)` (the previous fingerprint). After
serializing with canonical_json, **exactly the eleven previous hashes
from the #98 table** were recovered. The example was deleted after the audit.
This verifies that the rest of the results, including solver numbers,
retain their bytes; mechanics, tolerances and fixtures were not adjusted.

Cross-check reproduction: docs/DETERMINISMO.md commands with Rust 1.99.0
(LLVM 23.1.1), Node v22.19.0 and CARGO_TARGET_DIR exported to
/workspace/.cargo-target/mamut-engine, without special RUSTFLAGS. Debug and release:
16 cases per profile, ten runs per case/target, native/wasm hashes equal.
`cmp /tmp/eng-core-005-native-debug.txt /tmp/eng-core-005-native-release.txt`:
OK. The five scalar hashes are identical to #98.

Verification after this repair (all Cargo commands without network and with the
indicated exported CARGO_TARGET_DIR):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: **540 passed, 1 failure,
  0 ignored**, including documentation tests. Sole failure:
  `engine_crates_use_only_the_central_scalar_module`, due to direct scalar
  calls in `crates/engine-validation/tests/support/winkler_harness.rs`.
  They were confirmed to also be in origin/main. According to the user's
  instruction, this belongs to PR #106 and is not corrected here. engine-api/tests/determinism:
  four tests passed, including the projection check.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --offline -p engine-api --tests --target wasm32-unknown-unknown`:
  OK.
- `bash clean-room/check.sh origin/main`: OK; the previous vendor blockage no
  longer reproduces with the integration of #100.
- `bash scripts/check-vendor.sh`: OK, inventory and SHA manifest intact.
- `git diff --check`: OK. Protected paths and Cargo.lock unchanged in this
  repair. Local commit, no push. Human clean-room review: PENDING (J4).
