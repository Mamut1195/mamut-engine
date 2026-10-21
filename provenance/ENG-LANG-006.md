# ENG-LANG-006 · Technical documentation in English

- Ticket: MAM-840 (ENG-LANG-006).
- Date: 2026-10-21 (America/Guayaquil).
- Author: Codex, GPT-6.
- Role: documentation translator; no numerical implementation or specification change.
- Base: origin/main, `7375fa25a34a3cb004bcaec84f645743b864ff89`; HEAD matched this base at the start.
- Sources: the existing in-scope documentation at the base, `clean-room/forbidden_identifiers.txt`, `changelog.d/README.md`, and the existing clean-room checker. Targeted repository searches located document consumers and inbound anchors; the relevant existing determinism test was read to preserve its behavior.
- Third-party code copied: none. No network access, external engines, sibling repositories or external results were used.
- Clean-room reviewer: **PENDING (J4)**; author checks do not grant independent approval.

## Files

Translated documentation:

- `README.md`
- `CONTRIBUTING.md`
- `docs/CONVENCION_EJES.md`
- `docs/DETERMINISMO.md`
- `docs/GUIA_METODOS.md`
- `docs/PYTHON_BOUNDARY_BENCHMARKS.md`
- `docs/TICKETS_PRS.md`
- `docs/plantillas/FICHA_METODO.md`
- `docs/plantillas/PROMPT_IA.md`
- `docs/plantillas/PROVENANCE.md`
- `docs/plantillas/REVISION.md`

Translated comments and prose placeholders only:

- `docs/plantillas/conformidad.rs.tpl`
- `docs/plantillas/fixture_uniaxial.rs.tpl`
- `docs/plantillas/modulo_uniaxial.rs.tpl`
- `docs/plantillas/fixture_api.json.tpl`

Added records: `changelog.d/MAM-840.changed.md` and this provenance file.

## Preservation and exclusions

No files were renamed: path changes belong to MAM-839. Code fences, commands, link destinations, SHA-256 strings, field names, contract identifiers, equations and numerical values were preserved. Rust template code and strings remain unchanged; only comments were translated. JSON template keys, identifiers and values remain unchanged except for five prose placeholders.

The existing inbound `GUIA_METODOS.md#5-piloto-reproducible-elasticidad-cúbica` link is retained through an explicit HTML anchor. A targeted `rg` search across the repository found this inbound section link; README anchors found elsewhere refer to other README files. The hash-table header consumed by `all_sixteen_published_hashes_remain_unchanged` is retained in an HTML comment immediately before the translated table so the existing test still selects the same 16 rows.

Spanish inside fenced blocks is intentionally preserved, including the restricted prompt, shell comments and equation annotations. Existing instructions requiring Spanish API documentation, messages and records were translated faithfully without changing that policy. Historical paths and historical evidence remain as written; they do not describe builds performed for this ticket. No new capabilities were documented. No numerical values from other engines were identified in the in-scope files.

Excluded from editing: `CHANGELOG.md` and all other `changelog.d/` files (the fragment format README was read as requested); `docs/CLEAN_ROOM.md`, `docs/CLA.md`, root `CLEAN_ROOM.md` and `CLA.md`; `docs/adr/**`; existing `provenance/**`; `docs/validation-report/**`; `clean-room/**`; `.github/**`; `vendor/**`; `Cargo.*`, `CODEOWNERS`, `deny.toml`; `crates/**` and `bindings/**`. These are protected policy, legal, history, implementation, dependency or reference-material surfaces outside this translation ticket. Read-only targeted searches for document consumers and anchors did not authorize edits there.

`docs/validation-report/**`, `provenance/ENG-BENCH-001-SMOKE-VPS.md`, the restricted audit directory and logs of other Codex sessions were not opened. The validation-report tree and the prohibited provenance file were excluded from the inbound-link search.

## Validation

All requested checks passed on 2026-10-21. Cargo was placed on PATH with `export PATH=$HOME/.cargo/bin:$PATH`; all builds and tests used the environment-authorized `$CARGO_TARGET_DIR`, offline. Build output and logs are under `$CARGO_TARGET_DIR/mam840-evidence/`.

- The temporary `mam840-verify.py` script compares each of the 15 translated files against an untouched snapshot, also confirmed against `origin/main`. All 11 fenced code blocks, 61 Markdown link destinations, 2 URLs and 21 occurrences of 64-character hex strings match exactly. It also checks Rust template code, the five permitted JSON prose-placeholder changes and added lines for prohibited identifiers, including this provenance file.
- Additional comparison of inline code found only translated ticket-status prose labels; commands, paths, JSON snippets, field names and equations in inline code were unchanged. Numerical inventories were retained; the added legacy anchor contains the existing section number.
- `cargo test --workspace --offline`: exit 0; 749 passed, 0 failed, 0 ignored across 115 test and doctest result groups. The existing `all_sixteen_published_hashes_remain_unchanged` test passed without modification. Log: `$CARGO_TARGET_DIR/mam840-evidence/cargo-test.log`.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --offline`: exit 0; documentation generated successfully with warnings denied. Log: `$CARGO_TARGET_DIR/mam840-evidence/cargo-doc.log`.
- `bash clean-room/check.sh origin/main`: exit 0, `Clean-room: OK`. All 17 deliverable paths, including the new fragment and provenance, were added only to a temporary index initialized from HEAD. Temporary Git object storage used the existing object directory as an alternate. The real Git index was left unchanged. Log: `$CARGO_TARGET_DIR/mam840-evidence/clean-room.log`.
- `git diff --check`: passed. Scope review: 15 translated files and 2 new records, no renames or implementation changes.

The preservation script and its untouched snapshots are temporary verification artifacts, outside the deliverable. No commit was created. Independent J4 review remains pending.
