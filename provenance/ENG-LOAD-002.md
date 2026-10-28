# Clean-room log · ENG-LOAD-002

- Ticket / PR: ENG-LOAD-002 «Combinaciones lineales y envolventes». Branch stacked
  on ENG-RES-001 (#53, `crates/engine-analysis::results`) and ENG-LOAD-001
  (#51, `crates/engine-loads`), both unmerged. No commit or push.
- Author(s): J2 (agent, Grok Bot) · identification requested by the user;
  executing agent Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Role: implementer and author of own analytical derivations.
- Implemented contract: module `engine_analysis::combination`, superposition
  on `CaseResults` using `engine_model::Combination`/`CaseFactor`, and
  algebraic envelopes with governing entry per component and extremum. No new
  serialization formats, new crates or new external dependencies.
  `engine-model`, already present as a test dependency, becomes a regular
  dependency to reuse its combination declaration.
- Permitted sources used: ticket request and in-house APIs of `results`,
  `engine-model`, `engine-core`, `engine-contracts`, `engine-assembly`,
  `engine-elements-frame`, `StaticLinear` and existing reference backend.
  Template: `provenance/TEMPLATE.md`. No network access or external sources.
- Own derivations: [note in this log](#derivaciones-propias) and
  [analytical tests](../crates/engine-analysis/tests/combination.rs).
- Third-party code copied: none.
- Validation: exclusively analytical; superposition, equilibrium and closed-form
  cantilever beam solution. No external sources or oracles.
- Oracle comparison (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

## Contract and decisions

`combine(&Combination, &[CaseResults])` provides `CombinationResults`, with its own
id and reused `CaseResults` (`case` = combination id, `factor` = 1).
It does not reapply the load factor stored in each case: those values are already
scaled. The user's terms are the only coefficients. No factors
from any code are introduced. Each value is summed in the order of the
`CaseFactor`, with separate f64 multiplication and addition, without FMA or interpolation.
Each product and partial sum is checked for overflow; intermediate
overflow must not be hidden by subsequent cancellation.

Nodes and members are matched by id, regardless of their order in the
input; output is sorted by id. End actions preserve their local
order. Diagrams preserve exactly the original positions. Station count
and positions must match, without tolerance; there is no interpolation or
merging of different grids. Range [0,1], strict order and endpoints
0/1 are checked. Exclusively nodal results, without members, are allowed; each input
requires nodes. Zero-factor terms are validated like the others.

`envelope(&[EnvelopeInput])` accepts cases and combinations, with `ResultId::Case`
and `ResultId::Combination` to distinguish identical names. Each nodal DOF,
end action and station resultant has algebraic maximum and minimum,
with value and governing identity. Global member extrema also have
position x/L. Each input's extrema are recalculated from
its stations, without trusting the public `MemberResult::extremes` cache.

Exact ties retain the **first input in the user's order**,
without sorting by name or preferring cases over combinations. Within the same
member the first station (smallest x/L) is retained. An envelope maximum/minimum
may belong to a different input in each component: it does not represent
a simultaneous physical solution or an equilibrium case.

Diagnostics: ENG-E-0003 for a missing case, also with zero factor;
ENG-E-0008 for a nonfinite factor/datum or overflowed product/sum;
ENG-E-0010 for empty list, duplicates, inconsistent sizes/topology/stations.
The library adds no `panic!`, `unwrap` or `expect`. All referenced cases
are validated; cases outside the terms are checked only for duplication.

## Derivaciones propias

For constant stiffness K and homogeneous supports, K uᵢ = Fᵢ.
Multiplying each equation by the user's factor αᵢ and summing:

K (Σ αᵢ uᵢ) = Σ αᵢ Fᵢ.

Reactions R = K u − F, end actions and recovered resultants
are linear operators of displacements and loads; therefore the
same sum applies to each field, position and component. The extremum of the sum is
`max_s Σ αᵢ fᵢ(s)`/`min_s Σ αᵢ fᵢ(s)`, which generally differs from summing the
individual extrema. A test example uses two opposite diagrams:
their combination is zero at all stations although each isolated diagram
has nonzero extrema.

With nonzero prescribed displacements the load problem is affine:
u(F) = u(0) + L(F). Summing complete results contributes (Σ αᵢ) u(0).
The module performs the requested weighted sum of results, including
prescribed values; it does not automatically subtract the unloaded case. Equivalence to
solving combined loads while maintaining the same settlements requires
Σ αᵢ = 1, zero settlements or explicit treatment of the u(0) contribution.

The pointwise envelope of a field v is maxᵢ vᵢ/minᵢ vᵢ, storing the
argmáx/argmín. Strict comparisons `>`/`<` preserve the first governing entry
on a tie, even between +0 and −0. Global member extrema are
maxᵢ max_s fᵢ(s) and minᵢ min_s fᵢ(s), also preserving the station.

Extrema are limited to the available stations. Combining
sampled results does not allow deducing new interior critical roots of
arbitrary fields. Even if two cases have matching individual critical
stations, a combination may need other stations for the continuous
extremum; that calculation is not promised without additional recovery. The scope
of the discrete input diagrams is preserved exactly.

## Analytical fixture and tolerances

Horizontal cantilever beam, L=2 m, E=16 Pa, A=2 m², I=2 m⁴, no releases,
supports Ux=Uy=Rz=0. Case A: Pₓ=48 N at the free end. Case B: Pᵧ=−48 N
at the free end and uniform load qᵧ=−24 N/m. For each combination a case
D with loads αA+βB is assembled and solved through `StaticLinear` and
`EulerFrame2D`; all fields are recovered with `CaseResults::recover`.

Integrating N/(EA) gives uₓ(L)=PₓL/(EA)=3 m. For bending, integrating M/(EI) with
uᵧ(0)=θ(0)=0 gives:

uᵧ(L)=PᵧL³/(3EI)+qᵧL⁴/(8EI)=−5.5 m;
θ(L)=PᵧL²/(2EI)+qᵧL³/(6EI)=−4 rad.

These magnitudes serve as an algebraic fixture of the linear model, rather than
assumptions of a physical design with small rotations. For A: constant N=Pₓ.
For B: Vy(x)=Pᵧ+qᵧ(L−x), Mz(x)=Pᵧ(L−x)+qᵧ(L−x)²/2.
Reactions: Rₓ=−Pₓ, Rᵧ=−Pᵧ−qᵧL and Rz=−PᵧL−qᵧL²/2.
The member load uses the consistent actions already implemented by the
in-house frame2d. There are no numerical solutions or sources from another engine.

Factors [2,0.5], [−0.5,2], [0,−0.5] and [1.3,−0.7] are checked.
The binary test uses a variant with qᵧ=0: B has uᵧ=−4 m and θ=−3 rad.
For [2,0.5] and [−0.5,2], bit equality between the combination and case
D is required in all displacements, reactions, actions and stations. The same
comparison with zero axial factor uses algebraic equality within tolerance:
recovery may produce −0 when calculating N=−Ni, versus +0 from a sum
started at zero. The explicit ordered sum is verified bit for bit, including
zeros, negative factors, zero and nonbinary factors.

The variant with member load qᵧ=−24 uses
128·EPSILON·max(1,|esperado|,escala) to compare with D and the closed-form
formulas. Nodal DOFs use escala=1; end actions use the maximum
magnitude of both vectors; stations and their extrema use the maximum magnitude
of the same field in both diagrams. The analytical comparison uses
48|α|, 96|β| and 144|β| for N, Vy and Mz respectively. Thus, residuals in
fields whose theoretical value is zero are judged relative to the actions that
produce them. The nodal variant uses 128·EPSILON·max(1,|esperado|) for nonbinary
factors and extrema. When mixing nodal and member loads, numerical scaling and
solver recovery may vary by a few ULP between solving D and
superposing, even with power-of-two factors; the combination still
matches the ordered sum of input results bit for bit.

Envelope tests use different governing entries per DOF, field and sign,
an interior peak, cases/combinations with identical names, a governing
combination, reversal of tie order, and tied stations. Checks cover
stations differing by one ULP, empty lists, duplicates, topology, sizes,
NaN/inf, invalid zero terms, ±MAX values, overflowed sum/product,
cancellation of finite extrema and subnormals without panic.

## Session verification

Final results of this session, all with exit 0 and no network access
(`CARGO_NET_OFFLINE=true`):

- `cargo fmt`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, including the 11 new combination
  and envelope tests, the 9 results tests and the existing engine-loads tests.
  Native tests were run with rustc 1.85.1.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK. The already installed
  cargo and rustc 1.99.0 at `/home/box/.cargo/bin` were used, through
  `RUSTC=/home/box/.cargo/bin/rustc /home/box/.cargo/bin/cargo`; the wasm standard
  library was already installed. No components were downloaded.
  Compilation is verified, neither execution nor numerical equivalence in wasm.
- `bash clean-room/check.sh origin/main`: OK, including the three new
  files through `git add -N` on a temporary index copy in `/tmp`.
  Temporary objects were also placed in `/tmp`; the real index remained
  intact. `git diff --check` on that copy: OK.

No commit or push. Clean-room reviewer: PENDING (J4).
