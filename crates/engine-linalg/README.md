# engine-linalg · ENG-LA-007

Tipos matriciales definitivos y frontera numérica interna. `engine-contracts`
reexporta `DMat`, `SymMatrix` y `SparsityPattern` con sus firmas anteriores.
Los tipos no contienen identificadores de una librería externa. El crate solo
depende de `engine-core` y de la biblioteca estándar.

`LinAlgBackend` ofrece multiplicación/transposición, solución densa general,
ensamblaje por tripletas, factorización simétrica reutilizable y todos los
modos de un problema generalizado denso. `SymFactorization` posee sus factores;
`solve` y `solve_in_place` aceptan matrices con una RHS por columna y no vuelven
a factorizar. Una escritura fallida conserva la RHS original.

Las matrices densas guardan datos finitos fila-mayor, incluyendo dimensiones
vacías; las simétricas exigen cuadratura y simetría exacta. No se introduce una
tolerancia física para aceptar entradas asimétricas. `SparseMatrix` guarda CSR,
suma duplicados en orden de entrada y conserva ceros estructurales. Cada tripleta
representa una entrada; el llamador aporta ambas mitades si requiere simetría.
`SparsityPattern` guarda coordenadas ordenadas, con máscara compatible bajo
demanda. La llamada heredada `entries()` materializa O(n²) memoria; el código
nuevo debe usar `coordinates()`. El orden debe permitir n² representable en
`usize` para conservar esa API. `to_dense()` también se destina a problemas
pequeños: el ensamblaje disperso no hace una conversión densa implícita.

## Backend de referencia

`ReferenceBackend` es el segundo backend mínimo de referencia/pruebas autorizado
para este ticket. Usa eliminación con pivote de fila para sistemas generales,
LDLᵀ con pivotes simétricos escalares o bloques 2×2 para sistemas simétricos,
Cholesky de masa y rotaciones Jacobi para autovalores. No pretende reemplazar
el backend principal productivo. La eliminación usa O(n³) trabajo y O(n²)
memoria; la búsqueda del máximo fuera de diagonal en cada rotación Jacobi puede
costar O(n⁴) en el límite de iteraciones. No hay AMD ni paralelismo.

La inercia se obtiene por congruencia de D, contando los signos de cada pivote
y los dos autovalores de cada bloque. El indicador `min_pivot_ratio` usa el menor
módulo de esos valores dividido por el mayor. No estima el número de condición.
Pivotes ≤ 64 ε max(n,1), tras normalizar por la mayor entrada de la matriz, se
tratan como singularidad numérica. Se devuelve ENG-E-0006 con índice global
GDL del pivote; identificar el nodo o el movimiento del mecanismo requiere el
mapa del ensamblador, pendiente de ENG-LA-008. Un factor exitoso tiene
`singular_dofs` vacío; los factores singulares no se entregan para resolver.

El problema modal exige masa definida positiva. Masa singular devuelve ENG-E-0006
sin generar NaN; masa indefinida ENG-E-0010. No se eliminan GDL sin masa ni se
regulariza. Los autovalores son ascendentes; los vectores columna se normalizan
con φᵀMφ=1 y signo de su componente dominante positivo. Una multiplicidad admite
bases diferentes del mismo subespacio. Jacobi para cuando el máximo fuera de
diagonal de la matriz escalada es ≤ 4 ε max(n,1), con hasta 100 n² rotaciones;
agotar ese límite devuelve ENG-E-0007.

## Decisión sobre faer y pendientes

**Faer PENDIENTE; la feature `faer` solo reserva un stub.** El constructor
`FaerBackend::new()` devuelve ENG-E-0009 con el motivo. No implementa el trait
numérico ni redirige operaciones al backend de referencia.

Se intentó faer **0.24.4**, fijado exactamente, optional, sin features por defecto,
con `std` y `sparse-linalg`, sin `rayon`. Crates.io falló por DNS, pero la caché
permitió evaluar un adaptador real: compiló y pasó la misma función genérica de
conformidad que la referencia con Rust **1.85.1** en nativo. También compiló para
`wasm32-unknown-unknown` con ese compilador. Su grafo nativo de 54 crates y el
wasm de 48 fueron aceptados por cargo-deny, según metadatos SPDX, con config
permisiva temporal fuera del repo.

La inspección de documentos del paquete encontró `COPYING.EIGEN.MPL2` y un README
que menciona un algoritmo SVD portado. El manifest declara MIT. No se pudo
aclarar el alcance del aviso MPL-2.0 mediante documentación oficial accesible;
MPL-2.0 está fuera de la lista permitida. Por ello no se certifica el paquete
como exclusivamente permisivo ni se incorpora su dependencia al grafo final.
Esta decisión usa la alternativa PENDIENTE autorizada por falta de red; no se
ha elevado el MSRV ni se ha cambiado el ADR. Ver la bitácora del ticket para
los resultados de la evaluación y sus límites. La prueba exitosa de compilación
de la feature final verifica el stub, no un adaptador faer funcional.

La reincorporación requiere resolver ese alcance documental y volver a probar
la dependencia optional con la configuración exacta anterior y la misma suite.
El solver directo disperso con AMD y separación simbólica/numérica sigue en
ENG-LA-008. Autovalores dispersos, eliminación de GDL sin masa, controles de
Sturm y modos faltantes quedan en ENG-LA-009. **Nalgebra PENDIENTE**, sin dependencia
ni feature en esta entrega. Sprs-ldl permanece excluido.

Se usan `f64::sqrt` y `hypot`; `cos` solo calcula el esperado analítico en tests.
No se añadió libm: no es necesario para las pruebas nativas, y añadirlo como
dependencia directa excedería la restricción de este ticket. La política de
trascendentes y la igualdad bit a bit nativo/wasm siguen en ENG-CORE-010.

## Verificación reproducible

```sh
~/.cargo/bin/cargo +1.85.1 build --offline --locked -p engine-linalg
~/.cargo/bin/cargo +1.85.1 build --offline --locked -p engine-linalg --features faer
~/.cargo/bin/cargo +stable fmt --check
~/.cargo/bin/cargo +stable clippy --offline --all-targets -- -D warnings
~/.cargo/bin/cargo +stable clippy --offline --all-targets --features faer -- -D warnings
~/.cargo/bin/cargo +1.85.1 test --offline --locked
~/.cargo/bin/cargo +1.85.1 test --offline --locked --features faer
RUSTDOCFLAGS='-D warnings' ~/.cargo/bin/cargo +1.85.1 doc --offline --no-deps
RUSTDOCFLAGS='-D warnings' ~/.cargo/bin/cargo +1.85.1 doc --offline --no-deps --features faer
```

La suite genérica recibe `&mut impl LinAlgBackend`; solo corre contra la
referencia en el grafo final. La evaluación temporal corrió contra ambos. Los
esperados son identidades y soluciones cerradas: residual euclídeo relativo
< 1e-12 para cada RHS no nula, inversa con tolerancia absoluta 1e-12, autovalores
Toeplitz y ortonormalidad a masa con 2e-12. Ver derivación propia de la fórmula
clásica Toeplitz en `provenance/ENG-LA-007.md`; cita bibliográfica precisa
PENDIENTE. La prueba de integración de frontera escanea Cargo.toml y archivos
Rust de los crates del workspace, detectando proveedores directos y aliases,
sin crear ni modificar workflows.
