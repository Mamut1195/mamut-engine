# Clean-room log · SystemView kit T2 follow-up (J4 retro)

- Ticket / PR: T2 from /workspace/j4rev/tickets/engine-retro.md; branch `j2/conf-sysview-overwrite`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Contract implemented: SystemView (conformance kit in engine-contracts)
- Permitted sources used: none external; the trait's own contract (outputs are overwritten).
- Own derivations: `check_system_view` evaluates mass, tangent and Kg with zero output and with a symmetric sentinel proportional to the magnitude of the clean matrix (1e3·máx(|a|,1); J4 #30: a constant one did not detect K=1e12) and requires identical results; a system that accumulates fails. The second output is also required to retain the dimension. T2 LOW findings (bounded h, infinitesimal rotations): separate PRs.
- Third-party code copied: none
- Validation: analytical (test `system_kit_detects_accumulation_in_matrix_outputs` with systems that accumulate in tangent, mass and Kg + clean control)
- Comparison with oracle: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)

## J4 round (K=1e306)

- `overwritten` no longer uses expect/unwrap: construction errors → diagnostic (take). With magnitude ~1e306 the 1e3·máx sentinel overflowed and panicked; it now uses the magnitude itself if the factor overflows. Test with K=1e306 in `system_kit_detects_accumulation_in_matrix_outputs`.
