# Clean-room log · ENG-GEO-006

- Ticket: MAM-48 · ENG-GEO-006, «Multilayer 1D consolidation».
- Author: Codex (GPT-6), 2026-10-15, America/Guayaquil.
- Role: specifier and implementer in a clean session; no oracle operator or
  auditor role; no delegation to other agents.
- Own contract: `consolidation/1`, engine-geo Rust API, not element/1.
- Sources used: own derivations reproduced below; internal
  engine-geo code (profile and provenance), engine-core (catalog and math),
  engine-linalg (CSR and existing factorization) and engine-validation
  fixtures/WASM pattern; docs/CLEAN_ROOM.md and provenance/TEMPLATE.md.
- Conceptual published reference: Karl Terzaghi (1943), *Theoretical Soil
  Mechanics*, primera edición, John Wiley & Sons, New York, theory of
  one-dimensional consolidation. No copy of the book was consulted in this
  offline session and no unverified pages/equation numbers are attributed.
  Separation of variables, convolution and all fixture data are
  own, derived here; no book example or listing is transcribed.
- Third-party code copied: none. New dependency exclusively by path on
  engine-linalg; no new external packages or external version changes.
- Validation: own analytical solutions, invariants and refinement; the
  same seven fixtures and convergence tables in native and WASM/Node
  after review r32 (six in the initial delivery).
- External oracle comparison: no; no network or consultation of other engines.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Independent clean-room reviewer: pending assignment; no human
  review is simulated in this delivery.

## Model, units and mandatory data

Depth z positive downward, origin at the top face, t in seconds,
Δσ and excess u in kPa (compression positive). s in m, positive downward.
Saturated layers, small strains, linear skeleton, incompressible water,
1D Darcy flow and spatially uniform surcharge. Hydrostatic pressure is not
computed: it may be added separately using the existing geostatic profile.
Thicknesses may be transferred from Layer, but γ, γsat and zw, which do not
participate in the incremental equation, are not required. StressProfile was not modified.

Each layer requires thickness >0 in m, cv>0 in m²/s, mv>0 in 1/kPa and k>0 in m/s;
global γw>0 in kN/m³ is required. All these data, including thickness and each
load ordinate, use Parameter with Measured/Assumed Provenance. There are no
physical defaults; missing cv, mv, k and γw give ENG-E-0003.
cv=(k/γw)/mv is checked with relative tolerance 1e-10, a consistency
tolerance for redundant data, not a physical calibration. An incompatible
set gives ENG-E-0010: one of the three is never silently chosen.
Unloading and negative stress are allowed; results are not clipped.

Assumed is reported with unambiguous paths for all parameters: first γw,
then thickness/cv/mv/k per layer and finally the ordinates of the entire history,
even those after the last output. The complete table is validated
before integration. θ and mesh/time controls are numerical options, without
physical Provenance. Crank–Nicolson θ=0.5 is the only default; the
maximum step is always supplied explicitly.

## Conservative derivation and finite elements

Let ε=mv(Δσ−u). Volume continuity and Darcy q=−(k/γw)u_z give

```text
mv u_t − ∂z[(k/γw)u_z] = mv Δσ_t
cv = k/(γw mv)
```

For a homogeneous layer this is u_t=cv u_zz+Δσ_t. At an interface without
surface storage, u is continuous and (k/γw)u_z is continuous.
Integrating by parts with continuous test v, zero on draining faces:

```text
∫ mv v u_t dz + ∫ (k/γw) v_z u_z dz = ∫ mv v Δσ_t dz
```

Internal boundary terms cancel through flux continuity.
Nondraining faces have natural zero flux, draining faces essential u=0.
Top, bottom or both drains are allowed; no artificial symmetry is imposed.
Each layer has its own number of elements, without elements crossing an
interface. Interface nodes are unique, not duplicated, and k is not averaged.

With ξ in [0,1], N=[1−ξ,ξ], B=[−1/h,1/h], integrating monomials exactly gives

```text
Me = mv h/6 [2 1; 1 2]
Ke = (k/γw)/h [1 −1; −1 1]
fe = mv h/2 [1;1]
```

Reduced M and K are assembled as CSR with Triplet/SparseMatrix from
engine-linalg, in layer/element/row/column order. Vector f is integrated
before eliminating drains: f_f is NOT M_ff·1 when there are prescribed nodes.
Drain pressure is always zero. There is no HashMap, threading or regularization.

```text
(M + θ Δt K) u[n+1] = (M − (1−θ) Δt K) u[n] + f (Δσ[n+1]−Δσ[n])
```

History starts at t=0 and has at least two strictly increasing
abscissas. Δσ(t) interpolates linearly without extrapolation. Δσ(0) represents an
instantaneous load on the initial at-rest state: free nodes u=Δσ(0),
drains u=0. No independent initial pressure parameter is added.
Because of this interpolation, s(0) has a boundary error O(h); it is not presented
as an exact initial settlement. For constant positive times, observed spatial
convergence is O(h²). CN may oscillate initially if Δt/h² is large;
the scheme's solution is preserved, without hidden damping or clipping.
θ=1 allows the caller to choose high-frequency damping.

The integrator reaches all load breakpoints and outputs exactly.
Each interval is divided into ceil(longitud/max_step_s) equal substeps;
it is factorized once per interval with LinAlgBackend::factorize and the
factor is reused for all its RHS. solve uses ReferenceBackend; solve_with_backend
accepts the existing contract. Initial review confirmed that engine-linalg
does NOT contain an operational band/sparse solver: it has CSR and reference dense
symmetric factorization; faer is a stub. That factorization is used, without
writing another solver or claiming sparse scalability. Reference cost:
O(n³) per factor, O(n²) per solve and O(n²) memory.

For θ<0.5, a mode λ of the pair (K,M) requires Δt λ≤2/(1−2θ).
Gershgorin and the Rayleigh quotient give the sufficient bounds

```text
a = min_i (Mii − Σj≠i |Mij|) > 0
b = max_i Σj |Kij| > 0
λmax ≤ b/a; por tanto Δt ≤ 2a/[(1−2θ)b]
```

The consistent linear mass is strictly diagonally dominant even with
different layers and after eliminating drains. This bound is checked for every
interval with θ<0.5; it is conservative, not a search for exact λmax.
θ≥0.5 does not have this linear stability restriction.

## Outputs and integration

All nodal depths and u(z_i,t_j) are returned. pressure_at
interpolates the FE field at any depth of the profile, without extrapolation;
time is selected by output index. For each element:

```text
s_e = mv h [Δσ − (u_i+u_j)/2]
s = Σ s_e
s∞(t) = Δσ(t) Σ mv h
U(t) = s(t)/s∞(t)
```

This is the average degree based on settlement, weighted by compressibility. In homogeneous
soil it agrees with 1−∫u dz/(H Δσ). In multilayer soil it is not replaced with an
arithmetic mean without mv. For Δσ=0, U=None even if there is residual pressure;
s remains defined. During unloading U may fall outside [0,1].

Explicit limits of the reference backend: 2048 elements, up to 100000
history/output points, 8000000 nodal output values and 1000000
substeps per interval. Unrepresentable sizes and accumulations are rejected.
ENG-E-0010 for ranges, geometry, incompatibility, stability and limits;
ENG-E-0008 for NaN/infinity/overflow; ENG-E-0003 for missing data.
Solver errors retain code and entities within the Diagnostic
with catalog message and context in hint. No panic!, unwrap or expect in
the new library module; forbid(unsafe_code) is inherited from the crate.

## Serie de Terzaghi: derivación y fixtures

With a drain at x=0 and impermeable face at x=Hdr, separating u=X(x)T(t) gives
X=sin(Mx/Hdr), M=π(2m+1)/2, T=exp(−M²Tv), Tv=cv t/Hdr².
Projection of the uniform initial condition Δσ onto those sines has
coefficient 2Δσ/M, since ∫sin(Mx/Hdr)dx=Hdr/M and
∫sin²(Mx/Hdr)dx=Hdr/2. Integrating the field over thickness gives

```text
u(x,t)/Δσ = Σ (2/M) sin(M x/Hdr) exp(−M² Tv)
ū(t)/Δσ = Σ (2/M²) exp(−M² Tv)
U(Tv) = 1 − Σ (2/M²) exp(−M² Tv)
```

In double drainage Hdr=H/2 and each half-thickness is reflected. With a bottom drain
x=H−z is used. Tv does not change by a factor of four without changing Hdr.
For a ramp 0→Δσ over R (r=cv R/Hdr²), the step response is
convolved. Each exponential in the sum is replaced by

```text
A(M,Tv,r) = exp[−M²(Tv−min(Tv,r))] · (1−exp[−M² min(Tv,r)])/(M² r)
```

1−exp(−a) is evaluated with −engine_core::math::exp_m1(−a). Subsequent linear
unloading is the subtraction of a shifted ramp; no hidden jump is created.
The oracles use 2048 terms, math::sin/cos/exp/exp_m1 from the core.
For steps and Tv≥.01, the exponential tail is negligible in f64.
For a ramp, the mean tail is bounded by
2/r Σm≥N 1/M⁴ ≤ 2/[3r π⁴(N−1/2)³], <5e-12 with r=.2,N=2048;
for pressure, |sin|≤1 gives bound 1/[r π³(N−1/2)²]<4e-8.
These limits are below the discrete tolerances.

Versioned table: engine-validation/tests/support/consolidation_table.rs.
All fixtures contain source and citation with edition. H=2 m,
cv=.25 m²/s, mv=.002 /kPa, γw=10 kN/m³, k=.005 m/s, Δσ=100 kPa.
These are simple analytical data, without claiming to represent real soil.
Reference mesh: 64 elements for single drain, 128 for double;
maximum ΔTv=.0005. Outputs Tv=.01,.05,.1,.2,.4,.6,1.

| Fixture | History and solution |
| --- | --- |
| Top | Constant step; series with Hdr=2 |
| Bottom | Reflection of top, same U and s |
| Double | Step, Hdr=1, identical U(Tv) |
| Ramp | Ramp up to Tv=.2 and plateau; analytical convolution |
| Ramp/unloading | Ramp up to .2, plateau up to .4, linear unloading up to .6, zero up to 1 |
| Multilayer | Two transformed layers, same series and nonzero interfacial flux |

Acceptance requires RELATIVE error of U <.005, absolute pressure error
normalized by 100 kPa <.004 and settlement error normalized by .4 m <.0015.
At Δσ=0, U=None and residual pressure/settlement are checked against convolution.
All nodes u(z,t) are also compared, not just U.

### Multicapa con flujo no nulo: derivación propia

Take x=∫dz/√cv and constant a=mv√cv. Then
mv dz=a dx and (k/γw)u_z=mv√cv u_x=a u_x; the weak form becomes
a∫v u_t dx+a∫v_x u_x dx=a∫v Δσ_t dx. It is homogeneous diffusion with
coefficient 1 in x, preserving pressure, flux and settlement, even at the interface.

Layer 1: H1=1 m, cv1=.25, mv1=.002, k1=.005.
Layer 2: H2=2 m, cv2=1, mv2=.001, k2=.01.
γw=10; a=.001 in both, transformed lengths 2+2=4.
Top drain, Tv=t/16, 32 elements per layer; the transformed domain
is exactly a uniform mesh of 64 elements. ∫mv dz=.004.
The interface is at x/Hx=.5 and has nonzero u_x: validation does not cover only a plane
of symmetry with zero flux. At Tv=.2 both elemental fluxes
−(k/γw)Δu/h are compared with the series derivative; each must differ by <3%.
Elemental derivative fluxes converge on both sides; exact continuity
is weak/nodal, pointwise equality of P1 slopes is not promised.

### Revisión J4 r32: multicapa no transformable por autofunciones

General published reference: H. S. Carslaw y J. C. Jaeger (1959),
*Conduction of Heat in Solids*, segunda edición, Oxford University Press:
diffusion in composite media and separation of variables. The heat equation
with piecewise capacity and conductivity is the same diffusion problem.
No copy was consulted in this offline session; no pages or
specific formulas are attributed to the book. All the following development and its code are
own; no third-party listings, values or examples are copied.

Versioned fixture in `engine-validation/tests/support/consolidation_table.rs`
(`MULTILAYER`, `MULTILAYER_SOURCE`, `MULTILAYER_CITATION`), with spectral
reference in `support/consolidation_multilayer.rs`. Top drain u(0,t)=0,
impermeable base u_z(3,t)=0, uniform 100 kPa step, γw=10 kN/m³:

| Layer | H (m) | cv (m²/s) | mv (1/kPa) | k (m/s) | a=mv√cv |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | .5 | .25 | .002 | .005 | .001 |
| 2 | 1.5 | 2.25 | .001 | .0225 | .0015 |
| 3 | 1 | 1 | .003 | .03 | .003 |

The three k/mv ratios are 2.5, 22.5 and 10. The H/√cv lengths are 1,
but the impedances a differ. A coordinate change x=f(z) that
homogenizes the diffusion coefficient requires f' proportional to 1/√cv;
the resulting flux coefficient is proportional to mv√cv, discontinuous.
Therefore heterogeneity is not eliminated by rescaling z:
the transmission conditions remain nonhomogeneous in x.

Separating u=X(z)exp(−λt) in mv u_t=(D u_z)_z, D=k/γw, gives in each layer
X''+(λ/cv)X=0. Let ω=√λ and ξ=(z−z_superior)/√cv in [0,1]. Write
X_i=A_i cos(ωξ)+B_i sin(ωξ). Then p=D X_z=a_i X_ξ. Integrating
conservation at an interface without storage requires continuity of p;
pressure is also continuous. Since γw is common, this is equivalent to
continuity of k X_z. With c=cos ω, s=sin ω, both conditions give

```text
A_1=0, B_1=1                    (dren superior y normalización)
A_{i+1}=A_i c+B_i s
B_{i+1}=(a_i/a_{i+1})(−A_i s+B_i c)
```

Equivalently, state (X,q), q=p/ω, propagates in each layer through
the matrix [[c,s/a_i],[−a_i s,c]]. Starting at (0,a_1), flux at the base
is q_base=a_1 c[c²−B s²], where
B=a_2/a_1+a_3/a_1+a_3/a_2=6.5. The impermeable base requires q_base=0.
There is no λ=0 mode: the steady linear solution with drain and impermeable base
is zero. The complete positive spectrum is, for n=0,1,...,

```text
α=atan(1/√6.5)
ω = nπ+α, nπ+π/2, nπ+π−α
λ = ω²
```

These are the three simple roots per interval of length π. The operator
is self-adjoint with weight mv: subtracting the equations of two modes, the
end and interface terms cancel precisely through continuity
of X and p. Modes are orthogonal in ∫mv X_n X_m dz. Projection of
u(z,0)=100 onto this system gives b_n=100 I_n/N_n, with
I_n=∫mv X_n dz, N_n=∫mv X_n² dz. Since mv dz=a_i dξ, both integrals
are evaluated without mesh or quadrature, summing by layer:

```text
I_i = a_i [A_i sin ω+B_i(1−cos ω)]/ω
N_i = a_i { A_i²[1/2+sin(2ω)/(4ω)]
          + B_i²[1/2−sin(2ω)/(4ω)] + A_i B_i sin²ω/ω }
u(z,t) = Σ 100 I_n/N_n X_n(z) exp(−ω_n²t)
s(t) = .55 − Σ 100 I_n²/N_n exp(−ω_n²t)
U(t) = s(t)/.55
k u_z = Σ 100 I_n/N_n · 10 a_i ω_n
        [−A_i sin(ω_n ξ)+B_i cos(ω_n ξ)] exp(−ω_n²t)
```

∫mv dz=.0055 m/kPa and s∞=.55 m. The oracle does not use FE matrices, the linear
backend, time step or spatial integration of the implementation. It only
evaluates these closed-form expressions with `engine_core::math`.

16 groups of three roots (48 modes) are included. For omitted modes,
ω≥16π, |X|≤1 because each transmission rotates (A,B) and contracts its second
component (a_i/a_{i+1}≤1). Thus |I|≤.0055 and
N≥.001[1/2−1/(4ω)]>.000495. Both each u/100 coefficient and each
weighted mean pressure coefficient are bounded by .0055/.000495.
For t≥.1, the tail of either of those quantities is less than

```text
3(.0055/.000495) exp[−.1(16π)²] / {1−exp[−.1·33π²]} < 1e−107.
```

This bound is for truncation in exact arithmetic; it does not claim rounding
error of that order. Positive norm, both boundary conditions and continuity of X and k X_z
at both interfaces are checked for all 48 modes
(absolute tolerance 1e−12 in the mode's units).

FE: 32 elements per layer (96 in total), Crank–Nicolson and maximum Δt .002 s;
outputs t=.1,.5,1,3,10,30 s. All nodes are compared, including both
interfaces. Tolerances set as in the previous fixtures:
|u_FE−u_serie|/100<.004, |s_FE−s_serie|/.55<.0015 and relative error U<.005.
At t=1 s both interfaces have |k u_z|>.01 kPa·m/s; each elemental
slope on both sides is compared with the common analytical flux with relative
error <.03. Pointwise equality between P1 slopes is not required.

Native measured maxima: presión/100=.0004286818;
asiento/.55=.0000667471; relative U=.0010288205;
relative flux=.0181946330. These measurements verify the tolerances;
the reference is obtained from the equations, without fitting to the solver.
The shared harness now runs seven fixtures in native and WASM.

### Review r32: middle cv discrepancy and mutation test

`engine-geo/tests/consolidation.rs` adds
`middle_layer_cv_discrepancy_is_rejected_without_substitution`. First it
accepts the three compatible layers from the table above; then it changes
only cv of `layers[1]` by factors 1e−1 and 10, retaining k and mv.
It requires ENG-E-0010 and context `layers[1]`/`cv incompatible`. This checks
that the redundant datum is not silently replaced by k/(γw mv).

Actual check, with `CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6`
exported, using this command before, during and after the mutation:

```sh
cargo test --offline -p engine-geo --test consolidation \
  middle_layer_cv_discrepancy_is_rejected_without_substitution -- --exact
```

1. Original: 1 passed, 0 failed.
2. Temporarily inserted in `ConsolidationProfile::new`, immediately
   after reading cv and before reading mv:
   `let cv = if i == 1 { cv * 1e-1 } else { cv };`.
   The test failed (exit 101, 0 passed, 1 failed) when accepting the compatible
   profile: ENG-E-0010, `layers[1]: cv incompatible con k/(γw mv)`.
3. The original file was restored byte for byte through `finally` of a
   local Python script; the same test passed again.
4. Additional control: `i==1` was temporarily excluded from the discrepancy
   check (`if i != 1 && ...`). The test failed (exit 101) at
   `expect_err` when accepting the incompatible middle cv. The file was restored
   byte for byte again and the test passed.

Both mutations were removed; `src/consolidation.rs` has no diff.
Neither the library nor its error policy was changed. No external sources,
no network, no new dependencies and no delegation; the initial clean-room
declaration is retained.

### Review r34: cv relative tolerance boundary

`engine-geo/tests/consolidation.rs` adds
`cv_consistency_relative_tolerance_boundary`: accepts ratio `1+5e-11` and
rejects `1+2e-10` with ENG-E-0010 and context `layers[0]`/`cv incompatible`.

Exact construction within f64: k=.0078125=2⁻⁷ m/s,
γw=8=2³ kN/m³ and mv=.0009765625=2⁻¹⁰ /kPa. The divisions
`k/γw=2⁻¹⁰` and `(k/γw)/mv=1` are exact. `cv=1.0+δ` is assigned, so
`cv/cv_derivado` preserves exactly the bits of the f64 result of
`1.0+δ`, without additional parameter rounding. The test checks those
bits, the derived cv equal to one and the tolerance side of each case.
Mathematical decimals are not exactly representable in binary;
these are their values rounded to f64 and their effective discrepancies:

| δ | Bits of the f64 ratio `1.0+δ` | `razón−1.0` in f64 | Result |
| --- | --- | --- | --- |
| 5e-11 | `0x3ff0000000036f9c` | 5.000000413701855e-11 | Accepted |
| 2e-10 | `0x3ff00000000dbe70` | 2.000000165480742e-10 | Rejected |

Actual test of the four mutations, without network and with
`export CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6`:

```sh
cargo test --offline -p engine-geo --test consolidation \
  cv_consistency_relative_tolerance_boundary -- --exact
```

Only the comparison literal `> 1e-10` in
`ConsolidationProfile::new` was temporarily changed; the diagnostic message remained unchanged.

| Library tolerance | Exit | Test result |
| --- | ---: | --- |
| 1e-10 original | 0 | 1 passed, 0 failed |
| 1e-1 mutated | 101 | 0 passed, 1 failed |
| 1e-3 mutated | 101 | 0 passed, 1 failed |
| 1e-6 mutated | 101 | 0 passed, 1 failed |
| 1e-9 mutated | 101 | 0 passed, 1 failed |
| 1e-10 restored | 0 | 1 passed, 0 failed |

Each mutation fails at `expect_err("razón 1+2e-10 debe rechazarse")`:
the constructor incorrectly accepts the profile. Case `1+5e-11`, executed
first, still passes. A Python script restored the original bytes in
`finally` after each mutation and on exit; byte-for-byte equality was
checked before repeating the original test. No mutation or diff remains
in the library. No external sources, new dependencies or delegation.

r34 verification (2026-10-15), with the same exported `CARGO_TARGET_DIR`
and `CARGO_NET_OFFLINE=true`, all passed:

- `cargo fmt --all -- --check`.
- `/home/box/.cargo/bin/cargo clippy --offline --workspace --all-targets -- -D warnings`.
- `cargo test --offline --workspace --no-fail-fast`: 557 tests passed,
  0 failed, 0 ignored, across 87 test/doctest targets.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --no-deps`.
- `/home/box/.cargo/bin/cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown`.
- `bash clean-room/check.sh origin/main`: `Clean-room: OK` on files
  already known to Git; no temporary index is required for this change.
- `git diff --check`: passed; library without panic!, unwrap or expect.

Native and rustdoc with Cargo/Rust 1.85.1; clippy and WASM with already installed 1.99.0.
Only the integration test and this provenance change; docs/CLEAN_ROOM.md,
.github/**, clean-room/** and vendor/** were not touched. No network and no
`git add` or commit; the index remains unchanged for J2.

## Convergence and measured results

Series at Tv=.2: U=0.50408782020255 approximately. ΔTv=.00025 is set
for spatial refinement, top drain and nested uniform meshes.

| Elements | Max. ΔTv | U FE | Absolute error against series |
| ---: | ---: | ---: | ---: |
| 8 | .00025 | .508227050481 | .004139230278 |
| 16 | .00025 | .505126887757 | .001039067555 |
| 32 | .00025 | .504347873894 | .000260053691 |
| 64 | .00125 | .504153341943 | .000065521741 |
| .00025 | .504152865634 | .000065045431 |

Ratios near four: second-order spatial convergence for
this observable at positive time. The harness requires strict decrease.

For time, n=64 is set. The temporal reference is the same discretization
with ΔTv=.00025; it is not confused with the continuous solution. CN and the initial
step may retain high frequencies at large steps; total error
against the series may change sign when offset by spatial error.
The harness requires |U(ΔTv)−U(.00025)| to decrease strictly.

| Max. ΔTv | U FE | Absolute error against series |
| ---: | ---: | ---: |
| .02 | .497450256297 | .006637563906 |
| .01 | .502321203644 | .001766616559 |
| .005 | .503898123046 | .000189697157 |
| .0025 | .504148482059 | .000060661856 |
| .00125 | .504153341943 | .000065521741 |
| .00025 | .504152865634 | .000065045431 |

On the reference mesh, over all times of the six fixtures:

| Fixture | Maximum relative U error (%) | Maximum absolute u/100 kPa error (%) |
| --- | ---: | ---: |
| Top drain | .25941147 | .14919098 |
| Bottom drain | .25941147 | .14919098 |
| Double drainage | .25941147 | .14919098 |
| Ramp | .14477007 | .00368148 |
| Ramp and unloading | .14477007 | .00407868 |
| Multilayer with flux | .25941147 | .14919098 |

## Initial delivery verification (before r32)

The engine-geo tests include hand-derived one-element equations
for θ=0,.25,.5,.75,1 (consistent mass and complete load vector), both
single drainages, load breakpoints without outputs at them, zero load, interpolated
pressure, all Assumed provenance, missing data in deep layers,
incompatible materials, extreme geometry and dimensions, unstable θ,
invalid histories, absence of extrapolation, bit-for-bit repetition and asymptotic
settlement with double drainage.

The examples/consolidation_wasm.rs example shares the full table and harness
with the native test: it exports consolidation_benchmarks()=6 if the convergence
tables also pass. scripts/run-consolidation-wasm.cjs verifies absence
of external imports and a return of six. It requires no packages or network.

Final checks, all passed, without network (`--offline`) and with the
exported `CARGO_TARGET_DIR` `/workspace/.cargo-target/mamut-engine`:

- `cargo fmt --all -- --check`.
- `/home/box/.cargo/bin/cargo clippy --offline --workspace --all-targets -- -D warnings`.
- `cargo test --offline --workspace --no-fail-fast`: 555 tests passed,
  0 failed, 0 ignored, across 87 test/doctest targets. Includes the nine
  new API tests and the harness of six fixtures with convergence.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --no-deps`.
- `/home/box/.cargo/bin/cargo check --offline --workspace --all-targets
  --target wasm32-unknown-unknown`: includes library, tests and examples;
  also satisfies the requested workspace library check.
- `/home/box/.cargo/bin/cargo build --offline --release -p engine-validation
  --example consolidation_wasm --target wasm32-unknown-unknown`.
- `node crates/engine-validation/scripts/run-consolidation-wasm.cjs
  /workspace/.cargo-target/mamut-engine/wasm32-unknown-unknown/release/examples/consolidation_wasm.wasm`:
  six analytical fixtures and convergence passed, without external imports.
- `bash clean-room/check.sh origin/main`: passed with all new
  files in the index, to review the full diff.
- `git diff --check` and `git diff --cached --check`: passed.
- Inspection of the new module: no panic!, unwrap, expect, HashMap or threads.
  docs/CLEAN_ROOM.md, .github/**, clean-room/** and vendor/** were not modified.

Native and rustdoc with system Cargo/Rust 1.85.1 (workspace MSRV);
clippy and wasm with Cargo/Rust 1.99.0 previously installed at
/home/box/.cargo/bin. There were no downloads or new external dependencies.
The WASM execution covers these six fixtures and both convergence tables;
execution of the other suites in Node or binary equality between
architectures is not claimed. The dense reference backend and initial interpolation
still have the numerical and size limitations described above.

During verification two local issues were corrected: clippy requested
simplifying multiplication by −1 in the harness; the determinism guard
required separate `use engine_core::math;`, rejecting the grouped import although
the calls already used the central module. No guard was modified;
fmt, clippy, the entire suite, doc and WASM checks/build/runtime were repeated
successfully after those corrections.

## Final r32 verification (2026-10-15)

All Cargo commands were run without network and with
`export CARGO_TARGET_DIR=/workspace/.cargo-target/me-geo6`:

- `cargo fmt --all -- --check`: passed.
- `/home/box/.cargo/bin/cargo clippy --offline --workspace --all-targets -- -D warnings`:
  passed, without warnings.
- `cargo test --offline --workspace --no-fail-fast`: 556 tests passed,
  0 failed, 0 ignored, across 87 test/doctest targets. Includes the seven
  analytical fixtures and the new middle discrepancy test.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --workspace --no-deps`:
  passed.
- `/home/box/.cargo/bin/cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown`:
  passed.
- `/home/box/.cargo/bin/cargo build --offline --release -p engine-validation --example consolidation_wasm --target wasm32-unknown-unknown`:
  passed.
- `node crates/engine-validation/scripts/run-consolidation-wasm.cjs /workspace/.cargo-target/me-geo6/wasm32-unknown-unknown/release/examples/consolidation_wasm.wasm`:
  seven analytical fixtures and convergence passed, without external imports.
- `bash clean-room/check.sh origin/main`: passed on the full diff,
  including the new file through a temporary index in `/tmp` (the original
  worktree index is mounted read-only in this session).
- `git diff --check` and `git diff --cached --check` with that index: passed.
- The library module still has no panic!, unwrap or expect and is unchanged.
  docs/CLEAN_ROOM.md, .github/**, clean-room/** and vendor/** were not touched.

Native and rustdoc: Cargo/Rust 1.85.1; clippy and WASM: Cargo/Rust 1.99.0 already
installed, as in the initial delivery. No dependencies were downloaded.
