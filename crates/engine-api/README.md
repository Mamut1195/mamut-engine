# engine-api · ENG-API-001 · incremento 2c

Fachada Rust portable a wasm32, sin E/S ni dependencias de Python. Los tipos JSON
y la extensión de dominio se mantienen en engine-api, con serde/serde_json.
El SDK vendorizado mamut-workflow aporta registro y compilador; mamut-canon
v0.1.0 (SDK-WF-002) aporta la huella JCS y SHA-256. Los crates numéricos
conservan sus tipos propios. Versión del engine: 0.1.0.

API pública:

- `version() -> Value`: versión del engine y contratos reconocidos.
- `capabilities() -> Value`: operaciones, esquemas, elementos y métodos reales.
  `linearStatic`, `pDelta`, `modal` y `responseSpectrum` tienen `engine: true,
  run: true`; las combinaciones de cargas se ejecutan con linearStatic.
  El array adicional `method_descriptors` contiene los cinco métodos, en orden
  del registro SDK, con contratos, estado, fixtures, citas propias y provenance.
- `method_descriptors()` y `method_descriptor(id, input_contract, allow_experimental)`:
  descubrimiento y resolución mediante `MethodRegistry`/`WorkflowCompiler` de
  mamut-workflow v0.1.0. El Engine extiende el descriptor SDK con evidencia;
  no mantiene otro índice de métodos. Versión de método: 1.
  La entrada de los análisis usa `urn:mamut:engine:engine-input:1.0.0`;
  `combinations` es postproceso y consume `urn:mamut:engine:results:1.0.0`.
  Todos producen este último contrato. Las URN adaptan los nombres JSON
  existentes al formato del SDK; no cambian los documentos de entrada/salida.
  El SDK compara dominio, nombre y versión mayor antes de crear el adaptador;
  las opciones mecánicas permanecen en el documento tipado consumido.
- `schema(kind) -> Result<&'static str, ApiError>`: JSON Schema Draft 2020-12
  local, kind exacto `engine-input/1`, `results/1`, `validate-report/1`,
  `method-descriptor/1` o `capabilities/1`. `validate-report/1` documenta el informe
  directo del CLI; `validate`/`validate_with` conservan su retorno de diagnósticos.
- `explain(code)`: catálogo de engine-core y códigos locales documentados abajo;
  código ausente produce `UnknownCode`.
- `validate(input_json) -> Result<Vec<Diagnostic>, ApiError>` y `validate_with`
  con `&CancelToken` y callback síncrono `FnMut(Progress)`.
- `run(input_json, &CancelToken, progress: impl FnMut(Progress)) -> Outcome`:
  devuelve `status` y `document: String` con los bytes UTF-8 canónicos de results/1,
  también ante errores. `Outcome::exit_code()` y `exit_code(Status)` son públicos.
- `PreparedLinear::prepare(input_json, &CancelToken) -> Result<PreparedLinear, AnalysisFault>`:
  un solo análisis `linearStatic`, al menos un caso y sin combinaciones.
  Factoriza una vez. `temperature_loads` entra igual que en `run`: las fuerzas
  de nudo equivalentes se agregan como cargas nodales del modelo que ensambla
  el sistema. `displacements()` resuelve de nuevo y devuelve, por caso,
  los nodos en el orden de `results/1` con id y `[Ux, Uy, Uz, Rx, Ry, Rz]`.
  No construye el documento, no recupera esfuerzos de barras y no cambia los
  bytes de `run`. Otro análisis, una combinación o un resorte no lineal
  devuelven `invalid`: esos trabajos usan `run`. Los avisos que `run` escribiría
  en el documento no se emiten aquí; los desplazamientos coinciden con `run`,
  también cuando el caso lleva temperatura uniforme.
- `Outcome::query(&ResultQuery) -> Result<Value, ApiError>`: filtra IDs de nodos
  y miembros y proyecta rutas de campos sobre resultados ya calculados.
- `canonical_json(input_json) -> Result<String, ApiError>`: UTF-8 compacto,
  claves ordenadas recursivamente, arrays en orden de entrada y -0.0 → +0.0.
- `input_sha256(input_json) -> Result<String, ApiError>`: huella RFC 8785/JCS
  idéntica a `results/1.input_sha256`, con el parseo y límites de `run`, sin
  ejecutar análisis. Permite comprobar procedencia desde los adaptadores.

`engine-input/1` exige los campos de su [esquema](schemas/engine-input-1.json):
`schema`, `units: "SI"`, `model`, `cases`, `combinations`, `analyses`, `options`.
Colecciones vacías: `[]`; `options`: `{}` por defecto.
P-Delta tiene estado `Stable` y se ejecuta con `options: {}`; no requiere
`allow_experimental`. La promoción MAM-582 se apoya en
[ENG-BENCH-002 (PR #132)](../../provenance/ENG-BENCH-002.md):
AISC 360-16 Comm. C2, casos 1 y 2, con error ≤0.27% frente a tolerancias de
3% en momentos M y 5% en desplazamientos Δ, según la evidencia suministrada
para el ticket. Véase la [nota de provenance](../../provenance/ENG-CORE-006.md#mam-582--pdelta-estable--2026-10-15).
`nonlinearStatic`, `soilProfile` y `consolidation1d` son `Experimental` hasta
que ENG-VAL-006 cubra su validación (decisión de Jonathan vía J0 2026-10-20).
`buckling.linear` es `Experimental` mientras `provenance/ENG-ANL-006.md` siga
sin firma de J4 y la página de Timoshenko–Gere siga desconocida. pDelta sigue
`Stable`. `linearStatic`, `modal`, `responseSpectrum` y `combinations` siguen
`Validated` dentro de los fixtures publicados.
La opción booleana `options.allow_experimental` (por defecto `false`) exige
consentimiento `true` en la raíz del job para ejecutar métodos `Experimental`:
sin él, `run` devuelve `invalid`, código `API-E-INPUT`, salida 2 y causa con el
método y la opción requerida, antes del ensamblaje.
`Deprecated` se reserva para compatibilidad; no hay métodos deprecados.
Los estados no cierran los seguimientos #78/#84/#85/#86 ni certifican modelos arbitrarios.
`validate` conserva su alcance de contrato/topología y no ejecuta ni habilita métodos.
 `model.axes`: `"Z-up"`, terna diestra,
X-Y horizontal, Z arriba. D2 exige Z=0 y usa Ux, Uy, Rz; D3 admite los seis GDL.
Coordenadas en m, masa en kg, tiempo en s, fuerzas en N, momentos en N·m,
tensiones en Pa y giros en rad. No se convierten unidades.
Una etiqueta de texto en `units` distinta de `"SI"` produce
`ApiError::UnknownUnit` en `validate`; `run` devuelve `ENG-E-0001`, status
`invalid` y salida 2, sin resultados. Si falta `units` o su valor no es texto,
se conserva `ApiError::InvalidInput` (`API-E-INPUT` en `run`).
La causa es `UnknownUnit: Unknown unit system; only "SI" is accepted`;
por seguridad no repite la etiqueta recibida. Tras el parseo JSON y la
comprobación de `schema`, el chequeo de `units` se ejecuta antes de la validación
estructural: una entrada con `schema: "engine-input/1"`, `units` distinto de
`"SI"` y además mal formada estructuralmente devuelve
`ENG-E-0001`/`ApiError::UnknownUnit`, en lugar del error estructural.
Véase [convención aprobada](../../docs/AXES_CONVENTION.md).

## engine-input/1 = model/1 + job/1

El documento `engine-input/1` agrupa el modelo en `model` y el trabajo (job)
en `cases`, `combinations`, `analyses` y `options`. `schema` identifica el
documento completo y `units` fija sus unidades. Los schemas ya están publicados
mediante `engine_api::schema("engine-input/1")` y `engine_api::schema("results/1")`;
el subcomando CLI `mamut-engine schema` corresponde a ENG-SFC-002.

El nombre definitivo (`mamut.engine.model/1`, `job/1` o
`urn:mamut:engine:model:1.0.0`) está **PENDIENTE de confirmación de Jonathan**
en Notion «02 · Contratos». Se conserva el nombre y la estructura actuales;
model/1 y job/1 describen aquí las dos partes, sin publicar contratos separados.

Los [fixtures válidos](fixtures/input/valid) incluyen un voladizo linearStatic,
una viga con combinación explícita, un oscilador modal de 1 GDL, P-Delta y un
espectro constante. La prueba recorre las carpetas y exige validate sin
diagnósticos y run satisfactorio: salida 0 salvo la combinación, que conserva
el aviso `API-W-COMBINED-PRESCRIBED` y salida 1 de #85. Los
[fixtures inválidos](fixtures/input/invalid) tienen un `.expect` JSON adyacente
con `code`, `exit_code` y el resultado de `validation` (`invalidInput`,
`nonFiniteNumber`, `unknownUnit`, `unsupportedSchema` o `diagnostics`). Las
etiquetas de unidades desconocidas usan `ENG-E-0001`. Se conservan los códigos
publicados por #88: `ENG-E-0009` para contratos incompatibles, `ENG-E-0003` para referencias
ausentes y `API-E-INPUT` para parseo/estructura. `1e400` y los tokens NaN/±Infinity
se rechazan léxicamente con `ENG-E-0008`; NaN no es un literal JSON. Hay fronteras
distintas para enteros léxicos fuera de ±(2^53−1) e identificadores fuera de u32.

La prueba de nombres lee la lista clean-room mediante `include_str!` y examina
schemas y fixtures completos, incluidas claves escapadas y duplicadas. Compara
sin distinguir mayúsculas y eliminando `_` y `-`; añade claves de comandos
ajenas al contrato. Exige nombres ASCII `^[a-z][a-z0-9_]*$` en datos de fixtures
y bajo `properties`/`$defs`; las palabras clave de JSON Schema tienen una lista
blanca explícita. `pattern` solo se admite como palabra clave con valor string.

`element` selecciona un kind de la fábrica propia, como `truss2d.linear`, sin
argumentos posicionales. Su excepción queda limitada a `model.members[].element`
en fixtures y `/$defs/member/properties/element` del schema engine-input/1.
También se conserva el campo ya publicado
`/$defs/memberIdentity/properties/element` de results/1 mediante una excepción
contractual exacta. Los 15 nombres camelCase existentes en `$defs` tienen
excepciones por contrato y ruta: no se amplían a otras propiedades o fixtures.
La decisión sobre estas excepciones contractuales está **PENDIENTE de
Jonathan/J4**; véase [Correcciones J4 r23](../../provenance/ENG-CORE-004.md).

## Validación y ejecución

El parseo rechaza campos desconocidos, tipos erróneos, objetos posicionales,
claves duplicadas (también escapadas), texto sobrante, NaN/Inf y overflow f64.
`schema` es obligatorio: contrato desconocido produce `UnsupportedSchema` en
validate y `Status::Invalid` (2) en run. Solo se reconoce la etiqueta exacta `/1`.
El esquema describe estructura; duplicados y representación f64 son controles
adicionales del parser. validate conserva diagnósticos topológicos del dominio,
con código, gravedad, entidades, causa y sugerencia, sin deduplicar ids. No
comprueba estabilidad, propiedades físicas ni restricciones de ensamblado;
run verifica esas fronteras y nunca resuelve un modelo con errores topológicos.

Antes de parsear el documento con serde o construir el modelo, una inspección
léxica sin recursión comprueba estos límites públicos (inclusive). Se aplican
a `run`, `validate`, `validate_with`, `input_sha256` y `canonical_json`:

| Constante pública | Máximo | Alcance |
| --- | --- | --- |
| `MAX_INPUT_BYTES` | 8 388 608 (8 MiB) | Bytes UTF-8, incluidos espacios/cadenas |
| `MAX_NODES` | 256 | Entradas de model.nodes |
| `MAX_MEMBERS` | 1 024 | Entradas de model.members |
| `MAX_CASES` | 64 | Entradas de cases |
| `MAX_LOADS` | 16 384 | Suma de loads, member_loads y temperature_loads de todos los casos |
| `MAX_COMBINATIONS` | 128 | Entradas de combinations |
| `MAX_JSON_DEPTH` | 64 | Objetos/arrays abiertos; raíz = 1 |
| `MAX_JSON_ARRAY_ITEMS` | 16 384 | Entradas de cualquier array auxiliar |

El límite de nodos acota el sistema denso de ReferenceBackend. La inspección
reconoce claves escapadas y omite el contenido de cadenas al contar estructura.
Al superar un límite se obtiene `InvalidInput` con el nombre y máximo; en run,
`API-E-INPUT`, status invalid y salida 2, sin callback de parseo ni SHA del input.
La cancelación ya solicitada se comprueba primero. La canonización de results/1
opera sobre el árbol de salida y no aplica los límites del input a sus bytes.

Los tokens enteros (sin punto decimal ni exponente), en cualquier campo,
requieren `|n| <= 9007199254740991` (2^53−1, `Number.MAX_SAFE_INTEGER`).
La frontera léxica rechaza ambos signos fuera de ese rango antes de que serde
pueda convertirlos a float o JCS calcule una huella ambigua (#143).
Se devuelve `ApiError::InvalidInput`; en `run`, `API-E-INPUT`, status invalid,
salida 2 e `input_sha256: null`, sin callbacks de parseo.
Los ids y referencias numéricos siguen limitados a u32; los contadores
`max_iterations` y `mode_count` a 16 384. No hay seeds ni índices enteros
grandes en engine-input/1; los otros identificadores son cadenas y los GDL
son etiquetas. `bool` no es un entero. Los tokens con punto/exponente conservan binary64;
overflow como `-1e400` produce `ApiError::NonFiniteNumber`, igual que los tokens
NaN, -NaN y ±Infinity (sin signo `+` explícito). En `run` dan `ENG-E-0008`,
status invalid, salida 2 e `input_sha256: null`, sin resultados ni callbacks de
parseo. Las cadenas con esos textos no se clasifican como números no finitos.
Los ids además deben caber en u32 según el contrato.

Para run se requiere exactamente una entrada en `analyses`. linearStatic
requiere al menos un caso; modal y responseSpectrum pueden usar `cases: []`.
Los métodos dinámicos requieren `options` dentro de esa entrada, según los
ejemplos siguientes. No hay resultados parciales en errores o cancelación;
P-Delta conserva únicamente el reporte de convergencia cuando termina con
`notConverged`/`unstable`. Cerca o por encima del pandeo, P-Delta puede
terminar con esos estados: código de salida 4, sin resultados mecánicos.
El test
[`pdelta_nonconvergence_and_instability_return_four_with_iteration_report`](tests/dynamic.rs)
comprueba `notConverged` con P/Pcr=0,2 y `max_iterations=1` (una iteración
reportada y `cases: []`), y `unstable` con P/Pcr=1,2 (sin iteraciones y
`modes: []`); ambos devuelven 4. El caso de no convergencia comprueba el límite
de iteraciones, no un umbral de pandeo.
Las combinaciones de cargas requieren linearStatic.

En cada iteración P-Delta, `residual_convergence_ratio` compara el residual de
cada GDL libre con `tolerance·ref_grupo + 64·m_i·eps·(|K+Kg||u|)_i`.
Las referencias son los máximos de `|F|` y `|K+Kg||u|` en GDL libres de
traslación (fuerza) o giro (momento), separados; `m_i` cuenta las entradas CSR
de la fila global (mínimo 1). Completar exige un cociente máximo `<= 1` y
`relative_displacement_change <= tolerance`. `relative_free_residual` conserva
su normalización mixta histórica solo como diagnóstico y puede superar la
tolerancia aun cuando el análisis complete. Véase `provenance/ENG-ANL-003.md`.

Cada miembro mantiene `section` y `law`. `element` es opcional: por defecto
`frame2d.euler_elastic` en D2 y `frame3d.euler_elastic` en D3. También se admiten
`truss2d.linear` y `truss3d.linear`; kind desconocido produce Unsupported. Los
parámetros se obtienen de la unión de `section.properties` y `law.parameters`;
se rechazan claves duplicadas y parámetros desconocidos según la fábrica propia:

| Elemento | Parámetros escalares SI requeridos | Campo de miembro |
| --- | --- | --- |
| frame2d.euler_elastic | E, A, I, rho | — |
| frame3d.euler_elastic | E, G, A, J, Iy, Iz, rho | orientation: [x,y,z] |
| truss2d.linear / truss3d.linear | E, A, rho | — |

E/G en Pa, A en m², I/J/Iy/Iz en m⁴, rho en kg/m³ (cero permitido; en esta fachada la fuente de masa es nodal o desde cargas,
no se incorpora automáticamente rho a la matriz de masa). `orientation` es el vector global de referencia del z
local, explícito para frame3d, no nulo ni paralelo al miembro. La fachada no
expone liberaciones ni elementos no lineales en este incremento.

Cada caso tiene `loads` nodales y, opcionalmente, `member_loads` uniformes:
`{member, force_per_length: [qx,qy,qz], moment_per_length: [mx,my,mz]}` en ejes
globales, N/m y N·m/m. Se suman múltiples cargas sobre un miembro. La misma
colección sin escalar produce `Element::load_vector` para el ensamblado y las
vistas `MemberRecovery` de **todos** los miembros para `CaseResults::recover`.
Así se conserva la coherencia exigida por el equilibrio nodal de #69.

`temperature_loads` es un canal aparte:
`{member, alpha_per_kelvin, delta_t_kelvin}`, sin gradiente. Solo lo aceptan
`truss2d.linear` y `truss3d.linear`. Las fuerzas de nudo equivalentes se
agregan como cargas nodales del caso en el modelo que ensambla el sistema.
`run` y `PreparedLinear::prepare` usan ese mismo modelo.
No son un `MemberLoad`. El equilibrio nodal sigue cerrando: esas fuerzas
están en las cargas nodales y la recuperación suma el axil térmico
`N = −(E·A·α·ΔT)`. Los kinds de pórtico elástico rechazan el registro con
`ENG-E-0013`; no hay fórmula de αΔT para ellos. Un registro repetido en el
mismo miembro y caso es `ENG-E-0002`. La suma de 16384 incluye `loads`,
`member_loads` y `temperature_loads`. Un modal que reconstruye el sistema
desde el modelo original no conserva esas cargas nodales. Kg sigue el axil
mecánico y no incluye αΔT. `buckling.linear` rechaza con `ENG-E-0013` un caso
de referencia con `temperature_loads` no vacío. pDelta rechaza igual el caso
de gravedad. `validate` anticipa ese diagnóstico en el `Ok` de diagnósticos:
no tiene código de salida; `run` lo traduce a `unsupported` y salida 3.
No se publica factor ni desplazamiento. La temperatura solo en el caso
lateral de pDelta, o en un caso que no es la referencia de `buckling.linear`,
sigue permitida.

`model.prescribed` opcional: `[{node,dof,value}]`, constante para todos los casos,
sin factor de carga. `model.diaphragms` opcional: `[{master,slaves:[ids]}]`, diafragma
rígido XY D3. El ensamblador verifica referencias, GDL y restricciones, incluidas
declaraciones de prescritos repetidas y diafragmas incompatibles.

Se llama a StaticLinear por caso (factor 1), con un sistema ensamblado común y
backend ReferenceBackend determinista. Después se llama a `combine()` con los
factores explícitos del usuario, sin coeficientes normativos. Para que los
miembros sean compatibles, se recuperan los casos sobre la unión ordenada de
sus estaciones críticas. Los extremos de combinaciones se calculan sobre esas
estaciones, como define engine-analysis; no se prometen nuevos extremos críticos
interiores propios de la combinación. No hay declaraciones de envolventes en
engine-input/1 para linearStatic; en ese método `envelopes` y `modes` son `[]`
y `convergence` es null.

Los seguimientos de J4 permanecen abiertos:

- **#85:** toda combinación cuya salida contenga prescritos (incluidos apoyos
  cero) añade `API-W-COMBINED-PRESCRIBED` y status warning. La salida combinada
  es suma factor·δ; un asentamiento único se pondera íntegramente por su factor.
  `combine` rechaza varios casos con prescritos no nulos, incluso con factor cero.
  Como los prescritos de entrada son comunes a todos los casos, una combinación
  con dos casos y asentamiento no nulo se rechaza: no se separa automáticamente
  el aporte del asentamiento.
- **#84:** resultados con diafragmas añaden `API-W-DIAPHRAGM-RECOVERY` y status
  warning: la recuperación con diafragmas carece de cobertura de pruebas del
  dominio. La prueba smoke de esta fachada no cierra ese seguimiento.
- **#84(c):** P-Delta expone solo desplazamientos/reacciones laterales;
  `cases[].members` queda vacío. No se llama a `CaseResults::recover` con esos
  resultados, cuya validez no está confirmada. La identidad de miembros sí se conserva.
- **#78:** `API-W-PDELTA-INERTIA` cuando la factorización de rigidez total
  informa bloques 2×2 (o no publica ese dato). El cálculo de inercia usa copysign
  sin prueba confirmada en ese seguimiento. El aviso se conserva si el resultado
  es unstable; la prueba de aviso no certifica ni cierra #78.
- **#86:** toda conversión de cargas a masa añade `API-W-MASS-PROJECTION`,
  incluido un fallo posterior. Se conserva el rechazo del dominio a cualquier
  proyección negativa (succión), ENG-E-0010 → invalid/2. No se añade tolerancia:
  con gravedad no alineada a ejes puede rechazar por redondeo.

results/1 incluye schema, engine_version, git_commit, target, contracts, methods, input_sha256,
status, nodes, members, cases, combinations y diagnostics. `nodes` y `members`
superiores describen identidad/geometría; respuestas numéricas dentro de cada
caso o `combinations[].results`. Vectores nodales: Ux,Uy,Uz,Rx,Ry,Rz;
reacciones: Fx,Fy,Fz,Mx,My,Mz. Resultantes de estación: N,Vy,Vz,T,My,Mz.
Acciones locales de extremo sobre el miembro, por nodo y GDL del elemento.
El [esquema results/1](schemas/results-1.json) describe también los fallos sin
resultados: input_sha256 es null si no se completó el parseo/huella; después es
SHA-256 de los bytes UTF-8 RFC 8785 (JCS) del input, aunque falle el contrato o el
cálculo. Se usa exclusivamente `mamut_canon::fingerprint_value` sobre el árbol
del parseo estricto, que mantiene rechazo de duplicados/overflow y límites.

**Cambio de huella en ENG-CORE-005:** sustituye la representación propia de #88.
JCS serializa `1.0` como `1`, `-0.0` como `0`, ordena claves por UTF-16 y convierte
números a binary64. Por tanto pueden cambiar
los hashes de las mismas entradas. Los vectores fijos con hashes anteriores y
nuevos están en [fingerprints.json](fixtures/results/fingerprints.json);
incluyen sondas de JSON que se rechazan como engine-input/1 después de la huella.
La sonda histórica `9007199254740993` está marcada `rejected_by_engine`: desde
#143 se rechaza antes de la huella. Conserva los valores históricos del SDK
como evidencia; no son huellas emitidas por la fachada para esa entrada.
El control no modifica mamut-canon ni las huellas de entradas seguras; los
16 hashes de [DETERMINISM.md](../../docs/DETERMINISM.md) siguen iguales.
Por ejemplo `{"v":1.0}` pasa de
`f3c8be97307d38261061fa08e74365e54a239cddb1d189f3bea18022adbe2f35` a
`afbf9d0f3560b0fd7795e81c42a0a79ee6b6fc67e064f77826aee642cad28d91`.
Las pruebas de estático y dinámica ahora usan la huella del SDK.

La serialización de `results/1` y `canonical_json` conservan el formato propio
publicado: claves por orden escalar, floats round-trip y -0.0 → 0.0. No usan JCS,
porque migrarlas cambiaría la representación de resultados y enteros exactos.
`input_sha256` no se calcula con `canonical_json`.

`git_commit` es una revisión hexadecimal de 40/64 caracteres suministrada por
`MAMUT_ENGINE_GIT_COMMIT` al compilar; ausente o inválida produce `null`.
`target` es el triple `TARGET` real de Cargo capturado por build.rs. No se invoca
git, no hay red, rutas ni relojes. Ambos campos se emiten también en errores y
son opcionales en el schema para aceptar documentos publicados antes del cambio.

### Consultas Rust y validación del contrato

```rust
use engine_api::{Outcome, ResultQuery};
use engine_core::{NodeId, MemberId};

fn reactions(outcome: &Outcome) -> Result<serde_json::Value, engine_api::ApiError> {
    outcome.query(&ResultQuery {
        node_ids: Some(&[NodeId(2)]),
        member_ids: Some(&[MemberId(7)]),
        fields: Some(&["cases.nodes.reactions", "cases.members.stations.forces_local"]),
    })
}
```

`None` selecciona todo y `Some(&[])` selecciona nada. Los IDs filtran identidades,
casos, combinaciones, formas modales, respuestas espectrales y envolventes,
conservando orden; las selecciones de nodos/miembros son independientes.
La selección no altera el original ni recalcula agregados; conserva referencias
geométricas, diagnósticos y procedencia del análisis completo. Por ello un
miembro retenido puede referenciar un nodo que no esté seleccionado.
Las rutas atraviesan arrays sin índices; seleccionar una rama devuelve todos
sus descendientes. Se retiene `id` en cada objeto proyectado. Las rutas ausentes
no aportan campos y las mal formadas producen `InvalidInput`. La proyección es
un fragmento JSON y puede omitir campos requeridos por results/1; sin `fields`
se conserva el sobre completo, válido contra el schema.

Las pruebas nativas comprueban todas las salidas de run estático/dinámico contra
el schema, incluidos errores y cancelación, mediante un validador de pruebas
sin E/S del subconjunto Draft 2020-12 utilizado. Keywords/patrones desconocidos
fallan explícitamente; mutaciones negativas comprueban restricciones, referencias,
tipos y condiciones. Los mismos tests se comprueban con `cargo check --offline
-p engine-api --tests --target wasm32-unknown-unknown` (compilación, sin ejecutar
wasm). La validación mediante CLI queda pendiente de ENG-SFC-002: aún no existe.

| Status | Código de salida |
| --- | --- |
| ok | 0 |
| warning | 1 |
| invalid | 2 |
| unsupported | 3 |
| notConverged / unstable | 4 |
| cancelled | 5 |
| timeout (reservado) | 6 |
| internalError | 70 |

Los errores del dominio conservan código y entidades; ENG-E-0006 → unstable,
0007 → notConverged, 0013 → unsupported, 0012 → internalError; los restantes
son entrada inválida. Códigos locales de la fachada (también en `explain`):
`API-E-INPUT` para errores de parseo/estructura y `API-I-CANCELLED` para
cancelación, además de los avisos anteriores. Todos los diagnósticos tienen
code, severity, entities, cause y hint. Los fallos internos se comunican como
resultados con código 70; la biblioteca no usa panic/unwrap/expect. El callback
del anfitrión debe retornar normalmente; sus propios pánicos no son errores del
motor. No se establecen timeouts en este incremento.

Cancelación antes de empezar y antes/después de cada callback, incluido el final.
En validate: parse, model, topology (1/3, 2/3, 3/3). En run: parse, model validado,
assembly, un evento case por caso, combinations y serialization, con total
`n_cases + 5` para linearStatic. Se consulta también entre combinaciones y
recuperaciones. P-Delta añade gravity y pdeltaIteration tras cada paso;
`total = max_iterations + 5` reserva el máximo y serialization salta al total
cuando converge antes. Modal añade un evento modal tras el eigensolver y uno
por modo (`total = mode_count + 5`). Espectro añade esos eventos y spectrumMode
por cada modo/dirección, luego spectrumDirection por dirección
(`total = mode_count + mode_count*n_directions + n_directions + 5`). Los
completed son monótonos; se consulta antes/después de cada callback. El
eigensolver y cada resolución lineal individual son indivisibles.
El token monotónico usa AtomicBool, compartible con Arc y sin hilos en wasm.
`Progress.completed` y `total` son usize; el número de casos está acotado por
`MAX_CASES`.

La salida no añade relojes, rutas, commit ni target del entorno. Conserva
round-trip binary64 mediante `float_roundtrip` de serde_json. Distingue enteros
y floats (`0` y `0.0`); no es RFC 8785. Claves permutadas y -0.0 se canonizan a los
mismos bytes y SHA; el orden de arrays se conserva y forma parte de la entrada.

Pruebas: `cargo test --offline -p engine-api`. Oráculos derivados a mano:
voladizo (δ=PL³/3EI, M=PL), viga apoyada (Mmax=wL²/8, R=wL/2), barra (δ=PL/EA),
superposición, coherencia de cargas/miembros, determinismo, errores y cancelación,
fronteras exactas de tamaño/profundidad y rechazo numérico anterior al parseo.
También modal de 1 GDL (ω²=k/m, T=2π/ω y participación 100%), espectro
constante (u=Sa/ω²) y P-Delta mallado frente a la solución exacta propia.
Sin consulta de código de otros motores ni oráculos externos.

## Opciones y resultados del incremento 2c

Modal con masas declaradas en `model.masses`:

```json
{"method":"modal","options":{"mode_count":1,"mass_kind":"lumped","directions":[[1,0,0]]}}
```

`mass_kind`: `lumped` o `consistent`; las masas nodales diagonales aportan lo
mismo a ambos. No se condensan ni descartan GDL sin masa: M_ff singular
produce unstable/4. Se devuelven exactamente los modos solicitados, con
`period` (s, null si infinito), `frequency_hz`, `angular_frequency` (rad/s),
`eigenvalue` (s⁻²), `normalization: "mass"` y `shape` nodal (φᵀM_ffφ=1).
`shape[].displacements` contiene componentes de φ, no desplazamientos físicos;
reacciones cero y prescritos cero representan perturbaciones. `participation`
por dirección incluye Γ, masa efectiva, fracciones y acumulados. El denominador
`total_mass` incluye apoyos; `free_mass` da el límite de participación libre.
Las fracciones son adimensionales (1 significa 100%).

Para convertir un caso a masas, deje `model.masses: []` y añada a las opciones:

```json
{"mass_source":{"method":"masses_from_case","case":"G","factor":1,"direction":[0,0,-1],"gravity":9.81}}
```

Use `method: "system_case"` (alias `"masses_from_system_case"`) para incluir
fuerzas equivalentes de cargas de miembro. `masses_from_case` usa solo cargas
nodales. Se aplica factor una vez: m=(F·d̂)factor/gravity; se asigna esa masa a
cada traslación declarada, sin inercias rotacionales. No hay gravedad implícita.
Una fuente desde cargas y masas nodales simultáneas se rechazan con invalid/2.
El aviso #86 se aplica también en responseSpectrum.

P-Delta con dos casos declarados:

```json
{"method":"pDelta","options":{"gravity":{"case":"G","factor":1},"lateral":{"case":"H","factor":1},"tolerance":1e-8,"max_iterations":100,"displacement_scale":1,"rotation_scale":1}}
```

Se registra `geometric_producer` de cada elemento. Kg se congela usando los
axiales del caso de gravedad; no se actualizan materiales/ejes ni axiales por
carga lateral. El caso lateral es independiente: no incluye desplazamientos ni
reacciones de gravedad. Los prescritos no se escalan. La gravedad requiere
cargas nodales: con cargas de miembro se devuelve unsupported/3, porque el
productor no recibe la corrección de acciones de extremo para recuperar N.
El mismo estado y el código `ENG-E-0013` se devuelven si el caso de gravedad
trae `temperature_loads` no vacío: Kg no incluye el axil térmico. `validate`
informa ese diagnóstico antes de ejecutar. El caso lateral puede llevar
temperatura: su efecto axial en una barra alineada y libre es α·ΔT·L, porque
Kg de la cercha no entra en el grado axial.
`convergence` publica casos/factores, opciones, inercia y métricas ordenadas por
iteración (cambio relativo y residual absoluto/relativo). NotConverged y Unstable
producen código 4, sin respuestas parciales, conservando ese reporte y avisos.

Espectro tabulado, con todas las decisiones del usuario:

```json
{"method":"responseSpectrum","options":{"mode_count":1,"mass_kind":"lumped","modal_combination":"srss","directional_combination":"srss","directional_factors":[1],"directions":[{"direction":[1,0,0],"spectra":[{"pairs":[[0,3],[10,3]],"units":"periodSecondsAccelerationMetersPerSecondSquared","damping":0.05,"interpolation":"linear","extrapolation":"error"}]}]}}
```

Una tabla común o una por modo. Unidades alternativas:
`frequencyHertzAccelerationMetersPerSecondSquared`; interpolación `linear` o
`logLog`; extrapolación `error` o `plateau`; combinación modal `srss` o `cqc`;
direccional `srss` o `weightedSum`. No hay espectros ni coeficientes normativos.
`modes[].spectrum_responses` conserva respuesta firmada por dirección: Sa, q,
fuerzas equivalentes, desplazamientos/reacciones y cortante basal.
`envelopes` contiene primero la combinación modal por dirección (sin aplicar
su factor, que se informa en `factor`), y al final la combinación direccional
con factores aplicados (`direction`/`factor`: null). Cada escalar se combina
por separado; la envolvente no representa un estado simultáneo equilibrado.
No se recuperan esfuerzos de miembros en esta fachada espectral.

`max_iterations`, `mode_count` y `mode_count*n_directions` se limitan a
`MAX_JSON_ARRAY_ITEMS` (16 384); cero se rechaza. El dominio además verifica
mode_count contra los GDL libres y los límites de su eigensolver denso.

Derivación del voladizo: EI y''''+P y''=0 con base fija, momento nulo y
EI y'''(L)+P y'(L)=−H da A=3(tan α−α)/α³, α=L√(P/EI).
Su primer polo α=π/2 da Pcr=π²EI/(4L²). Para P/Pcr=0,2 se compara A con
1/(1−P/Pcr) al 0,4% (error de la aproximación del primer polo); con mallas de
4/8 elementos el error decrece y la malla de 8 difiere menos de 1e−5 de la
solución exacta. La tolerancia iterativa es 1e−10, separada del error de malla.


## Shared message catalog (MAM-836, phase 1)

`Lang::{En, Es, Pt}` and `Lang::ALL` select messages explicitly; `Lang::parse`
accepts only exact lowercase `en`, `es`, `pt`. English is the default. The API
never reads LANG, LC_ALL, environment settings or system locale for language.
Invalid language values produce a fixed English `ApiError::InvalidInput`.

The embedded, alphabetically ordered `messages/{en,es,pt}.json` catalogs are the
single production source for CLI/Python/WASM translations: 32 `errors.*`, 14
`diagnostic.*` defaults and 23 `explain.*` messages per language. `message(lang,
key)` returns an optional static string; `message_keys(lang)` lists keys in order.
Invalid embedded JSON returns no messages, with catalog tests enforcing validity.

Use `explain_in(code, lang)` for translated explanations; `explain(code)` preserves
the original English text. `localize_diagnostic`, `localize_diagnostics` and
`localize_document` translate a cause only when it exactly equals the English ENG
default. All codes, severities, entities and hints stay unchanged. Document
localization uses the existing canonical JSON; English returns the original bytes
without parsing or serialization. `Outcome::localized(lang)` preserves status and
localizes its results/1 document. Existing `run`/`validate` remain English.

The CLI's workflow fingerprint view excludes diagnostic `cause` and `hint` text,
while `engineResults` retains the full localized document. Workflow
`recordFingerprint` and SDK step input/output fingerprints attest to codes and
data, **not message wording**; input_sha256 is unchanged.

Phase 2 remains out of scope: approximately 900 parameterized causes, all hints,
ApiError and parse errors, Python-owned PY-E-* and facade messages,
mcp/compile/methods language, and describe/help language. These messages remain
unchanged. Spanish and Portuguese translations await native-speaker review,
especially Portuguese; see [ENG-I18N-001](../../provenance/ENG-I18N-001.md).
