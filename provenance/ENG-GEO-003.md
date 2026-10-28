# Clean-room log · ENG-GEO-003

- Ticket / PR: MAM-46 · ENG-GEO-003; PR Mamut1195/mamut-engine#168 (base commit 97d34a2a4c88bda0070157d8a38ce099e6153dd8).
- Author(s): Codex gpt-6.1-sol high, 2026-10-20 (America/Guayaquil).
- Role: implementer. Specifier: J2, equation summary supplied in the ticket.
- Implemented contract: own SI table generators `py.matlock_soft_clay`,
  `py.reese_sand`, `py.user`; consumption by ENG-MAT-002 interpolation and future ENG-GEO-005.
- Permitted sources used: J2 summary of Matlock, H. (1970), «Correlations
  for Design of Laterally Loaded Piles in Soft Clay», Offshore Technology
  Conference, Houston, **OTC 1204, edición 1970**, Ec. 1–5 and the static curve
  of Fig. 5a described in the specification; Reese, L. C., Cox, W. R., Koop, F. D.
  (1974), «Analysis of Laterally Loaded Piles in Sand», Offshore Technology
  Conference, **OTC 2080, edición 1974**, Ec. 4–9 y 11–16, pasos 2 y 4–8.
  The papers were not consulted over the network; the specifier's summary was implemented,
  without reproducing prose, tables or figures from the publications.
- Own derivations: sections of this log, affine reduction,
  deterministic intersection, normalized evaluation and SI fixtures.
- Third-party code copied: none. No new external dependencies.
- Validation: own analytical validation from the cited equations; Rust tables
  shared by native and wasm, with `source.kind = analytical` and explicit citation.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
  I also did not consult code or results from CalculiX, LAPACK or other prohibited
  engines. Session without external queries or network use; there are no E3S data.
- Clean-room reviewer: **PENDING (J4)**.

## API and units

`engine_geo::py` is added, without modifying equations or parameters of other
methods. Parameters use `Option<Real>` to represent absence and produce
`EngineError`: ENG-E-0003 missing, ENG-E-0010 invalid domain/degeneracy/range,
ENG-E-0008 non-finite/overflow. No new defaults or unwrap/expect/panic/unsafe
in the library. Tables are also validated for finite slopes,
strict order and monotonicity, rejecting loss of positive slopes due to
underflow. The bilateral range is also limited to finite differences.

Inputs use m, N, Pa, N/m³; output p in N/m, y in m. x is positive
downward and d=b is the diameter. `effective_stress_pa(&dyn StressProfile, x)` queries
the profile (including `StratifiedProfile`) without extrapolation, converts kPa to Pa
by 1000 and rejects negative σ'v. The profile may represent negative effective
stress, but the domain of these generators requires σ'v≥0. Measured/Assumed
provenance is preserved by querying the profile's own report; the
adapter does not declare external data as measured. c, ε50, J, geometry and
sand parameters are always supplied explicitly: the current profile lacks
those fields. Matlock accepts variable σ'v, without imposing γ'x on a profile.
Reese requires explicit constant γ': it is not replaced with σ'v/x and no
stratified extension of the homogeneous expression for Xt is invented.

`Sampling` requires a plateau endpoint and allows an explicit list (even
empty) of positive, finite, strictly increasing y values within the endpoint.
The origin, required vertices, plateau and odd reflections are added; only these
internal vertices are sorted and deduplicated. The endpoint must strictly exceed
8yc or yu; no user point is discarded. No default n is offered.
The approximation between points is secant; the user controls its refinement.
Matlock has infinite derivative at zero; the table uses the secant to the first y>0.
The Matlock branch includes 8yc; Reese includes yk, ym and yu (at x=0, without yk).

`py.user` receives a y≥0 branch that must already include (0,0), requires at least two
points, nondecreasing p≥0 and reflects without modifying its endpoints. `PyCurve::at`
interpolates only within the tabulated interval, returning an error outside. The
Matlock/Reese tables have explicit plateaus. `MultilinearElastic` extends the end
segments by its own contract: for `py.user` the consumer must
explicitly accept that policy or retain the bounded evaluator. No
plateau is injected and no silent extrapolation is performed by the generator.

The existing law interprets dimensionless ε and σ [Pa]. The integration test
uses explicit L=2 m and area S=0.5 m²: ε=y/L, σ=pL/S, p=σS/L. Its tangent is
recovered as dp/dy=(dσ/dε)S/L². Tuples may be passed directly as a
numerical interpolator, but the future pile must preserve units and
apply its own tributary length. No pile, axial bar or
truss is included. They are not exposed in engine-api/engine-input: the physical contract of
ENG-GEO-005 is missing and adding fields to models without a consumer provides no integration.
The JSON extension remains pending together with the pile, without blocking this crate.

## Parameters supplied by the user

- Matlock: x≥0, d>0, c>0, σ'v≥0, J>0 and ε50>0, all finite and mandatory.
  The J=0.5/0.25 and εc=0.005–0.020 mentioned in OTC 1204 are guidance,
  never defaults or universal calibration limits.
- Reese: x≥0, diámetro>0, 0<φ<π/2, γ'>0, 0<α<π/2, K0>0, A>B>0, k>0.
  α=φ/2 and K0=0.4 are documented as proposals from step 2 of the method;
  they are required explicitly even at x=0. β and Ka are calculated by Eq. 6/7.
  A and B are only in Fig. 8/9 and k is provided in a table in the paper:
  they are not copied or digitized; the user supplies these parameters.
  n>1, yk<ym and a unique positive intersection Xt are also required under the
  criteria described below. Errors are returned for other domains.
- Sampling points/endpoint are also mandatory; no final
  deformation, empirical tolerance or missing soil factor is assumed.

## Matlock: derivación analítica

Invented SI case, not a published example: d=1 m, c=20000 Pa, γ'=10000 N/m³,
J=0.5, ε50=0.01, σ'v=γ'x. Eq. 3 gives Np=3+x (x expressed in m for this case);
Eq. 2 limits it to 9. For x=2 m: Np=5 and Eq. 1 gives pu=100000 N/m.
Eq. 5 gives yc=2.5·0.01·1=0.025 m. The static curve of Fig. 5a allows:

- y=yc/8=0.003125 m: p=0.5pu·(1/8)^(1/3)=25000 N/m.
- y=yc=0.025 m: p=50000 N/m.
- y=8yc=0.2 m: p=pu=100000 N/m, continuous with the plateau.
- Explicit endpoint y=0.4 m: p=pu. Signs are reflected for negative y.

Eq. 4: xr=6·1/(10000·1/20000+0.5)=6 m. Eq. 3 at x=6 gives Np=9,
at x=5 gives 8 and at x=7 gives 10 before the limit (9 afterward). The fixture compares
all three points and the independent check xr=6. At x=0, Np=3: a zero curve
is not imposed on clay, because Eq. 1/3 retain surface resistance.

## Reese: derivación analítica

Invented case to simplify radicals: diámetro=1 m, γ'=10000 N/m³,
φ=π/6, **explicit α=π/6** (different from the proposed φ/2), K0=0.4, A=1,
B=0.5. It is not an example from the paper or a recommended calibration. The freedom
of parameterized α is exercised; conformance also accepts explicit α=φ/2.
β=π/3, tanβ=√3, tanφ=tanα=1/√3, sinβ=cosα=√3/2,
tan(β−φ)=1/√3, Ka=1/3 (Eq. 6/7).

Dividing Eq. 5 by γ'H for H>0:

q(H)=q1 H+q0,
q1=K0·tanφ·sinβ/[tan(β−φ)cosα] + tan²β·tanα/tan(β−φ)
   +K0·tanβ·(tanφ·sinβ−tanα)=3+0.2√3,
q0=diámetro·[tanβ/tan(β−φ)−Ka]=8/3 m.

The bracket in Eq. 5 is affine in H: each inner term has units of m;
pct is quadratic in H, with units N/m. Eq. 8 divided by γ'H gives
D=(80/3)+(6/5)√3=28.74512763574932 m. The equation for the nontrivial root
(step 4) is (3+√3/5)Xt−[24+(6/5)√3]=0. Its independent closed-form root is
Xt=[24+(6/5)√3]/[3+√3/5]=7.792966107085287 m.

In general the implementation requires q1>0, q0>0 and D−q0>0. Then
f(H)=q1 H−(D−q0) increases for H>0, f(0)<0 and has a unique positive root.
An upper bound is sought starting at the diameter and doubling up to
128 times; 96 bisections are performed in constant order. Final width
≤8·EPSILON·cota superior is required (binary64 relative precision); a root
that cannot be resolved within that budget is rejected. If no bound is found or
an intermediate overflows, a typed error is returned. Neither an assumed physical
tolerance nor the trivial root H=0 is used; every unsupported branch is explicit.
For H≤Xt, pct is taken and for H>Xt, pcd. They are continuous at the intersection.

For H=2 m, Eq. 5 and 8 give, calculated from the radicals:

pct=20000·[26/3+(2/5)√3]=187189.73979388434 N/m,
pcd=20000·[80/3+(6/5)√3]=574902.5527149864 N/m.

H<Xt, hence pc=pct. Eq. 9/11: pu=pc and pm=pc/2.
Steps 6/7: yu=3/80=0.0375 m, ym=1/60 m and yu−ym=1/48 m.
Eq. 13 gives m=24pc [N/m²]; Eq. 14 gives n=(pc/2)/(24pc/60)=5/4.
Eq. 15 gives C=(pc/2)/(1/60)^(4/5). Eq. 12 is evaluated as
pm·(y/ym)^(1/n), equivalent without powers of dimensional quantities.

For this case k=60pc per square meter is **chosen** =
11231384.38763306 N/m³ (synthetic value, without using the paper's table).
Thus kH=120pc [N/m²]. From normalized Eq. 16:
yk=ym·[pm/(kHym)]^[n/(n−1)]=(1/60)·(1/4)^5=1/61440 m.
It is less than ym. The k/m/u checks are:

| Point | y [m] | p [N/m] |
| --- | --- | --- |
| k | 0.000016276041666666666 | pc/512 = 365.60496053493034 |
| m | 0.016666666666666666 | pc/2 = 93594.86989694217 |
| u | 0.0375 | pc = 187189.73979388434 |
| plateau | 0.4 | pc = 187189.73979388434 |

Independent interior checks: p(yk/2)=pc/1024=182.80248026746517 N/m;
p(ym/2)=(pc/2)·2^(−4/5)=53756.13654338948 N/m;
p((ym+yu)/2)=3pc/4=140392.30484541325 N/m. They are evaluated as samples,
to distinguish the analytical parabola from its multilinear approximation.

For H=10 m: pct=100000·[98/3+2√3]=3613076.828180442 N/m,
pcd=100000·[80/3+(6/5)√3]=2874512.763574932 N/m. H>Xt, therefore
pu=pcd=2874512.763574932 N/m. The same explicit k is retained.
An intermediate check H=4<Xt gives pct=40000·[44/3+(4/5)√3]
=642092.2925088706 N/m, which must also be pu (A=1).

Additional check of the proposed α: α=φ/2=π/12, H=2 and the other data unchanged.
cosα=(√6+√2)/4 and tanα=2−√3. The four terms in the bracket of Eq. 5 are,
respectively, 0.7172603777344427, 5.7846096908265245,
0.3215390309173471 and −1/3 m. Their sum multiplied by γ'H=20000 N/m² gives
pct=129801.51532289962 N/m. The slope of q(H) is
1.9117045497391574 and Xt=13.64147036875595 m; they are separate checks of the
conformance test, derived from the formula without the generator's reduction.

Degenerate branches: A≤B or n≤1 are errors; yk≥ym is also an error, without
replacing the curve with another. Exact comparisons on f64, without tolerance
that accepts a degeneracy. At x=0 all parameters and n are validated through
the independent form of pc: n=[B/(A−B)]·[(yu−ym)/ym]. Eq. 5/8 produce
pct=pcd=0: a zero table is returned, yk=None, without dividing by kx. The
normalized formulas for n and yk are own algebraic derivations from Eq. 13–16.

## Validation and scope

The three fixtures in `crates/engine-validation/tests/support/py_table.rs`
have an analytical source, citation and literal checks derived above. They share
a native/wasm runner, without JS imports or npm. Tolerance A=1e-8 in the SI unit of
each check and R=1e-12: rounding margin for radicals/libm/bisection, without
discretization error at vertices. It does not come from an external engine.
Only `engine_core::math` is used for deterministic transcendental functions.

Only one local development dependency on engine-laws is added to
engine-validation to verify dimensional consumption of the tables; Cargo
updates the corresponding edge in the lockfile. The scalar guard accepts
the exact path of the new wasm runner with no_mangle and extern C, like the
existing runners; it does not allow unsafe or broaden permissions by pattern.
clean-room/**, docs/CLEAN_ROOM.md, workflows, CODEOWNERS and vendor are not modified.

Cyclic loading remains outside scope (ENG-GEO-009). J4 clean-room review and
physical pile/JSON integration (ENG-GEO-005) remain pending.
