# engine-validation · ENG-VAL-001 parcial

Suite de **42 fixtures analíticos estáticos lineales**, derivados aquí en forma
cerrada. Todos usan `StaticLinear`, elementos reales `LinearTruss` o
`EulerFrame2D`, recuperación `engine_analysis::results::CaseResults` y la
combinación de LOAD-002. Las cuatro filas de superposición/combinación comparan
además con un caso de cargas combinadas resuelto de manera independiente.

Ejecutar desde la raíz:

```sh
CARGO_NET_OFFLINE=true cargo test -p engine-validation
```

Un único test recorre `tests/support/table.rs`. Los errores incluyen nombre,
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
[bitácora ENG-VAL-001](../../provenance/ENG-VAL-001.md). La suite completa V1,
incluidos modal, P-Delta y espectro, queda pendiente de ANL-003/004 y ASM-002.
