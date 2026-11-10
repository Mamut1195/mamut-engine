# MAM-1056 · Perfil de la ruta densa

Medición de un pórtico 3D de 250 nudos. Esta nota no cambia el motor.
La cifra externa del ticket, 6,48 s para la ruta estática, no se usa como
oráculo: los tiempos de abajo son de esta máquina. Las metas del ticket son
estático ≤ 54 ms, modal ≤ 0,58 s y P-Delta ≤ 88 ms.

## Máquina

- SO: Ubuntu 24.04.5 LTS, Linux 6.12.113+, x86_64.
- CPU: el hipervisor publica `Intel(R) Xeon(R) Processor`, 4 CPU, 2400 MHz,
  bogomips 4800. El modelo comercial más fino queda DESCONOCIDO.
- Compilador: rustc 1.99.0 (b940084d7 2026-09-28), cargo 1.99.0.
- Perfil: `release` de Cargo, sin `RUSTFLAGS`.
- `perf` y `flamegraph` no están en la máquina. Esa vía queda DESCONOCIDO.

## Modelo

Generador `engine_bench::portal_250`, identificador `rc3d-250`. Sin azar y
sin diafragma.

- Retícula de 5 × 5 líneas, vano 6 m, 9 pisos de 3 m. Con la base son 10
  niveles y 250 nudos. Barras: 585 (225 columnas y 360 vigas).
- Grados totales: 1 500. Base empotrada en los 25 nudos inferiores.
  Grados libres medidos: 1 350.
- Hormigón f'c = 28 MPa. E = 4700·sqrt(f'c) MPa, cita ya usada en este
  crate (ACI 318-19 §19.2.2.1.a). ν = 0,20. G = E / (2 (1 + ν)).
  Densidad 2 400 kg/m³.
- Columna cuadrada de 0,70 m. Viga de 0,30 m por 0,70 m. J de Saint-Venant
  del rectángulo, la misma expresión que los pórticos oficiales del crate.
- Masa de piso = área · (0,15 m · 2 400 kg/m³ + 3 000 N/m² / 9,80665 m/s²),
  con planta de 24 m × 24 m. Cada nudo de piso recibe m/N en Ux, Uy y Uz y
  m·luz²/(12·N) en Rx, Ry y Rz. La base no lleva masa. Así el modal denso
  no condensa filas de masa nula.
- Casos `G` (peso, Uz negativo) y `EX` (10 % de ese peso en Ux). P-Delta
  usa esos casos, tolerancia 1e-8, 30 iteraciones, escalas 0,01 m y
  0,001 rad, las mismas cifras ya escritas en
  `crates/engine-bench/fixtures/capabilities/pdelta.json`.
- El modal pide 1 modo y masa `lumped` en X. El backend denso igual calcula
  el espectro completo antes de quedarse con ese modo.

El desglose fino vive en `crates/engine-api` porque ese crate no puede
depender de `engine-bench`. Construye la misma retícula. El `run` del
generador y el del desglose quedan a 5,82 s y 5,79 s.

## Comandos

Desde la raíz del repositorio, cinco repeticiones, mediana del valor
central tras ordenar:

```sh
cargo test --offline --locked --release -p engine-api --lib perfil_ruta_densa -- --ignored --nocapture
cargo test --offline --locked --release -p engine-bench --test portal_250 perfil_run_del_generador -- --ignored --nocapture
```

`cargo test` sin `--ignored` no ejecuta esas corridas. El control rápido
del generador sí entra en la suite.

## Estático

Orden libre 1 350. Todos los pivotes medidos fueron de 1×1
(`uses_2x2 = false` en las cinco repeticiones).

| Fase | Mediana | Qué incluye |
| --- | ---: | --- |
| Lectura: parse | 1,668 ms | `json::parse` del documento ya armado |
| Lectura: huella | 4,213 ms | SHA-256 RFC 8785 |
| Lectura: decode | 2,528 ms | paso a `Execution` |
| Validación | 0,654 ms | `validation` |
| Construcción de elementos | 0,943 ms | `build_members` |
| Ensamblaje elemental | 1,397 ms | rigidez y cargas de cada barra |
| Sistema global y reducción densa | 47,091 ms | `from_model_with_diaphragms`. El reparto entre el CSR y las cuatro reducciones densas (K, M concentrada, M consistente y Kg) queda DESCONOCIDO |
| Factorización LDLᵀ | 2 781,752 ms | una factorización de K reducido, orden 1 350 |
| Segundo miembro reducido | 0,320 ms | los dos casos, suma |
| Sustitución | 10,474 ms | los dos casos, suma |
| Solver de biblioteca, una factorización | 2 777,426 ms | `StaticLinear::run` de los dos casos |
| Recuperación de esfuerzos | 5,543 ms | los dos casos, estaciones por defecto |
| Serialización canónica | 7,501 ms | segunda pasada de `canonical_value` sobre el documento ya emitido |
| `run` completo, dos casos | 5 818,857 ms | generador `portal_250`, fachada |

La fachada no reutiliza el factor. Cada caso vuelve a factorizar:

| Tramo del `run` del generador | Mediana |
| --- | ---: |
| parse | 8,689 ms |
| model | 0,639 ms |
| assembly | 52,403 ms |
| case 0 | 2 833,429 ms |
| case 1 | 2 812,761 ms |
| combinations | 30,983 ms |
| serialization (solo el callback) | 0,140 ms |

El tramo `combinations` no combina casos: en el fuente, el JSON de los
casos se arma entre el último `case` y ese callback. 30,983 ms es ese
armado. El callback `serialization` es anterior a la canonización; esa
canonización está medida aparte, en 7,501 ms.

La sustitución de 10,474 ms y la recuperación de 5,543 ms son la suma de
los dos casos. El reparto de cada caso no se guardó: queda DESCONOCIDO.
El dominio de la ruta es la factorización, repetida una vez por caso.

## Ajuste dentro de la factorización

El callback de la factorización avisa al inicio y cada 15 columnas. Con
pivotes solo de 1×1 el calendario coincide con el fuente (90 marcas para
n = 1 350). Se ajustó `dt = a·Σ(n−k)² + b·15·n²` en los bloques desde la
columna 15. Mediana de R² = 0,978. Mediana de la fracción `a` sobre
`a+b` = 0,672. El otro tercio escala como un repaso de la matriz completa
en cada columna, que es lo que hace `finite` sobre todo el almacenaje.

Ese ajuste no separa tres cosas que viven en el término cuadrático: la
búsqueda del mayor extradiagonal, la actualización del bloque activo y la
escritura simétrica a[j·n+i], que recorre la columna. Esas tres fracciones
quedan DESCONOCIDO. Las primeras 15 columnas, el equilibrado y las copias
previas tampoco entran en el ajuste; su tiempo propio queda DESCONOCIDO y
ya está dentro de los 2 781,752 ms.

No hay `Vec<Vec<_>>` en el factor: el almacenaje es un `Vec` plano por
filas. No hay `debug_assert` en ese archivo. `finite` se ejecuta también
en `release`.

## Modal

Misma retícula, masa en los 1 350 grados libres. Cinco arranques cortados
a los 8 s, o a 100 000 callbacks.

| Fase | Mediana |
| --- | ---: |
| Construcción y ensamblaje elemental | 2,306 ms |
| Sistema y reducción | 48,147 ms |
| Espectro | DESCONOCIDO. Prefijo medido 8 002,372 ms, sin terminar |
| Recuperación de modos y serialización | DESCONOCIDO. No se alcanzó |

En el prefijo hubo unas 7 700 llamadas al callback. La mediana del hueco
entre llamadas es 0,30 ms. El hueco medio, 8 002 ms / 7 700 llamadas, es
cerca de 1,04 ms: la cola pesa más que la mediana. Qué parte de esas
llamadas es Cholesky, producto o Jacobi queda DESCONOCIDO: el callback no
dice la etapa. El fuente busca el mayor extradiagonal en cada rotación de
Jacobi y admite hasta 100·n·n rotaciones.

## P-Delta

Cinco `run` del generador. Las cinco terminaron en estado ok, con cinco
pasos de Picard (callbacks 0 a 4).

| Fase | Mediana |
| --- | ---: |
| parse | 8,353 ms |
| validación (`model`) | 0,641 ms |
| assembly | 47,376 ms |
| `gravity` | 2 899,804 ms |
| `pdeltaIteration` 0 | 5 930,110 ms |
| `pdeltaIteration` 1 | 4,769 ms |
| `pdeltaIteration` 2 | 4,632 ms |
| `pdeltaIteration` 3 | 4,390 ms |
| `pdeltaIteration` 4 | 4,572 ms |
| callback `serialization` | 1,553 ms |
| total | 8 852,465 ms |
| Recuperación de esfuerzos de barra | NO EJECUTADA. La fachada escribe `members` vacío |

`gravity` junta el alta de productores geométricos, la factorización de K
de la gravedad y su sustitución. El reparto interno queda DESCONOCIDO.
`pdeltaIteration` 0 junta, en este orden del fuente, Kg(u), la reducción
de K+Kg, la factorización de inercia, la segunda factorización de K y la
primera sustitución de Picard. El reparto interno queda DESCONOCIDO. Los
pasos 1 a 4 ya no factorizan: son sustitución y residuo, unos 4,6 ms.

## Causa

La ruta estática de dos casos tarda 5,82 s porque factoriza dos veces una
LDLᵀ densa de orden 1 350, y cada factorización tarda 2,78 s. Sustitución,
recuperación, lectura, validación y serialización suman decenas de
milisegundos. La reducción densa entra en esos 47 ms y no explica los
segundos. Dentro de la factorización, cerca de dos tercios del tiempo
ajustado crece con el bloque activo y cerca de un tercio repasa la matriz
entera en cada columna. P-Delta tarda 8,85 s por el mismo núcleo, repetido
en la gravedad y otra vez en el primer tramo lateral. El modal no termina:
en 8 s solo recorre un prefijo del espectro denso.

## Propuesta, por impacto

Ninguna de estas propuestas está aplicada aquí. Si una cambia hashes, se
documenta y J4 lo audita. Conservar los hashes no es criterio.

1. **Núcleo LDLᵀ del orden 1 350.** Es la única pieza que puede acercar el
   estático a 54 ms y el P-Delta a 88 ms: hoy una factorización ya son
   2,78 s. En este modelo todos los pivotes fueron 1×1, así que el barrido
   de todo el triángulo extradiagonal no cambió el tamaño del bloque. Quitar
   ese barrido cuando el pivote sigue siendo el mayor diagonal conserva la
   aritmética de este caso. La fracción exacta de los dos tercios que es
   barrido, actualización o escritura por columnas queda DESCONOCIDO.
   Si el orden de las sumas o la secuencia de pivotes cambia, cambian los
   `sha256` de las entradas `*-run` de
   `crates/engine-cli/tests/golden/stdout-sha256.json` que ejecutan
   estático, P-Delta, modal o espectro, y las 16 huellas nativo/wasm de la
   suite de determinismo. Las entradas `*-validate` no factorizan.

2. **Una sola factorización de K en la fachada estática.** Medido: dos
   casos cuestan 2,83 s + 2,81 s, y `StaticLinear::run` de los dos casos
   con un solo factor cuesta 2,78 s. El `run` bajaría de 5,82 s a cerca de
   2,8 s. Los valores no cambian si el factor y las sustituciones son los
   mismos. Los hashes publicados no deberían cambiar. No alcanza los 54 ms.

3. **No volver a factorizar K en P-Delta.** La gravedad ya factoriza K. El
   primer tramo lateral, de 5,93 s, incluye en el fuente una factorización
   de K+Kg y otra de K. Reusar el factor de la gravedad evita esa segunda
   de K, del orden de 2,8 s dentro de los 8,85 s. Los hashes no deberían
   cambiar. El reparto medido entre inercia y la segunda K queda
   DESCONOCIDO.

4. **Dejar de recorrer toda la matriz con `finite` en cada columna.** El
   ajuste le asigna cerca de un tercio de los bloques posteriores a la
   columna 15. En un éxito, mirar el bloque actualizado y cerrar con una
   sola comprobación no cambia la aritmética ni los documentos que hoy
   salen bien. Puede cambiar el momento en que se informa un valor no
   finito.

5. **No densificar M ni Kg en un estático que no los lee.** Esos 47 ms son
   casi toda la meta de 54 ms, aunque hoy queden tapados por la
   factorización. Los hashes del estático no cambian si `results/1` no
   depende de esas matrices. Modal y P-Delta sí las necesitan.

6. **Espectro modal sin el Jacobi denso completo.** Hace falta para la meta
   de 0,58 s: el prefijo ya son 8 s y el total queda DESCONOCIDO. Un camino
   que no reproduzca las mismas rotaciones cambia los hashes de modal y de
   espectro en el golden de stdout y en las 16 huellas nativo/wasm.
