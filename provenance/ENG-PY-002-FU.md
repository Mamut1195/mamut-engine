# Bitácora clean-room · ENG-PY-002-FU

- Ticket / PR: MAM-83 · ENG-PY-002-FU; seguimiento de J4 r22 sobre #89. Rama `j2/py-wheel-license`, desde `origin/main`.
- Autor(es): Codex gpt-6.1-sol high bajo dirección de J2, 2026-10-14 America/Guayaquil.
- Rol: implementador.
- Contrato implementado: incluir `LICENSE` y `NOTICE` en la wheel de `bindings/python`; sin cambios de API ni contratos de análisis.
- Fuentes permitidas usadas: petición del usuario; texto GNU AGPL versión 3, 19 November 2007, suministrado en `/workspace/fu/AGPL-3.0.txt` (documento completo); `bindings/python/pyproject.toml`, README del binding y README raíz; `provenance/TEMPLATE.md`; herramientas locales Cargo y maturin 1.15.0. Sin red.
- Derivaciones propias: aviso breve alineado a los README; sin algoritmos ni derivaciones numéricas.
- Código de terceros copiado: documento legal GNU AGPL-3.0 copiado literalmente de `/workspace/fu/AGPL-3.0.txt` a `bindings/python/LICENSE`; no se copia código de terceros. La cabecera del documento permite su copia y distribución literal. `bindings/python/NOTICE` es un aviso propio, no el contrato comercial completo.
- Validación: comprobaciones del workspace offline, construcción local de la wheel e inspección de su contenido y metadatos; sin validación numérica nueva.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Alcance

`[project].license-files = ["LICENSE", "NOTICE"]` conserva la expresión
`AGPL-3.0-only` y declara los dos archivos legales para la wheel mediante
PEP 639. El README del binding distingue el texto AGPL incluido de la
titularidad, el texto legal comercial y el aviso legal definitivo pendientes
en ENG-GOV-004. Se conserva la licencia comercial disponible con MAMPRO SAS
y la aprobación explícita de Jonathan como requisito para publicar en PyPI.

Sin cambios Rust: regla de `panic!`/`unwrap()`/`expect()` en biblioteca N/A.
Sin cambios en `docs/CLEAN_ROOM.md`, `.github/**` ni `clean-room/**`.
Sin publicación ni push; commit con la identidad configurada de J2.

## Verificación

Comandos desde `/workspace/me45`, con el entorno solicitado:

```bash
export PATH=/home/box/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
export RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu
export CARGO_TARGET_DIR=/workspace/.cargo-target
export CARGO_NET_OFFLINE=true
```

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK, salida 0 |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | OK, salida 0 |
| `cargo test --offline --workspace` | OK, salida 0; 421 pruebas/doctests, 0 fallidos, 0 ignorados |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace` | OK, salida 0 |
| `cargo check --offline --workspace --target wasm32-unknown-unknown` | OK, salida 0 |
| `bash clean-room/check.sh origin/main` | OK, salida 0, con los archivos nuevos añadidos al índice |
| `cd bindings/python && /workspace/venv-maturin/bin/maturin build --offline --out /tmp/eng-py-wheel-license` | OK, salida 0 |
| `unzip -l /tmp/eng-py-wheel-license/*.whl` | OK, salida 0; LICENSE y NOTICE presentes |
| Comparación de los archivos legales dentro del ZIP con los del binding | OK, idénticos byte a byte; LICENSE también idéntico a la fuente suministrada |
| Inspección de METADATA | OK; versión 2.4, `License-Expression: AGPL-3.0-only`, `License-File: LICENSE` y `License-File: NOTICE` |
| `git diff --cached --check` | OK, salida 0 |

Wheel construida localmente con maturin 1.15.0:
`/tmp/eng-py-wheel-license/mamut_engine-0.1.0-cp310-abi3-manylinux_2_34_x86_64.whl`.

El listado contiene 10 archivos, incluidos:

```text
34523  mamut_engine-0.1.0.dist-info/licenses/LICENSE
  479  mamut_engine-0.1.0.dist-info/licenses/NOTICE
```

Listado completo guardado en `/workspace/fu/wheel-license-list.txt` mediante
`unzip -l /tmp/eng-py-wheel-license/*.whl | tee /workspace/fu/wheel-license-list.txt`.
Registros de los comandos: `/tmp/eng-py-wheel-license-{fmt,clippy,test,doc,wasm,clean-room,wheel}.log`.
Resultados: `/tmp/eng-py-wheel-license-results.json`.
Sin verificación adicional de plataformas ni publicación en PyPI.
Revisión clean-room PENDIENTE (J4).
