# Clean-room log · ENG-ELM-005

- Ticket / PR: ENG-ELM-005; local delivery without commit or push. Stacked on
  ENG-ELM-004 (`spring.linear`) and ENG-MAT-002 (`multilinear_elastic`).
- Author(s): J2 (agent, Grok Bot), requested attribution; session executed by
  Codex (GPT-6), 2026-10-11 (America/Guayaquil). No delegation to other agents.
- Role: implementer; own analytical derivations and fixtures.
- Implemented contract: element/1 · kind = `spring.nonlinear`.
- Permitted sources used: user request and internal code/documentation
  listed below. No external sources; no network, external books or manuals.
- Own derivations: [this note](#derivaciones-propias-y-fixtures) and
  [analytical tests](../crates/engine-elements-spring/tests/nonlinear.rs).
- Third-party code copied: none. No new crates or dependencies,
  including development dependencies; manifests and lockfile unchanged.
- Validation: analytical; closed form, manual Newton, reversible path,
  covariance, kit finite differences with history and invalid inputs.
- Oracle comparison (sibling repo, informational): no. No external
  engine was run and no oracle results were used.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados"
- Clean-room reviewer: PENDING (J4).

## Internal documents consulted

`provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md`, `provenance/ENG-ELM-004.md`,
`provenance/ENG-MAT-002.md`; workspace manifests; existing implementation and
tests of `engine-elements-spring` and `engine-elements-truss`; types, traits,
examples and conformance kit of `engine-contracts`; implementation and tests of
`engine-laws::uniaxial`, including `multilinear_elastic`; types of `engine-core`;
state references in `engine-assembly`; `clean-room/check.sh`.
There is no applicable AGENTS.md in the repo or its ancestors.
No sources or paths of prohibited engines were opened.

## Derivaciones propias y fixtures

Let c be the constant global coupling, with the same selection, normalization,
rotation and canonical DOF order as in ELM-004. Between nodes:
δ=c·(u_j−u_i), b=[−c,c]. To ground: δ=c·u_i, b=c.
The law is supplied as **force-displacement**, with unit length and area:

```text
ε = δ                 σ = F(δ, historia_aceptada)
f_int = b F           K = (dF/dδ) b bᵀ
historia_elemento = historia_ley
```

For translations: δ in m, F in N, tangent in N/m. For rotations: δ in rad,
F represents moment in N·m and the tangent is N·m/rad. There is no implicit
conversion of a material table in Pa/dimensionless to force/displacement.
Selecting several DOFs of one family adds their projections and produces a single
coupled mode; they are not independent springs. The tangent is evaluated at every
trial, without replacing it with the initial tangent or a secant.

K is symmetric by construction, even for laws with zero or
negative tangent; the element does not restrict the constitutive range of a compatible law.
Between nodes a uniform translation does not modify δ. A ground support eliminates
that rigid mode. A constant axis between separated nodes is not corotational and may
deform under rigid rotation; mass and Kg are absent. These limitations are inherited
from ELM-004. Coincident coordinates are allowed, without physical length.

When rotating the model, u and its own axis with R, δ is preserved, f'=R f and K'=R K Rᵀ
with nodal blocks. An axis reversal changes the active side of an
asymmetric law: invariance under that reversal is not promised. Recovery:
local actions ±c_local F and constant resultants at stations [0,1],
which are labels without spatial integration. Only zero member
loads are allowed; loads applied to the spring are nodal.

### Reversible bilinear

Own fixture: points (δ,F) in m/N
`(-0.5,-18), (-0.1,-10), (0,0), (0.1,10), (0.5,18)`.
Thus F=100δ for |δ|<0.1 and
F=sign(δ)[10+20(|δ|−0.1)] outside that interval.
Generalized K is 100 N/m in the central segment and 20 N/m in the outer ones.
Both signs, extrapolation, coincident nodes, uniform displacements,
recovery and ground/between-node connection are tested.
Newton for a +16 N load to ground starts at u=0: u₁=0.16 m,
u₂=0.4 m; F(0.4)=16 N. The loading and unloading path reproduces
the same force and tangent when repeating δ, without dissipated energy or history.

### Compression-only gap

Own fixture: points `(-1.25,-100), (-0.25,0), (0,0)` in
`CompressionOnly` mode. F=100(δ+0.25) for δ<−0.25 and F=0 for δ≥−0.25.
The tangent is 100 N/m in contact and zero in the open gap.
It never gives positive force: the support does not pull. At −0.25 the MAT-002
convention is respected (right segment with zero tangent). There is no two-sided derivative there.
Newton for a −20 N load starts in contact at u=−1 m and converges to
u=−0.45 m. In the gap, the zero tangent requires a contact strategy from the
analysis; this element does not add regularization or modify the law.

### Nonempty history and conformance

Analytical plastic law **for tests only**, with k=100 N/m and limit |F|≤10 N:
history p=plastic displacement; predictor F_trial=100(δ−p).
If |F_trial|≤10: F=F_trial, tangent 100 and p_trial=p.
Otherwise: F=sign(F_trial)10, tangent zero and p_trial=δ−F/100.
At δ=0.2 m from p=0, F=10 N and p_trial=0.1 m are obtained.
Discarding the trial leaves F(0)=0; accepting it reproduces F(0.2)=10 and then
F(0.15)=5 N, whereas without acceptance F(0.15)=10 N.
Determinism, independent copies of initial history and absence
of acceptance in respond, recover, Kg and failures are verified.

`check_element` runs in D2/D3 with nonempty plastic history and both
multilinear laws. Global DOF Ux and coincident coordinates allow applying
the kit's rigid modes without exceptions. Samples with explicit acceptance
are δ=0.02, 0.15, 0.12, −0.03, −0.4 m. −0.05 is avoided because, after
p=0.05, it coincides with the plastic unloading breakpoint: a central difference
there does not represent the one-sided tangent. Diagnostics are not filtered and the
kit is not modified. Covariance of own axes and ground supports is checked with
separate fixtures, in accordance with the documented limitations of the existing kit.

## Architecture, API and diagnostics

`NonlinearSpring::new` retains an already resolved `Arc<dyn UniaxialLaw>`;
`from_params` obtains it from `ElementParams::law`. As in the existing bars,
the constitutive property lives in the element, without changing it from ctx.params.
The uniaxial/1 contract (compatible minor versions), initial history
and initial tangent are validated. `state_len` and `initial_state` describe exactly the
validated constitutive history, without added displacements or forces.

There are no new commit/revert methods in element/1: history is owned by
the caller. `respond` returns trial_state; after converging the caller replaces
its accepted state with that trial. Revert discards the trial and retains the
last accepted state. The element contains no mutable trial buffers.

Private composition with unit-stiffness `LinearSpring` reuses
projection (its generalized force equals δ), context, DOFs, mass and loads.
The matrix and recovery helpers now receive the constitutive tangent and
force; the public behavior of `spring.linear` is preserved.
The response also validates the size and finiteness of trial history, force
and tangent, even if the supplied law violates its contract.

ENG-E-0010: missing/incompatible law, sizes, geometry, DOFs, axes, stations
or other unsupported inputs. ENG-E-0008: non-finite inputs, initial/accepted/
trial history or results and projection/force/stiffness overflow.
Law errors propagate without changing their code. No panic/unwrap/expect
in the library; private indices come from previously validated geometry.

The crate did not depend on engine-laws and the prohibition on adding
any dependency is respected. The test integrates **the actual existing file**
`engine-laws/src/uniaxial/multilinear.rs` through `#[path]`, without copying or
rewriting its interpolation. An adapter private to the test binary supplies
the finite/size validators and auxiliary descriptor and factory interfaces
that the module imports. This does not change the library or production API
or add dependencies. MAT-002 conformance is still also tested in
its original crate during `cargo test --workspace`.

## Tests and local verification

Ten new integration tests and one doctest in Spanish. Absolute tolerance
1e−9 in forces, displacements and matrices of these SI fixtures; Newton requires
residual <1e−10 N; the kit uses h=1e−7 m and requires FD relative error <1e−6.
Negatives include incompatible law, invalid initial history, accepted
history of incorrect length/non-finite, defective constitutive outputs,
propagation of law errors, invalid displacements/context/axes/loads/stations,
relative differences and overflowing forces.

Final checks, all successful, with `CARGO_NET_OFFLINE=true`
and the toolchain installed at `/home/box/.cargo/bin` (no installations):

- `cargo fmt` and `cargo fmt --check`.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo test --workspace`: all unit tests, integration tests and
  doctests; the ten new tests and the doctest pass.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`.
- `cargo check --workspace --target wasm32-unknown-unknown`: successful
  compilation; no numerical tests were run in a wasm runtime.
- `bash clean-room/check.sh origin/main`: `Clean-room: OK`, also including
  the three new files through a temporary index and object directory
  in `/tmp`; the user's real Git index and objects unchanged.
- `git diff --check`, including the new files in that temporary index.
- Search for `panic!`, `unwrap` and `expect` in the library with no matches.

Only six files are modified: lib.rs, stiffness.rs and recover.rs in the crate,
the nonlinear.rs module, its tests and this log. No changes to contracts,
laws, manifests, lockfile or clean-room rules. No external sources, network,
oracles, commit or push. Clean-room reviewer: PENDING (J4).

## J4 round r7

- The tests use the actual `engine_laws::uniaxial::MultilinearElastic` law as an internal dev-dependency (path) instead of including its source through `#[path]`; unused support code is removed. Cargo.lock regenerated (internal dependency only).
