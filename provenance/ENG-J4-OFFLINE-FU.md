# Clean-room log · J4 offline follow-up (#21, #23, #25, #11)

- Ticket / PR: /workspace/j4rev/comments/engine-cleanroom-offline.md; branch `j2/j4-offline-followup`
- Author(s): J2 (agent, Grok Bot; Codex not used) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Implemented contract: corrections in engine-assembly, engine-contracts (kit), engine-linalg, engine-core (tests)
- Permitted sources used: no external sources; own floating-point arithmetic.
- Own derivations:
  - assemble.rs:113/124 — symmetry is compared as |a/2−b/2| ≤ ½·tol·máx (without overflow), non-finite values are always rejected and the mean is a/2+b/2 (previously (a+b)/2 overflowed to ∞).
  - conformance.rs:505 — rotations: the rigid mode is required in INFINITESIMAL form (K₀·u≈0 with the tangent at the origin), not with a finite rotation of 1 rad (which could reject valid nonlinear elements); translations still use force. Covers the T2 low-severity finding concerning infinitesimal rotations.
  - robustness.rs — `1 << 40` does not compile on wasm32: constant per target (2^40 on 64 bits, 2^29 on 32 bits).
  - catalog.rs:168 — the ENG-E-0013 test no longer requires it to be the last code (order and uniqueness are tested separately).
  - reference.rs `rescale` — three evaluation orders to avoid intermediate overflow and underflow (v·num underflowed even when the result was normal).
- Residual limitation: normalizing the RHS by its maximum (x/rs) may make tiny components subnormal; this is a documented f64 limitation, not corrected here.
- Third-party code copied: none
- Validation: analytical (tests `symmetry_check_and_mean_do_not_overflow`, `rescale_tests`, spatial kit)
- Oracle comparison: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)

## Second round (J4 engine-r2, 2026-10-11)

- assemble.rs: the difference is computed directly (exact with subnormals: 5e-324 versus 0 is asymmetric with tol=0) and halves are used only if it overflows; `a == b` remains exact; the mean uses `(a+b)·0.5` and only resorts to halves if the sum overflows (does not lose 5e-324).
- catalog.rs:153 no longer requires exactly 13 codes (`>= 13`).
- conformance.rs: the rotation mode calls `response()` and `check_response` again (length/finiteness validation is preserved) in addition to K₀·u≈0.
- Merge of `main` (without force push); conflict in j4_followup.rs resolved by preserving both tests.
- `rescale` («normal» candidate with possible error if there is partial underflow): unchanged in this round; preexisting limitation.
