# Clean-room log — ENG-GEO-005

- Ticket: MAM-49, first slice only.
- Author: Codex gpt-6.1-terra high, 2026-11-03 (America/Guayaquil).
- Role: specifier and implementer.
- Contract: library-level pile-axis discretization; no structural element contract.
- Sources: own derivation below; supplied `docs/CLEAN_ROOM.md`,
  `docs/METHODS_GUIDE.md`, `provenance/TEMPLATE.md`,
  `provenance/ENG-GEO-002.md`, `provenance/ENG-GEO-003.md`,
  and supplied engine-geo and engine-core excerpts.
- Third-party code copied: none. Oracle comparison: no.
- Declaration: No consulté código fuente de OpenSees, Frame3DD, Stabileo
  ni derivados para este ticket. No oracle outputs or external sources were
  consulted in this session.
- Clean-room reviewer: PENDING.

## Method and analytical checks

Let depths `z_0 < ... < z_n` be nonnegative, in m, and let
`h_i = z_(i+1) - z_i`. Integrating a linear interpolant over each span
gives `h_i * (p_i + p_(i+1)) / 2`. Collecting nodal coefficients gives
`w_0 = h_0/2`, `w_i = (h_(i-1) + h_i)/2`, and `w_n = h_(n-1)/2`.
Thus `sum(w_i) = z_n - z_0`; for an affine field the weighted sum
equals its exact integral. If `p` is in N/m, `w_i * p_i` is in N.
This quadrature is an approximation for nonlinear spatial fields and does
not imply a soil law, pile equilibrium, or a structural connection.

For depths 0, 2, 5, 9 m, the weights are 1, 2.5, 3.5, 2 m;
their sum is 9 m and `sum(z_i*w_i) = 40.5 m²`. These are own
analytical cases. At the time of this entry, execution, formatting, workspace
checks, native/wasm validation, and independent review were pending; no tests
had been run.

## Semi-infinite lateral analytical pilot

This pilot is a test of the existing tributary integration, not a new
public pile solver. It uses an own derivation and no external source or
oracle output. For depth z downward, take EI*y'''' + k*y = 0 with
beta = (k/(4*EI))^(1/4). A decaying solution is
y = exp(-beta*z)*(A*cos(beta*z) + B*sin(beta*z)).
Define soil force p = -k*y, moment M = -EI*y'', and shear V = EI*y'''.
At the head, M = 2*EI*beta^2*B and V = 2*EI*beta^3*(A+B).
These are sign definitions for this pilot.

For EI=1 N m^2, k=4 N/m^2, H=4 N and head moment 2 N m,
beta=1/m and A=B=1 m. The exact integrals of -p and -z*p over
the semi-infinite domain are 4 N and 2 N m. The test integrates the
field over 16 m using the existing tributary lengths and checks both
quantities and decreasing errors under refinement. Its tolerance includes
the omitted tail and quadrature error; it is not a general error bound.

The pilot assumes a bilateral, homogeneous, linear foundation and a
semi-infinite lateral field. It does not model a finite tip, axial
response, nonlinear soil, or layering, and makes no published example
claim. At the time of this entry, execution and independent review were pending.

## Semi-infinite lateral library solution

The analytical solution is a separate library capability based on the
equation and sign definitions of the pilot above. It does not assemble or
solve the discretization. With x = beta*z, differentiating the decaying
field gives:

```text
y'   = beta*exp(-x)*[(B-A)*cos(x) - (A+B)*sin(x)]
y''  = 2*beta^2*exp(-x)*[A*sin(x) - B*cos(x)]
y''' = 2*beta^3*exp(-x)*[(A+B)*cos(x) + (B-A)*sin(x)]
```

For prescribed head shear H and moment M0, B = M0/(2*EI*beta^2)
and A = H/(2*EI*beta^3) - B. This includes a moment-free head when
M0 is zero. For a fixed rotational head, y'(0) = 0 gives B = A,
and A = H/(4*EI*beta^3); its moment is a reaction. The stated M and V
are signed definitions, not a claim about the frame element's end signs.

The tests use the pilot's EI=1, k=4, H=4 and M0=2 case, and the
fixed head with the same H. At z=pi/2, y=e, M=-2e, V=0 and
p=-4e, where e=exp(-pi/2). A moment-free case with H=2 has
y=0, M=-2e, V=-2e and p=0 at that depth. These values follow by
substitution, not from a published example or external oracle.

Inputs and calculated intermediates are checked for finiteness; invalid
ranges and zero stiffness scales caused by underflow are rejected. Extreme
finite values may be rejected due to intermediate overflow or underflow.
Other response terms can underflow or lose precision through cancellation;
no full-range accuracy is claimed.

The model is bilateral, homogeneous, linear and semi-infinite. It does not
model a finite tip, axial response, nonlinear soil, layering, or a finite
discretized nonlinear axial or lateral pile. At the time of this entry, no
tests, formatting, workspace checks, native or wasm validation, or independent
review had been run for this library addition.

## Recorded validation

With Rust 1.99, `CARGO_NET_OFFLINE=true`, and an isolated target directory:

- `cargo test --locked --offline -p engine-geo`: exit code 0.
- `cargo check --locked --offline -p engine-geo --target wasm32-unknown-unknown`:
  exit code 0; compilation only.
- `cargo clippy --locked --offline -p engine-geo --all-targets -- -D warnings`:
  exit code 0.
- `cargo fmt --all -- --check`: exit code 1 due to the two formatting
  differences addressed in this patch. The check has not been rerun after
  this patch.

Native tests and wasm compilation passed. WASM execution and independent
review remain pending.
