# Clean-room log · ENG-ANL-002

## ENG-VAL-001 extension record (2026-10-11)

- Ticket / PR: ENG-ANL-002 (#61), modal validation from ENG-VAL-001;
  test extension, without modifying the modal analysis.
- Author(s): J2 (agent, Grok Bot), requested label; executing Codex based
  on GPT-6 · 2026-10-11 (America/Guayaquil).
- Role: implementer of fixtures and author of original derivations, without delegation.
- Implemented contract: validation of `Modal::run` with public APIs.
- Permitted sources used: request, own workspace APIs/tests,
  `provenance/TEMPLATE.md`, `docs/CLEAN_ROOM.md`; no network or external sources.
- Original derivations: [ENG-VAL-001, D1–D4](ENG-VAL-001.md#ampliación-dinámica-y-2º-orden).
- Third-party code copied: none; no new dependencies.
- Validation: analytical, 12 new fixtures; see count and verifications in
  ENG-VAL-001. No published examples or invented book citations.
- Oracle comparison (sibling repo, informational): no.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

## Previous ticket record

- Ticket / PR: ENG-ANL-002 «Modal analysis», branch `j2/eng-anl-002` stacked
  on ENG-MASS-001 (#55 → #51, unmerged), ENG-ANL-001 (#52, unmerged)
  and the implementable part of ENG-LA-009 (#58, unmerged).
  No commit or push.
- Author(s): J2 (agent, Grok Bot) · label requested by the user;
  executing Codex based on GPT-6 · 2026-10-11 (America/Guayaquil).
- Role: implementer and author of original analytical derivations, without delegation.
- Implemented contract: `modal`, API `engine_analysis::modal::Modal::run`
  over `LinearSystem`; modal results and directional participation.
  Does not adapt the generic trait `AnalysisMethod` (sink/control/settings).
- Permitted sources used: exclusively the user's request and the relevant
  own APIs and code of the workspace: engine-analysis, engine-assembly,
  engine-loads, engine-linalg, engine-core, engine-contracts, engine-model and
  engine-elements-frame; `provenance/TEMPLATE.md` and `docs/CLEAN_ROOM.md`.
  No network, external bibliography, published results or oracles.
- Original derivations: [equations and limits of this note](#derivaciones-propias)
  and [analytical fixtures](../crates/engine-analysis/tests/modal.rs), marked
  `SOURCE` / `CITATION`.
- Third-party code copied: none. One new internal path dependency:
  engine-loads; no new external dependencies.
- Validation: exclusively analytical: shear building, Euler–Bernoulli
  beams, energy identities, completeness and rotation.
- Oracle comparison (sibling repo, informational): no.
- Declaration: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'.
- Clean-room reviewer: PENDING (J4).

## Stacking and previous declarations

The three previous tickets in the context are ENG-MASS-001, ENG-ANL-001 and
ENG-LA-009. Their existing logs contain the full declaration above,
J2 authorship (agent, Grok Bot) and «Revisor clean-room: PENDIENTE (J4)».
This session does not replace their derivations or claim that their PRs are merged.
ENG-MASS-001 (#55) in turn depends on ENG-LOAD-001 (#51). The modal module
consumes those stacked interfaces and the partial delivery #58, without implementing
the pending reduction of massless DOFs or a new solver.

## Interface and decisions

`Modal::run(system, count, kind, directions)` receives count, MassKind and a
nonempty list of explicit directions. Each direction is normalized with
`engine_loads::mass::rigid_translation`. The rotations of r are zero. The
order of the directions is retained; they may be repeated or reversed without error.
There are no code-prescribed factors, implicit gravity or required mass threshold.

`LinearSystem::lowest_modes(kind, count)` uses the system backend with the
matrices K_ff and M_ff that it already reduced from global K and `global_mass(kind)`.
Masses are neither duplicated nor reassembled. It does not change accepted/trial states.
The module renormalizes the shape with the chosen mass before evaluating
participation. It does not use `Constraints::expand`, because that would include settlements:
prescribed modal perturbations are always zero.

Results include λ, ω, f, T, free and global shapes, Γ, M*, cumulative
sums and individual/cumulative fractions. A numerically zero λ
is represented with ω=f=0 and T=None (infinite period), without generating IEEE-754
infinity. The count of rigid modes or possible mechanisms is retained.
A direction with zero global mass has fraction None; there is no division by zero.
Count=0 retains the semantics of lowest_modes: it does not require positive-definite M
or factorize, but validates directions and returns their totals with zero sums.

## Derivaciones propias

### Modal problem, participation and completeness

From M_ff q̈+K_ff q=0 and q=φ exp(iωt) follows K_ffφ=ω²M_ffφ.
For positive-definite M_ff, take φ/√(φᵀM_ffφ), so that φᵀM_ffφ=1.
Then ω=√λ, f=ω/(2π) and T=2π/ω if λ>0.

Projection of the translational excitation r_f onto each shape gives
Γ_n=φ_nᵀM_ff r_f/(φ_nᵀM_ffφ_n). With mass normalization,
M*_n=(φ_nᵀM_ff r_f)². A complete modal basis Φ satisfies ΦᵀM_ffΦ=I;
since Φ is square and invertible, ΦΦᵀ=M_ff⁻¹. Thus:

Σ_n M*_n = r_fᵀ M_ff ΦΦᵀ M_ff r_f = r_fᵀ M_ff r_f.

This identity applies to all n free DOFs with positive M_ff, without eliminating
zero-mass DOFs. It is tested with all modes of the 1, 4 and 9 story building
(both mass kinds) and all axial/bending modes of the two beams.
Completeness of a truncated spectrum is not claimed.

The global mass of the direction is calculated using `engine_loads::mass::total_mass`,
equivalent to `system_total_mass`, over full r and global M: rᵀMr.
It is the denominator of the requested fractions. Supports may have mass
or consistent couplings: in that case rᵀMr need not coincide with
r_fᵀM_ff r_f. `total_mass` and `free_mass` are exposed to make this difference
explicit. The completeness tests set the prescribed components
of r to zero and evaluate the independent product with global M; thus they obtain exactly
r_fᵀM_ff r_f. The building uses mass only on free stories and reaches fraction 1.
The beams and a restrained axial bar check a final fraction less
than 1 without artificially modifying the mass in the denominator.

### Shear building with N identical stories, original derivation

Let q_0=0 at the base, mass m per story and stiffness k per story.
The energy V=½k Σ_{j=1}^N(q_j−q_{j−1})² produces K with diagonals 2k
up to story N−1, last diagonal k and neighboring entries −k. M=mI.
For φ_j=sin(jθ), the interior equation is:

−φ_{j−1}+2φ_j−φ_{j+1}=2(1−cosθ)φ_j=4sin²(θ/2)φ_j.

The last equation is φ_N−φ_{N−1}=(λm/k)φ_N; it may be written as
the interior one by introducing φ_{N+1}=φ_N. The top condition requires:

sin((N+1)θ)−sin(Nθ)=2cos((2N+1)θ/2)sin(θ/2)=0.

θ=0 is excluded (zero shape with restrained base). The N distinct roots are
θ_n=(2n−1)π/(2N+1), n=1,…,N; therefore:

ω_n=2√(k/m) sin((2n−1)π/[2(2N+1)]),
φ_{j,n}=sin(j(2n−1)π/(2N+1)).

Each shape is divided by √(m Σ_j φ_{j,n}²). The expected participation is
m Σ_j φ_{j,n}/√(m Σ_j φ_{j,n}²), with the sign chosen by the backend;
the effective mass is its square. No published figures are used. The tests
use k=17, m=3, N=1,4,9; relative tolerances ~3e−12 for frequencies,
absolute 3e−12 for shapes/projections and 3e−11 for masses.

### Euler–Bernoulli beams, original derivation

EI w''''+ρA ẅ=0 and w=W exp(iωt) give W''''=β⁴W,
β⁴=ρAω²/EI. Its solution is a combination of sin, cos, sinh and cosh.

Simply supported: W=W''=0 at x=0,L eliminates cos/cosh at the first end
and requires sin(βL)=0 at the second: β_nL=nπ.

Cantilever: W=W'=0 at the origin gives
W=a(cosh βx−cos βx)+b(sinh βx−sin βx).
W''=W'''=0 at L leaves the system with rows
[cosh βL+cos βL, sinh βL+sin βL] and
[sinh βL−sin βL, cosh βL+cos βL].
Its determinant is 2(1+cos βL cosh βL), hence
cos βL cosh βL+1=0. The tests obtain the first three roots by their own
bisection in (0.1,π), (π+0.1,2π), (2π+0.1,3π), 55 iterations; they do not copy
root approximations from another source.

In both cases ω_n=(β_nL)²√(EI/(ρA))/L². The existing frame2d is used,
consistent mass, E=ρ=A=L=1, I=10⁻⁴. This separates the first three bending
modes from the axial ones without suppressing axial DOFs or introducing rotary
inertia/shear deformation. The mesh is refined from 4 → 12 elements: the relative
errors of the first three ω must decrease by a factor >10 and remain <3e−4.
Continuum accuracy with a finite mesh is not promised.

### Orthogonality, rotation and limit cases

Symmetry of K and M gives (λ_i−λ_j)φ_iᵀMφ_j=0; an M-orthonormal basis
satisfies ΦᵀKΦ=diag(λ). Both identities are verified, also for all
beam modes, with tolerance 2e−9 scaled by λ for K.

An orthogonal rotation Q transforms translations of φ and r and leaves Rz unchanged;
K'=QKQᵀ, M'=QMQᵀ preserve λ, φᵀMφ, φᵀMr and M*. Each beam,
its supports (both translations restrained) and the direction are rotated together by
0.7 rad. Frequencies, masses, |Γ| and transformed shapes are compared, allowing
the arbitrary sign of each mode; tolerances between 2e−10 and 3e−9.

A single free axial DOF with E=ρ=A=L=1 has K_ff=1,
lumped M_ff=1/2 or consistent 1/3: λ=2 or 3 and M*=M_ff.
The same test applies a settlement of 0.2 at the base and previously accepts
u_f=0.7: φ at the support remains zero and the states remain intact.

## Errors and limits

- ENG-E-0010: empty/zero directions, count>n and numerical restrictions of
  lowest_modes (indefinite mass, indefinite K). Matrices are not regularized.
- ENG-E-0008: nonfinite/overflowed inputs or results, including Γ,
  mass products, sums, quotients and periods.
- ENG-E-0011: missing translation components for the requested direction.
- ENG-E-0006: singular M_ff; the solver code/message is retained and a
  global node/DOF is added if an entity with reduced index exists. Without an index the
  original diagnostic is retained; nonexistent index gives ENG-E-0012. In the frame with
  lumped mass the massless free rotation indicates node 2, Rz, global 5.
  Condensation/reduction of those DOFs remains pending in ENG-LA-009/ASM.
- ENG-E-0007: nonconvergence, incomplete spectrum or multiplicity cut;
  it is propagated without returning fewer modes. A cut of two zero modes is tested.
- ENG-E-0012: incompatible, negative/unordered spectrum or shape without a positive
  norm returned by a backend that violates the contract.
- There is no `panic!`, `unwrap` or `expect` in the new library. The
  preexisting `expect` in the `SystemView::dof_label` adapter of engine-assembly is not changed;
  the modal analysis does not call that path.
- The dense O(n²) memory path and numerical tolerances of
  Cholesky/Jacobi/Sturm from ENG-LA-009 are inherited. A cut within repeated modes is
  rejected and requires requesting the entire group. Accuracy of
  subnormal projections or extremely ill-conditioned problems is not guaranteed.

## Session verification

All Cargo checks were run with `CARGO_NET_OFFLINE=true`.

- `cargo fmt`: OK, exit 0.
- `cargo clippy --all-targets -- -D warnings`: OK, exit 0. An
  index-based iteration in the new test flagged by Clippy was corrected before rerunning it.
- `cargo test --workspace`: OK, exit 0; 245 tests and doctests, including
  the 6 new modal tests. It was rerun after the test adjustment.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK, exit 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, exit 0,
  with `PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable`.
- `bash clean-room/check.sh origin/main`: OK, exit 0. Includes the three
  new files through `git add -N` in a temporary index and objects
  under `/tmp`, without modifying the repository's real index or objects.
  `git diff --check` on that index: OK.

Native compilation, tests and rustdoc used rustc 1.85.1 (MSRV); installed Clippy
is 0.1.99. The wasm attempt with the system compiler failed due to
missing std for that target. The already installed `stable` toolchain was used,
rustc 1.99.0, which does have the wasm target. Nothing was installed or downloaded.
The wasm check verifies compilation, not test execution or numerical
equality between targets. An initial preparation of the temporary index mistakenly used
the worktree's object directory; it was corrected to the common Git directory
and the check was run successfully, always writing to `/tmp`.

Logs: `/tmp/eng-anl-002-{fmt,clippy,test,doc,wasm-stable,clean-room}.log`;
the initial wasm attempt remains in `/tmp/eng-anl-002-wasm.log`.
No commit or push. Clean-room reviewer: PENDING (J4).
