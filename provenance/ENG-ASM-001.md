# Clean-room log · ENG-ASM-001

- Ticket / PR: ENG-ASM-001, Assembler, supports and imposed displacements;
  implementation in the existing branch on ENG-LA-007. No commit, push or PR.
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil).
- Role: implementer; original analytical derivations and fixtures. No oracle
  operator or similarity auditor activity.
- Implemented contract: `engine_contracts::SystemView` over types and backend
  `engine-linalg` from ENG-LA-007; global/reduced numbering, K/M/Kg/F contributions,
  support elimination, reduction for imposed displacements and reactions.
  `ElementContribution` interface, without implementing `Element` or finite elements.
- Permitted sources used: own documents and contracts listed below;
  original derivations through energy, equilibrium and equation partitioning. Bibliographic
  citation with edition, page or book equation: **PENDING**. No example
  was consulted in or attributed to an unavailable book.
- Original derivations: [note in this log](#derivaciones-propias) and
  [embedded analytical fixtures](../crates/engine-assembly/tests/fixtures/mod.rs).
- Third-party code copied: none. There are no new external dependencies,
  copied library sources or solver implementation in this crate.
- Validation: exclusively analytical and conformance with own contracts.
  Matrices/solutions of two- and three-spring chains, two-segment axial
  bar and settlement; values written by hand and derived below. No
  numbers from other engines were used. Precise bibliographic citations: **PENDING**.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados"
- Clean-room reviewer: PENDING (J4)

## Internal documents consulted

The following were read: `/workspace/motor-rust/04_TICKETS.md` (ENG-ASM-001, ENG-ASM-002,
ENG-CORE-003, ENG-LA-007 and limits of ENG-LA-008/009);
`/workspace/motor-rust/00_PROYECTO.md`;
`/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md` (§3, §4, §4.7 and
determinism); `docs/CLEAN_ROOM.md`; `docs/CONVENCION_EJES.md`;
`provenance/TEMPLATE.md`; `provenance/ENG-LA-007.md`; the types/errors of
`engine-core`, the domain of `engine-model`, traits/types/conformance of
`engine-contracts` and own API of `engine-linalg` (DMat, SymMatrix,
SparsityPattern, SparseMatrix, Triplet and LinAlgBackend). No applicable AGENTS.md
was found. No external engine code, oracle results
or book listings were consulted. The sources are permitted under docs/CLEAN_ROOM.md §4.

The approved convention of `docs/CONVENCION_EJES.md` takes precedence over the provisional
text of the architecture document: D2 is in horizontal X-Y, with Rz;
D3 uses Z upward and a right-handed triad. No other 2D plane is introduced.

## Derivaciones propias

For a contribution with location L, each entry k_ij is added to K[L_i,L_j].
This is obtained by substituting u_e[i]=u[L_i] in the energy
V_e=½u_eᵀk_e u_e. The equivalent force is added to F[L_i]. There is no automatic
mirroring or division of halves: both symmetric entries are emitted.

In an axial chain V=½Σ k_i(u_{i+1}−u_i)². Each term contributes
[[k_i,−k_i],[−k_i,k_i]]. For example, with k=[100,200] N/m:

```text
K = [ 100  -100     0
     -100   300  -200
        0  -200   200 ] N/m
```

With k=[100,200,300], the rows are [100,−100,0,0], [−100,300,−200,0],
[0,−200,500,−300], [0,0,−300,300]. Setting u_1=0 and applying P=12 N at the
last node, equilibrium transmits P through each spring: Δu_i=P/k_i. Two
springs give u_f=[0.12,0.18] m, three give [0.12,0.18,0.22] m, with R_1=−12 N.
The fixtures contain the complete matrices and these solutions as literals.

For a bar homogeneous in each segment, ε=Δu/L, σ=Eε and N=Aσ, hence
k=EA/L. Taking E=200 Pa, A=2 m², L_1=2 m and L_2=4 m, k=[200,100] N/m.
The matrix has rows [200,−200,0], [−200,300,−100], [0,−100,100]. With P=12 N,
u_f=[0.06,0.18] m. It is an analytical example with consistent units, not a
real material calibration. The test supplies the calculated blocks, without
implementing bar element physics.

Partitioning the free f and prescribed p indices in Ku=F+R, in the free
rows R_f=0, so K_ff u_f=F_f−K_fp u_p. In the prescribed rows,
R_p=K_pf u_f+K_pp u_p−F_p. R is the support action on the node; positive
along the global axis of its component. For a chain with a rigid mode,
the row sum of K is zero: ΣR+ΣF=0 in equilibrium. Only forces in the
same direction are summed; forces are not mixed with moments.

Settlement: k_1=100, k_2=200 N/m, u_1=0, u_3=0.03 m. Without external forces,
300u_2−200·0.03=0, so u_2=0.02 m, R_1=−2 N, R_3=2 N.
With F_2=12 N and F_3=4 N, RHS=12+6=18 N, u_2=0.06 m, R_1=−6 N and
R_3=200(0.03−0.06)−4=−10 N. Equilibrium is −6−10+12+4=0.
The factor λ scales only forces: RHS=12λ+6; at λ=2 it gives 30 N,
not 36 N. These tests also use λ=0 and λ=−1 and verify equilibrium.

The nonconsecutive location test uses A at [2,0] with matrix
[[30,4],[4,10]] and B at [1,2] with [[20,5],[5,6]]. Index substitution gives
K=[[10,0,4],[0,20,5],[4,5,36]]. Restraining the central global index gives
K_ff=[[10,4],[4,36]]. This algebraic fixture checks location without
attributing those matrices to a physical element.

The M/Kg tests use algebraic interface data: in each link,
M_l=diag(2,2), M_c=[[2,1],[1,2]], Kg=[[3,−3],[−3,3]] and a nodal mass of 5 kg
at the central node. The sum of contributions reduces to M_l=diag(9,2),
M_c=[[9,1],[1,2]], Kg=[[6,−3],[−3,3]]. These are assembly checks; no
element mass model or geometric formula is announced.

## Design decisions and limits

1. New crate `engine-assembly` version 0.1.0 with four internal path-deps.
   Modules `dofmap`, `constraints`, `assemble`, `system`, `reactions`; public rustdoc
   under `#![deny(missing_docs)]` and compilable example. No existing crate
   was modified. Only root Cargo.toml adds the member and Cargo.lock
   records the internal package, without new external dependencies.
2. `DofMap` numbers only declared DOFs; it supports axial subsets. Order
   by node id and then Dim::dofs(), independently of insertion.
   `Constraints` retains global→reduced and reduced→global maps, plus
   prescribed values. `LinearSystem::reduced_index` links node/DOF to the
   reduced index. No HashMap iteration or threads are used.
3. A PrescribedDof may restrain a free DOF or replace the zero of a
   support. The intersection between those two lists is intentional; repetitions
   within each list fail even if the values match. All DOFs
   restrained is rejected with ENG-E-0010, as a case unsupported by SystemView.
4. Square and exactly symmetric K/M/Kg contributions according to SymMatrix. The
   DOFs of location are distinct and valid; their order belongs to the producer.
   Matrices/vectors are summed in input order. Absence of M/Kg contributes zero;
   diagonal nodal masses are added to both MassKind, without fabricating element
   matrices. Cases without loads exist; references to missing cases fail.
5. Global CSR matrices and union pattern of K, both M and Kg, including explicit
   zeros. Dense reduction because the current SymMatrix is dense: memory
   O(n_f²). CSR contraction with u sums rows in stable order without expanding global
   K. Factorization and solution are delegated entirely to the ENG-LA-007
   backend; no elimination or regularization is written in this ticket.
6. SystemView uses physical forces `(Ku)_f` and `λF_f`. Settlement is included
   once in the internal force. `reduced_rhs` subtracts it once for a direct
   solve. Constant tangent K_ff; constant u_p does not depend on case/factor.
   Reactions include loads on supports. Translations m, rotations rad, forces
   N, moments N·m; K/Kg N/m, N or N·m, M kg, kg·m or kg·m². Mixed norms
   need explicit scales from the analysis method.
7. Successful evaluations update the trial, without implicit acceptance.
   `accept` reevaluates and accepts exactly u atomically; failures preserve
   accepted state, trial and outputs; rollback is infallible/idempotent. There is no material
   history in the linear system. Initial state u_f=0 with imposed u_p, without
   claiming equilibrium. Private and immutable matrices and constraints.
8. Core catalog: ENG-E-0010 sizes/indices; ENG-E-0008 nonfinite inputs,
   products or sums; ENG-E-0003 missing cases/nodes; ENG-E-0002
   duplicates; ENG-E-0011 incompatible DOFs. ENG-E-0006 from the solver is translated
   from reduced to global index and adds Node and component in the message.
   ENG-E-0012 for an inconsistent mechanism diagnostic without a valid physical index.
   The pivot indicates part of the mechanism, without characterizing all its movements.
9. `ElementContribution` is an interface of evaluated linear data, not an
   Element. Kg is received for a reference state and remains constant;
   `geometric(u)` does not obtain axial forces or run P-Delta. Element evaluations,
   transformations, history, distributed loads and recovery of local
   actions remain for ENG-ELM-*. Full domain validation belongs
   to engine-model; this constructor validates the data it uses, without interpreting
   members/sections/laws/combinations. A dynamic element adapter
   must preserve this boundary and declare its Kg update policy.
10. ENG-ASM-002 retains master-slave and constraint transformations.
    ENG-LA-008/009 retains production sparse/modal solver. The reference
    backend used in tests is for small models. Bit-for-bit determinism
    between native/wasm PENDING ENG-CORE-010; there are no new transcendental functions
    or direct libm dependency in this crate.

## Tests and tolerances

The embedded Rust fixtures declare `source` and `citation`; they do not read files,
clock or threads. Validation is analytical, without an oracle or validation
references to external engines. Unavailable bibliographic citations are
marked PENDING in the fixture and here, without inventing edition/page/equation.

- Literal expected matrices, exact comparison with representable integers.
- Analytical displacements/forces: error ≤ 1e-10·max(1,|esperado|).
- Equilibrium ΣR+ΣF: absolute tolerance 1e-9 N.
- Kit check_system_view over linear samples and with settlement: tangent
  through finite differences, masses/Kg, case scaling and output overwriting.
- D2/D3 numbering, node/DOF of free indices, central support, permuted/nonconsecutive
  location, equivalent forces and both MassKind.
- Negative tests: duplicate DOF/node/case/support/prescribed value, out-of-range indices,
  incompatible component, NaN/±inf and overflows, missing case, support on
  nonexistent node, incompatible or asymmetric matrices/vectors, all DOFs
  restrained, free mechanism and isolated DOF without stiffness. ENG-E-0012 is also
  tested with a test backend reporting a nonexistent singular index.
- Accept/rollback and atomicity on failure; ten repetitions with equal matrices
  and bit-for-bit identical solutions on the same target.

## Local verification

Commands with `/home/box/.cargo/bin` in PATH and `RUSTUP_TOOLCHAIN=stable`
(rustc 1.99.0). Final results:

- `cargo fmt --check`: successful.
- `cargo clippy --all-targets -- -D warnings`: successful.
- `cargo test --workspace`: successful; 10 new integration tests and
  one new doctest, in addition to the existing tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`: successful.
- Target wasm32-unknown-unknown installed; `cargo check --target
  wasm32-unknown-unknown -p engine-assembly`: successful.
- Also `cargo check --target wasm32-unknown-unknown -p engine-assembly
  --tests`: successful. Test compilation; execution in a wasm runtime
  **PENDING**, it is not claimed that they were run in wasm.
- `git diff --check`: successful. No existing crate or protected path
  modified; no new license field, LICENSE files or legal text.

No commit, push, PR or changes to the user's index were made. Clean-room
review: see next section.

## Corrections after J4's review

Follow-up in branch `j2/asm-001-j4-findings` (Codex author of the original ticket;
corrections by J2, reviewer OpenCode). Findings and resolution:

1. **Medium — `geometric(u)` ignored `u`** (SystemView contract: «axial propio de
   u»). The system cannot recompute Kg(u): it receives already evaluated Kg.
   Decision: explicit error instead of a silently incorrect value. If there are
   nonzero geometric contributions, `geometric(u)` only accepts the reference `u`
   (by default 0; declarable with `LinearSystem::set_geometric_reference`) and
   returns ENG-E-0013 («capacidad no soportada», added to engine-core in the follow-up
   PR; ENG-E-0010 was previously used due to the lack of a code; ENG-E-0009 is only
   for contract versions); without contributions, Kg=0 for all `u`. Consequence:
   the `check_system_view` kit is run with Kg only at the reference state.
   **Before buckling/P-Delta** a Kg(u) producer must exist (ENG-ELM-*).
2. **Low — `dof_label` panics out of range.** The trait cannot return
   an error (precondition d < n of the contract); `try_dof_label` → ENG-E-0010 is added.
3. **Low — negative nodal mass and element stiffness** → ENG-E-0010 (catalog without
   a more specific code). Also element mass with negative diagonal. Full
   positive definiteness is not checked, only the diagonal.
4. **Low — reactions:** rustdoc clarified: recovered at every prescribed DOF
   (supports and imposed displacements), not only «nodos apoyados».
5. **Low — log:** reviewer updated (this section).
6. **Low — symmetry:** `assemble_matrix` remains exact (`EXACT_SYMMETRY = 0`);
   new `assemble_matrix_with_tolerance(.., tol)` relative to máx|a| of the block,
   symmetrizes with the mean; negative/nonfinite tolerance → ENG-E-0010. For ASM-002.

New tests: `crates/engine-assembly/tests/j4_followup.rs`.

OpenCode review (alibaba-token-plan/qwen3.7-max): first pass CHANGES
REQUIRED (code ENG-E-0009 misused, consistent mass test, comment);
corrected in this branch.

Follow-up 2026-10-11: ENG-E-0013 «Capacidad no soportada» in the engine-core catalog (uniqueness/stability test); `geometric(u)` ≠ reference now uses ENG-E-0013 instead of ENG-E-0010.

ENG-ANL-001 follow-up (2026-10-11): `LinearSystem::solve_cases` reuses one
factorization for multiple cases and `DofMap::coordinates` retains nodal
moment arms. Derivations, authorship, tests and pending review of this extension:
[provenance/ENG-ANL-001.md](ENG-ANL-001.md). The reviewer of the original
implementation is not attributed to this extension.
