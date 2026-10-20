# Bitácora clean-room · ENG-PY-004

- Ticket / PR: MAM-82 · ENG-PY-004 · PR #156; seguimiento local sin commit ni publicación.
- Autor(es): Codex gpt-6.1-sol high · **2026-10-17** (America/Guayaquil).
- Rol: implementador.
- Contrato implementado: proyección tabular de `results/1` en el paquete
  `mamut_engine` sobre PyO3; listas y NumPy opcional, sin modificar el motor.
- Fuentes permitidas usadas: exclusivamente fuentes propias del checkout:
  `bindings/python`, `crates/engine-api` (schemas, fixtures y serialización),
  `crates/engine-analysis/src/results.rs` (orden, unidades y estaciones),
  contratos de GDL de los elementos frame/truss propios, CLI propio,
  `docs/adr/ADR-003-python-v1.md`, `provenance/ENG-PY-003.md`,
  `docs/DETERMINISMO.md`, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md` y
  scripts locales de construcción/artefactos. Sin fuentes externas.
- Derivaciones propias: ninguna física nueva. El benchmark solo replica y
  renumera el fixture analítico propio del voladizo. Las referencias Rust de
  los 16 fixtures existentes son independientes del binding. Los datos mixtos
  importados de la prueba tabular verifican representación, no física.
- Código de terceros copiado: ninguno al repositorio. Sin cambios en Cargo.toml,
  Cargo.lock, vendor, clean-room, workflows ni CODEOWNERS; sin dependencias Rust
  nuevas, unsafe, ni liberaciones nuevas del GIL.
- Validación: comparación exacta de documentos con engine-api Rust y tablas
  contra sus filas originales; oráculos analíticos propios ya existentes.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código ni resultados de CalculiX o LAPACK,
  ni salidas/informes de otros motores. Trabajo exclusivamente offline.
- Revisor clean-room: **PENDIENTE (J4)**.

## Decisiones y alcance

[ADR-004](../docs/adr/ADR-004-numpy-python.md) registra el extra opcional
`numpy>=1.23`, importación perezosa y `MissingNumPyError` (`PY-E-NUMPY`).
`Results.to_arrays()` y `Results.to_numpy()` devuelven `ArrayResults` con
documento completo independiente, estado y bloques por caso, combinación,
modo, respuesta espectral y envolvente. Cada matriz lleva shape, IDs enteros,
índice de estación, GDL, unidad por columna y ejes. Las fuerzas de extremo se
separan por tipo de elemento. Se conservan los órdenes propios del documento.

NumPy recibe listas con `asarray(dtype=float64, order="C")`, reshape explícito
y `.copy(order="C")`, incluyendo vacíos `(0,k)`. Memoria propia, sin vistas
ni préstamos del backend; sobreviven a Results y no comparten almacenamiento
entre exportaciones. El estado textual, nullable, extremos y convergencia
permanecen en el documento completo. No se usan NaN como marcadores de null.
Solo se transforman representaciones y se comprueban shapes/finitud/identidad;
toda la física y aritmética de análisis permanece en Rust. No se cambian
resultados numéricos ni se optimiza el GIL.

## Construcción y pruebas offline

Rust 1.99.0 stable, maturin 1.15.0, CPython 3.13.5/Linux x86_64,
NumPy 2.5.4 y pytest 9.1.1. `CARGO_NET_OFFLINE=true`, Cargo `--offline`,
`PIP_NO_INDEX=1`, sin publicación en PyPI. `CARGO_TARGET_DIR` apunta al
directorio autorizado, sin artefactos binarios versionados.

```bash
export PATH="$RUST_TOOLCHAIN_BIN:$PATH"
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1
MATURIN="$MATURIN" bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/mam82-dist"
python3 -m venv --system-site-packages "$CARGO_TARGET_DIR/mam82-consumer"
"$CARGO_TARGET_DIR/mam82-consumer/bin/python" -I -m pip install \
  --no-index --no-deps --no-cache-dir "$CARGO_TARGET_DIR/mam82-dist/"*.whl
```

Pytest estaba provisionado en otro entorno local, fuera del site del Python
del sistema. Se lo expuso mediante un `.pth` únicamente dentro del venv
temporal, apuntando a `$PYTEST_SITE_PACKAGES`; sin descargar ni modificar
paquetes externos. NumPy procede del site del sistema. Se generaron las
16 referencias propias en `$CARGO_TARGET_DIR/mam82-references` con el binario
`facade_reference` de Rust. Ejecución de la wheel instalada:

```bash
export MAMUT_RUST_REFERENCES="$CARGO_TARGET_DIR/mam82-references"
export MAMUT_RUST_REFERENCE="$MAMUT_RUST_REFERENCES/cantilever-linear-static.json"
export PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 PYTHONDONTWRITEBYTECODE=1
"$CARGO_TARGET_DIR/mam82-consumer/bin/python" -I -B -m pytest \
  -q -p no:cacheprovider bindings/python/tests
```

- Artefactos: **25 passed / 0 failed** en `scripts/tests`, ejecutados por el
  constructor y `scripts/python-wheel-artifacts.py` para inspección y comparación.
- Consumidor stdlib aislado sin NumPy: **38 passed / 0 failed** (unittest).
- Consumidor temporal con NumPy: **59 passed / 0 failed**, además de
  **241 subtests passed** (pytest, incluye los 38 básicos).
- Ausencia simulada de NumPy en proceso nuevo: **6 passed / 0 failed,
  1 skipped**, además de 14 subtests; `pytest.importorskip` salta el módulo
  opcional limpiamente. La API básica y el error tipado se verifican sin NumPy.
- Casos de frontera: NaN/±Inf, valores no representables como f64, shapes
  incorrectos/vacíos, ndarray de entrada fuera del contrato JSON, IDs máximos
  no consecutivos, duplicados/desconocidos, tipos de elementos mixtos,
  unidades/etiquetas, stale, estado fallido, mutación y vida de copias.

Wheel `mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`:
**dos construcciones release independientes, mismo SHA-256**:
`bf353bdd5e1c2a2e3caa5f9bf7006d70cd59b990b1268db4291cb41bdb3f6b8f`.
Evidencia: `$CARGO_TARGET_DIR/mam82-dist/REPRODUCIBILITY.json`, `SHA256SUMS`,
SBOM/compatibilidad/smoke y `$CARGO_TARGET_DIR/mam82-dist.log`.

## Rendimiento

Plan y reproducción: [benchmark de frontera](../docs/PYTHON_BOUNDARY_BENCHMARKS.md).
`scripts/benchmark-python-boundary.py` usa la wheel instalada y el ejemplo Rust
`boundary_benchmark`; CLI y baselines se compilan release offline antes de medir.
Tres ejecuciones completas con tamaños 1/16/64, calentamiento 2 y 7 muestras por
métrica. Cada ejecución comprueba igualdad Rust/CLI/fachada. No hay umbrales CI.

Medianas de la tercera ejecución, **ms**, máquina compartida; evidencia local
exploratoria, sin promesa de rendimiento ni aislamiento de CPU:

| Réplicas | Rust API | CLI/proceso | PyO3/JSON | Fachada run | Parse Results | Listas | NumPy |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.1334 | 5.0433 | 0.2247 | 1.2682 | 0.5406 | 0.5031 | 0.5117 |
| 16 | 1.3964 | 4.4718 | 1.7600 | 8.4416 | 5.0583 | 2.8345 | 2.9979 |
| 64 | 14.3334 | 22.2011 | 14.1618 | 41.9114 | 21.9639 | 10.9600 | 11.8592 |

Las diferencias de medianas pueden ser negativas por ruido. CLI incluye arranque,
E/S y sobre; exportar incluye stale/huella, copia de documento y tablas. No se
identifica ese tiempo con el solve ni se suman fases solapadas. Informes con
muestras/versiones/bytes/hashes: `$CARGO_TARGET_DIR/mam82-benchmark.json`,
`$CARGO_TARGET_DIR/mam82-benchmark-2.json` y
`$CARGO_TARGET_DIR/mam82-benchmark-3.json`. No se mide RSS ni importación fría.

## Comprobaciones obligatorias y determinismo

- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: **658 passed / 0 failed / 0 ignored**,
  incluyendo doctests (104 suites).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `cargo check --workspace --offline --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main`: OK.
- `git diff --check`: OK.

Sin definir `MAMUT_ENGINE_GIT_COMMIT`, se ejecutaron los ejemplos
`determinism_native`/`determinism_wasm` en debug y release, con
`scripts/check-determinism.mjs`: **16/16 hashes nativo=wasm**, diez corridas
por caso/target/perfil; `cmp` nativo debug/release OK. Comparación explícita
con la tabla de `docs/DETERMINISMO.md`: **16/16 idénticos** en ambos perfiles.
Logs y salidas en `$CARGO_TARGET_DIR/mam82-determinism-*`; comprobaciones del
workspace en `$CARGO_TARGET_DIR/mam82-{fmt,clippy,test,doc,wasm}.log`.

Pendiente: revisión clean-room J4 y plataformas/intérpretes distintos de los
probados. No se requiere modificar las rutas protegidas ni publicar artefactos.

## Seguimiento de revisión OpenCode · PR #156

- Autor: Codex gpt-6.1-sol high · **2026-10-17** (America/Guayaquil).
- Rol: implementador. Revisor clean-room: **PENDIENTE (J4)**.
- Fuentes: exclusivamente el código, contratos, fixtures y scripts propios
  citados arriba, más las observaciones de revisión aportadas por el usuario.
  Sin fuentes externas, dependencias nuevas ni derivaciones físicas.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket." Tampoco consulté código ni resultados de CalculiX o
  LAPACK ni resultados de otros motores. Trabajo offline; código de terceros
  copiado: ninguno. Comparación con oráculo externo: no.

Se conserva el `ModuleNotFoundError` original como `__cause__` mediante
`raise MissingNumPyError(...) from exc`, con regresión en un proceso nuevo.
La validación tabular rechaza explícitamente `isinstance(v, bool)`; se prueban
True y False en desplazamientos, reacciones, fuerzas de extremo, fuerzas de
estación y posición directamente en el exportador, evitando que el esquema
JSON oculte esta comprobación.

Las regresiones mutan todas las matrices exportadas y comprueban documento,
snapshot (`to_dict` y `to_json`), una segunda exportación previa y una llamada
posterior. Comprueban objetos/listas/filas distintos y, con NumPy, ausencia de
memoria compartida y conservación de las copias tras liberar Results.
`numpy_test.py` pasa a `test_numpy.py`, con `unittest.TestCase`, subtests y
`unittest.skipUnless`; conserva los 16 fixtures Rust, los cuatro ndarray de
entrada inválidos, shapes vacíos, cancelación, IDs, etiquetas, unidades,
float64, orden C, memoria propia y finitud. Sin dependencia de pytest ni
cambios al constructor oficial.

La evidencia siguiente corresponde al seguimiento y sustituye los conteos
pytest y el hash de wheel históricos de las secciones anteriores.

### Reproducción y resultados del seguimiento

Rust 1.99.0, maturin 1.15.0, CPython 3.13.5 y NumPy 2.5.4. Todo offline:

```bash
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export CARGO_TARGET_DIR=/workspace/.cargo-target/me-py4
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1 PIP_DISABLE_PIP_VERSION_CHECK=1
export MATURIN=/workspace/venv-maturin/bin/maturin
bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/mam82-followup-dist"

mkdir "$CARGO_TARGET_DIR/mam82-followup-references"
for fixture in crates/engine-api/fixtures/input/valid/*.json \
    crates/engine-api/tests/fixtures/determinism/*.json; do
  cargo run --offline --locked --quiet -p mamut-engine-py \
    --example facade_reference -- "$fixture" \
    > "$CARGO_TARGET_DIR/mam82-followup-references/${fixture##*/}"
done
export MAMUT_RUST_REFERENCES="$CARGO_TARGET_DIR/mam82-followup-references"
export MAMUT_RUST_REFERENCE="$MAMUT_RUST_REFERENCES/cantilever-linear-static.json"

python3 -I -m venv --system-site-packages "$CARGO_TARGET_DIR/mam82-followup-with-numpy"
python3 -I -m venv "$CARGO_TARGET_DIR/mam82-followup-without-numpy"
for variant in with-numpy without-numpy; do
  consumer="$CARGO_TARGET_DIR/mam82-followup-$variant"
  "$consumer/bin/python" -I -m pip install --no-index --no-deps --no-cache-dir \
    "$CARGO_TARGET_DIR/mam82-followup-dist/"*.whl
  "$consumer/bin/python" -I -B -m unittest discover -s bindings/python/tests -v
done
```

- Constructor oficial: **25 passed / 0 failed** en scripts/tests y **40 passed /
  0 failed / 3 skipped** de 43 tests descubiertos en el consumidor sin NumPy.
- Venv con NumPy 2.5.4: **43 passed / 0 failed / 0 skipped**; los tres métodos
  NumPy se ejecutan mediante el mismo `unittest discover` del pipeline.
- Venv sin NumPy (ausencia real comprobada): **40 passed / 0 failed / 3 skipped**.
  El error tipado y su causa original se comprueban en ambos entornos.
- Wheel: dos construcciones release independientes, contenido y SHA-256
  idénticos: `56f8d9680ccf0615dba67aea2fc702f432ad5d987ff21a7096d26fdd8f567ab5`.
  Evidencia: `$CARGO_TARGET_DIR/mam82-followup-dist/REPRODUCIBILITY.json`,
  `SHA256SUMS`, SBOM/compatibilidad/smoke y `mam82-followup-wheel.log`.
- `cargo fmt --all -- --check`: OK.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: OK.
- `cargo test --workspace --offline`: **658 passed / 0 failed / 0 ignored**,
  incluyendo doctests (104 suites).
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline`: OK.
- `cargo check --workspace --offline --target wasm32-unknown-unknown`: OK.
- `bash clean-room/check.sh origin/main` y `git diff --check`: OK.
- Determinismo: ejemplos propios `determinism_native`/`determinism_wasm` y
  `scripts/check-determinism.mjs`, debug y release, diez corridas por caso:
  **16/16 hashes nativo=wasm**, `cmp` debug/release OK y **16/16 idénticos** a
  `docs/DETERMINISMO.md` en ambos perfiles, sin `MAMUT_ENGINE_GIT_COMMIT`.

Logs y referencias persistentes bajo `$CARGO_TARGET_DIR/mam82-followup-*`.
Cambios locales sin commit; sin modificar Rust, Cargo.toml/Cargo.lock, vendor,
clean-room, docs/CLEAN_ROOM.md, workflows ni CODEOWNERS. Pendiente únicamente
revisión clean-room **J4** y la matriz de otras plataformas ya pendiente.
