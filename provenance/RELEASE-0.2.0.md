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
- Seguimiento del 2026-11-10 00:13 ECT: `crates/engine-wasm/package.json` pasa de `0.1.0` a `0.2.0`, la misma versión que `crates/engine-wasm/Cargo.toml`. Para J4: ese cambio no altera ninguna otra huella, digesto ni golden. Cero entradas. La única comprobación de ese número es la igualdad con la versión del `Cargo.toml` del crate en `crates/engine-wasm/js/test-package.mjs`. Las huellas del CLI, de `describe` y de `docs/DETERMINISM.md` no leen ese manifiesto.

## Reproducción de huellas

Árbol temporal, sin commit, a partir de `3724b036426b94d44ae3cdee536d1e0587b61f2c`. Se devolvió `engine_version` a `0.1.0` dejando en `0.1.0` solo la versión del paquete `engine-api`, su línea en `Cargo.lock` y el requisito de `engine-api` en `bindings/python/Cargo.toml`, para que `cargo` en modo `--locked` resolviera. Los demás crates siguieron en `0.2.0`. Los valores esperados se tomaron de `main` `9aeebcc000ca42c0f94b5aa41507b1da1e378743`.

```
git worktree add --detach "$ARBOL" 3724b036426b94d44ae3cdee536d1e0587b61f2c
# en $ARBOL, sin commit: version de engine-api, su línea de Cargo.lock
# y el requisito de bindings/python/Cargo.toml, de 0.2.0 a 0.1.0
git checkout 9aeebcc000ca42c0f94b5aa41507b1da1e378743 -- \
  docs/DETERMINISM.md \
  crates/engine-api/tests/facade.rs \
  crates/engine-cli/tests/golden/stdout-sha256.json \
  crates/engine-cli/tests/snapshots/describe-1.stdout.sha256
cargo metadata --offline --locked --format-version 1 --no-deps
cargo test --offline --locked -p engine-api --test facade valid_domain_and_all_declared_methods -- --exact
cargo test --offline --locked -p engine-api --test determinism all_sixteen_published_hashes_remain_unchanged -- --exact
cargo test --offline --locked -p engine-cli --test describe description_is_valid_deterministic_and_has_a_versioned_digest_snapshot -- --exact
cargo test --offline --locked -p engine-cli --test contract all_accepted_fixtures_match_schema_and_stdout_golden -- --exact
```

`cargo metadata` informó `engine-api` `0.1.0`. Resultados: 58/58 huellas de `crates/engine-cli/tests/golden/stdout-sha256.json`, 1/1 digesto de `describe-1.stdout.sha256` (`7a76a5bbfe58cb284e4cf6c9f64e8da4eb3c3854e1e582a6d7bfded3c358d049`), 11/11 filas de análisis de `docs/DETERMINISM.md` y 1/1 aserción de `facade.rs` (`engine_version` igual a `0.1.0`). Las cuatro pruebas salieron 0. El árbol temporal se eliminó sin commit.

Cargo.lock sha256 anterior `bc4eb03e44c6334c541248e73f0358584f40f1475c21e3aea1765a5c9b81fae4`, nuevo `facf8c4e97d8111d10bcba8cf008a0273109e6e191da3dcc2b3865bb801b400d`: subida de versión de los paquetes propios. El valor nuevo es el sha256 real del `Cargo.lock` después de integrar main y coincide con el cálculo de J4 anterior al merge, porque main no modificó ese archivo. `package_count` sigue en 83 y `registry_package_count` en 60. Integrar #24 y #25 no cambió huellas del CLI, el digesto de `describe` ni `docs/DETERMINISM.md`.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados, para este ticket. Esta sesión no abrió ese código ni esas salidas."
- Revisor clean-room: PENDING
