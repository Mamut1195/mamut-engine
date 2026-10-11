# Bitácora clean-room · kit SystemView seguimiento T2 (J4 retro)

- Ticket / PR: T2 de /workspace/j4rev/tickets/engine-retro.md; rama `j2/conf-sysview-overwrite`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: SystemView (kit de conformidad en engine-contracts)
- Fuentes permitidas usadas: ninguna externa; el propio contrato del trait (las salidas se sobrescriben).
- Derivaciones propias: `check_system_view` evalúa masa, tangente y Kg con salida en ceros y con centinela simétrico proporcional a la magnitud de la matriz limpia (1e3·máx(|a|,1); J4 #30: uno fijo no detectaba K=1e12) y exige resultados idénticos; un sistema que acumule falla. También se exige que la segunda salida conserve la dimensión. Bajas de T2 (h acotado, rotaciones infinitesimales): PRs aparte.
- Código de terceros copiado: ninguno
- Validación: analítico (prueba `system_kit_detects_accumulation_in_matrix_outputs` con sistemas que acumulan en tangente, masa y Kg + control limpio)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)

## Ronda J4 (K=1e306)

- `overwritten` ya no usa expect/unwrap: errores de construcción → diagnóstico (take). Con magnitud ~1e306 el centinela 1e3·máx desbordaba y paniqueaba; ahora usa la propia magnitud si el factor desborda. Prueba con K=1e306 en `system_kit_detects_accumulation_in_matrix_outputs`.
