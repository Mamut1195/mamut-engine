# Clean-room log · ENG-LA-009

- Ticket / PR: ENG-LA-009, IMPLEMENTABLE part «Autovalores generalizados K φ = λ M φ»; branch found `j2/eng-la-009`. No commit or push.
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil), identification requested by the user; this session executed by Codex (GPT-6). No delegation to other agents.
- Role: implementer and author of own analytical derivations.
- Implemented contract: internal API `lowest_modes(k, m, count)` as a default method of `LinAlgBackend` and free function; `LowestModes`, `sturm_count` and `symmetric_inertia`. In-house reference backend; no change to the external contract or connection to MemoryBudget (#38/#50).
- Permitted sources used: no external sources. Existing in-house contracts and code of `engine-linalg`, `engine-elements-frame`, `engine-core` catalog, `docs/CLEAN_ROOM.md` and `provenance/TEMPLATE.md`; conditions and β values provided by the user.
- Own derivations: analytical notes in this log; implementation in `crates/engine-linalg/src/modes.rs` and LDLᵀ extension in `reference.rs`; analytical fixtures in `engine-linalg/tests/modes.rs` and `engine-elements-frame/tests/modes.rs`.
- Third-party code copied: none.
- Validation: exclusively analytical; exact mass–spring chain, spectra of small matrices, Euler–Bernoulli beam vibration with mesh refinement.
- Oracle comparison (sibling repo, informative): no. No network access, external sources or execution of external engines.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'
- Clean-room reviewer: PENDING (J4)

## Scope and pending decisions

The existing dense path is preserved, Cholesky of M and Jacobi for the full
spectrum, then selecting the first columns. An independent check by
LDLᵀ congruence of K−σM is added. The default method is compatible
with earlier implementations of the trait and also works through
`&mut dyn LinAlgBackend`. No sparse solver, partial iteration or changes to
memory accounting. No massless DOF is automatically eliminated.

**PENDING (J0): any new external dependency requires his decision.**
No dependency is added, external or local. **Faer PENDING / unused**:
it is among the libraries that are not consulted in this session by instruction
of the user, in addition to the protocol in `docs/CLEAN_ROOM.md`; none of
the other libraries in the declaration is consulted either. Its feature stub and
the historical ENG-LA-007 evaluation are preserved. A potential J0 decision
is not replaced by approval of this in-house implementation.

To verify beams with the real element, the integration is placed in the crate
`engine-elements-frame`, which already depends on `engine-linalg` and `engine-contracts`.
This avoids adding frame as a reverse dev-dependency or modifying manifests
or lockfile. The tests directly exercise the new linalg API.

«los tres tickets» is interpreted as ENG-LA-007, ENG-LA-008 and ENG-LA-009;
an additional declaration from this session is recorded in the two earlier ones,
without replacing their historical sources or results.

## Derivation of the Sturm count and inertia

If M=LLᵀ is positive definite, C=L⁻¹KL⁻ᵀ is symmetric and
K−σM=L(C−σI)Lᵀ. By invertible congruence, the number of negative signs
of K−σM is that of λᵢ−σ<0: the number of eigenvalues strictly less than σ.
LDLᵀ with symmetric permutations and positive diagonal equilibration preserves that
inertia. A chain of principal-minor determinants is not used,
as it may fail without pivoting; counting is done in D by blocks.

A scalar pivot d<0 contributes one. For B=[[d,e],[e,g]], the roots satisfy
μ²−(d+g)μ+(dg−e²)=0. If det<0 there is one positive and one negative; if det>0
both signs are that of tr=d+g. If det=0, one root is zero and the other tr;
if det=tr=0, both are zero. Thus it is incorrect to count only
the diagonal entries of a 2×2 block. Classification by
determinant/trace is implemented; the magnitudes
of tr/2±hypot((d−g)/2,e) are preserved for the pivot indicator.

The in-house pivot selection uses the diagonal maximum if it dominates the
off-diagonal maximum by at least 1/2. Otherwise, |d|,|g|<|e|/2 implies
dg−e²<0, so the selected 2×2 block is indefinite and nonsingular.
A rank-one semidefinite block is reduced through a 1×1 pivot, leaving
a zero remainder. The inertia path accepts that remainder when all its entries
are ≤64 ε max(n,1) after equilibration and normalization; it counts zeros and terminates.
The path that provides factors for solving continues to reject singularities
with ENG-E-0006. The incomplete factor from the inertia path is never provided.
The count strictly at a root is interpreted at that numerical resolution:
an exact sign near roots in ill-conditioned problems is not promised.

## Missing modes and zeros

For count>0 a full, sorted dense spectrum, n×n dimensions,
finiteness, positive definite M and absence of significant negative λ are required.
Then Sturm(σ)=count is checked. σ=λ_count(1+1e−7) is used, with floor
λ_count+16·tol_cero and tol_cero=128 ε n max|λ|. With an entirely zero spectrum
σ=1e−7 is used. The floor prevents evaluating σ=0 for rigid motions;
it does not regularize the matrices and scales inversely with a uniform scaling of M.

If the count does not match, ENG-E-0007 is returned with «MODOS FALTANTES» and
the counts and shift. This also rejects count within a multiple
cluster (or very close within the margin); requesting the full cluster
allows it to be returned. One test simulates a backend that omits the first value
and repeats the last while preserving size: the independent check detects it.
Another simulates a spectrum of length less than n. Fewer modes are not returned.

Values |λ|≤tol_cero are represented as zero and counted in `rigid_mode_count`.
They may be rigid motions or mechanisms: the matrices alone do not allow
deciding whether they correspond to a permitted rigid motion or incorrect connectivity/
releases. This is explicitly documented in the result;
in a static analysis a mechanism corresponds to ENG-E-0006. Semidefinite
K is not rejected, a requirement of this delivery. λ<−tol_cero is rejected with
ENG-E-0010, a diagnostic of indefinite K/possible instability, without producing
imaginary frequencies. Singular mass: ENG-E-0006; indefinite mass,
incompatible orders or count>n: ENG-E-0010. count=0 returns n×0 without
factorization or checking positivity of M, validating the orders.
ENG-E-0008 covers nonfinite inputs or results.

Normalization reuses φ/√(φᵀMφ), with positive dominant sign. When changing
M→cM, λ→λ/c and φ→φ/√c: the mode shapes are preserved. The zero
tolerance is numerical, relative to the full spectrum; it does not separate by a physical criterion
small from large eigenvalues in extremely ill-conditioned spectra.

## Analytical solutions of the tests

Chain of n equal masses between restrained ends, springs k and mass m:
qⱼ=sin(jθ), θ=rπ/(n+1), q₀=qₙ₊₁=0. Adding neighboring sines,
−qⱼ₋₁+2qⱼ−qⱼ₊₁=2(1−cosθ)qⱼ=4sin²(θ/2)qⱼ, hence
λᵣ=4(k/m)sin²(rπ/[2(n+1)]). Tests with n=1,4,9, relative tolerance
2e−12 in λ and absolute M-orthogonality 2e−12. Relative modal residual
2e−11, with an absolute floor for zero modes. Exact Sturm count in
intervals between roots; diagonal matrices and 2×2 blocks also verify
strict equality at roots, multiplicities and singular inertia.

Homogeneous beam: EI w''''+ρA ẅ=0. For w=W(x)e^{iωt},
W''''=β⁴W, β⁴=ρAω²/EI, solution combining cos, sin, cosh, sinh.
Simply supported: W=W''=0 at both ends ⇒ βᵣL=rπ.
Cantilever: W=W'=0 at x=0 and W''=W'''=0 at x=L ⇒
cos(βL)cosh(βL)+1=0. Roots indicated by the user (1.8751, 4.6941,
7.8548), preserved with more digits to prevent their rounding from dominating the error.
Frequency fᵣ=(βᵣL)²√(EI/(ρA))/(2πL²).

Consistent K and M of the in-house EulerFrame2D element are assembled for
E=ρ=A=I=L=1; only uy/rz are selected because the horizontal beam
decouples bending and axial response. Simply supported restrains uy at both ends;
cantilever restrains uy/rz at the first. Meshes of 4 and 12 Hermite elements;
the first three relative errors in f must improve at least ×10 and remain
<3e−4 in the fine mesh. Neither rotary inertia nor shear deformation is mixed
with this Euler–Bernoulli solution. These are mesh convergence tests,
not tests of element accuracy on a finite mesh.

## Verification

PENDING completion after running the requested commands; local/cached
dependencies are used, without network access (`CARGO_NET_OFFLINE=true`). wasm is not run:
the requested check verifies compilation for that target.
