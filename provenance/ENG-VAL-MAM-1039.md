# Bitácora clean-room · ENG-VAL-MAM-1039

- Ticket / PR: MAM-1039. Seguimiento de VAL-037 y VAL-038 (MAM-850, pull request 23, ya fusionado). Rama `j2/mam-1039-seguimiento-val-037`. Base `9aeebcc000ca42c0f94b5aa41507b1da1e378743`. Notas bajas de la r2 de J4. No se mueve el head de ese pull request. El número de este pull request queda DESCONOCIDO en esta bitácora: se abre después del push.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-09 (América/Guayaquil, ECT, UTC-5). El commit que introduce esta bitácora registró 2026-11-10T05:00:01Z, esto es 2026-11-10 00:00:01 ECT. Esa hora sale de git y no se asignó a mano.
- Sesión: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO. Esta sesión no consultó código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados.
- Rol: especificador. Sin código de producción.
- Contrato implementado: ninguno. Cambia la redacción, el nombre de un ensayo y las erratas. No cambia tolerancias ni valores esperados.
- Fuentes permitidas: `docs/validation-report/casos/deriv/VAL-037.tex`; `docs/validation-report/casos/VAL-037.tex`; `docs/validation-report/casos/VAL-038.tex`; `provenance/ENG-VAL-MAM-850.md`; `provenance/TEMPLATE.md`; `crates/engine-api/tests/val037_val038.rs`; `changelog.d/README.md`. No entró ningún libro ni PDF.
- Derivaciones propias: ninguna nueva. Se corrige la redacción ya escrita en la derivación de VAL-037.
- Código de terceros copiado: ninguno. No se añadió ninguna dependencia.
- Validación: la conclusión del presupuesto de VAL-037 no cambia. El criterio sigue siendo el operador de Hermite `50/47` con la banda ya escrita.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados, para este ticket."
- Revisor clean-room: PENDING

## Notas de la r2

### Causa del hueco

La redacción atribuía el hueco de la primera corrida al acortamiento por gravedad. En pDelta ese acortamiento no altera B2. La causa es la flexibilidad axial de las columnas por el vuelco. Quedó así en la derivación y en la bitácora de MAM-850. La conclusión del presupuesto no cambia.

### Escala de la gravedad

El ensayo de sensibilidad se llama `gravity_scale_sensitivity_stays_outside_the_1e_6_band`. La variante gravedad por 5/6 escala toda la Kg. No es la mutación 6/5 a 1 (36 a 30). Coincide con `1/(1-r)` solo porque los giros son casi nulos. La mutación real la detecta el ensayo principal `three_story_hermite_b2_stays_within_1e_6`. «L x 2» es aritmética de la forma cerrada y no una corrida del motor. No se cambiaron tolerancias ni valores esperados.

### Corrida local anterior al preregistro

Si el modelo se ejecutó en local antes del push del preregistro `5696f82a806b05ab9745e2557dfac2d2b0e04cd4`: DESCONOCIDO. La fecha de committer de ese objeto, leída en el forge, es 2026-11-10T04:34:24Z, esto es 2026-11-09 23:34:24 ECT. Esta sesión no asignó esa hora. Esa hora no prueba si hubo una corrida local anterior al push.

### Erratas

«Los fuentes» pasa a «Las fuentes» en las fichas de VAL-037 y VAL-038. La línea de sesión de la bitácora de MAM-850 queda en español y nombra OpenSees, Frame3DD, Stabileo, CalculiX y LAPACK en la declaración de no exposición.

## Comandos de esta entrega

- `cargo fmt --all -- --check`: exit 0
- `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`: exit 0
- `cargo test --offline --locked --workspace`: exit 0. Dentro, `val037_val038`: `three_story_hermite_b2_stays_within_1e_6` y `gravity_scale_sensitivity_stays_outside_the_1e_6_band` en verde.
- `RUSTDOCFLAGS=-D warnings cargo doc --offline --locked --no-deps --workspace`: exit 0
- `cargo check --offline --locked --workspace --target wasm32-unknown-unknown`: exit 0
- `cargo build --offline --locked --release --target wasm32-unknown-unknown -p engine-wasm`: exit 0
- `bash clean-room/check.sh origin/main`: exit 0 en el árbol de esta entrega
- `bash scripts/changelog-check.sh origin/main`: exit 0. Changelog: OK (1 new fragment).

Cargo imprimió, en clippy, doc, el check de wasm y el build de wasm, un aviso ya presente del manifiesto de `engine-cli`: la dependencia `toml` lleva metadatos de semver. Este cambio no toca ese manifiesto. `docs/contracts/error-codes-1.json` no se editó y sigue con 32 entradas. Esta sesión no compiló LaTeX. `lualatex` y `latexmk` no están en el PATH.

## Lo que queda DESCONOCIDO

- Modelo del agente: DESCONOCIDO.
- Si el modelo de VAL-037 se ejecutó en local antes del push de `5696f82a806b05ab9745e2557dfac2d2b0e04cd4`: DESCONOCIDO.
- Número de este pull request al escribir esta bitácora: DESCONOCIDO.
- Orden de trabajo distinto del orden de los commits de esta rama: DESCONOCIDO. No se fijaron fechas de autor ni de committer a mano.
