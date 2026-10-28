# Clean-room log · ENG-LA-007

## Additional declaration for the ENG-LA-009 session

- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil), requested identification; session executed by Codex (GPT-6).
- Role: implementer of the own modal follow-up; the historical ENG-LA-007 record is retained below.
- Sources used in this session: only the repo's own code and documentation and analytical derivations; no network or external sources.
- Third-party code copied: none. See scope, derivations and validation in `ENG-LA-009.md`.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'
- Clean-room reviewer: PENDING (J4)

This declaration corresponds to the current session and does not modify the sources or
the historical assessment of the dependency recorded below.

## Original record

- Ticket / PR: ENG-LA-007, internal numerical backend trait and adapter; branch
  `j2/eng-la-007-linalg-backend`, stacked on ENG-LA-006. No commit, push or PR.
- Author(s): Codex gpt-6.1-sol high · 2026-10-10 (America/Guayaquil), identification
  requested for this ticket. Final verification extended to 2026-10-11.
  No delegation to other agents.
- Role: implementer and author of own derivations/tests; no oracle
  operator or similarity auditor activity.
- Implemented contract: `LinAlgBackend`, `SymFactorization`, `FactorInfo`,
  `GeneralizedEigen` and definitive own matrix types in `engine-linalg`;
  compatible reexports in `engine-contracts`. Boundary for linsolve/1 and
  eigen/1, without implementing the production sparse solver or the full modal
  analysis of ENG-LA-008/009.
- Permitted sources used: internal contracts listed below, locally generated
  API documentation for faer 0.24.4, manifests/SPDX metadata and license
  notices of the cached package; own derivations of elimination,
  congruence, Cholesky and orthogonal rotations. No equation or
  result is attributed to an unconsulted page.
- Own derivations: [analytical note in this log](#derivaciones-propias),
  [reference backend](../crates/engine-linalg/src/reference.rs) and
  [generic suite](../crates/engine-linalg/tests/conformance.rs).
- Third-party code copied: none. The previous matrix definitions
  were moved from own `engine-contracts` code; algorithms and
  adapters were written from equations and API signatures. No implementation
  code from libraries or external engines was opened.
- Validation: exclusively analytical/identities. Classical eigenvalue formula
  for tridiagonal Toeplitz, derived via sine below; **precise bibliographic citation
  PENDING**, without attributing pages. No external fixtures or outputs were used.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
  CalculiX code was not consulted either, nor was any engine run as an oracle.
- Clean-room reviewer: PENDING human review. No merge
  approval or similarity audit is performed in this session.

## Internal and dependency documents consulted

`/workspace/motor-rust/04_TICKETS.md` was read, in particular ENG-LA-007/008/009
and ENG-CORE-007; `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`, in
particular §§4.5–4.6, 9 and 10; `docs/adr/ADR-002-algebra-lineal.md`;
`docs/CLEAN_ROOM.md`; `provenance/TEMPLATE.md`; `provenance/ENG-CORE-007.md`;
`crates/engine-contracts/src/matrix.rs`, `lib.rs` and the matrix references
in their examples/tests; `engine-core/src/lib.rs` and `catalog.rs`; manifests and
workspace configuration. No applicable AGENTS.md was found.
The `clean-room` lists/checker were also read to verify the
new files, without modifying them or opening prohibited paths.

For faer, the normalized Cargo.toml and README of package **0.24.4** were consulted,
its native/wasm graph metadata obtained through Cargo, and HTML
documentation generated with `cargo doc` in a temporary project: module `linalg::solvers`,
`Lblt`, `PartialPivLu`, `Llt`, `SelfAdjointEigen`, `Solve`, matrix views,
diagonals and permutations. Only signatures and documentation prose, without opening the
«Source» links or implementation files. The reference documents are
[faer 0.24.4 docs.rs](https://docs.rs/faer/0.24.4/faer/linalg/solvers/index.html)
and [crates.io listing](https://crates.io/crates/faer/0.24.4); web and DNS access
failed, so no successful remote consultation or new release verification is
claimed. The cached package declares MIT and MSRV 1.84.0.

Subsequent inspection of legal document names found
`COPYING.EIGEN.MPL2`, `COPYING.LAPACK.BSD`, `COPYING.SUITE_SPARSE.AMD.BSD` and
`COPYING.SUITE_SPARSE.COLAMD.BSD`. The MPL-2.0 header and the AMD BSD-3-Clause
header were read; the README mentions a ported SVD algorithm. No Eigen,
LAPACK, SuiteSparse or faer code was opened to determine that scope.
A crate's permissive metadata alone does not resolve this discrepancy.

## Types and implementation decisions

1. `DMat` retains dimensions, private row-major storage and finite data.
   `SymMatrix` requires a square matrix and exact symmetry. Its constructors/accessors
   retain the previous signatures. `SparsityPattern` stores unique sorted
   coordinates; it retains `new(n, Vec<bool>)` and `entries() -> &[bool]` through an
   on-demand mask in OnceLock. Equality only compares order/coordinates,
   regardless of the cache. `from_coordinates` avoids the initial mask;
   n² must fit in usize to keep the inherited accessor infallible.
2. `SparseMatrix` is own general CSR. Assembly uses BTreeMap, adds by
   coordinate in arrival order and then emits columns sorted by row.
   It does not remove zeros or automatically mirror triplets. Memory O(filas+nnz);
   explicit dense expansion only for small problems. Non-finite sums
   and invalid triplets return ENG-E-0008/0010.
3. `LinAlgBackend` returns owned factors as objects of the own trait.
   Factorization does not depend on an RHS or retain a borrow of the matrix/backend.
   Several columns are solved with the same factor. `solve_in_place` writes
   only after validating all output. No public faer,
   nalgebra or sprs types are created.
4. Minimal reference explicitly authorized by the requester, as a testing exception
   to the §10/ADR preference for library algorithms: general elimination
   with row pivoting, symmetric LDLᵀ with 1×1/2×2 blocks, mass Cholesky
   and Jacobi. No production performance or sparse solver/AMD is advertised.
5. Existing codes are reused: ENG-E-0006 singularity with EntityRef::Dof;
   ENG-E-0007 iteration limit; ENG-E-0008 non-finite input/output; ENG-E-0010
   dimensions/symmetry/indefinite mass. The stub uses ENG-E-0009 to express that
   the required backend is unavailable. The catalog was not modified.
6. Each matrix is scaled by its largest absolute entry before elimination.
   Pivots ≤ 64 ε max(n,1) are classified as numerically singular; no physical
   tolerance or regularization is applied. The DOF identifies a pivot of the
   mechanism, not all its movements. Node/component require the future assembler
   map. FactorInfo of a successful factor has empty singular_dofs;
   a singular failure returns the error entities and not a solvable factor.
7. `sqrt`/`hypot` are f64 standard library operations; `cos` is only
   used in the test's analytical expected value. No libm dependency was added. Bit-for-bit
   reproducibility between targets and the transcendental function policy
   remain PENDING under ENG-CORE-010; that property is not promised by absence
   of threads. Libm appeared as a transitive dependency in the faer evaluation,
   but the final graph contains no external dependencies.

## Derivaciones propias

General elimination subtracts a multiple of the pivot row from each lower row
to nullify its column, applying the same operation to all RHS; then
solves by back substitution. Swapping rows does not change the solution if
loads are also permuted. This derives the dense solve without needing a
third-party code listing.

For a symmetric partition A=[[B,Cᵀ],[C,S]], write
A=L diag(B,S−CB⁻¹Cᵀ) Lᵀ with L=[[I,0],[CB⁻¹,I]]. Pivot B is scalar or 2×2.
In the block [[d,e],[e,g]], det=dg−e² and B⁻¹=(1/det)[[g,−e],[−e,d]]. Its
values are (d+g)/2 ± hypot((d−g)/2,e). The maximum absolute diagonal is chosen
if it dominates the off-diagonal maximum by a factor of at least 1/2; otherwise
the pair with the largest off-diagonal term is permuted into a block. Rows/columns
are updated together, as are the permutation and the already constructed columns
of L. An invertible congruence preserves inertia, so the signs
of those blocks give the count (positive, negative, zero). Factors with zero values
are not provided: singularity is returned with the index permuted
back to the original DOF. To solve, b is permuted and scaled, substitution is performed
in L, D blocks are inverted and substitution is performed in Lᵀ.

For positive definite mass, M=LLᵀ is obtained by equating terms of the lower
triangle: Lii=sqrt(Mii−ΣLik²) and Lij=(Mij−ΣLikLjk)/Ljj. With
φ=L⁻ᵀq, Kφ=λMφ is equivalent to symmetric Cq=λq, C=L⁻¹KL⁻ᵀ. Each orthogonal Jacobi
rotation Q eliminates the largest off-diagonal term; vectors are accumulated
and the spectrum of QᵀCQ is preserved. It stops at 4 ε max(n,1) in the
scaled matrix, with up to 100n² rotations. Finally values are sorted,
q is transformed to φ, normalized by sqrt(φᵀMφ) and its dominant sign is set.
No particular basis is required within a subspace with multiplicity.

For T=tridiag(−1,2,−1), q_i=sin(iθ) with θ=kπ/(n+1), q_0=q_(n+1)=0.
The identity sin((i−1)θ)+sin((i+1)θ)=2cos(θ)sin(iθ) proves
Tq=(2−2cosθ)q. It is the classical tridiagonal Toeplitz eigenvalue formula;
the test uses this own derivation, with precise bibliographic citation PENDING.
For M=cI, values are divided by c and vectors normalized with that mass.

The spring chain with unit uniform load satisfies
−x_(i−1)+2x_i−x_(i+1)=1, zero endpoints. Substituting x_i=i(n+1−i)/2 proves the
exact solution. For nondiagonal mass [[2,1/2],[1/2,1]] and K=diag(4,3),
det(K−λM)=12−10λ+(7/4)λ²; its roots are 12/7 and 4. Values,
residuals and mass orthogonality are checked. K=3M is also used for multiplicity
of two, checking the subspace without requiring specific vectors.

## Decision on faer, licenses and MSRV

An actual optional dependency on faer =0.24.4 was evaluated with
`default-features=false`, `features=["std","sparse-linalg"]`, without rayon.
Crates.io failed due to DNS on all retries; official web queries
also failed. The local cache allowed resolution and compilation. The toolchain
at the start was stable Rust 1.99.0; during the session 1.85.1 became available
as default, together with the wasm target. rustup configuration
was not changed. +1.85.1 was explicitly selected to verify builds/tests and +stable
for fmt/clippy, whose components are not installed in 1.85.1.

The experimental adapter used faer multiplication/LU/LBLᵀ and a Cholesky
congruence followed by SelfAdjointEigen. B_diag and B_subdiag allowed counting
signs, including 2×2 blocks. That same generic suite ran against reference
and faer and passed with 1.85.1. The first empty test detected a faer panic in
order-zero eigenvalues; it was handled explicitly before calling the
library. wasm compilation with 1.85.1 was also verified. wasm was not executed
in Node and performance, size or cross-target determinism is not measured.

The evaluation Cargo.lock had 67 external packages, including unused
platforms. Filtered metadata/tree gave 54 for Linux x86_64 and 48 for wasm;
the declared expressions allow MIT, Apache-2.0, BSD-2-Clause, Zlib and
Unicode-3.0. Cargo-deny 0.20.2 accepted both graphs: zero errors, four warnings
for allowlist permissions not found. There is no deny.toml in the repo, so a
temporary config in /tmp was used with MIT/Apache-2.0/BSD-2-Clause/
BSD-3-Clause/ISC/Zlib/Unicode-3.0/Unicode-DFS-2016. The first direct workspace
check failed due to missing license fields in its four local crates;
neither metadata nor legal text was edited. It was repeated with a temporary private
project and publish=false depending only on faer configured exactly.
The sets (name, version, source, license) of all external dependencies of
both projects were compared for each target: identical. That check
isolated third-party policy without allowing additional licenses.
Other targets of the experimental lock were not audited; cache was missing, for example
for crunchy, and there was no network. Global success is not attributed to cargo-deny.

Review of COPYING files detected Eigen's MPL-2.0 notice, outside
the allowed list; the README mentions a ported SVD. The scope could not be
clarified through accessible official documents. **Faer remains PENDING and
neither its dependency nor functional adapter is included in the final graph**, using the
stub/non-inclusion alternative authorized in the absence of network. The success of
cargo-deny on metadata is not presented as certification of exclusively
permissive licensing. Reincorporating it requires resolving the scope of
that notice through documentation and repeating the exact configuration and conformance; do not enable defaults
or raise MSRV as a shortcut. The final feature `faer=[]` documents/reserves a stub
whose constructor explicitly fails with ENG-E-0009. It does not implement LinAlgBackend
or silently fall back to the reference backend.

The workspace MSRV remains **1.85**. The final graph contains only the four
own crates, both with default features and with all features;
final Cargo.lock/tree contain no external dependencies or faer/nalgebra/
sprs/rayon. Nalgebra PENDING, without dependency or feature. Sprs-ldl excluded.
ENG-LA-008 retains sparse solver, AMD and node/DOF map; ENG-LA-009 retains
sparse modal solver, reduction of massless DOFs and control of missing modes.
Interchangeability of two implementations was demonstrated in the temporary
evaluation; the final suite runs only the reference, since the adapter
whose licensing could not be resolved is not distributed.

## Tests and tolerances

The function `conformidad(&mut impl LinAlgBackend)` covers:

- A·A⁻¹=I and nonsymmetric dense solution: absolute tolerance 1e-12 and relative
  Euclidean residual per RHS strictly < 1e-12.
- Transposition and rectangular multiplication with exact integers; CSR assembly,
  duplicate sum, pattern and solution after explicit expansion.
- Springs n=8, closed-form solution x_i=i(9−i)/2 for loads 1 and 2 and zero RHS;
  absolute comparison 1e-12 and relative residual < 1e-12. Zero RHS uses absolute
  norm < 1e-12. Reuse of factor with eight new RHS and constant
  metadata; incorrect dimensions/overflow preserve the entire RHS.
- Inertia of known diagonal, [[0,1],[1,0]], known LDLᵀ congruence and a
  permutation with a noninitial block: exact counts, ratio in (0,1], residuals
  < 1e-12. Singular matrices are rejected with DOF index, including a mechanism
  [[1,−1],[−1,1]]. Scales 1e-100 and 1e100 have residual < 1e-12.
- Toeplitz n=8 with M=I and M=2I: eigenvalues and products φᵀMφ with absolute tolerance
  2e-12, relative modal residuals < 1e-12. Proportional and nonproportional
  nondiagonal mass: values, residual and orthogonality with 1e-12. Zero K allows
  zero values; empty order preserves dimensions.
- Negatives: nonsymmetric, nonsquare, NaN and infinities, overflow, invalid
  indices, incompatible dimensions, duplicates whose sum overflows, singular
  mass and indefinite mass, including an invertible mass with zero diagonal.
- Sparse storage of order 100000 with a single entry, without materializing
  its dense mask; pattern validation and cache-independent comparison.

The boundary test scans all Cargo.toml and .rs files in crates/ (plus the
root manifest), except engine-linalg. It detects providers, dependency aliases,
target/workspace dependency tables, Rust paths and extern crate; it ignores
comments and strings. Includes own scanner control cases. It is a
conservative token inspection, not a full TOML/Rust parser; the current
workspace layout is under crates/. No workflow was modified.

## Final verification

Commands and results of the delivered state, with binaries from ~/.cargo/bin:

- `cargo +1.85.1 build --offline --locked -p engine-linalg`: successful.
- The same build with `--features faer`: successful **for the stub**, without actual faer.
- `cargo +stable fmt --check`: successful.
- `cargo +stable clippy --offline --all-targets -- -D warnings`, and with
  `--features faer`: successful.
- `cargo +1.85.1 test --offline --locked`, and with `--features faer`:
  successful: 47 tests and 26 doctests without features; 48 and 26 with the stub.
- `RUSTDOCFLAGS='-D warnings' cargo +1.85.1 doc --offline --no-deps`, and with
  `--features faer`: successful.
- `cargo tree --offline --locked --workspace --all-features`: only local crates.
- Final build with Rust 1.85.1 for wasm32-unknown-unknown and `--features faer`:
  successful for the stub; no tests were run in a wasm runtime.
- `bash clean-room/check.sh origin/main`: successful, including new
  files through temporary index and objects in /tmp, without modifying .git.
- `git diff --check`: successful; real index unchanged and protected paths
  without modifications.

All actual faer results described above belong exclusively
to the temporary evaluation discarded from the graph. clean-room/**,
.github, CODEOWNERS, docs/CLEAN_ROOM.md, LICENSE files/legal text and the ADR were not modified.
There are no new external dependencies in the delivery, commits or changes to the
user's index. Human review, remote CI and wasm execution remain pending.

## Corrections after J4's clean-room review

**Clean-room reviewer:** J4 (report on PR #18: 3 medium and 2 low findings
according to log `cx-engine-pr18.txt`; the assignment mentioned 1 medium and 4 low).
Branch `j2/la-007-j4-findings`. All corrections are own, derived
from the cases described by J4 and elementary arithmetic; no code from
faer, nalgebra, Eigen, LAPACK or OpenSees/CalculiX was read.

| Finding (J4) | Correction | Test |
|---|---|---|
| Medium: `DMat::zeros(1, usize::MAX/8)` and CSR with enormous rows cause a capacity panic (matrix.rs:31, sparse.rs:47) | `alloc_vec`: `checked_mul` in bytes, limit of 8 GiB per allocation and `try_reserve_exact`; ENG-E-0010 with diagnostic (elements, bytes, limit) | `dense_extreme_dimensions_*`, `sparse_extreme_dimensions_*` |
| Medium: RHS ≈1e308 was divided by the scale of A and overflowed (reference.rs:48, :264) | Independent scale per RHS column; final rescaling with `rescale` (avoids avoidable intermediate overflow). If the solution is not representable, ENG-E-0008 | `solve_dense_rhs_near_f64_max_is_finite`, `solve_dense_columns_scale_independently`, `ldlt_rhs_near_f64_max_is_finite`, `unrepresentable_solution_reports_eng_e_0008` |
| Medium: 2×2 block lost `1e-10·1e-320` to underflow (reference.rs:278) | Block normalized by its largest entry before solving; RHS divided, not multiplied. Documented limitation: resolution of subnormal inputs is that of f64 (~0,05 % at 1e-320) | `ldlt_block_2x2_keeps_subnormal_rhs` |
| Low: singular mass with negative diagonal gave ENG-E-0010 or ENG-E-0006 depending on DOF order (reference.rs:330) | Defined precedence: singularity (ENG-E-0006) over indefiniteness (ENG-E-0010) on every Cholesky pivot failure; the responsible DOF is the one with the lowest original index among the remaining null ones | `singular_mass_error_is_independent_of_dof_order` (all 6 permutations and cases with several null DOFs) |
| Low: ADR-002 l.259 vs l.264 on MPL-2.0 | Unified wording: Eigen's MPL-2.0 notice is inside the faer package; the condition (license review and Jonathan's approval before being the default backend or distributed commercially) is retained; no legal conclusion | text review |

The table's line references (reference.rs:48/:264/:278/:330, etc.) are those of the original code reviewed by J4, before this correction. In the singular 2×2 block the responsible DOF is the lowest original index of the pair.

**Finding not found:** the assignment mentioned a fourth low finding that the available
J4 report does not contain (there are only 2 low findings); it remains unidentified.

**Residual limitation:** `SparsityPattern::entries()` materializes an
O(orden²) mask without possibility of error (signature `&[bool]`); the API was not changed.

Local verification: see the PR (fmt, clippy, tests, doc, clean-room).
