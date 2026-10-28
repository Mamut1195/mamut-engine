# Clean-room log · ENG-ELM-001

- Ticket / PR: ENG-ELM-001, «Linear 2D/3D bar»; existing branch stacked on
  ENG-ASM-001 (PR #20). File delivery, without commit, push or PR.
- Author(s): Codex gpt-6.1-sol high · 2026-10-11 (America/Guayaquil),
  identification requested for this ticket. No delegation to other agents.
- Role: implementer; own analytical derivations and fixtures, no oracle
  operator or similarity auditor activity.
- Contract implemented: `element/1` version 1.0, kinds `truss2d.linear` and
  `truss3d.linear`, and `engine_contracts::ElementFactory`. `elastic` material
  from `engine-laws` (`uniaxial/1`), without modifying its implementation. Crate/method
  0.1.0; contract and method versions are independent.
- Permitted sources used: internal documents and contracts listed below;
  own derivations of elasticity, energy and equilibrium. McGuire et al.,
  reference suggested for Kg by the ticket: **PENDING: full citation,
  edition, chapter/section and equation**. No precise consulted source is
  available and the derivation is not attributed to an invented page or equation.
  Exact bibliography for statics/strength of materials: **PENDING**.
- Own derivations: [note in this log](#derivaciones-propias),
  [fixtures](../crates/engine-elements-truss/tests/fixtures/mod.rs) and
  [invariant tests](../crates/engine-elements-truss/tests/invariants.rs).
- Third-party code copied: none. Own code and internal dependencies
  by path; no new external dependencies or copied book listings.
- Validation: exclusively analytical; closed-form solutions for an axial bar,
  three-bar statically determinate triangle and spatial tripod, mass by integration,
  second variation of length, uniform load equilibrium and conformance.
  All fixtures declare `source` and `citation`, with exact bibliography
  **PENDING** and reference to the own derivations in this note.
- Comparison with oracle (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: J4, **SUITABLE** (2026-10-11, communicated via J0). J0 decides the merge.

## Internal documents consulted

Before implementation, `/workspace/motor-rust/04_TICKETS.md` was read
(ENG-ELM-001, ENG-MAT-001, ENG-ASM-001 and limits of ENG-ELM-002/003),
`/workspace/motor-rust/00_PROYECTO.md`,
`/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md` (§3 and §4.4),
`docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-ASM-001.md` and `provenance/ENG-MAT-001.md`.
The traits, types, bar example and conformance kit of
`engine-contracts`, `engine-laws::uniaxial::Elastic`, `engine-core` errors,
`DMat` and `engine-linalg` backend, and `ElementContribution`, `LinearSystem`,
map, constraints and tests of `engine-assembly` were consulted. No applicable AGENTS.md
was found in the repo or its ancestor directories. No external sources,
other engine code, book listings or oracle numbers were consulted;
the network was not used.

The approved convention prevails over the provisional architecture
description: D2 is in horizontal X-Y, Z=0; D3 is right-handed with vertical Z.
The D2 elevation proposal remains pending. Only the local axis
x from i to j matters: no transverse D3 orientation is invented.

## Derivaciones propias

### Geometry, material and stiffness

Let r=x_j−x_i, L=||r|| and d=r/L, all in reference geometry. For a=2
in D2 or a=3 in D3, u_e=[u_i,u_j] is ordered with a translations per node.
Infinitesimal elongation is Δ=d·(u_j−u_i)=bᵀu_e, b=[−d,d]. Then
ε=Δ/L, σ=Eε (evaluated by `Elastic::respond`) and N=Aσ, positive in tension.
Elastic energy U=EA Δ²/(2L) gives f_int=N b and K=(EA/L)bbᵀ.
K is symmetric, positive semidefinite of rank one for EA>0, without pretension.
The halves are constructed by copying each entry to preserve exact symmetry.

A uniform translation has Δ=0. An infinitesimal rotation ω has
u_j−u_i=ω×r and d·(ω×r)=0, so the three D2 and six D3 rigid modes
produce no force and K u_rígido=0. Invariance under finite rotations
of displacements is not required in an infinitesimal theory. When rotating coordinates and
components with orthogonal Q, d'=Qd, f'=Tf and K'=TKTᵀ, T=diag(Q,Q).

### Mass

For constant rho the global velocity is interpolated with
H=[(1−s)I,sI], s=x/L. Kinetic energy is
½rho A∫₀ᴸ v_eᵀHᵀH v_e dx. The integrals of (1−s)², s² and s(1−s)
are 1/3, 1/3 and 1/6 respectively. With m=rho A L:

```text
M_consistente = (m/6) [2I  I]
                       [ I 2I]
M_concentrada = (m/2) diag(I,I)
```

Total physical mass is verified with vᵀMv=m for a uniform unit
translation in EACH axis; summing the trace or all axes does not define physical mass.
Both matrices are isotropic and covariant. rho=0 returns None; mass or
rotational inertia is never fabricated. With rho=3 kg/m³, A=2 m², L=2 m, m=12 kg:
lumped diagonal 6 kg, consistent diagonal 4 and coupling 2 kg per axis.

### Geometric stiffness

Considering a relative variation h, the expansion of ||r+h|| through order
two is L+d·h+(hᵀ(I−ddᵀ)h)/(2L). The work of a reference axial force N
produces the second variation N/L (I−ddᵀ) in h. Substituting h=δu_j−δu_i:

```text
P = I−ddᵀ
Kg = (N/L) [ P −P]
             [−P  P]
```

Here N is obtained from the total u_e itself through the elastic law; no external
axial force is accepted and no history is stored. P projects onto the transverse subspace.
Kg is symmetric, annihilates translations and relative axial movements; it has
rank a−1 when N≠0, rank zero when N=0. Its a−1 nonzero eigenvalues
are 2N/L, positive in tension and negative in compression. Covariance uses
the same T as K. For a 2 m bar with EA=400 N and elongation ±0.06 m,
N=±12 N, N/L=±6 N/m and transverse eigenvalues ±12 N/m.

This is an own derivation by second variation. The precise suggested citation
from McGuire et al. remains **PENDING**. `respond` remains linear with consistent
elastic K; Kg is offered separately and is not secretly added to K.
No P-Delta equilibrium, geometry update or buckling is implemented.

### Loads and recovery

`MemberLoad` accepts uniform force q_global N/m and uniform moment N.
For this idealization q_global=q d and zero moment are required. Integration
of the linear shapes gives f_eq=[q L d/2,q L d/2], in active global DOFs.
A transverse load cannot be balanced within a straight pinned
bar in linear theory without bending: it is rejected, also in `recover`, with
ENG-E-0010. A physical transverse load is not silently projected.
In D2 q_z≠0 is always rejected. The permitted projection residual is only
rounding: ≤64ε máquina·max|q_global|, without an absolute threshold; an arbitrarily small pure transverse
load is rejected. Moments must be exactly zero.

Self-weight requires the caller to supply q_global=rho A g in N/m and
that this force be axial. g is not inferred from axes or density. Example
D3 vertical upward bar of 2 m, rho=3, A=2 and g=(0,0,−10) m/s²:
q_z=−60 N/m and each node receives −60 N in Uz. An inclined bar subjected to
transverse gravity must be loaded nodally through an explicit idealization
outside this method; this ticket's load_vector rejects it.
Horizontal D2 does not accept self-weight in Z.

Let N_e=EAΔ/L and q be the sum of the uniform axial loads. From equilibrium
dN/dx=−q and the compatible average N_e,
N(s)=N_e+qL(1/2−s) is obtained. The conjugate end actions are
[−N_e−qL/2,0,…,N_e−qL/2,0,…]; negative face sign at i and positive at j.
Local vectors have a components per end and stations
[N,0,0,0,0,0], without arbitrary transverse axes. Station order and
duplicates are preserved, including ends; all must be in [0,1].
Reversing connectivity reverses d and q, permutes global blocks and preserves
N at the same physical section when s becomes 1−s.

With N_e=12 N, q=8 N/m and L=2 m, requested stations [1,0,0.5] give
[4,20,12] N; ends [−20,4] N satisfy equilibrium with total load 16 N.
In the vertical bar with u=0 and q=−60 N/m, N at [0,0.5,1] is [−60,0,60] N.

### Statics and strength of materials fixtures

All are own analytical choices in SI with E=200 Pa and A=2 m²;
they do not represent calibration of a real material or an example attributed to a
book. Each fixture has source and citation to this log; the exact external
reference remains **PENDING**.

1. Axial bar L=2 m, end i restrained and j transversely constrained,
   P=12 N: k=EA/L=200 N/m, δ=PL/EA=0.06 m, N=12 N and R_i=−12 N.
   Solved through ASM and backend in both dimensions.
2. Statically determinate triangle D2: A=(0,0), B=(2,0), C=(1,1); bars AB, AC and BC.
   A constrains Ux,Uy and B constrains Uy. C receives (6,−12) N. By moments at A,
   2R_By−12−6=0, thus R_B=(0,9) N and R_A=(−6,3) N.
   At C: (−N_AC+N_BC)/√2+6=0 and −(N_AC+N_BC)/√2−12=0.
   This yields N_AC=−3√2, N_BC=−9√2 N; by equilibrium at B, N_AB=9 N.
   Compatibility: u_Bx=9·2/400=0.045 m,
   u_Cx=0.0225+0.015√2 m, u_Cy=−0.0225−0.03√2 m.
   A 90° rotation of the entire model, loads and roller is also verified.
3. D3 tripod: restrained bases A=(1,0,0), B=(0,1,0), C=(−1,−1,0), apex
   D=(0,0,1), bars AD, BD and CD, load D=(6,−3,−12) N.
   Let a=N_AD/√2, b=N_BD/√2, c=N_CD/√3. At D:
   a−c+6=0, b−c−3=0, −a−b−c−12=0; a=−9, b=0, c=−3 N.
   Axial forces [−9√2,0,−3√3] N; reactions A=(−9,0,9), B=(0,0,0), C=(3,3,3) N.
   Compatibility: u_Dz=−3√3/400−3√2/200, u_Dy=u_Dz,
   u_Dx=u_Dz+9√2/200 m. A 0.73 rad rotation about Y is also verified.
4. Axial bar with terminal P=12 N and q=4 N/m, L=2 m:
   f_eq=(4,4) N, F_j=16 N, δ=0.08 m, N_e=16 N, R_i=−20 N.
   N at [0,0.5,1]=[20,16,12] N. K, both M and f_eq are assembled through
   ElementContribution. Kg is not queried through SystemView (see
   decision 8).

Global force and moment equilibrium are verified per direction; forces
from different axes are not summed in a single scalar condition.

## Decisions and limits

1. New crate `engine-elements-truss` 0.1.0 and workspace member.
   Runtime dependencies only by path to engine-core/contracts/laws/linalg;
   test dependencies only by path to engine-assembly/model. Cargo.lock
   only records the new package and those links; no external packages are added.
   Outside the new crate, Cargo.toml, Cargo.lock, this log and the
   minimal additive portability adjustment described below in engine-linalg change.
2. Small private modules geometry, stiffness, mass, geometric, loads,
   recover and factory; `lib.rs` gathers API and validators. Public API documented
   with `#![deny(missing_docs)]`; no I/O, threads, clock or mutable global.
3. `LinearTruss::new(dim,E,A,rho)` preserves validated `Elastic`, A, rho and EA.
   As the contract and internal example allow, properties in self and
   coordinates in ctx. `ctx.params` does not participate in these kinds: there is no
   implicit precedence between owned properties and context. Arbitrary/nonlinear
   laws or JSON material references are not accepted;
   the factory constructs Elastic from E. Generic resolved parameters for
   other kinds remain available in contracts, without modifying the trait.
4. The factory implements the existing trait without serde or parser: in-memory JSON
   object with exactly explicit numeric E, A, rho, ranges and units
   in the schema. Validates ModelView dimension, unique node ids and finite coordinates,
   and Z=0 in D2. There are no reference or connectivity fields in this
   schema; no bar geometry is presumed from ModelView. The concrete
   geometry is validated when evaluating ElementCtx.
5. Length accumulated with hypot, without squaring coordinates; finiteness
   of differences and L is verified. L≤1e-12 m uses ENG-E-0005, like the approved convention.
   D2 requires exactly Z=0 and two coordinates; incompatible dimension uses
   ENG-E-0010. Coordinates are validated even when rho=0.
6. E,A>0; rho≥0, all finite. EA, EA/L and positive mass coefficients
   reduced to zero by underflow are rejected with ENG-E-0010 when
   needed. NaN, infinities and overflows in geometry, material,
   forces, stiffnesses, masses and loads produce ENG-E-0008. Sizes of u_e,
   nonempty history, incompatible stations and loads use ENG-E-0010.
   The method does not guarantee the entire extreme f64 range; explicit numerical
   errors are preferable to zero physical matrices from underflow.
7. Zero mass returns None; Kg always returns Some, even a zero matrix
   for N=0. Empty initial, accepted and trial history; no hidden
   state, implicit acceptance, initial strain or pretension.
8. Integration uses ElementContribution with evaluated global matrices.
   WARNING (J0, 2026-10-11): engine-assembly `SystemView::geometric(u)`
   (system.rs) ignores u and returns precalculated Kg, against the
   SystemView contract; this ticket does not depend on it (only K, M, F) and another PR
   corrects it. Element Kg is validated only at element level (matrix and
   spectrum). No dynamic adapter
   or solver is added in this ticket. Solving is delegated to the existing
   backend, without implementing new elimination, factorization or eigen.
9. ENG-ELM-002+ retains bending, shear, torsion, releases,
   transverse/distributed moment loads and transverse 3D beam orientation.
   Material nonlinearity, corotational geometry, large deformations,
   stability and second-order solvers are outside this method.
10. Semver evolution from 0.1.0: compatible corrections in patches;
    changes in API, physical assumptions, parameters/schema or signs require
    an appropriate version and fixture review. element/1 is unchanged and
    no new core errors are registered. Cross native/wasm binary determinism
    and test execution in wasm runtime remain **PENDING**.
11. The wasm check discovered prior E0080 in
    `engine-linalg/src/lib.rs`: `MAX_ALLOC_BYTES: usize = 1 << 33` does not fit
    in wasm32. cfg was added to preserve exactly the
    64-bit constant and an alternative constant for smaller targets with
    `isize::MAX as usize`, the byte limit for a Vec allocation. The target-specific
    limit (2 GiB−1 on 32 bits) was documented. It is a minimal additive correction
    necessary to compile element dependencies in wasm; no changes
    to API, dependencies, algorithms or 64-bit behavior. The
    rest of engine-linalg and all other existing crates are unchanged. Existing extreme
    dimension tests verify rejection without allocation/panic.

## Tests and tolerances

Fourteen integration tests and one new doctest. Four analytical tests
assemble real elements and solve with engine-linalg `ReferenceBackend`.
Ten invariant tests check both elements and both factories.

- Fixture forces/reactions and ΣF: absolute 1e-9 N per axis. Also ΣM
  absolute 1e-9 N·m; expected displacements absolute 1e-12 m.
- Invariants/matrices/spectra/loads/recovery: absolute 1e-10 in the
  quantity's SI unit in the small fixtures.
- `check_element` and `check_element_factory` kits: FD increment 1e-7 m,
  relative FD tangent kit tolerance <1e-6, pure states, sizes,
  symmetry, rigid modes, nontrivial rotations and covariant mass.
- Exact symmetry verified with SymMatrix; explicit K u_rígido=0 for all
  infinitesimal modes; total mass per axis and complete analytical entries.
- Kg: spectrum obtained with existing backend and M=I; eigenvalues 0 and 2N/L,
  rank with absolute threshold 1e-9 N/m, tension/compression/zero axial force,
  axial nullspace and translations, covariance of an unaligned rotated matrix.
- Covariance of loads and recovered N, reversed connectivity, out-of-order
  stations, multiple axial loads and vertical D3 axial self-weight.
- Repeatability: ten identical evaluations per samples in each dimension;
  response with empty history and immutable private properties.
- Negatives: zero or negative E/A, negative rho, NaN/±inf in properties,
  every coordinate and every displacement, short/zero and out-of-plane
  geometry, incompatible dimensions/sizes/history, invalid stations,
  transverse components (including 1e-30 N/m), moments, non-finite loads,
  overflows in geometry, stiffness, force, mass and load summation,
  underflowed EA; schema with missing fields, extra fields, wrong types,
  factory model with duplicate ids, non-finite coordinates or incompatible dimension/plane.
  No external result is used as a reference.

## Local verification

Commands run with `/home/box/.cargo/bin` in PATH and
`RUSTUP_TOOLCHAIN=stable`, rustc 1.99.0. Final results after correcting
the engine-linalg constant for wasm32:

- `cargo fmt --check`: successful.
- `cargo clippy --all-targets -- -D warnings`: successful.
- `cargo test --workspace`: successful; 106 workspace unit/integration tests
  and 33 doctests, including 14 and 1 from the new crate.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`: successful.
- Installed target wasm32-unknown-unknown: `cargo check --target
  wasm32-unknown-unknown -p engine-elements-truss`: successful.
- `cargo check --target wasm32-unknown-unknown -p engine-elements-truss
  --tests`: successful. Compilation only; execution in wasm runtime and
  cross native/wasm numerical validation **PENDING**, not claimed.
- `git diff --check` and trailing whitespace review in new files:
  successful. Diff restricted to the new crate, Cargo.toml, Cargo.lock,
  this log and the engine-linalg adjustment by target.

The first wasm compilation failed with E0080 in an existing dependency;
decision 11 records the cause and correction. No dependencies,
targets or tools were installed, and no remote operations were performed.

No protected paths or the user's Git index are modified; the only change
in another crate is the engine-linalg constant by target explained above.
No license fields, LICENSE files or legal text are added. Human
clean-room review: J4 SUITABLE (2026-10-11, via J0).

Follow-up (2026-10-11): MAX_ALLOC_BYTES by target (8 GiB on 64 bits, 1 GiB on 32 bits) with test; fallible station allocation in recover.rs; stiffness.rs assumption documented.
