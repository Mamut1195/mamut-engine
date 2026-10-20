# Bitácora clean-room · ENG-VAL-005

- Ticket / PR: MAM-105 · ENG-VAL-005, oráculos libres como línea base
  independiente. Cambios sin commit ni push.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: especificador e implementador de metadatos y pruebas de validación.
- Contrato implementado: inventario de oráculos V1 y guardia sobre las tablas
  existentes; `fixture/1` conserva su contrato y declara explícitamente los
  límites que antes obtenía por defecto.
- Fuentes permitidas usadas: petición del ticket; `docs/CLEAN_ROOM.md`, §4/§8;
  `provenance/TEMPLATE.md`; derivaciones locales ENG-VAL-001 (estática y
  D1–D12), ENG-GEO-002 (viga sobre lecho), ENG-GEO-006 (serie, convolución y
  autofunciones multicapa r32), ENG-CORE-009 (`fixture/1`) y ENG-BENCH-002
  (viga-columna). No se consultaron fuentes externas ni se usó red.
- Derivaciones propias: se conservan y enlazan las notas anteriores desde
  [ORACULOS.md](../crates/engine-validation/ORACULOS.md), con ecuaciones,
  condiciones y parámetros ya versionados. No se introduce un método físico.
- Código de terceros copiado: ninguno. Sin dependencias nuevas ni cambios
  de Cargo.toml/Cargo.lock, vendor, workflows o reglas clean-room.
- Validación: soluciones analíticas propias y pruebas numéricas existentes;
  guardia de fuente, cita, justificación, tolerancia y cobertura por caso.
  Referencias bibliográficas no cotejadas: PENDIENTES, detalladas en la matriz.
- Comparación con oráculo (repo hermano, informativa): no. No se abrió ningún
  archivo de ese repositorio ni se incorporaron datos o resultados de motores.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados
  para este ticket. Tampoco consulté ni usé código o resultados de CalculiX o
  LAPACK, ni salidas de OpenSees o Stabileo."
- Revisor clean-room: PENDIENTE (J4).

## Decisiones y alcance

Se inventarían 91 casos de engine-validation: 73 de ENG-VAL-001, seis Winkler,
siete de consolidación (incluido multicapa no transformable) y cinco JSON
`fixture/1`. Cada caso tiene ecuación o valores esperados, fuente y tolerancia;
las suites numéricas existentes siguen recorriendo todas sus filas.
No hay artefactos identificados como ENG-VAL-002 en el árbol inspeccionado.
Se documenta también el contraste analítico independiente de los benchmarks
publicados P-Delta existentes, sin copiar sus tablas o figuras.

La guardia `tests/oracles.rs` comprueba todas las tablas y fixtures, rechaza
fuentes ausentes o de motores prohibidos, verifica límites finitos y positivos
y detecta casos omitidos, duplicados o tolerancias desactualizadas en la matriz.
Sus controles negativos retiran fuente, valores esperados o tolerancia de cada
JSON y ensayan fuentes prohibidas y límites inválidos. El test de fixtures
existente comprueba que el inventario embebido coincide con todos los JSON del
directorio; no puede quedar un archivo sin ejecutar.

Los cinco JSON añaden absolute=relative=1e-10, idénticos al default anterior.
Consolidación mueve los límites existentes a metadatos compartidos y usados
por los ejecutores nativo/wasm: asiento/s∞<0.0015, U relativo<0.005,
presión/100<0.004 y flujo relativo<0.03. No cambia ningún valor esperado,
entrada mecánica, fórmula, operador de aceptación ni tolerancia efectiva.

ENG-REL-001 depende de ENG-VAL-005 y no exige ENG-VAL-003. OpenSees queda
solo como contraste adicional externo, nunca como único oráculo ni requisito
de CI o release. No entran sus archivos ni resultados al repositorio.

## Verificación offline

Se usa la toolchain stable provisionada, `CARGO_NET_OFFLINE=true`,
`PIP_NO_INDEX=1` y un directorio de compilación aislado `$CARGO_TARGET_DIR`.

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 661 passed, 0 failed, 0 ignored, incluidos doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK, incluidos los tres archivos nuevos |

La suite focalizada de engine-validation pasó 24 tests, incluidos los tres
nuevos, y recorre los 91 casos. La comparación de los cinco JSON contra HEAD
confirmó igualdad de `input`, `expected`, `source` y los demás campos previos;
solo se añadió la tolerancia explícita idéntica al default anterior.

El índice Git real es de solo lectura. La primera invocación de clean-room no
contó el provenance nuevo porque todavía era un archivo sin seguimiento.
Se ejecutó la misma guardia con `GIT_INDEX_FILE` apuntando a un índice temporal
bajo `$CARGO_TARGET_DIR/eng-val-005-evidence/`, inicializado desde HEAD y con
entradas para ORACULOS.md, tests/oracles.rs y esta bitácora. Así el diff
inspecciona sus contenidos completos. No se modificó el índice real ni se
alteró la guardia para obtener el resultado.

Determinismo: los 16 hashes publicados en `docs/DETERMINISMO.md` coinciden
exactamente con los calculados; diez corridas por caso en nativo y wasm,
perfil debug, `MAMUT_ENGINE_GIT_COMMIT` sin definir. El script existente
`scripts/check-determinism.mjs` verificó igualdad entre targets, repetibilidad
y cero imports externos. El documento de hashes permanece intacto.

Ejecución wasm adicional, offline y con los ejecutores existentes: seis casos
Winkler, siete de consolidación más convergencia y cinco `fixture/1`, estos
últimos en ambos targets del runner (diez filas aprobadas). Esto ejecuta los
harnesses y documentos cuyos metadatos cambiaron, además de compilarlos.
Logs de comprobaciones, hashes y reportes solo en
`$CARGO_TARGET_DIR/eng-val-005-evidence/`; no se añaden resultados al repo.

`git diff --check`: OK. Sin cambios en bibliotecas numéricas, entradas de
determinismo, Cargo.toml/Cargo.lock ni archivos protegidos.

## Seguimiento de revisión OpenCode · PR #155

- Autor: Codex gpt-6.1-sol high · 2026-10-17 (America/Guayaquil).
- Rol: especificador e implementador de metadatos y guardias.
- Fuentes: solo las derivaciones locales citadas arriba y la solicitud de
  seguimiento; sin red ni consulta de bibliografía externa.
- Declaración clean-room: no consulté ni usé código o resultados de OpenSees,
  Stabileo, CalculiX o LAPACK; no copié tablas, figuras ni código de terceros.
- Revisor clean-room: PENDIENTE (J4).

Las citas de Winkler y consolidación apuntan ahora a secciones concretas de las
derivaciones propias; CQC conserva la evaluación algebraica D11 como fuente.
Las atribuciones no cotejadas se reúnen en «Referencia bibliográfica opcional
(por cotejar)» de ORACULOS.md y no son el oráculo de aceptación.

El inventario se calcula sumando las longitudes de las tablas y fixtures,
incluido el caso espectral multicapa individual. La guardia compara los nombres
en ambos sentidos e informa qué casos faltan o sobran en la matriz y en las
tablas/fixtures, además de rechazar duplicados. Cada fila debe enlazar un ancla
de encabezado existente en el archivo de provenance correspondiente. Los cinco
JSON enlazan la sección de sus derivaciones; multicapa transformable enlaza su
desarrollo específico.

Los controles negativos incluyen enlaces sin ancla, anclas/archivos inexistentes,
diferencias de inventario, citas con la palabra PENDIENTE (sin distinguir
mayúsculas) y justification vacía o compuesta solo de espacios/tabuladores/saltos
de línea en cada fixture/1. «Independiente» sigue siendo texto válido.
Cada JSON justifica ahora el default según escala de los resultados, rigideces,
condicionamiento y ceros del caso. No se modifica el contrato de la biblioteca.

Comparación contra HEAD: los cinco JSON conservan todos sus datos, incluidas
entradas, resultados y límites; solo cambian citation y tolerance.justification.
Las tres tablas modificadas conservan todos sus literales numéricos. No se
modifican bibliotecas, dependencias, archivos protegidos ni hashes publicados.

### Verificación del seguimiento

Toolchain stable rustc 1.99.0, `CARGO_TARGET_DIR=/workspace/.cargo-target/me-val5`,
`CARGO_NET_OFFLINE=true` y `PIP_NO_INDEX=1`. Todo ejecutado sin red:

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | OK |
| `cargo test --workspace --offline` | 665 passed, 0 failed, 0 ignored, incluidos doctests |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline` | OK |
| `cargo check --workspace --offline --target wasm32-unknown-unknown` | OK |
| `bash clean-room/check.sh origin/main` | OK, sin índice alternativo |

Guardia de oráculos: siete tests aprobados, cuatro añadidos en el seguimiento;
el inventario calculado coincide con las 91 filas actuales. Los 16 hashes
recalculados coinciden exactamente con `docs/DETERMINISMO.md`. Cruce debug
nativo/wasm con los ejecutores existentes y `scripts/check-determinism.mjs`:
diez corridas por caso/target, igualdad bit a bit y cero imports externos.
`MAMUT_ENGINE_GIT_COMMIT` sin definir, como en la tabla publicada.

Logs y resúmenes locales en `$CARGO_TARGET_DIR/eng-val-005-followup/`.
`git diff --check`: OK. Cambios sin commitear en el árbol de trabajo.

## Pendientes

Revisión clean-room de J4; cotejo humano de las referencias bibliográficas y
localizadores marcados PENDIENTES en ORACULOS.md. La aceptación numérica usa
las derivaciones locales y no depende de completar esas atribuciones.
