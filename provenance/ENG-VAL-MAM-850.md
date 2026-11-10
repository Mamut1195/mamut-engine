# Clean-room log · ENG-VAL-MAM-850

- Ticket / PR: MAM-850 · VAL-037 and VAL-038
- Author(s): agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-09 (America/Guayaquil, ECT, UTC−5)
- Session: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO. No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni salidas de esos programas.
- Role: specifier. No production code. The acceptance band was committed before the engine run.
- Contract implemented: none. Validation pages and an API test only.
- Permitted sources used: docs/CLEAN_ROOM.md; docs/METHODS_GUIDE.md; docs/tickets/README.md; provenance/TEMPLATE.md; provenance/ENG-ANL-003.md; provenance/ENG-ANL-006.md; provenance/ENG-ELM-002.md; the in-repo frame geometric stiffness and the pdelta and buckling.linear facades. No book and no PDF entered the worktree. A steel-design appendix with a similar story amplifier was not in the session: cita desconocida. Timoshenko and Gere, page, section, table, and equation: cita desconocida.
- Own derivations: docs/validation-report/casos/deriv/VAL-037.tex and the Euler summary in docs/validation-report/casos/deriv/VAL-038.tex, which points at ENG-ANL-006.
- Third-party code copied: none
- Validation: analytical. VAL-037 compares story B2 from pdelta and linearStatic displacements with 1/(1−ΣP/Pe), Pe = H·h/Δ_linear. VAL-038 records the four Euler eigenvalues already covered by the existing tests.
- Comparison with oracle (sibling repo, informative): no
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni salidas de esos programas, para este ticket."
- Clean-room reviewer: PENDING

## Preregistration

The band `|B2_engine − B2_story| ≤ 0.35 (B2_story − 1) + 0.005`, the regime `B2_story − 1 > 0.02` and `0 < ΣP/Pe < 0.25`, and the three-story model were committed before any engine run of this case.

That commit is `396759950c446499102756793b8fa45409ce7d8f`, author date 2026-11-10 04:06:35 +0000 (2026-11-09 23:06:35 ECT). No author or committer date was set by hand.

## What B2 approximates

B2 here is the story drift ratio under constant gravity axial force when the geometric effect kept in the closed form is only the chord shear `(ΣP) Δ / h`. The engine geometric stiffness is the Hermite integral already in ENG-ELM-002, whose prevented-rotation projection is `(6/5) ΣP / h`. The closed form is therefore an approximation of the engine operator. The coefficient 0.35 covers that bowing gap, of order `0.2 (B2 − 1)`, while the ratio stays below 0.25. The added 0.005 covers roundoff and the small joint rotations of the chosen beams (`G ≤ 0.01`). A run with no geometric stiffness would land on B2 = 1, outside the band once `B2 − 1 > 0.02`.

## VAL-037 run

`cargo test --offline --locked -p engine-api --test val037_val038`, test `three_story_b2_stays_inside_the_preregistered_band`, passed. Convergence status in the pdelta document: ok. Picard controls were the ones written beforehand: relative tolerance 1e-10, 40 iterations, displacement scale 1e-3 m, rotation scale 1e-3 rad.

| Story | ΣP/Pe | B2 story | B2 engine | Band |
| --- | ---: | ---: | ---: | ---: |
| 1 | 0.08823775990105970 | 1.0967771596808886 | 1.1128306264450898 | 0.03887200588831102 |
| 2 | 0.07319529298956912 | 1.0789759616410164 | 1.0878838165440201 | 0.03264158657435574 |
| 3 | 0.05066387564480843 | 1.0533676896359763 | 1.0674337800597360 | 0.02367869137259169 |

All three stayed inside the band. The reference column of the report is this closed form, not a published table.

## VAL-038 run

Same test binary, test `four_euler_cases_are_the_val038_series`, passed. Eight elements, unit compression, relative tolerance 1e-3 already recorded in ENG-ANL-006. It was not retuned.

| C | Pcr (N) | Engine λ (N) |
| ---: | ---: | ---: |
| 0.25 | 92.52754126021273 | 92.52773188644720 |
| 1 | 370.1101650408509 | 370.1222921022026 |
| 2.0457485159382953 | 757.1523208659985 | 757.2550398772300 |
| 4 | 1480.4406601634037 | 1481.1988538461794 |

`buckling.linear` is Experimental and the job sets `allow_experimental`. `pdelta` (facade `pDelta`) is Stable in the method registry. This ticket does not reclassify it. The case text says so. Production code was not changed.

## Commands

- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`: exit 0
- `cargo test --offline --locked --workspace`: exit 0
- `RUSTDOCFLAGS=-D warnings cargo doc --offline --locked --no-deps --workspace`: exit 0
- `cargo check --offline --locked --workspace --target wasm32-unknown-unknown`: exit 0
- `cargo build --offline --locked --release --target wasm32-unknown-unknown -p engine-wasm`: exit 0
- LaTeX: `lualatex` and `latexmk` are not installed in this environment. The report was not compiled here. No packages were fetched to obtain them.
- `docs/contracts/error-codes-1.json` was not edited. It still has 32 entries.
