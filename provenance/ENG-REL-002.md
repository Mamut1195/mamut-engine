# Clean-room log · ENG-REL-002

- Ticket: MAM-869 (ENG-REL-002), publication of versioned schemas.
- Author: Codex, GPT-6, 2026-10-21 (America/Guayaquil).
- Role: implementer.
- Sources: ticket requirements; engine-api's own API, schemas and tests;
  changelog.d/README.md; local list of prohibited identifiers and clean-room
  checker; existing Rust workflow to document the runner. No network.
- Contract: schemas/ tree identical to schema(), canonical index with sizes and
  SHA-256; extensible API/CLI registry; deterministic USTAR TAR without publication.
- Derivations: lexicographic order, constant archive metadata and SHA-256
  of original bytes. No work on mechanical models.
- Third-party code copied: none. Python uses the standard library and Rust
  uses existing dependencies. No manifest or lockfile changes.
- Decisions: preserve URNs and capabilities; associate URNs with URLs in the
  index. Proposed base https://schemas.mamut.systems/, pending Jonathan.
  Future identity migration reserved for Jonathan.
- Declaration: only permitted own sources were used; no external structural
  analysis code, restricted repositories or records from other sessions
  were consulted. No mechanical elements were modified.
- Validation: automated tests for byte equality, registry coverage,
  hashes and sizes, exact inventory, CLI registration, links, corruption,
  regeneration blocking in CI and TAR repeatability/metadata.
- Checks: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets --offline -- -D warnings` and
  `cargo test --workspace --offline`: OK, 751 tests passed, none failed
  or ignored. The two new Cargo integration tests also include six
  Python tooling tests.
- `bash clean-room/check.sh origin/main`: OK including new files through
  a temporary index and object store in /tmp, without modifying the real index.
  `git diff --check`: OK. Identifier list also checked in all
  new files, including docs and provenance: OK.
- Two local packager runs: same TAR SHA-256,
  `29df0f6a55ebedb5a96a23ee6a2db8f1cf81f343b7947009ae55748f70d98e29`.
- Clean-room reviewer: pending human review.
- Publication: none; no push, tags, releases or commit.
