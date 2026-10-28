# Guía de usuario de MAMUT Engine para Python

> **validación humana requerida** antes de usar resultados en una decisión de
> ingeniería. Ejecutar un ejemplo o recibir `status="ok"` no certifica un proyecto.

Python prepara documentos, llama al motor Rust y presenta sus resultados. No
implementa un segundo solver. Esta guía usa exclusivamente el
[voladizo del repositorio](../../../crates/engine-api/fixtures/input/valid/cantilever-linear-static.json).
Son datos sintéticos para comprobar flexión lineal; no representan un diseño real
ni una verificación de grandes desplazamientos.

## Instalación desde una wheel local

**No hay paquete en PyPI. Publicación: PENDIENTE, sin decidir.** No se debe
intentar instalar por nombre desde un índice. La construcción requiere Rust de
la toolchain del repo, maturin y dependencias ya disponibles localmente; no
descarga herramientas. Desde la raíz del repositorio:

```bash
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1 RUSTUP_AUTO_INSTALL=0
maturin build --offline --locked --release \
  --manifest-path bindings/python/Cargo.toml --out dist/python
python -m venv .venv
# Sustituye <wheel> por el nombre exacto del archivo recién construido.
.venv/bin/python -m pip install --no-index --no-deps 'dist/python/<wheel>.whl'
.venv/bin/python -c 'import mamut_engine as me; print(me.version())'
```

El consumidor de la wheel no necesita Rust. Estos comandos asumen un shell
POSIX y un destino compatible con la wheel construida. El procedimiento de dos
builds, hashes y artefactos de distribución está en el
[README del binding](../README.md#construcción-y-consumo-offline);
esta instalación básica no demuestra reproducibilidad ni portabilidad.

## Inicio rápido con el documento real

Ejecuta desde la raíz del repo o cualquier subdirectorio suyo, con la wheel
accesible al intérprete. La búsqueda ascendente evita depender de una ubicación
de máquina:

```python
from pathlib import Path
import mamut_engine as me

relative = Path("crates/engine-api/fixtures/input/valid/cantilever-linear-static.json")
cwd = Path.cwd().resolve()
root = next((p for p in (cwd, *cwd.parents)
             if (p / relative).is_file() and (p / "bindings/python").is_dir()), None)
if root is None:
    raise RuntimeError("Ejecuta dentro del repositorio para localizar el fixture")

model = me.Model.from_json((root / relative).read_text(encoding="utf-8"))
report = me.validate(model, lang="es")
print(report["diagnostics"])
assert not any(d["severity"] == "error" for d in report["diagnostics"])
results = me.run(model, lang="es")
document = results.to_dict()
case = next(c for c in document["cases"] if c["case"] == model.data["cases"][0]["name"])
tip_id = model.data["model"]["members"][0]["nodes"][1]
tip = next(n for n in case["nodes"] if n["id"] == tip_id)
print(results.status, tip["displacements"][1])  # Uy, m; aproximadamente -0.4266666667
```

`validate()` devuelve diagnósticos topológicos; no ejecuta el análisis ni
demuestra estabilidad. Los errores de argumento/contrato pueden lanzar una
excepción incluso antes del informe. `Model`, un diccionario `engine-input/1`
o un JSON `str` son entradas admitidas por `run` y `validate`.

## Versión, contratos y compatibilidad

```python
print(me.version())
print(me.schema("engine-input", 1))
print(me.schema("results/1"))
print(results.version)
```

En la ejecución local de esta tarea, `version()` informa
`engine_version="0.1.0"` y contratos `engine-input/1`, `results/1`.
Consulta siempre la versión instalada: una cadena de versión no sustituye la
comprobación del contrato ni de las capacidades. Los mayores desconocidos se
rechazan con `Unsupported` (`ENG-E-0009`). `schema()` publica también
`capabilities/1` y `method-descriptor/1`.

`Results.version` describe al productor del documento e incluye sus contratos
de dominio, como `element/1` y `system/1`; puede diferir de `version()`.
`__version__` es la cadena de versión del motor. Los DTO y anotaciones están en
`mamut_engine.types`, con `py.typed` y stubs incluidos.

Los metadatos declaran Python >=3.10 y ABI `abi3-py310`. La evidencia local
documentada en ENG-PY-007 y la ejecución de estos notebooks usan Linux x64 con
CPython 3.13.5. El tag ABI no acredita otros intérpretes, plataformas ni
arquitecturas: su validación es **DESCONOCIDO** en esta tarea. La matriz
candidata del README conserva sus casillas PENDIENTES.

## Capacidades y madurez

La fuente de disponibilidad y evidencia es el motor instalado:

```python
caps = me.capabilities()
for descriptor in caps["method_descriptors"]:
    print(descriptor["workflow"]["method_id"], descriptor["status"],
          descriptor["fixtures"], descriptor["provenance"])
```

La consulta local durante esta tarea informa:

| Método o postproceso | Estado del descriptor |
| --- | --- |
| `linearStatic` | `Validated` |
| `combinations` | `Validated` |
| `modal` | `Validated` |
| `responseSpectrum` | `Validated` |
| `pDelta` | `Stable` (Stable; ENG-BENCH-002; Validated pendiente de ENG-VAL-006) |
| `nonlinearStatic` | **Experimental** |
| `soilProfile` | **Experimental** |
| `consolidation1d` | **Experimental** |

**Fuente del criterio:** la descripción de `schemas/engine-input/1.json` (línea 5)
y el registro de `crates/engine-api/src/methods.rs` declaran `Experimental` solo
`nonlinearStatic`, `soilProfile` y `consolidation1d`, que exigen
`options.allow_experimental=true`. `pDelta` no es experimental, pero tampoco está
validado: Stable; ENG-BENCH-002; Validated pendiente de ENG-VAL-006. Elementos,
leyes y otras entradas sin estado de madurez propio: validación independiente
**DESCONOCIDO**.
`run: true` indica disponibilidad, no madurez. Una capacidad ausente no se debe
suponer disponible. La evidencia de un método se limita a los casos que cita;
no valida cualquier combinación de elementos, cargas o geometría. Los notebooks
comprueban únicamente el voladizo lineal y su barrido de carga. P-Delta no
necesita `allow_experimental`; los métodos experimentales que lo exigen requieren
`options.allow_experimental=true` y revisión de sus límites.

Geotecnia sí tiene capacidades `soilProfile` y `consolidation1d` y pruebas
existentes en [sfc010.rs](../../../crates/engine-api/tests/sfc010.rs), además de
[transporte Python](../tests/test_sfc010.py). Las pruebas Rust incluyen un perfil
hidrostático y límites analíticos de consolidación. Ambas capacidades siguen
siendo **experimentales**; estos notebooks no las ejecutan y no acreditan una
validación geotécnica general. Sus campos con sufijos `_kpa`, `_kn_m3`, `_m`,
`_s` declaran las unidades concretas del contrato; no se deben reinterpretar.

## Ejes, unidades y signos

La entrada exige `units: "SI"` y `model.axes: "Z-up"`: terna diestra, X-Y
horizontal y Z hacia arriba. D2 usa el plano XY, Z=0, y activa Ux, Uy, Rz;
no hay transformación implícita al plano XZ. El ejemplo carga en −Y, no en −Z.

En este ejemplo estructural se usan m, kg, s, N, N·m, Pa y rad: E en N/m²,
A en m² e I en m⁴. No hay conversión automática de unidades o signos. Los
vectores nodales de resultados tienen siempre seis posiciones
`[Ux, Uy, Uz, Rx, Ry, Rz]`; Uy ocupa 1 y Rz ocupa 5 aunque D2 solo active tres
GDL. Traslaciones en m, rotaciones en rad, reacciones en N y N·m. Los GDL
inactivos conservan ceros; consulta las identidades nodales para distinguirlos.
Rz positivo gira de X hacia Y.

Para el miembro horizontal del fixture, el eje local x va de i a j (+X).
Las acciones locales de extremo D2 siguen
`[Fx_i, Fy_i, Mz_i, Fx_j, Fy_j, Mz_j]`; no se deben confundir con los esfuerzos
de sección `stations[].forces_local` (`N,Vy,Vz,T,My,Mz`, cara positiva).

## Resultados y procedencia

`run()` devuelve `Results` solo para `ok` o `warning`; revisa siempre
`diagnostics`. `partial` distingue documentos fallidos o incompletos importados.
`to_json()` conserva los bytes de texto originales del motor y `to_dict()`
produce una copia independiente. El documento conserva casos, combinaciones,
modos, envolventes, convergencia, identidades y diagnósticos. En este voladizo
se leen desplazamientos y reacciones desde el caso por nombre y nodo por ID.

La procedencia incluye `engine_version`, `contracts`, `methods`, `target`,
`git_commit` e `input_sha256`. Un `git_commit` nulo significa **DESCONOCIDO**;
no se completa con un valor supuesto. La huella del input es RFC 8785/JCS
calculada por Rust, no el hash de `Model.to_json()`. Si cambia el modelo
vinculado, los accesos al resultado lanzan `StaleResultsError` (`PY-E-STALE`):
vuelve a ejecutar y conserva la entrada utilizada. Importar resultados sin
modelo conserva la procedencia declarada, sin autenticarla.

`to_arrays()` proporciona listas sin dependencias adicionales; `to_numpy()`
requiere NumPy y devuelve copias `float64` en orden C con memoria propia.
Cada bloque conserva `path`; cada tabla, `row_ids`, `dofs`, `units`, `shape`
y `frame`. No mezcles casos ni presupongas que un ID es un índice. pandas
es opcional para presentación y no forma parte de las dependencias del motor.

## Errores y explicación

```python
try:
    results = me.run(model, lang="es")
except me.EngineError as error:
    print(error.code, error.severity, error.entities, error.cause, error.details)
    print(error.diagnostics)
    try:
        print(me.explain(error.code, lang="es"))
    except me.InvalidInputError:
        print("Sin entrada en el catálogo Rust; consulta cause y details")
    # document conserva la salida fallida si existe; results permite inspeccionarla.
    # Ninguna de las dos propiedades garantiza una solución utilizable.
    raise
```

`EngineError` es la base Python. Conserva el código y los diagnósticos Rust;
`document` y `results` pueden ser `None`. Los errores de frontera añaden rutas
en `details`. `explain()` consulta el catálogo local; un código desconocido,
incluidos los códigos locales Python sin entrada Rust, produce `InvalidInputError`.

| Código | Interpretación / clase habitual |
| --- | --- |
| `API-E-INPUT` | Entrada inválida, `InvalidInputError` |
| `ENG-E-0009` | Contrato incompatible, `Unsupported` |
| `ENG-E-0013` | Método/operación no soportado, `Unsupported` |
| `ENG-E-0006` | Inestabilidad/singularidad, `Unstable` |
| `ENG-E-0007` | No convergencia, `NotConverged` |
| `API-I-CANCELLED` | Cancelación cooperativa, `Cancelled` / `CancelledError` |
| `ENG-E-0012` | Fallo interno contenido, `InternalEngineError` |
| `PY-E-STALE` | Cambió el input vinculado, `StaleResultsError` |
| `PY-E-NUMPY` | Falta NumPy para la exportación, `MissingNumPyError` |

`AnalysisError` agrupa fallos de análisis, incluidas `Unstable` y
`NotConverged`. No conviertas una excepción o un resultado parcial en éxito.

## Notebooks reproducibles

1. [Modelo 2D básico](../notebooks/01_modelo_2d_basico.ipynb): ejecuta Rust y
   compara Uy con `P·L³/(3·E·I)` usando los parámetros del propio fixture.
2. [Barrido paramétrico](../notebooks/02_barrido_parametrico.ipynb): usa
   `mamut_engine.experiments.sweep`, verifica linealidad y la forma cerrada.
3. [Resultados, NumPy y pandas](../notebooks/03_resultados_numpy_pandas.ipynb):
   conserva IDs, GDL y unidades; si falta alguna biblioteca opcional lo indica
   y continúa con listas.

La comparación usa P con signo (−12 N en el original), L=4 m, E=200 Pa,
I=3 m⁴. Para P negativa, Uy es negativa. La tolerancia relativa `1e-9`, sin
tolerancia absoluta, deja margen al redondeo binary64 de este pequeño sistema;
no es un umbral de aceptación de proyectos ni se extrapola a otros modelos.

Con `nbclient`, `nbformat` e `ipykernel` ya provisionados en el mismo entorno
Python que consume la wheel, desde la raíz:

```bash
python bindings/python/notebooks/run_notebooks.py
```

El ejecutor usa ese intérprete, ejecuta cada notebook en un kernel nuevo y
termina con error si falla una celda. No instala dependencias ni guarda outputs
en el repo. NumPy y pandas tampoco se instalan automáticamente.

## Licencia y contribuciones

Modelo dual **AGPL-3.0-only + licencia comercial**. La AGPL permite usos
comerciales cumpliendo sus términos; no impone una condición de uso no
comercial. Consulta [LICENSE](../../../LICENSE) y [NOTICE](../../../NOTICE).
Titular: **MAMPRO SAS**, confirmado en
[ADR-001, 2026-10-15](../../../docs/adr/ADR-001-fundacionales.md).
Contacto: **mampro.proyectos@gmail.com**.

El texto de la licencia comercial y el texto del CLA están **PENDIENTES** de
redacción/revisión legal; esta guía no los sustituye ni concede términos
comerciales nuevos. El documento de proceso [CLA.md](../../../docs/CLA.md)
existe en `docs/`, y la regla de contribución está en
[CONTRIBUTING.md](../../../CONTRIBUTING.md). El registro de firmas y su control
automatizado siguen PENDIENTES según ese documento.

Para proponer métodos, sigue el
[procedimiento comunitario](PROCEDIMIENTO_COMUNITARIO.md). La prueba con un
usuario real de Jonathan («gate usuario»), el smoke de una wheel
publicada/candidata en entorno limpio con capturas y la publicación PyPI
siguen **PENDIENTES**; la evidencia local está en
[ENG-PY-008](../../../provenance/ENG-PY-008.md).
