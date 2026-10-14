# Bitácora clean-room · ENG-LOAD-002-FU

- Ticket / PR: issue #70, seguimiento J4 r15 de #54, ENG-LOAD-002;
  rama `j2/fu-70-settlements` desde `main`. Corrección antes de CLI/API.
- Autor(es): Codex gpt-6.1-sol high · 2026-10-14 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: `engine_analysis::combination::combine` y metadato
  `NodeResult::prescribed_displacements`, recuperado por `CaseResults::recover`.
  Sin nuevo contrato elemental, dependencias, CLI/API ni serialización.
- Fuentes permitidas usadas: petición de issue #70 en esta sesión, catálogo y
  APIs propias del repositorio (`engine-analysis`, `engine-assembly`,
  `engine-elements-frame`, `engine-core`), `provenance/TEMPLATE.md` y bitácora
  original `provenance/ENG-LOAD-002.md`. Sin red ni fuentes externas.
- Derivaciones propias: nota analítica incluida abajo y
  [pruebas](../crates/engine-analysis/tests/combination.rs).
- Código de terceros copiado: ninguno.
- Validación: analítico; viga con giros nulos y asentamiento relativo, equilibrio
  y superposición con caso independiente. Oráculo derivado a mano.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Revisor clean-room: PENDIENTE (J4).

## Decisión y alcance

Se elige el rechazo conservador. La respuesta con desplazamientos prescritos
es afín, `r(F,p) = r(0,p) + L(F)`. Por tanto, sumar los resultados completos de
dos casos con el mismo asentamiento pondera el aporte independiente de la carga
por `1.2 + 1.6 = 2.8`. El contrato actual conserva resultados completos; no
separa `r(0,p)` en desplazamientos, reacciones, acciones ni estaciones. Separarlo
automáticamente requeriría otro análisis o ampliar ese contrato para guardar
todos esos campos por separado. El rechazo evita esa ampliación y no intenta
inferir apoyos o asentamientos a partir de una reacción o desplazamiento libre.

La única extensión de datos es `[Option<f64>; 6]` por nodo para conservar qué
GDL eran prescritos y su valor, incluidos ceros; `None` significa libre,
dependiente o ausente. `recover` obtiene ese dato de las restricciones del
sistema, sin aplicar el factor de carga. Los constructores manuales de
`NodeResult` deben suministrarlo; esto cambia su inicialización en Rust, sin
introducir un esquema serializado. Los apoyos/GDL prescritos deben coincidir
entre términos; es responsabilidad del llamador mantener la misma rigidez y
transformación de restricciones, que los resultados no contienen.

`combine` devuelve ENG-E-0010 con las dos entidades `Case` cuando más de un
término contiene algún prescrito no nulo, sean iguales, diferentes u opuestos,
incluso con factor cero. También rechaza conjuntos de GDL prescritos distintos
o un metadato que difiera del desplazamiento nodal, indicando el caso afectado.
Los metadatos no finitos producen ENG-E-0008 con entidad `Case`, como otros
datos no finitos. Se usa ENG-E-0010 siguiendo la validación de compatibilidad
ya existente en este módulo y en `CaseResults::recover`, sin ampliar el catálogo.
No se exige igualdad de valores entre un caso de cargas con prescrito cero y
el único caso de asentamiento. Los casos ajenos a los términos no activan este
rechazo; las envolventes pueden comparar varios casos con asentamiento.

Tratamiento explícito: resolver D y L con todos los prescritos cero, resolver
S sin cargas y con el asentamiento, conservando los mismos GDL prescritos y
rigidez, y combinar `1.2D + 1.6L + αS`. `α=1` cuenta el asentamiento una vez;
otros factores son decisión explícita del usuario. Un único término con cargas
y asentamiento sigue ponderando toda su respuesta por su factor. El resultado
combinado conserva los prescritos ponderados para validar una combinación
posterior, incluido `α=0`. No se modifica la suma f64 ordenada ni el
emparejamiento por id. No se añaden `panic!`, `unwrap` ni `expect` a la biblioteca.

Esta decisión sustituye, para `combine`, el comportamiento de suma de todos
los asentamientos descrito en la bitácora original ENG-LOAD-002.

## Derivación analítica propia

Viga Euler–Bernoulli horizontal, longitud L, rigidez EI constante, sin carga
transversal, con `v(0)=0`, `v(L)=δ`, `v'(0)=v'(L)=0`. Integrar `v''''=0` y
aplicar esas cuatro condiciones da, con `t=x/L`:

`v(x) = δ(3t² − 2t³)`.

Con la convención local propia `M=EI v''`, `V=−M'`:

`M(x) = 6EIδ/L² (1−2t)`, `V(x) = 12EIδ/L³`.

Los momentos de acción en ambos extremos son `−6EIδ/L²`; las fuerzas verticales
son `−12EIδ/L³` en i y `+12EIδ/L³` en j. Equilibrio: suma de fuerzas cero y
`Vj L + Mi + Mj = 0`. Se deja libre `Ux` en j y se fija en i para conservar
un GDL libre; su respuesta independiente bajo fuerza P es `u_x(L)=PL/(EA)`,
`N=P` y reacción axial `−P` en i.

Fixture: L=2, E=16, A=2, I=2, δ=0.125 (SI). El momento inicial es 6 N·m y
el cortante 6 N. D tiene P=48 N y L tiene P=−16 N. Con asentamiento en ambos,
la suma anterior produciría 16.8 N·m y δ=0.35 m; ahora se rechaza. Con cargas
homogéneas y S propio, `α=1` conserva 6 N·m y δ=0.125 m mientras la fuerza
axial combinada es `1.2·48 − 1.6·16 = 32 N`.

Se comprueban `α` en [1, 0.5, −1, 0], nodos, reacciones, acciones y todas
las estaciones frente a estas fórmulas y un análisis directo con cargas y
asentamiento combinados. Tolerancia: `128·EPSILON·max(1,|esperado|)`, acorde con
la escala del fixture. El rechazo cubre asentamientos iguales, opuestos y
dobles, factores [1.2,1.6], [1,0], [0,0], [1,−1], metadatos incoherentes,
no finitos y conjuntos de GDL distintos.

Además se comprueba la suma ordenada con [1e16, −1e16, 1]: resultado 1;
con [1e16, 1, −1e16]: resultado 0, en todos los campos y extremos, con
repositorio invertido. Se refuerza la prueba por id con valores distintos entre
nodos y miembros y entre casos, orden invertido y factores 2 y −0.5.

## Verificación

Todos los checks de código se ejecutaron sin red (`CARGO_NET_OFFLINE=true`) y
con `CARGO_TARGET_DIR=/workspace/.cargo-target`, después de restaurar la corrección:

- `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, 388 pruebas entre 65 suites, sin fallos ni
  ignoradas; las 15 pruebas de combinación incluyen cuatro nuevas y una
  reforzada para emparejamiento por id.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK, usando
  `/home/box/.cargo/bin/cargo` y `RUSTC=/home/box/.cargo/bin/rustc` (1.99.0),
  con std wasm ya instalada; sin descargar componentes. Los checks nativos
  usan rustc/cargo 1.85.1. Se verifica compilación wasm, no ejecución.
- `bash clean-room/check.sh origin/main`: OK, incluyendo la bitácora nueva
  mediante índice y objetos temporales en `/tmp`; `git diff --cached --check`: OK.

Para comprobar regresión se sustituyó temporalmente **todo**
`crates/engine-analysis/src/combination.rs` por su versión en HEAD, conservando
el campo nodal de metadatos y su recuperación para que las pruebas compilen.
`cargo test -p engine-analysis --test combination` salió con código 101:
12 pruebas pasaron y estas tres fallaron por aserción (sin fallo de compilación):

- `repeated_settlement_rejected_against_fixed_end_beam_oracle`: devolvió éxito
  con M=16.8 N·m y δ=0.35 m, en lugar de rechazar el asentamiento duplicado.
- `dedicated_settlement_case_counted_once_with_explicit_factor`: el resultado
  conservó `Some(0)` del primer caso en lugar del prescrito combinado `Some(0.125)`.
- `incoherent_prescribed_metadata_rejected_with_case_entity`: aceptó metadatos
  incoherentes, en lugar del error esperado.

Las dos pruebas adicionales comprueban comportamientos ya correctos en HEAD;
por eso se validaron con regresiones deliberadas independientes: ordenar los
términos por nombre hizo fallar la cancelación (1 en lugar de 0); acumular por
posición original de entrada hizo fallar la prueba por id con valores distintos.
Ambas ejecuciones salieron con código 101 por aserción. Todas las variantes
temporales se restauraron byte a byte antes de los checks completos.

El índice real del worktree está montado en solo lectura en
`/workspace/mamut-engine/.git/worktrees/me45`. `git add` falló al crear
`index.lock` (Read-only file system); `git commit` también falló por ese motivo.
Esto impide completar el commit solicitado
en este entorno. No se hizo push. La revisión clean-room sigue PENDIENTE (J4).
Se comprueba `bash clean-room/check.sh origin/main` con una copia temporal del
índice y objetos en `/tmp`, incluyendo los cuatro archivos; el índice real
permanece intacto. Se prepara un parche completo en `/tmp/ENG-LOAD-002-FU.patch`.

Nota J2 (2026-10-14): el commit lo creó J2 con el mismo contenido verificado; el bloqueo anterior era solo del sandbox.
