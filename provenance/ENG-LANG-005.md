# ENG-LANG-005 — API, CLI, WASM, and Python Rust comments in English

- Ticket: MAM-842 (ENG-LANG-005), tranche 1.
- Date: 2026-10-27.
- Base: `origin/main`, `29503f07cd26babe7e560cdeab2d4a779a874375`.
- Sources: existing Rust source comments, the language audit, the ticket scope,
  `provenance/ENG-LANG-001.md`, `provenance/ENG-LANG-002.md`,
  `provenance/ENG-LANG-003.md`, and `changelog.d/README.md`.
  Work and verification were offline. No third-party code, numerical changes,
  or new mathematical derivations were introduced.

## Scope and translated lines

Reviewed all 25 Rust files under the four requested `src/` trees. Translated
Spanish rustdoc and code comments in 22 files, including comments in inline
Rust test modules within those source files. No files under `tests/` or
`examples/` were edited. WASM comments, API localization comments, and CLI
signal comments were already English and remain unchanged.

Counts compare corresponding physical lines with the base; each changed
comment line counts once. Existing English, blank lines, and executable code
are excluded. Physical line counts and ordering are unchanged.

| Rust source file | Rustdoc lines | Code comment lines | Total translated lines |
| --- | ---: | ---: | ---: |
| `bindings/python/src/lib.rs` | 4 | 4 | 8 |
| `bindings/python/src/unwind.rs` | 1 | 1 | 2 |
| `crates/engine-api/src/analysis_input.rs` | 1 | 0 | 1 |
| `crates/engine-api/src/extensions.rs` | 1 | 1 | 2 |
| `crates/engine-api/src/geo_input.rs` | 12 | 2 | 14 |
| `crates/engine-api/src/i18n.rs` | 0 | 0 | 0 |
| `crates/engine-api/src/input.rs` | 1 | 3 | 4 |
| `crates/engine-api/src/json.rs` | 1 | 2 | 3 |
| `crates/engine-api/src/lib.rs` | 49 | 1 | 50 |
| `crates/engine-api/src/limits.rs` | 11 | 4 | 15 |
| `crates/engine-api/src/methods.rs` | 19 | 4 | 23 |
| `crates/engine-api/src/query.rs` | 17 | 1 | 18 |
| `crates/engine-api/src/run/advanced.rs` | 1 | 4 | 5 |
| `crates/engine-api/src/run/geo.rs` | 1 | 0 | 1 |
| `crates/engine-api/src/run/nonlinear.rs` | 1 | 2 | 3 |
| `crates/engine-api/src/run/static_linear.rs` | 1 | 0 | 1 |
| `crates/engine-api/src/run.rs` | 24 | 5 | 29 |
| `crates/engine-cli/src/describe.rs` | 1 | 3 | 4 |
| `crates/engine-cli/src/error.rs` | 2 | 1 | 3 |
| `crates/engine-cli/src/lib.rs` | 21 | 24 | 45 |
| `crates/engine-cli/src/main.rs` | 0 | 1 | 1 |
| `crates/engine-cli/src/progress.rs` | 1 | 5 | 6 |
| `crates/engine-cli/src/signals.rs` | 0 | 0 | 0 |
| `crates/engine-cli/src/timeout.rs` | 1 | 9 | 10 |
| `crates/engine-wasm/src/lib.rs` | 0 | 0 | 0 |
| **Total** | **171** | **77** | **248** |

| Crate / binding | Reviewed Rust files | Changed Rust files | Rustdoc lines | Code comment lines | Total translated lines |
| --- | ---: | ---: | ---: | ---: | ---: |
| engine-api | 15 | 14 | 140 | 29 | 169 |
| engine-cli | 7 | 6 | 26 | 43 | 69 |
| engine-wasm | 1 | 0 | 0 | 0 | 0 |
| bindings/python | 2 | 2 | 5 | 5 | 10 |
| **Total** | **25** | **22** | **171** | **77** | **248** |

Only comment prose changed in Rust. Identifiers, attributes, string literals,
JSON keys, stable codes, message keys, executable statements, and their order
remain byte-identical after removing comments. Inline code references, doc
links, and fenced example code are preserved. Goldens, snapshots (including
the describe-1 hash), schema sources, published schemas, and index.json remain
byte-identical. Existing provenance and protected files were not edited.

One runtime-visible effect: PyO3 exposes Rust doc comments as Python
docstrings, so `_native.run.__doc__` (`bindings/python/src/lib.rs`, the
`run` function) and `_native.CancelToken.__doc__` (the `CancelToken` class)
are now English. No test covers these two docstrings. No other runtime value
changes.

## Terminology

Terminology follows ENG-LANG-003: DOF, node, member, beam, truss, spring,
station, prescribed displacement, load case, mode shape, base shear, recovery,
and end actions. Existing English terms and API identifiers retain their form.
Boundary, facade, adapter, checkpoint, cooperative cancellation, canonical JSON,
fingerprint, host, and terminal envelope describe the public integration layer.
Geotechnical wording uses load breakpoints, substeps, free nodes, and saturating
u128 arithmetic consistently with the existing English explanations.

## Remaining (later tranches)

- Python package docstrings and comments under `bindings/python/python/**`,
  including `.pyi` text, remain outside this tranche.
- README prose remains in `crates/engine-api/README.md`,
  `crates/engine-cli/README.md`, `crates/engine-wasm/README.md`, and
  `bindings/python/README.md`.
- Comments in the four components' `tests/` and `examples/` trees remain for
  later tranches. Other files outside `src/`, including the API build script,
  Python project configuration, and binding notice, were not edited.
- All string literals are deliberately unchanged. The following 65 source
  lines contain Spanish prose, including discovery/help descriptions,
  capability evidence citations, diagnostics, fallback messages, and inline
  test data. Line numbers match the base. Locale identifiers and symbolic
  expressions alone are not Spanish prose.

| Spanish string literal locations retained verbatim |
| --- |
| `bindings/python/src/lib.rs:31` |
| `bindings/python/src/lib.rs:34` |
| `bindings/python/src/lib.rs:52` |
| `bindings/python/src/lib.rs:113` |
| `bindings/python/src/lib.rs:219` |
| `bindings/python/src/lib.rs:222` |
| `bindings/python/src/lib.rs:251` |
| `crates/engine-api/src/methods.rs:244` |
| `crates/engine-api/src/methods.rs:253` |
| `crates/engine-api/src/methods.rs:262` |
| `crates/engine-api/src/methods.rs:274` |
| `crates/engine-api/src/methods.rs:286` |
| `crates/engine-api/src/methods.rs:299` |
| `crates/engine-api/src/methods.rs:311` |
| `crates/engine-api/src/methods.rs:324` |
| `crates/engine-api/src/run.rs:214` |
| `crates/engine-api/src/run.rs:296` |
| `crates/engine-api/src/run.rs:312` |
| `crates/engine-api/src/run/advanced.rs:245` |
| `crates/engine-cli/src/describe.rs:82` |
| `crates/engine-cli/src/describe.rs:90` |
| `crates/engine-cli/src/describe.rs:98` |
| `crates/engine-cli/src/describe.rs:106` |
| `crates/engine-cli/src/describe.rs:115` |
| `crates/engine-cli/src/describe.rs:117` |
| `crates/engine-cli/src/describe.rs:118` |
| `crates/engine-cli/src/describe.rs:119` |
| `crates/engine-cli/src/describe.rs:120` |
| `crates/engine-cli/src/describe.rs:121` |
| `crates/engine-cli/src/describe.rs:122` |
| `crates/engine-cli/src/describe.rs:123` |
| `crates/engine-cli/src/describe.rs:125` |
| `crates/engine-cli/src/describe.rs:126` |
| `crates/engine-cli/src/describe.rs:127` |
| `crates/engine-cli/src/describe.rs:128` |
| `crates/engine-cli/src/describe.rs:131` |
| `crates/engine-cli/src/describe.rs:132` |
| `crates/engine-cli/src/describe.rs:133` |
| `crates/engine-cli/src/describe.rs:136` |
| `crates/engine-cli/src/describe.rs:137` |
| `crates/engine-cli/src/describe.rs:138` |
| `crates/engine-cli/src/describe.rs:141` |
| `crates/engine-cli/src/describe.rs:142` |
| `crates/engine-cli/src/describe.rs:143` |
| `crates/engine-cli/src/describe.rs:146` |
| `crates/engine-cli/src/describe.rs:147` |
| `crates/engine-cli/src/describe.rs:148` |
| `crates/engine-cli/src/describe.rs:149` |
| `crates/engine-cli/src/describe.rs:150` |
| `crates/engine-cli/src/describe.rs:151` |
| `crates/engine-cli/src/describe.rs:152` |
| `crates/engine-cli/src/describe.rs:153` |
| `crates/engine-cli/src/describe.rs:154` |
| `crates/engine-cli/src/describe.rs:155` |
| `crates/engine-cli/src/describe.rs:156` |
| `crates/engine-cli/src/describe.rs:157` |
| `crates/engine-cli/src/describe.rs:158` |
| `crates/engine-cli/src/describe.rs:159` |
| `crates/engine-cli/src/describe.rs:160` |
| `crates/engine-cli/src/describe.rs:161` |
| `crates/engine-cli/src/describe.rs:182` |
| `crates/engine-cli/src/lib.rs:821` |
| `crates/engine-cli/src/lib.rs:906` |
| `crates/engine-cli/src/lib.rs:935` |
| `crates/engine-cli/src/lib.rs:951` |

## Spanish heuristic

Rechecked every in-scope source line for Spanish diacritics or punctuation,
or at least two Spanish function words, following the audit method. The
explicit word set was: que, para, del, una, los, las, por, sin, debe, según,
el, la, de, en, con, un, se, al, como, no, es, son, su, sus, entre, cada,
todos, todas, también, más, antes, después, hasta, sobre, solo, desde, durante.
Matching was case-insensitive and used complete words.

The scan reports 73 raw source-line hits. Manual review classifies 52 as
Spanish literal lines and 21 as false positives from locale identifiers,
Rust lifetime syntax, and two already-English WASM comments enumerating
language codes (`crates/engine-wasm/src/lib.rs:88` and
`crates/engine-wasm/src/lib.rs:115`). No Spanish comment prose remains.
Literal-by-literal review adds 13 short or formula-heavy lines missed by the
heuristic, producing the complete 65-line remaining inventory above.

## Verification

Verification uses the existing shared Cargo target directory and build lock,
with Cargo invoked directly. Logs and scratch data are outside the checkout;
no system temporary directory or network access was used. The checks ran before the commit was created.

| Check | Result |
| --- | --- |
| Comment boundary verification | PASS: all 25 Rust sources have identical non-comment bytes, literal sequences, inline code/doc links, fenced code, and physical line counts against the base. |
| `cargo fmt --all --check` | PASS. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | PASS. |
| `cargo test --workspace --offline` | PASS: 845 Rust tests (800 unit/integration and 45 doctests), 129 groups including 22 doc-test groups; zero failures or ignored tests. Two embedded Python suites also passed, 7 tests each (14 total). |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --offline` | PASS. |
| `bash clean-room/check.sh origin/main` | PASS, including both new documents registered with `git add -N` in a separate index. |
| `bash scripts/changelog-check.sh origin/main` | PASS; the script inspects committed HEAD changes and reports no code changes. Supplemental checks validate the uncommitted fragment. |
| `python3 scripts/generate-schemas.py --check` | PASS: schema bytes, index, and inventory are current. |
| `git diff --check origin/main` | PASS, including both new documents through the separate index. |
| Scope and identifier checks | PASS: exactly 22 Rust source files plus the two required documents; no added restricted identifiers, including in provenance; no other tracked file changes. |

The managed worktree Git index and object database are read-only. Intent-to-add
registration used a separate writable index and object directory, with the
existing object database as a read-only alternate. The real index is unchanged.
The supplemental changelog check validates the new fragment name, nonempty
Spanish text, required ticket prefix, absence of headings, and unchanged
CHANGELOG.md directly in the working tree.
