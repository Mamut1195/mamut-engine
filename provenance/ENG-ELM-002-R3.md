# Bitácora clean-room · ENG-ELM-002 seguimiento #41 (test discriminante)

- Ticket / PR: seguimiento de #41 (J0); rama `j2/frame-recover-test`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: element/1 · frame2d.euler_elastic (solo pruebas)
- Fuentes permitidas usadas: ninguna externa.
- Derivaciones propias: test con EI ∈ {1e12, 1e20, 1e30}, L ∈ {3.1, 0.7, 11.3, 2/3} y tres estados u en doble liberación; exige cortante y momento exactamente 0. Verificado que FALLA sin el arreglo de #41 (EI=1e12, L=3.1: cortante espurio 3.05e-5) y pasa con él.
- Código de terceros copiado: ninguno
- Validación: analítico (viga doblemente articulada sin carga: V=M=0)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)
