# Implementar un método en Rust · ENG-CORE-011

Esta guía permite pasar de una ficha matemática a una implementación revisable.
El contrato vigente manda: no se cambia para acomodar un método. Se trabaja
desde la raíz del worktree, sin red, con herramientas y dependencias provisionadas.
El [protocolo clean-room](CLEAN_ROOM.md) es obligatorio; esta guía no lo sustituye.

## 1. Preparar la sesión y elegir el contrato

Lee, en este orden:

1. [CLEAN_ROOM.md](CLEAN_ROOM.md), especialmente §§3–5, 7, 8 y 10, y
   [provenance/TEMPLATE.md](../provenance/TEMPLATE.md).
2. [engine-core](../crates/engine-core/src/lib.rs): `Real = f64`, `EResult`,
   `EngineError`, `ContractId`; [catálogo](../crates/engine-core/src/catalog.rs)
   y [funciones escalares](../crates/engine-core/src/math.rs).
3. [traits](../crates/engine-contracts/src/traits.rs),
   [tipos de intercambio](../crates/engine-contracts/src/types.rs) y
   [conformidad ENG-CORE-007](../crates/engine-contracts/src/conformance.rs).
4. [engine-laws](../crates/engine-laws/src/lib.rs), sus módulos y pruebas.
   Revisa qué existe antes de elegir una ley: una parametrización de una ley
   existente no constituye un método nuevo.
5. [benchmarks ENG-CORE-009](../crates/engine-validation/README.md),
   [schema fixture/1](../crates/engine-validation/schemas/fixture-1.json) y
   [determinismo](DETERMINISMO.md).

Registra rol de especificador/implementador y fuentes locales consultadas.
Si hubo exposición a fuentes o informes prohibidos, no implementes: aplica
las restricciones de roles del protocolo. No se inspeccionan repos hermanos.
Las fuentes son derivaciones propias o teoría publicada aportada localmente
con cita verificable; no se inventan páginas, ecuaciones ni resultados.

Configura la sesión (el entorno proporciona un `$CARGO_TARGET_DIR` exclusivo):

```sh
export PATH=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH
: "${CARGO_TARGET_DIR:?Configura el directorio de compilación autorizado}"
export CARGO_TARGET_DIR
export CARGO_NET_OFFLINE=true
export PIP_NO_INDEX=1
rustc --version  # toolchain del ticket: 1.99.0
git status --short
```

## 2. Escribir la ficha antes del código

Copia [FICHA_METODO.md](plantillas/FICHA_METODO.md) y completa hipótesis,
ecuaciones numeradas, derivación de la tangente, cita de cada fuente,
unidades, signos, dominio físico y numérico, historia y casos analíticos.
No copies tablas, figuras ni listados publicados. Evalúa las fórmulas por tu
cuenta para fijar los esperados **antes** de ejecutar el método.

Para `uniaxial/1`, el contrato concreto es:

| Aspecto | Obligación |
| --- | --- |
| Entrada | Deformación **total** adimensional; tensión positiva en tracción. Si representa un resorte: desplazamiento relativo en m y fuerza en N, declarados por el kind. |
| Salida | `UniaxialResponse { stress, tangent, trial_state }`; tangente dσ/dε en Pa (o N/m), todo finito. |
| Estado | `state_len()` exacto; historia inicial y de prueba planas y finitas. Documentar orden/unidades de componentes. |
| Propiedad | `respond(&self, strain, committed)` no muta el objeto ni la historia. El llamador acepta con `committed = response.trial_state`; descartar una prueba equivale a rollback. |
| Repetición | Mismas entradas producen la misma respuesta; repetir una deformación aceptada reproduce su tensión. |
| Origen | `initial_tangent()` coincide con `respond(0, initial_state).tangent`. |
| Identidad | `ContractId::new("uniaxial", 1, 0)` y kind propio estable; versión semántica del método independiente del contrato. |
| Errores | `ENG-E-0008`: NaN, infinitos o desbordamiento; `ENG-E-0010`: parámetros, tamaños o rangos inválidos. Mensajes con magnitud y unidades. |

Un método de sección usa `SectionLaw` y `check_section_law`; un elemento,
`Element`/`ElementFactory` y `check_element`/`check_element_factory`; un análisis,
`AnalysisMethod`, `SystemView` y sus comprobadores. Sus tamaños, unidades y
convenciones se toman del trait específico, no de la tabla uniaxial.

## 3. Crear el módulo y exponerlo como los existentes

La opción habitual para otra ley es un módulo de `engine-laws`; no necesita
otro crate ni modificar manifests. Copia
[modulo_uniaxial.rs.tpl](plantillas/modulo_uniaxial.rs.tpl) a
`crates/engine-laws/src/uniaxial/mi_metodo.rs`, sustituye nombres, parámetros
y la respuesta pendiente por las ecuaciones de la ficha. La plantilla se
detiene con `ENG-E-0013`: no es una ley validada ni debe publicarse sin completar.

Mantén parámetros privados validados, constructor falible, `Params`, fábrica
tipada `UniaxialFactory` y constante `LawDescriptor`. Reutiliza los validadores
privados `finite`, `positive` y `vector` de `engine-laws`. Valida longitud antes
de indexar y todas las salidas/intermedios susceptibles de desbordamiento.
Documenta la API pública en español; la biblioteca exige `missing_docs`.
No añadas `unsafe`, `unwrap`, `expect`, `panic!`, estado global, E/S ni hilos.
Las funciones escalares se invocan mediante `engine_core::math`; conserva el
orden de operaciones/reducciones de los métodos existentes.

Expón `mod mi_metodo; pub use mi_metodo::{...};` desde `uniaxial.rs` y actualiza
la documentación de `lib.rs`. Este es el registro de leyes actual: descriptor,
fábrica y exportaciones públicas, sin registro global. No implica soporte JSON
en engine-api ni convertir el análisis lineal en no lineal. Una extensión de
la API requiere su propio contrato y validación explícitos.

Si un contrato exige otro crate, justifica primero esa separación; usa edición,
licencia y versión mínima del workspace, dependencias internas ya existentes y
`#![forbid(unsafe_code)]` en la raíz. Solo entonces adapta los manifests y lock.

## 4. Conformidad y benchmark analítico

Copia [conformidad.rs.tpl](plantillas/conformidad.rs.tpl) y
[fixture_uniaxial.rs.tpl](plantillas/fixture_uniaxial.rs.tpl) a `engine-laws/tests/`.
Los nombres de importación/constructor y los casos se completan desde la ficha.
La tabla vacía de la plantilla falla intencionadamente: evita validación vacua.

Ejecuta `check_uniaxial_law(&law, &samples, h)` y exige cero diagnósticos.
El kit acepta explícitamente historia entre muestras y compara tangente con
diferencia central a historia aceptada fija, con error relativo < 1e-6.
Escoge `h` dentro de los límites publicados en `conformance.rs`, con escala
adecuada y muestras cuyo intervalo ε ± h sea suave. En vértices o fluencia,
prueba por separado la convención unilateral, sin FD central cruzando ramas.
No relajes el kit para que pase una implementación.

Añade negativos para cada parámetro/rango, NaN/±∞ en entradas e historia,
longitudes incorrectas y desbordamiento, verificando códigos del catálogo.
Comprueba fábrica/descriptor/objeto, `Send + Sync`, pureza y propiedad del estado.
El kit verifica contrato y consistencia; un error compartido por tensión y
tangente puede pasar. Por eso el benchmark usa esperados analíticos independientes.

El fixture uniaxial tipado contiene `source`, cita, parámetros SI, entradas,
esperados y tolerancia justificada; se ejecuta como prueba nativa de la ley.
No es `fixture/1`: ENG-CORE-009 exige `engine-input/1 → results/1` mediante
`Target`, y la API actual no publica estas leyes no lineales. No fabriques un
JSON que el runner no pueda ejecutar ni añadas otro caso al inventario cerrado
de cinco JSON existente. Cuando el método sea accesible por API, usa
[fixture_api.json.tpl](plantillas/fixture_api.json.tpl), completa la entrada
real y los JSON Pointers y ejecútalo con el runner nativo de ENG-CORE-009.
Ese formato exige `source.kind` y cita, y compara con
`|actual−esperado| ≤ max(absolute, relative·|esperado|)`; un override requiere
justificación. No cambies tolerancias a partir de salidas de otros motores.

## 5. Piloto reproducible: elasticidad cúbica

Ficha propia: Codex gpt-6.1-sol high, *Elasticidad cúbica conservativa*,
2026-10-20, esta sección, ecuaciones (1)–(3) y tabla siguiente.
No se consultaron fuentes externas. Antes del piloto existían `Elastic`,
`MultilinearElastic` y `SteelBilinearKinematic`; la multilineal ya representa
curvas bilineales, y la cinemática permite plasticidad perfecta con b = 0.
Ninguna representa exactamente una curva cúbica suave con C > 0.

Se propone una densidad de energía propia (Pa = J/m³), sin historia:

```text
(1) W(ε) = E ε²/2 + C ε⁴/4,       E > 0, C ≥ 0, ambos en Pa.
(2) σ(ε) = dW/dε = Eε + Cε³,       σ positiva en tracción.
(3) K(ε) = dσ/dε = E + 3Cε² > 0,  K(0) = E.
```

W es par, no negativa y estrictamente convexa; σ es impar. La descarga sigue
la misma curva y el trabajo en un ciclo cerrado es cero. Se trata de una
idealización elástica local 1D, sin plasticidad, daño ni validez física universal
para grandes deformaciones. Historia inicial/aceptada/de prueba: longitud cero.
C = 0 recupera la ley lineal como límite, sin sustituir su implementación.

Implementación fijada por la ficha: `q = (C·ε)·ε`, `σ = (E+q)·ε`,
`K = E+3q`. Cada producto/suma se exige finito; un desbordamiento devuelve
`ENG-E-0008`. Se evita formar ε² antes de multiplicar por C para que C = 0
admita deformaciones grandes cuando la respuesta siga siendo representable.
Se admite el redondeo/subdesbordamiento habitual de f64; no se certifica
exactitud relativa de términos por debajo de la resolución del número.

Benchmark analítico propio con E = 8 Pa, C = 16 Pa. Sustituyendo fracciones
binarias en (2)–(3) se obtiene, antes de correr la implementación:

| ε total | σ (Pa) | K (Pa) |
| --- | --- | --- |
| 0 | 0 | 8 |
| 1/4 | 9/4 | 11 |
| 1/2 | 6 | 20 |
| 1 | 24 | 56 |
| 1/2 (descarga) | 6 | 20 |
| 0 (descarga) | 0 | 8 |
| −1/4 | −9/4 | 11 |
| −1/2 | −6 | 20 |
| −1 | −24 | 56 |
| 0 (cierre) | 0 | 8 |

Se usa piso absoluto 1e-12 Pa y relativo 1e-12: holgura sobre el redondeo f64
de pocas operaciones; no proviene de calibración externa. Para FD, h = 1e-6:
el error de truncamiento central es exactamente C h² = 1,6e-11 Pa.

Aplicación de los pasos 1–4: ficha anterior escrita primero; módulo
[`cubic_elastic.rs`](../crates/engine-laws/src/uniaxial/cubic_elastic.rs),
kind `elastic.cubic`, versión 0.1.0, contrato `uniaxial/1`, `CubicElasticParams`,
`CubicElasticFactory`, descriptor `CUBIC_ELASTIC` y exportaciones públicas.
La prueba [`cubic_elastic.rs`](../crates/engine-laws/tests/cubic_elastic.rs)
instancia el kit y ejecuta la tabla nativa con source/cita, además de negativos,
pureza, simetría y límite lineal. No requiere solver, API ni dependencias nuevas.

```sh
cargo test --offline -p engine-laws --test cubic_elastic
```

La evidencia completa y el estado de revisión están en
[ENG-CORE-011](../provenance/ENG-CORE-011.md). El piloto verifica esta guía en
la misma sesión autora; la revisión independiente sigue pendiente de J4.

## 6. Cerrar y revisar

Completa [PROVENANCE.md](plantillas/PROVENANCE.md), añade CHANGELOG y aplica la
[lista de revisión](plantillas/REVISION.md). Para una IA, parte del
[prompt restringido](plantillas/PROMPT_IA.md). Mantén los cambios sin commit.

Ejecuta todas las comprobaciones desde la raíz:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace --offline
cargo check --workspace --offline --target wasm32-unknown-unknown
bash clean-room/check.sh origin/main
```

El comprobador clean-room solo incluye archivos conocidos por Git: para cubrir
los nuevos sin alterar el índice real, usa un índice temporal bajo
`$CARGO_TARGET_DIR`, inicializado con `git read-tree HEAD`, añade allí las rutas
del entregable y repite el comando con `GIT_INDEX_FILE` apuntando a ese índice.
Si los objetos de `.git` son de solo lectura, configura además
`GIT_OBJECT_DIRECTORY` bajo `$CARGO_TARGET_DIR` y
`GIT_ALTERNATE_OBJECT_DIRECTORIES` con la salida de
`git rev-parse --path-format=absolute --git-path objects` obtenida antes de
exportar esa configuración. Así los objetos nuevos también son temporales.
Conserva los logs y no cambies listas ni workflows para eludir una comprobación.

Verifica además los 16 hashes contra la tabla de DETERMINISMO.md, con
`MAMUT_ENGINE_GIT_COMMIT` sin definir, los ejemplos `determinism_native` y
`determinism_wasm`, y `scripts/check-determinism.mjs`; ejecuta nativo/wasm y
debug/release como indica ese documento. Guarda las salidas en
`$CARGO_TARGET_DIR`; la tabla de hashes no se regenera para aceptar diferencias.
`cargo check` wasm solo comprueba compilación; no acredita ejecución wasm del
piloto. Reporta por separado ejecución nativa, cruce de hashes y pendientes.

## 7. Fronteras CLI, Python y WASM: MAM-835 / ENG-SFC-010

Las capacidades siguientes se ejecutan con `engine-input/1` mediante la misma
función `engine_api::run`. CLI conserva su sobre de workflow y coloca el
resultado en `engineResults[0]`; Python `run(...).to_json()` y WASM `run(...)`
entregan el documento canónico `results/1`. Los campos añadidos son opcionales:
las entradas anteriores conservan sus bytes y los 16 hashes publicados en
[DETERMINISMO.md](DETERMINISMO.md). La procedencia `target` sigue identificando
el destino de compilación: la igualdad del documento completo entre superficies
se verifica compilándolas para el mismo destino, incluida la función WASM en
nativo. No se elimina `target` de los resultados para simular igualdad entre
binarios de destinos diferentes.

Los métodos `nonlinearStatic`, `soilProfile` y `consolidation1d` figuran como
`Experimental` en `capabilities` y en los descriptores del registro hasta que
ENG-VAL-006 cubra su validación (decisión de Jonathan vía J0 2026-10-20).
Para ejecutarlos, añada `"options": {"allow_experimental": true}` en la raíz
del documento `engine-input/1`, junto a `analyses`; no dentro de las opciones
del análisis. Sin esa habilitación explícita, `run` devuelve `invalid`
(`API-E-INPUT`, salida 2). Los fixtures SFC-010 de estos métodos ya la incluyen.
Las fuentes, soluciones analíticas y tolerancias de los fixtures
están en [ENG-SFC-010](../provenance/ENG-SFC-010.md). No se modifica el núcleo
numérico ni se añade plasticidad de sección.

### 7.1. Winkler y liberaciones de extremo

En `model.members[]`, seleccione `element: "frame2d.winkler"` y añada
`"winkler": {"k": 10}`. Para `frame3d.winkler`, use
`"winkler": {"ky": 10, "kz": 20}` y la orientación 3D habitual. Las rigideces
son N/m² por longitud de viga, en las direcciones transversales locales.
El lecho es bilateral, lineal, sin masa adicional. Se conservan las propiedades
Euler de sección/material y los requisitos previos de `rho` y orientación.
Las fábricas existentes también aceptan estas rigideces entre los parámetros
escalares de sección/ley; no duplique una propiedad dentro y fuera de `winkler`.

Las vigas `frame2d.euler_elastic` aceptan `release_i` y `release_j` booleanos
que liberan el momento z local de cada extremo. En 3D los campos son arrays de
tres booleanos `[T, My, Mz]`, siempre locales. No liberan traslaciones.
Los giros nodales liberados quedan desconectados: restrínjalos o conéctelos con
otros elementos para evitar un mecanismo. Las cargas uniformes y los giros
internos recuperados usan la condensación existente. Winkler no admite estas
liberaciones: se devuelve `API-E-SFC-OPTIONS`, sin sustituir el elemento.

Ejemplos completos:
[Winkler 2D](../crates/engine-api/fixtures/input/valid/sfc010-winkler2d.json),
[Winkler 3D](../crates/engine-api/fixtures/input/valid/sfc010-winkler3d.json),
[liberaciones 2D](../crates/engine-api/fixtures/input/valid/sfc010-release2d.json),
[liberaciones 3D](../crates/engine-api/fixtures/input/valid/sfc010-release3d.json).
La solución constante `v=q/k` satisface exactamente la ecuación de
Winkler–Hetényi `EI v''''+kv=q` y pertenece al espacio Hermite del elemento.

### 7.2. Resortes y leyes uniaxiales

La colección opcional `model.springs` no exige sección. Cada resorte declara
`id`, `nodes`, `kind` y un `dof` local. Un nodo conecta a tierra; dos nodos
distintos conectan entre sí y pueden tener coordenadas coincidentes. Los IDs
comparten espacio con `model.members`. Cada resorte representa un único GDL;
resortes independientes requieren objetos separados.

```json
{"id": 21, "nodes": [1, 2], "kind": "spring.linear", "dof": "Ux", "k": 200}
```

`spring.linear` exige `k` ≥ 0 y no admite `law`; `spring.nonlinear` exige la
referencia `law` y no admite `k`. Los ejes son globales si se omiten ambos campos
de orientación. `direction: [x,y,z]` selecciona el eje de Ux local; `rotation`
es una matriz 3×3 cuyas filas son ejes locales ortonormales. Son excluyentes.
El constructor existente comprueba dimensión, ejes y GDL globales necesarios.
No se deduce una dirección de las coordenadas.

Las leyes se declaran en `model.laws` con `name` y `parameters` habituales, más
un `kind` opcional:

| Kind | Parámetros | Respuesta |
| --- | --- | --- |
| `elastic` | `E` | `F=Eδ` |
| `steel.bilinear_kinematic` | `E`, `fy`, `b` | Pendiente posfluencia `bE`, `0≤b<1`; historia `[δp,α]` y descarga cinemática |
| `multilinear_elastic` | `parameters: []`; `points: [[δ,F],...]`; `mode` opcional | Tabla creciente que contiene `[0,0]`, pendientes no negativas, extrapolación con el tramo extremo |
| `elastic.cubic` | `E`, `C` | `F=Eδ+Cδ³`, `E>0`, `C≥0` |

Los valores de `mode` son `both` (por defecto), `compression_only` y
`tension_only`. No se reflejan puntos ni se supone simetría. En resortes la ley
se interpreta directamente como fuerza–desplazamiento: δ en m, F en N y E como
rigidez N/m. Para giros se interpreta momento–giro en N·m/rad. No se convierten
automáticamente curvas tensión–deformación ni se multiplican por área/longitud.
Las leyes no lineales solo se conectan a `spring.nonlinear`; una viga elástica
con esa referencia devuelve `ENG-E-0013`.

Resortes lineales se admiten en `linearStatic` (incluidas combinaciones) y
`nonlinearStatic`. Los no lineales requieren `nonlinearStatic`. Otros análisis
con resortes devuelven `ENG-E-0013`. Las identidades de salida aparecen en
`members`: `section` vacío, referencia de ley si existe y `end_dofs` con las
etiquetas de recuperación. A tierra el extremo móvil se llama `j`. Las tablas
Python agrupan resortes por kind y etiquetas para conservar sus anchos y unidades.

Ejemplos: [serie lineal](../crates/engine-api/fixtures/input/valid/sfc010-spring-series.json),
[resortes 3D](../crates/engine-api/fixtures/input/valid/sfc010-spring3d.json),
[multilineal](../crates/engine-api/fixtures/input/valid/sfc010-multilinear.json),
[cúbico](../crates/engine-api/fixtures/input/valid/sfc010-cubic.json) y
[acero con descarga/reversión](../crates/engine-api/fixtures/input/valid/sfc010-steel.json).

### 7.3. Newton completo con control de carga

```json
{"method": "nonlinearStatic", "options": {
  "case": "P", "load_factors": [0.25, 0.5, 1.0],
  "tolerance": 1e-10, "force_scale": 16, "max_iterations": 30
}}
```

`load_factors` contiene factores **absolutos**, no incrementos acumulativos.
Se permiten cero, negativos, repeticiones y descarga. De uno a 16384 pasos y
correcciones máximas por paso. `tolerance` controla `‖R_libre‖∞/force_scale`.
Las escalas SI positivas son responsabilidad del usuario, especialmente si
conviven fuerzas y momentos. Opcionalmente se pueden añadir
`displacement_tolerance` con `displacement_scale`, y `energy_tolerance` con
`energy_scale`. Cada criterio adicional exige su escala; deben cumplirse todos
los activos. Las escalas de criterios inactivos se serializan como 1 si se omiten.

Se conserva un único `NewtonSystem` con su historia aceptada durante todos los
pasos. Los resortes variables se excluyen de la matriz lineal para evitar contar
su rigidez dos veces. Los desplazamientos prescritos son constantes, sin factor
de carga. Este método no admite diafragmas ni combinaciones (`ENG-E-0013`).
No hay búsqueda de línea, subdivisión automática ni reintentos.

`convergence.steps[]` conserva el factor, iteraciones, normas absolutas y
relativas, `converged`, y para cada paso aceptado `results` con nodos/miembros
e `element_states` en el orden de los resortes no lineales del input. Las
reacciones incluyen las fuerzas de los resortes variables. Los extremos de
miembro se muestrean en las cinco estaciones del dominio; no se prometen
extremos interiores exactos en este análisis. `cases[0]` contiene el último paso
si toda la ejecución termina.

Si no converge, `results` y `element_states` del intento fallido son null:
`status: "notConverged"`, diagnóstico `ENG-E-0007`, salida 4 y sin pasos
posteriores. Si la tangente es singular, se devuelve `unstable`/`ENG-E-0006`
y `convergence.failed_factor`. Se conservan los resultados aceptados dentro de
`convergence`, junto a `accepted_factor`; `cases`, `nodes` y `members` de nivel
superior se vacían según la política de error existente. Cancelación entre pasos
(`newtonStep`) elimina también convergence. `API-E-SFC-OPTIONS` explica opciones
incompatibles o fuera de rango; está disponible mediante `explain`.

### 7.4. Perfil estratificado y consolidación

Los métodos `soilProfile` y `consolidation1d` usan `analyses[0].options` y requieren
modelo estructural y colecciones de casos/combinaciones vacíos. No requieren
nodos ni GDL artificiales. La salida añade exclusivamente `geotechnical` a
`results/1`; las salidas estructurales históricas no reciben esa propiedad.

Un dato geotécnico lleva siempre valor y procedencia:
`{"value": 10, "provenance": "assumed"}` o `"measured"`. El motor conserva
las rutas de todos los parámetros supuestos en `assumed_parameters`.
Un dato físico ausente requerido por el dominio devuelve `ENG-E-0003`, sin
sustituciones. Rangos incompatibles usan `ENG-E-0010`; datos/resultados no finitos
usan `ENG-E-0008` (o rechazo léxico `API-E-INPUT` para números JSON no finitos).

`soilProfile` recibe `layers` ordenadas desde el terreno, `groundwater_depth_m`,
`water_unit_weight_kn_m3` y `depths_m`. Cada capa declara `thickness_m`,
`gamma_kn_m3` y, si queda parcialmente bajo el agua, `gamma_sat_kn_m3`.
Los pesos unitarios y el nivel freático requieren procedencia; γw y zw son
obligatorios incluso en seco. La profundidad es positiva hacia abajo, dentro
del perfil, sin extrapolación. `states[]` devuelve `total_vertical_kpa`,
`pore_pressure_kpa` y `effective_vertical_kpa`, compresión positiva, con
`σ'v=σv−u` y presión hidrostática. No se recortan tensiones efectivas negativas.
[Fixture hidrostático](../crates/engine-api/fixtures/input/valid/sfc010-soil.json).

`consolidation1d` recibe capas con `thickness_m`, `cv_m2_s`, `mv_per_kpa`,
`k_m_s` (todos con procedencia) y `elements`. Se exigen `water_unit_weight_kn_m3`
y la relación `cv=k/(mv γw)`; no se calcula un dato ausente. `drainage` admite
`top`, `bottom` o `double`. La historia `history` contiene puntos
`time_s`/`delta_stress_kpa` con tiempo creciente desde cero; interpola linealmente.
`output_times_s` es creciente, no negativo y queda dentro de la historia.
`theta` en [0,1] y `max_step_s` positivo son explícitos. Las capas pueden tener
propiedades distintas y comparten nodos en sus interfaces.

La salida contiene `depths_m` comunes y estados con presión excedente nodal
`excess_pore_pressure_kpa`, `settlement_m` y `mean_degree`. El grado medio es
null con incremento de tensión cero y puede salir de [0,1] al descargar.
La discretización finita puede producir asiento inicial no nulo por el dren;
no se modifica ni recorta. El dominio conserva sus límites de 2048 elementos,
8000000 valores nodales de salida y 1000000 subpasos por intervalo. La frontera
aplica además los siguientes topes públicos, antes de integrar:

| Constante de `engine_api` | Tope | Rechazo (`invalid`, salida 2) |
| --- | ---: | --- |
| `MAX_CONSOLIDATION_WORK` (r2, conservada) | 50_000_000 subpasos × elementos | `API-E-CONSOLIDATION-WORK` |
| `MAX_CONSOLIDATION_COST` (r3) | 500_000_000 | `API-E-CONSOLIDATION-COST` |
| `MAX_CONSOLIDATION_SEGMENTS` (r3) | 256 intervalos de historia/salida | `API-E-CONSOLIDATION-COST` |
| `MAX_CONSOLIDATION_SUBSTEPS` (r3) | 100_000 subpasos agregados | `API-E-CONSOLIDATION-COST` |

El coste denso es `Σ_intervalos n³ + Σ_subpasos n²`, donde `n` es el número
de nodos libres: elementos totales con drenaje simple, elementos menos uno con
drenaje doble. Cada intervalo entre quiebres de carga y tiempos de salida aporta
`max(1, ceil(intervalo/max_step_s))` subpasos. Se cuenta **toda** la historia,
incluso después de la última salida solicitada, sin descontar reutilizaciones
de factores. Sumas, potencias y productos usan `u128` saturante; el cociente no
finito también satura. Se comprueba primero la cota r2. El tope de intervalos
limita también la historia a 257 puntos; el coste implica como máximo 793
elementos con drenaje simple o 794 con doble. No hace falta otro tope de malla.
Los topes de intervalos y subpasos acotan el trabajo fijo de mallas pequeñas.
Reduzca historia/malla/salidas o aumente `max_step_s` si se rechaza la entrada.

Calibración release en esta máquina y casos extremos aceptados:
[provenance r3](../provenance/ENG-SFC-010.md#r3--revisión-j4-h1).
El objetivo es terminar en pocos segundos (≤ aproximadamente 5 s); no se
promete un plazo en otras máquinas, destinos o perfiles debug. El backend
sigue siendo denso: O(n³) por factorización y O(n²) por solve.

`ConsolidationProfile::solve_with_cancel` consulta el cierre antes de
combine/factorize de cada intervalo, entre bloques de hasta 16 columnas de la
factorización de referencia y antes de cada subpaso. La fachada conecta el
`CancelToken` y devuelve `API-I-CANCELLED` (`cancelled`, salida 5), sin salidas
parciales. El equilibrado y cada solve denso siguen siendo indivisibles.
`solve` usa un cierre false. Solo el backend de referencia reutiliza el último
factor si el `dt` consecutivo coincide por bits: M, K y theta son inmutables y
la matriz y el factor son idénticos. Un dt distinto invalida la caché. Los
backends aportados por el llamador conservan cada llamada de factorización.
Se conservan el orden de operaciones, la partición temporal y los resultados;
el progreso sigue informándose antes/después de la integración.
[Fixture multicapa de Terzaghi](../crates/engine-api/fixtures/input/valid/sfc010-consolidation.json).

Las tres superficies descubren estos contratos con `schema("engine-input/1")`,
`schema("results/1")`, `capabilities` y `explain`; Python publica también sus DTO
con campos opcionales. Las pruebas de paridad comparan bytes canónicos de cada
fixture, extrayendo el documento del sobre CLI cuando corresponde.
