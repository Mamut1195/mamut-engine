# engine-linalg · ENG-LA-007

Definitive matrix types and internal numerical boundary. `engine-contracts`
re-exports `DMat`, `SymMatrix`, and `SparsityPattern` with their previous signatures.
Types contain no external library identifiers. The crate depends only
on `engine-core` and the standard library.

`LinAlgBackend` provides multiplication/transposition, general dense solution,
triplet assembly, reusable symmetric factorization, and all
modes of a dense generalized problem. `SymFactorization` owns its factors;
`solve` and `solve_in_place` take matrices with one RHS per column and do not
refactorize. A failed write preserves the original RHS.

Dense matrices store finite row-major data, including empty
dimensions; symmetric matrices require squareness and exact symmetry. No physical
tolerance is introduced to accept asymmetric inputs. `SparseMatrix` stores CSR,
sums duplicates in input order, and preserves structural zeros. Each triplet
represents one entry; the caller supplies both halves if symmetry is required.
`SparsityPattern` stores sorted coordinates, with a compatible mask on
demand. The legacy `entries()` call materializes O(n²) memory; new code
must use `coordinates()`. The order must allow n² to be representable in
`usize` to preserve that API. `to_dense()` is also intended for small
problems: sparse assembly does not perform an implicit dense conversion.

## Reference backend

`ReferenceBackend` is the second minimal reference/test backend authorized
for this ticket. It uses row-pivoted elimination for general systems,
LDLᵀ with symmetric scalar pivots or 2×2 blocks for symmetric systems,
mass Cholesky and Jacobi rotations for eigenvalues. It is not intended to replace
the main production backend. Elimination uses O(n³) work and O(n²)
memory; searching for the off-diagonal maximum in each Jacobi rotation may
cost O(n⁴) at the iteration limit. There is no AMD or parallelism.

Inertia is obtained by congruence of D, counting signs of each pivot
and the two eigenvalues of each block. The `min_pivot_ratio` indicator uses the smallest
magnitude of those values divided by the largest. It does not estimate the condition number.
Pivots ≤ 64 ε max(n,1), after normalization by the largest matrix entry, are
treated as numerical singularity. ENG-E-0006 is returned with the pivot's global
DOF index; identifying the node or mechanism motion requires the
assembler map, pending ENG-LA-008. A successful factor has
empty `singular_dofs`; singular factors are not provided for solving.

The `generalized_eigen` primitive requires positive definite mass. Singular mass
returns ENG-E-0006 without generating NaN; indefinite mass ENG-E-0010. Explicit
massless DOF reduction is in `guyan_condense`; matrices are not regularized. Eigenvalues are ascending; column vectors are normalized
with φᵀMφ=1 and positive dominant component sign. A multiplicity allows
different bases of the same subspace. Jacobi stops when the off-diagonal
maximum of the scaled matrix is ≤ 4 ε max(n,1), with up to 100 n² rotations;
exhausting that limit returns ENG-E-0007.

## Decision on faer and pending work

**Faer PENDING; the `faer` feature only reserves a stub.** The constructor
`FaerBackend::new()` returns ENG-E-0009 with the reason. It does not implement the numerical
trait or redirect operations to the reference backend.

Faer **0.24.4** was tried, pinned exactly, optional, without default features,
with `std` and `sparse-linalg`, without `rayon`. Crates.io failed due to DNS, but the cache
allowed evaluating a real adapter: it compiled and passed the same generic
conformance function as the reference with Rust **1.85.1** on native. It also compiled for
`wasm32-unknown-unknown` with that compiler. Its native graph of 54 crates and
wasm graph of 48 were accepted by cargo-deny, according to SPDX metadata, with a temporary
permissive configuration outside the repo.

Inspection of package documents found `COPYING.EIGEN.MPL2` and a README
mentioning a ported SVD procedure. The manifest declares MIT. The scope of
the MPL-2.0 notice could not be clarified through accessible official documentation;
MPL-2.0 is outside the allowlist. Therefore the package is not certified as
exclusively permissive and its dependency is not included in the final graph.
This decision uses the PENDING alternative authorized for lack of network access; the
MSRV has not been raised and the ADR has not changed. See the ticket work log for
evaluation results and their limits. The successful compilation test
of the final feature verifies the stub, not a functional faer adapter.

Reintroduction requires resolving that documentary scope and retesting
the optional dependency with the exact previous configuration and the same suite.
The direct sparse solver with AMD and symbolic/numerical separation remains in
ENG-LA-008. Sparse eigenvalues remain pending; massless DOF
condensation is implemented in ENG-LA-009 (#184).
Dense Sturm and missing-mode checks are implemented in ENG-LA-009.
**Nalgebra PENDING**, without a dependency
or feature in this delivery. Sprs-ldl remains excluded.

ENG-CORE-010 centralizes scalar functions in `engine_core::math`, with
libm 0.2.16 (MIT) pinned and approved by Jonathan on 2026-10-14. `hypot` and the
trigonometric test reference values use libm; `sqrt` retains std's correct IEEE-754
rounding within the module. Policy and native/wasm bit cross-check in
[docs/DETERMINISMO.md](../../docs/DETERMINISMO.md).

## Dense lowest modes · ENG-LA-009

`backend.lowest_modes(&k, &m, count)` and the free function `lowest_modes` return
`LowestModes`: ascending eigenvalues, mass-normalized columns, and
`rigid_mode_count`. These methods are compatible with existing backends: they reuse
`generalized_eigen`, followed by an independent Sturm count through LDLᵀ.
No dependency is added and MemoryBudget is not connected (#38/#50).

`sturm_count(&mut backend, &k, &m, sigma)` counts λ<σ through the negative inertia
of K−σM; M is validated as positive definite. `symmetric_inertia` queries the factor
by default; the reference reuses LDLᵀ allowing a zero remaining block
only for counting, without exposing singular factors for solving. A negative 1×1
pivot counts one. In 2×2, det<0 counts one; det>0 counts two if the trace is
negative, zero if positive. With det=0, zeros are excluded and the trace is used
for the remaining sign; the reference chooses a 1×1 pivot when the block is
semidefinite, and 2×2 blocks chosen for off-diagonal dominance have
det<0. Inertia is preserved by congruence and positive diagonal balancing.

The completeness check requires Sturm(σ)=count, with σ=λ_count(1+1e−7), and a floor
λ_count+16·tol_cero for zero or very small values. The numerical zero
tolerance is 128 ε n max|λ|; for entirely zero K, σ=1e−7 is used. This is not a physical
tolerance. A cut within a multiple eigenvalue or cluster separated by less than
that margin returns **ENG-E-0007: MISSING MODES / multiplicity cut**;
the entire group must be requested. A lower count also returns an error.
Neither K nor M is modified to pass this check. Pivots near a root are
classified with the LDLᵀ tolerance, not unlimited mathematical exactness.

Semidefinite K is valid. Modes with |λ|≤tol_cero are supplied as λ=0 and
identified as rigid motions or possible mechanisms. Distinguishing them
requires model supports, connectivity, and releases; ENG-E-0006 corresponds
to those mechanisms in static analysis. λ<−tol_cero returns ENG-E-0010 with
an indefinite stiffness/instability diagnostic. Singular mass returns
ENG-E-0006; indefinite mass, count>n, or different orders, ENG-E-0010.
count=0 returns n×0 without factorization, even if mass is singular.

Closed-form chain tests are in `tests/modes.rs`; simply supported and
cantilever Euler beams are assembled using the internal element in
`engine-elements-frame/tests/modes.rs`. Locating tests there uses its
existing dependencies and avoids adding a cycle between crates. Refinement
from 4 to 12 elements improves error in the first three frequencies at least
10 times; final relative tolerance 3e−4. Derivations, limits, and verification:
`provenance/ENG-LA-009.md`.

## Reproducible verification

```sh
~/.cargo/bin/cargo +1.85.1 build --offline --locked -p engine-linalg
~/.cargo/bin/cargo +1.85.1 build --offline --locked -p engine-linalg --features faer
~/.cargo/bin/cargo +stable fmt --check
~/.cargo/bin/cargo +stable clippy --offline --all-targets -- -D warnings
~/.cargo/bin/cargo +stable clippy --offline --all-targets --features faer -- -D warnings
~/.cargo/bin/cargo +1.85.1 test --offline --locked
~/.cargo/bin/cargo +1.85.1 test --offline --locked --features faer
RUSTDOCFLAGS='-D warnings' ~/.cargo/bin/cargo +1.85.1 doc --offline --no-deps
RUSTDOCFLAGS='-D warnings' ~/.cargo/bin/cargo +1.85.1 doc --offline --no-deps --features faer
```

The generic suite takes `&mut impl LinAlgBackend`; it only runs against the
reference in the final graph. The temporary evaluation ran against both. Expected
values are identities and closed-form solutions: relative Euclidean residual
< 1e-12 for each nonzero RHS, inverse with absolute tolerance 1e-12, Toeplitz
eigenvalues, and mass orthonormality with 2e-12. See the independent derivation of the
classical Toeplitz formula in `provenance/ENG-LA-007.md`; precise bibliographic citation
PENDING. The boundary integration test scans Cargo.toml and Rust files
of workspace crates, detecting direct providers and aliases,
without creating or modifying workflows.

## Guyan static condensation · ENG-LA-009 (#184)

`guyan_condense(backend, &k, &m)` operates on matrices already transformed by
supports and diaphragms. Returns `None` if all rows have mass, preserving
the previous numerical path. Only exactly zero rows/columns are considered
massless: small positive mass and couplings are preserved.

`GuyanReduction` contains Kred=Kmm−Kms Kss⁻¹ Ksm, Mmm, and the recovery
φs=−Kss⁻¹ Ksm φm. Kss is factorized once with the internal LDLᵀ, checking
inertia (s,0,0); all RHS are solved together. `recover` preserves the original
order and mass norm. Assembly/modal integration also recovers
diaphragm slaves before participation, effective mass, and spectrum.

**ENG-E-0014: Massless mechanism** identifies singular or non-positive-definite Kss,
with DOFs of the involved block (assembly adds the node and global component).
Negative mass or absence of free DOFs with mass yields ENG-E-0006; no artificial
mass or stiffness is added. Existing checkpoints and step budgets
are retained. Dense reduction, like the eigensolver, is indivisible.
Derivation, theoretical source, and checks in
[provenance/ENG-LA-009-GUYAN.md](../../provenance/ENG-LA-009-GUYAN.md).
