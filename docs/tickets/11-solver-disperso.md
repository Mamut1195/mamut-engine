# 11 · Solver directo disperso simétrico

- Estado: diseño escrito. La implementación no empieza en este archivo.
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

La implementación vendrá en los PR de abajo, después de que Jonathan decida las dependencias. Lanczos, MAM-965 y MAM-995 no se implementan aquí. El diseño deja el factor de `K − σM` reusable para los dos primeros usos: varias sustituciones y el conteo de Sturm.

## Criterios medibles

- Perfil V2b: hasta 60 000 GDL libres. Por encima, rechazo antes de reservar.
- El modelo de 10 pisos con losas, de 26 460 GDL libres, se resuelve en el ensayo nativo en 60 s o menos, con pico de proceso de 2 GiB o menos. Esas dos cotas son el criterio de MAM-67 para ese modelo. No son un presupuesto de release y no cierran MAM-107.
- El informe del ensayo publica `n`, el número de entradas almacenadas de K, el de L, el ordenamiento, el tiempo de cada fase y la memoria.
- El residual relativo de los casos bien condicionados queda por debajo de 1e-12, con la norma escrita en el diseño.
- Varias cargas usan una sola factorización.
- La inercia publica los signos de D, incluidos los bloques 2×2, para el control de Sturm.
- Un mecanismo sale como ENG-E-0006 con nudo y GDL.
- El mismo caso pequeño produce los mismos bits en dos ejecuciones y entre nativo y wasm, en las condiciones de `docs/DETERMINISM.md`.
- `docs/contracts/error-codes-1.json` sigue en 32 entradas mientras Jonathan no acepte un código nuevo.

## Fases

Cada fase es un PR pequeño, con ensayos, fragmento de changelog y bitácora. El revisor queda en PENDING. Cargo sigue `--offline --locked`. Ninguna fase añade una dependencia por su cuenta.

1. Este PR: documentos. Sin código.
2. Matriz simétrica dispersa, triangular, y su construcción desde tripletas. Rechazo de tamaño antes de reservar. Sin factorizar.
3. Reducción de restricciones a esa matriz, sin pasar por densa. En un modelo pequeño, las entradas coinciden con la reducción densa de hoy.
4. Análisis simbólico: permutación, árbol de eliminación y conteos. La estimación de memoria rechaza con ENG-E-0013 antes de la reserva numérica.
5. Factorización numérica definida, varias columnas y residual.
6. Factorización indefinida, bloques 2×2 e inercia. El conteo de Sturm de un par diagonal disperso coincide con el conteo denso ya existente.
7. Pivote nulo traducido a nudo y GDL con el traductor que ya tiene el ensamblaje.
8. `LinearSystem` puede guardar K, M y Kg reducidas en disperso. Si esa ruta rechaza, no llama a la densa.
9. Factor de `K − σM` con sustitución reusable e inercia. Sin Lanczos.
10. Ensayo nativo del modelo de 10 pisos y paridad de bits en la suite pequeña. En wasm32, un caso que no cabe se rechaza con ENG-E-0013.

## Hecho cuando

- Están el diseño, la bitácora y el fragmento de changelog.
- No hay cambios de código, de manifiestos ni de `error-codes-1.json`.
- El revisor clean-room queda en PENDING.
- MAM-107 permanece abierto.

## Riesgos

- Una factorización propia puede no cumplir los 60 s. El ensayo de la fase 10 lo mide. No se inventa el tiempo en el diseño.
- Una crate puede ser más rápida y a la vez incumplir la igualdad de bits, la inercia sin alterar pivotes, o la licencia. La decisión es de Jonathan.
- El diafragma rígido puede llenar el factor. La estimación simbólica tiene que verlo antes de reservar.
- Cambiar el backend de los casos pequeños movería los hashes de `docs/DETERMINISM.md`. Esos casos siguen en la ruta densa hasta un PR que publique los hashes nuevos.
