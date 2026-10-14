# engine-validation · ENG-VAL-001 parcial

Suite de **73 fixtures analíticos propios**: los **42 estáticos lineales**
existentes permanecen intactos y se añaden **31** (12 modales, 10 P-Delta,
9 espectrales). Los lineales usan `StaticLinear`, elementos reales `LinearTruss` o
`EulerFrame2D`, recuperación `engine_analysis::results::CaseResults` y la
combinación de LOAD-002. Las cuatro filas de superposición/combinación comparan
además con un caso de cargas combinadas resuelto de manera independiente.

Ejecutar desde la raíz:

```sh
CARGO_NET_OFFLINE=true cargo test -p engine-validation
```

El test lineal recorre `tests/support/table.rs`; tres tests, uno por grupo,
recorren `tests/support/dynamic_table.rs` desde `tests/dynamic.rs`.
Los errores incluyen nombre,
campo, valor analítico, tolerancia, `source` y `citation`. Cada fila representa
un problema completo; no se cuentan sus comprobaciones como fixtures distintos.
La biblioteca es documental, sin dependencias de producción. Todas las
`dev-dependencies` son internas por `path`; `publish = false`.

La tabla contiene geometría, material, acciones, factores y tolerancia. Las
magnitudes SI artificiales permiten comprobar álgebra lineal; no representan
un diseño físico de pequeños desplazamientos. Se usa interpolación nodal exacta
situando nodos donde se comprueban flechas; las cargas puntuales interiores se
aplican en esos nodos. Los diagramas se recuperan en estaciones y raíces
críticas exactas del elemento, sin interpolar momentos.

No hay E/S, red, reloj, procesos ni hilos en el ejecutor. El código de pruebas
se comprueba también con `cargo check --workspace --all-targets --target
wasm32-unknown-unknown`; las pruebas se ejecutan en nativo. Compilar para wasm
no implica haber ejecutado un runtime wasm.

Derivaciones, convenciones, tolerancias y estado de revisión:
[bitácora ENG-VAL-001](../../provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden).
La ampliación usa `Modal`, `PDelta` y `Spectrum` de la cadena apilada. Las
frecuencias de vigas se contrastan con soluciones continuas y refinamiento;
el P-Delta distingue la aproximación de amplificación del contraste continuo
exacto. Las tolerancias de cada fila explican esa diferencia.

ENG-VAL-001 sigue parcial: ejemplos publicados con cita exacta **PENDIENTES**
(no verificables sin acceso, fuera del alcance de J2), revisión clean-room
**PENDIENTE (J4)** y ejecución en un runtime wasm **PENDIENTE**. La cobertura
analítica de modal, P-Delta y espectro y el umbral de ≥30 fixtures están cubiertos.
