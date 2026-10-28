# Clean-room log · ENG-VND-001

- Ticket: MAM-481 · ENG-VND-001; branch `j2/vnd-mamut-workflows` (PR #100).
- Date: 2026-10-14 (America/Guayaquil).
- Author: Codex gpt-6.1-sol high under J2. Role: implementer.
- Clean-room reviewer: PENDING (Jonathan, @Mamut1195).
- Source: <https://github.com/Mamut1195/mamut-workflows>, MAMPRO's own
  repository; local source `/workspace/mamut-workflows`, consulted read-only.
- Source pinned to the `main` commit:
  `585f32dc3455208566a0ffb894ad03ce3816b888`, resolved through
  `git -C /workspace/mamut-workflows rev-parse 585f32d`.
  Declared version: `0.1.0`. The `v0.1.0` tag **does not correspond** to that commit;
  it points to `5c4e75af21564edc1a3b900d565a2ab90447ac1c`.
  The source checkout remains clean at the previous commit; the new
  commit is consulted with `git show`, without modifying the repository or its checkout.
- Upstream and both crates' license: Apache-2.0. LICENSE copied exactly.
  NOTICE written for this distribution (upstream has no NOTICE), copyright
  Mamut1195 / MAMPRO SAS; the original MAMUT notice in LICENSE is also retained.
- Literal clean-room declaration: "código propio de MAMPRO, sin OpenSees/Stabileo".
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

## Files and adjustments

`crates/mamut-canon` is copied to `vendor/mamut-workflows/mamut-canon`
(SDK-WF-002, RFC 8785 + SHA-256), and `crates/mamut-workflow` to
`vendor/mamut-workflows/mamut-workflow` (SDK-WF-003, registry and compilation).
Exact inventory with hashes: `vendor/mamut-workflows/MANIFEST.sha256`.
All `src/**/*.rs` and `tests/**/*.rs` remain byte for byte identical to upstream.
No panic/unwrap/expect calls in the library or logic changes are added.

Only upstream files modified: the `Cargo.toml` of both crates.
The `.workspace` fields are replaced with version 0.1.0, edition 2021,
license Apache-2.0, rust-version 1.83 and repository
https://github.com/Mamut1195/mamut-workflows. `readme = "../README.md"`,
`publish = false` and a comment identifying the modification and NOTICE are set.
In this commit there are no dependencies between the two crates, so there are no
internal dependencies to convert to path. External dependencies unchanged.
No Clippy allows or rustfmt exclusions are needed.

Cargo.lock incorporates both local packages 0.1.0, `json-canon 0.1.3` and its
transitive dependency `ryu-js 0.2.2`, available in the local registry.
The existing `sha2 0.10.9`, `serde 1.0.229` and `serde_json 1.0.154` are reused;
no previously pinned version is updated.

New distribution files: `vendor/mamut-workflows/NOTICE`, `README.md`
and `MANIFEST.sha256`; `LICENSE` comes exactly from the upstream root.
Own integration: root `Cargo.toml` (both crates in members/default-members),
`Cargo.lock` updated offline, `scripts/check-vendor.sh` and this log.
The script verifies the complete inventory and SHA-256, excluding only its own
manifest.

## Jonathan's decision (2026-10-14)

Jonathan authorizes and merges option (a): exclude exclusively the canonical
paths under `vendor/mamut-workflows/` from the identifier scan in
`clean-room/check.sh`. The prohibited-path check still applies.
If the diff touches vendor, the validator's trusted code checks inline that the
manifest is present and nonempty, the inventory is complete (including ignored extras),
SHA-256 is strict, commit `585f32dc3455208566a0ffb894ad03ce3816b888` is in README
or NOTICE and LICENSE is present. It rejects symlinks and noncanonical paths, without running
PR scripts. `scripts/check-vendor.sh` is retained as an additional control.

Tests of the exception and its limits are added in `clean-room/tests/run.sh`.
`/vendor/mamut-workflows/` is assigned to @Mamut1195 in `.github/CODEOWNERS`,
the same owners as `clean-room/**`, for review of joint changes
to files and manifest. Approval and merge are Jonathan's responsibility.
No workflows or `docs/CLEAN_ROOM.md` are modified.

In the first update, `vendor/mamut-workflows/README.md`, listed in the
manifest, was modified, and `MANIFEST.sha256` regenerated: only the README
hash changed. The other listed files, including all vendored code,
remained intact in that update.

## Source change by J0/Jonathan's decision (2026-10-14)

History: the initial source was `5c4e75af21564edc1a3b900d565a2ab90447ac1c`
(tag `v0.1.0`); J0/Jonathan later ordered it pinned to
`585f32dc3455208566a0ffb894ad03ce3816b888` (`main`), although the declared version
remains `0.1.0`. The previous tag emits `jcs-rfc8785` instead of
`rfc8785-jcs`, required by the contract, and its manifest does not comply with
`execution-manifest/1`. The new commit corrects both issues upstream.
The manifest emitter corrections are outside the two vendored
crates: the runner is not incorporated and scope is not extended.

All files in `crates/mamut-canon`,
`crates/mamut-workflow` and LICENSE were compared through `git show 585f32d:<ruta>`.
Only `crates/mamut-canon/src/lib.rs` differs: it is copied verbatim, including the
new fingerprint contract test. LICENSE and the other Rust files
remain identical. The upstream Cargo.toml files did not change and retain
their distribution adjustments documented above. Cargo.lock does not change.

README, NOTICE and this log are updated; MANIFEST.sha256 is regenerated
(the hashes of README, NOTICE and mamut-canon/src/lib.rs change).
The inline check now requires the new full hash. Its tests
include rejection of the previous commit and the abbreviated hash, even with
valid inventory and SHA-256.

MAM-490: do not patch vendored tests that assume `../../target` and fail
with `CARGO_TARGET_DIR` exported. That assumption was not found in these two crates;
the result of running tests with the directory exported is recorded
in the verification of the new source at the end of this log.

## Initial vendoring verification (historical, 5c4e75a)

The inherited CARGO_TARGET_DIR
`/workspace/.cargo-target/mamut-engine`, `CARGO_NET_OFFLINE=true`
and the local Rust 1.99.0 toolchain (`PATH=/home/box/.cargo/bin:$PATH`) are used.
No network queries or modification of the source repository.

- `cargo fmt --all -- --check`: OK, without exceptions.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 491 tests, 0 failed/ignored;
  17 from the vendored crates (canon: 6, workflow: 6, two_step: 5).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK,
  including both vendored crates.
- `cargo metadata --offline --locked --no-deps --format-version 1`: OK;
  both crates in members/default-members, publish disabled and local README.
- Byte-for-byte comparison of the 10 Rust files and LICENSE with upstream: OK;
  source checkout remains clean at the indicated commit.
- `bash scripts/check-vendor.sh` and `bash -n scripts/check-vendor.sh`: OK.
  In a temporary copy, the check rejects an altered file, an added file
  and a deleted file; the intact copy passes before and after those simulations.
- `git diff --cached --check`: flags only a final blank line in
  `vendor/mamut-workflows/LICENSE:202`, inherited from upstream; it is retained
  to comply with the exact license copy.
- The previous blocker due to the four occurrences of `algorithm` in the
  `mamut-canon` upstream code is resolved by option (a) authorized above.
  Human review PENDING (Jonathan).

## Previous MAM-481 verification (2026-10-14, historical, 5c4e75a)

With `CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine` exported,
`CARGO_NET_OFFLINE=true` and the same local toolchain, without network:

- `bash clean-room/tests/run.sh`: OK, 35 tests, 0 failures. Covers integrity,
  metadata, symlinks, complete removal, invalid manifests, escapes,
  untrusted scripts and the identifier exclusion limits.
- `bash clean-room/check.sh origin/main`: OK, including inline hashes.
- `bash scripts/check-vendor.sh`: OK; only the README hash changes.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash -n clean-room/check.sh clean-room/tests/run.sh` and `git diff --check`: OK.

## Verification of new source 585f32d (2026-10-14)

Source: `585f32dc3455208566a0ffb894ad03ce3816b888`.
`CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine`,
`CARGO_NET_OFFLINE=true` and `PATH=/home/box/.cargo/bin:$PATH` are exported.
Local toolchain: cargo 1.99.0 / rustc 1.99.0. All checks without network:

- `bash clean-room/tests/run.sh`: OK, 37 tests, 0 failures; includes rejection
  of the previous commit and the abbreviated hash.
- `bash clean-room/check.sh origin/main`: OK, including inline control.
- `bash scripts/check-vendor.sh`: OK, complete inventory and hashes.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 526 tests including documentation
  tests, 0 failed/ignored; 18 from the vendored crates
  (canon: 7, workflow: 6, two_step: 5), including the new fingerprint test.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash -n clean-room/check.sh clean-room/tests/run.sh scripts/check-vendor.sh`
  and `git diff --check`: OK.
- Comparison of the 13 upstream files (10 Rust, 2 Cargo.toml, LICENSE):
  OK against `git show 585f32d:<ruta>`, except the documented distribution
  adjustments to the two Cargo.toml. LICENSE identical; Cargo.lock unchanged.
- Upstream checkout clean and unchanged, still at
  `5c4e75af21564edc1a3b900d565a2ab90447ac1c`; `main` resolves to the new source.
- MAM-490 not reproduced with `CARGO_TARGET_DIR` exported. The two included
  crates do not assume `../../target`; no vendored test was patched.

## Re-pinning to 4cb9ecd (2026-10-14, J0)

Current source: `4cb9ecd0e1117d86444149b17a6b34d1283d9455` (mamut-workflows
main, PR #5: CI on VPS and MAM-490 correction). J0 indicated that the
library code is the same as in 585f32d; this was checked:
`git diff --stat 585f32d 4cb9ecd -- crates/mamut-canon crates/mamut-workflow LICENSE`
empty, and byte-for-byte `cmp` of each vendored file (except adapted `Cargo.toml`
files) and LICENSE against `git show 4cb9ecd:<ruta>`: identical. Only
README.md, NOTICE (provenance), MANIFEST.sha256 (due to those two) and the
commit required in `clean-room/check.sh` and its tests change. Documentation editing done
by J2 (without Codex: no new code).

## FU J4 r28 (2026-10-14)

MAM-481 · ENG-VND-001-FU, branch `j2/vnd-001-fu`, follow-up to PR #100.
Change in `clean-room/**` authorized; Jonathan reviews and merges.
This section replaces the broad exclusion documented in the previous historical
decisions. No `.github/**` or `docs/CLEAN_ROOM.md` are modified.

- Own NOTICE: the prohibited identifier `fix` is replaced with
  `correction`. Own README is reviewed and updated without introducing
  prohibited identifiers, and MANIFEST.sha256 is regenerated: only
  NOTICE and README hashes change. No upstream file is modified.
  **Note (2026-10-15):** these NOTICE/README edits and MANIFEST
  regeneration were discarded when integrating #127 and never entered `main`;
  `vendor/` remains exactly as in #127 and the exception policy is
  documented only in this log (see «Integration of #110 and #127»).
- `clean-room/check.sh` maintains an explicit list with a single pair:
  `vendor/mamut-workflows/mamut-canon/src/lib.rs:algorithm`.
  No other identifiers are allowed in that file or that identifier
  in other paths. The prohibited-path control is retained.
- If the diff touches the distribution, its integrity is verified inline and then
  all its physical files and all their lines are scanned, including unchanged
  inherited lines, final lines without a newline and ignored or
  untracked files present in the manifest. Inventory,
  strict SHA-256, rejection of symlinks and noncanonical paths, LICENSE and full commit
  `4cb9ecd0e1117d86444149b17a6b34d1283d9455` are retained, without running PR scripts.
- Complete independent scan with the current list: five occurrences of
  `algorithm`, all in the permitted path; zero additional findings after
  correcting NOTICE. The exception is not extended for other upstream names.
- Comparison against `git show 4cb9ecd:<ruta>` in the read-only local
  source: the 10 Rust files and LICENSE remain byte-for-byte identical.
  The two adapted Cargo.toml files and Cargo.lock do not change either.

Verification without network with `CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine`
exported, `CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH`
and cargo/rustc 1.99.0:

- `bash clean-room/tests/run.sh`: OK, 55 tests, 0 failures; includes all
  previous ones, the permitted pair, rejections in other vendor paths,
  another identifier in the permitted path, NOTICE/README and content outside
  the diff (also ignored or untracked with valid hashes).
- `bash clean-room/check.sh origin/main`: OK, full vendor scan
  and inline verification of the manifest and commit.
- `bash scripts/check-vendor.sh`: OK, complete inventory and hashes.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace --no-fail-fast`: OK, 538 tests,
  including documentation tests, 0 failed/ignored.
- `RUSTDOCFLAGS='-D warnings' cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash -n clean-room/check.sh clean-room/tests/run.sh scripts/check-vendor.sh`
  and `git diff --check`: OK.


## Re-vendoring to 1eeb91a · MAM-481 / MAM-493 (2026-10-15)

- Date: 2026-10-15 (America/Guayaquil).
- Author: Codex gpt-6.1-sol high. Role: implementer.
- Branch: `j2/eng-vnd-001-wf-1eeb91a`, already created and clean at the start,
  from `c290b10984b9f7d929e4bca4c17e6481fe4a670a` (origin/main).
- Source: `1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`, mamut-workflows main
  commit, PR #6 and #7; SDK-WF-012 / MAM-493, CLI extension points
  required by SFC-002. This is the current source of the distribution.
- History: `5c4e75a → 585f32d → 4cb9ecd → 1eeb91a`; full SHAs and
  previous decisions are retained in historical entries and README/NOTICE.
- Local source `/workspace/mamut-workflows`, read-only. Each copy is obtained
  **always** from `git -C /workspace/mamut-workflows show 1eeb91a:<ruta>`;
  the working tree is not used. Upstream status clean before and after.
- License: Apache-2.0; LICENSE identical to the blob from the pinned commit.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- No network, no commit or push in the Engine. No changes to `crates/`, bindings
  or `.github/`. Clean-room/vendor owner review: PENDING (Jonathan).

`mamut-canon`, `mamut-workflow`,
`mamut-workflow-execution` and `mamut-workflow-cli` are included in full, with all their files,
own README, tests and snapshots. All four are members/default-members.
`mamut-workflow-run` (binary/facade) and
`mamut-tool-conformance` (conformance tool) are deliberately excluded, outside this
library scope and not dependencies of the four included crates.

Only modifications relative to upstream: the four Cargo.toml. Literal
version 0.1.0, edition 2021, license Apache-2.0, rust-version 1.83 and
repository https://github.com/Mamut1195/mamut-workflows are used;
readme="../README.md", publish=false and a modification comment are set.
Internal dependencies are already paths between siblings, unchanged.
The parsed TOML of each manifest was compared against upstream, replacing
only those fields: all other fields and dependencies are identical.
Cargo.lock was updated through `cargo check --workspace --offline`:
it only adds the two local packages, without changing external dependencies.
All necessary cache entries were available.

No tests are excluded and no Rust is patched. CLI `tests/extensions.rs` uses
literal paths to `examples/open-local/{catalog,workflow}.json` from the upstream
root. Both blobs are copied to `vendor/mamut-workflows/examples/open-local/`,
covered by the manifest, and to `vendor/examples/open-local/`, where the original
paths resolve. Both verifiers compare those copies with the blobs
covered by the anchor and reject links in their paths. Changes only in
`vendor/examples/` also trigger verify_workflows; the exception to the identifier
scan is not extended. The mamut-workflow binary is not needed.
MAM-490 not reproduced: the new tests use CARGO_TARGET_TMPDIR, and are
verified with CARGO_TARGET_DIR outside the tree.

README/NOTICE and the complete inventory (46 files) are updated, as well as
scripts/check-vendor.sh, which requires full PINNED_COMMIT in both documents.
J4 r31: clean-room/check.sh pins the SHA-256 of MANIFEST.sha256 itself in
EXPECTED_WORKFLOWS_MANIFEST_SHA256:
`214b71529137733aa14babaefa3eac67365f94bd986407ea7e7218bf68174c68`.
Physical inventory, absence of links, canonical paths, strict
hashes, full commit and LICENSE are retained. The anchor must be reviewed in the
trusted base validator; the local result of the working-tree script does not constitute
approval of the change by the owner.

Clean-room tests use synthetic distributions with a base verifier
that pins their initial manifest, without updating it after mutations. An
altered payload with a regenerated manifest and consistent hashes is checked
to fail. Previous/abbreviated/missing commit cases explicitly pin the
corresponding synthetic manifest to also check metadata rejection
independently. Cases of altered external fixture,
external-only change and linked directory are added. Inventory,
path, link, license and untrusted script controls continue to pass.

### Verbatim comparison, file by file

Each blob is obtained with `git -C /workspace/mamut-workflows show 1eeb91a:<ruta>`
into a temporary file under /tmp and `cmp -- <temporal> <destino>` is run.
The table records the actual exit code of each comparison: **42 identical
files**, including the two duplicated fixtures. Each crate's inventory
was also compared against `git ls-tree -r --name-only 1eeb91a`: no
omitted or extra files. The four Cargo.toml files are the only exceptions
verified above; no Rust file was reformatted.

| Upstream path in 1eeb91a | Destination in the Engine | cmp |
| --- | --- | --- |
| `crates/mamut-canon/src/lib.rs` | `vendor/mamut-workflows/mamut-canon/src/lib.rs` | 0 |
| `crates/mamut-workflow/src/catalog.rs` | `vendor/mamut-workflows/mamut-workflow/src/catalog.rs` | 0 |
| `crates/mamut-workflow/src/compiler.rs` | `vendor/mamut-workflows/mamut-workflow/src/compiler.rs` | 0 |
| `crates/mamut-workflow/src/contract.rs` | `vendor/mamut-workflows/mamut-workflow/src/contract.rs` | 0 |
| `crates/mamut-workflow/src/descriptor.rs` | `vendor/mamut-workflows/mamut-workflow/src/descriptor.rs` | 0 |
| `crates/mamut-workflow/src/error.rs` | `vendor/mamut-workflows/mamut-workflow/src/error.rs` | 0 |
| `crates/mamut-workflow/src/lib.rs` | `vendor/mamut-workflows/mamut-workflow/src/lib.rs` | 0 |
| `crates/mamut-workflow/src/registry.rs` | `vendor/mamut-workflows/mamut-workflow/src/registry.rs` | 0 |
| `crates/mamut-workflow/src/workflow.rs` | `vendor/mamut-workflows/mamut-workflow/src/workflow.rs` | 0 |
| `crates/mamut-workflow/tests/two_step.rs` | `vendor/mamut-workflows/mamut-workflow/tests/two_step.rs` | 0 |
| `crates/mamut-workflow-execution/src/cli_run.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/cli_run.rs` | 0 |
| `crates/mamut-workflow-execution/src/executor.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/executor.rs` | 0 |
| `crates/mamut-workflow-execution/src/exit_codes.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/exit_codes.rs` | 0 |
| `crates/mamut-workflow-execution/src/lib.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/lib.rs` | 0 |
| `crates/mamut-workflow-execution/src/manifest.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/manifest.rs` | 0 |
| `crates/mamut-workflow-execution/src/mcp.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/mcp.rs` | 0 |
| `crates/mamut-workflow-execution/src/method.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/method.rs` | 0 |
| `crates/mamut-workflow-execution/src/observe.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/observe.rs` | 0 |
| `crates/mamut-workflow-execution/src/registered.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/registered.rs` | 0 |
| `crates/mamut-workflow-execution/src/tests.rs` | `vendor/mamut-workflows/mamut-workflow-execution/src/tests.rs` | 0 |
| `crates/mamut-workflow-cli/README.md` | `vendor/mamut-workflows/mamut-workflow-cli/README.md` | 0 |
| `crates/mamut-workflow-cli/src/config.rs` | `vendor/mamut-workflows/mamut-workflow-cli/src/config.rs` | 0 |
| `crates/mamut-workflow-cli/src/lib.rs` | `vendor/mamut-workflows/mamut-workflow-cli/src/lib.rs` | 0 |
| `crates/mamut-workflow-cli/src/parser.rs` | `vendor/mamut-workflows/mamut-workflow-cli/src/parser.rs` | 0 |
| `crates/mamut-workflow-cli/tests/extensions.rs` | `vendor/mamut-workflows/mamut-workflow-cli/tests/extensions.rs` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/compile.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/compile.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/describe.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/describe.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/help.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/help.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/methods.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/methods.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/missing.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/missing.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-document.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-document.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-method.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-method.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-usage.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-usage.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run-validation.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run-validation.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/run.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/unknown.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/unknown.txt` | 0 |
| `crates/mamut-workflow-cli/tests/fixtures/validate.txt` | `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/validate.txt` | 0 |
| `LICENSE` | `vendor/mamut-workflows/LICENSE` | 0 |
| `examples/open-local/catalog.json` | `vendor/mamut-workflows/examples/open-local/catalog.json` | 0 |
| `examples/open-local/catalog.json` | `vendor/examples/open-local/catalog.json` | 0 |
| `examples/open-local/workflow.json` | `vendor/mamut-workflows/examples/open-local/workflow.json` | 0 |
| `examples/open-local/workflow.json` | `vendor/examples/open-local/workflow.json` | 0 |

### Offline verification results

Local Rust/cargo toolchain 1.99.0; PATH=/home/box/.cargo/bin:$PATH,
CARGO_NET_OFFLINE=true, CARGO_TARGET_DIR=/tmp/eng-vnd2-target.
The wasm32-unknown-unknown target is installed. All Cargo commands that
resolve dependencies are run with --offline; no downloads or missing
dependencies. Session logs: /tmp/eng-vnd2-*.log (not incorporated into the repo).

- `cargo fmt --check` and `cargo fmt --all -- --check`: OK. No exclusions.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 597 tests, 0 failed/ignored,
  including doctests. Vendor: canon 7, workflow 6 + two_step 5, execution 10,
  CLI 1 + extensions 16 + doctest 1 (46 in total).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `bash scripts/check-vendor.sh`: OK, inventory of 46 files, hashes,
  commit and external fixture copies.
- `bash clean-room/tests/run.sh`: OK, 41 tests, 0 failures.
- `bash clean-room/check.sh origin/main`: OK. Run expressly with
  check.sh from the modified tree, against origin/main=c290b10, as requested;
  not with the previous validator version obtained from the base.
- `cargo check --workspace --target wasm32-unknown-unknown --offline`: OK.
- `cargo metadata --offline --locked --no-deps --format-version 1`: OK;
  four members/default-members, publish=false, metadata and local readme.
- Determinism: `cargo build --offline -p engine-api --example determinism_wasm
  --target wasm32-unknown-unknown`, `cargo run --offline -p engine-api
  --example determinism_native` and `node scripts/check-determinism.mjs
  /tmp/eng-vnd2-target/wasm32-unknown-unknown/debug/examples/determinism_wasm.wasm
  /tmp/eng-vnd2-native.txt`: OK, 16 cases, ten runs per case,
  identical SHA-256 between native and wasm (debug profile).
- `bash -n clean-room/check.sh clean-room/tests/run.sh scripts/check-vendor.sh`
  and `git diff --check`: OK.


## Integration of #110 and #127 · J4 r28 / r31 (2026-10-15)

The current distribution is that of #127, pinned to
`1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d`. No file in `vendor/` is modified
in this integration, including NOTICE, README and MANIFEST.sha256.
References to a single pair and to changes to own texts in the previous
FU J4 r28 entry describe the historical distribution of 2026-10-14.

The exception policy that #110 documented in NOTICE/README is moved here:
`clean-room/check.sh` scans all physical files and all lines of
the distribution when the diff touches it, including unchanged inherited
lines, final lines without a newline and ignored or untracked files.
Before scanning, complete inventory, strict SHA-256, absence of
links, canonical paths, LICENSE, full commit and the
`EXPECTED_WORKFLOWS_MANIFEST_SHA256` anchor from #127 are verified inline:
`214b71529137733aa14babaefa3eac67365f94bd986407ea7e7218bf68174c68`.
Checking of external examples and
`scripts/check-vendor.sh` are also retained; PR scripts are never run from the trusted
validator. Prohibited paths continue to be checked.

The only exception is these **4 literal path:identifier pairs**:

- `vendor/mamut-workflows/mamut-canon/src/lib.rs:algorithm`
- `vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run.txt:algorithm`
- `vendor/mamut-workflows/mamut-workflow-execution/src/mcp.rs:algorithm`
- `vendor/mamut-workflows/mamut-workflow-execution/src/tests.rs:algorithm`

The name corresponds to the fingerprints' SHA-256 algorithm, its tests and
its CLI snapshot. No other identifiers are allowed in those paths or that
identifier in other paths; prefixes or globs are not accepted. A scan with
`rg --hidden --no-ignore --text` and ASCII identifier boundaries, using the full
`clean-room/forbidden_identifiers.txt` list over `vendor/`, confirms that
only `algorithm` appears, exactly in those four paths. There are no additional
findings and the list is not extended for other upstream names.

Attribution and source history remain in the #127 NOTICE/README.
`CODEOWNERS` keeps the same owner for the distribution and `clean-room/**`
(@Mamut1195, Jonathan); human review and merge remain pending.

Verification of this integration, without network, with cargo/rustc 1.99.0,
`PATH=/home/box/.cargo/bin:$PATH`, `CARGO_NET_OFFLINE=true` and
`CARGO_TARGET_DIR=/tmp/eng-vndfu-target`:

- `bash clean-room/tests/run.sh`: OK, 73 tests, 0 failures. Retains the #127
  cases (examples, commit and anchor), and incorporates the four exact pairs,
  rejections for other files, prefixes and globs, other identifiers in the
  four paths and unchanged content, without final newline, ignored or untracked.
  Identifier negatives require their diagnostic and valid integrity;
  the anchor is not updated after the r31 attack case mutations.
- `bash clean-room/check.sh origin/main`: OK, with the working-tree script.
- `bash scripts/check-vendor.sh`: OK.
- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: OK, 610 tests including doctests,
  0 failed/ignored.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `git diff MERGE_HEAD -- vendor`: empty; entire #127 distribution.
- `verify_workflows` and the anchor were compared against #127: identical.
  Both historical provenance entries were retained in full.

Cargo and verifier logs in `/tmp/eng-vndfu-*.log`, outside the repository.
The merge was committed in `9c985da` (push without force to `j2/vnd-001-fu`).
