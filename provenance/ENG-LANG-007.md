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
