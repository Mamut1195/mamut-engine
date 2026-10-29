# Clean-room log · ENG-VAL-040

- Ticket: MAM-852 · VAL-040, linear static invariance.
- Author: Codex (model gpt-6.1-sol) · 2026-10-29 (America/Guayaquil).
- Role: author of the new integration test code and this documentation.
- Permitted sources used: the ticket request and repository public API/test
  code, principally `crates/engine-validation/tests/analytical.rs`,
  `tests/support/harness.rs`, and the frame element contracts.
  Log structure follows `provenance/ENG-VAL-005.md`.
- Expected values: fresh results of the same Engine reference model, transformed
  geometrically or mapped by identifiers, and elementary closed forms derived
  below. No third-party source, code, numerical value, table, or result was used.
  No network access or downloads were used.
- Declaration: no OpenSees, Frame3DD, Stabileo, books, or other third-party
  sources or values were consulted or incorporated for this ticket.
- Dependencies: existing dependencies only; no Cargo manifest or lock changes.
- Clean-room reviewer: PENDING (J4).

## Tested scope

`crates/engine-validation/tests/invariance.rs` uses the same public
`DofMap`, `ElementContribution`, `LinearSystem`, `ReferenceBackend`,
`StaticLinear`, and `CaseResults::recover` path as the analytical harness.
There are fourteen integration tests, seven each for `EulerFrame2D` and
`EulerFrame3D` (twelve in the first revision, plus the two raw Maxwell-Betti
tests added after review r1). All loads are global
nodal forces or couples. Each solve through result recovery
checks physical force and moment equilibrium, including the
cross product of position and force about the global origin.

The transformation reference is an asymmetric, three-member branched
cantilever with four nodes and a fully clamped root. The spatial geometry is
nonplanar, has unequal bending inertias, and carries all six generalized load
components. The tested invariants are:

- Translation by a nontrivial vector: global displacements, reactions, and
  local end forces remain identical. Dyadic coordinates and offsets preserve
  member differences exactly; these comparisons additionally require exact
  floating-point equality. The planar translation stays in the XY plane.
- Proper rotations through thirty and ninety degrees: geometry and global
  loads rotate together; displacements, rotations, reaction forces, and
  reaction couples follow the same rotation matrix. Spatial rotation uses
  Rodrigues' formula about an axis proportional to `(1,2,3)` and rotates the
  section orientation reference too. Local end forces remain identical.
- Permutation: sparse nonmonotonic node and member identifiers, changed node
  insertion order, and reversed member assembly order preserve mapped results.
  Member endpoint direction is retained.
- Maxwell-Betti reciprocity: two unit generalized loads on distinct free
  nodes give equal cross-compliances. Both dimensions include force-force and
  force-moment pairs. Spatial tests also include an out-of-plane force pair,
  an out-of-plane force-moment pair, and a moment-moment pair. Every pair
  requires nonzero cross-response to avoid vacuous acceptance.
- Symmetry: a two-column portal with a split top beam and symmetric nodal
  loading has mirrored global displacements and reactions, including the
  center node on the mirror plane. Under reflection of X, polar vectors
  transform as `(-x,y,z)` and axial vectors as `(x,-y,-z)`. Vertical support
  reactions are equal and match half the applied total. The spatial portal
  also carries symmetric out-of-plane forces.
- Analytical anchor: a two-member straight cantilever agrees with the
  elementary tip bending displacement and rotation. Spatial checks cover
  both unequal bending inertias and uniform Saint-Venant torsion.

Only full clamps are used: the public support model lists global restrained
DOFs, without an arbitrary rotated support basis. A full clamp's restrained
subspace is itself invariant under any rotation.

## Own closed forms and tolerances

For a cantilever of length L with downward tip force P, integrating
`EI v''(x) = -P(L-x)` with `v(0)=v'(0)=0` gives
`v(L)=-PL^3/(3EI)` and `theta_z(L)=-PL^2/(2EI)`.
For a positive spatial tip force along Z, the corresponding signs are
`w(L)=PL^3/(3EIy)` and `theta_y(L)=-PL^2/(2EIy)`.
Integrating `GJ theta_x'=M` gives `theta_x(L)=ML/(GJ)`.
Support resultants in the symmetric portal follow directly from balance
and reflection. These derivations use no tabulated third-party values.

The comparison helper reports maximum absolute error, tolerance, reference
scale, worst component index, and actual/expected values. Relative tolerance
is **1e-10** times the largest absolute value of the reference series, with
no additive absolute floor. Global vector comparisons separate translational
and rotational blocks (or forces and couples). Symmetry series span all three
mirror pairs so numerical zeros at the center use the full response scale.
Reciprocity compares each scalar cross-compliance at this same relative
tolerance. A reference series containing only zeros requires exact zeros.

For equilibrium, let Fscale and Mscale be the largest componentwise sums of
absolute physical force and moment contributions, and Lchar the longest
member length. Force tolerance is **1e-10** times
`max(Fscale, Mscale/Lchar)`; moment tolerance is **1e-10** times
`max(Mscale, Fscale*Lchar)`. This dimensional normalization gives a pure couple
a nonzero force error scale and a pure force a nonzero moment error scale,
without an absolute floor. The residual includes all nodal loads and
recovered reactions, including direct couples and position cross force.
It does not use the solver's residual as the physical equilibrium oracle.

## Not tested · DESCONOCIDO

- DESCONOCIDO: partially restrained or inclined supports under general rotation;
  prescribed nonzero displacements; released member ends; member reversal.
- DESCONOCIDO: distributed member loads, truss elements, springs, foundations,
  geometric/material nonlinearity, dynamics, and load combinations.
- DESCONOCIDO: arbitrary geometries, conditioning extremes, random permutations,
  rotations other than the two angles above, and other section orientations.
- DESCONOCIDO: other numerical backends, release builds, WASM, CLI, Python,
  serialized API transport, station diagrams, and internal extrema.
- DESCONOCIDO: comparison with external solvers; none was performed or used.

## Verification

The new test was compiled directly with the installed `rustc --test`, reading
the already compiled debug libraries in `$CARGO_TARGET_DIR/debug/deps` and
writing only to a temporary directory. The binary ran all twelve tests:
**12 passed, 0 failed, 0 ignored**. This checks execution against the cached
libraries; it does not replace rebuilding the checkout through Cargo.

The required `cargo test -p engine-validation --test invariance --offline`
invocation, through a local serialization wrapper that is not part of this
repository, with the required shared `CARGO_TARGET_DIR` and
`CARGO_NET_OFFLINE=true`, could not complete: it waited for the shared
serialization lock and a bounded retry timed out after 180 seconds (exit 124),
without Cargo output. The lock was neither bypassed nor modified. The direct
compiler check only read the cached libraries and wrote outside the shared
target directory.

The initial execution identified a test-helper normalization problem for
pure moment loads: the nominal force scale was itself only numerical
cancellation noise. The dimensionally consistent equilibrium normalization
documented above resolves that problem with the same relative tolerance.
No Engine invariant failure was observed, and no test is ignored.

| Check | Result |
| --- | --- |
| Direct `rustc --test`, existing debug libraries | 12 passed |
| `rustfmt --check --edition 2021 crates/engine-validation/tests/invariance.rs` | OK |
| `git diff --check`, including the three new files via a temporary index | OK |
| `bash clean-room/check.sh origin/main clean-room`, including new files | OK |
| `bash scripts/changelog-check.sh origin/main` | OK; committed-diff scope only |
| Serialized offline Cargo test inside the Codex sandbox | Unavailable: shared lock wait, 180-second timeout, exit 124 |
| `cargo test --offline -p engine-validation --test invariance` (local serialized wrapper, not in the repository), run by J2 outside the sandbox | 12 passed, 0 failed, 0 ignored |

The real Git index and object database are read-only. A temporary index and
temporary object directory, using the existing objects as read-only
alternates, exposed exactly the three new ticket files with intent-to-add
for the clean-room and whitespace checks. The real index was not changed.
The changelog script compares committed changes only and reports that no
fragment is required at this uncommitted HEAD; the new fragment was separately
checked for nonempty content, its requested filename, and the prohibited word.

Codex created no commit, push, or pull request. J2 then rebuilt the checkout
through the serialized Cargo command outside the sandbox (12 passed) and ran
mutation checks, restoring every file afterwards:

| Mutation | Result |
| --- | --- |
| 2D frame stiffness `6EI/L²` scaled by 1.01 | 5 tests fail (2D translation, rotation, permutation, reciprocity, cantilever) |
| 2D frame stiffness made asymmetric (one `6EI/L²` entry scaled by 1.001) | 6 tests (all 2D) fail; result recovery returns an error before reciprocity is compared |
| Node map in the permutation comparison swapped | 2 tests fail |

## Revision after review r1 (J4)

The local serialization wrapper mentioned above only takes a shared lock and
sets a shared `CARGO_TARGET_DIR`; it is not in the repository. The equivalent
repository commands are the plain `cargo` invocations listed below.

- H0: the test called `f64` methods (`sqrt`, `sin_cos`, `powi`) directly,
  which `engine_crates_use_only_the_central_scalar_module` in
  `crates/engine-linalg/tests/determinism_guard.rs` rejects. They now go
  through `engine_core::math::{sqrt, sin_cos, powi}`, as in the other
  engine-validation tests. `engine_core::math::powi` uses `libm::pow`, so
  expected values may differ in the last bits from `f64::powi`; all
  tolerances are unchanged and every test passes.
- H1: the recovery-based Maxwell-Betti tests cannot see an asymmetric element
  stiffness, because `CaseResults::recover` already stops with ENG-E-0010.
  Two tests (`maxwell_betti_raw_2d`, `maxwell_betti_raw_3d`) now compare the
  same pairs from `StaticCaseResult::global_displacements`, without recovery
  or the equilibrium check. Measured result: they also pass with an
  asymmetric element stiffness (mutations M2 and M5 below). The mutated
  matrix is symmetrized before assembly, in the element projection
  `Hᵀ M H`: only the lower triangle is computed and mirrored
  (`crates/engine-elements-frame/src/release.rs:81-84`,
  `crates/engine-elements-frame3d/src/release.rs:144-147`). Assembly then
  requires exact symmetry (`crates/engine-assembly/src/assemble.rs:75`,
  `EXACT_SYMMETRY`), and `SymMatrix::new` validates it
  (`crates/engine-linalg/src/matrix.rs:63-71`). The solved system is
  therefore symmetric, so the raw solution is reciprocal by construction.
  Neither Maxwell-Betti form is an independent detector of element
  asymmetry; that defect is detected only by result recovery (ENG-E-0010).
  Their value is to check the load and DOF mapping of the cross-compliances.
- H2: mutations now include the 3D element (M4, M5 below).
- H4: equilibrium uses the reactions produced by `CaseResults::recover`
  (from K·u). It checks balance of loads against recovered reactions about
  the origin, but it is not a fully independent oracle of the stiffness.
- H5: unit scaling is neither declared nor tested. All inputs are in the
  consistent SI set N, m, Pa, rad; invariance under a change of units is
  DESCONOCIDO.

Mutation checks on this revision, each reverted afterwards and confirmed
byte-identical with `cmp`, running
`cargo test --offline -p engine-validation --test invariance` (14 tests):

| Mutation | Failing tests |
| --- | --- |
| M1: 2D `6EI/L²` × 1.01 | 5: translation, rotation, permutation, maxwell_betti, cantilever_analytical (2D) |
| M2: 2D one `6EI/L²` entry × 1.001 (asymmetric) | 6: all 2D except maxwell_betti_raw_2d; recovery stops with ENG-E-0010 |
| M3: node map in the permutation comparison swapped | 2: permutation_2d, permutation_3d |
| M4: 3D `6EI/L²` × 1.01 | 6: translation, rotation, permutation, maxwell_betti, symmetry, cantilever_analytical (3D) |
| M5: 3D one `6EI/L²` entry × 1.001 (asymmetric) | 6: all 3D except maxwell_betti_raw_3d; recovery stops with ENG-E-0010 |

The workspace and `determinism_guard` results first recorded for revision r1
came from the shared Cargo target directory and were not reliable: review r1
found that the guard fails on that revision, because the test imported
`engine_core::{math, ...}` instead of `use engine_core::math;` on its own line.
That import is now split. All results below were measured on this revision
with a new, empty, dedicated `CARGO_TARGET_DIR` (71 crates compiled from
scratch):

| Check | Result |
| --- | --- |
| `cargo test --offline --workspace` | exit 0; 872 passed, 0 failed, 0 ignored |
| `cargo test --offline -p engine-linalg --test determinism_guard` | 37 passed, 0 failed |
| Same guard with the previous combined import restored temporarily | 36 passed, 1 failed |
| `cargo test --offline -p engine-validation --test invariance` | 14 passed |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | OK |
