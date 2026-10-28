# Clean-room log · ENG-CORE-007

- Ticket / PR: ENG-CORE-007, «V1 contracts in Rust (traits)» · branch `j2/eng-core-007-traits`; no commit, push or PR in this session.
- Author(s): Codex (GPT-6) as specification, implementation, documentation and verification agent under user instructions · 2026-10-10 (America/Guayaquil). No delegation to other agents.
- Role: specifier and implementer; no oracle operator or similarity auditor activity.
- Contract implemented: crate `engine-contracts`; `uniaxial/1`, `section/1`, `element/1`, `analysis/1` and minimal `results/1` transport. Traits `UniaxialLaw`, `SectionLaw`, `Element`, `ElementFactory`, `SystemView`, `AnalysisMethod`, `ResultSink`, `RunControl`, exchange types, public `conformance` kit and public `examples` templates. Does not include a linear solver, eigenvalue solver, method registry, assembler or complete results schema.
- Permitted sources used: exclusively the categories permitted in [docs/CLEAN_ROOM.md §4](../docs/CLEAN_ROOM.md), internal contracts and own derivations. Bibliographic reference: William McGuire, Richard H. Gallagher y Ronald D. Ziemian, *Matrix Structural Analysis*, 2.ª edición, John Wiley & Sons, 2000. Authors, title, edition and year confirmed through the [Bucknell institutional record](https://digitalcommons.bucknell.edu/books/7/); edition/publisher checked against the [Wiley publisher site](https://bcs.wiley.com/he-bcs/Books?action=index&bcsId=4986&itemId=0471376515). **Chapter/section/equation number: PENDING**, the book text was not consulted and no specific pages or equations were attributed. No code listings, supplementary software or book PDF were downloaded. Numerical validation is based on the own closed-form derivation described below, not on data supposedly extracted from the book.
- Own derivations: [analytical note in this log](#derivación-analítica-propia-y-casos-verificados), [contracts](../crates/engine-contracts/src/traits.rs), [conformance kit](../crates/engine-contracts/src/conformance.rs), [examples](../crates/engine-contracts/src/examples.rs) and [tests](../crates/engine-contracts/src/tests.rs).
- Third-party code copied: none. All code is original. Sole dependency of the new crate: `engine-core` by path; Rust standard library. Neither `engine-core` nor `engine-model` was modified.
- Validation: analytical, with closed forms σ=Eε, N=EAε0, Mz=EIκz, k=EA/L, bar forces, axial equilibrium under uniform load and mass conservation. Standard reference: McGuire, Gallagher y Ziemian, *Matrix Structural Analysis*, 2.ª ed. (2000); chapter/exercise/page **PENDING**, without attributing a specific published example. Also generic conformance, negative tests and own doctests. No production method is declared validated from these mocks.
- Comparison with oracle (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING human review; this session performs no similarity audit or merge approval.

## Internal documents consulted

`docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-CORE-003.md`, `clean-room/forbidden_identifiers.txt`,
`crates/engine-core/src/*.rs`, `crates/engine-model/src/*.rs` and
`docs/CONVENCION_EJES.md` were read first. The workspace configuration,
existing manifests, `README.md` and the `clean-room/check.sh` checker were also consulted.
There is no applicable `AGENTS.md` in the repo or its ancestor directories.

The authorized specification was read directly, without modifying or copying it
outside the repo: `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`,
§§4.1–4.10, and `/workspace/motor-rust/04_TICKETS.md`, ENG-CORE-007.
The contracts were implemented with own types and original documentation.
No paths or sources forbidden by the protocol were opened.

## Scope and architecture decisions

1. **No new external dependencies.** `JsonValue` is a minimal own enum with
   null, boolean, real number, text, array and ordered object. It does not parse or serialize
   JSON. Consumers validate types, finite numbers, keys and ranges; factory
   and method schemas are JSON Schema `String`. The JSON
   boundary and complete schemas belong to other tickets. This replaces
   the serialization dependency suggested in the architecture document.
2. **Provisional matrix storage.** `DMat` preserves dimensions and a
   row-major `Vec<Real>`, validating size/overflow and finiteness. `SymMatrix`
   validates squareness and exact symmetry, without choosing implicit physical tolerances.
   `SparsityPattern` uses a symmetric dense row-major boolean mask. All three
   **are replaced by the ENG-LA-007 backend**. No factorization,
   solving, ordering or eigenvalues are implemented. The examples' closed formulas
   and the kit's private operations serve exclusively for their verification.
3. **Minimal own ModelView.** Borrows dimension and (NodeId, coordinates) pairs.
   `ElementParams` preserves SI properties and optional references already resolved
   to section/law through `Arc`. It does not adapt or alter `engine-model`. The example
   factory receives E, A and density, stores them in the element and needs no
   node references or additional context properties.
4. **Approved repo axes.** X-Y horizontal and Z vertical are applied in D3,
   D2 in X-Y with Z=0 and DOFs Ux, Uy, Rz. The approved decision in
   `docs/CONVENCION_EJES.md` prevails over the pending mention of vertical Y in the
   initial specification. No new D2 elevation convention is defined.
5. **Explicit state.** Flat and finite history, of exact declared length;
   pure response with total strains/displacements and accepted state.
   The kit traverses a path accepting the trial state between samples,
   verifies repeatability and that the response with accepted input preserves forces.
   Includes a test law with nonempty history to exercise that traversal.
   `SystemView` overwrites outputs, accepts exactly u and performs idempotent
   rollback without deleting the last accepted state.
6. **Minimal supporting types.** Uniform member load in global axes;
   x/L stations with [N,Vy,Vz,T,My,Mz]. `CaseRef` is nominal. `Requirements`
   declares mass, Kg, nonlinearity and cases. Result keys/values
   cover minimal blocks and metadata; complete provenance, queries and
   `results/1` serialization remain in ENG-CORE-005. Dimensionless relative
   report norms; the producer declares its scaling for mixed DOFs.
7. **Explicit mocks.** Elastic law and section, linear D2 bar, factory,
   scalar spring system, in-memory sink, observable control and prescribed
   displacement acceptance method. The method does not solve equilibrium;
   its zero norms indicate absence of iterative correction, with explicit
   tolerances of 1e-8. Acceptance occurs before emission to the sink; a subsequent I/O failure
   preserves that last already accepted state and propagates as an error. Prior
   cancellation neither accepts nor emits anything. There are no hidden retries.
8. **Existing diagnostics.** Core `ContractId`, `Dof`, `Dim`, `Real`,
   `NodeId`, `DofIndex`, `Diagnostic`, `EngineError` and `EResult` are reused.
   Conformance violations use ENG-E-0009 with detail in `hint`; implementation
   errors retain their code/context. The kit's nominal reference
   `EntityRef::Law(kind)` identifies the component tested, without asserting
   that a constitutive entity with that name exists in the model.
9. **No unconventional editorial changes.** The existing README only links
   foundational decisions; there is no convention for adding crates or CHANGELOG.
   No lines were added there, or legal files or changes to protected paths.

## Derivación analítica propia y casos verificados

For a linear material σ=Eε is adopted, thus dσ/dε=E. An uncoupled D2 section
integrates axial stress and bending: N=EAε0 and Mz=EIκz; its tangent
is diagonal (EA, EI). Units: E in Pa, A in m², I in m⁴, ε0 dimensionless
and κz in m⁻¹, yielding N in N and Mz in N·m.

For a bar between i and j, L=||xj−xi|| and n=(xj−xi)/L. In D2,
b=[−nx,−ny,0,nx,ny,0] is defined. Elongation is δ=bᵀu; ε=δ/L, N=EAδ/L and
k=EA/L. Elastic energy is U=½kδ², thus F=∂U/∂u=k b(bᵀu)
and K=∂F/∂u=k bbᵀ. This demonstrates symmetry. A common translation or infinitesimal
rotation of the two ends produces bᵀu=0, so F=0. When rotating
coordinates and displacements simultaneously, F and K transform by the
same rotation; physical behavior does not change.

With E=200 Pa, A=3 m², L=2 m and δ=0.01 m, k=300 N/m and N=3 N; the
local face actions are −3 N at i and +3 N at j. The test compares
these values directly and checks N=3 N at all unloaded stations.
With E=200 Pa and ε=0.02, σ=4 Pa. With EA=600 N, EI=10 N·m²,
ε0=0.02 and κz=−0.03 m⁻¹, N=12 N and Mz=−0.3 N·m.

A uniform axial load q gives equivalent nodal forces qL/2 at both
ends. Recovery subtracts those loads from elastic actions and,
by equilibrium dN/dx=−q, obtains N(ξ)=N_el+qL(½−ξ), ξ=x/L.
With q=4 N/m, L=2 m and zero displacements, actions on both faces
−4 N and stations N(0)=4, N(½)=0, N(1)=−4 N; their resultant balances qL.
Transverse loads or moments are rejected because this bar has no bending.

For mass m=ρAL, each translational direction has lumped mass
(m/2) diag(1,1) or consistent mass (m/6)[[2,1],[1,2]], obtained by integrating
the linear functions 1−ξ and ξ. Both conserve m under uniform translation;
with ρ=2 kg/m³, A=3 m², L=2 m, m=12 kg per direction is checked.
Rotations have no inertia in this bar. These numbers are not attributed to
a book example: they are own cases with a closed-form solution.

## Conformance, coverage and limits

The public kit includes checks for all eight traits. For RunControl,
which offers no event reading, it receives an observed trace from any
implementation; checks fractions, monotonicity and notes. The sink kit sends
valid/invalid entries; the concrete sink test checks retention and
atomicity. The factory kit runs physical invariants on its product.
The system and method kits exercise outputs, acceptance, rollback, case scaling,
reports and cancellation. Invariants not observable through a trait
(e.g. internal sink storage) require their own reading in tests.

Tangents: central differences holding the accepted state constant, with infinity-norm
relative error strictly less than 1e-6. State,
symmetry and covariance comparisons: 1e-10 relative + 1e-12 absolute. Rigid modes: residual
scaled by tangent and amplitude, 1e-10 relative + 1e-12 absolute. Three
D2 and six D3 modes are run, and two nontrivial rotations for each permitted axis.
Covariance requires DOFs closed under rotation; global orientations contained
in parameters would need a fixture that expressly transforms them. The kit
does not infer a law's range or test all its points: the consumer supplies
interior differentiable samples and an appropriate absolute SI increment.

Nineteen unit tests and four doctests in the new crate: conformance of
each example, closed-form solution and loads, conserved mass, nonempty state,
four-component spatial section and independent spatial test bar
to verify each of the six rigid modes and rotations about the three axes.
Negative tests detect false derivatives, NaN/inf, incorrect sizes/histories/contracts,
inconsistent accepted stress, spurious rigid force, asymmetric mass,
covariance violation, incomplete system output, missing reports,
false convergence and ignored cancellation. Inputs, geometry,
parameters, stations, loads, nested results, overflows and I/O errors are exercised.

## Local verification

Final session results:

- `~/.cargo/bin/cargo fmt --check`: successful.
- `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: successful.
- `~/.cargo/bin/cargo test`: successful; full workspace, 43 unit
  tests and 25 doctests, including 19 and 4 respectively from the new crate.
- `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`: successful.
- `bash clean-room/check.sh origin/main`: successful; also run
  with temporary index/objects to include all new files without
  changing the user's real index. Additional review of forbidden identifiers
  and protected paths in all ticket files: successful.

`#![deny(missing_docs)]` requires documenting the entire public API. The first
run detected an exact decimal comparison in a doctest (3×0.2 versus
0.6); it was corrected to a comparison with absolute tolerance 1e-12. The formulation
was unchanged. Strict rustdoc detected mathematical notation with brackets
interpreted as links; it was marked as code in documentation. Cargo.lock only records the local crate and its existing dependency.
No remote CI, production benchmark, oracle comparison or wasm tests were
run; that work is not part of this contract delivery.
