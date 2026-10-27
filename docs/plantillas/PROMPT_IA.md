# Prompt for implementing a method with restricted sources

Replace angle-bracketed fields before starting a clean session.
Do not attach code, data or reports from prohibited sources.

```text
Implementa <método> para <ticket> en Rust, dentro del worktree actual.
Lee docs/CLEAN_ROOM.md y sigue docs/METHODS_GUIDE.md.
Contrato vigente: <trait y versión>. Ficha: <ruta relativa>.
Fuentes autorizadas: SOLO los contratos locales, la ficha y las fuentes
permitidas de docs/CLEAN_ROOM.md §4 disponibles localmente y citadas en ella.
Puedes usar una derivación propia, escrita y registrada antes del código.
No consultes fuentes externas ni uses red, web, buscadores, repos hermanos,
foros, descargas o documentación en línea. Cargo siempre --offline;
PIP_NO_INDEX=1; usa el CARGO_TARGET_DIR autorizado por el entorno.

PROHIBIDO consultar, abrir, buscar, citar como especificación, recordar,
reproducir o usar código, resultados, tablas, fixtures, informes, manuales o
salidas de OpenSees, Stabileo, CalculiX, code_aster, LAPACK y TODOS sus
derivados, forks y envoltorios. Esto incluye su uso indirecto para escoger
casos, tolerancias o parámetros. No ejecutes esos programas como oráculos.
Se aplican además todas las fuentes/rutas prohibidas de CLEAN_ROOM.md §5
y los nombres prohibidos de §6. No copies listados de libros ni crates.
Si faltan teoría o fuentes permitidas, DETENTE en la parte dependiente y
reporta la duda; nunca reconstruyas de memoria cómo lo hace otro motor.
Si hubo exposición prohibida, registra el hecho y no implementes el método.

Revisa primero engine-laws para evitar duplicar una ley existente. Completa
ficha, derivación de tangente y esperados analíticos propios antes del código.
Adapta las plantillas de docs/plantillas/, implementa el trait sin cambiarlo,
expón descriptor/fábrica como los métodos existentes y verifica conformidad,
negativos, pureza y benchmark nativo independiente. Sin unsafe nuevo ni
unwrap/expect/panic en biblioteca, sin dependencias nuevas injustificadas.
No modifiques clean-room/**, docs/CLEAN_ROOM.md, CODEOWNERS,
.github/workflows ni vendor/. No cambies los 16 hashes de DETERMINISM.md.
No hagas commit, push, rebase, reset, stash ni checkout de ramas.
Documentación, mensajes y bitácora en español, sin rutas locales absolutas;
usa rutas relativas y marcadores como $CARGO_TARGET_DIR.

Completa provenance/<ticket>.md siguiendo provenance/TEMPLATE.md:
autor <agente/modelo>, fecha <fecha del ticket>, fuentes, declaración,
validación y revisor clean-room PENDIENTE (J4). Añade CHANGELOG.md.
Ejecuta las seis comprobaciones de la guía y el cruce de los 16 hashes.
Al terminar informa archivos, decisiones, tests passed/failed, resultado de
cada comando y límites o pendientes; no declares aprobación clean-room.
```
