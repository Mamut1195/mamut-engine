# Clean-room log · ENG-TOML-003

- Ticket / PR: docs/tickets/08-toml-linea-columna.md. Continuation of the mamut-toml/1 converter (MAM-940). Does not close MAM-107.
- Author(s): implementation agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer.
- Contract implemented: U11 from docs/V1.md section 13, on the existing cli-error/1 document diagnostic. No schema field was added.
- Permitted sources used: docs/V1.md section 13, docs/formats/mamut-toml-1.md sections 5 and 8, docs/adr/ADR-006-mamut-toml.md, and the converter already in crates/engine-cli. No book page was required. The line and column count is an own reading of a byte span: lines split on U+000A, columns count Unicode scalar values, and both start at 1.
- Own derivations: the position function in crates/engine-cli/src/toml/core.rs. It does not copy the parser's display routine.
- Third-party code copied: none.
- Validation: `cargo test --offline --locked -p engine-cli` exited 0 on 2026-11-04 (127 tests). The new cases cover an invalid document at line 1 column 1, a second line, a multibyte scalar column, a parser error without a span, and nesting depths 64 and 65. Comparison with an external program: no.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING

## Decisions and evidence

U11 is applied as written. The root table stays level 1, so depth 64 is unchanged. When `span()` exists and its start is a character boundary inside the text, the document error reports `line N, column M` in the existing hint. The code stays `CLI-E-DOCUMENT`. A span that is absent, past the text, or inside a multibyte scalar does not invent a line. The parser message is not copied into the diagnostic.

U7 was not reopened. An identifier longer than 256 bytes is still rejected.

## Pending work

Clean-room review of this log. J4 has not signed it.
