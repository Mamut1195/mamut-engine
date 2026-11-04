# Clean-room log · ENG-TOML-003

This branch carries three increments. U9 and U5 are already on main. U11 adds the document position. None of them signs J4.

## Canonical export · U9

- Ticket / PR: docs/tickets/07-toml-export-canonico.md. Decision U9 in docs/V1.md section 13. Related specification ticket MAM-939. This note does not close MAM-107 and does not approve a time or memory budget.
- Author(s): implementation agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer.
- Contract implemented: canonical mamut-toml/1 export. Same model, same UTF-8 bytes. LF, one trailing newline, two-space indentation under each table header, root keys and headers at column 0, keys in the order of docs/formats/mamut-toml-1.md section 6. A written value is kept, including `allow_experimental = false` and `release_i = false`. `[defaults]` is not extracted by deleting a value written on a node.
- Permitted sources used: docs/V1.md section 13, docs/formats/mamut-toml-1.md, docs/adr/ADR-006-mamut-toml.md, docs/CLEAN_ROOM.md, docs/tickets/07-toml-export-canonico.md, and the existing engine-cli converter. No book page was required.
- Own derivations: none. The emitter applies the closed textual rules. Empty collections and all-zero member-load vectors stay omitted because section 6 says the reader reconstructs them.
- Third-party code copied: none.
- Validation: `cargo test --offline --locked -p engine-cli` exited 0 (125 tests) on the U9 commit, before U5 was merged into that branch.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING

## Decisions applied

U9 is not reopened. That increment did not implement U5. The following increment does. U10 stays open. U11 is the third increment. The escape spelling already used by the emitter is unchanged.

## Pending work

J4 review of this log. The reviewer line stays PENDING. This session does not sign it.

## Optional names · U5

- Ticket / PR: docs/tickets/06-toml-nombres.md (U5, MAM-993). Branch `ticket/06-toml-names`.
- Author(s): implementation agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer.
- Contract implemented: optional `name` on nodes, members and springs in mamut-toml/1. Export restores those names only from an accompanying idmap/1 document. engine-input/1 is unchanged.
- Permitted sources used: docs/V1.md section 13 (U5), docs/tickets/06-toml-nombres.md, docs/formats/mamut-toml-1.md, and the idmap/1 field list in docs/adr/ADR-005-alias-name-idmap.md. docs/CLEAN_ROOM.md. No book page was required. Cita desconocida for any external publication.
- Own derivations: none. The converter applies the closed U5 decision.
- Third-party code copied: none.
- Validation: `cargo test --offline --locked -p engine-cli` exited 0 (131 tests) before merging U9. After that merge, the same command exited 0 (132 tests). Name assertions expect the two-space indent.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4). This session does not sign that line.

## Decisions applied

U5, as written in docs/V1.md section 13, is applied and is not reopened. `name` is optional. The integer id is required and is not invented. The same name may occur on a node, a member and a spring. A repeated name inside one of those collections is rejected. The name is not copied into engine JSON. `mamut-engine convert --to toml` writes names only when `--idmap` supplies an idmap/1 document. U10 is untouched: a support stays `restrained`, and `type` stays rejected.

ADR-005 still leaves schema publication and `input_sha256` unknown. This change does not publish an idmap schema and rejects any key other than `format`, `nodes`, `members` and `springs`.

## Pending work

J4 review of this log.

## Line and column · U11

- Ticket / PR: docs/tickets/08-toml-linea-columna.md. Continuation of the mamut-toml/1 converter (MAM-940). Does not close MAM-107.
- Author(s): implementation agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer.
- Contract implemented: U11 from docs/V1.md section 13, on the existing cli-error/1 document diagnostic. No schema field was added.
- Permitted sources used: docs/V1.md section 13, docs/formats/mamut-toml-1.md sections 5 and 8, docs/adr/ADR-006-mamut-toml.md, and the converter already in crates/engine-cli. No book page was required. The line and column count is an own reading of a byte span: lines split on U+000A, columns count Unicode scalar values, and both start at 1.
- Own derivations: the position function in crates/engine-cli/src/toml/core.rs. It does not copy the parser's display routine.
- Third-party code copied: none.
- Validation: `cargo test --offline --locked -p engine-cli` exited 0 on 2026-11-04 (127 tests) before this merge with main. After merging U9 and U5, the same command exited 0 (135 tests). The new cases cover an invalid document at line 1 column 1, a second line, a multibyte scalar column, a parser error without a span, and nesting depths 64 and 65. Comparison with an external program: no.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING

## Decisions and evidence

U11 is applied as written. The root table stays level 1, so depth 64 is unchanged. When `span()` exists and its start is a character boundary inside the text, the document error reports `line N, column M` in the existing hint. The code stays `CLI-E-DOCUMENT`. A span that is absent, past the text, or inside a multibyte scalar does not invent a line. The parser message is not copied into the diagnostic.

U7 was not reopened. An identifier longer than 256 bytes is still rejected.

## Pending work

Clean-room review of this log. J4 has not signed it.
