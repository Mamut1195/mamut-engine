# Implementar un método en Rust · ENG-CORE-011

Esta guía permite pasar de una ficha matemática a una implementación revisable.
El contrato vigente manda: no se cambia para acomodar un método. Se trabaja
desde la raíz del worktree, sin red, con herramientas y dependencias provisionadas.
El [protocolo clean-room](CLEAN_ROOM.md) es obligatorio; esta guía no lo sustituye.

## 1. Preparar la sesión y elegir el contrato

Lee, en este orden:

1. [CLEAN_ROOM.md](CLEAN_ROOM.md), especialmente §§3–5, 7, 8 y 10, y
   [provenance/TEMPLATE.md](../provenance/TEMPLATE.md).
2. [engine-core](../crates/engine-core/src/lib.rs): `Real = f64`, `EResult`,
   `EngineError`, `ContractId`; [catálogo](../crates/engine-core/src/catalog.rs)
   y [funciones escalares](../crates/engine-core/src/math.rs).
3. [traits](../crates/engine-contracts/src/traits.rs),
   [tipos de intercambio](../crates/engine-contracts/src/types.rs) y
   [conformidad ENG-CORE-007](../crates/engine-contracts/src/conformance.rs).
4. [engine-laws](../crates/engine-laws/src/lib.rs), sus módulos y pruebas.
   Revisa qué existe antes de elegir una ley: una parametrización de una ley
   existente no constituye un método nuevo.
5. [benchmarks ENG-CORE-009](../crates/engine-validation/README.md),
   [schema fixture/1](../crates/engine-validation/schemas/fixture-1.json) y
   [determinismo](DETERMINISMO.md).

Registra rol de especificador/implementador y fuentes locales consultadas.
Si hubo exposición a fuentes o informes prohibidos, no implementes: aplica
las restricciones de roles del protocolo. No se inspeccionan repos hermanos.
Las fuentes son derivaciones propias o teoría publicada aportada localmente
con cita verificable; no se inventan páginas, ecuaciones ni resultados.

Configura la sesión (el entorno proporciona un `$CARGO_TARGET_DIR` exclusivo):

```sh
export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
: "${CARGO_TARGET_DIR:?Configura el directorio de compilación autorizado}"
export CARGO_TARGET_DIR
export CARGO_NET_OFFLINE=true
export PIP_NO_INDEX=1
rustc --version  # toolchain del ticket: 1.99.0
git status --short
```

## 2. Escribir la ficha antes del código

Copia [FICHA_METODO.md](plantillas/FICHA_METODO.md) y completa hipótesis,
ecuaciones numeradas, derivación de la tangente, cita de cada fuente,
unidades, signos, dominio físico y numérico, historia y casos analíticos.
No copies tablas, figuras ni listados publicados. Evalúa las fórmulas por tu
cuenta para fijar los esperados **antes** de ejecutar el método.

Para `uniaxial/1`, el contrato concreto es:

| Aspecto | Obligación |
| --- | --- |
| Entrada | Deformación **total** adimensional; tensión positiva en tracción. Si representa un resorte: desplazamiento relativo en m y fuerza en N, declarados por el kind. |
| Salida | `UniaxialResponse { stress, tangent, trial_state }`; tangente dσ/dε en Pa (o N/m), todo finito. |
| Estado | `state_len()` exacto; historia inicial y de prueba planas y finitas. Documentar orden/unidades de componentes. |
| Propiedad | `respond(&self, strain, committed)` no muta el objeto ni la historia. El llamador acepta con `committed = response.trial_state`; descartar una prueba equivale a rollback. |
| Repetición | Mismas entradas producen la misma respuesta; repetir una deformación aceptada reproduce su tensión. |
| Origen | `initial_tangent()` coincide con `respond(0, initial_state).tangent`. |
| Identidad | `ContractId::new("uniaxial", 1, 0)` y kind propio estable; versión semántica del método independiente del contrato. |
| Errores | `ENG-E-0008`: NaN, infinitos o desbordamiento; `ENG-E-0010`: parámetros, tamaños o rangos inválidos. Mensajes con magnitud y unidades. |

Un método de sección usa `SectionLaw` y `check_section_law`; un elemento,
`Element`/`ElementFactory` y `check_element`/`check_element_factory`; un análisis,
`AnalysisMethod`, `SystemView` y sus comprobadores. Sus tamaños, unidades y
convenciones se toman del trait específico, no de la tabla uniaxial.

## 3. Crear el módulo y exponerlo como los existentes

La opción habitual para otra ley es un módulo de `engine-laws`; no necesita
otro crate ni modificar manifests. Copia
[modulo_uniaxial.rs.tpl](plantillas/modulo_uniaxial.rs.tpl) a
`crates/engine-laws/src/uniaxial/mi_metodo.rs`, sustituye nombres, parámetros
y la respuesta pendiente por las ecuaciones de la ficha. La plantilla se
detiene con `ENG-E-0013`: no es una ley validada ni debe publicarse sin completar.

Mantén parámetros privados validados, constructor falible, `Params`, fábrica
tipada `UniaxialFactory` y constante `LawDescriptor`. Reutiliza los validadores
privados `finite`, `positive` y `vector` de `engine-laws`. Valida longitud antes
de indexar y todas las salidas/intermedios susceptibles de desbordamiento.
Documenta la API pública en español; la biblioteca exige `missing_docs`.
No añadas `unsafe`, `unwrap`, `expect`, `panic!`, estado global, E/S ni hilos.
Las funciones escalares se invocan mediante `engine_core::math`; conserva el
orden de operaciones/reducciones de los métodos existentes.

Expón `mod mi_metodo; pub use mi_metodo::{...};` desde `uniaxial.rs` y actualiza
la documentación de `lib.rs`. Este es el registro de leyes actual: descriptor,
fábrica y exportaciones públicas, sin registro global. No implica soporte JSON
en engine-api ni convertir el análisis lineal en no lineal. Una extensión de
la API requiere su propio contrato y validación explícitos.

Si un contrato exige otro crate, justifica primero esa separación; usa edición,
licencia y versión mínima del workspace, dependencias internas ya existentes y
`#![forbid(unsafe_code)]` en la raíz. Solo entonces adapta los manifests y lock.

## 4. Conformidad y benchmark analítico

Copia [conformidad.rs.tpl](plantillas/conformidad.rs.tpl) y
[fixture_uniaxial.rs.tpl](plantillas/fixture_uniaxial.rs.tpl) a `engine-laws/tests/`.
Los nombres de importación/constructor y los casos se completan desde la ficha.
La tabla vacía de la plantilla falla intencionadamente: evita validación vacua.

Ejecuta `check_uniaxial_law(&law, &samples, h)` y exige cero diagnósticos.
El kit acepta explícitamente historia entre muestras y compara tangente con
diferencia central a historia aceptada fija, con error relativo < 1e-6.
Escoge `h` dentro de los límites publicados en `conformance.rs`, con escala
adecuada y muestras cuyo intervalo ε ± h sea suave. En vértices o fluencia,
prueba por separado la convención unilateral, sin FD central cruzando ramas.
No relajes el kit para que pase una implementación.

Añade negativos para cada parámetro/rango, NaN/±∞ en entradas e historia,
longitudes incorrectas y desbordamiento, verificando códigos del catálogo.
Comprueba fábrica/descriptor/objeto, `Send + Sync`, pureza y propiedad del estado.
El kit verifica contrato y consistencia; un error compartido por tensión y
tangente puede pasar. Por eso el benchmark usa esperados analíticos independientes.

El fixture uniaxial tipado contiene `source`, cita, parámetros SI, entradas,
esperados y tolerancia justificada; se ejecuta como prueba nativa de la ley.
No es `fixture/1`: ENG-CORE-009 exige `engine-input/1 → results/1` mediante
`Target`, y la API actual no publica estas leyes no lineales. No fabriques un
JSON que el runner no pueda ejecutar ni añadas otro caso al inventario cerrado
de cinco JSON existente. Cuando el método sea accesible por API, usa
[fixture_api.json.tpl](plantillas/fixture_api.json.tpl), completa la entrada
real y los JSON Pointers y ejecútalo con el runner nativo de ENG-CORE-009.
Ese formato exige `source.kind` y cita, y compara con
`|actual−esperado| ≤ max(absolute, relative·|esperado|)`; un override requiere
justificación. No cambies tolerancias a partir de salidas de otros motores.

## 5. Piloto reproducible: elasticidad cúbica

Ficha propia: Codex gpt-6.1-sol high, *Elasticidad cúbica conservativa*,
2026-10-20, esta sección, ecuaciones (1)–(3) y tabla siguiente.
No se consultaron fuentes externas. Antes del piloto existían `Elastic`,
`MultilinearElastic` y `SteelBilinearKinematic`; la multilineal ya representa
curvas bilineales, y la cinemática permite plasticidad perfecta con b = 0.
Ninguna representa exactamente una curva cúbica suave con C > 0.

Se propone una densidad de energía propia (Pa = J/m³), sin historia:

```text
(1) W(ε) = E ε²/2 + C ε⁴/4,       E > 0, C ≥ 0, ambos en Pa.
(2) σ(ε) = dW/dε = Eε + Cε³,       σ positiva en tracción.
(3) K(ε) = dσ/dε = E + 3Cε² > 0,  K(0) = E.
```

W es par, no negativa y estrictamente convexa; σ es impar. La descarga sigue
la misma curva y el trabajo en un ciclo cerrado es cero. Se trata de una
idealización elástica local 1D, sin plasticidad, daño ni validez física universal
para grandes deformaciones. Historia inicial/aceptada/de prueba: longitud cero.
C = 0 recupera la ley lineal como límite, sin sustituir su implementación.

Implementación fijada por la ficha: `q = (C·ε)·ε`, `σ = (E+q)·ε`,
`K = E+3q`. Cada producto/suma se exige finito; un desbordamiento devuelve
`ENG-E-0008`. Se evita formar ε² antes de multiplicar por C para que C = 0
admita deformaciones grandes cuando la respuesta siga siendo representable.
Se admite el redondeo/subdesbordamiento habitual de f64; no se certifica
exactitud relativa de términos por debajo de la resolución del número.

Benchmark analítico propio con E = 8 Pa, C = 16 Pa. Sustituyendo fracciones
binarias en (2)–(3) se obtiene, antes de correr la implementación:

| ε total | σ (Pa) | K (Pa) |
| --- | --- | --- |
| 0 | 0 | 8 |
| 1/4 | 9/4 | 11 |
| 1/2 | 6 | 20 |
| 1 | 24 | 56 |
| 1/2 (descarga) | 6 | 20 |
| 0 (descarga) | 0 | 8 |
| −1/4 | −9/4 | 11 |
| −1/2 | −6 | 20 |
| −1 | −24 | 56 |
| 0 (cierre) | 0 | 8 |

Se usa piso absoluto 1e-12 Pa y relativo 1e-12: holgura sobre el redondeo f64
de pocas operaciones; no proviene de calibración externa. Para FD, h = 1e-6:
el error de truncamiento central es exactamente C h² = 1,6e-11 Pa.

Aplicación de los pasos 1–4: ficha anterior escrita primero; módulo
[`cubic_elastic.rs`](../crates/engine-laws/src/uniaxial/cubic_elastic.rs),
kind `elastic.cubic`, versión 0.1.0, contrato `uniaxial/1`, `CubicElasticParams`,
`CubicElasticFactory`, descriptor `CUBIC_ELASTIC` y exportaciones públicas.
La prueba [`cubic_elastic.rs`](../crates/engine-laws/tests/cubic_elastic.rs)
instancia el kit y ejecuta la tabla nativa con source/cita, además de negativos,
pureza, simetría y límite lineal. No requiere solver, API ni dependencias nuevas.

```sh
cargo test --offline -p engine-laws --test cubic_elastic
```

La evidencia completa y el estado de revisión están en
[ENG-CORE-011](../provenance/ENG-CORE-011.md). El piloto verifica esta guía en
la misma sesión autora; la revisión independiente sigue pendiente de J4.

## 6. Cerrar y revisar

Completa [PROVENANCE.md](plantillas/PROVENANCE.md), añade CHANGELOG y aplica la
[lista de revisión](plantillas/REVISION.md). Para una IA, parte del
[prompt restringido](plantillas/PROMPT_IA.md). Mantén los cambios sin commit.

Ejecuta todas las comprobaciones desde la raíz:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline
cargo check --workspace --offline --target wasm32-unknown-unknown
bash clean-room/check.sh origin/main
```

El comprobador clean-room solo incluye archivos conocidos por Git: para cubrir
los nuevos sin alterar el índice real, usa un índice temporal bajo
`$CARGO_TARGET_DIR`, inicializado con `git read-tree HEAD`, añade allí las rutas
del entregable y repite el comando con `GIT_INDEX_FILE` apuntando a ese índice.
Si los objetos de `.git` son de solo lectura, configura además
`GIT_OBJECT_DIRECTORY` bajo `$CARGO_TARGET_DIR` y
`GIT_ALTERNATE_OBJECT_DIRECTORIES` con la salida de
`git rev-parse --path-format=absolute --git-path objects` obtenida antes de
exportar esa configuración. Así los objetos nuevos también son temporales.
Conserva los logs y no cambies listas ni workflows para eludir una comprobación.

Verifica además los 16 hashes contra la tabla de DETERMINISMO.md, con
`MAMUT_ENGINE_GIT_COMMIT` sin definir, los ejemplos `determinism_native` y
`determinism_wasm`, y `scripts/check-determinism.mjs`; ejecuta nativo/wasm y
debug/release como indica ese documento. Guarda las salidas en
`$CARGO_TARGET_DIR`; la tabla de hashes no se regenera para aceptar diferencias.
`cargo check` wasm solo comprueba compilación; no acredita ejecución wasm del
piloto. Reporta por separado ejecución nativa, cruce de hashes y pendientes.
