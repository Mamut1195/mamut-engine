# Clean-room log · ENG-ANL-001

- Ticket / PR: ENG-ANL-001 «Linear static analysis». No commit or push.
- Author(s): J2 (agent, Grok Bot) · identification requested by the user;
  executing agent Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Role: implementer and author of original analytical derivations.
- Implemented contract: `static.linear`, API `StaticLinear::run` / `run_about`
  over `LinearSystem`, according to the specific API requested. Adaptation to the
  generic trait `AnalysisMethod` (`analysis/1`, sink/control/settings) remains
  outside this interface.
- Permitted sources used: exclusively the ticket request, own APIs
  of engine-core, engine-contracts, engine-model, engine-assembly,
  engine-elements-truss, engine-elements-frame and the public boundary of
  engine-linalg. Template: `provenance/TEMPLATE.md`. No external sources,
  network searches, books, results from other engines or oracles.
- Original derivations: [note in this log](#derivaciones-propias) and
  [embedded linear benchmarks](../crates/engine-analysis/tests/analytical.rs).
- Third-party code copied: none. Only internal dependencies by `path`,
  including test dependencies; no new external dependencies.
- Validation: analytical, closed forms derived below and call counter
  for the existing reference backend. No new numerical solver was implemented.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

## Design, convention and limits

`solve_cases` validates a nonempty list of distinct names and all their RHS
before factorizing K_ff once. It reuses that factor for each RHS in the
requested order and translates errors in the same way as `solve`. It does not change the semantics
of `solve`, which still factorizes on each call. No operation accepts or
modifies the trial state. The pivot of a mechanism indicates an involved node/DOF
without describing the entire mode. The map retains the original coordinates
for force moment arms; the model is no longer required during analysis.

Physical loads include nodal actions, equivalent element loads and loads
on supports. F is scaled, never u_p. Partitioning Ku=F+R gives:

```text
K_ff u_f = λ F_f − K_fp u_p
R_p = K_pf u_f + K_pp u_p − λ F_p
```

The six components are ordered Fx,Fy,Fz,Mx,My,Mz. The fields
`applied_force_sum`, `reaction_sum` and `residual_force` sum by component;
their three rotational components are nodal couples without moment arms. The total moment
`residual_moment` is Σ(M + (x−p)×F) of physical loads and reactions about p,
the origin by default. Thus couples alone may have a nonzero sum
even in equilibrium: the cantilever with an end load has ΣMz=PL, canceled
by the force moment arms. The sign follows the right-hand rule, with undeformed
positions. D2 represents XY, Z=0 and Rz; D3 uses XYZ and Rx,Ry,Rz. The
original coordinates are respected without projection; D2 geometry must satisfy Z=0.

The global relative residual is the maximum of the residual forces divided by
max(1 N, sum of force magnitudes by component) and the total residual moments
divided by max(1 N·m, sum of magnitudes of couples and moment-arm terms
by component). Loads and reactions are included in both denominators. It is not
normalized by the algebraic sum, which would be zero in equilibrium. The free residual
is ‖(Ku−λF)_f‖∞ / max(1, ‖(Ku)_f‖∞, ‖λF_f‖∞), with all values in SI.
This last norm mixes forces and moments: it is a numerical indicator, not a
physical norm invariant under changes of units. The absolute norm is also
retained. Nonfinite inputs and products/sums/reports are rejected.

The analysis is linear and first-order: it does not use Kg, regularize mechanisms or
update geometry. engine-model combinations are declarative; the
combined cases must be supplied as loads of a system case. With u_p=0,
u(A+B)=u(A)+u(B), and likewise for reactions. With settlements,
u(A+B)=u(A)+u(B)−u(0), where u(0) includes the same settlement only once.
The reference backend and dense reduction are intended for small models;
scalability or binary identity between native and wasm is not promised.

## Derivaciones propias

All fixtures are embedded in Rust, without access to files, network, clock or
processes during tests. Absolute tolerances: 1e-12 m/rad for
displacements and 1e-9 N/N·m for reactions and equilibrium; relative indicators
below 1e-12. These are artificial SI data for checking equations.

### Symmetric two-bar truss and superposition

Bases (−4,0,0), (4,0,0), apex (0,3,0); E=200 Pa, A=2 m², L=5 m.
With the bases restrained, each bar has energy EA/(2L)(d·u)². The cosines
c=4/5, s=3/5 give Kxx=2(EA/L)c²=102.4 N/m,
Kyy=2(EA/L)s²=57.6 N/m and Kxy=0 by symmetry.
Case A applies H=10 N, B applies −P=−12 N in Y; AB applies both.
u_A=(10/102.4,0), u_B=(0,−12/57.6); u_AB is their sum.
In B, Ry=P/2 at each base; left Rx=Pa/(2h)=8 N and right=−8 N.
The counter checks exactly one factorization and three solves,
including superposition of displacements and reactions. It verifies that the
previous accepted/trial state remains intact.

### Bar with settlement and load on a support

Nodes at x=0,4,6 m; EA=400 N. Axial stiffnesses 100 and 200 N/m.
u1=0, u3=s=0.03 m, u2 free; the Uy are restrained. Equilibrium at node 2 is
100u2+200(u2−s)=λP, so u2=(λP+6)/300 m.
Without load u2=0.02 m, R1=−2 N, R3=2 N. Case A applies P=12 N at node 2 and
Q=7 N directly on support 1: R1=−100u2−λQ,
R3=200(s−u2). With λ=2, u2=0.1 m, R1=−24 N, R3=−14 N;
the total physical load is 2(P+Q)=38 N and ΣF+ΣR=0. The prescribed values remain
s, not 2s. Another case B with P=5 N verifies the documented affine superposition.

### Cantilever with point and distributed loads

E=200 Pa, I=3 m⁴, A=2 m², length L=4 m. Clamped end at (2,3,0), tip
at (6,3,0); thus the moment arm must use coordinates rather than nodal indices.
With end force −P in Y, EI v''=−P(L−x). Integrating with
v(0)=v'(0)=0: v(L)=−PL³/(3EI), θ(L)=−PL²/(2EI).
Ry=P and Mz=PL at the clamped end. About the origin,
ΣMz=−P·6+P·2+PL=0; about p, each x is replaced by x−p_x.
For constant q=−6 N/m, EI v''=−6(L−x)²/2; integrating:
v(L)=−6L⁴/(8EI), θ(L)=−6L³/(6EI), Ry=6L, Mz=6L²/2.
The test uses the equivalent load vector of the frame element, including
its couples, and checks global physical equilibrium. A backend returning u/2
produces free residual P/2, residual Fy=−P/2 and global relative indicator 1/3;
M(p)=M(0)−p×ΣF is also checked.

### Symmetric portal frame with finite axial stiffness

Height h=3 m, span b=4 m; two clamped columns and a rigidly connected beam.
Each upper corner receives H/2=6 N in X. Symmetry: both horizontal
displacements Δ and rotations θ are equal; the vertical ones are v and −v. Summing the bending
energy of the columns and beam, and the axial energy of the columns:

```text
V = 12EI/h³ Δ² + 12EI/h² Δθ + (4EI/h + 6EI/b)θ²
  + (EA/h + 24EI/b³)v² + 24EI/b² vθ − HΔ
```

The beam does not elongate under this symmetry. From ∂V/∂v=0,
v=−βθ, β=(12EI/b²)/(EA/h+24EI/b³).
From ∂V/∂θ=0, θ=−6EIΔ/(h²D), D=4EI/h+6EI/b−12EIβ/b².
Finally ∂V/∂Δ=0 gives
Δ=H/(24EI/h³−72(EI)²/(h⁴D)). The bases each have Rx=−H/2;
left Ry=−EA v/h and right=EA v/h. Δ,v,θ, the reactions,
ΣH+ΣRx=0 and the global moment are checked, including the moment arm of height h.

### Spatial bar and invalid inputs

Bar between (1,2,3) and (3,5,7) m: L=√29, d=(2,3,4)/L. Only Uz at the end
remains free; Kzz=EA·16/L³ and u_z=PL³/(16EA). The supports take the remaining
forces and couples; Rx=7, Ry=−8, Rz=9 N·m are also applied on prescribed DOFs.
The bar forces are collinear: their combined moment is zero. The applied
couples are canceled by opposite-sign reactions. The test reverses the
node order and uses a moment reference point away from the origin.

A horizontal D2 bar with free end Uy lacks transverse stiffness:
the mechanism must report ENG-E-0006 with node 2 and Uy, and global index 3 in
entities. Empty/duplicate lists (ENG-E-0010), missing case
(ENG-E-0003), NaN/±∞ factors and overflow (ENG-E-0008), zero and
negative factor, nonfinite point and invalid backend diagnostics (ENG-E-0012) are checked.
Springs are omitted because engine-elements-spring does not exist in main.

## Local verification

Verification on 2026-10-11, without network access:

- `cargo fmt` and `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK; ten new integration tests (eight for
  analysis and two for batch solving), in addition to the existing tests and
  the extension of the ENG-E-0012 diagnostic test.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --workspace --all-targets --target wasm32-unknown-unknown`: OK;
  includes the analytical fixtures and the counting backend.
- `bash clean-room/check.sh origin/main`: OK. The same
  script was also run with temporary Git metadata and index in `/tmp`, sharing the
  read-only local objects and registering the new files in that
  temporary index, so they would also be included in the reviewed diff.
  The real index was not modified and no commits were created.
- `git diff --check`: OK, including the diff with new files from the temporary
  check. No `panic!`, `.unwrap()` or `.expect()` in engine-analysis/src.

Native tests, Clippy, formatting and documentation: system Cargo/rustc 1.85.1.
The wasm check with that compiler failed because the std library for
that target was absent. This was resolved using the **already installed** rustup `stable` toolchain
(rustc 1.99.0), with `/home/box/.cargo/bin` at the start of PATH and
`RUSTUP_TOOLCHAIN=stable`; nothing was downloaded. `cargo check` for wasm checks
compilation; it does not amount to running the tests in a wasm runtime. Execution
in a wasm runtime and J4's clean-room review remain pending.
