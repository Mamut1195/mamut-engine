# Implementing a method in Rust · ENG-CORE-011

This guide takes a mathematical method sheet to a reviewable implementation.
The current contract governs: it is not changed to accommodate a method. Work
from the worktree root, offline, with provisioned tools and dependencies.
The [clean-room protocol](CLEAN_ROOM.md) is mandatory; this guide does not replace it.

## 1. Prepare the session and choose the contract

Read, in this order:

1. [CLEAN_ROOM.md](CLEAN_ROOM.md), especially §§3–5, 7, 8 and 10, and
   [provenance/TEMPLATE.md](../provenance/TEMPLATE.md).
2. [engine-core](../crates/engine-core/src/lib.rs): `Real = f64`, `EResult`,
   `EngineError`, `ContractId`; [catalog](../crates/engine-core/src/catalog.rs)
   and [scalar functions](../crates/engine-core/src/math.rs).
3. [traits](../crates/engine-contracts/src/traits.rs),
   [interchange types](../crates/engine-contracts/src/types.rs) and
   [ENG-CORE-007 conformance](../crates/engine-contracts/src/conformance.rs).
4. [engine-laws](../crates/engine-laws/src/lib.rs), its modules and tests.
   Review what exists before choosing a law: parameterizing an existing
   law does not constitute a new method.
5. [ENG-CORE-009 benchmarks](../crates/engine-validation/README.md),
   [fixture/1 schema](../crates/engine-validation/schemas/fixture-1.json) and
   [determinism](DETERMINISMO.md).

Record the specifier/implementer role and local sources consulted.
If exposure to prohibited sources or reports occurred, do not implement: apply
the protocol's role restrictions. Sibling repositories are not inspected.
Sources are original derivations or published theory supplied locally
with a verifiable citation; do not invent pages, equations or results.

Configure the session (the environment provides a dedicated `$CARGO_TARGET_DIR`):

```sh
export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
: "${CARGO_TARGET_DIR:?Configura el directorio de compilación autorizado}"
export CARGO_TARGET_DIR
export CARGO_NET_OFFLINE=true
export PIP_NO_INDEX=1
rustc --version  # toolchain del ticket: 1.99.0
git status --short
```

## 2. Write the method sheet before the code

Copy [FICHA_METODO.md](plantillas/FICHA_METODO.md) and complete assumptions,
numbered equations, tangent derivation, citation of each source,
units, signs, physical and numerical domain, history and analytical cases.
Do not copy published tables, figures or listings. Evaluate the formulas
yourself to establish expected values **before** executing the method.

For `uniaxial/1`, the specific contract is:

| Aspect | Obligation |
| --- | --- |
| Input | Dimensionless **total** strain; positive stress in tension. If representing a spring: relative displacement in m and force in N, declared by the kind. |
| Output | `UniaxialResponse { stress, tangent, trial_state }`; tangent dσ/dε in Pa (or N/m), all finite. |
| State | Exact `state_len()`; flat, finite initial and trial histories. Document component order/units. |
| Ownership | `respond(&self, strain, committed)` does not mutate the object or history. The caller accepts with `committed = response.trial_state`; discarding a trial is equivalent to rollback. |
| Repetition | Identical inputs produce the same response; repeating an accepted strain reproduces its stress. |
| Origin | `initial_tangent()` matches `respond(0, initial_state).tangent`. |
| Identity | `ContractId::new("uniaxial", 1, 0)` and a stable kind of its own; method semantic version independent of the contract. |
| Errors | `ENG-E-0008`: NaN, infinities or overflow; `ENG-E-0010`: invalid parameters, sizes or ranges. Messages with quantity and units. |

A section method uses `SectionLaw` and `check_section_law`; an element uses
`Element`/`ElementFactory` and `check_element`/`check_element_factory`; an analysis uses
`AnalysisMethod`, `SystemView` and their checkers. Their sizes, units and
conventions come from the specific trait, rather than the uniaxial table.

## 3. Create the module and expose it like existing ones

The usual choice for another law is an `engine-laws` module; it does not need
another crate or manifest changes. Copy
[modulo_uniaxial.rs.tpl](plantillas/modulo_uniaxial.rs.tpl) to
`crates/engine-laws/src/uniaxial/mi_metodo.rs`, replace names, parameters
and the pending response with the method sheet equations. The template
stops with `ENG-E-0013`: it is not a validated law and must not be published incomplete.

Keep validated private parameters, a fallible constructor, `Params`, a typed
`UniaxialFactory` factory and a `LawDescriptor` constant. Reuse the private
`finite`, `positive` and `vector` validators from `engine-laws`. Validate length before
indexing and all outputs/intermediates susceptible to overflow.
Document the public API in Spanish; the library requires `missing_docs`.
Do not add `unsafe`, `unwrap`, `expect`, `panic!`, global state, I/O or threads.
Invoke scalar functions through `engine_core::math`; preserve the
operation/reduction order of existing methods.

Expose `mod mi_metodo; pub use mi_metodo::{...};` from `uniaxial.rs` and update
the `lib.rs` documentation. This is the current law registration mechanism: descriptor,
factory and public exports, without a global registry. It does not imply JSON support
in engine-api or turn linear analysis into nonlinear analysis. An API
extension requires its own explicit contract and validation.

If a contract requires another crate, first justify that separation; use the workspace
edition, license and minimum version, existing internal dependencies and
`#![forbid(unsafe_code)]` at the root. Only then adapt manifests and lock.

## 4. Conformance and analytical benchmark

Copy [conformidad.rs.tpl](plantillas/conformidad.rs.tpl) and
[fixture_uniaxial.rs.tpl](plantillas/fixture_uniaxial.rs.tpl) to `engine-laws/tests/`.
Complete import/constructor names and cases from the method sheet.
The template's empty table intentionally fails: it prevents vacuous validation.

Run `check_uniaxial_law(&law, &samples, h)` and require zero diagnostics.
The kit explicitly commits history between samples and compares the tangent with
a central difference at fixed committed history, with relative error < 1e-6.
Choose `h` within the limits published in `conformance.rs`, with an appropriate
scale and samples whose ε ± h interval is smooth. At corners or yielding,
test the one-sided convention separately, without central FD crossing branches.
Do not relax the kit to make an implementation pass.

Add negatives for each parameter/range, NaN/±∞ in inputs and history,
incorrect lengths and overflow, checking catalog codes.
Check factory/descriptor/object, `Send + Sync`, purity and state ownership.
The kit checks contract and consistency; an error shared by stress and
tangent can pass. Therefore, the benchmark uses independent analytical expected values.

The typed uniaxial fixture contains `source`, citation, SI parameters, inputs,
expected values and justified tolerance; it runs as a native test of the law.
It is not `fixture/1`: ENG-CORE-009 requires `engine-input/1 → results/1` through
`Target`, and the current API does not expose these nonlinear laws. Do not fabricate
JSON that the runner cannot execute or add another case to the existing closed
inventory of five JSON files. When the method is accessible through the API, use
[fixture_api.json.tpl](plantillas/fixture_api.json.tpl), complete the actual
input and JSON Pointers and run it with the native ENG-CORE-009 runner.
That format requires `source.kind` and a citation, and compares with
`|actual−esperado| ≤ max(absolute, relative·|esperado|)`; an override requires
justification. Do not change tolerances based on outputs from other engines.

<a id="5-piloto-reproducible-elasticidad-cúbica"></a>

## 5. Reproducible pilot: cubic elasticity

Original method sheet: Codex gpt-6.1-sol high, *Conservative cubic elasticity*,
2026-10-20, this section, equations (1)–(3) and the following table.
No external sources were consulted. Before the pilot, `Elastic`,
`MultilinearElastic` and `SteelBilinearKinematic` existed; the multilinear law already represents
bilinear curves, and the kinematic law allows perfect plasticity with b = 0.
None represents exactly a smooth cubic curve with C > 0.

An original energy density (Pa = J/m³), without history, is proposed:

```text
(1) W(ε) = E ε²/2 + C ε⁴/4,       E > 0, C ≥ 0, ambos en Pa.
(2) σ(ε) = dW/dε = Eε + Cε³,       σ positiva en tracción.
(3) K(ε) = dσ/dε = E + 3Cε² > 0,  K(0) = E.
```

W is even, nonnegative and strictly convex; σ is odd. Unloading follows
the same curve and work over a closed cycle is zero. This is a
local 1D elastic idealization, without plasticity, damage or universal physical validity
at large strains. Initial/committed/trial history: zero length.
C = 0 recovers the linear law as a limit, without replacing its implementation.

Implementation determined by the method sheet: `q = (C·ε)·ε`, `σ = (E+q)·ε`,
`K = E+3q`. Every product/sum must be finite; overflow returns
`ENG-E-0008`. Forming ε² before multiplying by C is avoided so that C = 0
allows large strains when the response remains representable.
Ordinary f64 rounding/underflow is accepted; relative accuracy of terms
below the number's resolution is not certified.

Original analytical benchmark with E = 8 Pa, C = 16 Pa. Substituting binary
fractions into (2)–(3) gives, before running the implementation:

| Total ε | σ (Pa) | K (Pa) |
| --- | --- | --- |
| 0 | 0 | 8 |
| 1/4 | 9/4 | 11 |
| 1/2 | 6 | 20 |
| 1 | 24 | 56 |
| 1/2 (unloading) | 6 | 20 |
| 0 (unloading) | 0 | 8 |
| −1/4 | −9/4 | 11 |
| −1/2 | −6 | 20 |
| −1 | −24 | 56 |
| 0 (closure) | 0 | 8 |

An absolute floor of 1e-12 Pa and a relative tolerance of 1e-12 are used: allowance for f64 rounding
of a few operations; they do not come from external calibration. For FD, h = 1e-6:
the central truncation error is exactly C h² = 1,6e-11 Pa.

Application of steps 1–4: the method sheet above written first; module
[`cubic_elastic.rs`](../crates/engine-laws/src/uniaxial/cubic_elastic.rs),
kind `elastic.cubic`, version 0.1.0, contract `uniaxial/1`, `CubicElasticParams`,
`CubicElasticFactory`, descriptor `CUBIC_ELASTIC` and public exports.
The test [`cubic_elastic.rs`](../crates/engine-laws/tests/cubic_elastic.rs)
instantiates the kit and runs the native table with source/citation, plus negatives,
purity, symmetry and the linear limit. It requires no solver, API or new dependencies.

```sh
cargo test --offline -p engine-laws --test cubic_elastic
```

Full evidence and review status are in
[ENG-CORE-011](../provenance/ENG-CORE-011.md). The pilot verifies this guide in
the same authoring session; independent J4 review remains pending.

## 6. Close and review

Complete [PROVENANCE.md](plantillas/PROVENANCE.md), add CHANGELOG and apply the
[review checklist](plantillas/REVISION.md). For an AI, start with the
[restricted prompt](plantillas/PROMPT_IA.md). Keep changes uncommitted.

Run all checks from the root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline
cargo check --workspace --offline --target wasm32-unknown-unknown
bash clean-room/check.sh origin/main
```

The clean-room checker includes only files known to Git: to cover
new files without altering the real index, use a temporary index under
`$CARGO_TARGET_DIR`, initialized with `git read-tree HEAD`, add the deliverable
paths there and repeat the command with `GIT_INDEX_FILE` pointing to that index.
If `.git` objects are read-only, also configure
`GIT_OBJECT_DIRECTORY` under `$CARGO_TARGET_DIR` and
`GIT_ALTERNATE_OBJECT_DIRECTORIES` with the output of
`git rev-parse --path-format=absolute --git-path objects` obtained before
exporting that configuration. New objects are then temporary too.
Retain logs and do not change lists or workflows to evade a check.

Also verify all 16 hashes against the DETERMINISMO.md table, with
`MAMUT_ENGINE_GIT_COMMIT` unset, the `determinism_native` and
`determinism_wasm` examples, and `scripts/check-determinism.mjs`; run native/wasm and
debug/release as described in that document. Save outputs under
`$CARGO_TARGET_DIR`; the hash table is not regenerated to accept differences.
`cargo check` wasm only checks compilation; it does not establish wasm execution of the
pilot. Report native execution, comparison across targets and pending work separately.

## 7. CLI, Python and WASM boundaries: MAM-835 / ENG-SFC-010

The following capabilities execute with `engine-input/1` through the same
`engine_api::run` function. CLI retains its workflow envelope and places the
result in `engineResults[0]`; Python `run(...).to_json()` and WASM `run(...)`
return the canonical `results/1` document. Added fields are optional:
previous inputs retain their bytes and the 16 hashes published in
[DETERMINISMO.md](DETERMINISMO.md). The `target` provenance still identifies
the compilation target: complete document equality across surfaces
is verified by compiling them for the same target, including the WASM function on
native. `target` is not removed from results to simulate equality between
binaries for different targets.

The `nonlinearStatic`, `soilProfile` and `consolidation1d` methods are listed as
`Experimental` in `capabilities` and registry descriptors until
ENG-VAL-006 covers their validation (Jonathan's decision via J0 2026-10-20).
To execute them, add `"options": {"allow_experimental": true}` at the root
of the `engine-input/1` document, alongside `analyses`; not inside the analysis
options. Without this explicit opt-in, `run` returns `invalid`
(`API-E-INPUT`, exit 2). SFC-010 fixtures for these methods already include it.
Fixture sources, analytical solutions and tolerances
are in [ENG-SFC-010](../provenance/ENG-SFC-010.md). The numerical core is unchanged
and no section plasticity is added.

### 7.1. Winkler and end releases

In `model.members[]`, select `element: "frame2d.winkler"` and add
`"winkler": {"k": 10}`. For `frame3d.winkler`, use
`"winkler": {"ky": 10, "kz": 20}` and the usual 3D orientation. Stiffnesses
are N/m² per beam length, in local transverse directions.
The foundation is bilateral, linear, without additional mass. Euler
section/material properties and existing `rho` and orientation requirements are retained.
Existing factories also accept these stiffnesses among the scalar
section/law parameters; do not duplicate a property inside and outside `winkler`.

`frame2d.euler_elastic` beams accept boolean `release_i` and `release_j`
that release each end's local z moment. In 3D, the fields are arrays of
three booleans `[T, My, Mz]`, always local. They do not release translations.
Released nodal rotations become disconnected: restrain them or connect them to
other elements to avoid a mechanism. Uniform loads and recovered internal
rotations use the existing condensation. Winkler does not support these
releases: `API-E-SFC-OPTIONS` is returned, without substituting the element.

Complete examples:
[Winkler 2D](../crates/engine-api/fixtures/input/valid/sfc010-winkler2d.json),
[Winkler 3D](../crates/engine-api/fixtures/input/valid/sfc010-winkler3d.json),
[2D releases](../crates/engine-api/fixtures/input/valid/sfc010-release2d.json),
[3D releases](../crates/engine-api/fixtures/input/valid/sfc010-release3d.json).
The constant solution `v=q/k` exactly satisfies the
Winkler–Hetényi equation `EI v''''+kv=q` and belongs to the element's Hermite space.

### 7.2. Springs and uniaxial laws

The optional `model.springs` collection does not require a section. Each spring declares
`id`, `nodes`, `kind` and a local `dof`. One node connects to ground; two distinct
nodes connect to each other and may have coincident coordinates. IDs
share a namespace with `model.members`. Each spring represents a single DOF;
independent springs require separate objects.

```json
{"id": 21, "nodes": [1, 2], "kind": "spring.linear", "dof": "Ux", "k": 200}
```

`spring.linear` requires `k` ≥ 0 and does not accept `law`; `spring.nonlinear` requires a
`law` reference and does not accept `k`. Axes are global if both orientation
fields are omitted. `direction: [x,y,z]` selects the local Ux axis; `rotation`
is a 3×3 matrix whose rows are orthonormal local axes. They are mutually exclusive.
The existing constructor checks dimension, axes and required global DOFs.
No direction is inferred from coordinates.

Laws are declared in `model.laws` with the usual `name` and `parameters`, plus
an optional `kind`:

| Kind | Parameters | Response |
| --- | --- | --- |
| `elastic` | `E` | `F=Eδ` |
| `steel.bilinear_kinematic` | `E`, `fy`, `b` | Post-yield slope `bE`, `0≤b<1`; history `[δp,α]` and kinematic unloading |
| `multilinear_elastic` | `parameters: []`; `points: [[δ,F],...]`; optional `mode` | Increasing table containing `[0,0]`, nonnegative slopes, extrapolation with the end segment |
| `elastic.cubic` | `E`, `C` | `F=Eδ+Cδ³`, `E>0`, `C≥0` |

`mode` values are `both` (default), `compression_only` and
`tension_only`. Points are not mirrored and symmetry is not assumed. In springs the law
is interpreted directly as force–displacement: δ in m, F in N and E as
stiffness in N/m. For rotations it is interpreted as moment–rotation in N·m/rad. Stress–strain
curves are not automatically converted or multiplied by area/length.
Nonlinear laws connect only to `spring.nonlinear`; an elastic beam
with that reference returns `ENG-E-0013`.

Linear springs are supported in `linearStatic` (including combinations) and
`nonlinearStatic`. Nonlinear springs require `nonlinearStatic`. Other analyses
with springs return `ENG-E-0013`. Output identities appear in
`members`: empty `section`, law reference if present and `end_dofs` with the
recovery labels. For a grounded spring the moving end is called `j`. Python tables
group springs by kind and labels to preserve their widths and units.

Examples: [linear series](../crates/engine-api/fixtures/input/valid/sfc010-spring-series.json),
[3D springs](../crates/engine-api/fixtures/input/valid/sfc010-spring3d.json),
[multilinear](../crates/engine-api/fixtures/input/valid/sfc010-multilinear.json),
[cubic](../crates/engine-api/fixtures/input/valid/sfc010-cubic.json) and
[steel with unloading/reversal](../crates/engine-api/fixtures/input/valid/sfc010-steel.json).

### 7.3. Full Newton with load control

```json
{"method": "nonlinearStatic", "options": {
  "case": "P", "load_factors": [0.25, 0.5, 1.0],
  "tolerance": 1e-10, "force_scale": 16, "max_iterations": 30
}}
```

`load_factors` contains **absolute** factors rather than cumulative increments.
Zero, negative values, repetitions and unloading are allowed. From one to 16384 steps and
maximum corrections per step. `tolerance` controls `‖R_libre‖∞/force_scale`.
Positive SI scales are the user's responsibility, especially when
forces and moments coexist. Optionally,
`displacement_tolerance` with `displacement_scale`, and `energy_tolerance` with
`energy_scale` may be added. Each additional criterion requires its scale; all active
criteria must be satisfied. Scales of inactive criteria serialize as 1 if omitted.

A single `NewtonSystem` with its committed history is retained throughout all
steps. Variable springs are excluded from the linear matrix to avoid counting
their stiffness twice. Prescribed displacements are constant, without a load
factor. This method does not support diaphragms or combinations (`ENG-E-0013`).
There is no line search, automatic subdivision or retry.

`convergence.steps[]` retains the factor, iterations, absolute and relative
norms, `converged`, and for each accepted step `results` with nodes/members
and `element_states` in the order of nonlinear springs in the input. Reactions
include variable spring forces. Member extrema
are sampled at the domain's five stations; exact interior extrema are not promised
in this analysis. `cases[0]` contains the last step
if the entire run completes.

On nonconvergence, `results` and `element_states` of the failed attempt are null:
`status: "notConverged"`, diagnostic `ENG-E-0007`, exit 4 and no subsequent
steps. If the tangent is singular, `unstable`/`ENG-E-0006`
and `convergence.failed_factor` are returned. Accepted results are retained within
`convergence`, alongside `accepted_factor`; top-level `cases`, `nodes` and `members`
are emptied according to the existing error policy. Cancellation between steps
(`newtonStep`) also removes convergence. `API-E-SFC-OPTIONS` explains incompatible
or out-of-range options; it is available through `explain`.

### 7.4. Layered profile and consolidation

The `soilProfile` and `consolidation1d` methods use `analyses[0].options` and require
an empty structural model and empty case/combination collections. They do not require
artificial nodes or DOFs. Output adds only `geotechnical` to
`results/1`; historical structural outputs do not receive that property.

A geotechnical datum always includes value and provenance:
`{"value": 10, "provenance": "assumed"}` or `"measured"`. The engine retains
the paths of all assumed parameters in `assumed_parameters`.
A missing physical datum required by the domain returns `ENG-E-0003`, without
substitutions. Incompatible ranges use `ENG-E-0010`; nonfinite data/results
use `ENG-E-0008` (or lexical rejection `API-E-INPUT` for nonfinite JSON numbers).

`soilProfile` receives `layers` ordered from ground level, `groundwater_depth_m`,
`water_unit_weight_kn_m3` and `depths_m`. Each layer declares `thickness_m`,
`gamma_kn_m3` and, if partly underwater, `gamma_sat_kn_m3`.
Unit weights and groundwater level require provenance; γw and zw are
mandatory even in dry conditions. Depth is positive downward, within
the profile, without extrapolation. `states[]` returns `total_vertical_kpa`,
`pore_pressure_kpa` and `effective_vertical_kpa`, compression positive, with
`σ'v=σv−u` and hydrostatic pressure. Negative effective stresses are not clipped.
[Hydrostatic fixture](../crates/engine-api/fixtures/input/valid/sfc010-soil.json).

`consolidation1d` receives layers with `thickness_m`, `cv_m2_s`, `mv_per_kpa`,
`k_m_s` (all with provenance) and `elements`. `water_unit_weight_kn_m3`
and the relation `cv=k/(mv γw)` are required; missing data are not computed. `drainage` accepts
`top`, `bottom` or `double`. The `history` contains
`time_s`/`delta_stress_kpa` points with increasing time from zero; interpolation is linear.
`output_times_s` is increasing, nonnegative and within the history.
`theta` in [0,1] and positive `max_step_s` are explicit. Layers may have
different properties and share nodes at their interfaces.

Output contains common `depths_m` and states with nodal excess pressure
`excess_pore_pressure_kpa`, `settlement_m` and `mean_degree`. The mean degree is
null with zero stress increment and may leave [0,1] during unloading.
Finite discretization can produce nonzero initial settlement due to the drain;
it is neither modified nor clipped. The domain retains its limits of 2048 elements,
8000000 output nodal values and 1000000 substeps per interval. The boundary
also applies the following public caps, before integration:

| `engine_api` constant | Cap | Rejection (`invalid`, exit 2) |
| --- | ---: | --- |
| `MAX_CONSOLIDATION_WORK` (r2, retained) | 50_000_000 substeps × elements | `API-E-CONSOLIDATION-WORK` |
| `MAX_CONSOLIDATION_COST` (r3) | 500_000_000 | `API-E-CONSOLIDATION-COST` |
| `MAX_CONSOLIDATION_SEGMENTS` (r3) | 256 history/output intervals | `API-E-CONSOLIDATION-COST` |
| `MAX_CONSOLIDATION_SUBSTEPS` (r3) | 100_000 aggregate substeps | `API-E-CONSOLIDATION-COST` |

The dense cost is `Σ_intervalos n³ + Σ_subpasos n²`, where `n` is the number
of free nodes: total elements for single drainage, elements minus one for
double drainage. Each interval between load breakpoints and output times contributes
`max(1, ceil(intervalo/max_step_s))` substeps. The **entire** history is counted,
even after the last requested output, without discounting factor
reuse. Sums, powers and products use saturating `u128`; a nonfinite quotient
also saturates. The r2 bound is checked first. The interval cap
also limits history to 257 points; the cost implies at most 793
elements with single drainage or 794 with double drainage. No other mesh cap is needed.
Interval and substep caps bound the fixed work for small meshes.
Reduce history/mesh/outputs or increase `max_step_s` if the input is rejected.

Release calibration on this machine and accepted extreme cases:
[r3 provenance](../provenance/ENG-SFC-010.md#r3--revisión-j4-h1).
The goal is completion in a few seconds (≤ approximately 5 s); no deadline
is promised on other machines, targets or debug profiles. The backend
remains dense: O(n³) per factorization and O(n²) per solve.

`ConsolidationProfile::solve_with_cancel` queries the closure before
combine/factorize of each interval, between blocks of up to 16 columns of the
reference factorization and before each substep. The facade connects the
`CancelToken` and returns `API-I-CANCELLED` (`cancelled`, exit 5), without partial
outputs. Equilibration and each dense solve remain indivisible.
`solve` uses a false-returning closure. Only the reference backend reuses the last
factor if consecutive `dt` values match bitwise: M, K and theta are immutable and
the matrix and factor are identical. A different dt invalidates the cache.
Caller-supplied backends retain every factorization call.
Operation order, time partitioning and results are preserved;
progress is still reported before/after integration.
[Multilayer Terzaghi fixture](../crates/engine-api/fixtures/input/valid/sfc010-consolidation.json).

All three surfaces discover these contracts with `schema("engine-input/1")`,
`schema("results/1")`, `capabilities` and `explain`; Python also publishes its DTOs
with optional fields. Parity tests compare canonical bytes of each
fixture, extracting the document from the CLI envelope when appropriate.
