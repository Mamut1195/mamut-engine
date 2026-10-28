# ENG-PY-008 · Guías y notebooks Python

- Tarea: MAM-84, rama `j2/MAM-84-docs-python`.
- Autor: Codex, bajo instrucciones del usuario, 2026-10-28.
- Alcance: documentación, notebooks didácticos, ejecutor de notebooks y fragmento
  de changelog. Sin cambios de API Python, Rust o contratos.
- Fuentes utilizadas: README, módulos, stubs y ejemplos de `bindings/python`;
  registros ENG-PY-001 a ENG-PY-007 (incluido ENG-PY-002-FU); fixture propio
  `crates/engine-api/fixtures/input/valid/cantilever-linear-static.json`;
  pruebas `bindings/python/tests/test_sfc010.py` y
  `crates/engine-api/tests/sfc010.rs`; LICENSE, NOTICE, CONTRIBUTING.md,
  `docs/CLA.md`, ADR-001 y ADR-003 del repositorio. Consulta local de
  `version()` y `capabilities()` mediante la wheel desempaquetada disponible.
- Sin fuentes externas, libros, PDFs, código de terceros copiado ni resultados
  de otros motores. Trabajo sin red, sin commits y sin publicación.
- Revisión humana de esta entrega: **PENDIENTE**; revisor asignado:
  **DESCONOCIDO**. No se declara aprobación técnica ni legal.

## Entrega

La [guía de usuario](../bindings/python/docs/GUIA_USUARIO.md), enlazada desde
el README del binding, documenta instalación offline desde una wheel local,
inicio con el fixture real, contratos, versión, capacidades, errores, ejes,
unidades, resultados y licencia dual. El
[procedimiento comunitario](../bindings/python/docs/PROCEDIMIENTO_COMUNITARIO.md)
recorre prototipo Python con fuentes/fixtures y test reproducible, ticket de
método, implementación Rust clean-room y revisión humana. Python conserva
su función de transporte, experimentación y presentación, sin duplicar cálculo.

Los [tres notebooks](../bindings/python/notebooks/) buscan la raíz hacia arriba
desde el cwd. Cada uno es independiente y se guarda sin outputs, con
`execution_count=null`, IDs deterministas y metadatos mínimos de lenguaje/kernel,
sin rutas de máquina ni timestamps. El
[ejecutor](../bindings/python/notebooks/run_notebooks.py) usa nbclient y el
intérprete que lo invoca, con un kernel nuevo por notebook; no guarda resultados
de ejecución en el repo. Exige que fallen las celdas con error incluso si llevan
la etiqueta `raises-exception`.

## Expectativa analítica y límites

El fixture es un voladizo horizontal Euler–Bernoulli con empotramiento en i y
carga puntual en j. Integrar la curvatura con desplazamiento y giro nulos en i
da `Uy(L) = P·L³/(3·E·I)` usando P con signo. Cada notebook analítico extrae
del documento P=−12 N, L=4 m, E=200 Pa e I=3 m⁴; no introduce propiedades
físicas implícitas. Esta fórmula sirve como expectativa didáctica independiente,
no como un segundo método de producción Python.

Se exige `math.isclose(..., rel_tol=1e-9, abs_tol=0.0)`. La expectativa no es
cero y el margen cubre el redondeo binary64 del sistema pequeño; no se
extrapola a otros modelos ni constituye una tolerancia de aceptación de diseño.
Los parámetros son sintéticos y no verifican grandes desplazamientos.

Consulta local: motor 0.1.0, `engine-input/1` y `results/1` en `version()`.
El resultado añade `element/1` y `system/1`. El registro informa `linearStatic`,
`modal`, `responseSpectrum` y `combinations` como `Validated`, P-Delta como
`Stable`, y `nonlinearStatic`, `soilProfile`, `consolidation1d` como
`Experimental`, igual que `schemas/engine-input/1.json` (línea 5) y
`crates/engine-api/src/methods.rs`, fuente del criterio. P-Delta: Stable;
ENG-BENCH-002; Validated pendiente de ENG-VAL-006 (no experimental, no
validado). Resto de capacidades: validación independiente **DESCONOCIDO**.
Estas clasificaciones proceden del registro, no de los notebooks.

Geotecnia tiene capacidades y pruebas propias existentes, inspeccionadas en
los archivos citados; no se ejecutaron ni validaron aquí. La guía conserva
su estado experimental. Las verificaciones nuevas se limitan al voladizo,
su barrido de carga y la presentación de sus resultados. No acreditan todos
los métodos, casos, leyes, elementos ni combinaciones de entrada.

## Verificación local

Entorno disponible: Linux x64, CPython 3.13.5, wheel desempaquetada del motor
0.1.0, nbclient 0.11.0, nbformat 5.11.1, ipykernel 7.4.0, NumPy 2.5.4 y
pandas 3.1.0. El paquete se importó desde la wheel disponible mediante
`PYTHONPATH`. Para la corrida con kernel real se construyó una wheel local con
`maturin build --offline --locked --release` y se usó desempaquetada con
`PYTHONPATH`; no se instaló con pip (instalación limpia: **DESCONOCIDO**). El resultado declara
`git_commit=null`: la identidad del commit productor es **DESCONOCIDO**.

| Comprobación | Resultado y alcance |
| --- | --- |
| Ejemplos Python de la guía | CUMPLIDO; sus cuatro bloques Python se ejecutaron en orden sobre la wheel disponible |
| Notebook 01, celdas ejecutadas secuencialmente en memoria | CUMPLIDO; Uy Rust = −0.42666666666666664 m, forma cerrada = −0.4266666666666667 m; error relativo ≈ 1.30e−16 |
| Notebook 02, `experiments.sweep` | CUMPLIDO; P = −6, −12, −24 N, estados `ok`; forma cerrada y Uy/P constante pasan con `1e-9`; input original intacto |
| Notebook 03, listas + NumPy + pandas | CUMPLIDO; tabla (2,6), IDs, GDL, unidades, copia propia NumPy y snapshot conservado |
| Notebook 03 sin dependencias opcionales | CUMPLIDO con importaciones ausentes simuladas en procesos nuevos: solo NumPy, solo pandas y ambas; imprime aviso y termina |
| Búsqueda desde cwd | CUMPLIDO; celdas de 01/02 desde la raíz y de 03 desde el directorio de notebooks |
| `nbformat.validate`, sintaxis Python y limpieza de notebooks | CUMPLIDO; sin outputs/contadores ni metadatos de rutas/timestamps; archivos intactos después de ejecutar las celdas |
| Ejecutor sin nbclient | CUMPLIDO; invocación con `python -S` devuelve 2 e informa que falta nbclient |
| Ejecutor nbclient con kernel real | CUMPLIDO fuera del sandbox del autor: `run_notebooks.py` ejecutó los 3 notebooks con kernel ipykernel local y devolvió 0; con la forma cerrada mutada (3·E·I → 2·E·I) en 01 y en 02 devuelve 1 |
| Enlaces relativos, archivos legales y alcance de cambios | CUMPLIDO; CLA existente en `docs/CLA.md`, no en la raíz; API/Rust y archivos protegidos intactos |
| Fragmento de changelog y revisión de texto añadido | CUMPLIDO; una frase inglesa, nombre MAM-84.added.md, sin términos ni rutas de máquina prohibidos |
| Espacios y formato de diff | CUMPLIDO; `git diff --check` y revisión directa de los archivos nuevos |

El intento inicial con un manager síncrono no completó la ejecución y fue
interrumpido. El ejecutor entregado usa `AsyncKernelManager`, adecuado para
nbclient; dentro del sandbox del autor la invocación falla por la restricción de sockets; fuera de él pasó (ver tabla).
La ejecución directa de celdas comprueba su contenido, pero **no equivale a una
ejecución completa del ejecutor nbclient**. También se comprobó que IPC local
está restringido; no se eludieron las restricciones del entorno.

Para repetir el ejecutor desde la raíz, con las variables apuntando al contenido
de la wheel y al intérprete del entorno de notebooks ya provisionado:

```bash
PYTHONPATH="$WHEEL_CONTENTS" PYTHONDONTWRITEBYTECODE=1 \
  "$NOTEBOOK_PYTHON" bindings/python/notebooks/run_notebooks.py
```

La comprobación alternativa del contenido, sin kernel ni escritura de outputs,
se puede repetir desde la raíz con ese mismo entorno:

```bash
PYTHONPATH="$WHEEL_CONTENTS" PYTHONDONTWRITEBYTECODE=1 "$NOTEBOOK_PYTHON" - <<'PY'
import json
from pathlib import Path

for path in sorted(Path("bindings/python/notebooks").glob("*.ipynb")):
    namespace = {"__name__": "__main__"}
    notebook = json.loads(path.read_text(encoding="utf-8"))
    for cell in notebook["cells"]:
        if cell["cell_type"] == "code":
            exec(compile("".join(cell["source"]), path.name, "exec"), namespace)
    print("OK:", path.name)
PY
```

No se repitieron pruebas Rust ni de la API al tratarse de documentación y
notebooks sin cambios de implementación. Los checks de esta entrega no
sustituyen pruebas de compatibilidad de distribución.

## Autoría y ediciones manuales

Codex redactó el contenido inicial. Codex no pudo abrir sockets del kernel en su
sandbox, así que la fila «Ejecutor nbclient con kernel real» de la tabla de
verificación, la frase «fuera de él pasó» del párrafo posterior y el punto del
DoD sobre la ejecución completa con nbclient los escribió a mano el helper tras
ejecutar los notebooks fuera de ese sandbox. Revisión J4 r1 (M1, B1, B2): las
correcciones de esta segunda entrega también las escribió el helper.

## Límites de las pruebas de notebooks

- Ningún job de CI ejecuta los notebooks: **PENDIENTE**, decisión de Jonathan
  (`.github`).
- Quitar la única celda de aserción de 01 o de 02 no se detecta: el ejecutor
  solo informa excepciones.
- `run_notebooks.py` acumula los fallos y sale con 1 si alguno falla, aunque
  el bucle continúe; con 01 mutado (3·E·I → 2·E·I) devuelve 1.

## DoD: cumplidos y pendientes

- [x] Guía española, enlace desde README, inicio reproducible y advertencia
  visible «validación humana requerida».
- [x] Licencia dual AGPL-3.0-only + comercial sin prohibir usos comerciales
  permitidos por AGPL; MAMPRO SAS (ADR-001, 2026-10-15), contacto y enlaces
  legales existentes. Textos de licencia comercial y CLA PENDIENTES.
- [x] Procedimiento comunitario y responsabilidad numérica exclusiva de Rust.
- [x] Tres notebooks limpios, fixture real, fórmula desde los datos,
  comprobaciones analíticas y de linealidad, NumPy/pandas opcionales.
- [x] Ejecutor nbclient entregado, errores propagados, outputs solo en memoria.
- [x] Registro de procedencia, límites de evidencia y fragmento de changelog.
- [x] Ejecución completa con nbclient fuera del sandbox del autor (3/3 OK; la
  mutación de la forma cerrada hace fallar 01 y 02).
- [ ] **PENDIENTE: «gate usuario»**: prueba con un usuario real, de Jonathan;
  **NO cumplido**. Usuario, fecha y resultado: **DESCONOCIDO**.
- [ ] **PENDIENTE: smoke desde wheel publicado/candidato en entorno limpio con
  capturas**. La wheel desempaquetada disponible no cumple este gate.
- [ ] **PENDIENTE: publicación PyPI**, sin decidir. No hay paquete publicado
  según el alcance de esta tarea y no se realizó publicación.
- [ ] **PENDIENTE: revisión humana** de la entrega y revisión legal de los textos
  comercial/CLA, registro de firmas y controles descritos en `docs/CLA.md`.

Compatibilidad fuera del entorno descrito, revisión por un usuario real y
aprobaciones no documentadas: **DESCONOCIDO**. Esta entrega no declara V1
completa, autorización de distribución ni aceptación humana.
