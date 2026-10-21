# ENG-LANG-003 — Core crate documentation in English

- Ticket: MAM-837.
- Author: Codex (GPT-6), documentation implementer.
- Date: 2026-10-21 (America/Guayaquil).
- Baseline: `origin/main`, `7375fa25a34a3cb004bcaec84f645743b864ff89`.
- Sources: existing documentation and comments in the five crates below, the
  local task instructions, `changelog.d/README.md`, and the local verification
  rules. No external implementation, numerical reference, or network source was
  consulted. No third-party code was copied; no mathematical derivation was added.

## Scope and coverage

Reviewed every Rust file under `src/`, `tests/`, `benches/`, and `examples/` in
`engine-core`, `engine-model`, `engine-linalg`, `engine-contracts`, and
`engine-assembly`, together with their existing README files. These directories
contain 44 Rust files and two README files; all 46 have translated text. There
are no additional Rust files in these five crates, and no existing `benches/`
or `examples/` Rust files or further README files to translate.

| Crate | Rust files | README files | Translated comment lines | Translated README lines | Total translated lines |
| --- | ---: | ---: | ---: | ---: | ---: |
| engine-core | 4 | 0 | 197 | 0 | 197 |
| engine-model | 2 | 0 | 112 | 0 | 112 |
| engine-linalg | 17 | 1 | 520 | 136 | 656 |
| engine-contracts | 6 | 0 | 457 | 0 | 457 |
| engine-assembly | 15 | 1 | 360 | 78 | 438 |
| Total | 44 | 2 | 1,646 | 214 | 1,860 |

Counts compare corresponding physical lines with the baseline; a changed line
is counted once. Line counts and line order are unchanged in these 46 files.
Blank lines, existing English text, symbolic expressions, and unchanged code
inside documentation examples are excluded from the translation counts.

Only comment text and README prose were translated. Identifiers, string
literals (including Spanish diagnostics), attributes, executable statements,
code formatting, and the code in documentation examples remain unchanged.
Diagnostic strings belong to MAM-841. Existing ticket references, equations,
citations, proper names, paths, and API references were preserved.

## Documentation intentionally left untranslated

The MAM-837 instructions explicitly exclude comments documenting axial-bar or
elastic-truss behavior. The following locations retain their baseline text
verbatim; line numbers are unchanged:

| Location | Preserved documentation and reason |
| --- | --- |
| `crates/engine-contracts/src/examples.rs:3–4` | Shared derivation paragraph containing the axial-bar stiffness and force expression; preserved as one paragraph. |
| `crates/engine-contracts/src/examples.rs:120–126` | `LinearBar2D` assumptions, DOFs, stiffness, mass, supported loads, and limitations. |
| `crates/engine-contracts/src/examples.rs:133–313` | All comments in the constructor and implementations of `LinearBar2D`, including geometry, trial forces, tangent, mass, geometric stiffness, loads, and recovery. |
| `crates/engine-contracts/src/examples.rs:317–345` | `BarFactory` documentation and implementation comments for construction and validation of the axial-bar example. |
| `crates/engine-contracts/src/tests.rs:72,96,117` | Shared derivation note and axial-bar stiffness/mass checks. |
| `crates/engine-contracts/src/tests.rs:351,354,358` | Deliberately invalid bar responses used to check rigid-body behavior, derivatives, and covariance. |
| `crates/engine-contracts/src/tests.rs:634` | Independent spatial-bar formulation used for the six rigid-body modes. |
| `crates/engine-assembly/tests/assembly.rs:32` | Axial Ux subset and explicit spring/bar boundary. |
| `crates/engine-assembly/tests/fixtures/mod.rs:34–36` | Axial-bar energy, stiffness, and displacement derivation. |
| `crates/engine-assembly/README.md:75–85` | Truss geometric producer, fixed geometry, P-Delta scope, linear force/tangent behavior, and trial/accepted-state semantics; preserved as a complete paragraph under the same exclusion. |

The `engine-elements-truss` crate was not edited. Generic element contracts,
coordinate conventions, assembly interfaces, and scalar constitutive-law
documentation were translated where they do not document the excluded element
implementations.

Additional text remains verbatim to preserve existing code references and
equations, rather than as untranslated prose:

- `crates/engine-model/src/lib.rs:308,310`: the nominal reference forms
  `Law(id_sección)` and `Case(id_combinación)`.
- `crates/engine-linalg/tests/determinism_guard.rs:54,154`: the identifier
  examples `r#sin` and `math::función`.
- `crates/engine-linalg/tests/conformance.rs:481`: the existing `orden²` label.
- `crates/engine-linalg/src/reference.rs:185`,
  `crates/engine-assembly/src/assemble.rs:84`, and
  `crates/engine-assembly/tests/j4_followup.rs:165`: existing equations using
  `máx` or `máx_j`.
- `crates/engine-assembly/tests/diaphragm.rs:531`: the equation using
  `ΣM_sobre_j`.
- Existing equations using `tol_cero`, purely symbolic comments, SI unit names,
  and code identifiers and literals in all rustdoc/README examples.

## Review adjustments (2026-10-21)

- `crates/engine-core/src/catalog.rs:176`: corrected the stale comment reference
  `codes_are_unique_and_sorted` to the existing test
  `unique_ordered_codes_with_valid_format` (line 159). The obsolete reference
  was already present in `origin/main`; only the comment was corrected.
- `crates/engine-linalg/src/matrix.rs:176,178`: translated `orden²` to `order²`
  in rustdoc; diagnostic string literals remain unchanged.
- `crates/engine-assembly/src/lib.rs:4–5`: made the crate the explicit subject
  of the finite-element and local-transformation limitation, checked against
  the original Spanish text in `origin/main`.

Review validation: `cargo fmt --check` and
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --offline` passed
again. The comment-only verification also passed for all 44 Rust files,
including unchanged diagnostic strings and documentation example code.
Logs: `/tmp/lang003-review-{fmt,doc,verify}.log`.

## Verification

Verification uses `CARGO_TARGET_DIR=/workspace/.cargo-target/lang003`, Cargo
offline mode, and the local Rust toolchain. Temporary verification scripts and
logs live outside the repository under `/tmp`.

The comment-only check compares each modified Rust file against the baseline after removing
comments with a tokenizer that respects quoted strings, raw strings, character
literals, lifetimes, and nested block comments. It also compares executable code
inside documentation fences and checks newly introduced text against the local
identifier restrictions on the same baseline line.

| Check | Result |
| --- | --- |
| `python3 -u /tmp/lang003-resume-verify.py` (also runs `/tmp/lang003.py verify`) | PASS: 10 lexer fixtures; complete 46-file coverage; all 44 Rust files identical after removing comments; rustdoc and README code fences unchanged; 93 protected comment spans and 11 protected README lines unchanged; no new restricted identifiers in changed lines or new documents. |
| `cargo fmt --check` | PASS. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | PASS. |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --offline` | PASS. |
| `cargo test --workspace --offline` | PASS: 749 passed, zero failed or ignored, across 115 test groups, including 21 documentation-test groups. |
| `bash clean-room/check.sh origin/main` | PASS, including both new documents through a temporary index. |
| `git diff --check origin/main` | PASS. |

The clean-room check used a temporary Git index and temporary object directory
under `/tmp`, with the repository object directory as a read-only alternate.
The real index was left unchanged. Verification logs are
`/tmp/lang003-resume-{verify,fmt,clippy,doc,test,clean-room}.log`.
All requested checks completed without the earlier disk-space failure.
No commit was created.
