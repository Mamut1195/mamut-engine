# Bitácora clean-room · ENG-SFC-004

- Ticket / PR: ENG-SFC-004 · MAM-92; PR PENDING.
- Autor: Codex gpt-6.1-sol high · 2026-10-29 (America/Guayaquil).
- Rol: implementador del contrato de integración CLI; sin lógica numérica nueva.
- Contrato implementado: ficha cli-program/1.0.0 para `mamut-engine`;
  stdin engine-input/1, cli-run/1 con results/1, validate-report/1 y cli-describe/1.
  Decisión de Jonathan del 2026-10-29: V1 solo CLI; MCP fuera de V1.
- Fuentes permitidas utilizadas: repositorio propio `harness-determinista`,
  commit `85f912ea3225fe72c9f0f475a43dce8d6f5e9ec4`, archivo
  `contracts/schemas/cli-program.schema.json`; contratos, implementación,
  README y tests propios de `crates/engine-cli` y `crates/engine-api`, incluidos
  `tests/support/schema.rs` y `tests/fixtures/determinism/cantilever.json` de
  engine-api; `vendor/mamut-workflows/README.md`, solo para comprobar la exclusión
  de `mamut-tool-conformance`; `provenance/ENG-SFC-002.md` y `provenance/TEMPLATE.md`.
- Derivaciones propias: ninguna ecuación ni referencia numérica nueva.
  Decisiones de transporte y presupuestos en
  [la nota de integración](../docs/integration/ENG-SFC-004-harness-cli.md).
- Código de terceros copiado: ninguno. Se copia únicamente el schema del
  repositorio propio de Harness como fixture; no se modifica vendor.
- Validación: revisión estática, formato Rust directo sin Cargo, identidad
  byte a byte del schema y comprobaciones locales de JSON/patrones;
  compilación y tests Rust **PENDIENTES** del agente externo.
- Comparación con oráculo: no; el schema hermano es una especificación de
  transporte, sin ejecución de otro motor ni comparación numérica.
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni
  derivados para este ticket. No consulté rutas ni provenance prohibidos.
  Trabajé sin red y sin ejecutar Cargo."
- Revisor clean-room: **PENDING (J4)**.

## Procedencia del fixture de Harness

La cabecera de procedencia se registra aquí, sin agregar comentarios ni
propiedades al JSON. Origen: `harness-determinista`, commit completo
`85f912ea3225fe72c9f0f475a43dce8d6f5e9ec4`,
`contracts/schemas/cli-program.schema.json`, schema_version `1.0.0`,
`$id: https://mamut.local/schemas/cli-program.schema.json`.
Destino: `crates/engine-cli/tests/fixtures/harness/cli-program-1.0.0.schema.json`.
Obtenido del objeto Git local mediante `git show`, sin red ni cambios al origen.
SHA-256 de los bytes originales y copiados:
`f3c283313347ac4791fd4a817ee604e39082a07cccd7cc0169ce6ae87a84306e`.

La copia permanece íntegra. El validador compartido soporta `const`; sus patrones
limitados no incluyen los de Harness y tampoco admite longitudes de cadenas.
El test aplica esos patrones y longitudes a mano y delega las demás reglas en
el helper existente. No se añade una dependencia de validación ni un nuevo
validador general. La inspección de caracteres y el matching local confirman
que los escapes de la especificación aceptan la ficha prevista.

## Alcance y evidencia

La ficha usa versión `0.1.0`, contrastada en el test con la versión del crate y
con `describe --json`, y solo argumentos existentes. Los límites externos son
300/30/5 segundos y 64/64/1 MiB para run/validate/describe. La entrada mantiene
el límite inclusivo de 8 MiB. El presupuesto de salida no garantiza que todos
los modelos aceptados quepan en él. No se cambia código de producción.

Se reutilizan el validador compartido, los schemas publicados y el fixture
cantilever. El escritor de stdin existente se traslada a
`tests/support/stdin.rs` para compartir su cierre del canal y tratamiento
exclusivo de BrokenPipe con `tests/cli.rs`. No se introduce ejecución de motor
en el proceso de prueba: los cálculos se ejecutan mediante subprocesos del
binario; la llamada directa a engine-api calcula el hash esperado. Los helpers
existentes acceden a schemas y metadatos de errores.

Las pruebas añadidas comprueban argumentos exactos, entorno mínimo, códigos
0/1/2 de los escenarios elegidos, schemas, límites y determinismo íntegro de
stdout. Comprueban también stderr en escenarios de error, claves reordenadas
y SHA-256 RFC 8785 de todos los results/1 de esos escenarios. La tabla completa
0/1/2/3/4/5/6/70 ya cuenta con cobertura en `tests/cli.rs`; no se repite su suite.

La nota de integración relaciona describe, códigos y cli-run/1 con pruebas
existentes. SDK-WF-009 / `mamut-tool-conformance` no está vendorizado y no se
reimplementa: ejecución del kit real **PENDIENTE / DESCONOCIDO**.
SDK-ADO-014 no está en Linear según el contexto recibido, sin consulta en línea.
ENG-SFC-003 (MCP) está fuera de V1. Ficha **experimental** hasta probarla con
Harness real en Windows. No se afirma conformidad ejecutada por ese consumidor.

No se modifican Cargo.toml/Cargo.lock, `.github`, `clean-room/**`,
`docs/CLEAN_ROOM.md`, CODEOWNERS ni `vendor/`. Sin commits.
