# Clean-room log · ENG-VAL-001 (partial, stacked)

- Ticket / PR: ENG-VAL-001 «Structural benchmark suite V1», partial:
  linear statics, modal, P-Delta and spectrum. Branch stacked on ENG-ANL-004
  (#62), ENG-ANL-002 (#61), ENG-MASS-001 (#55), ENG-LOAD-001 (#51), with
  ENG-ANL-003 (#59) and the linear delivery (#57/#54/#53/#52). No commit or push.
- Author(s): J2 (agent, Grok Bot) · identification requested by the user;
  executing agent Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Role: implementer and author of own analytical derivations.
- Contract implemented: `engine-validation` development crate, one tabulated
  linear test over `StaticLinear`, `element/1` (truss/frame),
  `engine_analysis::results` and `engine_analysis::combination` from LOAD-002;
  three additional tabulated tests over `Modal`, `PDelta` and `Spectrum`.
- Permitted sources used: exclusively the request, own workspace APIs
  and tests and `provenance/TEMPLATE.md`. No network, external sources, books,
  pages, published numerical examples, sources from other engines or oracles.
- Own derivations: the closed-form notes in this log; data in
  [fixture table](../crates/engine-validation/tests/support/table.rs),
  [checks](../crates/engine-validation/tests/analytical.rs) and
  [setup](../crates/engine-validation/tests/support/harness.rs).
- Third-party code copied: none. No new external dependencies;
  exclusively internal `dev-dependencies` via `path`. Documentation library
  without production dependencies, `publish = false`.
- Validation: exclusively analytical, through integration, energy, compatibility,
  equilibrium and superposition. The SI numbers are own artificial data.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

The logs of the three stacked tickets
[ANL-001](ENG-ANL-001.md), [RES-001](ENG-RES-001.md) and
[LOAD-002](ENG-LOAD-002.md) already contain that full declaration, the requested
author and pending reviewer. This delivery retains those declarations;
it does not declare J4 review or merge of the previous PRs.

## Scope and count

**42 linear fixtures** are delivered; each row is a model with parameters and
actions, not a single assertion. One test traverses all rows, checking
unique names, metadata and results. Variants change geometry,
load position, direction, settlement or factors.

| Family | Rows | Data that change |
|---|---:|---|
| Cantilevers: tip, uniform and couple | 5 | L=4/7; P, w or end couple |
| Simply supported | 7 | L=4/7; uniform; a/L=.5, .25, .4, .7 |
| Clamped at both ends | 5 | L=4/7; uniform; a/L=.5, .25, .7 |
| Propped cantilever | 2 | L=4/7 |
| Continuous, two equal spans | 2 | L of each span=4/7 |
| Two bars, triangle, three concurrent bars | 6 | H=0/5 |
| Bar with settlement | 2 | s=.03/−.02 |
| Lateral and vertical portal frame | 4 | h=3/5 |
| Moment release | 2 | L=4/7 |
| Rotation of the entire model | 3 | α=30°, 90°, 137° |
| Superposition and combination | 4 | (α,β)=(1,1),(1.3,−.7),(−.5,2),(0,.5) |
| **Linear total of this delivery** | **42** | |

**ENG-VAL-001 remains partial.** The extension at the end of this note adds 31
fixtures: **73 in total**, exceeds ≥30 and covers modal, P-Delta and spectrum. The
previous 42 rows do not change. Published references with exact citations,
J4 review and execution in a wasm runtime remain pending; detailed below.

## Data and conventions

`Parameters` defaults: E=200 Pa, A=2 m², I=3 m⁴, L=4 m,
h=3 m, P=12 N, w=6 N/m, a/L=.5, H=0, s=.03 m, ángulo=0, factores=(1,1),
r=100000. In the couple fixture, the value P represents 12 N·m.
Overrides for each row are in the Rust table. For trusses L
represents the half-width a, not the inclined length ℓ=√(a²+h²). For portal frames
L is the span b; for the continuous beam it is the length of each span.

D2 in XY; displacements and reactions recovered in six global components:
`[Ux,Uy,Uz,Rx,Ry,Rz]` and `[Fx,Fy,Fz,Mx,My,Mz]`. θ=v′ is positive counterclockwise.
In horizontal beams, v is positive upward and M=EI v″ positive in sagging.
Downward loads −P, −w. Reaction couples at ends i/j are −M(0)
and +M(L), respectively. N positive in tension. No numerical formulas from
a third party are used or expected values adjusted to an engine output.

Observation nodes are placed at x=L/2 or x=a. Under uniform load, the
internal polynomial is quartic but the element with consistent loads obtains
**exact nodal** displacements and rotations: the particular solution with
v=θ=0 at the ends contributes no nodal values. The cubic interpolated deflected shape
between nodes is not used as an interior deflection oracle. Requested
stations are [0,.2,.8,1]; RES-001 adds critical roots where applicable.
M maxima are checked against closed forms, including that of the
propped cantilever outside the requested stations, with its exact location.

## Voladizos

With constant EI and v(0)=θ(0)=0, integrate EI v″=M twice:

```text
Punta: M(x)=−P(L−x)
θ(x)=−P(Lx−x²/2)/EI
v(x)=−P(Lx²/2−x³/6)/EI
v(L)=−PL³/(3EI), θ(L)=−PL²/(2EI)
Ry=P, Rz=PL, Mmin=−PL, Mmax=0.

Uniforme: M(x)=−w(L−x)²/2
θ(x)=−w(L²x−Lx²+x³/3)/(2EI)
v(x)=−w(L²x²/2−Lx³/3+x⁴/12)/(2EI)
v(L)=−wL⁴/(8EI), θ(L)=−wL³/(6EI)
Ry=wL, Rz=wL²/2, Mmin=−wL²/2, Mmax=0.

Par terminal C: M(x)=C
θ(L)=CL/EI, v(L)=CL²/(2EI), Ry=0, Rz=−C.
```

Deflection, rotation, reactions and exact moment range are checked. Beams are
placed with origin (2,3) so that the moment balance includes actual lever arms.

## Simplemente apoyadas

With uniform load, equilibrium gives RA=RB=wL/2 and M=wx(L−x)/2. Integrate with v(0)=v(L)=0:

```text
v(x)=w(Lx³/12−x⁴/24−L³x/24)/EI
θA=−wL³/(24EI), θB=+wL³/(24EI)
v(L/2)=−5wL⁴/(384EI)
M′=w(L−2x)/2=0 => x=L/2, Mmax=wL²/8, Mmin=0.
```

With a point load at a, b=L−a, RA=Pb/L, RB=Pa/L. Writing
`<x−a>=max(0,x−a)` allows integrating both segments at once:

```text
EI v(x)=RA x³/6−P<x−a>³/6+EI θA x
θA=−Pb(L²−b²)/(6EIL)      (de v(L)=0)
θB=+Pa(L²−a²)/(6EIL)
v(a)=−Pa²b²/(3EIL)
M(x)=RA x−P<x−a>
Mmax=M(a)=Pab/L, Mmin=0.
```

The piecewise representation ensures continuity of v, θ and M and a −P jump in
M′. For a=b=L/2 this gives v(a)=−PL³/(48EI), θA=−PL²/(16EI), θB=+PL²/(16EI),
Mmax=PL/4. In asymmetric cases v(a) is checked; it is not claimed to be the
maximum deflection, whose location differs. Both rotations and the
exact maximum moment at the load position are checked.

## Biempotradas

With uniform load, imposing v=θ=0 at 0,L on the polynomial satisfying EI v⁽⁴⁾=−w gives:

```text
v(x)=−w x²(L−x)²/(24EI)
M(x)=−wL²/12+wx(L−x)/2
MA=MB=−wL²/12, RA=RB=wL/2
v(L/2)=−wL⁴/(384EI), Mmax=wL²/24.
```

With a point load, v(0)=θ(0)=0 eliminates the free constants:

```text
EI v(x)=MA x²/2+RA x³/6−P<x−a>³/6
θ(L)=0 => MA L+RA L²/2−Pb²/2=0
v(L)=0 => MA L²/2+RA L³/6−Pb³/6=0
MA=−Pab²/L², MB=−Pa²b/L²
RA=Pb²(3a+b)/L³, RB=Pa²(a+3b)/L³
v(a)=−Pa³b³/(3EIL³)
Mmax=M(a)=2Pa²b²/L³, Mmin=min(MA,MB).
```

At the center MA=MB=−PL/8 and v(a)=−PL³/(192EI). Both moments,
vertical reactions, zero rotations, deflection at the load and extrema of M are checked.
Right Ux is left free and without axial load; it is exactly zero due to EA>0.
Bending constraints remain those of a beam clamped at both ends.

## Voladizo apuntalado

Superpose an upward force RB at the tip on the uniformly loaded cantilever:

```text
v(L)=−wL⁴/(8EI)+RB L³/(3EI)=0 => RB=3wL/8
RA=wL−RB=5wL/8
Rz=wL²/2−RB L=wL²/8
M(x)=−w(L−x)²/2+RB(L−x)
v(x)=−w x²(6L²−4Lx+x²)/(24EI)+RB x²(3L−x)/(6EI)
v(L/2)=−wL⁴/(192EI), θ(L)=wL³/(48EI)
M(0)=−wL²/8, M(L)=0
M′=0 => x=5L/8, Mmax=9wL²/128.
```

The `articulacion-extremo` rows use an element with an actual j=true release,
right end supported in Uy and disconnected external rotation restrained to zero to
eliminate the mechanism without restricting its condensed internal rotation. The bending
solution is that of the propped cantilever: reactions, Mmax and M(L)=0 are checked both
in end action and in section, and zero external reaction couple. The restrained
external rotation is not interpreted as the physical internal rotation of the hinge.

## Dos vanos

Two equal spans of length L, identical EI, identical uniform loads and
vertical supports at 0,L,2L. Symmetry requires θ(L)=0; each half, viewed from the
center, has the propped cantilever conditions. By compatibility from the
previous section, the central reaction contributed by each span is 5wL/8:

```text
R0=R2L=3wL/8, RL=2·5wL/8=5wL/4
M(L)=−wL²/8
Primer vano: M(x)=3wLx/8−wx²/2
θ(0)=−wL³/(48EI), θ(2L)=+wL³/(48EI)
v(L/2)=v(3L/2)=−wL⁴/(192EI)
M′=0 => x=3L/8 y x=13L/8, Mmax=9wL²/128.
```

The initial rotation is obtained by integrating M and imposing v(0)=v(L)=0. The sum of
reactions is 2wL; the result is not used as a published number.

## Armaduras

Bases (−a,0),(a,0), apex (0,h); ℓ=√(a²+h²), c=a/ℓ, s=h/ℓ.
For restrained bases, the energy of two bars is
`U=EA[(c ux+s uy)²+(−c ux+s uy)²]/(2ℓ)`.
Differentiating `U−Hux+Puy` gives:

```text
Kx=2EAa²/ℓ³, Ky=2EAh²/ℓ³
ux=Hℓ³/(2EAa²), uy=−Pℓ³/(2EAh²)
N1=Hℓ/(2a)−Pℓ/(2h), N2=−Hℓ/(2a)−Pℓ/(2h)
R1=(−N1c,−N1s), R2=(+N2c,−N2s).
```

Statically determinate triangle: add a horizontal bar of length 2a and release Ux of
the right base. Apex equilibrium preserves N1,N2; right-base equilibrium
gives `Nb=−N2c=H/2+Pa/(2h)`. Reactions:
`Rx1=−H`, `Ry1=P/2−Hh/(2a)`, `Ry2=P/2+Hh/(2a)`.
To obtain displacements, differentiate the complementary energy
`U*=Σ Ni² ℓi/(2EA)` with respect to H and the vertical load +Y=−P
(equivalently unit virtual work):

```text
ux=Hℓ³/(2EAa²)+Nb a/(EA)
uy=−Pℓ³/(2EAh²)−Nb a²/(EAh)
ux_base_derecha=Nb·2a/(EA).
```

Simple statically indeterminate truss: keep both bases restrained and add a
third bar from (0,0) to the apex with a restrained base. Its energy `EA uy²/(2h)` is
added to the previous U, ensuring compatibility of all three bars:

```text
Kx=2EAa²/ℓ³, Ky=2EAh²/ℓ³+EA/h
ux=H/Kx, uy=−P/Ky
N1=(EA/ℓ)(c ux+s uy), N2=(EA/ℓ)(−c ux+s uy), N3=EA uy/h
R3y=−N3.
```

Forces are not deduced solely by equilibrium in the statically indeterminate case. Both
displacements, N of all bars at all stations, reactions
and the five zero non-axial resultants are checked.

## Asentamiento

Axial bar with u(0)=0, u(L)=s and no distributed load. From N′=0 and u′=N/(EA),
`N=EA s/L`, `u(L/2)=s/2`, reactions `R0=−N`, `RL=+N`.
Two elements in series are used to retain a free interior axial DOF;
transverse DOFs are restrained. Settlements of both signs,
forces and prescribed displacement are checked. The identity combination uses factor
1; the same settlement is not added multiple times as if it were a homogeneous load.

## Portal lateral

Span b, height h, two clamped columns. H/2 at each upper corner.
In the limit of a rigid beam and inextensible columns, the columns have
translation Δ and zero upper rotation. The energy of each column is
`6EI Δ²/h³`; differentiating total energy minus HΔ gives
`K=2·12EI/h³`, `Δ=Hh³/(24EI)`, `Rx1=Rx2=−H/2`.

Only increasing beam EI would not prevent its rotation due to axial deformation of
the columns. Therefore the rows simultaneously increase beam EI and EA
of columns/beam by r=100000, leaving column EI intact.
The **finite** closed form is derived before taking the limit. Let J=EI_col,
B=EI_dintel and C=EA_col; symmetry imposes translations Δ, equal rotations θ and
vertical displacements v,−v. The beam does not elongate. Summing energies:

```text
Π=12JΔ²/h³+12JΔθ/h²+(4J/h+6B/b)θ²
 +(C/h+24B/b³)v²+24Bvθ/b²−HΔ
β=(12B/b²)/(C/h+24B/b³), v=−βθ
D=4J/h+6B/b−12Bβ/b²
θ=−6JΔ/(h²D)
Δ=H/(24J/h³−72J²/(h⁴D)).
```

The table uses B=rJ, C=rEA. Defining q=3J/(hD), the relative error against the
limit is `q/(1−q)`: <1.80e-5 for h=3 and <1.53e-5 for h=5, b=4 and the above
SI data. Limit Δ is checked with a relative margin of 5e-5 and
Δ, θ, v of the finite solution are also checked with an independent numerical tolerance.

## Portal vertical

Columns and beam with equal E,A,I and uniform load −w only on the beam.
Symmetry: upper corners `(t,v,θ)` and `(−t,v,−θ)`. The reduced energy,
including work of the beam's consistent loads, is (constant omitted):

```text
Π=(12EI/h³+2EA/b)t²+12EI tθ/h²+(4EI/h+2EI/b)θ²
  +EA v²/h+wb v+wb²θ/6
γ=(6EI/h²)/(12EI/h³+2EA/b)
t=−γθ
D=4EI/h+2EI/b−6EIγ/h²
θ=−(wb²/12)/D
v=−wbh/(2EA), Ry1=Ry2=wb/2.
```

These are the three equations ∂Π/∂t=∂Π/∂θ=∂Π/∂v=0. At the beam center, add
the cubic rotation interpolation and the particular solution clamped at both ends:
`vmedio=v+bθ/4−wb⁴/(384EI)` and `θmedio=0`. Displacements,
rotations, equal reactions and opposite horizontal reactions are checked without a stiffness approximation.

## Rotacion

Rotate all cantilever points about the clamped support (2,3) and its
end force by Q(α), with α=30°,90°,137°. Since the clamped support
constraints are complete, they remain identical when rotated. Elastic
energy and external work are invariant under orthogonal change of axes:

```text
Q=[[cosα,−sinα],[sinα,cosα]]
u_global=Q(0,−PL³/(3EI))
R_global=Q(0,P)
θ=−PL²/(2EI), Rz=PL, M(x)=−P(L−x).
```

Each global component is compared with the rotated solution, the transverse
projection with the same original deflection, and invariant rotation and moments.
Equilibrium about the global origin is checked with the rotated positions.

## Superposicion

Homogeneous supports: `K uP=FP`, `K uw=Fw` implies
`K(αuP+βuw)=αFP+βFw`. Reactions `R=Ku−F` and action recovery are
linear in u and F. In the cantilever:

```text
v(L)=−αPL³/(3EI)−βwL⁴/(8EI)
θ(L)=−αPL²/(2EI)−βwL³/(6EI)
Ry=αP+βwL, Rz=αPL+βwL²/2.
```

Each of four rows solves P, w and D=αP+βw through `StaticLinear`, recovers
all results and uses `combine` from LOAD-002. It compares the sum against D in
displacements, reactions, actions and common stations; it also compares
deflection, rotation and reactions against the above closed forms. The combination
with opposite factors may have new roots in D: its continuous extrema are not
required to coincide with the discrete extrema of `combine`, nor is a
root search invented for LOAD-002. Each fixture also passes its
results through an identity combination, checking the recovered fields.

## Equilibrium

Each solved case checks the analysis free/global residual and calculates an
independent balance with the recovered results:

```text
ΣFx=ΣRx+ΣFx_nodal+Σqx ℓ=0
ΣFy=ΣRy+ΣFy_nodal+Σqy ℓ=0
ΣMz=Σ[Rz+x Ry−y Rx]+Σ[Mz_nodal+x Fy−y Fx]
    +Σ[mz ℓ+xcentro qy ℓ−ycentro qx ℓ]=0.
```

Original physical loads are used; the assembled equivalent vector is
not reused as an equilibrium oracle. The balance includes force lever arms
and does not require the sum of couples without lever arms to be zero. It applies to the 42
fixtures, including P,w,D cases and settlements.

## Tolerances

Each row explicitly stores `Tolerance { absolute, relative,
justification }`. Rule: `|obtenido−esperado| ≤ absoluto+relativo·|esperado|`;
both values must be finite. The obtained magnitude is not used to increase
the threshold. Quantities are SI according to the field (m, rad, N, N·m); positions
are dimensionless. For ordinary models, absoluto=1e-9 and relativo=1e-10:
a few nodes, moderate parameters and closed-form nodal solutions justify
a conservative margin against f64 elimination and recovery. Zeros,
residuals and equilibrium are compared with 1e-9 absolute.

For the lateral portal frame: absoluto=1e-8 due to stiffness contrast 1e5 and cancellation in
recovery; relativo=5e-5 only to approximate the rigid limit. The finite
error is analytically bounded above. The additional check of the
finite solution uses absoluto=1e-8 and relativo=1e-10, justified by that
conditioning. In native execution the moment residual of the h=5 portal frame was 2.45e-9;
it is within the documented absolute margin. Residuals/zeros receive no
relative approximation margin, because their expected value is zero.

## Session verification

Local verification on 2026-10-11, without network (`CARGO_NET_OFFLINE=true`):

- `cargo fmt` and `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK; 241 native tests, including doctests.
  The only new test traverses the 42 fixtures and indicates the name on failure.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --workspace --all-targets --target wasm32-unknown-unknown`: OK;
  includes the runner, table and test oracles, without I/O or threads.
- `bash clean-room/check.sh origin/main`: OK. Additional check including
  all new files in a temporary index and objects in `/tmp`, with
  original repository objects read-only: OK. The real index remained intact.
- `git diff --check`: OK for already tracked changes; review with new
  files through the same temporary index: OK.

Native tests and documentation: system rustc/cargo 1.85.1. Available Clippy
0.1.99. For wasm, `PATH=/home/box/.cargo/bin:$PATH
RUSTUP_TOOLCHAIN=stable` was used, with toolchain and target already installed (rustc 1.99.0), without
downloading components. Compiling for wasm does not mean running a wasm runtime.
The real Git index was not modified; no commit or push. This verification
corresponds to the previous linear delivery; the extension and its verification are
recorded below. J4 review and execution in a wasm runtime pending.

## Ampliación dinámica y 2.º orden

### Extension log

- Ticket / PR: ENG-VAL-001, extension on ENG-ANL-002 (#61, modal),
  ENG-ANL-003 (#59, P-Delta) and ENG-ANL-004 (#62, spectrum), open chain.
- Author(s): J2 (agent, Grok Bot), requested label; executing agent Codex
  based on GPT-6 · 2026-10-11 (America/Guayaquil).
- Role: implementer and author of own derivations, without delegation.
- Contract implemented: validation fixtures through public APIs `Modal::run`,
  `PDelta::run`, `Spectrum::run`, spectral tables and combinations, elements
  `EulerFrame2D` and `LinearTruss`; without changing the analyses' implementation.
- Permitted sources used: user request, own workspace APIs/tests,
  `provenance/TEMPLATE.md` and `docs/CLEAN_ROOM.md`. No network,
  external bibliography, books, pages, published numerical examples or oracles.
- Own derivations: equations D1–D12 below;
  [data](../crates/engine-validation/tests/support/dynamic_table.rs),
  [setup](../crates/engine-validation/tests/support/dynamic_harness.rs) and
  [oracles](../crates/engine-validation/tests/dynamic.rs).
- Third-party code copied: none; no new dependencies.
- Validation: exclusively analytical; own artificial SI data.
- Comparison with oracle (sibling repo, informative): no.
- Declaration for ENG-ANL-002, ENG-ANL-003 and ENG-ANL-004 in this session:
  "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

Previous declarations are retained; a record of this session is added to
ANL-002/004 and a record of ANL-003 validation is created, absent in this
checkout. This does not attribute retroactive authorship of its implementation to this session.

### Count and scope

| New group | Rows | Complete problems |
|---|---:|---|
| Modal | 12 | Shear N=1..5; 2 variable stories; free-restrained axial chain; supported beam and cantilever with 4/12 meshes; rotated cantilever |
| P-Delta | 10 | Column r=0,.2,.5,.8,.9,−.5,1.05; frame r=.05,.1; iteration series and limit |
| Spectrum | 9 | SRSS 2 stories; modal displacement with another Sa; linear/log-log interpolations; separated/close/coincident CQC; two directional combinations |
| **New** | **31** | One test traverses each group and names fixture, field, source/citation and tolerance on failure |
| Existing linear | 42 | Original table and test intact |
| **Total ENG-VAL-001** | **73** | ≥30 total and ≥24 new |

Internal assertions are not counted as fixtures. Orthogonality, mass
accumulation and participation are checked within each shear/chain problem.
All new `source/citation` refer to this derivation, without book citations.
Results calculated by the engine are not converted into expected values.

### Modal: derivation D1–D4

**D1. Uniform shear.** From the energy
`U=k/2 Σ(q_j−q_{j−1})²`, `T=m/2 Σq̇_j²`, `q_0=0`, we obtain
`k(2φ_j−φ_{j−1}−φ_{j+1})=mω²φ_j`. The free roof is equivalent to
`φ_{N+1}=φ_N`. Proposing `φ_j=sin(jθ)` gives `ω²=4k/m sin²(θ/2)` and
`sin((N+1)θ)=sin(Nθ)`, that is

```text
θ_n=(2n−1)π/(2N+1), n=1..N
ω_n=2√(k/m) sin((2n−1)π/[2(2N+1)])                    (D1)
φ_j=sin(jθ_n)/√[m Σ_l sin²(lθ_n)].
```

The five rows use k=23 N/m, m=4 kg, N=1..5. The free-restrained chain uses
three axial bars of length 1, EA=31, zero density and point masses
of 6 kg. Its axial energy is exactly D1 with k=EA/ℓ; one end restrained,
the other free and transverse displacements restrained. Thus it verifies
actual elements and point masses with the same physical oracle.

**D2. Two variable stories and manual Γ.** With k1=19, k2=7 N/m and
m1=2, m2=5 kg:

```text
K=[[k1+k2,−k2],[−k2,k2]], M=diag(m1,m2)
det(K−λM)=m1m2 λ²−[m2(k1+k2)+m1k2]λ+k1k2=0
10λ²−144λ+133=0 ⇒ λ±=(144±√15416)/20
v=(1,t), t=(k1+k2−m1λ)/k2
s=√(m1+m2t²), φ=v/s, Γ=(m1+m2t)/s, M*=Γ².             (D2)
```

For the two equal stories of D1, `λ±=(k/m)(3±√5)/2`,
`t=(1±√5)/2`, `s=√[m(1+t²)]`, `Γ=m(1+t)/s` and
`M*±=m(1±2/√5)`. Their sum is `2m`. The test adapts only the arbitrary
sign of the eigenvector when comparing Γ; the oracle for λ, φ and M* is manual.

**D3. Orthogonality and completeness.** Subtracting `φᵢᵀKφⱼ=λⱼφᵢᵀMφⱼ` and its
transpose gives `(λᵢ−λⱼ)φᵢᵀMφⱼ=0`; with distinct modes and mass normalization,
`ΦᵀMΦ=I` and `ΦᵀKΦ=diag(λ)`. Expanding free rigid translation `r` in
the complete basis, `r=ΣΓ_n φ_n`, gives

```text
Γ_n=φ_nᵀMr, ΣM*_n=ΣΓ_n²=rᵀMr=Σm_j.                 (D3)
```

Products with assembled M/K are calculated, including all pairs;
expected results are I and λ, derived above. Per-mode accumulations
and the free/total mass report are compared with Σm_j, without using
reported mass as an oracle. In beams with mass at supports, completeness
equality corresponds to free mass, not necessarily total physical mass;
therefore exact equality to the total is required for point-mass models.

**D4. Beams and rotation.** Separating variables in `ρA v̈+EI v''''=0` gives
`v''''=β⁴v` and `ω=β²√(EI/ρA)/L²` (dimensionless β). The spatial solution
is a combination of cos, sin, cosh and sinh. For a supported beam, `v=v''=0` at both
ends ⇒ `β=nπ`. For a cantilever, `v=v'=0` at the base and `v''=v'''=0` at the
tip ⇒ determinant `cosβ coshβ+1=0`. The test obtains the first three
roots by 55 bisections in `(jπ+.1,(j+1)π)`; it uses no published numbers.
Here E=ρ=A=L=1, I=1e−4 ⇒ `ω=.01β²`, first three bending modes.
Consistent mass and `EulerFrame2D` Hermite elements with 4/12
elements are used: relative margins 1.9% / 0.03%, verified for these rows,
and error reduction >10 times upon refinement. These are margins for these meshes,
not a universal bound for any beam or mode.

For α=.83 rad, `Q=[[cosα,−sinα],[sinα,cosα]]` rotates coordinates, shapes and
excitation direction. Simultaneous orthogonal congruence of K/M preserves
λ and `φᵀMr`; frequencies, |Γ|, M* and each `Qφ` component are compared with
a numerical margin independent of the discretization margin.

### P-Delta: derivation D5–D8

**D5. Compressed cantilever.** The functional
`Π=1/2 ∫[EI(v'')²−P(v')²] dx−Hv(L)` gives
`EI v''''+P v''=0`, `v(0)=v'(0)=0`, `v''(L)=0` and
`EI v'''(L)+P v'(L)=−H`. Integrating the trigonometric solution gives

```text
a=√(P/EI), z=aL, Pcr=π²EI/(4L²), r=P/Pcr
v(L)=H/P [tan(aL)/a−L]
v0=HL³/(3EI), v/v0=3(tan z−z)/z³, z=π√r/2.          (D5)
```

The pole of tan at z=π/2 corresponds to instability. Its coefficient
with respect to `1/(1−r)` is `96/π⁴≈.9855`; therefore that amplification is an
approximation even with an infinite mesh. The rows use EI=600 N·m²,
L=4 m, H=.1 N and r=.2,.5,.8,.9. The approximate amplification is compared
with 1.6% relative (model error), and the D5 solution with 2e−6 relative
for 16 elements (mesh error). An additional solve with 4 elements
must have error >10 times greater. r=1.05 returns `Unstable`, one negative
direction by inertia and no lateral displacement or history.

**D6. Tension and zero.** Substituting P=−T in D5 gives
`v=H/T [L−tanh(aL)/a]`, `a=√(T/EI)`; for r=−.5,
`v/v0=3(z−tanh z)/z³<1`. Greater stiffness and the exact solution are checked
with the same mesh margin. For P=0, the expansion
`tan z=z+z³/3+…` recovers `v0`; the entire lateral vector is also compared
with `StaticLinear(H)` and the final convergence of each analysis is checked.

**D7. One-story frame.** n=2 clamped columns, h=4, rigid beam,
zero upper rotation. For each column, the linear cubic shape
`v=Δ(3s²−2s³)`, s=x/h, has energy `6EIΔ²/h³`.
The story stiffness is `Kpiso=12nEI/h³`. In the chord idealization,
the work of total gravity P produces `−PΔ²/(2h)`:

```text
(Kpiso−P/h)Δ=H
Pcr_piso=Kpiso h=12EI_c n/h²
Δ/Δ0≈1/(1−P/Pcr_piso), Δ0=Hh³/(12nEI_c).             (D7)
```

The symbol ≈ is essential: the geometric energy of the actual cubic field is
`−(P/n)/2 ∫(v')²dx=−(P/n)·3Δ²/(5h)`, a factor 6/5 relative to the chord.
The analysis with distributed Kg cannot have exactly the Pcr of that
idealization. Solving D5 with `v'(0)=v'(h)=0` and terminal displacement Δ
gives the continuous comparison for each guided column:

```text
K_col(P/n)=EI a³/[2tan(z/2)−z], a=√[(P/n)/EI], z=ah
Δ/Δ0=12[2tan(z/2)−z]/z³, z=√(12r).
```

The continuous physical pole is `Pcr=π²nEI/h²`, distinct from `12nEI/h²`.
The r=.05,.1 rows compare D7 with margin 2.6%, sufficient for the difference
in idealization at those two points. They also compare with the continuous guided
solution at 1e−4 relative. 12 elements per column are used, column EA and
beam EA/EI multiplied by 1e5; the continuous margin includes that
finite flexibility and mesh error. Drift of both upper nodes
and their symmetry are checked. Chord Pcr is not presented as exact Euler Pcr.

**D8. Iteration convergence.** Scalar model with k=80 N/m,
gravity −rk, r=.3, lateral load F=8 N and producer `kg=k(u_g2−u_g1)=−rk`.
The iteration toward the invariant point from u0=0 satisfies `u_i=F/k+r u_{i−1}`:

```text
u_i=F/[k(1−r)](1−r^i), residual_relativo=r^i
cambio_relativo=(F/k)r^(i−1), escala=1 y |u_i|<1.     (D8)
```

The row checks each displacement, residual, change and the first step
that meets both thresholds at 1e−7, with limit 32. With limit 3 it requires
`NotConverged` without retry. Columns and frames also require
the last residual and change to be within the declared tolerance 1e−10.

### Spectrum: derivation D9–D12

**D9. SRSS and maximum modal coordinate.** Multiplying the dynamic equation
by a mass-normalized mode gives `q̈_n+ω_n²q_n=−Γ_n a_g` (the conventional
sign in the signed amplitude is omitted according to the API). The table's pseudoacceleration
produces `qmax_n=Γ_n Sa/ω_n²`; the equivalent force is
`Mφ_nΓ_nSa`. Its projected sum is `Vb,n=Γ_n²Sa=M*_nSa`. By independence:

```text
Vb²=Σ_n(M*_n Sa)², u_n=φ_nΓ_nSa/ω_n².               (D9)
```

For the two stories of D2, k=23, m=4 and Sa=2.7 / 5.1 m/s²:
`M*±=m(1±2/√5)`, `Vb²=(18/5)(mSa)²`. Each row checks Sa, maximum modal
coordinate, shear of each mode, each signed physical displacement and
SRSS envelopes against D2/D9, without obtaining expected Γ from the modal result.

**D10. Tables.** Linear between `(1,3),(5,11)`:
`Sa(T)=3+(T−1)(11−3)/(5−1)` ⇒ `Sa(2.5)=6`.
Log-log between `(1,3),(4,48)`:
`ln Sa=ln3+ln(T/1) ln(48/3)/ln(4/1)` ⇒ `Sa=3T²`,
`Sa(2)=12`. Both endpoints are also verified. Log-log interpolation
is not given the expected value of linear interpolation.

**D11. CQC.** The WDKB 1981 equation is the formula supplied by the
request; the attribution identifies that equation, without claiming access to the paper,
its code or a published example. In the test it is expanded directly:

```text
r=ω_j/ω_i
ρ_ij=8√(ζ_iζ_j)(ζ_i+rζ_j)r^(3/2) /
 [(1−r²)²+4ζ_iζ_jr(1+r²)+4(ζ_i²+ζ_j²)r²]
R_CQC=√(A²+B²+2ρ_ijAB).                              (D11)
```

The close row uses ω=(10,10.2), ζ=(.05,.07), A=3, B=−4; the sign of
AB distinguishes premature magnitude combination. ρ is compared with the
manual expansion, its symmetry and the combination. For equal modes and equal ζ,
r=1 ⇒ numerador=denominador=16ζ² ⇒ `ρ_ii=1`, `R=|A+B|`.
For ω=(1,100), ζ=.05, `ρ<2.1e−5`: CQC≈SRSS with relative margin 2e−5;
it is not exact identity with positive damping and finite separation.
With those distinct frequencies and ζ=0, the numerator is zero, therefore
**CQC=SRSS exactly** (within f64 margin). Both cases are checked.

**D12. Directional factors.** Isotropic point mass of 2 kg,
independent springs kx=18, ky=50 N/m connected to a restrained base: ωx=3, ωy=5.
Exciting x/y separately gives `Vx=2Sa,x`, `Vy=2Sa,y`, `ux=Sa,x/9`,
`uy=Sa,y/25`. For user factors ax,ay:

```text
V_ponderado=axVx+ayVy
V_SRSS=√[(axVx)²+(ayVy)²].                            (D12)
```

Two rows run the full `Spectrum::run` with Sa=(2,5), factores=(1.4,.35)
and Sa=(3,4), factores=(.65,1.2), respectively; they verify both base shears and
the two combined displacement components. There are no code-prescribed factors.

### Tolerances, portability and pending criteria

The rule remains `|a−e|≤absoluto+relativo·|e|`, with mandatory finite values;
each row has a justification. Algebraic comparisons use 1e−9 absolute
and relative; orthogonality, rotation and linear comparison use 2e−9 absolute
and 2e−8 relative for f64 accumulation/conditioning. Mesh,
idealization and CQC limit margins are separate and declared above. A tolerance is not
increased based on the obtained value to make a failure pass.

Fixtures are constant tables and tests operate only in memory. No
I/O, network, threads, processes or clock. `Arc` is a producer API
requirement; it introduces no threads. The documentation library adds no code that
may enter panic/unwrap/expect; helpers return errors from the
`ENG-E-0012` catalog and propagate public errors. Only the test boundary
converts a failure into panic with name, field and provenance.

Status of VAL-001 thresholds:

- **COVERED:** ≥30 total fixtures, ≥24 new and the requested analytical modal,
  P-Delta and spectral coverage; 73 total / 31 new.
- **COVERED:** tabulated data, own source/citation, declared tolerances,
  failure with name and test compilation for wasm without new dependencies.
- **PENDING:** book/published examples with edition, page and exact citation:
  not verifiable without access; outside J2 scope. No citations are invented.
- **PENDING:** test execution in a wasm runtime; compiling is not equivalent
  to running them. Nor is coverage of large deformation,
  plasticity or time histories declared enabled based on elastic P-Delta/spectrum.
- **Clean-room reviewer: PENDING (J4)**. Full VAL-001 is not declared complete.

### Extension verification (2026-10-11)

All Cargo commands were run with `CARGO_NET_OFFLINE=true`, without
downloading dependencies or components:

- `cargo fmt` and `cargo fmt --check`: **OK**.
- `cargo clippy --all-targets -- -D warnings`: **OK**.
- `cargo test --workspace`: **OK**, **294 native tests**, including
  doctests; the original linear test traverses 42 fixtures and the three new tests
  traverse 12/10/9 rows. No failures or existing fixtures modified.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: **OK**.
- `cargo check --workspace --target wasm32-unknown-unknown`: **OK**.
- `cargo check --workspace --all-targets --target wasm32-unknown-unknown`:
  **OK**, explicitly includes the new tables, helpers and oracles.
- `bash clean-room/check.sh origin/main`: **OK**. Additional check with
  all new files recognized through a temporary index and objects
  in `/tmp`: **OK**, also `git diff --cached --check` in that index.
- `git diff --check`: **OK**. Cargo.toml, Cargo.lock, original linear
  table/harness/test and analysis/element libraries remain intact.

Native tests/documentation: system rustc/cargo 1.85.1; Clippy 0.1.99.
Wasm: `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`, rustc 1.99.0
with target already installed. No wasm runtime was run. The real Git index
remains intact; **no commit or push**, no network or new dependencies.
Clean-room reviewer: PENDING (J4).
