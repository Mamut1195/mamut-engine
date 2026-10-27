# ENG-LANG-001 / MAM-839

Date: 2026-10-23

## Identifier renames

The audit was rechecked against the current sources: 21 function identifiers were renamed (20 tests and one helper). Test bodies, assertions and behavior are unchanged. No Spanish identifier remains in executable fn/let/const/struct declarations in the five requested files. The audit finding in `crates/engine-laws/src/uniaxial/multilinear.rs` is stale: its identifiers are already English. Rustdoc example identifiers and other comment/string text remain outside the identifier edits, as requested.

### `crates/engine-core/src/units.rs`

| Former identifier | English identifier |
| --- | --- |
| `simbolos_alias_y_texto_desconocido` | `symbols_aliases_and_unknown_text` |
| `codigos_ascii_de_units_1` | `units_1_ascii_codes` |
| `factores_y_unidades_si_por_magnitud` | `factors_and_si_units_by_quantity` |
| `ida_y_vuelta_momento_y_presion` | `moment_and_pressure_round_trip` |
| `compatibilidad_de_todas_las_magnitudes` | `compatibility_of_all_quantities` |
| `no_finitos_desbordamiento_e_identidad` | `nonfinite_values_overflow_and_identity` |
| `declaraciones_si_ordenadas_y_reemplazo` | `ordered_si_declarations_and_replacement` |

### `crates/engine-laws/tests/cubic_elastic.rs`

| Former identifier | English identifier |
| --- | --- |
| `benchmark_analitico_nativo` | `native_analytical_benchmark` |
| `kit_de_conformidad_en_traccion_descarga_y_compresion` | `conformance_kit_in_tension_unloading_and_compression` |
| `fabrica_descriptor_y_propiedad_del_estado` | `factory_descriptor_and_state_ownership` |
| `simetria_y_limite_lineal` | `symmetry_and_linear_limit` |
| `parametros_y_entradas_invalidos_usan_catalogo` | `invalid_parameters_and_inputs_use_catalog` |
| `desbordamientos_de_productos_y_sumas_son_errores` | `product_and_sum_overflows_are_errors` |

### `crates/engine-linalg/tests/conformance.rs`

| Former identifier | English identifier |
| --- | --- |
| `conformidad` | `conformance` |

### `crates/engine-validation/tests/oracles.rs`

| Former identifier | English identifier |
| --- | --- |
| `inventario_cubre_todas_las_tablas_y_fixtures` | `inventory_covers_all_tables_and_fixtures` |
| `cada_fila_enlaza_un_ancla_existente_de_su_derivacion` | `each_row_links_to_an_existing_derivation_anchor` |
| `guardia_detecta_enlaces_de_derivacion_rotos` | `guard_detects_broken_derivation_links` |
| `guardia_informa_casos_faltantes_y_sobrantes_en_ambos_lados` | `guard_reports_missing_and_extra_cases_on_both_sides` |
| `guardia_rechaza_fuentes_o_limites_incompletos` | `guard_rejects_incomplete_sources_or_limits` |
| `fixtures_rechazan_justificacion_vacia_o_en_blanco_y_cita_pendiente` | `fixtures_reject_empty_or_blank_justification_and_pending_citation` |
| `fixtures_versionados_rechazan_oraculo_o_tolerancia_ausente` | `versioned_fixtures_reject_missing_oracle_or_tolerance` |

## Schema translations

Translated 21 strings on 21 lines: 11 descriptions in engine-input/1, one title and one description in method-descriptor/1, and one title and seven descriptions in results/1. This exceeds the stale audit count of 11 lines. The engine-input title already uses English-neutral identifiers and is unchanged. Terminology follows the existing English API explanations: Experimental methods, job root, load breakpoints, substeps, free nodes, single/double drainage, saturating u128 arithmetic, validation and integration.

Only title/description string contents changed in the three source schemas. Whitespace, structure, keys, enums, constraints, numbers, stable codes and $id values are preserved. The published tree was regenerated offline with `python3 scripts/generate-schemas.py --write`. Only three entries in `schemas/index.json` changed. `scripts/schema-sources.json` contains source registrations rather than digest/size entries and remains byte-identical. Other published schemas remain byte-identical.

| Schema id (version 1) | Former SHA-256 | New SHA-256 | Former bytes | New bytes |
| --- | --- | --- | --- | --- |
| `engine-input` | `ecd32e2e0a7551a80224958d55d4cc7d451341e2c6f7c111d2417c6dc94d25ee` | `7d5f859cce8f51145e2d6f1a70b41b112ae8f56a2aea3bd56e94ac7d1842a3ff` | 19404 | 19435 |
| `method-descriptor` | `317f2d9c6dee114d1bfd19d4f3ffd66088a7148a878f63edc7be0c33bd29dcac` | `946986dca994c7ec8589cdba2479f602c2c793a3349e48729eb5900abe4109bd` | 3905 | 3902 |
| `results` | `c14672ee90fe5a577a317c0783323539f20f895bdd091ea490b6eee6c1f1765f` | `cfb1e2e37aed454f3a3f054b8a710deb45081d7d9f586f585f8f2e6f11aa340a` | 15254 | 15191 |

Each digest/size pair describes both the corresponding source under `crates/engine-api/schemas/` and its published copy under `schemas/<id>/1.json`.

## Document renames and references

| Former name | English name |
| --- | --- |
| `docs/DETERMINISMO.md` | `docs/DETERMINISM.md` |
| `docs/GUIA_METODOS.md` | `docs/METHODS_GUIDE.md` |
| `crates/engine-validation/ORACULOS.md` | `crates/engine-validation/ORACLES.md` |
| `docs/CONVENCION_EJES.md` | `docs/AXES_CONVENTION.md` |

All four moves used `git mv`. The managed worktree's normal Git index is read-only, so the moves and intent-to-add registrations used a separate writable index. The normal index remains unchanged; the final worktree contains the renamed files and can be staged normally once its Git metadata is writable. No commit was created.

Updated live references in CONTRIBUTING.md, crate READMEs, API explanations, include_str! paths, validation assertion messages, Rust comments/rustdoc, the methods guide, the schema-publication guide, the Python ADR, the prompt template and the validation report TeX source. No matching reference was found in protected locations. The live-reference scan excludes existing provenance notes and CHANGELOG.md and returns zero matches. The oracles assertion message now names ORACLES.md; no test asserts the former message text.

Existing provenance files were left intact as historical records; their old document links refer to the former names. Other filename candidates, including `docs/CLEAN_ROOM.md`, `docs/CLA.md`, `docs/PUBLICACION_ESQUEMAS.md`, `docs/PYTHON_BOUNDARY_BENCHMARKS.md`, `docs/TICKETS_PRS.md`, ADR filenames and template filenames, remain unchanged because only the four requested document renames are in scope. Document prose, other comments/string literals and Rustdoc example identifiers remain in their original language except for the requested references and schema strings. Protected files and CHANGELOG.md were not edited.

## Output contracts and digest snapshots

| File | Entry | Former SHA-256 | New SHA-256 |
| --- | --- | --- | --- |
| `crates/engine-cli/tests/snapshots/describe-1.stdout.sha256` | `describe --json` stdout | `61dce5dbd4ad7a5bb42665c89835c7c87c708ee832b5b4ce15ea6cbfc88e41f8` | `116652a37c0cd7904b1f84821d79fdf7c56d0f1645ccd5ab5bf3681787949e42` |

The description output changes only the three schema inventory hashes; substituting their previous values exactly recovers the previous stdout digest. Help JSON aliases share that output. The three schema command outputs change byte-for-byte with their sources. Capabilities output is unaffected. The explanations for API-E-SFC-OPTIONS, API-E-CONSOLIDATION-WORK and API-E-CONSOLIDATION-COST change only the methods-guide filename, in both text and JSON stdout; there are no fixed digest snapshots for those outputs. The fixture run/validate golden in `crates/engine-cli/tests/golden/stdout-sha256.json` remains unchanged. API schema-publication and schema-conformance checks derive their expectations from the current sources and require no fixed hash updates.

All 16 hashes in the renamed determinism document remain unchanged and pass the dedicated API test. No numerical or execution behavior changed.

## Changed-file inventory

| Change | File |
| --- | --- |
| Modify | `CONTRIBUTING.md` |
| Add | `changelog.d/MAM-839.changed.md` |
| Modify | `crates/engine-api/README.md` |
| Modify | `crates/engine-api/schemas/engine-input-1.json` |
| Modify | `crates/engine-api/schemas/method-descriptor-1.json` |
| Modify | `crates/engine-api/schemas/results-1.json` |
| Modify | `crates/engine-api/src/run.rs` |
| Modify | `crates/engine-api/tests/determinism.rs` |
| Modify | `crates/engine-assembly/README.md` |
| Modify | `crates/engine-cli/README.md` |
| Modify | `crates/engine-cli/tests/snapshots/describe-1.stdout.sha256` |
| Modify | `crates/engine-core/src/units.rs` |
| Modify | `crates/engine-laws/src/section.rs` |
| Modify | `crates/engine-laws/src/uniaxial/cubic_elastic.rs` |
| Modify | `crates/engine-laws/tests/cubic_elastic.rs` |
| Modify | `crates/engine-linalg/README.md` |
| Modify | `crates/engine-linalg/src/lib.rs` |
| Modify | `crates/engine-linalg/src/modes.rs` |
| Modify | `crates/engine-linalg/src/reference.rs` |
| Modify | `crates/engine-linalg/tests/conformance.rs` |
| Modify | `crates/engine-model/src/lib.rs` |
| Rename | `crates/engine-validation/ORACULOS.md` → `crates/engine-validation/ORACLES.md` |
| Modify | `crates/engine-validation/README.md` |
| Modify | `crates/engine-validation/tests/oracles.rs` |
| Rename | `docs/CONVENCION_EJES.md` → `docs/AXES_CONVENTION.md` |
| Rename | `docs/DETERMINISMO.md` → `docs/DETERMINISM.md` |
| Rename | `docs/GUIA_METODOS.md` → `docs/METHODS_GUIDE.md` |
| Modify | `docs/PUBLICACION_ESQUEMAS.md` |
| Modify | `docs/adr/ADR-003-python-v1.md` |
| Modify | `docs/plantillas/PROMPT_IA.md` |
| Modify | `docs/validation-report/casos/deriv/VAL-024.tex` |
| Add | `provenance/ENG-LANG-001.md` |
| Modify | `schemas/engine-input/1.json` |
| Modify | `schemas/index.json` |
| Modify | `schemas/method-descriptor/1.json` |
| Modify | `schemas/results/1.json` |

## Verification

- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo test --workspace --offline`: passed, 829 tests across 126 suites (784 unit/integration tests and 45 doctests); zero failures, ignored tests or filtered tests.
- `cargo doc --workspace --no-deps --offline`: passed.
- `cargo test -p engine-api --test determinism --offline`: passed separately, five tests, including equality of all 16 published hashes.
- `bash clean-room/check.sh origin/main`: passed with the writable index/object store and intent-to-add entries for the two new notes.
- `bash scripts/changelog-check.sh origin/main`: passed. This script compares committed revisions only and reports no committed code changes; the uncommitted fragment's filename, nonempty regular-file content and heading-free format were validated separately.
- `python3 scripts/generate-schemas.py --check`: passed.
- The requested live-reference scan returned zero matches (exit 1, as expected for no matches), using the writable index to include renamed documents.
- Additional comparisons confirmed identical bytes outside translated schema strings, identical parsed schema constraints, unchanged source registration and run/validate goldens, unchanged renamed document contents except methods-guide links, no protected-file changes and no forbidden identifiers on added lines.

All commands ran offline. Temporary build/test storage stayed in the shared build area. No network operations or commits were performed.
