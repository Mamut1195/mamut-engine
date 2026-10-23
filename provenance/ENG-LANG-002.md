# ENG-LANG-002 — English default messages

- Ticket: MAM-841; future catalog: MAM-836.
- Date: 2026-10-22.
- Base audit: the ENG-LANG-002 language audit (audited commit 43b5f4505e6189f8109d50ea2254e0d4dee1f60a).
- Sources: the existing repository messages and the language audit; no external sources or network access.

## Scope and counts

Translated 918 production message literal occurrences across 16 crates. Updated 81 expected string occurrences (65 distinct file/text pairs), including two doctest expectations, and added one exact diagnostic message assertion to an existing regression test. Two source-hash expectations in the VAL-032 index were refreshed individually. No numerical result fixtures or goldens were regenerated.

All 74 audit rows under crates/*/src were reviewed: 70 contain production messages translated here; three test-source rows receive only necessary expectation updates; the CLI row is deliberately unchanged. Additional diagnostics of the same kind, including newer benchmark and geotechnical messages, are included. Counts below refer to literal occurrences, not audit heuristic line counts.

Only displayed text and corresponding test expectations changed, with one added regression assertion and rustfmt formatting where required. Stable diagnostic codes, JSON keys, messageKey values, identifiers, production control flow, ordering, model data, schema descriptions, and evidence metadata retain their original values. The CLI library and README are unchanged.

Terminology: GDL → DOF; miembro → member; viga → beam; barra/celosía → truss when naming the element, member in generic benchmark connectivity; resorte → spring; estación → station; desplazamiento prescrito → prescribed displacement; historia de prueba → trial history; entrada/resultado no finito → non-finite input/result.

| Crate | Production literals | Expected strings |
| --- | ---: | ---: |
| engine-analysis | 85 | 7 |
| engine-api | 120 | 7 |
| engine-assembly | 49 | 1 |
| engine-bench | 66 | 1 |
| engine-contracts | 76 | 25 |
| engine-core | 46 | 3 |
| engine-elements-frame | 46 | 0 |
| engine-elements-frame3d | 68 | 0 |
| engine-elements-spring | 61 | 1 |
| engine-elements-truss | 39 | 0 |
| engine-geo | 127 | 14 |
| engine-laws | 27 | 0 |
| engine-linalg | 48 | 11 |
| engine-loads | 19 | 0 |
| engine-model | 21 | 9 |
| engine-validation | 20 | 3 |

## Translated source rows by crate

Audit ranges refer to the original audit. A dash marks a comparable message source absent from that audit. Test-only rows retain fixture identifiers and assertion failure prose; only expectations for translated production text change.

| Source file | Audit row/range | Production literals | Expected strings |
| --- | --- | ---: | ---: |
| crates/engine-analysis/src/combination.rs | 174–511 (7 lines), plus prescribed-displacement consistency diagnostic | 20 | 0 |
| crates/engine-analysis/src/lib.rs | 140 | 1 | 0 |
| crates/engine-analysis/src/modal.rs | 123–193 (5 lines) | 5 | 0 |
| crates/engine-analysis/src/newton.rs | 91–363 (6 lines) | 7 | 0 |
| crates/engine-analysis/src/pdelta.rs | 155–300 (4 lines) | 9 | 0 |
| crates/engine-analysis/src/results.rs | 278–695 (11 lines) | 21 | 0 |
| crates/engine-analysis/src/spectrum/combination.rs | 75–164 (4 lines) | 6 | 0 |
| crates/engine-analysis/src/spectrum/mod.rs | 167–313 (5 lines) | 8 | 0 |
| crates/engine-analysis/src/spectrum/table.rs | — | 8 | 0 |
| crates/engine-api/src/extensions.rs | — | 32 | 0 |
| crates/engine-api/src/geo_input.rs | — | 11 | 0 |
| crates/engine-api/src/input.rs | 31–305 (2 lines) | 4 | 0 |
| crates/engine-api/src/json.rs | 34 | 3 | 0 |
| crates/engine-api/src/lib.rs | — | 2 | 0 |
| crates/engine-api/src/limits.rs | 44–181 (2 lines) | 6 | 0 |
| crates/engine-api/src/methods.rs | 135–240 (7 lines) | 4 | 0 |
| crates/engine-api/src/query.rs | 57 | 2 | 0 |
| crates/engine-api/src/run.rs | 243–486 (10 lines) | 18 | 0 |
| crates/engine-api/src/run/advanced.rs | 37–432 (12 lines) | 22 | 0 |
| crates/engine-api/src/run/geo.rs | — | 1 | 0 |
| crates/engine-api/src/run/nonlinear.rs | — | 9 | 0 |
| crates/engine-api/src/run/static_linear.rs | 22–138 (4 lines) | 6 | 0 |
| crates/engine-assembly/src/assemble.rs | 54–338 (5 lines) | 11 | 0 |
| crates/engine-assembly/src/constraints.rs | 75–235 (6 lines) | 9 | 0 |
| crates/engine-assembly/src/diaphragm.rs | 55–94 (2 lines) | 6 | 0 |
| crates/engine-assembly/src/dofmap.rs | 35–137 (5 lines) | 10 | 0 |
| crates/engine-assembly/src/lib.rs | 85–96 (2 lines) | 3 | 0 |
| crates/engine-assembly/src/system.rs | 198–677 (9 lines) | 10 | 0 |
| crates/engine-bench/src/index.rs | — | 12 | 0 |
| crates/engine-bench/src/translate.rs | — | 54 | 0 |
| crates/engine-cli/src/lib.rs | 427 | 0 | 0 |
| crates/engine-contracts/src/conformance.rs | 17–1155 (29 lines) | 59 | 0 |
| crates/engine-contracts/src/examples.rs | 27–508 (7 lines) | 15 | 0 |
| crates/engine-contracts/src/lib.rs | — | 2 | 0 |
| crates/engine-contracts/src/tests.rs | 27–1024 (8 lines) | 0 | 25 |
| crates/engine-core/src/catalog.rs | 36–121 (25 lines) | 42 | 2 |
| crates/engine-core/src/lib.rs | — | 0 | 1 |
| crates/engine-core/src/units.rs | 131–267 (3 lines) | 4 | 0 |
| crates/engine-elements-frame/src/factory.rs | 15–88 (6 lines) | 11 | 0 |
| crates/engine-elements-frame/src/geometry.rs | 16 | 5 | 0 |
| crates/engine-elements-frame/src/lib.rs | 257 | 6 | 0 |
| crates/engine-elements-frame/src/loads.rs | 14 | 2 | 0 |
| crates/engine-elements-frame/src/mass.rs | — | 5 | 0 |
| crates/engine-elements-frame/src/recover.rs | 32–55 (6 lines) | 7 | 0 |
| crates/engine-elements-frame/src/release.rs | 26 | 5 | 0 |
| crates/engine-elements-frame/src/stiffness.rs | 7 | 1 | 0 |
| crates/engine-elements-frame/src/winkler.rs | — | 4 | 0 |
| crates/engine-elements-frame3d/src/factory.rs | 16–127 (10 lines) | 15 | 0 |
| crates/engine-elements-frame3d/src/geometry.rs | 16–63 (4 lines) | 12 | 0 |
| crates/engine-elements-frame3d/src/lib.rs | 301 | 5 | 0 |
| crates/engine-elements-frame3d/src/loads.rs | — | 1 | 0 |
| crates/engine-elements-frame3d/src/mass.rs | — | 5 | 0 |
| crates/engine-elements-frame3d/src/producer.rs | 28–49 (2 lines) | 3 | 0 |
| crates/engine-elements-frame3d/src/recover.rs | 25–46 (9 lines) | 10 | 0 |
| crates/engine-elements-frame3d/src/release.rs | 26–75 (2 lines) | 11 | 0 |
| crates/engine-elements-frame3d/src/stiffness.rs | 7 | 2 | 0 |
| crates/engine-elements-frame3d/src/winkler.rs | — | 4 | 0 |
| crates/engine-elements-spring/src/factory.rs | 36–141 (11 lines) | 19 | 0 |
| crates/engine-elements-spring/src/geometry.rs | 48–144 (13 lines) | 17 | 0 |
| crates/engine-elements-spring/src/lib.rs | 110–214 (2 lines) | 3 | 0 |
| crates/engine-elements-spring/src/loads.rs | — | 3 | 0 |
| crates/engine-elements-spring/src/nonlinear.rs | 65–89 (2 lines) | 8 | 0 |
| crates/engine-elements-spring/src/recover.rs | 36–38 (2 lines) | 4 | 0 |
| crates/engine-elements-spring/src/stiffness.rs | 28–47 (3 lines) | 7 | 0 |
| crates/engine-elements-truss/src/factory.rs | 30–60 (4 lines) | 6 | 0 |
| crates/engine-elements-truss/src/geometry.rs | 16 | 6 | 0 |
| crates/engine-elements-truss/src/lib.rs | 205 | 5 | 0 |
| crates/engine-elements-truss/src/loads.rs | 11–31 (2 lines) | 6 | 0 |
| crates/engine-elements-truss/src/mass.rs | — | 3 | 0 |
| crates/engine-elements-truss/src/producer.rs | 28–49 (2 lines) | 3 | 0 |
| crates/engine-elements-truss/src/recover.rs | 21–32 (6 lines) | 7 | 0 |
| crates/engine-elements-truss/src/stiffness.rs | 18 | 3 | 0 |
| crates/engine-geo/src/consolidation.rs | 124–577 (12 lines) | 41 | 0 |
| crates/engine-geo/src/profile.rs | 29–204 (7 lines) | 11 | 0 |
| crates/engine-geo/src/py.rs | — | 40 | 0 |
| crates/engine-geo/src/tests.rs | 45 | 0 | 1 |
| crates/engine-geo/src/tz.rs | — | 30 | 0 |
| crates/engine-geo/src/winkler.rs | 83 | 5 | 0 |
| crates/engine-laws/src/lib.rs | 48 | 3 | 0 |
| crates/engine-laws/src/section.rs | 229 | 3 | 0 |
| crates/engine-laws/src/uniaxial.rs | — | 1 | 0 |
| crates/engine-laws/src/uniaxial/cubic_elastic.rs | — | 2 | 0 |
| crates/engine-laws/src/uniaxial/multilinear.rs | 216 | 14 | 0 |
| crates/engine-laws/src/uniaxial/steel_bilinear.rs | — | 4 | 0 |
| crates/engine-linalg/src/budget.rs | 116–220 (5 lines) | 5 | 0 |
| crates/engine-linalg/src/faer_backend.rs | — | 1 | 0 |
| crates/engine-linalg/src/guyan.rs | — | 4 | 0 |
| crates/engine-linalg/src/lib.rs | 185–212 (4 lines) | 6 | 0 |
| crates/engine-linalg/src/matrix.rs | 21–192 (9 lines) | 9 | 0 |
| crates/engine-linalg/src/modes.rs | 74–142 (4 lines) | 6 | 0 |
| crates/engine-linalg/src/reference.rs | 284–630 (6 lines) | 15 | 0 |
| crates/engine-linalg/src/sparse.rs | 33 | 2 | 0 |
| crates/engine-loads/src/lib.rs | — | 2 | 0 |
| crates/engine-loads/src/mass.rs | 76–276 (3 lines) | 10 | 0 |
| crates/engine-loads/src/nodal.rs | 18–28 (2 lines) | 3 | 0 |
| crates/engine-loads/src/span.rs | 152–245 (2 lines) | 3 | 0 |
| crates/engine-loads/src/weight.rs | — | 1 | 0 |
| crates/engine-model/src/lib.rs | 259–601 (8 lines) | 21 | 0 |
| crates/engine-model/src/tests.rs | 9–346 (40 lines) | 0 | 9 |
| crates/engine-validation/src/fixtures.rs | 163–405 (8 lines) | 20 | 0 |

## Rows and text deliberately left unchanged

- engine-cli/src/lib.rs, audit row 427: the audited literal is the guarded test-reader error (now near line 831), not a production message. The other Spanish strings there are test-only synthetic diagnostics. The library and its README remain untouched to minimize overlap with the open CLI work.
- engine-contracts/src/tests.rs, audit row 27–1024: test assertion failure messages remain; matching expectations are translated. engine-geo/src/tests.rs, audit row 45: only the catalog-text expectation changes. engine-model/src/tests.rs, audit row 9–346: Spanish fixture IDs, parameter names, test-only keys, and assertion prose remain; displayed context expectations change.
- engine-api/src/methods.rs, audit row 135–240 and newer entries: method evidence/citation strings are metadata, not error/diagnostic messages, causes, hints, or usage.
- Embedded factory parameter schemas remain unchanged, including descriptions. Benchmark fixture descriptions and index pointer/hash descriptions remain unchanged; they are model/evidence metadata rather than messages.
- engine-api/src/run.rs: the API-W-DIAPHRAGM-RECOVERY cause beginning "#84:" remains because its exact text is copied into archived docs/validation-report/fixtures/val032 JSON documents and the reference generator, without a test that pins those message copies.
- engine-api/src/run/advanced.rs: the P-Delta cause beginning "P-Delta: consulte convergence" remains because its exact text is copied into docs/validation-report/fixtures/val032/_ref/engine_r3.json without a test that pins that message copy.
- engine-api/src/run.rs: both occurrences of the serialization failure cause remain because the constant fallback results/1 document has no direct test covering its cause. This preserves the same text in the ordinary serialization error and fallback.
- Internal expect/precondition assertion messages, including registered-code assertions in engine-model, engine-geo, and engine-contracts, remain because they are internal invariant text. Test-only messages and fixture identifiers in other src test modules remain outside the translation scope.

## Contract-visible expected text changes

Each row lists an exact full-string or substring expectation changed or added in an existing test. Repeated occurrences in the same file are grouped. These tests cover the corresponding output text. The core catalog completeness tests additionally cover all catalog entries and preserve their stable code meanings. No stdout/schema keys or messageKey values change.

| Test or doctest file | Old expected text → new expected text |
| --- | --- |
| crates/engine-analysis/tests/analytical.rs | `Nodo 2` → `Node 2` |
| crates/engine-analysis/tests/analytical.rs | `GDL Uy` → `DOF Uy` |
| crates/engine-analysis/tests/combination.rs | `caso propio de asentamiento` → `separate settlement case` |
| crates/engine-analysis/tests/combination.rs, `incoherent_prescribed_metadata_rejected_with_case_entity` | `Desplazamiento prescrito incoherente con el resultado nodal.` → `Prescribed displacement is inconsistent with the nodal result.` (added exact assertion; also checks `ENG-E-0010` and the `Case("B")` entity) |
| crates/engine-analysis/tests/modal.rs | `Nodo 2, GDL Rz (global 2).` → `Node 2, DOF Rz (global 2).` |
| crates/engine-analysis/tests/modal.rs | `multiplicidad` → `multiplicity` |
| crates/engine-analysis/tests/newton.rs | `Nodo 1, GDL Ux` → `Node 1, DOF Ux` |
| crates/engine-api/tests/dynamic.rs | `Nodo 2, GDL Uy (global 3).` → `Node 2, DOF Uy (global 3).` |
| crates/engine-api/tests/dynamic.rs | `redondeo` → `rounding` |
| crates/engine-api/tests/limits.rs | `Entero JSON fuera de rango` → `JSON integer is outside` |
| crates/engine-api/tests/limits.rs | `fuera del rango finito f64` → `outside the finite f64 range` |
| crates/engine-api/tests/sfc010.rs | `nodos libres` → `free nodes` |
| crates/engine-assembly/tests/assembly.rs | `Nodo` → `Node` |
| crates/engine-bench/tests/capabilities.rs | `no tiene opciones por defecto` → `has no default options` |
| crates/engine-contracts/src/tests.rs | `error relativo` → `relative error` |
| crates/engine-contracts/src/tests.rs | `finitos` → `finite` |
| crates/engine-contracts/src/tests.rs | `Tensión aceptada` → `Accepted stress` |
| crates/engine-contracts/src/tests.rs | `Longitud esperada` → `Expected length` |
| crates/engine-contracts/src/tests.rs | `Simetría` → `Symmetry` |
| crates/engine-contracts/src/tests.rs | `Contrato` → `Contract` |
| crates/engine-contracts/src/tests.rs | `Modo rígido` → `Rigid mode` |
| crates/engine-contracts/src/tests.rs | `Tangente inconsistente` → `Inconsistent tangent` |
| crates/engine-contracts/src/tests.rs | `Covarianza` → `covariance` |
| crates/engine-contracts/src/tests.rs | `Simetría masa` → `Symmetry mass` |
| crates/engine-contracts/src/tests.rs | `incremento` → `increment` |
| crates/engine-contracts/src/tests.rs | `muestras` → `samples` |
| crates/engine-contracts/src/tests.rs | `fuera de` → `outside` |
| crates/engine-contracts/src/tests.rs | `Longitud` → `length` |
| crates/engine-contracts/src/tests.rs | `Contexto` → `context` |
| crates/engine-contracts/src/tests.rs | `no rechazada` → `was not rejected` |
| crates/engine-contracts/src/tests.rs | `Modo rígido {component}` → `Rigid mode {component}` |
| crates/engine-contracts/src/tests.rs | `sobrescribir` → `overwrite` |
| crates/engine-contracts/src/tests.rs | `cambió de dimensión` → `changed dimensions` |
| crates/engine-contracts/src/tests.rs | `cancelación` → `cancellation` |
| crates/engine-core/src/catalog.rs | `Unidad desconocida` → `Unknown unit` |
| crates/engine-core/src/catalog.rs | `Capacidad no soportada` → `Unsupported capability` |
| crates/engine-core/src/lib.rs | `ENG-E-0005: La longitud del miembro es nula.` → `ENG-E-0005: The member length is zero.` |
| crates/engine-elements-spring/tests/invariants.rs | `Resorte: la fábrica requiere exactamente un GDL de la dimensión seleccionada.` → `Spring: factory requires exactly one DOF of the selected dimension.` |
| crates/engine-geo/src/tests.rs | `catálogo` → `catalog` |
| crates/engine-geo/tests/consolidation.rs | `cv incompatible` → `cv is incompatible` |
| crates/engine-geo/tests/tz.rs | `Explicit requiere resistencia ≤ 0 para z < 0 y ≥ 0 para z > 0` → `Explicit option requires resistance ≤ 0 for z < 0 and ≥ 0 for z > 0` |
| crates/engine-geo/tests/tz.rs | `ablandamiento` → `softening` |
| crates/engine-geo/tests/tz.rs | `Resistencia reducida a cero al adaptar unidades` → `Resistance reduced to zero when adapting units` |
| crates/engine-geo/tests/tz.rs | `producto` → `product` |
| crates/engine-geo/tests/tz.rs | `incremento` → `increment` |
| crates/engine-geo/tests/tz.rs | `pendiente` → `slope` |
| crates/engine-geo/tests/tz.rs | `subdesbordamiento` → `underflow` |
| crates/engine-linalg/tests/guyan.rs | `bloque del GDL {dof}.` → `DOF block {dof}.` |
| crates/engine-linalg/tests/lowest_modes_anti_revert.rs | `sin ordenar` → `unordered` |
| crates/engine-linalg/tests/lowest_modes_anti_revert.rs | `inercia incompatible` → `inertia incompatible` |
| crates/engine-linalg/tests/lowest_modes_anti_revert.rs | `corte de multiplicidad` → `multiplicity cutoff` |
| crates/engine-linalg/tests/modes.rs | `negativo` → `negative` |
| crates/engine-linalg/tests/modes.rs | `MODOS FALTANTES` → `MISSING MODES` |
| crates/engine-linalg/tests/robustness.rs | `GDL 1` → `DOF 1` |
| crates/engine-linalg/tests/robustness.rs | `GDL 0` → `DOF 0` |
| crates/engine-linalg/tests/robustness.rs | `GDL {zero}` → `DOF {zero}` |
| crates/engine-linalg/tests/robustness.rs | `GDL {first}` → `DOF {first}` |
| crates/engine-model/src/tests.rs | `Nodo 1` → `Node 1` |
| crates/engine-model/src/tests.rs | `Miembro 10` → `Member 10` |
| crates/engine-model/src/tests.rs | `Nodo 3` → `Node 3` |
| crates/engine-model/src/tests.rs | `tolerancia` → `tolerance` |
| crates/engine-model/src/tests.rs | `carga 0` → `load 0` |
| crates/engine-model/src/tests.rs | `Nodo 2` → `Node 2` |
| crates/engine-validation/tests/fixtures.rs | `prohibida` → `prohibited` |
| crates/engine-validation/tests/fixtures.rs | `FALLO` → `FAILED` |
| crates/engine-validation/tests/fixtures.rs | `esperado=` → `expected=` |

The VAL-032 test also pins the bytes of two translated source files. Only these expected SHA-256 values change in docs/validation-report/fixtures/val032/index.json:

| Referenced source | Old SHA-256 → new SHA-256 |
| --- | --- |
| crates/engine-analysis/src/pdelta.rs | 39200dd3e75ea59a81095e8e4a8ce98b61d75fa2941440b42c978149c9426f16 → 57abffed35ee7e97c56fb03981f1483a58a3b54526d78e2d5d2d7252d0eb1515 |
| crates/engine-analysis/src/modal.rs | 8dc1c4afc74309baca1a55015cc132d6c105c9c22cb1c37b2d8997bbbd357442 → 972fe35570aabeed83f4aa8ddd0c82bc4c1a77f4a5f09b94256c01c153d031ca |

## Validation

All checks completed on 2026-10-22, offline.

- cargo fmt --all --check: passed.
- cargo clippy --workspace --all-targets --offline -- -D warnings: passed.
- cargo test --workspace --offline: passed; 813 Rust tests/doctests across 124 suites, no failures or ignored tests. The benchmark suite also ran its seven Python tests successfully.
- bash clean-room/check.sh origin/main: passed. The real index is unchanged.
- bash scripts/changelog-check.sh origin/main: passed. This script examines committed HEAD differences and reports no committed code changes; a supplemental working-tree check verified the new fragment filename, nonempty content, absence of headings, and unchanged CHANGELOG.md.
- python3 scripts/generate-schemas.py --check: passed; published schema bytes, index, and inventory remain current.
- git diff --check: passed. A source token comparison confirmed unchanged code outside string literals and required rustfmt punctuation/whitespace. An additional identifier scan covered all added text, including this provenance file.

The first two workspace test runs exposed Spanish text assertions and the two pinned source hashes; their expected values were updated individually. The final exact required workspace test command passed. No commit was created.

### MAM-841 r1 regression coverage

The prescribed-displacement consistency diagnostic in crates/engine-analysis/src/combination.rs (near line 420) is translated. The existing `incoherent_prescribed_metadata_rejected_with_case_entity` test in crates/engine-analysis/tests/combination.rs now asserts its exact English text for the finite mismatch, alongside the existing code and Case entity checks. Running that test before translating the literal failed on the exact message assertion, showing that it reaches this diagnostic branch.

After the translation, the targeted test passed. The r1 checks also passed: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --offline -- -D warnings`, and `cargo test --workspace --offline`. All used the designated shared Cargo target directory; temporary build files stayed outside the system temporary directory. No commit was created.
