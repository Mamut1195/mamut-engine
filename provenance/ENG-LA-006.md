# Bitácora clean-room · ENG-LA-006

- Ticket / PR: ENG-LA-006, «Selección de librerías de álgebra lineal (ADR-002)» · rama `j2/eng-la-006-adr-002`; sin commit, push ni PR en esta sesión.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-10 (America/Guayaquil), bajo instrucciones del usuario. No se delegó a otros agentes.
- Rol: implementador documental; sin actividad de operador de oráculo ni auditor de similitud.
- Contrato implementado: ninguno en código. [ADR-002](../docs/adr/ADR-002-algebra-lineal.md) propone la selección para `LinAlgBackend` (ENG-LA-007), `linsolve/1` y `eigen/1` (arquitectura §§4.5–4.6); integración y pruebas quedan en ENG-LA-007/008/009.
- Fuentes permitidas usadas: metadatos de crates.io verificados el 2026-10-10 y proporcionados por el solicitante: [faer 0.24.4](https://crates.io/crates/faer/0.24.4), [nalgebra 0.35.0](https://crates.io/crates/nalgebra/0.35.0), [nalgebra-sparse 0.12.0](https://crates.io/crates/nalgebra-sparse/0.12.0), [sprs 0.11.5](https://crates.io/crates/sprs/0.11.5) y [scirs2-sparse 0.6.5](https://crates.io/crates/scirs2-sparse/0.6.5); arquitectura interna `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`, §§4.5–4.6 (contratos), 9 (determinismo) y 10 (prueba de compilación del 2026-10-08, funciones y licencias de candidatos, exclusión de sprs-ldl y arpack-ng). No se atribuyen capítulos, ecuaciones o mediciones a fuentes no consultadas. La reconsulta de páginas y API de crates.io no fue posible en esta sesión; no se presenta como verificación remota independiente.
- Derivaciones propias: ninguna derivación matemática ni algoritmo nuevo. La [decisión y el plan de verificación del ADR](../docs/adr/ADR-002-algebra-lineal.md#decisión) contienen la comparación documental, la recomendación de mantener MSRV 1.85 con nalgebra opcional que requiere al menos 1.89, y los ensayos propuestos para resolver pendientes.
- Código de terceros copiado: ninguno. No se añadió código Rust ni dependencias; no se copiaron listados de código ni texto legal.
- Validación: revisión documental contra tickets, contratos y arquitectura; comprobación del diff, enlaces locales y alcance de archivos. No hay validación numérica, benchmark, ejecución nativa/wasm ni `cargo-deny` realizado para candidatos en esta sesión. La compilación wasm citada es evidencia histórica de arquitectura §10, no un ensayo nuevo. [Plan V1–V7](../docs/adr/ADR-002-algebra-lineal.md#plan-de-verificación) **PENDIENTE** para licencias, MSRV, ejecución, APIs, inercia, autovalores dispersos, rendimiento, tamaño, determinismo y mantenimiento.
- Comparación con oráculo (repo hermano, informativa): no. No se ejecutaron OpenSees, CalculiX ni otros motores como oráculos.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE de revisión humana y aprobación de Jonathan; esta sesión no aprueba el ADR ni el merge.

## Documentos internos consultados

- `/workspace/motor-rust/04_TICKETS.md`: ENG-LA-006, ENG-LA-007, ENG-LA-008 y ENG-LA-009.
- `/workspace/motor-rust/00_PROYECTO.md`: contexto, selección mediante librerías y pendientes técnicos.
- `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`: §§4.5–4.6, 9 y 10.
- [Protocolo clean-room](../docs/CLEAN_ROOM.md), [ADR-001](../docs/adr/ADR-001-fundacionales.md), [plantilla de procedencia](TEMPLATE.md) y [bitácora ENG-CORE-007](ENG-CORE-007.md).
- [README](../README.md) y [Cargo.toml](../Cargo.toml): lista existente de ADR y MSRV 1.85 del workspace.

No se encontraron instrucciones `AGENTS.md` aplicables en las rutas de trabajo
y sus directorios ascendientes. Los documentos de `/workspace/motor-rust` solo
se leyeron. No se abrió código fuente de OpenSees, CalculiX, Stabileo ni otros
motores, ni se usaron como oráculos. No se accedió a repos externos de código
para obtener algoritmos o implementaciones.

## Alcance y evidencia

Se creó el ADR en español con estado «Propuesto, pendiente de aprobación de
Jonathan», esta bitácora y un enlace en la lista existente del README. No se
modificaron manifests, lockfile, código Rust, `clean-room/**`, `.github`,
`CODEOWNERS` ni `docs/CLEAN_ROOM.md`; no se crearon workflows, `deny.toml`,
`LICENSE` ni texto legal. No se hizo commit.

La consulta web de las páginas de crates.io y de su API devolvió errores de
acceso. Una consulta directa de metadatos mediante la API falló por resolución
DNS. Se conservó el corte verificado suministrado por el usuario, distinguiéndolo
de la prueba de compilación registrada en arquitectura §10. No se inventaron
fechas de actualización faltantes, MSRV no declarados ni cifras de rendimiento
o tamaño. La revisión de archivos LICENSE y del grafo por versión sigue
**PENDIENTE**, aunque §10 registra comprobaciones históricas para tres crates.

La recomendación de faer se apoya en la cobertura y compilación que registra
la arquitectura. No presupone ejecución correcta en wasm, acceso a inercia,
disponibilidad de todas las APIs con features mínimas ni un solver disperso
de autovalores aprobado. Esas limitaciones figuran en los criterios de
aceptación y en el plan de prueba del ADR.

## Verificación local

- `git diff --check` y comprobación de espacios de los documentos nuevos: satisfactorios.
- Enlaces Markdown locales de los tres archivos de esta entrega: destinos existentes.
- Revisión de archivos cambiados: solo ADR-002, esta bitácora y una línea en README.

No se ejecutaron tests Rust por tratarse de un cambio exclusivamente documental.
No se declara ningún criterio numérico ni la aprobación del ticket como cumplido.
