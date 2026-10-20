# ADR-004: tablas de resultados y NumPy opcional en Python

- **Ticket:** MAM-82 · ENG-PY-004.
- **Estado:** decisión implementada; revisión clean-room **PENDIENTE (J4)**.
- **Fecha:** 2026-10-20 (America/Guayaquil).
- **Antecedentes:** [ADR-003](ADR-003-python-v1.md),
  [ENG-PY-003](../../provenance/ENG-PY-003.md).

## Contexto

`results/1` transporta listas JSON producidas exclusivamente por Rust. Los
notebooks necesitan matrices con identidad y unidades explícitas, sin depender
de la vida de buffers del backend. Los modelos pueden mezclar elementos con
distinto número de GDL; las identidades del documento no necesariamente tienen
el mismo orden que las filas de respuesta. El estado fallido puede carecer de
respuestas físicas y conservar diagnósticos y convergencia útiles.

## Decisión

1. `Results.to_arrays()` devuelve `ArrayResults` con listas independientes y
   metadatos; `Results.to_numpy()` devuelve el mismo contrato con `ndarray`.
   Ambos verifican stale antes de acceder al snapshot. `ArrayResults.document`
   es una copia completa de `results/1`, conservando estado textual, huella,
   identidades, GDL activos, conectividad, diagnósticos y campos no tabulados.
   `status` y `partial` mantienen la semántica de `Results`. No se transforma
   el estado en una magnitud f64 ni se presenta un fallo como éxito.
2. NumPy es el extra `mamut-engine[numpy]` (`numpy>=1.23`). Importarlo solo
   ocurre dentro de `to_numpy`; importar el paquete, resolver anotaciones,
   ejecutar análisis y exportar listas funciona sin NumPy. Su ausencia lanza
   `MissingNumPyError`, subclase de `EngineError`, código `PY-E-NUMPY`, con el
   nombre del extra. Un error interno de instalación de NumPy no se disfraza
   de dependencia ausente. El tipo `values: Any` permite resolver las
   anotaciones públicas sin importar la dependencia opcional.
   NumPy se distribuye bajo licencia BSD-3-Clause, compatible con AGPL-3.0 y
   con la licencia comercial, y no se vendoriza.
3. Cada `ResultMatrix` publica `values`, `shape`, `row_ids`, `row_indices`,
   `dofs`, `units` y `frame`. IDs e índices se mantienen como tuplas de enteros
   Python exactos; nunca se convierten a floats. Las etiquetas y unidades son
   tuplas de strings. Solo los valores numéricos son matrices f64.
4. La conversión es `numpy.asarray(listas, dtype=numpy.float64, order="C")`,
   reshape al shape explícito y `.copy(order="C")`. Incluso las matrices
   vacías tienen dos dimensiones, memoria propia (`OWNDATA`, `base is None`)
   y orden C. Las salidas son mutables, independientes entre sí, del JSON y
   del backend. Sobreviven a la destrucción de Results o cambios posteriores
   del modelo; esos cambios bloquean nuevas exportaciones mediante stale.
   Las listas también se copian. No hay vistas cero-copia ni préstamos de
   buffers mutables del motor. Los metadatos contenedores son copias locales;
   modificar un export no modifica el snapshot ni otro export.
5. Las tablas siguen las respuestas sin reordenarlas, interpolarlas,
   completarlas ni calcular fuerzas. Las fuerzas de extremo se agrupan por
   tipo de elemento para evitar matrices ragged o rellenos inventados.
   Un tipo desconocido requiere nuevas etiquetas explícitas y da `Unsupported`
   (`ENG-E-0013`); un shape o identidad incompatible da `InvalidInputError`.
   El JSON original sigue disponible. El contrato detallado está en el
   [README del binding](../../bindings/python/README.md#tablas-y-numpy-opcional).
6. No se añaden dependencias Rust ni se cambian Cargo.toml/Cargo.lock.
   El transporte JSON/listas existente basta; añadir rust-numpy duplicaría
   infraestructura y complicaría ownership sin evidencia de necesidad.
   Python solo valida estructura y convierte representación; Rust conserva
   toda la física y la aritmética del análisis.
7. No se añade liberación del GIL. La proyección Python/NumPy requiere objetos
   Python vivos. Se conserva el `py.detach` previo exclusivamente alrededor de
   llamadas Rust: captura `&str` inmutable cuya vida abarca la llamada, un Arc
   de token atómico y acumulación Rust de eventos; no accede a objetos Python
   ni ejecuta callbacks Python durante el detach. La conversión de salida y
   callbacks se ejecutan tras readquirir el GIL.

## Consecuencias y comprobación

Las copias consumen memoria y tiempo proporcionales a la salida. `document`
conserva además el snapshot JSON con listas; no se promete mínima memoria ni
aceleración del solve. Solo los campos tabulados se convierten a ndarray;
prescritos nullable, extremos, participación modal y convergencia permanecen
en `document`, evitando NaN como sustituto de null. Los ceros de componentes
ausentes y reacciones modales son los placeholders originales del motor.

Las pruebas instaladas comparan con referencias Rust independientes, rechazan
NaN/Inf, shapes e identidades incorrectos, verifican copias/orden C/dtype,
vacíos y funcionamiento sin NumPy. Las pruebas NumPy usan
`pytest.importorskip("numpy")`; la API básica solo necesita stdlib.
La wheel se construye dos veces en targets separados y se exige igualdad de
SHA-256 mediante el verificador existente. No se publica en PyPI.

El [plan de rendimiento](../PYTHON_BOUNDARY_BENCHMARKS.md) separa solve,
transporte, parseo y copias, y conserva muestras sin umbrales de CI.
