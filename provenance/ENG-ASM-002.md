# Clean-room log · ENG-ASM-002

## Integration with main (2026-10-13)

- Ticket / PR: ENG-ASM-002, branch `j2/eng-asm-002`, PR #60; resolution of the
  ongoing local merge with `origin/main`, including the `solve_cases` extension from #52.
- Author: Codex (GPT-6-based agent), implementer role. Independent review
  of this integration: PENDING; not attributed to previous reviews.
- Sources: the local versions of both branches in the merge index
  stages; own contracts and code of `engine-assembly`, `engine-analysis`,
  `engine-model`, `engine-core` and `engine-linalg`; own tests and logs
  `ENG-ASM-001.md` and `ENG-ANL-001.md`; `docs/CLEAN_ROOM.md`. No network, external
  queries, oracles, third-party code or new dependencies.
- Declaration for this session: I did not consult source code from OpenSees, Stabileo
  or the engines or paths prohibited by `docs/CLEAN_ROOM.md`.

### Integration decisions

`DofMap` retains a single geometry representation:
`BTreeMap<NodeId, (Dim, [Real; 3])>`. `geometry()` obtains dimension and position
to validate diaphragms; `coordinates()` obtains the position from that same
entry for force moment arms and `engine-analysis` recovery.
The original coordinates and ENG-E-0003 with missing node entity are retained.
No copy of the positions is added.

`solve_cases` validates the full batch before factorizing, retains order
and factorizes `K_red = T_fᵀ K T_f` only once. Each case uses exactly the
`reduced_rhs` of `solve`: `T_fᵀ(factor F − K u_0)`. Both return free
independents; `Constraints::expand` recovers `u = T_f q_f + u_0`, including
slaves and prescribed masters. The factor scales only F, never u_0.
Without diaphragms main's partition is retained:
`K_ff u_f = factor F_f − K_fp u_p`.

`R_p = (Tᵀ(Ku − factor F))_p` is retained: the reaction of a prescribed master
includes the forces of its slaves. Without master-slave constraints it
reduces to `K_pf u_f + K_pp u_p − factor F_p`. A dependent DOF is not reported
as a support reaction.

All solving methods use the same translation of ENG-E-0006 to
node, DOF and global numbering. An inconsistent backend diagnostic gives
ENG-E-0012, including «El solver informó un GDL global inexistente.» if global
labeling fails. An invalid public query to `DofMap::label` retains
ENG-E-0010; it is not confused with an internal solver violation.

`StaticLinear` still uses `solve_cases` and expands the results. Its
free residual is now `T_fᵀ(Ku − factor F)`; the relative scale uses
`max(1, ‖T_fᵀ Ku‖∞, ‖T_fᵀ factor F‖∞)`. Selecting only the physical rows of
masters would omit the actions transferred by slaves. Global
equilibrium retains all physical forces, their moment arms and the projected
support reactions. The methods do not modify accepted or trial states.

### Original derivation and analytical tests

Shared Fixture `crates/engine-assembly/tests/fixtures/rigid_cases.rs`:
two slaves at (1,1,3) and (3,1,3) m, master at (0,0,3) m, and two springs
Ux/Uy with k=100 N/m per slave. Each restrained anchor coincides with its slave;
all springs are on the same floor, so no out-of-plane
couple is introduced without a support to balance it. Each pair contributes the energy
`½k[(Ux − yθ)² + (Uy + xθ)²]`, with θ=Rz and moment arms relative to the master.
By summation and differentiation with respect to (Ux,Uy,θ):

```text
K_ind = [ 200     0   -200
            0   200    400
         -200   400   1200 ]
```

Load A=(10,20,5) at the first slave is transferred to (10,20,15) at the
master; B=(-20,10,-5) at the second is transferred to (-20,10,45). Forces
are in N and moments in N·m. With prescribed Ux=a=0.03 m:

```text
θ  = [factor (Mz − 2 Fy) + 200 a] / 400
Uy = factor Fy / 200 − 2 θ
R_maestro,Ux = 200 a − 200 θ − factor Fx
```

With free master: `θ = factor (Mz + Fx − 2 Fy) / 200`,
`Ux = factor Fx / 200 + θ` and the same expression for Uy. The slaves satisfy
`Ux_s = Ux − y θ`, `Uy_s = Uy + x θ`, `Rz_s = θ`; anchors have
zero displacement and reactions `−100 Ux_s`, `−100 Uy_s`.

Batches A×2, B×(−1) and an empty case ×0 are compared with these expressions,
with `solve` case by case (exact equality of the reduced vectors), and with
each slave's constraints. The counter confirms one factorization
per batch and one solve per case. Free and prescribed master,
result order and preservation of distinct accepted and trial states are tested.
Analytical tolerance: `1e-12 max(1, |esperado|)`.

In `engine-analysis/tests/diaphragm.rs`, global recovery, reactions, forces
and moments about (4,-2,1) m and zero projected residual are also
checked. A test solver returns half the solution:
the physical rows of masters have zero stiffness, but the projected
residual detects the error (norms 20 and 22.5, relative values 0.5).

It is verified that a late invalid case (duplicate, missing, NaN, infinity
or overflow when projecting finite forces with moment arm x=3) prevents any
factorization/solve of the batch and preserves both states. Previous
tests from main and the branch are retained, including solver errors.
The formulas are original derivations; no outputs from other engines were used
and they are not attributed to a book not consulted.

### Local integration verification

Commands run with `/home/box/.cargo/bin` in PATH and
`CARGO_NET_OFFLINE=true`, without network access:

- `cargo fmt` and final check `cargo fmt --check`: successful.
- `cargo clippy --all-targets -- -D warnings`: successful.
- `cargo test --workspace`: successful; includes four new integration
  tests (two assembly and two analysis tests), plus existing tests
  from both branches and doctests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: successful.
- `cargo check --workspace --target wasm32-unknown-unknown`: successful.
  Compilation was verified; execution in a wasm runtime is not claimed.
- `git diff --check`: successful; working files without merge markers.
- Libraries `engine-assembly` and `engine-analysis` without `panic!`, `unwrap` or
  `expect`; no new dependencies.

No `git add`, commit or push. The index retains the merge stages in
`dofmap.rs` and `system.rs` (status `UU` until the user adds the files);
the working contents are resolved. The merge remains open.
