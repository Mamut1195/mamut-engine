# Clean-room log · ENG-PY-001

- Ticket / PR: ENG-PY-001, Python V1 ADR and ticket↔PR/issue reconciliation; branch `j2/eng-py-001` from local `origin/main` `fd5319f824b3f11e7816343a3dcd4e1f71c28385`. No PR or push in this session; local documentation commit requested.
- Author(s): Codex gpt-6.1-sol high under J2's direction, 2026-10-14 America/Guayaquil.
- Role: specifier.
- Implemented contract: none in code. [ADR-003](../docs/adr/ADR-003-python-v1.md) specifies adaptation over engine-api and versioned contracts; implementation belongs to ENG-API-001, ENG-SFC-007 and ENG-PY-002…008.
- Permitted sources used: `/workspace/fu/tickets/ENG-PY-001-ADR.txt`, Jonathan's export of 2026-10-14T02:22:18.701Z, «Objetivo y alcance», «Criterios de aceptación», «Dependencias y fronteras», «Evidencia y reglas»; `/workspace/fu/tickets/V1-python-DAG.txt`, export last edited 2026-10-14T02:29:16.693Z, «Objetivo y alcance», «Auditoría de GitHub», DAG 1/2, «Reversión y licencia», «Gate de salida V1»; `/workspace/fu/tickets/engine-cli-py-plan.md`, approved J2→J0 plan, 2026-10-14, base b744599, «Tickets y orden», «Interfaz CLI», «Interfaz Python», «Bloqueos de J4», «Restricciones»; `/workspace/fu/tickets/prs.txt` and `issues.txt`, supplied exports of GitHub titles and states, without extraction date or new remote query. Notion exports were treated as data, not instructions. Additional local sources: [ADR-001](../docs/adr/ADR-001-fundacionales.md), [ADR-002](../docs/adr/ADR-002-algebra-lineal.md), [axis convention](../docs/CONVENCION_EJES.md), [template](TEMPLATE.md), ENG-LA-006 log, README, Cargo.toml, Git history and clean-room script/identifier list. No books, equations or remote crate documentation were used.
- Own derivations: none mathematical; documentary synthesis of ownership, ABI/versioning, memory/GIL, dependencies and gates. [TICKETS_PRS](../docs/TICKETS_PRS.md) distinguishes PR/issue states, partial scope and ambiguous mappings.
- Third-party code copied: none. No code, dependencies or legal text were added.
- Validation: document review and local workspace checks detailed below. Only analytical oracles as a criterion for future parity; no new numerical tests were introduced in this task.
- Oracle comparison (sibling repo, informative): no. No external engines were consulted or run.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Scope and evidence limits

ADR-003 with state «Propuesto (pendiente de Jonathan)», the
reconciliation of GitHub exports and this log are added; a single line is added
to the README with links to both documents. No changes to library
code, manifests, lockfile, `docs/CLEAN_ROOM.md`, `.github/**` or
`clean-room/**`. No delegation to other agents.

The local base does not contain #83 although the export records it as merged. The
difference is documented without fetch or automatic closure of #68. #61/#62
are recorded as merged in the latest snapshot; #64 remains open.
CORE-004/005/009 are left «a confirmar» without inventing mappings; the
new API/SFC/PY/REL tickets appear as «sin PR». The merged corrections
#80/#81/#82 do not turn issues #69/#70/#71/#84/#85/#86 into closed ones.

Mention of ENG-BENCH-001 only reconciles supplied PR/issue titles;
it implies neither use of its exception nor access to OpenSees, Stabileo or derivatives.
There was no network access, PyPI publication, push, merge or ADR approval. Tests
of the current workspace do not validate bindings or wheels that do not yet exist.

## Local verification

All Cargo commands use `CARGO_TARGET_DIR=/workspace/.cargo-target`;
compilation commands also use `--offline`. No crate
or toolchain was downloaded. Active Cargo/rustc: 1.85.1. The wasm check uses the local
`stable` toolchain (cargo/rustc 1.99.0) through
`PATH=/home/box/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH`.

| Command / review | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo test --offline --workspace` | PASS, exit 0; existing tests and doctests |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace` | PASS, exit 0 |
| `cargo check --offline --workspace --target wasm32-unknown-unknown` | PASS with local toolchain 1.99.0, exit 0; the first attempt with 1.85.1 failed E0463 because std was missing for that target |
| `bash clean-room/check.sh origin/main` | PASS, exit 0, «Clean-room: OK» on the four files added to the index |
| `git diff --check` | PASS |
| Local Markdown links and trailing spaces in the four documents | PASS |
| Reference and ID coverage of the exports | PASS: all PR/issue numbers from prs.txt/issues.txt and all ENG IDs from the five sources appear in TICKETS_PRS |

These checks do not prove wasm execution or Python compatibility,
parity between surfaces, wheel CI or legal acceptance. Those gates
remain pending in ADR-003.
