# Bitácora clean-room · ENG-LA-009 · condensación de Guyan (#184)

- Ticket / PR: GitHub #184, J2; prioridad V1 aprobada por J0.
- Autor: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: especificador e implementador, sin delegación.
- Contrato implementado: función reutilizable `engine_linalg::guyan_condense`,
  `GuyanReduction::recover`; integración con `LinearSystem::lowest_modes`,
  modal y espectro CQC/SRSS.
- Fuente teórica citada: R. J. Guyan, “Reduction of stiffness and mass matrices”,
  AIAA Journal, vol. 3, núm. 2, 1965, **p. 380**, DOI 10.2514/3.2874.
  Localización concreta: nota de una página, reducción estática de las matrices
  de rigidez y masa mediante la transformación de los GDL eliminados.
  La especialización a filas de masa nulas se deriva aquí de forma propia.
  Se cita el método; no se descargó ni copió código, texto, tablas o figuras.
- Fuentes usadas: contratos y código propios del repositorio; derivación
  algebraica propia abajo; referencia NumPy propia ya versionada
  `docs/validation-report/fixtures/val032/_ref/val032_calibrate.py` y períodos
  `calibrate_r3.json`, `edificios[].comprobaciones.T1_T15.nuevo_s`.
- Código de terceros copiado: ninguno. Dependencias nuevas: ninguna.
- Comparación con oráculos externos: no. Trabajo y comprobaciones offline.
- Declaración: "No consulté código fuente ni resultados de OpenSees, Frame3DD,
  Stabileo, CalculiX, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket."
  No se abrió el informe de benchmark prohibido ni archivos de auditoría.
- Revisor clean-room: PENDIENTE (J4).

## Derivación propia y decisiones

Tras apoyos y transformación maestro/esclavo, se conserva el orden de los GDL.
Una fila de M exactamente nula (y su columna, por simetría) pertenece a s;
las restantes pertenecen a m, incluso con diagonal nula y acoplamiento no nulo.
No hay umbral físico ni masa artificial. Una diagonal negativa da ENG-E-0006;
un modelo sin masa libre también da ENG-E-0006 con GDL identificable.

La ecuación sin inercia es Ksm φm + Kss φs = 0. Con Kss definida positiva,
R = −Kss⁻¹ Ksm, φs = R φm y Kred = Kmm + Kms R. Entonces
T = [I; R] (con la permutación original) satisface TᵀKT = Kred y
TᵀMT = Mmm. Por det(K−λM) = det(Kss) det(Kred−λMmm), sus raíces
finitas coinciden exactamente en aritmética real. La norma de masa se conserva.

Se usa una sola factorización LDLᵀ propia de Kss, se comprueba su inercia
(s,0,0) y se resuelven todas las columnas Ksm en una llamada. No se forma
una inversa. Kss singular o no positiva da **ENG-E-0014: Mecanismo sin masa**,
registrado en el catálogo de engine-core y traducido a nudo/GDL por assembly;
la API devuelve Unstable. En el caso indefinido el índice señala un GDL del
bloque implicado, no promete localizar una componente única del mecanismo.
El complemento se calcula en una mitad y se refleja para simetría exacta.

Si s está vacío no se copian matrices ni se factoriza Kss: se llama al mismo
`lowest_modes` anterior. La expansión adicional maestro/esclavo está limitada
al camino condensado (`if condensed && diafragmas`) para conservar los
resultados numéricos y los hashes del camino existente sin condensación.
Ese camino mantiene el aviso **API-W-DIAPHRAGM-RECOVERY (#84)** y su alcance
actual de recuperación; corregirlo globalmente corresponde a otro ticket.
El aviso también se conserva tras Guyan: recuperar la cinemática modal no
resuelve por sí solo la recuperación general de esfuerzos/reacciones de #84.
En el camino condensado se recuperan primero las componentes sin masa y después T_f φ, sin sumar
asentamientos. Participaciones, masas efectivas y espectro usan esta forma
completa y la masa global original; el solver normaliza con Mmm y modal con M.

Se conservan el presupuesto de pasos y los checkpoints de modal/espectro,
la cancelación antes/después del solver y la ausencia de cambios de estado.
Como el eigensolver existente, la reducción densa es una operación indivisible;
las reservas matriciales usan DMat::zeros y sus límites fallibles por destino.
No se introduce una conexión a MemoryBudget que el camino modal no tenía.

## Pruebas

- Linalg: eliminación densa independiente con solve_dense de reference.rs y
  transformación TᵀKT, espectro finito, residual completo Kφ−λMφ, norma de masa,
  recuperación intercalada, masa Mmm acoplada, un factor/una resolución con
  varias RHS; bloque singular e indefinido (incluido pivote 2×2), masa negativa,
  ausencia de masa, dimensiones y no eliminación de masas diminutas/acopladas.
  Ronda 2: Gram completo φᵀMφ=I tras `recover()` sin renormalizar, con términos
  de masa fuera de diagonal; `None` sin resultados alternativos, mutación de
  bits ni sustitución de matrices, cero factores y cero resoluciones.
  Mecanismos singulares en dos posiciones: GDL exacto en entidad y mensaje.
- Dominio: voladizo Euler con masa lumped solo traslacional, E=A=ρ=L=1,
  I=1e−4; m_punta=0.5, k=3EI/L³, T=2π√(m/k), tolerancia relativa 1e−9.
  Recuperación φ_rotación=3φ_traslación/(2L), ortogonalidad y completitud.
  Mecanismo rotacional sin masa con nudo/GDL global exactos en ENG-E-0014.
  Ronda 2: igualdad bit a bit de todos los campos de `lowest_modes` contra
  backend directo sin GDL sin masa (lumped y consistente, 0/1/todos los modos).
  Masa consistente acoplada con giro sin masa contra Schur analítico propio.
  Diafragma propio con esclavo excéntrico (Δx=2, Δy=3), Uz sin masa y K acoplada:
  ux_s=ux_m−3θ, uy_s=uy_m+2θ, θ_s=θ_m, norma y M-ortogonalidad globales.
- VAL-032: ocho runs modales ejecutables, 15 períodos por run contra la
  referencia NumPy ya calibrada. El test lee la tolerancia pre-registrada de
  cada fixture (1e−5 relativa); no se modifican fixtures, hashes ni referencias.
  Cada run conserva Warning API-W-DIAPHRAGM-RECOVERY (#84), aviso existente
  sobre cobertura general de recuperación de diafragmas. Se comprueba además
  la cinemática maestro/esclavo de cada modo. E3-pdelta conserva #185.

## Verificación de la ronda 2 (2026-10-20)

Toolchain: Rust/Cargo 1.99.0 stable; `PATH` antepone el toolchain solicitado,
`CARGO_TARGET_DIR=/workspace/.cargo-target/mamut-engine-guyan`, Cargo offline.
Los logs, documentos de resultados y resúmenes se guardan en
`$CARGO_TARGET_DIR/guyan-review/r2/` (fuera del diff).

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK, salida 0. |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK, salida 0. |
| `cargo test --workspace --offline` | 713 passed, 0 failed, 0 ignored; 112 suites/binarios, incluidos doctests. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK, salida 0. |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK, salida 0. |
| `bash clean-room/check.sh origin/main` | Clean-room: OK, salida 0, incluidos los archivos nuevos. |
| `git diff --check` | OK, salida 0. |
| Fixtures existentes frente a origin/main | 96/96 idénticos byte a byte; VAL-032 26/26, incluidas sus referencias y hashes. |
| Nativo/wasm debug y release | 16/16 hashes iguales por perfil, diez corridas por caso/target. |
| Tabla docs/DETERMINISMO.md | 16/16 hashes originales conservados en cada perfil; salidas nativas debug/release idénticas byte a byte. |

Las comprobaciones de esta ronda usan el índice ya presente del worktree,
que incluye los tres archivos nuevos con intención de añadir; el verificador
clean-room inspecciona también sus líneas. No se modificó el índice en esta
ronda. No hay commits, cambios de rama ni acceso de red.

Recuentos de la ronda 2: cuatro tests nuevos respecto de la ronda 1
(709 → 713), más ampliaciones de las pruebas existentes. Las suites focalizadas
pasan: linalg/Guyan 4/4, analysis/modal 10/10, API/dynamic 14/14
(dentro del workspace), VAL-032 1/1 (también reejecutada con `--nocapture`).
Los registros `checks.json`, `summary.json`, `determinism-checks.json` y
`fixture-integrity.json` contienen los resultados de esta ronda.

### VAL-032 · períodos y avisos revisados por run

Referencia: valores `nuevo_s` ya calibrados del modelo propio NumPy; no se
regeneró ningún JSON de referencia. Tolerancia: leída de cada fixture, sin
cambios. Los 15 períodos se verifican individualmente (120/120 aprobados).

| Run VAL-032 | Estado | Máximo error relativo T1–T15 | Tolerancia relativa | Justificación de Warning |
| --- | --- | --- | --- | --- |
| E1-modal | Warning | 4.130029651606e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |
| E2-cqc-X | Warning | 4.130029651606e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |
| E2-srss-X | Warning | 4.130029651606e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |
| E2-cqc-Y | Warning | 4.130029651606e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |
| E2b-cqc-X | Warning | 3.819167204711e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |
| E2b-cqc-Y | Warning | 3.819167204711e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |
| E2b-cqc-XY-srss | Warning | 3.819167204711e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |
| E4-modal | Warning | 5.928590951498e-14 | 1e-05 | Únicamente aviso existente #84 API-W-DIAPHRAGM-RECOVERY. |

E3-pdelta permanece NotConverged/ENG-E-0007, bloqueado por #185, sin cambios.
Además se verificó sobre los documentos exportados que los 15 modos completos
son M-ortonormales (máximo error absoluto 5.61e−16) y que la suma de masas
efectivas alcanza la masa libre direccional (máximo error relativo 6.67e−16).
La suite API comprueba período analítico con masa nodal exclusivamente
traslacional, participación, recuperación de giros en SRSS/CQC, salida
Unstable/ENG-E-0014 con nodo/GDL y cancelación en cada frontera dinámica.

Pendientes: revisión clean-room J4; seguimiento independiente #84 del aviso de
recuperación de diafragmas y #185 de P-Delta. Sin dependencias nuevas, unsafe,
unwrap/expect/panic en src ni cambios en manifiestos, lockfile, vendor,
clean-room, protocolo, workflows o CODEOWNERS.
