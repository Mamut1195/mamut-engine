# ENG-PY-004 · plan offline de rendimiento de frontera

Benchmark manual, fuera de CI, sin red ni oráculos externos. Mide el mismo
motor Rust y el mismo documento por cada tamaño; Python solo replica el
fixture propio `cantilever-linear-static.json` y transporta resultados.
No implementa ecuaciones físicas. El oráculo analítico del fixture y las
referencias independientes permanecen en las pruebas del binding.

## Tamaños y baselines

| Tamaño por defecto | Voladizos independientes | Nodos | Miembros | GDL activos/libres |
| --- | ---: | ---: | ---: | ---: |
| Pequeño | 1 | 2 | 1 | 6/3 |
| Mediano | 16 | 32 | 16 | 96/48 |
| Grande | 64 | 128 | 64 | 384/192 |

Son tamaños relativos a la fachada V1 acotada y su backend de referencia;
no representan una estructura grande de producción. Réplicas independientes
mantienen el problema bien definido sin añadir leyes, conexiones ni métodos.
La traslación de coordenadas y renumeración de IDs no calculan respuestas.

- `rust_api`: ejemplo `boundary_benchmark`, múltiples llamadas directas a
  `engine_api::run` dentro del mismo proceso release. Excluye arranque y lectura
  del input; incluye parseo, análisis y serialización JSON del motor.
- `cli_process`: binario release `mamut-engine`, workflow/catalog propios.
  Incluye proceso, lectura, despacho y sobre CLI. El parseo comparativo del
  sobre se realiza fuera de la medición; no es una medida pura del FFI.
- `native_json`: `_native.run` sobre el mismo string, incluyendo cruce PyO3,
  detach/readquisición y copia de JSON al intérprete.
- `facade_run`: API pública completa, incluyendo validación previa, huella JCS
  y lectura/validación de `results/1`, además del análisis Rust.
- `parse_results`: `Results.from_json` de una salida ya calculada.
- `export_lists` y `export_numpy`: exportación de un Results ya calculado,
  incluyendo comprobación stale/huella del input y copia completa del documento.
  `export_numpy` incluye construcción de listas y copia explícita ndarray;
  se importa NumPy antes del cronómetro. Sin NumPy se omite esa métrica.

Se comprueba igualdad de bytes de Rust/FFI/fachada y del documento contenido
en CLI antes de medir; discrepancias abortan. `native_minus_rust_ns` aproxima
el coste del cruce y copia de JSON; `facade_minus_rust_ns` incluye toda la
fachada. Son diferencias de medianas de procesos separados, con ruido y posibles
valores negativos; no deben interpretarse como perfiles internos exactos.
Las fases no se suman: algunas incluyen huellas/copias comunes.

## Reproducción

Provisionar previamente Rust 1.99.0, maturin 1.15.0, pytest y, opcionalmente,
NumPy. Desde la raíz, exportar `CARGO_TARGET_DIR` a la carpeta autorizada y
`MATURIN` al ejecutable instalado. La salida de wheel debe ser nueva:

```bash
export PATH="$RUST_TOOLCHAIN_BIN:$PATH"
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1
bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/python-dist-nuevo"
python3 -m venv --system-site-packages "$CARGO_TARGET_DIR/python-bench-venv"
"$CARGO_TARGET_DIR/python-bench-venv/bin/python" -m pip install \
  --no-index --no-deps "$CARGO_TARGET_DIR/python-dist-nuevo/"*.whl
cargo build --release --offline --locked -p engine-cli
cargo build --release --offline --locked -p mamut-engine-py --example boundary_benchmark
"$CARGO_TARGET_DIR/python-bench-venv/bin/python" scripts/benchmark-python-boundary.py \
  --rust-baseline "$CARGO_TARGET_DIR/release/examples/boundary_benchmark" \
  --cli "$CARGO_TARGET_DIR/release/mamut-engine" \
  --work-dir "$CARGO_TARGET_DIR/python-bench-inputs-nuevo" \
  --repeats 7 --warmups 2 > "$CARGO_TARGET_DIR/python-boundary-report.json"
```

El directorio de inputs debe ser nuevo. `--sizes` permite una selección de
1 a 64 réplicas. No se ejecuta Cargo ni pip dentro del cronómetro. El informe
JSON contiene tamaños, bytes de entrada/salida, hashes de resultados, versiones
Python/NumPy/engine, arquitectura, muestras individuales, mínimo, mediana y
máximo en ns. Conservar junto al informe las versiones Rust/maturin, perfil,
flags, CPU, carga de la máquina y commit/diff probado. Mantener las mismas
herramientas y flags entre baselines y wheel; sin paralelismo interno nuevo.
Para comparar cambios, repetir al menos tres ejecuciones completas con máquina
ociosa y comparar dispersión además de medianas; no imponer umbrales de CI.

El calentamiento predeterminado es 2 y cada métrica tiene 7 muestras. La
destrucción de cada resultado se hace fuera del intervalo. El script no mide
RSS pico: ndarray.nbytes omitiría listas/JSON/copias temporales, por lo que
no se usa como aproximación de memoria total. El tiempo de primera importación
NumPy tampoco está incluido. No se promete speedup ni se relaja el GIL para
mejorar cifras. La evidencia local y sus límites se registran en
[provenance](../provenance/ENG-PY-004.md).

## Pruebas de la wheel con NumPy

El constructor ejecuta las pruebas stdlib en un consumidor aislado sin NumPy.
Para ejecutar también pytest sobre la wheel, usar el venv temporal anterior con
`--system-site-packages` (pytest/NumPy provisionados). Generar referencias
independientes con el ejemplo Rust, sin pasar por PyO3:

```bash
mkdir "$CARGO_TARGET_DIR/python-test-references"
for fixture in crates/engine-api/fixtures/input/valid/*.json \
  crates/engine-api/tests/fixtures/determinism/*.json; do
  cargo run --offline --locked --quiet -p mamut-engine-py --example facade_reference -- \
    "$fixture" > "$CARGO_TARGET_DIR/python-test-references/${fixture##*/}"
done
export MAMUT_RUST_REFERENCES="$CARGO_TARGET_DIR/python-test-references"
export MAMUT_RUST_REFERENCE="$MAMUT_RUST_REFERENCES/cantilever-linear-static.json"
export PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 PYTHONDONTWRITEBYTECODE=1
"$CARGO_TARGET_DIR/python-bench-venv/bin/python" -I -B -m pytest \
  -q -p no:cacheprovider bindings/python/tests
```

`numpy_test.py` se recoge con el patrón estándar `*_test.py` de pytest y usa
`pytest.importorskip("numpy")`; no se carga en unittest, que recoge `test_*.py`
y no requiere pytest. `test_arrays.py` prueba listas y simula la ausencia de
NumPy mediante un importador que lo bloquea en un proceso nuevo, sin desinstalar
paquetes del sistema. La simulación verifica importación perezosa, análisis,
exportación básica y `MissingNumPyError`.
