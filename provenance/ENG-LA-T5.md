# Clean-room log · T5 (issue #33) pivot invariant under diagonal scaling

- Ticket / PR: issue #33; branch `j2/t5-pivot-scaling`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Implemented contract: engine-linalg (LinAlgBackend; singularity policy)
- Permitted sources used: no external sources; symmetric diagonal equilibration is derived here: with B = S A S, s_i = 1/√(máx_j |a_ij|), we have A x = b ⇔ B y = S b, x = S y; and M = S⁻¹ B S⁻¹ ⇒ if B = L_B L_Bᵀ, L = S⁻¹ L_B. (Ruiz/Jacobi equilibration: PENDING precise citation; not consulted.)
- Own derivations: applied in LDLᵀ (factorize/solve) and mass Cholesky; `solve_dense` (nonsymmetric) is unchanged. `min_pivot_ratio` refers to the equilibrated matrix.
- Third-party code copied: none
- Validation: analytical (diag(1e6,1e-9), diag(1e8,1e-6), coupled D·A·D, true singularities still yield ENG-E-0006)
- Oracle comparison: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)

## Second round (J4 r3, 2026-10-11)

- Equilibration now iterative (Ruiz type, ≤40 passes, ±1 %). Exact invariance is no longer claimed (lib.rs): mitigates, does not guarantee.
- RHS: the scale of b is taken before equilibration and equilibration is applied afterwards (b·s no longer overflows): A=[[.5,.5],[.5,-.5]], b=[1.3e308,0] → [1.3e308,1.3e308].
- Undo scaling with `scale_back`, which tries several evaluation orders to avoid intermediate subnormals: A=[[1e-200,1e-200],[1e-200,1]], b=[0,7.41e-224] no longer gives 33 % error.
- New tests: J4's three cases, coupled 3×3 up to D=1e8 and 2×2 up to 1e14, eigenvalues with K=diag(1e6,1e3), M=diag(1e8,1e-6).
- Limit: no guarantee of exact invariance in coupled matrices with extreme ranges.
