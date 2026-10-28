# Clean-room log · ENG-LA-008 (part: cumulative memory budget)

## Additional declaration from the ENG-LA-009 session

- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil), requested identification; session executed by Codex (GPT-6).
- Role: implementer of the in-house modal follow-up; no changes to MemoryBudget (#38/#50).
- Sources used in this session: only the repo's own code and documentation and analytical derivations; no network access or external sources.
- Third-party code copied: none. See scope, derivations and validation in `ENG-LA-009.md`.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'
- Clean-room reviewer: PENDING (J4)

## Original record

- Ticket / PR: ENG-LA-008 (cumulative memory budget in engine-linalg); branch `j2/la-008-memory-budget`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Implemented contract: engine-linalg (new API `MemoryBudget`, `dense_bytes`, `sparse_bytes`; no changes to LinAlgBackend)
- Permitted sources used: no external sources; elementary byte accounting.
- Own derivations: explicit and deterministic counter (no OS, threads or global state) with limit, usage and peak; `charge` rejects with ENG-E-0013 without changing the account; `zeros` and `solve_dense` charge the working set (A, B, X) and only the solution remains charged. `alloc_vec` continues to limit each individual allocation. Scope: per-caller accounting; the backend has no internal budget (wiring in factorize/eigen and sparse solver: PENDING, remaining ENG-LA-008).
- Third-party code copied: none
- Validation: analytical (sum of permitted allocations exceeds the budget; overflow/release boundaries; exact working set ±1 byte)
- Oracle comparison: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)

## Second round (J4 r3, 2026-10-11)

- `ReferenceBackend::solve_dense` with A 0×0 returns B without allocating vectors per column (previously: panic «capacity overflow» with B 0×usize::MAX).
- `MemoryBudget::solve_dense` uses an RAII guard: a recovered backend panic releases the charge; `used` does not remain charged. `peak` documented (preserves the maximum charged working set). `release` without identity: double release documented as caller undercounting.
- Tests: `empty_system_with_huge_column_count_does_not_panic`, `recovered_backend_panic_does_not_leave_budget_charged`.
- Clean-room reviewer: PENDING (J4)
