# Clean-room log - ENG-VAL-004

- Ticket: MAM-107 / ENG-VAL-004.
- Author: Codex, model gpt-6.1-terra, 2026-11-03 (America/Guayaquil).
- Role: implementer.
- Contract: existing CLI `run` and `engine-input/1` and `results/1` interfaces.
- Permitted sources: user-supplied checkout excerpts of CONTRIBUTING.md,
  docs/CLEAN_ROOM.md, docs/METHODS_GUIDE.md, provenance/TEMPLATE.md,
  engine-bench fixture index and README, one Engine input, engine-cli and
  engine-wasm READMEs, parity and determinism tests and scripts, and the
  existing Engine campaign orchestrator.
- Own derivations: none; this is a measurement harness.
- Third-party code copied: none.
- Coordinator-reported execution and negative checks are documented in
  docs/ENG-VAL-004-local-report.md. The checkout was dirty; the observations
  are provisional and do not constitute approval or independent review.
- This documentation patch has been applied; independent review is pending.
- The existing focused test `cargo test --offline --locked -p engine-api --test
  determinism shared_native_wasm_suite_hashes_are_repeatable` returned rc0 (1 passed).
- Comparison with oracle: no.
- Declaration: No consulte codigo fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket.
- Clean-room reviewer: pending.
