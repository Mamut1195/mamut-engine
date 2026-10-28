# Clean-room log · ENG-ASM-001 follow-up T1 (J4 retro)

- Ticket / PR: T1 from /workspace/j4rev/tickets/engine-retro.md; branch `j2/asm-geometric-has-flag`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil).
- Role: implementer
- Implemented contract: SystemView::geometric (engine-assembly)
- Permitted sources used: none external; own reasoning (P-Δ: with zero axial force at the reference Kg(ref)=0 but Kg(u≠ref)≠0).
- Original derivations: `has_geometric` is set at construction (some `ElementContribution.geometric.is_some()`); if true and u≠referencia → ENG-E-0013 even if Kg(ref) is zero. The real Kg(u) producer remains pending (recalculation of axial forces).
- Third-party code copied: none
- Validation: analytical (test `zero_reference_geometric_still_rejects_other_states`)
- Oracle comparison: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados"
- Clean-room reviewer: PENDING (J4)
