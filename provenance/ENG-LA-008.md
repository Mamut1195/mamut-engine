# Bitácora clean-room · ENG-LA-008 (parte: presupuesto acumulado de memoria)

- Ticket / PR: ENG-LA-008 (presupuesto acumulado de memoria en engine-linalg); rama `j2/la-008-memory-budget`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: engine-linalg (API nueva `MemoryBudget`, `dense_bytes`, `sparse_bytes`; sin cambios en LinAlgBackend)
- Fuentes permitidas usadas: ninguna externa; contabilidad elemental de bytes.
- Derivaciones propias: contador explícito y determinista (sin SO, hilos ni estado global) con límite, uso y pico; `charge` rechaza con ENG-E-0013 sin alterar la cuenta; `zeros` y `solve_dense` cargan el conjunto de trabajo (A, B, X) y solo la solución queda cargada. `alloc_vec` sigue limitando cada reserva individual. Alcance: contabilidad por llamador; el backend no lleva presupuesto interno (cableado en factorize/eigen y solver disperso: PENDIENTE, ENG-LA-008 restante).
- Código de terceros copiado: ninguno
- Validación: analítico (suma de reservas permitidas excede el presupuesto; bordes de desborde/liberación; conjunto de trabajo exacto ±1 byte)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)

## Segunda ronda (J4 r3, 2026-10-11)

- `ReferenceBackend::solve_dense` con A 0×0 devuelve B sin reservar vectores por columna (antes: pánico «capacity overflow» con B 0×usize::MAX).
- `MemoryBudget::solve_dense` usa una guarda RAII: un pánico recuperado del backend libera la carga; `used` no queda cargado. `peak` documentado (conserva el máximo del conjunto de trabajo cargado). `release` sin identidad: doble liberación documentada como subcontabilidad del llamador.
- Pruebas: `empty_system_with_huge_column_count_does_not_panic`, `recovered_backend_panic_does_not_leave_budget_charged`.
- Revisor clean-room: PENDIENTE (J4)
