# Clean-room log · ENG-ELM-002

- Ticket / PR: ENG-ELM-002, «Elastic 2D beam-column with releases»;
  existing branch `j2/eng-elm-002-frame2d`, created from main with ENG-ELM-001
  merged. File delivery, without commit, push or PR.
- Author(s): Codex, agent based on GPT-6 · 2026-10-11
  (America/Guayaquil). No delegation to other agents.
- Role: implementer; own specification, derivations and analytical
  fixtures. No oracle operator or similarity auditor activity.
- Contract implemented: `element/1` version 1.0, kind
  `frame2d.euler_elastic`, and `engine_contracts::ElementFactory`.
  Crate/method 0.1.0; independent method and contract versions.
- Permitted sources used: internal contracts and documents listed
  below; own derivations of energy, equilibrium and polynomial integration.
  Exact bibliography for strength of materials/classical structural
  analysis (author, title, edition, chapter/section/equation): **PENDING**.
  No precise external source was consulted: no pages,
  equations or examples are attributed to a book without having verified them.
- Own derivations: [note in this log](#derivaciones-propias),
  [fixtures](../crates/engine-elements-frame/tests/fixtures/mod.rs),
  [statics and buckling](../crates/engine-elements-frame/tests/analytical.rs),
  [invariants](../crates/engine-elements-frame/tests/invariants.rs).
- Third-party code copied: none. Own code and internal
  dependencies by path; no new external packages or book listings.
- Validation: exclusively analytical, through closed-form solutions and energy
  integrals. Each fixture declares `source`, `citation` and tolerances;
  exact bibliography **PENDING**, with a link to the own derivations.
- Comparison with oracle (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: J4, **PENDING**.

## Internal documents consulted

Before implementation, `/workspace/motor-rust/04_TICKETS.md` was read
(ENG-ELM-002), `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`
(§4.4 element/1), `docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`,
`provenance/TEMPLATE.md`, `provenance/ENG-ELM-001.md`, and the structure,
implementation and tests of `engine-elements-truss`.
The traits, types, examples and conformance kit of
`engine-contracts`; uniaxial law and elastic section of `engine-laws`;
`engine-core` error catalog; `engine-linalg` matrices/backend;
map, contributions, constraints, system and reactions of `engine-assembly` were consulted.
No applicable AGENTS.md was found. No external sources,
other engine code or external results were consulted; the network was not used.

The approved convention prevails: D2 is in horizontal X-Y with Z=0,
local x from i to j, local y=(-s,c), local z=+Z. In the frame fixture,
«height» is a dimension in Y within this mathematical plane; it does not redefine
Y as vertical. 2D elevation and its gravity remain pending.

## Derivaciones propias

### Geometry, interpolation and stiffness

With r=x_j−x_i, L=hypot(r_x,r_y), c=r_x/L, s=r_y/L, each global→local
displacement block is [[c,s,0],[-s,c,0],[0,0,1]].
We denote the diagonal block of both nodes by T. f_global=Tᵀf_local
and K_global=TᵀK_local T are calculated; the same congruence applies to mass and Kg.

Local order d=[u_i,v_i,theta_i,u_j,v_j,theta_j], theta=v′. Let t=x/L.
u=(1−t)u_i+t u_j. For v the cubic shapes interpolating
value and derivative at the ends are used:

```text
h1 = 1−3t²+2t³       h2 = L(t−2t²+t³)
h3 = 3t²−2t³         h4 = L(−t²+t³)
v  = h1 v_i + h2 theta_i + h3 v_j + h4 theta_j
```

Energy is ½∫[EA u′²+EI v″²]dx. N=EA(u_j−u_i)/L, positive in
tension. Axial force is obtained through `engine-laws::uniaxial::Elastic`,
without implicitly substituting one law for another. I is Iz, inertia about
local z; EI matches the D2 bending stiffness of the repo's elastic
section. G, Iy or J are not fabricated to construct an unnecessary 3D section.

The integral of the second derivatives of h gives, in [v_i,theta_i,v_j,theta_j]:

```text
Kb = (EI/L³) [[12,   6L, −12,   6L],
              [6L, 4L², −6L,  2L²],
              [−12,−6L,  12,  −6L],
              [6L, 2L², −6L,  4L²]]
Ka = (EA/L) [[1,−1], [−1,1]] en [u_i,u_j].
```

The tangent is constant, f_int=Kc u, without loads or Kg secretly added.
Stiffness includes no shear deformation, large rotations, geometry
update or history. The three D2 rigid-body shapes (X translation,
Y translation and u=(-Y,X), theta=1) have u′=v″=0; K u_rígido=0.

### Releases and rotation recovery

Dividing d into retained components a and released components r, internal
rotation equilibrium requires K_ra d_a+K_rr d_r=f_r. Therefore:

```text
d_r = −K_rr⁻¹ K_ra d_a + K_rr⁻¹ f_r
d = H d_nodal + d_carga
Kc = Hᵀ K H = Kaa − Kar Krr⁻¹ Kra (embebido en 6×6)
fc = Hᵀ f    = fa − Kar Krr⁻¹ fr
```

Columns corresponding to released nodal rotations are zero; member
rotations are internal variables distinct from those nodal rotations.
The inverse is derived analytically, no additional solver is implemented:
Krr=4EI/L for one release; with two it is (EI/L)[[4,2],[2,4]],
whose inverse is (L/EI)[[1/3,−1/6],[−1/6,1/3]].

- Only i: theta_i=3(v_j−v_i)/(2L)−theta_j/2+f_theta_i L/(4EI).
- Only j: theta_j=3(v_j−v_i)/(2L)−theta_i/2+f_theta_j L/(4EI).
- Both: each unloaded rotation is (v_j−v_i)/L, plus the components
  Krr⁻¹[f_theta_i,f_theta_j].

A single release leaves bending energy
3EI/(2L³)(v_j−v_i−L theta_fijo)², of rank one.
With both ends released the curvature of the unloaded interpolation is
zero and **Kflexión=0 exactly**. This analytical Schur is constructed to
avoid artificial stiffness due to numerical cancellation. This member is valid
as a pinned element; disconnected nodal rotations must be removed,
constrained or receive contributions from other members. If left free without contributions,
the backend reports mechanism ENG-E-0006. Matrices are not regularized.

`local_displacements` returns all six local member displacements,
including released rotations and all supplied loads. `recover` uses
those rotations and full K to obtain K d−f. Released moments are set
to zero after solving their equilibrium, eliminating only rounding.
It is checked against independent condensation with `ReferenceBackend`:
Kc, fc, masses, Kg and recovered actions for the three releases.

### Mass

Kinetic energy is ½rho A∫[u̇²+v̇²]dx, with linear u and Hermite v.
rho I is not added: the following rotational inertias come exclusively
from interpolated transverse velocity, not from section rotary inertia.
With m=rho A L:

```text
Ma = (m/6) [[2,1],[1,2]]
Mb = (m/420) [[156,   22L,  54,  −13L],
               [22L,  4L², 13L,  −3L²],
               [54,   13L, 156,  −22L],
               [−13L,−3L²,−22L,   4L²]]
M_lumped = (m/2) diag(1,1,0,1,1,0)
Mc = Hᵀ M H
```

Static mass reduction preserves kinetic energy in the condensed
elastic subspace (Guyan reduction); no Schur of M is calculated,
which would imply independent inertial equilibrium and eliminate different physics.
It is a dynamic approximation: it does not solve internal rotations with inertia at finite
frequency. The load-dependent affine part is excluded from the velocity
transformation, appropriate for small-oscillation mass under constant load.
For two releases, transverse shapes become linear and condensed Mb
in [v_i,v_j] is (m/6)[[2,1],[1,2]].
Physical mass is verified by vᵀMc v=m for a uniform unit
translation in **each** global direction; summing trace or all axes is not
physical mass. rho=0 returns None, validating geometry first.

### Beam-column geometric stiffness

An infinitesimal transverse slope produces additional elongation
½∫v′²dx. The work of the reference axial force N has second variation
N∫δv′²dx. Integrating the first derivatives of the four shapes h:

```text
Kgb = (N/(30L)) [[36,   3L, −36,   3L],
                 [3L, 4L², −3L,  −L²],
                 [−36,−3L,  36,  −3L],
                 [3L, −L², −3L,  4L²]]
```

The rest of local Kg is zero. It is symmetric; tension stabilizes, compression
destabilizes. N comes from the element's u_e, not from an external parameter or
segment loads (the geometric_stiffness contract receives no loads).
A variable axial field from q_x is not inferred: constant N compatible
with total axial displacement is used. Accumulation order is constant.

For releases **Kgc=HᵀKgH** is used, with the elastic stiffness
transformation. Indeed, for A(t)=K+t Kg, energy stationary with respect to
d_r has derivative at t=0 equal to ½d_aᵀHᵀKgH d_a: the variation of
internal rotations cancels by stationarity. It is the first variation of the
Schur of K+tKg, consistent with this initial-stress formulation.
It is **not** the Schur of separate Kg (singular at N=0) or the exact
N-dependent condensation of K+Kg at finite load. The latter would require a
joint evaluation of tangent and loads, outside the current linear element.
With two releases transverse Kgc is (N/L)[[1,−1],[-1,1]], since v is linear.

The matrix test integrates polynomials independently, checking
∫h″ᵀh″, ∫hᵀh and ∫h′ᵀh′ against K, M and Kg. The precise beam-column
bibliography for this own derivation remains **PENDING**.

### Uniform loads and forces at stations

MemberLoad supplies uniform global forces N/m and uniform moments N·m/m.
XY forces are rotated to local qx,qy; mz retains its sign. Nonzero qz,mx,my,
even arbitrarily small, are rejected with ENG-E-0013. No loads are
omitted. For theta=v′, external work is ∫(qx u+qy v+mz v′)dx:

```text
f = [qxL/2, qyL/2−mz, qyL²/12, qxL/2, qyL/2+mz, −qyL²/12]
f_empotramiento = −f; con liberaciones se usa −fc.
```

`self_weight(g)` constructs q_global=rho A g with explicit g in m/s²,
rejecting nonzero g_z. It does not assume gravity from density or the Y axis.
In horizontal D2 real weight toward Z has no representation in this
element; the unsupported capability is reported. The convention is unchanged.
Interior point loads are not in MemberLoad: they are modeled by subdividing
the segment and applying a nodal load, as in the beam fixture clamped at both ends.

Summing the intensities of all loads, let p_i be the three local end i
actions obtained from K d−f. Equilibrium of segment [0,x] gives:

```text
N(x) = −p_i,x − qx x
V(x) = −p_i,y − qy x
M(x) = −p_i,theta + p_i,y x + qy x²/2 − mz x
N′=−qx, V′=−qy, M′=−V−mz.
```

At x=0 they are the opposites of the actions on negative face i; at x=L they match
the positive face j actions. [N,V,0,0,0,M] is delivered at x/L
stations preserving order and duplicates. This recovery includes the
parabolic field due to qy even when all nodal displacements are zero.
An interior cubic deflection is not published as an exact solution under uniform
load: to check deflections, subdivision is performed and interior nodes are evaluated.

### Closed-form statics and buckling solutions

Own SI fixtures: E=200 Pa, A=2 m², I=3 m⁴, rho=3 kg/m³,
L=4 m, w=6 N/m toward −Y, P=12 N. They are test scales, without claiming
to represent a real material. Solutions come from integrating
EI v''''=q, with continuity at point loads and the support conditions.

1. Cantilever: clamped at i, force −P at j.
   v_j=−PL³/(3EI), theta_j=−PL²/(2EI), R_y=P, R_z=PL;
   V=−P, M(x)=−P(L−x). One element is exact for this cubic deflected shape.
2. Beam clamped at both ends under uniform load: v(x)=−w x²(L−x)²/(24EI),
   v(L/2)=−wL⁴/(384EI). Transverse reactions wL/2 and nodal
   moments (+wL²/12,−wL²/12). Section M at ends −wL²/12,
   at center +wL²/24. Two elements allow verification of central deflection.
3. Beam clamped at both ends with central force −P through subdivision into two:
   v(L/2)=−PL³/(192EI); w reactions are replaced by P/2,
   nodal moments (+PL/8,−PL/8).
4. Pinned at both ends: v(L/2)=−5wL⁴/(384EI),
   R_y=wL/2 at each support, Mextremos=0, Mcentro=wL²/8,
   theta_i=−wL³/(24EI). Two elements, release i of the first and j of the
   second. Disconnected outer nodal rotations are constrained
   and physical rotation is recovered through equilibrium.
5. Propped cantilever and its reversal: for i clamped and j pinned,
   v(x)=−w x²(3L²−5Lx+2x²)/(48EI),
   v(L/2)=−wL⁴/(192EI), R_i=5wL/8, R_j=3wL/8,
   clamped nodal moment +wL²/8. Reversal permutes reactions and
   changes the nodal moment sign. Both releases are tested.
6. Symmetric frame: two columns of length h=3 m and beam of span b=4 m,
   clamped bases, P/2 in Ux of each top node, same properties.
   Finite column axial deformations are included. By symmetry,
   top displacements (delta,+v,theta), (delta,−v,theta).
   Left vertical equilibrium and moment equilibrium give:

   ```text
   beta = (12EI/b²)/(EA/h+24EI/b³), v=−beta theta
   D = 4EI/h+6EI/b−12EI beta/b²
   theta = −6EI delta/(h² D)
   P = 24EI delta/h³+12EI theta/h²
   delta = P/[24EI/h³−72(EI)²/(h⁴ D)]
   ```

   delta,v,theta and reactions are verified with actual assembly and backend,
   without replacing axial deformation with infinite stiffness.
7. Euler pinned-pinned: EI v″+P v=0, v(0)=v(L)=0,
   Pcr=pi²EI/L². Free nodal rotations at the supports are the natural
   condition M=0. K and −Kg are assembled with unit axial force N=−1 N obtained
   from u_j−u_i=−L_elemento/(EA). Axial and supported transverse translations
   are removed; the backend solves K phi=P(−Kg) phi.
   One cubic element gives Pcr=12EI/L² (discretization error +21.59%);
   two elements give approximately 9.94384680 EI/L² (+0.7522%).
   Explicit relative tolerances: 22% and 0.8%, respectively.
   Euler accuracy is not intended with a single cubic field.
   A double release in a single element with restrained translations removes
   all bending DOFs and cannot represent an interior buckling mode;
   that idealization is not used to attribute a finite critical load.

## Decisions and limits

1. New crate `engine-elements-frame` 0.1.0, workspace member.
   Runtime path dependencies on core/contracts/laws/linalg and dev path on
   assembly/model. No new external dependencies or license field or LICENSE.
   Cargo.lock only incorporates the new package and its internal links.
2. Small private modules geometry, stiffness, mass, geometric, loads,
   release, recover, factory; API and validators in lib.rs.
   `#![deny(missing_docs)]`, documented SI, no threads/fs/clock or global state.
3. Properties in self, geometry in ctx: ctx.params does not participate, as in
   the bar template. Factory with explicit SI E,A,I,rho and optional boolean
   release_i/j (false by default); rejects unknown fields,
   invalid types/ranges and ModelView outside D2, duplicate ids or non-finite/out-of-plane
   coordinates. No section ids or JSON references.
4. L by hypot; differences and products validated. L≤1e-12 m uses ENG-E-0005;
   non-finite/overflows ENG-E-0008; u_e sizes, nonempty history,
   stations outside [0,1], plane and parameters ENG-E-0010. Properties
   E,A,I and EA/EI positive; nonnegative rho. Positive elastic
   and mass coefficients underflowed to zero are rejected when needed.
   The entire extreme f64 range or binary equality between
   targets is not guaranteed. Geometry is validated even with rho=0.
5. Kg is delivered separately from respond; own beam-column derivation and
   initial-stress condensation documented above. No P-Delta
   solver, exact stability at finite axial force or exact dynamic mass
   with releases is implemented. N includes no segment loads in geometric_stiffness.
6. Actual static assembly uses ElementContribution, DofMap, LinearSystem and
   ReferenceBackend. Buckling assembles K/Kg with assemble_matrix and solves with
   the backend's generalized_eigen, without implementing additional solvers/eigen.
   Current SystemView is not assumed to reevaluate elements or axial forces: it receives
   precalculated contributions and its Kg has an explicit reference state.
7. ENG-ELM-003 (3D), spatial orientation, torsion, biaxial bending, nonlinear
   materials and shear deformations: **out of scope**.
8. wasm compilation is checked if the target is installed; execution in
   wasm runtime and native/wasm numerical validation: **PENDING**.
   Initial semver 0.1.0; physical/API/sign changes require fixture review
   and an appropriate version. element/1 and the error catalog are unchanged.

## Tests and tolerances

- Five assembled analytical tests: cantilever; beam clamped at both ends under uniform/point load;
  pinned and propped in both directions; frame with finite axial force; Euler 1/2
  elements. source/citation in fixtures, exact bibliography PENDING.
- Ten invariant/conformance/negative tests; one rustdoc test.
- Displacements/rotations: absolute 1e-12 m/rad; reactions/forces/equilibrium:
  absolute 1e-9 N/N·m. Matrices, mass, rotations, Schur and recovery:
  absolute 1e-10 in each entry's SI units in these small cases.
- element/1 kit and factory: central FD h=1e-7 m/rad, tangent relative error
  <1e-6 from the kit; four release combinations, repeatability 10 times,
  sizes, purity, symmetry, rigid modes and nontrivial global rotations.
- Independent polynomial energy integrals for K, M and Kg;
  block condensation with backend and energy/load/rotation comparison.
- Total mass rho A L per direction for both masses and all releases;
  double release with exactly zero Kflexión, linear transverse mass,
  transverse Kg N/L, zero moments and recovered loaded rotations.
- Covariant Kg/loads/forces under nontrivial rotation; N/V/M recovery
  at ends and unordered/duplicate stations; multiple loads, uniform
  axial loads, transverse loads, distributed moments and explicit self-weight.
- Euler: relative tolerance 22% (one), 0.8% (two); eigenvector residual
  absolute 1e-8 in equation units; one-element value 12EI/L²
  absolute 1e-9. Documented discretization error, not a solver tolerance.
- Negatives: E/A/I≤0, rho<0, non-finite properties, underflowed EA/EI,
  zero/short/non-finite/out-of-plane geometry, invalid states and sizes,
  invalid stations, non-finite and out-of-plane loads even 1e-30,
  product and sum overflows, missing/extra JSON fields/wrong
  types, nonboolean releases, duplicate node ids and disconnected rotation
  mechanism. Core catalog codes without adding new codes.

## Local verification

Final results:

- `cargo fmt --check`: successful.
- `cargo clippy --all-targets -- -D warnings`: successful.
- `cargo test --workspace`: successful; 129 workspace unit/integration tests
  and 34 doctests, including 15 tests and 1 doctest from the new crate.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`: successful.
- Installed target wasm32-unknown-unknown: `cargo check --workspace --target
  wasm32-unknown-unknown`: successful with `/home/box/.cargo/bin` in PATH
  (stable toolchain, rustc 1.99.0). `cargo check -p engine-elements-frame
  --tests --target wasm32-unknown-unknown`: successful. No fixtures were executed
  in wasm runtime: cross numerical execution **PENDING**.
- Native tests and rustdoc were also run with system cargo/rustc 1.85.1,
  compatible with rust-version=1.85. Clippy uses the installed stable
  toolchain component. No tools or targets were installed.
- `bash clean-room/check.sh origin/main`: successful.
  The script only examines paths known to Git; additionally it is run
  with temporary `GIT_INDEX_FILE` and `GIT_OBJECT_DIRECTORY` in /tmp, and existing
  objects as read-only alternates, incorporating new
  files to review the full diff. The user's real index is not
  modified. The first preparation with only a temporary index encountered the
  repo's read-only object directory; it was repeated also isolating
  the objects, without requesting permissions or modifying .git.
- `git diff --check`: successful, also for the diff with new files
  through the temporary index. Changed paths: Cargo.toml, Cargo.lock,
  new crate and this log; no other crate or protected path.

In the initial iteration rustdoc detected three vectors written with brackets
without code formatting and interpreted them as links; their delimiters were
corrected. Clippy flagged a redundant constant assertion in a test,
which was removed. Results above correspond to the corrected files.

No changes are made to clean-room/**, .github, CODEOWNERS or docs/CLEAN_ROOM.md,
or other existing crates. No commit, push or PR is performed and the user's Git
index is unchanged. Human clean-room review J4: **PENDING**.
