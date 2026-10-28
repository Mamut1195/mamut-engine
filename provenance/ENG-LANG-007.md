# ENG-LANG-007 — Tests, validation and scripts in English

- Ticket: MAM-843 (ENG-LANG-007), tranche 1.
- Date: 2026-10-27.
- Base: `origin/main`, `340b54222470583318a0866d29b8ad243a60376b`.
- Sources: the scoped engine-validation sources, the ticket instructions,
  `provenance/ENG-LANG-004.md`, `provenance/ENG-LANG-005.md`,
  `changelog.d/MAM-838.changed.md`, `changelog.d/README.md`, and the repository
  restricted-identifier list. Work and verification were offline. No third-party
  code, numerical changes, or new mathematical derivations were introduced.

## Scope and translated lines

Reviewed 30 sources: one crate root, seven test entry points, eleven test support
files, six examples, and five CommonJS scripts. Translated text in 28 sources;
`tests/support/consolidation_harness.rs` and
`tests/support/consolidation_multilayer.rs` already have English comments and
contain no eligible Spanish assertion/panic message literals. Paths in the table
are repository-relative.

Counts record translated physical lines, once per category. Rustdoc includes
`//!` and `///`; comments include inline Rust comments and JavaScript comments.
Messages count eight Rust assertion/expect format strings and five JavaScript
usage strings. Unchanged English, symbolic expressions, blank lines, executable
code, and formatting-only changes are excluded. Rust formatting collapses one
assertion call in `tests/analytical.rs` by four lines; final line numbers below
refer to the working sources.

| File | Rustdoc | Comments | Messages | Total |
| --- | ---: | ---: | ---: | ---: |
| `crates/engine-validation/src/lib.rs` | 8 | 0 | 0 | 8 |
| `crates/engine-validation/tests/analytical.rs` | 1 | 4 | 3 | 8 |
| `crates/engine-validation/tests/axial.rs` | 1 | 0 | 1 | 2 |
| `crates/engine-validation/tests/consolidation.rs` | 1 | 0 | 1 | 2 |
| `crates/engine-validation/tests/dynamic.rs` | 1 | 4 | 0 | 5 |
| `crates/engine-validation/tests/py.rs` | 1 | 2 | 0 | 3 |
| `crates/engine-validation/tests/winkler.rs` | 1 | 0 | 0 | 1 |
| `crates/engine-validation/tests/winkler_elements.rs` | 1 | 5 | 1 | 7 |
| `crates/engine-validation/tests/support/consolidation_harness.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-validation/tests/support/consolidation_multilayer.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-validation/tests/support/consolidation_table.rs` | 2 | 0 | 0 | 2 |
| `crates/engine-validation/tests/support/dynamic_harness.rs` | 1 | 1 | 2 | 4 |
| `crates/engine-validation/tests/support/dynamic_table.rs` | 1 | 0 | 0 | 1 |
| `crates/engine-validation/tests/support/harness.rs` | 1 | 3 | 0 | 4 |
| `crates/engine-validation/tests/support/py_harness.rs` | 1 | 2 | 0 | 3 |
| `crates/engine-validation/tests/support/py_table.rs` | 1 | 1 | 0 | 2 |
| `crates/engine-validation/tests/support/table.rs` | 1 | 0 | 0 | 1 |
| `crates/engine-validation/tests/support/winkler_harness.rs` | 1 | 3 | 0 | 4 |
| `crates/engine-validation/tests/support/winkler_table.rs` | 1 | 1 | 0 | 2 |
| `crates/engine-validation/examples/axial_wasm.rs` | 2 | 0 | 0 | 2 |
| `crates/engine-validation/examples/consolidation_wasm.rs` | 2 | 0 | 0 | 2 |
| `crates/engine-validation/examples/fixture_runner.rs` | 1 | 0 | 0 | 1 |
| `crates/engine-validation/examples/fixture_wasm.rs` | 6 | 0 | 0 | 6 |
| `crates/engine-validation/examples/py_wasm.rs` | 2 | 0 | 0 | 2 |
| `crates/engine-validation/examples/winkler_wasm.rs` | 2 | 0 | 0 | 2 |
| `crates/engine-validation/scripts/run-axial-wasm.cjs` | 0 | 1 | 1 | 2 |
| `crates/engine-validation/scripts/run-consolidation-wasm.cjs` | 0 | 1 | 1 | 2 |
| `crates/engine-validation/scripts/run-fixture-wasm.cjs` | 0 | 2 | 1 | 3 |
| `crates/engine-validation/scripts/run-py-wasm.cjs` | 0 | 1 | 1 | 2 |
| `crates/engine-validation/scripts/run-winkler-wasm.cjs` | 0 | 1 | 1 | 2 |
| **Total** | **40** | **32** | **13** | **85** |

Code tokens, identifiers, logic, statement order, macro arguments, format
placeholders, stable ENG-E-* codes, case names, data labels, JSON keys, file
names, and successful output remain unchanged. Only the listed diagnostic
messages change at runtime, on assertion/expect failure or missing script
arguments. Crate rustdoc is documentation; these sources contain no macro that
exports it as runtime data. Protected files and existing provenance are unchanged.

## Terminology

Terminology follows ENG-LANG-004 and ENG-LANG-005: DOF, node, member, beam,
truss, spring, fixture, golden, oracle, station, prescribed displacement,
load case, mode shape, base shear, recovery, and end actions. Geotechnical
comments use settlement, pressure, flux, foundation, and tributary pile length.
The words fixture and fixtures retain their form. Existing units, equations,
API references, and citations preserve their meaning.

## String literals changed

Before any source literal changed, a repository-wide `rg --hidden --no-ignore`
literal-text search covered every complete old literal, including hidden CI
files, tests, goldens, JSON, scripts, clean-room files, and docs. Git metadata,
build output, and the explicitly prohibited provenance document were excluded.
Only matching text was returned, avoiding unrelated reference data. The search
found no literal comparison or compared output consuming these messages.
The short rejection phrase also occurs in unrelated prose and longer test
diagnostics; the axial message is reused as an expect diagnostic in another
crate. No dependent contract was found.

| File:line | Old -> new |
| --- | --- |
| `crates/engine-validation/scripts/run-axial-wasm.cjs:5` | `Uso: node run-axial-wasm.cjs <axial_wasm.wasm>` -> `Usage: node run-axial-wasm.cjs <axial_wasm.wasm>` |
| `crates/engine-validation/scripts/run-consolidation-wasm.cjs:5` | `Uso: node run-consolidation-wasm.cjs <consolidation_wasm.wasm>` -> `Usage: node run-consolidation-wasm.cjs <consolidation_wasm.wasm>` |
| `crates/engine-validation/scripts/run-fixture-wasm.cjs:6` | `Uso: node run-fixture-wasm.cjs módulo.wasm < entrada.json` -> `Usage: node run-fixture-wasm.cjs module.wasm < input.json` |
| `crates/engine-validation/scripts/run-py-wasm.cjs:5` | `Uso: node run-py-wasm.cjs <py_wasm.wasm>` -> `Usage: node run-py-wasm.cjs <py_wasm.wasm>` |
| `crates/engine-validation/scripts/run-winkler-wasm.cjs:5` | `Uso: node run-winkler-wasm.cjs <winkler_wasm.wasm>` -> `Usage: node run-winkler-wasm.cjs <winkler_wasm.wasm>` |
| `crates/engine-validation/tests/analytical.rs:481` | `Actualizar el conteo documentado si cambia la tabla` -> `Update the documented count if the table changes` |
| `crates/engine-validation/tests/analytical.rs:485` | `Nombre de fixture repetido: {}` -> `Duplicate fixture name: {}` |
| `crates/engine-validation/tests/analytical.rs:490` | `Metadatos incompletos: {}` -> `Incomplete metadata: {}` |
| `crates/engine-validation/tests/axial.rs:7` | `Fixtures axiales` -> `Axial fixtures` |
| `crates/engine-validation/tests/consolidation.rs:8` | `benchmarks consolidación` -> `consolidation benchmarks` |
| `crates/engine-validation/tests/support/dynamic_harness.rs:44` | `Fixture repetido: {}` -> `Duplicate fixture: {}` |
| `crates/engine-validation/tests/support/dynamic_harness.rs:49` | `Metadatos: {}` -> `Metadata: {}` |
| `crates/engine-validation/tests/winkler_elements.rs:20` | `debe rechazar` -> `must reject` |

## Left in Spanish

All other literals remain byte-identical. This includes Spanish table labels,
case names, citations, tolerance explanations, helper-returned EngineError
diagnostics, custom `require` messages, example error output, and printed
summaries. These are outside the explicit literal allowlist; their comparison
or output contracts were not assumed safe to change. JavaScript assertion
messages also remain: the script literal allowance explicitly names usage
errors only. Existing English and language-neutral literals are preserved too.

The inventory below lists **every unchanged literal occurrence**, including
neutral literals, so short Spanish labels missed by a language heuristic are
covered. Each line number identifies its literal(s) directly in the named
source; multiple occurrences on one line share a location. No retained literal
is omitted. Source files with no retained literals do not need an entry.

| Source file | Lines containing unchanged literals | Reason |
| --- | --- | --- |
| `crates/engine-validation/tests/analytical.rs` | 2, 4, 41, 42, 47, 49, 95, 96, 98, 103, 105, 106, 107, 133, 137, 138, 139, 140, 165, 166, 167, 168, 169, 170, 171, 180, 181, 183, 189, 194, 200, 209, 218, 241, 248, 251, 255, 257, 262, 268, 291, 292, 308, 312, 314, 319, 324, 325, 326, 327, 330, 339, 343, 344, 345, 365, 369, 374, 379, 385, 388, 391, 406, 407, 409, 415, 419, 421, 424, 434, 438, 446, 451, 457, 462, 467, 488, 495 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/tests/axial.rs` | 2 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/tests/consolidation.rs` | 2, 4 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/tests/dynamic.rs` | 2, 4, 33, 41, 50, 86, 94, 97, 99, 105, 111, 113, 114, 116, 121, 177, 191, 201, 203, 208, 234, 257, 263, 275, 282, 286, 306, 307, 310, 316, 319, 322, 328, 333, 342, 349, 352, 356, 358, 374, 375, 403, 416, 421, 426, 431, 438, 439, 447, 507, 509, 514, 521, 530, 536, 559, 562, 578, 579, 580, 583, 593, 595, 597, 603, 622, 690, 691, 693, 702, 707 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/tests/py.rs` | 2, 4 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/tests/winkler.rs` | 2, 4, 8 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/tests/winkler_elements.rs` | 16, 285, 290, 292, 399, 400, 401, 405, 406, 407, 408, 409, 410, 415, 423, 424, 441, 442, 443, 448, 456, 458, 464, 467, 469, 472, 474, 481, 492, 503, 514, 515, 518, 519, 522, 533, 544, 560, 561, 562, 563, 564, 567 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/tests/support/consolidation_harness.rs` | 12, 34, 37, 69, 139, 144, 148, 154, 183, 199, 203, 207, 232, 234, 236, 253, 255, 264 | Helper-returned errors, custom checks, labels, codes, and paths: outside the assertion/panic allowance; existing English is retained. |
| `crates/engine-validation/tests/support/consolidation_multilayer.rs` | 81, 85, 86, 89, 95, 137, 152, 169, 176, 183 | Helper-returned errors, custom checks, labels, codes, and paths: outside the assertion/panic allowance; existing English is retained. |
| `crates/engine-validation/tests/support/consolidation_table.rs` | 17, 40, 42, 44, 49, 51, 58, 64, 66, 73, 84, 86 | Fixture data, case names, citations, provenance references, and tolerance text: outside the literal allowance. |
| `crates/engine-validation/tests/support/dynamic_harness.rs` | 17, 18, 26, 27, 38, 47, 54, 62, 129, 148, 231, 232, 275, 283 | Helper-returned errors, custom checks, labels, codes, and paths: outside the assertion/panic allowance; existing English is retained. |
| `crates/engine-validation/tests/support/dynamic_table.rs` | 109, 115, 119, 123, 127, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 145, 153, 161, 169, 177, 185, 193, 200, 208, 216, 224, 232, 240, 250, 257, 264, 271, 278, 285, 292, 299, 307, 315, 327, 336, 345, 354, 363, 371, 379, 387, 397 | Fixture data, case names, citations, provenance references, and tolerance text: outside the literal allowance. |
| `crates/engine-validation/tests/support/harness.rs` | 19, 26, 27, 57, 68, 117, 155, 156, 203, 204, 265, 275, 276, 290, 291, 293, 297, 360, 365, 400, 484, 485, 486, 491, 496, 503, 506, 516, 521, 530 | Helper-returned errors, custom checks, labels, codes, and paths: outside the assertion/panic allowance; existing English is retained. |
| `crates/engine-validation/tests/support/py_harness.rs` | 8, 13, 14, 23, 24, 28, 33, 95 | Helper-returned errors, custom checks, labels, codes, and paths: outside the assertion/panic allowance; existing English is retained. |
| `crates/engine-validation/tests/support/py_table.rs` | 17, 20, 22, 23, 24, 30, 32, 33, 34, 42, 44, 45, 46 | Fixture data, case names, citations, provenance references, and tolerance text: outside the literal allowance. |
| `crates/engine-validation/tests/support/table.rs` | 64, 69, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 102, 107, 122, 124, 131, 133, 140, 141, 143, 150, 152, 160, 168, 176, 183, 185, 192, 194, 202, 209, 211, 218, 220, 227, 229, 236, 238, 245, 247, 254, 256, 263, 265, 272, 274, 281, 283, 291, 299, 307, 314, 316, 324, 332 | Fixture data, case names, citations, provenance references, and tolerance text: outside the literal allowance. |
| `crates/engine-validation/tests/support/winkler_harness.rs` | 15, 23, 30, 31, 32, 38, 39, 40, 41, 42, 43, 45, 60, 61, 93, 113, 145, 146, 151, 192, 199, 206, 213, 222, 224, 280, 285, 292, 297, 308 | Helper-returned errors, custom checks, labels, codes, and paths: outside the assertion/panic allowance; existing English is retained. |
| `crates/engine-validation/tests/support/winkler_table.rs` | 27, 29, 31, 39, 41, 46, 47, 48, 49, 50, 51 | Fixture data, case names, citations, provenance references, and tolerance text: outside the literal allowance. |
| `crates/engine-validation/examples/axial_wasm.rs` | 2, 10, 12 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/examples/consolidation_wasm.rs` | 2, 4, 10, 12 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/examples/fixture_runner.rs` | 2, 17, 20, 22, 23, 30, 33, 38, 54, 55, 56, 61, 67, 80, 98, 100, 106, 112, 118 | Example error output, arguments, file paths, and target labels: outside the assertion/panic allowance. |
| `crates/engine-validation/examples/fixture_wasm.rs` | 18, 29, 42, 57 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/examples/py_wasm.rs` | 2, 4, 10, 12 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/examples/winkler_wasm.rs` | 2, 4, 10, 12 | Data/check labels, codes, keys, paths, or existing English diagnostics: outside the Spanish assertion/panic allowance. |
| `crates/engine-validation/scripts/run-axial-wasm.cjs` | 2, 3, 7, 9, 10 | Assertion diagnostics, printed summaries, module names, and wire labels: outside the usage-message allowance. |
| `crates/engine-validation/scripts/run-consolidation-wasm.cjs` | 2, 3, 7, 9, 10 | Assertion diagnostics, printed summaries, module names, and wire labels: outside the usage-message allowance. |
| `crates/engine-validation/scripts/run-fixture-wasm.cjs` | 3, 4, 8, 11, 12, 13, 15, 16, 20, 23, 24, 25 | Assertion diagnostics, printed summaries, module names, and wire labels: outside the usage-message allowance. |
| `crates/engine-validation/scripts/run-py-wasm.cjs` | 2, 3, 7, 9, 10 | Assertion diagnostics, printed summaries, module names, and wire labels: outside the usage-message allowance. |
| `crates/engine-validation/scripts/run-winkler-wasm.cjs` | 2, 3, 8, 10, 11 | Assertion diagnostics, printed summaries, module names, and wire labels: outside the usage-message allowance. |

Total: 557 unchanged literal occurrences.

## Remaining (later tranches)

- engine-validation: `src/fixtures.rs`, `tests/oracles.rs`, `tests/fixtures.rs`,
  `README.md`, and `ORACLES.md`.
- `crates/*/tests` in the other crates.
- `scripts/**` and `scripts/tests`.
- `bindings/python/tests`.
- Outside this ticket's editable scope (owner-only): `clean-room/tests` and
  `.github/workflows`.
- The Spanish literals inventoried above require their own authorized scope
  and comparison/output review before translation.

## Verification

Verification uses Cargo directly with the shared build lock, offline. No commit
was created. The completed checks are recorded below.

| Check | Result |
| --- | --- |
| Source boundary audit | PASS: all 30 scoped sources retain identical code tokens after restoring the 13 approved messages and removing comments/whitespace; all other literal occurrences match the base. |
| `cargo fmt --all --check` | PASS after the assertion formatting adjustment. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | PASS. |
| `cargo test -p engine-validation --offline` | PASS: 31 integration tests across nine suites; zero failures or ignored tests. Crate unit tests and doctests contain no tests. |
| `node --check` on all five scoped `.cjs` files | PASS. |
| `bash clean-room/check.sh origin/main` | PASS, including both new documents after `git add -N` in a separate writable index. |
| `bash scripts/changelog-check.sh origin/main` | PASS; the script compares committed HEAD changes and reports no code changes. Supplemental working-tree fragment checks pass. |
| `git diff --check` | PASS; the base comparison also passes with both new documents included through the separate index. |

The managed worktree Git index and object database are read-only. Intent-to-add
registration therefore used a separate writable index and object directory,
with the existing object database as a read-only alternate. The real index
remains unchanged. Supplemental working-tree checks validate the new fragment
name, nonempty Spanish content, required ticket marker and tramo 1 wording,
absence of headings, and unchanged CHANGELOG.md. Scope and added-text checks
confirm exactly 28 edited sources plus the two new documents and no added
restricted identifiers. Check scratch data was placed outside the checkout and
outside the system temporary directory.

## Tranche 2

- Ticket: MAM-843 (ENG-LANG-007), tranche 2.
- Date: 2026-10-27.
- Base: `origin/main`, `277334e68c0302794242897293f35981195b940f`.
- Sources: all scoped Rust test sources, the ticket instructions,
  `provenance/ENG-LANG-004.md`, `provenance/ENG-LANG-005.md`, tranche 1 above,
  `changelog.d/README.md`, and the repository restricted-identifier list.
  Work and verification were offline. No third-party code, numerical changes,
  or new mathematical derivations were introduced.

### Scope and translated lines (tranche 2)

Reviewed 93 Rust sources: every `.rs` under `crates/*/tests/**` except
engine-validation, including support modules and Rust fixture modules, plus
`crates/engine-contracts/src/tests.rs`, `crates/engine-geo/src/tests.rs`, and
`crates/engine-model/src/tests.rs`. Translated text in 82 sources; ten already
have English comments and no eligible Spanish messages. The additional unchanged source,
`crates/engine-api/tests/val032_runs.rs`, is pinned by a full-file SHA-256 in
the protected fixture index; its 13 Spanish comment lines and eleven eligible
message literals are deferred. No source was identified as containing
numerical output from an external structural program. Non-Rust fixture data and every other source remain unchanged.

Counts record translated physical lines, once per category. Rustdoc includes
`//!` and `///`; comments include inline Rust comments. Messages count only
Spanish assertion/panic/expect format strings in the explicit allowance.
Existing English, symbolic expressions, blank lines, executable code, and
formatting-only changes are excluded. Rust formatting adjusts assertion and
panic layouts in seven files; final line numbers below refer to the working
sources. Paths in the table are repository-relative.

| File | Rustdoc | Comments | Messages | Total |
| --- | ---: | ---: | ---: | ---: |
| `crates/engine-analysis/tests/analytical.rs` | 2 | 8 | 0 | 10 |
| `crates/engine-analysis/tests/bench_pdelta.rs` | 9 | 7 | 0 | 16 |
| `crates/engine-analysis/tests/combination.rs` | 1 | 18 | 2 | 21 |
| `crates/engine-analysis/tests/diaphragm.rs` | 1 | 5 | 0 | 6 |
| `crates/engine-analysis/tests/modal.rs` | 2 | 11 | 1 | 14 |
| `crates/engine-analysis/tests/newton.rs` | 2 | 10 | 0 | 12 |
| `crates/engine-analysis/tests/pdelta.rs` | 1 | 28 | 1 | 30 |
| `crates/engine-analysis/tests/results.rs` | 2 | 15 | 1 | 18 |
| `crates/engine-analysis/tests/spectrum.rs` | 2 | 9 | 0 | 11 |
| `crates/engine-analysis/tests/unit/pdelta_projection.rs` | 1 | 7 | 0 | 8 |
| `crates/engine-api/tests/determinism.rs` | 1 | 2 | 3 | 6 |
| `crates/engine-api/tests/dynamic.rs` | 1 | 10 | 1 | 12 |
| `crates/engine-api/tests/facade.rs` | 0 | 1 | 0 | 1 |
| `crates/engine-api/tests/i18n.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-api/tests/input_fixtures.rs` | 1 | 2 | 6 | 9 |
| `crates/engine-api/tests/input_names.rs` | 1 | 12 | 4 | 17 |
| `crates/engine-api/tests/limits.rs` | 1 | 7 | 0 | 8 |
| `crates/engine-api/tests/methods.rs` | 1 | 2 | 0 | 3 |
| `crates/engine-api/tests/pdelta_diaphragm.rs` | 1 | 3 | 6 | 10 |
| `crates/engine-api/tests/results.rs` | 1 | 5 | 1 | 7 |
| `crates/engine-api/tests/run.rs` | 1 | 7 | 0 | 8 |
| `crates/engine-api/tests/schema_publication.rs` | 0 | 1 | 3 | 4 |
| `crates/engine-api/tests/sfc010.rs` | 2 | 10 | 0 | 12 |
| `crates/engine-api/tests/support/determinism.rs` | 1 | 4 | 1 | 6 |
| `crates/engine-api/tests/support/mod.rs` | 3 | 0 | 1 | 4 |
| `crates/engine-api/tests/support/scalar_cases.rs` | 1 | 3 | 0 | 4 |
| `crates/engine-api/tests/support/schema.rs` | 2 | 1 | 7 | 10 |
| `crates/engine-api/tests/support/sfc010.rs` | 2 | 0 | 0 | 2 |
| `crates/engine-api/tests/val032_runs.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-api/tests/validate_performance.rs` | 1 | 1 | 0 | 2 |
| `crates/engine-assembly/tests/assembly.rs` | 0 | 1 | 1 | 2 |
| `crates/engine-assembly/tests/diaphragm.rs` | 0 | 0 | 2 | 2 |
| `crates/engine-assembly/tests/fixtures/counting.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-assembly/tests/fixtures/mod.rs` | 0 | 2 | 0 | 2 |
| `crates/engine-assembly/tests/fixtures/rigid_cases.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-assembly/tests/geometric_producers.rs` | 0 | 0 | 5 | 5 |
| `crates/engine-assembly/tests/j4_followup.rs` | 0 | 0 | 3 | 3 |
| `crates/engine-assembly/tests/solve_cases.rs` | 0 | 0 | 1 | 1 |
| `crates/engine-bench/tests/capabilities.rs` | 1 | 0 | 0 | 1 |
| `crates/engine-bench/tests/models.rs` | 0 | 0 | 3 | 3 |
| `crates/engine-cli/tests/agent_guide.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-cli/tests/cli.rs` | 0 | 17 | 5 | 22 |
| `crates/engine-cli/tests/contract.rs` | 1 | 12 | 19 | 32 |
| `crates/engine-cli/tests/describe.rs` | 0 | 2 | 0 | 2 |
| `crates/engine-cli/tests/error_registry.rs` | 0 | 8 | 9 | 17 |
| `crates/engine-cli/tests/error_schema.rs` | 0 | 1 | 2 | 3 |
| `crates/engine-cli/tests/i18n.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-cli/tests/support/progress.rs` | 0 | 1 | 0 | 1 |
| `crates/engine-cli/tests/support/schema.rs` | 2 | 0 | 8 | 10 |
| `crates/engine-cli/tests/support/validate_report.rs` | 1 | 0 | 2 | 3 |
| `crates/engine-elements-frame/tests/analytical.rs` | 1 | 10 | 0 | 11 |
| `crates/engine-elements-frame/tests/fixtures/mod.rs` | 1 | 2 | 0 | 3 |
| `crates/engine-elements-frame/tests/invariants.rs` | 1 | 11 | 1 | 13 |
| `crates/engine-elements-frame/tests/modes.rs` | 1 | 9 | 2 | 12 |
| `crates/engine-elements-frame3d/tests/analytical.rs` | 1 | 3 | 0 | 4 |
| `crates/engine-elements-frame3d/tests/fixtures/mod.rs` | 1 | 1 | 0 | 2 |
| `crates/engine-elements-frame3d/tests/invariants.rs` | 1 | 8 | 1 | 10 |
| `crates/engine-elements-frame3d/tests/r6.rs` | 1 | 11 | 3 | 15 |
| `crates/engine-elements-spring/tests/analytical.rs` | 1 | 4 | 0 | 5 |
| `crates/engine-elements-spring/tests/fixtures/mod.rs` | 1 | 0 | 0 | 1 |
| `crates/engine-elements-spring/tests/invariants.rs` | 1 | 10 | 2 | 13 |
| `crates/engine-elements-spring/tests/nonlinear.rs` | 3 | 7 | 1 | 11 |
| `crates/engine-elements-truss/tests/analytical.rs` | 1 | 8 | 0 | 9 |
| `crates/engine-elements-truss/tests/fixtures/mod.rs` | 1 | 5 | 0 | 6 |
| `crates/engine-elements-truss/tests/geometric_producer.rs` | 2 | 15 | 4 | 21 |
| `crates/engine-elements-truss/tests/invariants.rs` | 1 | 6 | 1 | 8 |
| `crates/engine-geo/tests/consolidation.rs` | 0 | 5 | 6 | 11 |
| `crates/engine-geo/tests/py.rs` | 1 | 9 | 0 | 10 |
| `crates/engine-geo/tests/support/axial_benchmarks.rs` | 2 | 6 | 0 | 8 |
| `crates/engine-geo/tests/tz.rs` | 1 | 1 | 1 | 3 |
| `crates/engine-laws/tests/cubic_elastic.rs` | 3 | 8 | 1 | 12 |
| `crates/engine-laws/tests/elastic.rs` | 1 | 2 | 2 | 5 |
| `crates/engine-laws/tests/multilinear.rs` | 1 | 4 | 1 | 6 |
| `crates/engine-laws/tests/steel_bilinear.rs` | 2 | 13 | 2 | 17 |
| `crates/engine-linalg/tests/budget.rs` | 0 | 0 | 5 | 5 |
| `crates/engine-linalg/tests/conformance.rs` | 0 | 0 | 1 | 1 |
| `crates/engine-linalg/tests/dependency_boundary.rs` | 0 | 0 | 1 | 1 |
| `crates/engine-linalg/tests/determinism_guard.rs` | 0 | 0 | 10 | 10 |
| `crates/engine-linalg/tests/guyan.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-linalg/tests/heap_peak.rs` | 0 | 0 | 4 | 4 |
| `crates/engine-linalg/tests/lowest_modes_anti_revert.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-linalg/tests/modes.rs` | 0 | 0 | 2 | 2 |
| `crates/engine-linalg/tests/robustness.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-loads/tests/fixtures/mod.rs` | 1 | 0 | 1 | 2 |
| `crates/engine-loads/tests/mass.rs` | 1 | 17 | 1 | 19 |
| `crates/engine-loads/tests/nodal.rs` | 0 | 1 | 0 | 1 |
| `crates/engine-loads/tests/span.rs` | 0 | 6 | 0 | 6 |
| `crates/engine-loads/tests/weight.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-wasm/tests/i18n.rs` | 0 | 0 | 0 | 0 |
| `crates/engine-wasm/tests/sfc010.rs` | 1 | 0 | 0 | 1 |
| `crates/engine-contracts/src/tests.rs` | 0 | 6 | 1 | 7 |
| `crates/engine-geo/src/tests.rs` | 0 | 13 | 0 | 13 |
| `crates/engine-model/src/tests.rs` | 0 | 0 | 1 | 1 |
| **Total** | **82** | **434** | **153** | **669** |

Code tokens, identifiers, logic, statement order, macro arguments, format
placeholders, stable ENG-E-* codes, case names, data labels, JSON keys, file
names, and successful output remain unchanged. Only the listed diagnostic
messages change at runtime, on assertion/expect failure or panic. Crate and
module rustdoc is documentation, with no runtime export of these comments.
The CLI and API source timestamps were refreshed before verification without
changing their contents. Protected files and the entire tranche 1 record are
unchanged. No commit was created.

### Terminology (tranche 2)

Terminology follows tranche 1, ENG-LANG-004, and ENG-LANG-005: DOF, node,
member, beam, truss, spring, fixture, golden, oracle, station, prescribed
displacement, load case, mode shape, base shear, recovery, and end actions.
Geotechnical wording uses settlement, pressure, flux, foundation, and tributary
pile length. Project derivations and code use own or in-house; existing uses
of independent that describe separate calculations or input-order independence
are preserved. The words fixture and fixtures retain their form. Existing
units, equations, API references, citations, and symbolic references retain
their meaning. Behavioral doc comments use indicative mood.

### String literals changed (tranche 2)

Before any source literal changed, a repository-wide `rg --hidden --no-ignore`
literal-text search covered all 169 candidate occurrences (155 distinct old
literals), including hidden CI files, tests, goldens, JSON, scripts, clean-room
files, docs, and production catalogs. Git metadata, build output, and both
explicitly prohibited repository documents were excluded. No prohibited
external location was accessed. Only matching text was returned, avoiding
unrelated reference data. Four short expect literals were retained after this
review, as were eleven messages in the full-file hash-pinned test and one
message consumed by a same-file `should_panic` expectation; 153 occurrences
were translated. No literal comparison or compared
output consuming the translated messages was found. Repeated rejection text
occurs only in other test diagnostics; the axial message also occurs in the
tranche 1 record. The cubic-elastic assertion phrase also appears in a template
assertion diagnostic, without a literal comparison or output contract.

| File:line | Old -> new |
| --- | --- |
| `crates/engine-analysis/tests/combination.rs:85` | `{actual} != {expected}, tolerancia {tolerance}` -> `{actual} != {expected}, tolerance {tolerance}` |
| `crates/engine-analysis/tests/combination.rs:494` | `factores={factors:?}, componente={index}: {actual} != {direct}` -> `factors={factors:?}, component={index}: {actual} != {direct}` |
| `crates/engine-analysis/tests/modal.rs:308` | `cantilever={cantilever}, modo={i}, errores={ce},{fe}` -> `cantilever={cantilever}, mode={i}, errors={ce},{fe}` |
| `crates/engine-analysis/tests/pdelta.rs:222` | `factor lateral={lateral_factor}` -> `lateral factor={lateral_factor}` |
| `crates/engine-analysis/tests/results.rs:442` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-api/tests/determinism.rs:17` | `{name}, corrida {iteration}` -> `{name}, run {iteration}` |
| `crates/engine-api/tests/determinism.rs:18` | `{name}, corrida {iteration}` -> `{name}, run {iteration}` |
| `crates/engine-api/tests/determinism.rs:110` | `{name}: orientación ignorada` -> `{name}: orientation ignored` |
| `crates/engine-api/tests/dynamic.rs:261` | `El fixture debe activar bloques 2×2; no valida copysign de #78` -> `The fixture must activate 2×2 blocks; does not validate copysign from #78` |
| `crates/engine-api/tests/input_fixtures.rs:17` | `{path:?}: huérfano` -> `{path:?}: orphan` |
| `crates/engine-api/tests/input_fixtures.rs:19` | `Fixture inesperado: {path:?}` -> `Unexpected fixture: {path:?}` |
| `crates/engine-api/tests/input_fixtures.rs:23` | `{category}: faltan fixtures` -> `{category}: missing fixtures` |
| `crates/engine-api/tests/input_fixtures.rs:51` | `{path:?}: faltan resultados combinados` -> `{path:?}: missing combined results` |
| `crates/engine-api/tests/input_fixtures.rs:91` | `{path:?}: validate {actual:?}; esperado {kind:?}` -> `{path:?}: validate {actual:?}; expected {kind:?}` |
| `crates/engine-api/tests/input_fixtures.rs:103` | `{path:?}: esperado {}, recibido {document}` -> `{path:?}: expected {}, received {document}` |
| `crates/engine-api/tests/input_names.rs:80` | `La lista clean-room no puede estar vacía` -> `The clean-room list cannot be empty` |
| `crates/engine-api/tests/input_names.rs:131` | `String JSON sin cierre` -> `Unclosed JSON string` |
| `crates/engine-api/tests/input_names.rs:284` | `Contenido después del documento JSON` -> `Content after the JSON document` |
| `crates/engine-api/tests/input_names.rs:312` | `{path:?}: faltan documentos` -> `{path:?}: missing documents` |
| `crates/engine-api/tests/pdelta_diaphragm.rs:21` | `se requiere un único run E3-pdelta` -> `A single E3-pdelta run is required` |
| `crates/engine-api/tests/pdelta_diaphragm.rs:47` | `regenerar referencia al cambiar el fixture` -> `Regenerate the reference when changing the fixture` |
| `crates/engine-api/tests/pdelta_diaphragm.rs:52` | `regenerar referencia al cambiar el script independiente` -> `Regenerate the reference when changing the independent script` |
| `crates/engine-api/tests/pdelta_diaphragm.rs:125` | `piso {}: {count} elem, amp {amp_error:e}` -> `floor {}: {count} elem, amp {amp_error:e}` |
| `crates/engine-api/tests/pdelta_diaphragm.rs:130` | `piso {}: {count} elem, deriva {drift_error:e}` -> `floor {}: {count} elem, drift {drift_error:e}` |
| `crates/engine-api/tests/pdelta_diaphragm.rs:148` | `piso {}: {chord_excess:e}` -> `floor {}: {chord_excess:e}` |
| `crates/engine-api/tests/results.rs:351` | `No detectó mutación de {field}` -> `Did not detect mutation of {field}` |
| `crates/engine-api/tests/schema_publication.rs:18` | `{kind} sin publicar` -> `{kind} unpublished` |
| `crates/engine-api/tests/schema_publication.rs:27` | `fuente duplicada` -> `duplicate source` |
| `crates/engine-api/tests/schema_publication.rs:45` | `árbol de esquemas desactualizado` -> `outdated schema tree` |
| `crates/engine-api/tests/support/determinism.rs:11` | `SHA-256 hexadecimal del SDK` -> `SDK hexadecimal SHA-256` |
| `crates/engine-api/tests/support/mod.rs:15` | `results/1 fuera de schema: {value}` -> `results/1 outside schema: {value}` |
| `crates/engine-api/tests/support/schema.rs:40` | `Keyword de schema no soportada: {key}` -> `Unsupported schema keyword: {key}` |
| `crates/engine-api/tests/support/schema.rs:63` | `Referencia externa local ausente` -> `Missing local external reference` |
| `crates/engine-api/tests/support/schema.rs:70` | `Referencia local ausente` -> `Missing local reference` |
| `crates/engine-api/tests/support/schema.rs:77` | `type inválido` -> `invalid type` |
| `crates/engine-api/tests/support/schema.rs:155` | `Keyword de schema no soportada: {key}` -> `Unsupported schema keyword: {key}` |
| `crates/engine-api/tests/support/schema.rs:168` | `Tipo de schema no soportado: {name}` -> `Unsupported schema type: {name}` |
| `crates/engine-api/tests/support/schema.rs:226` | `Expresión de schema no soportada: {expression}` -> `Unsupported schema expression: {expression}` |
| `crates/engine-assembly/tests/assembly.rs:20` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-assembly/tests/diaphragm.rs:24` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-assembly/tests/diaphragm.rs:588` | `convergencia: {errors:?}` -> `convergence: {errors:?}` |
| `crates/engine-assembly/tests/geometric_producers.rs:57` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-assembly/tests/geometric_producers.rs:90` | `sobrescribe, no acumula` -> `overwrites, does not accumulate` |
| `crates/engine-assembly/tests/geometric_producers.rs:96` | `la API global conserva la instantánea` -> `the global API preserves the snapshot` |
| `crates/engine-assembly/tests/geometric_producers.rs:142` | `Ubicación inválida no se evalúa` -> `Invalid location is not evaluated` |
| `crates/engine-assembly/tests/geometric_producers.rs:248` | `el registro reemplaza también el patrón dinámico` -> `registration also replaces the dynamic pattern` |
| `crates/engine-assembly/tests/j4_followup.rs:13` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-assembly/tests/j4_followup.rs:53` | `salida intacta ante error` -> `output intact on error` |
| `crates/engine-assembly/tests/j4_followup.rs:132` | `se simetriza con la media` -> `symmetrized with the mean` |
| `crates/engine-assembly/tests/solve_cases.rs:80` | `solve sigue factorizando cada llamada` -> `solve still factorizes on every call` |
| `crates/engine-bench/tests/models.rs:18` | `no se pudo leer {relative}: {error}` -> `could not read {relative}: {error}` |
| `crates/engine-bench/tests/models.rs:96` | `Sa debe estar en m/s^2, no en g` -> `Sa must be in m/s^2, not in g` |
| `crates/engine-bench/tests/models.rs:317` | `el puntero estático apunta al segundo caso` -> `the static pointer points to the second case` |
| `crates/engine-cli/tests/cli.rs:96` | `Error al escribir stdin del hijo: {error}` -> `Error writing child stdin: {error}` |
| `crates/engine-cli/tests/cli.rs:112` | `stdout de fallo debe estar vacío` -> `failure stdout must be empty` |
| `crates/engine-cli/tests/cli.rs:172` | `El timeout no terminó en {limit:?}: {output:?}` -> `The timeout did not finish within {limit:?}: {output:?}` |
| `crates/engine-cli/tests/cli.rs:353` | `Línea stderr debe ser JSON completo` -> `The stderr line must be complete JSON` |
| `crates/engine-cli/tests/cli.rs:379` | `Debe comprobarse también un evento de progreso` -> `A progress event must also be checked` |
| `crates/engine-cli/tests/contract.rs:58` | `grupo de fixtures vacío: {group}` -> `empty fixture group: {group}` |
| `crates/engine-cli/tests/contract.rs:117` | `metadato: {key}` -> `metadata: {key}` |
| `crates/engine-cli/tests/contract.rs:149` | `token de stdout ausente: {old}` -> `missing stdout token: {old}` |
| `crates/engine-cli/tests/contract.rs:156` | `stdout debe terminar con LF` -> `stdout must end with LF` |
| `crates/engine-cli/tests/contract.rs:158` | `stdout no admite CRLF` -> `stdout does not allow CRLF` |
| `crates/engine-cli/tests/contract.rs:173` | `SHA-256 de stdout requiere Python 3 (véase README)` -> `SHA-256 of stdout requires Python 3 (see README)` |
| `crates/engine-cli/tests/contract.rs:212` | `La regeneración de golden está prohibida en CI` -> `Golden regeneration is prohibited in CI` |
| `crates/engine-cli/tests/contract.rs:216` | `MAMUT_UPDATE_CLI_GOLDENS solo admite 1 o ausencia` -> `MAMUT_UPDATE_CLI_GOLDENS only allows 1 or absence` |
| `crates/engine-cli/tests/contract.rs:241` | `{}: {error}; véase README para regenerar` -> `{}: {error}; see README to regenerate` |
| `crates/engine-cli/tests/contract.rs:246` | `versión de manifiesto no soportada` -> `unsupported manifest version` |
| `crates/engine-cli/tests/contract.rs:267` | `{name}/{operation}: fuera de cli-run/1` -> `{name}/{operation}: outside cli-run/1` |
| `crates/engine-cli/tests/contract.rs:297` | `caso ausente: {name}; véase README para regenerar` -> `missing case: {name}; see README to regenerate` |
| `crates/engine-cli/tests/contract.rs:301` | `{name}: SHA-256 esperado {}, obtenido {}; bytes esperados {}, obtenidos {}` -> `{name}: expected SHA-256 {}, got {}; expected bytes {}, got {}` |
| `crates/engine-cli/tests/contract.rs:312` | `{name}: no determinista` -> `{name}: nondeterministic` |
| `crates/engine-cli/tests/contract.rs:330` | `casos duplicados en manifiesto` -> `duplicate cases in manifest` |
| `crates/engine-cli/tests/contract.rs:331` | `golden ausente u obsoleto` -> `missing or obsolete golden` |
| `crates/engine-cli/tests/contract.rs:335` | `solo se versiona stdout-sha256.json` -> `only stdout-sha256.json is versioned` |
| `crates/engine-cli/tests/contract.rs:347` | `campo obligatorio: {key}` -> `required field: {key}` |
| `crates/engine-cli/tests/contract.rs:364` | `mutación aceptada: {pointer}` -> `mutation accepted: {pointer}` |
| `crates/engine-cli/tests/error_registry.rs:102` | `Código borrado o definición congelada modificada` -> `Code deleted or frozen definition changed` |
| `crates/engine-cli/tests/error_registry.rs:115` | `Salida sin categoría: {definition}` -> `Exit without category: {definition}` |
| `crates/engine-cli/tests/error_registry.rs:172` | `Mutación no detectada: {field}` -> `Mutation not detected: {field}` |
| `crates/engine-cli/tests/error_registry.rs:191` | `Deprecated reutilizado: {field}` -> `Deprecated reused: {field}` |
| `crates/engine-cli/tests/error_registry.rs:201` | `ID deprecated duplicado: {status}` -> `Duplicate deprecated ID: {status}` |
| `crates/engine-cli/tests/error_registry.rs:252` | `Edición coordinada: {position}/{field}` -> `Coordinated edit: {position}/{field}` |
| `crates/engine-cli/tests/error_registry.rs:311` | `raw string sin cierre` -> `unclosed raw string` |
| `crates/engine-cli/tests/error_registry.rs:434` | `Código sin registro en {}: {code}` -> `Unregistered code in {}: {code}` |
| `crates/engine-cli/tests/error_registry.rs:455` | `El inventario de emisores quedó vacío o incompleto` -> `The emitter inventory is empty or incomplete` |
| `crates/engine-cli/tests/error_schema.rs:29` | `Campo obligatorio no comprobado: {field}` -> `Required field not checked: {field}` |
| `crates/engine-cli/tests/error_schema.rs:57` | `Mutación no detectada: {invalid}` -> `Mutation not detected: {invalid}` |
| `crates/engine-cli/tests/support/schema.rs:11` | `Sobre sin terminador NDJSON: {text}` -> `Envelope without NDJSON terminator: {text}` |
| `crates/engine-cli/tests/support/schema.rs:17` | `cli-error/1 fuera de schema: {value}` -> `cli-error/1 outside schema: {value}` |
| `crates/engine-cli/tests/support/schema.rs:22` | `Progreso tras el sobre terminal` -> `Progress after the terminal envelope` |
| `crates/engine-cli/tests/support/schema.rs:28` | `progress/1 fuera de schema: {value}` -> `progress/1 outside schema: {value}` |
| `crates/engine-cli/tests/support/schema.rs:33` | `Emisor ajeno al contrato: {value}` -> `Emitter outside contract: {value}` |
| `crates/engine-cli/tests/support/schema.rs:41` | `Debe haber un único error terminal: {text}` -> `There must be a single terminal error: {text}` |
| `crates/engine-cli/tests/support/schema.rs:55` | `Código no registrado` -> `Unregistered code` |
| `crates/engine-cli/tests/support/schema.rs:72` | `Diagnóstico no registrado: {diagnostic}` -> `Unregistered diagnostic: {diagnostic}` |
| `crates/engine-cli/tests/support/validate_report.rs:11` | `Informe fuera de esquema` -> `Report outside schema` |
| `crates/engine-cli/tests/support/validate_report.rs:31` | `Requerido: {pointer}/{key}` -> `Required: {pointer}/{key}` |
| `crates/engine-elements-frame/tests/invariants.rs:26` | `debe fallar` -> `must fail` |
| `crates/engine-elements-frame/tests/modes.rs:86` | `modo {}: f={}, exacta={exact}, error={fine_error}` -> `mode {}: f={}, exact={exact}, error={fine_error}` |
| `crates/engine-elements-frame/tests/modes.rs:92` | `refinamiento modo {}: gruesa={coarse_error}, fina={fine_error}` -> `mode refinement {}: coarse={coarse_error}, fine={fine_error}` |
| `crates/engine-elements-frame3d/tests/invariants.rs:14` | `debe fallar` -> `must fail` |
| `crates/engine-elements-frame3d/tests/r6.rs:16` | `orientación {hint:?} debe rechazarse` -> `orientation {hint:?} must be rejected` |
| `crates/engine-elements-frame3d/tests/r6.rs:36` | `fuerza espuria {f}` -> `spurious force {f}` |
| `crates/engine-elements-frame3d/tests/r6.rs:78` | `Kg se anuló por subdesbordamiento de E·ε` -> `Kg was zeroed by underflow of E·ε` |
| `crates/engine-elements-spring/tests/invariants.rs:18` | `debe fallar` -> `must fail` |
| `crates/engine-elements-spring/tests/invariants.rs:628` | `la fábrica debe rechazar una lista vacía de GDL` -> `the factory must reject an empty DOF list` |
| `crates/engine-elements-spring/tests/nonlinear.rs:35` | `Debe producir diagnóstico` -> `Must produce a diagnostic` |
| `crates/engine-elements-truss/tests/geometric_producer.rs:89` | `Kg(0) coincide con la estática nula` -> `Kg(0) matches the zero static matrix` |
| `crates/engine-elements-truss/tests/geometric_producer.rs:151` | `Kg(0) coincide también con estática no nula` -> `Kg(0) also matches the nonzero static matrix` |
| `crates/engine-elements-truss/tests/geometric_producer.rs:289` | `error relativo={error}, tol={FD_REL_TOL}` -> `relative error={error}, tol={FD_REL_TOL}` |
| `crates/engine-elements-truss/tests/geometric_producer.rs:294` | `error transversal={transverse_error}` -> `transverse error={transverse_error}` |
| `crates/engine-elements-truss/tests/invariants.rs:20` | `debe fallar` -> `must fail` |
| `crates/engine-geo/tests/consolidation.rs:224` | `sin salidas intermedias` -> `without intermediate outputs` |
| `crates/engine-geo/tests/consolidation.rs:227` | `con salidas en quiebres` -> `with outputs at breakpoints` |
| `crates/engine-geo/tests/consolidation.rs:370` | `las tres capas compatibles deben aceptarse` -> `all three compatible layers must be accepted` |
| `crates/engine-geo/tests/consolidation.rs:379` | `contexto de discrepancia` -> `mismatch context` |
| `crates/engine-geo/tests/consolidation.rs:409` | `razón 1+5e-11 debe aceptarse` -> `ratio 1+5e-11 must be accepted` |
| `crates/engine-geo/tests/consolidation.rs:413` | `contexto de discrepancia` -> `mismatch context` |
| `crates/engine-geo/tests/tz.rs:30` | `Fixtures axiales` -> `Axial fixtures` |
| `crates/engine-laws/tests/cubic_elastic.rs:48` | `{actual} frente a {expected}, límite={limit}; {}: {}` -> `{actual} versus {expected}, limit={limit}; {}: {}` |
| `crates/engine-laws/tests/elastic.rs:117` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-laws/tests/elastic.rs:269` | `Campo de prueba inválido` -> `Invalid test field` |
| `crates/engine-laws/tests/multilinear.rs:54` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-laws/tests/steel_bilinear.rs:14` | `actual={actual}, esperado={expected}` -> `actual={actual}, expected={expected}` |
| `crates/engine-laws/tests/steel_bilinear.rs:22` | `Se esperaba {expected}` -> `Expected {expected}` |
| `crates/engine-linalg/tests/budget.rs:119` | `fallo simulado del backend` -> `simulated backend failure` |
| `crates/engine-linalg/tests/budget.rs:146` | `el pánico se propaga al llamador` -> `the panic propagates to the caller` |
| `crates/engine-linalg/tests/budget.rs:150` | `used no queda cargado tras el pánico` -> `used does not remain charged after the panic` |
| `crates/engine-linalg/tests/budget.rs:154` | `peak documenta el conjunto de trabajo cargado` -> `peak documents the charged working set` |
| `crates/engine-linalg/tests/budget.rs:194` | `peak incluye el trabajo transitorio` -> `peak includes transient work` |
| `crates/engine-linalg/tests/conformance.rs:66` | `Se esperaba {code}` -> `Expected {code}` |
| `crates/engine-linalg/tests/dependency_boundary.rs:152` | `Proveedor numérico fuera de engine-linalg: {}` -> `Numerical provider outside engine-linalg: {}` |
| `crates/engine-linalg/tests/determinism_guard.rs:598` | `La guardia no encontró fuentes` -> `The guard found no sources` |
| `crates/engine-linalg/tests/determinism_guard.rs:607` | `Colección hash o concurrencia prohibida en {}` -> `Hash collection or concurrency prohibited in {}` |
| `crates/engine-linalg/tests/determinism_guard.rs:672` | `La guardia no encontró fuentes` -> `The guard found no sources` |
| `crates/engine-linalg/tests/determinism_guard.rs:684` | `Función escalar, FMA, include! o #[path] fuera de política en {}` -> `Scalar function, FMA, include!, or #[path] outside policy in {}` |
| `crates/engine-linalg/tests/determinism_guard.rs:689` | `include! o #[path] fuera del inventario Rust en {}` -> `include! or #[path] outside the Rust inventory in {}` |
| `crates/engine-linalg/tests/determinism_guard.rs:709` | `Excepción de tests/examples sin archivo inspeccionado o justificación` -> `Test/example exception without inspected file or justification` |
| `crates/engine-linalg/tests/determinism_guard.rs:717` | `Construcción prohibida o excepción obsoleta en {}` -> `Prohibited construct or obsolete exception in {}` |
| `crates/engine-linalg/tests/determinism_guard.rs:724` | `Biblioteca sin #![forbid(unsafe_code)] en {}` -> `Library without #![forbid(unsafe_code)] in {}` |
| `crates/engine-linalg/tests/determinism_guard.rs:729` | `La guardia no encontró bibliotecas` -> `The guard found no libraries` |
| `crates/engine-linalg/tests/determinism_guard.rs:736` | `Binding Python sin #![forbid(unsafe_code)]` -> `Python binding without #![forbid(unsafe_code)]` |
| `crates/engine-linalg/tests/heap_peak.rs:32` | `medición anidada` -> `nested measurement` |
| `crates/engine-linalg/tests/heap_peak.rs:224` | `factor retenido n={n}: {retained}` -> `retained factor n={n}: {retained}` |
| `crates/engine-linalg/tests/heap_peak.rs:237` | `la cota vieja debe subestimar algún pico medido` -> `the old bound must underestimate some measured peak` |
| `crates/engine-linalg/tests/heap_peak.rs:258` | `modos retenidos n={n}: {retained}` -> `retained modes n={n}: {retained}` |
| `crates/engine-linalg/tests/modes.rs:69` | `M-ortogonalidad: {dot}` -> `M-orthogonality: {dot}` |
| `crates/engine-linalg/tests/modes.rs:113` | `Debe consultar dentro de los kernels: {checks}` -> `Must check inside the kernels: {checks}` |
| `crates/engine-loads/tests/fixtures/mod.rs:12` | `{actual} != {expected}, tolerancia {tol}` -> `{actual} != {expected}, tolerance {tol}` |
| `crates/engine-loads/tests/mass.rs:343` | `Entidad inesperada: {actual:?}; se esperaba {expected:?}` -> `Unexpected entity: {actual:?}; expected {expected:?}` |
| `crates/engine-contracts/src/tests.rs:27` | `No detectó {text}: {diagnostics:#?}` -> `Did not detect {text}: {diagnostics:#?}` |
| `crates/engine-model/src/tests.rs:67` | `El modelo no emite referencias a índices globales` -> `The model does not emit global index references` |

### Left in Spanish (tranche 2)

All other literals remain byte-identical, including test inputs, labels, case
names, JSON keys and values, file names, expected-error substrings, production
i18n test text, table citations, custom `require`/`check` labels, returned
EngineError diagnostics, printed output, and `expect_err` text outside the
explicit allowance. In particular, the literal sequences in
`crates/engine-api/tests/i18n.rs`, `crates/engine-cli/tests/i18n.rs`, and
`crates/engine-wasm/tests/i18n.rs` are unchanged. CLI assertions that compare
Spanish output retain their expected values; only the separately listed failure
messages change.

The hash-pinned `crates/engine-api/tests/val032_runs.rs` remains entirely
byte-identical: comments and messages cannot change without changing its
protected `docs/validation-report/fixtures/val032/index.json` contract.
Workspace verification exposed this constraint; restoring the source preserves
the numerical reference and source-digest checks. Its messages are included in
the retained-literal inventory below.

The expect message `Código reutilizado` at
`crates/engine-cli/tests/error_registry.rs:65` is also retained: the same
file's `should_panic(expected = "Código reutilizado")` compares it literally.
The workspace rerun caught the missed same-file consumer; the final inventory
excludes that substitution. All repository search hits, including same-file
hits, were rechecked before the final verification run.

Four other eligible expect messages remain after the repository-wide use review:

| Source | Literal | Reason |
| --- | --- | --- |
| `crates/engine-geo/tests/consolidation.rs:26` | `perfil` | Short label also occurs in production validation, JSON, and protected contract prose; retained conservatively. |
| `crates/engine-geo/tests/consolidation.rs:262` | `interpolación` | Shared with fixture metadata, citation literals, and prose; retained conservatively. |
| `crates/engine-geo/tests/consolidation.rs:325` | `contexto` | Shared diagnostic and contract vocabulary; literal/output independence is uncertain. |
| `crates/engine-geo/tests/consolidation.rs:516` | `estable` | Also appears in production i18n and compared JSON output; retained conservatively. |

The inventory below lists every source line containing unchanged literals,
including English and neutral literals, so short Spanish labels missed by a
language heuristic are covered. Multiple occurrences on one line share a
location. No retained string literal is omitted. Existing character literals
also remain unchanged. Source files without retained strings need no entry.

| Source file | Lines containing unchanged literals | Reason |
| --- | --- | --- |
| `crates/engine-analysis/tests/analytical.rs` | 3, 25, 39, 138, 139, 140, 151, 192, 193, 194, 195, 207, 224, 235, 236, 244, 259, 287, 312, 316, 337, 349, 371, 375, 376, 377, 396, 397, 398, 399, 400, 401, 402, 407, 410, 415, 418, 435, 436 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/bench_pdelta.rs` | 43, 121, 122, 130, 131, 138, 246, 257, 271, 287, 295 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/combination.rs` | 17, 73, 91, 92, 94, 99, 103, 104, 115, 121, 122, 126, 145, 150, 151, 152, 250, 255, 323, 327, 342, 346, 348, 350, 361, 362, 363, 367, 390, 396, 402, 407, 417, 419, 421, 423, 425, 431, 437, 444, 455, 456, 480, 521, 608, 609, 619, 624, 634, 635, 636, 639, 640, 682, 686, 691, 697, 698, 726, 743, 744, 765, 769, 778, 783, 786, 787, 791, 796, 799, 802, 806, 810, 811, 815, 818, 821, 826, 828, 836, 842, 845, 854, 861, 864, 871, 872, 873, 877, 880, 882, 884, 886, 889, 897, 903, 916, 932 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/diaphragm.rs` | 2, 4, 113, 162 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/modal.rs` | 15, 19, 102, 140, 399, 416, 417, 481, 482, 483, 484, 485, 486, 507, 508, 577 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/newton.rs` | 3, 22, 25, 35, 129, 158, 165, 243, 278, 305, 514, 515, 557, 574, 581, 586, 592, 600, 620, 661, 690, 760, 767, 777 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/pdelta.rs` | 2, 44, 58, 122, 123, 134, 173, 174, 205, 206, 264, 266, 269, 296, 302, 303, 347, 351, 413, 417, 471, 472, 519, 520, 566, 594, 595, 629, 643, 661, 663, 719, 734, 735, 781, 782, 801, 802, 806, 807, 810, 811, 815, 816, 819, 820, 823, 824, 829, 830, 836, 841, 842, 846, 847, 857, 858, 863, 864, 867, 880, 881, 906, 932, 933, 939, 949, 980, 981, 1023, 1030, 1037, 1040, 1054, 1055, 1066, 1067 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/results.rs` | 27, 31, 42, 50, 103, 118, 436, 437, 609, 611, 627, 631, 634, 637, 640, 649, 659, 670, 673, 674, 675, 708, 718, 720, 729, 750, 911 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/spectrum.rs` | 22, 46, 69, 105, 168, 183, 191, 206, 208, 223, 236, 248, 255, 325, 605, 612, 618, 630, 632, 644, 648, 650, 669, 679, 685, 691, 697, 700, 701, 702, 703, 704, 715, 742, 793 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-analysis/tests/unit/pdelta_projection.rs` | 3, 59 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/determinism.rs` | 2, 13, 31, 33, 34, 36, 37, 45, 46, 47, 57, 67, 68, 75, 77, 81, 82, 83, 84, 85, 86, 91, 94, 95, 96, 102, 103, 106, 117, 119, 126, 132, 134 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/dynamic.rs` | 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 23, 24, 25, 26, 27, 28, 35, 36, 37, 38, 40, 43, 45, 52, 56, 59, 60, 61, 63, 66, 67, 68, 71, 72, 73, 74, 82, 85, 86, 87, 89, 90, 92, 98, 101, 102, 103, 105, 108, 109, 112, 113, 119, 120, 121, 123, 124, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 149, 155, 157, 159, 160, 162, 163, 164, 165, 167, 170, 171, 172, 173, 174, 180, 182, 184, 186, 189, 191, 195, 211, 212, 217, 218, 219, 220, 223, 224, 226, 227, 232, 236, 237, 239, 241, 242, 249, 252, 256, 267, 268, 269, 270, 272, 273, 274, 275, 277, 278, 282, 283, 287, 288, 293, 294, 296, 297, 300, 301, 302, 303, 304, 305, 307, 309, 310, 311, 317, 319, 320, 323, 327, 328, 332, 333, 335, 337, 341, 349, 350, 352, 354, 356, 359, 362, 363, 366, 367, 379, 389, 390, 394, 407, 422, 426, 427, 428, 429, 430, 431, 435, 442, 443, 446, 447, 451, 454, 457, 462, 467, 468, 472, 477, 482, 483, 485, 488, 494, 495, 496, 499, 501 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/facade.rs` | 9, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 22, 23, 24, 31, 33, 34, 35, 37, 40, 41, 43, 44, 52, 53, 54, 55, 56, 58, 64, 69, 79, 80, 81, 82, 83, 84, 85, 89, 93, 94, 97, 101, 102, 115, 116, 117, 118, 119, 125, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 143, 146, 150, 151, 152, 153, 154, 155, 156, 162, 170, 171, 172, 173, 174, 175, 176, 177, 183, 191, 192, 193, 195, 202, 203, 204, 205, 206, 207, 208, 209, 215, 216, 217, 218, 219, 220, 221, 224, 225, 226, 229, 240, 241, 242, 246, 256, 257, 264, 265, 266, 270, 271, 272, 276, 277, 279, 280, 285, 286, 287, 291, 293, 294, 316, 331, 337, 358, 375, 383 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/i18n.rs` | 4, 17, 21, 29, 39, 45, 47, 57, 61 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/input_fixtures.rs` | 8, 9, 15, 16, 17, 29, 31, 32, 36, 37, 38, 39, 43, 44, 45, 46, 47, 49, 50, 58, 75, 78, 87, 88, 94, 95, 98, 102 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/input_names.rs` | 4, 5, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 48, 50, 51, 52, 53, 54, 55, 56, 57, 61, 63, 64, 65, 66, 67, 68, 69, 197, 199, 200, 203, 204, 225, 227, 237, 247, 248, 266, 289, 295, 296, 318, 325, 326, 327, 328, 330, 341, 346, 353, 356, 363, 370, 371, 375, 384, 386, 387, 389, 395, 396, 397, 398, 401, 403, 411, 424, 425, 426, 427, 428, 429, 430, 431, 432, 433, 434, 435, 436, 437, 438, 443, 444, 447, 448, 450, 453, 463, 464, 465, 466, 467, 468, 469, 473, 474, 478, 479, 481, 483, 486, 491, 494, 503, 506, 511, 513, 521, 523, 524, 527, 528, 531, 532, 535, 539, 540, 547, 548, 549, 550, 551, 552, 556, 559, 560, 561, 563, 567, 568, 575, 576, 577, 578, 579, 580, 581, 582, 586, 593, 604, 610, 614, 615 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/limits.rs` | 16, 24, 25, 26, 27, 36, 38, 43, 47, 53, 55, 56, 57, 60, 62, 63, 64, 67, 69, 70, 71, 75, 76, 77, 81, 82, 86, 93, 96, 97, 104, 105, 106, 109, 111, 112, 113, 116, 118, 126, 129, 131, 133, 134, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 152, 155, 156, 159, 160, 169, 174, 176, 180, 182, 185, 190, 191, 192, 202, 203, 205, 206, 211 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/methods.rs` | 9, 10, 11, 15, 22, 27, 28, 29, 31, 34, 36, 37, 38, 45, 53, 54, 57, 62, 66, 67, 70, 72, 73, 103, 104, 105, 106, 107, 108, 109, 110, 113, 116, 122, 127, 128, 130, 133, 134, 136, 141, 144, 146, 147, 149, 151, 154, 155, 158, 161, 168, 169, 170, 171, 172, 173, 174, 175, 182, 183, 187, 189, 193, 196, 200, 209, 210, 212, 213, 218, 223, 225, 228, 229, 230, 231, 234, 237, 238, 239, 240, 241, 242, 244 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/pdelta_diaphragm.rs` | 6, 8, 10, 15, 19, 22, 28, 30, 31, 32, 33, 34, 35, 36, 39, 40, 43, 45, 50, 55, 59, 60, 68, 72, 74, 78, 79, 80, 81, 85, 88, 90, 94, 96, 97, 99, 106, 107, 110, 112, 115, 116, 119, 120, 121, 133, 137, 140, 141, 152, 163, 164 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/results.rs` | 9, 10, 11, 12, 13, 23, 24, 27, 28, 31, 32, 33, 34, 37, 38, 41, 45, 55, 57, 60, 64, 75, 79, 83, 86, 90, 94, 100, 110, 111, 112, 113, 114, 121, 142, 143, 144, 145, 146, 147, 156, 157, 170, 199, 200, 201, 207, 208, 209, 215, 216, 218, 219, 222, 223, 226, 227, 230, 231, 245, 263, 264, 265, 270, 271, 273, 275, 276, 279, 280, 288, 289, 300, 304, 306, 311, 320, 323, 324, 325, 326, 327, 329, 330, 333, 334, 337, 338, 341, 342, 344, 345, 355, 359, 360 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/run.rs` | 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 32, 44, 47, 48, 49, 50, 51, 59, 60, 61, 62, 63, 65, 68, 73, 77, 85, 86, 88, 91, 92, 93, 94, 98, 99, 100, 107, 108, 109, 112, 120, 123, 125, 128, 129, 131, 144, 147, 156, 157, 158, 159, 160, 161, 162, 165, 169, 171, 174, 175, 177, 178, 184, 185, 186, 193, 196, 209, 212, 215, 223, 224, 225, 226, 227, 228, 229, 235, 240, 242, 245, 246, 249, 256, 258, 260, 266, 267, 268, 270, 272, 276, 279, 282, 283, 286, 289, 291, 292, 299, 300, 301, 303, 304, 307, 308, 311, 312, 320, 324, 349, 350, 351, 352, 353, 354, 355, 356, 359, 360, 362, 363, 364, 365, 371, 372, 374, 375, 376, 377, 378, 379, 381, 384, 388, 396, 397, 398, 400, 401, 402, 403, 404, 405, 408, 409, 411, 412, 415, 418, 420, 421, 427, 428, 429, 431, 432, 433, 434, 435, 436, 438, 439, 442, 445, 447, 455, 456, 457, 458, 459, 460, 462, 463, 465, 468, 478, 482, 486, 487, 488, 490, 491, 498, 499, 500, 502, 503, 504, 505, 506, 507, 509, 510, 511, 516, 520, 523, 530, 531, 532, 540, 541, 542, 543 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/schema_publication.rs` | 8, 14, 16, 22, 24, 35, 36, 43, 51, 52, 53 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/sfc010.rs` | 3, 21, 28, 34, 36, 42, 47, 48, 54, 55, 56, 57, 58, 59, 60, 61, 63, 64, 67, 68, 69, 72, 73, 75, 76, 80, 82, 83, 87, 89, 90, 93, 94, 96, 97, 102, 105, 106, 107, 108, 109, 110, 112, 113, 114, 118, 120, 121, 122, 123, 124, 125, 128, 129, 132, 134, 135, 136, 137, 140, 141, 152, 153, 154, 155, 156, 157, 158, 162, 164, 165, 166, 169, 170, 171, 182, 183, 185, 186, 189, 190, 192, 197, 201, 202, 207, 209, 210, 211, 212, 213, 216, 217, 225, 226, 230, 235, 239, 240, 242, 250, 253, 254, 255, 256, 262, 263, 264, 266, 267, 270, 271, 274, 275, 280, 281, 282, 283, 284, 285, 286, 291, 292, 293, 295, 296, 299, 300, 301, 309, 310, 311, 312, 313, 315, 320, 321, 323, 324, 329, 330, 331, 332, 333, 335, 337, 338, 342, 343, 344, 345, 346, 347, 349, 352, 353, 366, 368, 371, 372, 373, 382, 383, 385, 386, 387, 388, 389, 390, 393, 394, 395, 407, 412, 413, 415, 416, 417, 418, 420, 421, 422, 423, 424, 425, 426, 437, 443, 456, 457, 458, 459, 467, 477, 480, 481, 482, 483, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 500, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 527, 528, 529, 530, 531, 532, 533, 534, 535, 536, 537, 542, 543, 557, 558, 564, 565, 567, 569, 570, 571, 572, 573 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/support/determinism.rs` | 19, 20, 24, 25, 31, 32, 35, 36, 39, 40, 43, 44, 47, 48, 51, 52, 55, 56, 59, 60, 63, 64, 67, 68, 71, 72, 94, 98 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/support/mod.rs` | 21 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/support/scalar_cases.rs` | 6, 7, 8, 9, 10, 130, 133 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/support/schema.rs` | 16, 17, 18, 21, 22, 23, 26, 27, 29, 34, 39, 51, 52, 54, 56, 74, 79, 80, 85, 91, 97, 100, 105, 107, 112, 115, 121, 124, 127, 128, 129, 132, 137, 145, 147, 149, 161, 162, 163, 164, 165, 166, 167, 182, 188, 191, 192, 193, 195, 203, 204, 211 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/support/sfc010.rs` | 5, 6, 9, 10, 13, 14, 17, 18, 21, 22, 25, 26, 29, 30, 33, 34, 37, 38, 41, 42, 45, 46 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/val032_runs.rs` | 8, 9, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 58, 62, 63, 64, 68, 69, 70, 72, 74, 82, 87, 97, 98, 102, 104, 105, 107, 110, 114, 116, 118, 119, 121, 125, 127, 129, 130, 132, 133, 134, 135, 136, 139, 142, 145, 149, 154, 157, 160, 161, 166, 169, 170, 172, 173, 175, 179, 181, 182, 185, 187, 189, 190, 193, 194, 197, 199, 202, 203, 209, 211, 214, 216, 219, 221, 223, 227, 229, 233, 235, 238, 242, 246, 251, 252, 255, 256, 257, 261, 264, 271, 272, 273, 276, 277, 279, 283, 284, 288, 289, 292, 303, 309, 313, 317, 318, 321, 322, 325, 328, 329, 330, 334, 336, 340, 343, 350 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-api/tests/validate_performance.rs` | 20, 60, 62 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/assembly.rs` | 16, 21, 27, 39, 44, 117, 216, 227, 235, 236, 243, 247, 262, 283, 309, 310, 317, 321, 322, 327, 332, 334, 335, 336, 344, 347, 350, 355, 359, 365, 371, 377, 387, 390, 393, 399, 408, 415, 421, 427, 433, 437, 442, 448, 456, 462, 472, 482, 483, 490, 505, 517, 539, 540, 597, 598 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/diaphragm.rs` | 17, 20, 25, 64, 223, 299, 300, 302, 303, 320, 329, 336, 343, 350, 360, 366, 375, 388, 398, 402, 413, 422, 440, 690, 692, 697, 698, 709 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/fixtures/counting.rs` | 49 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/fixtures/mod.rs` | 14, 16, 24, 26, 38, 40, 49, 51 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/fixtures/rigid_cases.rs` | 13, 31, 39, 53, 96, 110, 111 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/geometric_producers.rs` | 17, 22, 58, 103, 119, 135, 136, 137, 149, 160, 177, 190, 213, 214, 215, 216, 217, 218, 225, 266, 269, 288, 298, 308 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/j4_followup.rs` | 14, 25, 52, 57, 58, 59, 76, 77, 84, 93, 99, 111, 123, 127, 137, 162, 184, 187, 199, 207, 208 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-assembly/tests/solve_cases.rs` | 2, 4, 21, 25, 65, 87, 89, 90, 92, 93, 94, 138, 139, 140, 141, 143, 145 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-bench/tests/capabilities.rs` | 8, 13, 15, 16, 31, 34, 36, 37, 41, 43, 44, 45, 46, 50, 52, 53, 54, 57, 58, 64, 72, 76, 78, 79, 80, 85, 87, 92, 94, 95, 96, 98, 102, 106, 109, 111, 113, 114, 115, 116, 121, 124, 125, 126, 127, 128, 129, 130, 131, 132, 136, 138, 141, 142, 152, 157, 160, 164, 166, 167, 168, 170, 171, 172, 177, 180, 181, 182, 183, 184, 188, 189, 192, 194, 196, 197 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-bench/tests/models.rs` | 13, 38, 41, 42, 49, 55, 58, 111, 120, 123, 127, 134, 136, 137, 138, 139, 151, 152, 153, 168, 169, 170, 180, 181, 182, 183, 184, 187, 188, 189, 190, 191, 194, 195, 196, 197, 198, 213, 214, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 231, 234, 235, 238, 239, 242, 243, 245, 246, 247, 248, 249, 250, 251, 265, 266, 274, 275, 280, 281, 282, 284, 286, 296, 302, 307, 311, 316, 320, 338, 340, 356, 376, 377, 380 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/agent_guide.rs` | 14, 15, 26, 30, 37, 49, 64, 68, 85, 90, 91, 92, 107, 113, 125, 127, 128, 133, 146, 148, 149, 152, 153, 166, 174, 177, 179, 181, 182, 183, 185, 190, 208, 217, 218, 219, 220, 221, 223, 226, 229, 230, 238, 239, 240, 241, 245, 246, 250, 253, 262, 263, 268, 270, 273, 275, 290, 292, 294, 296, 298, 301, 304, 305, 307, 308, 309, 316, 322, 347, 350, 352, 358, 361, 362, 363, 371, 375, 380, 383, 385, 388, 390, 393, 397, 399, 405, 410, 411, 413, 414, 415, 416, 419, 425, 430, 437, 442, 444, 446, 450, 453, 454, 458, 459, 463, 464, 471, 472, 473, 474, 475, 476, 481, 482, 484, 492, 500, 502, 504, 506, 507, 508, 510, 512, 513, 514, 516, 521 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/cli.rs` | 18, 22, 23, 24, 25, 33, 34, 40, 41, 51, 52, 53, 54, 59, 60, 61, 63, 76, 77, 78, 79, 80, 108, 117, 127, 130, 131, 132, 133, 135, 140, 143, 144, 154, 157, 159, 179, 188, 193, 195, 199, 200, 201, 202, 203, 217, 218, 225, 226, 227, 228, 230, 256, 259, 262, 270, 277, 289, 292, 295, 297, 298, 299, 300, 301, 302, 303, 304, 305, 306, 309, 312, 313, 321, 328, 334, 340, 344, 359, 362, 364, 365, 371, 372, 373, 374, 375, 376, 386, 387, 388, 389, 390, 391, 392, 393, 394, 400, 402, 408, 409, 410, 411, 412, 415, 424, 426, 429, 430, 431, 432, 433, 445, 448, 453, 454, 456, 459, 460, 465, 466, 475, 476, 477, 478, 492, 493, 495, 497, 498, 499, 500, 504, 512, 513, 521, 539, 540, 544, 549, 550, 551, 563, 569, 570, 576, 579, 582, 583, 591, 593, 594, 596, 597, 607, 608, 609, 610, 611, 612, 613, 621, 625, 626, 627, 628, 629, 641, 643, 644, 646, 647, 648, 649, 653, 660, 662, 665, 666, 672, 681, 684, 685, 686, 687, 692, 693, 699, 701, 704, 707, 708, 709, 710, 711, 712, 716, 717, 720, 721, 722, 723, 734, 736, 738, 739, 743, 745, 746, 747, 748, 749, 750, 752, 757, 758, 761, 764, 765, 768, 769, 782, 785, 791, 793, 794, 795, 799, 800, 812, 815, 816, 817, 836, 845, 846, 856, 857, 864, 866, 870, 871, 872, 873, 874, 876, 881, 882, 887, 893, 897, 898, 903, 904, 907, 908, 912, 921, 923, 924, 925, 926, 927, 928, 929, 930, 931, 932, 933, 934, 935, 936, 937, 938, 939, 944, 945, 946, 955, 956, 957, 958, 963, 964, 979, 998, 999, 1008, 1009, 1010, 1013, 1014, 1015, 1016, 1020, 1021, 1022, 1023, 1024, 1025, 1026, 1032, 1033, 1034, 1037, 1038, 1049, 1050, 1063, 1064, 1070, 1074, 1076, 1077, 1081, 1082, 1083, 1088, 1089, 1092, 1093, 1103, 1105, 1106, 1108, 1109, 1111, 1112, 1127, 1129, 1130, 1131, 1132, 1133, 1141, 1144, 1145, 1146, 1147, 1151, 1179, 1193, 1195, 1202, 1203, 1204, 1205, 1206, 1212, 1215, 1216, 1219, 1222, 1224, 1233, 1235, 1237, 1239, 1240, 1241, 1243, 1254, 1259, 1260, 1261, 1262, 1273, 1276, 1277, 1278, 1282, 1283, 1293, 1297, 1298, 1300, 1303, 1304, 1305, 1306, 1307, 1311, 1332, 1333, 1340, 1345, 1348, 1353, 1370, 1373, 1374, 1376, 1387, 1388, 1390, 1391, 1392, 1395, 1396, 1397, 1401, 1403, 1404, 1405, 1409, 1410, 1415, 1416, 1417, 1419, 1420, 1422, 1433, 1442, 1446, 1448, 1449, 1450, 1451, 1459, 1466, 1467, 1468, 1469, 1470, 1471, 1480, 1481, 1482, 1493, 1499, 1510, 1511, 1520, 1533, 1534, 1536, 1539, 1540, 1541, 1542, 1544, 1548, 1554, 1558, 1560, 1569, 1572, 1573, 1574, 1579, 1580, 1581, 1584, 1592, 1594, 1595, 1597, 1601, 1605, 1610, 1614, 1618, 1622, 1623, 1624, 1626, 1629, 1630, 1635, 1636, 1637, 1644, 1647, 1648, 1659, 1661, 1676 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/contract.rs` | 4, 16, 17, 18, 19, 20, 35, 38, 39, 46, 49, 50, 55, 61, 63, 70, 73, 74, 76, 77, 81, 82, 87, 88, 89, 90, 92, 113, 115, 116, 119, 120, 126, 129, 131, 133, 135, 136, 140, 142, 145, 146, 164, 166, 167, 176, 185, 186, 189, 190, 193, 194, 207, 209, 211, 218, 220, 223, 224, 235, 236, 237, 246, 247, 248, 249, 254, 259, 262, 269, 271, 272, 275, 276, 279, 284, 288, 290, 292, 296, 302, 304, 317, 320, 325, 328, 333, 344, 350, 351, 352, 353, 354, 355, 356, 357, 358, 368, 370, 371, 373, 374, 377, 378, 381, 382, 384, 389, 390, 394, 395, 409, 411, 412, 413, 416, 420, 422, 423 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/describe.rs` | 14, 18, 19, 20, 21, 24, 26, 27, 28, 38, 40, 42, 58, 60, 61, 75, 77, 80, 81, 82, 83, 84, 88, 90, 91, 94, 96, 98, 99, 100, 101, 104, 108, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 127, 128, 129, 130, 131, 133, 136, 137, 138, 141, 142, 143, 145, 149, 151, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 167, 168, 170, 174, 185, 187, 188, 191, 192, 196, 197, 198, 200, 201, 202, 204, 205, 206, 207, 209, 211, 215, 217, 223, 229, 232, 234, 235, 239, 242, 243, 245, 246, 252, 254, 257, 258, 259, 260, 268, 269, 270, 271, 272, 273, 274, 275, 276, 277, 278, 279, 280, 281, 285, 287, 289, 290, 291, 296, 297, 304, 305, 325, 326, 327, 328, 329, 330, 331, 332, 340, 347, 348, 349, 350, 351, 355, 362, 363, 364, 371, 374, 377, 380, 381, 382, 383, 384, 385, 386, 387, 388, 392 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/error_registry.rs` | 7, 8, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 49, 55, 56, 65, 78, 83, 97, 98, 99, 107, 108, 109, 110, 111, 112, 113, 114, 117, 119, 120, 122, 131, 132, 139, 158, 161, 162, 163, 164, 165, 166, 169, 176, 180, 181, 182, 183, 184, 185, 188, 194, 196, 197, 198, 207, 208, 209, 210, 214, 215, 220, 232, 235, 236, 239, 240, 241, 242, 243, 244, 248, 249, 257, 276, 278, 282, 285, 292, 294, 306, 371, 397, 401, 412, 429, 448, 450, 460, 478, 479, 480, 481, 482, 483, 484, 485, 489, 498, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 521, 524, 528, 531, 532, 533 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/error_schema.rs` | 3, 13, 20, 21, 33, 34, 35, 36, 37, 38, 39, 40, 42, 43, 45, 46, 47, 48, 49, 50, 51, 60, 62 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/i18n.rs` | 9, 10, 13, 15, 16, 17, 18, 19, 52, 58, 60, 67, 72, 75, 77, 81, 82, 98, 100, 103, 112, 134, 149, 150, 153, 154, 155, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 181, 182, 184, 187, 188, 192, 193, 195, 196, 203, 205, 206, 211, 214, 217, 222, 223, 224, 226, 228, 229, 235, 236, 241, 253, 254 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/support/progress.rs` | 8, 21, 24, 25, 26, 28, 29, 45, 46, 47, 49, 50, 51, 52, 56, 58, 59, 62, 63, 68, 70, 71, 74, 75, 76, 77, 78, 82, 90, 92, 95, 96, 98, 99, 110, 113, 116, 117, 118, 128, 129, 132, 133, 135, 136, 137, 138, 147, 148, 153, 156, 157, 158, 159, 160, 161, 162, 163, 167, 182, 183, 194, 196, 207, 213, 218, 219, 237, 240, 241, 250, 252, 253, 264, 269 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/support/schema.rs` | 16, 19, 23, 25, 32, 48, 51, 54, 56, 57, 58, 59, 62, 65, 69, 71, 77, 81 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-cli/tests/support/validate_report.rs` | 7, 17, 18, 19, 21, 34, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 55 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-frame/tests/analytical.rs` | 15, 46, 52, 63, 392 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-frame/tests/fixtures/mod.rs` | 2, 3, 20, 27, 28 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-frame/tests/invariants.rs` | 12, 14, 23, 47, 48, 49, 50, 51, 52, 81, 83, 415, 462, 463, 464, 465, 466, 467, 468, 469, 470, 471, 479, 487, 488, 489, 490, 492, 493, 494, 501, 502, 503, 509, 510, 511, 512, 514, 517, 522, 523, 527, 530, 540, 550, 551, 566, 569, 571, 578, 579, 582, 585, 589, 592, 597, 600, 601, 605, 610, 628, 640, 644, 645, 650, 653, 662, 669, 672, 677, 681, 682, 691, 694, 697, 701, 706, 708, 763, 764, 766, 767 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-frame3d/tests/analytical.rs` | 28, 33, 72, 171, 178, 181, 333 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-frame3d/tests/fixtures/mod.rs` | 10, 11, 24, 53, 54, 55, 56, 57, 58, 59, 64, 68, 72 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-frame3d/tests/invariants.rs` | 45, 49, 58, 87, 480, 484, 487, 492, 496, 500, 504, 506, 507, 513, 514, 528, 533, 537, 543, 549, 550, 551, 552, 561, 562, 565, 568, 569, 573, 574, 575, 576, 578, 579, 580, 584, 642, 645, 646, 648, 651, 665, 666, 681, 688, 691, 693, 697, 710, 714, 715, 718, 721, 723, 726, 728, 732, 736, 741, 744, 746, 747, 753, 754, 756, 760, 765, 785, 788, 807, 810, 814 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-frame3d/tests/r6.rs` | 51, 54, 127, 154 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-spring/tests/analytical.rs` | 14, 39, 45, 102, 144, 181, 229 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-spring/tests/fixtures/mod.rs` | 2, 3 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-spring/tests/invariants.rs` | 15, 50, 52, 84, 93, 212, 354, 355, 382, 390, 392, 395, 399, 408, 413, 418, 426, 432, 447, 448, 449, 450, 451, 454, 455, 456, 457, 458, 459, 461, 462, 463, 466, 467, 470, 475, 476, 479, 495, 497, 502, 511, 527, 528, 531, 534, 535, 536, 537, 538, 539, 541, 543, 546, 547, 548, 551, 552, 553, 556, 557, 558, 560, 562, 564, 567, 569, 571, 572, 574, 576, 579, 581, 584, 586, 594, 596, 599, 601, 602, 603, 607, 629, 632, 653 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-spring/tests/nonlinear.rs` | 19, 24, 32, 85, 86, 87, 295, 307, 308, 360, 394, 473, 486, 497, 507, 508, 509, 512, 515, 517, 525, 526, 534, 539, 544, 548, 551, 555, 564, 567, 574, 578, 579, 580, 603, 604, 622, 623, 624, 625, 626, 642, 643, 644, 645, 646 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-truss/tests/analytical.rs` | 17, 63, 74, 121, 274, 281, 291 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-truss/tests/fixtures/mod.rs` | 2, 4 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-truss/tests/geometric_producer.rs` | 18, 30, 37, 84, 109, 112, 343, 350, 356, 361, 378 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-elements-truss/tests/invariants.rs` | 14, 15, 17, 44, 45, 46, 71, 76, 292, 293, 294, 295, 296, 297, 298, 303, 304, 305, 313, 314, 315, 317, 320, 324, 331, 332, 337, 338, 339, 343, 349, 355, 356, 360, 365, 372, 391, 394, 404, 407, 410, 423, 429, 433, 439, 447, 455, 458, 469, 474, 478, 483, 486, 494, 501, 505, 508, 519, 529, 539, 550 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-geo/tests/consolidation.rs` | 26, 31, 91, 111, 187, 199, 248, 260, 262, 266, 269, 270, 273, 274, 286, 299, 303, 304, 305, 306, 307, 308, 323, 324, 325, 329, 331, 346, 348, 376, 378, 380, 411, 412, 414, 424, 427, 430, 433, 436, 439, 440, 441, 445, 452, 454, 462, 464, 471, 473, 482, 483, 484, 485, 486, 487, 488, 489, 501, 516, 534, 536, 542, 544, 556, 558, 568, 570, 584, 595 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-geo/tests/py.rs` | 36, 60, 61, 139, 146, 151, 152, 158, 160, 177, 183, 199, 200, 207, 208, 213, 214, 221, 222, 228, 240, 294, 300, 305, 323, 324, 331, 339, 340, 352, 355, 356, 364, 368, 371, 374, 377, 381 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-geo/tests/support/axial_benchmarks.rs` | 5, 7, 9, 17, 25, 53, 59, 60, 94, 96, 110, 111, 119 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-geo/tests/tz.rs` | 2, 23, 24, 25, 40, 44, 48, 52, 55, 56, 58, 68, 72, 76, 79, 83, 84, 88, 89, 97, 99, 109, 110, 119, 121, 125, 126, 139, 144, 148, 179, 180, 184, 185, 192, 193, 197, 198, 207, 208, 215, 216, 223, 224, 236, 237, 241, 242, 318, 319, 336, 337, 343, 344, 355, 356, 360, 361, 365, 366, 370, 372, 377, 382, 387, 388, 392, 393, 407, 408, 417, 418, 425, 428, 431, 434, 440, 446, 447, 454, 455 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-laws/tests/cubic_elastic.rs` | 23, 24, 81, 93, 94, 118, 122, 154, 156, 159, 160, 161, 169, 185 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-laws/tests/elastic.rs` | 25, 26, 32, 33, 39, 40, 70, 71, 79, 80, 88, 89, 110, 116, 165, 176, 241, 244, 247, 252, 254, 279, 282, 294, 315, 327, 331, 337, 373, 374, 380, 384, 385, 392 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-laws/tests/multilinear.rs` | 26, 27, 50, 64, 234, 252, 261, 268, 272, 310, 315, 319, 320, 326, 361, 362, 382, 397, 403 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-laws/tests/steel_bilinear.rs` | 21, 202, 220, 223, 236, 244, 245, 248, 252, 253, 254, 255, 256, 257, 260, 291, 292, 294, 296, 298 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/budget.rs` | 22, 36, 37, 44, 56, 72, 87, 110, 175, 207, 239 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/conformance.rs` | 24, 44, 166, 170, 301, 313, 316, 318, 321, 324, 330, 334, 339, 343, 355, 367, 379, 383, 421, 424, 431, 433, 435, 437, 438, 464, 468, 474, 482, 485, 487, 491 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/dependency_boundary.rs` | 8, 10, 11, 30, 50, 54, 58, 61, 120, 121, 129, 134, 135, 137, 145, 160, 161, 162, 163, 164, 169, 170, 171, 172, 173, 178, 179, 180, 181, 182 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/determinism_guard.rs` | 8, 18, 22, 26, 29, 50, 104, 105, 118, 125, 127, 128, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 172, 173, 175, 176, 177, 182, 190, 194, 196, 207, 216, 237, 248, 251, 252, 258, 259, 277, 278, 279, 293, 299, 311, 315, 316, 317, 322, 330, 331, 377, 382, 383, 384, 390, 393, 396, 401, 403, 405, 406, 408, 429, 431, 434, 436, 439, 441, 444, 446, 449, 451, 454, 456, 459, 461, 468, 520, 521, 526, 527, 528, 529, 544, 545, 546, 548, 551, 554, 557, 561, 562, 563, 573, 580, 589, 591, 592, 617, 618, 621, 623, 624, 625, 626, 632, 633, 634, 635, 636, 637, 639, 646, 647, 648, 649, 650, 651, 653, 656, 657, 658, 659, 660, 662, 668, 698, 720, 730, 734, 744, 745, 746, 749, 750, 751, 752, 753, 754, 755, 756, 757, 759, 762, 763, 764, 765, 766, 768, 771, 772, 775, 776, 777, 778, 779, 780, 782, 789, 792, 795, 796, 802, 805, 808, 809, 810, 811, 812, 813, 814, 815, 816, 817, 818, 819, 820, 821, 823, 830, 833, 836, 837, 838, 840, 843, 849, 850, 852, 853, 854, 855, 857, 861, 864, 865, 866, 868, 872, 873, 880, 883, 886, 887, 888, 889, 890, 891, 893, 895, 896, 899, 900, 907, 908, 909, 910, 911, 912, 913, 914, 916, 918, 919, 926, 933, 936, 937, 938, 939, 941, 944, 945, 946, 947, 948, 949, 951, 959, 960, 961, 962, 964, 965, 966, 973, 974, 975, 976, 977, 978, 979, 982, 983, 986, 987, 990, 991, 998, 1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1010, 1013, 1014, 1015, 1017, 1024, 1025, 1026, 1027, 1028, 1029, 1030, 1031, 1032, 1033, 1036, 1037, 1041, 1046, 1047, 1049, 1052, 1053, 1063, 1064, 1065, 1068, 1070, 1073, 1074, 1076, 1084, 1085, 1086, 1087, 1088, 1097, 1098, 1099, 1100, 1109, 1110, 1111, 1112, 1121, 1122, 1123, 1124, 1133, 1134, 1135, 1136, 1137, 1138, 1147, 1148, 1149, 1158, 1159, 1160, 1169, 1170, 1171, 1180, 1181, 1182, 1191, 1192, 1193, 1202, 1203, 1212, 1213, 1214, 1222, 1227, 1235, 1236, 1244, 1254, 1276, 1277, 1285, 1288, 1289, 1293, 1294, 1295, 1296, 1297, 1298, 1299, 1300, 1301, 1302, 1303, 1304, 1305, 1308, 1309, 1316, 1317, 1318, 1328, 1329, 1330, 1331, 1332, 1333, 1334, 1335, 1336, 1337, 1338, 1340, 1341, 1342 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/guyan.rs` | 3, 123, 127, 148, 158 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/heap_peak.rs` | 220, 230, 254, 261 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/lowest_modes_anti_revert.rs` | 25, 87, 88, 126, 127, 146, 147, 148 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/modes.rs` | 146, 152, 214, 246, 251, 252, 257, 266, 267, 268, 304, 339, 340, 350, 357, 360, 361, 382, 390, 394, 400 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-linalg/tests/robustness.rs` | 18, 26, 29, 36, 46, 55, 76, 97, 109, 141, 143, 144, 158, 159, 165, 166, 169, 170, 193, 218, 219, 230, 238, 249, 262, 266, 271, 274, 293, 306, 312, 394, 395, 404 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-loads/tests/fixtures/mod.rs` | 3, 5 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-loads/tests/mass.rs` | 15, 16, 29, 49, 62, 180, 181, 182, 211, 296, 303, 337, 386, 391, 406, 444, 531, 540, 549, 553, 559, 573, 575, 577, 589, 590, 591, 594, 595, 597, 604, 614, 629, 651, 665, 674, 688, 694, 700, 705, 708, 713, 716, 719, 733, 734, 735 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-loads/tests/nodal.rs` | 11, 62, 77, 97, 141, 150, 155, 169, 182, 188, 199, 206, 210, 222, 234 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-loads/tests/span.rs` | 284, 287, 289, 292, 293, 307, 330, 331, 342, 346, 374, 382 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-loads/tests/weight.rs` | 41, 53, 56, 57, 58, 59, 60 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-wasm/tests/i18n.rs` | 4, 6, 36, 64, 67 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-wasm/tests/sfc010.rs` | 2, 11, 15, 23, 24, 28, 30, 32, 34, 37, 42, 44, 53, 54, 55 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-contracts/src/tests.rs` | 14, 15, 16, 20, 163, 196, 228, 233, 238, 243, 251, 288, 291, 318, 322, 326, 336, 404, 405, 406, 407, 408, 422, 424, 427, 430, 446, 461, 540, 544, 567, 570, 571, 572, 573, 582, 608, 640, 643, 741, 749, 752, 832, 841, 921, 924, 937, 940, 950, 958, 964, 967, 1024, 1031, 1032, 1033 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-geo/src/tests.rs` | 30, 43, 45, 46, 129, 130, 131, 132, 140, 141, 149, 150, 154, 155, 156, 160, 166, 167, 177, 178, 186, 188, 189, 190, 194, 200, 201, 205, 206, 214, 215, 223, 224, 228, 234, 235, 239, 240, 245, 246, 255, 256, 265, 266, 331, 332, 347, 348, 356, 357, 361, 363, 365, 372, 381, 385, 388 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |
| `crates/engine-model/src/tests.rs` | 9, 11, 16, 18, 25, 26, 29, 34, 44, 46, 51, 63, 64, 65, 66, 109, 110, 111, 112, 113, 114, 115, 116, 120, 130, 131, 134, 139, 142, 143, 144, 145, 146, 147, 148, 149, 153, 159, 171, 187, 196, 211, 212, 213, 214, 215, 216, 217, 218, 222, 237, 238, 239, 240, 244, 265, 267, 273, 281, 282, 283, 287, 288, 289, 293, 294, 295, 299, 300, 301, 313, 332, 334, 335, 338, 339, 340, 344, 345, 346, 364 | Inputs, comparison values, labels, keys, paths, codes, citations, returned errors, custom check text, or existing English diagnostics: outside the Spanish assertion/panic allowance; ambiguous uses are retained. |

Total: 5027 source lines containing unchanged string literals.

Apart from the 13 deferred comment lines in the hash-pinned test, no Spanish
comment prose remains in the reviewed sources. The tranche 1 Spanish heuristic
reports three other comment hits: one English sentence with
repeated no, and two preserved symbolic references, `máx` in
`crates/engine-assembly/tests/j4_followup.rs:165` and `math::función` in
`crates/engine-linalg/tests/determinism_guard.rs:167`. Existing symbolic names
inside equations and code references are not translated prose.

### Remaining (tranche 3 and later)

Spanish-line counts below use the same reproducible heuristic as ENG-LANG-004
and ENG-LANG-005: Spanish diacritics or punctuation, or at least two
case-insensitive whole words from que, para, del, una, los, las, por, sin,
debe, según, el, la, de, en, con, un, se, al, como, no, es, son, su, sus,
entre, cada, todos, todas, también, más, antes, después, hasta, sobre, solo,
desde, durante. Each matching physical line counts once, including literal
data; these are raw heuristic counts, not a claim that every hit is eligible
prose. Short labels can be missed and English/code can produce false positives.
All these files remain unchanged.

| Remaining scope | Spanish-line hits | Files with hits |
| --- | ---: | ---: |
| `scripts/**` | 195 | 16 |
| `bindings/python/tests` | 70 | 10 |
| `engine-validation leftovers` | 338 | 5 |
| `clean-room/tests (owner-only)` | 50 | 1 |
| `.github/workflows (owner-only)` | 16 | 2 |

| Remaining source | Spanish-line hits |
| --- | ---: |
| `scripts/benchmark-python-boundary.py` | 6 |
| `scripts/build-python-wheel.sh` | 12 |
| `scripts/build-wasm.sh` | 1 |
| `scripts/changelog-assemble.sh` | 16 |
| `scripts/changelog-check.sh` | 11 |
| `scripts/check-determinism.mjs` | 8 |
| `scripts/eng-val-006/orchestrate.py` | 19 |
| `scripts/eng-val-006/report.py` | 4 |
| `scripts/eng-val-006/stats.py` | 4 |
| `scripts/eng-val-006/test_orchestrator.py` | 1 |
| `scripts/generate-schemas.py` | 18 |
| `scripts/package-schemas.sh` | 2 |
| `scripts/python-wheel-artifacts.py` | 30 |
| `scripts/tests/test_changelog.sh` | 49 |
| `scripts/tests/test_python_wheel_artifacts.py` | 13 |
| `scripts/tests/test_schema_publication.py` | 1 |
| `bindings/python/tests/test_arrays.py` | 5 |
| `bindings/python/tests/test_experiments.py` | 24 |
| `bindings/python/tests/test_facade.py` | 11 |
| `bindings/python/tests/test_i18n.py` | 6 |
| `bindings/python/tests/test_numpy.py` | 5 |
| `bindings/python/tests/test_run.py` | 14 |
| `bindings/python/tests/test_sfc010.py` | 1 |
| `bindings/python/tests/test_typing.py` | 2 |
| `bindings/python/tests/test_version.py` | 1 |
| `bindings/python/tests/unwind.rs` | 1 |
| `crates/engine-validation/src/fixtures.rs` | 39 |
| `crates/engine-validation/tests/oracles.rs` | 24 |
| `crates/engine-validation/tests/fixtures.rs` | 8 |
| `crates/engine-validation/README.md` | 135 |
| `crates/engine-validation/ORACLES.md` | 132 |
| `clean-room/tests/run.sh` | 50 |
| `.github/workflows/clean-room.yml` | 12 |
| `.github/workflows/rust.yml` | 4 |

- `crates/engine-api/tests/val032_runs.rs`: 13 Spanish comment lines and eleven
  eligible messages remain deferred by the full-file digest contract.
- `scripts/**`, including `scripts/tests`, and `bindings/python/tests` require
  later tranches with their own literal/output review.
- The five engine-validation leftovers above remain deferred, together with
  the retained literals inventoried in tranche 1. Previously translated
  engine-validation files and all engine-validation fixture data are unchanged.
- `clean-room/tests` and `.github/workflows` are owner-only and remain outside
  this ticket's editable scope.
- The retained literals inventoried for this tranche require their own
  authorized scope and comparison/output review before translation.

### Verification (tranche 2)

Verification uses Cargo directly with the shared build lock, offline. Scratch
files and logs are outside the checkout and outside the system temporary
directory. No commit was created.

| Check | Result |
| --- | --- |
| Source boundary audit | PASS: all 93 reviewed Rust sources retain identical code tokens after restoring the 153 approved messages and removing comments/whitespace; the other literal sequences and all character literals match the base. All changed comments match the translation inventory. |
| `cargo fmt --all --check` | PASS after assertion/panic layout adjustments. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | PASS. |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --offline` | PASS. |
| `cargo test --workspace --offline` | PASS: 850 Rust tests (805 unit/integration and 45 doctests), 0 failed, 0 ignored, across 130 groups (108 unit/integration and 22 doctest groups). Both embedded Python suites passed 7 tests each (14 total). |
| `cargo test -p engine-api --test val032_runs --offline` | PASS: 1 test; all numerical checks and full-file source hashes pass. |
| `bash clean-room/check.sh origin/main` | PASS, including both updated documents. |
| `bash scripts/changelog-check.sh origin/main` | PASS; the script compares committed HEAD changes and reports no code changes. Supplemental working-tree fragment checks pass. |
| `python3 scripts/generate-schemas.py --check` | PASS: schema bytes, index, and inventory are current. |
| `git diff --check` | PASS. |

The boundary audit compares every non-comment token in each reviewed source,
allowing only the explicitly inventoried message substitutions and whitespace
from Rust formatting. Format placeholders, their order, and every other
literal remain identical. Scope and added-text checks cover the two required
documents as well as all 82 changed Rust sources. The provenance prefix is
compared byte-for-byte with the base to preserve all tranche 1 sections.

The final source diff contains 676 added and 692 deleted Rust lines, including
Rust formatting; the translated-content total remains 669. Exactly 84 tracked
files change: 82 Rust test sources and the two required documents. Added-text
scans find no restricted identifiers or external program names. The protected
fixture index, both pinned analysis sources, and the pinned API test remain
byte-identical to the base. Supplemental working-tree changelog checks validate
the existing fragment prefix, appended Spanish bullet, required ticket marker,
tramo 2 wording, absence of headings, and unchanged CHANGELOG.md.
