# Clean-room log · ENG-VAL-MAM-845

- Ticket / PR: MAM-845 · ENG-VAL-006. Rama `j2/mam-845-preregistro-tolerancias`. Al escribir esta bitácora el número de PR es DESCONOCIDO.
- Author(s): agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO. Fecha: 2026-11-09 23:43 (America/Guayaquil).
- Role: implementer. Solo documentación: tabla, marco, integración en el informe y esta bitácora.
- Contract implemented: ninguno de producción. El informe sigue siendo ENG-VAL-006.
- Permitted sources used: ticket MAM-845, aprobado por Jonathan vía J0 el 2026-10-20; extracto del plan en sus apartados de marco, métrica y recomendaciones; tolerancias ya escritas en el repositorio y citadas fila a fila en `docs/validation-report/preregistro/tolerancias.csv`. ASME V&V 10-2019 (R2025) se nombra como encuadre. Sección y página de la norma: cita desconocida. No se inventan apartados.
- Own derivations: ninguna ecuación nueva y ninguna campaña. La métrica relativa, y la normalización por el máximo de la serie cuando la referencia es aproximadamente cero, se declaran como regla del plan. El umbral numérico de «aproximadamente cero» es DESCONOCIDO.
- Third-party code copied: none.
- Validation: no se ejecutó campaña de oráculos. Treinta y cuatro filas consolidan una tolerancia ya escrita. Cinco filas son preregistro (VAL-033, VAL-034, VAL-035, VAL-036 y VAL-041). VAL-031 queda en DESCONOCIDO. VAL-037 queda en revisión en #23, sin valor fijo.
- Comparison with oracle (sibling repo, informative): no.
- Session: declaración de no exposición a OpenSees, Frame3DD, Stabileo, CalculiX y LAPACK.
- Declaration: «No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK ni derivados para este ticket.»
- Clean-room reviewer: PENDING.

## Commit de la tabla

- SHA: `c0109829dcc54af0c781c5287e64726275552aa7`.
- Fecha real `%cI`: `2026-11-10T04:43:09+00:00`.
- Ese commit se empujó a `origin` antes de escribir esta bitácora y antes del commit que la contiene. La hora del push, distinta de la hora del commit, es DESCONOCIDO: no se registró aparte.

## Orden

El orden de ejecución de los casos consolidados es DESCONOCIDO. Git solo muestra la introducción de los archivos citados en la columna `introduccion_git`. Los CSV de datos del informe, de VAL-001 a VAL-030, seguían en el marcador de dos líneas. No se leyó ninguna serie numérica para llenar la tabla.

## Qué no se abrió

No se abrieron `results/` ni archivos de salida de un programa de contraste. Para VAL-032 la tabla cita solo el nombre del campo `tolerances_preregistered` y no transcribe sus valores numéricos. No se tocaron los archivos del PR #23. No hay código de producción en este cambio. `docs/contracts/error-codes-1.json` no se modifica.
