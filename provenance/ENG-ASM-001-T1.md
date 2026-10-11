# Bitácora clean-room · ENG-ASM-001 seguimiento T1 (J4 retro)

- Ticket / PR: T1 de /workspace/j4rev/tickets/engine-retro.md; rama `j2/asm-geometric-has-flag`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: SystemView::geometric (engine-assembly)
- Fuentes permitidas usadas: ninguna externa; razonamiento propio (P-Δ: con axial nulo en la referencia Kg(ref)=0 pero Kg(u≠ref)≠0).
- Derivaciones propias: `has_geometric` se fija al construir (algún `ElementContribution.geometric.is_some()`); si es true y u≠referencia → ENG-E-0013 aunque Kg(ref) sea cero. Sigue pendiente el productor real de Kg(u) (recálculo de axiales).
- Código de terceros copiado: ninguno
- Validación: analítico (prueba `zero_reference_geometric_still_rejects_other_states`)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)
