# Bitácora clean-room · ENG-LA-008: MemoryBudget en factorize y generalized_eigen

- Ticket / PR: cableado de MemoryBudget (J0); rama `j2/la-008-budget-factorize` (apilada sobre #49)
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: engine-linalg (MemoryBudget)
- Fuentes permitidas usadas: ninguna externa.
- Derivaciones propias: `MemoryBudget::factorize` y `::generalized_eigen`: cargan retenido+trabajo, guarda RAII, éxito deja solo lo retenido; error/pánico deja `used` intacto; orden de errores independiente del presupuesto (forma → desborde → ENG-E-0013 → backend). La afirmación inicial de «cotas superiores» era incorrecta para factorize: omitía una matriz transitoria, la cabecera del Box y la capacidad sobrante de blocks. La corrección y el alcance de las cotas actuales se detallan abajo.
- Código de terceros copiado: ninguno
- Validación: tests/budget.rs (cargo, límite, error del backend, órdenes distintos).
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)

## Corrección del pico de heap (J4, 2026-10-13)

Se conserva el cálculo numérico de LDLᵀ. `balanced`, la copia normalizada `a` y
`l` siguen vivas simultáneamente: el trabajo transitorio son **dos** matrices,
además de la L retenida. `blocks` reserva `with_capacity(n)` antes de los pushes,
porque hay como máximo n pivotes; así su capacidad queda contabilizada incluso
para órdenes pequeños o distintos de una potencia de dos. `factor_bytes` incluye
también `size_of::<Ldlt>()`, la cabecera reservada por el Box del resultado.

Con U = `size_of::<usize>()` y H = `size_of::<Ldlt>()`, las cotas actuales son:

- Factor retenido: `8n² + (24 + 2U)n + H`.
- Carga máxima de factorize: `24n² + (24 + 2U)n + H`.
- Trabajo de generalized_eigen: `72n² + (32 + 2U)n + H`, **sin el resultado**.
  Las nueve matrices cubren Cholesky, inversa, productos, Jacobi y la capacidad
  sobrante al recoger modos; los términos adicionales cubren vectores auxiliares,
  índices/espacio de ordenamiento y la cabecera LDLᵀ del diagnóstico de masa.
- Resultado modal, cargado aparte: `8n² + 8n`.

Son cotas del almacenamiento numérico de heap del backend de referencia: bytes
de Layout solicitados al allocator, excluyendo las entradas ya existentes,
metadatos internos del allocator y construcción de mensajes de error. No se
prometen cotas para backends arbitrarios. El presupuesto se libera con la misma
guarda RAII; al terminar con éxito queda solo el factor o resultado retenido.

`tests/heap_peak.rs` usa un GlobalAlloc propio que delega en System y cuenta
reservas exitosas, liberaciones y cambios de tamaño con AtomicUsize. El unsafe
está limitado al allocator del test y documenta sus contratos; la biblioteca no
incorpora unsafe ni llamadas nuevas a panic/unwrap/expect. Un único test en su
ejecutable y un Mutex serializan las mediciones; se reinicia el pico con los bytes
vivos como base antes de cada operación. Las matrices SPD deterministas se
construyen antes de medir. También se comprueban órdenes 0, 1, 3, 9 y 65 para
cubrir cabeceras y capacidades fuera de potencias de dos, y el heap retenido.
La prueba exige que el pico real no supere la carga del presupuesto y que la
cota antigua subestime al menos uno de los órdenes obligatorios.

Medición nativa de 64 bits (`cargo test --offline -p engine-linalg --test heap_peak -- --nocapture`;
U=8, H=232), en bytes:

| n | Pico factorize | Cota antigua `16n²+40n` | Carga corregida | Pico generalized_eigen | Carga generalized_eigen |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 1856 | 1344 | 2088 | 4224 | 5800 |
| 64 | 100864 | 68096 | 101096 | 263168 | 331496 |
| 256 | 1583104 | 1058816 | 1583336 | 4198400 | 5257448 |

Los picos nuevos de factorize son menores que los de la sonda de J4
(1888/101120/1584128 B) porque blocks ya no crece mediante reasignaciones.
La cota antigua queda por debajo de los tres picos; la corregida los cubre.
Los valores medidos son evidencia de regresión, además del recuento de reservas
anterior; no se usan constantes de pico como oráculo en los asserts.

Fuentes: revisión del código propio de engine-linalg y medición propia; no se
consultó ni copió la sonda externa del revisor. Sin dependencias nuevas.

Validación final, sin red (`CARGO_NET_OFFLINE=true`):

- `cargo fmt --all` y `cargo fmt --all -- --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK (Rust 1.85.1).
- `cargo test --workspace`: OK, incluidos heap_peak y doctests (Rust 1.85.1).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK (Rust 1.85.1).
- `cargo check --workspace --target wasm32-unknown-unknown`: OK con el toolchain
  estable ya instalado en `/home/box/.rustup/toolchains/stable-x86_64-unknown-linux-gnu`
  (Rust 1.99.0), usando sus binarios cargo/rustc explícitamente. El rustc 1.85.1
  de `/usr/bin` carece de std para ese destino; el primer intento devuelve E0463.
  No se instaló ningún componente ni se accedió a la red.
