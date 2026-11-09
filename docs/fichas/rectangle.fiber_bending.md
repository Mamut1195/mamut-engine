# Method sheet · rectangle.fiber_bending · issue 5

Complete before implementation. The expected axial force and moment below were
evaluated from the equations in this sheet before the section was written.
This session is a clean session, distinct from the session of 2026-11-04.
It did not view numerical output of an external structural program.

- Author, model if AI, date and role: agente cloud de Cursor (Cursor Agent),
  modelo: DESCONOCIDO, 2026-11-09 (America/Guayaquil). Role: specifier and implementer.
- Own kind, contract and method version: `rectangle.fiber_bending`,
  `section/1` (`ContractId::new("section", 1, 0)`), method `0.1.0`.
- Module/crate and intended consumer: `crates/engine-laws/src/section/rectangle_fiber.rs`,
  crate `engine-laws`. Consumer: a caller that already holds a `SectionLaw`.
  The kind is not added to `capabilities().laws` and it is not an
  engine-input kind. Ticket 03 can call `respond`.
- Existing methods reviewed and why this one is distinct: `elastic_frame`
  is a homogeneous linear section with stiffnesses supplied by the caller.
  `concrete.kent_park` and `steel.bilinear_kinematic` are uniaxial laws.
  This section samples those two laws on a rectangle and sums fiber forces.
  It does not replace `elastic_frame` and it does not add another steel law.

## Permitted sources

Kinematics and resultants already written in this repository:
`docs/AXES_CONVENTION.md` (approved 2026-10-10: X-Y horizontal, Z upward;
in D2 local z points toward +Z) and `crates/engine-laws/src/section.rs`
(`ElasticFrame`). For D2 the stored convention is ε(y) = ε0 − κz y,
N = ∫ σ dA and Mz = ∫ (−y) σ dA, with tension positive. A textbook page
for that statement was not in this session: cita desconocida.

Concrete. The envelope already transcribed in
`docs/fichas/concrete.kent_park.md`, equations (A.9) and the sign map
σ = −f_c, K = df_c/dε_c. The bibliographic record in that sheet is
Sharma, Akanshu; Reddy, G. R.; Vaze, K. K.; Ghosh, A. K.; Kushwaha, H. S.;
Eligehausen, Rolf. *Joint model to simulate inelastic shear behavior of
poorly detailed exterior and interior beam-column connections reinforced
with deformed bars under seismic excitations.* Bhabha Atomic Research
Centre, report BARC/2009/E/026, 2009, Appendix A, section A.1.1.3.
This session did not open the report. Printed page: cita desconocida.
Only the parabola (A.9) is evaluated at the control strains. Tension, which
that curve does not define, stays the zero stress and zero tangent already
chosen in the existing law.

Steel. Original statement in `crates/engine-laws/src/uniaxial/steel_bilinear.rs`:
with accepted history `[ε_p, α] = [0, 0]` and |E(ε − ε_p) − α| ≤ fy, the
response is σ = E ε and tangent E. A textbook page for kinematic hardening
was not in this session: cita desconocida.

Quadrature, strip count, bar positions and the sums (6)–(9) are an original
derivation of this sheet (2026-11-09). No third-party listing, table or
figure was copied. The strip count was not moved to resemble another program.

## Assumptions, units and domain

- Physics and limitations: a rectangle in the D2 plane, discretized into
  equal concrete strips, plus discrete steel bars. Concrete uses one shared
  `concrete.kent_park` law. Steel uses one shared `steel.bilinear_kinematic`
  law. The concrete area is the gross rectangle. Bar area is added and is
  not removed from the concrete. There is no shear strain, no torsion and
  no biaxial curvature. D3 is rejected. This is not a member: a later ticket
  integrates the section along a bar.
- Total inputs, SI units and signs: D2 e = [ε0, κz]. ε0 is dimensionless.
  κz is in m⁻¹. y is in m, positive along local y. Tension is positive, the
  same sign as both uniaxial laws and as `ElasticFrame`. N is in N. Mz is
  in N·m.
- Parameters, units, ranges and validation:
  - `width` b in m and `height` h in m, finite and strictly positive.
  - `strip_count` n, an integer from 1 to 64 inclusive. The count is the
    domain of method 0.1.0. It does not approve a time or memory budget.
  - Concrete parameters are the existing Kent–Park parameters. The same
    errors apply (`ENG-E-0008`, `ENG-E-0010`).
  - Steel parameters are the existing bilinear parameters. The same errors
    apply.
  - From 1 to 32 bars. Each bar has area in m², finite and strictly
    positive, and a centroid y in m with |y| ≤ h/2. This count is a domain
    of the method, not a time or memory budget.
  - The control section uses n = 4 and 2 bars. Those values are the ones
    written here, not a mesh adjusted after a comparison.
- State: concrete history length is 0 and is not stored. Each bar stores
  `[ε_p, α]` in the steel law’s units, in the order the bars were given.
  Section state length is 2 times the bar count. The initial state is zeros.
  A history of another length is `ENG-E-0010`. A nonfinite history entry is
  `ENG-E-0008`.
- Outputs and units: resultants `[N, Mz]`, a 2×2 tangent in row-major order
  `[∂N/∂ε0, ∂N/∂κz, ∂Mz/∂ε0, ∂Mz/∂κz]`, and the trial history. `respond`
  does not mutate the section or the committed history.
- Numerical domain, operation order, overflow/underflow. For each fiber,
  in the order concrete strips i = 0..n−1 and then the bars:
  1. Δy = h/n and y_i = (i + 1/2) Δy − h/2, formed when the section is built.
     Strip area A_i = b Δy. Both must be finite and strictly positive.
  2. Fiber strain ε = ε0 − (κz · y). The product is formed first.
  3. The uniaxial law returns σ and K.
  4. Fiber force σ A, then fiber moment (−y)(σ A).
  5. Fiber axial stiffness K A, coupling (−y)(K A), bending (−y) times that
     coupling. The two coupling entries of the tangent are this one sum.
  6. Running sums are updated in that fiber order. A nonfinite intermediate
     or sum is `ENG-E-0008`.
- Nondifferentiable points and the convention at each boundary. Each fiber
  inherits the branch choice of its law. Concrete: engine strain 0, the peak
  ε_c = 0.002, and ε_c = ε_20c. Steel: the yield surface |ξ| = fy, where the
  elastic branch is the one the steel law returns. Central differences in
  the conformance sample stay inside one smooth piece of every fiber. They
  do not cross those points.
- Constant axial force. κz and the committed history are held. The unknown
  is ε0. The update is equation (10): it uses ∂N/∂ε0 from the section
  tangent. The caller supplies the first guess, a residual limit in N, and
  an update count from 1 to 32. The count is a domain of the procedure, not
  a time budget. A zero axial tangent returns `ENG-E-0006` because the
  strain is not determined. Exhausting the updates without meeting the
  residual limit returns `ENG-E-0007`. A trial that is not returned does
  not replace the committed history.

## Original derivation

1. Strip geometry, equation (1). For the control section, b = 1/4 m,
   h = 1/2 m and n = 4. Then Δy = 1/8 m and A_i = 1/32 m². The centroids are
   y = −3/16, −1/16, +1/16, +3/16 m.
2. Bars, equation (2). Two bars, each of area A_s = 1/1024 m², at
   y = −3/16 m and y = +3/16 m. They sit on the outer strip centroids.
   Their area is still added on top of the gross concrete area.
3. Kinematics, equation (3): ε(y) = ε0 − κz y.
4. Materials. Concrete parameters of the control section: f'_c = 3000 psi,
   ρ_s = 0, b'' = 0.2 m, s_h = 0.1 m. With ρ_s = 0 the hoop sizes do not
   enter (A.9). f'_c in pascals is 3000 times the existing unit factor
   4.4482216152605 / 0.0254². Steel: E = 200000000000 Pa, fy = 500000000 Pa,
   b = 0. The control history is the initial zeros.
5. Control deformation, chosen so that every concrete strain is compressive,
   below the peak, and a dyadic rational: ε0 = −1/1024, κz = 1/256 m⁻¹.
   The four strains, shared by the bars at the outer stations, are
   ε = −1/4096, −3/4096, −5/4096, −7/4096.
   Then ε_c / 0.002 = 125 k / 1024 for k = 1, 3, 5, 7. All four values are
   below 1, so (A.9) applies. The largest steel stress has magnitude
   7 E / 4096 = 341796875 Pa, which is below fy, so equation (5) applies
   and the tangent is E.
6. Parabola ratios r = x (2 − x) = 125 k (2048 − 125 k) / 1048576:
   k = 1 gives 240375/1048576, k = 3 gives 627375/1048576,
   k = 5 gives 889375/1048576, k = 7 gives 1026375/1048576.
   The numerator sum is 2783500. With A_i = 1/32,
   N_concrete = −f'_c · 2783500 / 33554432 = −f'_c · 695875 / 8388608.
   The moment sum Σ y r = 2620000 / 16777216, so
   M_concrete = f'_c · 81875 / 16777216.
7. Steel equilibrium at the same strains:
   N_steel = −E / 524288,
   M_steel = 9 E / 33554432.
   The second line is (3/16)(−E/4096) A_s + (−3/16)(−7 E/4096) A_s
   with A_s = 1/1024, which reduces to 18 E / 67108864 and then to
   9 E / 33554432.
8. Control resultants, the expected values of the test:
   N = −f'_c · 695875 / 8388608 − E / 524288,
   Mz = f'_c · 81875 / 16777216 + 9 E / 33554432.
9. Tangent of that same point, from (7) and the parabola derivative
   K = f'_c · (2 / 0.002) · (1 − x) = f'_c · 1000 · (1024 − 125 k) / 1024.
   The sum of (1024 − 125 k) is 2096, and Σ K A plus the two steel terms
   E A_s gives
   ∂N/∂ε0 = f'_c · 16375 / 256 + E / 512.
   The coupling sum has coefficient 2500 on f'_c · 1000 / 524288, which
   reduces to
   ∂N/∂κz = ∂Mz/∂ε0 = f'_c · 78125 / 16384.
   The bending sum has coefficient 10480 on f'_c · 1000 / 8388608, which
   reduces to f'_c · 81875 / 65536, plus the two steel terms
   2 · (3/16)² E A_s = 9 E / 131072:
   ∂Mz/∂κz = f'_c · 81875 / 65536 + 9 E / 131072.
10. Constant-axial update. With κz and the committed history held,
    r = N(ε0) − N★ and ε0 ← ε0 − r / (∂N/∂ε0), using the tangent entry from
    (7). The target N★ for the control search is the N of (8). The curvature
    stays 1/256. The first guess is ε0 = −5/4096, which keeps every control
    fiber on the parabola and both bars elastic: the extreme strains at that
    guess are −8/4096 and −2/4096. Evaluated from (4)–(7) before the section
    existed, the first residual is −3.798542×10⁵ N. Four updates meet a
    residual limit of 10⁻⁶ N and return ε0 = −1/1024 with the moment of (8).

An axial limit with κz = 0 and ε0 = 1/1024 puts every concrete fiber in
tension, so concrete contributes neither force nor tangent. Both bars stay
elastic (E/1024 = 195312500 Pa < fy). Then N = 2 E / 1048576 and Mz = 0.
At ε0 = 0 and κz = 0 every stress is zero, so both resultants are zero.

## Cases established before execution

| Case / source / citation | SI parameters | Input and history | Analytical expected values | Tolerance and justification |
| --- | --- | --- | --- | --- |
| Control curvature. Equations (1)–(8) of this sheet. Concrete ratio from (A.9) as transcribed in `docs/fichas/concrete.kent_park.md`; printed page: cita desconocida. Steel from `steel_bilinear.rs` while elastic. | b = 1/4 m, h = 1/2 m, n = 4, f'_c = 3000 psi, ρ_s = 0, b'' = 0.2 m, s_h = 0.1 m, E = 200000000000 Pa, fy = 500000000 Pa, b = 0, bars A_s = 1/1024 m² at y = ±3/16 m | e = [−1/1024, 1/256], history four zeros | N and Mz from equation (8). Tangent from equation (9). | Absolute floor 1×10⁻⁹ and relative 1×10⁻¹², combined as the maximum of the two. The floor sits above the last-bit gap between the single rational expression and a fiber-by-fiber f64 sum (about 1×10⁻¹¹ N·m on Mz). It is not taken from another program. |
| Constant axial at the same curvature. Equation (8) and the update in the assumptions. | Same section | Target N from equation (8), κz = 1/256, guess ε0 = −5/4096, residual limit 1×10⁻⁶ N, 8 updates, history four zeros | Returned ε0 = −1/1024 and returned Mz from equation (8). The returned N is within the residual limit of the target. | Same resultant tolerance. Strain absolute tolerance 1×10⁻⁹. |
| Tension limit, κz = 0. Concrete tension rule of the existing law; steel σ = E ε. | Same section | e = [1/1024, 0], history four zeros | N = 2 E / 1048576, Mz = 0 | Exact for these dyadic products. |
| Origin. | Same section | e = [0, 0], history four zeros | N = 0, Mz = 0 | Exact. |

Finite-difference sample: the control strain only, h = 1×10⁻⁶. That h is
inside the kit limits 1×10⁻⁹ and 1×10⁻². On the parabola,
dK/dε = f'_c · 500000. Summed over the concrete area 1/8 m², the second
derivative of N is f'_c · 62500. The central truncation |f''| h² / 6 is
about 0.2 N. Divided by ∂N/∂ε0, about 1.7×10⁹ N, the relative truncation
is about 1×10⁻¹⁰, below the kit limit 1×10⁻⁶. A curvature step h changes
the extreme fiber strain by h · 3/16, about 2×10⁻⁷. The nearest concrete
kink is 1/4096 from zero strain and about 2.9×10⁻⁴ from the peak. The
nearest steel yield is about 7.9×10⁻⁴ in strain. The sample ± h does not
cross a kink.

Separate checks, not extra benchmark stresses: D3, a strip count outside
1..=64, a bar count outside 1..=32, a nonpositive size, a bar outside the
rectangle, a nonfinite parameter, a bad history length, a nonfinite
deformation, a zero axial tangent (`ENG-E-0006`), and an update count that
stops before the residual limit (`ENG-E-0007`).

## Integration and evidence

- Exports, descriptor and factory: `RectangleFiberBending`,
  `RectangleFiberBendingParams`, `SteelBar`, `RectangleFiberBendingFactory`,
  `CurvaturePoint`, `RECTANGLE_FIBER_BENDING`, re-exported from `section`.
- Conformance appropriate to the trait: `check_section_law` on the control
  strain, with h = 1×10⁻⁶.
- Native benchmark and, if applicable, ENG-CORE-009 fixture/1: in-crate
  analytical rows only. No fixture/1 document, because this section is not
  an engine-input kind.
- Checks, tests passed/failed and logs under `$CARGO_TARGET_DIR`:
  `cargo test --offline --locked -p engine-laws --test rectangle_fiber`:
  9 passed, 0 failed.
  `cargo test --offline --locked -p engine-laws --doc rectangle_fiber`:
  1 passed.
  `cargo test --offline --locked --workspace`: exit 0.
  `cargo fmt --all -- --check`: exit 0.
  `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`:
  exit 0.
  `cargo doc --offline --locked --no-deps --workspace` with warnings denied:
  exit 0.
  `cargo check --offline --locked --workspace --target wasm32-unknown-unknown`:
  exit 0. That check compiles; it does not execute the section on wasm.
- Comparison of the 16 hashes across targets without modifying the reference:
  `docs/DETERMINISM.md` is not modified. The hashes are not regenerated to
  accept a difference.
- Log, clean-room review and validation limits: `provenance/ENG-SEC-002.md`.
  Clean-room reviewer: PENDING. This change does not close MAM-107 and does
  not approve a time or memory budget. The report BARC/2009/E/026 was not
  opened in this session. No program listing was transcribed.
