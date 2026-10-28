# Clean-room log · ENG-ASM-001-KG

- Ticket / PR: real Kg(u) producer, branch `j2/geometric-producer`, stacked on
  the `has_geometric` change from PR #29. No commit, push or PR in this work.
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil).
- Role: implementer.
- Implemented contract: system/1, `LinearSystem::geometric`; adapter for
  element/1, kinds `truss2d.linear` and `truss3d.linear`.
- Permitted sources used: none external. Only code and contracts from the local
  tree; no network or bibliographic consultation.
- Original derivations: the sections of this same note.
- Third-party code copied: none.
- Validation: own analytical validation through energy, second variation of length,
  equilibrium and central differences of analytical incremental force.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados"
- Clean-room reviewer: PENDING (J4)
- Precise bibliographic citation for bar P-Delta: **PENDING**; pages
  or equations are not attributed to publications not consulted.

## Boundary, identity and atomicity

The type `GeometricProducer` owns `location: Vec<DofIndex>` and a closure
`Box<dyn Fn(&[Real]) -> EResult<DMat>>`, without Send/Sync. Its result must be
deterministic for each u_e; it does not modify history. Registration validates distinct,
nonempty and global indices. It evaluates at the expanded reduced reference,
including prescribed values, and validates each matrix through the same assembly path
as static contributions: squareness, size, finiteness and exact symmetry.

The API retains the literals `ElementContribution { .. }`. Since that API has no
element id, the producer identity is the exact `location` sequence,
including order. It replaces the static Kg of its sole matching contribution,
even if it was None. Without a match it adds an independent block. Two
producers with the same sequence or multiple matching contributions are rejected
with ENG-E-0002; no silent choice is made between matching elements.
A different permutation does not identify the same contribution. Clearing the registry
restores the static Kg matrices originally received.

Each `geometric(u,out)` validates the size of out and expands finite reduced u with
`constraints.expand`, extracts u_e, evaluates producers in registry order,
assembles in original contribution order followed by independent producers,
and reduces to free DOFs. Only then does it overwrite out and update trial. An error
in evaluation, symmetry, reduction or accumulation preserves out, trial and accepted.
Producer errors retain their code, message and entities. Registration
does not modify trial/accepted; a failure preserves the registry and previous pattern.
The backend's private internal state follows its own contract.

If any static Kg contribution remains without a producer, u must be exactly
equal to the reference (ENG-E-0013), even when Kg(ref)=0. With all
contributions replaced, any finite u of the correct size is accepted, subject to
Kg evaluation errors; Ku is not calculated to validate Kg, avoiding
rejections due to overflows unrelated to this operation. Size: ENG-E-0010;
nonfinite displacement: ENG-E-0008.

The pattern unites K, masses, original Kg and all free pairs of each producer,
including their zeros. Changing the registry rebuilds this union. The API
`global_geometric()` keeps its original static snapshot; the current reduced
Kg is queried through `SystemView::geometric`. The declared reference
only labels static snapshots, it does not store a new matrix.

The only dependency change moves the existing internal truss dependency
in assembly from dev-dependencies to dependencies. No package is added
to the graph and Cargo.lock is not changed. The crate
`engine-elements-frame` is not in this tree (PR #31 unmerged): it is not touched.

## Units and original second variation of length

Let r=x_j−x_i, L=|r|, d=r/L, q=u_j−u_i and k=EA/L. Coordinates and
displacements in m; E in Pa; A in m²; EA and N in N. For an incremental
perturbation v of r:

```text
l(v) = |r+v|
dl(0)[v] = d·v
d²l(0)[v,v] = (|v|²−(d·v)²)/L = vᵀ(I−ddᵀ)v/L.
```

The linear element calculates N(u_e)=k d·q, positive in tension. The incremental
work of the axial force contributes N d²l, hence:

```text
P = I−ddᵀ
Kg(u_e) = (N(u_e)/L) [ P  −P ]
                         [−P   P ]
K        = k             [ ddᵀ  −ddᵀ ]
                         [−ddᵀ   ddᵀ ].
```

Kg is symmetric and transverse to the axis; it has units N/m for these bars.
In compression N<0 it reduces transverse stiffness; in tension it increases it.
The adapter calls `LinearTruss::geometric_stiffness` with the received u_e and
empty committed state, rebuilding `ElementCtx` with owned geometry/params.
D2 follows Ux,Uy per node; D3 Ux,Uy,Uz. It does not update r,L,d with u.
General K/Kg with rotations have units N/m, N or N·m depending on the components.

## Axial bar with zero reference

With r=(2,0[,0]) m, E=200 Pa, A=2 m² and axial displacement δ of node j:
k=200 N/m, N=200δ N, transverse Kg=100δ N/m. For δ=0.06 m this gives
N=12 N and Kg_yy=6 N/m (also Kg_zz in D3); for δ=−0.04 m this gives
−4 N/m. Transverse displacement does not change the projected axial force.
Kg(0)=0 matches the static snapshot; without a producer, a query
δ≠0 retains ENG-E-0013 even if that snapshot is zero.

With prescribed u_ix=−0.06 m and zero reduced u_j, the extension is +0.06 m:
Kg(0 reducido)_yy=6 N/m. It is also compared against the static snapshot
evaluated with that same global u: nonzero match.

## Symmetric planar two-bar arch: incremental stiffness and singularity

The bases (−a,0) and (+a,0) are restrained; the apex is (0,h). This fixture uses
−Y as the local vertical load direction; it does not change the engine's global
convention. We define L=sqrt(a²+h²), c=a/L, s=h/L, k=EA/L.
The axes toward the apex are (c,s) and (−c,s). For the reduced state
u=(0,w), N=k s w in both bars. With w<0 they are compressed and the load
balancing the axial force in the held configuration is P=−2Ns along −Y.
The apex matrices are:

```text
K  = 2k    diag(c²,s²)
Kg = 2N/L  diag(s²,c²)
Kt = K+Kg.
```

To physically validate this sum, an incremental perturbation
v of the apex is considered, about the configuration where those matrices are evaluated.
For each bar a local energy with initial axial force N and constant modulus k is used:

```text
l(v) = sqrt((±a+v_x)²+(h+v_y)²)
V(v) = N(l(v)−L) + (k/2)(l(v)−L)²
f(v) = [N+k(l(v)−L)] (±a+v_x,h+v_y)/l(v).
```

Equivalently, l0=L−N/k and f(v)=k(l(v)−l0)r(v)/l(v).
This analytical force expression uses exact lengths under the perturbation;
its derivative at v=0 is k ddᵀ+(N/L)(I−ddᵀ). Summing the two bars gives Kt.
The test calculates f(v) independently of the matrix routines and compares
central differences in both directions with K+Kg(u). This is a local
analysis of prestressed bars whose N comes from the projected linear u_e.
It is not the derivative of `LinearTruss::respond` or `LinearSystem::internal_force`:
those operations remain linear, with derivative K. Nor does it represent
the exact large-displacement path from an initial global natural
length; such a path requires updating geometry and the axial law.

For the local multiplier buckling problem, K+λKg loses stiffness at:

```text
λ_cr,y = −EA s²/(N c²)
λ_cr,x = −EA c²/(N s²).
```

With a=2 m, h=1 m, EA=400 N, w=−0.1 m, L=√5 m,
c²=4/5, s²=1/5, N=−8 N, P=16/√5 N. This gives λ_cr,y=12.5,
λ_cr,x=200. The first mode is vertical/transverse [0,1], with positive
horizontal stiffness. At λ=12.5 vertical stiffness vanishes and changes sign
between 0.99λ and 1.01λ. The local critical load is λ_cr P=200/√5 N.
The residual of that mode and the sign change are verified. It is not presented as
an Euler column: the pin-connected bars have no bending stiffness.

## Tests and explicit tolerances

- `engine-elements-truss/tests/geometric_producer.rs`: D2/D3 bar and oblique
  global D3 projector with L=7 m, d=(2,3,6)/7 and N=4 N; change
  with N(u), zero and nonzero snapshot with prescribed values, owned context,
  adapter validation, arch and conformance with samples away from the reference.
  Formulas and critical residual: absolute tolerance 1e-12 N/m (and 1e-12 in
  the compared scalar quantities). Incremental force through central
  differences: step 1e-5 m; relative error <1e-6 both in the maximum norm of the
  full matrix and in the separate vertical component.
- `engine-assembly/tests/geometric_producers.rs`: ordered extraction of u_e,
  prescribed values, replacement without double counting, exact symmetry, overwriting,
  mix of static contributions and producers, restoration on clearing, dynamic pattern,
  Rc without Send/Sync, nonfinite inputs, sizes/asymmetries, overflowed sums,
  nonassociative accumulation order and error propagation after partial
  evaluations preserving out/trial. Simple binary values: exact equality.
  `check_system_view` uses step 1e-5 m and its contractual relative threshold 1e-6.
- The existing regression `zero_reference_geometric_still_rejects_other_states`
  retains ENG-E-0013 without a producer even with Kg(ref)=0.

## Local verification

Final run with toolchain `stable-x86_64-unknown-linux-gnu`, requested PATH
and `CARGO_NET_OFFLINE=true`, all with exit 0:

```text
cargo fmt --all -- --check
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo test --offline --workspace
RUSTDOCFLAGS='-D warnings' cargo doc --offline --workspace --no-deps
cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown
git diff --check
```

130 unit/integration tests and 33 doctests passed in the workspace,
including 13 new tests (8 assembly and 5 bar tests). Compilation of
tests for wasm also passed; they were not run in a wasm runtime.

Execution of the clean-room guard against origin/main is omitted because the
user requires no commits to be created and that guard is requested after commit. No changes
are made to clean-room/**, .github, CODEOWNERS or docs/CLEAN_ROOM.md.
