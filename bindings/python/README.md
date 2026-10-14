# MAMUT Engine · Python local

Scaffold PyO3/maturin para Python >=3.10. Solo expone `version() -> str` y
`__version__`: la versión procede de `engine_core::VERSION`, derivada del
manifiesto del núcleo. No ofrece análisis todavía. El único crate del motor
consumido es `engine-core`; no depende de `engine-api` ni duplica el solver.

Licencia **AGPL-3.0-only**, con licencia comercial disponible con **MAMPRO SAS**.
El texto de licencia y el aviso legal siguen pendientes en ENG-GOV-004, como
indica el README raíz. Esta wheel es para verificación local.
**Nada se publica en PyPI sin aprobación explícita de Jonathan.**

Desde la raíz del repositorio, con Rust y maturin ya instalados y las
dependencias disponibles en el registro local:

```bash
export CARGO_TARGET_DIR=/workspace/.cargo-target
export CARGO_NET_OFFLINE=true
cd bindings/python
maturin build --offline --out /tmp/eng-py-002-wheels
python3 -m venv /tmp/eng-py-002-venv
/tmp/eng-py-002-venv/bin/python -m pip install --no-index /tmp/eng-py-002-wheels/*.whl
/tmp/eng-py-002-venv/bin/python -m unittest discover -s tests -v
/tmp/eng-py-002-venv/bin/python -c 'import mamut_engine; print(mamut_engine.version())'
```

El consumidor de la wheel no necesita Rust ni conexión. La prueba usa `unittest`
y no requiere pytest. El módulo nativo utiliza ABI estable `abi3-py310`.
La feature `pyo3/extension-module` se activa solo desde `[tool.maturin]` en
`pyproject.toml`; las comprobaciones Cargo del workspace no la activan.

Los 14 crates anteriores son los `default-members`: desde la raíz,
`cargo test --offline`, `cargo clippy --offline --all-targets -- -D warnings` y
`cargo check --offline --target wasm32-unknown-unknown` excluyen Python.
`--workspace` incluye el binding. PyO3 es una dependencia solo para destinos
distintos de `wasm32`, y el módulo Python usa la misma condición de compilación.
En WebAssembly el crate queda vacío y documentado, conservando su pertenencia
al workspace. Las comprobaciones completas de Rust y CI se ejecutan sin
exclusiones:

```bash
cargo check --workspace --target wasm32-unknown-unknown
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
cargo fmt --check
bash clean-room/check.sh origin/main
```

Verificación local: maturin 1.15.0 construyó e instaló sin red una wheel
`cp310-abi3-manylinux_2_34_x86_64`, y la prueba pasó en CPython 3.13.5/Linux.
En este entorno se usaron `/workspace/venv-maturin/bin/maturin` y
`PATH=/home/box/.cargo/bin:$PATH RUSTUP_TOOLCHAIN=stable` (Rust 1.99.0 ya
instalado, la versión fijada por el repositorio). No se verificaron otros
sistemas operativos ni intérpretes; no se afirma compatibilidad manylinux2014.
La corrección del 2026-10-14 permite comprobar WASM con `--workspace` sin
excluir el binding ni compilar `pyo3-ffi`; la wheel conserva su API Python.
