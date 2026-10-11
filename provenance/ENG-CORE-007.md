# Bitácora clean-room · ENG-CORE-007

- Ticket / PR: ENG-CORE-007, «Contratos v1 en Rust (traits)» · rama `j2/eng-core-007-traits`; sin commit, push ni PR en esta sesión.
- Autor(es): Codex (GPT-6) como agente de especificación, implementación, documentación y verificación bajo instrucciones del usuario · 2026-10-10 (America/Guayaquil). No se delegó a otros agentes.
- Rol: especificador e implementador; sin actividad de operador de oráculo ni auditor de similitud.
- Contrato implementado: crate `engine-contracts`; `uniaxial/1`, `section/1`, `element/1`, `analysis/1` y transporte mínimo `results/1`. Traits `UniaxialLaw`, `SectionLaw`, `Element`, `ElementFactory`, `SystemView`, `AnalysisMethod`, `ResultSink`, `RunControl`, tipos de intercambio, kit público `conformance` y plantillas públicas `examples`. No incluye solver lineal, solver de autovalores, registro de métodos, ensamblador ni esquema completo de resultados.
- Fuentes permitidas usadas: exclusivamente las categorías permitidas en [docs/CLEAN_ROOM.md §4](../docs/CLEAN_ROOM.md), contratos internos y derivaciones propias. Referencia bibliográfica: William McGuire, Richard H. Gallagher y Ronald D. Ziemian, *Matrix Structural Analysis*, 2.ª edición, John Wiley & Sons, 2000. Autores, título, edición y año confirmados mediante la [ficha institucional de Bucknell](https://digitalcommons.bucknell.edu/books/7/); edición/editorial contrastadas con el [sitio editorial de Wiley](https://bcs.wiley.com/he-bcs/Books?action=index&bcsId=4986&itemId=0471376515). **Capítulo/sección/número de ecuación: PENDIENTE**, no se consultó el texto del libro ni se atribuyeron páginas o ecuaciones específicas. No se descargaron listados de código, software complementario ni el PDF del libro. La validación numérica se basa en la derivación propia de forma cerrada descrita abajo, no en datos supuestamente extraídos del libro.
- Derivaciones propias: [nota analítica de esta bitácora](#derivación-analítica-propia-y-casos-verificados), [contratos](../crates/engine-contracts/src/traits.rs), [kit de conformidad](../crates/engine-contracts/src/conformance.rs), [ejemplos](../crates/engine-contracts/src/examples.rs) y [pruebas](../crates/engine-contracts/src/tests.rs).
- Código de terceros copiado: ninguno. Todo el código es propio. Única dependencia del nuevo crate: `engine-core` por path; biblioteca estándar de Rust. No se modificaron `engine-core` ni `engine-model`.
- Validación: analítica, con formas cerradas σ=Eε, N=EAε0, Mz=EIκz, k=EA/L, fuerzas de barra, equilibrio axial bajo carga uniforme y conservación de masa. Referencia estándar: McGuire, Gallagher y Ziemian, *Matrix Structural Analysis*, 2.ª ed. (2000); capítulo/ejercicio/página **PENDIENTE**, sin atribuir un ejemplo publicado concreto. Además, conformidad genérica, pruebas negativas y doctests propios. No se declara validado un método productivo a partir de estos mocks.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE de revisión humana; esta sesión no realiza auditoría de similitud ni aprobación de merge.

## Documentos internos consultados

Se leyeron primero `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`,
`provenance/ENG-CORE-003.md`, `clean-room/forbidden_identifiers.txt`,
`crates/engine-core/src/*.rs`, `crates/engine-model/src/*.rs` y
`docs/CONVENCION_EJES.md`. También se consultaron la configuración del workspace,
los manifests existentes, `README.md` y el comprobador `clean-room/check.sh`.
No hay `AGENTS.md` aplicable en el repo ni en sus directorios ascendientes.

La especificación autorizada se leyó directamente, sin modificarla ni copiarla
fuera del repo: `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`,
§§4.1–4.10, y `/workspace/motor-rust/04_TICKETS.md`, ENG-CORE-007.
Los contratos se implementaron con tipos propios y documentación original.
No se abrieron rutas ni fuentes prohibidas por el protocolo.

## Decisiones de alcance y arquitectura

1. **Sin dependencias externas nuevas.** `JsonValue` es un enum mínimo propio con
   nulo, lógico, número real, texto, array y objeto ordenado. No parsea ni serializa
   JSON. Los consumidores validan tipos, números finitos, claves y rangos; los
   esquemas de las fábricas y métodos son `String` de JSON Schema. La frontera
   JSON y los esquemas completos corresponden a otros tickets. Esto sustituye
   la dependencia de serialización sugerida en el documento de arquitectura.
2. **Almacenamiento matricial provisional.** `DMat` conserva dimensiones y un
   `Vec<Real>` fila-mayor, validando tamaño/desbordamiento y finitud. `SymMatrix`
   valida cuadratura y simetría exacta, sin escoger tolerancias físicas implícitas.
   `SparsityPattern` usa máscara booleana densa fila-mayor simétrica. Los tres
   **se reemplazan por el backend de ENG-LA-007**. No se implementan factorización,
   resolución, ordenamiento ni autovalores. Las fórmulas cerradas de los ejemplos
   y las operaciones privadas del kit sirven exclusivamente a su verificación.
3. **ModelView mínimo propio.** Presta dimensión y pares (NodeId, coordenadas).
   `ElementParams` conserva propiedades SI y referencias opcionales ya resueltas
   a sección/ley mediante `Arc`. No adapta ni altera `engine-model`. La fábrica
   de ejemplo recibe E, A y densidad, los guarda en el elemento y no necesita
   referencias de nodos ni propiedades adicionales del contexto.
4. **Ejes aprobados del repo.** Se aplica X-Y horizontal y Z vertical en D3,
   D2 en X-Y con Z=0 y GDL Ux, Uy, Rz. Prevalece la decisión aprobada en
   `docs/CONVENCION_EJES.md` sobre la mención pendiente de Y vertical en la
   especificación inicial. No se define una convención nueva de elevación D2.
5. **Estado explícito.** Historia plana y finita, de longitud exacta declarada;
   respuesta pura con deformaciones/desplazamientos totales y estado aceptado.
   El kit recorre una trayectoria aceptando el estado de prueba entre muestras,
   verifica repetibilidad y que la respuesta con entrada aceptada conserve fuerzas.
   Incluye una ley de prueba con historia no vacía para ejercer ese recorrido.
   `SystemView` sobrescribe salidas, acepta exactamente u y hace rollback
   idempotente sin borrar el último aceptado.
6. **Tipos de apoyo mínimos.** Carga de miembro uniforme en ejes globales;
   estaciones x/L con [N,Vy,Vz,T,My,Mz]. `CaseRef` es nominal. `Requirements`
   declara masa, Kg, no linealidad y casos. Las claves/valores de resultados
   cubren bloques y metadatos mínimos; la procedencia completa, consultas y
   serialización de `results/1` siguen en ENG-CORE-005. Normas de informes
   relativas adimensionales; el productor declara su escalado para GDL mixtos.
7. **Mocks explícitos.** Ley y sección elásticas, barra lineal D2, fábrica,
   sistema de resorte escalar, sink en memoria, control observable y método
   de aceptación de desplazamientos prescritos. El método no resuelve equilibrio;
   sus normas cero indican ausencia de corrección iterativa, con tolerancias
   explícitas de 1e-8. Se acepta antes de emitir al sink; un fallo de E/S posterior
   conserva ese último estado ya aceptado y se propaga como error. La cancelación
   previa no acepta ni emite nada. No hay reintentos ocultos.
8. **Diagnósticos existentes.** Se reutilizan `ContractId`, `Dof`, `Dim`, `Real`,
   `NodeId`, `DofIndex`, `Diagnostic`, `EngineError` y `EResult` del núcleo.
   Incumplimientos de conformidad usan ENG-E-0009 con detalle en `hint`; errores
   de implementaciones conservan su código/contexto. La referencia nominal
   `EntityRef::Law(kind)` del kit identifica el componente probado, sin afirmar
   que exista una entidad constitutiva con ese nombre en el modelo.
9. **Sin cambios editoriales no convencionales.** El README existente solo enlaza
   decisiones fundacionales; no hay convención de altas de crates ni CHANGELOG.
   No se añadieron líneas allí, ni archivos legales o cambios en rutas protegidas.

## Derivación analítica propia y casos verificados

Para un material lineal se adopta σ=Eε, así dσ/dε=E. Una sección D2 sin
acoplamiento integra tensión axial y flexión: N=EAε0 y Mz=EIκz; su tangente
es diagonal (EA, EI). Unidades: E en Pa, A en m², I en m⁴, ε0 adimensional
y κz en m⁻¹, de donde N en N y Mz en N·m.

Para una barra entre i y j, L=||xj−xi|| y n=(xj−xi)/L. En D2 se define
b=[−nx,−ny,0,nx,ny,0]. La elongación es δ=bᵀu; ε=δ/L, N=EAδ/L y
k=EA/L. La energía elástica es U=½kδ², así F=∂U/∂u=k b(bᵀu)
y K=∂F/∂u=k bbᵀ. Esto demuestra simetría. Una traslación común o rotación
infinitesimal de los dos extremos produce bᵀu=0, por lo que F=0. Al rotar
coordenadas y desplazamientos simultáneamente, F y K se transforman por la
misma rotación; no cambia el comportamiento físico.

Con E=200 Pa, A=3 m², L=2 m y δ=0.01 m, k=300 N/m y N=3 N; las
acciones locales de caras son −3 N en i y +3 N en j. La prueba contrasta
estos valores directamente y comprueba N=3 N en todas las estaciones sin carga.
Con E=200 Pa y ε=0.02, σ=4 Pa. Con EA=600 N, EI=10 N·m²,
ε0=0.02 y κz=−0.03 m⁻¹, N=12 N y Mz=−0.3 N·m.

Una carga axial uniforme q da fuerzas nodales equivalentes qL/2 en ambos
extremos. La recuperación resta esas cargas a las acciones elásticas y,
por equilibrio dN/dx=−q, obtiene N(ξ)=N_el+qL(½−ξ), ξ=x/L.
Con q=4 N/m, L=2 m y desplazamientos nulos, acciones en ambas caras
−4 N y estaciones N(0)=4, N(½)=0, N(1)=−4 N; su resultante equilibra qL.
Cargas transversales o momentos se rechazan porque esta barra no tiene flexión.

Para masa m=ρAL, cada dirección traslacional tiene masa concentrada
(m/2) diag(1,1) o consistente (m/6)[[2,1],[1,2]], obtenida integrando
las funciones lineales 1−ξ y ξ. Ambas conservan m bajo traslación uniforme;
con ρ=2 kg/m³, A=3 m², L=2 m, se comprueba m=12 kg por dirección.
Los giros no tienen inercia en esta barra. No se atribuyen estos números a
un ejemplo del libro: son casos propios con solución cerrada.

## Conformidad, cobertura y límites

El kit público incluye comprobaciones para los ocho traits. Para RunControl,
que no ofrece lectura de eventos, recibe una traza observada de cualquier
implementación; comprueba fracciones, monotonía y notas. El kit de sink envía
entradas válidas/invalidas; la prueba del sink concreto comprueba retención y
atomicidad. El kit de fábrica ejecuta los invariantes físicos sobre su producto.
Los de sistema y método ejercen salidas, aceptación, rollback, escalado de caso,
informes y cancelación. Los invariantes no observables a través de un trait
(p. ej. almacenamiento interno del sink) requieren lectura propia en sus pruebas.

Tangentes: diferencias centrales manteniendo fijo el estado aceptado, con error
relativo norma infinito estrictamente menor que 1e-6. Comparaciones de estado,
simetría y covarianza: 1e-10 relativo + 1e-12 absoluto. Modos rígidos: residuo
escalado por tangente y amplitud, 1e-10 relativo + 1e-12 absoluto. Se ejecutan
tres modos D2 y seis D3, y dos rotaciones no triviales por cada eje permitido.
La covarianza exige GDL cerrados bajo rotación; orientaciones globales contenidas
en parámetros necesitarían un fixture que las transforme expresamente. El kit
no infiere el rango de una ley ni prueba todos sus puntos: el consumidor aporta
muestras diferenciables interiores y un incremento absoluto apropiado en SI.

Diecinueve pruebas unitarias y cuatro doctests del nuevo crate: conformidad de
cada ejemplo, solución cerrada y cargas, masa conservada, estado no vacío,
sección espacial de cuatro componentes y barra espacial independiente de prueba
para verificar cada uno de los seis modos rígidos y rotaciones de los tres ejes.
Pruebas negativas detectan derivadas falsas, NaN/inf, tamaños/historias/contratos
incorrectos, tensión aceptada inconsistente, fuerza rígida espuria, masa asimétrica,
violación de covarianza, salida de sistema incompleta, falta de informes,
convergencia falsa y cancelación ignorada. Se ejercen entradas, geometría,
parámetros, estaciones, cargas, resultados anidados, desbordamientos y errores E/S.

## Verificación local

Resultados finales de la sesión:

- `~/.cargo/bin/cargo fmt --check`: satisfactorio.
- `~/.cargo/bin/cargo clippy --all-targets -- -D warnings`: satisfactorio.
- `~/.cargo/bin/cargo test`: satisfactorio; workspace completo, 43 pruebas
  unitarias y 25 doctests, incluidos 19 y 4 respectivamente del nuevo crate.
- `RUSTDOCFLAGS="-D warnings" ~/.cargo/bin/cargo doc --no-deps`: satisfactorio.
- `bash clean-room/check.sh origin/main`: satisfactorio; también ejecutado
  con índice/objetos temporales para incluir todos los archivos nuevos sin
  cambiar el índice real del usuario. Revisión adicional de identificadores
  prohibidos y rutas protegidas en todos los archivos del ticket: satisfactoria.

`#![deny(missing_docs)]` obliga a documentar toda la API pública. La primera
corrida detectó una comparación decimal exacta en un doctest (3×0.2 frente a
0.6); se corrigió a comparación con tolerancia absoluta 1e-12. No cambió la
formulación. Rustdoc estricto detectó notación matemática con corchetes
interpretada como enlaces; se marcó como código en la documentación. Cargo.lock solo registra el crate local y su dependencia existente.
No se ejecutó CI remota, benchmark productivo, comparación con oráculo ni pruebas
wasm; esos trabajos no forman parte de esta entrega de contratos.
