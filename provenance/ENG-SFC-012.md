# MAM-863 / ENG-SFC-012 · direct validate --input

Date: **2026-10-21**. Author: Codex (GPT-6), implementer.
Worktree: `/workspace/mamut-engine-sfc12`, branch stacked on MAM-862 (#197).

## Sources and scope

- Local code from `engine-cli`, `engine-api::validate/validate_with`, limits,
  DTO decoding, `run::validation`, model validator and diagnostic
  catalog; local determinism documentation and existing JSON contracts.
- For performance, exclusively `runs[0].engine_input` from
  `docs/validation-report/fixtures/val032/E1-v1.1.2.json`. The test deserializes
  structs with only `runs[].engine_input`, ignoring the other fields. It neither reads
  nor prints reference/oracle values; it measures the own API in process.
- No network, new dependencies or changes to Cargo, vendor, `.github`,
  `clean-room`, axial bar or truss. No access to restricted provenance,
  audit directories or logs from other sessions. No changes to arithmetic,
  solving, fixtures or bytes of the other schemas. No commit.

## Contract decisions

`validate --input <ruta|->`, without an explicit catalog/workflow, calls
`validate_with` directly. It creates no workflow, manifest, fingerprints or temporary files.
The cli-run/1 envelope of that shorthand from #197, still unmerged, is deliberately
replaced with a single deterministic `validate-report/1` JSON on stdout:
`schema`, `operation: "validate"`, `status`, `diagnostics`, with a final newline.
Minimal identity through schema and operation, without timestamps or local paths.

Statuses and codes: `ok`/0, `warning`/1, `invalid`/2. Any diagnostic with
error severity invalidates the model, without using the solution status mapping.
`InvalidInput`/`UnsupportedSchema` failures are mapped to `ENG-E-0009`
(incompatible contract), with the original cause and empty entities. Domain/extension
diagnostics are preserved in full and in order. This validates the
contract, conventions, limits and topology; it does not certify stability or
convergence and does not solve the system. A model without supports that validates is tested.

Transport errors preserve MAM-862 behavior: exceeding the
inclusive 8 MiB cap, invalid UTF-8, nonexistent paths or failed reads yield
2, empty stdout and the same sanitized cli-run/1 envelope on stderr as in #197;
the tests compare its bytes with the run path. Usage errors also yield 2.
Cooperative cancellation preserves 5 and failed output I/O, 70.
The optional `run --dry-run` alias is not implemented.

`validate --catalog … --workflow …` preserves the base dispatch exactly,
including with `--input`. The engine.validate method within a workflow preserves
its previous format. `run` and results/1 preserve their bytes and manifests; no changes
are made to `version().contracts`, methods or solution serialization.

New schema: `crates/engine-api/schemas/validate-report-1.json`, Draft 2020-12,
`$id: urn:mamut:validate-report:1`. Registered in `SCHEMAS` for
`engine_api::schema` and advertised in `capabilities().schemas`, as well as the
API README index. Local generation with `scripts/generate-schemas.py --write`
publishes its bytes in `schemas/validate-report/1.json` and its entry with SHA-256 in
`schemas/index.json`. The existing results/1 test validator is extracted
without functional changes to `tests/support/schema.rs` for reuse
from the binary tests, without adding another validator or dependencies.

## Evidence and limits

Binary tests: valid/0 and schema, JSON, internal MAX_NODES limits
and topology/2 errors with ENG-* codes, schema-compliant stdout and empty
stderr. Exact stdin/path comparison for success and errors; exact preservation
of explicit workflow validation. Previous run tests preserve
byte and manifest equality. Additional coverage of cancellation before and
between every stage, invalid flags and negative schema mutations at all
levels. The same validator continues checking results/1.

No fixture produces a warning during `validate_with`: the current validator
only generates errors and E1 receives the diaphragm warning when running run.
No artificial warning is added to the binary. The `warning`/1 branch is tested in the
classifier with a synthetic diagnostic, and the schema's warning form is also
checked. This is an explicit limitation of the binary's code 1 coverage
for model validation.

Final debug performance with Rust **1.99.0**, x86_64-unknown-linux-gnu, median of
**11** alternating repetitions after
warming up both paths, input extraction/serialization outside the
timer: **validate 5,440681 ms**, **run 151,456720 ms**, **27,84×**.
The test requires ratio ≥10 and remains active (without `ignore`); it is no guarantee
of absolute time on other machines. Offline reproduction:

```sh
export PATH=/home/box/.cargo/bin:$PATH
CARGO_TARGET_DIR=/workspace/.cargo-target/sfc12 cargo test -p engine-api \
  --test validate_performance --offline -- --nocapture
```

Final verification in this worktree with Rust/Cargo **1.99.0** (rustup PATH),
`CARGO_TARGET_DIR=/workspace/.cargo-target/sfc12` and Cargo offline. The initial PATH
selected system Rust/Cargo 1.85.1; checks were repeated with the
1.99.0 toolchain pinned in the repository.

| Check | Result |
| --- | --- |
| Targeted CLI, facade and results/1 tests | OK |
| `cargo fmt --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 764 passed / 0 failed / 0 ignored; CLI: 6 unit + 27 integration |
| `bash clean-room/check.sh origin/main` | OK |
| `git diff --check` | OK |

External human clean-room review: **PENDING (J4)**.
