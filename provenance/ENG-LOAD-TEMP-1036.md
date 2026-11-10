# Bitácora clean-room · ENG-LOAD-TEMP-1036

- Ticket / PR: MAM-1036. Seguimiento de MAM-1033: test de un modelo inválido
  con temperatura en la referencia geométrica, y `validate` anticipa el
  rechazo por `member_loads` y el de un solo análisis. No cierra MAM-1036,
  MAM-1033 ni MAM-107. No aprueba presupuestos.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-09
  (América/Guayaquil, ECT, UTC-5).
- Rol: implementador. Sesión limpia. Sin operador de oráculo.
- Contrato implementado: el mismo `validate` y el mismo `run` de engine-api.
  No hay kind nuevo, ni fórmula nueva de Kg, ni código de error nuevo.
  `docs/contracts/error-codes-1.json` sigue con 32 entradas.
- Fuentes permitidas: solo este repositorio. Los textos de ENG-E-0013 y
  API-E-INPUT ya estaban en `crates/engine-api`. No se reclama libro,
  artículo ni número de ecuación externo. Cita externa: cita desconocida.
- Derivación propia: ninguna ley nueva. La temperatura en la referencia
  geométrica sigue la nota de `provenance/ENG-LOAD-TEMP-1033.md`.
- Código de terceros copiado: ninguno.
- Validación: los tests de `crates/engine-api/tests/temperature_permitted.rs`,
  `crates/engine-api/tests/validate_run_agreement.rs` y
  `bindings/python/tests/test_temperature_permitted.py`. Comparan los
  diagnósticos de `validate` con el documento de `run`.
- Comparación con oráculo (repositorio hermano, informativa): no. No se
  ejecutaron ni se leyeron OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK.
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK ni derivados para este ticket."
- Session: no abrí, no busqué ni traduje código de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK.
- Revisor clean-room: PENDING

## Qué cambia

`validate` ya devolvía ENG-E-0013 cuando la referencia geométrica trae
temperatura. También repite el rechazo por `member_loads` y el de un
documento que no trae exactamente un análisis, con el mismo código y el mismo
texto que `run`. Si `run` se detiene antes (método experimental, elemento,
ensamblado, sistema con prescritos o diafragmas, combinaciones o pasos),
`validate` devuelve esa causa. Un diafragma que el sistema acepta conserva
el aviso que `run` escribe antes del método, y no añade otro. `run` no
cambia: un modelo inválido con temperatura en la referencia sigue en
`invalid`, sin un segundo diagnóstico.

## Corrida posterior a la revisión r1

La CLI `run --input … --progress ndjson` se comparó con la base
f29a9673f0010c757674e1ed8f4004fbbf228f07 sobre el corpus de la revisión. De 313
casos, los cuatro modelos rc3d modal y spectrum grandes quedaron fuera en
debug. Los otros 309 coinciden en stdout, stderr y código de salida: 0
diferencias. Fecha de esta corrida: 2026-11-09 (ECT).

## Corrida posterior a la revisión r2

La misma CLI se volvió a comparar con esa base. El corpus principal repite
309 casos y 0 diferencias, con los cuatro rc3d grandes fuera. El corpus
extra compara 101 casos y 0 diferencias; cuatro variantes rc3d modal y
spectrum no terminan en debug. Fecha: 2026-11-09 (ECT).

## Declaración de sesión

Esta sesión no vio salidas de programas externos. No abrió código de
OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni derivados.
