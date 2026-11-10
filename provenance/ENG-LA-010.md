# Bitácora clean-room · ENG-LA-010

- Ticket / PR: MAM-1055, fase 2 de MAM-67. Rama `j2/mam-1055-matriz-dispersa`. ENG-LA-009 ya existía; este identificador es el siguiente libre. No cierra MAM-67 ni MAM-107 y no aprueba un presupuesto de tiempo o memoria.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-10 (America/Guayaquil, ECT, UTC-5).
- Rol: implementador. La ficha `docs/tickets/15-matriz-dispersa.md` se escribió antes del código.
- Session: sesión limpia del 2026-11-10 (America/Guayaquil, ECT, UTC-5), distinta de la del 2026-11-04. Esta sesión no vio código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados. Tampoco vio código ni salidas de SuiteSparse, CSparse, CHOLMOD, AMD, MUMPS ni PARDISO, ni de sus derivados.
- Contrato implementado: tipo interno `SymCscMatrix` en `engine-linalg`. No cambia `LinearSystem`, la ruta densa, `SparseMatrix` ni el catálogo de errores.
- Fuentes permitidas: el diseño de MAM-67 en el PR #27 (leído con `git show`, sin copiarlo a esta rama), el código ya presente de `engine-linalg` y la ficha 15. No se abrió un libro. Página de libro: cita desconocida. No se copió un listado.
- Derivación propia: la de abajo. Los esperados de los ensayos salen de esa cuenta, no de otro programa.
- Código de terceros copiado: ninguno. No se añadió ninguna dependencia y no se tocó Cargo.
- Comparación con oráculo (repositorio hermano, informativa): no.
- Declaración: "No consulté código fuente ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni de sus derivados, para este ticket. Esta sesión no abrió ese código ni esas salidas. Tampoco consulté código ni salidas de SuiteSparse, CSparse, CHOLMOD, AMD, MUMPS ni PARDISO, ni de sus derivados."
- Revisor clean-room: PENDING

## Decisiones de esta fase

La opción A ya estaba tomada: implementación propia, sin crates nuevas. El rechazo usa ENG-E-0013. `docs/contracts/error-codes-1.json` no se editó y sigue con 32 entradas. El tipo no convierte unidades. El corte entre la ruta densa y la dispersa no se elige aquí.

El diseño queda alineado con esta fase: triángulo superior, en la columna `j`, filas `i ≤ j`, diagonal incluida. La otra mitad no se materializa.

`SparseMatrix` suma duplicados en el orden de entrada. Aquí la suma es canónica: dentro de cada lado, de menor a mayor según `f64::total_cmp`. Dos órdenes de inserción dejan los mismos bits. No hay `HashMap`. Si las dos mitades vienen informadas, cada una se suma por su lado y las sumas tienen que ser iguales; se guarda la superior. No se suman entre sí.

La cota, antes de reservar, usa solo `n` y el número de tripletas: punteros `(n + 1)`, como máximo `nnz + n` índices y otros tantos valores, y un borrador de un registro por tripleta. La ordenación es en el sitio. Si el orden, la cota o el desborde de `usize` superan el límite que pasa el llamador, ENG-E-0013 y no se reserva. El límite no es 2048 ni 60000.

## Derivación de los ensayos

Para `0.1`, `0.2` y `0.3`, `total_cmp` deja ese mismo orden. `(0.1 + 0.2) + 0.3` es el binary64 `0x3fe3333333333334`. El pliegue inverso `(0.3 + 0.2) + 0.1` es otro valor. Los dos órdenes de inserción guardan el primero.

Para `1e16`, `-1e16` y `1`, el orden total es `-1e16`, luego `1`, luego `1e16`. La unidad está por debajo del ulp 2 de `1e16`, así que se pierde, y `-1e16 + 1e16` deja `+0`. Sumar en el orden `1e16`, `-1e16`, `1` dejaría `1`. Los dos órdenes de inserción guardan `+0`.

Matriz de ensayo, con las dos mitades ya iguales a `0.5` en la posición `(0, 1)`:

```
2    0.5   0
0.5  3    -1
0   -1     4
```

Con `x = (1, 2, -1)` el producto es `(3, 7.5, -6)`, porque `2·1 + 0.5·2 = 3`, `0.5·1 + 3·2 + 1 = 7.5` y `-1·2 + 4·(-1) = -6`. La densa se escribe a mano y se multiplica por filas. Coincide con el producto disperso.

Una sola tripleta inferior `(1, 0, 5)` en orden 2 guarda punteros `(0, 1, 3)`, filas `(0, 0, 1)` y valores `(0, 5, 0)`: diagonal explícita donde nadie aportó, y el 5 una sola vez.

## Alineación del diseño

Estos cambios de documentación vienen de la r2 de J4 sobre #27 y de la decisión de J2. El almacenamiento de esta fase se queda como está implementado.

- Triángulo superior, filas `i ≤ j`, y suma canónica con `f64::total_cmp`. Motivo de J2: dos órdenes de inserción dejan los mismos bits.
- La fuente de las metas de velocidad es la Cursor Page «MAMUT Engine: camino a superar a OpenSees». Esa Page no nombra a Q3: cita desconocida.
- El pórtico de 250 nudos queda en 1.350 GDL, que es la cifra de esa Page.

Lo mismo quedó escrito en `docs/design/MAM-67-solver-disperso.md`, en la ficha 11 y en `provenance/ENG-LA-008-DISENO.md`. La frase que sigue describiendo el CSR general suma en el orden de entrada: ese tipo no cambió.

En esta ronda, después de integrar `main`: `cargo fmt --all -- --check` exit 0; `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` exit 0; `cargo test --offline --locked --workspace` exit 0; `bash clean-room/check.sh origin/main` Clean-room OK; `bash scripts/changelog-check.sh origin/main` Changelog OK, un fragmento nuevo.

## Verificación

`cargo fmt --all -- --check`: exit 0. `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`: exit 0. `cargo test --offline --locked --workspace`: exit 0. `cargo doc --offline --locked --no-deps --workspace` con `RUSTDOCFLAGS=-D warnings`: exit 0. `cargo check --offline --locked --workspace --target wasm32-unknown-unknown`: exit 0. `bash clean-room/check.sh origin/main`: Clean-room OK, también con esta bitácora ya en el árbol. `bash scripts/changelog-check.sh origin/main`: Changelog OK, un fragmento nuevo. `docs/contracts/error-codes-1.json` sigue con 32 entradas. `docs/DETERMINISM.md` no se modificó. Ningún golden cambió.
