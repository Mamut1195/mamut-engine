# Bitácora clean-room · ENG-CORE-001

- Ticket / PR: ENG-CORE-001 (tipos base, errores y diagnósticos) · PR en borrador de la rama `j2/eng-core-001-engine-core`
- Autor(es): Codex (gpt-6.1-sol, esfuerzo medium) como autor del código, bajo la dirección de J2 (agente Grok Bot); bitácora y verificación por J2 · 2026-10-10 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: tipos base de `engine-core` (sección 4.1 de la arquitectura interna del proyecto, documento de diseño propio del Engine: `Real`, ids, `Dof`, `Dim`, `ContractId`, `EntityRef`, `Severity`, `Diagnostic`, `EngineError`, `EResult`) y catálogo de códigos `ENG-E-xxxx`. No implementa ningún `kind` de elemento.
- Fuentes permitidas usadas: ninguna fuente externa. Los tipos son estructuras de datos genéricas (identificadores numéricos, enumeraciones de grados de libertad, códigos de error) definidas por el diseño propio del proyecto. Los nombres de los grados de libertad (Ux, Uy, Uz, Rx, Ry, Rz) son notación estándar de la mecánica estructural, no de una herramienta particular. Los textos de mensajes y explicaciones son redacción propia.
- Derivaciones propias: no aplica (no hay ecuaciones ni métodos numéricos en este ticket).
- Código de terceros copiado: ninguno. Sin dependencias externas (solo biblioteca estándar de Rust).
- Validación: pruebas unitarias y doctests propios (unicidad y formato `ENG-E-NNNN` del catálogo, mensaje y explicación no vacíos, `explain` de código desconocido, `Display`, GDL por dimensión); `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo doc` en local (la CI remota no corre por la facturación de GitHub Actions).
- Comparación con oráculo (repo hermano, informativa): no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: pendiente (OpenCode comenta el veredicto en el PR; la aprobación formal es de la persona responsable).
