# Bitácora clean-room · ENG-LA-007

- Ticket / PR: ENG-LA-007, trait interno de backend numérico y adaptador; rama
  `j2/eng-la-007-linalg-backend`, apilada sobre ENG-LA-006. Sin commit, push ni PR.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-10 (America/Guayaquil), identificación
  solicitada para este ticket. Verificación final extendida al 2026-10-11.
  No se delegó a otros agentes.
- Rol: implementador y autor de las derivaciones/pruebas propias; sin actividad
  de operador de oráculo ni auditor de similitud.
- Contrato implementado: `LinAlgBackend`, `SymFactorization`, `FactorInfo`,
  `GeneralizedEigen` y tipos matriciales propios definitivos en `engine-linalg`;
  reexportaciones compatibles en `engine-contracts`. Frontera para linsolve/1 y
  eigen/1, sin implementar el solver productivo disperso ni el análisis modal
  completo de ENG-LA-008/009.
- Fuentes permitidas usadas: contratos internos enumerados abajo, documentación
  de API generada localmente para faer 0.24.4, manifests/metadatos SPDX y avisos
  de licencia del paquete en caché; derivaciones propias de eliminación,
  congruencia, Cholesky y rotaciones ortogonales. No se atribuye ninguna ecuación
  o resultado a una página no consultada.
- Derivaciones propias: [nota analítica de esta bitácora](#derivaciones-propias),
  [backend de referencia](../crates/engine-linalg/src/reference.rs) y
  [suite genérica](../crates/engine-linalg/tests/conformance.rs).
- Código de terceros copiado: ninguno. Las definiciones anteriores de matrices
  se trasladaron desde código propio de `engine-contracts`; los algoritmos y
  adaptadores fueron escritos desde las ecuaciones y firmas de API. No se abrió
  código de implementación de librerías ni de motores externos.
- Validación: exclusivamente analítica/identidades. Fórmula clásica de autovalores
  de Toeplitz tridiagonal, derivada por seno abajo; **cita bibliográfica precisa
  PENDIENTE**, sin atribuir páginas. No se usaron fixtures u outputs externos.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
  Tampoco se consultó código de CalculiX ni se ejecutó ningún motor como oráculo.
- Revisor clean-room: PENDIENTE de revisión humana. No se realiza aprobación de
  merge ni auditoría de similitud en esta sesión.

## Documentos internos y de dependencia consultados

Se leyeron `/workspace/motor-rust/04_TICKETS.md`, en particular ENG-LA-007/008/009
y ENG-CORE-007; `/workspace/motor-rust/01_ARQUITECTURA_Y_CONTRATOS.md`, en
particular §§4.5–4.6, 9 y 10; `docs/adr/ADR-002-algebra-lineal.md`;
`docs/CLEAN_ROOM.md`; `provenance/TEMPLATE.md`; `provenance/ENG-CORE-007.md`;
`crates/engine-contracts/src/matrix.rs`, `lib.rs` y las referencias a matrices
en sus ejemplos/pruebas; `engine-core/src/lib.rs` y `catalog.rs`; manifests y
configuración del workspace. No se encontró AGENTS.md aplicable.
También se leyeron las listas/comprobador de `clean-room` para verificar los
archivos nuevos, sin modificarlos ni abrir rutas prohibidas.

Para faer se consultaron el Cargo.toml normalizado y README del paquete **0.24.4**,
los metadatos de su grafo nativo/wasm obtenidos mediante Cargo, y documentación
HTML generada con `cargo doc` en un proyecto temporal: módulo `linalg::solvers`,
`Lblt`, `PartialPivLu`, `Llt`, `SelfAdjointEigen`, `Solve`, vistas de matrices,
diagonales y permutaciones. Solo firmas y prosa de documentación, sin abrir los
links «Source» ni archivos de implementación. Los documentos de referencia son
[docs.rs de faer 0.24.4](https://docs.rs/faer/0.24.4/faer/linalg/solvers/index.html)
y [ficha crates.io](https://crates.io/crates/faer/0.24.4); el acceso web y DNS
falló, así que no se afirma una consulta remota exitosa ni nueva verificación de
releases. El paquete cached declara MIT y MSRV 1.84.0.

La inspección posterior de nombres de documentos legales encontró
`COPYING.EIGEN.MPL2`, `COPYING.LAPACK.BSD`, `COPYING.SUITE_SPARSE.AMD.BSD` y
`COPYING.SUITE_SPARSE.COLAMD.BSD`. Se leyó el encabezado MPL-2.0 y el encabezado
BSD-3-Clause de AMD; el README menciona un algoritmo SVD portado. No se abrió
código de Eigen, LAPACK, SuiteSparse ni de faer para determinar ese alcance.
Los metadatos permisivos de un crate no resuelven por sí solos esta discrepancia.

## Tipos y decisiones de implementación

1. `DMat` conserva dimensiones, almacenamiento fila-mayor privado y datos finitos.
   `SymMatrix` exige cuadratura y simetría exacta. Sus constructores/accessors
   mantienen las firmas previas. `SparsityPattern` guarda coordenadas únicas
   ordenadas; conserva `new(n, Vec<bool>)` y `entries() -> &[bool]` mediante una
   máscara en OnceLock bajo demanda. La igualdad solo compara orden/coordenadas,
   independientemente de la caché. `from_coordinates` evita máscara inicial;
   n² debe caber en usize para mantener el accessor heredado infalible.
2. `SparseMatrix` es CSR general propio. El ensamblaje usa BTreeMap, suma por
   coordenada en orden de llegada y luego emite columnas ordenadas por fila.
   No elimina ceros ni refleja tripletas automáticamente. Memoria O(filas+nnz);
   expansión densa explícita solo para problemas pequeños. Las sumas no finitas
   y las tripletas inválidas devuelven ENG-E-0008/0010.
3. `LinAlgBackend` devuelve factores poseídos como objetos del trait propio.
   Factorizar no depende de una RHS ni retiene el préstamo de la matriz/backend.
   Varias columnas se resuelven con el mismo factor. `solve_in_place` escribe
   solo después de validar toda la salida. No se crean tipos públicos de faer,
   nalgebra o sprs.
4. Referencia mínima autorizada explícitamente por el solicitante, como excepción
   de pruebas a la preferencia de §10/ADR por algoritmos de librería: eliminación
   general con pivote de fila, LDLᵀ simétrico con bloques 1×1/2×2, Cholesky de masa
   y Jacobi. No se anuncia rendimiento productivo ni solver disperso/AMD.
5. Se reutilizan códigos existentes: ENG-E-0006 singularidad con EntityRef::Dof;
   ENG-E-0007 límite iterativo; ENG-E-0008 entrada/salida no finita; ENG-E-0010
   dimensiones/simetría/masa indefinida. El stub usa ENG-E-0009 para expresar que
   el backend requerido no está disponible. No se modificó el catálogo.
6. Cada matriz se escala por su mayor entrada absoluta antes de eliminación.
   Pivotes ≤ 64 ε max(n,1) se clasifican numéricamente singulares; no se aplica
   una tolerancia física ni regularización. El GDL identifica un pivote del
   mecanismo, no todos sus movimientos. Nodo/componente requieren el mapa futuro
   del ensamblador. FactorInfo de un factor exitoso tiene singular_dofs vacío;
   un fallo singular retorna las entidades del error y no un factor resoluble.
7. `sqrt`/`hypot` son operaciones de f64 de la biblioteca estándar; `cos` solo se
   usa en el esperado analítico del test. No se añadió dependencia de libm. La
   reproducibilidad bit a bit entre destinos y la política de trascendentes
   permanecen PENDIENTES de ENG-CORE-010; no se promete esa propiedad por ausencia
   de hilos. Libm apareció como dependencia transitiva en la evaluación de faer,
   pero el grafo final no contiene ninguna dependencia externa.

## Derivaciones propias

La eliminación general resta un múltiplo de la fila pivote a cada fila inferior
para anular su columna, aplicando la misma operación a todas las RHS; después
resuelve por sustitución hacia atrás. Cambiar filas no cambia la solución si
se permutan también las cargas. Esto deriva el solve denso sin necesidad de
un listado de código de terceros.

Para una partición simétrica A=[[B,Cᵀ],[C,S]], se escribe
A=L diag(B,S−CB⁻¹Cᵀ) Lᵀ con L=[[I,0],[CB⁻¹,I]]. El pivote B es escalar o 2×2.
En el bloque [[d,e],[e,g]], det=dg−e² y B⁻¹=(1/det)[[g,−e],[−e,d]]. Sus
valores son (d+g)/2 ± hypot((d−g)/2,e). Se elige el máximo diagonal absoluto
si domina al máximo fuera de diagonal por un factor al menos 1/2; en otro caso
se permuta la pareja de mayor término fuera de diagonal a un bloque. Se
actualizan filas/columnas juntas, la permutación y las columnas ya construidas
de L. Una congruencia invertible conserva la inercia, por lo que los signos
de esos bloques dan el conteo (positivos, negativos, nulos). No se entregan
factores con valores nulos: se devuelve singularidad con el índice permutado
de vuelta al GDL original. Para resolver se permuta y escala b, se sustituye
en L, se invierten los bloques D y se sustituye en Lᵀ.

Para masa definida positiva, M=LLᵀ se obtiene igualando términos de la
triangular inferior: Lii=sqrt(Mii−ΣLik²) y Lij=(Mij−ΣLikLjk)/Ljj. Con
φ=L⁻ᵀq, Kφ=λMφ equivale a Cq=λq, C=L⁻¹KL⁻ᵀ simétrica. Cada rotación Jacobi
ortogonal Q elimina el mayor término fuera de diagonal; se acumulan los
vectores y se conserva el espectro de QᵀCQ. Se para a 4 ε max(n,1) en la
matriz escalada, con hasta 100n² rotaciones. Finalmente se ordenan los valores,
se transforma q a φ, se normaliza por sqrt(φᵀMφ) y se fija su signo dominante.
No se exige una base particular dentro de un subespacio con multiplicidad.

Para T=tridiag(−1,2,−1), q_i=sin(iθ) con θ=kπ/(n+1), q_0=q_(n+1)=0.
La identidad sin((i−1)θ)+sin((i+1)θ)=2cos(θ)sin(iθ) demuestra
Tq=(2−2cosθ)q. Es la fórmula clásica de autovalores de Toeplitz tridiagonal;
la prueba usa esta derivación propia, con cita bibliográfica precisa PENDIENTE.
Para M=cI los valores se dividen por c y los vectores se normalizan con esa masa.

La cadena de resortes con carga uniforme unitaria satisface
−x_(i−1)+2x_i−x_(i+1)=1, extremos cero. Sustituir x_i=i(n+1−i)/2 demuestra la
solución exacta. Para la masa no diagonal [[2,1/2],[1/2,1]] y K=diag(4,3),
det(K−λM)=12−10λ+(7/4)λ²; sus raíces son 12/7 y 4. Se comprueban valores,
residuales y ortogonalidad a masa. También se usa K=3M para una multiplicidad
de dos, comprobando el subespacio sin exigir vectores específicos.

## Decisión sobre faer, licencias y MSRV

Se evaluó una dependencia real optional de faer =0.24.4 con
`default-features=false`, `features=["std","sparse-linalg"]`, sin rayon.
Crates.io falló por DNS en todos sus reintentos; las consultas web oficiales
también fallaron. La caché local permitió resolver y compilar. El toolchain
al comienzo era stable Rust 1.99.0; durante la sesión apareció 1.85.1 disponible
como predeterminado, junto con el target wasm. No se cambió la configuración
de rustup. Se fijó explícitamente +1.85.1 para verificar builds/tests y +stable
para fmt/clippy, cuyos componentes no están instalados en 1.85.1.

El adaptador experimental usó multiplicación/LU/LBLᵀ de faer y una congruencia
Cholesky seguida de SelfAdjointEigen. B_diag y B_subdiag permitieron contar los
signos, incluidos bloques 2×2. Esa misma suite genérica corrió contra referencia
y faer y pasó con 1.85.1. La primera prueba vacía detectó un panic de faer en
autovalores de orden cero; se resolvió explícitamente antes de llamar a la
librería. Se verificó también compilación wasm con 1.85.1. No se ejecutó wasm
en Node ni se mide rendimiento, tamaño o determinismo cruzado.

Cargo.lock de la evaluación tenía 67 paquetes externos, incluyendo plataformas
no usadas. Metadata/tree filtrados dieron 54 para Linux x86_64 y 48 para wasm;
las expresiones declaradas permiten MIT, Apache-2.0, BSD-2-Clause, Zlib y
Unicode-3.0. Cargo-deny 0.20.2 aceptó ambos grafos: cero errores, cuatro avisos
por permisos de la lista no encontrados. No hay deny.toml en el repo, por lo
que se usó una config temporal en /tmp con MIT/Apache-2.0/BSD-2-Clause/
BSD-3-Clause/ISC/Zlib/Unicode-3.0/Unicode-DFS-2016. El primer check directo del
workspace falló por campos license ausentes en sus cuatro crates locales;
no se editaron metadatos ni texto legal. Se repitió con un proyecto temporal
privado y publish=false que depende solo de faer exactamente configurado.
Se compararon los conjuntos (nombre, versión, fuente, licencia) de todas las
dependencias externas de ambos proyectos para cada target: idénticos. Esa
comprobación aisló la política de terceros sin permitir licencias adicionales.
Otros targets del lock experimental no se auditaron; faltaba caché, por ejemplo
para crunchy, y no había red. No se atribuye un éxito global a cargo-deny.

La revisión de archivos COPYING detectó el aviso MPL-2.0 de Eigen, fuera de
la lista permitida; el README menciona un SVD portado. No se pudo aclarar el
alcance mediante documentos oficiales accesibles. **Faer queda PENDIENTE y no
se incorpora su dependencia ni adaptador funcional al grafo final**, usando la
alternativa de stub/no incorporación autorizada ante falta de red. El éxito de
cargo-deny sobre metadatos no se presenta como certificación de exclusividad
permisiva. Para reincorporarlo hay que resolver documentalmente el alcance de
ese aviso y repetir la configuración exacta y conformidad; no activar defaults
ni elevar MSRV como atajo. La feature final `faer=[]` documenta/reserva un stub
cuyo constructor falla expresamente con ENG-E-0009. No implementa LinAlgBackend
ni pasa silenciosamente al backend de referencia.

El MSRV del workspace sigue en **1.85**. El grafo final contiene solo los cuatro
crates propios, tanto con features por defecto como con todas las features;
Cargo.lock/tree finales no contienen dependencias externas ni faer/nalgebra/
sprs/rayon. Nalgebra PENDIENTE, sin dependencia ni feature. Sprs-ldl excluido.
ENG-LA-008 conserva solver disperso, AMD y mapa nodo/GDL; ENG-LA-009 conserva
solver modal disperso, reducción de GDL sin masa y control de modos faltantes.
La intercambiabilidad de dos implementaciones fue demostrada en la evaluación
temporal; la suite final ejecuta solo la referencia, pues no se distribuye el
adaptador cuya licencia no pudo cerrarse.

## Pruebas y tolerancias

La función `conformidad(&mut impl LinAlgBackend)` cubre:

- A·A⁻¹=I y solución densa no simétrica: tolerancia absoluta 1e-12 y residual
  euclídeo relativo por RHS estrictamente < 1e-12.
- Transposición y multiplicación rectangular con enteros exactos; ensamblaje CSR,
  suma de duplicados, patrón y solución tras expansión explícita.
- Resortes n=8, solución cerrada x_i=i(9−i)/2 para cargas 1 y 2 y RHS cero;
  comparación absoluta 1e-12 y residual relativo < 1e-12. RHS cero usa norma
  absoluta < 1e-12. Reutilización del factor con nuevas ocho RHS y metadatos
  constantes; dimensiones erróneas/overflow conservan la RHS completa.
- Inercia de diagonal conocida, [[0,1],[1,0]], congruencia LDLᵀ conocida y una
  permutación con bloque no inicial: conteos exactos, ratio en (0,1], residuales
  < 1e-12. Matrices singulares se rechazan con índice GDL, incluido un mecanismo
  [[1,−1],[−1,1]]. Escalados 1e-100 y 1e100 tienen residual < 1e-12.
- Toeplitz n=8 con M=I y M=2I: autovalores y productos φᵀMφ con tolerancia absoluta
  2e-12, residuales modales relativos < 1e-12. Masa no diagonal proporcional y
  no proporcional: valores, residual y ortogonalidad con 1e-12. K nula permite
  valores cero; orden vacío conserva las dimensiones.
- Negativos: no simétrica, no cuadrada, NaN e infinitos, desbordamientos, índices
  inválidos, dimensiones incompatibles, duplicados cuya suma desborda, masa
  singular y masa indefinida, incluida una masa invertible de diagonal cero.
- Almacenamiento disperso de orden 100000 con una sola entrada, sin materializar
  su máscara densa; validación de patrón y comparación independiente de caché.

La prueba de frontera recorre todos los Cargo.toml y .rs de crates/ (más el
manifest raíz), salvo engine-linalg. Detecta proveedores, alias de dependencias,
tablas de dependencias target/workspace, paths de Rust y extern crate; ignora
comentarios y cadenas. Incluye casos de control propios del scanner. Es una
inspección conservadora de tokens, no un parser completo de TOML/Rust; el layout
actual del workspace está bajo crates/. No se modificó ningún workflow.

## Verificación final

Comandos y resultados del estado entregado, con binarios de ~/.cargo/bin:

- `cargo +1.85.1 build --offline --locked -p engine-linalg`: satisfactorio.
- El mismo build con `--features faer`: satisfactorio **del stub**, sin faer real.
- `cargo +stable fmt --check`: satisfactorio.
- `cargo +stable clippy --offline --all-targets -- -D warnings`, y con
  `--features faer`: satisfactorio.
- `cargo +1.85.1 test --offline --locked`, y con `--features faer`:
  satisfactorio: 47 pruebas y 26 doctests sin features; 48 y 26 con el stub.
- `RUSTDOCFLAGS='-D warnings' cargo +1.85.1 doc --offline --no-deps`, y con
  `--features faer`: satisfactorio.
- `cargo tree --offline --locked --workspace --all-features`: solo crates locales.
- Build final con Rust 1.85.1 para wasm32-unknown-unknown y `--features faer`:
  satisfactorio del stub; no se ejecutaron tests en un runtime wasm.
- `bash clean-room/check.sh origin/main`: satisfactorio, incluyendo archivos
  nuevos mediante índice y objetos temporales en /tmp, sin modificar .git.
- `git diff --check`: satisfactorio; índice real sin cambios y rutas protegidas
  sin modificaciones.

Todos los resultados de faer real descritos arriba pertenecen exclusivamente
a la evaluación temporal descartada del grafo. No se modificaron clean-room/**,
.github, CODEOWNERS, docs/CLEAN_ROOM.md, archivos LICENSE/texto legal, ni el ADR.
No hay nuevas dependencias externas en la entrega, commits ni cambios en el
índice del usuario. Revisión humana, CI remota y ejecución wasm siguen pendientes.
