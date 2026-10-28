# Clean-room log · ENG-VAL-032

- Ticket / PR: MAM-847 · VAL-032, buildings E1–E5, E2b and E5b · PR #180.
- Author(s): Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Role: specifier and implementer of fixtures and validation tests.
- Contract implemented: `val032-fixture/1` fixtures with twelve
  `runs[].engine_input`, run through `engine_api::run`; checking
  `results/1` documents, expected statuses, errors and warnings.
- Permitted sources used: ticket request; `provenance/TEMPLATE.md`;
  `provenance/ENG-VAL-005.md`; NEC-SE-DS 2015, cited by section and page
  in the [VAL-032 README](../docs/validation-report/fixtures/val032/README.md):
  §3.1.1 (pág. 27), §3.3.1 (págs. 33–34), §6.1.7 (pág. 55),
  §6.2.2 e (pág. 58), §6.3.2–6.3.3 (págs. 61–62), §6.3.4
  (pág. 65), §6.3.5 (pág. 66) y §6.3.6–6.3.8 (págs. 66–68).
  Books cited by page/equation, without copying text. References and
  caveats are retained in the fixtures and their README; no external
  sources were consulted or network used for this log.
- Own derivations: fixture generation through
  [`_ref/gen.py`](../docs/validation-report/fixtures/val032/_ref/gen.py) and
  own independent numpy model
  [`_ref/val032_calibrate.py`](../docs/validation-report/fixtures/val032/_ref/val032_calibrate.py).
  The r3 physical equivalence is documented in the README and in
  [`_ref/calibrate_r3.json`](../docs/validation-report/fixtures/val032/_ref/calibrate_r3.json),
  with relative differences ≤1e-10.
- Third-party code copied: none. No new dependencies or changes
  to Cargo.toml/Cargo.lock, workflows or clean-room rules.
- Validation: own numerical reference for r3 physical equivalence and
  integration test of the twelve actual runs with mandatory statuses and
  diagnostics; Engine blockers are explicitly documented.
- Comparison with oracle (sibling repo, informative): no. No OpenSees
  or Frame3DD outputs were opened or their results incorporated.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket. Tampoco consulté ni usé código o resultados de CalculiX o
  LAPACK, ni salidas de OpenSees o Stabileo."
- Clean-room reviewer: PENDING (J4).

## Decisions and scope

In `crates/` the only change is the test
[`crates/engine-api/tests/val032_runs.rs`](../crates/engine-api/tests/val032_runs.rs).
It traverses the seven fixtures in `docs/validation-report/fixtures/val032`, requires
the twelve unique runs and executes each input through `engine_api::run`.
It rejects `invalid`, unexpected errors and unreviewed warnings. Fixtures are
generated with `_ref/gen.py`; the Engine implementation is not modified.

The eight modal/spectrum runs (E1-modal, E2-cqc-X, E2-srss-X, E2-cqc-Y,
E2b-cqc-X, E2b-cqc-Y, E2b-cqc-XY-srss and E4-modal) are blocked by
`ENG-E-0006`, with status `unstable`, due to massless DOFs. The numpy
reference condenses those DOFs; the Engine does not yet do so. No artificial
masses or constraints are added to hide this gap.

E3-pdelta is blocked by `ENG-E-0007`, with status `notConverged`:
pDelta does not project Kg and the geometric forces/residual through the
diaphragm transformation. The Engine correction is outside VAL-032.
The test requires the exact statuses and codes of the nine blocked runs and fails
if they start converging, to require a review of fixtures, evidence and README.

E3-lineal, E5-combinaciones and E5b-combinaciones return `warning`.
The README individually justifies warnings #84
(`API-W-DIAPHRAGM-RECOVERY`) and #85 (`API-W-COMBINED-PRESCRIBED`).
The test checks their codes per run, the multiplicity of #84, the ten combination
ids for #85 and the zero prescribed displacements. E5 member forces
retain their diagnostic status due to #84.

## r3 physical equivalence and validation limits

The own calibrator uses geometry, properties, masses and loads from the JSON,
without consuming Engine results. It compares the previous definition with
a floating master at the center of mass and the new one with a connected master,
mass distribution and transport of equivalent forces and moments.
The periods T1–T15, linear story responses and required reactions
show relative differences ≤1e-10. E5b is exclusively static and has
no periods to compare. This equivalence does not enable modal/spectrum
analyses or validate blocked P-Δ responses.

Code and bibliographic locators are retained without transcribing
source text. I = 1.0 against Tabla 6 of NEC-SE-DS and the numbering
and combination factors of NEC-SE-CG §3.4.3 remain unverified, as
the README states. Comparisons with oracles remain pending.

## Verification of this log

The template, permitted example, README and local test were reviewed.
This task only creates the log; it does not rerun the suites or the
calibrator. The linked numerical evidence corresponds to the r3 completion
documented in the README. Work without git or network.

## Pending items

J4 clean-room review and follow-up of the Engine blockers indicated
in MAM-847; oracle comparisons and code cross-checks pending according to
the README.
