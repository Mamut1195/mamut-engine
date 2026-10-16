# Bitácora clean-room · ENG-GOV-002 · incidente de acceso web de sesiones revisoras

- Ticket / rama: ENG-GOV-002, `j2/clean-room-incident`.
- Fecha del registro: 2026-10-11 (America/Guayaquil). Las horas son locales y salen de los nombres de los archivos de sesión de Codex (`~/.codex/sessions/2026/10/{10,11}/`).
- Agente y modelo: Codex CLI 0.162.0, modelo `gpt-6.1-sol`.
- Naturaleza: auditoría de solo lectura de las sesiones de Codex. Este registro no corrige código; deja constancia de hechos y de la mitigación.

## 1. Hecho

J4 informó que Codex, usado como revisor, intentó abrir por web código fuente de OpenSees, CalculiX y LAPACK. `docs/CLEAN_ROOM.md` §5 prohíbe abrir OpenSees y CalculiX; LAPACK no figura en §5, pero el protocolo pide revisar la similitud con él (§9). La auditoría de las 27 sesiones de Codex de mamut-engine registradas hasta las 07:40 del 2026-10-11 (8 en `/workspace/j4rev/*`, 19 en clones de trabajo) confirma el informe.

Causa: el sandbox de Codex (`workspace-write`, `network_access=false`, o `read-only`) solo limita los comandos de shell. La herramienta web del modelo (`web__run`) es otra vía y seguía activa con `web_search` sin configurar. Ninguna sesión tenía `web_search` en `~/.codex/config.toml` ni `--search`.

## 2. Sesiones revisoras con acceso a fuentes prohibidas

Rol revisor, solo lectura (`sandbox: read-only`), lanzadas por J4.

| Sesión (hora local) | Directorio | Qué se intentó | Qué se abrió realmente |
|---|---|---|---|
| 2026-10-11 00:18 (`01a12966-5ee1`) | `j4rev/engine-pr18` (revisión de #18, ENG-LA-007) | búsqueda web «netlib lapack DSYTF2…» y apertura de `COPYING.EIGEN.MPL2` de faer | La apertura de `COPYING.EIGEN.MPL2` falló. La búsqueda devolvió fragmentos (≤200 palabras por resultado) de páginas de código de LAPACK en netlib (`dsytf2.f`, `dsytf2_rook.f`, `dsytrf.f`, `dsytri.f`) y de espejos. No se abrió ninguna página completa. |
| 2026-10-11 07:28 (`01a12af0-1524`) | `j4rev/engine-retro` (revisión retroactiva) | OpenSees `Truss.cpp`; CalculiX `shape2l.f`, `shape3l.f`; LAPACK `dgemm.f` | **Abiertos con éxito:** OpenSees `SRC/element/truss/Truss.cpp` (raw.githubusercontent, 1582 líneas) y LAPACK `BLAS/SRC/dgemm.f` (408 líneas). CalculiX falló («Failed to fetch») y también `netlib.org/lapack/double/dgemm.f` y `dhondt.de/ccx_2.23.src.tar.bz2`. Además aparecieron como resultados de búsqueda `OpenSees/SRC/element/Element.cpp` y un espejo de CalculiX. |
| 2026-10-11 07:28 (`01a12af0-1527`, log C) | `j4rev/engine-retro` | OpenSees `SparseGenRowLinSOE.cpp` (raw, GitHub y doxygen); CalculiX `mafillsm.f`; LAPACK `dsytrf.f`, `dsytf2.f` | **Abiertos con éxito:** CalculiX `src/mafillsm.f` (976 líneas) y LAPACK `SRC/dsytf2.f` (610 líneas). OpenSees `SparseGenRowLinSOE.cpp` falló por raw, por doxygen y por la URL de GitHub («Failed to fetch» / «Internal Error»). Una búsqueda previa devolvió además el índice de archivos de OpenSees (doxygen) y espejos de CalculiX. |
| 2026-10-11 07:28 (`01a12af0-151b`, log A) | `j4rev/engine-retro` | sin acceso web propio | ninguno (sin llamadas a la herramienta web) |

El resto de sesiones revisoras de mamut-engine (#13, #14, #20) no usó la herramienta web. La del #24 (2026-10-11 01:17) solo abrió documentación de `doc.rust-lang.org` y `docs.kernel.org`.

### 2.1 Revisión retroactiva: correspondencia con los logs de J4 y uso en veredictos

> **Nota (2026-10-15):** los logs citados en esta sección (`/workspace/j4rev/cx-retro-{A,B,C}.txt`) ya no existen: la carpeta se perdió el 2026-10-15. Las citas de línea se conservan como registro histórico y no pueden reverificarse sobre esos archivos.

Los logs de J4 (`/workspace/j4rev/cx-retro-{A,B,C}.txt`) solo registran las líneas `web search:` (consulta o URL pedida), no el contenido devuelto. El contenido se verificó en las transcripciones de sesión (`~/.codex/sessions/2026/10/11/rollout-…01a12af0-{151b,1524,1527}…jsonl`, salidas de `web__run`).

| Log | Sesión | URL pedida (línea del log) | Resultado según la transcripción | Contenido |
|---|---|---|---|---|
| A | `…151b` | ninguna | sin llamadas web | — |
| B | `…1524` | `raw.githubusercontent.com/OpenSees/OpenSees/master/SRC/element/truss/Truss.cpp` (5154); búsqueda `site:github.com/OpenSees/OpenSees Truss.cpp…` (5086); búsqueda `'Truss::getTangentStiff'` (5255) | **devolvió contenido** (1582 líneas) | código fuente C++ de OpenSees |
| B | `…1524` | `raw.githubusercontent.com/Reference-LAPACK/lapack/master/BLAS/SRC/dgemm.f` (solo consta como llamada en la transcripción; el log no la lista) | **devolvió contenido** (408 líneas) | código fuente Fortran de LAPACK (BLAS de referencia) |
| B | `…1524` | `raw.githubusercontent.com/Dhondtguido/CalculiX/master/src/shape2l.f` (5533, 5562) y `shape3l.f` | falló («Failed to fetch») | ninguno |
| B | `…1524` | `netlib.org/lapack/double/dgemm.f`, `dhondt.de/ccx_2.23.src.tar.bz2`, `docs.rs/…/matown.rs.html`, `docs.rs/…/vec_storage.rs.html` | fallaron / no accesibles | ninguno |
| C | `…1527` | `…/OpenSees/master/SRC/system_of_eqn/linearSOE/sparseGEN/SparseGenRowLinSOE.cpp` (6417), `opensees.berkeley.edu/…SparseGenRowLinSOE_8cpp-source.html` (6483), `github.com/OpenSees/…SparseGenRowLinSOE.cpp` (6937) | fallaron las tres | ninguno de OpenSees; la búsqueda de 5981 devolvió solo resultados de índice (p. ej. `files.html` de doxygen) |
| C | `…1527` | `raw.githubusercontent.com/Dhondtguido/CalculiX/master/src/mafillsm.f` | **devolvió contenido** (976 líneas) | código fuente Fortran de CalculiX |
| C | `…1527` | `raw.githubusercontent.com/Reference-LAPACK/lapack/master/SRC/dsytf2.f` (y `dsytrf.f`) | `dsytf2.f` **devolvió contenido** (610 líneas) | código fuente Fortran de LAPACK |

Resumen de los hechos: OpenSees (`Truss.cpp`) y LAPACK (`dgemm.f`, `dsytf2.f`) y CalculiX (`mafillsm.f`) **sí devolvieron código fuente** en las sesiones B y C. Las demás aperturas de CalculiX y OpenSees fallaron. La creencia de J4 de que CalculiX y LAPACK funcionaron se confirma (`mafillsm.f`, `dsytf2.f`, `dgemm.f`); además funcionó `Truss.cpp` de OpenSees, que no estaba en esa lista.

Uso en veredictos (texto final de cada log):

- **A** (`cx-retro-A.txt:4182`, repetido en 4195): declara «no se realizó comparación externa de similitud con OpenSees/CalculiX/faer/nalgebra/LAPACK». Coherente con la transcripción: no abrió nada.
- **B** (`cx-retro-B.txt:5582`, repetido en 5603): «Sin indicios distintivos de copia en la comparación parcial realizada; no se completó el contraste de fuentes de CalculiX/faer». No nombra OpenSees ni LAPACK como fuente de la comparación, aunque la sesión había abierto `Truss.cpp` y `dgemm.f`. El veredicto menciona el contraste como parcial; qué parte se hizo contra qué fuente no consta.
- **C** (`cx-retro-C.txt:6957`, repetido en 6976): cita las URL de `mafillsm.f` y `dsytf2.f` y afirma «no detecté similitud sustantiva»; «OpenSees quedó sin verificar por fallo de acceso». **Este veredicto sí se apoyó en código fuente de CalculiX y LAPACK abierto por la sesión**, contra la regla del §5 (CalculiX) y sin que LAPACK conste como fuente permitida.

Ninguno de los tres logs contiene un hallazgo que cite o reproduzca líneas de esas fuentes. Los hallazgos (p. ej. #21, #23, #11) se describen con ejemplos numéricos propios del repo. No se verificó si ideas de esas fuentes influyeron en la formulación de los hallazgos.

No consta que ninguna de estas sesiones copiara texto al repo: eran de solo lectura y no escribieron archivos. Lo que sí pudo pasar es que el revisor haya incorporado a sus hallazgos ideas vistas en esas fuentes; los hallazgos viajaron como texto a quienes autoran las correcciones.

## 3. Sesiones autoras

De las 19 sesiones en clones de trabajo, 17 fueron autoras (`sandbox: workspace-write`, `network_access=false`) y 2 fueron de solo lectura (2026-10-10 09:01 y 14:09, sin uso web). Ninguna ejecutó `curl`, `wget`, `git clone` ni `gh api` desde la shell. Las autoras tenían, sin embargo, la herramienta web disponible. Seis la usaron:

| Sesión (hora local) | Directorio / rama | Fuentes web abiertas |
|---|---|---|
| 2026-10-10 20:22 | `mamut-engine` (memo ENG-GOV-004, sin código del motor) | `opensees.berkeley.edu/OpenSees/copyright.php` y un hilo del foro de OpenSees sobre términos de copyright (`viewtopic.php?p=122700`); páginas de la AGPL (fallaron). Son páginas de licencia, no código. Alcance del §5 («fragmentos de código fuente en foros»): J4 debe confirmar que el hilo no contiene código. |
| 2026-10-10 23:16 y 23:20 | `mamut-engine-e` (`j2/eng-core-007-traits`) | páginas editoriales del libro de McGuire, Gallagher y Ziemian (Wiley, Bucknell) |
| 2026-10-10 23:37 y 23:50 | `mamut-engine-g` (`j2/eng-la-007-linalg-backend`, #18) | crates.io, docs.rs y README de faer en codeberg; `nalgebra`, `sprs`, `scirs2-sparse` (metadatos y documentación de crates) |
| 2026-10-10 23:55 | `mamut-engine-i` (`j2/eng-geo-001-soil-profile`) | página del editor (Cengage) del libro de Das |

**Ninguna sesión autora abrió código de OpenSees, CalculiX o LAPACK.** Verificado contra las URL de cada llamada y contra las salidas: no hay coincidencias de esas fuentes salvo la excepción de licencia de arriba y menciones de LAPACK en la documentación de faer. Las sesiones autoras de ASM-001 (`mamut-engine-n`) y ELM-001 (`mamut-engine-p`) no usaron la herramienta web.

Tickets cuyas sesiones autoras usaron la web sin abrir fuentes prohibidas: ENG-CORE-007, ENG-LA-007 (#18) y ENG-GEO-001. Ningún PR queda afectado por el §5 según este registro. Sí fueron *revisados* por sesiones con acceso web a fuentes prohibidas: #18 y los PR de la revisión retroactiva (tabla 2).

### 3.1 Incidente OpenCode en mamut-contracts (2026-10-09)

El 2026-10-09, hacia las 21:24 ECT, dos sesiones de OpenCode en mamut-contracts usaron la red: un `webfetch` y tres llamadas `urllib` desde bash hacia `rfc8785.txt`, más una `websearch` sobre RFC 9745. Eran RFC públicos, no código prohibido, pero violan la regla de §10 (revisores sin red ni fuentes externas). Las 118 sesiones de OpenCode de engine, j4rev y audit no usaron red. Mitigación: permisos de OpenCode y restricción de red por sistema operativo (§10), **PENDIENTE** de decisión de Jonathan.

## 4. Mitigación

1. Codex corre siempre sin red y sin herramienta web (`docs/CLEAN_ROOM.md` §10). Configuración verificada con `codex-cli 0.162.0`: `web_search = "disabled"`, `[sandbox_workspace_write] network_access = false` y `[features] apps = false` en `~/.codex/config.toml`, igual que en §10 (por línea de comandos: `-c web_search=disabled -c sandbox_workspace_write.network_access=false --disable apps`). Probada: con las tres la sesión no dispone de herramienta web y la shell sigue funcionando. `web_search=disabled` por sí solo **no** bastaba.
2. J4 repite la revisión de las sesiones del punto 2 (si el hallazgo depende de lo visto en OpenSees/CalculiX/LAPACK, se vuelve a derivar sin red) y confirma el hilo del foro del punto 3.
3. Los PR revisados por esas sesiones (#18 y la revisión retroactiva: #6, #7, #8, #10, #11, #19, #21, #23, #25) se validan con una revisión de procedencia sin red; no se reescribe código mientras J4 no encuentre similitud.
4. OpenCode, como revisor, sigue la misma regla (§10): invocación no interactiva sobre archivos locales, sin herramientas web; Codex y OpenCode nunca abren código de OpenSees, CalculiX ni LAPACK.
5. Revisor clean-room humano: PENDIENTE (firma de Jonathan).
