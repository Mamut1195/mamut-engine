# Clean-room log · ENG-CORE-004

- Ticket: MAM-17 · ENG-CORE-004, «mamut.engine.model/1 and job/1 input schema».
- Branch: j2/eng-core-004, stacked on j2/eng-api-001 (PR #88, unmerged).
- Author: Codex gpt-6.1-sol high under J2, 2026-10-14 America/Guayaquil.
- Role: implementer. Clean-room reviewer: PENDING (J4).
- Contract: preserves engine-input/1, serde and schemas published in #88;
  adds fixtures, folder/name tests and model + job documentation.
  Final name PENDING Jonathan, Notion «02 · Contratos».
- Permitted sources used: J2 requirements received in this conversation;
  own crates/engine-api schemas, API, README and tests; own engine-core
  catalog; clean-room/README.md, check.sh and forbidden_identifiers.txt
  (the latter incorporated through relative include_str! in the test).
- Own derivations: SI examples declared by hand using the analytical equations
  already checked in #88. Cantilever: equilibrium R=P, M=PL and integration
  of EI y''=−P(L−x) → y(L)=−PL³/(3EI). Uniform simply supported beam:
  R=wL/2, Mmax=wL²/8; explicit combination D+0,5L by superposition,
  without code coefficients. Oscillating bar: EA/L=100 N/m and m=5 kg,
  ω²=k/m=20 s⁻². Constant spectrum Sa=3 m/s²: u=Sa/ω²=0,15 m,
  V=m Sa=15 N. P-Delta: explicit axial compression of 10 N and lateral load
  of 1 N on a cantilever with L=4 m, EI=600 N·m²; the own equation
  EI y''''+P y''=0 has first pole Pcr=π²EI/(4L²)>10 N. Existing
  analytical tests remain the oracles; solvers are not duplicated.
- Rejections: unknown schema, unknown major version, 1e400 (replaces
  NaN, which is not representable in JSON), missing reference, units/axes,
  integer outside u64, id outside u32 and unknown field. Each .expect pins
  the actual #88 code, output and validate result; without changing the library.
- Names: schemas and fixtures are inspected, including .expect, optional public
  fields and escaped JSON keys. The additional key list complements
  the repository's forbidden identifiers. `member.element` is preserved
  as a legitimate selector of the project's own factory, without a command structure or
  positional arguments; exception documented in README and in the test.
  `pattern` with a string value in schemas is the standard regex keyword
  (SHA-256/codes for results/1); it is forbidden in fixtures and public fields.
- #85 is preserved: the combination runs with a warning and exit 1 because of
  the zero prescribed support values; the ticket does not modify or close that follow-up.
- Third-party code copied: none. No network queries, no new dependencies,
  no changes to docs/CLEAN_ROOM.md, .github/** or clean-room/**.
- Comparison with external oracle or sibling repo: no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

Checks on the final tree, with Rust 1.99.0,
`CARGO_TARGET_DIR=/workspace/.cargo-target`, `CARGO_NET_OFFLINE=true`
and `PATH=/home/box/.cargo/bin:$PATH`:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 479 tests, 0 failures/ignored;
  includes five new tests (two fixture and three name tests).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK, including new files.
- `git diff --check`: OK. Only fixtures, tests, README and this log;
  no changes to library, schemas, dependencies or protected files.
- Human clean-room review: PENDING (J4).

## MAM-480 · ENG-CORE-004-FU

- `LEGACY_DEFS` is frozen without growth: exactly the ordered lists documented
  below for `urn:mamut:engine-input:1` (8) and `urn:mamut:results:1` (7),
  15 names in total, without duplicates. In `tests/input_names.rs`,
  `legacy_definitions_are_frozen_by_mam_480` pins the IDs, order and content;
  `frozen_legacy_definitions_exist_in_the_published_schemas` checks their
  presence under published root `$defs` and rejects other non-snake_case names.
- Renaming remains pending a major contract change (`results/2`,
  `engine-input/2` or aliases); when: DESCONOCIDO.
- Existing `element` scope tests are preserved: `model.members[].element`,
  engine-input/1 `/$defs/member/properties/element` and results/1
  `/$defs/memberIdentity/properties/element`, with rejections elsewhere.
- String values now reject Unicode Cf characters (including zero-width and
  bidi controls) and Cyrillic letters, including decoded JSON escapes,
  nested objects and arrays. Greek is allowed in all string values, without
  path exceptions. Existing accented Latin text and mathematical symbols
  remain unchanged; no dependencies or contract files are changed.
- Initial audit: the previous guard passed all 25 tests. Four published
  descriptions already contain legitimate Greek mathematics: engine-input/1
  `/$defs/spring/description` (δ, θ), `/$defs/soil_options/description` (γ),
  `/$defs/consolidation_options/description` (γ, Σ), and results/1
  `/description` (φ). Fixtures contain no non-ASCII string values. Greek is
  not rejected because these published descriptions use it as mathematical
  notation. Known limitation: Greek confusables in string values remain
  undetected. The positive description test accepts δ/γ/φ, both literal and
  JSON-escaped; negative tests retain nested Cyrillic and Cf values.

## J4 r23 corrections

- Date: 2026-10-14 (America/Guayaquil).
- Author: Codex gpt-6.1-sol high under J2. Role: implementer.
- Reviewer: PENDING (J4).
- Ticket / PR: MAM-17 · ENG-CORE-004 / #95.
- Permitted sources: J2 request and local review
  `/workspace/j4rev/comments/engine-95-r23.md`; own
  `clean-room/forbidden_identifiers.txt` list; existing engine-api schemas, fixtures and
  tests. No network queries or code from other engines.
- Correction base: j2/eng-core-004 branch already integrated with `origin/main`
  at 9a27e9f, including #88; the published contract is preserved.
- Guard: case-insensitive ASCII comparison removing `_`/`-`
  for the clean-room list and COMMAND_KEYS. Structural traversal with paths,
  decoded strings and duplicate keys, without converting out-of-range
  numbers from negative fixtures. Does not change the library parser.
- Data: ASCII regex `^[a-z][a-z0-9_]*$` on all fixture keys and
  names under `properties`/`$defs`. JSON Schema keywords separated
  through an explicit whitelist; `pattern` only as a keyword with a string value.
  Objects in `const`, `enum`, `default` and `examples` are inspected as
  data, without inheriting keyword permissions.
- Explicit contractual exceptions, PENDING Jonathan/J4 decision:
  a decision is requested on retaining them or planning a later migration. No
  contract, property or `$ref` target published in #88 is renamed.
  All published property names satisfy the regex; these 15
  internal `$defs` names use camelCase and are excepted only at the
  root and in the indicated contract:
  - engine-input/1: `/$defs/uniformLoad`, `/$defs/dofValue`,
    `/$defs/massSource`, `/$defs/pDeltaOptions`, `/$defs/modalOptions`,
    `/$defs/spectrumTable`, `/$defs/spectrumDirection`, `/$defs/spectrumOptions`.
  - results/1: `/$defs/nodeIdentity`, `/$defs/memberIdentity`,
    `/$defs/fieldExtremes`, `/$defs/spectrumResponse`, `/$defs/caseFactor`,
    `/$defs/pDeltaIteration`, `/$defs/pDeltaConvergence`.
- `element` is added to COMMAND_KEYS. It is only allowed in
  fixture `model.members[].element` and `/$defs/member/properties/element`
  in the engine-input/1 schema: declarative selector of the project's own factory.
  The ACTUAL already published field
  `/$defs/memberIdentity/properties/element` in results/1 was also detected, identity of the kind
  used by each member. It is preserved with an exact contractual exception,
  also PENDING a decision, to respect the #88 contract. `element` is not
  allowed at other paths, nor normalized variants of its name.
- Tests: an independent negative for each case requested by r23, plus
  hyphen, new camelCase, variants of the entire list, Unicode escapes,
  duplicates, unknown keywords and exceptions outside the path/contract.
- Oracles: only the own analytical ones already derived by hand; this correction
  adds no solvers, dependencies, external sources or library changes.
- Third-party code copied: none. Comparison with external oracle or sibling
  repo: no. No changes to docs/CLEAN_ROOM.md, .github/** or clean-room/**.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."

Correction checks with Rust 1.99.0, without network,
`CARGO_TARGET_DIR=/workspace/.cargo-target` and `CARGO_NET_OFFLINE=true`:

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline --workspace`: OK, 501 tests, 0 failures/ignored;
  includes 25 name tests (22 added relative to r23).
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK.
- `git diff --check`: OK. Changes limited to tests, README and provenance.
- Human clean-room review: PENDING (J4).
