# Bitácora clean-room · ENG-VAL-032

- Ticket / PR: MAM-847 · VAL-032, edificios E1–E5, E2b y E5b · PR #180.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-20 (America/Guayaquil).
- Rol: especificador e implementador de fixtures y pruebas de validación.
- Contrato implementado: fixtures `val032-fixture/1` con doce
  `runs[].engine_input`, ejecutados mediante `engine_api::run`; comprobación
  de los documentos `results/1`, estados, errores y avisos esperados.
- Fuentes permitidas usadas: petición del ticket; `provenance/TEMPLATE.md`;
  `provenance/ENG-VAL-005.md`; NEC-SE-DS 2015, citada por sección y página
  en el [README de VAL-032](../docs/validation-report/fixtures/val032/README.md):
  §3.1.1 (pág. 27), §3.3.1 (págs. 33–34), §6.1.7 (pág. 55),
  §6.2.2 e (pág. 58), §6.3.2–6.3.3 (págs. 61–62), §6.3.4
  (pág. 65), §6.3.5 (pág. 66) y §6.3.6–6.3.8 (págs. 66–68).
  Libros citados por página/ecuación, sin copiar texto. Las referencias y
  salvedades se conservan en los fixtures y su README; no se consultaron
  fuentes externas ni se usó red para esta bitácora.
- Derivaciones propias: generación de fixtures mediante
  [`_ref/gen.py`](../docs/validation-report/fixtures/val032/_ref/gen.py) y
  modelo numpy propio e independiente
  [`_ref/val032_calibrate.py`](../docs/validation-report/fixtures/val032/_ref/val032_calibrate.py).
  La equivalencia física r3 queda documentada en el README y en
  [`_ref/calibrate_r3.json`](../docs/validation-report/fixtures/val032/_ref/calibrate_r3.json),
  con diferencias relativas ≤1e-10.
- Código de terceros copiado: ninguno. Sin dependencias nuevas ni cambios
  en Cargo.toml/Cargo.lock, workflows o reglas clean-room.
- Validación: referencia numérica propia para la equivalencia física r3 y
  test de integración de los doce runs reales con estados y diagnósticos
  obligatorios; los bloqueos del Engine se documentan explícitamente.
- Comparación con oráculo (repo hermano, informativa): no. No se abrieron
  salidas de OpenSees ni Frame3DD ni se incorporaron sus resultados.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket. Tampoco consulté ni usé código o resultados de CalculiX o
  LAPACK, ni salidas de OpenSees o Stabileo."
- Revisor clean-room: PENDIENTE (J4).

## Decisiones y alcance

En `crates/` el único cambio es el test
[`crates/engine-api/tests/val032_runs.rs`](../crates/engine-api/tests/val032_runs.rs).
Recorre los siete fixtures de `docs/validation-report/fixtures/val032`, exige
los doce runs únicos y ejecuta cada entrada mediante `engine_api::run`.
Rechaza `invalid`, errores inesperados y avisos sin revisar. Los fixtures se
generan con `_ref/gen.py`; no se modifica la implementación del Engine.

Los ocho runs modal/espectro (E1-modal, E2-cqc-X, E2-srss-X, E2-cqc-Y,
E2b-cqc-X, E2b-cqc-Y, E2b-cqc-XY-srss y E4-modal) están bloqueados por
`ENG-E-0006`, con estado `unstable`, debido a GDL sin masa. La referencia
numpy condensa esos GDL; el Engine todavía no lo hace. No se añaden masas
artificiales ni restricciones para ocultar esta brecha.

E3-pdelta está bloqueado por `ENG-E-0007`, con estado `notConverged`:
pDelta no proyecta Kg y las fuerzas geométricas/residual mediante la
transformación de diafragma. La corrección del Engine queda fuera de VAL-032.
El test exige los estados y códigos exactos de los nueve bloqueados y falla
si empiezan a converger, para obligar a revisar fixtures, evidencia y README.

E3-lineal, E5-combinaciones y E5b-combinaciones devuelven `warning`.
El README justifica individualmente los avisos #84
(`API-W-DIAPHRAGM-RECOVERY`) y #85 (`API-W-COMBINED-PRESCRIBED`).
El test comprueba sus códigos por run, la multiplicidad de #84, los diez ids
de combinación de #85 y los desplazamientos prescritos nulos. Las fuerzas
de barra de E5 conservan su condición de diagnóstico por #84.

## Equivalencia física r3 y límites de validación

El calibrador propio usa geometría, propiedades, masas y cargas de los JSON,
sin consumir resultados del Engine. Compara la definición anterior con
maestro flotante en el centro de masa y la nueva con maestro conectado,
reparto de masas y transporte de fuerzas y momentos equivalentes.
Los períodos T1–T15, respuestas lineales de piso y reacciones exigidas
presentan diferencias relativas ≤1e-10. E5b es exclusivamente estático y no
tiene períodos que comparar. Esta equivalencia no habilita los análisis
modal/espectro ni valida respuestas P-Δ bloqueadas.

Los localizadores normativos y bibliográficos se conservan sin transcribir
texto de las fuentes. I = 1.0 frente a Tabla 6 de NEC-SE-DS y la numeración
y factores de combinaciones de NEC-SE-CG §3.4.3 siguen no verificados, como
declara el README. Las comparaciones con oráculos permanecen pendientes.

## Verificación de esta bitácora

Se revisaron la plantilla, el ejemplo permitido, el README y el test local.
Esta tarea solo crea la bitácora; no vuelve a ejecutar las suites ni el
calibrador. La evidencia numérica enlazada corresponde al cierre r3
documentado en el README. Trabajo sin git y sin red.

## Pendientes

Revisión clean-room de J4 y seguimiento de los bloqueos del Engine indicados
en MAM-847; comparaciones con oráculos y cotejos normativos pendientes según
el README.
