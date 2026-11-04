# Method sheet · concrete.kent_park · issue 4

Complete before implementation. The expected values below were evaluated from
the cited equations before the law was written.

- Author, model if AI, date and role: implementation sub-agent, Grok 4.7,
  2026-11-04 (America/Guayaquil). Role: specifier and implementer.
- Own kind, contract and method version: `concrete.kent_park`,
  `uniaxial/1` (`ContractId::new("uniaxial", 1, 0)`), method `0.1.0`.
- Module/crate and intended consumer: `crates/engine-laws/src/uniaxial/kent_park.rs`,
  crate `engine-laws`. Consumer: a caller that already holds a `UniaxialLaw`.
  Publishing the kind in `capabilities().laws` does not add it to the
  engine-input kind enum.
- Existing methods reviewed and why this one is distinct: `elastic` and
  `elastic.cubic` are history-free and elastic in tension and compression;
  `multilinear_elastic` is a piecewise elastic polyline; `steel.bilinear_kinematic`
  is kinematic plasticity with a length-2 history. This law is a monotonic
  concrete compression envelope. It is not a parameterisation of those laws,
  and the steel law is left unchanged.

## Permitted sources

Sharma, Akanshu; Reddy, G. R.; Vaze, K. K.; Ghosh, A. K.; Kushwaha, H. S.;
Eligehausen, Rolf. *Joint model to simulate inelastic shear behavior of poorly
detailed exterior and interior beam-column connections reinforced with deformed
bars under seismic excitations.* Government of India, Atomic Energy Commission,
Bhabha Atomic Research Centre, Mumbai, report BARC/2009/E/026, 2009.
Appendix A, section A.1.1.3 “Kent and Park Model”, printed pages 135–138,
equations (A.9), (A.10), (A.11), (A.12), (A.13) and (A.15). Equation (A.14)
is only the hoop-ratio definition the caller may use before calling the law.
The report was read in this session from the INIS copy, record c1s4b-7v057,
report number 41076014. The PDF is not stored in the repository.

The report’s reference 54, printed as consulted in that report, is:
Kent, D. C., and Park, R., “Flexural Mechanics with Confined Concrete”,
*Journal of the Structural Division*, ASCE, Vol. 97, ST7, July 1971,
pp. 1969–1990. That journal article was not opened. Equation numbers in this
sheet are the report’s (A.9)–(A.15), not 1971 equation numbers. The title is
copied as the report prints it.

No third-party listing, table or figure was copied. The modified envelope that
the same section starts at (A.16) is not used. No structural-program memory
curve is reconstructed: the history length is zero.

## Assumptions, units and domain

- Physics and limitations: monotonic compression envelope for concrete
  confined by rectangular hoops, as printed in (A.9)–(A.15). The ascending
  branch is the second-degree parabola (A.9) up to the peak strain 0.002.
  The report states that confinement does not change that ascending branch or
  the strain at peak stress. The falling branch is the straight line (A.10)
  of slope parameter Z from (A.11)–(A.13). Past the strain where (A.10) meets
  0.2 f'_c, (A.15) holds the stress at 0.2 f'_c. There is no cyclic unloading,
  no residual plastic strain and no damage variable. Tension is outside
  (A.9)–(A.15). The assumption used here, and only here, is σ = 0 and
  K = 0 for tensile strain. This is a 1D stress–strain idealization, not a
  member or section model.
- Total inputs, SI units and signs: one dimensionless total strain. The
  existing uniaxial laws take tension as positive (`Elastic`: “positive in
  tension”). The report’s f_c and ε_c are compression-positive. The mapping
  is ε = −ε_c, σ = −f_c and K = df_c/dε_c. K keeps its sign because both
  stress and strain change sign.
- Parameters, units, ranges and validation:
  - `fc`: cylinder strength f'_c in Pa, finite and strictly positive.
    (A.12) is printed with f'_c in psi. The only conversion is
    1 psi = 4.4482216152605 / 0.0254² Pa (1 lbf = 4.4482216152605 N,
    1 inch = 0.0254 m). It is a unit factor, not a fitted material constant.
    f'_c in psi must be strictly greater than 1000, because the denominator
    of (A.12) is f'_c − 1000. Equality and every smaller positive value are
    `ENG-E-0010`.
  - `rho_s`: volumetric hoop ratio ρ_s, dimensionless, finite and ≥ 0.
    The law does not evaluate (A.14).
  - `b_double_prime`: confined-core width b'' in m, finite and strictly
    positive.
  - `s_h`: hoop spacing s_h in m, finite and strictly positive.
  - Nonfinite parameters are `ENG-E-0008`. A nonpositive `fc`,
    `b_double_prime` or `s_h`, a negative `rho_s`, or f'_c ≤ 1000 psi is
    `ENG-E-0010`.
- State: length 0. Initial and trial histories are empty. There is no
  admissible nonzero state. A history of any other length is `ENG-E-0010`.
- Outputs and units: stress and tangent in Pa, both finite, and an empty
  trial history. `respond` does not mutate the law or the committed history.
- Numerical domain, operation order, overflow/underflow:
  1. Convert f'_c to psi and reject the (A.12) domain above.
  2. From (A.12), ε_50u − 0.002 = 5 / (f'_c,psi − 1000). The direct
     quotient is not subtracted from 0.002, because both sides round to
     0.002 when f'_c is huge.
  3. If ρ_s = 0, (A.13) is identically 0 and b''/s_h is not evaluated.
     If ρ_s > 0, ε_50h = (3/4) ρ_s √(b''/s_h), with the square root from
     `engine_core::math::sqrt`.
  4. Z = 0.5 (f'_c,psi − 1000) / (5 + ε_50h (f'_c,psi − 1000)). This is
     (A.11) after the substitution in step 2. The scale in the denominator
     must be finite and strictly positive.
  5. ε_20c is not numbered in the report. Continuity of (A.10) and (A.15)
     gives 0.2 = 1 − Z (ε_20c − 0.002), so ε_20c = 0.002 + 0.8/Z. It must
     be finite.
  6. The origin tangent f'_c · (2 / 0.002) must be finite.
  Any nonfinite intermediate is `ENG-E-0008`. For f'_c > 1000 psi the
  identity ε_50u − 0.002 = 5 / (f'_c,psi − 1000) shows that the unconfined
  denominator of Z is positive. A huge ρ_s can still overflow ε_50h or
  ε_20c; that stays an error.
- Nondifferentiable points and convention at each boundary. Strains below
  are the report’s compression-positive ε_c.
  - ε_c = 0 (engine strain 0): (A.9) gives f_c = 0 and
    df_c/dε_c = f'_c · (2 / 0.002). The tensile side of this implementation
    has derivative 0. The value returned at exactly 0 is the (A.9) value, so
    `initial_tangent()` matches `respond(0, initial_state).tangent`. This
    point is not a central-difference sample.
  - ε_c = 0.002: (A.9) and (A.10) both give f_c = f'_c. Their slopes differ.
    The convention is (A.9) for ε_c ≤ 0.002, so the tangent is exactly 0.
    The factored derivative f'_c · (2 / 0.002) · (1 − ε_c/0.002) is used so
    that the factor (1 − 1) makes the product 0. The expanded form
    f'_c · (2/0.002 − 2 ε_c/0.002²) is algebraically the same and is not
    used, because at the peak it is not a clean zero in f64.
  - ε_c = ε_20c: (A.10) and (A.15) both give 0.2 f'_c. The convention is
    (A.10) for 0.002 < ε_c ≤ ε_20c, with tangent −Z f'_c, and (A.15) for
    ε_c > ε_20c, with tangent 0.
  Central differences stay inside one smooth piece. They do not cross
  0, 0.002 or ε_20c.

## Original derivation

1. Starting equations. (A.9) for ε_c ≤ 0.002:
   f_c = f'_c [2 (ε_c/0.002) − (ε_c/0.002)²].
   (A.10) for 0.002 ≤ ε_c ≤ ε_20c:
   f_c = f'_c [1 − Z (ε_c − 0.002)].
   (A.11) Z = 0.5 / (ε_50u + ε_50h − 0.002).
   (A.12) ε_50u = (3 + 0.002 f'_c) / (f'_c − 1000), with f'_c in psi.
   (A.13) ε_50h = (3/4) ρ_s √(b''/s_h).
   (A.15) for ε_c ≥ ε_20c: f_c = 0.2 f'_c.
   The branch closures in the previous section resolve the shared endpoints.
2. Let x = ε_c / 0.002. On the parabola, f_c / f'_c = x (2 − x).
   On the falling line, f_c / f'_c = 1 − Z (ε_c − 0.002).
   On the plateau, f_c / f'_c = 0.2. Engine stress is the negation.
   No other program’s curve is used.
3. At fixed empty history the derivative with respect to ε_c is:
   parabola, f'_c · (2 / 0.002) · (1 − x);
   falling line, −Z f'_c;
   plateau, 0;
   tension (the assumption above), 0.
   Engine tangent equals that derivative.
4. Initial tangent is the parabola derivative at x = 0,
   f'_c · (2 / 0.002). The same value is stored at construction and returned
   by `initial_tangent()` and by `respond(0, [])`. Accepted and discarded
   trials are empty vectors. Repeating a strain after committing the empty
   trial returns the same stress.
5. ρ_s = 0 recovers the unconfined falling branch, ε_50h = 0.
   At x = 1 the parabola stress is f'_c and the tangent is 0.
   C = 0 of the cubic law is not a special case of this envelope.

## Cases established before execution

Hand evaluation, unconfined, before any execution of the law.
f'_c = 3000 psi, ρ_s = 0, b'' = 0.2 m, s_h = 0.1 m.
(A.13) gives ε_50h = 0.
(A.12): ε_50u = (3 + 0.002·3000) / (3000 − 1000) = 9/2000 = 0.0045.
(A.11): Z = 0.5 / (0.0045 − 0.002) = 200.
Continuity: ε_20c = 0.002 + 0.8/200 = 0.006.
SI strength: f'_c,Pa = 3000 · (4.4482216152605 / 0.0254²).

| Case / source / citation | SI parameters | Input and history | Analytical expected values | Tolerance and justification |
| --- | --- | --- | --- | --- |
| Parabola interior. (A.9), x = 1/2. | f'_c as above, ρ_s = 0, b'' = 0.2 m, s_h = 0.1 m | Engine strain −0.001 (ε_c = 0.001), history empty | f_c / f'_c = 2·(1/2) − (1/2)² = 3/4, so f_c = 2250 psi. Engine σ = −(3/4) f'_c,Pa. K = 500 f'_c,Pa | Exact for this f64 product. Relative 1e-12 and absolute 1e-9 Pa still guard the comparison. |
| Falling branch. (A.10), Z = 200. | Same | Engine strain −0.004 (ε_c = 0.004), and 0.002 < 0.004 < 0.006, history empty | f_c / f'_c = 1 − 200·(0.004 − 0.002) = 3/5, so f_c = 1800 psi. Engine σ = −(3/5) f'_c,Pa. K = −200 f'_c,Pa | (A.11) in f64 makes Z = 200 + one ulp. Absolute discrepancy is about 4e-9 Pa in stress and 5e-7 Pa in tangent. Relative error is about 3e-16. Limit = max(1e-9, 1e-12 · \|expected\|). |

The two rows above are the only stress comparisons. Separate checks, not extra
benchmark stresses: peak of (A.9) at ε_c = 0.002 gives engine σ = −f'_c,Pa and
K = 0; origin tangent equals respond(0); tension returns 0, 0; a nonpositive
or nonfinite parameter, f'_c = 1000 psi, a nonfinite strain and a history of
the wrong length return the catalog codes.

Finite-difference samples stay off the corners: engine strains −0.001 and
−0.004, h = 1e-6. That h is inside the kit limits 1e-9 and 1e-2.
On the parabola, d²f/dε_c² = −f'_c · 2 / 0.002² = −5·10⁵ f'_c.
The central-difference truncation is about \|f''\| h² / 6.
Divided by the local tangent 500 f'_c this is 1000 · h² / 6 ≈ 1.7·10⁻¹⁰,
below the kit limit 1e-6. The falling line has zero second derivative, so
the same h is exact aside from rounding.

## Integration and evidence

- Exports, descriptor and factory: `KentPark`, `KentParkParams`,
  `KentParkFactory`, `KENT_PARK`, re-exported from `uniaxial`.
- Conformance appropriate to the trait: `check_uniaxial_law` on the two
  smooth samples above, with h = 1e-6.
- Native benchmark and, if applicable, ENG-CORE-009 fixture/1: in-crate
  analytical rows only. No fixture/1 document, because this law is not an
  engine-input kind.
- Checks, tests passed/failed and logs under `$CARGO_TARGET_DIR`:
  `cargo test --offline --locked -p engine-laws --test kent_park`: 5 passed.
  `cargo test --offline --locked -p engine-laws --doc -- kent_park`: 1 passed.
  `cargo test --offline --locked -p engine-api --test methods capabilities_are_deterministic`: 1 passed.
  `cargo clippy --offline --locked -p engine-laws --all-targets -- -D warnings`: exit 0.
  `cargo clippy --offline --locked -p engine-api --lib --tests -- -D warnings`: exit 0.
  `cargo doc --offline --locked -p engine-laws --no-deps` with warnings denied: exit 0.
  The workspace suite, wasm and the 16 determinism hashes were not run.
- Comparison of the 16 hashes across targets without modifying the reference:
  `docs/DETERMINISM.md` is not modified. The hashes were not recomputed.
- Log, clean-room review and validation limits: `provenance/ENG-MAT-004.md`.
  Clean-room reviewer: PENDING. This change does not close MAM-107 and does
  not approve a time or memory budget. The 1971 article was not opened.
  No program listing was transcribed.
