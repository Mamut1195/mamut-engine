# MAMUT Engine · Python local

Scaffold PyO3/maturin: solo expone `version() -> str` y `__version__`, desde
`engine_core::VERSION`. Consume `engine-core`; la API de análisis sigue
PENDIENTE en PY-003. Los contratos `engine-input/1` y `results/1` se registran
como referencia del engine, sin afirmar que este binding los implemente.

Los metadatos declaran `AGPL-3.0-only`. La wheel incluye `LICENSE` y `NOTICE`.
Titularidad, aviso definitivo, licencia comercial y CLA: **PENDIENTE de revisión**
(ENG-GOV-004 / ENG-GOV-007). El inventario de licencias generado no resuelve
estas revisiones. **Verificación local únicamente: nada se publica ni se sube.**

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
nativa. No existe otra operación del binding que ejecutar actualmente.

Junto a la wheel produce:

- `SHA256SUMS`: wheel y todos los artefactos adjuntos.
- `COMPATIBILITY.json`: versiones binding/engine, contratos, tags reales de
  Python/ABI/plataforma, herramientas e intérprete probado.
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
"$CONSUMER_PYTHON" -I -c 'import sys, mamut_engine as e; assert e.version() == sys.argv[1]' "$PREVIOUS_ENGINE_VERSION"
```

Comprueba también que la versión de distribución coincide con el manifiesto
anterior. Usa un proceso Python nuevo: el módulo nativo ya cargado no se reemplaza
en memoria. Detén/reinicia consumidores antes de reabrirlos. Si falla el smoke,
reinstala la wheel conservada de la versión que funcionaba y vuelve a comprobar.
La evidencia y las limitaciones del ensayo local se registran en
`provenance/ENG-PY-007.md`; no equivale a rollback de una versión publicada.
