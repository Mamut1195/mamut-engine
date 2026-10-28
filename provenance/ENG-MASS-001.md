# Clean-room log · ENG-MASS-001

- Ticket / PR: ENG-MASS-001 «Masas»; branch stacked on ENG-LOAD-001, unmerged PR #51. No commit or push in this session.
- Author(s): J2 (agent, Grok Bot), 2026-10-11 (America/Guayaquil), label requested by the user. Execution model: Codex based on GPT-6.
- Role: specifier and implementer through own derivations; no auditor or oracle operator activity.
- Implemented contract: public module `engine-loads::mass`; reuses `engine-model::NodalMass`, `DofMap`, `Element::mass`, `engine-contracts::MassKind` and `LinearSystem::global_mass`. No new DOF numbering or parallel element formulation.
- Permitted sources used: exclusively the user's specification, documentation, contracts and relevant internal code of this repository. No external sources, network queries or oracle.
- Own derivations: equations (1)-(4) of this note, developed for this ticket.
- Third-party code copied: none. No new external dependencies; only path dependencies on existing workspace crates.
- Validation: analytical, through kinetic energy, closed-form mass tables and force/weight-to-mass conversion. Fixtures with `SOURCE` and `CITATION` in `crates/engine-loads/tests/mass.rs`.
- Oracle comparison (sibling repo, informative): no. No external engine was consulted or run.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Clean-room reviewer: PENDING (J4).

## Location and dependencies

`crates/engine-loads/src/mass.rs` is chosen: engine-loads already depends on
engine-assembly, and engine-assembly does not depend on engine-loads.
engine-linalg is added by path to name the existing CSR matrices and their backend
contract; this introduces no cycles or external dependencies. engine-elements-truss
is added only as a test dependency, alongside the existing frame.
The `Element::mass` producers were read before implementing the module:
both matrices are global and LinearSystem already adds nodal masses to both
MassKind. That path is unchanged and element mass is not added again.

## Own derivations

### (1) Kinetic energy and total mass per direction

For a nodal velocity v, T=½vᵀMv. A rigid translation with unit
velocity d̂ has the same vector d̂ in each node's translations and zero
rotations: v=r_d. A bar or beam of constant section and density has
T=½∫rho·A·|d̂|²dx=½rho·A·L. Therefore r_dᵀM_elemento r_d=rho·A·L.

An anisotropic diagonal nodal mass contributes Σ_a m_n,a·d̂_a²; a nodal inertia
does not contribute because its angular velocity is zero. Summing contributions:

masa(d)=r_dᵀM_global r_d=Σ_n Σ_a m_n,a·d̂_a²+Σ_e rho_e·A_e·L_e.

For d=X,Y,Z, only the masses of that component are selected.
An isotropic nodal mass contributes its scalar once per direction, never
three times. The full global matrix is used, including restrained DOFs:
the support-reduced matrix does not represent a complete rigid translation.
All nodes are required to declare the nonzero translations of d̂;
eliminating a missing component would give a lower apparent mass and is rejected.

### (2) Analytical tables for bar and beam

For two nodes with axial interpolation t=1−x/L and r=x/L,
∫t²dx=∫r²dx=L/3 and ∫tr dx=L/6. Each direction of a bar has
M_consistente=(rho·A·L/6)[[2,1],[1,2]]. The sum against [1,1] is rho·A·L.
Lumped mass distributes half at each end by symmetry:
M_concentrada=(rho·A·L/2)I, which satisfies the same identity.

For the beam, the transverse Hermite interpolation is
H=[t²(1+2r), Lrt², r²(1+2t), −Lr²t]. Integrating rho·A·HᵀH dx through
∫₀¹r^k dr=1/(k+1) gives the matrix, in order [v_i,theta_i,v_j,theta_j]:

```text
(rho A L/420) *
[ 156      22L      54     -13L   ]
[  22L      4L²     13L     -3L²  ]
[  54      13L     156     -22L   ]
[ -13L     -3L²    -22L      4L²  ]
```

With zero rotations and v_i=v_j=1, its sum is (156+54+54+156)/420=1.
The transverse translational trace is 312/420 times the mass and cannot be used to
compute total mass. Tests use rho=3, A=2, L=4: masa=24 kg,
consistent axial diagonal 8 kg, axial coupling 4 kg and lumped mass 12 kg
per end/direction. The Hermite table is checked with independent literals.
For releases, the elastic transformation H_release preserves a
rigid translation, because v_i=v_j and theta_i=theta_j=0 give zero released rotations;
therefore rᵀH_releaseᵀ M H_release r=rᵀMr. Tests cover no releases,
one released end and both released ends.

### (3) Change of orientation

An orthogonal rotation Q preserves |Qd̂|²=|d̂|² and L. Isotropic element
mass preserves rho·A·L in each direction, even if the entries of
the beam's global matrix change. The bar is checked along X, Y, Z and
the vector (4/3,8/3,8/3), all of length 4, with a translated origin.
The beam is checked with endpoints (4,0), (0,4) and (2.4,3.2).
An oblique direction is normalized before evaluating rᵀMr. For anisotropic nodal
masses, the expression in (1) depends on direction and isotropy is not
assumed. A fixture mixes element mass with nodal masses of 15 kg in X,
13 kg in Y and inertia 19 kg·m²: totals 39 and 37 kg; for d=(3,4,0),
24+15·9/25+13·16/25 kg.

### (4) Mass derived from a case

The user provides direction d≠0, factor f≥0 and g_usuario>0, all finite.
d is normalized using s=máx|d_a|, d̂=(d/s)/|d/s|, avoiding overflow
of the norm for large components and its loss for subnormal directions.
The projection of weight at each node is |F_n·d̂|. From peso=m·g:

m_n=|F_n·d̂|·f/g_usuario.

F_n is the net global force per node of the case; element and nodal
contributions are summed first and then the absolute value is taken. The
model's global resultant is not converted, nor is the absolute value of each contribution taken before
summing it. Opposite forces at different nodes contribute positive masses.
Moments are validated as finite but do not produce scalar mass. The obtained
mass is assigned to each declared translation of the node, without rotational inertia.
The direction selects the weight source; this assignment is isotropic in
the available translations. There is no code factor or default gravity.

Fixture: Fz=−120 and +80 N at two nodes, d=−Z, f=0.25, g_usuario=10 m/s²:
masses 3 and 2 kg, total 5 kg per direction. With equivalent element forces
of +20 and −40 N on those DOFs, the net values are −100 and +40 N: 2.5+1=3.5 kg.
Force X=30 N and moment Z=999 N·m do not contribute to the −Z projection.
It is verified that the factor is applied only once and that conversion does not
modify existing masses. Joint rotation of F and d and an explicit change
of gravity are tested. Cases without loads and zero factor give zero masses.

## Errors and numerical limits

- ENG-E-0010: negative masses/inertias or factors; g_usuario≤0; zero direction;
  incompatible matrix/vector size; positive mass lost to zero in
  multiplication by factor or division by g; failed working vector allocation.
- ENG-E-0008: NaN, infinities or overflow in inputs and results, including
  sum of masses per DOF, projection, multiplication by factor, division and rᵀMr.
- ENG-E-0003/0011: nonexistent node/case or DOF. ENG-E-0002: repeated ids.
- Repeated components within a mass and different assignments to the same
  node/DOF are summed, according to the existing LinearSystem semantics.
- No panic, unwrap or expect in the new library code. Direct
  tests with MAX, NaN, infinities, minimum subnormal, zero direction and empty
  lists: any panic would make the test fail.
- The directional measure requires a physical matrix validated by its producer;
  it checks size, nonnegative diagonals and finiteness of the result, but does not
  certify symmetry or positive semidefiniteness of an arbitrary CSR matrix.
- Overflowed intermediate products are rejected even if a reordering could
  make the result representable. IEEE-754 does not guarantee relative precision
  of subnormal projections or recover direction components
  lost when dividing by their scale.

## Metadata of the three tickets

«los tres tickets» is interpreted as ENG-GEO-001, ENG-LOAD-001 and ENG-MASS-001.
The requested metadata is updated; the historical evidence of
ENG-GEO-001 is preserved, including its earlier bibliographic consultation. This session does not
consult that source or any other external source, and all its mass
tests are own analytical derivations.

## Session verification

All Cargo commands were run without network access (`CARGO_NET_OFFLINE=true`).

- `cargo fmt`: OK, exit 0.
- `cargo clippy --all-targets -- -D warnings`: OK, exit 0.
- `cargo test --workspace`: OK, exit 0; 220 workspace tests and doctests,
  including 9 new analytical mass tests and 1 new doctest.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, exit 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, exit 0,
  with `PATH=/home/box/.cargo/bin:$PATH`, already installed rustc 1.99.0 toolchain.
  Native commands used rustc 1.85.1 (workspace MSRV). No
  components were downloaded. This check verifies wasm compilation,
  neither execution nor numerical equality between targets.
- `bash clean-room/check.sh origin/main`: OK, exit 0; includes the three new
  files through `git add -N` in a temporary index and objects in `/tmp`,
  without modifying the real index. `git diff --check` on that index: OK.
  Record: `/tmp/eng-mass-001-clean-room.log`.

Cargo records are at `/tmp/eng-mass-001-{fmt,clippy,test,doc}.log`.
No commit or push. Clean-room review: PENDING (J4).
