# Bitácora clean-room · ENG-CORE-002

- Ticket / PR: ENG-CORE-002 (política de unidades y conversión en frontera) · PR en borrador de la rama `j2/eng-core-002-units`, apilada sobre `j2/eng-core-001-engine-core` (#6)
- Autor(es): Codex (gpt-6.1-sol, esfuerzo medium) como autor del código, bajo la dirección de J2 (agente Grok Bot); bitácora y verificación por J2 · 2026-10-10 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: lista cerrada de unidades (`Unit`), magnitudes (`UnitKind`), análisis de texto de unidad (error `ENG-E-0001` si es desconocida, nunca se supone), conversión a SI y desde SI solo en la frontera, `convert` entre unidades de la misma magnitud y `UnitsOut` para declarar `units_out` en resultados. Ver `crates/engine-core/src/units.rs`.
- Fuentes permitidas usadas: ninguna fuente externa. Los factores de conversión (potencias de diez; grados a radianes con π) son definiciones del Sistema Internacional de conocimiento general. Textos y rustdoc son redacción propia.
- Derivaciones propias: factores exactos como constantes; la ida y vuelta se verifica por pruebas con error relativo <= 1e-15 (valores 0.1, 123.456789, 1e-9, 1e9, negativos y cero).
- Código de terceros copiado: ninguno. Solo biblioteca estándar de Rust. No se reutilizó código de otros repositorios del programa.
- Validación: pruebas unitarias y doctests propios; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo doc --no-deps` con avisos denegados y `bash clean-room/check.sh` en local (la CI remota no corre por la facturación de GitHub Actions).
- Comparación con oráculo (repo hermano, informativa): no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: pendiente (OpenCode comenta el veredicto en el PR; la aprobación formal es de la persona responsable).
