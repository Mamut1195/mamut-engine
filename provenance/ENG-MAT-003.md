# Clean-room log · ENG-MAT-003

- Ticket / PR: ENG-MAT-003, «Acero bilineal con endurecimiento cinemático»; local changes, without commit or push.
- Author(s): J2 (agent, Grok Bot); date 2026-10-11 (America/Guayaquil). Identification requested for the delivery; implementation session executed by Codex. No delegation to other agents.
- Role: implementer and author of the own analytical derivation; no oracle operator or similarity auditor activity.
- Implemented contract: `uniaxial/1`, kind `steel.bilinear_kinematic`, contract 1.0 and method 0.1.0.
- Permitted sources used: user's specification, internal contracts and the repository's own code listed below. Citation: **own derivation of 1D plasticity with linear kinematic hardening**, documented in this note. No books, invented pages, network access or external sources.
- Own derivations: [1D return](#derivación-propia-del-retorno-1d), [cycle and energy](#tabla-analítica-del-ciclo) and [fixtures](../crates/engine-laws/tests/steel_bilinear.rs).
- Third-party code copied: none. Only standard library and the existing local dependencies `engine-core` and `engine-contracts`; no new crate or dependencies.
- Validation: analytical; hysteresis table, Bauschinger effect, closed-cycle area, perfect plasticity, tangents through finite differences, conformance with history and invalid input diagnostics.
- Oracle comparison (sibling repo, informative): no. No external engine was used and the oracle repo was not consulted.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Clean-room reviewer: PENDING (J4).

## Internal documents consulted

`provenance/TEMPLATE.md`, `provenance/ENG-MAT-001.md`,
`docs/CLEAN_ROOM.md`, `UniaxialLaw`, `UniaxialResponse`,
`conformance::check_uniaxial_law`, the `Elastic` law, its validators,
factory and tests, the manifests and `clean-room/check.sh` were read.

This checkout does not contain the multilinear law or the ENG-MAT-002 log.
Through `git show`, only the in-house files
`crates/engine-laws/src/uniaxial/multilinear.rs` and `uniaxial.rs` from the
local branch `j2/eng-mat-002-multilinear` were consulted, to follow the pattern of
`MultilinearElasticFactory`: private module re-exported in `uniaxial`,
typed parameters, static descriptor and construction through
`UniaxialFactory`. That branch was not merged and the real Git index was not altered.
No applicable `AGENTS.md` was found in this checkout or its ancestors.

The declaration requested for tickets ENG-MAT-001, ENG-MAT-002 and
ENG-MAT-003 is the one transcribed above. This log records the activity of
the ENG-MAT-003 session; it does not retrospectively certify other sessions
or replace the historical reviews of the other tickets.

## Derivación propia del retorno 1D

ε = ε_e + ε_p, σ = Eε_e and a translatable surface
|σ−α| ≤ fy are adopted. E, fy and α are expressed in Pa; ε and ε_p are dimensionless.
There is no isotropic hardening: fy remains constant.
The flow rule is Δε_p = sΔγ and kinematic evolution is
Δα = HsΔγ, where s = sign(σ_tr−α) and H ≥ 0.
For a zero initial state, α = Hε_p along any path.

The prediction from the committed state is σ_tr = E(ε−ε_p),
ξ = σ_tr−α and f = |ξ|−fy. If f ≤ 0, Δγ = 0 and σ = σ_tr.
If f > 0, consistency on the final surface requires
s(σ−α') = fy. Substituting σ = σ_tr−EsΔγ and
α' = α+HsΔγ, we obtain |ξ|−(E+H)Δγ = fy and therefore
Δγ = f/(E+H), nonnegative. This is the exact implicit return in 1D.

Away from a vertex, ds/dε = 0 and df/dε = sE. Then
dσ/dε = E−E²/(E+H) = EH/(E+H). We define the parameter
b = (dσ/dε)_plástico/E, so that H = bE/(1−b) and 0 ≤ b < 1.
The elastic branch has tangent E and the plastic branch bE.
At f = 0 the elastic branch is selected; the two-sided derivative is not unique.

When reversing from σ = α+fy toward compression, α and ε_p remain
constant until σ = α−fy. The elastic excursion is exactly
2fy in stress and 2fy/E in strain. From compression the same
occurs with the opposite sign. The center α changes with plastic
strain, producing the Bauschinger effect without changing the yield width.

Stored energy is ψ = σ²/(2E) + α²/(2H) for H > 0.
Using α = Hε_p, the second term is equivalent to Hε_p²/2.
Plastic work satisfies σ dε_p = d(Hε_p²/2) + fy |dε_p|.
In a cycle returning to the same ε, σ, ε_p and α, the change in ψ is zero,
so ∮σ dε = fy∫|dε_p|. For H = 0 there is no stored kinematic
term; the same dissipation formula is preserved.

## Tabla analítica del ciclo

Source and citation for all fixtures: own derivation in this note,
without published examples or results from another engine.
E = 200 Pa, fy = 1 Pa, b = 0,1; H = 200/9 Pa.
Under positive plastic loading from the origin:
σ = 0,9 + 20ε, ε_p = 0,9(ε−0,005), α = 20(ε−0,005).
The peak ε = 0,02 leaves σ = 1,3, ε_p = 0,0135 and α = 0,3.
Elastic unloading ends at ε = 0,01 and σ = −0,7.
Then, σ = 20ε−0,9 until ε = −0,02, where
σ = −1,3, ε_p = −0,0135 and α = −0,3. Elastic reloading
ends at ε = −0,01 and σ = 0,7; then σ = 20ε+0,9.

| Target ε | σ (Pa) | α (Pa) | ε_p | Tangent (Pa) |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 0 | 0 | 0 | 200 |
| 0,0025 | 0,5 | 0 | 0 | 200 |
| 0,005 | 1 | 0 | 0 | 200 (f = 0) |
| 0,02 | 1,3 | 0,3 | 0,0135 | 20 |
| 0,0175 | 0,8 | 0,3 | 0,0135 | 200 |
| 0,015 | 0,3 | 0,3 | 0,0135 | 200 |
| 0,0125 | −0,2 | 0,3 | 0,0135 | 200 |
| 0,005 | −0,8 | 0,2 | 0,009 | 20 |
| −0,02 | −1,3 | −0,3 | −0,0135 | 20 |
| −0,0175 | −0,8 | −0,3 | −0,0135 | 200 |
| −0,015 | −0,3 | −0,3 | −0,0135 | 200 |
| −0,0125 | 0,2 | −0,3 | −0,0135 | 200 |
| −0,005 | 0,8 | −0,2 | −0,009 | 20 |
| 0,02 | 1,3 | 0,3 | 0,0135 | 20 |

The closed cycle starts at peak A = (0,02; 1,3), excluding the initial
loading, and traverses B = (0,01; −0,7), C = (−0,02; −1,3),
D = (−0,01; 0,7) and A. Vectors AB = (−0,01; −2) and
AD = (−0,03; −0,6) give area |det(AB, AD)| = 0,054 Pa.
It is also obtained as 4fy(1−b)(ε_max−fy/E) = 4·1·0,9·0,015.
The exact integral along the sides gives −0,003+0,03−0,003+0,03 = 0,054 Pa.
Accumulated plastic variation is 2·(0,0135−(−0,0135)) = 0,054;
multiplying by fy matches the dissipated area (J/m³).

For b = 0, initial loading reaches σ = fy at ε = fy/E = 0,005
and continues on the plateau σ = 1; ε_p = ε−0,005, α = 0 and tangent 0.
The negative branch has σ = −1 and ε_p = ε+0,005; unloading/reloading
inside the surface preserves tangent 200 and committed history.

## Numerical stability, errors and history

The implementation does not form H, E+H or fy/E. It uses the identities
EΔγ = (1−b)f, HΔγ = bf, Δγ = ((1−b)f)/E and tangent bE.
It evaluates σ = α'+s fy to avoid cancellation of σ_tr−EsΔγ when b = 0
and the prediction is much larger than fy. E = 1e300, ε = 1e−300 and enormous fy
give σ ≈ 1 without overflow. A b close to 1 is also verified
with unrepresentable H, and small E with unrepresentable fy/E.

`ENG-E-0010` diagnoses E ≤ 0, fy ≤ 0, b outside [0,1) and history
of length other than two. `ENG-E-0008` diagnoses nonfinite inputs
and overflows in ε−ε_p, σ_tr, ξ, Δγ or the results.
f = |ξ|−fy is finite when ξ and fy are finite and fy > 0;
(1−b)f, bf and bE cannot overflow within their validated ranges either.
Library code does not use `panic`, `unwrap` or `expect`.

The API does not check physical consistency between two arbitrary components
of a finite history supplied by the caller; it guarantees its length
and finiteness. Histories produced from the origin satisfy α = Hε_p
within rounding. At extreme scales f64 may lose increments
or fail to resolve ε−ε_p with sufficient precision; uniform relative
accuracy or exact representation of any real history is not claimed.

Every call is pure. Tests repeat responses, discard trials,
modify a copy of the trial history and verify that the law and the
committed state retain their values. Acceptance consists only of
the caller assigning `committed = trial.trial_state`.

## Local verification

The 11 new tests and the material doctest pass natively, together with
the existing elastic tests. Central finite differences use
h = 1e−7 and samples away from vertices, with committed history held constant and relative
error ≤ 1e−6. The existing kit traverses cycles explicitly accepting history
for b = 0, 0,1 and 0,8. Tests need no files,
network, threads or clock. wasm compilation verifies portability, not numerical
execution of tests in a wasm runtime.

Final results:

- `cargo fmt` and `cargo fmt --check`: successful.
- `cargo clippy --all-targets -- -D warnings`: successful.
- `cargo test --workspace`: successful; includes the 11 new tests and
  the material doctest.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: successful.
- `cargo check --workspace --target wasm32-unknown-unknown`: successful
  with `PATH=/home/box/.cargo/bin:$PATH` and `CARGO_NET_OFFLINE=true`, using
  the local Rust 1.99.0 toolchain that already has the target installed.
  The initial attempt with the default Rust 1.85.1 failed with E0463 due to
  absence of the wasm standard library in that toolchain. The other
  checks above passed with Rust 1.85.1. There were no downloads.
- `bash clean-room/check.sh origin/main`: `Clean-room: OK`, on a temporary index
  and object directory in `/tmp`, with the three new files
  included through intent-to-add. The attempt on the real index did not see
  the new untracked log and requested provenance; the temporary index
  allows checking all the new files as well without altering the real one.
- `git diff --check`: successful, also on the full diff of the
  temporary index. The diff contains only five files: general
  documentation and re-exports of `engine-laws`, new module, tests and this note.

No changes to manifests, dependencies, contracts or protected paths.
There was no commit, push, network access or oracle execution. Independent clean-room
review: PENDING (J4).
