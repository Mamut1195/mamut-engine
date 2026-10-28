# MAM-864 / ENG-SFC-013 · CLI self-description for agents

Date: **2026-10-22**. Author: Codex, model **gpt-6.1-sol high**, implementer.
Working branch based on `origin/main`, with the additions for input
transport, direct validation, timeout, signals and schema publication
already present.

## Sources and scope

Only local code and documentation were consulted: the public
`engine-api` facade (`version`, `capabilities`, `schema_kinds`, `schema`, `explain` and its
public constants), the `engine-cli` adapter, the parser and dispatch of the vendored
base, `ExitCodes::engine_v1`, schemas and their publisher, existing
tests and changelog and determinism conventions.

No network or new dependencies. No changes to vendor, Cargo, CI
configuration, protection files, clean-room rules, axial bar or truss. No
access to restricted provenance, audit directories, external reference
sources or logs from other sessions. No changes to run computation or serialization.
No `git add` or commit is performed. Cargo, Rust and Python temporary files
and verification logs are placed in the configured build
directory, with the environment's temporary directory pointing to a subfolder
of it.

## Decisions and contracts

The logic is concentrated in `crates/engine-cli/src/describe.rs`; `lib.rs` only
registers the module and invokes it after validating the timeout and before preparing
input/workflows. The new queries do not open files, read stdin, create
temporary workflows, install signals or start timers. They preserve the
existing timeout validation and flag/environment precedence.

- `describe --json`: `cli-describe/1` document with real identity, commands,
  aliases, arguments, flags, empty subcommands because there are no nested
  commands, environment, inputs/outputs, effective exit table and public limits.
- `--help --json`, `help --json`, `-h --json`: bytes identical to the description.
- `--version`: text `mamut-engine <engine_version>` and newline.
  `--version --json` serializes `engine_api::version()` without added fields;
  its external contract is `version/1`. No `version` subcommand is added.
- `capabilities`: directly serializes the API document and reuses
  capabilities/1. It does not accept `--json` because its only output is JSON.
- `schema <id>`: `write_all` of the original `engine_api::schema` bytes,
  without reserializing or adding a newline. It accepts exactly the API's ids.
- `explain <code>`: original explanation plus newline; with `--json`, exactly
  the `{code, explanation}` object, explain/1 contract.

Success: 0. Unknown id/code, missing or extra argument: 2, empty stdout and
a `cli-error/1` envelope on stderr with `CLI-E-USAGE`, registered by main.
Write failures: internal category, 70 and `CLI-E-INTERNAL`, also registered,
with the possibility of incomplete stdout as in existing commands.

The base already offers `describe` with a different document: **it is preserved unchanged
when it does not carry `--json`**, including its historical reserved timeout
metadata. The new version advertises effective timeout/6 and preserves the categories
that share a code through `exitCodeCategories`. The base does not offer the other
new queries. Text help already takes `mamut-engine` from
`CliConfig::with_identity`; nested help is also checked and no
replacement or vendor editing is required. These decisions are recorded in the English README,
following the language of its section in main.

The schema inventory uses `schema_kinds()` and `schema()` from the facade and the
published index embedded through `include_str!`. Each entry advertises contract
id, `$id`, SHA-256 and `availableViaSchema`. Fingerprints are taken from the index,
which is checked against the original bytes through tests and the publisher;
no cryptographic implementation or dependency is added to the adapter.
The inventory includes main's `cli-error/1` and the output list advertises it
as the terminal stderr contract. Additional CLI schemas are advertised
as static publication and do not expand the ids accepted by the API. Changing the
publication requires regenerating the index and recompiling the binary.

The three new schemas are stored in `crates/engine-cli/schemas/`, registered
in `scripts/schema-sources.json` and published with
`python3 scripts/generate-schemas.py --write`. No byte of the previous
API schemas or their publication is changed.

Merge: changes from `origin/main` (#205 and #183) are integrated, preserving the
`cli-error/1` emitter, its code registry and its publication alongside the three
MAM-864 schemas; the merge remains uncommitted.

## Tests and verification

Seven new binary tests check contracts, determinism and a versioned
SHA-256 snapshot of the entire describe stdout, JSON help equivalence,
equality with the API's version/capabilities/explain, inventory and fingerprints against
each published file, all API schemas byte for byte, text help,
0/2 codes, invalid arguments and timeout precedence/validation. The
snapshot avoids copying full outputs into source files. The validator is the
existing one in `engine-api/tests/support/schema.rs`; for capabilities,
its external reference to the real method-descriptor schema is resolved in memory, without network.
SHA-256 in the tests uses Python's standard library, already required by
publication. Negative mutations check required and additional fields
and nested types. Only the two rejection expectations for commands
that are now implemented are removed from an earlier test.

An eighth test checks write failures of all queries,
including JSON variants, against the `cli-error/1` schema and main's code
registry. Argument rejections also use that validator and
check `CLI-E-USAGE` and the sanitized operation. The describe snapshot is
updated to include `cli-error/1` as a published schema and terminal output.

Environment: Cargo offline, locally available Rust tools and temporary files
in the configured build directory.

Rust/Cargo **1.99.0**, native Linux platform. Final results:

| Check | Result |
| --- | --- |
| `cargo fmt --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 821 passed / 0 failed / 0 ignored |
| `python3 scripts/generate-schemas.py --check` | OK; bytes, index and inventory |
| `python3 -m unittest scripts/tests/test_schema_publication.py` | 7 tests, OK |
| engine-cli tests | 12 unit + 40 CLI + 8 queries + 7 registry + 1 error schema, OK |
| Prohibited identifiers in new text | No matches; text inherited from main is preserved |
| `git diff --check` | OK |

Existing run tests preserve byte, manifest and result
comparisons, including stdin/path, embedded/explicit workflows, timeout and signals.
External human clean-room review: **PENDING (J4)**.
