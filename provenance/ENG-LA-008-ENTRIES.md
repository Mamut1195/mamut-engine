# Clean-room log · issue #22 follow-up (SparsityPattern::entries)

- Ticket / PR: issue #22; PR for this branch `j2/la-007-entries-fallible`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Implemented contract: engine-linalg (internal API; no change to external contracts)
- Permitted sources used: no external sources; elementary size arithmetic (orden² bytes) and the existing allocation limit policy (`MAX_ALLOC_BYTES`, PR #21/#24).
- Own derivations: `entries()` (infallible, O(orden²)) is replaced by `try_entries()` (fallible: ENG-E-0013 if orden² overflows or exceeds `MAX_ALLOC_BYTES`; ENG-E-0010 if allocation fails), `contains(i,j)` (O(log nnz)) and `mask_iter()` (lazy, O(1) memory). Pre-1.0 API change (0.1.0); the only uses were tests.
- Third-party code copied: none
- Validation: analytical (extreme size and mask/coordinate equivalence tests)
- Oracle comparison: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)
