# Clean-room log · ENG-ELM-003

- Ticket / PR: ENG-ELM-003 · 3D elastic Euler–Bernoulli beam-column; PR #44.
- Author(s): J2 (agent, Grok Bot) · execution model for this session:
  GPT-6 (Codex) · 2026-10-11 (America/Guayaquil).
- Role: specifier and implementer of own analytical derivations.
- Implemented contract: element/1 · kind = `frame3d.euler_elastic`.
- Permitted sources used: no external sources. Only this repo's internal documents and patterns
  listed below. No network was consulted.
- Own derivations: [derivations section of this log](#derivaciones-propias).
- Third-party code copied: none. Internal frame2d patterns
  and patterns from the truss producer are reused in accordance with the request.
- Validation: analytical, own polynomial integrals, closed-form solutions,
  independent condensation and existing conformance kit.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket (docs/CLEAN_ROOM.md §5)."
- Clean-room reviewer: PENDING (J4).

## Internal documents consulted

`docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-ELM-002.md`, all modules, factory and tests of
`crates/engine-elements-frame`; the producer and producer tests of
`engine-elements-truss`; traits, types and kit of `engine-contracts`; the
`engine-core` catalog, `engine-laws::uniaxial::Elastic` law, matrices and backend of
`engine-linalg`, assembly and system of `engine-assembly` and types of
`engine-model`. There is no applicable AGENTS.md. No other engines,
external sources, external examples or oracle results were consulted.

## Derivaciones propias

### Triad, interpolation and energy

The approved convention is global X×Y=Z, X-Y horizontal and Z upward;
positive translations and rotations along their axes and by the right-hand rule. The
face convention and positive N in tension from CONVENCION_EJES.md are retained.

Let d=p_j−p_i, L=hypot(d_x,d_y,d_z), x=d/L. The `orientation` argument
is a global reference for **local z**, not y. It is normalized by scaling by
its maximum absolute component (avoids overflow and accepts tiny scales).
With unit reference a, y=unit(a×x), z=unit(x×y) are computed.
This is equivalent to projecting a onto the plane perpendicular to x, using the projection
as z and choosing y=z×x. It satisfies x×y=z; the explicit transverse orientation
provided for by the convention is obtained through y, without an automatic axis.
The case a=0 or |a×x|≤1e-12 is rejected; the second threshold is angular,
dimensionless and distinct from the absolute length tolerance.

R has rows x,y,z. For global→local displacements,
T=diag(R,R,R,R) is used, one for translation and rotation of both nodes:
K_global=TᵀK_local T and f_global=Tᵀf_local. A proper rotation Q of the
entire model rotates **a as well**; then K'=Q_e K Q_eᵀ, f'=Q_e f.

Local order:
`[u_i,v_i,w_i,rx_i,ry_i,rz_i,u_j,v_j,w_j,rx_j,ry_j,rz_j]`.
Without shear, section rotation has rz=v′ and ry=−w′, obtained from
r×x=(0,rz,−ry). u and rx are interpolated linearly. With t=x/L,
the Hermite value and derivative shapes are:

```text
h1=1−3t²+2t³       h2=L(t−2t²+t³)
h3=3t²−2t³         h4=L(−t²+t³)
v=h1 vi+h2 rzi+h3 vj+h4 rzj
w=h1 wi−h2 ryi+h3 wj−h4 ryj
Π=½∫[EA u′²+GJ rx′²+EIz v″²+EIy w″²]dx
```

J is the Saint-Venant torsional constant, **not** Iy+Iz. G is not derived
through Poisson and shear deformation is not added. Axial response is evaluated with
the internal elastic law: ε=(uj−ui)/L, N=A·Eε. There is no history.

### Euler 12×12 matrix

The integral of h″ᵀh″ produces the Euler–Bernoulli beam stiffness block (classical textbook result; no specific manual is cited), derived here by own integration of h″ᵀh″:

```text
B(EI)=(EI/L³) [[12,6L,−12,6L],
               [6L,4L²,−6L,2L²],
               [−12,−6L,12,−6L],
               [6L,2L²,−6L,4L²]]
```

In v it is inserted at indices (1,5,7,11); in w, at (2,4,8,10) with congruence
D=diag(1,−1,1,−1). The u and rx blocks are respectively
(EA/L)[[1,−1],[-1,1]] at (0,6) and (GJ/L)[[1,−1],[-1,1]] at (3,9).
Defining a=EA/L, t=GJ/L, bα=12EIα/L³, cα=6EIα/L²,
dα=4EIα/L, eα=2EIα/L, we obtain explicitly:

```text
[ a   0    0   0    0    0  −a   0    0   0    0    0 ]
[ 0  bz    0   0    0   cz   0 −bz    0   0    0   cz ]
[ 0   0   by   0  −cy    0   0   0  −by   0  −cy    0 ]
[ 0   0    0   t    0    0   0   0    0  −t    0    0 ]
[ 0   0  −cy   0   dy    0   0   0   cy   0   ey    0 ]
[ 0  cz    0   0    0   dz   0 −cz    0   0    0   ez ]
[−a   0    0   0    0    0   a   0    0   0    0    0 ]
[ 0 −bz    0   0    0  −cz   0  bz    0   0    0  −cz ]
[ 0   0  −by   0   cy    0   0   0   by   0   cy    0 ]
[ 0   0    0  −t    0    0   0   0    0   t    0    0 ]
[ 0   0  −cy   0   ey    0   0   0   cy   0   dy    0 ]
[ 0  cz    0   0    0   ez   0 −cz    0   0    0   dz ]
```

There is no numerical integration in production. Tests integrate products of
polynomials through ∫₀¹tⁿdt=1/(n+1), with coefficients independent of
the matrix construction. `respond` provides Kc and Kc·u, without loads or Kg.

### Releases of T, My, Mz

`MomentReleases { i:[T,My,Mz], j:[T,My,Mz] }` contains local booleans.
Partitioning displacements into retained a and released internal r:

```text
ur=−Krr⁻¹Kra ua+Krr⁻¹fr
u=H unodal+ucarga
Kc=Kaa−Kar Krr⁻¹Kra = HᵀKH
fc=fa−Kar Krr⁻¹fr = Hᵀf
```

The three rotational blocks are independent. For each plane, in the
slope variable θ (rz for v, −ry for w):

- Only i: θi=3(vj−vi)/(2L)−θj/2+fθi L/(4EI).
- Only j: permuted expression.
- Both: θi=θj=(vj−vi)/L without load. The affine part uses
  Krr⁻¹=(L/EI)[[1/3,−1/6],[-1/6,1/3]].

One release leaves energy 3EI/(2L³)(vj−vi−Lθfijo)²; two nullify
Kflexión **exactly**, with zero columns and rows for released rotations.
This analytical Schur is constructed to avoid residual stiffness with enormous EI.

In torsion, one release gives rx_r=rx_a+fr/(GJ/L) and condensed Ktorsión=0:
torsion cannot be transmitted through a free end. The equivalent load
transfers all torque to the retained end. With both free,
Krr=(GJ/L)[[1,−1],[-1,1]] is singular in the common rotation. For compatible load
fi+fj=0, zero mean rotation is taken and rx_i=fi/(2GJ/L), rx_j=−rx_i;
H has both torsional rows zero. Under uniform load mx≠0,
fi+fj=mx L≠0: there is no equilibrium, ENG-E-0006. A pair of opposite uniform
loads may indeed sum to zero in `recover`/`local_displacements`.
The mechanism is not hidden and no artificial stiffness is added.

`local_displacements` recovers the twelve physical local DOFs including
the loaded affine parts. `recover` obtains the ends through Kc u−fc,
algebraically equal to K u_rec−f in retained DOFs, and zero in released ones.
This avoids cancellation of enormous terms also with loads and two
releases. Tests compare with an independent numerical Schur,
full recovery and matrix projection; for double torsion the independent
solver sets a different gauge, without altering Kc, fc or internal forces.

### Consistent and lumped mass

The kinetic energy is
½∫[rho A(u̇²+v̇²+ẇ²)+rho(Iy+Iz)rẋ²]dx.
The physical polar inertia Ip=Iy+Iz is independent of J. Bending rotary
inertia is omitted, as in frame2d. The inertias in ry/rz come from the
transverse interpolation, rather than from adding rho Iy or rho Iz.

```text
m=rho A L, p=rho(Iy+Iz)L
M_axial=(m/6)[[2,1],[1,2]], M_torsión=(p/6)[[2,1],[1,2]]
M_Hermite=(m/420)[[156,22L,54,−13L],
                 [22L,4L²,13L,−3L²],
                 [54,13L,156,−22L],
                 [−13L,−3L²,−22L,4L²]]
```

The Hermite block is inserted in the same planes and with the same D as K.
Lumped mass: m/2 in each translation, p/2 in rx of both nodes;
ry/rz without their own lumped mass. rho=0 returns None after validating geometry.
With releases Mc=HᵀMH (Guyan), not Schur of M. It is a static reduction
for small oscillations at constant load, not exact dynamics of internal rotations.
In double torsion the zero common rotation gauge excludes that mechanism from nodal
mass; disconnected rotations must be eliminated or constrained in the system.
Tests check physical mass rho A L in each global direction, symmetry,
rotation congruence and independent integrals.

### Kg and owned producer

The reference length increment due to slopes is
½∫(v′²+w′²)dx. The second variation of the work of constant N gives:

```text
Bg=N/(30L) [[36,3L,−36,3L],
           [3L,4L²,−3L,−L²],
           [−36,−3L,36,−3L],
           [3L,−L²,−3L,4L²]]
```

Bg is inserted in both planes, using D in w. Tension stabilizes;
compression destabilizes. There is no axial or torsional Kg in this formulation.
N comes from total axial displacements, not from member loads:
`geometric_stiffness` does not receive loads and does not infer a variable N(x).
Kgc=HᵀKgH is the derivative of the Schur of K+λKg at λ=0 by stationarity,
not its exact condensation at finite N. Constant geometry, without corotation.

`geometric_producer` follows the owned truss adapter: it captures the element,
coords and params; validates 12 distinct locations; reconstructs ElementCtx and
calls Kg with empty state at each evaluation of total global displacement.
The system validates the location range on registration; the producer validates size
(ENG-E-0010), repetition (ENG-E-0002), geometry and finiteness.
It is tested both in isolation and registered in LinearSystem.

### Uniform loads and recovery

The MemberLoad contract expresses global q and m. They are rotated by R; `global_load`
converts **local** intensities to a global MemberLoad to use all
contract operations without new variants or changes to element/1.
External work: ∫(qx u+qy v+qz w+mx rx−my w′+mz v′)dx:

```text
f_i=[qxL/2, qyL/2−mz, qzL/2+my, mxL/2, −qzL²/12, +qyL²/12]
f_j=[qxL/2, qyL/2+mz, qzL/2−my, mxL/2, +qzL²/12, −qyL²/12]
```

fc=Hᵀf is condensed; the fully restrained end actions are −fc.
All six intensities are supported, without omitting distributed moments.
`self_weight(g)` computes global q=rho A g, requires explicit g and accepts −Z.
Let pi be the six local actions at end i; by equilibrium in [0,x]:

```text
N=−pi_u−qx x          Vy=−pi_v−qy x         Vz=−pi_w−qz x
T=−pi_rx−mx x
My=−pi_ry−pi_w x−qz x²/2−my x
Mz=−pi_rz+pi_v x+qy x²/2−mz x
```

They satisfy F′=−q and M′=−x×V−m. At x=0 they are the opposites of the actions
on negative face i; at x=L they coincide with the actions on positive face j.
Order and duplicates of stations x/L in [0,1] are preserved. The interior
Hermite deflection is not provided as the exact quartic solution for a uniform load.

## Tests and tolerances

Own SI fixtures: E=200, G=80, A=2, J=1.5, Iy=3, Iz=4, rho=3, L=4.
`source` and `citation` refer to these derivations, without pending external citations.

- Cantilever with Py=12, Pz=−15 and T=9: δy=PyL³/(3EIz),
  δz=PzL³/(3EIy), rx=TL/(GJ), ry=−PzL²/(2EIy), rz=PyL²/(2EIz).
  Assembly with DofMap/LinearSystem, reactions and six internal forces at stations.
- Uniform load with both ends restrained in both planes: two elements and a free interior node;
  reactions −qL/2 and end moments of magnitude qL²/12.
  Uniform axial load, torque and distributed transverse couples; face equilibrium.
- Uniform load with both ends pinned: reactions −qL/2, zero end moments,
  central bending moment of magnitude |q|L²/8. Pinned-restrained in both directions:
  reactions 3|q|L/8 and 5|q|L/8, restrained-end moment |q|L²/8. Torsion released
  at each end with uniform load: reaction −mxL at the retained end.
- Euler with both ends pinned in both planes: Kφ=P(−Kg)φ, N=−1 from u.
  Pcr=π²EI/L². One Hermite gives 12EI/L² (+21.59%); two give error <0.8%;
  four <0.06%. Free support rotations retained, without condensing them.
  Eigenvector residual and one-element closed-form value are checked.
- Existing kit `check_element` and `check_element_factory`, without modification:
  16 combinations of isotropic releases (Iy=Iz and My/Mz paired per
  end). The kit rotates coordinates and u but **retains self and ctx.params**:
  it cannot represent a full rotation of an orientation owned by
  an anisotropic section. Requiring its partial covariance for Iy≠Iz would physically
  change the section roll; it is not a property of the original model.
- For Iy≠Iz: own tests of all 64 combinations, general Rodrigues rotation
  of the model **including orientation**, K'=Qe K Qeᵀ, f, mass, Kg,
  ends, stations and internal rotations; also rigid translation of coordinates.
  Tangent by central FD in all 12 DOFs, h=1e-7 and repeatability.
- Symmetric K, six rigid modes K u≈0 and total mass per global axis for all
  64 combinations. Independent Schur with backend, loads and recovered rotations;
  independent polynomial integrals of elastic/kinetic/geometric energy.
- E and G of 1e200: no combination panics, double released bending
  without spurious shears; double torsion with incompatible load generates a diagnostic.
- Negatives: ranges, non-finites, underflowing/overflowing products,
  coordinates/length, dimension, history and vector sizes, loads,
  zero/parallel/nearly parallel orientation; invalid JSON, model and locations.
  Orientations of scale 1e-310 and f64::MAX produce the same triad.

SI comparison tolerance for these fixtures:
|a−b|≤1e-9+1e-11|b|, applied to each entry with its corresponding unit.
FD: absolute error <1e-7+1e-6|Kij|; kit h=1e-7 and original tolerances.
The Euler discretization thresholds are not solver tolerances.

## Structure, diagnostics and limitations

New crate 0.1.0, workspace member; **internal path** dependencies on
core/contracts/laws/linalg/assembly, dev on model. No new external
dependencies. Private modules factory, geometry, stiffness, release, loads, mass,
geometric, recover and producer; API/validators in lib.rs, `deny(missing_docs)`.
Doc comments in Spanish. Contracts, catalog, frame2d and truss are not changed.

E,G,A,J,Iy,Iz and products EA/GJ/EIy/EIz must be finite positive values, rho≥0.
Required results, differences, sums, projections and coefficients are validated.
ENG-E-0008 for non-finite inputs/results and overflow; ENG-E-0010 for
ranges/sizes, positive coefficients underflowing to zero and invalid
orientation; ENG-E-0005 for L≤1e-12; ENG-E-0006 for double released torsion
without equilibrium. There are no unwrap, expect or explicit panics in library code.
Stations use fallible allocation; no I/O, threads, clock or global state.
The full extreme f64 range and binary equality between targets are not guaranteed.

Linear homogeneous model, Euler–Bernoulli without shear deformation or warping,
constant reference geometry. No nonlinear solver inside the element.
Condensed mass/Kg are declared approximations; mechanisms are not regularized.
Numerical execution and comparison in wasm runtime remains PENDING;
compilation to the installed target is checked.

## Local verification

Final results:

- `cargo fmt` and `cargo fmt --check`: successful.
- `cargo clippy --all-targets -- -D warnings`: successful, with
  `/home/box/.cargo/bin` in PATH (installed stable component).
- `cargo test --workspace`: successful; 164 unit/integration tests
  and 35 doctests. The new crate contributes 15 integration tests and one doctest.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: successful.
- `cargo check --workspace --target wasm32-unknown-unknown`: successful,
  with `/home/box/.cargo/bin` in PATH and target already installed; nothing was downloaded.
- Native tests and rustdoc with system cargo/rustc 1.85.1, compatible with
  rust-version=1.85; Clippy and wasm with installed stable 1.99.0.
- `bash clean-room/check.sh origin/main`: successful. The script only sees
  paths known to Git; it was also run with temporary GIT_INDEX_FILE and
  GIT_OBJECT_DIRECTORY in /tmp and original objects as read-only alternates,
  including all new files in the reviewed diff.
  The user's real index and objects were preserved unchanged.
- `git diff --check`: successful, also on the full diff of new
  files through the temporary index.
- Final inspection: library code without unwrap/expect/explicit panics.
  Only Cargo.toml, Cargo.lock, the new crate and this log change;
  no modification to existing crates, contracts or clean-room/**.

During the first iteration, the fixture with both ends restrained left all DOFs
constrained, a situation LinearSystem explicitly rejects; it was subdivided
analytically into two elements with a free interior node. The kit's partial
rotation limitation described above was also detected, and isotropic conformance
was separated from own verification of the full anisotropic model.
Final results correspond to the corrected fixtures.

See PR #44. Independent clean-room review: PENDING (J4).
Numerical execution in wasm runtime: PENDING.

## J4 round r6

- Convention (decided by J0): the orientation vector is the reference for the local z axis (reference for the local z axis, x-z plane); y = z×x; coincides with +Z in 2D. Established in docs/CONVENCION_EJES.md.
- geometry.rs: sen(ángulo orientación–x) ≤ 1e-8 is rejected and the triad is reorthogonalized (Gram–Schmidt).
- release.rs: torsional equilibrium with relative tolerance 1e-12·máx|f|.
- stiffness.rs: axial response robust to underflow of E·ε.
- Tests: tests/r6.rs.
