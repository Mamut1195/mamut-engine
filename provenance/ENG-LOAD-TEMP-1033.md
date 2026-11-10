# Bitácora clean-room · ENG-LOAD-TEMP-1033

- Ticket / PR: MAM-1033. Tests de los casos de temperatura que sí se permiten,
  y `validate` anticipa el rechazo ENG-E-0013. No cierra MAM-1033 ni MAM-107.
  No aprueba presupuestos.
- Autor: agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-09
  (América/Guayaquil).
- Rol: implementador. Sesión limpia. Sin operador de oráculo.
- Contrato implementado: el mismo `validate` de engine-api. No hay kind nuevo
  ni fórmula nueva de Kg.
- Fuentes permitidas: solo este repositorio. El alargamiento δ = α·ΔT·L sale
  de la partición ε = ε_mecánica + α·ΔT en
  `provenance/ENG-LOAD-TEMP-972.md`. La rigidez geométrica de la cercha,
  (N/L)[P, −P; −P, P] con P = I − ddᵀ, está en
  `crates/engine-elements-truss/src/geometric.rs`. No se reclama libro,
  artículo ni número de ecuación externo. Cita externa: cita desconocida.
- Derivación propia: la de abajo.
- Código de terceros copiado: ninguno.
- Validación: analítica, en `crates/engine-api/tests/temperature_permitted.rs`
  y `bindings/python/tests/test_temperature_permitted.py`.
- Comparación con oráculo (repositorio hermano, informativa): no. No se
  ejecutaron ni se leyeron OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK.
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX, LAPACK ni derivados para este ticket."
- Revisor clean-room: PENDING

## Qué hace validate

`validate` devuelve `Result<Vec<Diagnostic>, ApiError>`. Un defecto de modelo
es `Ok` con diagnósticos; no hay código de salida. ENG-E-0013 ya es el código
de capacidad no soportada, y `validate` ya lo usa cuando un pórtico recibe
temperatura sin fórmula. `run` traduce ese diagnóstico a `unsupported` y
salida 3. Se reutiliza el mismo código y el mismo mensaje que el rechazo de
ejecución. No se añade una entrada al catálogo.

## Derivación

Barra alineada con X, extremo i impedido, extremo j libre en Ux y con Uy
impedido. La fuerza de nudo en j es E·A·α·ΔT y la rigidez axial es E·A/L, así
que Ux = α·ΔT·L. Datos: α = 0.5 1/K, ΔT = 3 K, L = 2 m. El producto
0.5·3·2 es 3.0 m.

P = I − ddᵀ con d = (1, 0) anula la fila axial. Kg de la compresión de
gravedad no suma nada a ese Ux. La temperatura está solo en el caso lateral.

El pandeo añade un puntal vertical de longitud Lv = 1 m. K_y = E·A/Lv.
Con N = −P en la barra horizontal, −Kg_y = P/L. Entonces
λ = (E·A/Lv) / (P/L) = E·A·L / (Lv·P). Con E = 8 Pa, A = 4 m² y P = 16 N,
λ = 32·2/(1·16) = 4. Un caso distinto de la referencia puede llevar
temperatura: no entra en el axil de referencia ni en Kg.

La tolerancia absoluta de los tests es 1e-9. Acota el camino del solver.

## Declaración de sesión

Esta sesión no vio salidas de programas externos. No abrió código de
OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni derivados, ni el
informe de humo del banco comparativo.
