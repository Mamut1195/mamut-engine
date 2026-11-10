# 11 · Solver directo disperso simétrico

- Estado: diseño escrito. Decisiones de J0 registradas. La implementación no empieza en este archivo.
- Relación: MAM-67, ENG-LA-008. La bitácora es `provenance/ENG-LA-008-DISENO.md`. El revisor clean-room queda en PENDING. Este ticket no cierra MAM-107 y no aprueba un presupuesto de release.
- Diseño: [MAM-67-solver-disperso.md](../design/MAM-67-solver-disperso.md).
- Reglas comunes: [README.md](README.md).

## Qué ya existe

En el commit `9aeebcc000ca42c0f94b5aa41507b1da1e378743`:

- `ReferenceBackend` factoriza y resuelve. `SymMatrix` es densa.
- `crates/engine-assembly/src/system.rs` guarda la K, la M y la Kg globales en CSR y, en `reduced()`, las convierte a densas.
- `FaerBackend::new` devuelve ENG-E-0009. La feature `faer` no arrastra ninguna crate. `faer` no está en `vendor-crates/` ni en `Cargo.lock`.
- No hay ordenamiento por grado mínimo aproximado, ni LDLᵀ disperso, ni inercia dispersa.
- `FactorInfo`, el residual de los ensayos y `translate_solver_error` ya existen. El diseño los reutiliza.

## Qué construir

Solo el diseño, en este corte. El diseño fija la ruta dispersa de principio a fin: ensamblaje, restricciones, K, M y Kg reducidas, análisis simbólico y factorización numérica. Esa ruta no se convierte en silencio a la ruta densa.

La implementación vendrá en los PR de abajo. Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: la opción A, implementación propia en sala limpia, sin crates nuevas. Lanczos, MAM-965 y MAM-995 no se implementan aquí. El diseño deja el factor de `K − σM` reusable para los dos primeros usos: varias sustituciones y el conteo de Sturm. La fase 2 es MAM-1055, con su propio PR.

## Criterios medibles

- Perfil V2b: hasta 60 000 GDL libres. Por encima, rechazo antes de reservar.
- El modelo de 10 pisos con losas, de 26 460 GDL libres, se resuelve en el ensayo nativo en 60 s o menos, con pico de proceso de 2 GiB o menos. Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: esas dos cotas valen solo para MAM-67 y no cierran MAM-107.
- El pórtico 3D de 250 nudos, unos 1 500 GDL libres, entra en el criterio de salida de la opción A: estático en 54 ms o menos, modal en 0,58 s o menos y P-Delta en 88 ms o menos. Si la fase 10 no llega, se reabren B y C con números medidos. Por qué la ruta densa tarda 6,48 s en ese pórtico es MAM-1056.
- La fuente de esas tres metas es la página de J2 «Ruta para superar a OpenSees», https://cursor.com/canvas/store-2a8ecc30-58b0-4b46-8a05-3be45ddfeb2e/19df7ff1-4c3f-440d-af09-e4dfbeec48ec, con mediciones de rendimiento de Q3 en caja negra.
- El informe del ensayo publica `n`, el número de entradas almacenadas de K, el de L, el ordenamiento, el tiempo de cada fase y la memoria.
- El residual relativo de los casos bien condicionados queda por debajo de 1e-12, con la norma escrita en el diseño.
- Varias cargas usan una sola factorización.
- La inercia publica los signos de D, incluidos los bloques 2×2, para el control de Sturm.
- Un mecanismo sale como ENG-E-0006 con nudo y GDL.
- El mismo caso pequeño produce los mismos bits en dos ejecuciones y entre nativo y wasm, en las condiciones de `docs/DETERMINISM.md`.
- `docs/contracts/error-codes-1.json` sigue en 32 entradas. El rechazo de capacidad reutiliza ENG-E-0013. No hay ENG-E-0015.

## Fases

Cada fase es un PR pequeño, con ensayos, fragmento de changelog y bitácora. El revisor queda en PENDING. Cargo sigue `--offline --locked`. Ninguna fase añade una dependencia por su cuenta.

1. Este PR: documentos. Sin código.
2. MAM-1055, en su propio PR: matriz simétrica dispersa, triangular, y su construcción desde tripletas. Rechazo de tamaño antes de reservar. Sin factorizar.
3. Reducción de restricciones a esa matriz, sin pasar por densa. En un modelo pequeño, las entradas coinciden con la reducción densa de hoy.
4. Análisis simbólico: permutación, árbol de eliminación y conteos. La estimación de memoria rechaza con ENG-E-0013 antes de la reserva numérica.
5. Factorización numérica definida, varias columnas y residual.
6. Factorización indefinida, bloques 2×2 e inercia. El conteo de Sturm de un par diagonal disperso coincide con el conteo denso ya existente.
7. Pivote nulo traducido a nudo y GDL con el traductor que ya tiene el ensamblaje.
8. `LinearSystem` puede guardar K, M y Kg reducidas en disperso. Si esa ruta rechaza, no llama a la densa.
9. Factor de `K − σM` con sustitución reusable e inercia. Sin Lanczos.
10. Ensayo nativo del modelo de 10 pisos (60 s y 2 GiB) y del pórtico 3D de 250 nudos (estático 54 ms, modal 0,58 s, P-Delta 88 ms). Ahí se mide el umbral entre la ruta densa y la dispersa. Conservar los hashes publicados no fija ese umbral: si cambian, se documentan y J4 los audita. En wasm32, un caso que no cabe se rechaza con ENG-E-0013. Si las metas no se cumplen, se reabren B y C con los números medidos.

## Hecho cuando

- Están el diseño, la bitácora y el fragmento de changelog.
- No hay cambios de código, de manifiestos ni de `error-codes-1.json`.
- El revisor clean-room queda en PENDING.
- MAM-107 permanece abierto.

## Riesgos

- Una factorización propia puede no cumplir los 60 s, los 2 GiB o las metas del pórtico de 250 nudos. El ensayo de la fase 10 lo mide. Si no llega, se reabren B y C con esos números. No se inventa el tiempo en el diseño.
- El diafragma rígido puede llenar el factor. La estimación simbólica tiene que verlo antes de reservar.
- El umbral densa/dispersa sale de la medición de la fase 10, no de conservar los hashes. Si los hashes cambian, se documentan y J4 los audita.
