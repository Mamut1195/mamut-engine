# Clean-room log · ENG-TOML-003

- Ticket / PR: docs/tickets/07-toml-export-canonico.md. Decision U9 in docs/V1.md section 13. Related specification ticket MAM-939. This note does not close MAM-107 and does not approve a time or memory budget.
- Author(s): implementation agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer.
- Contract implemented: canonical mamut-toml/1 export. Same model, same UTF-8 bytes. LF, one trailing newline, two-space indentation under each table header, root keys and headers at column 0, keys in the order of docs/formats/mamut-toml-1.md section 6. A written value is kept, including `allow_experimental = false` and `release_i = false`. `[defaults]` is not extracted by deleting a value written on a node.
- Permitted sources used: docs/V1.md section 13, docs/formats/mamut-toml-1.md, docs/adr/ADR-006-mamut-toml.md, docs/CLEAN_ROOM.md, docs/tickets/07-toml-export-canonico.md, and the existing engine-cli converter. No book page was required.
- Own derivations: none. The emitter applies the closed textual rules. Empty collections and all-zero member-load vectors stay omitted because section 6 says the reader reconstructs them.
- Third-party code copied: none.
- Validation: `cargo test --offline --locked -p engine-cli` exited 0 (125 tests). `cargo clippy --offline --locked -p engine-cli --all-targets -- -D warnings -A unused_mut` exited 0. The new test compares the bytes of two exports and checks that a written value remains. The workspace suite was not run.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING

## Decisions applied

U9 is not reopened. U5 is not implemented: names and idmap stay absent, as on this base. U10 and U11 stay as they were. The escape spelling already used by the emitter is unchanged.

## Pending work

J4 review of this log. The reviewer line stays PENDING. This session does not sign it.
