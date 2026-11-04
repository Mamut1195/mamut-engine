# Clean-room log · ENG-LOAD-TEMP-05

- Ticket / PR: docs/tickets/05-temperatura-uniforme.md, related to MAM-972.
  MAM-972 and MAM-107 stay open. No J4 signature.
- Author(s): implementation sub-agent, Grok 4.7, 2026-11-04 (America/Guayaquil).
- Role: implementer. Public wiring of an existing closed form. No new element
  kind and no new mechanical law.
- Contract implemented: engine-input/1 field `cases[].temperature_loads` on
  the existing element/1 kinds `truss2d.linear` and `truss3d.linear`.
- Permitted sources used: this repository only. The derivation, sign
  convention, and fixture 8 Pa · 4 m² · 0.5 1/K · 3 K = 48 N are in
  `provenance/ENG-LOAD-TEMP-972.md` and
  `crates/engine-elements-truss/tests/temperature.rs`. No external book,
  paper, or equation number is claimed.
- Own derivations: none beyond the integration choice recorded below. The
  axial formula was not re-derived and was not edited.
- Third-party code copied: none.
- Validation: analytical. A restrained truss read by `run` and by the TOML
  converter recovers N = -48 N, reactions +48 N and -48 N, and a free
  companion displacement of 0. The cold case on the same bar recovers 0.
- Comparison with oracle (sibling repo, informative): no. OpenSees, Frame3DD,
  and Stabileo were not run and were not read.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING

## Claim

The public record is `member`, `alpha_per_kelvin` (1/K), and
`delta_t_kelvin` (K). There is no gradient field. One record per member
per case. A second record in the same case is ENG-E-0002. A missing member
uses the existing "Load: missing member {id}" input error. A nonfinite
coefficient or increment is ENG-E-0008 through the existing bar formula.
Zero α or zero ΔT is valid and yields N = 0.

`run` calls `LinearTruss::fixed_end_uniform_temperature` for the joint
forces and `LinearTruss::with_fixed_end_uniform_temperature` for recovery.
The numbers are not recomputed in the facade. The assembly element stays
mechanical. Recovery of a heated case uses a temporary bar that carries
that case's change, so a second case on the same bar stays at N = 0.

## Why the joint forces are nodal loads

`CaseResults` closes equilibrium when the assembled element-load vector
matches `MemberLoad`. A thermal joint force is not a `MemberLoad`: an axial
force per length cannot keep N constant and independent of L, and the truss
load vector does not apply opposite end forces. Putting the joint forces
only in the element-load channel would leave the balance open and `run`
would not return results. The facade therefore clones the model when any
temperature record exists and appends the joint forces as nodal loads of
that case. The original `cases[].loads` document is left unchanged.
`loads`, `member_loads`, and `temperature_loads` share the 16384 record
limit.

## Limits

Elastic beam-column kinds, including an omitted element kind that becomes
`frame2d.euler_elastic` or `frame3d.euler_elastic`, and both Winkler kinds,
have no αΔT formula. A record on those kinds is ENG-E-0013. No frame term
was invented.

Geometric stiffness stays mechanical. pDelta still emits no member forces,
so that document does not show the thermal axial. A modal or spectrum
rebuild that clones the original model drops the injected nodal loads.
No thermal mass formula was added. A nonlinear step scales the stored
change by the accepted load factor so the reported axial matches the joint
forces the solver applied. Linear static uses factor 1.
