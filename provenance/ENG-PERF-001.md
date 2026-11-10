# Bitácora clean-room · ENG-PERF-001

- Ticket / PR: MAM-1056. Perfil de la ruta estática densa de un pórtico de 250 nudos. Rama `j2/mam-1056-perfil-ruta-densa`. Esta bitácora no cierra MAM-107 y no aprueba un presupuesto de tiempo o memoria.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-10 (America/Guayaquil, ECT, UTC-5).
- Rol: medidor. El motor numérico no cambia.
- Session: sesión limpia del 2026-11-10 (America/Guayaquil, ECT, UTC-5), distinta de la del 2026-11-04. Esta sesión no vio código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK, SuiteSparse, CSparse, CHOLMOD, AMD, MUMPS ni PARDISO, ni de sus derivados.
- Contrato implementado: ninguno nuevo. Se ejercitan `engine-input/1`, `results/1`, `static.linear`, `modal` y `pdelta` ya publicados, con el backend `reference.dense`.
- Fuentes permitidas: el código de este repositorio y las constantes ya escritas en `crates/engine-bench`. E = 4700·sqrt(f'c) MPa sigue la cita ya anotada en ese crate, ACI 318-19 §19.2.2.1.a. Página de otro libro: cita desconocida. No se copió un listado.
- Código de terceros copiado: ninguno. No se añadió ninguna dependencia.
- Validación: no hay un esperado numérico nuevo. El ensayo comprueba 250 nudos, 585 barras, 1 500 grados totales y 1 350 grados libres, y que el estático y el P-Delta terminan en estado ok. Los tiempos están en `docs/perf/MAM-1056-perfil-ruta-densa.md`.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK, SuiteSparse, CSparse, CHOLMOD, AMD, MUMPS ni PARDISO, ni de sus derivados, para este ticket. Esta sesión no abrió ese código ni esas salidas."
- Revisor clean-room: PENDING

## Qué se midió

El generador es `portal_250`. El desglose de la factorización está en el ensayo ignorado de `engine-api`. `perf` y `flamegraph` no estaban disponibles: DESCONOCIDO. El espectro modal no terminó: el total queda DESCONOCIDO. El reparto interno de la reducción densa, del primer tramo de P-Delta y del término cuadrático de la factorización (búsqueda, actualización y escritura por columnas) queda DESCONOCIDO.

## Pendiente

Revisión de J4. La mejora del núcleo no entra en este cambio. Si esa mejora cambia hashes publicados, hay que documentarlo y auditarlo; conservarlos no es criterio. `docs/DETERMINISM.md` y los goldens no se tocaron.
