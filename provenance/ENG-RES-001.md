# Clean-room log · ENG-RES-001

- Ticket / PR: ENG-RES-001 «Structural results and postprocessing». Stacks on
  ENG-ANL-001 (`engine-analysis`, `StaticLinear`/`StaticResults`, PR #52 still
  unmerged). No commit or push.
- Author(s): J2 (agent, Grok Bot) · identification requested by the user;
  executing agent Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Role: implementer and author of own analytical derivations.
- Implemented contract: `engine_analysis::results` module within
  `engine-analysis`, with recovery through `element/1`. Own data model
  with documented public fields, prepared for future serialization.
  The `results/1` schema and its serialization remain pending ENG-CORE-005:
  no JSON, hashes, serialized metadata or ResultSink adapters are invented.
- Permitted sources used: exclusively the ticket request, the own APIs
  of engine-core, engine-contracts, engine-assembly, engine-model,
  engine-analysis, engine-elements-frame and engine-elements-truss, and the existing
  public boundary/reference backend of engine-linalg for tests.
  Template: `provenance/TEMPLATE.md`. No network or external sources.
- Own derivations: [note in this log](#derivaciones-propias) and
  [embedded analytical tests](../crates/engine-analysis/tests/results.rs).
- Third-party code copied: none. No new crate or dependencies.
- Validation: exclusively analytical, through equilibrium and own closed forms;
  no comparison with other engines.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

## Design and conventions

`CaseResults::recover(solution, system, members)` uses only the
`global_displacements` vector from `StaticCaseResult`, which includes prescribed
displacements. It checks size, finiteness and consistency with the system's prescribed values.
It obtains each `u_e` through the DofMap, node and then DOF according to the element, and calls
`Element::recover(ctx, u_e, committed, loads, stations)` without accepting or modifying
history. It does not use the case's copies of free displacements or reactions:
it recalculates reactions through K and the system's physical loads.

The assembled system does not retain the elements or their original span
loads. `MemberRecovery` supplies id, connectivity, element, dimension, parameters,
committed history and **unscaled** loads of the specific case. The list must include
all system members; follow-up #69 checks their nodal equilibrium
against the nodal loads and reactions of the same assembly, according to the rule in
[ENG-RES-001-FU](ENG-RES-001-FU.md). There is no implicit member registry
in LinearSystem: a missing reference is diagnosed when querying the ids of
CaseResults or resolving nodes/DOFs in DofMap; the numeric identifier is not
a vector index. Repeated ids are rejected. Span loads are
scaled once by the case factor; settlements are never scaled.

Nodes and members are returned in ascending id order. Nodal vectors have
six global components: `[Ux,Uy,Uz,Rx,Ry,Rz]` (m/rad) and
`[Fx,Fy,Fz,Mx,My,Mz]` (N/N·m). Undeclared components and free DOF
reactions are zero. In D2, Rz occupies the sixth component, not the third.
End actions retain the local order declared by the element;
frame2d uses `[Ni,Vi,Mi,Nj,Vj,Mj]`. Diagrams use the local positive face
`[N,Vy,Vz,T,My,Mz]`, with N positive in tension. In frame2d, V=Vy and M=Mz;
Vz, T and My are zero. In trusses only N is nonzero.

Default stations are `[0, 0.25, 0.5, 0.75, 1]`. An explicit list
replaces them; 0 and 1 are always added, sorted and deduplicated. An empty list
obtains the ends and applicable critical roots. For frame2d with a nonzero uniform
transverse load, interior stations of zero shear
and zero moment derivative are added. Every resultant is obtained again from the
element, which includes the loads exactly; end actions are not
interpolated. Recovery validates output sizes, stations and finiteness.

Each local field has **algebraic** maximum and minimum, value and station x/L.
Ties retain the lowest station. For **frame2d**, uniform
loads yield affine N/V and quadratic M, so span ends
plus the added roots yield exact extrema. **Frame3d does not add interior
critical stations**: its extrema are limited to recovered stations.
For other kinds it is documented
that extrema are limited to recovered stations; there is no promise to search for
interior roots of arbitrary fields or approximate their interpolation.

`query(nodes, members, fields)` returns an own projection with only the
selected fields. `None` means all; `Some(&[])`, none. The list of
ids preserves result order and does not duplicate entities; local fields
retain the N/Vy/Vz/T/My/Mz order. Excluded displacements, reactions and end
actions are `None`; stations and extrema contain only the selected
local fields. A field that does not apply to the block adds no values. Missing
ids generate ENG-E-0003, with their entity; duplicates in recovery,
ENG-E-0002; sizes, prescribed values or stations outside `[0,1]`, ENG-E-0010;
nonfinite values/overflow, ENG-E-0008; inconsistent element outputs,
ENG-E-0012. Element errors are propagated with the member in entities.

## Derivaciones propias

Artificial SI data: E=200 Pa, A=2 m², I=3 m⁴, L=4 m, w=6 N/m, P=12 N.
Tests check values with absolute tolerance 1e-9 SI and stations
with 1e-12; the analysis equilibrium report must have a relative residual
less than 1e-12. There are no files, processes, network or oracles in the fixtures.

### Exact interior roots

In frame2d local axes, under uniform loads qy and mz:

```text
V(x) = V(0) − qy x
M(x) = M(0) − (V(0)+mz)x + qy x²/2
V(x*)=0       => x*/L = V(0)/(qy L)
M′(x**)=0     => x**/L = (V(0)+mz)/(qy L)
```

Without a distributed couple, both roots coincide. They are added only if they fall
strictly in `(0,1)`. qy is projected from global to local with
`qy=−sin(alpha) qX + cos(alpha) qY`, summing all scaled loads.
The interior check precedes division to avoid overflow of exterior
roots. Exact element recovery is used to evaluate each root.

### Simply supported beam

With qy=−w, Ry_i=Ry_j=wL/2; V(x)=−wL/2+wx and
M(x)=wx(L−x)/2. The root is at x=L/2 and Mmax=wL²/8=12 N·m.
The test requests only x/L=0.2 and 0.8 (unordered and repeated) and checks
that 0.5 is added, along with the ends. Rotations are
theta_i=−wL³/(24EI), theta_j=+wL³/(24EI). The moment at 0.2 is checked against
the parabola. With an end couple +8 N·m, Ry_i=14 N, Ry_j=10 N and
M(x)=14x−3x²: the maximum shifts to x/L=7/12 and equals 14²/(2w).

One variant rotates the beam 90° and sums two global +X uniform loads.
Factors 2, −1.5 and 0 are checked; the negative factor changes the positive maximum
into a negative minimum and the zero factor zeroes loads and results.

### Cantilever with nodal loads

An axial end force H=5 N gives N=H and u_x=HL/(EA). The transverse force
−P gives V=−P and M(x)=−P(L−x), v(L)=−PL³/(3EI), theta(L)=−PL²/(2EI).
End actions: `[−H,P,PL,H,−P,0]`. An additional load −7 N at the support
gives Ry=P+7, Rx=−H and Mz=PL; it does not modify the member diagrams. Checks cover
the lever arms using coordinates away from the origin, equilibrium and
constant-field extrema. The reduced/reaction copies of
StaticCaseResult are emptied to verify that the global vector is used.

### Beam restrained at both ends in bending

With zero rotations and transverse displacements at both ends, local
actions are Mi=+wL²/12, Mj=−wL²/12 and Vi=Vj=wL/2. On the positive face,
M(x)=−wL²/12+wx(L−x)/2: minima at the ends, −wL²/12, and maximum at the
center, +wL²/24. The test requests an empty list and finds the center.

LinearSystem rejects models with all DOFs prescribed. Ux_j is left free
without axial load, retaining the restraint of ALL bending DOFs.
EA/L>0 forces Ux_j=0; since there is no axial-bending coupling in this straight beam,
its solution is exactly that of a beam restrained at both ends under transverse load. No changes are made to
the assembler for this ticket. One variant applies mz=3 N of distributed couple:
V=0 occurs at x/L=0.625, but M′=0 remains at 0.5; both stations are recovered.

### Truss and settlement

Two bars between bases (−4,0) and (4,0) and apex (0,3), L_barra=5 m.
From vertical equilibrium, 2N(3/5)=−P: N=−5P/6=−10 N in each bar.
Vertical reactions P/2 and opposite horizontal reactions ±4P/6. Diagrams are
constant, all other local resultants zero. Checks cover queries
of nodes/members/fields, empty lists, repeated ids and missing references.

Two axial bars in series, total length 4 m, with u_i=0 and u_j=0.03 m, without
load, give N=EA(0.03)/L=3 N, midpoint displacement 0.015 m and reactions ±3 N.
With factor 2 the settlements and these actions are preserved. A free interior
DOF allows solving the system without modifying the existing restriction.

### Invalid inputs

Tests cover negative stations, stations greater than one, NaN and ±infinity;
repeated members/connectivities; missing nodes; inconsistent connectivity,
history and global displacement sizes; nonfinite load/factor;
out-of-plane load; missing case and inconsistent prescribed displacement.
Diagnostics must belong to the catalog. Errors do not modify
committed and trial states. No `panic!`, `unwrap` or `expect` in the module.

## Local verification

Verification on 2026-10-11, without network access:

- `cargo fmt` and `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK; 215 tests, including the nine new
  postprocessing integration tests and existing doctests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, using
  `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`, with the target and
  toolchain already installed. Nothing was downloaded.
- `bash clean-room/check.sh origin/main`: OK. Also run with a temporary index
  and object directory in `/tmp`, using the original objects
  read-only and `git add --intent-to-add` in the temporary index, to include the
  three new files in the checked diff. The real index was not modified,
  no commits were created and no push was performed.
- `git diff --check`: OK, including new files through the same
  temporary index. No `panic!`, `.unwrap()` or `.expect()` in the module or
  its new tests.

Native tests and rustdoc: system Cargo/rustc 1.85.1. Installed Clippy:
0.1.99; wasm compilation: existing stable toolchain, rustc 1.99.0.
The wasm check validates compilation, not execution in a wasm runtime.
ENG-CORE-005 `results/1` serialization and J4 clean-room review
remain pending according to the ticket scope.
