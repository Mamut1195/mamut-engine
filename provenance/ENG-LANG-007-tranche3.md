# ENG-LANG-007 — Tests, validation and scripts in English

- Ticket: MAM-843 (ENG-LANG-007), tranche 3.
- Date: 2026-10-27.
- Base: `origin/main`, `44407663bfb8b7a2b75c738ee6f5ef82895d2678`.
- Sources: all scoped scripts and Python test sources, the ticket instructions,
  `provenance/ENG-LANG-007.md` (tranches 1 and 2), and the repository
  restricted-identifier list. Work and verification were offline. No third-party
  code, numerical changes, or new mathematical derivations were introduced.

## Scope and translated lines

Reviewed 28 sources: every file under `scripts/**` except the excluded
`scripts/schema-sources.json`, and every file under `bindings/python/tests`,
including `unwind.rs`. Translated text in 24 sources. The four unchanged
sources are `scripts/build-wasm.sh`, `scripts/check-vendor.sh`,
`scripts/serve-wasm.py`, and `bindings/python/tests/test_i18n.py`.
The first three already have English prose; the last intentionally checks
catalog translations and Spanish output. No source was identified as containing
numerical output from an external structural program. Paths are repository-relative.

Counts record translated physical lines, once per category. Docstrings-rustdoc
includes Python docstrings and `//!`; comments include standalone and inline
comments. Messages include usage/help text, printed diagnostics, assertion
diagnostics, skip reasons, and exceptions. Unchanged English, symbolic expressions,
blank lines, executable code, and formatting-only changes are excluded.
Final line numbers below refer to the working sources; no source lines were added
or removed.

| File | Docstrings-rustdoc | Comments | Messages | Total |
| --- | ---: | ---: | ---: | ---: |
| `bindings/python/tests/test_arrays.py` | 1 | 3 | 0 | 4 |
| `bindings/python/tests/test_experiments.py` | 1 | 11 | 24 | 36 |
| `bindings/python/tests/test_facade.py` | 1 | 10 | 1 | 12 |
| `bindings/python/tests/test_i18n.py` | 0 | 0 | 0 | 0 |
| `bindings/python/tests/test_numpy.py` | 1 | 4 | 1 | 6 |
| `bindings/python/tests/test_run.py` | 1 | 15 | 1 | 17 |
| `bindings/python/tests/test_sfc010.py` | 1 | 0 | 1 | 2 |
| `bindings/python/tests/test_typing.py` | 3 | 0 | 0 | 3 |
| `bindings/python/tests/test_version.py` | 1 | 0 | 0 | 1 |
| `bindings/python/tests/unwind.rs` | 1 | 0 | 0 | 1 |
| `scripts/benchmark-python-boundary.py` | 2 | 2 | 3 | 7 |
| `scripts/build-python-wheel.sh` | 0 | 8 | 5 | 13 |
| `scripts/build-wasm.sh` | 0 | 0 | 0 | 0 |
| `scripts/changelog-assemble.sh` | 0 | 0 | 5 | 5 |
| `scripts/changelog-check.sh` | 0 | 0 | 5 | 5 |
| `scripts/check-determinism.mjs` | 0 | 1 | 11 | 12 |
| `scripts/check-vendor.sh` | 0 | 0 | 0 | 0 |
| `scripts/eng-val-006/orchestrate.py` | 8 | 2 | 10 | 20 |
| `scripts/eng-val-006/report.py` | 1 | 0 | 1 | 2 |
| `scripts/eng-val-006/stats.py` | 3 | 0 | 2 | 5 |
| `scripts/eng-val-006/test_orchestrator.py` | 1 | 0 | 0 | 1 |
| `scripts/generate-schemas.py` | 3 | 1 | 22 | 26 |
| `scripts/package-schemas.sh` | 0 | 2 | 1 | 3 |
| `scripts/python-wheel-artifacts.py` | 5 | 12 | 7 | 24 |
| `scripts/serve-wasm.py` | 0 | 0 | 0 | 0 |
| `scripts/tests/test_changelog.sh` | 0 | 0 | 23 | 23 |
| `scripts/tests/test_python_wheel_artifacts.py` | 1 | 10 | 0 | 11 |
| `scripts/tests/test_schema_publication.py` | 1 | 0 | 0 | 1 |
| **Total** | **36** | **81** | **123** | **240** |

Code tokens, identifiers, logic, statement order, argument order, format
placeholders, stable ENG-E-* codes, case names, input data, expected values,
JSON keys, file names, and protected output contracts remain unchanged.
Only the listed documentation and human-facing diagnostics change.
Protected files and the entire existing tranche record are unchanged.
No checkout commit was created.

## Terminology

Terminology follows tranches 1 and 2: DOF, node, member, beam, truss, spring,
fixture, golden, oracle, station, prescribed displacement, load case, mode shape,
base shear, recovery, and end actions. Geotechnical wording uses settlement,
pressure, flux, foundation, and tributary pile length. Project derivations and
code use own or in-house; independence of nested objects, separate builds, and
references retains its original meaning. The words fixture and fixtures retain their form.
Existing units, equations, API references, citations, and symbolic references
retain their meaning. Behavioral docstrings use indicative mood.

## String literals changed

Before any source literal changed, repository-wide `rg --hidden --no-ignore`
literal-text searches covered complete literals or distinctive substrings,
including hidden CI files, tests, goldens, JSON, scripts, clean-room files,
docs, bindings, production catalogs, changelog fragments, and README files.
Git metadata, build output, the explicitly prohibited provenance document,
and the five unopened engine-validation sources were excluded. No prohibited
external location was accessed. Matching references were reviewed for literal
comparisons, regular-expression expectations, and generated-artifact contracts.
No dependent contract was found for the translated text.

This table lists every changed non-comment string literal, including Python
docstrings. Quotes and string prefixes identify the source literals; `\n`
represents physical line breaks in multiline docstrings and preserves existing
escape sequences in single-line literals. Multiple rows can share a source line.

| File:line | Old -> new |
| --- | --- |
| `bindings/python/tests/test_arrays.py:1` | `"""Frontera tabular sin depender de NumPy ni de pytest."""` -> `"""Checks the tabular boundary without depending on NumPy or pytest."""` |
| `bindings/python/tests/test_experiments.py:1` | `"""ENG-PY-005: contratos de experimento sobre la wheel; pytest y unittest offline."""` -> `"""Checks ENG-PY-005 experiment contracts on the wheel with offline pytest and unittest."""` |
| `bindings/python/tests/test_experiments.py:254` | `"El experimento intentó importar NumPy"` -> `"The experiment attempted to import NumPy"` |
| `bindings/python/tests/test_experiments.py:283` | `f"{subcase}: no apareció un PID completo en {pid_file}; "` -> `f"{subcase}: no complete PID appeared in {pid_file}; "` |
| `bindings/python/tests/test_experiments.py:284` | `f"contenido={text!r}, espera={time.monotonic() - waiting:.3f} s, "` -> `f"content={text!r}, wait={time.monotonic() - waiting:.3f} s, "` |
| `bindings/python/tests/test_experiments.py:302` | `f"{subcase}: PID={pid} desapareció antes de consultar su PGID, "` -> `f"{subcase}: PID={pid} disappeared before querying its PGID, "` |
| `bindings/python/tests/test_experiments.py:327` | `f"{subcase}: PID registrados={[p.pid for p in processes]}, "` -> `f"{subcase}: recorded PIDs={[p.pid for p in processes]}, "` |
| `bindings/python/tests/test_experiments.py:384` | `"Requiere grupos de procesos POSIX"` -> `"Requires POSIX process groups"` |
| `bindings/python/tests/test_experiments.py:390` | `"Requiere sleep disponible en las rutas del sistema"` -> `"Requires sleep available on the system paths"` |
| `bindings/python/tests/test_experiments.py:425` | `f"{context}: PID publicado={wrapper}, "` -> `f"{context}: published PID={wrapper}, "` |
| `bindings/python/tests/test_experiments.py:441` | `f"{subcase}: la terminación del grupo tardó demasiado, "` -> `f"{subcase}: group termination took too long, "` |
| `bindings/python/tests/test_experiments.py:448` | `f"{subcase}: PID registrados={[p.pid for p in processes]}, "` -> `f"{subcase}: recorded PIDs={[p.pid for p in processes]}, "` |
| `bindings/python/tests/test_experiments.py:468` | `f"{subcase}: el nieto PID={pid} sigue existiendo "` -> `f"{subcase}: grandchild PID={pid} still exists "` |
| `bindings/python/tests/test_experiments.py:469` | `f"tras el timeout, "` -> `f"after the timeout, "` |
| `bindings/python/tests/test_experiments.py:488` | `"Requiere grupos de procesos POSIX"` -> `"Requires POSIX process groups"` |
| `bindings/python/tests/test_experiments.py:515` | `f"{subcase}: PID={pid}, PGID esperado={process.pid}, "` -> `f"{subcase}: PID={pid}, expected PGID={process.pid}, "` |
| `bindings/python/tests/test_experiments.py:530` | `f"{subcase}: PID registrados={[p.pid for p in processes]}, "` -> `f"{subcase}: recorded PIDs={[p.pid for p in processes]}, "` |
| `bindings/python/tests/test_experiments.py:544` | `f"{subcase}: PID={pid} sigue existiendo tras "` -> `f"{subcase}: PID={pid} still exists after "` |
| `bindings/python/tests/test_experiments.py:563` | `"Requiere grupos de procesos POSIX"` -> `"Requires POSIX process groups"` |
| `bindings/python/tests/test_experiments.py:594` | `"Requiere grupos de procesos POSIX"` -> `"Requires POSIX process groups"` |
| `bindings/python/tests/test_experiments.py:600` | `"Requiere setsid disponible en las rutas del sistema"` -> `"Requires setsid available on the system paths"` |
| `bindings/python/tests/test_experiments.py:625` | `f"{context}: PID={pid}, setsid no se completó, "` -> `f"{context}: PID={pid}, setsid did not complete, "` |
| `bindings/python/tests/test_experiments.py:642` | `f"{subcase}: la limpieza del timeout quedó bloqueada, "` -> `f"{subcase}: timeout cleanup was blocked, "` |
| `bindings/python/tests/test_experiments.py:646` | `f"{subcase}: PID registrados={[p.pid for p in processes]}, "` -> `f"{subcase}: recorded PIDs={[p.pid for p in processes]}, "` |
| `bindings/python/tests/test_experiments.py:662` | `f"{subcase}: PID={pid}, PGID del CLI={process.pid}, "` -> `f"{subcase}: PID={pid}, CLI PGID={process.pid}, "` |
| `bindings/python/tests/test_experiments.py:744` | `"Defina MAMUT_EXPERIMENT_CLI con el binario Rust local"` -> `"Set MAMUT_EXPERIMENT_CLI to the local Rust binary"` |
| `bindings/python/tests/test_facade.py:1` | `"""ENG-PY-003: API pública instalada, referencias Rust y contratos sin red."""` -> `"""Checks the ENG-PY-003 installed public API, Rust references, and offline contracts."""` |
| `bindings/python/tests/test_facade.py:26` | `"Defina MAMUT_RUST_REFERENCES; véase README"` -> `"Set MAMUT_RUST_REFERENCES; see README"` |
| `bindings/python/tests/test_numpy.py:1` | `"""Contrato NumPy opcional, descubierto por el pipeline stdlib de la wheel."""` -> `"""Checks the optional NumPy contract, discovered by the wheel's stdlib pipeline."""` |
| `bindings/python/tests/test_numpy.py:23` | `"NumPy opcional no instalado"` -> `"Optional NumPy is not installed"` |
| `bindings/python/tests/test_run.py:1` | `"""Integración sobre wheel: fachada Rust, errores, GIL y cancelación."""` -> `"""Checks wheel integration: Rust facade, errors, GIL, and cancellation."""` |
| `bindings/python/tests/test_run.py:25` | `"Defina MAMUT_RUST_REFERENCE (véase README)"` -> `"Set MAMUT_RUST_REFERENCE (see README)"` |
| `bindings/python/tests/test_sfc010.py:1` | `"""MAM-835: paridad de bytes Rust/FFI/fachada y diagnósticos de SFC-010."""` -> `"""Checks MAM-835 Rust/FFI/facade byte parity and SFC-010 diagnostics."""` |
| `bindings/python/tests/test_sfc010.py:17` | `"Defina MAMUT_RUST_REFERENCES con facade_reference"` -> `"Set MAMUT_RUST_REFERENCES using facade_reference"` |
| `bindings/python/tests/test_typing.py:1` | `"""Consistencia offline de stubs instalados y DTO frente a los contratos Rust.\n\nComplementa get_type_hints cuando no hay mypy/pyright. Comprueba exports,\nfirmas PyO3 y campos requeridos/opcionales; no sustituye un checker completo.\n"""` -> `"""Checks installed stubs and DTO against Rust contracts offline.\n\nComplements get_type_hints when mypy/pyright is unavailable. Checks exports,\nPyO3 signatures, and required/optional fields; does not replace a full checker.\n"""` |
| `bindings/python/tests/test_version.py:1` | `"""Contrato público de la wheel instalada; solo biblioteca estándar."""` -> `"""Checks the installed wheel's public contract using only the standard library."""` |
| `scripts/benchmark-python-boundary.py:2` | `"""Benchmark offline manual sobre wheel instalada; no se ejecuta en CI."""` -> `"""Runs a manual offline benchmark on the installed wheel; does not run in CI."""` |
| `scripts/benchmark-python-boundary.py:21` | `"""Réplicas independientes del voladizo analítico propio, sin resolverlo."""` -> `"""Creates separate copies of the project's own analytical cantilever without solving it."""` |
| `scripts/benchmark-python-boundary.py:75` | `"repeats >= 1, warmups >= 0 y tamaños entre 1 y 64"` -> `"repeats >= 1, warmups >= 0 and sizes between 1 and 64"` |
| `scripts/benchmark-python-boundary.py:97` | `"La frontera cambió results/1"` -> `"The boundary changed results/1"` |
| `scripts/benchmark-python-boundary.py:109` | `"CLI difiere"` -> `"CLI differs"` |
| `scripts/build-python-wheel.sh:13` | `"${CARGO_TARGET_DIR:?Exporta CARGO_TARGET_DIR antes de ejecutar este script}"` -> `"${CARGO_TARGET_DIR:?Export CARGO_TARGET_DIR before running this script}"` |
| `scripts/build-python-wheel.sh:18` | `'Uso: %s [directorio-de-salida-nuevo]\nLa salida no debe existir.\n'` -> `'Usage: %s [new-output-directory]\nThe output must not exist.\n'` |
| `scripts/build-python-wheel.sh:25` | `'Se requiere Rust %s instalado localmente; ajusta PATH a esa toolchain. No se descargará.\n'` -> `'Rust %s must be installed locally; adjust PATH to that toolchain. It will not be downloaded.\n'` |
| `scripts/build-python-wheel.sh:30` | `'Se requiere maturin 1.15.0 local (MATURIN=ejecutable).\n'` -> `'Local maturin 1.15.0 is required (MATURIN=executable).\n'` |
| `scripts/build-python-wheel.sh:98` | `'Verificación offline completa; artefactos locales: %s\n'` -> `'Offline verification complete; local artifacts: %s\n'` |
| `scripts/changelog-assemble.sh:18` | `'VERSION inválida (usa un identificador sin espacios).'` -> `'Invalid VERSION (use an identifier without spaces).'` |
| `scripts/changelog-assemble.sh:31` | `'falta un CHANGELOG.md regular.'` -> `'a regular CHANGELOG.md is missing.'` |
| `scripts/changelog-assemble.sh:32` | `'falta el directorio changelog.d.'` -> `'the changelog.d directory is missing.'` |
| `scripts/changelog-assemble.sh:68` | `'no se pudo comprobar el estado del árbol de trabajo Git.'` -> `'could not check the Git working tree status.'` |
| `scripts/changelog-assemble.sh:142` | `'Changelog: versión %s ensamblada; %s fragmentos eliminados.\n'` -> `'Changelog: version %s assembled; %s fragments removed.\n'` |
| `scripts/changelog-check.sh:12` | `'Changelog: exento por CHANGELOG_RELEASE=1 (PR de release).\n'` -> `'Changelog: exempt under CHANGELOG_RELEASE=1 (release PR).\n'` |
| `scripts/changelog-check.sh:17` | `"no se pudo comparar $base...HEAD."` -> `"could not compare $base...HEAD."` |
| `scripts/changelog-check.sh:31` | `'Changelog: sin cambios en crates/ ni bindings/; no se exige fragmento.\n'` -> `'Changelog: no changes in crates/ or bindings/; no fragment required.\n'` |
| `scripts/changelog-check.sh:41` | `"no se pudo leer el fragmento: $path"` -> `"could not read the fragment: $path"` |
| `scripts/changelog-check.sh:44` | `'Changelog: OK (%s fragmentos nuevos).\n'` -> `'Changelog: OK (%s new fragments).\n'` |
| `scripts/check-determinism.mjs:7` | `'Uso: node scripts/check-determinism.mjs módulo.wasm hashes-native.txt'` -> `'Usage: node scripts/check-determinism.mjs module.wasm hashes-native.txt'` |
| `scripts/check-determinism.mjs:10` | `'Se requieren diez hashes por caso'` -> `'Ten hashes are required per case'` |
| `scripts/check-determinism.mjs:12` | `'El módulo debe ejecutarse sin imports'` -> `'The module must run without imports'` |
| `scripts/check-determinism.mjs:17` | `'Índice wasm fuera de rango'` -> `'Wasm index out of range'` |
| `scripts/check-determinism.mjs:19` | `'Palabra de hash fuera de rango'` -> `'Hash word out of range'` |
| `scripts/check-determinism.mjs:25` | ```Caso duplicado: ${name}``` -> ```Duplicate case: ${name}``` |
| `scripts/check-determinism.mjs:29` | ```${name}: identidad y orden de casos nativo/wasm``` -> ```${name}: native/wasm case identity and order``` |
| `scripts/check-determinism.mjs:31` | ```${name}: repetibilidad nativa``` -> ```${name}: native repeatability``` |
| `scripts/check-determinism.mjs:32` | ```${name}: caso wasm falló``` -> ```${name}: wasm case failed``` |
| `scripts/check-determinism.mjs:34` | ```${name}: igualdad bit a bit nativo/wasm, corrida ${iteration}``` -> ```${name}: native/wasm bitwise equality, run ${iteration}``` |
| `scripts/check-determinism.mjs:44` | ```Falta cobertura obligatoria: ${required}``` -> ```Missing required coverage: ${required}``` |
| `scripts/eng-val-006/orchestrate.py:2` | ``"""Orquestador del Engine para ENG-VAL-006.\n\nMide el binario release. No lanza la campaña contra el VPS: el valor por defecto\nqueda disponible para esa corrida, y --smoke hace una pasada local mínima.\nLas tablas salen después, solo de los CSV.\n\n`LocalHost.measure` retiene en memoria, hasta que el proceso termina, el stdout\ndel motor y el stderr de GNU time (`subprocess.run` con `capture_output=True`).\nNo lee esa salida en flujo: una corrida completa cabe en RAM antes de extraer\nel bloque de `time -v`.\n"""`` -> ``"""Orchestrates the Engine for ENG-VAL-006.\n\nMeasures the release binary. Does not launch the campaign against the VPS: the default\nremains available for that run, and --smoke performs a minimal local pass.\nProduces tables afterwards, only from the CSV files.\n\n`LocalHost.measure` retains Engine stdout and GNU time stderr in memory until\nthe process ends (`subprocess.run` with `capture_output=True`).\nDoes not stream that output: an entire run fits in RAM before extracting\nthe `time -v` block.\n"""`` |
| `scripts/eng-val-006/orchestrate.py:79` | `f"sha256 distinto en {path}"` -> `f"sha256 mismatch in {path}"` |
| `scripts/eng-val-006/orchestrate.py:103` | `"la salida de time -v no trae tiempo de pared o memoria"` -> `"time -v output has no wall time or memory"` |
| `scripts/eng-val-006/orchestrate.py:207` | `"la corrida superó el tiempo límite"` -> `"the run exceeded the time limit"` |
| `scripts/eng-val-006/orchestrate.py:210` | `f"el Engine terminó con código {completed.returncode}: {tail}"` -> `f"the Engine exited with code {completed.returncode}: {tail}"` |
| `scripts/eng-val-006/orchestrate.py:221` | `"los runners no quedaron idle antes del bloque"` -> `"the runners did not become idle before the block"` |
| `scripts/eng-val-006/orchestrate.py:247` | `"un job apareció durante el bloque; no se publica"` -> `"a job appeared during the block; it is not published"` |
| `scripts/eng-val-006/orchestrate.py:289` | `"se requiere el binario release de mamut-engine"` -> `"the mamut-engine release binary is required"` |
| `scripts/eng-val-006/orchestrate.py:291` | `"el binario release no es ejecutable"` -> `"the release binary is not executable"` |
| `scripts/eng-val-006/orchestrate.py:384` | `f"no está el binario de time: {args.time_bin}"` -> `f"the time binary is missing: {args.time_bin}"` |
| `scripts/eng-val-006/orchestrate.py:387` | `"el binario no es ejecutable"` -> `"the binary is not executable"` |
| `scripts/eng-val-006/report.py:1` | `"""Tablas y gráficos leídos solo de los CSV crudos."""` -> `"""Builds tables and charts only from raw CSV files."""` |
| `scripts/eng-val-006/report.py:71` | `"el CSV no tiene medidas"` -> `"the CSV has no measurements"` |
| `scripts/eng-val-006/stats.py:1` | `"""Estadísticos de las medidas. Hyndman y Fan tipo 6, sin ajuste manual."""` -> `"""Computes measurement statistics using Hyndman and Fan type 6, without manual adjustment."""` |
| `scripts/eng-val-006/stats.py:7` | `"""Cuantil tipo 6: h = (n + 1) p, con interpolación lineal entre estadísticos de orden."""` -> `"""Computes a type 6 quantile: h = (n + 1) p, with linear interpolation between order statistics."""` |
| `scripts/eng-val-006/stats.py:9` | `"no hay muestras"` -> `"there are no samples"` |
| `scripts/eng-val-006/stats.py:11` | `"la probabilidad está fuera de [0, 1]"` -> `"the probability is outside [0, 1]"` |
| `scripts/eng-val-006/stats.py:27` | `"""Mediana, rango intercuartil, mínimo y máximo de una muestra no vacía."""` -> `"""Computes the median, interquartile range, minimum, and maximum of a nonempty sample."""` |
| `scripts/eng-val-006/test_orchestrator.py:2` | `"""Pruebas del orquestador que no publican tiempos del Engine."""` -> `"""Tests the orchestrator without publishing Engine timings."""` |
| `scripts/generate-schemas.py:2` | `"""Generación, comprobación y archivo locales; solo biblioteca estándar."""` -> `"""Generates, checks, and archives locally; uses only the standard library."""` |
| `scripts/generate-schemas.py:32` | `"La base debe ser una URL HTTPS ASCII terminada en / sin query ni fragmento"` -> `"The base must be an ASCII HTTPS URL ending in / without a query or fragment"` |
| `scripts/generate-schemas.py:37` | `"""Inventario exacto, sin seguir enlaces ni aceptar archivos especiales."""` -> `"""Inventories exactly, without following links or accepting special files."""` |
| `scripts/generate-schemas.py:39` | `f"Directorio ausente o enlazado: {directory}"` -> `f"Missing or linked directory: {directory}"` |
| `scripts/generate-schemas.py:44` | `f"Enlace no permitido: {path}"` -> `f"Link not allowed: {path}"` |
| `scripts/generate-schemas.py:50` | `f"Archivo especial no permitido: {path}"` -> `f"Special file not allowed: {path}"` |
| `scripts/generate-schemas.py:62` | `f"Entrada adicional inválida: {entry}"` -> `f"Invalid additional entry: {entry}"` |
| `scripts/generate-schemas.py:64` | `"Registro adicional duplicado"` -> `"Duplicate additional registration"` |
| `scripts/generate-schemas.py:66` | `f"Fuente enlazada: {source}"` -> `f"Linked source: {source}"` |
| `scripts/generate-schemas.py:74` | `"Hay esquemas CLI sin registrar o fuentes adicionales ausentes"` -> `"There are unregistered CLI schemas or missing additional sources"` |
| `scripts/generate-schemas.py:82` | `f"Kind repetido entre API y CLI: {kind}"` -> `f"Duplicate kind across API and CLI: {kind}"` |
| `scripts/generate-schemas.py:91` | `f"Kind inválido: {kind}"` -> `f"Invalid kind: {kind}"` |
| `scripts/generate-schemas.py:95` | `f"$id inválido: {kind}"` -> `f"Invalid $id: {kind}"` |
| `scripts/generate-schemas.py:109` | `"Inventario de schemas/ distinto del registro (ausentes o huérfanos)"` -> `"The schemas/ inventory differs from the registry (missing or orphaned files)"` |
| `scripts/generate-schemas.py:112` | `f"Bytes desactualizados: schemas/{name}"` -> `f"Outdated bytes: schemas/{name}"` |
| `scripts/generate-schemas.py:116` | `"""Comprueba el índice y los bytes antes de empaquetar, sin ejecutar Cargo."""` -> `"""Checks the index and bytes before packaging, without running Cargo."""` |
| `scripts/generate-schemas.py:125` | `"Kind inválido o duplicado en índice"` -> `"Invalid or duplicate kind in index"` |
| `scripts/generate-schemas.py:129` | `"Índice no canónico o metadatos incorrectos"` -> `"Noncanonical index or incorrect metadata"` |
| `scripts/generate-schemas.py:138` | `"La salida debe quedar fuera de schemas/"` -> `"The output must be outside schemas/"` |
| `scripts/generate-schemas.py:142` | `"El archivo de salida no puede ser un enlace"` -> `"The output archive cannot be a link"` |
| `scripts/generate-schemas.py:158` | `"Transporte interno para el test Cargo"` -> `"Internal transport for the Cargo test"` |
| `scripts/generate-schemas.py:159` | `f"Por defecto al generar: {DEFAULT_BASE}"` -> `f"Default when generating: {DEFAULT_BASE}"` |
| `scripts/generate-schemas.py:166` | `"--package no admite opciones de generación"` -> `"--package does not accept generation options"` |
| `scripts/generate-schemas.py:187` | `"Retirar manualmente los huérfanos antes de regenerar"` -> `"Remove orphaned files manually before regenerating"` |
| `scripts/generate-schemas.py:194` | `"schemas/: bytes, índice e inventario OK"` -> `"schemas/: bytes, index, and inventory OK"` |
| `scripts/package-schemas.sh:6` | `'Uso: bash scripts/package-schemas.sh DIRECTORIO_SALIDA\n'` -> `'Usage: bash scripts/package-schemas.sh OUTPUT_DIRECTORY\n'` |
| `scripts/python-wheel-artifacts.py:2` | `"""Artefactos deterministas de wheel, solo biblioteca estándar (CPython >=3.11)."""` -> `"""Produces deterministic wheel artifacts using only the standard library (CPython >=3.11)."""` |
| `scripts/python-wheel-artifacts.py:57` | `"""Rechaza rutas; bytes con NUL o UTF-8 inválido se consideran binarios.\n\n    Los str y los bytes UTF-8 sin NUL mantienen el patrón estricto de texto.\n    No se descartan ni decodifican con pérdida los bytes de un binario.\n    """` -> `"""Rejects paths; treats bytes with NUL or invalid UTF-8 as binary.\n\n    Strings and UTF-8 bytes without NUL retain the strict text pattern.\n    Does not discard or decode binary bytes with data loss.\n    """` |
| `scripts/python-wheel-artifacts.py:87` | `"""Inspecciona TODOS los miembros ZIP, incluidos dist-info/RECORD y .so."""` -> `"""Inspects ALL ZIP members, including dist-info/RECORD and .so."""` |
| `scripts/python-wheel-artifacts.py:114` | `"Identidades SBOM ambiguas: revisar fuentes de crates duplicados"` -> `"Ambiguous SBOM identities: review sources of duplicate crates"` |
| `scripts/python-wheel-artifacts.py:133` | `f"Falta License-File: {name}"` -> `f"Missing License-File: {name}"` |
| `scripts/python-wheel-artifacts.py:135` | `"Versión/nombre de wheel no corresponde al manifiesto"` -> `"Wheel version/name does not match the manifest"` |
| `scripts/python-wheel-artifacts.py:169` | `"Wheel sin tags"` -> `"Wheel without tags"` |
| `scripts/python-wheel-artifacts.py:201` | `f"Diferencia {name}: {left} / {right}"` -> `f"Difference {name}: {left} / {right}"` |
| `scripts/python-wheel-artifacts.py:204` | `f"ZIP: tamaños {len(left_bytes)}/{len(right_bytes)}, primera diferencia byte {offset}"` -> `f"ZIP: sizes {len(left_bytes)}/{len(right_bytes)}, first differing byte {offset}"` |
| `scripts/python-wheel-artifacts.py:205` | `f"Wheels diferentes: {a.name} {ha} / {b.name} {hb}"` -> `f"Different wheels: {a.name} {ha} / {b.name} {hb}"` |
| `scripts/tests/test_changelog.sh:40` | `"se esperaba un error: $*"` -> `"expected an error: $*"` |
| `scripts/tests/test_changelog.sh:44` | `"mensaje esperado '$expected', recibido '$message'"` -> `"expected message '$expected', received '$message'"` |
| `scripts/tests/test_changelog.sh:123` | `'dry-run modificó archivos'` -> `'dry-run changed files'` |
| `scripts/tests/test_changelog.sh:125` | `'dry-run sin mutaciones; orden, formato, legado y versiones anteriores exactos'` -> `'dry-run without mutations; exact order, formatting, legacy content, and previous versions'` |
| `scripts/tests/test_changelog.sh:130` | `'no eliminó solo los fragmentos'` -> `'did not remove only the fragments'` |
| `scripts/tests/test_changelog.sh:139` | `'dos ensamblados idénticos byte a byte; fecha desde SOURCE_DATE_EPOCH en UTC'` -> `'two byte-identical assemblies; date from SOURCE_DATE_EPOCH in UTC'` |
| `scripts/tests/test_changelog.sh:141` | `'rechazo de versión duplicada'` -> `'duplicate version rejected'` |
| `scripts/tests/test_changelog.sh:154` | `'un error eliminó fragmentos'` -> `'an error removed fragments'` |
| `scripts/tests/test_changelog.sh:172` | `'fechas, nombres, archivos vacíos, enlaces y encabezados inválidos sin consumo de fragmentos'` -> `'invalid dates, names, empty files, links, and headings without consuming fragments'` |
| `scripts/tests/test_changelog.sh:178` | `'release sin fragmentos ni legado'` -> `'release without fragments or legacy content'` |
| `scripts/tests/test_changelog.sh:190` | `'ensamblado rechaza CHANGELOG.md sin seguimiento sin modificar archivos'` -> `'assembly rejects untracked CHANGELOG.md without changing files'` |
| `scripts/tests/test_changelog.sh:206` | `'el rechazo modificó el árbol'` -> `'rejection changed the tree'` |
| `scripts/tests/test_changelog.sh:208` | `'ensamblado rechaza cambios sin preparar, preparados y sin seguimiento sin mutaciones'` -> `'assembly rejects unstaged, staged, and untracked changes without mutations'` |
| `scripts/tests/test_changelog.sh:215` | `'ensamblado requiere un árbol Git; dry-run funciona fuera de Git'` -> `'assembly requires a Git tree; dry-run works outside Git'` |
| `scripts/tests/test_changelog.sh:225` | `'BASE_REF creó un archivo con --output'` -> `'BASE_REF created a file with --output'` |
| `scripts/tests/test_changelog.sh:227` | `'check: documentación exenta y referencia/argumentos inválidos rechazados'` -> `'check: documentation is exempt and invalid references/arguments are rejected'` |
| `scripts/tests/test_changelog.sh:248` | `'check: crates y bindings exigen fragmento, rechazan CHANGELOG y permiten release'` -> `'check: crates and bindings require a fragment, reject CHANGELOG, and allow release'` |
| `scripts/tests/test_changelog.sh:258` | `'check: modificar un fragmento existente no cuenta como añadido'` -> `'check: changing an existing fragment does not count as an addition'` |
| `scripts/tests/test_changelog.sh:276` | `'check: nombres inválidos, fragmentos vacíos y enlaces rechazados'` -> `'check: invalid names, empty fragments, and links rejected'` |
| `scripts/tests/test_changelog.sh:286` | `'faltan fragmentos tras merge'` -> `'missing fragments after merge'` |
| `scripts/tests/test_changelog.sh:287` | `'merge con conflictos'` -> `'merge conflicts'` |
| `scripts/tests/test_changelog.sh:291` | `'dos ramas añaden fragmentos, fusionan sin conflictos y ensamblan ambas entradas'` -> `'two branches add fragments, merge without conflicts, and assemble both entries'` |
| `scripts/tests/test_changelog.sh:293` | `'Changelog: %s grupos de pruebas OK.\n'` -> `'Changelog: %s test groups OK.\n'` |
| `scripts/tests/test_python_wheel_artifacts.py:1` | `"""Regresiones de portabilidad y selección del SBOM (issues #94 y #121)."""` -> `"""Checks portability and SBOM selection regressions (issues #94 and #121)."""` |
| `scripts/tests/test_schema_publication.py:2` | `"""Pruebas offline ejecutadas también por cargo test."""` -> `"""Runs offline tests, also executed by cargo test."""` |

Total: 147 changed non-comment string literal occurrences.

## Left in Spanish

All other literals remain byte-identical. This includes Spanish test inputs,
expected values, regular-expression patterns, exception payloads, catalog
selection data, generated headings, schema-related contracts, wheel metadata,
legal notices, and report labels. Existing English and language-neutral literals
are preserved too. The retained Spanish locations are listed below; ranges
include embedded source strings and generated-output templates.

| Source file | Retained locations | Reason |
| --- | --- | --- |
| `scripts/changelog-assemble.sh` | 6, 22–23, 25–27, 38–39, 44, 55, 60, 62, 66–67, 69, 111, 126 | Usage and rejection vocabulary consumed by `scripts/tests/test_changelog.sh`; date tokens are expected literally. Labels and legacy headings are written into CHANGELOG.md and compared byte for byte. The shared usage vocabulary is retained conservatively. |
| `scripts/changelog-check.sh` | 6, 7, 10, 34, 35, 37, 40, 42 | Usage, reference rejection, fragment requirements, names, regular-file checks, and empty-fragment diagnostics have literal substring expectations in the shell suite. |
| `scripts/tests/test_changelog.sh` | 27, 50–114, 140, 144–171, 181, 186, 197–203, 211, 218, 221–226, 231–245, 251–268, 272–289 | Fixture inputs, generated changelog goldens, and `expect_fail` patterns remain unchanged. This includes short labels and ASCII Spanish missed by the heuristic. |
| `scripts/check-determinism.mjs` | 36, 46 | Published cross-target summary vocabulary is retained conservatively because its output/report contract is uncertain. |
| `scripts/eng-val-006/report.py` | 33, 121, 124 | Markdown table headings and SVG titles are written into generated reports. |
| `scripts/eng-val-006/test_orchestrator.py` | 82 | Model-path test input remains unchanged. |
| `scripts/generate-schemas.py` | 162 | The CI regeneration rejection is compared as UTF-8 bytes by `scripts/tests/test_schema_publication.py:128`. |
| `scripts/tests/test_schema_publication.py` | 128 | The expected CI rejection bytes remain unchanged. |
| `scripts/python-wheel-artifacts.py` | 43–44, 72, 83, 94, 131, 137–140, 149, 151, 158, 178, 180, 217, 221, 223, 225 | Contract-status, notices, legal-review values, and ABI status enter generated artifacts. Legal-file, facade, version, and visible-tool errors have regex expectations. The portability error family is retained conservatively: wrapper member text and byte offsets have regex expectations. |
| `scripts/tests/test_python_wheel_artifacts.py` | 73, 76, 89, 119, 156, 161–162, 201–205, 219–220, 256, 286, 297 | Test path data, legal-file fixture contents, expected metadata vocabulary, and error patterns remain unchanged, including short Spanish path labels. |
| `bindings/python/tests/test_arrays.py` | 28–52, 140 | The embedded subprocess test source includes a compared simulated NumPy error; its full input string is retained conservatively. The finite-f64 pattern is produced by protected Python production code. |
| `bindings/python/tests/test_experiments.py` | 179–180, 190, 207, 367–381, 505, 569 | User exceptions, simulated Engine errors, UTF-8 stdout/stderr payloads, expected envelopes, keyboard interruption, and denied-probe messages are test inputs or expected values. |
| `bindings/python/tests/test_i18n.py` | Entire file | Spanish language-selection data and catalog-derived Spanish expectations are intentional. The file is unchanged. |

No Spanish comment or docstring prose remains in the reviewed sources. The
headings inside the shell test heredocs are expected output data, not comments.

## Remaining (later tranches)

Spanish-line counts use the same reproducible heuristic as tranches 1 and 2:
Spanish diacritics or punctuation, or at least two case-insensitive whole words
from que, para, del, una, los, las, por, sin, debe, según, el, la, de, en, con,
un, se, al, como, no, es, son, su, sus, entre, cada, todos, todas, también, más,
antes, después, hasta, sobre, solo, desde, durante. Each matching physical line
counts once, including literal data. These are raw heuristic counts, not a
claim that every hit is eligible prose. Short labels and escaped byte strings
can be missed; English prose, locale codes, and command options can produce
false positives. The explicit retained-location inventory above covers these
missed Spanish literals.

| Remaining in-scope source | Spanish-line hits |
| --- | ---: |
| `bindings/python/tests/test_arrays.py` | 1 |
| `bindings/python/tests/test_experiments.py` | 3 |
| `bindings/python/tests/test_i18n.py` | 6 |
| `scripts/build-python-wheel.sh` | 1 |
| `scripts/build-wasm.sh` | 1 |
| `scripts/changelog-assemble.sh` | 13 |
| `scripts/changelog-check.sh` | 9 |
| `scripts/check-determinism.mjs` | 1 |
| `scripts/eng-val-006/report.py` | 2 |
| `scripts/generate-schemas.py` | 1 |
| `scripts/python-wheel-artifacts.py` | 13 |
| `scripts/tests/test_changelog.sh` | 33 |
| `scripts/tests/test_python_wheel_artifacts.py` | 4 |
| **Total** | **88** |

The residual totals are 78 hits under `scripts/**` and 10 under
`bindings/python/tests`. The 88 hits include six intentional i18n locale
checks and English/code false positives in `scripts/build-python-wheel.sh:89`,
`scripts/build-wasm.sh:35`, `scripts/changelog-check.sh:17`, and
`scripts/changelog-check.sh:31`. Retained data and contracts require their
own authorized comparison/output review before translation.

The five engine-validation files below were not opened. Their counts are carried
forward from the tranche 2 inventory, not recomputed.

| Remaining source | Spanish-line hits |
| --- | ---: |
| `crates/engine-validation/src/fixtures.rs` | 39 |
| `crates/engine-validation/tests/oracles.rs` | 24 |
| `crates/engine-validation/tests/fixtures.rs` | 8 |
| `crates/engine-validation/README.md` | 135 |
| `crates/engine-validation/ORACLES.md` | 132 |
| **Total** | **338** |

- `docs/**`: 1482 heuristic hits in 140 text files; unchanged and deferred.
- `scripts/schema-sources.json`: explicitly excluded; unchanged, zero heuristic hits.
- `crates/engine-api/tests/val032_runs.rs`: 13 Spanish comment lines and eleven
  messages remain deferred by the full-file digest contract, as recorded in
  tranche 2. The file was not opened for this tranche.
- `clean-room/tests/run.sh` and the two workflows are handled in a separate
  owner-authorized pull request, outside this tranche. Tranche 2 records 50 hits in one source and 16 in two sources,
  respectively; those counts were not recomputed.
- The retained literals inventoried in tranches 1 and 2 remain deferred.
- Protected production code, schemas, goldens, and the other excluded sources
  remain unchanged.

## Verification

All checks ran offline against base `44407663bfb8b7a2b75c738ee6f5ef82895d2678`.

- Python boundary audit with `ast`: for all 17 changed `.py` files, the tree with
  docstrings and string literals masked is identical to the base; literal counts
  are equal per file; 75 Python string literals differ, all listed above.
- Shell and JavaScript: non-comment diff lines are only the listed messages;
  `printf` formats keep their placeholders and argument order.
- `python3 -m py_compile` on the 17 changed `.py` files: OK. `bash -n` on the
  changed shell sources: OK. `node --check scripts/check-determinism.mjs`: OK.
- Base versus branch, same exit codes: `generate-schemas.py --check` and
  `--help` (0), `python-wheel-artifacts.py --help` (0), `orchestrate.py --help`
  and `report.py --help` (0), `package-schemas.sh` without arguments (2),
  `build-python-wheel.sh` with extra arguments (1), `check-determinism.mjs`
  without arguments (1), `changelog-assemble.sh --help` (1),
  `benchmark-python-boundary.py --help` (1 on both: the extension module is not
  installed).
- `scripts/tests/test_changelog.sh`, `scripts/tests/test_schema_publication.py`,
  `scripts/tests/test_python_wheel_artifacts.py`, and the eng-val-006 unittest
  suite: OK on base and branch.
- `bindings/python/tests` with pytest: UNKNOWN; pytest and the built extension
  are not installed in the offline environment.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --offline
  -- -D warnings`: OK. `cargo test --workspace --offline`: 850 passed, 0 failed.
- `clean-room/check.sh origin/main`, `changelog-check.sh origin/main`,
  `generate-schemas.py --check`, and `git diff --check`: OK.
