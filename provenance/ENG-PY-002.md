# Bitácora clean-room · ENG-PY-002

- Ticket / PR: ENG-PY-002 · rama `j2/eng-py-002`, desde `origin/main`.
- Autor(es): Codex gpt-6.1-sol high bajo dirección de J2, 2026-10-14 America/Guayaquil.
- Rol: implementador.
- Contrato implementado: scaffold Python local; `version() -> str` con la versión de `engine-core`. Sin contratos de análisis nuevos.
- Fuentes permitidas usadas: petición del usuario; `/workspace/fu/tickets/ENG-PY-002.txt` y `engine-cli-py-plan.md` como datos; manifiestos, núcleo Rust, README raíz y `provenance/TEMPLATE.md` del repositorio; herramientas locales Cargo, PyO3 y maturin. Sin red.
- Derivaciones propias: adaptación de la constante de versión del núcleo al módulo Python; sin algoritmos numéricos.
- Código de terceros copiado: ninguno.
- Validación: compilación, análisis estático y smoke test de instalación/importación local; no requiere oráculo numérico.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Alcance y base

Base local actualizada por fast-forward a `origin/main`
`d1249f1d533d923957ead3626fe2f43cf5f6fca1`, sin red. El scaffold solo depende
de `engine-core` y, en destinos distintos de `wasm32`, PyO3 0.25.1
(`abi3-py310`; `extension-module` solo desde maturin). No utiliza
`engine-api`. La versión viene del manifiesto de `engine-core`, no de un
literal del binding. Un único `Cargo.lock` del workspace.

El binding pertenece al workspace y queda fuera de `default-members`, que
enumera los 14 crates originales. `--workspace` omite esa selección por defecto;
desde la corrección del 2026-10-14, WASM incluye el crate Python vacío y
documentado, sin compilar PyO3. No se modifica
`docs/CLEAN_ROOM.md`, `.github/**` ni `clean-room/**` respecto de la base.

Metadatos AGPL-3.0-only y nota comercial MAMPRO SAS; texto legal pendiente en
ENG-GOV-004. Sin publicación en PyPI; se requiere aprobación de Jonathan.
La revisión clean-room sigue pendiente de J4. Commit local, sin push.

## Verificación inicial (antes de la corrección WASM)

Todas las verificaciones Rust usaron `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH` y
`RUSTUP_TOOLCHAIN=stable`. El toolchain local informa rustc 1.99.0
(`b940084d7 2026-09-28`), igual a la versión fijada por el repositorio;
`stable` selecciona la instalación existente sin descargar otro nombre.
El destino `wasm32-unknown-unknown` ya estaba instalado. No hubo red.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK, salida 0 |
| `cargo clippy -p mamut-engine-py --offline -- -D warnings` | OK, salida 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, salida 0 |
| `cargo test --workspace` | OK, 416 pruebas/doctests, 0 fallidos, 0 ignorados |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | OK, salida 0 |
| `cargo check --workspace --target wasm32-unknown-unknown` | Falla, salida 101: `pyo3-ffi` 0.25.1 importa tipos de `libc` ausentes en ese destino (`wchar_t`, `size_t`, entre otros) |
| `cargo check --offline --target wasm32-unknown-unknown` | OK, salida 0: usa los 14 miembros por defecto |
| `cargo check --offline --workspace --exclude mamut-engine-py --target wasm32-unknown-unknown` | OK, salida 0 |

La falla del comando WASM literal queda registrada: `--workspace` incluye el
binding pese a `default-members`. No se agregan stubs WASM ni se cambia el
núcleo para ocultarla. Los crates numéricos no dependen del binding; la
compilación WASM por defecto conserva su aislamiento.

`cargo metadata --offline --format-version 1` y comprobaciones del DAG:
15 miembros, 14 por defecto, binding excluido de estos últimos; dependencias
directas exactamente `engine-core` y `pyo3`, ningún crate depende del binding,
ningún paquete `engine-api`; versiones núcleo/binding iguales a `0.1.0`.
Sin `panic!`, `unwrap()` ni `expect()` en el código de biblioteca añadido.
El cdylib desactiva el arnés Rust y sus doctests; la función se prueba mediante
la extensión realmente instalada en Python.

Construcción local con maturin 1.15.0:

```bash
cd bindings/python
/workspace/venv-maturin/bin/maturin build --offline \
  --out /tmp/eng-py-002-fxn2hdfj/wheels \
  --interpreter /workspace/venv-maturin/bin/python
/workspace/venv-maturin/bin/python -m venv /tmp/eng-py-002-fxn2hdfj/venv
/tmp/eng-py-002-fxn2hdfj/venv/bin/python -m pip install --no-index \
  /tmp/eng-py-002-fxn2hdfj/wheels/mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl
/tmp/eng-py-002-fxn2hdfj/venv/bin/python -m unittest discover \
  -s /workspace/mamut-engine/bindings/python/tests -v
```

Construcción, instalación y prueba: OK, salida 0. Se usó también
`PIP_NO_INDEX=1`. Prueba ejecutada desde `/tmp` en CPython 3.13.5/Linux, con
importación desde el `site-packages` del venv temporal. Una prueba verifica
tipo/valor `0.1.0`, igualdad entre versión nativa, `__version__` y metadatos de
distribución, y que el módulo nativo solo expone `version`.
La inspección de la wheel confirma `__init__.py`, `_native.pyi`, `py.typed`,
extensión `.so`, `License-Expression: AGPL-3.0-only`, `Requires-Python: >=3.10`
y nota MAMPRO SAS. Sin dependencia Python de ejecución ni pytest.

No se verificaron Windows/macOS, otros intérpretes ni manylinux2014. La etiqueta
abi3 expresa el objetivo de ABI, no una matriz de pruebas completada.
El texto legal completo sigue pendiente; esta verificación no es una release.

Registros: `/tmp/eng-py-002-{fmt,clippy,test,doc,wasm-workspace,wasm-default,wasm-exclude,wheel}.log`,
`/tmp/eng-py-002-results.json` y `/tmp/eng-py-002-metadata.json`.
Wheel y venv temporal: `/tmp/eng-py-002-fxn2hdfj/`.

`bash clean-room/check.sh origin/main`: OK, salida 0, con todos los archivos
nuevos añadidos al índice. `git diff --cached --check`: OK, salida 0.
El diff de este ticket no modifica las rutas protegidas.

## Corrección WASM y CI del workspace · 2026-10-14

Autor: Codex (GPT-6), como implementador bajo petición del usuario.
Fuentes: solicitud actual, manifiestos y código del binding, README,
esta bitácora, `docs/CLEAN_ROOM.md` y herramientas locales. Sin red ni
código de terceros copiado. Se mantiene la declaración clean-room inicial.

Se corrige el fallo en `pyo3-ffi` al ejecutar el comando WASM con
`--workspace`, utilizado por el CI de Rust del PR #83. El binding permanece
en `[workspace].members` y conserva los mismos `default-members`.
PyO3 pasa a `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` y
el módulo que contiene sus imports y macros lleva la misma condición
`#[cfg(not(target_arch = "wasm32"))]`. En WASM no hay API Python y el crate
queda vacío, con documentación que explica este comportamiento.

`extension-module` se elimina de las features de la dependencia Cargo y
se activa mediante `features = ["pyo3/extension-module"]` en
`[tool.maturin]`. Las comprobaciones Rust no requieren esa configuración
de enlace; la wheel mantiene ABI estable `abi3-py310` y la misma API.
Se conserva la prueba Python sobre la extensión instalada y el cdylib sin
arnés Rust ni doctests. El README ya documenta WASM sin exclusiones.

La tabla siguiente sustituye el resultado WASM fallido de la verificación
inicial. Comandos desde la raíz, con `PATH=/home/box/.cargo/bin:$PATH`,
`CARGO_TARGET_DIR=/workspace/.cargo-target` y `CARGO_NET_OFFLINE=true`.
El entorno conserva `RUSTUP_TOOLCHAIN=stable`; esa instalación informa
rustc `1.99.0 (b940084d7 2026-09-28)`, la versión fijada por el repositorio.

| Comprobación | Resultado |
| --- | --- |
| `cargo check --workspace --target wasm32-unknown-unknown` | OK, salida 0, sin excluir el binding |
| `cargo clippy --workspace --all-targets -- -D warnings` | OK, salida 0 |
| `cargo test --workspace` | OK, 416 pruebas/doctests, 0 fallidos, 0 ignorados |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | OK, salida 0 |
| `cargo fmt --check` | OK, salida 0 |
| `bash clean-room/check.sh origin/main` | OK, salida 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p mamut-engine-py --target wasm32-unknown-unknown` | OK, salida 0 |
| `cargo tree -p mamut-engine-py --target wasm32-unknown-unknown` | Solo `engine-core`; sin PyO3 ni `pyo3-ffi` |

Wheel reconstruida con maturin 1.15.0 y `--offline`, activando las features
de `pyproject.toml`, e instalada con `pip --no-index` y `PIP_NO_INDEX=1`
en un venv nuevo. CPython 3.13.5/Linux: `unittest discover` pasa
1 prueba; `version()` devuelve `0.1.0` y el módulo se importa desde el
`site-packages` del nuevo venv, ejecutando desde `/tmp`.

Artefacto:
`/tmp/eng-py-002-wasm-fix-1or15acg/wheels/mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.
Venv: `/tmp/eng-py-002-wasm-fix-1or15acg/venv`.
Registros: `/tmp/eng-py-002-wasm-fix-*.log` y
`/tmp/eng-py-002-wasm-fix-results.json`.
Sin ampliación de la matriz de plataformas ni publicación. Commit local,
sin push; revisión clean-room externa pendiente de J4.
