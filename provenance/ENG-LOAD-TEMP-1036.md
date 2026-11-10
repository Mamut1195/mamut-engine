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
temperatura. Ahora también lo añade si el modelo ya tiene otro error, y
`run` elige `unsupported` porque ese código está en la lista. El rechazo por
`member_loads` en el caso de gravedad o en el de referencia, y el rechazo de
un documento que no trae exactamente un análisis, salen de las mismas
funciones que usa `run`, con el mismo código y el mismo texto.

## Declaración de sesión

Esta sesión no vio salidas de programas externos. No abrió código de
OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni derivados.
