# Clean-room log · ENG-LOAD-002-FU

- Ticket / PR: issue #70, J4 r15 follow-up to #54, ENG-LOAD-002;
  branch `j2/fu-70-settlements` from `main`. Correction before CLI/API.
- Author(s): Codex gpt-6.1-sol high · 2026-10-14 (America/Guayaquil).
- Role: implementer and author of own analytical derivations.
- Implemented contract: `engine_analysis::combination::combine` and metadata
  `NodeResult::prescribed_displacements`, recovered by `CaseResults::recover`.
  No new element contract, dependencies, CLI/API or serialization.
- Permitted sources used: issue #70 request in this session, the repository's own catalog and
  APIs (`engine-analysis`, `engine-assembly`,
  `engine-elements-frame`, `engine-core`), `provenance/TEMPLATE.md` and original
  log `provenance/ENG-LOAD-002.md`. No network access or external sources.
- Own derivations: analytical note included below and
  [tests](../crates/engine-analysis/tests/combination.rs).
- Third-party code copied: none.
- Validation: analytical; beam with zero rotations and relative settlement, equilibrium
  and superposition with an independent case. Oracle derived by hand.
- Oracle comparison (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Decision and scope

Conservative rejection is chosen. The response with prescribed displacements
is affine, `r(F,p) = r(0,p) + L(F)`. Therefore, adding the complete results of
two cases with the same settlement weights the load-independent contribution
by `1.2 + 1.6 = 2.8`. The current contract preserves complete results; it does not
separate `r(0,p)` in displacements, reactions, actions or stations. Separating it
automatically would require another analysis or extending that contract to store
all those fields separately. Rejection avoids that extension and does not attempt
to infer supports or settlements from a reaction or free displacement.

The only data extension is `[Option<f64>; 6]` per node to preserve which
DOFs were prescribed and their value, including zeros; `None` means free,
dependent or absent. `recover` obtains that data from the system
constraints, without applying the load factor. Manual constructors of
`NodeResult` must supply it; this changes its initialization in Rust, without
introducing a serialized schema. The prescribed supports/DOFs must match
between terms; the caller is responsible for maintaining the same stiffness and
constraint transformation, which the results do not contain.

`combine` returns ENG-E-0010 with both `Case` entities when more than one
term contains any nonzero prescribed value, whether equal, different or opposite,
even with a zero factor. It also rejects different sets of prescribed DOFs
or metadata that differs from the nodal displacement, indicating the affected case.
Nonfinite metadata produces ENG-E-0008 with a `Case` entity, like other
nonfinite data. ENG-E-0010 is used following the compatibility validation
already present in this module and in `CaseResults::recover`, without extending the catalog.
Equality of values between a load case with a zero prescribed value and
the sole settlement case is not required. Cases outside the terms do not trigger this
rejection; envelopes may compare several cases with settlement.

Explicit treatment: solve D and L with all prescribed values zero, solve
S without loads and with the settlement, preserving the same prescribed DOFs and
stiffness, and combine `1.2D + 1.6L + αS`. `α=1` counts the settlement once;
other factors are the user's explicit decision. A single term with loads
and settlement still weights its entire response by its factor. The combined
result preserves the weighted prescribed values to validate a subsequent
combination, including `α=0`. The ordered f64 sum and matching
by id are unchanged. No `panic!`, `unwrap` or `expect` are added to the library.

This decision supersedes, for `combine`, the behavior of summing all
settlements described in the original ENG-LOAD-002 log.

## Own analytical derivation

Horizontal Euler–Bernoulli beam, length L, constant stiffness EI, no transverse
load, with `v(0)=0`, `v(L)=δ`, `v'(0)=v'(L)=0`. Integrating `v''''=0` and
applying those four conditions gives, with `t=x/L`:

`v(x) = δ(3t² − 2t³)`.

With the in-house local convention `M=EI v''`, `V=−M'`:

`M(x) = 6EIδ/L² (1−2t)`, `V(x) = 12EIδ/L³`.

The action moments at both ends are `−6EIδ/L²`; the vertical forces
are `−12EIδ/L³` at i and `+12EIδ/L³` at j. Equilibrium: sum of forces zero and
`Vj L + Mi + Mj = 0`. `Ux` is left free at j and restrained at i to retain
one free DOF; its independent response under force P is `u_x(L)=PL/(EA)`,
`N=P` and axial reaction `−P` at i.

Fixture: L=2, E=16, A=2, I=2, δ=0.125 (SI). The initial moment is 6 N·m and
the shear 6 N. D has P=48 N and L has P=−16 N. With settlement in both,
the previous sum would produce 16.8 N·m and δ=0.35 m; it is now rejected. With homogeneous
loads and a separate S, `α=1` preserves 6 N·m and δ=0.125 m while the combined
axial force is `1.2·48 − 1.6·16 = 32 N`.

Values of `α` in [1, 0.5, −1, 0], nodes, reactions, actions and all
stations are checked against these formulas and a direct analysis with combined loads and
settlement. Tolerance: `128·EPSILON·max(1,|esperado|)`, consistent with
the fixture's scale. Rejection covers equal, opposite and double
settlements, factors [1.2,1.6], [1,0], [0,0], [1,−1], inconsistent metadata,
nonfinite values and different DOF sets.

The ordered sum is also checked with [1e16, −1e16, 1]: result 1;
with [1e16, 1, −1e16]: result 0, in all fields and ends, with
reversed repository order. The test by id is strengthened with different values between
nodes and members and between cases, reversed order and factors 2 and −0.5.

## Verification

All code checks were run without network access (`CARGO_NET_OFFLINE=true`) and
with `CARGO_TARGET_DIR=/workspace/.cargo-target`, after restoring the correction:

- `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, 388 tests across 65 suites, no failures or
  ignored tests; the 15 combination tests include four new ones and one
  strengthened for matching by id.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, using
  `/home/box/.cargo/bin/cargo` and `RUSTC=/home/box/.cargo/bin/rustc` (1.99.0),
  with wasm std already installed; no components downloaded. Native checks
  use rustc/cargo 1.85.1. wasm compilation is verified, not execution.
- `bash clean-room/check.sh origin/main`: OK, including the new log
  through temporary index and objects in `/tmp`; `git diff --cached --check`: OK.

To check regression, **all** of
`crates/engine-analysis/src/combination.rs` was temporarily replaced with its HEAD version, preserving
the nodal metadata field and its recovery so that the tests compile.
`cargo test -p engine-analysis --test combination` exited with code 101:
12 tests passed and these three failed by assertion (no compilation failure):

- `repeated_settlement_rejected_against_fixed_end_beam_oracle`: returned success
  with M=16.8 N·m and δ=0.35 m, instead of rejecting the duplicate settlement.
- `dedicated_settlement_case_counted_once_with_explicit_factor`: the result
  preserved `Some(0)` from the first case instead of the combined prescribed value `Some(0.125)`.
- `incoherent_prescribed_metadata_rejected_with_case_entity`: accepted inconsistent
  metadata, instead of the expected error.

The two additional tests check behaviors already correct in HEAD;
therefore they were validated with independent deliberate regressions: sorting the
terms by name made cancellation fail (1 instead of 0); accumulating by
original input position made the test by id with different values fail.
Both runs exited with code 101 by assertion. All temporary
variants were restored byte for byte before the full checks.

The worktree's real index is mounted read-only at
`/workspace/mamut-engine/.git/worktrees/me45`. `git add` failed when creating
`index.lock` (Read-only file system); `git commit` also failed for that reason.
This prevents completing the requested commit
in this environment. No push was made. Clean-room review remains PENDING (J4).
`bash clean-room/check.sh origin/main` is checked with a temporary copy of the
index and objects in `/tmp`, including the four files; the real index
remains intact. A complete patch is prepared at `/tmp/ENG-LOAD-002-FU.patch`.

J2 note (2026-10-14): J2 created the commit with the same verified content; the earlier block was only due to the sandbox.
