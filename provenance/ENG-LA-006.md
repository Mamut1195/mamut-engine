# Clean-room log · ENG-LA-006

- Ticket / PR: ENG-LA-006, «Selection of linear algebra libraries (ADR-002)» · branch `j2/eng-la-006-adr-002`; no commit, push or PR in this session.
- Author(s): Codex gpt-6.1-sol high · 2026-10-10 (America/Guayaquil), under the user's instructions. No delegation to other agents.
- Role: documentation implementer; no oracle operator or similarity auditor activity.
- Implemented contract: none in code. [ADR-002](../docs/adr/ADR-002-algebra-lineal.md) proposes the selection for `LinAlgBackend` (ENG-LA-007), `linsolve/1` and `eigen/1` (architecture §§4.5–4.6); integration and tests remain in ENG-LA-007/008/009.
- Permitted sources used: crates.io metadata verified on 2026-10-10 and provided by the requester: [faer 0.24.4](https://crates.io/crates/faer/0.24.4), [nalgebra 0.35.0](https://crates.io/crates/nalgebra/0.35.0), [nalgebra-sparse 0.12.0](https://crates.io/crates/nalgebra-sparse/0.12.0), [sprs 0.11.5](https://crates.io/crates/sprs/0.11.5) and [scirs2-sparse 0.6.5](https://crates.io/crates/scirs2-sparse/0.6.5); internal architecture `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`, §§4.5–4.6 (contracts), 9 (determinism) and 10 (compilation test of 2026-10-08, candidate functions and licenses, exclusion of sprs-ldl and arpack-ng). No chapters, equations or measurements are attributed to sources not consulted. Reconsulting crates.io pages and APIs was not possible in this session; it is not presented as independent remote verification.
- Own derivations: no mathematical derivation or new algorithm. The [ADR decision and verification plan](../docs/adr/ADR-002-algebra-lineal.md#decisión) contain the documentary comparison, the recommendation to keep MSRV 1.85 with optional nalgebra requiring at least 1.89, and the proposed trials to resolve pending items.
- Third-party code copied: none. No Rust code or dependencies were added; no code listings or legal text were copied.
- Validation: documentary review against tickets, contracts and architecture; checking the diff, local links and file scope. No numerical validation, benchmark, native/wasm execution or `cargo-deny` was performed for candidates in this session. The cited wasm compilation is historical evidence from architecture §10, not a new trial. [Plan V1–V7](../docs/adr/ADR-002-algebra-lineal.md#plan-de-verificación) **PENDING** for licenses, MSRV, execution, APIs, inertia, sparse eigenvalues, performance, size, determinism and maintenance.
- Oracle comparison (sibling repo, informational): no. OpenSees, CalculiX and other engines were not run as oracles.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING human review and Jonathan's approval; this session does not approve the ADR or the merge.

## Internal documents consulted

- `/workspace/motor-rust/04_TICKETS.md`: ENG-LA-006, ENG-LA-007, ENG-LA-008 and ENG-LA-009.
- `/workspace/motor-rust/00_PROYECTO.md`: context, selection through libraries and pending technical items.
- `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`: §§4.5–4.6, 9 and 10.
- [Clean-room protocol](../docs/CLEAN_ROOM.md), [ADR-001](../docs/adr/ADR-001-fundacionales.md), [provenance template](TEMPLATE.md) and [ENG-CORE-007 log](ENG-CORE-007.md).
- [README](../README.md) and [Cargo.toml](../Cargo.toml): existing ADR list and workspace MSRV 1.85.

No applicable `AGENTS.md` instructions were found in the working paths
and their ancestor directories. The documents in `/workspace/motor-rust` were only
read. No source code of OpenSees, CalculiX, Stabileo or other
engines was opened, nor were they used as oracles. No external code repos were accessed
to obtain algorithms or implementations.

## Scope and evidence

The ADR was created in Spanish with status «Propuesto, pendiente de aprobación de
Jonathan», together with this log and a link in the existing README list. No
manifests, lockfile, Rust code, `clean-room/**`, `.github`,
`CODEOWNERS` or `docs/CLEAN_ROOM.md` were modified; no workflows, `deny.toml`,
`LICENSE` or legal text were created. No commit was made.

Web queries of the crates.io pages and its API returned access
errors. A direct metadata query through the API failed due to DNS
resolution. The verified snapshot supplied by the user was retained, distinguishing it
from the compilation test recorded in architecture §10. No missing update
dates, undeclared MSRVs or performance or size figures were
invented. Review of LICENSE files and the graph per version remains
**PENDING**, although §10 records historical checks for three crates.

The faer recommendation rests on the coverage and compilation recorded by
the architecture. It does not assume correct execution in wasm, access to inertia,
availability of all APIs with minimal features or an approved sparse
eigenvalue solver. These limitations appear in the acceptance
criteria and in the ADR test plan.

## Local verification

- `git diff --check` and whitespace check of the new documents: successful.
- Local Markdown links in the three files of this delivery: targets exist.
- Review of changed files: only ADR-002, this log and one line in README.

No Rust tests were run because this is a documentation-only change.
No numerical threshold or ticket approval is declared as met.
