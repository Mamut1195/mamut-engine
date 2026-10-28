# Clean-room log · ENG-LOAD-001

- Ticket / PR: ENG-LOAD-001 «Casos de carga»; no commit, push or PR in this session.
- Author(s): J2 (agent, Grok Bot), 2026-10-11 (America/Guayaquil), label requested for the ticket. Execution model for this session: Codex based on GPT-6.
- Role: specifier and implementer through own derivations; no auditor or oracle operator role.
- Implemented contract: crate `engine-loads` 0.1.0; adaptation to the uniform `engine-contracts::MemberLoad` type of element/1; reuse of `engine-model::{NodalLoad,LoadCase,DofValue}` and `engine-assembly::{DofMap,PrescribedDof}`.
- Permitted sources used: exclusively the user's specification, contracts and relevant internal code of this repository; no external sources or network access.
- Own derivations: equations (1)-(7) of this note, written for this ticket.
- Third-party code copied: none. No new external dependencies; internal path dependencies.
- Validation: analytical; closed-form tables, equilibrium, superposition, axis transformation and agreement with the internal frame2d element. Fixtures with `source` and `citation` in `crates/engine-loads/tests/fixtures/mod.rs`.
- Oracle comparison (sibling repo, informative): no. No oracle was run.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Clean-room reviewer: PENDING (J4).

## Own derivations

### (1) Virtual work and interpolation

In a 2D Euler–Bernoulli beam without releases, let r=x/L and t=1−r.
The axial interpolation compatible with the end values is
u=t·u_i+r·u_j. For the transverse interpolation, a cubic polynomial satisfying
v(0)=v_i, v'(0)=theta_i, v(L)=v_j, v'(L)=theta_j determines:

H_i=t²(1+2r), H_theta_i=Lrt², H_j=r²(1+2t), H_theta_j=−Lr²t.

Each polynomial satisfies its unit value or slope condition and the other
three zero conditions. From ∫(qx·δu+qy·δv)dx=f_eq·δd we obtain:

f_eq=∫[t·qx, H_i·qy, Lrt²·qy, r·qx, H_j·qy, −Lr²t·qy]dx.

For a point force the integrand is evaluated by replacing qdx with P.
The clamped-end actions on the member are f_fix=−f_eq. They are not
confused with section resultants according to the outward normal.

### (2) Uniform and axial

Integrating (1) between r=0 and r=1 and using dx=Ldr:

f_eq=[qxL/2, qyL/2, qyL²/12, qxL/2, qyL/2, −qyL²/12].

The uniform implementation orders the products as q(L/2) and [q(L/12)]L,
avoiding calculating L² beforehand. It agrees with the internal frame2d formula.

### (3) Full trapezoid and triangular

For each component q(r)=qi(1−r)+qj·r, multiplying by the polynomials (1)
and integrating monomials through ∫₀¹r^k dr=1/(k+1) gives:

N_i=L(2qx_i+qx_j)/6, N_j=L(qx_i+2qx_j)/6;
V_i=L(7qy_i+3qy_j)/20, V_j=L(3qy_i+7qy_j)/20;
M_i=L²(3qy_i+2qy_j)/60, M_j=−L²(2qy_i+3qy_j)/60.

With qi=qj=w it reduces to (2). For the increasing triangular load qi=0, qj=w:
axial forces wL/6 and wL/3; shears 3wL/20 and 7wL/20; moments wL²/30
and −wL²/20. The decreasing triangular load swaps ends and moment
magnitudes. A trapezoid is a uniform load qi plus a triangular load of height qj−qi,
also when both occupy the same partial interval.

### (4) Point force

With a=x and b=L−a, substitute r=a/L and t=b/L into (1):

N_i=Px·b/L, N_j=Px·a/L;
V_i=Py·b²(L+2a)/L³, V_j=Py·a²(L+2b)/L³;
M_i=Py·ab²/L², M_j=−Py·a²b/L².

At a=b=L/2: shears Py/2, moments ±PyL/8. The expression «PL/2» for
the point-load shear in the statement is interpreted as P/2 on dimensional grounds
(N, whereas PL has moment units). For x=0 or L, the force is
applied exactly at the corresponding node and the moments are zero.

### (5) Partial intervals and exact integration

For 0≤a<b≤L, define z∈[0,1], x=a+(b−a)z and q=qi(1−z)+qjz.
The integrand (1) is polynomial of degree ≤4 in z. The three-point rule
with z=(1−√(3/5))/2, 1/2, (1+√(3/5))/2 and weights 5/18, 4/9, 5/18
has exact moments 1/(k+1) for k=0,…,5: by symmetry its odd central
moments are zero, and the even ones k=0,2,4 are 1,1/12,1/80.
These are the moments of the uniform integral on [0,1]. It is therefore
exact analytical integration for this problem, apart from f64 rounding.
No approximate quadrature, refinement or external sources are used.

Independent partial table, derived through monomial antiderivatives between
r=1/4 and 3/4, for both components equal:

| Distribution | N_i/(wL) | V_i/(wL) | M_i/(wL²) | N_j/(wL) | V_j/(wL) | M_j/(wL²) |
| --- | --- | --- | --- | --- | --- | --- |
| Uniform w | 1/4 | 1/4 | 11/192 | 1/4 | 1/4 | −11/192 |
| Triangular 0 → w | 5/48 | 61/640 | 31/1280 | 7/48 | 99/640 | −127/3840 |

In the second row q(r)=w(2r−1/2). Each polynomial of (1) is multiplied
by (2r−1/2) and integrated term by term. Resultant wL/4 and transverse
moment about i of 7wL²/48. For extremely small intervals
relative precision may be lost through underflow; recovering
quantities outside the IEEE-754 representation is not promised.

### (6) Equilibrium and axes

The identities t+r=1, H_i+H_j=1 and
H_theta_i+H_theta_j+L·H_j=x demonstrate, respectively:

N_i+N_j=∫qx dx; V_i+V_j=∫qy dx;
M_i+M_j+L·V_j=∫x·qy dx.

For clamped-end reactions, the sums with the external actions are zero.
An axial load contributes no moment about i. The tests verify these
identities for uniform, trapezoidal, triangular, partial and point loads.
For a full trapezoid: ∫q dx=L(qi+qj)/2 and ∫xq dx=L²(qi+2qj)/6.

For c=cos(angle), s=sin(angle), q_local=[cQx+sQy,−sQx+cQy].
The inverse transformation is Q_global=[cqx−sqy,sqx+cqy]. Z moments
are preserved. The x coordinate of a load always measures distance along the
member rather than a global projection. A rotated beam is tested with all three
load types and agreement of uniform load with load_vector and recover of the
frame2d element without releases. The MemberLoad contract receives global
components and zero moments per unit length.

### (7) Self-weight, nodal cases and displacements

A length dx has mass rho·A·dx and force rho·A·g·dx. Integration gives
rho·A·L·g, and for several members Σrho·A·L·g. The direction and magnitude of g
are explicit three-dimensional inputs; no code factors, rho≥0, A>0,
L>0. Total forces, forces per unit length and member summation are tested.

Nodal actions reuse engine-model's values and cases; the position
of each component is provided by DofMap, including Rz in the third position
in D2. assemble_vector performs global assembly. No other numbering is defined.
ImposedDisplacement is an alias of PrescribedDof; the Option vector distinguishes
free from prescribed zero. Its application through Constraints, D2/D3
dimensions, DOF subsets, missing references, duplicates and nonfinite values are tested.
Repeated node/DOF entries in the same case list are rejected, even
with equal values. Force and displacement on the same DOF are compatible.

## Session verification

Results of this session, all with final exit 0 and no network access (`CARGO_NET_OFFLINE=true`):

- `cargo fmt`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, including 13 integration tests and 1 doctest
  of engine-loads. Native tests use rustc 1.85.1, the workspace's minimum version.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
  Four links interpreted by rustdoc were corrected before the final result.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK using the
  toolchain already installed at `/home/box/.cargo/bin` (rustc 1.99.0). The system rustc
  (1.85.1) lacks the wasm standard library; the first attempt
  reported compilation error E0463. No components were downloaded.
  This check verifies compilation; neither execution nor wasm numerical equivalence.
- `bash clean-room/check.sh origin/main`: OK, including all new files
  through `git add -N` in a temporary copy of the index in `/tmp`.
  Temporary objects were also placed in `/tmp`; the real index was
  kept intact. `git diff --check` on that copy: OK.

No commit or push. Clean-room review: PENDING (J4).
