# Bitácora clean-room · ENG-PERF-002

- Ticket / PR: MAM-1064. Ruta densa sin cambio de hashes. Rama `j2/mam-1064-ruta-densa`, en borrador contra main y apilada sobre #33 (MAM-1056, `63c3347d8f32d94dcb77a43eeab7e325ec8a08b5`). Esta bitácora no cierra MAM-107 y no aprueba un presupuesto de tiempo o memoria.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-10 (America/Guayaquil, ECT, UTC-5).
- Rol: implementador de las propuestas (2), (3), (4) y (5) del perfil MAM-1056. Quedan fuera la (1), el núcleo LDLᵀ, y la (6), el Jacobi del modal.
- Session: sesión limpia del 2026-11-10 (America/Guayaquil, ECT, UTC-5), distinta de la del 2026-11-04. Esta sesión no vio código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK, SuiteSparse, CSparse, CHOLMOD, AMD, MUMPS ni PARDISO, ni de sus derivados.
- Contrato implementado: ninguno nuevo. Se ejercitan `static.linear` y `pdelta` ya publicados, con el backend `reference.dense`.
- Fuentes permitidas: el código de este repositorio y la nota `docs/perf/MAM-1056-perfil-ruta-densa.md`. Página de otro libro: cita desconocida. No se copió un listado.
- Código de terceros copiado: ninguno. No se añadió ninguna dependencia.
- Validación: los resultados publicados no cambian. Una prueba compara el error de un NaN, un Inf y un −Inf en la matriz con ENG-E-0008 y el mensaje «Linear algebra: non-finite input or result.», el mismo que devuelve el commit de partida. Los tiempos están en `docs/perf/MAM-1064-ruta-densa.md`.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK, SuiteSparse, CSparse, CHOLMOD, AMD, MUMPS ni PARDISO, ni de sus derivados, para este ticket. Esta sesión no abrió ese código ni esas salidas."
- Revisor clean-room: PENDING

## Qué cambió

El estático de la fachada resuelve todos los casos con una sola factorización de K. P-Delta guarda el factor de la gravedad y no vuelve a factorizar K para la sustitución de Picard. La LDLᵀ de referencia revisa la finitud de la matriz una sola vez antes de eliminar, y ya no recorre todo el almacenaje al cerrar cada columna. El orden de las sumas y la secuencia de pivotes no se tocaron. El armado del estático deja M y Kg en su forma dispersa hasta que alguien las pida; modal y P-Delta siguen densificándolas.

`docs/contracts/error-codes-1.json` no se editó y sigue con 32 entradas. Las huellas de `engine_verification_r3.fuentes` en el índice de VAL-032 se actualizaron solo para `pdelta.rs` y `modal.rs`, porque esos archivos cambiaron. No son los 58 golden de stdout ni los 16 digestos de determinismo.

## Tiempos

Cinco repeticiones en `release`, mediana, en `portal_250`. El antes es `63c3347d8f32d94dcb77a43eeab7e325ec8a08b5`. El detalle está en `docs/perf/MAM-1064-ruta-densa.md`.

| Run | Antes | Después |
| --- | ---: | ---: |
| Estático, fases: factorización LDLᵀ | 2 872,946 ms | 1 729,017 ms |
| Estático, fases: reducción densa | 47,807 ms | 22,036 ms |
| Estático, `run` del generador | 5 996,616 ms | 1 797,286 ms |
| P-Delta, `run` del generador | 9 190,481 ms | 3 534,788 ms |

El reparto que queda dentro de la factorización de 1 729,017 ms queda DESCONOCIDO. El ajuste viejo del repaso por columna ya no describe una fracción de tiempo: el coeficiente de ese término salió negativo. El modal no se cambió; su espectro completo sigue DESCONOCIDO.

## Comprobaciones

Todas con `--offline --locked` cuando el comando es de Cargo. Salida 0 el 2026-11-10 (ECT):

- `cargo fmt --all -- --check`
- `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`
- `cargo test --offline --locked --workspace`
- `RUSTDOCFLAGS=-D warnings cargo doc --offline --locked --no-deps --workspace`
- `cargo check --offline --locked --workspace --target wasm32-unknown-unknown`
- `bash clean-room/check.sh origin/main`
- `bash scripts/changelog-check.sh origin/main`

Dentro de `cargo test`, pasaron `all_accepted_fixtures_match_schema_and_stdout_golden` (58 entradas), `all_sixteen_published_hashes_remain_unchanged` y `description_is_valid_deterministic_and_has_a_versioned_digest_snapshot`. Esos tres archivos de huellas no se editaron.

## Pendiente

Revisión de J4. La propuesta (1), el núcleo LDLᵀ, y la (6), el Jacobi, no entran en este cambio.
