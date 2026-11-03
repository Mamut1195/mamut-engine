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

## Continuation: uniform mesh and dimensional spring tables

Specifier and implementer: Codex gpt-6.1-terra high, 2026-11-03
(America/Guayaquil). Sources are the supplied excerpts of the existing
engine and this log, including the own quadrature derivation above; no
external source, prohibited path, manual, or oracle output was consulted.
No third-party code was copied.

The uniform mesh uses z_i = L*i/n and assigns z_n = L exactly. The explicit
limit of 4096 elements bounds this helper's allocation; it is not a solver
capacity or accuracy claim. Existing geometry checks reject coincident
floating-point depths and underflowed half-spans.

For nodal mobilized resistance, the p-y ordinate is multiplied by its
tributary length; the t-z ordinate is multiplied by circumference times
tributary length; q-z is already a force and is copied without a geometric
multiplier. These conversions follow directly from their supplied units.
Positive resistance follows positive displacement and soil force is opposite.
Conversion rejects negative slopes, nonfinite values, and nonzero products
or slopes reduced to zero. The caller explicitly accepts end-segment
extension or requires plateaus at both ends. A returned table alone cannot
enforce a bounded p-y evaluation.

Analytical tests use a 2 m tributary, 3 m circumference and a 10-unit
ordinate: the resulting lateral, shaft and tip forces are 20, 60 and 10 N.
The mesh test integrates z over 0 to 8 m as 32 m^2. These are synthetic
own derivations, not published examples.

This continuation does not implement or validate finite pile equilibrium,
Newton assembly, structural recovery, load-settlement curves, or convergence
against the semi-infinite solution. Those remain required work. The published
Reese/Matlock example remains pending a supplied verifiable source; no example
has been invented. No checks were run for this continuation. Formatting,
compilation, native and wasm execution, workspace checks, determinism checks,
and independent clean-room review remain pending.

## Continuation: finite pile candidate

Specifier and implementer: Codex gpt-6.1-terra high, 2026-11-03
(America/Guayaquil), using only the supplied local excerpts and own
derivations. No external source, prohibited path, manual, oracle output, or
third-party code was consulted. Independent clean-room review is pending.

The local planar X coordinate is depth z, increasing downward, and Y is the
chosen lateral direction. This computational plane does not redefine the
engine's global vertical axis. Axial displacement and load are positive
downward; lateral displacement and H are positive Y; applied M is positive
about +Z. The linear Euler frame supplies EA and EI. Grounded spring internal
forces equal mobilized resistances; actual soil forces are their negatives.
The p-y ordinate is multiplied by nodal tributary length, t-z by circumference
and tributary length, and q-z is already force and acts once at the tip.
The supplied diameter determines only circumference, while A and I are explicit.
The free head accepts M; the fixed head prescribes zero rotation. Both head
translations and the tip are free unless restrained by soil springs.

The discrete equilibrium is `K_frame u + sum(f_spring(u)) = factor F_head`.
The frame contributions enter LinearSystem once; the variable springs enter
NewtonSystem only. Each accepted factor is recovered before the next attempt.
Absolute factors scale H, M and axial load together. Nonconvergence returns
the failed report and previously accepted profiles, without retries or a
failed trial. Singular tangents and other infrastructure errors propagate;
partial results are not returned with an error. End segments are extended
under the caller's explicit policy. Multilinear elastic behavior is stateless;
negative slopes, including clay softening, are rejected. No cyclic behavior,
geometric stiffness, load-independent prescribed settlement, or separate
axial/lateral load histories are provided.

For the frame oriented along increasing depth, its local positive-face
resultants satisfy `Mz = EI*y''` and `Vy = -EI*y'''` between point springs.
Negating these gives the analytical pilot convention `M = -EI*y''` and
`V = EI*y'''`. Profiles select the element immediately below each node,
except at the tip, where the element immediately above is selected; shear can
jump across a nodal spring. Soil p is minus the nodal spring force divided by
its tributary length. These are discrete one-sided values, not exact
continuous extrema. Head rotation reaction is not returned separately.

The synthetic lateral test uses EI=1 N m^2, k=4 N/m^2, H=4 N and
M=2 N m. It compares finite meshes of a 12 m pile with the existing
semi-infinite head displacement, including finite-length and discretization
errors. The independent axial check has EA=100 N, L=12 m, no shaft
resistance, tip stiffness 10 N/m and head load 10 N. Series compatibility
gives head settlement `10*(12/100 + 1/10) = 2.2 m`, and half load gives
1.1 m. These are own analytical cases, not published examples.

At the time of this continuation, this candidate and its tests had not been
compiled, formatted, executed natively or on wasm, or independently reviewed;
workspace and determinism checks had not been performed. The published
Reese/Matlock example remained pending a supplied source. Verification of profile signs and convergence,
nonlinear and asymmetric axial cases, nonconvergence behavior, and the
workspace dependency graph remains necessary before claiming MAM-49 complete.

## Additional finite-pile test derivations

These proposed tests use the supplied implementation and the analytical
derivations above; no external source or oracle is used. They have not been
executed as part of this entry.

For the lateral comparison, EI=1, k=4, H=4 and L=12 give beta=1.
The free-head frame load M=+2 maps to the analytical head value M=+2:
for the frame aligned with depth, its positive-face Mz is EI*y'', and
its Vy is -EI*y'''; the reported pile values negate both. The head
shear immediately below the first spring differs from H by that spring's
force. A fixed head instead uses zero slope and its moment is a reaction.
The tests compare y, M, V and p at depths 0, 1.5, 3 and 4.5 m on 32-
and 64-element meshes. Absolute fine-mesh tolerances are respectively
0.03 m, 0.08 N m, 0.5 N and 0.12 N/m. Shear has the largest allowance
because it is recovered on one side of a nodal spring. The tolerances also
allow finite-length error; they are case-specific, not general error bounds.
The weighted soil forces additionally satisfy H + sum(p_i*w_i)=0 and,
for the free head, M + sum(z_i*p_i*w_i)=0.

For the axial benchmark, L=2 m and EA=100 N give bar stiffness 50 N/m.
The diameter 1/pi m gives circumference 1 m and each tributary is 1 m.
Only the head shaft spring is nonzero; its positive branch is
t=30z for 0<=z<=0.1 m and t=10z+2 for 0.1<=z<=0.3 m.
The tip resistance is Q=50z. Bar and tip are in series, so toe
settlement is half the head settlement and their effective head stiffness
is 25 N/m. Thus P=3 N gives head 3/55 m and toe 3/110 m; P=9 N
gives head 0.2 m and toe 0.1 m, with 4 N in the shaft spring and
5 N at the tip. The symmetric elastic tables give the negatives of the
3 N displacements at P=-3 N after unloading.

For the one-correction failure case, the zero factor is accepted first.
The initial tangent predicts head settlement 9/55 m at P=9 N, beyond
the shaft breakpoint. The actual shaft force there is 40/11 N, whereas
the initial tangent predicts 54/11 N, leaving a nonzero residual. One
correction therefore cannot converge at the stated tolerance. The test
checks that this trial is excluded and the following factor is not attempted.

## Subsequent offline validation

With Rust 1.99, `CARGO_NET_OFFLINE=true`, and a dedicated target directory:

- `cargo test --locked --offline -p engine-geo -p engine-analysis`: exit code
  0; 178 tests passed, including 5 doc tests. An earlier attempt failed with
  E0502 and exit code 101; it was corrected.
- `cargo clippy --locked --offline -p engine-geo -p engine-analysis
  --all-targets -- -D warnings`: exit code 0. An earlier attempt failed on a
  range-loop warning with exit code 101; it was corrected.
- `cargo check --locked --offline -p engine-geo -p engine-analysis --target
  wasm32-unknown-unknown`: exit code 0; compilation only.
- `cargo fmt --all -- --check`: exit code 0. The first format check failed
  and was corrected.
- `git diff --check`: exit code 0.

With newly added files staged, `clean-room/check.sh --all` was run via Git
Bash with the script converted from CRLF to LF in memory: --all passed with
rc0. The full
`clean-room/check.sh` was also run against base
`c64d9977d62f539a3a61e69a078bf4d8bc5526df`, using the in-memory LF
script and temporary LF copies of the unmodified path and identifier lists:
exit code 0, `Clean-room: OK`. An initial direct Bash attempt failed with
exit code 2 due to CRLF in the script; an earlier baseline attempt failed
with exit code 2 due to CRLF in a list. A later attempt failed with exit
code 127 due to shell quoting; it was corrected.

The published Reese/Matlock example has not been supplied, wasm fixtures
have not been executed, and independent clean-room review remains pending.
No public API, CLI, or JSON interface has been added.

The finite pile candidate uses a linear frame and independent nonlinear
monotonic soil springs. It rejects softening, and absolute load factors scale
the head loads together. These results do not establish completion of MAM-49.
