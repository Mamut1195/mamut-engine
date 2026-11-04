# Clean-room log · ENG-LOAD-TEMP-972

- Ticket / PR: MAM-972 «Carga de temperatura en barras». Local commit only.
  Not marked Done. No release. No pull request.
- Author(s): implementation sub-agent, Grok 4.7, 2026-11-03 (America/Guayaquil).
- Role: implementer. Own derivation and analytical fixture. No oracle operator.
- Contract implemented: existing element/1 kinds `truss2d.linear` and
  `truss3d.linear` (`LinearTruss`). No new element kind.
- Permitted sources used: this repository only. Sign convention and bar
  geometry from `crates/engine-elements-truss` and `provenance/ENG-ELM-001.md`.
  No external book, paper, or equation number.
- Own derivations: the section below. No external citation is claimed.
- Third-party code copied: none.
- Validation: analytical fixture in
  `crates/engine-elements-truss/tests/temperature.rs`.
- Comparison with oracle (sibling repo, informative): no. OpenSees, Frame3DD,
  and Stabileo were not run and were not read.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: not assigned by this session.

## Claim

For one straight linear bar, both axial ends fixed, uniform temperature
change, tension positive:

- axial force `N = -(E · A · α · ΔT)` in newtons;
- the magnitude of that force is `E · A · α · ΔT`;
- equivalent joint forces are `f = (E · A · α · ΔT) · b`, with `b = [-d, d]`
  and `d` the unit vector from end i to end j.

The same N is constant at every station. It does not depend on L.
`Element::recover` on a bar from `with_fixed_end_uniform_temperature`
returns that N at every station when the displacement is zero. A mechanical
elongation adds `E · A · ε` to it.

## Not in the claim

The thermal gradient is UNKNOWN in MAM-972 and is not implemented. There is
no second temperature, no `dT/dx`, and no thermal moment. Free elongation
`δ = α · ΔT · L` is not this case and is not returned. `Element::recover`
does not infer temperature from `MemberLoad`. A bar that does not carry the
change returns N = 0 at zero displacement. `respond` and geometric stiffness
stay mechanical. No code-compliance statement is made.

## Evidence record

- claim: the statement in «Claim», with the fixture values below.
- candidate: `engine-elements-truss` 0.1.0 method
  `LinearTruss::fixed_end_uniform_temperature`, parent commit
  `dc2b5b3a27cfc12cfa71345b1f877b515a04c283`.
- state: VERIFIED for that claim only.
- gap: none inside the claim. The gradient exclusion above is outside it.
- inputs: E = 8 Pa, A = 4 m², α = 0.5 1/K, ΔT = 3 K, and the sign variants
  ΔT = −3 K, α = −0.5 1/K, α = 0, ΔT = 0. Lengths 2 m, 5 m, and 7 m.
  Source: this derivation. Unknown gradient stays unknown.
- method: own closed form in this file, evaluated by
  `fixed_end_uniform_temperature` in `src/loads.rs`. Recovery adds that
  axial in `recover::output`.
- sources: this file and the in-repo sign convention. No external source.
- toolchain: rustc 1.99.0 (b940084d7 2026-09-28), cargo 1.99.0 (5f94df478 2026-08-27).
  Arithmetic is f64. No added dependency.
- units: SI. E in Pa, A in m², α in 1/K, ΔT in K, L in m, N and joint forces
  in N, stations as dimensionless x/L.
- tolerances: aligned bars, 0 N (exact equality). Direction-cosine bars,
  1×10⁻¹² N absolute. Derivation in «Tolerance».
- parameters: none. No seed and no iterative solver setting changes N.
- oracle: hand product `8·4·0.5·3 = 48` and the projections
  `(48 · component) / 5` and `(48 · component) / 7`, written in the test and
  not taken from a call to the method. The existing assembler then checks
  that the joint forces, placed on the existing element-load channel, produce
  support reactions +48 N and −48 N on the fixed axial ends and a solved
  displacement of 0.
- execution: `cargo test --offline --locked -p engine-elements-truss`,
  exit code 0, on 2026-11-03. Counts: analytical 4, geometric_producer 5,
  invariants 10, temperature 6, doctests 2. `--offline` did not fail.
  The recovery extension below reran that command: temperature 7,
  doctests 3, exit code 0.
- comparison: pass. See the table.

| Quantity | Expected | Observed | Tolerance | Result |
| --- | --- | --- | --- | --- |
| N, ΔT = 3 K, α = 0.5 1/K | −48 N | −48 N | 0 | pass |
| N, L = 2 m and L = 5 m | equal, −48 N | equal | 0 | pass |
| N, ΔT = −3 K or α = −0.5 1/K | +48 N | +48 N | 0 | pass |
| N, α = 0 or ΔT = 0 | 0 N | 0 N | 0 | pass |
| Joint forces, bar along +X, D2 | [−48, 0, 48, 0] N | same | 0 | pass |
| Joint forces, bar along −X, D2 | [48, 0, −48, 0] N | same | 0 | pass |
| Joint forces, bar along +Z, D3 | [0, 0, −48, 0, 0, 48] N | same | 0 | pass |
| Local end actions, +X | [48, 0, −48, 0] N | same | 0 | pass |
| Joint forces, d = (3/5, 4/5) | ±(144/5, 192/5) N | within bound | 1×10⁻¹² N | pass |
| Joint forces, (2, 3, 6), L = 7 m | ±48·(2, 3, 6)/7 N | within bound | 1×10⁻¹² N | pass |
| Axial reactions, fixed ends | +48 N at i, −48 N at j | same | 0 | pass |
| Free unknown of the unloaded companion bar | 0 m | 0 m | 0 | pass |
| Recovered station N, u = 0, L = 2 m and L = 5 m | −48 N | −48 N | 0 | pass |
| Recovered station N, ΔT = −3 K, u = 0 | +48 N | +48 N | 0 | pass |
| Recovered station N, bar without the change, u = 0 | 0 N | 0 N | 0 | pass |
| Recovered station N, Δu = 0.25 m along d, L = 2 m | −44 N | −44 N | 0 | pass |

`MemberLoad` was not given a temperature field. The public JSON schema was
not edited. A withheld schema note lives outside the repository.

## Own derivation

The bar is straight, homogeneous, and linear elastic. Positive N is tension,
as in ENG-ELM-001: ε = d·(u_j − u_i) / L and N = A σ with σ = E ε when
temperature does not change.

With a uniform change ΔT the axial strain splits as

ε = ε_mechanical + α ΔT.

Stress uses only the mechanical part, σ = E (ε − α ΔT), so

N = E A (ε − α ΔT) = E A ε − E A α ΔT.

Both axial ends fixed means u_i and u_j have no component along d, so ε = 0.
Therefore

N = −(E · A · α · ΔT).

Dimensions: Pa · m² · (1/K) · K = N. The length cancels because ε_thermal is
already dimensionless and uniform. A gradient would leave a strain that
varies along the bar; that term is not written here.

Heating with α > 0 and ΔT > 0 gives N < 0, compression. The product
E · A · α · ΔT is the magnitude of the prevented expansion, not the
tension-positive sign.

Virtual work of σ = E (ε − α ΔT) with ε = (b · u) / L and b = [−d, d] moves
the thermal term to the load side:

K u = f_applied + (E A α ΔT) b.

The equivalent joint forces of this case are f = (E A α ΔT) b = −N b.
For d pointing from i to j and E A α ΔT > 0, end i is pushed against d and
end j along d: the bar pushes outward. The two joint forces sum to zero.

At u = 0 the support reaction on a prescribed DOF is R = −f. The supports
push inward. Along d, R_i = −N and R_j = N.

A uniform force per length cannot replace this. At u = 0 the existing
recovery gives N(x) = q L (1/2 − x/L), which varies along the bar and
depends on L. The thermal N is constant and independent of L.

## Tolerance

Aligned fixtures use E = 8, A = 4, α = 1/2, ΔT = 3. Each value is an exact
f64. The products 8·4 = 32, 32·(1/2) = 16, and 16·3 = 48 are exact, and so
is −48. Direction components are 0 or ±1. The comparison is equality:
absolute tolerance 0 N.

For d = (3/5, 4/5), L = hypot(3, 4) = 5 when correctly rounded, and for
(2, 3, 6), L = 7. Let u = 2⁻⁵³. One division or product rounds by at most
half an ulp, and the length perturbation used above stays under one ulp of
the result. For a force near 48 N that first-order absolute error is below
1×10⁻¹³ N. The declared absolute tolerance is 1×10⁻¹² N, above that bound.
It still rejects a missing sign, a factor two, or a stray factor L.

## Assembly note

`LinearSystem` returns ENG-E-0010, "All DOFs are constrained.", when every
DOF is fixed. The fixture therefore keeps both axial ends of the heated bar
fixed and adds a second, unloaded bar of the same kind with one collinear
DOF free. That unknown solves to 0. The axial reactions remain +48 N and
−48 N. This does not implement a second thermal case.

## Session verification

Command, from the worktree at parent `dc2b5b3a27cfc12cfa71345b1f877b515a04c283`:

`cargo test --offline --locked -p engine-elements-truss`

Exit code 0. No dependency was downloaded. `--offline` did not fail.

## Recovery extension

`Element::recover` adds the fixed-end uniform axial to the mechanical axial.
At zero displacement the station N is `-(E · A · α · ΔT)`. The thermal term
does not depend on L. For the fixture, Δu = 0.25 m on L = 2 m gives
ε = 0.125 and `E · A · ε = 4 N`, so the recovered N is 4 − 48 = −44 N.
`respond` and geometric stiffness do not use this axial.

Command, on the worktree whose parent is
`e4db3fab2aad3602c4d133a9c303df14b5f27760`:

`cargo test --offline --locked -p engine-elements-truss`

Exit code 0 on 2026-11-03. Counts: analytical 4, geometric_producer 5,
invariants 10, temperature 7, doctests 3. `--offline` did not fail. No
dependency was downloaded. `cargo fmt -p engine-elements-truss -- --check`
also exited 0.
