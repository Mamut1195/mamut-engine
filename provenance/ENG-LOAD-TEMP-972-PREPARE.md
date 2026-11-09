# Bitácora clean-room · ENG-LOAD-TEMP-972-PREPARE

- Ticket / PR: MAM-972. `PreparedLinear::prepare` aplica `temperature_loads`
  igual que `run`. No cierra MAM-972 ni MAM-107. No aprueba presupuestos.
- Autor: agente de implementación, Grok 4.7, 2026-11-09 (América/Guayaquil).
- Rol: implementador. Sesión limpia. Sin operador de oráculo.
- Contrato implementado: el mismo `PreparedLinear::prepare` ya publicado.
  No hay kind nuevo ni ley nueva. La carga sigue siendo
  `cases[].temperature_loads` sobre `truss2d.linear` y `truss3d.linear`.
- Fuentes permitidas: solo este repositorio. La partición
  ε = ε_mecánica + α·ΔT y el signo de tracción positiva están en
  `provenance/ENG-LOAD-TEMP-972.md`. No se reclama libro, artículo ni número
  de ecuación externo. Cita externa: cita desconocida.
- Derivación propia: la de abajo. El alargamiento libre δ = α·ΔT·L no estaba
  afirmado en la bitácora anterior, que trata la barra impedida en ambos
  extremos.
- Código de terceros copiado: ninguno.
- Validación: analítica, en `crates/engine-api/tests/prepare_temperature.rs`,
  `crates/engine-api/tests/temperature_geometric.rs` y los tests Python
  `test_prepare_temperature.py` y `test_temperature_geometric.py`.
- Comparación con oráculo (repositorio hermano, informativa): no. No se
  ejecutaron ni se leyeron OpenSees, Frame3DD ni Stabileo.
- Declaración: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDING

## Qué se eligió

Opción 1 para `prepare`. Ensambla con el mismo modelo que `run`. La función
`temperature::model_for_system` es el único sitio que decide si hay registros
de temperatura y, si los hay, llama a `temperature::with_joint_forces`. No hay
una segunda fórmula.

Para `buckling.linear` y pDelta no se modifica Kg. El productor geométrico
sigue el axil mecánico y no resta αΔT. Esa resta queda fuera de este cambio.
El caso de referencia del pandeo, y el caso de gravedad de pDelta, se rechazan
si traen `temperature_loads` no vacío.

## Código ENG-E-0013

El catálogo de `engine-core` define ENG-E-0013 como capacidad no soportada:
la entrada es válida y el componente no implementa la operación pedida. El
ejemplo del catálogo es evaluar la rigidez geométrica fuera de lo que el
componente soporta. El mismo código ya rechaza una carga de miembro en la
referencia de pandeo y en la gravedad de pDelta, porque Kg no corrige las
acciones de extremo. La temperatura uniforme es el mismo hueco: Kg no incluye
αΔT. No se añade otro código.

## Derivación

La barra es recta, homogénea y elástica. El extremo i está impedido y el
extremo j está libre a lo largo de la barra. Con tracción positiva,

ε = ε_mecánica + α·ΔT.

Sin fuerza exterior el axil es nulo, luego ε_mecánica = 0 y el alargamiento
del extremo libre es

δ = α·ΔT·L.

Datos de la barra libre: α = 0.5 1/K, ΔT = 3 K, L = 2 m. El producto
0.5·3·2 es 3.0 m y es exacto en binary64. Ese valor es Ux del nudo libre.
Los demás componentes de desplazamiento son 0.

La misma barra con una fuerza axial F en el nudo libre suma
δ_mec = F·L/(E·A). Con F = 16 N, E = 8 Pa y A = 4 m²,
δ_mec = 16·2/(8·4) = 1.0 m, también exacto. Fuerza y temperatura en el mismo
caso dan Ux = 1.0 + 3.0 = 4.0 m por superposición lineal.

`prepare` sigue rechazando combinaciones. En `run`, el factor 2 sobre el caso
térmico da Ux = 2·δ = 6.0 m.

La tolerancia absoluta de los tests es 1e-12 m. Acota el camino del solver;
no calibra el producto cerrado.

## Declaración de sesión

Esta sesión no vio salidas de programas externos. No abrió código de
OpenSees, Frame3DD, Stabileo ni derivados, ni el informe de humo del
banco comparativo.
