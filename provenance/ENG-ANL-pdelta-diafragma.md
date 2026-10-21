# Bitácora clean-room · ENG-ANL-pdelta-diafragma

- Ticket / PR: J2, GitHub #185, ENG-E-0007; decisión J4 r3.
- Autor(es): J2, ejecutor Codex basado en GPT-6 · 2026-10-20
  (America/Guayaquil).
- Rol: implementador y autor de derivaciones propias, sin delegación.
- Contrato implementado: `PDelta::run` / `run_with`, `pDelta` de engine-input/1;
  proyección con restricciones rígidas y apoyos prescritos.
- Fuentes permitidas usadas: petición del usuario, parche preliminar local
  `val032-r3-engine-pdelta-diaphragm.patch` (revisado y comprobado), código
  propio pertinente de engine-analysis, engine-assembly y frame3d, pruebas
  propias del repositorio, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`,
  formato de `provenance/ENG-ANL-002.md` y referencia NumPy propia VAL-032
  identificada en el [README del fixture](../crates/engine-api/tests/fixtures/pdelta_diaphragm/README.md).
  Sin red ni bibliografía externa.
- Derivaciones propias: ecuaciones y pruebas descritas debajo; integración
  Hermite y solución directa en `reference.py`.
- Código de terceros copiado: ninguno. Ensamblaje adaptado del script propio
  VAL-032; NumPy preinstalado como dependencia de generación BSD-3-Clause,
  sin consultar ni copiar su implementación. Sin dependencias Cargo nuevas.
- Validación: analítica (trabajo virtual, congruencia y multiplicadores de
  Lagrange); E3 con referencia NumPy propia de pretensión inicial consistente.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX,
  code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra
  ni derivados". No se usaron datos ni resultados de OpenSees ni Frame3DD;
  no se abrió `provenance/ENG-BENCH-001-SMOKE-VPS.md` ni archivos en
  `/workspace/audit/`.
- Revisor clean-room: PENDIENTE (J4).

## Derivación y alcance

Con independientes libres q, restricciones afines `u=T_f q+u_0` y trabajo
virtual `δu=T_f δq`, el equilibrio exige
`T_fᵀ[(K+Kg)u-F]=0`. Por tanto, la rigidez de estabilidad es
`T_fᵀ(K+Kg)T_f`, no la submatriz de filas/columnas de maestros. Se acumula
una mitad y se refleja para mantener simetría exacta, igual que el ensamblaje
elástico. La inercia LDLᵀ utiliza esa matriz proyectada.

El punto fijo mantiene el factor elástico y resuelve
`(T_fᵀ K T_f) q_(n+1) = T_fᵀ(F-Ku_0)-T_fᵀ Kg(T_f q_n+u_0)`.
`Constraints::expand` ya recuperaba los esclavos y asentamientos; se conserva.
No cambian axiales, ejes, estados, estrategia ni límites de iteración.

El residuo libre usa `T_fᵀ internal - T_fᵀ force`. Las fuerzas de restricción
en esclavos pueden ser distintas de cero: su trabajo virtual proyectado se
anula. La referencia por grupo fuerza/momento y el piso de redondeo de
ENG-ANL-003 pasan a `|T_f|ᵀ |K+Kg| |u|`. El recuento m de términos incluye
las entradas CSR de todas las filas que contribuyen, mínimo uno. Cuando T_f
solo selecciona GDL libres, se conserva la operación anterior y los hashes.

La prueba unitaria contrasta `T_fᵀ Kg T_f` contra productos densos de columnas
obtenidas perturbando `expand`; verifica simetría exacta. La solución se
contrasta además con un sistema KKT de multiplicadores construido imponiendo
directamente las ecuaciones del diafragma, con y sin maestro prescrito.
El residuo proyectado se anula aunque existan fuerzas físicas de restricción.
Otra prueba comprueba identidad sin diafragma. Las pruebas de integración
usan `Kg=alpha*K`, compresión/tracción, inercia negativa y un solver inexacto
estacionario que debe seguir sin converger.

## E3 y definición de θ

Tras incorporarse #180 a `origin/main` `8a1f21f`, el test Rust y `reference.py`
leen directamente `runs[id=E3-pdelta].engine_input` de
`docs/validation-report/fixtures/val032/E3-v1.1.1.json`. Se eliminó la copia
local `e3.json`, antes extraída de `2177dee615fc925ebedac392423d96202e029e22`:
su entrada coincide numéricamente con la de main. El fixture contiene
72 nodos, 145 miembros y cinco diafragmas; conserva sus opciones originales
(tolerancia 1e-10, máximo 100 pasos).

La referencia original solo calculaba θ lineal. El nuevo script conserva su
ensamblaje y obtiene los axiales de la solución de gravedad. Integra
`N ∫ H′_a H′_b dx` por cuadratura exacta de tres puntos y resuelve
directamente la matriz total proyectada. Incluye la Kg consistente de todos
los miembros, sin aproximar por P/L. El archivo de valores guarda la huella
del contenedor (`fixture_sha256`), el run (`run_id`), estabilidad, residuo y
ambos θ. Se regeneró `reference.json` con Python/NumPy: al excluir la antigua
huella `input_sha256` y los dos metadatos nuevos, el documento anterior y el
nuevo son idénticos, incluidos todos los valores de los cinco pisos. No se
modifican los JSON, herramientas, CSV ni hashes congelados de VAL-032.

Se compara `theta_pdelta=P_i*Delta_PD,i/(V_EX,i*h_i)`, con Δ entre centros
de masa de pisos consecutivos, gravedad acumulada desde el piso a cubierta,
cortante EX aplicado acumulado y altura de entrepiso. La respuesta lateral
es independiente de gravedad, igual que en el Engine. Los valores lineales
se guardan aparte y reproducen los del modelo propio VAL-032. Criterio de
aceptación: error relativo por piso ≤1e-4, tanto para θ P-Δ como para deriva.
Generación y ecuaciones completas en el README del fixture.

`val032_runs.rs` acepta E3-pdelta entre los cuatro runs ejecutables, sin errores
y solo con avisos revisados. Conserva los ocho bloqueos modales/espectrales de
#184 con `Unstable`/`unstable` y error único `ENG-E-0006`; también comprueba el
estado serializado de los ejecutables. El README de VAL-032 distingue el estado
actual del registro histórico r3, que mantiene bytes y hashes de main.

Se ejecutaron cinco mutaciones temporales y se restauraron las fuentes byte
a byte: restaurar `pdelta.rs` de main (sin proyección), alterar el estado modal,
alterar su código de error, añadir un aviso sin revisar y omitir E3-pdelta.
Las cinco compilaron y fueron rechazadas por `val032_runs` (salida 101 por
aserción fallida, sin confundir fallos de compilación con detección).
Registros: `/tmp/pdelta-main-mutations.log` y
`/tmp/pdelta-main-mutation-<nombre>.log`.

## Verificación de la sesión

Todas las ejecuciones son offline. En esta continuación sobre `origin/main`
`8a1f21f` se usa `CARGO_TARGET_DIR=/tmp/pdelta-main-target` para nativo y
`/tmp/pdelta-main-target-wasm` para wasm. El directorio de compilación de la
sesión anterior ya no existe y su padre no permite crearlo en este sandbox.

| Comando / comprobación | Resultado |
| --- | --- |
| `cargo fmt --all -- --check` | OK, salida 0 |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | OK, salida 0; cero advertencias |
| `cargo test --offline --workspace` | OK, 708 pruebas y doctests; 0 fallidas, 0 ignoradas, 112 ejecutables/documentaciones |
| `cargo check --offline --workspace --target wasm32-unknown-unknown` | OK, salida 0 |
| `cargo build --offline --release -p engine-api --target wasm32-unknown-unknown --example determinism_wasm` | OK, salida 0 |
| `cargo run --offline --locked -p engine-api --example determinism_native` + `cmp` con la base anterior al cambio | 160/160 hashes idénticos (16 casos × 10 corridas) |
| `node scripts/check-determinism.mjs <wasm> <hashes-native>` | OK, 16/16 casos, 10 comparaciones por caso, nativo=wasm bit a bit |
| Comparación de bytes con `git show origin/main:<ruta>` | 49/49 fixtures JSON preexistentes idénticos, incluidos 18/18 JSON de VAL-032; 25/25 archivos VAL-032 salvo README idénticos (también herramientas y CSV) |
| `bash clean-room/check.sh origin/main` con índice temporal | Clean-room: OK, salida 0; incluye los seis archivos nuevos |
| `git diff --check origin/main` con índice temporal | OK, salida 0; once archivos revisados |
| Cinco mutaciones temporales de regresión y documentos de salida | 5/5 rechazadas por el test; fuentes restauradas |

Las cinco pruebas nuevas de #185 están incluidas en las 708: dos unitarias de
proyección, dos de integración con diafragmas y una regresión E3; el total
incluye ahora el test VAL-032 incorporado por #180. Se ejecutó también
`cargo test --offline -p engine-api --test val032_runs --test pdelta_diaphragm -- --nocapture`:
2 tests passed, 0 failed; VAL-032 comprueba doce runs, cuatro `Warning` y
ocho `Unstable`/ENG-E-0006, con 31 avisos revisados.
**E3-pdelta converge en 14 iteraciones**, error relativo máximo de θ
**3.539303943e-11** (piso 1), frente al umbral 1e-4. Solo conserva el aviso
preexistente `API-W-DIAPHRAGM-RECOVERY`; no emite ENG-E-0007. La referencia
directa tiene residuo relativo proyectado 2.2440244542387273e-13.

Compilación/pruebas nativas: rustc 1.85.1; Clippy 0.1.99. El destino wasm
se comprobó y ejecutó con el toolchain preinstalado rustc 1.99.0 mediante
`PATH=/home/box/.cargo/bin:$PATH`. No se descargó ni instaló nada.
Python 3.13.5 / NumPy 2.5.4 generan la referencia. Registros de comprobación
en `/tmp/pdelta-main-{fmt,clippy,workspace-test,wasm-check,wasm-build,native-wasm,clean-room,e3-val032,fixtures,mutations}.log`;
referencia regenerada en `/tmp/pdelta-reference-main.log`, comparación numérica
con `/tmp/pdelta-reference-before-main.json`. Hashes base en
`/tmp/pdelta-baseline-hashes.txt` y actuales en `/tmp/pdelta-main-current-hashes.txt`.

Sin commit, push ni rebase. Se copió el índice a
`/tmp/pdelta-main-review-<id>/index`, se ejecutó `git add -N` para los seis
archivos nuevos y se usó `GIT_INDEX_FILE` para clean-room y la revisión del
diff; los objetos nuevos también se aislaron en ese directorio de `/tmp`.
Se comprobó la huella del índice real antes/después: permanece intacto, y los
archivos nuevos aparecen sin seguimiento en el `git status` habitual.
No se añadieron `unwrap`/`expect` en `src`. Clean-room automático satisfactorio;
revisión humana de J4 pendiente.
