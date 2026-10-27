# ENG-LANG-004 — Analysis and element crate Rust comments in English

- Ticket: MAM-838 (ENG-LANG-004).
- Date: 2026-10-27.
- Base: `origin/main`, `7583bce29a1cb4346743e3ea945a92d9976871f8`.
- Sources: existing Rust source comments, the base language audit, the ticket
  instructions, `provenance/ENG-LANG-003.md`, `provenance/ENG-LANG-005.md`,
  `changelog.d/MAM-842.changed.md`, the base commit summary, and
  `changelog.d/README.md`. Work and verification were offline. No third-party
  code, numerical changes, or new mathematical derivations were introduced.

## Scope and translated lines

Reviewed all 66 Rust files under the eight requested `src/` trees; 63 files
contain translated text. Comments inside inline `#[cfg(test)]` modules in
`src/` were translated. The digest-pinned `modal.rs` and `pdelta.rs`, and the
separate test file `crates/engine-geo/src/tests.rs`, remain unchanged as
detailed below. Files in `tests/`, `examples/`, and `benches/`, and README
files, are outside this tranche.

Counts come from the actual `git diff --numstat origin/main` for Rust sources;
each file has equal added and deleted counts, and each changed physical line
counts once. Added comment lines are split into rustdoc (`///` or `//!`) and
ordinary code comments, including comments following code on the same line.
Existing English, unchanged symbolic expressions, blank lines, and executable
code are excluded. Rustdoc lines include three translated
comments inside documentation examples; their executable statements and string
literals are unchanged. Physical line counts and ordering are unchanged.

| Rust source file | Rustdoc lines | Code comment lines | Total translated lines |
| --- | ---: | ---: | ---: |
| `crates/engine-analysis/src/combination.rs` | 78 | 1 | 79 |
| `crates/engine-analysis/src/lib.rs` | 49 | 1 | 50 |
| `crates/engine-analysis/src/newton.rs` | 68 | 3 | 71 |
| `crates/engine-analysis/src/results.rs` | 108 | 4 | 112 |
| `crates/engine-analysis/src/spectrum/combination.rs` | 22 | 1 | 23 |
| `crates/engine-analysis/src/spectrum/mod.rs` | 63 | 1 | 64 |
| `crates/engine-analysis/src/spectrum/table.rs` | 25 | 3 | 28 |
| `crates/engine-loads/src/lib.rs` | 13 | 1 | 14 |
| `crates/engine-loads/src/mass.rs` | 60 | 0 | 60 |
| `crates/engine-loads/src/nodal.rs` | 13 | 1 | 14 |
| `crates/engine-loads/src/span.rs` | 37 | 2 | 39 |
| `crates/engine-loads/src/weight.rs` | 8 | 0 | 8 |
| `crates/engine-laws/src/lib.rs` | 17 | 0 | 17 |
| `crates/engine-laws/src/section.rs` | 57 | 0 | 57 |
| `crates/engine-laws/src/uniaxial/cubic_elastic.rs` | 24 | 0 | 24 |
| `crates/engine-laws/src/uniaxial/multilinear.rs` | 40 | 1 | 41 |
| `crates/engine-laws/src/uniaxial/steel_bilinear.rs` | 36 | 1 | 37 |
| `crates/engine-laws/src/uniaxial.rs` | 25 | 0 | 25 |
| `crates/engine-geo/src/consolidation.rs` | 67 | 4 | 71 |
| `crates/engine-geo/src/lib.rs` | 82 | 0 | 82 |
| `crates/engine-geo/src/profile.rs` | 29 | 1 | 30 |
| `crates/engine-geo/src/py.rs` | 96 | 2 | 98 |
| `crates/engine-geo/src/tz.rs` | 99 | 4 | 103 |
| `crates/engine-geo/src/winkler.rs` | 12 | 0 | 12 |
| `crates/engine-elements-truss/src/factory.rs` | 6 | 0 | 6 |
| `crates/engine-elements-truss/src/geometric.rs` | 2 | 0 | 2 |
| `crates/engine-elements-truss/src/geometry.rs` | 1 | 0 | 1 |
| `crates/engine-elements-truss/src/lib.rs` | 31 | 0 | 31 |
| `crates/engine-elements-truss/src/loads.rs` | 1 | 2 | 3 |
| `crates/engine-elements-truss/src/mass.rs` | 1 | 0 | 1 |
| `crates/engine-elements-truss/src/producer.rs` | 11 | 0 | 11 |
| `crates/engine-elements-truss/src/recover.rs` | 1 | 1 | 2 |
| `crates/engine-elements-truss/src/stiffness.rs` | 1 | 6 | 7 |
| `crates/engine-elements-frame/src/factory.rs` | 4 | 0 | 4 |
| `crates/engine-elements-frame/src/geometric.rs` | 1 | 0 | 1 |
| `crates/engine-elements-frame/src/geometry.rs` | 1 | 0 | 1 |
| `crates/engine-elements-frame/src/lib.rs` | 43 | 0 | 43 |
| `crates/engine-elements-frame/src/loads.rs` | 1 | 0 | 1 |
| `crates/engine-elements-frame/src/mass.rs` | 1 | 0 | 1 |
| `crates/engine-elements-frame/src/recover.rs` | 1 | 5 | 6 |
| `crates/engine-elements-frame/src/release.rs` | 5 | 1 | 6 |
| `crates/engine-elements-frame/src/stiffness.rs` | 1 | 2 | 3 |
| `crates/engine-elements-frame/src/winkler.rs` | 11 | 1 | 12 |
| `crates/engine-elements-frame3d/src/factory.rs` | 5 | 0 | 5 |
| `crates/engine-elements-frame3d/src/geometric.rs` | 1 | 0 | 1 |
| `crates/engine-elements-frame3d/src/geometry.rs` | 3 | 5 | 8 |
| `crates/engine-elements-frame3d/src/lib.rs` | 48 | 2 | 50 |
| `crates/engine-elements-frame3d/src/loads.rs` | 1 | 0 | 1 |
| `crates/engine-elements-frame3d/src/mass.rs` | 1 | 0 | 1 |
| `crates/engine-elements-frame3d/src/producer.rs` | 11 | 0 | 11 |
| `crates/engine-elements-frame3d/src/recover.rs` | 1 | 2 | 3 |
| `crates/engine-elements-frame3d/src/release.rs` | 5 | 8 | 13 |
| `crates/engine-elements-frame3d/src/stiffness.rs` | 1 | 3 | 4 |
| `crates/engine-elements-frame3d/src/winkler.rs` | 11 | 1 | 12 |
| `crates/engine-elements-spring/src/factory.rs` | 9 | 5 | 14 |
| `crates/engine-elements-spring/src/geometric.rs` | 1 | 0 | 1 |
| `crates/engine-elements-spring/src/geometry.rs` | 6 | 2 | 8 |
| `crates/engine-elements-spring/src/lib.rs` | 50 | 0 | 50 |
| `crates/engine-elements-spring/src/loads.rs` | 1 | 0 | 1 |
| `crates/engine-elements-spring/src/mass.rs` | 1 | 0 | 1 |
| `crates/engine-elements-spring/src/nonlinear.rs` | 20 | 1 | 21 |
| `crates/engine-elements-spring/src/recover.rs` | 1 | 0 | 1 |
| `crates/engine-elements-spring/src/stiffness.rs` | 1 | 1 | 2 |
| **Total** | **1,430** | **79** | **1,509** |

| Crate | Reviewed / changed Rust files | Rustdoc lines | Code comment lines | Total translated lines |
| --- | ---: | ---: | ---: | ---: |
| engine-analysis | 9 / 7 | 413 | 14 | 427 |
| engine-loads | 5 / 5 | 131 | 4 | 135 |
| engine-laws | 6 / 6 | 199 | 2 | 201 |
| engine-geo | 7 / 6 | 385 | 11 | 396 |
| engine-elements-truss | 9 / 9 | 55 | 9 | 64 |
| engine-elements-frame | 10 / 10 | 69 | 9 | 78 |
| engine-elements-frame3d | 11 / 11 | 88 | 21 | 109 |
| engine-elements-spring | 9 / 9 | 90 | 9 | 99 |
| **Total** | **66 / 63** | **1,430** | **79** | **1,509** |

Only comment prose changed in Rust. Identifiers, attributes, executable
statements, their order, and all string literals remain byte-identical after
removing comments. Intra-doc links are unchanged. Fenced example statements
and literals are unchanged after removing their comments. Existing equations,
source citations, stable codes, and API references retain their meaning.
Goldens, snapshots, the describe-1 hash, schemas, and index.json are unchanged.
Protected files and existing provenance were not edited.

## Terminology

Terminology follows ENG-LANG-003 and ENG-LANG-005: DOF, node, member, beam,
truss, spring, station, prescribed displacement, settlement, load case, mode
shape, base shear, recovery, end actions, and accepted/trial state. Frame
comments use moment release, static condensation, Guyan reduction, and
geometric stiffness. Geotechnical comments use groundwater level, unit weight,
effective stress, pore pressure, settlement, load breakpoint, substep,
foundation, and plateau. Existing units and sign conventions are retained.

## Runtime note

No translated doc comment is exported to runtime output. The reviewed sources
use ordinary Rust documentation and built-in derives; no documentation-reading
macro or exported doc attribute consumes this prose. Factory schemas and their
human-readable descriptions are string literals, remain unchanged, and are
included below where Spanish remains. No runtime values or outputs change.

## Remaining (later tranches)

- `crates/engine-analysis/src/pdelta.rs` and
  `crates/engine-analysis/src/modal.rs` retain their Spanish rustdoc and code
  comments. Their full-file SHA-256 digests are pinned by
  `docs/validation-report/fixtures/val032/index.json` and checked by
  `crates/engine-api/tests/val032_runs.rs`. The sources and fixture index remain
  byte-identical to the base. Translating these files needs a separate decision
  to regenerate the index.
- `crates/engine-geo/src/tests.rs` retains 13 Spanish test-comment lines. This
  separate test file is outside the translation scope; inline `#[cfg(test)]`
  modules within the translated source files were included.
- README prose remains in `crates/engine-loads/README.md`; the other seven
  crates have no README file at this base.
- Comments and rustdoc under the following `tests/` trees remain untouched.
  These eight crates have no `examples/` or `benches/` Rust files at this base;
  those trees remain outside the scope of this tranche.

| Crate | Unedited Rust files under tests/ |
| --- | ---: |
| `crates/engine-analysis/tests/` | 10 |
| `crates/engine-loads/tests/` | 5 |
| `crates/engine-laws/tests/` | 4 |
| `crates/engine-geo/tests/` | 4 |
| `crates/engine-elements-truss/tests/` | 4 |
| `crates/engine-elements-frame/tests/` | 4 |
| `crates/engine-elements-frame3d/tests/` | 4 |
| `crates/engine-elements-spring/tests/` | 4 |

All string literals are deliberately unchanged. The following 29 source lines
retain Spanish, including short diagnostic labels, schema descriptions, and
the documentation example case name. Of these, 28 lines are in translated
files and one is in the deferred digest-pinned `pdelta.rs`. Line numbers match
the base.

| Spanish string literal locations retained verbatim | Source status |
| --- | --- |
| `crates/engine-analysis/src/pdelta.rs:208` | Deferred: digest-pinned source. |
| `crates/engine-elements-frame/src/factory.rs:15` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame/src/factory.rs:77` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame/src/lib.rs:183` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame/src/lib.rs:204` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame/src/lib.rs:228` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame/src/mass.rs:18` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame/src/mass.rs:32` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/factory.rs:16` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/factory.rs:116` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/lib.rs:227` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/lib.rs:246` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/lib.rs:269` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/mass.rs:15` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/mass.rs:25` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/mass.rs:29` | Translated comments; literal unchanged. |
| `crates/engine-elements-frame3d/src/mass.rs:44` | Translated comments; literal unchanged. |
| `crates/engine-elements-spring/src/factory.rs:36` | Translated comments; literal unchanged. |
| `crates/engine-elements-spring/src/factory.rs:48` | Translated comments; literal unchanged. |
| `crates/engine-elements-spring/src/geometric.rs:15` | Translated comments; literal unchanged. |
| `crates/engine-elements-spring/src/geometry.rs:128` | Translated comments; literal unchanged. |
| `crates/engine-elements-spring/src/stiffness.rs:9` | Translated comments; literal unchanged. |
| `crates/engine-elements-truss/src/factory.rs:30` | Translated comments; literal unchanged. |
| `crates/engine-elements-truss/src/factory.rs:41` | Translated comments; literal unchanged. |
| `crates/engine-elements-truss/src/mass.rs:19` | Translated comments; literal unchanged. |
| `crates/engine-geo/src/consolidation.rs:261` | Translated comments; literal unchanged. |
| `crates/engine-geo/src/profile.rs:29` | Translated comments; literal unchanged. |
| `crates/engine-geo/src/py.rs:147` | Translated comments; literal unchanged. |
| `crates/engine-loads/src/mass.rs:28` | Translated comments; literal unchanged. |

Existing symbolic names inside comment equations also remain verbatim:
`valor`, `K_lineal`, `K_elemento`, `historia_aceptada`, `u_nuevo`, `u_previo`,
`piso_i`, `f_carga`, `Σ_apoyos`, `R_traslacional`, `ζ_tabla`, `Z_superficie`,
`espesor_total`, `N_elástico`, `Kflexión`, `u_recuperado`, and `zmeseta`.
These are formula references, not untranslated comment prose.

## Spanish heuristic

Rechecked all 66 reviewed source files, including the three deferred files,
for Spanish diacritics or punctuation,
or at least two Spanish function words, using case-insensitive whole words:
que, para, del, una, los, las, por, sin, debe, según, el, la, de, en, con,
un, se, al, como, no, es, son, su, sus, entre, cada, todos, todas, también,
más, antes, después, hasta, sobre, solo, desde, durante.

The scan reports 152 raw source-line hits: 132 Spanish comment lines in the
three deferred files (50 in `modal.rs`, 70 in `pdelta.rs`, and 12 in the
separate geotechnical test file), seven Spanish string-literal lines, and
13 false positives or preserved formula references. The latter comprise
four lines with the Rust variable `el`, two English sentences containing
repeated `no`, four citations of Hetényi or mathematical `sin`, and three
lines containing `N_elástico` or `Kflexión`. No Spanish comment prose remains
in the 63 translated files. Manual review finds one additional Spanish test
comment missed by the heuristic (13 total in the deferred test file) and adds
22 literal lines, producing the unchanged 29-line literal inventory above,
including `pdelta.rs:208` and the short `peso` example literal.

| Remaining heuristic location | Classification |
| --- | --- |
| `crates/engine-analysis/src/modal.rs` (lines 1, 3–5, 7, 9–12, 14–19, 28, 31, 34, 36, 38, 40, 44, 57, 59, 63, 66, 68, 70, 72, 74, 78, 81, 85, 87, 89, 94, 97–103, 113–114, 125–126, 198–200) | 50 hits: Deferred Spanish comments; source digest pinned by val032. |
| `crates/engine-analysis/src/pdelta.rs` (lines 1, 3–4, 6–13, 22, 25–26, 28, 30, 32, 36, 39, 41, 43, 45, 50, 53, 58, 63–64, 69–81, 85, 88, 92, 96, 98, 100, 102, 104, 106–107, 111, 115, 119–123, 133–134, 145, 170, 232, 300–305, 383, 462) | 70 hits: Deferred Spanish comments; source digest pinned by val032. |
| `crates/engine-geo/src/tests.rs` (lines 50–51, 107, 170, 242, 248, 258, 292, 301, 317, 324, 334) | 12 hits: Deferred Spanish test comments. |
| `crates/engine-analysis/src/spectrum/mod.rs:20` | English prose: repeated no. |
| `crates/engine-geo/src/consolidation.rs:337` | Rust variable el in executable code. |
| `crates/engine-geo/src/consolidation.rs:338` | Rust variable el in executable code. |
| `crates/engine-geo/src/consolidation.rs:345` | Rust variable el in executable code. |
| `crates/engine-geo/src/consolidation.rs:575` | Rust variable el in executable code. |
| `crates/engine-geo/src/profile.rs:29` | Spanish string literal retained verbatim. |
| `crates/engine-geo/src/profile.rs:47` | Unchanged English literal: repeated no. |
| `crates/engine-geo/src/winkler.rs:1` | Proper-name citation in English prose. |
| `crates/engine-elements-truss/src/factory.rs:30` | Spanish string literal retained verbatim. |
| `crates/engine-elements-truss/src/recover.rs:1` | Preserved equation symbol N_elástico. |
| `crates/engine-elements-frame/src/factory.rs:15` | Spanish string literal retained verbatim. |
| `crates/engine-elements-frame/src/factory.rs:77` | Spanish string literal retained verbatim. |
| `crates/engine-elements-frame/src/lib.rs:25` | Preserved equation symbol Kflexión. |
| `crates/engine-elements-frame/src/release.rs:10` | Preserved equation symbol Kflexión. |
| `crates/engine-elements-frame/src/winkler.rs:14` | Proper-name citation in English prose. |
| `crates/engine-elements-frame3d/src/factory.rs:16` | Spanish string literal retained verbatim. |
| `crates/engine-elements-frame3d/src/factory.rs:116` | Spanish string literal retained verbatim. |
| `crates/engine-elements-frame3d/src/geometry.rs:54` | Mathematical sin in English prose. |
| `crates/engine-elements-frame3d/src/winkler.rs:14` | Proper-name citation in English prose. |
| `crates/engine-elements-spring/src/factory.rs:36` | Spanish string literal retained verbatim. |

## Verification

Fresh checks were run offline with Cargo invoked directly. No commit was
created.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | PASS. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | PASS. |
| `cargo test --workspace --offline` | PASS: 850 Rust tests (805 unit/integration and 45 doctests), 0 failed, 0 ignored, across 130 groups (108 unit/integration and 22 doctest groups). Both embedded Python suites passed 7 tests each (14 total). |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --offline` | PASS. |
| `bash clean-room/check.sh origin/main` | PASS, including both new documents registered with `git add -N` in a separate `GIT_INDEX_FILE`. |
| `bash scripts/changelog-check.sh origin/main` | PASS; the script compares committed HEAD changes and reports no code changes. Supplemental working-tree fragment checks also pass. |
| `python3 scripts/generate-schemas.py --check` | PASS: schema bytes, index, and inventory are current. |
| `git diff --check` | PASS. The additional `git diff --check origin/main` also passes with both new documents included through the separate index. |

`engine-api --test val032_runs` passes, including the full-file SHA-256 checks
against `docs/validation-report/fixtures/val032/index.json`. The two pinned
sources, the separate geotechnical test file, and the fixture index remain
byte-identical to the base. All 63 translated sources retain their physical
line counts and identical non-comment prefixes on changed lines. Their file
and crate counts were independently recomputed from the current diff.

Intent-to-add registration used a separate writable Git index and object
directory, with the existing object database as a read-only alternate. The
real Git index is unchanged. Supplemental changelog checks validate the new
fragment name, nonempty Spanish text, required ticket marker, absence of
headings, and unchanged CHANGELOG.md in the working tree. Only this report
was edited during the follow-up; no Rust sources or changelog text changed.
