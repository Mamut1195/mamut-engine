# engine-validation · ENG-VAL-001 parcial

Suite de **73 fixtures analíticos propios**: los **42 estáticos lineales**
existentes permanecen intactos y se añaden **31** (12 modales, 10 P-Delta,
9 espectrales). Los lineales usan `StaticLinear`, elementos reales `LinearTruss` o
`EulerFrame2D`, recuperación `engine_analysis::results::CaseResults` y la
combinación de LOAD-002. Las cuatro filas de superposición/combinación comparan
además con un caso de cargas combinadas resuelto de manera independiente.

Ejecutar desde la raíz:

```sh
CARGO_NET_OFFLINE=true cargo test -p engine-validation
```

El test lineal recorre `tests/support/table.rs`; tres tests, uno por grupo,
recorren `tests/support/dynamic_table.rs` desde `tests/dynamic.rs`.
Los errores incluyen nombre,
campo, valor analítico, tolerancia, `source` y `citation`. Cada fila representa
un problema completo; no se cuentan sus comprobaciones como fixtures distintos.
Las suites históricas usan dependencias internas por `path`. El runner
`fixture/1` añade engine-api, serde y serde_json (versiones ya presentes en
el workspace y Cargo.lock); `publish = false`.

La tabla contiene geometría, material, acciones, factores y tolerancia. Las
magnitudes SI artificiales permiten comprobar álgebra lineal; no representan
un diseño físico de pequeños desplazamientos. Se usa interpolación nodal exacta
situando nodos donde se comprueban flechas; las cargas puntuales interiores se
aplican en esos nodos. Los diagramas se recuperan en estaciones y raíces
críticas exactas del elemento, sin interpolar momentos.

No hay E/S, red, reloj, procesos ni hilos en los ejecutores históricos. El código de pruebas
se comprueba también con `cargo check --workspace --all-targets --target
wasm32-unknown-unknown`; las pruebas se ejecutan en nativo. Compilar para wasm
no implica haber ejecutado un runtime wasm.

Derivaciones, convenciones, tolerancias y estado de revisión:
[bitácora ENG-VAL-001](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden).
La ampliación usa `Modal`, `PDelta` y `Spectrum` de la cadena apilada. Las
frecuencias de vigas se contrastan con soluciones continuas y refinamiento;
el P-Delta distingue la aproximación de amplificación del contraste continuo
exacto. Las tolerancias de cada fila explican esa diferencia.

ENG-VAL-001 sigue parcial: ejemplos publicados con cita exacta **PENDIENTES**
(no verificables sin acceso, fuera del alcance de J2), revisión clean-room
**PENDIENTE (J4)** y ejecución en un runtime wasm **PENDIENTE**. La cobertura
analítica de modal, P-Delta y espectro y el umbral de ≥30 fixtures están cubiertos.

ENG-GEO-002 añade **seis fixtures Winkler** en
`tests/support/winkler_table.rs`: viga infinita, semiinfinita y corta rígida,
cada una en 2D y 3D. Usan fábricas `frame2d.winkler`/`frame3d.winkler`,
refinamiento, recuperación e integración independiente de reacciones del lecho.
Las pruebas de invariantes están en `tests/winkler_elements.rs`; la derivación,
unidades y cotas están en [ENG-GEO-002](../../provenance/ENG-GEO-002.md).

El mismo harness y tabla se ejecutan en nativo y en wasm, sin paquetes de JS:

```sh
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PWD/target}  # un directorio por worktree
cargo test --offline -p engine-validation --test winkler --test winkler_elements
cargo build --offline -p engine-validation --example winkler_wasm --release --target wasm32-unknown-unknown
node crates/engine-validation/scripts/run-winkler-wasm.cjs "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/examples/winkler_wasm.wasm"
```

Se necesita el target wasm ya instalado y Node con WebAssembly. El export
`winkler_benchmarks()` devuelve seis si todos los fixtures pasan, cero si falla
alguno. El script exige ese resultado y ausencia de imports externos. Esta
prueba ejecuta estos seis fixtures en wasm; las demás suites conservan el
estado de ejecución documentado arriba.

## Línea base independiente · ENG-VAL-005 (MAM-105)

La [matriz de oráculos](ORACLES.md) inventaría los 91 casos analíticos V1,
sus fuentes y tolerancias, incluidos Winkler, consolidación y `fixture/1`.
La guardia `tests/oracles.rs` recorre las tablas y documentos versionados;
las suites numéricas existentes verifican sus valores sin alterar límites.
ENG-REL-001 depende de ENG-VAL-005 y no exige ENG-VAL-003. OpenSees queda
como contraste externo adicional; sus archivos y resultados no entran al repo.
Las referencias bibliográficas sin cotejo permanecen PENDIENTES en la matriz.

## Formato fixture/1 y runner · ENG-CORE-009 (MAM-22)

Los cinco archivos de `fixtures/` son casos analíticos propios, independientes
de las tablas históricas. El contrato está en
[`schemas/fixture-1.json`](schemas/fixture-1.json) (JSON Schema 2020-12), y
`engine_validation::fixtures::Fixture::parse` aplica además las reglas semánticas
de procedencia, números finitos, claves duplicadas y JSON Pointers únicos.
No se necesitan validadores externos ni acceso a los identificadores URL del schema.

Cada documento contiene:

| Campo | Contrato |
| --- | --- |
| `schema` | Exactamente `fixture/1`. Otras versiones se rechazan. |
| `id` | Texto no vacío y único dentro de la ejecución. |
| `source` | Objeto obligatorio con cita completa; todos sus campos tienen contenido. |
| `input` | Entrada `engine-input/1` **embebida**; esta versión no resuelve referencias ni rutas. engine-api comprueba el contrato completo al ejecutarla. |
| `expected` | Lista no vacía de `{ "pointer": "/cases/0/nodes/1/displacements/0", "value": 0.12 }`. Rutas JSON Pointer RFC 6901 sobre `results/1`, sin rutas duplicadas. |
| `tolerance` | Opcional: `{ "relative": 1e-10, "absolute": 1e-10, "justification": "justificación específica" }`. Ambos límites son finitos; relative ≥ 0, absolute > 0. |
| `provenance` | Metadatos JSON opcionales, sujetos al mismo veto de procedencia que source. |

`source.kind` admite tres variantes, con estos campos obligatorios:

- `analytical`: `citation` (autor, nota, fecha y ubicación de la derivación) y
  `derivation` (ecuaciones, condiciones de contorno, parámetros, unidades y evaluación propia).
- `book`: `citation`, `author`, `title`, `edition`, `page` (todos como texto).
- `published`: `citation`, `author`, `title`, `publication`, `locator`
  (página/ejemplo/DOI/URL que permita localizarlo; el runner no abre la referencia).

No se copian tablas ni figuras. Se rechazan source ausente, null, vacío, campos
en blanco y variantes incompletas. El runner inspecciona recursivamente los
valores y claves de `source` y `provenance`: cualquier mención de OpenSees,
Stabileo o CalculiX, incluso dentro de otra cadena y sin distinguir mayúsculas,
invalida el fixture **antes de ejecutar un target**. Estos nombres aparecen
solamente en la política de rechazo y sus tests negativos; los fixtures no
contienen datos de esos motores.

Para cada magnitud se exige:

```text
|actual - esperado| <= max(absolute, relative * |esperado|)
```

Sin override, `linearStatic` usa relative = absolute = **1e-10**. Los métodos
iterativos (`pDelta`, `modal`, `responseSpectrum`) usan **1e-7** para ambos.
El piso absoluto está expresado en la unidad SI de la magnitud comparada (m,
rad, N o N·m); evita dividir por cero o amplificar el error relativo de valores
casi nulos. La referencia es siempre el valor esperado. Un override se aplica
a todas las magnitudes de ese fixture y requiere justificación no vacía.
Los cinco casos cerrados declaran explícitamente el mismo límite 1e-10
que antes se obtenía del valor predeterminado. Una ruta ausente,
no numérica, no finita, o una salida con status distinto de `ok` produce fallo;
el runner continúa con el resto de fixtures y targets.

| Fixture | Oráculo propio principal |
| --- | --- |
| `axial-bar` | u = PL/(EA), reacción y fuerza axial. |
| `axial-series` | Dos barras como resortes en serie: u = P(L₁/EA₁ + L₂/EA₂). |
| `cantilever-point` | v = −PL³/(3EI), giro y reacciones. |
| `simply-supported-uniform` | v central = −5wL⁴/(384EI), M máximo = wL²/8, giro central cero. |
| `spatial-cantilever` | Flexión espacial sobre Y: z = −PL³/(3EIy), reacción My = −PL. |

Los parámetros SI son artificiales para verificar el álgebra. La viga bajo
carga uniforme tiene un nodo central y dos miembros, para comprobar la flecha
nodal exacta y la recuperación de momentos. Los resortes se realizan con barras
porque engine-api todavía no publica el elemento resorte.
Derivaciones y revisión: [provenance/ENG-CORE-009.md](../../provenance/ENG-CORE-009.md).

Ejecución desde la raíz, sin red (toolchain y target wasm ya provisionados):

```sh
export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PWD/target}  # un directorio por worktree
export CARGO_NET_OFFLINE=true

# Cargo ejecuta el runner nativo sobre TODOS los JSON del directorio fixtures/.
cargo test --offline -p engine-validation --test fixtures
cargo run --offline -p engine-validation --example fixture_runner
cargo run --offline -p engine-validation --example fixture_runner -- --json

# Compilar y ejecutar ambos targets, comparados contra el mismo oráculo.
cargo build --offline -p engine-validation --example fixture_wasm --target wasm32-unknown-unknown
cargo run --offline -p engine-validation --example fixture_runner -- \
  --wasm "$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/examples/fixture_wasm.wasm"
cargo run --offline -p engine-validation --example fixture_runner -- \
  --wasm "$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/examples/fixture_wasm.wasm" --json

# También admite una lista explícita de archivos, con los mismos dos targets.
cargo run --offline -p engine-validation --example fixture_runner -- \
  --wasm "$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/examples/fixture_wasm.wasm" \
  crates/engine-validation/fixtures/axial-bar.json --json
```

Sin rutas explícitas, el runner descubre todos los `.json` de `fixtures/`.
`--wasm` añade `wasm32-node` al target obligatorio `native-api`. Node usa
solo `node:fs` y `node:assert/strict`, sin npm, wasm-bindgen, WASI ni red.
Comprueba cero imports, límites de la ABI y target `wasm32-unknown-unknown`.
El script `scripts/run-fixture-wasm.cjs` transporta una entrada engine-input/1
por stdin y devuelve results/1 por stdout. La ABI acumula/lee bytes con funciones
escalares seguras; no expone punteros ni contiene unsafe. Cada ejecución Node
usa una instancia nueva del módulo y el comparador Rust común evalúa la salida.

El informe legible muestra fixture/target, cada valor esperado/calculado,
error y límite. `--json` emite `fixture-report/1` con claves ordenadas, filas
ordenadas por fixture/target, comprobaciones en orden declarado, source,
provenance, tolerancia efectiva, justificación y errores. Sin tiempos ni reloj:
este runner es de benchmarks de exactitud, no de rendimiento. Es determinista
para los mismos documentos, targets y entorno; errores de herramientas externas
conservan su diagnóstico. Salida del proceso: 0 si todas las filas pasan, 1
si falla algún caso (también una suite vacía), 2 para errores de opciones o E/S.
`cargo test` no necesita Node; la ejecución wasm se comprueba con el comando anterior.

La interfaz pública `Target` recibe engine-input/1 y devuelve results/1, por
lo que otros adaptadores comparten el mismo comparador. El target CLI
`mamut-engine` se añadirá cuando **ENG-SFC-002** se fusione; no hay dependencia
del CLI ni de su contrato aún pendiente.

## Curvas p-y estáticas · ENG-GEO-003

Tres fixtures SI con `source.kind = analytical` y controles literales de
OTC 1204 (1970), Ec. 1–5/Fig. 5a estática, y OTC 2080 (1974), Ec. 4–9,
11–16/pasos 2 y 4–8, derivados en [la bitácora](../../provenance/ENG-GEO-003.md).
La tabla `tests/support/py_table.rs` y su runner se comparten entre targets.
No usan el formato de modelos fixture/1 ni añaden modelos estructurales.

```bash
cargo test --offline -p engine-validation --test py
cargo build --offline -p engine-validation --example py_wasm --target wasm32-unknown-unknown
node crates/engine-validation/scripts/run-py-wasm.cjs "$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/examples/py_wasm.wasm"
```

El script exige tres fixtures aprobados y ningún import externo. Los controles
incluyen Matlock en yc/8, yc, 8yc y la transición Np=9; Reese verifica pct, pcd,
Xt, k/m/u, los interiores de cada tramo y la selección de la rama profunda.

## Curvas axiales · ENG-GEO-004 (MAM-45)

Cinco fixtures compartidos con `engine-geo/tests/support/axial_benchmarks.rs`:
API arena, arcilla y punta, más dos tablas de usuario artificiales. Fuente:
API RP 2GEO, primera edición (abril 2011), Addendum 1 (octubre 2014),
reafirmada 2021; §8.4.2 figura 2/tabla 2 y §8.4.3 figura 3. Comprueban
vértices, interpolación calculada a mano, mesetas y tracción. La tolerancia
relativa `2e-14` (escala mínima 1 SI) cubre solo redondeo binary64;
las ordenadas de los vértices se comprueban además exactamente.

```sh
cargo test --offline -p engine-validation --test axial
cargo build --offline -p engine-validation --example axial_wasm --release --target wasm32-unknown-unknown
node crates/engine-validation/scripts/run-axial-wasm.cjs "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/examples/axial_wasm.wasm"
```

El export `axial_benchmarks()` devuelve cinco al aprobar todos, cero si falla
alguno. El script exige ausencia de imports. No incorpora barras ni pilotes,
ni altera los 91 casos de la línea base ENG-VAL-005.
