# Clean-room log · ENG-INT-001

- Ticket / PR: ENG-INT-001 (MAM-60) · especificación del adaptador para StructuralSolverPort · rama j2/MAM-60-spec-adaptador-structural-solver-port.
- Author(s): helper de J2, 2026-10-28 America/Guayaquil. Sin Codex (solo prosa y tablas).
- Role: redactor de especificación.
- Implemented contract: ninguno; documento de mapeo contra engine-input/1, results/1, capabilities/1, cli-run/1, cli-error/1 y docs/contracts/error-codes-1.json.
- Permitted sources used: schemas publicados del repo; docs/GUIA_CLI_AGENTES.md; crates/engine-api/src/methods.rs (estados de métodos); salida de `mamut-engine capabilities`; interfaz pública `packages/structural-core/src/solver-port/index.ts` de Mamut1195/mamut-structures (solo lectura, solo ese archivo).
- Not consulted: adaptador Stabileo de Structures, código de Stabileo, tipos de dominio de Structures (sus campos quedan DESCONOCIDO).
- Third-party code copied: none.
- Numerical values from OpenSees, Frame3DD or Stabileo: none.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisión r1 (J4): notas N1–N5 incorporadas en un commit nuevo (salida de `--version`/`version/1`, `--timeout 0`, fuente del criterio Experimental, alcance de `validate`, DESCONOCIDOS); verificadas contra crates/engine-cli/src/describe.rs, crates/engine-cli/src/timeout.rs, crates/engine-api/src/lib.rs y methods.rs.
- Clean-room reviewer: PENDING (J4).
