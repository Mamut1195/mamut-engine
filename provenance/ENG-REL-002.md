# Bitácora clean-room · ENG-REL-002

- Ticket: MAM-869 (ENG-REL-002), publicación de esquemas versionados.
- Autor: Codex, GPT-6, 2026-10-21 (America/Guayaquil).
- Rol: implementador.
- Fuentes: requisitos del ticket; API, esquemas y pruebas propios de engine-api;
  changelog.d/README.md; lista local de identificadores prohibidos y verificador
  clean-room; workflow Rust existente para documentar el runner. Sin red.
- Contrato: árbol schemas/ idéntico a schema(), índice canónico con tamaños y
  SHA-256; registro extensible API/CLI; TAR USTAR determinista sin publicación.
- Derivaciones: orden lexicográfico, metadatos constantes del archivo y SHA-256
  de bytes originales. Sin trabajo sobre modelos mecánicos.
- Código de terceros copiado: ninguno. Python usa biblioteca estándar y Rust
  utiliza dependencias existentes. Sin cambios de manifiestos ni lockfile.
- Decisiones: conservar los URN y capabilities; relacionar URN con URL en el
  índice. Base propuesta https://schemas.mamut.systems/, pendiente de Jonathan.
  Migración futura de identidad reservada a Jonathan.
- Declaración: se usaron únicamente fuentes propias permitidas; no se consultó
  código externo de cálculo estructural, repositorios restringidos ni registros
  de otras sesiones. No se modificaron elementos mecánicos.
- Validación: pruebas automáticas de igualdad de bytes, cobertura del registro,
  hashes y tamaños, inventario exacto, registro CLI, enlaces, corrupción,
  bloqueo de regeneración en CI y repetibilidad/metadatos del TAR.
- Verificaciones: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets --offline -- -D warnings` y
  `cargo test --workspace --offline`: OK, 751 pruebas aprobadas, ninguna fallida
  ni ignorada. Las dos pruebas nuevas de integración Cargo incluyen además seis
  pruebas Python de herramientas.
- `bash clean-room/check.sh origin/main`: OK incluyendo archivos nuevos mediante
  índice y almacén de objetos temporales en /tmp, sin modificar el índice real.
  `git diff --check`: OK. Lista de identificadores comprobada también en todos
  los archivos nuevos, incluidos docs y provenance: OK.
- Dos ejecuciones locales del empaquetador: mismo SHA-256 del TAR,
  `29df0f6a55ebedb5a96a23ee6a2db8f1cf81f343b7947009ae55748f70d98e29`.
- Revisor clean-room: pendiente de revisión humana.
- Publicación: ninguna; sin push, tags, releases ni commit.
