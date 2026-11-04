# Clean-room log · ENG-TOML-003

- Ticket / PR: docs/tickets/06-toml-nombres.md (U5, MAM-993). Branch `ticket/06-toml-names`.
- Author(s): implementation agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer.
- Contract implemented: optional `name` on nodes, members and springs in mamut-toml/1. Export restores those names only from an accompanying idmap/1 document. engine-input/1 is unchanged.
- Permitted sources used: docs/V1.md section 13 (U5), docs/tickets/06-toml-nombres.md, docs/formats/mamut-toml-1.md, and the idmap/1 field list in docs/adr/ADR-005-alias-name-idmap.md. docs/CLEAN_ROOM.md. No book page was required. Cita desconocida for any external publication.
- Own derivations: none. The converter applies the closed U5 decision.
- Third-party code copied: none.
- Validation: `cargo test --offline --locked -p engine-cli` exited 0 (131 tests). `cargo clippy --offline --locked -p engine-cli --all-targets -- -D warnings -A unused_mut` exited 0. The `unused_mut` allowance is the existing Windows local allowance. Workspace clippy was not run.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4). This session does not sign that line.

## Decisions applied

U5, as written in docs/V1.md section 13, is applied and is not reopened. `name` is optional. The integer id is required and is not invented. The same name may occur on a node, a member and a spring. A repeated name inside one of those collections is rejected. The name is not copied into engine JSON. `mamut-engine convert --to toml` writes names only when `--idmap` supplies an idmap/1 document. U10 is untouched: a support stays `restrained`, and `type` stays rejected.

ADR-005 still leaves schema publication and `input_sha256` unknown. This change does not publish an idmap schema and rejects any key other than `format`, `nodes`, `members` and `springs`.

## Pending work

J4 review of this log.
