# 16 · Perfil de la ruta estática densa

- Estado: abierto. Puede empezar ya, en paralelo con el resto.
- Relación: MAM-1056. Esta ficha no cierra MAM-107 y no aprueba un presupuesto de tiempo o memoria.
- Reglas comunes: [README.md](README.md).
- Número: en las ramas remotas `j2/*` consultadas al abrir la ficha están el 11 (`j2/mam-67-diseno-solver-disperso`) y el 12 (`j2/mam-1045-errores-de-referencia`). El 13, el 14 y el 15 no aparecen en esas ramas. El 16 está libre y es el que usa esta ficha.

## Qué ya existe

La fachada `run` resuelve el estático lineal con el backend denso de referencia: arma el modelo, reduce apoyos y escribe `results/1`. La factorización simétrica con pivotes de 1×1 y 2×2 vive en ese backend. El modal denso pasa por Cholesky de la masa y por rotaciones de Jacobi. P-Delta reutiliza el estático de gravedad y vuelve a factorizar.

Los pórticos de `engine-bench` son de 64, 144 y 256 nudos, con diafragma. No hay uno de 250 nudos sin diafragma. El límite de `engine-input/1` es 256 nudos y 1024 barras.

## Qué medir

Un pórtico 3D de 250 nudos, del orden de 1 500 grados de libertad, en la ruta estática densa. Hay que separar lectura y validación, ensamblaje, reducción de restricciones, factorización, sustitución, recuperación de esfuerzos y serialización. Cinco repeticiones en `--release` y la mediana. Modal y P-Delta, con las mismas fases, si el modelo lo permite.

Las metas de esta ruta, y la única cifra externa de este ticket, son: estático a lo sumo 54 ms, modal a lo sumo 0,58 s y P-Delta a lo sumo 88 ms. Esta fase no cambia el motor para alcanzarlas. Lo que no se mida queda como DESCONOCIDO. `perf` y `flamegraph` se usan solo si ya están en la máquina; si no, esa parte queda como DESCONOCIDO.

## Qué construir

Un generador propio y determinista del pórtico, en `engine-bench`, sin archivos ni salidas de otro programa. Un ensayo reproducible que no altere resultados. La nota `docs/perf/MAM-1056-perfil-ruta-densa.md`, la propuesta de mejora en sala limpia ordenada por impacto, el fragmento de changelog y la bitácora `provenance/ENG-PERF-001.md`.

Si una mejora futura cambia hashes publicados, se documenta y J4 lo audita. Conservar los hashes no es un criterio. En este PR los hashes, los goldens y `docs/DETERMINISM.md` no cambian.

## Hecho cuando

- La ficha está en el índice y el 16 sigue siendo el número usado.
- El generador y el ensayo están en el repositorio y el ensayo no corre en `cargo test` salvo que se pida de forma explícita.
- La nota de perfil trae la máquina, el comando y la mediana por fase.
- La propuesta dice qué hashes publicados cambiarían.
- El revisor clean-room queda en PENDING.
