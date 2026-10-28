# Clean-room log · ENG-MASS-001-FU

- Ticket / PR: issue #71, J4 r16 follow-up to #55 / ENG-MASS-001; branch `j2/fu-71-suction` from `main`. Correction before CLI/API.
- Author(s): Codex gpt-6.1-sol high, 2026-10-14 (America/Guayaquil).
- Role: specifier and implementer through own derivations.
- Implemented contract: `engine-loads::mass`, force→mass conversion in `mass_from_force`, `masses_from_load_vector`, `masses_from_case` and `masses_from_system_case`. No changes to element contracts or force assembly.
- Permitted sources used: exclusively the user's ticket and the relevant internal contracts, catalog, documentation and code of mamut-engine. No external sources or network queries.
- Own derivations: equations (1)-(5) of this log.
- Third-party code copied: none. No new dependencies.
- Validation: analytical; tests in `crates/engine-loads/tests/mass.rs`, including temporary reversal of the correction.
- Oracle comparison (sibling repo, informative): no. Only own analytical oracles derived by hand.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Corrected contract

This note supersedes the absolute-value contract historically described in
section (4) of `provenance/ENG-MASS-001.md`. The user's direction defines
the direction of gravity. The algebraic sum of all contributions
per node is preserved before projection; projections between nodes do not offset each other.
For net nodal force F_n and normalized direction d̂:

(1) q_n = F_n·d̂; if q_n ≥ 0, m_n = q_n·factor/g_usuario.

If q_n < 0, it is rejected with ENG-E-0010, without tolerance, even with zero factor.
The check precedes multiplication by factor to avoid losing the sign of
subnormal projections. The projection's finiteness is validated before its sign;
nonfinite values/overflows retain ENG-E-0008. Exact zero, including -0.0,
is valid and generates zero mass. A positive projection with zero factor gives zero.
Validation of g and factor and rejection of underflow
of positive mass with positive factor are retained. Moments do not generate mass.
Nodal conversion errors include `Node`; case adapters
preserve `Case`. Scalar conversion receives no entities.

## Own analytical oracles

(2) For d = -Z and g = 10 m/s², Fz = -120 N represents W = 120 N:
q = 120 N and m = W/g = 12 kg (3 kg with factor 0.25).
Fz = +120 N produces q = -120 N and must be rejected. The smallest
representable negative subnormal is also rejected; no tolerance is applied and its magnitude is not taken.
F = 0 and both representations of zero produce zero mass.
At different nodes, Fz = -120 and +80 N have global resultant -40 N,
but the second node has q = -80 N and is rejected.

(3) For d̂ = (3/5, 4/5, 0), F = (3, -4, 0) N gives
q = (9-16)/5 = -1.4 N: error. F = (4, -2, 0) N gives
q = (12-8)/5 = 0.8 N, m = 0.08 kg with factor 1 and g = 10.
An opposite component is not enough for rejection: the net projection decides.
The perpendicular fixture uses F = (4,4,0) and d = (-1,1,0), with symmetric
products that cancel exactly in f64. The previous fixture (3,4,0)
against (-4,3,0) had a negative rounding residual after normalization; under
the strict contract that residual is rejected. No tolerance is introduced.
In a system, node 2 has nodal Fz = -80 N; element contributions of
+40, +80 and +100 N give net values -40, 0 and +20 N: respectively 4 kg, 0 kg and error.
Node 1 retains -120 N, so even in the last case the global
resultant remains gravitational. This checks subtraction before projection.

(4) M13: an X mass block between two nodes [[1,-2],[-2,1]] kg has positive
diagonals, but r_X = [1,1] gives r_Xᵀ M r_X = 1-2-2+1 = -2 kg.
`total_mass` must reject with ENG-E-0010 for the negative total.

(5) M24: an XY model declaring only Ux and Uy does not contain Uz. Direction
Z requires Uz and produces ENG-E-0011 in the directional measure and all three
nodal adapters, even with zero forces.

The historical valid-conversion fixture changes the second weight from +80 to
-80 N and its element contribution from -40 to +40 N: it preserves the expected masses of
3 and 2 kg initially and 2.5 and 1 kg after assembly. Suction is covered through
explicit rejection tests in all four functions.
M13 and M24 only add coverage of existing behaviors.

## Session verification

All Cargo commands use `CARGO_NET_OFFLINE=true` and
`CARGO_TARGET_DIR=/workspace/.cargo-target`. No changes to `docs/CLEAN_ROOM.md`,
`.github/**` or `clean-room/**`. No push; local commit in Spanish.

- Temporary reversal: only `crates/engine-loads/src/mass.rs` was restored
  from `HEAD`, keeping the new tests, and
  `cargo test -p engine-loads --test mass suction_` was run. The four regressions
  failed (0 passed, 4 failed, Cargo exit 101) because the previous implementation
  returned `Ok` instead of the error. The correction was restored in a
  `finally` block; with it all 15 mass tests pass, also within
  the complete workspace run. M13/M24 provide additional coverage of
  existing contracts rather than regressions of the sign change.
- `cargo fmt --check`: OK, exit 0.
- `cargo clippy --all-targets -- -D warnings`: OK, exit 0.
  The first attempt mixed Cargo/Rust 1.85.1 with Clippy 1.99.0 and failed with
  E0463 due to incompatible artifacts. It was resolved using the already installed
  toolchain 1.99.0, with `PATH=/home/box/.cargo/bin:$PATH`, without downloads.
- `cargo test --workspace`: OK, exit 0; 396 tests and doctests passed,
  0 failures, 0 ignored, with Rust 1.85.1. Includes six new tests: four
  suction regressions and the two M13/M24 checks.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK, exit 0,
  with Rust 1.85.1.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, exit 0,
  using Rust 1.99.0 and the already installed target through the same PATH adjustment.
  Verifies wasm compilation, not execution on that target.
- `bash clean-room/check.sh origin/main`: OK, exit 0; includes the new
  log registered in the index. `git diff --check`: OK.

Local records: `/tmp/eng-mass-001-fu-{revert,fmt,clippy,test,doc,wasm,clean-room}.log`.
No code from external engines was consulted or copied. No `panic`, `unwrap` or
`expect` in the modified library code. Clean-room review pending J4.
