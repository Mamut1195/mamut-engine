# Bitácora clean-room · ENG-LA-008-FU

- Ticket / PR: issue #77 · MAM-451; rama `j2/fix-77-heap-peak`, desde
  `origin/main` (`d1249f1`).
- Autor(es): Codex gpt-6.1-sol high bajo dirección de J2, 2026-10-14
  America/Guayaquil.
- Rol: implementador.
- Contrato implementado: regresión de cotas de heap de `engine-linalg` para
  `MemoryBudget::factorize` y `MemoryBudget::generalized_eigen`.
- Fuentes permitidas usadas: únicamente código propio del repositorio,
  `crates/engine-linalg/tests/heap_peak.rs`, `provenance/ENG-LA-008-WIRE.md` y
  `provenance/TEMPLATE.md`. Sin red ni consulta de fuentes externas.
- Derivaciones propias: aislamiento y contabilidad descritos abajo.
- Código de terceros copiado: ninguno.
- Validación: medición propia de bytes de Layout, regresión de aislamiento entre
  hilos y estrés concurrente; verificaciones detalladas abajo.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Aislamiento y significado de la medición

El GlobalAlloc anterior contaba todos los hilos de libtest. El Mutex solo
serializaba las mediciones; no impedía reservas o liberaciones del harness en
otros hilos, que podían alterar el pico o hacer que LIVE quedara bajo la base.

Se conserva el harness estándar: el indicador thread-local
`Cell<bool>` con inicializador `const` habilita únicamente el hilo medidor.
Así `cargo test --workspace` sigue descubriendo y ejecutando la prueba con su
nombre y propaga los fallos de sus assertions mediante libtest. No hace falta
`harness = false` ni un main alternativo.

El indicador no requiere reservas ni destructor. El allocator consulta TLS con
`try_with`, ignora el acceso si TLS no está disponible y desactiva temporalmente
el indicador mientras actualiza los contadores para proteger frente a reentrada.
Estas actualizaciones usan solo Cell y atómicos, sin asignar memoria. Una guarda
RAII desactiva el indicador incluso al desenrollar un pánico. El Mutex conserva
la exclusión entre pruebas que usan los mismos contadores globales.

Se habilita el contador durante toda la prueba, incluidas la construcción y
liberación de entradas y resultados; cada operación conserva su base de bytes
vivos y reinicia su pico. Las entradas ya existentes quedan excluidas de la
medición por diferencia con esa base. Todas las restas de contadores usan
`saturating_sub`, también en liberaciones y cambios de tamaño de realloc.
La resta atómica de LIVE usa compare_exchange para conservar compatibilidad con
el MSRV 1.85 sin usar APIs deprecadas por el toolchain 1.99.0.

No se cambia ninguna cota, dimensión, fórmula de presupuesto ni assertion de la
regresión original: permanecen los órdenes 0, 1, 3, 8, 9, 64, 65 y 256, las
comprobaciones de pico y memoria retenida, y la exigencia de que la cota antigua
subestime al menos un pico. La regresión adicional crea otro hilo antes de
habilitar el contador, ejercita reservas, realloc creciente/decreciente y
liberaciones, y exige que LIVE y PEAK permanezcan exactamente intactos. Reinicia
PEAK con LIVE para que un pico de una prueba anterior no oculte interferencias.
Otro caso libera un Vec construido antes de medir: comprueba que, aunque LIVE
baje legítimamente de la base, pico y retenido medidos se saturen en cero.

## Verificación offline

Toolchain instalado: Rust/Cargo 1.99.0; se antepone
`/home/box/.rustup/toolchains/1.99.0-x86_64-unknown-linux-gnu/bin` a PATH porque
`/usr/bin` contiene 1.85.1. Todas las invocaciones de Cargo usan
`CARGO_TARGET_DIR=/workspace/.cargo-target` y `CARGO_NET_OFFLINE=true`; las que
aceptan ese flag usan además `--offline`. No se instala ningún componente.

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: OK.
- `cargo test --offline -p engine-linalg --test heap_peak -- --nocapture`: OK,
  las tres pruebas pasan.
- `cargo test --offline --workspace`: OK, incluidos doctests y las tres pruebas
  del contador; 19 ejecuciones completas en paralelo al estrés final.
- `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace`: OK.
- `cargo check --offline --workspace --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK.

## Estrés concurrente

Con el mismo entorno offline y CARGO_TARGET_DIR, se lanza en segundo plano un
bucle de `cargo test --offline --workspace` en otro proceso. En primer plano:

```bash
for attempt in {1..50}; do
    cargo test --offline -p engine-linalg --test heap_peak -- \
        --exact charged_bounds_cover_actual_factorize_and_eigen_heap_peaks \
        --nocapture || exit 1
done
```

El bucle del workspace se mantiene activo durante las 50 ejecuciones, no solo
durante la primera. Al terminar se señala su parada y se espera a que complete
su ejecución en curso, comprobando también su código de salida. Cada invocación
tiene su log separado; se comprueba que la prueba filtrada se ejecutó y que no
hubo fallos. Script local: `/tmp/me45-issue77-stress/run.sh`; logs de la versión
final: `/tmp/me45-issue77-final/`.

Resultado final del estrés: **50/50 OK**, con **19/19 ejecuciones completas del
workspace OK**. Las tres pruebas del contador se ejecutaron también en cada
invocación del workspace. Los seis picos reportados son idénticos en las 50
ejecuciones; se mantienen las assertions para todos los órdenes, los picos,
la memoria retenida y la insuficiencia de la cota antigua.

Picos reportados (bytes de Layout), sin modificar cotas:

| n | Pico factorize | Carga factorize | Cota antigua | Pico eigen | Carga eigen |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 1856 | 2088 | 1344 | 4224 | 5800 |
| 64 | 100864 | 101096 | 68096 | 263168 | 331496 |
| 256 | 1583104 | 1583336 | 1058816 | 4198400 | 5257448 |
