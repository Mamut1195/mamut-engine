# Clean-room log · ENG-RES-001-FU

- Ticket / PR: issue #69, J4 r14 follow-ups to #53, ENG-RES-001.
- Author(s): Codex gpt-6.1-sol high · 2026-10-14 (America/Guayaquil).
- Role: implementer and author of own analytical derivations.
- Implemented contract: `element/1` recovery in `CaseResults::recover`;
  load and member consistency before CLI/API and `results/1`.
- Permitted sources used: ticket request and own APIs/code from
  engine-analysis, engine-assembly, engine-contracts and repository elements;
  `provenance/TEMPLATE.md` and `provenance/ENG-RES-001.md`. No network or external sources.
- Own derivations: formulas and equilibrium rule in this log;
  [analytical tests](../crates/engine-analysis/tests/results.rs).
- Third-party code copied: none. No new dependencies.
- Validation: own analytical work, nodal equilibrium and beam under uniform load.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Consistency rule and tolerance

The assembly retains equivalent force vectors, not the original span
distributions or elements. It cannot generically reconstruct the
original loads of `MemberRecovery`. Explicit nodal loads are retained
per case in the same assembly traversal, without mixing them with equivalent
element loads. `global_nodal_force` exposes that source, scaled once by the factor.

For each member, end actions **on the member** in global
coordinates are obtained as `p_e = Element::respond(u_e, committed).internal_force − Σ load_vector`.
This is the global recovery of those actions using `element/1`: no
transformations are inferred from local vectors, whose axis system depends on the kind.
Supplied loads are scaled once, including negative or zero factors;
prescribed displacements retain their physical values.

Member actions on the node have the opposite sign. For each global DOF:

```text
r_d = −Σ p_e,d + F_nodal,d + R_d + D_d
s_d = Σ |f_int,e,d| + Σ |f_carga,e,d| + |F_nodal,d| + |R_d| + Σ |D_d|
|r_d| / max(1, s_d) ≤ 1e-9
```

`R` is recalculated with K and the system's physical loads. `D` denotes the actions
of diaphragm constraints, if any, recovered from the same system.
The scale is calculated before cancellation of element terms; thus zero
moments in a simply supported beam allow for rounding when subtracting
internal forces and equivalent loads. Each DOF uses exclusively its units
N or N·m: force and moment norms are not mixed. The floor of 1 N or 1 N·m
follows the SI convention of analysis reports; it implies an absolute tolerance
of 1e-9 in components with scale less than one. The relative threshold
1e-9 leaves margin above double-precision rounding in assembly, solving,
projection and recovery, and is well below the probe discrepancy.
This is a numerical check, not a physical acceptance tolerance.

The recovery list must include all system members;
partial queries are made afterward through `query`. An imbalance generates
ENG-E-0010 with case, node, global DOF and supplied incident members, plus the
relative residual in the message. An omitted member cannot be identified by
id because the assembly does not retain that registry. Erroneous contributions that
cancel nodally, or members without actions in the specific state, may be
indistinguishable by equilibrium; full model identity validation is not
promised. No history or committed/trial state is modified.

Sizes and finiteness of `respond.internal_force` and `load_vector` are checked
before indexing them for equilibrium. Inconsistent sizes generate ENG-E-0012
with the member. Station and local action checks are retained.

## Stations and scope of extrema

−0.0 is normalized to +0.0 before sorting/deduplicating stations and also when
receiving element output. Numerical equality accepts both representations,
but published stations, queries and extrema locations use +0.0.
Exact algebraic ties retain the lowest station x/L.

The claim of exact extrema under uniform loads is limited to frame2d:
interior roots of shear and the moment derivative are added. Frame3d
does not have that search and its extrema belong to recovered stations.
`results/1` serialization remains out of scope.

## Own analytical oracles

With L=4 m, w=6 N/m and transverse load −w, equilibrium of a simply
supported beam gives Ry_i=Ry_j=wL/2=12 N. Integrating shear:

```text
M(x) = wx(L−x)/2
M′(x) = w(L−2x)/2 = 0 => x=L/2
Mz_max = wL²/8 = 12 N·m
```

The probe without recovery loads used only the rotations of the loaded
solution, producing maximum Mz 8 N·m. It is now rejected by nodal equilibrium;
the same solution recovered with its loads gives 12 N·m, even when requesting only
the ends, because the critical station is added.

In the beam restrained at both ends in bending, symmetry and zero rotations give end
actions Mi=+wL²/12 and Mj=−wL²/12, with magnitude 8 N·m. On the positive face:
`M(x)=−wL²/12+wx(L−x)/2`, minima −8 N·m at both ends and maximum
wL²/24=4 N·m at the center. This existing oracle is preserved.

With end couple +8 N·m, equilibrium: Ry_i=14 N and Ry_j=10 N;
`M(x)=14x−3x²`, peak at x=14/6 m with value 14²/(2w). A load −7 N
directly on the support increases its reaction to 21 N without changing the diagram.
The new test checks that combination, as well as rejecting changed loads,
omitted members and an element with stiffness different from that assembled.

An own fake element delegates physics to the repository frame and alters
output lengths, stations or the sign of zero to test the boundary.
Constant and zero N, Vz, T and My allow testing exact ties without depending
on Mz rounding. No files, processes or external sources are used in fixtures.

## Local verification

Verification on 2026-10-14, without network, with `CARGO_NET_OFFLINE=true` and
`CARGO_TARGET_DIR=/workspace/.cargo-target` in all Cargo commands.
`RUSTFLAGS='--cfg eng_res001_fu_69'` was used, a marker unused in the code,
to isolate artifacts from other builds in the shared directory:
before isolating them, a cached dependency was observed that lacked the new
method despite its presence in the source file.

- `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, 388 tests (including doctests), zero failures;
  13 result integration tests, four new.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, using
  `PATH=/home/box/.cargo/bin:$PATH` and the already installed stable toolchain and target.
  Validates compilation, not execution in a wasm runtime. Nothing was downloaded.
- `bash clean-room/check.sh origin/main`: OK, with the six changed files
  in the index to include the new log.
- `git diff --check` and `git diff --cached --check`: OK. No
  `panic!`, `unwrap` or `expect` were added to library code. No changes to
  `docs/CLEAN_ROOM.md`, `.github/**` or `clean-room/**`.

### Verified temporary rollback

The corrected `results.rs` was saved and temporarily replaced by
`git show HEAD:crates/engine-analysis/src/results.rs`, keeping all
new tests intact. `cargo test -p engine-analysis --test results`
returned exit 101: **9 existing tests passed and the 4 new ones failed**.

- `loads=[]` probe: recovery was accepted; the ENG-E-0010 requirement failed.
- Changed loads/inconsistent members: the first recovery with omitted loads
  was accepted; the requirement for a diagnostic with entities failed.
- Station −0.0: bits obtained 9223372036854775808, expected 0 for +0.0.
- Fake element: malformed `respond`/`load_vector` outputs were not
  checked; the ENG-E-0012 requirement failed. Preexisting local size
  and station checks are retained and also exercised.

The corrected file was restored through `finally` before workspace
checks. The 13 result tests passed with the correction, and the
final workspace passed in full. Git history was neither reverted nor modified
for this check.

Native test/rustdoc tools: Cargo/rustc 1.85.1; Clippy 0.1.99 and
wasm compilation with already installed stable/rustc 1.99.0. No push.
