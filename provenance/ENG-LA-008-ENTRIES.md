# Bitácora clean-room · seguimiento issue #22 (SparsityPattern::entries)

- Ticket / PR: issue #22; PR de esta rama `j2/la-007-entries-fallible`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: engine-linalg (API interna; sin cambio de contratos externos)
- Fuentes permitidas usadas: ninguna externa; aritmética elemental de tamaños (orden² bytes) y la política de límites de reserva existente (`MAX_ALLOC_BYTES`, PR #21/#24).
- Derivaciones propias: `entries()` (infalible, O(orden²)) se reemplaza por `try_entries()` (fallible: ENG-E-0013 si orden² desborda o supera `MAX_ALLOC_BYTES`; ENG-E-0010 si la reserva falla), `contains(i,j)` (O(log nnz)) y `mask_iter()` (perezoso, O(1) memoria). Cambio de API pre-1.0 (0.1.0); únicos usos eran pruebas.
- Código de terceros copiado: ninguno
- Validación: analítico (pruebas de tamaño extremo y equivalencia máscara/coordenadas)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)
