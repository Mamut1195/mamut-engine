# Clean-room log · ENG-GEO-002

- Ticket: MAM-44 · ENG-GEO-002, «Beam on elastic foundation (Winkler) 2D/3D».
- Author: Codex gpt-6.1-sol high under J2; 2026-10-14 (America/Guayaquil).
- Role: specifier and implementer, clean session; no operator or auditor role.
- Contract: element/1; kinds `frame2d.winkler` and `frame3d.winkler`.
- Sources used: own derivations developed below; internal contracts and code
  of frame2d/frame3d, engine-assembly, engine-linalg and engine-validation;
  docs/CLEAN_ROOM.md, docs/CONVENCION_EJES.md and provenance/TEMPLATE.md.
- Conceptual references, without consulting their texts or code: E. Winkler (1867),
  *Die Lehre von der Elasticitaet und Festigkeit*, H. Dominicus, Praga;
  M. Hetényi (1946), *Beams on Elastic Foundation*, University of Michigan Press,
  Ann Arbor. They are cited for the physical model and classical problem; no unverified
  page or equation is attributed. The fixture equations are own.
- Third-party code copied: none. No external sources, network, manuals of
  other engines, books with listings or oracle outputs.
- Validation: polynomial integration, energy, equilibrium and closed-form solutions
  derived here; same fixtures in native and wasm. No mamut-workflows.
- External oracle comparison: no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Model and units

The foundation is bilateral, linear, homogeneous and attached to ground: the distributed force
on the beam is q_b=−k v. k has units N/m²: k·v is N/m, per axial
beam length. It is not pressure per area and requires no implicit width. The energy
is Π_b=½∫₀ᴸ k v² dx. There is no soil mass, damping, unilateral contact,
shear coupling, prescribed ground movement or adjustment of k.

The axes, SI, orientation and DOFs of the existing frames are retained:
D2 in XY, order [ux,uy,rz] per node; D3 [ux,uy,uz,rx,ry,rz].
In 3D orientation is the local z reference, y=z×x and x×y=z.
ky resists v in local y (EI=EIz); kz resists w in local z (EI=EIy).
The slopes are v′=rz and w′=−ry. There is no coupling between planes,
axial or torsional resistance from the foundation.

## Own derivation of the consistent matrix

Let t=x/L. I interpolate v=N d with d=[v_i,θ_i,v_j,θ_j], θ=v′:

```text
N1=1−3t²+2t³       N2=L(t−2t²+t³)
N3=3t²−2t³         N4=L(−t²+t³)
δΠ_b = δdᵀ ∫ k NᵀN dx d
K_b = k L ∫₀¹ NᵀN dt
```

The rule ∫₀¹ tⁿ dt=1/(n+1) allows integrating the ten distinct products.
I explicitly show two expansions:

```text
N1²=1−6t²+4t³+9t⁴−12t⁵+4t⁶
∫N1²dt=1−2+1+9/5−2+4/7=13/35=156/420
N1 N2/L=t−2t²−2t³+8t⁴−7t⁵+2t⁶
∫N1 N2/L dt=1/2−2/3−1/2+8/5−7/6+2/7=11/210=22/420
```

The other products, including the signs of N4, give:

```text
∫ N1 N3 dt=9/70=54/420
∫ N1 N4 dt=−13L/420
∫ N2² dt=L²/105=4L²/420
∫ N2 N3 dt=13L/420
∫ N2 N4 dt=−L²/140=−3L²/420
∫ N3² dt=13/35=156/420
∫ N3 N4 dt=−11L/210=−22L/420
∫ N4² dt=L²/105=4L²/420

           [ 156    22L     54    −13L  ]
K_b=kL/420 [ 22L    4L²    13L    −3L² ]
           [ 54     13L    156    −22L  ]
           [−13L   −3L²   −22L     4L² ]
```

The implementation computes a=kL/420, b=aL and c=bL to avoid unnecessary
powers and validates each coefficient and entry. It does not integrate numerically
in production or use a set of lumped nodal springs.

In 2D I insert K_b at indices [1,2,4,5] of the local 6×6 matrix.
In 3D I insert K_b(ky) at [1,5,7,11] and D K_b(kz) D at [2,4,8,10],
D=diag(1,−1,1,−1), because w′=−ry. With each frame's transformation T:

```text
K_global = K_frame,global + Tᵀ K_b,local T
f_int,global = f_frame,global + Tᵀ K_b,local T u_global
```

The foundation does not alter the frame's mass, Kg or external loads. It has empty history,
is linear and deterministic. k=0 produces exactly the frame without foundation in all
supported operations. Geometry is validated even with k=0.

### Symmetry, positivity and rigid modes

K_b is symmetric because it is Gram. If k>0 and d≠0, N d is not the zero polynomial
(endpoint value/slope interpolation is unisolvent); therefore
 dᵀK_b d=k∫(N d)² dx>0. The transverse 4×4 block is positive definite.
The full matrix is positive semidefinite: its axial and torsional DOFs
are null; with k=0 the entire block vanishes. Those modes are not regularized.

A transverse rigid displacement moves the beam relative to the stationary ground and
does generate a reaction. The `check_element` helper requires all rigid modes
to be force-free and cannot be applied literally to a ground-supported element.
This assumption is clarified in the element/1 documentation and identity, state,
sizes, symmetry, FD, repetition, covariance, mass and recovery are tested
directly. Traits, versions and conformance helpers are not changed.

## Recovery by equilibrium with distributed reaction

Local end actions include the consistent foundation:
p=(K_frame+K_b)u−f_ext,equiv. K_frame u−f is not reported as if the foundation
were a physical nodal load. For v(t)=Σ₀³ a_j tʲ:

```text
a0=vi; a1=Lθi
a2=−3vi−2Lθi+3vj−Lθj; a3=2vi+Lθi−2vj+Lθj
F_b(x)=∫₀ˣ k v(s)ds = kL Σ a_j t^(j+1)/(j+1)
J_b(x)=∫₀ˣ (x−s) k v(s)ds = kL² Σ a_j t^(j+2)/[(j+1)(j+2)]
Vy=−p_vi−qy x+F_b(x)
Mz=−p_rzi+p_vi x+qy x²/2−mz x−J_b(x)
```

In local z, d=[wi,−ryi,wj,−ryj] is used,
Vz=−p_wi−qz x+F_b and My=−p_ryi−p_wi x−qz x²/2−my x+J_b.
Thus the faces at x=0 and x=L agree with the ends, and V′=−q_ext+k v.
These are equilibrated internal forces of the discrete field, not a claim of an exact
continuous solution within each element. The curvature EI v_h″ may differ
locally from this equilibrated internal force; the difference converges with refinement.

## Oráculos analíticos y cotas

### Viga infinita y semiinfinita

Away from the load EI v''''+k v=0. With λ=(k/(4EI))^(1/4), the roots are
±λ±iλ. At x≥0 the decaying solution is
v=e^(−λx)[A cos(λx)+B sin(λx)]. Its initial values are:

```text
v(0)=A; v′(0)=λ(B−A); v″(0)=−2λ² B; v'''(0)=2λ³(A+B)
```

For an infinite beam and central load P, symmetry imposes v′(0)=0, hence B=A.
The jump EI[v''']=P implies 2 EI v'''(0+)=P, giving
A=P/(8EIλ³)=Pλ/(2k). |M0|=2EIλ²A=P/(4λ).

For a semi-infinite beam loaded at the free end, zero moment imposes B=0 and
EI v'''(0)=P (negative face sign). Then
A=P/(2EIλ³)=2Pλ/k. Distant free ends are used, without bending supports.

Infinite-beam fixtures have λL=20, with the center at distance 10/λ
from each end; semi-infinite ones λL=12. To quantify truncation,
at a dimensionless distance a, let c=cos a, s=sin a, d=[A,B] be the
coefficients of the field e^(−xλ)(A cos xλ+B sin xλ), and r=[C,D] those of the
growing field e^(xλ)(C cos xλ+D sin xλ). The two free conditions give:

```text
       [ s     −c   ]          [ −s       c   ]
e^(−a) [ c−s   s+c  ] d + e^a [ −s−c    c−s ] r = 0
```

The second matrix has determinant 1. Both matrices and their inverse have
infinity norm ≤2√2; therefore ||r||∞≤8 e^(−2a)||d||∞. In the infinite case,
the symmetry/jump conditions at the center give
A=A0+C, B=A0−D (A0=Pλ/(2k)). In the semi-infinite case,
A=A0+C−2D, B=D (A0=2Pλ/k). The norm of the dependence of d on r is
respectively 1 and 3; solving that feedback gives denominators
1−8e^(−2a) and 1−24e^(−2a). The central or terminal deflection differs from A0
by 2C or 2(C−D), and the central moment by a term proportional to 2D.
A conservative uniform relative bound for these observations is
64 e^(−2a)/(1−24 e^(−2a)), less than 1.4e-7 for a≥10.
Truncation is not presented as zero discretization error.

The solution is smooth on each side of the load, always located at a node.
In nested Hermite subspaces, the Galerkin approximation minimizes
energy and the energy error does not increase. For the point load,
P[v(0)−v_h(0)]=a(v−v_h,v−v_h)≥0, so the loaded deflection converges
monotonically to the finite domain. Interpolating a smooth solution with Hermite
gives energy error O(h²) and error in that deflection O(h⁴). Universal monotonicity
of moment is not deduced from this identity; it is checked in these fixtures.

Refinement uses 16/32/64 elements (infinite; λh=1.25/.625/.3125) and
12/24/48 (semi; λh=1/.5/.25). The relative acceptance bound 2e-4
includes truncation <1e-6 and Hermite discretization; strictly decreasing error
is also verified across all three meshes for deflection and, in the infinite case,
for moment. The bound is a regression bound for these dimensionless data,
not a universal a priori estimate for any EI, k, domain or mesh.

### Viga corta rígida

With free ends and centered load, integrating the equation gives ∫k v dx=P,
and symmetry nullifies rigid rotation. As EI→∞, v is constant:
v=P/(kL). For finite EI I write v=v̄+z, ∫z=0 and v̄=P/(kL).
In the symmetric subspace v′(0)=0, Cauchy–Schwarz applied twice to
z(x)−z(0)=∫₀ˣ(x−s)z″(s)ds and to its average gives
|z(0)|≤C||z″||, C²=L³/320. Indeed, in 0≤x≤a=L/2,
z(0)=−(1/(2a))∫₀ᵃ(a−s)²z″(s)ds and symmetry gives
||z″||²=2∫₀ᵃz″²; integrating (a−s)⁴ gives that value of C².
The weak equation tested with z gives EI||z″||²+k||z||²=P z(0).
Therefore |z(0)|≤P L³/(320EI), and the relative central deflection error is bounded
by ε/320, ε=kL⁴/EI. In the fixtures L=2, EI=[1e5,2e5], k=[3,6],
ε=4.8e-4 and ε/320=1.5e-6. The acceptance bound 2e-5 adds f64 margin.
2/4/8 elements are used and the limit is verified on each final mesh.

The sum of soil reactions is not obtained by subtracting external nodal
forces: k v_h is integrated with three-point Gauss (exact for the cubic
field) and its first moment (quartic polynomial). ΣR=P and
ΣxR=P a are required with relative error ≤1e-7 and absolute 1e-8 on all meshes.
The numerical free residual has threshold 1e-8, or 1e-6 in the rigid limit:
large EI and a fine mesh amplify cancellation of bending forces.
End faces use absolute tolerance 1e-8 (1e-6 in the rigid
limit) and relative 1e-9 due to cancellation of large bending terms.
The applied/support reaction sums in the generic report do not include
a foundation that is part of K; physical balance is explicitly checked
with its integrals, without modifying the assembler.

## API, factory and diagnostics

Each frame crate exports `WinklerFrame2D`/`WinklerFrame3D` and its
`WinklerFactory`, matching the registration pattern through `ElementFactory`
of `FrameFactory`; there is no central global element registry.
Factories own the properties and reuse validated construction
of the base frame. Own schemas with additionalProperties=false, required k or ky/kz,
minimum zero and explicit units. There are no new external
dependencies: only a path dependency from both frames to engine-geo.

This version does not support end releases: they are rejected both in the factory
and in the composite constructor. Condensing with the transformation of the frame
without soil would be incorrect; supporting it would require condensing K_frame+K_b and
recovering rotations with that matrix. That extension remains outside this ticket.
The behavior of existing Euler frames is not modified.

ENG-E-0010 for negative k, invalid fields/types/ranges, releases and
underflowing positive coefficients; ENG-E-0008 for NaN/infinity/overflow;
ENG-E-0005 for length ≤1e-12; ENG-E-0002 for repeated ids in ModelView.
The new library modules contain no panic!, unwrap or expect.
Representing the full extreme f64 range is not promised.

## Fixtures and verification

`engine-validation/tests/support/winkler_table.rs` contains six problems
with source, citation, SI data and tolerances: infinite, semi-infinite and rigid,
each in 2D and 3D. Spatial ones load both planes with opposite signs,
different stiffnesses and independent analytical solutions. The harness uses
the actual factories, DofMap, LinearSystem and StaticLinear, without artificial
transverse supports; it only constrains unloaded uncoupled axial/torsional DOFs.

Element tests: integral of each independent polynomial product,
positive 4×4 Cholesky, local insertion and sign in z, absence of coupling,
k=0 in all operations, FD of all DOFs, repeatability, full rotation
of geometry/orientation, K/M, resisted transverse modes,
factory, history, geometry and extreme result validations.

The example `engine-validation/examples/winkler_wasm.rs` reuses exactly
the native harness and table. It exports `winkler_benchmarks()` which returns six
if everything passes, zero on an error, executable from Node without external imports.

Final results (all without network, CARGO_NET_OFFLINE=true or --offline and
CARGO_TARGET_DIR=/workspace/.cargo-target):

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: 428 tests passed including doctests,
  zero failed and zero ignored. The extension adds seven Rust tests:
  six for invariants and one that runs six problems with three meshes each.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: passed.
- `cargo check --workspace --target wasm32-unknown-unknown`: passed.
- Additional: `cargo check --workspace --all-targets --target
  wasm32-unknown-unknown`: passed, including fixtures and tests.
- Release build of the example for wasm and execution through
  `node crates/engine-validation/scripts/run-winkler-wasm.cjs
  /workspace/.cargo-target/wasm32-unknown-unknown/release/examples/winkler_wasm.wasm`:
  six fixtures passed; the script verifies that there are no external imports.
- `bash clean-room/check.sh origin/main`: passed with all new files
  included in the index, so that the script reviews the full diff.
- `git diff --check` and `git diff --cached --check`: passed.
- Library inspection: no added panic!, unwrap or expect;
  no changes to docs/CLEAN_ROOM.md, .github/** or clean-room/**.

Native (tests and rustdoc) with system Rust/Cargo 1.85.1, to respect the
workspace MSRV. Clippy and wasm with installed stable 1.99.0, from
/home/box/.cargo/bin, with wasm target previously installed; no downloads.
The wasm execution covers the six Winkler problems; it does not claim execution of
the other suites in that runtime or binary equality between targets.

Two temporary mutations were made and the original content was restored
before the final checks:

1. Replacing K_b with zero while retaining types/factories: the benchmark fails
   due to singularity without transverse support; four invariant tests also
   fail (integration/positivity, planes, solvable block and resisted rigid
   movement). Both test targets terminate with exit 101.
2. Reversing the sign of ∫k v in recovery: the independent quadrature
   test with distributed loads fails (exit 101); the other five
   invariant tests still pass. This detects removal/corruption of
   soil recovery even if the tangent is retained.

The first execution of the rigid fixture used EI=[1e6,2e6], whose f64 cancellation
with eight elements exceeded the relative equilibrium threshold 1e-7.
EI=[1e5,2e5] was chosen, still 1/ε>2000 relative to the foundation, and the
ε/320 bound above was derived to justify comparison without relaxing physical
balance. Face recovery tolerances separate this rounding from the
deflection error. Rustdoc initially detected `[0,1]` as a broken link;
the documentation was corrected and verification was repeated successfully.

Independent clean-room review: PENDING (J4).
