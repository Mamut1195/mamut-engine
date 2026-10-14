# engine-assembly · ENG-ASM-001

Ensamblador lineal de matrices elementales globales en SI. Solo depende de
crates internos. La teoría y los fixtures analíticos están documentados en
[la bitácora](../../provenance/ENG-ASM-001.md).

`DofMap::new(&model.nodes)` numera nodos por id creciente y componentes por
`Dim::dofs()`: Ux, Uy, Rz en D2 y seis componentes en D3. Admite subconjuntos
de GDL declarados, por ejemplo Ux para una cadena axial. Los índices globales
no cambian al eliminar apoyos; `LinearSystem::reduced_index` devuelve el índice
reducido o `None` para un GDL prescrito o dependiente. `coordinates()` consulta
las posiciones originales del mismo mapa de geometría usado por los diafragmas.

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

`from_model_with_diaphragms` añade restricciones maestro-esclavo XY en D3:
`u = T_f q_f + u_0`. Los esclavos cumplen `Ux_s = Ux_m − ry Rz_m`,
`Uy_s = Uy_m + rx Rz_m` y `Rz_s = Rz_m`; Uz/Rx/Ry siguen independientes.
La reducción de K/M/Kg usa `T_fᵀ A T_f`.

El contrato `SystemView` devuelve fuerza interna física `T_fᵀ K u`, que incluye
los prescritos, y fuerza externa física `factor T_fᵀ F`. Para resolver
directamente `K_red q_f = T_fᵀ(factor F − K u_0)`, use `reduced_rhs` o `solve`.
`solve_cases` conserva el orden del lote y reutiliza una sola factorización;
devuelve los mismos independientes libres que `solve` caso a caso.
`Constraints::expand` recupera dependientes y prescritos sin escalar estos últimos.
La recuperación `reactions` devuelve `R_p = (Tᵀ(Ku − factor F))_p`, incluida
la fuerza transferida por esclavos a un maestro prescrito. Sin diafragmas se
obtienen `K_ff u_f = factor F_f − K_fp u_p` y
`R_p = K_pf u_f + K_pp u_p − factor F_p`. Incluye las fuerzas sobre apoyos.

K y M son constantes. Sin productores registrados, Kg es una instantánea del
estado declarado con `set_geometric_reference` (por defecto u reducido nulo).
Si existe cualquier aporte Kg estático, incluso cero, `geometric(u)` rechaza
otro estado con ENG-E-0013. Sin aportes, devuelve cero para cualquier u finito.

`GeometricProducer { location, eval }` permite evaluar Kg tipo P-Delta con los
esfuerzos N del estado u. `set_geometric_producers` sustituye el registro entero:
cada productor sustituye la Kg estática del único `ElementContribution` cuya
`location` coincide exactamente, incluido su orden, y si no coincide añade un
bloque independiente. Ubicaciones repetidas entre productores o coincidencias
con varios aportes dan ENG-E-0002 por ambigüedad. Un aporte Kg estático que no
tiene productor sigue sujeto a su referencia; vaciar el registro restaura las
instantáneas originales. `global_geometric()` conserva la instantánea original;
la matriz vigente reducida se obtiene con `geometric(u, out)`.

El registro valida ubicaciones y evalúa en la referencia expandida. Cada consulta
expande u reducido con `Constraints::expand`, incluidos desplazamientos
prescritos, extrae u_e en el orden elemental, evalúa, ensambla por
`assemble_matrix` con simetría exacta y reduce a libres. Sobrescribe `out`.
Propaga los errores del productor; ENG-E-0010 por tamaño/asimetría/índices y
ENG-E-0008 por datos o sumas no finitos. El patrón incluye todos los pares
libres de las ubicaciones registradas, incluso si Kg es cero en la referencia.
Las clausuras son deterministas y no requieren Send/Sync. El ensamblaje sigue
el orden de los aportes originales, después el de productores independientes;
las evaluaciones siguen el orden del registro.

El adaptador `engine_elements_truss::geometric_producer` captura elemento y
contexto para calcular N=(EA/L)d·Δu y Kg=(N/L)(I−ddᵀ), con geometría fija.
La dependencia interna de assembly, ya usada por truss en pruebas, pasa a ser
de producción; no se incorpora ningún crate adicional.
Las [derivaciones y pruebas P-Delta](../../provenance/ENG-ASM-001-KG.md) explican
el alcance incremental de K+Kg y el multiplicador crítico del arco de dos barras.
Esto proporciona Kg al método de análisis; `tangent` e `internal_force` siguen
siendo las operaciones lineales K y Ku. Cada evaluación
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
