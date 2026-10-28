# ENG-PY-006 — matriz única de conformidad de superficies

Tarea MAM-86. Fixtures analíticos propios, escritos para esta prueba en
`crates/engine-api/tests/fixtures/parity/`. No se consultaron implementaciones
ni resultados de terceros como oráculos. El principio usado es la flexión
lineal de Euler–Bernoulli y el equilibrio estático.

## Matriz y ubicación

`matrix.json` es la única lista de casos, resultados esperados, unidades,
escalas, tolerancias y estados. Cada entrada señala un documento completo
`engine-input/1`; los valores esperados son fracciones de enteros, calculadas
con las ecuaciones siguientes, no salidas grabadas del motor.

`crates/engine-wasm/tests/parity_matrix.rs` lee esa matriz y ejecuta cada
entrada por API Rust, CLI real y llamada Rust a la fachada WASM. Se aloja en
`engine-wasm` porque `engine-api` no depende de WASM ni del CLI, y este crate
ya tiene acceso a la API y a su propia fachada. El CLI se ejecuta mediante
`cargo run --offline -p engine-cli --bin mamut-engine -- run --input -`,
desde la raíz del repo, sin añadir dependencias ni modificar manifests.
Cargo libera el bloqueo de compilación antes de ejecutar el test. Esta
prueba requiere Cargo y las dependencias existentes disponibles offline.

`bindings/python/tests/test_parity_matrix.py` consume exactamente la misma
matriz por `_native.run` y `mamut_engine.run`, con el estilo unittest de
`test_sfc010.py`. Las referencias se exportan opcionalmente desde el test
Rust mediante `MAMUT_PARITY_REFERENCES`; Python exige esa variable y no se
omite silenciosamente cuando falta. Las referencias son temporales y no
constituyen el oráculo numérico: ese papel corresponde a las fracciones.

Ambos tests exigen exactamente esos tres inputs, sin duplicados, y los
conteos de checks 11 para `cantilever-tip.json`, 15 para
`simply-supported-center.json` y 0 para `unsupported-beam.json`.

Patrones reutilizados: llamada nativa WASM y comparación de bytes de
`engine-wasm/tests/sfc010.rs`; referencias Rust independientes del binding
de `bindings/python/tests/test_sfc010.py`; fingerprints y envoltorio real
de `engine-cli/tests/contract.rs`; conformidad mediante soluciones cerradas
de `engine-linalg/tests/conformance.rs`. Alcance requerido por ADR-003,
apartados de paridad ENG-PY-006 y aceptación pendiente de plataformas/CI.

## Casos y soluciones cerradas

Todos los modelos son vigas horizontales D2 con `frame2d.euler_elastic`,
coordenadas en el plano XY, eje local x orientado en +X y eje local y en +Y.
Se usan L = 4 m, E = 200 N/m², I = 3 m⁴, A = 2 m², rho = 0 y P = 12 N.
Son parámetros sintéticos para verificar el modelo matemático lineal;
no representan un dimensionamiento físico ni verifican grandes desplazamientos.

1. **cantilever-tip.json**: un elemento, empotramiento en el nodo 1 y carga
   −P en Uy del nodo 2. Reutiliza geometría y parámetros del voladizo neutro
   existente `fixtures/input/valid/cantilever-linear-static.json`, con el
   elemento explicitado y nombre de carga propio. Integrando
   EI v''(x) = −P(L−x), con v(0) = v'(0) = 0:
   v(L) = −PL³/(3EI) = −768/1800 m y
   theta(L) = −PL²/(2EI) = −192/1200 rad.
   Por equilibrio, Ry(0) = P = 12 N, Mz(0) = PL = 48 N·m y Rx(0) = 0.
   El apoyo tiene v = theta = 0. Las acciones locales de extremo comprobadas
   son [Vy_i, Mz_i, Vy_j, Mz_j] = [12, 48, −12, 0], en N y N·m.

2. **simply-supported-center.json**: dos elementos iguales con nodo central
   en x = L/2, articulación izquierda (Ux, Uy restringidos), rodillo derecho
   (Uy restringido), rotaciones libres y carga −P en el centro. Por simetría
   y equilibrio Ry_i = Ry_j = P/2 = 6 N; los momentos de apoyo y Rx_i son cero.
   Integrando EI v'' = M por mitades y haciendo continuos v y theta:
   v(L/2) = −PL³/(48EI) = −768/28800 m,
   theta_i = −PL²/(16EI) = −192/9600 rad,
   theta_centro = 0 y theta_j = +192/9600 rad.
   Las flechas en los apoyos son cero. El momento físico en el centro es
   PL/4 = 12 N·m; las acciones de extremo en los dos miembros allí son
   respectivamente +12 y −12 N·m, y en los extremos exteriores cero.
   La solución es cúbica por cada mitad sin carga distribuida, por lo que
   estos valores nodales no tienen error de discretización Euler–Bernoulli.

3. **unsupported-beam.json**: la misma viga del primer caso sin apoyos.
   Conserva modos rígidos y una carga con resultante no nula; no existe
   solución estática única. No se le asigna desplazamiento esperado ni
   número de condición finito. Se exige `Status::Unstable`, estado JSON
   `unstable`, primer error `ENG-E-0006`, entidades no vacías y `cases: []`.
   El CLI conserva ese documento y diagnóstico en stderr, emite
   `cli-error/1`, código de proceso 4 y outcome `not_converged`, conforme a
   su clasificación pública. Python exige `_native.AnalysisError` y
   `mamut_engine.Unstable`, con código, entidades, diagnósticos y documento
   idénticos. Son representaciones de transporte del mismo error tipado.

## Unidades y signos

La entrada debe declarar `units: SI` y `axes: Z-up`. `results/1` define SI
implícitamente; no publica campos `units` ni `sign_convention`. La matriz
verifica el esquema, la conservación de IDs y DOFs D2 y cantidades con sus
unidades. Los vectores nodales tienen orden [Ux, Uy, Uz, Rx, Ry, Rz]: Uy
ocupa el índice 1 y Rz el 5, aunque el nodo solo active Ux, Uy y Rz.
Desplazamientos en m, rotaciones en rad, fuerzas en N, momentos en N·m;
carga −Uy produce flecha negativa y reacción +Uy. +Rz es antihorario.
Los vectores locales D2 de extremo siguen [Fx_i, Fy_i, Mz_i, Fx_j, Fy_j, Mz_j]
y representan acciones sobre el elemento; sus momentos en un nodo interno
tienen signos opuestos por equilibrio. No se confunden con los signos de
esfuerzos de sección en `stations`, que aquí solo se comparan entre superficies.
Ambos tests derivan la unidad SI esperada del puntero de cada check y exigen
igualdad con `unit`: desplazamientos nodales 0–2 en `m`, 3–5 en `rad`;
reacciones 0–2 en `N`, 3–5 en `N*m`; acciones locales D2 0, 1, 3, 4 en `N`
y 2, 5 en `N*m`. Un puntero desconocido o cualquier otra unidad hace fallar
la prueba. El orden se verificó en `engine-core::Dof::index` y
`engine-elements-frame/src/recover.rs`.
La etiqueta Z-up no demuestra la convención de elevación 2D pendiente del ADR.

## Tolerancias justificadas

Para cada cantidad se exige finitud y
`abs(actual − esperado) <= t * max(abs(esperado), escala)`, donde
`t = 64 * epsilon_f64 * n * condition_bound` y epsilon_f64 = 2⁻⁵².
Ambos tests fijan `MAX_TOLERANCE_FACTOR = 64`: `tolerance_factor` debe ser
finito, positivo y no superar ese tope, independiente de lo declarado en la matriz.
`n` es el número de DOFs libres. La cota de condición corresponde a la
norma infinito de la matriz reducida, con las unidades numéricas SI de estos
modelos; no se extrapola a otros tamaños o sistemas de unidades.

Para el voladizo, en orden [Ux_2, Uy_2, Rz_2], la matriz reducida es
`[[100,0,0],[0,225/2,-225],[0,-225,600]]`.
Su norma infinito es 825, la norma de su inversa es 11/225 y
kappa_infinito = 121/3. La matriz usa la cota 41 y n = 3, dando
t ≈ 1.748e-12.

Para la viga apoyada, el bloque de flexión en orden
[Rz_1, Uy_2, Rz_2, Rz_3] es
`[[1200,-900,600,0],[-900,1800,0,900],[600,0,2400,600],[0,900,600,1200]]`.
El bloque de traslaciones en X libres es `[[400,-200],[-200,200]]`.
La matriz completa tiene norma infinito 3600 y su inversa norma 3/200,
por lo que kappa_infinito = 54; se usan esa cota y n = 6, dando
t ≈ 4.604e-12. Las normas se obtienen ensamblando las rigideces clásicas
EA/l, 12EI/l³, 6EI/l², 4EI/l y 2EI/l, e invirtiendo con aritmética racional;
no se estiman a partir de los resultados del motor.

La estimación de error de una resolución estable es del orden de
epsilon * n * kappa. El factor 64 deja margen conservador para ensamblaje,
factorización, recuperación de reacciones y evaluación de las fracciones.
No es una certificación de estabilidad del solver para matrices arbitrarias.
Las escalas absolutas son 1 m, 1 rad, P = 12 N y PL = 48 N·m; permiten
verificar ceros sin dividir por ellos y reflejan las magnitudes de entrada.
El manifiesto contiene esas escalas por observación, también para valores
no nulos pequeños. No hay tolerancia numérica para estados ni códigos.

## Bytes y SHA-256

API y fachada WASM entregan exactamente los mismos bytes canónicos UTF-8
del motor (claves ordenadas, floats round-trip y cero negativo normalizado),
sin LF añadido. Esta serialización no es RFC 8785/JCS: por ejemplo,
conserva `0.0` donde JCS representa `0`. El CLI agrega su envoltorio y un LF: se extrae su único
`engineResults[0]` y se canoniza con la API pública antes de comparar.
Así se exige igualdad de bytes del mismo `results/1`, no del envoltorio CLI.
También se comprueba el canal correcto y un único LF terminal del CLI.

La tolerancia entre superficies es cero: comparten núcleo, solver de
referencia y serialización canónica, dentro del mismo target y build.
Se verifican `input_sha256`, SHA-256 de la representación JCS de cada resultado
con la API pública JCS/SHA-256 y, en éxitos, los fingerprints existentes de salida y
del manifiesto CLI. La proyección CLI excluye `cause`/`hint` de los
diagnósticos; aquí los éxitos no tienen diagnósticos, así que su proyección
es identidad. El error CLI no provee fingerprint de salida del workflow;
sí se compara y hashea su documento canónico completo.

Python compara bytes y SHA-256 de bytes calculado con `hashlib` contra los
documentos de referencia Rust, tanto para éxitos como para el documento
retenido por las excepciones. También compara el fingerprint público JCS
contra archivos temporales `.jcs-sha256` exportados por el test Rust y
verifica la canonización propia del motor. El hash de bytes y el fingerprint
JCS se comprueban por separado y nunca se suponen iguales. Las referencias se generan
con el mismo código, target y metadatos de compilación; no se eliminan ni
normalizan campos para conseguir paridad. Una discrepancia de build falla
la prueba y exige regenerar las referencias apropiadas.

## Reproducción y límites

Desde la raíz, con dependencias ya disponibles y el binding actual accesible
en el intérprete con pytest:

```sh
export MAMUT_PARITY_REFERENCES="$(mktemp -d)"
cargo test --offline -p engine-wasm --test parity_matrix
python -m pytest -q bindings/python/tests/test_parity_matrix.py
cargo fmt --check
cargo clippy --offline -p engine-api -p engine-wasm -p mamut-engine-py --all-targets -- -D warnings
bash clean-room/check.sh origin/main clean-room
bash scripts/changelog-check.sh origin/main
```

Para compilar la extensión local se usa la feature existente
`pyo3/extension-module`, igual que en `bindings/python/pyproject.toml`;
no hace falta añadir dependencias. Las referencias y paquetes temporales
quedan fuera de los archivos versionados.

El script de changelog compara commits (`BASE...HEAD`) y no evalúa archivos
nuevos sin commit. El check clean-room incluye cambios locales conocidos
por Git; no certifica por sí solo archivos nuevos sin seguimiento. Esta
tarea no crea commits ni altera el índice real. Para examinar también los
archivos nuevos con clean-room se usaron un índice y un directorio de objetos
temporales, con entradas intent-to-add. El fragmento nuevo se revisó además
directamente por nombre, contenido no vacío, una frase inglesa y ausencia
de la palabra prohibida por la tarea.

Verificación local (2026-10-28): `cargo fmt --check` y Clippy de los tres
crates indicados, con todos los targets y warnings como errores, pasan.
`cargo test --offline -p engine-wasm --test parity_matrix` pasa (1 test,
3 casos por 3 superficies). El binding se compiló offline con
`cargo build --offline -p mamut-engine-py --features pyo3/extension-module`;
pytest en el venv disponible pasa (1 test y 6 subtests), usando un paquete
temporal con la extensión recién compilada y la fachada Python actual.
La compilación inicial sin esa feature no enlazaba por falta de la librería
de desarrollo del intérprete; la configuración de extensión existente
resuelve ese requisito sin nuevas dependencias.
Clean-room pasa tanto en el worktree como con el índice temporal que incluye
los ocho archivos nuevos. `git diff --check` sobre ese índice también pasa.
El script de changelog termina con código 0 y reporta que no hay cambios de
código entre commits; no se presenta ese resultado como validación del
fragmento sin commit. La revisión directa del fragmento pasa.

La ejecución wasm32 real sigue **DESCONOCIDO**; no se valida JavaScript,
navegador ni Node: WASM aquí
significa únicamente la llamada Rust a la fachada, como en SFC-010.
El job de pytest en CI sigue **DESCONOCIDO**. CI de pytest/wheel queda
pendiente porque requiere modificar `.github`,
que está protegido y fuera del alcance autorizado. Matriz completa de
plataformas/CPython y portabilidad binaria: **DESCONOCIDO**.
No se validan modal, P-Delta, geotecnia, memoria, GIL, concurrencia,
cancelación, rendimiento, combinaciones ni otras geometrías/elementos.
La exactitud física de campos sin una observación analítica en la matriz,
incluidas las estaciones, es **DESCONOCIDO** aunque sus bytes coincidan.
Todo lo no validado por esta matriz queda **experimental** a efectos de
esta evidencia; no se cambian las clasificaciones públicas del registro.
Esta tarea no acredita distribución, compatibilidad de wheels ni V1 completa.
