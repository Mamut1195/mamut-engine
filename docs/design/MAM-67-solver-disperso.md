# Diseño · MAM-67 · solver directo disperso simétrico

- Ticket: MAM-67, ENG-LA-008.
- Estado: propuesta de diseño. No hay código en este corte.
- Base leída: `9aeebcc000ca42c0f94b5aa41507b1da1e378743`.
- Ficha: [11-solver-disperso.md](../tickets/11-solver-disperso.md).
- Bitácora: [ENG-LA-008-DISENO.md](../../provenance/ENG-LA-008-DISENO.md).
- Dependencias: decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67. Opción A. Este texto no incorpora crates.

## 1. Para qué es este diseño

El motor ya ensambla K, M y Kg en CSR. La reducción a GDL libres y la factorización siguen siendo densas. Con unos miles de GDL libres la matriz densa deja de caber en el perfil V2b. Este diseño describe la ruta dispersa simétrica que tiene que sustituir esa reducción y esa factorización, sin implementarlas aquí.

El encargo de la sesión trae una medición del propio motor, en un modelo de 250 nudos 3D: 6,48 s en estático y 130 s en modal, con más del 90 % del tiempo en la factorización densa y en Jacobi. El informe de origen no se abrió en esta sesión: cita desconocida. Esas cifras motivan el trabajo. No son el criterio de aceptación y no se comparan con otro programa.

Después de esta ruta vendrán Lanczos, para el modal, y los tickets MAM-965 y MAM-995. El diseño deja un factor de `K − σM` que esas fases pueden usar. No las implementa.

## 2. Estado verificado en el commit de base

| Pieza | Dónde | Qué hace hoy |
| --- | --- | --- |
| Backend real | `crates/engine-linalg/src/reference.rs` | LDLᵀ denso, inercia y varias columnas. |
| Marcador faer | `crates/engine-linalg/src/faer_backend.rs` | `FaerBackend::new` devuelve ENG-E-0009. No implementa `LinAlgBackend`. |
| Feature | `crates/engine-linalg/Cargo.toml` | `faer = []`. No hay dependencia. |
| Registro | `Cargo.lock`, `vendor-crates/` | No contienen la crate `faer`. |
| Matriz simétrica | `crates/engine-linalg/src/matrix.rs` | `SymMatrix(DMat)` guarda las dos mitades y exige simetría exacta. |
| Dispersa general | `crates/engine-linalg/src/sparse.rs` | CSR con duplicados sumados en el orden de entrada, vía `BTreeMap`. `to_dense` existe y el comentario dice que no es el solver de ENG-LA-008. |
| Reducción | `reduced()` en `crates/engine-assembly/src/system.rs` | Aplica `Tᵀ A T` y termina en `to_dense` + `SymMatrix`. |
| Sistema | `LinearSystem` | K, M y Kg globales son `SparseMatrix`. Las reducidas `k_free`, `lumped_free`, `consistent_free` y `kg_free` son `SymMatrix` densas. |
| Mecanismo | `translate_solver_error` en el mismo archivo | ENG-E-0006 con índice reducido se reescribe con nudo y GDL global. |
| Inercia y Sturm | `crates/engine-linalg/src/modes.rs` | `sturm_count` forma `K − σM` densa y cuenta negativos. La excepción de `mul_add` ya está en `docs/DETERMINISM.md`. |
| Memoria | `crates/engine-linalg/src/budget.rs` | `MemoryBudget::charge` rechaza con ENG-E-0013. `MAX_ALLOC_BYTES` es 8 GiB en 64 bits y 1 GiB en 32 bits. |
| Contrato | `FactorInfo` en `crates/engine-linalg/src/lib.rs` | Ya tiene inercia, razón de pivote y `singular_dofs`. |

No hay ordenamiento por grado mínimo aproximado, ni factor disperso, ni inercia dispersa.

## 3. Estructuras de datos

### 3.1. Qué sigue denso, y por qué

La matriz de un elemento es pequeña: 6×6 en una barra plana, 12×12 en una barra espacial, y el bloque de un diafragma no se almacena como elemento denso global. Esos bloques siguen en `DMat`. El coste que hay que eliminar es el de la matriz reducida de todo el modelo y el de su factor.

`ReferenceBackend` y `SymMatrix` se quedan para los casos que ya pasan los hashes de `docs/DETERMINISM.md` y para el álgebra densa de elemento. No se borra esa ruta. La ruta dispersa es otra, elegida por el llamador. Si la dispersa rechaza, no llama a `to_dense` ni a `ReferenceBackend::factorize`.

### 3.2. Almacenamiento simétrico

Tipo nuevo del motor, dueño de sus datos, sin tipos de una crate:

`SimetricaDispersa`, CSC de la triangular inferior, diagonal incluida.

- `orden: usize`.
- `punteros: Vec<usize>` de longitud `orden + 1`.
- `filas: Vec<usize>`, índices de fila `i ≥ j`, crecientes dentro de la columna.
- `valores: Vec<Real>`, paralelos a `filas`.
- La diagonal de cada columna está presente. Si nadie aportó ese término, el valor es cero explícito. Hace falta para nombrar el pivote.
- Un cero explícito se conserva, igual que en el CSR actual.
- Solo se guarda una mitad. La otra no se materializa y no se compara byte a byte contra sí misma: la simetría es de construcción.
- Duplicados: suma en orden de entrada, con `BTreeMap` o con un acumulador que recorra claves ordenadas. Sin `HashMap`. Es la misma regla que `SparseMatrix::from_triplets`.

`nnz` de esta matriz cuenta las entradas almacenadas, diagonal incluida. El informe lo publica como entradas almacenadas de K, no como el recuento de las dos mitades.

La conversión desde el CSR global es un filtro: de cada entrada `(i, j, a)` se conserva la de `i ≥ j`. No es una factorización.

### 3.3. Mapa de GDL y restricciones

No se inventa otro mapa.

- `DofMap` numera por id de nudo creciente y, dentro del nudo, por el orden de `Dim`. El índice es global e incluye apoyos y esclavos.
- `Constraints::free_dofs` lista los GDL independientes libres. La posición en esa lista es el índice reducido.
- `free_transformation_row` da los pares `(índice reducido, coeficiente)` de `T_f`. Un coeficiente puede ser adimensional o un brazo en la unidad de longitud del sistema coherente.
- La matriz que se factoriza está en el orden reducido, no en el orden del ordenamiento interno.

La reducción dispersa de una matriz global `A`:

1. Recorre las entradas de `A` en el orden ya definido del CSR.
2. Para cada entrada y para cada par de coeficientes `(r, a)` y `(c, b)`, si `r ≥ c`, acumula `a * valor * b` en la clave `(r, c)`.
3. Suma los duplicados en ese orden de recorrido.
4. Construye `SimetricaDispersa`. No llama a `to_dense`.

Hoy `reduced()` acumula una sola mitad y luego copia la otra sobre la densa, para que el redondeo no rompa la simetría exacta de `SymMatrix`. En la triangular no hay segunda mitad que redondear. El orden de la suma queda fijo por el recorrido del CSR y por la clave ordenada.

K, M concentrada, M consistente y Kg usan el mismo procedimiento. El patrón estructural reducido es la unión de sus claves, construida con `SparsityPattern::from_coordinates`, que ya existe y no necesita la máscara densa. `SparsityPattern::new` y `try_entries`, que piden `orden²` booleanos, no se usan en esta ruta.

Los productores de Kg siguen evaluando el bloque pequeño del elemento. El bloque entra al CSR global y de ahí a la triangular reducida. No hay una Kg densa intermedia del modelo.

### 3.4. Qué deja de devolver el sistema

`SystemView::tangent`, `mass` y `geometric` escriben hoy un `&mut SymMatrix`. Llenar ese argumento en el modelo de 26 460 GDL libres reservaría la densa.

Una sola matriz densa de orden `n` ocupa `n² × 8` bytes:

- `n = 26 460`: `700 131 600` entradas y `5 601 052 800` bytes, más de 5 GiB.
- Tres reducidas, K, M y Kg, pasan de 15 GiB antes del factor.
- `n = 60 000`: `28 800 000 000` bytes, más de 26 GiB, por una sola matriz.

Esas cuentas son aritmética de este diseño. No son mediciones de un ensayo.

La ruta dispersa no implementa `SystemView` escribiendo una `SymMatrix`. Añade operaciones que prestan la `SimetricaDispersa` o que factorizan directo. La vista densa solo se ofrece cuando el llamador está en la ruta densa y `dense_bytes(n, n)` cabe en el presupuesto. Pedir la densa por encima de ese tamaño devuelve ENG-E-0013, que es el código que `MemoryBudget` ya usa, y no reserva.

Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: se retira el corte `n ≤ 2048`. El pórtico 3D de 250 nudos tiene unos 1 500 GDL libres y en la ruta densa tarda 6,48 s; ese corte no lo saca de la ruta densa y no mejora el tiempo. El umbral entre las dos rutas se fija midiendo en la fase 10. Conservar los hashes de `docs/DETERMINISM.md` no es criterio para fijarlo. Si cambian, se documentan y J4 los audita. Por qué unos 1 500 GDL tardan 6,48 s es la pregunta de MAM-1056, el perfil de la ruta densa. Este diseño no la responde. Si la ruta dispersa rechaza, el error se propaga. No hay segundo intento denso.

## 4. Ordenamiento, factorización e inercia

### 4.1. Ordenamiento

El ordenamiento por defecto es el grado mínimo aproximado, para reducir el relleno de un patrón simétrico general. La cita del método publicado es Patrick R. Amestoy, Timothy A. Davis e Iain S. Duff, «An Approximate Minimum Degree Ordering Algorithm», SIAM Journal on Matrix Analysis and Applications, 1996. El artículo no se abrió en esta sesión. Volumen, número y páginas: cita desconocida.

El procedimiento que implementemos se escribe desde esa cita cuando la sesión la tenga delante, o desde una derivación propia registrada antes del código. No se copia un listado. No se abre el código de AMD, CSparse, CHOLMOD ni de un derivado.

Contrato del ordenamiento, sin listado:

- Entrada: el patrón simétrico de la matriz reducida, no los valores.
- Salida: una permutación y su inversa, de longitud `n`.
- Empate de grado: gana el menor índice reducido original. Así el resultado no depende de la dirección de un hash ni de la hora.
- Sin hilos y sin entropía del sistema.
- La permutación se publica en el informe con el nombre `grado_minimo_aproximado`.

Alternativas que no son el defecto:

- Cuthill–McKee inverso. Cita: E. Cuthill y J. McKee, «Reducing the bandwidth of sparse symmetric matrices», proceedings de la ACM National Conference, 1969. Páginas: cita desconocida. El artículo no se abrió. Reduce ancho de banda y, en un pórtico mal numerado, puede dejar más relleno que el grado mínimo. Solo se reconsidera si un ensayo nuestro, sobre nuestras matrices, muestra menos entradas en L.
- Disección anidada. Útil en una malla regular. Un edificio con diafragmas no es esa malla, y las bibliotecas habituales de disección no son Rust puro ni entran en wasm. No se elige.
- El orden de `DofMap`, sin permutar. Es determinista y puede bastar en una cadena de barras. No es el defecto del perfil V2b.

Quedan fuera de candidatura, sin abrir su código: cualquier enlace a CHOLMOD, MUMPS, PARDISO o a la AMD de SuiteSparse, y la crate `amd` 0.2.2 (BSD-3-Clause, ficha de crates.io consultada el 2026-11-10). Esa ficha la describe como ordenamiento de grado mínimo aproximado y `faer` la declara como dependencia de desarrollo al lado de `suitesparse_sys`. No hace falta leerla para no elegirla.

### 4.2. Análisis simbólico

Sobre el patrón ya permutado:

- El árbol de eliminación: cada columna conoce a su padre. Cita del papel de ese árbol: Joseph W. H. Liu, «The Role of Elimination Trees in Sparse Factorization», SIAM Journal on Matrix Analysis and Applications, 1990. Páginas: cita desconocida. El artículo no se abrió.
- El conteo de entradas de cada columna de L, diagonal incluida, y la suma `nnz(L)`.
- Esos conteos existen antes de reservar los valores de L. La reserva numérica usa exactamente esos conteos. Si el conteo y la reserva discrepan, es ENG-E-0012, estado interno incoherente, y no se sigue.

El primer almacenamiento del simbólico es O(`n`) enteros: árbol, conteos y permutación. También ese O(`n`) se comprueba contra el presupuesto antes de `try_reserve`.

No se propone un factor supernodal en la primera implementación. Los núcleos por bloques suelen apoyarse en SIMD y en una contracción de productos que `docs/DETERMINISM.md` no admite. Si la fase 10 no cumple las metas, se reabren B y C con números medidos, como dice el apartado 11. Un núcleo por bloques sería otro diseño, no un rescate silencioso de esta ruta.

### 4.3. Factorización numérica

Dos modos, los dos sin sumar nada a la diagonal para «hacerla invertible»:

- Matriz definida positiva, la K de un estático bien apoyado y la masa del Cholesky interno: Cholesky `LLᵀ`, o LDLᵀ con D positivo. La raíz cuadrada pasa por `engine_core::math::sqrt`, la excepción IEEE ya escrita en `docs/DETERMINISM.md`.
- Matriz indefinida, `K + Kg` en P-Delta y `K − σM` en el conteo de Sturm: LDLᵀ. Un pivote 1×1 se acepta cuando su magnitud supera el umbral. Si no, se ensaya un bloque 2×2. Si los dos valores propios de ese bloque también caen bajo el umbral, la columna es singular.

El umbral es el que ya publica `LinAlgBackend` en `crates/engine-linalg/src/lib.rs`: después de escalar la matriz por su entrada de mayor magnitud, un pivote es numéricamente nulo si su magnitud es menor o igual que `64 ε max(n, 1)`. No es una tolerancia física nueva.

La regla de signos de un bloque 2×2 es la que `sturm_count` ya documenta en `crates/engine-linalg/src/modes.rs`:

- determinante negativo: un negativo;
- determinante positivo y traza negativa: dos negativos;
- determinante positivo y traza positiva: cero negativos;
- determinante nulo: el signo distinto de cero es el de la traza, y un bloque nulo cuenta dos ceros.

Esa regla es del propio motor. La ruta dispersa la repite. No se copia un listado de un libro. El nombre clásico del pivote 2×2 es el de J. R. Bunch y L. Kaufman, Mathematics of Computation, 1977. Páginas: cita desconocida. El artículo no se abrió.

`FactorInfo` sale siempre relleno:

- `inertia = Some((positivos, negativos, ceros))`. `None` no es un resultado válido de esta ruta. Si una implementación no puede publicarla, falla con ENG-E-0013, que es lo que `symmetric_inertia` ya hace cuando el backend no informa.
- `uses_2x2_pivots` es `Some(true)` o `Some(false)`, no `None`.
- `singular_dofs` va en el orden de la matriz que el llamador entregó, después de deshacer la permutación. Un bloque 2×2 singular informa sus dos índices reducidos.

No hay desplazamiento añadido dentro del factor. Un σ de Sturm lo elige el llamador y se aplica al formar `K − σM`, no para esconder un cero.

### 4.4. Mecanismo, con nudo y GDL

El factor no conoce nudos. Informa índices reducidos. `LinearSystem::translate_solver_error` ya hace el resto:

1. El índice reducido se busca en `free_dofs`.
2. `DofMap::label` devuelve nudo y componente.
3. El mensaje añade el nudo y el GDL, y las entidades quedan `Node` y `Dof` global.
4. Si el índice no existe, ENG-E-0012.
5. El contraste entre la diagonal elemental y la diagonal ensamblada puede reescribir ENG-E-0006 como ENG-E-0010, «mal condicionada, no es un mecanismo». Esa regla se queda en el ensamblaje. El factor no la duplica.

Un GDL libre sin rigidez, en un modelo mínimo derivado por nosotros, tiene que producir ENG-E-0006 con ese nudo y ese componente. El ensayo de la fase 7 cubre ese caso.

### 4.5. Varias cargas, un solo factor

`SymFactorization::solve_in_place` ya acepta una `DMat` de varias columnas y no modifica el factor. La ruta dispersa cumple el mismo contrato:

- El simbólico y el numérico ocurren una vez.
- Cada caso de carga, o cada bloque de columnas, es una sustitución hacia adelante, una escala por D y una sustitución hacia atrás.
- El orden de la suma dentro de una columna de L es el de filas crecientes, de izquierda a derecha, empezando en cero.
- Cero columnas es válido, como hoy.
- Un error no escribe una solución a medias.

### 4.6. Listo para Lanczos y para Sturm

`sturm_count` hoy reserva `K − σM` densa. La fase dispersa forma esa combinación sobre el patrón unión de K y M:

- Cada entrada almacenada usa la misma expresión ya excepcional, `(-sigma).mul_add(mi, ki)`, documentada en `docs/DETERMINISM.md`.
- Donde solo existe una de las dos matrices, la que falta vale cero.
- El resultado es otra `SimetricaDispersa`, o directamente su factor, sin densa.

Tipo de la fase 9, dueño del motor:

`FactorDesplazado`.

- Lo construye `factorizar_desplazado(k, m, sigma, limites)`.
- `inercia()` devuelve el trío de signos. El número de negativos es el conteo de Sturm, con la misma validación de masa definida positiva que `sturm_count` ya exige.
- `resolver(b)` sustituye contra el mismo factor, tantas veces como Lanczos lo pida. No refactoriza.
- Si el desplazamiento cae en un valor propio, la inercia incluye ceros y `resolver` devuelve ENG-E-0006. Consultar la inercia no exige que el factor sea invertible. Es el mismo corte que el backend denso: `symmetric_inertia` admite ceros y `factorize` para resolver no entrega un factor singular.
- El objeto se puede soltar. Soltarlo libera la carga de `MemoryBudget` que la construcción hubiera anotado.

Lanczos, cuando exista, solo necesita `resolver`. El conteo de modos que faltan solo necesita `inercia`. MAM-965 y MAM-995 no se diseñan aquí más allá de ese uso.

## 5. Memoria y rechazo

### 5.1. Orden de las comprobaciones

El llamador pasa un límite explícito: máximo de GDL libres y un `MemoryBudget`. La fachada del perfil V2b pasa 60 000 GDL. El presupuesto de bytes lo fija el llamador. Para el modelo de aceptación, el arnés de ensayo usa 2 GiB como tope de pico de proceso. Esa cifra no se esconde como constante mágica dentro del factor.

Orden, y cada paso ocurre antes de la reserva que le corresponde:

1. Si `n` supera el máximo de GDL, ENG-E-0013. No se reserva el árbol.
2. Si los enteros O(`n`) del simbólico no caben en el presupuesto, ENG-E-0013. No se reservan.
3. El simbólico calcula `nnz(L)` y la cota de bytes de abajo.
4. Si la cota supera lo que queda del presupuesto, ENG-E-0013. No se reservan los valores de L.
5. `try_reserve_exact` de esos valores. Si el asignador rechaza, ENG-E-0010, igual que `alloc_vec`, y no queda una estructura a medias.
6. Solo entonces se escriben números.

Una cota que se quede corta no puede acabar en una reserva ilimitada: la reserva pide exactamente el conteo simbólico, y ese conteo ya fue aceptado. Una cota que se pase rechaza un caso que quizá habría cabido. Es el fallo preferible. Se publica la cota y, en el ensayo, el pico medido, para ajustar la fórmula con evidencia nuestra.

### 5.2. Qué entra en la cota

Bytes de valores de L: `nnz(L) × 8`.

Bytes de índices de L: `nnz(L) × tamaño de usize` (8 en nativo, 4 en wasm32).

Punteros de columna, permutación, inversa, árbol y conteos: O(`n`) enteros, con la fórmula escrita en la implementación y ensayada en tamaños pequeños, al estilo de `factor_bytes`.

Trabajo numérico: un vector de longitud la columna más larga de L, más el panel de segundos miembros que el llamador pida en la misma llamada.

Siguen vivos, y la cota los suma si el sistema los retiene: la triangular de K, la de M y la de Kg. En el modelo de tres matrices no se cuenta solo el factor.

La cota no incluye la cabecera del asignador del sistema operativo. Por eso el criterio de 2 GiB es el pico del proceso, medido con el mismo tipo de instrumentación que `crates/engine-linalg/tests/heap_peak.rs`, y además el pico de `MemoryBudget`. Si el presupuesto contable pasa y el proceso supera 2 GiB, el criterio de MAM-67 no se cumple.

`nnz(L)` del modelo de 10 pisos es desconocido hasta el simbólico de ese modelo. No se inventa. El diafragma rígido acopla los GDL de planta de un piso: ese bloque puede dominar el relleno. El simbólico tiene que contarlo. Si la cota pasa de 2 GiB, el ensayo registra el rechazo y el ticket no se da por cumplido.

### 5.3. Código de error

Se reutiliza ENG-E-0013, resultado `unsupported`, salida 3. Es el código con el que `MemoryBudget::charge` ya rechaza un exceso de bytes. El mensaje nombra la ruta dispersa, `n`, el `nnz(L)` estimado, los bytes estimados y el límite. No dice que falte una capacidad genérica sin cifras.

ENG-E-0010 se reserva para un tamaño incoherente o para un `try_reserve` que falla después de haber aceptado la cota. ENG-E-0006 sigue siendo el mecanismo. ENG-E-0009 sigue siendo el marcador de `FaerBackend` y no se usa para «no cabe».

ENG-E-0013 hoy también significa otras cosas: inercia no disponible, máscara densa imposible, esclavo con apoyo, Kg estática sin productor. El mensaje tiene que distinguir el rechazo de capacidad por el texto, con `n`, la cota y el límite.

Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: se reutiliza ENG-E-0013. No hay ENG-E-0015. `docs/contracts/error-codes-1.json` permanece en 32 entradas. No se toca `catalog.rs`.

### 5.4. wasm32

El mismo código corre en wasm32. No hay otra factorización para el navegador.

El tope de una sola reserva sigue siendo `MAX_ALLOC_BYTES`, 1 GiB en 32 bits. El espacio de direcciones de wasm32 no alberga el pico de 2 GiB como una sola reserva. El criterio de 60 s y 2 GiB del modelo de 26 460 GDL libres es nativo. En wasm32 ese modelo se ejecuta solo si la cota cabe en el presupuesto y en el tope de 1 GiB; si no cabe, ENG-E-0013 antes de reservar. No se promete que quepa.

## 6. Determinismo

La política vigente es `docs/DETERMINISM.md`. Este diseño no la reescribe.

Lo que la ruta dispersa tiene que cumplir para poder aspirar a los mismos bits:

- Un hilo. Sin rayon, sin reducción repartida, sin `HashMap`.
- El empate del ordenamiento, el recorrido de columnas y la suma dentro de la columna, como en el apartado 4.
- IEEE-754 binary64, redondeo al par más cercano. Sin fast-math, sin `target-cpu=native`, sin elegir núcleo según la CPU.
- Sin SIMD explícito y sin activar simd128. Una autovectorización que no reasocie un reducción de `f64` es la misma tolerancia que el documento ya da al backend de referencia.
- Sin FMA implícito. La única `mul_add` de esta ruta es la de `K − σM`, la excepción ya escrita.
- `sqrt` por `engine_core::math::sqrt`.

Con eso, dos ejecuciones en el mismo binario tienen que coincidir bit a bit, incluidos los ceros con signo que la política ya compara en los casos escalares.

Entre nativo y wasm la igualdad de bits no se declara por anticipado. `docs/DETERMINISM.md` la exige como ensayo, no como consecuencia de desactivar rayon. La fase 10 añade un caso disperso pequeño a la comparación nativo/Node, en depuración y en release, y publica el hash. Si los bits difieren, se registran el campo y los valores, y la ruta no se da por cumplida. Una tolerancia numérica no sustituye esa comparación.

Conservar los dieciséis hashes publicados no decide el umbral entre rutas. Si una fase los cambia, el PR documenta el cambio y J4 lo audita.

## 7. Opciones de implementación

Ninguna opción de este apartado se incorpora en este PR. Los números salen de las fichas públicas consultadas el 2026-11-10. Donde no hubo `cargo tree`, el grafo se marca no verificado. No se abrió el código fuente de ninguna crate.

### 7.1. (A) Implementación propia, sin dependencias

| Pregunta | Respuesta |
| --- | --- |
| Licencia | La del motor. No hay SPDX nuevo. El `license` de los `Cargo.toml` sigue pendiente de Jonathan, como ya dice ADR-002. |
| Tamaño | El código que escribamos. Cero crates añadidas. |
| Crates transitivas | 0. Verificado: no se añade ninguna. |
| wasm | Sí, Rust puro, la misma política escalar que `ReferenceBackend`. |
| Inercia | Sí. La define este diseño y la llena `FactorInfo`. |
| Vendor y clean-room | No crece `vendor-crates/`. No entra código de terceros. El procedimiento se escribe desde la cita, cuando esté delante, o desde la derivación propia, y se contrasta con el LDLᵀ denso que ya es nuestro. |
| Determinismo | Compatible con el apartado 6. |
| Riesgo | El tiempo de 26 460 GDL libres no está medido. Un simbólico mal contado se cierra con el rechazo del apartado 5, no con una reserva abierta. Hay más código nuestro que revisar. |

### 7.2. (B) `faer`, vendorizada

Ficha de crates.io consultada el 2026-11-10. Última versión publicada en esa ficha: **0.24.4** (2026-06-24). SPDX del manifiesto: **MIT**. MSRV declarado en la ficha: 1.84.0. Es la versión que ADR-002 ya evaluó.

Dependencias normales obligatorias, leídas del endpoint de dependencias de `faer` 0.24.4: 12. Verificado en esa respuesta: `bytemuck`, `dyn-stack`, `equator`, `faer-traits`, `gemm`, `generativity`, `libm`, `nano-gemm`, `num-complex`, `num-traits`, `pulp`, `reborrow`.

La feature `std`, en la misma ficha, activa además `libm/arch` y, solo en `x86_64`, `private-gemm-x86`. `pulp` entra con la feature `x86-v3`. La feature `rayon` es del defecto y este diseño la dejaría apagada, como ADR-002.

Crates transitivas del grafo que Cargo resolvería: **no verificado**. Un recorrido manual de la API, sin unificar versiones en conflicto y sin `cargo tree`, contó 38 nombres al incluir `private-gemm-x86` y al omitir aristas con `target`. Ese 38 no es un número de aceptación. Entre los nombres apareció `pulp-wasm-simd-flag`. No se abrió su código. Basta el nombre, junto con `x86-v3` y `libm/arch`, para ver el conflicto con `docs/DETERMINISM.md`.

Documentación de API leída, sin seguir los enlaces al código:

- `faer::sparse::linalg::cholesky` documenta `LLᵀ`, `LDLᵀ` y `LBLᵀ`, análisis simbólico, árbol de eliminación, conteos de columna y los módulos simplicial y supernodal.
- `faer::sparse::linalg::amd::order` documenta el ordenamiento de grado mínimo aproximado.
- `SimplicialLdltRef` documenta `symbolic()` y `values()`. No documenta un método de inercia ni un acceso a D. El LDL denso `faer::linalg::solvers::Ldlt::D` sí devuelve la diagonal. La inercia dispersa no tiene, en esas páginas, un contrato utilizable.
- `LdltRegularization` documenta que los valores por debajo de épsilon, o con el signo indebido, se sustituyen por delta con el signo corregido. El ejemplo público pasa `LdltRegularization::default()`. El valor numérico de ese defecto no se comprobó, porque está en la implementación. La existencia del tipo ya choca con la regla de no alterar pivotes.

ADR-002 registra, dentro del paquete 0.24.4, los avisos `COPYING.EIGEN.MPL2`, `COPYING.LAPACK.BSD`, `COPYING.SUITE_SPARSE.AMD.BSD` y `COPYING.SUITE_SPARSE.COLAMD.BSD`. El manifiesto dice MIT. Esos archivos no se reabrieron en esta sesión. La condición de ADR-002 sigue en pie: Jonathan tiene que aceptar esos avisos antes de que `faer` sea el backend por defecto. Hasta entonces la feature queda apagada y `FaerBackend::new` sigue en ENG-E-0009.

| Pregunta | Respuesta |
| --- | --- |
| Licencia | MIT en el manifiesto. Avisos MPL-2.0 y BSD dentro del paquete, ya descritos en ADR-002. Aceptación de Jonathan: pendiente. |
| Tamaño | El paquete y el grafo de arriba. No medido en el árbol de `vendor-crates/` porque no se vendoriza aquí. |
| wasm | ADR-002 dice que compiló con `default-features = false` y `features = ["std", "sparse-linalg"]`. La ejecución sigue PENDING en ese ADR. Esta sesión no recompiló. |
| Inercia | No hay método disperso documentado en las páginas leídas. No se afirma que `values()` contenga D en posiciones conocidas. |
| Vendor y clean-room | Vendorizar trae al árbol el paquete cuyos avisos citan AMD de SuiteSparse, Eigen y LAPACK. Esta sesión no leyó ese código. El riesgo clean-room queda abierto hasta la revisión de Jonathan. |
| Determinismo | ADR-002 ya dice que con `faer` no se promete igualdad de bits entre nativo y wasm. `libm/arch`, `pulp` con `x86-v3` y el nombre `pulp-wasm-simd-flag` van en la misma dirección. |
| Riesgo | Licencia, inercia no contratada, regularización documentada, SIMD y un grafo transitivo sin `cargo tree`. |

### 7.3. (C) Otras crates de Rust puro

Se miraron fichas y, donde se indica, la documentación de API. No se abrió código.

| Crate y versión | SPDX | Qué ofrece en la ficha | Por qué no es la recomendación |
| --- | --- | --- | --- |
| `sparse-ldlt` 0.3.3, publicada el 2026-09-27, MSRV 1.75 | MIT | La descripción pública dice LDLᵀ disperso, sin dependencias de ejecución, con los signos de D. El endpoint de dependencias solo lista `criterion` como dependencia de desarrollo. Transitivas de ejecución: **0, verificado** en ese endpoint. | La misma descripción dice que no hay pivoteo y remite a un desplazamiento cuando el pivote es pequeño. No cubre el bloque 2×2 ni la inercia sin alterar D. Decenas de descargas en la ficha. wasm y la igualdad de bits: no ensayados. La descripción dice que sigue el método publicado por T. A. Davis, *Direct Methods for Sparse Linear Systems*, SIAM, 2006, y que no traduce el código de ese autor. El libro no se abrió. El código de la crate tampoco. La afirmación queda no verificada. |
| `sprs-ldl` 0.10.0 | LGPL-2.1 | LDL disperso sobre `sprs`, con dependencias opcionales hacia SuiteSparse. | Excluida por la licencia, como ya hizo ADR-002. No se abre. |
| `sprs` 0.11.5, la versión ya escrita en ADR-002 | MIT OR Apache-2.0 | Almacenamiento. El solver LGPL va en la otra crate. | No factoriza la ruta de este ticket. ADR-002 no la eligió como almacén principal. |
| `nalgebra-sparse` 0.12.0 | Apache-2.0 | Almacenamiento sobre `nalgebra` 0.35. La ficha no anuncia un LDL simétrico. | No es un solver. ADR-002 ya señaló el MSRV de `nalgebra` por encima de 1.85. Transitivas: no contadas, no verificado. |
| `scirs2-sparse` 0.6.5 | Apache-2.0 | La descripción pública lista métodos iterativos, Cholesky incompleto, Lanczos y shift-and-invert. El defecto activa paralelismo. | No es una factorización directa exacta con inercia de D. El Cholesky incompleto no garantiza el residual de 1e-12. El grafo de `scirs2-core` no se resolvió: no verificado. Puede volver a mirarse cuando exista Lanczos, en otro ticket. |
| `amd` 0.2.2 | BSD-3-Clause | Ordenamiento de grado mínimo aproximado. | Descartada en el apartado 4.1. Código no abierto. |

No apareció otra crate de Rust puro, en esta consulta del registro, que documente a la vez LDLᵀ disperso, bloques 2×2, inercia sin alterar pivotes, ausencia de C o Fortran, y un modo escalar compatible con `docs/DETERMINISM.md`.

### 7.4. Decisión

**Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: (A).** Implementación propia, en sala limpia, simplicial, sin crates nuevas.

Condición de salida de A, medida en la fase 10 sobre el pórtico 3D de 250 nudos: estático en 54 ms o menos, modal en 0,58 s o menos y P-Delta en 88 ms o menos. Si la fase 10 no llega a esas metas, o a las de 60 s y 2 GiB del modelo de 10 pisos, se reabren B y C con los números medidos por nosotros. No se cambia de crate dentro del mismo PR.

La fuente de esas tres metas es la página de J2 «Ruta para superar a OpenSees», https://cursor.com/canvas/store-2a8ecc30-58b0-4b46-8a05-3be45ddfeb2e/19df7ff1-4c3f-440d-af09-e4dfbeec48ec, con mediciones de rendimiento de Q3 en caja negra.

Con A aprobada, la pregunta de B sobre avisos de licencia e igualdad de bits ya no aplica. Esas preguntas vuelven solo si B se reabre.

Los motivos que llevaron a A siguen en el texto de arriba: inercia por contrato, cero crates transitivas (verificado por ausencia de dependencia nueva) y una ruta escribible dentro de `docs/DETERMINISM.md`.

## 8. Plan de validación

Los esperados numéricos de abajo son derivación propia de esta sesión. Donde el resultado es diádico, el ensayo puede comparar bits. Donde no lo es, el criterio es el residual. No se calibra contra otro programa.

Norma del residual relativo, para una sola columna: `‖Au − b‖_∞ / ‖b‖_∞`. Si `‖b‖_∞` es cero, `u` y el residual tienen que ser cero. Para varias columnas, la norma se aplica a cada columna. En los casos bien condicionados de este apartado el cociente tiene que ser menor que 1e-12. Un caso mal condicionado se informa; no se le exige ese cociente a la fuerza.

### 8.1. Casos analíticos

1. Definida, solución diádica. `A = [[2, 1], [1, 2]]`, `b = [3, 3]`. Entonces `u = [1, 1]`, porque `2·1 + 1 = 3`. Inercia `(2, 0, 0)`. Residual cero.
2. Segunda carga, mismo factor. `b = [1, 0]`. De `2x + y = 1` y `x + 2y = 0` sale `x = 2/3`, `y = −1/3`. El factor de la carga anterior no se reconstruye: el ensayo cuenta las llamadas numéricas y exige una sola.
3. Indefinida 2×2. `A = [[1, 2], [2, 1]]`. Determinante `1 − 4 = −3`, traza `2`. Inercia `(1, 1, 0)`. `b = [1, 0]` da `x = −1/3`, `y = 2/3`.
4. Mecanismo. Una matriz diagonal `[[1, 0], [0, 0]]` en el GDL reducido 1. ENG-E-0006 con ese índice. En el modelo de dos nudos y un único resorte, el traductor tiene que nombrar el nudo libre y la componente sin rigidez. El modelo se escribe en la fase 7, con `source` y esta derivación.
5. Sturm. `K = diag(1, 4, 9)`, `M = I`, `σ = 5`. `K − 5M = diag(−4, −1, 4)`. Inercia `(1, 2, 0)`. El conteo por debajo de 5 es 2. El mismo par, pasado por el `sturm_count` denso actual en un `n` pequeño, tiene que dar el mismo entero.
6. Varias columnas de un solo panel, con `b` de dos columnas tomadas de los casos 1 y 2. Una factorización.

Los voladizos y pórticos pequeños que el repositorio ya compara con una forma cerrada se vuelven a correr cuando la fachada pueda elegir la ruta dispersa en un modelo chico de ensayo. El esperado sigue siendo el de ese ensayo, con su `source`. No se fabrican cifras nuevas para que cierre.

### 8.2. Caso publicado ya presente en el repositorio

El P-Delta de `provenance/ENG-BENCH-002.md` cita AISC 360-16, comentario C2, y una forma cerrada derivada en esa nota. Cuando la ruta dispersa esté enchufada al análisis, ese ensayo se repite y se compara con los valores que la nota ya guarda. Este diseño no los copia otra vez. La página de Timoshenko y Gere que esa nota deja en PENDING sigue en PENDING.

No se añade una tabla publicada que esta sesión no tenga delante.

### 8.3. Ensayos de escala

Los tiempos y la memoria los mide el ensayo nuestro, en la fase 10, y se publican en la bitácora de esa fase. Este documento no los anticipa.

Qué se publica, en el mismo proceso release, después de un calentamiento, con tres repeticiones y la mediana:

- `n`, entradas almacenadas de K, entradas almacenadas de L, nombre del ordenamiento;
- tiempo de ensamblaje y reducción, del simbólico, del numérico y de una sustitución;
- pico de `MemoryBudget` y pico del proceso;
- commit, toolchain, objetivo, máquina y si el caso cupo.

Familias generadas por nosotros, antes del modelo grande, para ver que la cota crece como el simbólico predice:

- cadena de barras, `nnz` lineal en `n`;
- malla plana regular de vigas, de tamaño creciente, parando en cuanto la cota supere un presupuesto chico de ensayo.

El modelo de aceptación de memoria es el de 10 pisos con losas y 26 460 GDL libres. Su malla no está en este diseño y no se inventa aquí. La fase 10 lo construye o lo toma del modelo que el proyecto fije, y registra `n` real. Si ese `n` no es 26 460, el criterio no se da por medido.

El tope de ese ensayo es 60 s de reloj de pared, de la entrada al ensamblaje hasta los desplazamientos, y 2 GiB de pico de proceso. Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: esas dos cotas valen solo para MAM-67. No cierran MAM-107.

La misma fase 10 mide el pórtico 3D de 250 nudos, el de unos 1 500 GDL libres que en la ruta densa tarda 6,48 s en estático. El criterio de salida de la opción A en ese pórtico es: estático en 54 ms o menos, modal en 0,58 s o menos y P-Delta en 88 ms o menos. El perfil de por qué la ruta densa tarda 6,48 s es MAM-1056. Si la fase 10 no cumple las metas, se reabren B y C con los tiempos y la memoria medidos.

La fuente de esas tres metas es la página de J2 «Ruta para superar a OpenSees», https://cursor.com/canvas/store-2a8ecc30-58b0-4b46-8a05-3be45ddfeb2e/19df7ff1-4c3f-440d-af09-e4dfbeec48ec, con mediciones de rendimiento de Q3 en caja negra.

No se publican cifras de otro programa.

## 9. Unidades

### 9.1. Regla nueva

Jonathan fija dos sistemas de primera clase: SI y el sistema de Estados Unidos. La conversión es explícita en la frontera. El interior no supone SI.

Hoy el código no cumple esa regla. `crates/engine-core/src/units.rs` dice que el cálculo interno es SI y convierte con `to_si` y `from_si`. El catálogo tiene metro, centímetro, milímetro, newton y sus múltiplos, pascal y sus múltiplos, radián, grado y la unidad adimensional. No tiene pulgada, pie, kip, libra ni ksi. `SystemView` documenta metros, radianes, newtons y kilogramos.

Este diseño no cambia `units.rs`. Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: el solver trabaja en unidades coherentes y no convierte nada. La conversión vive en la frontera, en MAM-1043.

### 9.2. Dónde se convierte

La frontera es la fachada que recibe el documento: JSON, TOML y Python, antes de `DofMap`, del ensamblaje y de cualquier factorización.

1. El documento declara el sistema coherente del análisis. Los nombres de ese sistema y el par de unidades de Estados Unidos los decide MAM-1043, no este solver. No hay valor por omisión dentro del factor. Si falta la declaración, el error es de frontera, antes de factorizar. No se rellena «SI» en silencio.
2. Cada magnitud del documento nombra su unidad. La frontera convierte al sistema coherente elegido, con factores que viven solo en `engine-core::units`.
3. A partir de ahí, ensamblaje, elementos, masa, Kg y el solver ven `f64` de un solo sistema. `K u = f` es homogéneo. El solver no llama a `convert`, no lee símbolos y no multiplica por 0,0254 ni por un factor de fuerza.
4. Al escribir resultados, la misma frontera convierte de vuelta a la unidad pedida por el documento. El factor no se aplica dentro de la sustitución triangular.

Un sistema coherente tiene una longitud, una fuerza y un ángulo, con momento = fuerza × longitud y rigidez = fuerza / longitud. Mezclar pulgadas en un lado y metros en el otro, ya dentro del solver, está prohibido. Si el documento mezcla unidades, la mezcla se resuelve en el paso 2, no en la factorización.

### 9.3. Qué sistema coherente de Estados Unidos

Queda en MAM-1043. Este apartado solo recuerda el encaje, ya decidido, de que el solver no convierte:

- longitud: pulgada, o pie, una de las dos, declarada;
- fuerza: kip, o libra-fuerza, la que case con la longitud elegida;
- momento y tensión: los productos de ese par;
- el ángulo interno sigue en radianes en los dos sistemas, porque el diafragma acopla desplazamiento y giro con un brazo y un seno pequeño. El grado se convierte en la frontera, como ya hace `Unit::Degree`.

Los factores numéricos no se copian a este diseño como si ya estuvieran en el catálogo. El factor libra/newton que la ficha `docs/fichas/concrete.kent_park.md` ya usa se queda en esa ficha hasta que el ticket de unidades lo mueva a `units.rs`. El solver no lo lee.

Las constantes que no son adimensionales, como la gravedad de una conversión fuerza-masa, entran desde la frontera en la unidad coherente. No se escriben dentro del factor.

### 9.4. Qué hace el solver con todo esto

Nada. Opera en el sistema coherente que la frontera ya entregó. Un ensayo del solver puede usar números puros, sin unidad, o un modelo que la frontera de prueba ya convirtió. Un documento con unidades mezcladas no llega a `factorizar`.

Cuando el ticket de unidades cambie los comentarios que hoy dicen «metros» y «newtons», los cambiará en la frontera y en los textos de `SystemView`. La aritmética del factor no se toca por ese cambio.

## 10. Python

No se añade API en este corte. Cuando se publique, va entera en tipos, con nombres legibles. El cálculo sigue en Rust. Python no factoriza y no convierte unidades por su cuenta.

Decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67: el objeto opcional en `results/1` queda aprobado. El PR que actualice el esquema lo publica con la forma de abajo. Un lector viejo no tiene que romperse: el objeto es opcional y el major no cambia por añadirlo.

```python
from enum import Enum
from typing import Optional


class SistemaCoherente(Enum):
    """Sistema en el que la frontera entregó el modelo al solver."""

    SI = "si"
    ESTADOS_UNIDOS = "estados_unidos"


class OrdenamientoDisperso(Enum):
    """Permutación usada antes del análisis simbólico."""

    GRADO_MINIMO_APROXIMADO = "grado_minimo_aproximado"


class InerciaDeSylvester:
    """Signos de D. La suma de los tres conteos es el orden."""

    positivos: int
    negativos: int
    ceros: int

    def __init__(self, positivos: int, negativos: int, ceros: int) -> None:
        self.positivos = positivos
        self.negativos = negativos
        self.ceros = ceros


class TiemposDeFase:
    """Duración de cada fase, en nanosegundos de reloj de pared."""

    ensamblaje_y_reduccion: int
    analisis_simbolico: int
    factorizacion_numerica: int
    sustitucion: int

    def __init__(
        self,
        ensamblaje_y_reduccion: int,
        analisis_simbolico: int,
        factorizacion_numerica: int,
        sustitucion: int,
    ) -> None:
        self.ensamblaje_y_reduccion = ensamblaje_y_reduccion
        self.analisis_simbolico = analisis_simbolico
        self.factorizacion_numerica = factorizacion_numerica
        self.sustitucion = sustitucion


class InformeSolverDisperso:
    """Informe de una factorización dispersa ya terminada. No resuelve de nuevo."""

    sistema_coherente: SistemaCoherente
    grados_de_libertad_libres: int
    entradas_almacenadas_de_la_rigidez: int
    entradas_almacenadas_del_factor: int
    ordenamiento: OrdenamientoDisperso
    uso_de_pivotes_dos_por_dos: bool
    inercia: InerciaDeSylvester
    tiempos: TiemposDeFase
    pico_presupuesto_bytes: int
    pico_proceso_bytes: Optional[int]

    def __init__(
        self,
        sistema_coherente: SistemaCoherente,
        grados_de_libertad_libres: int,
        entradas_almacenadas_de_la_rigidez: int,
        entradas_almacenadas_del_factor: int,
        ordenamiento: OrdenamientoDisperso,
        uso_de_pivotes_dos_por_dos: bool,
        inercia: InerciaDeSylvester,
        tiempos: TiemposDeFase,
        pico_presupuesto_bytes: int,
        pico_proceso_bytes: Optional[int],
    ) -> None:
        self.sistema_coherente = sistema_coherente
        self.grados_de_libertad_libres = grados_de_libertad_libres
        self.entradas_almacenadas_de_la_rigidez = entradas_almacenadas_de_la_rigidez
        self.entradas_almacenadas_del_factor = entradas_almacenadas_del_factor
        self.ordenamiento = ordenamiento
        self.uso_de_pivotes_dos_por_dos = uso_de_pivotes_dos_por_dos
        self.inercia = inercia
        self.tiempos = tiempos
        self.pico_presupuesto_bytes = pico_presupuesto_bytes
        self.pico_proceso_bytes = pico_proceso_bytes
```

El rechazo de capacidad, cuando llegue a Python, es una `EngineError` con ENG-E-0013, el mensaje en español y los enteros `grados_de_libertad_libres`, `bytes_estimados` y `bytes_limite` como campos con tipo, no como texto que el llamador tenga que partir.

`run` sigue siendo la entrada. No se propone un solver paralelo en Python.

## 11. Decisiones

Estado: decidido por J0 el 2026-11-10 01:17 ECT, con delegación escrita de Jonathan a J0 (2026-11-10 01:13 ECT), registrada en el comentario de Linear MAM-67. Este PR no añade crates ni códigos de error.

1. Opción A aprobada: implementación propia en sala limpia, sin crates nuevas. Condición: el criterio de salida incluye las metas de velocidad en el pórtico 3D de 250 nudos, con estático en 54 ms o menos, modal en 0,58 s o menos y P-Delta en 88 ms o menos. Si la fase 10 no llega, se reabren B y C con números medidos.
   La fuente de esas tres metas es la página de J2 «Ruta para superar a OpenSees», https://cursor.com/canvas/store-2a8ecc30-58b0-4b46-8a05-3be45ddfeb2e/19df7ff1-4c3f-440d-af09-e4dfbeec48ec, con mediciones de rendimiento de Q3 en caja negra.
2. Con A, la pregunta de B sobre avisos de licencia e igualdad de bits ya no aplica.
3. Se reutiliza ENG-E-0013. No hay ENG-E-0015. El catálogo se queda en 32 entradas.
4. Unidades aprobadas: el solver trabaja en unidades coherentes y no convierte nada. La conversión vive en la frontera (MAM-1043).
5. Se retira el corte `n ≤ 2048`. El pórtico de 250 nudos tiene unos 1 500 GDL y en la ruta densa tarda 6,48 s, así que ese corte no mejora nada. El umbral se fija midiendo en la fase 10. Conservar los hashes publicados no es criterio para fijarlo: si cambian, se documenta y J4 lo audita. MAM-1056 es el perfil de la ruta densa: por qué 1 500 GDL tardan 6,48 s.
6. El objeto opcional en `results/1` queda aprobado.
7. Los límites de 60 s y 2 GiB valen solo para MAM-67 y no cierran MAM-107. Aprobado.

La fase 2 se lleva en MAM-1055, con su propio PR.

## 12. Fuera de este diseño

- No se implementa el solver, Lanczos, MAM-965 ni MAM-995.
- No se modifica `Cargo.toml`, `Cargo.lock`, `vendor-crates/`, `deny.toml` ni el catálogo de errores.
- No se abre código de AMD, CSparse, CHOLMOD, MUMPS, PARDISO ni de sus derivados, ni el de las crates candidatas.
