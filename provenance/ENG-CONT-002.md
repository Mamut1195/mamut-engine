# ENG-CONT-002 · MAM-868 · cli-run/1 success/warning contract

Date: **2026-10-21**. Local work without network in
the local worktree, branch `j2/MAM-868-cli-run-contrato`.
Observed base: HEAD and `origin/main` at
`7375fa25a34a3cb004bcaec84f645743b864ff89`. Remote state is not consulted.

## Contract and decisions

[cli-run-1.json](../crates/engine-cli/schemas/cli-run-1.json) is published,
contract version **1.0.0**, in `crates/engine-cli/schemas/`, following the
convention indicated for MAM-867 / PR #205. The local base does not contain that error
schema; neither its implementation nor any failure envelope is modified.
`docs/contracts/` did not exist; the policy is in the
[CLI README](../crates/engine-cli/README.md#contrato-publicado-cli-run1--100).

The stdout envelope for `run` preserves exactly the bytes produced by the
adapter. No `schema` field is added and `src/` is not modified. `type:result`,
identity, `run` operation, outcome/code (`ok`/0, `warning`/1), `data` and
`engineResults` are required. The envelope and validate report are strict.
`engineResults` publishes one complete result per step, in order: results/1 or
`{operation:"validate",status,diagnostics}`. The schema relates outcome and
code; `ok` requires ok results and `warning` at least one warning result.

The output decision is settled: **engineResults is the adapter's contract**.
A standard channel for complete results remains SDK work. The vendored SDK,
input transport and MCP decision are not modified.
Major changes are not permitted during V1. Compatible additions require
a minor version; clarifications without observable changes, a patch. `engineVersion`
still identifies the engine, without duplicating the contract version in stdout.

`data` and its nested structures belong to `mamut-workflow-execution`.
Its vendored producer was read solely to check fields, states and
preimages: the adapter preserves the manifest, which supports SDK extensions.
`recordFingerprint` is always present in this producer, covers data without
that field and does not cover the envelope. Each step's fingerprints cover its result without the `cause` and `hint` fields of
the diagnostics (MAM-836: the fingerprint covers codes and data, not message text).
No second manifest contract is created.
The schema only requires object type for `recordFingerprint` and nested
fingerprints: their keys belong to the SDK and are not enumerated. The adapter
parts retain their strict restrictions.

The `$ref` use the existing `urn:mamut:results:1` and its `diagnostic` definition.
The engine-api test validator is extracted to
[tests/support/schema.rs](../crates/engine-api/tests/support/schema.rs), shared
with the CLI. It preserves its rules and adds only explicit local resolution
of results/1 and `contains` for warning aggregation. It fails on unknown keywords,
patterns or references, without network or new packages. It is a
validator for the subset used, not a general JSON Schema engine.

## Coverage and bytes

[tests/contract.rs](../crates/engine-cli/tests/contract.rs) discovers the eleven
determinism fixtures and the sixteen valid engine-api JSONs (including
the eleven SFC-010). It extracts only `runs[*].engine_input` from versioned E1;
it does not use expected values or external numerical references. It adds the combination
warning already present in CLI tests. There are **29 inputs × 2 workflows**
(engine.run and engine.validate): **58 goldens**, **55 codes 0 and 3 codes 1**.
Each case runs the real binary twice, validates stdout against the schema,
checks empty stderr, code/outcome and step/result count, and compares
the length and SHA-256 of all bytes. The goldens also freeze the current order and
structure.

[tests/golden/stdout-sha256.json](../crates/engine-cli/tests/golden/stdout-sha256.json)
is the only versioned golden: a version 1 manifest with name, byte length
and SHA-256 per case, plus the base build provenance. It contains no
stdout and no `.stdout.json` files are retained. The hashes cover compact UTF-8
with exactly one final LF, without CR. The binary already uses `writeln!` and emits LF
on Windows too; that behavior is unchanged. The test rejects CRLF and
missing LF; it does not normalize stdout or depend on locale or execution paths.
The base was generated on Linux x86_64 with `git_commit:null`. Raw byte hashing
uses Python 3's `hashlib`, available in the parity environment, without new
Cargo dependencies. On discrepancies, expected and obtained hashes and lengths
are printed and actual and comparable stdout are saved under
`env!("CARGO_TARGET_TMPDIR")/cli-contract-failure-<pid>/`.

The [DETERMINISMO.md](../docs/DETERMINISMO.md) policy includes `target` and
`git_commit` in results: they cannot be compared literally across different
builds. For another build, stdout is brought to the manifest's provenance by literally replacing
those two metadata fields and dependent fingerprints (step result and
manifest). Step preimages are the results without `cause` or `hint` in
their diagnostics (MAM-836). Preimages of actual stdout are verified first and recalculated
through JCS/SHA-256 using the existing public API. Metadata is obtained
from compiled engine-api, independently of the stdout under test. Fields are not
projected and actual stdout is not reserialized; numbers, keys, arrays,
spaces and diagnostics remain frozen. A round-trip test checks
the substitution for macOS x86_64/arm64 and Windows MSVC triples, including a
non-null build revision. It is a materializer test, not an
execution of the binary on those systems.

Negative tests cover required fields, identity, engineResults type,
results version/status, numeric vector, fingerprint types, diagnostics, warning
correspondence and rejection of adapter extensions; SDK extensions are accepted.

Explicit local regeneration:

```sh
export PATH=$HOME/.cargo/bin:$PATH
export CARGO_TARGET_DIR=<directorio-target-del-worktree>
# Linux; MAMUT_ENGINE_GIT_COMMIT ausente al compilar para conservar la base.
MAMUT_UPDATE_CLI_GOLDENS=1 cargo test -p engine-cli --test contract --offline \
  all_accepted_fixtures_match_schema_and_stdout_golden
cargo test -p engine-cli --test contract --offline
```

Without the variable no golden is written. Only `1` or absence is accepted;
regenerating with `CI` or `GITHUB_ACTIONS` defined fails before writing.
Missing or stale goldens are rejected. The diff is always reviewed.

ENG-PY-006, MAM-86, parity and conformance were searched for in crates and bindings/python.
The existing Rust/Python byte parity test is in
[test_run.py](../bindings/python/tests/test_run.py),
`RunTest.test_cantilever_matches_rust_bytes_and_sha256`. The CLI goldens are linked
there and from the [Python README](../bindings/python/README.md).
The binding's direct Rust reference is preserved. The Python wheel suite is not run
for a comment and link change; the Rust workspace is run.

## Local verification and limits

Environment: Rust 1.99.0, offline Cargo,
`CARGO_TARGET_DIR=<directorio-target-del-worktree>`, Linux x86_64.

- `cargo fmt --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test -p engine-cli --test contract --offline`: **3 tests OK**.
- `cargo test --workspace --offline`: **OK, 752 tests passed in 116 suites,
  no failures or ignored tests**, including the three new contract tests.
- Regeneration guard with `CI=1`: expected rejection and manifest with its
  **58 cases unchanged**, checked byte by byte before/after.
- Temporarily injected hash discrepancy: expected and obtained hashes
  are verified in the failure and diagnostic bytes under `CARGO_TARGET_TMPDIR`.
  The manifest is restored byte by byte and the contract passes again.
- `git diff --check`: OK.
- `bash clean-room/check.sh origin/main`: **OK**, with new files
  included through `git add --intent-to-add` in a temporary index. The
  hash manifest and opaque SDK objects allow preserving the guard
  without changes or new exceptions. Complete new and modified files are also
  checked against the forbidden identifier list.

The real index and Git object database are read-only in this environment.
The execution indicated above also uses a temporary object directory in
a local temporary directory, with the original objects as read-only alternates.
Neither the original index nor objects are written to.

Actual execution on **macOS and Windows needs a matrix in `rust.yml`**,
**Jonathan's decision**. `.github/workflows` is not edited; this evidence only
supports Linux. No executed cross-platform CI is claimed.

No changes are made to vendor, Cargo manifests/lock, workflows, clean-room,
CLEAN_ROOM.md, CODEOWNERS, deny.toml, axial bar or truss. No restricted material
is opened and no results from other engines are incorporated. The clean-room script
is run without modification; human J4 review remains pending.
CHANGELOG.md is not edited: only
[changelog.d/MAM-868.added.md](../changelog.d/MAM-868.added.md) is added. No commit is made.


## origin/main merge · 2026-10-22

Author: **Codex gpt-6.1-sol high**. What main incorporated is preserved,
including #205 and #183, and #208 is integrated. The three conflicting files
have no markers remaining. The index remains intact: no commit, staging, rebase
or merge abort is performed.

The only validator implementation is main's in
`crates/engine-api/tests/support/schema.rs`; CLI helpers delegate to it.
It is extended with `contains`, known local results/1 references and
cli-error/1 patterns, preserving the stricter API diagnostic pattern.
`cli-run/1` is registered alongside `cli-error/1` in `additional_schemas`; the generator
publishes `schemas/cli-run/1.json` and updates `schemas/index.json`. run errors
retain cli-error/1 and its registry; no codes are added. The 58 goldens
still pass without regeneration or changes: only hashes, lengths and metadata.
`changelog.d/MAM-868.added.md` is preserved; CHANGELOG.md is not edited.

Offline verification with temporary files exclusively under `CARGO_TARGET_DIR`:
`cargo fmt --check`, `cargo clippy --workspace --all-targets --offline -- -D warnings`
and `cargo test --workspace --offline` green (**816 tests, 125 suites, no failures
or ignored tests**), including the error registry coverage test.
`cargo test -p engine-cli --offline --test contract`: **3 tests OK**, without the update
variable; the actual Cargo package name is `engine-cli`.
`python3 scripts/generate-schemas.py --write` and `--check` green;
`python3 -m unittest scripts/tests/test_schema_publication.py`: **7 tests OK**.
Pytest on `bindings/python/tests/test_run.py`: **9 tests and 36 subtests OK**,
with pytest and dependencies from the local cache, a copy of the Python package and native
extension compiled offline; the byte reference is generated with `facade_reference`.
This check does not build or verify a wheel. No network is used.
