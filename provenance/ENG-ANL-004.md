# Clean-room log · ENG-ANL-004

- Ticket / PR: ENG-ANL-004 «Response spectrum», branch `j2/eng-anl-004`
  stacked on ENG-ANL-002 (#61, unmerged), its chain ENG-MASS-001
  (#55 → ENG-LOAD-001 #51), ENG-ANL-001 (#52) and ENG-LA-009 (#58), unmerged.
  No commit or push.
- Author(s): J2 (agent, Grok Bot) · label requested by the user;
  executing Codex based on GPT-6 · 2026-10-11 (America/Guayaquil).
- Role: implementer and author of original analytical derivations; without delegation,
  auditor activity or oracle operation.
- Implemented contract: `spectrum`, API `engine_analysis::spectrum::Spectrum::run`
  over `LinearSystem` and ENG-ANL-002; SI tables, peak modal response,
  reactions, base shear, SRSS/CQC and explicit directional combination.
  Does not adapt the generic trait `AnalysisMethod` (sink/control/settings).
- Permitted sources used: exclusively the user's request and the relevant
  own APIs and code of the workspace: engine-analysis, engine-assembly,
  engine-loads, engine-linalg, engine-core, engine-contracts, engine-model and
  engine-elements-frame; `provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md` and
  the original analytical notes of ENG-ANL-001/002. The CQC formula and its
  attribution to Wilson–Der Kiureghian–Bayo (1981) come from the request;
  that article or any other publication was not consulted. No network or external sources.
- Original derivations: [equations of this note](#derivaciones-propias) and
  [analytical fixtures](../crates/engine-analysis/tests/spectrum.rs), marked
  `SOURCE` / `CITATION`.
- Third-party code copied: none. No new dependencies, internal or external.
- Validation: exclusively analytical, without published values: N-story
  shear building, correlations, combination identities, independent
  oscillators and axial bar with consistent mass and member recovery.
- Oracle comparison (sibling repo, informational): no. No external engine
  was consulted or run.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Clean-room reviewer: PENDING (J4).

## Stacking and declarations of the three tickets

The logs of ENG-ANL-001, ENG-ANL-002 and ENG-ANL-004 contain the
same full declaration, author J2 (agent, Grok Bot) and «Revisor clean-room:
PENDIENTE (J4)». The three previous dependencies ENG-MASS-001, ENG-ANL-001
and ENG-LA-009 also retain those fields. Their derivations are not replaced
and their PRs are not claimed to be merged. The modal module from #61 is consumed
without changing the solver, mass reduction or contracts of the base branches.

## Interface, units and explicit decisions

`TabulatedSpectrum::new(pairs, units, damping, interpolation, extrapolation)`
requires at least two pairs and a decision for each option. The table is
immutable after validation. `SpectrumUnits` declares periods in seconds or
frequencies in Hz, always with Sa in m/s². Sa is spectral pseudoacceleration
for the damping ζ of that table. Sa is not interpreted as a multiple of g,
there is no gravity conversion or automatic adjustment for ζ. ζ=0 and
Sa=0 are allowed in linear interpolation; ζ must be less than 1.

The original abscissas must be strictly increasing. Periods
may start at zero; frequencies must be positive. A frequency
table is transformed through T=1/f and its order is reversed: both
interpolations are ALWAYS in period. Invalid inputs are not sorted.

- Linear: a=(T−T₀)/(T₁−T₀), Sa=(1−a)Sa₀+aSa₁.
- Log-log: a=ln(T/T₀)/ln(T₁/T₀), ln Sa=(1−a)ln Sa₀+a ln Sa₁;
  requires strictly positive T and Sa at every point. `ln_1p` is used on
  close intervals and differences of ln when the quotient would overflow.
- Values at exact abscissas are retained. `Extrapolation::Error`
  rejects values outside the closed interval. `Extrapolation::Plateau`, chosen
  explicitly by the user, retains Sa of the corresponding endpoint;
  includes T=0 outside a log-log table. Extrapolation is never silent.

`Spectrum::run(system, directions, options)` receives nonzero directions,
normalized without overflowing their norm. Each `SpectrumDirection` declares a
table common to all modes or exactly one table per mode, in ascending
frequency order. This allows common ζ or ζ_n and Sa(T,ζ_n) supplied
by the user. There is no interpolation between damping values or modification of Sa.
`SpectrumOptions` declares mode count, MassKind, modal combination,
directional combination and a nonnegative factor per direction; it has no Default.
There are no code-prescribed modal mass minima or built-in percentages.

Example of options supplied by a caller (illustrative user
values, without code-prescribed meaning):

```rust
use engine_analysis::spectrum::{
    DirectionalCombination, Extrapolation, Interpolation, ModalCombination,
    SpectrumOptions, SpectrumUnits, TabulatedSpectrum,
};
use engine_contracts::MassKind;
let table = TabulatedSpectrum::new(
    &[(0.0, 1.0), (10.0, 3.0)],
    SpectrumUnits::PeriodSecondsAccelerationMetersPerSecondSquared,
    0.05,
    Interpolation::Linear,
    Extrapolation::Error,
)?;
let options = SpectrumOptions {
    mode_count: 4,
    mass_kind: MassKind::Consistent,
    modal_combination: ModalCombination::Cqc,
    directional_combination: DirectionalCombination::WeightedSum,
    directional_factors: vec![1.0, 0.3], // Dos direcciones declaradas por el usuario.
};
# Ok::<(), engine_core::EngineError>(())
```

Results retain tables, options, unit directions, shapes and
participations from ENG-ANL-002, ζ and Sa per mode, modal coordinate, equivalent
forces, free/global displacements, reactions and base shear. Modal
responses are signed; combined responses are envelopes for each scalar. The system
does not retain `Element` objects or their contexts; for members, the caller extracts
u_e from each global displacement, uses `Element::recover` and combines each
signed action using `combine_modal`. A test demonstrates this path with the
axial force of an existing frame. Members are not recovered from the combined
displacement, which would lose modal signs and correlations. No results/1
facade is added and nonlinear states are not combined.

## Derivaciones propias

### Modal response, forces and reactions

For mass-normalized shapes, φ_nᵀM_ffφ_n=1, directional excitation uses
Γ_n=φ_nᵀM_ff r_f. From the uncoupled system and the definition of pseudoacceleration
Sa=ω_n² S_d the signed peak coordinate is obtained:

```text
q_n,max = Γ_n Sa(T_n,ζ_n)/ω_n²
u_n = φ_n q_n,max
F_n = M_global φ_n Γ_n Sa(T_n,ζ_n)
R_n,p = (K_global u_n − F_n)_p
V_n = −Σ_apoyos R_n,traslacional · d̂
```

Prescribed shapes and perturbations are zero, even with settlements.
Static loads are not added and `Constraints::expand`, which would add the
static prescribed values, is not called. M_fp r_p is not used in Γ either. M_global φ may have
components at supports with consistent mass: F_p is subtracted when recovering R_p.
Physical base shear is not generally replaced by M*_n Sa, because the
latter only projects the free forces in the modal definition used.

For the axial bar in the test, E=12, A=L=1, ρ=6, M_axial=[[2,1],[1,2]],
K_ff=12, M_ff=2, ω²=6, φ_f=1/√2 and Γ=√2. With Sa=2:
u_f=1/3, F_p=2, F_f=4, R_p=−12/3−2=−6 and V=6 N.
The recovered axial action is EA u_f/L=4 N. A settlement of 0.2 m and the
previous accepted state u_f=0.7 do not participate or change during the analysis.

### Shear building: original analytical substitute, without a published example

**No published example from Chopra is reproduced**: its
values are unavailable and his book was not consulted. The requirement is verified with the shear
building derived in ENG-ANL-002. For N identical stories, k per story and m per
story, q₀=0 and free roof:

```text
θ_n = (2n−1)π/(2N+1), n=1,…,N
ω_n² = 4(k/m) sin²(θ_n/2)
s_jn = sin(jθ_n)
c_n = √(m Σ_j s_jn²)
φ_jn = s_jn/c_n
Γ_n = m Σ_j s_jn/c_n
M*_n = Γ_n²
u_jn = φ_jn Γ_n Sa_n/ω_n²
F_jn = m φ_jn Γ_n Sa_n
V_n = Σ_j F_jn = Γ_n² Sa_n = M*_n Sa_n
R_base,n = −k u_1n = −V_n
```

A simultaneous sign change of φ and Γ leaves u, F and V unchanged. The tests
calculate these expressions directly with sines, finite sums and the parameters
N=1,4,9; k=17 N/m; m=3 kg. Constant Sa of 2.5 m/s² and the linear table
(T,Sa)=(0,1),(100,21), which gives Sa(T)=1+0.2T over the interval, are used. Γ, q, u, F,
reaction and base shear are verified with tolerance 3e−11·max(1,|valor esperado|).
Completeness satisfies Σ M*_n=Nm, for the N modes of this fixture.

By DEFINITION of SRSS applied to the base-shear scalar:

```text
V_b,SRSS² = Σ_n V_n² = Σ_n (M*_n Sa_n)²
Para Sa_n=Sa constante: V_b,SRSS² = Sa² Σ_n (M*_n)².
```

This is derived here and verified with all modes; it is not
Σ_n M*_n Sa_n = V_b,SRSS², a dimensionally incompatible expression. The linear
sum of forces, Σ_n M*_n Sa, for this complete fixture is Nm Sa, but
it is not its SRSS combination. This derivation is not attributed to a publication.

### CQC and directional combination

The general formula supplied by the user is:

```text
r = ω_j/ω_i
ρ_ij = 8√(ζ_iζ_j)(ζ_i+rζ_j)r^(3/2) /
       [(1−r²)² + 4ζ_iζ_j r(1+r²) + 4(ζ_i²+ζ_j²)r²]
```

With ζ_i=ζ_j=ζ, the denominator reduces to
(1−r²)²+4ζ²r(1+r)² and the numerator to 8ζ²(1+r)r^(3/2), the
Wilson–Der Kiureghian–Bayo (1981) expression given in the request. Swapping i,j is equivalent
to r→1/r and dividing both polynomials by r⁴; thus ρ_ji=ρ_ij. It is evaluated with
r≤1 and products normalized by the square root of the denominator, avoiding large
quotients and subnormal squares. Final rounding is clamped to the interval [0,1].
For r=1 and common positive ζ, numerator=denominator=16ζ² ⇒ ρ=1.
With ζ_i=ζ_j=0 and r=1 the expression is 0/0: the convention ρ=1 is declared.
With both ζ=0 and r≠1, ρ=0. With r=1 and different ζ:
ρ=2√(ζ_iζ_j)/(ζ_i+ζ_j), zero if only one mode is damped.

The tests compare both formulas, symmetry, bounds [0,1], unit diagonal,
subnormal ζ and extreme frequencies. For ω=(1,1000) rad/s, ζ=0.05 and
R=(3,4), ρ<1e−6 and |CQC/SRSS−1|<1e−6 are required. For close frequencies
correlation >0.9 and cross terms with opposite responses are checked.

```text
SRSS: √Σ_n R_n²
CQC:  √Σ_iΣ_j ρ_ij R_i R_j
Direcciones SRSS: √Σ_d (a_d R_d)²
Direcciones suma ponderada: Σ_d a_d R_d
```

R is scaled by its maximum magnitude before the quadratic form. Only a
rounding negative of up to 64ε n² is allowed before taking the square root; a more
negative form returns ENG-E-0012. Free/global components and each
reaction are combined separately. For V each set of signed reactions
is summed per mode first and then the scalar is combined. Two separately supported
oscillators with m=(2,3), k=(18,48), Sa=2 give V=(4,6): V_SRSS=√52,
whereas the sum of support envelopes is 10. A test verifies this difference.
Combination across directions gathers quantities projected in different
directions; it is not interpreted as a new direction or as equilibrium.

## Errors and limits

- ENG-E-0010: short tables, abscissas without strict ordering/negative, negative Sa,
  f=0, log-log with zeros, ζ outside [0,1), negative/out-of-range T queries
  with Error policy, empty lists, rigid modes, zero direction,
  incompatible table/factor cardinality or negative factors.
- ENG-E-0008: nonfinite input or result, inversion of f that overflows,
  unrepresentable displacements/forces/products/sums.
- Modal/LinearSystem diagnostics are propagated, including ENG-E-0006,
  ENG-E-0007, ENG-E-0011 and ENG-E-0012; matrices are not regularized or condensed.
- There is no `panic!`, `unwrap` or `expect` in engine-analysis/src. Internal
  indexing uses dimensions guaranteed by private constructors or validated
  Modal results. `SystemView::dof_label` with the preexisting `expect`
  in engine-assembly is not called.
- The shapes are for undamped free vibration; CQC uses classical modal
  damping, not nonproportional damping matrices. Responses
  are estimated maxima, not histories, simultaneous states or design signs.
- Limits and tolerances of the stacked modal solver are inherited. Completeness
  with truncated modes is not guaranteed. The modal path still uses the dense solver.

## Local verification

All Cargo checks were run with `CARGO_NET_OFFLINE=true`,
without network, downloads or dependency installation.

- `cargo fmt` and `cargo fmt --check`: OK, exit 0.
- `cargo clippy --all-targets -- -D warnings`: OK, exit 0.
- `cargo test --workspace`: OK, exit 0; 257 tests and doctests, including
  12 new spectral tests. The spectral tests were first run
  separately; the test's access to `StationResult::forces_local` was corrected
  before the full verification.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, exit 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, exit 0,
  with `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`, an already
  installed toolchain containing std for that target. Nothing was downloaded.
- `bash clean-room/check.sh origin/main`: OK, exit 0. The new files
  were included through `git add -N` exclusively in a temporary index and
  object directory under `/tmp`, using the existing objects
  as read-only alternates. The real index was not modified and no
  commits were created. `git diff --check` with that index: OK.
- Search for `panic!`, `.unwrap(` and `.expect(` in engine-analysis/src:
  no matches. The tests use assertions and unwrap in the fixture,
  following the convention of the existing analytical tests.

Native compilation, tests and rustdoc: system Cargo/rustc 1.85.1
(MSRV); installed Clippy: 0.1.99. The wasm check used rustc 1.99.0
from the already installed `stable` toolchain.

The wasm check verifies compilation, not execution in a wasm runtime or
numerical equality between targets. J4's clean-room review remains pending.
Logs: `/tmp/eng-anl-004-{fmt,clippy,test,doc,wasm,clean-room,diff-check}.log`.
No commit or push. Clean-room reviewer: PENDING (J4).
