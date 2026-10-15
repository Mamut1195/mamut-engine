# MAMUT Engine · Python local

Bindings PyO3/maturin para Python >=3.10 sobre `engine-api`. El adaptador no
implementa cálculos numéricos: validación, análisis, códigos, capacidades y
contratos proceden de la fachada Rust.

Los metadatos declaran `AGPL-3.0-only`. La wheel incluye `LICENSE` y `NOTICE`.
Titularidad, aviso definitivo, licencia comercial y CLA: **PENDIENTE de revisión**
(ENG-GOV-004 / ENG-GOV-007). El inventario de licencias generado no resuelve
estas revisiones. **Verificación local únicamente: nada se publica ni se sube.**

## API

```python
import json
import mamut_engine as engine

source = open("input.json", encoding="utf-8").read()
diagnostics = engine.validate(source)["diagnostics"]
events = []
results_json = engine.run(source, progress=events.append)
results = json.loads(results_json)
```

- `validate(input_json: str) -> dict`: `{"diagnostics": [...]}`; como en Rust,
  los defectos topológicos son diagnósticos, no excepciones. No prueba estabilidad.
- `run(input_json: str, progress=None, cancel=None) -> str`: bytes JSON canónicos
  `results/1` de engine-api, sin volver a serializarlos en Python. Los estados
  `ok` y `warning` retornan normalmente; los demás producen `EngineError`.
- `version() -> dict`: `engine_version` y `contracts` de la fachada. Cambia el
  retorno del scaffold (antes cadena); `__version__` sigue siendo una cadena.
- `capabilities() -> dict`, `schema(name: str) -> dict` y
  `explain(code: str) -> str` delegan en las funciones homónimas de engine-api.

`EngineError` conserva `code`, `entities` (lista de `{kind, id}`) y `cause`.
Sus subclases son `InvalidInputError`, `AnalysisError`, `CancelledError` e
`InternalEngineError`. Los fallos de `run` además conservan `document` (JSON
canónico completo sin resultados parciales) y `diagnostics` (todos los
avisos/errores). Los atributos principales corresponden al primer diagnóstico
con severidad `error`. Contrato desconocido usa `ENG-E-0009`; los demás errores
de frontera, incluidos nombres/códigos desconocidos, usan `API-E-INPUT`.
Un unwind interceptado en la frontera produce `InternalEngineError`,
`ENG-E-0012`, entidades vacías y causa explícita, sin propagar el unwind al FFI.
Los tipos Python incorrectos producen `TypeError` de PyO3.

## GIL, progreso y cancelación

`run` y `validate` liberan el GIL con `py.allow_threads`. `CancelToken()` posee
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

## Construcción y pruebas sin red

Desde la raíz, con dependencias y herramientas ya instaladas:

```bash
# MATURIN: ruta a un maturin instalado; OUT: directorio temporal nuevo.
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1
OUT=$(mktemp -d)
cargo run --offline --quiet -p mamut-engine-py --example facade_reference -- \
  crates/engine-api/fixtures/input/valid/cantilever-linear-static.json \
  > "$OUT/cantilever-rust.json"
"$MATURIN" build --offline \
  --manifest-path bindings/python/Cargo.toml --out "$OUT/wheels"
python3 -m venv "$OUT/venv"
"$OUT/venv/bin/python" -m pip install --no-index --no-cache-dir "$OUT"/wheels/*.whl
MAMUT_RUST_REFERENCE="$OUT/cantilever-rust.json" \
  "$OUT/venv/bin/python" -m unittest discover -s bindings/python/tests -v
```

Use rutas temporales nuevas para una repetición en venv limpio. El ejemplo Rust
emite directamente `engine_api::run`; no llama al binding. `unittest` compara
bytes y SHA-256 con esa referencia, tres veces, con tolerancia cero. Además
contrasta el voladizo 2D con `u=-PL³/(3EI)`, `R=P`, `M=PL` para P=12, L=4,
E=200 e I=3; tolerancias absoluta/relativa `1e-12` por redondeo f64 de este
sistema pequeño. Cubre métodos avanzados, avisos, diagnósticos, errores,
callbacks, cancelación y liberación del GIL. Solo usa la biblioteca estándar.

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
intérpretes, Windows/macOS ni manylinux2014. No hay dependencias Python de
runtime. El consumidor de la wheel no necesita Rust ni conexión.

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
