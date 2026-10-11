# Bitácora clean-room · ENG-ELM-002 seguimiento J4 r2 (recover.rs)

- Ticket / PR: bajo de J4 sobre #31; rama `j2/frame-recover-spurious`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: element/1 · frame2d.euler_elastic (recover)
- Fuentes permitidas usadas: ninguna externa; con doble liberación la flexión condensada es exactamente cero.
- Derivaciones propias: en doble liberación la parte flexional elástica (índices 1,2,4,5) se fija en cero en vez de calcular K·u_recuperado, que con EI=1e20 solo aportaba ruido de redondeo (cortante ±64 N, M=198 N·m con L=3.1). Cortante/momento quedan exactamente los de la carga (±qL/2, qL²/8).
- Código de terceros copiado: ninguno
- Validación: analítico (viga articulada con carga uniforme; sin carga: cortante y momento exactamente 0)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)
