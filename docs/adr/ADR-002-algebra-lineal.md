# ADR-002: selection of linear algebra libraries

- **Task:** ENG-LA-006
- **Status:** Accepted (2026-10-11): approved by J0 (program lead); revocable by Jonathan
- **Date:** 2026-10-10 (America/Guayaquil)

## Context

The engine needs dense algebra for element matrices, sparse storage
for assembly, symmetric factorization for definite and
indefinite systems and generalized eigenvalues for modal analysis. The selection must
work natively and on `wasm32-unknown-unknown`, with no threads in V1 or
C/Fortran dependencies in the wasm build. The [clean-room protocol](../CLEAN_ROOM.md) requires
permissive dependencies and documented provenance.

The reference specification is
`/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`, §§4.5–4.6, 9 and 10,
together with `00_PROYECTO.md` and tickets ENG-LA-006/007/008/009 from
`04_TICKETS.md` in that same directory. Those documents are external to this
repository. The format follows [ADR-001](ADR-001-fundacionales.md).

This ticket is documentation only: it proposes the selection and the tests for approving it.
It adds no crates, Rust code, license configuration or workflows.

## Evidence and comparison

The following crates.io metadata corresponds to the **2026-10-10 snapshot
verified and provided by the requester**. Downloads are approximate
totals for the crate, not for that version or a measure of quality. In this
session it was not possible to repeat the query: web access to crates.io failed and
the direct API query failed due to DNS resolution. The links identify
the snapshot sources; no new remote verification is claimed.

Compilation evidence comes exclusively from the disposable test on
**2026-10-08 recorded in architecture §10**. It was not repeated here. That section
also records a review of LICENSE files for faer, nalgebra and sprs, but
does not provide a reproducible review of the full graph by version in this ADR.

| Candidate and evaluated function | License | `wasm32-unknown-unknown`: compiles / runs | MSRV | Maintenance: available evidence | Performance | wasm size | Determinism |
|---|---|---|---|---|---|---|---|
| [faer 0.24.4](https://crates.io/crates/faer/0.24.4): dense, sparse, symmetric solver with AMD and dense eigenvalues according to §10 | MIT | **Compiles**, according to §10, with `default-features = false`, `features = ["std", "sparse-linalg"]`; fails by default due to `getrandom`. **Runs: PENDING**, V2 | 1.84; compatible with the declared 1.85, subject to the resolved graph (V1) | ~5.0 M downloads; updated 2026-06-24. Maintenance health: **PENDING**, V7 | **PENDING**, V4 | **PENDING**, V5 | No `rayon` proposed; repeatability and equivalence across targets **PENDING**, V6 |
| [nalgebra 0.35.0](https://crates.io/crates/nalgebra/0.35.0): second minimal dense backend | Apache-2.0 | **Compiles**, according to §10; exact configuration to be reproduced. **Runs: PENDING**, V2 | 1.89; incompatible with 1.85 when enabled | ~94.5 M downloads; updated 2026-05-24. Maintenance health: **PENDING**, V7 | **PENDING**, V4 | **PENDING**, V5 | **PENDING**, V6 |
| [nalgebra-sparse 0.12.0](https://crates.io/crates/nalgebra-sparse/0.12.0): alternative sparse storage | Apache-2.0 | **Compiles**, according to §10. **Runs: PENDING**, V2 if reconsidered | No declared MSRV; effective minimum **PENDING**, V1 | ~669 k downloads; update date and maintenance health **PENDING**, V7 | **PENDING**, V4 if reconsidered | **PENDING**, V5 if reconsidered | **PENDING**, V6 if reconsidered |
| [sprs 0.11.5](https://crates.io/crates/sprs/0.11.5): alternative sparse storage | MIT OR Apache-2.0; its LGPL-2.1 solver `sprs-ldl` is **excluded** according to §10 | **Compiles**, according to §10. **Runs: PENDING**, V2 if reconsidered | 1.88; incompatible with 1.85 when enabled | ~8.5 M downloads; update date and maintenance health **PENDING**, V7 | **PENDING**, V4 if reconsidered | **PENDING**, V5 if reconsidered | **PENDING**, V6 if reconsidered |
| [scirs2-sparse 0.6.5](https://crates.io/crates/scirs2-sparse/0.6.5): candidate for generalized sparse eigenvalues | Apache-2.0 | **Compiles**, according to §10. **Runs: PENDING**, V2/V3 | No declared MSRV; effective minimum **PENDING**, V1 | ~42 k downloads; §10 considers it young. Update date and maturity **PENDING**, V7 | **PENDING**, V4 | **PENDING**, V5 | **PENDING**, V6 |
| arpack-ng (bindings): alternative for sparse eigenvalues | BSD-3-Clause, according to §10; bindings version and license of its graph **PENDING**, V1 only if reconsidered | **Rejected** for wasm due to Fortran/C dependency, according to §10; execution untested (**PENDING**, V2 only if reconsidered) | Unverified: **PENDING**, V1 only if reconsidered | Not evaluated: **PENDING**, V7 only if reconsidered | **PENDING**, unmeasured; V4 only if reconsidered | **PENDING**, unmeasured; V5 only if reconsidered | **PENDING**, V6 only if reconsidered |

The comparison supports recommending a configuration based on functional coverage and
declared compatibility. It does not support claims about which library is faster, which
artifact is smaller or whether a numerical result is identical across targets.
V1–V7 identify specific tests in the following plan.

<a id="decisión"></a>

## Decision

1. **Recommend faer 0.24.4 as the primary backend**, with
   `default-features = false` and `features = ["std", "sparse-linalg"]`, without
   `rayon`. Use its sparse storage and symmetric factorization with
   AMD: Cholesky LLᵀ for definite matrices and LDLᵀ/LBLᵀ for indefinite ones,
   according to §10 and ENG-LA-008. Exact API coverage with these features,
   including dense operations and eigenvalues, remains **PENDING** V3.
   If an API requires another feature, document and review the adjustment before
   incorporating it; do not enable default features as an implicit solution.
2. **Recommend nalgebra 0.35.0 as a second minimal dense backend** in
   ENG-LA-007, behind an optional feature disabled by default.
   It will pass the same suite of dense operations and small system solves
   as faer to demonstrate interchangeability. It is not credited with
   coverage of the sparse solver or full modal analysis.
3. **Keep the workspace MSRV at 1.85** for the primary configuration.
   [Cargo.toml](../../Cargo.toml) currently declares `rust-version = "1.85"`;
   nalgebra requires **1.89** and sprs **1.88**. The alternative would be to raise the entire
   workspace to 1.89; this is rejected for now to preserve compatibility of the
   primary path with faer. The nalgebra feature will require Rust **at least
   1.89**, with the effective minimum confirmed against transitive dependencies in V1.
   ENG-LA-007 must document both profiles and test the primary one with 1.85
   and the second with its compatible toolchain. Optional selection does not make
   nalgebra compatible with 1.85; `--all-features` will not have that guarantee.
   This ADR does not modify the manifest.
4. **Define an engine-owned trait `LinAlgBackend` in ENG-LA-007**, with engine
   types for dense matrices, sparse triplets, factorization with `solve`
   and inertia information and generalized eigenvalues. Only `engine-linalg`
   will import the libraries. The `linsolve/1` and `eigen/1` contracts from §§4.5–4.6
   will remain visible through engine-owned types; errors will be translated to
   `ENG-E` with node/DOF context when available.
5. **Keep the inertia exposed by faer PENDING**: confirm access to the
   signs of D, including 2×2 blocks, and their interpretation in V3. The
   `FactorInfo.inertia` field allows information to be absent; no Sturm checking
   or missing-mode detection will be advertised without evidence. No in-house
   factorization will be implemented to cover this uncertainty.
6. **Use faer as the dense candidate for ENG-LA-009**, according to
   `self_adjoint_eigen`/`gevd` cited in §10, subject to V3 verification.
   **Sparse eigenvalue solver: PENDING.** Evaluate scirs2-sparse and the
   interoperability of `eigsh_generalized`/shift-invert with the factorization
   from ENG-LA-008. Compiling that crate does not demonstrate maturity, correctness or that
   it allows reuse of our factorization. It has not yet been selected.

## Rejected alternatives

- **nalgebra as primary:** in this evaluation it does not offer the proposed
  integrated coverage for the sparse solver and requires raising the MSRV. It is retained
  as an optional second dense backend.
- **nalgebra-sparse and sprs as primary storage:** they would add another
  representation and conversions to faer's solver without measured benefit. They may be
  reconsidered with V2/V4/V5/V6; their historical compilation does not determine the selection.
- **sprs-ldl:** excluded due to LGPL-2.1 under the permissive dependency
  policy. Excluding the solver does not affect sprs's permissive license.
- **scirs2-sparse as an immediate choice for sparse eigenvalues:**
  deferred until V1/V2/V3/V7 are resolved. There is insufficient evidence of maturity or
  integration with the required contracts.
- **arpack-ng and builds with C/Fortran BLAS/LAPACK/ARPACK:** rejected for the
  wasm build and the Rust backend requirement. Their permissive license does not resolve
  the dependency on external toolchains and libraries.
- **Default faer features or V1 parallelism:** incompatible with the
  recorded wasm configuration or the no-threads policy. No in-house
  linear algebra is proposed to replace libraries.

## Consequences

- The rest of the engine will depend on the engine-owned contract, with common tests for
  shared capabilities and sparse capabilities verified separately.
  Integration and changes to provisional types from ENG-CORE-007
  belong to ENG-LA-007, not this ADR.
- The nalgebra feature will have a higher toolchain requirement than the primary
  profile. Graph resolution and feature unification must
  verify that it does not activate nalgebra or `rayon` in the primary build.
- The primary selection is provisional until wasm execution, APIs,
  transitive licenses and effective MSRV are validated. There are no verified performance or
  wasm size estimates; budgets remain **PENDING**.
- ENG-LA-008 retains outstanding singularity diagnostics and inertia;
  ENG-LA-009 retains the outstanding sparse solver and missing-mode
  checking. Selecting faer does not declare modal analysis complete.
- No threads, stable assembly order and constant seeds reduce sources of
  variation, but SIMD and FMA may differ across targets. Bitwise
  equality remains **PENDING** in ENG-CORE-010.

<a id="plan-de-verificación"></a>

## Verification plan

These actions will be carried out in the integration and governance tickets; no
numerical tests or candidate builds were performed in this ticket.
Each report must record versions, features, toolchain, target, inputs,
commands and results, with a clean-room log. Only permitted sources
and analytical solutions will be used, without external engines as oracles.

1. **V1 — Licenses and MSRV (ENG-GOV-005 / ENG-LA-007).** Repeat the query
   to crates.io for the exact versions; compare their SPDX expressions with the
   official LICENSE files for those versions and those of the transitive graph,
   without opening engine code. Prepare a `deny.toml` in ENG-GOV-005 with an
   allow-list of MIT, Apache-2.0, BSD, ISC, Zlib and Unicode, using specific
   SPDX identifiers for the BSD/Unicode variants present; reject licenses
   that are unknown or outside that list. Run `cargo-deny check licenses`
   for both profiles and targets, with nalgebra enabled and disabled, including
   scirs2-sparse if it is proposed. Review `cargo tree -e features` and
   `cargo tree --target wasm32-unknown-unknown`: absence of `sprs-ldl`,
   `rayon` and C/Fortran/BLAS/LAPACK/ARPACK dependencies in the wasm profile.
   Build the primary profile with Rust 1.85 and the nalgebra profile with 1.89;
   record any higher transitive requirement. For crates without a declared
   MSRV, determine the effective minimum through builds with pinned
   toolchains. **No `deny.toml` is created in this delivery.**
2. **V2 — Native/wasm compilation and execution (ENG-LA-007/008/009).** Create
   the integration suite when implementing the adapters and compile the
   profiles separately for `wasm32-unknown-unknown`, reproducing the
   faer features from §10. Run the same suite natively and in Node using
   a wasm test runner (`wasm-bindgen-test`, for example with
   `wasm-pack test --node`), recording the Node version and toolchain. Cover
   dense operations, triplet assembly, known SPD and
   symmetric indefinite solutions, singularity and multiple RHS with one factorization.
   Require relative residual < 1e-12 in well-conditioned systems according to
   ENG-LA-008; declare the norm and handling of zero RHS. Run in CI when
   integrating those tickets; **this ADR creates no workflows**. Storage
   alternatives will receive this suite only if reconsidered.
3. **V3 — Functional coverage and inertia (ENG-LA-007/008/009).** Check in
   documentation/API and tests for the pinned version the dense operations,
   symbolic/numerical separation with AMD and factor reuse.
   Verify that the minimal features cover the required dense and sparse
   APIs. Test inertia with matrices of known signs, permutations,
   a singular case and an indefinite matrix requiring a 2×2 block pivot; verify
   that any shift/regularization does not obscure the original inertia.
   If no verifiable API exists, keep inertia absent and document the
   limitation of Sturm checking. For eigenvalues, use diagonal K and M
   with a closed-form solution, singular mass and analytical benchmarks for a simply supported beam
   and a cantilever; check residual `Kφ − λMφ`, normalization `φᵀMφ = 1` and
   multiplicities through subspaces. Evaluate scirs2-sparse with shift-invert,
   shifts near eigenvalues, nonconvergence and missing modes; confirm
   whether it can consume our solve operation without duplicating factorizations.
   Set test case tolerances before evaluating results. Record
   selection or rejection of the sparse solver in a revision of this ADR.
4. **V4 — Performance (ENG-LA-007/008/009 / ENG-VAL-004).** Measure
   reproducible families of element, banded and structural sparse matrices with
   sizes, `nnz`, condition and seeds recorded. Separate assembly,
   symbolic analysis, factorization, solve per RHS and eigenvalues; measure time
   and memory natively and in Node, after warm-up and with repetitions, on the same
   machine. Compare faer/nalgebra only on common capabilities, and other candidates
   if reconsidered. Publish variability and environment; agree on budgets
   before accepting or rejecting on performance grounds. Do not extrapolate from downloads.
5. **V5 — wasm size (ENG-LA-007 / ENG-VAL-004).** Build equivalent release
   artifacts with the same consuming code, LTO, optimization and
   pinned tools; compare the baseline without a backend, minimal faer and the profile with
   nalgebra, also separating any modal candidate. Measure `.wasm` bytes
   and gzip with constant options, retaining commands and hashes. The target of
   **1 MB gzip or less** from `00_PROYECTO.md` is a **PENDING** proposal,
   not a result or an approved budget. Also determine
   model/memory limits; §10 limits V1 to wasm32 without Memory64.
6. **V6 — Determinism (ENG-CORE-010).** Repeatedly run the same
   test cases on each target and cross-check native/Node. Record triplet
   and summation order, backend, features, SIMD, flags and seeds; without threads or enabling
   `-C target-feature=+fma`, with ordered structures and the `libm` policy
   from §9 for scalar functions. Compare bits of scalar outputs; for
   eigenvectors set a sign convention and compare subspaces for repeated
   values. If targets differ, publish differences and justified
   tolerances. Do not promise bitwise equality from disabling `rayon`.
7. **V7 — Maintenance.** For each candidate still under consideration, review
   release history and breaking changes, MSRV policy, documentation,
   published native/wasm tests, issues and maintainer responses. Record
   dates and official documentation links without opening prohibited code. For
   scirs2-sparse, require specific evidence for the generalized solver and cases
   of convergence, singular mass and interoperability. Download counts
   alone do not satisfy this review.

## Acceptance (2026-10-11)

This ADR moves from provisional to **accepted** on 2026-10-11: **approved by J0
(program lead); revocable by Jonathan**. faer is accepted as the primary
backend and nalgebra as the optional second backend. Measured evidence is in
[ADR-002-evidencia.md](ADR-002-evidencia.md).

Still **PENDING** (not covered by this acceptance): sparse matrices
(sparse solver), LDLt, eigenvalues, `simd128`, `wasm-opt`, memory and CI.

Determinism guarantees: with faer **bitwise equality between
native and wasm is not promised**, nor with `rayon` across different thread counts.

Manifest licensing: ADR-001 records the dual AGPL-3.0 + commercial license,
which is not a single SPDX expression; the `license` expression in the `Cargo.toml` files
is a **decision pending Jonathan** and this acceptance does not settle it.

## ENG-LA-006 acceptance criteria

| Ticket criterion | Status | Evidence or remaining work |
|---|---|---|
| Documented evaluation and decision on dense algebra, sparse storage, symmetric solver and generalized eigenvalues | **Met** for the documented comparison and recommendation; sparse solver **PENDING** | Table, decision and alternatives; V3 must resolve the sparse solver |
| License of each crate verified on crates.io and in the official LICENSE, and accepted by `cargo-deny` | **PENDING** | Provided crates.io snapshot; §10 records LICENSE for faer/nalgebra/sprs. Reproducible review by version and full graph, and `cargo-deny` still missing (V1) |
| Each selected crate compiles and runs tests on wasm (Node) and natively in CI | **PENDING** | Historical wasm compilation recorded in §10; execution and integration CI unverified (V2) |
| No C/Fortran dependencies (BLAS, LAPACK, ARPACK) in the wasm build | **PENDING** build verification | Restriction and documented rejection of arpack-ng **met**; graph and build of the selected profiles still missing (V1/V2) |
| Decision approved | **Met** (2026-10-11) | Approved by J0 (program lead); revocable by Jonathan |

## Outstanding items

- [x] Approval (2026-10-11) by J0; revocable by Jonathan. The MSRV recommendation per profile is included.
- [ ] V1: graph licenses, `cargo-deny`, effective MSRV and resolved features.
- [ ] V2: actual test execution natively and on wasm in Node/CI.
- [ ] V3: APIs with minimal features, faer inertia and sparse eigenvalue solver.
- [ ] V4/V5: performance, memory, wasm size and approved budgets.
- [ ] V6/V7: verified determinism and maintenance/maturity review.

The [ENG-LA-006 log](../../provenance/ENG-LA-006.md) records the sources,
scope and verification limits of this session.

## faer licensing condition (ENG-LA-007 finding)

**Condition:** faer (its 0.24.4 package includes Eigen's MPL-2.0 notice in `COPYING.EIGEN.MPL2`; it is not a separate dependency or listed in its crate graph, according to the LA-007 finding) requires a license review and Jonathan's approval before becoming the default backend or being distributed in the commercial offering. Until then faer may only be used behind the Cargo feature `faer`, never by default (ENG-LA-007 leaves the adapter PENDING). This section records technical evidence; **it is not a legal conclusion**.

Evidence (2026-10-11; test crate outside the workspace with `faer =0.24.4`, `default-features = false`, `features = ["std", "sparse-linalg"]`, without rayon):

- `cargo tree` on that crate: the resolved graph consists of crates with no copyleft declared in their manifest (e.g. gemm*, pulp, dyn-stack, num-complex, bytemuck, libm, qd); no crate in the graph declares MPL-2.0 in its `license` field.
- `COPYING.EIGEN.MPL2` is a file **inside the `faer-0.24.4` package itself** (alongside `COPYING.LAPACK.BSD`, `COPYING.SUITE_SPARSE.AMD.BSD` and `COPYING.SUITE_SPARSE.COLAMD.BSD`), not a separate dependency. faer's manifest declares `license = "MIT"`. The file header says «ported algorithms: divide and conquer SVD» followed by the MPL-2.0 text; that is, the notice refers to algorithms ported from Eigen and its exact scope in faer's code is not documented in the manifest.
- faer's SVD modules (`src/linalg/svd/`, including `bidiag_svd.rs`) are part of the `linalg` feature, which `sparse-linalg` enables. Therefore the file is present with the proposed configuration. Exactly which code paths the notice covers or whether dense or sparse Cholesky/LU/LDLᵀ are excluded was not verified: **PENDING** (requires review by faer's rights holder/a lawyer; no implementation code was read for this ADR).
- `cargo deny check licenses` on the test crate: the graph's manifest licenses resolve with the MIT/Apache-2.0/BSD-3-Clause/ISC/Zlib/Unicode-3.0 list; cargo-deny **does not detect** the `COPYING.EIGEN.MPL2` file because it reads the manifest's SPDX expression (and, with confidence, LICENSE files), so a passing cargo-deny result does not rule out the notice. Compare with [ADR-002-evidencia.md](ADR-002-evidencia.md): its cargo-deny results are consistent with this and do not examine the package's `COPYING.*` files.
