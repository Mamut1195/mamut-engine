# MAMUT Engine · Python local

Bindings PyO3/maturin para Python >=3.10 sobre `engine-api`. El adaptador no
implementa cálculos numéricos: validación, análisis, códigos, capacidades y
contratos proceden de la fachada Rust.

Los metadatos declaran `AGPL-3.0-only`. La wheel incluye `LICENSE` y `NOTICE`.
Titular: MAMPRO SAS (ADR-001, 2026-10-15). Texto de la licencia comercial, aviso
definitivo, texto del CLA y registro de firmas: **PENDIENTES** (ENG-GOV-004 /
ENG-GOV-007). El inventario de licencias generado no resuelve
estas revisiones. **Verificación local únicamente: nada se publica ni se sube.**

## API tipada

```python
from pathlib import Path
import mamut_engine as engine

model = engine.Model.from_json(Path("input.json").read_text(encoding="utf-8"))
report = engine.validate(model)
events: list[engine.Progress] = []
try:
    results = engine.run(model, progress=events.append)
    document = results.to_dict()  # ResultsDocument, copia independiente
    Path("results.json").write_text(results.to_json(), encoding="utf-8")
except engine.EngineError as error:
    print(error.code, error.severity, error.entities, error.details)
    # error.document conserva la salida Rust; error.results permite inspeccionarla.
    raise
```

El paquete incluye `py.typed` y stubs. Los DTO `TypedDict` están en
`mamut_engine.types`: modelo, análisis discriminados por `method`, opciones,
resultados, diagnósticos y descriptores de capacidades. Son tipos Python;
la API pública no devuelve objetos ni excepciones PyO3. Los campos opcionales
se omiten y se conservan en los documentos; la fachada no rellena propiedades
físicas. Las extensiones de resultados se conservan incluso si el DTO instalado
todavía no les da nombre. `ModelDocument` representa el trabajo completo
`engine-input/1`, incluidos casos, combinaciones y análisis.

- `validate(model: ModelLike, *, lang="en") -> ValidationReport`: informe `diagnostics` de
  Rust. Los defectos topológicos son diagnósticos; errores de argumento o
  contrato lanzan `InvalidInputError`/`Unsupported`. No prueba estabilidad.
- `run(model: ModelLike, *, lang="en", cancel=None, progress=None) -> Results`: ejecuta
  engine-api. Solo `ok` y `warning` retornan normalmente. `Results.to_json()`
  devuelve exactamente la salida Rust, sin serialización intermedia en Python.
- `version() -> VersionInfo`: versión y contratos reconocidos por el motor;
  `__version__` conserva la cadena. `Results.version` refleja el productor y
  todos los contratos de ese resultado, incluidos los contratos de dominio.
- `capabilities() -> Capabilities`: métodos, elementos y descriptores reales
  de Rust, con madurez y evidencia. P-Delta es `Stable`: se ejecuta con
  `options: {}` y no necesita `allow_experimental`.
- `schema(name: str, version: int | None = None) -> JSONDocument`: admite
  `schema("results/1")` o `schema("results", 1)`. Publica los cuatro schemas
  actuales: engine-input/1, results/1, method-descriptor/1 y capabilities/1.
- `explain(code: str, *, lang="en") -> str`: explicación del catálogo Rust; los nombres
  de schemas y códigos desconocidos producen `InvalidInputError`.

`ModelLike` admite `Model`, `ModelDocument` o JSON `str`. `Engine` agrupa las
mismas seis funciones como métodos sin estado. `Model.from_dict()` conserva
la referencia mutable; `to_dict()` devuelve una copia y `to_json()` delega la
canonización compacta propia del motor. `Results.from_json(source, model=...)`
valida el contrato, conserva bytes, orden, ids y campos opcionales y comprueba
la procedencia si se proporciona modelo. Sin modelo conserva la procedencia
declarada, sin autenticarla. Mayores desconocidos se rechazan.

`Model.input_sha256` y la detección de estados stale usan la huella RFC 8785/JCS
calculada en Rust, igual que `results/1.input_sha256`. No es el hash de
`Model.to_json()`: por ejemplo, JCS iguala `4` y `4.0`. Tras modificar el input,
los accesos a `Results` vinculados comprueban la huella y lanzan
`StaleResultsError` (`PY-E-STALE`); `is_stale` permite consultarlo. La comparación
incluye el trabajo completo, no solo la geometría. Las copias exportadas no
modifican el snapshot ni siguen sus cambios.

Unidades y ejes son obligatorios: `units: "SI"`, `model.axes: "Z-up"`, terna
diestra X-Y horizontal y Z arriba. D2 usa el plano XY (Z=0) y Ux/Uy/Rz; D3
admite seis GDL. Se usan m, kg, s, N, N·m, Pa y rad, sin conversión automática.
La validación estructural previa rechaza NaN/Inf, coerciones, claves duplicadas,
ids fuera de u32, tamaños/índices fuera de contrato y campos desconocidos de
entrada. Los límites son los schemas del Rust instalado (además de 8 MiB de
entrada, profundidad 64 y arrays de hasta 16384 entradas). La topología y toda
validación/cálculo físico pertenecen a engine-api.

Los enteros Python y los literales JSON sin decimal/exponente requieren el
rango simétrico `-(2**53-1) <= n <= 2**53-1`, también en campos numéricos físicos.
Fuera de él se lanza `InvalidInputError` con la ruta antes del FFI (#143).
Rust aplica el mismo control antes de JCS para evitar identidades ambiguas en
stale. `bool` no se acepta como id ni contador. Los floats finitos conservan
binary64 y su identidad canónica (incluida la igualdad de `4` y `4.0`);
no se conserva la escritura original de un float. Los resultados importados
conservan extensiones sin imponerles el rango de los enteros de entrada.

`EngineError` conserva `code`, `entities`, `cause`, `severity` y `details` del
primer diagnóstico con severidad `error`; `diagnostics` conserva todos los
avisos/errores y `document` conserva el JSON completo cuando Rust lo produjo.
`results` permite inspeccionar ese documento fallido con `partial=True`;
si no puede interpretarse, sigue disponible `document` sin ocultar el error.
No se devuelven fallos como éxito. Las subclases son `InvalidInputError`,
`AnalysisError`, `NotConverged`, `Unstable`, `Cancelled` (también
`CancelledError`), `Unsupported` e `InternalEngineError`. `StaleResultsError`
es local a Python. Contrato incompatible usa `ENG-E-0009`, método no soportado
`ENG-E-0013`; otros defectos de frontera usan `API-E-INPUT` y un unwind contenido
usa `ENG-E-0012`. Los errores locales incluyen el path en `details`.

## GIL, progreso y cancelación

`run` y `validate` liberan el GIL con `py.detach`. `CancelToken()` posee
un `Arc<engine_api::CancelToken>` atómico. Otro hilo Python puede llamar
`token.cancel()` durante `run(..., cancel=token)`; `token.is_cancelled` consulta
el estado. Cancelar es idempotente y permanente; cada operación nueva requiere
un token nuevo si el anterior se canceló. El motor consulta este mismo token
antes/después de etapas y entre iteraciones P-Delta/modos/direcciones. Cada
solve/eigensolver, parseo y ensamblado es indivisible: la cancelación es
cooperativa, no interrumpe una etapa en curso.

Si hay `progress`, el callback Rust únicamente acumula estructuras `Progress`.
Tras terminar el motor, se readquiere el GIL y se invoca el callback Python por
cada evento, en orden, también si el motor falló. Cada evento es un diccionario
con `operation`, `stage`, `completed`, `total`. Sin callback no se acumulan
eventos. Esta decisión evita ejecutar código Python dentro de iteraciones;
el progreso es diferido, no una actualización en vivo. Cancelar desde ese
callback afecta a futuros usos del token, pues el análisis ya terminó. Para
cancelar la operación actual se usa otro hilo. Un fallo del callback se propaga
sin envolver y detiene la entrega de eventos; si el motor también falló,
prevalece su `EngineError`.

## Tablas y NumPy opcional

`Results.to_arrays()` exporta `ArrayResults` con listas; `Results.to_numpy()`
exporta el mismo contrato con copias `ndarray`, **dtype float64, orden C,
memoria propia** (`OWNDATA`, `base is None`). Cada llamada crea una salida
independiente; modificarla o destruir Results nunca afecta al backend ni a
otra salida. Ambos accesos verifican stale. Los arreglos ya exportados siguen
siendo válidos al cambiar el modelo. La conversión usa
`numpy.asarray(..., dtype=numpy.float64, order="C").reshape(shape).copy(order="C")`.

```python
import mamut_engine as me

result = me.run(model)  # model es un engine-input/1 explícito
listed = result.to_arrays()  # funciona sin NumPy
exported = result.to_numpy()  # requiere el extra mamut-engine[numpy]
table = exported.blocks[0].matrices["displacements"]
print(exported.status, exported.partial, table.shape)
print(table.row_ids, table.dofs, table.units, table.values)
```

El extra `numpy = ["numpy>=1.23"]` es opcional. Solo `to_numpy()` importa NumPy;
si falta lanza `MissingNumPyError` (`EngineError`, `PY-E-NUMPY`) indicando
`mamut-engine[numpy]`. La API básica, JSON, listas y anotaciones no lo requieren.
En un entorno offline debe estar provisionado o instalarse desde un almacén
local con `pip --no-index`; el paquete no descarga dependencias por sí mismo.
Sin NumPy, las pruebas opcionales usan `pytest.importorskip("numpy")`.

`ArrayResults.document` conserva una copia completa de `results/1`, incluyendo
estado, diagnósticos, convergencia, huella, identidades nodales/de elemento,
conectividad, GDL activos y campos no tabulados. `status` es el string original;
`partial` mantiene el criterio de Results, incluso para errores con bloques
vacíos. El estado no se convierte a códigos f64. No se fabrica una solución
cuando el análisis falla.

Cada `ResultBlock.path` localiza el origen dentro de document: `("cases", i)`,
`("combinations", i, "results")`, `("modes", i, "shape")`,
`("modes", i, "spectrum_responses", j)` o `("envelopes", i)`.
Los índices son posiciones originales, no IDs. No se mezclan bloques.

| Matriz en `block.matrices` | Shape | GDL/columnas | Unidades por columna | Ejes |
| --- | --- | --- | --- | --- |
| `displacements` | `(n,6)` | Ux,Uy,Uz,Rx,Ry,Rz | m,m,m,rad,rad,rad | globales |
| `reactions` | `(n,6)` | Ux,Uy,Uz,Rx,Ry,Rz (fuerza conjugada) | N,N,N,N*m,N*m,N*m | globales |
| `end_forces_local/frame2d.euler_elastic` | `(e,6)` | i.Ux,i.Uy,i.Rz,j.Ux,j.Uy,j.Rz | N,N,N*m por extremo | locales |
| `end_forces_local/frame3d.euler_elastic` | `(e,12)` | i.Ux,…,i.Rz,j.Ux,…,j.Rz | N,N,N,N*m,N*m,N*m por extremo | locales |
| `end_forces_local/truss2d.linear` | `(e,4)` | i.Ux,i.Uy,j.Ux,j.Uy | N por columna | locales |
| `end_forces_local/truss3d.linear` | `(e,6)` | i.Ux,i.Uy,i.Uz,j.Ux,j.Uy,j.Uz | N por columna | locales |
| `forces_local` | `(s,6)` | N,Vy,Vz,T,My,Mz | N,N,N,N*m,N*m,N*m | locales, cara positiva |
| `positions` | `(s,1)` | x/L | 1 (adimensional) | locales |

`n` cuenta filas nodales, `e` miembros de ese tipo, `s` estaciones. Las tablas
de extremos solo existen para tipos presentes en el bloque. Las de estaciones
solo aparecen en casos/combinaciones con campo `members`. Vacíos conservan
shape `(0,k)`, también `(0,1)` para posiciones; no se devuelve `(0,)`.
Los modos normalizados a masa conservan amplitudes originales: sus columnas
de desplazamiento tienen unidades kg^-1/2 (traslaciones) y m^-1*kg^-1/2
(rotaciones). Sus reacciones son los ceros originales del motor, no una
recuperación de fuerzas modales. Respuestas espectrales/envolventes usan SI.
Los GDL ausentes conservan ceros del contrato; consulte `document["nodes"]`
para distinguirlos de GDL activos. No se convierten unidades ni signos.

Cada tabla lleva `row_ids` (IDs enteros exactos en orden de filas), `row_indices`
(0 para nodos/extremos, índice original de estación por miembro), `dofs`,
`units`, `shape` y `frame`. En estaciones se repite el ID del miembro; posiciones
y fuerzas comparten IDs/índices. No se presupone que el orden de identidades
coincida con el de las respuestas. Elementos mixtos se separan por tipo sin
relleno ni dtype object. Shapes incompatibles, valores no finitos e IDs sin
identidad/duplicados dan `InvalidInputError`; etiquetas de elementos desconocidos
dan `Unsupported`. Los campos nullable, extremos, participación y convergencia
siguen como listas/JSON en document; no se reemplaza null por NaN.

Decisión: [ADR-004](../../docs/adr/ADR-004-numpy-python.md).
Plan/script de rendimiento: [benchmarks offline](../../docs/PYTHON_BOUNDARY_BENCHMARKS.md).
No hay dependencias Cargo nuevas, física en Python ni liberaciones nuevas del GIL.

## Construcción y pruebas sin red

Desde la raíz, con toolchain, dependencias y maturin 1.15.0 ya instalados:

```bash
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1
# Exportar CARGO_TARGET_DIR a la carpeta de compilación autorizada.
# MATURIN debe ser el ejecutable absoluto instalado localmente.
MATURIN="$MATURIN" bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/python-dist-nuevo"
```

La salida debe ser nueva. El script construye dos wheels release independientes,
compara sus bytes y las instala con `--no-index` en un venv temporal dentro de
`CARGO_TARGET_DIR`. Ejecuta `bindings/python/tests` contra la wheel instalada,
sin Rust en el PATH del consumidor. Genera referencias directas de engine-api
para los cinco ejemplos válidos y once casos de determinismo, incluido frame3d.
Los tests comparan los documentos completos y SHA-256 con tolerancia cero;
verifican round-trip, extensiones opcionales, contratos mayores incompatibles,
SI/ejes, validación estructural, stale, errores, callbacks y cancelación.
También contrastan el voladizo con `u=-PL³/(3EI)`, `R=P`, `M=PL` para P=12,
L=4, E=200 e I=3, con tolerancias absoluta/relativa `1e-12` por redondeo f64.
Son fixtures propios; las referencias nunca se generan mediante el binding.
La conformidad de transporte del CLI usa además los
[manifiesto SHA-256 de stdout cli-run/1](../../crates/engine-cli/tests/golden/stdout-sha256.json) (MAM-868),
verificados por [tests/contract.rs](../../crates/engine-cli/tests/contract.rs).
`tests/test_run.py::RunTest.test_cantilever_matches_rust_bytes_and_sha256`
conserva la referencia directa de results/1; los golden del CLI añaden el sobre
publicado y contemplan explícitamente la procedencia de compilación.

Si se dispone de mypy o pyright offline se ejecuta el checker. En su ausencia,
`test_typing.py` verifica los exports/reexports `.pyi`, firmas nativas instaladas
y campos requeridos/opcionales de los DTO frente a los schemas Rust. Además se
resuelven las anotaciones públicas con `get_type_hints`. Esta alternativa
comprueba consistencia contractual y no equivale a un análisis estático completo.
Solo se requiere la biblioteca estándar para las pruebas offline.

La prueba Rust `tests/unwind.rs` inyecta un unwind desde el callback de progreso
de engine-api, comprueba la barrera compartida con PyO3 y una ejecución posterior
válida. No necesita enlazar un intérprete Python embebido; la extensión se
comprueba instalada en Python. No hay `panic!`, `unwrap()` ni `expect()` en la
biblioteca del adaptador.

## Aislamiento y verificaciones Rust

Python sigue fuera de `default-members`. Ningún crate del motor depende de
PyO3 ni del binding; las dependencias del binding son `engine-api`,
`serde_json` y PyO3, solo para destinos nativos. En wasm32 queda vacío.
`pyo3/extension-module` se activa solo desde maturin; ABI estable `abi3-py310`.

```bash
cargo fmt --check
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo clippy --offline -p mamut-engine-py --all-targets -- -D warnings
cargo test --offline --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace
cargo check --offline --workspace --all-targets --target wasm32-unknown-unknown
bash clean-room/check.sh origin/main
```

Verificado localmente con Rust 1.99.0, maturin 1.15.0 y CPython 3.13.5/Linux.
La wheel es `cp310-abi3-manylinux_2_34_x86_64`; no se verificaron otros
intérpretes, Windows/macOS ni manylinux2014. No hay dependencias Python obligatorias de
runtime; NumPy se ofrece como extra opcional. El consumidor de la wheel no necesita Rust ni conexión.

## Construcción y consumo offline

Desde la raíz del repo, con CPython >=3.11 (usa `tomllib`), Rust de
`rust-toolchain.toml`, maturin **1.15.0** y dependencias ya disponibles localmente:

```bash
# Selecciona binarios locales de la toolchain fijada; no descarga ni instala Rust.
export PATH="$RUST_TOOLCHAIN_BIN:$PATH"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
export MATURIN="${MATURIN:-maturin}" # También acepta un ejecutable absoluto.
bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/python-dist-v1"
(cd "$CARGO_TARGET_DIR/python-dist-v1" && sha256sum --check SHA256SUMS)
```

La salida debe ser nueva. `PYTHON` permite seleccionar el intérprete. El script
no instala herramientas; falla si falta una dependencia offline. Usa Cargo
`--offline --locked`, prohíbe la instalación automática de rustup y comprueba
la versión exacta del compilador. `SOURCE_DATE_EPOCH=1767225600` es una constante
versionada (2026-01-01 UTC). Dos builds release con directorios target separados,
rutas Rust remapeadas y binario stripped deben dar la misma wheel y SHA-256.
Si difieren, falla e imprime hashes globales y diferencias de entradas ZIP.
La reproducibilidad acreditada es local con las mismas herramientas, no entre SO.

Instala con `pip --no-index --no-deps --no-cache-dir` en un venv nuevo cuyo
`PATH` solo contiene sus binarios. Comprueba que `rustc`, `cargo` y `rustup` no
se encuentran, importa el paquete y llama realmente `version()` y la función
nativa. Ejecuta además las pruebas de la fachada completa sobre la wheel
instalada: validate/run/capabilities/schema/explain/version, errores, progreso y
cancelación. Antes genera una referencia independiente con `facade_reference`
para comparar los bytes de `results/1` con Rust.

Junto a la wheel produce:

- `SHA256SUMS`: wheel y todos los artefactos adjuntos.
- `COMPATIBILITY.json`: versiones binding/engine, contratos, tags reales de
  Python/ABI/plataforma, herramientas e intérprete probado; `contract_status`
  declara la fachada completa, igual que la propiedad del SBOM.
- `SBOM.cdx.json`: CycloneDX 1.5 determinista desde `cargo metadata`, cierre
  transitivo del binding para todos los destinos, incluyendo build dependencies;
  excluye miembros ajenos del workspace, herramientas y bibliotecas del sistema.
  Los `bom-ref` son identidades de paquete independientes del checkout. Un chequeo
  recursivo rechaza rutas absolutas; incluye pruebas de regresión del issue #94.
- `THIRD_PARTY_NOTICES`: expresiones de licencia declaradas por esos crates;
  las ausentes y la revisión legal se marcan PENDIENTE. Es un inventario, no un
  paquete completo de textos legales ni autorización para distribuir.
- `REPRODUCIBILITY.json` y `SMOKE.json`: evidencia de igualdad y consumo sin Rust.

La inspección de la wheel exige `LICENSE`/`NOTICE` idénticos a los del binding
y sus campos `License-File` (#93). No se versionan wheels ni artefactos.

## Matriz candidata V1

| Plataforma | CPython 3.11 | 3.12 | 3.13 | 3.14 |
| --- | --- | --- | --- | --- |
| Linux x64 | PENDIENTE | PENDIENTE | **Probado 3.13.5** | PENDIENTE |
| Windows x64 | PENDIENTE | PENDIENTE | PENDIENTE | PENDIENTE |
| macOS arm64 | PENDIENTE | PENDIENTE | PENDIENTE | PENDIENTE |

El binding ya configura `abi3-py310` y `requires-python >=3.10`. El tag
`cp310-abi3-manylinux_2_34_x86_64` se inspeccionó y cargó en Linux x64/3.13.5;
**compatibilidad abi3 entre intérpretes PENDIENTE hasta probar cada celda**.
3.10 no forma parte de la matriz candidata V1. No se afirma manylinux2014.
Los candidatos aún no constituyen soporte validado.

CI de Actions: PENDIENTE por falta de presupuesto. Runner VPS: PENDIENTE de
migración por J0. No se modifica `.github/**`. El script es la comprobación local
reutilizable cuando haya runners; otros SO necesitan su propia validación.

Comprobaciones del workspace, con `CARGO_TARGET_DIR` exportado:

```bash
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
cargo fmt --all -- --check
cargo clippy --offline --locked --workspace --all-targets -- -D warnings
cargo test --offline --locked --workspace --no-fail-fast
RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked --no-deps --workspace
cargo check --offline --locked --workspace --target wasm32-unknown-unknown
bash clean-room/check.sh origin/main
```

## Rollback local

Conserva cada versión anterior en su propia carpeta, junto con `SHA256SUMS` y
`COMPATIBILITY.json`. En el mismo SO/intérprete, valida primero sus sumas y tags.
Define `PREVIOUS_DIST`, `PREVIOUS_WHEEL` (archivo exacto, sin glob ambiguo),
`PREVIOUS_ENGINE_VERSION` y `CONSUMER_PYTHON` (Python absoluto del venv consumidor):

```bash
(cd "$PREVIOUS_DIST" && sha256sum --check SHA256SUMS)
"$CONSUMER_PYTHON" -I -m pip install --no-index --no-deps --no-cache-dir \
  --force-reinstall "$PREVIOUS_WHEEL"
"$CONSUMER_PYTHON" -I -c 'import importlib.metadata as m, mamut_engine as e; print(m.version("mamut-engine"), e.version())'
"$CONSUMER_PYTHON" -I -c 'import sys, mamut_engine as e; v=e.version(); assert (v["engine_version"] if isinstance(v, dict) else v) == sys.argv[1]' "$PREVIOUS_ENGINE_VERSION"
```

Comprueba también que la versión de distribución coincide con el manifiesto
anterior. La comprobación admite tanto el dict de la fachada como la cadena
del scaffold anterior. Usa un proceso Python nuevo: el módulo nativo ya cargado no se reemplaza
en memoria. Detén/reinicia consumidores antes de reabrirlos. Si falla el smoke,
reinstala la wheel conservada de la versión que funcionaba y vuelve a comprobar.
La evidencia y las limitaciones del ensayo local se registran en
`provenance/ENG-PY-007.md`; no equivale a rollback de una versión publicada.

## Experimentos externos · ENG-PY-005 / MAM-80

`mamut_engine.experiments` es Python puro en el proceso del usuario. Ejecuta
copias del modelo mediante `run` de ENG-PY-003; Rust sigue validando el modelo,
resolviendo el registro SDK y calculando. No añade métodos oficiales, servidor,
planificador, plugins ni callbacks. No depende de NumPy.

```python
from mamut_engine.experiments import sweep, comparison_table, methods

profile = {
    "schema": "mamut.experiment/1", "name": "carga-voladizo", "version": 1,
    "seed": 42,
    "parameters": {"P": {
        "path": ["cases", 0, "loads", 0, "values", 0, "value"],
        "values": [-6, -12, -24],
    }},
    "metrics": {"Uy": {
        "path": ["cases", 0, "nodes", 1, "displacements", 1], "unit": "m",
    }},
}
rows = sweep(model, profile)  # model: Model, dict engine-input/1 o JSON str
print(comparison_table(rows))
print(methods())  # descriptores originales de capabilities(), sin registro Python
```

El perfil es JSON versionado: `schema` fija el contrato, `version` identifica la
revisión del experimento y `name` su identidad. `seed` es metadato opcional para
el productor de los datos; este módulo no genera números aleatorios. El mismo
perfil y modelo, con la misma compilación/target Rust, repiten las huellas.
El barrido usa producto cartesiano en orden de nombres de parámetros y en el
orden declarado de valores. Las rutas son listas de claves/índices existentes,
sin expresiones ejecutables; se rechazan solapamientos, bool y NaN/Inf antes
del lote. No se mutan el modelo, perfil ni listas originales.

`compare_scenarios({nombre: modelo, ...}, profile)` ejecuta por nombre ordenado
con un perfil sin `parameters`; devuelve los mismos registros que `sweep`.
`comparison_table` produce filas con métricas, unidades declaradas, estado,
huellas y procedencia. Las métricas seleccionan escalares de results/1 por
ruta, sin agregación ni conversión; `unit` es una etiqueta declarada por el
usuario, que debe corresponder a las unidades SI del campo seleccionado.
El resultado completo conserva unidades implícitas del contrato y sus GDL.

Cada `VariantResult` conserva `result`, `error` tipado original y su diagnóstico,
`input_sha256` JCS de Rust, `result_sha256` de los **bytes UTF-8 exactos** de
results/1, perfil completo, versión del engine, `git_commit` y target emitidos
por Rust. Un commit no suministrado al compilar queda `None`. Se conserva el
estado original incluso en inestabilidad/no convergencia. Una entrada que no
puede obtener huella se identifica con `None`. Las fallas Engine se guardan por
variante y continúa el lote; `row.raise_for_error()` vuelve a lanzar la misma
instancia. Errores del perfil se lanzan antes de empezar; otras excepciones
Python se propagan. Un selector de métrica fallido marca `metricError` y conserva
el resultado Rust y su estado en `row.result.status`.

Si el error no adjunta `Results`, se usa primero el estado de fallo de su
documento JSON, si es legible. En su ausencia se mapea la categoría pública:
`InvalidInputError` → `invalid`, `Unstable` → `unstable`, `NotConverged` →
`notConverged`, `Unsupported` → `unsupported`, `Cancelled` → `cancelled` e
`InternalEngineError` → `internalError`. Un `AnalysisError` genérico queda
`analysisError`; otro `EngineError`, `error`. Estos dos últimos son estados
de la variante Python. Se conserva la instancia original con su código y
diagnósticos. Un documento ilegible no oculta el error original.

El campo opcional `analyses` del perfil sustituye la lista completa del modelo
para cada ejecución, con el contrato engine-input/1 existente. La procedencia
conserva `base_analyses`, `requested_analyses` y los `methods` realmente emitidos
por Rust. Esto permite comparar métodos registrados sin introducir otro
registro ni aceptar funciones Python como métodos.

Los ejemplos pequeños usan el fixture existente de voladizo:

```sh
python bindings/python/examples/parametric_sweep.py
python bindings/python/examples/compare_scenarios.py
```

### Workflows SDK existentes

La fachada FFI no expone objetos `CompiledWorkflow`. El transporte mínimo
`compile_workflow` / `run_workflow` usa el **CLI Rust local existente**, sin Cargo
en runtime. Python solo escribe snapshots temporales de los documentos y lee
su respuesta. El compilador, validación, registro, proveedor y executor son los
de ENG-SFC-002 / SDK-WF-003 / ENG-CORE-006, sin implementación paralela.

```python
import json
from pathlib import Path
from mamut_engine.experiments import compile_workflow, run_workflow

catalog = json.loads(Path("crates/engine-cli/catalog.json").read_text())
workflow = json.loads(Path("crates/engine-cli/run.workflow.json").read_text())
compiled = compile_workflow(catalog, workflow, executable="/ruta/local/mamut-engine")
execution = run_workflow(compiled, model, {
    "schema": "mamut.experiment/1", "name": "workflow-voladizo", "version": 1,
})
manifest = execution["envelope"]["data"]
```

`compiled.plan` conserva el plan emitido por Rust. El CLI actual no importa
planes serializados: `run` **recompila los mismos snapshots** antes de ejecutar.
No se simula la ejecución del plan en Python. El sobre íntegro conserva
`engineResults`, `recordFingerprint`, huellas de entradas/salidas de cada paso,
selecciones efectivas (incluidas sustituciones) y estados del SDK. La respuesta
adjunta el perfil y la huella del modelo; no cambia el manifiesto sellado.
Los workflows usan el modelo recibido y sus selecciones compiladas: el perfil
en esta vía no admite `parameters`, `metrics` ni `analyses`. Los errores del CLI se mapean
a errores públicos tipados; si hay results/1, se conservan los diagnósticos y
el documento semántico. Sin documento Engine, `cli-error/1` conserva el código
estable y `error.message`, los diagnósticos y el sobre íntegro en
`details["workflow_envelope"]`; `results` y `document` quedan en `None`.
El código `SDK-EXIT-*` se mantiene para sobres anteriores del SDK.
La representación JSON anidada del CLI no conserva
los bytes originales de results/1; sus huellas SDK sí se conservan intactas.
El binario debe instalarse aparte de la wheel. No se carga Python en CLI,
WASM, MCP ni dentro del solver.

Ambas funciones aceptan `timeout` solo por keyword, en segundos, con valor
predeterminado de 300. Debe ser un número finito mayor que cero (sin bool).
Al expirar se lanza `AnalysisError` con código `PY-E-WORKFLOW-TIMEOUT` y causa
`subprocess.TimeoutExpired`. En POSIX el CLI se lanza en una sesión propia y
el timeout termina su grupo de procesos completo, incluidos los descendientes
de envoltorios shell/scripts: SIGTERM, hasta 0,2 s de espera y SIGKILL si el
grupo sigue existiendo. En plataformas sin `os.killpg` (Windows) solo se mata
el proceso directo. Un descendiente que abandone el grupo (por ejemplo, con
`setsid`) sobrevive al timeout; si conserva las tuberías abiertas, el error puede
llegar hasta unos 5 s más tarde por la espera acotada de limpieza. Ctrl-C y otras
excepciones durante la espera también terminan el grupo; `KeyboardInterrupt`
se propaga como tal. El transporte recoge el proceso CLI y elimina sus archivos
temporales antes de propagar el error o la interrupción.

Las pruebas nuevas son descubribles tanto con pytest como con unittest, para
incluirlas en `scripts/build-python-wheel.sh`. Para cubrir el SDK real, construir
`cargo build -p engine-cli --offline` y exportar `MAMUT_EXPERIMENT_CLI` con la
ruta absoluta al binario antes de ejecutar pytest o el script de wheel. Sin esa
variable solo se omiten los tests de transporte SDK, no los de la fachada.


## Explicit language (MAM-836, phase 1)

`validate(model, *, lang="en")`, `run(model, *, lang="en", cancel=None,
progress=None)` and `explain(code, *, lang="en")` (also through `Engine`) accept
only `"en"`, `"es"`, `"pt"`. The facade checks type and value before FFI.
The native signatures are `validate(input_json, lang=None)`,
`run(input_json, progress=None, cancel=None, lang=None)` and
`explain(code, lang=None)`; native None means English, and unsupported identifiers
raise InvalidInputError with code API-E-INPUT. Locale settings never select a
language. For example, `engine.validate(model, lang="es")` returns Spanish default
ENG diagnostic causes and `engine.explain("ENG-E-0004", lang="pt")` explains the
code in Portuguese.

engine-api is the sole localization implementation. English keeps the existing
bytes. Localized run failures carry localized `cause`, `diagnostics` and
`document`; codes, severity, entities, status and hint stay unchanged. Successful
results differ only in translatable `diagnostics[].cause`.

Phase 2 covers specific/parameterized causes, hints, ApiError/parse messages,
Python-owned PY-E-* and facade validation messages, mcp/compile/methods and
describe/help language. Those messages are unchanged in phase 1. Translations
await native-speaker review, especially Portuguese. The CLI workflow fingerprint
excludes diagnostic cause/hint text, so a sealed record does not attest to message
wording; Python results retain the full localized document.
