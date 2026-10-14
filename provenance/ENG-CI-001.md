# Bitácora clean-room · ENG-CI-001

- Ticket / PR: issue #68 · rama `j2/fu-68-rust-ci`, desde `main`.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-14 (America/Guayaquil).
- Rol: implementador.
- Contrato implementado: CI Rust del workspace; toolchain reproducible, formato,
  Clippy, pruebas, documentación y compilación wasm. No cambia `element/1`.
- Fuentes permitidas usadas: petición del usuario; `provenance/TEMPLATE.md`,
  `.github/workflows/clean-room.yml`, `Cargo.toml`, `CONTRIBUTING.md`,
  `README.md`, `clean-room/check.sh` y bitácora propia `provenance/ENG-ANL-002.md`;
  consultas locales de rustc/rustup. Sin red ni fuentes externas.
- Derivaciones propias: no aplica; configuración de CI, sin algoritmos nuevos.
- Código de terceros copiado: ninguno. El SHA de checkout se toma del workflow
  propio existente, sin consultar ni copiar su implementación.
- Validación: comprobaciones locales de configuración y del workspace, detalladas
  abajo; sin nuevos oráculos ni pruebas numéricas.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Alcance

Se fija Rust 1.99.0, obtenido con `rustc --version` del rustup local
(`/home/box/.cargo/bin/rustc`); el ejecutable del sistema informa 1.85.1.
El workflow usa `rustup toolchain install` desde la raíz para instalar el
toolchain y los componentes/destino declarados en `rust-toolchain.toml`, sin
acciones de terceros ni caché, seguido de `rustup show`. Checkout usa el SHA
completo existente de v4.2.2, sin credenciales persistidas. El job tiene permisos
de lectura, concurrencia por workflow/ref,
20 minutos de límite y su propio directorio de compilación temporal.

La creación de `.github/workflows/rust.yml` es imprescindible para la petición
explícita del ticket y constituye la única excepción a la prohibición general
de tocar `.github/**`. No se modifican `.github/workflows/clean-room.yml`,
`docs/CLEAN_ROOM.md` ni `clean-room/**`. La revisión del dueño que exige el
workflow existente para archivos protegidos queda pendiente en el futuro PR.

## Verificación

Todas las comprobaciones Rust usaron `CARGO_TARGET_DIR=/workspace/.cargo-target`,
`CARGO_NET_OFFLINE=true`, `PATH=/home/box/.cargo/bin:$PATH` y
`RUSTUP_TOOLCHAIN=stable`. Este último selecciona el toolchain ya instalado con
rustc 1.99.0 (b940084d7 2026-09-28), cuya versión se comprobó antes de validar;
evita que rustup intente descargar otro nombre para la misma versión. Rustfmt,
Clippy y el destino wasm ya estaban instalados. No hubo descargas.

- `python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1]))' .github/workflows/rust.yml`:
  OK, salida 0.
- Comprobación local de TOML y estructura del workflow: OK; verifica versión,
  componentes/destino, eventos, permisos, concurrencia, timeout, checkout,
  directorio temporal y comandos exactos. Al retirar temporalmente cada archivo
  nuevo de configuración (equivalente a revertir su adición), falla por archivo
  ausente; tras restaurarlos vuelve a pasar. No se añaden tests al workspace,
  pues no hay cambios de comportamiento en la biblioteca.
- `cargo fmt --check`: OK, salida 0.
- `cargo clippy --all-targets -- -D warnings`: OK, salida 0.
- `cargo test --workspace`: OK, salida 0; 390 pruebas y doctests aprobados,
  0 fallidos y 0 ignorados.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK, salida 0.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, salida 0.
- `bash clean-room/check.sh origin/main`: OK, salida 0, incluyendo los archivos
  nuevos añadidos al índice.
- `git diff --cached --check`: OK, salida 0.

Registros Rust: `/tmp/eng-ci-001-{fmt,clippy,test,doc,wasm}.log`.
La validación wasm comprueba compilación; no ejecuta tests en ese destino.
El workflow no se ha ejecutado en GitHub ni se ha probado la descarga del
toolchain, porque esta sesión no usa red. Commit local solicitado, sin push.

## Corrección · 2026-10-14

Se fija `runs-on: ubuntu-24.04` y se configura `CARGO_TARGET_DIR` mediante
`$RUNNER_TEMP/target` en `$GITHUB_ENV` tras checkout, porque el contexto `runner`
no está disponible en `env` del job. Se instala el toolchain con
`rustup toolchain install` sin argumentos, que lee `rust-toolchain.toml`, seguido
de `rustup show`, que ya no instala en rustup moderno. YAML validado con
`python3` y `yaml.safe_load`: OK.
