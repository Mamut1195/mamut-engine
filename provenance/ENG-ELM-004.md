# Clean-room log · ENG-ELM-004

- Ticket / PR: ENG-ELM-004; local implementation, no commit or push.
- Author(s): J2 (agent, Grok Bot), session executed by Codex (GPT-6),
  2026-10-11 (America/Guayaquil). The requested attribution is retained.
- Role: implementer; own analytical derivations.
- Implemented contract: element/1 · kind = `spring.linear`.
- Permitted sources used: no external sources. Only internal workspace contracts, patterns
  and documentation listed below.
- Own derivations: [this note](ENG-ELM-004.md), next section.
- Third-party code copied: none.
- Validation: analytical, equilibrium and energy; own fixtures in
  `crates/engine-elements-spring/tests/fixtures/mod.rs`.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Internal documents consulted

`provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md`, `docs/CONVENCION_EJES.md`,
`provenance/ENG-ELM-002.md`; structure, implementation and fixtures of
`engine-elements-truss` and `engine-elements-frame`; contracts, types and conformance
kit of `engine-contracts`; types/DOFs of `engine-core`; nodal model of
`engine-model`; assembly, constraints, contributions, system and backend
of `engine-assembly` and `engine-linalg`; `clean-room/check.sh`.
There is no applicable AGENTS.md. No network, external sources, code from
other engines, oracle execution or oracle results were used.

## Own derivations

### Energy, stiffness and internal force

Let c be the constant coupling vector per node in global axes. For a chosen global
DOF, c is its canonical vector. For several selected local DOFs,
c_local has a one per selected component; they all share **one**
generalized deformation, not independent stiffnesses. Only components
of the same family are added to preserve units (translations or rotations).
For independent springs, several elements are created, which the assembler adds.

Between two nodes, δ=c·(u_j−u_i) and b=[−c,c]. To ground, δ=c·u_i and b=c.
The support's only node represents the positive moving end; the other end
is stationary and is not part of the context or DOF map.

```text
W = ½ k δ²,       k ≥ 0
p = k δ
f_int = b p
K = k b bᵀ
```

For one DOF between nodes K=k[[1,−1],[−1,1]], and to ground K=[k].
For two selected global DOFs, the block per node is k[[1,1],[1,1]]:
there is coupling and rank one. Components outside the selected mode
remain free, without regularization; other elements or constraints must
eliminate mechanisms when solving an invertible system is needed.

K is symmetric, positive semidefinite and constant. Between nodes, any uniform
translation gives δ=0 and K·u=0; uniform rotations do so as well for rotational springs.
For separated nodes, a rigid rotation may change the projection onto a constant
axis and deform a translational spring: it is not a corotational element.
The ground support eliminates the associated rigid mode by construction.
There is no updated geometry, initial stress or second-order term:
Kg is zero. The ideal spring has no kinetic energy: zero mass for both
MassKind. None is used for both, as element/1 allows, validating inputs.

### Orientation and recovery

Global retains the chosen DOFs, in engine-core canonical order. Rotation
receives Q global→local, with local axes as rows; Q Qᵀ=I and det(Q)=+1,
absolute tolerance 1e-12. D2 preserves XY and +Z exactly. Then
c_global=Qᵀc_local, for both translations and rotations. All global
DOFs of that family are activated, even those with zero coefficient, so as
not to lose components when rotating. The static subset table satisfies the
&'static [Dof] signature without memory leaks or mutable global registries.

Direction only accepts local Ux and computes c_global=d/||d||. It does not define an
arbitrary transverse orientation: the other local resultants are zero.
d is normalized by first dividing by max|d_i|, then by the norm of the scaled
vector obtained with hypot. This accepts finite MAX or subnormal
components without squaring them or overflowing the original norm. A zero/non-finite
direction and a D2 direction outside XY are diagnosed.

Reversing d changes the signs of δ and p, but leaves K and global force invariant.
Rotating the model, displacements and its own axis by R gives f'=R f and K'=R K Rᵀ,
with R repeated in nodal blocks. The explicit matrix transforms as
Q'=Q Rᵀ. Invariance is not expected if the model is rotated while its axis stays constant.

Recovery: local actions [−c_local p,c_local p] between nodes and
[c_local p] to ground, according to the active DOFs. Each station provides the six
canonical components [N,Vy,Vz,T,My,Mz]=c_local p. Positions [0,1] are
contract labels, not integration over length; order and duplicates are
preserved. Nonzero distributed load is not supported: nodal loads are applied.
Length, axis and gravity are not inferred from coordinates, and coincident
coordinates are allowed. D2 requires Z=0 and all context must be finite.

### Assembled closed-form solutions

Own SI fixtures: k1=120 N/m, k2=80 N/m and F=12 N.

1. Series: equilibrium requires p1=p2=F; u_intermedio=F/k1 and
   u_final=F(1/k1+1/k2). Therefore k_eq=1/(1/k1+1/k2)=48 N/m,
   u_intermedio=0.1 m, u_final=0.25 m, base reaction −12 N.
2. Parallel: both have the same δ, F=(k1+k2)δ; k_eq=200 N/m,
   u=0.06 m, forces 7.2 N and 4.8 N, base reaction −12 N.
3. Ground support: ku=F, u=0.1 m, internal force at the node +12 N.
4. Oblique support: one spring k1 along d and another k2 perpendicular to it eliminate
   the transverse mechanism. Under Fd the exact solution is u=(F/k1)d.
   It is checked at three angles without adding artificial stiffness.

All are assembled with DofMap, ElementContribution and LinearSystem; the solve
uses ReferenceBackend. Member forces are recovered after expanding
constrained displacements. There is no dedicated solver in this crate.

## Architecture, API and diagnostics

- Crate 0.1.0, workspace member, path dependencies on core/contracts/linalg,
  and dev on assembly/model. No new external dependencies. Private modules
  factory, geometry, stiffness, recover, mass, geometric and loads as in the
  internal templates; API and validators in lib.rs; deny(missing_docs).
- LinearSpring::new(dim, connection, k, dofs, axes) accepts one or several distinct
  DOFs of the same family and validates immutable properties;
  ctx.params is not involved. The bar Elastic law is not used: it requires a positive
  modulus, whereas spring.linear accepts k=0 and does not represent a stress in Pa.
- SpringFactory::new(dim): k and dofs required; optional boolean ground false;
  dofs contains exactly one local DOF of the selected dimension,
  as required by the JSON schema (minItems=maxItems=1). Empty lists or lists with
  more DOFs than the dimension are rejected; several valid distinct DOFs produce
  ENG-E-0013 to avoid unrequested coupling.
  Optional direction and rotation, mutually exclusive; without them, it uses global
  axes. Own in-memory JsonValue, without parser. JSON schema and validation
  of type, range, unknown fields, dimension and ModelView as in truss/frame.
- ENG-E-0008: non-finite inputs or results, including overflowing products/sums and
  transformed stiffness; ENG-E-0010: ranges, sizes, duplicate DOFs,
  empty DOFs, DOFs outside the dimension or mixed families, zero direction, non-
  orthonormal/right-handed matrix, D2 plane, unsupported loads and stations;
  ENG-E-0002: duplicate nodal ids in ModelView;
  ENG-E-0013: several valid distinct DOFs in the factory. No new codes.
- No panic/unwrap/expect in the library. The number of DOFs is at most six
  per node and all internal indices depend on already validated inputs.
  k=0 disconnects exactly after validating u and history; it avoids overflowing
  differences that do not affect the output. Tiny k is not replaced by a threshold;
  IEEE underflow of projections/products may produce finite zeros.
  Representability of every extreme calculation is not guaranteed: overflow gives a
  diagnostic. No mutable state, I/O, clock or threads.

## Tests and tolerances

- Four assembled analytical tests and twelve for invariants/conformance,
  negatives, factory, energy, recovery, mass/Kg and numerical extremes;
  one doctest in Spanish. All fixtures cite this own derivation.
- Displacements: absolute 1e-12 m; assembled forces and reactions: 1e-9 N.
  Invariant matrices/forces/energy: absolute 1e-10 in their SI unit.
  Central finite differences: h=1e-7 m/rad, absolute error <1e-6 in entries
  of K; the kit requires its relative error <1e-6.
- Kit check_element and check_element_factory: global Ux/Rz springs in D2/D3,
  with compatible rigid modes; repeatability, empty state, symmetry and FD.
  A k=0 support also runs through the full kit. Grounded k>0 cases and own
  oblique axes use explicit FD/symmetry/covariance/mass/Kg/recovery,
  because the kit assumes no ground support and rotates scalar parameters
  without rotating their own global orientations. Kit diagnostics are not filtered
  and engine-contracts is not altered to make physical exceptions pass.
- Oblique D2/D3 direction, reversals and scales 1e±200; spatial covariance
  of force and independent congruence Q K Qᵀ; matrix/direction equivalence,
  coupled translational/rotational DOFs, energy and null modes between nodes.
- k=0, MAX, 1e-300 and minimum subnormal; MAX/minimum subnormal direction;
  overflow of kδ, differences and rotated stiffness; invalid parameters, coordinates,
  displacements, loads/stations and JSON, all with diagnostics.

## Local verification

Results of the initial implementation, all successful:

- `cargo fmt` and `cargo fmt --check`.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo test --workspace`: all unit tests, integration tests and
  doctests; the new crate contributes 14 integration tests and one doctest.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`.
- `cargo check --workspace --target wasm32-unknown-unknown`: installed toolchain
  via `/home/box/.cargo/bin` in PATH; no tools/targets were installed.
  `cargo check -p engine-elements-spring --tests --target
  wasm32-unknown-unknown` also passes. Fixture execution in wasm runtime and
  numerical comparison between targets: PENDING; not attributed to a check.
- `bash clean-room/check.sh origin/main`: passes in the working tree and with
  all new files included in a temporary index. GIT_INDEX_FILE and
  GIT_OBJECT_DIRECTORY in /tmp were used, with the existing objects
  as read-only alternates. The user's real index was not changed.
- `git diff --check`, also with new files in the temporary index.
- Local JSON syntax check of the D2/D3 schemas with the Python standard
  library; search for panic/unwrap/expect in the new library
  with no matches. No external dependencies or network access.

Only Cargo.toml, Cargo.lock, the new crate and this log were modified.
Existing crates, clean-room rules and protected paths were not modified.
No commit or push was made. Clean-room reviewer: PENDING (J4).

## J4 round r7

- The `spring.linear` factory accepts exactly one DOF per element and rejects
  several valid distinct DOFs with ENG-E-0013: with several DOFs the spring has
  rank one (K=k·bbᵀ) and couples the DOFs; for independent springs, one
  element per DOF is used. `LinearSpring::new` retains the documented coupling.
  Test `factory_rejects_multi_dof_springs_that_would_couple_dofs` for 2..3 DOFs
  in D2 and 2..6 in D3, with no changes to the flow or error codes.

## Diagnostic correction · 2026-10-13

- The factory diagnostic for empty lists or lists of more than six entries
  indicates that it requires exactly one DOF of the selected dimension.
  `factory_empty_dofs_reports_exactly_one_dof_required` pins the message and
  ENG-E-0010 for the empty list in D2 and D3.
- Spanish documentation of the factory, lib.rs and this log: exactly
  one DOF in the factory; one or several of the same family in the direct API,
  without modifying `LinearSpring::new` or the factory validations.
- Offline verification (`CARGO_NET_OFFLINE=true`): `cargo fmt`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace` and
  `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace` pass.
  The crate now contributes 16 integration tests and one doctest.
- `cargo check --workspace --target wasm32-unknown-unknown` passes (with the
  target installed in the local environment) and `bash clean-room/check.sh origin/main` gives OK.
- No commit or push.
