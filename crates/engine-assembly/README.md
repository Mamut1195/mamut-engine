# engine-assembly · ENG-ASM-001

Ensamblador lineal de matrices elementales globales en SI. Solo depende de
crates internos. La teoría y los fixtures analíticos están documentados en
[la bitácora](../../provenance/ENG-ASM-001.md).

`DofMap::new(&model.nodes)` numera nodos por id creciente y componentes por
`Dim::dofs()`: Ux, Uy, Rz en D2 y seis componentes en D3. Admite subconjuntos
de GDL declarados, por ejemplo Ux para una cadena axial. Los índices globales
no cambian al eliminar apoyos; `LinearSystem::reduced_index` devuelve el índice
reducido o `None` para un GDL restringido.

`ElementContribution` recibe `location`, K, dos clases de M opcionales, Kg
opcional y vectores de carga por caso. Las matrices y vectores ya están en ejes
globales y su orden sigue `location`. No se interpreta conectividad geométrica,
no se crea ningún elemento y no se transforma ninguna matriz local. Un futuro
adaptador puede obtener estos datos de `engine_contracts::Element`.

`LinearSystem::from_model(backend, model, elements, prescribed)` suma los aportes
elementales y después cargas y masas nodales. Solo usa nodos, apoyos, casos y
masas del dominio; la validación topológica completa corresponde a la capa
anterior. No interpreta combinaciones. Cada caso debe declararse incluso si
solo contiene fuerzas equivalentes de elementos. `None` en M/Kg aporta cero;
las masas nodales se añaden a ambas clases de masa sin convertir una matriz
consistente en concentrada.

Las restricciones de apoyos valen cero. Un `PrescribedDof` puede sustituir ese
cero o restringir otro GDL; no admite duplicados dentro de ninguna lista. Los
valores impuestos son constantes y no dependen del factor del caso. El sistema
rechaza un modelo sin GDL libres con ENG-E-0010.

El contrato `SystemView` devuelve fuerza interna física `(K u)_f`, que incluye
`K_fp u_p`, y fuerza externa física `factor F_f`. Para resolver directamente
`K_ff u_f = factor F_f − K_fp u_p`, use `reduced_rhs` o `solve`. La recuperación
`reactions` devuelve la acción del apoyo sobre el nodo:
`R_p = K_pf u_f + K_pp u_p − factor F_p`. Incluye las fuerzas sobre apoyos.

K y M son constantes. Kg es un aporte ya evaluado en el estado de referencia
por su productor y permanece constante incluso al consultar `geometric(u)`;
este crate no calcula axiales ni implementa un análisis P-Delta. Cada evaluación
exitosa de desplazamiento actualiza la prueba; `accept(u)` reevalúa exactamente
u y lo acepta, y `rollback` recupera el último aceptado. Los errores no dejan
salidas ni estados parcialmente modificados.

Las matrices globales usan CSR y las reducidas `SymMatrix` denso, por la API de
ENG-LA-007. `solve` delega factorización y solución al backend suministrado y
traduce pivotes singulares a nodo/GDL; no implementa un solver propio. El patrón
reducido une K, ambas M y Kg y conserva ceros estructurales. La contracción
CSR con u recorre filas y columnas ordenadas, sin expandir la matriz global.
Sumas en orden de aportes, `BTreeMap`/vectores, sin hilos; reproducibilidad entre
arquitecturas bit a bit PENDIENTE de ENG-CORE-010.

Traslaciones: m; giros: rad; fuerzas: N; momentos: N·m. K y Kg: N/m, N o
N·m según componentes; M: kg, kg·m o kg·m². D2 usa el plano X-Y horizontal
según `docs/CONVENCION_EJES.md`; no se interpreta Y como vertical. Las normas
con fuerzas/momentos o traslaciones/giros requieren escalas del método de análisis.

Los fixtures se compilan desde `tests/fixtures/mod.rs` y contienen `source`,
cita y valores calculados a mano. Las pruebas no usan archivos, reloj ni hilos
y compilan también para wasm. La ejecución en un runtime wasm queda PENDIENTE.
ENG-ASM-002 conserva restricciones maestro-esclavo; ENG-ELM-* conserva la física,
transformación, evaluación e historia de los elementos.
