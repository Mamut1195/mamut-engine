# Clean-room log · ENG-GEO-004

- Ticket / PR: ENG-GEO-004 · MAM-45; PR #172.
- Author(s): Codex gpt-6.1-sol high, 2026-10-20 (America/Guayaquil).
- Role: implementer; specifier: J2 (specification supplied in the ticket).
- Implemented contract: generators `tz.api_clay`, `tz.api_sand`, `qz.api`,
  `tz.user`, `qz.user`; immutable tables of local axial resistance,
  consumable by the future ENG-GEO-005 pile and, when there is no softening,
  by the ENG-MAT-002 elastic multilinear law through explicit adapters.
- Permitted sources used: American Petroleum Institute, *Recommended Practice
  2GEO — Geotechnical and Foundation Design Considerations*, primera edición,
  abril de 2011, con Addendum 1, octubre de 2014, reafirmada 2021 (ISO
  19901-4:2003 modificada), according to J2's specification; not verified by this session.
  §8.4.2, figura 2 y tabla 2 (fuste); §8.4.3, figura 3 (punta).
  The vertices and scope were supplied by J2 in the ticket;
  this session did not open the PDF or consult external sources.
- Own derivations: interpolation, negative branches and unit adaptation
  described below. Own code; no source text or figures were copied,
  only the numerical vertices that define the curves.
- Third-party code copied: none.
- Validation: cited published vertices and own analytical oracles for
  interpolation, tension and artificial tables; same native/wasm harness.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." I also did not consult code or results from CalculiX or LAPACK,
  or results from other engines, or code from prohibited sources.
- Clean-room reviewer: J4 r1 on 99fc5f9bf5820e7dd3d7e1c09f38a8cc9b5e74f4
  (SUITABLE with follow-up, 2026-10-20); r2 on
  47ef2f93aee5bfac3d7152d0084647face2d174f; r3 on
  90c6534a36bdaa7e1c9a69216c54b938b821e289 (SUITABLE).

## Source authorization and limitations

Jonathan confirmed on 2026-10-20 that he has a legal copy of API RP 2GEO and
granted the ENG-GOV-004 waiver **limited to API RP 2GEO in ENG-GEO-004**:
implement these curves with own code and citations by section/figure/table,
without copying text or figures. The waiver is recorded in `docs/CLEAN_ROOM.md` §4
for this PR, edited by J2; J4 r1 follow-up H1 adds only
the authorized clarification about numerical vertices as code constants.
The PDF is not included in the repository. Direct approval of the merge
rests with Jonathan; this log does not replace it.

The Reese/Isenhower/Wang (2006) book was not used, nor its data, curves,
tables or values. Coyle–Reese (1966) may only be represented through points
supplied by the user to `tz.user`: there are no built-in data from that method.
`tz.coyle_reese_clay` is reserved in documentation as «not implemented —
pending the original source», Coyle & Reese (1966), ASCE JSMFD.
There is no function that fabricates data from that method or invented constants.
ENG-GEO-003 / PR #168 was not consulted and is not used as a dependency.

## Parameters supplied by the user

All scalars are received as `Option<Real>` to diagnose absences
with ENG-E-0003. There are no defaults or data inferences.

- Shaft: `tmax_compression_pa`, `tmax_tension_pa` and `zpeak_m`, all positive
  and finite; tension and compression may have equal scale by the user's explicit
  decision. The source refers f(z) and ft(z) to §8.1; their calculation,
  including equations 17–21, remains **pending and outside scope**.
- `zpeak_m` is supplied directly: §8.4.2 recommends zpeak/D = 0.01 for
  routine design and indicates 0.0025–0.02; no value is adopted and that range
  is not imposed. D is not needed to generate t-z if zpeak is already known.
- Clay: `tres/tmax` mandatory, 0 < ratio ≤ 1. The source's 0.70–0.90
  interval is documented without imposing it. The residual is set at 2·zpeak from
  table 2; other zres allowed by the standard are outside this delivery.
- Tip: `qp_n` (capacity Qp in N, calculation in §8.1 outside scope) and
  `diameter_m` (m), positive and finite.
- `plateau_z_m` is mandatory: greater than zpeak in sand, 2·zpeak in clay,
  0.10·D at the tip. It closes the table; no default mesh is created.
- User tables: mandatory, finite points, origin included and strictly
  increasing z. `UserTraction::Explicit` requires both branches
  and resistance ≤ 0 if z < 0, ≥ 0 if z > 0;
  `Symmetric` receives only the origin and positive branch and reflects by explicit
  choice. `qz.user` allows a zero negative branch supplied by the user;
  it does not impose the unilateral contact of `qz.api` on a free-form table.

## Own derivations and integration decisions

1. Between two vertices (zi, ri), (zj, rj),
   r(z) = ri + (rj − ri)·(z − zi)/(zj − zi) is used. It is an own interpretation
   of linearly joining the numerical vertices, not reproduced text or figures.
   Evaluation chooses the nearest endpoint to reduce cancellation;
   at a vertex its ordinate is returned exactly. Outside the table the end
   segment is extended, as in ENG-MAT-002. API tables end
   with plateaus; user tables determine their own extensions.
2. Negative shaft: shape reflection about the origin, with independent
   `tmax_tension_pa` scale; the same zpeak and residual ratio are
   preserved. It is an own derivation; the reflection is not attributed to API.
3. API tip: (−zmeseta, 0) is added before the origin; extending the
   first segment produces Q = 0 for any z < 0. Own idealization
   of a tip without tension. The positive plateau starts at 0.10·D.
4. Units are z and D in m, t in Pa, Q in N, without prefixes. z > 0 indicates
   that the pile moves downward relative to the soil. Ordinates represent mobilized
   resistance; soil force on the pile has the opposite sign.
5. Adaptation to ENG-MAT-002: ε = −z/L, σ = −t for unit shaft, or
   σ = −Q/A for tip. L (m) and A (m²) are mandatory references from the
   caller, without inferring geometry; point order is reversed and it is
   consumed with `MultilinearMode::Both`. The API tip retains its gap with ε > 0;
   `CompressionOnly` may be used to choose the compression-side tangent at
   the origin. With `Both`, the law chooses the right tangent (zero in
   that gap) at the origin; resistances agree in both modes.
   Bilateral user tables require `Both`.
   The future pile must integrate t over the surface to obtain N.
6. The current multilinear law requires nonnegative slopes. The generator
   preserves clay/user table softening; the adapter
   rejects it with ENG-E-0010 and a message identifying ENG-MAT-002. A curve is neither clipped
   nor artificially hardened. Residual ratio 1 is compatible.
7. No dependencies, bars, trusses or structural axial law are added.
   The only extension to the determinism guard allows `no_mangle` and a
   public `extern "C"` function in the exact path of the new wasm example;
   it does not allow unsafe or other constructs. Hashes are not modified.

## Fixtures and checks

`crates/engine-geo/tests/support/axial_benchmarks.rs` contains five fixtures
shared with native engine-validation and the `axial_wasm` example:

- Sand: all vertices; zpeak = 0.01 m, tmax = 50000 Pa;
  t(0.0016) = 15000 Pa, t(0.01) = 50000 Pa.
- Clay: ratio 0.8; t(0.015) = (50000 + 40000)/2 = 45000 Pa;
  t(0.02) = t(0.03) = 40000 Pa; tension scaled by 30000/50000.
- Tip: Qp = 1000000 N, D = 1 m; Q(0.013) = 500000 N;
  Q(0.1) = 1000000 N; Q(0.0075) = (250000 + 500000)/2 = 375000 N;
  Q(z < 0) = 0.
- Artificial t-z table: independent branches and softening.
- Artificial q-z table: explicitly requested reflection.

Vertices use exact ordinate comparison; interpolation allows
2e-14 relative error with minimum scale 1 SI, only for binary64 rounding.
Additional contract tests cover every mandatory absence,
non-finites, domains, order, origin, symmetry, scale, plateaus, rejection of
softening in the adapter, overflow and underflow.

Final results (offline, Rust 1.99.0, native x86_64 target and
wasm32-unknown-unknown, Node v22.19.0):

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: **685 passed, 0 failed, 0 ignored**
  (includes 12 new tests in engine-geo and 1 in engine-validation).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `cargo check --workspace --offline --target wasm32-unknown-unknown`: OK.
- Release build of the `axial_wasm` example and Node execution: **5 fixtures
  passed**, without external imports.
- `bash clean-room/check.sh origin/main`: OK. A temporary `GIT_INDEX_FILE`
  inside CARGO_TARGET_DIR was used so that the script also inspected
  new files still untracked; the real index was not modified.
- The **16 hashes in docs/DETERMINISMO.md are identical**, checked with
  ten native runs per case of the `determinism_native` example and exact
  comparison with the published table; that document was not modified.
- `git diff --check`: OK. No commits or changes to Cargo.toml/Cargo.lock,
  workflows, CODEOWNERS, vendor or clean-room/** by the implementer.

Pending items outside the implemented scope:
§8.1 capacity calculations; original source to add Coyle–Reese;
and future ENG-GEO-005 pile. The current ENG-MAT-002 law cannot consume
softening: the adapter's typed rejection is the intended behavior.

J4 r1 follow-up for MAM-45: sign validation of `Explicit` in `tz.user`
and `qz.user`, rejecting opposite signs and accepting zeros and correct
signs; analytical regression for resistance underflow when adapting
units (N4).

Checks for this follow-up (offline, Rust 1.99.0):
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --offline -- -D warnings`,
`RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`,
`cargo check --workspace --offline --target wasm32-unknown-unknown`,
`bash clean-room/check.sh origin/main` and `git diff --check`: OK.
`cargo test --workspace --offline`: **688 passed, 0 failed, 0 ignored**,
including the three added tests. N4 mutation detected: when removing
the «Resistencia reducida a cero» check, the test
`adapter_rejects_resistance_reduced_to_zero_by_area` fails
(**0 passed, 1 failed**); it passes before and after restoring `tz.rs`
exactly, byte for byte (**1 passed, 0 failed** in each run).
The **16 hashes in docs/DETERMINISMO.md remain identical** to the published
table, with ten runs per case in native and wasm, both in debug and
release; native outputs of both profiles are also identical.
Release build of `axial_wasm` and Node v22.19.0 execution: **5 fixtures
passed**, without external imports. Local evidence in
`$CARGO_TARGET_DIR/j4-r1-evidence/`. No pending technical checks
for this follow-up; its closure rests with J4.
