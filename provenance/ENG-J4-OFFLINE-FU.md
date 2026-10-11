# Bitácora clean-room · seguimiento J4 offline (#21, #23, #25, #11)

- Ticket / PR: /workspace/j4rev/comments/engine-cleanroom-offline.md; rama `j2/j4-offline-followup`
- Autor(es): J2 (agente, Grok Bot; Codex no usado) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: correcciones en engine-assembly, engine-contracts (kit), engine-linalg, engine-core (pruebas)
- Fuentes permitidas usadas: ninguna externa; aritmética de punto flotante propia.
- Derivaciones propias:
  - assemble.rs:113/124 — la simetría se compara como |a/2−b/2| ≤ ½·tol·máx (sin desbordar), se rechazan siempre valores no finitos y la media es a/2+b/2 (antes (a+b)/2 desbordaba a ∞).
  - conformance.rs:505 — rotaciones: el modo rígido se exige de forma INFINITESIMAL (K₀·u≈0 con la tangente del origen), no con giro finito de 1 rad (podía rechazar elementos no lineales válidos); traslaciones siguen con la fuerza. Cubre la baja de T2 sobre rotaciones infinitesimales.
  - robustness.rs — `1 << 40` no compila en wasm32: constante por destino (2^40 en 64 bits, 2^29 en 32 bits).
  - catalog.rs:168 — la prueba de ENG-E-0013 ya no exige que sea el último código (orden y unicidad se prueban aparte).
  - reference.rs `rescale` — tres órdenes de evaluación para evitar desbordamiento y subdesbordamiento intermedios (v·num subdesbordaba aunque el resultado fuera normal).
- Límite residual: la normalización de RHS por su máximo (x/rs) puede llevar componentes diminutas a subnormales; es límite de f64 documentado, no corregido aquí.
- Código de terceros copiado: ninguno
- Validación: analítico (pruebas `symmetry_check_and_mean_do_not_overflow`, `rescale_tests`, kit espacial)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)

## Segunda ronda (J4 engine-r2, 2026-10-11)

- assemble.rs: la diferencia se calcula directamente (exacta con subnormales: 5e-324 frente a 0 es asimétrico con tol=0) y solo si desborda se usan mitades; `a == b` se conserva exacto; la media usa `(a+b)·0.5` y solo recurre a mitades si la suma desborda (no pierde 5e-324).
- catalog.rs:153 ya no exige exactamente 13 códigos (`>= 13`).
- conformance.rs: el modo de giro vuelve a llamar `response()` y `check_response` (se conserva la validación de longitudes/finitud) además de K₀·u≈0.
- Merge de `main` (sin force push); conflicto en j4_followup.rs resuelto conservando ambas pruebas.
- `rescale` (candidato «normal» con posible error si hay subdesbordamiento parcial): no cambia en esta ronda; límite preexistente.
