# Clean-room log · ENG-SOL-001

- Ticket / PR: ENG-SOL-001 «Static Newton-Raphson with load control».
  Branch `j2/eng-sol-001`, stacked on ENG-ELM-005 (#48: `spring.nonlinear`,
  includes #46/#45) and ENG-ANL-001 (#52: `crates/engine-analysis`), unmerged.
  Local delivery without commit or push.
- Author(s): J2 (agent, Grok Bot), attribution requested by the user;
  executing agent Codex (GPT-6), 2026-10-11 (America/Guayaquil).
- Role: implementer and author of own derivations and analytical fixtures.
- Contract implemented: `static.newton` method, typed API
  `newton::StaticNewton::run` over `NewtonSystem<B>`; `element/1` responses
  and `uniaxial/1` law history. Like StaticLinear, this delivery does not adapt
  the method to the AnalysisMethod trait (JSON, sink and cooperative control).
- Permitted sources used: only the user request, internal code and
  documentation of engine-core, engine-contracts, engine-assembly,
  engine-elements-spring, engine-laws, engine-analysis, engine-model and the
  engine-linalg boundary, including the own backend's error policy.
  `provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md` and ENG-ELM-005 and
  ENG-ANL-001 logs. No network, external sources, manuals or third-party listings.
- Own derivations: [equations and solutions in this note](#derivaciones-propias)
  and [analytical tests](../crates/engine-analysis/tests/newton.rs).
- Third-party code copied: none. The existing own laws and backend
  are reused. Only two internal path dev-dependencies to test
  the actual spring and law; no new external dependencies.
- Validation: exclusively analytical, piecewise closed-form equations and
  backend counting factorizations/solves; executable Rustdoc example.
- Comparison with oracle (sibling repo, informative): no. No external
  engine was run or oracle consulted.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

## Design and acceptance

NewtonElement owns a pure Element, SI context and global indices. It validates
the element/1 contract with compatible minor version, sizes, coordinates and
node/DOF order against the DofMap. NewtonSystem retains the existing LinearSystem and
the independent accepted histories, in variable-element order.
The initial displacement is the one already accepted by LinearSystem; the initial
histories come from Element::initial_state. The initial evaluation validates the
response but accepts neither its trial history nor any load factor.

Variable elements are **additional** to K_lineal: they must not be duplicated
as ElementContribution stiffness. This rule allows combining constant
linear elements with any nonlinear Element with a symmetric tangent.
Constant equivalent loads are declared in ElementContribution::loads
(it may have zero stiffness if it only contributes loads); nodal loads, in the model.
Kg is not interpreted and the reference geometry is not altered by the analysis.

At each evaluation, expand(u_f) incorporates constant u_p and gather obtains TOTAL
u_e. respond(ctx, u_e, historia_aceptada) returns force, consistent tangent
and trial history. assemble_vector sums the forces and assemble_matrix sums
the tangents; sizes, finiteness and exact symmetry are revalidated at each evaluation.

```text
u = expand(u_f, u_p)
F_int(u) = K_lineal u + Σ scatter(F_e(u_e, historia_aceptada))
K_t(u)   = K_lineal   + Σ scatter(K_e(u_e, historia_aceptada))
R_f      = (λ_k F − F_int(u))_f
K_t,ff Δu_f = R_f
u_f ← u_f + Δu_f
```

Condensation selects the ff partition like LinearSystem. Δu_p=0: K_fp u_p is
not subtracted in the correction RHS, because u_p already appears in F_int(u).
Each correction requires exactly one factorization and one solve through the
engine-linalg backend. Assembling or evaluating the response does not factorize.

After updating u, the response is reevaluated using **the same accepted history
from the previous step**, never a history from an intermediate iteration.
If all active tolerances are satisfied, exactly u is accepted and the
validated trial_state values are moved to the accepted histories. If the maximum is exceeded,
Status::NotConverged is returned with the failed step reported, and the list
stops; results and system retain u, factor and histories of the last accepted
step. If there was no prior acceptance, they retain initial u/histories and factor
None. Errors also discard trials; LinearSystem rollback leaves its
trial_displacement equal to accepted_displacement. There are no hidden retries,
bisection, step reduction, line search or regularization.

Factors are absolute over ONE case: [0.25, 0.5, 1] ends at F_total.
Repeated, zero and negative factors and explicit unloading are allowed. The entire
list, its products λF and the options are validated before the first commit.
λ_final=1 is not imposed: the user may request another load target.

## Criteria and diagnostics

R is measured after updating u. Δu is the last correction actually applied:

```text
‖R_f‖∞ ≤ tol_f · force_scale
‖Δu_f‖∞ ≤ tol_u · displacement_scale
|Δu_fᵀR_f| ≤ tol_e · energy_scale
```

Each tolerance is Option<Real>; None disables that criterion. At least one
must be active, all active ones must be met, with inclusive comparison.
Tolerances and scales are finite and strictly positive; max_iterations>0.
The report retains absolute and relative norms, options/scales used,
iterations performed, factor and convergence. SI scales are explicit,
constant, defaulting to 1; there is no hidden normalization or change between iterations.
Mixed SI norms are numerical indicators; no physical invariance is claimed
when mixing forces/moments or translations/rotations.

By default only force is active (1e−10), maximum 25. A linear problem
converges in one correction with force alone (it may also satisfy energy).
If small displacement is also required, it may need another correction:
no hidden solve is performed to estimate a future correction.
Force alone/displacement alone/energy alone may authorize acceptance;
no force checks are added when the user has disabled them.

ENG-E-0006: singular tangent, mapped to node, DOF and global index as in
LinearSystem; an open gap is not stabilized without other physical stiffness.
ENG-E-0008: non-finite inputs, forces, tangents, histories, corrections,
work products/sums and relative norms. ENG-E-0010: invalid parameters,
scales, sizes/context/contract. Missing cases ENG-E-0003, duplicates
ENG-E-0002 and inconsistent backend diagnostic
ENG-E-0012 are propagated as in the existing APIs. No panic!, unwrap or expect is added
to library code. No existing contract changes.

## Derivaciones propias

All fixtures use artificial SI, embedded data, without network or I/O during
tests. Absolute analytical comparisons <1e−9; default residual
≤1e−10. Tolerance equality tests use exact binary numbers.

1. **Bilinear to ground:** points (δ,F)=(-.5,-18),(-.1,-10),(0,0),(.1,10),(.5,18).
   F=100δ for |δ|≤.1 and F=sign(δ)[10+20(|δ|−.1)] outside.
   For P=16, from zero: u1=.16, F1=11.2, R1=4.8;
   Δu2=4.8/20=.24, u2=.4 and F2=16. Exactly two factorizations and solves.
   For factors .25,.5,1, u=.04,.08,.4 are accepted in 1,1,2 corrections.
2. **Series:** first spring k=100/20 with vertex (.1,10), second k=200/40
   with vertex (.1,20). Equilibrium F1=F2=P. δ1=P/100 if P≤10, or
   .1+(P−10)/20; δ2=P/200 if P≤20, or .1+(P−20)/40. u1=δ1 and u2=δ1+δ2.
   P=4,12,24 gives (δ1,δ2)=(.04,.02),(.2,.06),(.8,.2).
3. **Compression-only gap that closes:** points (-1.25,-100),(-.25,0),(0,0),
   CompressionOnly mode. F_gap=100(u+.25) for u<−.25, zero outside.
   Physical linear spring k=10 in parallel; load −20. From open u=0,
   u1=−2; closed, 10u+100(u+.25)=−20 → u=−45/110 in two corrections.
   The same gap without a parallel spring is singular and produces ENG-E-0006.
4. **Linear and prescribed values:** inter-node spring k=100, u1=.03, node 2 free,
   nodal load 12 and equivalent 3, factor 2. u2=.03+2(12+3)/100=.33.
   Converges in one correction and matches StaticLinear, without scaling u1.
   The nonlinear inter-node spring with u1=.03 and P=16 gives u2=.43.
5. **History and rollback:** own test law with k=100, limit |F−Hp|=10,
   H=25. Predictor ξ=100(u−p)−25p; if |ξ|>10, p_trial=p+sign(ξ)(|ξ|−10)/125.
   F=100(u−p_trial), tangent kH/(k+H)=20; elastic branch k=100.
   With max_iter=1 and factors .25,1,2: accepts u=.04,p=0; tries u=.16,p=.048
   and R=4.8, Δu=.12, trabajo=.576; returns the first accepted state, two
   total factorizations/solves and does not attempt factor 2. Another execution
   accepts P=16,u=.4,p=.24; a subsequent failure retains that nonempty history.
6. **Independent thresholds:** the first bilinear correction from zero
   gives (‖R‖∞,‖Δu‖∞,|ΔuᵀR|)=(4.8,.16,.768). They are activated separately with
   scale 2 and tolerances that pass or fail; each active threshold may
   prevent acceptance even if the other two pass. All three strict criteria require
   three corrections (the last is numerically zero). A half-solution backend
   with P=8,k=2 produces Δu=2,R=4,trabajo=8: exact inclusive equality.
7. **No hidden strategy:** from u=.4, unloading directly to P=0 oscillates
   between ±.4 with tangent 20. The test requires five corrections and five
   factorizations, a single failed step λ=0 and the accepted u=.4 intact.
   Unloading with EXPLICIT intermediate factors does converge.
8. **Errors:** invalid options/factors/cases, overflow λF,
   non-finite backend and law returning non-finite history after accepting
   u=.04 are tested. The error retains that displacement, its history and factor .25.
   Out-of-range locations, inconsistent sizes and coordinates are
   rejected before solving.

## Local verification

Verification on 2026-10-11, without network, with CARGO_NET_OFFLINE=true:

- `cargo fmt` and `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, including the 16 new tests and the doctest
  for StaticNewton use with the actual law and spring.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK using the
  stable toolchain already installed through rustup, with `/home/box/.cargo/bin` at the start
  of PATH; no downloads. Wasm compilation, no execution in a wasm runtime.
- `bash clean-room/check.sh origin/main`: OK; the diff is also checked
  including new files with temporary Git index and objects in `/tmp`,
  without modifying the user's real index or objects.
- `git diff --check`: OK, also including new files in the temporary
  index. No panic!, unwrap or expect in engine-analysis/src or in the
  three new engine-assembly operations.

engine-analysis is modified (module, export, example and tests), the three
compatible LinearSystem operations and two internal test dependencies are
added; Cargo.lock only records those two relationships between existing crates.
The ENG-ELM-005 declaration is extended; ENG-ANL-001 already contains the requested
declaration, author and reviewer. No laws, elements, contracts or clean-room
rules are modified. No new external dependencies, commit or push.
J4 clean-room review remains PENDING.
