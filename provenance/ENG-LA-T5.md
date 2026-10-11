# Bitácora clean-room · T5 (issue #33) pivote invariante al escalado diagonal

- Ticket / PR: issue #33; rama `j2/t5-pivot-scaling`
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Rol: implementador
- Contrato implementado: engine-linalg (LinAlgBackend; política de singularidad)
- Fuentes permitidas usadas: ninguna externa; el equilibrado diagonal simétrico se deriva aquí: con B = S A S, s_i = 1/√(máx_j |a_ij|), se tiene A x = b ⇔ B y = S b, x = S y; y M = S⁻¹ B S⁻¹ ⇒ si B = L_B L_Bᵀ, L = S⁻¹ L_B. (Equilibrado de Ruiz/Jacobi: PENDIENTE de cita precisa; no se consultó.)
- Derivaciones propias: aplicado en LDLᵀ (factorize/solve) y Cholesky de masa; `solve_dense` (no simétrico) no cambia. `min_pivot_ratio` se refiere a la matriz equilibrada.
- Código de terceros copiado: ninguno
- Validación: analítico (diag(1e6,1e-9), diag(1e8,1e-6), D·A·D acoplada, singulares verdaderos siguen en ENG-E-0006)
- Comparación con oráculo: no
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4)

## Segunda ronda (J4 r3, 2026-10-11)

- Equilibrado ahora iterativo (tipo Ruiz, ≤40 pasadas, ±1 %). Ya no se declara invariancia exacta (lib.rs): mitiga, no garantiza.
- RHS: la escala de b se toma sin equilibrar y el equilibrado se aplica después (b·s ya no desborda): A=[[.5,.5],[.5,-.5]], b=[1.3e308,0] → [1.3e308,1.3e308].
- Deshacer escalas con `scale_back`, que prueba varios órdenes de evaluación para no pasar por subnormales intermedios: A=[[1e-200,1e-200],[1e-200,1]], b=[0,7.41e-224] ya no da 33 % de error.
- Pruebas nuevas: los tres casos de J4, acoplada 3×3 hasta D=1e8 y 2×2 hasta 1e14, autovalores con K=diag(1e6,1e3), M=diag(1e8,1e-6).
- Límite: sin garantía de invariancia exacta en matrices acopladas con rangos extremos.
