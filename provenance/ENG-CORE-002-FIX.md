# Bitácora clean-room · ENG-CORE-002-FIX

- Ticket / PR: ENG-CORE-002-FIX (compatibilidad con los códigos ASCII de units/1).
- Autor(es): Codex (gpt-6.1-sol, esfuerzo medium), bajo la dirección de J2 · 2026-10-10 (America/Guayaquil).
- Rol: implementador
- Contrato implementado: `units/1`; `Unit::parse` reconoce `N.m` como `NewtonMeter` y `kN.m` como `KilonewtonMeter`, conservando todos los símbolos y alias anteriores. `MN.m`, `N.mm` y `n.m` siguen devolviendo `ENG-E-0001`. Ver `crates/engine-core/src/units.rs`.
- Fuentes permitidas usadas: códigos ASCII de `units/1` del repositorio `mamut-contracts`, `schemas/core/units/1.0.0.schema.json`, según la especificación suministrada por J2 para este ticket. Rustdoc y pruebas son redacción propia.
- Derivaciones propias: ninguna; correspondencia explícita de los dos códigos ASCII con las variantes existentes.
- Código de terceros copiado: ninguno. No se reutilizó código de otros repositorios del programa.
- Validación: pruebas unitarias propias para ambos códigos ASCII, equivalencia entre `N.m` y `N·m` y rechazo de `MN.m`, `N.mm` y `n.m`; pruebas existentes de símbolos y alias. `~/.cargo/bin/cargo fmt --check`, `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`, `~/.cargo/bin/cargo test` y `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`.
- Comparación con oráculo (repo hermano, informativa): no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: pendiente (la aprobación formal es de la persona responsable).
