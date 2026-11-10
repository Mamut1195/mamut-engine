# Bitácora clean-room · release 0.2.0

- Ticket / PR: release 0.2.0, borrador hacia `main`. Rama `j2/release-0.2.0`. No se crea tag ni release de GitHub en este paso.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-10 (America/Guayaquil, ECT, UTC-5). La sesión empezó el 2026-11-09 a las 23:49 ECT.
- Rol: documentador de la release. El análisis numérico no cambia. Cambian el número de versión de los crates del workspace y del binding Python, el changelog publicado, las menciones de versión que describen el estado actual y la prosa de capacidades, Actions y limitaciones.
- Session: sesión limpia abierta el 2026-11-09 a las 23:49 ECT y cerrada el 2026-11-10 (America/Guayaquil, ECT, UTC-5). Esta sesión no vio código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados.
- Contrato implementado: ninguno nuevo. `engine_api::version()` sigue leyendo `CARGO_PKG_VERSION`. Esta release deja ese número en `0.2.0`.
- Fuentes permitidas: `changelog.d/README.md`, `scripts/changelog-assemble.sh`, `scripts/changelog-check.sh`, `.github/workflows/rust.yml`, `.github/workflows/clean-room.yml`, `docs/V1.md`, `crates/engine-api/src/methods.rs`, `crates/engine-api/src/run.rs`, `crates/engine-api/src/lib.rs`, `crates/engine-api/src/limits.rs`, `schemas/engine-input/1.json`, `crates/engine-linalg/README.md` y `docs/validation-report/secciones/limitaciones.tex`.
- Derivaciones propias: la sección `0.2.0` del changelog sale de los fragmentos ya versionados, con la fecha por fijar al etiquetar. Los fragmentos no se borran. Las citas de edición que no se pudieron verificar no se reescribieron; quedan listadas en la descripción del PR como cita desconocida.
- Código de terceros copiado: ninguno. No se añadió ninguna dependencia. No se regeneró la resolución del lock: solo el número de versión de los paquetes propios.
- Validación: los comandos y sus salidas quedan en la descripción del PR. `docs/contracts/error-codes-1.json` no se edita.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados, para este ticket. Esta sesión no abrió ese código ni esas salidas."
- Revisor clean-room: PENDING
