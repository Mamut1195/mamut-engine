# Clean-room log · ENG-ANL-003

- Ticket / PR: ENG-ANL-003 «P-Delta with instability detection» · PR #59.
  Original implementation: commit `5e54ea9` on 2026-10-11. Log added
  on 2026-10-13 for inclusion in a separate commit; no git commit or
  push in this session.
- Author(s): Codex (gpt-6.1-sol, high effort) under the direction of J2 (agent,
  Grok Bot). Log incorporation date: 2026-10-13
  (America/Guayaquil).
- Role: implementer and author of original analytical derivations.
- Implemented contract: `pdelta`, API `PDelta::run` over `LinearSystem`,
  with `PDeltaOptions`, `PDeltaOutcome`, `PDeltaIteration`, `PDeltaResult` and
  `geometric_producer` adapter. Uses the public `Element` boundary for Kg
  and `LinAlgBackend` / `FactorInfo` for solving and inertia. Does not implement
  adaptation to the generic trait `AnalysisMethod` (`analysis/1`).
- Permitted sources used: exclusively the ticket request and the own APIs
  and code of the repository: engine-analysis, engine-assembly,
  engine-linalg, engine-core, engine-contracts, engine-model,
  engine-elements-frame and engine-elements-truss. Template:
  `provenance/TEMPLATE.md`; format: `provenance/ENG-ANL-001.md`.
  `workspace-write` sandbox without network; no external sources, books, network
  searches, OpenSees, results from other engines or oracles.
- Original derivations: [note in this log](#derivaciones-propias) and
  [embedded analytical tests](../crates/engine-analysis/tests/pdelta.rs).
- Third-party code copied: none. No new external dependencies.
- Validation: analytical, closed-form solutions, mesh refinement, LDLᵀ
  inertia, factorization/solve counter and error
  and state preservation checks.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: J4 APTO with follow-up (2026-10-13, abe0690); self-declared provenance, written after the commit.

## Integration with main (#58), 2026-10-13

The modules `combination`, `pdelta` and `results` are integrated. LDLᵀ retains
`FactorInfo.inertia: Option<(usize, usize, usize)>` (positive, negative,
zero), main's `symmetric_inertia` query for Sturm/spectrum and the
ENG-E-0006 rejection of singular factors used by P-Delta. main's error
messages and the branch's validated counter are retained; 2×2
blocks use the eigenvalue of greatest magnitude and `det/λ_grande` to avoid cancellation.
A regression combines 1×1/2×2 blocks and zero modes with permutations and extreme
scales, checking both APIs. Local integration without network or git add,
commit or push.

Verification with `CARGO_NET_OFFLINE=true`: `cargo fmt --all` and its `--check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`
(373 tests, including the 13 P-Delta tests) and workspace documentation
with `RUSTDOCFLAGS='-D warnings'`: OK, Rust 1.85.1. `cargo check --workspace
--target wasm32-unknown-unknown`: OK with `stable` 1.99.0 and the already
installed target. The first focused pass failed on the global heap counter;
the isolated test with/without capture and the subsequent full suite passed.

## Late incorporation

According to the facts supplied for this record, the original implementation of
commit `5e54ea9` was carried out using the clean-room method described above. That
session ended with `No space left on device` before the log was written.
That is why it is incorporated now, on 2026-10-13, as documentation separate from the
implementation commit. This record describes the existing code and tests,
read locally, and distinguishes the checks run today
from the original session. It does not attribute today's results to that session.

## Design, convention and limits

The main scope is
[pdelta.rs](../crates/engine-analysis/src/pdelta.rs) and its tests. Support
in [system.rs](../crates/engine-assembly/src/system.rs) exposes
`geometric_at`, `reduced_stiffness` and `factorize_reduced`: it retains the global
Kg blocks, including the restrained ones, and translates backend mechanisms
to node/DOF. `SystemView::geometric` shares the global evaluation.
In [reference.rs](../crates/engine-linalg/src/reference.rs), the inertia
counter distinguishes positive, negative and zero signs; it validates finiteness and the
presence of the accumulator, and correctly counts the signs of 2×2 blocks.
Accesses to empty blocks/permutations return explicit errors.

First the lateral case is validated and gravity is solved with `StaticLinear`.
Each producer evaluates Kg at the expanded gravity displacement, including
prescribed values. The adapter owns original coordinates, parameters and accepted
history and delegates to `Element::geometric_stiffness`; it does not estimate a member's
axial force from the model's total load. Axial forces, axes and materials
remain frozen during the lateral analysis. Tension has positive sign;
compression reduces transverse stiffness. There are no code-prescribed factors,
geometry updates or large-deformation analysis.

With `F` already scaled by the lateral factor, `Kt = K + Kg` is formed and the
inertia of `Kt_ff` is checked before iterating. Gravity and lateral are independent
cases: lateral displacement does not include gravity displacement and lateral
reactions are not those of the gravity+lateral combination. Prescribed
values `u_p` are not scaled. Iteration starts with `u_f = 0` and solves:

```text
K_ff u_f^(i+1) = F_f − K_fp u_p − (Kg u^(i))_f
u^(i) = expandir(u_f^(i), u_p)
```

In the limit, `(K+Kg)_ff u_f = F_f − (K+Kg)_fp u_p`.
Reactions are calculated as `(Kt u − F)_p`. The factor of `Kt_ff` is used
only to diagnose stability; fixed-point iteration reuses the factor of K_ff.
`generalized_eigen` is not called, there is no regularization and there are no retries or
automatic strategy changes.

The inertia declared by the backend must sum to the reduced order. An inertia
with zeros returns ENG-E-0006; with negatives and no zeros it returns `Unstable`
with the negative count, without iterations, lateral displacement or reactions.
The reference backend already rejects numerically singular pivots during
factorization: its zero count does not mean it supplies a usable singular
factor. A zero diagonal in an invertible 2×2 block does not imply
a zero mode.

At each step the free displacement, the relative change
`‖Δu/s‖∞ / max(1, ‖u_nuevo/s‖∞, ‖u_previo/s‖∞)` and the absolute free residual
`‖(Kt u − F)_f‖∞` are reported. The historical diagnostic `relative_free_residual` divides by
`max(1, ‖(Kt u)_f‖∞, ‖F_f‖∞)`. The scales `s` distinguish translations in m
and rotations in rad and do not change the equations. The residual mixes
SI N/N·m components; it is a numerical indicator, not a unit-invariant physical norm.
Since the rounding correction of 2026-10-15, that diagnostic does not determine
stopping: `Completed` requires relative change `<= tolerance` and
`residual_convergence_ratio <= 1`, with separate references by DOF type
and a per-row floor according to the following derivation.
`NotConverged` exhausts exactly `max_iterations` and retains the last trial.
No result accepts or modifies the system's accepted/trial state.
A stable matrix may fail to converge by fixed-point iteration, especially with
large tension. The reduction and reference backend are dense for
small models; scalability or wasm execution is not promised in these tests.

## Derivaciones propias

### Relative residual by DOF type and rounding floor (2026-10-15)

For the stored global matrix `Kt = K + Kg`, the dot product of a
row is evaluated as a sequential sum of `m_i` products. If
`q_i = Σ_j |Kt_ij| |u_j|`, the absolute error of the product is bounded,
in the usual rounding model without overflow/underflow, by
`γ_m q_i`, where `γ_m = m_i·eps / (1 − m_i·eps)`; we use
`eps = Real::EPSILON = 2^-52`, conservative relative to unit roundoff
`eps/2`. Subtracting `F_i` adds a term on the order of
`eps·(|(Kt u)_i| + |F_i|)`. Near equilibrium, these quantities are on the
order of `q_i`; cancellation of signed terms reduces the final
product but not its evaluation error. Thus normalizing only by the
signed product or external load may require a residual below
the available rounding accuracy. An absolute constant 1 also mixes force and
moment and depends on the chosen units.

Two groups are defined among the free DOFs: translations (force equations)
and rotations (moment equations). At each iteration:

```text
q_i       = (|Kt| |u|)_i
ref_g     = max_{j libre del grupo g} (|F_j|, q_j)
piso_i    = 64 · max(1, m_i) · eps · q_i
límite_i  = tolerance · ref_grupo(i) + piso_i
indicador = max_{i libre} |(Kt u − F)_i| / límite_i
```

`m_i` counts the CSR entries of the **global** row, including stored
zeros. `q_i` includes all displacements, including prescribed ones;
only the checked rows and group references are limited to free
DOFs. The factor 64 is an explicit numerical margin, not a physical or code-prescribed
factor: for `m_i·eps << 1` it leaves several tens of times the error
scale of the sum, allowing prior rounding from assembly, Kg recovery
and elastic solving. It is not a universal bound on the error of
any backend or a guarantee of forward error in ill-conditioned
systems. The floor is not estimated using the observed residual or
increased with the number of iterations.

The accepted residual does not directly equal displacement error either:
if `r = Kt u − F` and `u*` is the exact solution of the discrete system,
`u_f − u*_f = Kt_ff^-1 r_f`. Near the first buckling mode, the stiffness of that
mode decreases approximately as `1−P/Pcr`; therefore, the accepted residual
corresponds to a displacement error amplified by `~1/(1−P/Pcr)`.
The effective tolerance on `u` is relaxed by that factor. This is a modal
conditioning estimate, not a universal bound or a forward-error
guarantee; the independent displacement-change criterion remains
mandatory.

In practice, the rounding floor is only active with prescribed
displacements: a nonzero residual may remain even when the increment
stagnates at zero. W14x48 stagnation is resolved by the separate
force and moment references based on `q_i`, not by the floor. The latter
is proportional to `q_i`, like the reference; with the W14x48 tolerances and
meshes, its contribution remains below `tolerance·ref_g`.

Each limit retains the units of its equation. Consistently scaling
forces or moments equally scales the residual, reference and floor of that group;
there is no absolute force constant or comparison of forces with moments.
A group without load or absolute product has zero reference and residual:
`0/0` is defined as 0. If the limit underflows to zero with nonzero residual,
the indicator is `Real::MAX` and does not converge. Nonfinite products, sums and indicators
remain ENG-E-0008.

The indicator is published in `PDeltaIteration.residual_convergence_ratio`
and in the P-Delta JSON report. `free_residual_norm` and
`relative_free_residual` retain their previous meaning, solely as
diagnostics. Relative displacement change is unchanged: **both**
criteria must pass, without updating the accepted state or retrying.
The old checks that identified the mixed diagnostic with the stopping
condition are updated to the new indicator (analysis, validation and
API); physical comparisons and the scalar count of ten steps are retained.

The regression uses the ticket's W14x48 cantilever: E=29000 kip/in²,
A=14.1 in², I=484 in⁴, L=336 in, H=1 kip and compressive P=200 kip,
EulerFrame2D, scales 1 in and 0.01 rad. The user-supplied
`/tmp/bench_pdelta_ref.rs` was consulted as a model construction reference;
the tests are self-contained and do not include or execute it. Meshes
32 with tolerance `1e-10` and 8 with `1e-12` must complete; both must
return `NotConverged` with only two steps. The continuum reference solution
is `H(tan(kL) − kL)/(Pk)`, `k²=P/(EI)`.

The continuum deflection is 2.5648953977433653 in. Mesh 8 gives
2.5648889686672063 in, relative error `2.5066e-6` due to discretization:
it does not satisfy the originally requested `1e-6` even if the iteration converges.
The scope of the convergence threshold is retained: the regression requires
`3e-6` with 8 elements and `1e-6` with 32, and also checks both iterative
responses against a direct solve of the same reduced matrix `Kt_ff`
(relative error `< 1e-8`). This comparison distinguishes iterative error from
spatial error; the EulerFrame2D formulation is not changed and deflections
are not artificially corrected. The tolerance difference was communicated to the user.

A scalar test with prescribed value 0.03 and tolerance `1e-20` verifies that the
increment reaches zero but a nonzero residual remains: the indicator is checked
against the floor calculated explicitly using both displacements of the row.
Zero reference with zero load is also verified. The invariance test
solves the 8-element W14x48 in kip·in and N·m, with the exact conversions
`1 kip = 4448.2216152605 N` and `1 in = 0.0254 m`. It also converts E, A, I
and the translation scale; it retains the angular scale. With P=200 kip
(`~0.65 Pcr`), it checks the same outcome and iteration count, all equivalent
global displacements and the initial dimensionless indicator.
It repeats with the lateral load multiplied by `1e-6` in both systems,
retaining P and, therefore, proximity to buckling. This case also checks
invariance with small references, sensitive to introducing an absolute
`max(1.0, ref_g)`.

Two independent Ux/Rz springs,
alternating a load of 1 and another of `1e12`, check that
an equilibrated group does not hide the other's remaining residual: the change and the
mixed diagnostic pass, but the new indicator prevents completion.

The fixtures are embedded in Rust, without external files, network or oracles.
The cantilever uses E=200 Pa, A=2 m², I=3 m⁴, L=4 m and H=0.1 N.
Common settings are tolerance `1e-10`, maximum 2000 steps and scales
`0.01` m and `0.01` rad. The following comparison tolerances are those
declared by the tests; absolute ones use the units of each quantity.

### Compressed cantilever, critical load and amplification

For compression P>0, the equation is `EI v'''' + P v'' = 0`, with
`v(0)=v'(0)=0`, `v''(L)=0` and `EI v'''(L)+P v'(L)=−H`.
Let `k²=P/EI`: integrating and applying the conditions gives

```text
v(L) = H/P · [tan(kL)/k − L]
v_lineal(L) = H L³ / (3EI)
Pcr = π²EI / (4L²)
r = P/Pcr, z = kL = π√r/2
v(L)/v_lineal(L) = 3(tan z − z)/z³
```

The first singularity appears at `cos(kL)=0`, that is `kL=π/2`.
The amplification `1/(1−P/Pcr)` is approximate: near `r=1`, the exact
solution has pole coefficient `96/π⁴`, rather than one.

The test uses `r=0.9` and `0.95` and meshes of 1, 2, 4, 8, 16 and 32 elements.
It requires a strict decrease in relative error with each refinement. With 32
elements it requires relative error `< 5e-7` against the exact solution and `< 0.02`
against `v_lineal/(1−r)`. Final relative change must be `<= 1e-10`
and the residual indicator with floor `<= 1`. The axial force recovered from gravity in each member,
`EA/Δx · (u_x,j−u_x,i)=−P`, is verified with absolute tolerance `1e-10` N.

### Unstable from inertia and 2×2 blocks

LDLᵀ congruence preserves the inertia of the symmetric matrix. For a
block `[[d,e],[e,g]]`, signs are obtained from
`h=(d+g)/2`, `ρ=hypot((d−g)/2,e)` and `det=dg−e²`.
The code first calculates `λ_grande=h+copysign(ρ,h)` and then
`λ_pequeño=det/λ_grande`, avoiding cancellation when obtaining the smaller one.
The fixture `K_ff=I`, `Kg_ff=[[-1,2],[2,-1]]` produces
`Kt_ff=[[0,2],[2,0]]`: its signs are positive and negative, with exact inertia
`(1,1,0)` and `Unstable { negative_eigenvalues: 1 }` even though both diagonals
are zero. This check uses count equality, without numerical tolerance.

With 16 elements, `P/Pcr=1.01` and `1.2` give one negative; `10` gives two.
It is checked that the result contains no iterations or lateral response.
The mesh affects the crossing: at `1.001 Pcr`, a single element returns
`NotConverged`, while eight give `Unstable` with one negative. The
continuum Euler threshold is not identified with that of every discretization.

### P=0 equals linear analysis; tension stiffens

With zero axial force, Kg=0 and fixed-point iteration solves the same equations as
`StaticLinear`. The test uses four elements, prescribed base displacement
`Uy=0.003` m and lateral factor 2. It compares all displacements
with absolute tolerance `1e-12` m/rad and reactions with `1e-10` N/N·m.
The settlement is not scaled.

For tension T>0, `EI v'''' − T v''=0` gives, with `k²=T/EI`,
`v(L)=H/T · [L−tanh(kL)/k]`. The test takes `T=0.5 Pcr` and 16 elements,
requires `v(L)<v_lineal(L)` and relative error `< 2e-7` against that closed form.
It also checks `uᵀ Kg u>0`: the geometric contribution stiffens.

### Truss and fixed-point iteration controls

Symmetric truss: bases (−4,0), (4,0), apex (0,3), bars of 5 m,
E=200 Pa, A=2 m², gravity P=40 N and lateral H=10 N.
With `c=4/5`, `s=3/5`, vertical equilibrium gives `N=−P/(2s)` per bar.
Thus `Kxx=2(EA/5)c²=102.4 N/m` and
`Kgxx=2(N/5)s²=−P·0.6/5=−4.8 N/m`.
`ux=H/(Kxx+Kgxx)` is verified with absolute tolerance `1e-9` m and
`uy=0` with `1e-12` m using existing truss producers.

The scalar fixture `K=100`, `Kg=−25`, `F=10` has recurrence
`u_(i+1)=0.1+0.25u_i`, starting from zero. By geometric summation,
`u_i=(10/75)(1−0.25^i)`, relative residual `0.25^i` and relative change
`0.1·0.25^(i−1)` with unit scales. The three expressions are compared
with absolute tolerance `1e-15`. With stopping tolerance `1e-6` and maximum
20, exactly ten steps, three factorizations (gravity,
total inertia, elastic K) and eleven solves (one gravity and ten lateral) are required.
With a three-step limit, `NotConverged` is required, without retry or acceptance.
The states previously set to 0.7 and 0.8 remain unchanged.

With prescribed value `s=0.03`, `(100−25)(u2−s)=10` implies
`u2=s+10/75`, `R1=−10`. Absolute tolerances are `1e-15` m for
the prescribed value, `1e-10` m for u2 and `1e-8` N for the reaction.
With `Kg=200`, the total matrix is 300>0 but the recurrence
`u_nuevo=0.1−2u` has spectral radius 2: `NotConverged` is required
after four steps and the accepted state remains intact.

## Error codes and input validation

- ENG-E-0002: repeated element DOF in the adapter.
- ENG-E-0003: missing gravity or lateral case.
- ENG-E-0006: elastic mechanism or singular total stiffness. The reference
  backend supplies a reduced index, which the system translates to node/DOF;
  the singular scalar test verifies node 2 and Ux. The branch receiving zeros
  in `FactorInfo` returns the code directly, without that translation.
- ENG-E-0008: nonfinite inputs, Kg evaluation, solutions, products, sums or
  indicators, including NaN, ±∞ and overflows.
- ENG-E-0010: tolerance outside `(0,1)`, zero step limit, nonpositive
  scales, incompatible element/history sizes or invalid dimensions.
- ENG-E-0012: inconsistent backend diagnostic, inertia counts whose
  sum does not match the order (including count overflow), invalid
  solution size or missing LDLᵀ accumulator.
- ENG-E-0013: backend without inertia, element without Kg or reuse of a
  static Kg without a producer outside the declared reference state.

The tests cover invalid parameters, nonfinite values, missing cases,
overflow of the lateral factor and displacement normalization,
nonfinite Kg and state preservation. They also check absent and
inconsistent inertia through an instrumented backend, adapter sizes/DOFs,
consistency of global/reduced Kg and reference protection
of a static Kg. `Unstable` and `NotConverged` are method outcomes,
not input error codes.

## Local verification

Run on 2026-10-13, without network. All Cargo invocations used
`CARGO_NET_OFFLINE=true`. No toolchains or dependencies were downloaded.

- `cargo fmt --all --check`: OK, exit 0. Format checking was used
  to respect the restriction against modifying other files.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK, exit 0.
- `cargo test --workspace`: OK, exit 0; 286 tests passed in total,
  including documentation tests and the 13 P-Delta integration tests;
  zero failed and zero ignored.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, exit 0.
- `cargo check --workspace --target wasm32-unknown-unknown` with system Cargo/rustc
  1.85.1: FAILED, exit 101, E0463 (`can't find crate for std`);
  the standard library for that target is missing for that compiler.
- `RUSTUP_TOOLCHAIN=stable PATH=/home/box/.cargo/bin:$PATH cargo check --workspace --target wasm32-unknown-unknown`:
  OK, exit 0, with Cargo/rustc 1.99.0 and the already installed wasm target.
- `bash clean-room/check.sh origin/main`: OK, exit 0 (`Clean-room: OK`).
  The script compares files known to Git against the local merge-base;
  it does not include new untracked files, such as this log at closing.

### Rounding floor correction, 2026-10-15

Local verification without network, commit or push, with Cargo/rustc 1.99.0 and
`CARGO_NET_OFFLINE=true`. `/home/box/.cargo/bin` was prepended to PATH to
use the toolchain pinned by the repository.
`CARGO_TARGET_DIR=/tmp/me-conv-target` was used because the target configured outside
the workspace was read-only.

- Regression before the correction: `NotConverged` at 2000 steps with
  32 elements; change `4.33e-15`, mixed residual `5.68e-10` (the initial
  reproduction was run with Cargo/rustc 1.85.1 from the system PATH).
- `cargo fmt --all`: OK, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK, exit 0.
- `cargo test --workspace`: OK, exit 0; 621 tests passed,
  zero failed or ignored, including the 16 P-Delta tests and the scalar
  determinism guard. Local log: `/tmp/me-conv-workspace-tests.log`.
- The scalar fixture retains ten iterations, three factorizations and eleven
  solves. Checks of the final indicator in analysis,
  validation and API are updated because the new stopping condition may be reached before
  the mixed diagnostic crosses its old tolerance.

Formatting, Clippy, native tests and documentation were run with
Cargo/rustc 1.85.1. The wasm check with `stable` verifies compilation, not
execution of tests in a wasm runtime. J4's clean-room review remains
pending. The only working file added in this session is this log;
no code was modified, nor were git commit, push or network access performed.

### J4 review follow-up (2026-10-15)

Codex (GPT-6), implementer role, using exclusively the review request
and the own APIs, tests and derivations of this checkout. No external
sources, network, copied third-party code or new dependencies. Added scalar
functions go through `engine_core::math`, with
`use engine_core::math;`. I did not consult source code from OpenSees, Stabileo,
CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer,
nalgebra or derivatives.

- The invariance test passes with the current criterion. Temporary mutation:
  `tolerance * references[group].max(1.0) + floor`. It fails as expected
  with lateral load `1e-6`: N·m requires 32 iterations and kip·in only 23.
  The criterion without an absolute reference was restored before validation.
  Log: `/tmp/me-conv-j4-mutation.log`.
- `cargo fmt --all`: OK.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK.
  Log: `/tmp/me-conv-j4-clippy.log`.
- `cargo test --workspace`: OK; 625 tests passed, zero failed or
  ignored, including the 17 P-Delta integration tests and the scalar guard.
  Log: `/tmp/me-conv-j4-workspace-tests.log`.

Invocations with `CARGO_NET_OFFLINE=true`, Cargo/rustc 1.85.1 and
`CARGO_TARGET_DIR=/workspace/me-conv/target`. `advanced.rs` and
`docs/DETERMINISMO.md` remain intact; no commit or push was run.
The following table transcribes existing evidence; this review does not
rerun the native/wasm cross-check.

### Determinism hashes after merging main (#103)

When incorporating main (52f0613, 5d0139c), the field
`residual_convergence_ratio` enters P-Delta results/1 and changes its stopping
threshold. The table lists the 16 cases before and after; the other 14
remain equal to [docs/DETERMINISMO.md](../docs/DETERMINISMO.md).
The already recorded cross-check evidence is native = wasm (Node), in debug and
release, ten runs per case.

| Input / bit case | Before | After |
| --- | --- | --- |
| cantilever | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` | `dc3e62defc8012b55ceba0108b76930bc73f9d805329f272631233d80a1ea153` |
| inclined-cantilever | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` | `d28509b76b283de84be528b593295c2fc735c62aa222c53d8374bcfb256a2492` |
| modal-1dof | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` | `bf52691f1eafd4c963e6f3947b141d8210faace668533b5673343d1926073da3` |
| pdelta | `980bf21b60a5b931a0f0c137ebc782a4a878abb95f7e8bce880c91c4374f56ca` | `f2cc535ed32fb2ed071a3ed8048ebf53c2cd81083efe8ba528e5c95d8cf46740` |
| modal-coupled | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` | `5e7f69005b39e8f4e93189aef46895770b5d1c1d4fe99d14d95b9a6ea9d8a86b` |
| spectrum-log-cqc | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` | `f68278ad240accbbc99bd58d0db43b161d74884af9d4cce05fab0b408c6f7dd2` |
| pdelta-inclined | `9b5a4584e2151bb4d72c34933f564e8642e85600d44662e78ea376a61541a16b` | `70825cf724a03f9b82f1fd21c76175645a78bc4fcaea648504a7bfc6fd3b978b` |
| modal-inclined | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` | `d0ba8d1aefa2287fa6f7c2361864958aba0b1db89d68ec6e39583bf0d20913d7` |
| spectrum-inclined | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` | `aac5433acbe9f3a22fa5b610b1aac1d7842508503b30c8bc369c81e03b7d8baf` |
| frame3d-inclined-static | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` | `44e60765807c2dd654fb3f018cc11993c510d61f86da737cc29387e96c2c8da1` |
| frame3d-inclined-modal | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` | `c74d3acbdd2299dc4a7b2ed99223e31f78d7bcf304057c1ae2400e1188816d53` |
| scalar-trig | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` | `99dd07ae2d88c8d218ab3b7c029b87c23cb3c7b952164f1327ffe46c7b7a7667` |
| scalar-exp-log | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` | `aabcfc5827b7eb9c7ad318f728959f09f54ff44d20aa596d09ef41d2656e3ceb` |
| scalar-powers-norm | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` | `63ce475e83d8fdbf276556f7481f68f03e62026131ae83d349d4409a0cd52b2c` |
| load-rotation-bits | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` | `1fbb21074fd7d82ef12b3a7de12bba2191cb9cbd42067f1454f2222a3533d341` |
| scalar-sin-cos | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` | `b01bfda9f708e6cb7aa4ff9922480b6d26db245e0da1ae4c4b9694bfa555cf7a` |
