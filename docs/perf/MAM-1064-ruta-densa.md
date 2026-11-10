# MAM-1064 · Ruta densa sin cambio de hashes

Medición del mismo pórtico `portal_250` que MAM-1056, antes y después de
reutilizar la factorización de K, quitar el repaso de finitud por columna y
no densificar M ni Kg en el estático. No se tocó el núcleo aritmético de la
LDLᵀ ni el Jacobi del modal. Los tiempos son de esta máquina. Lo que no se
midió queda como DESCONOCIDO.

## Máquina

- SO: Linux 6.12.113+, x86_64.
- CPU: el hipervisor publica `Intel(R) Xeon(R) Processor`, 4 CPU. El modelo
  comercial más fino queda DESCONOCIDO.
- Compilador: rustc 1.99.0 (b940084d7 2026-09-28), cargo 1.99.0.
- Perfil: `release` de Cargo, sin `RUSTFLAGS`.
- `perf` y `flamegraph` no se usaron. Esa vía queda DESCONOCIDO.

## Comandos

Desde la raíz del repositorio, cinco repeticiones, mediana del valor
central tras ordenar. El antes es el commit
`63c3347d8f32d94dcb77a43eeab7e325ec8a08b5`. El después es este cambio.

```sh
cargo test --offline --locked --release -p engine-api --lib perfil_ruta_densa -- --ignored --nocapture
cargo test --offline --locked --release -p engine-bench --test portal_250 perfil_run_del_generador -- --ignored --nocapture
```

`cargo test` sin `--ignored` no ejecuta esas corridas. Las dos terminaron
con salida 0. En las cinco repeticiones del después, igual que en el antes,
`uses_2x2` fue `false`.

## Antes · fases del estático

Orden libre 1 350. Mediana de `perfil_ruta_densa`.

| Fase | Mediana |
| --- | ---: |
| Lectura: parse | 1,693 ms |
| Lectura: huella | 4,211 ms |
| Lectura: decode | 2,539 ms |
| Validación | 0,649 ms |
| Construcción de elementos | 0,969 ms |
| Ensamblaje elemental | 1,461 ms |
| Sistema global y reducción densa | 47,807 ms |
| Factorización LDLᵀ | 2 872,946 ms |
| Segundo miembro reducido | 0,317 ms |
| Sustitución | 11,116 ms |
| Solver de biblioteca, una factorización | 2 864,470 ms |
| Recuperación de esfuerzos | 5,636 ms |
| Serialización canónica | 7,857 ms |
| `run` completo, dos casos | 5 825,037 ms |

Tramos del `run` dentro de ese mismo ensayo:

| Tramo | Mediana |
| --- | ---: |
| parse | 8,671 ms |
| model | 0,649 ms |
| assembly | 55,325 ms |
| case 0 | 2 909,829 ms |
| case 1 | 2 796,238 ms |
| combinations | 32,780 ms |
| serialization (solo el callback) | 1,442 ms |

## Antes · runs completos de `portal_250`

Mediana de `perfil_run_del_generador`.

| Run | Mediana |
| --- | ---: |
| Estático, total | 5 996,616 ms |
| Estático, parse | 8,675 ms |
| Estático, model | 0,673 ms |
| Estático, assembly | 55,020 ms |
| Estático, case 0 | 2 960,617 ms |
| Estático, case 1 | 2 930,819 ms |
| Estático, combinations | 34,917 ms |
| Estático, serialization | 0,169 ms |
| P-Delta, total | 9 190,481 ms |
| P-Delta, parse | 9,023 ms |
| P-Delta, model | 0,689 ms |
| P-Delta, assembly | 51,521 ms |
| P-Delta, gravity | 3 011,356 ms |
| P-Delta, iteración 0 | 6 043,206 ms |
| P-Delta, iteración 1 | 5,402 ms |
| P-Delta, iteración 2 | 4,715 ms |
| P-Delta, iteración 3 | 4,613 ms |
| P-Delta, iteración 4 | 4,468 ms |
| P-Delta, serialization | 1,745 ms |

## Después · fases del estático

| Fase | Mediana |
| --- | ---: |
| Lectura: parse | 1,687 ms |
| Lectura: huella | 4,143 ms |
| Lectura: decode | 2,492 ms |
| Validación | 0,646 ms |
| Construcción de elementos | 0,921 ms |
| Ensamblaje elemental | 1,383 ms |
| Sistema global y reducción densa | 22,036 ms |
| Factorización LDLᵀ | 1 729,017 ms |
| Segundo miembro reducido | 0,328 ms |
| Sustitución | 9,527 ms |
| Solver de biblioteca, una factorización | 1 698,140 ms |
| Recuperación de esfuerzos | 5,462 ms |
| Serialización canónica | 6,945 ms |
| `run` completo, dos casos | 1 810,201 ms |

Tramos del `run` dentro de ese mismo ensayo:

| Tramo | Mediana |
| --- | ---: |
| parse | 8,360 ms |
| model | 0,638 ms |
| assembly | 24,234 ms |
| case 0 | 1 734,225 ms |
| case 1 | 2,533 ms |
| combinations | 30,141 ms |
| serialization (solo el callback) | 1,486 ms |

El caso 1 ya no vuelve a factorizar K. El armado del estático ya no
densifica M ni Kg: la reducción baja de 47,807 ms a 22,036 ms. El reparto
que queda dentro de esos 22,036 ms, entre el CSR y la reducción de K, queda
DESCONOCIDO.

## Después · runs completos de `portal_250`

| Run | Mediana |
| --- | ---: |
| Estático, total | 1 797,286 ms |
| Estático, parse | 8,376 ms |
| Estático, model | 0,660 ms |
| Estático, assembly | 25,927 ms |
| Estático, case 0 | 1 719,360 ms |
| Estático, case 1 | 2,613 ms |
| Estático, combinations | 34,201 ms |
| Estático, serialization | 0,116 ms |
| P-Delta, total | 3 534,788 ms |
| P-Delta, parse | 8,238 ms |
| P-Delta, model | 0,643 ms |
| P-Delta, assembly | 58,864 ms |
| P-Delta, gravity | 1 726,624 ms |
| P-Delta, iteración 0 | 1 713,197 ms |
| P-Delta, iteración 1 | 4,548 ms |
| P-Delta, iteración 2 | 4,349 ms |
| P-Delta, iteración 3 | 4,191 ms |
| P-Delta, iteración 4 | 4,183 ms |
| P-Delta, serialization | 1,757 ms |

P-Delta sigue densificando M y Kg, porque no es el estático. La gravedad
factoriza K una vez y la iteración 0 ya no vuelve a factorizar esa K: le
queda la factorización de K+Kg. El reparto interno de la iteración 0 entre
Kg, la reducción y esa factorización queda DESCONOCIDO.

## Ajuste de la factorización

El ajuste de dos términos de MAM-1056, `dt = a·Σ(n−k)² + b·15·n²`, sigue
en el ensayo. En el después el coeficiente del repaso salió negativo en las
cinco repeticiones (mediana −8,819×10⁻¹¹) y la mediana de la fracción
`a/(a+b)` que imprime el ensayo fue 1,1312. Esa fracción ya no parte el
tiempo: el repaso de toda la matriz por columna ya no está. La duración
medida de la factorización es 1 729,017 ms. Qué parte de esa duración es
búsqueda del pivote, actualización o escritura por columnas queda
DESCONOCIDO.

El modal no entra en este cambio. El mismo ensayo lo cortó a los 8 s: la
mediana del prefijo fue 8 003,574 ms. El espectro completo sigue
DESCONOCIDO.
