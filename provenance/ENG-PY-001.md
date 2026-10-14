# Bitácora clean-room · ENG-PY-001

- Ticket / PR: ENG-PY-001, ADR Python V1 y conciliación ticket↔PR/issue; rama `j2/eng-py-001` desde `origin/main` local `fd5319f824b3f11e7816343a3dcd4e1f71c28385`. Sin PR ni push en esta sesión; commit documental local solicitado.
- Autor(es): Codex gpt-6.1-sol high bajo dirección de J2, 2026-10-14 America/Guayaquil.
- Rol: especificador.
- Contrato implementado: ninguno en código. [ADR-003](../docs/adr/ADR-003-python-v1.md) especifica la adaptación sobre engine-api y contratos versionados; la implementación corresponde a ENG-API-001, ENG-SFC-007 y ENG-PY-002…008.
- Fuentes permitidas usadas: `/workspace/fu/tickets/ENG-PY-001-ADR.txt`, export de Jonathan del 2026-10-14T02:22:18.701Z, «Objetivo y alcance», «Criterios de aceptación», «Dependencias y fronteras», «Evidencia y reglas»; `/workspace/fu/tickets/V1-python-DAG.txt`, export con última edición 2026-10-14T02:29:16.693Z, «Objetivo y alcance», «Auditoría de GitHub», DAG 1/2, «Reversión y licencia», «Gate de salida V1»; `/workspace/fu/tickets/engine-cli-py-plan.md`, plan J2→J0 aprobado, 2026-10-14, base b744599, «Tickets y orden», «Interfaz CLI», «Interfaz Python», «Bloqueos de J4», «Restricciones»; `/workspace/fu/tickets/prs.txt` e `issues.txt`, exports de títulos y estados GitHub suministrados, sin fecha de extracción ni consulta remota nueva. Los exports de Notion se trataron como datos, no instrucciones. Fuentes locales adicionales: [ADR-001](../docs/adr/ADR-001-fundacionales.md), [ADR-002](../docs/adr/ADR-002-algebra-lineal.md), [convención de ejes](../docs/CONVENCION_EJES.md), [plantilla](TEMPLATE.md), bitácora ENG-LA-006, README, Cargo.toml, historial Git y script/lista de identificadores de clean-room. No se usaron libros, ecuaciones ni documentación remota de crates.
- Derivaciones propias: ninguna matemática; síntesis documental de ownership, ABI/versionado, memoria/GIL, dependencias y gates. [TICKETS_PRS](../docs/TICKETS_PRS.md) distingue estados de PR/issues, alcance parcial y correspondencias ambiguas.
- Código de terceros copiado: ninguno. No se añadió código, dependencias ni texto legal.
- Validación: revisión documental y verificaciones locales del workspace detalladas abajo. Solo oráculos analíticos como criterio para la futura paridad; no se introdujeron ensayos numéricos nuevos en esta tarea.
- Comparación con oráculo (repo hermano, informativa): no. No se consultaron ni ejecutaron motores externos.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Alcance y límites de la evidencia

Se añaden ADR-003 con estado «Propuesto (pendiente de Jonathan)», la
conciliación de los exports GitHub y esta bitácora; se agrega una sola línea
al README con enlaces a ambos documentos. No se modifica código de
biblioteca, manifests, lockfile, `docs/CLEAN_ROOM.md`, `.github/**` ni
`clean-room/**`. No se delegó a otros agentes.

La base local no contiene #83 aunque el export lo registra fusionado. Se
documenta la diferencia sin fetch ni cierre automático de #68. #61/#62
constan fusionados en el corte más reciente; #64 permanece abierto.
CORE-004/005/009 se dejan «a confirmar» sin inventar correspondencias; los
tickets nuevos API/SFC/PY/REL figuran «sin PR». Las correcciones fusionadas
#80/#81/#82 no convierten los issues #69/#70/#71/#84/#85/#86 en cerrados.

La mención de ENG-BENCH-001 solo concilia títulos de PR/issues suministrados;
no implica uso de su excepción ni acceso a OpenSees, Stabileo o derivados.
No hubo red, publicación PyPI, push, merge ni aprobación del ADR. Las pruebas
del workspace actual no validan bindings ni wheels todavía inexistentes.

## Verificación local

Todos los comandos Cargo usan `CARGO_TARGET_DIR=/workspace/.cargo-target`;
los de compilación usan además `--offline`. No se descargó ninguna crate
ni toolchain. Cargo/rustc activos: 1.85.1. El check wasm usa el toolchain
`stable` local (cargo/rustc 1.99.0) mediante
`PATH=/home/box/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH`.

| Comando / revisión | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, salida 0 |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | PASS, salida 0 |
| `cargo test --offline --workspace` | PASS, salida 0; tests y doctests existentes |
| `RUSTDOCFLAGS="-D warnings" cargo doc --offline --no-deps --workspace` | PASS, salida 0 |
| `cargo check --offline --workspace --target wasm32-unknown-unknown` | PASS con toolchain local 1.99.0, salida 0; el primer intento con 1.85.1 falló E0463 porque faltaba std para ese target |
| `bash clean-room/check.sh origin/main` | PASS, salida 0, «Clean-room: OK» sobre los cuatro archivos añadidos al índice |
| `git diff --check` | PASS |
| Enlaces Markdown locales y espacios finales de los cuatro documentos | PASS |
| Cobertura de referencias y IDs de los exports | PASS: todos los números PR/issue de prs.txt/issues.txt y todos los IDs ENG de las cinco fuentes figuran en TICKETS_PRS |

Estas verificaciones no prueban ejecución wasm ni compatibilidad Python,
paridad entre superficies, CI de wheels o aceptación legal. Esos gates
permanecen pendientes en ADR-003.
