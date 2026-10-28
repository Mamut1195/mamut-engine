# ENG-BENCH-002 · Published P-Delta benchmarks (MAM-582)

Tests: `crates/engine-analysis/tests/bench_pdelta.rs`. Oracle: only published values and
closed-form solutions of the beam-column differential equation. Clean-room: no
code or outputs from OpenSees, Stabileo, CalculiX or LAPACK were opened or used.

Review as author: Codex (GPT-6), 2026-10-15 (America/Guayaquil), implementer role.
Sources for this review: commit HEAD, own code and derivations of this note; no network,
without consulting the original books or third-party code. Published references are
retained from the commit; pending attributions remain pending. Clean-room reviewer: PENDING.

## Sources

1. AISC, *ANSI/AISC 360-16 Specification for Structural Steel Buildings*, 7 jul. 2016,
   Commentary C2.1 "General Analysis Requirements", paragraph "Analysis Benchmark Problems",
   pp. 16.1-290 a 16.1-291; Fig. C-C2.2 (Case 1) y Fig. C-C2.3 (Case 2), p. 16.1-291.
   Tolerances (p. 16.1-291): moments within 3 % and deflections within 5 %.
   The bracketed values ("exclude shear deformations") are used, because `EulerFrame2D` has
   no shear deformation.
   - Geometric data read from the figures: W14x48 strong axis (A = 14.1 in², I = 484 in⁴),
     E = 29 000 ksi, L = 28 ft; case 1 w = 0.200 kip/ft; case 2 H = 1.0 kip. Checked with
     P = 0: wL²/8 = 235.2 kip·in, 5wL⁴/384EI = 0.197 in, HL = 336 kip·in, HL³/3EI = 0.901 in.
   - AISC 360-22 was not verified in an accessible source: PENDING confirmation that the tables do not change.
2. S. P. Timoshenko y J. M. Gere, *Theory of Elastic Stability*, 2.ª ed., McGraw-Hill, 1961,
   cap. 1 (simply supported beam-column with concentrated load; χ(u) = 3(tan u − u)/u³ and
   approximate amplification 1/(1 − P/Pe)). Section and page: PENDING verification in a copy;
   the formulas are re-derived from EI v'''' + P v'' = q and are the effective oracle.
3. W. F. Chen y E. M. Lui, *Structural Stability: Theory and Implementation*, Elsevier, 1987
   (cited by AISC C2): PENDING; there was no accessible source and no values were used.

## Model and discretization

- `EulerFrame2D` + `geometric_producer` (consistent Kg), `PDelta::run`, tolerance 1e-10,
  max 5000 steps. Forces in kip, lengths in in, moments in kip·in.
  Convergence scales: 1 in for translations and 0.01 rad for rotations.
  Declared mesh: 8 elements per member. P > 0 denotes compression and is applied as −P in Ux;
  lateral loads and tabulated deflections are positive in Uy.
- Case 1: supports Ux,Uy / Uy; w with consistent nodal loads (wΔx at interior nodes,
  wΔx/2 at each support and ±wΔx²/12 at end rotations; interior moments cancel).
  M_mid from half-bar statics in the deformed configuration: wL²/8 + P·Δ_mid.
- Case 2: clamped; M_base = −Rz_base from the engine, positive for H > 0.
- Closed-form solution: case 1 M = w/k²(sec u − 1), Δ = w/(EIk⁴)(sec u − 1) − wL²/(8P), u = kL/2;
  case 2 M = H tan(kL)/k, Δ = H(tan kL − kL)/(Pk); k = √(P/EI).
- Central point load Q (original derivation): on the left half-span, 0 ≤ x ≤ L/2,
  EI v'' + Pv = −Qx/2, with v(0) = 0 and v'(L/2) = 0. Thus, with u = kL/2,
  M_mid = Q tan(u)/(2k) and Δ_mid = [QL³/(48EI)]·3(tan u − u)/u³.
  The P = 0 limit is M_mid = QL/4, Δ_mid = QL³/(48EI).
  The approximation Δ/Δ₀ ≈ 1/(1 − P/Pe) is checked at 2 % only for the tested ratios;
  this check is not attributed to the book pending verification.

## Results (mesh 8)

Errors are calculated before rounding the displayed results.

| Case | P | Quantity | Engine | Reference | Error % | Tol | Status |
|---|---|---|---|---|---|---|---|
| AISC 1 | 0 | M_mid kip·in | 235.20 | 235 | 0.09 | 3 % | pass |
| AISC 1 | 150 | M_mid | 268.89 | 269 | 0.04 | 3 % | pass |
| AISC 1 | 300 | M_mid | 313.52 | 313 | 0.17 | 3 % | pass |
| AISC 1 | 450 | M_mid | 375.41 | 375 | 0.11 | 3 % | pass |
| AISC 1 | 0 | Δ_mid in | 0.1971 | 0.197 | 0.03 | 5 % | pass |
| AISC 1 | 150 | Δ_mid | 0.2246 | 0.224 | 0.27 | 5 % | pass |
| AISC 1 | 300 | Δ_mid | 0.2611 | 0.261 | 0.02 | 5 % | pass |
| AISC 1 | 450 | Δ_mid | 0.3116 | 0.311 | 0.19 | 5 % | pass |
| AISC 2 | 0 | M_base kip·in | 336.00 | 336 | 0.00 | 3 % | pass |
| AISC 2 | 100 | M_base | 469.07 | 469 | 0.01 | 3 % | pass |
| AISC 2 | 150 | M_base | 598.65 | 598 | 0.11 | 3 % | pass |
| AISC 2 | 200 | M_base | 848.98 | 848 | 0.12 | 3 % | pass |
| AISC 2 | 0 | Δ_tip in | 0.9009 | 0.901 | 0.02 | 5 % | pass |
| AISC 2 | 100 | Δ_tip | 1.3307 | 1.33 | 0.05 | 5 % | pass |
| AISC 2 | 150 | Δ_tip | 1.7510 | 1.75 | 0.06 | 5 % | pass |
| AISC 2 | 200 | Δ_tip | 2.5649 | 2.56 | 0.19 | 5 % | pass |
| T&G | P/Pe 0.2–0.7 | M, Δ vs closed form | — | — | ≤ 0.0053 | 0.1 % | pass |

Case 2 with P = 200 has P/Pe ≈ 0.65, with Pe = π²EI/(4L²) for the cantilever
(AISC requires including 0.6–0.7). In the simply supported cases, Pe = π²EI/L².
All engine values agree with the closed-form solution to ≤ 0.1 %;
the largest error against the retained AISC values is 0.27 %.

## Mesh sensitivity (error vs closed-form solution, maximum P)

| n | C1 M | C1 Δ | C2 M | C2 Δ |
|---|---|---|---|---|
| 2 | 0.172 % | 0.461 % | 0.037 % | 0.062 % |
| 4 | 0.011 % | 0.030 % | 0.002 % | 0.004 % |
| 8 | 0.0007 % | 0.0019 % | 0.0002 % | 0.0003 % |
| 16 | <0.0001 % | 0.0001 % | <0.0001 % | <0.0001 % |

The test requires all four errors to decrease with each refinement.
Tolerance 1e-3 against the closed-form solution is required on the declared mesh of 8 elements;
the study meshes maintain the published tolerances of 3 % in M and 5 % in Δ.

## Diagnostic: relative residual floor

With tolerance 1e-12, case 2 (P = 200, mesh 8) ends `NotConverged` after 5000 steps, and with
1e-10 the same happens with mesh 32: relative change reaches ~3e-15, but the relative residual
stagnates at ~4.8e-10 (rounding floor; the residual combines force components in kip and
moment components in kip·in without scaling them by DOF; translational stiffness grows as 12EI/Δx³).
Comparison with the closed-form solutions quantifies the accuracy of the tabulated meshes;
stagnation alone does not guarantee accuracy for a `NotConverged` run.
A tolerance requested by the user may be unattainable and the engine only reports it as
`NotConverged`. Follow-up is recommended before or
together with declaring P-Delta stable (e.g. documenting the floor or normalizing the residual by DOF).

## Verification of this review

Without network, with `CARGO_NET_OFFLINE=true` and `CARGO_TARGET_DIR=/tmp/me-bench-review-target`:

- `cargo fmt`: OK.
- `cargo clippy -p engine-analysis --all-targets -- -D warnings`: OK.
- `cargo test -p engine-analysis --test bench_pdelta -- --nocapture`: OK, 4 tests;
  results consistent with both tables. References and tolerances unchanged.

No commit or push.
