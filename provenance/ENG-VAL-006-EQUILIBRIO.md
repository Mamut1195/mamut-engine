# EQ-ALL equilibrio global (MAM-854) — ID VAL DESCONOCIDO

## Alcance y fuentes

Ticket MAM-854, padre MAM-604. El identificador de informe recibido es
ENG-VAL-006. El ticket no asigna un VAL-0xx propio a EQ-ALL: **DESCONOCIDO**.
Esta nota es independiente del documento previo `provenance/ENG-VAL-006.md`,
que trata otra campaña. No se deduce de él cobertura para este ticket.

Nota N3 (J4, engine#245 r2): el nombre de este archivo colisiona con
`ENG-VAL-006.md` y `ENG-VAL-006-CLEAN-ROOM.md`. No se renombra en este PR porque
rompería referencias; la decisión queda **PENDIENTE**.

Inventario leído exclusivamente en este repositorio: `tests/analytical.rs`,
`tests/support/table.rs`, `tests/support/harness.rs`, `tests/oracles.rs`,
`ORACLES.md`, `tests/dynamic.rs` y sus tablas/arneses, `tests/winkler.rs`,
`tests/winkler_elements.rs` y sus tablas/arneses, `tests/consolidation.rs`
y sus tablas/arneses, `tests/py.rs` y sus tablas/arneses, `tests/axial.rs`,
`tests/fixtures.rs`, `src/fixtures.rs` y los cinco JSON. Estas rutas son
relativas a `crates/engine-validation/`. Para axial se leyó además el arnés
reutilizado `crates/engine-geo/tests/support/axial_benchmarks.rs`.
Se revisaron las listas locales de identificadores y rutas prohibidos y
`crates/engine-linalg/tests/determinism_guard.rs`.

No se consultaron fuentes externas, otros repositorios ni resultados de otros
motores. Las únicas fuentes numéricas son las entradas y derivaciones propias
y existentes del árbol. Sin red, commits, eliminaciones ni cambios a archivos
preexistentes. Revisión clean-room humana: **PENDIENTE**.

## Inventario y decisiones por familia

`ORACLES.md` contiene **94** casos y `tests/oracles.rs` verifica sus metadatos:
42 estáticos lineales, 12 modales, 10 P-Delta, 9 espectrales, 6 Winkler,
7 de consolidación, 3 p-y y 5 documentos `fixture/1`. VAL-005 es el inventario,
no una nueva solución mecánica que agregar como una fila numérica.

- **(a) Lineal: 42 fixtures incluidas.** Reacciones completas disponibles en
  `CaseResults`. Se conservan exactamente los modelos del arnés existente,
  incluidos apoyos, liberaciones, asentamientos y rigidez finita de pórticos.
  Sus cuatro fixtures de combinación generan Q, W y D; además se comprueba
  la combinación C con `combine`. Son 38 filas Q + 4×4 filas = **54 filas**.
- **(a) Cinco JSON incluidos.** `NativeTarget` ejecuta sus entradas existentes;
  se leen todas las reacciones nodales de `results/1`. Incluyen barra axial,
  barras en serie, voladizo puntual, viga con carga distribuida y voladizo 3D.
- **(a) P-Delta con preesfuerzo cero: una fila incluida.** `pdelta-cero/H`
  reutiliza `column` y sus datos, con el mismo mallado y opciones de
  `dynamic.rs`. La gravedad G tiene carga cero, Kg=0, y el caso lateral H
  tiene reacciones físicas lineales. El adaptador nuevo reproduce únicamente
  las coordenadas y la carga H de esa entrada para sumar acciones físicas.
- **(b) Nueve P-Delta restantes: PENDIENTE.** No se confunden con modos.
  El resultado estable expone reacciones laterales de `(K+Kg)u−F`, separadas
  de la gravedad; el momento físico del sistema completo requiere decidir
  cómo incluir el preesfuerzo y la geometría desplazada. No equivale a sumar
  esas reacciones con H sobre posiciones originales. El caso inestable no
  tiene solución lateral, y la iteración escalar no es una estructura espacial.
- **(b) Doce modales y nueve espectrales: PENDIENTE/no aplicable a esta suma
  estática.** Modos libres y máximos combinados carecen de un único estado
  simultáneo de cargas físicas y reacciones. El cortante espectral existente
  no se presenta como cortante basal estático firmado.
- **(b) Seis Winkler: PENDIENTE.** Son estáticos, pero las reacciones físicas
  distribuidas del suelo no son las reacciones de los apoyos axiales/torsionales.
  `winkler_harness::solve` es privado y solo devuelve `Observation { w, m }`;
  integra el lecho internamente y no expone sus fuerzas/momentos completos.
  La prueba existente usa otra tolerancia para ese equilibrio. EQ-ALL no
  reconstruye reacciones desde la carga ni adopta esa tolerancia. Hace falta
  una salida del lecho evaluable a rtol=1e-10; no se declara que esté validada.
  `winkler_elements.rs` comprueba operaciones constitutivas, matrices,
  recuperación, rotación y cuadratura; no añade un caso global resuelto.
- **(b) Siete de consolidación: PENDIENTE/no aplicable.** La salida contiene
  presión de poros, asiento y flujo; no reacciones estructurales 3D.
- **(b) Tres p-y: PENDIENTE/no aplicable.** Tablas constitutivas locales y
  adaptación de unidades, sin estructura ni apoyos globales resueltos.
- **(b) Axial geotécnico: cinco fixtures adicionales fuera de ORACLES.md,
  PENDIENTE/no aplicable.** `tests/axial.rs` llama `run_all()` del arnés
  compartido: arena t-z, arcilla t-z, punta q-z, tabla de usuario t-z y tabla
  simétrica de usuario q-z. Son curvas locales, sin reacciones globales.
  Se describen por su función: ese arnés no les asigna identificadores VAL.

Resultado del inventario: **48 casos fuente incluidos**, que producen
**60 filas EQ-ALL**, y **46 casos fuente de la matriz pendientes**. Los cinco
axiales geotécnicos se registran aparte, sin inventarles ids.

## Tabla única y origen de las acciones

La prueba imprime una única tabla numérica de 60 filas: id, familia,
ΣF=[Fx,Fy,Fz,Mx,My,Mz] aplicada, ΣR en el mismo orden, escalas S_F/S_M,
residuo relativo máximo y cortante basal/carga lateral firmados con su eje.
Todos los momentos incluyen brazos respecto al punto fijo global
**O=(0,0,0) m**, tanto en las cargas como en las reacciones.

El manifiesto siguiente enumera exactamente las filas de esa tabla. La prueba
compara sus ids con esta nota, impone 60 filas distintas, 48 casos fuente,
94 nombres en la matriz y 46 pendientes documentados. No basta una tabla vacía
ni un subconjunto: verifica también acciones no nulas por fila, reacciones de
asentamientos, al menos ocho filas con carga lateral, presencia de Fz y My y
al menos treinta filas con Mz aplicado.

`N(c)` significa la suma de **todas** las cargas nodales del caso c de la
entrada del modelo, incluidos pares. `U(i–j; q)` significa integrar la carga
uniforme de la barra de entrada: F=q_f L y M=q_m L+r_c×F, con
L=norma(x_j−x_i) y r_c=(x_i+x_j)/2. α y β son los factores firmados existentes.
Se incluyen las seis componentes, aunque el modelo sea 2D.

El arnés lineal mantiene privadas sus cargas de barra: el manifiesto físico
nuevo usa sus pares de nodos y los parámetros de entrada de `table.rs`, con
un `match` exhaustivo por `Kind`. Las cargas nodales sí se leen directamente
de `problem.model.cases`. Los JSON se recorren directamente por `loads` y
`member_loads`, resolviendo cada barra por id en `input.model.members`.
La fila C suma entradas Q/W ponderadas, independientemente de `combine`.
No se usa el vector equivalente ensamblado, `load_vector`, ni el diagnóstico
de equilibrio del solver para calcular ΣF o la escala de EQ-ALL.

**Resultados de ejecución (verificador, fuera del sandbox de autoría, target
limpio y sin red, 2026-10-29).** La prueba pasa con las 60 filas. Residuos
relativos medidos (`--nocapture`, recompilado), de mayor a menor:
`portal-lateral-h5/Q` 4.069855563671e-11; `portal-lateral-h3/Q`
4.867069710220e-12; `pdelta-cero/H` 1.013911177239e-13;
`voladizo-par-terminal/Q` 1.421085471520e-14; las 56 filas restantes
≤ 7.105427357601e-15 (`voladizo-punta-L7/Q`). Es decir, 57 filas ≤ 1e-13 y
`pdelta-cero/H` apenas por encima (1.01e-13).

Margen: `portal-lateral-h5/Q` queda a 0.41 de la tolerancia (rtol 1e-10),
margen ≈2.5×, atribuible al contraste de rigidez 1e5 (`rigid_ratio`) de los
pórticos. La tolerancia no se relaja.
La tabla completa se reimprime con `--nocapture`; no se copia aquí para no
duplicar valores que salen de la ejecución.

| Caso EQ-ALL | Familia | Suma aplicada desde la entrada |
| --- | --- | --- |
| `voladizo-punta-L4/Q` | lineal | N(Q) |
| `voladizo-punta-L7/Q` | lineal | N(Q) |
| `voladizo-uniforme-L4/Q` | lineal | U(1–2, 2–3; w) |
| `voladizo-uniforme-L7/Q` | lineal | U(1–2, 2–3; w) |
| `voladizo-par-terminal/Q` | lineal | N(Q) |
| `simple-uniforme-L4/Q` | lineal | U(1–2, 2–3; w) |
| `simple-uniforme-L7/Q` | lineal | U(1–2, 2–3; w) |
| `simple-punto-centro-L4/Q` | lineal | N(Q) |
| `simple-punto-centro-L7/Q` | lineal | N(Q) |
| `simple-punto-cuarto/Q` | lineal | N(Q) |
| `simple-punto-40porciento/Q` | lineal | N(Q) |
| `simple-punto-70porciento/Q` | lineal | N(Q) |
| `biempotrada-uniforme-L4/Q` | lineal | U(1–2, 2–3; w) |
| `biempotrada-uniforme-L7/Q` | lineal | U(1–2, 2–3; w) |
| `biempotrada-punto-centro/Q` | lineal | N(Q) |
| `biempotrada-punto-cuarto/Q` | lineal | N(Q) |
| `biempotrada-punto-70porciento/Q` | lineal | N(Q) |
| `apuntalado-uniforme-L4/Q` | lineal | U(1–2, 2–3; w) |
| `apuntalado-uniforme-L7/Q` | lineal | U(1–2, 2–3; w) |
| `continua-dos-vanos-L4/Q` | lineal | U(1–2, 2–3, 3–4, 4–5; w) |
| `continua-dos-vanos-L7/Q` | lineal | U(1–2, 2–3, 3–4, 4–5; w) |
| `dos-barras-vertical/Q` | lineal | N(Q) |
| `dos-barras-mixta/Q` | lineal | N(Q) |
| `triangulo-vertical/Q` | lineal | N(Q) |
| `triangulo-mixta/Q` | lineal | N(Q) |
| `tres-concurrentes-vertical/Q` | lineal | N(Q) |
| `tres-concurrentes-mixta/Q` | lineal | N(Q) |
| `barra-asentamiento-positivo/Q` | lineal | N(Q)=0; desplazamiento prescrito s |
| `barra-asentamiento-negativo/Q` | lineal | N(Q)=0; desplazamiento prescrito s |
| `portal-lateral-h3/Q` | lineal | N(Q) |
| `portal-lateral-h5/Q` | lineal | N(Q) |
| `portal-vertical-h3/Q` | lineal | U(3–5, 5–4; w) |
| `portal-vertical-h5/Q` | lineal | U(3–5, 5–4; w) |
| `articulacion-extremo-L4/Q` | lineal | U(1–2; w) |
| `articulacion-extremo-L7/Q` | lineal | U(1–2; w) |
| `rotacion-modelo-30grados/Q` | lineal | N(Q) |
| `rotacion-modelo-90grados/Q` | lineal | N(Q) |
| `rotacion-modelo-137grados/Q` | lineal | N(Q) |
| `superposicion-P-mas-w/Q` | lineal | N(Q) |
| `superposicion-P-mas-w/W` | lineal | U(1–2, 2–3; w) |
| `superposicion-P-mas-w/D` | lineal | N(D) + U(1–2, 2–3; βw) |
| `superposicion-P-mas-w/C` | combinación | αN(Q) + βU(1–2, 2–3; w) |
| `combinacion-1_3-menos-0_7/Q` | lineal | N(Q) |
| `combinacion-1_3-menos-0_7/W` | lineal | U(1–2, 2–3; w) |
| `combinacion-1_3-menos-0_7/D` | lineal | N(D) + U(1–2, 2–3; βw) |
| `combinacion-1_3-menos-0_7/C` | combinación | αN(Q) + βU(1–2, 2–3; w) |
| `combinacion-menos-0_5-mas-2/Q` | lineal | N(Q) |
| `combinacion-menos-0_5-mas-2/W` | lineal | U(1–2, 2–3; w) |
| `combinacion-menos-0_5-mas-2/D` | lineal | N(D) + U(1–2, 2–3; βw) |
| `combinacion-menos-0_5-mas-2/C` | combinación | αN(Q) + βU(1–2, 2–3; w) |
| `combinacion-cero-mas-0_5/Q` | lineal | N(Q) |
| `combinacion-cero-mas-0_5/W` | lineal | U(1–2, 2–3; w) |
| `combinacion-cero-mas-0_5/D` | lineal | N(D) + U(1–2, 2–3; βw) |
| `combinacion-cero-mas-0_5/C` | combinación | αN(Q) + βU(1–2, 2–3; w) |
| `axial-bar/D` | fixture/1 | N(D) de input.cases[0].loads |
| `axial-series/D` | fixture/1 | N(D) de input.cases[0].loads |
| `cantilever-point/D` | fixture/1 | N(D) de input.cases[0].loads |
| `simply-supported-uniform/D` | fixture/1 | N(D) + U de input.cases[0].member_loads |
| `spatial-cantilever/D` | fixture/1 | N(D) de input.cases[0].loads |
| `pdelta-cero/H` | preesfuerzo cero | N(H); P=0; malla 16 de dynamic_table.rs |

## Tolerancia y justificación

rtol=**1e-10**, sin sumandos adicionales a la desigualdad relativa.
Sean A_F la suma L1 de las magnitudes de cada fuerza aplicada, y A_M la
suma de las magnitudes de los pares y de cada término de r×F, antes de
cancelaciones. Para C se acumulan magnitudes con |α| y |β|; los signos solo
se conservan en las sumas físicas. Sea L_O=max(1 m, max_n norma(x_n)), medido
respecto al mismo origen, y P_s=EA|s|/L únicamente para asentamientos.

- S_F=max(1 N, A_F, P_s).
- S_M=max(1 N·m, A_M, S_F L_O).
- e=max_i |ΣR_i+ΣF_i|/S_grupo(i), con grupo fuerza para i=0..2 y momento
  para i=3..5. Se exige e≤1e-10 y que cada cociente sea finito.

Así se comparan por separado fuerzas y momentos, sin mezclar unidades.
La escala usa únicamente datos de entrada; las reacciones no pueden inflar
el denominador. L_O convierte la escala de fuerza a momento y mantiene una
referencia cuando los momentos netos se cancelan. Las magnitudes antes de
cancelar permiten combinaciones firmadas y componentes teóricamente nulas.
Los pisos SI de escala equivalen a límites absolutos mínimos de 1e-10 N y
1e-10 N·m, útiles para ceros. No son una tolerancia sumada ni se amplían
para los pórticos con contraste de rigideces. En asentamientos sin cargas,
P_s es una referencia de esfuerzo físico de la deformación prescrita;
se exige además que las reacciones opuestas sean no nulas.

El cortante basal se declara con signo V_b=−ΣR_lateral y se compara con
ΣF_lateral usando S_F y la misma rtol. Para estas estructuras lineales se
usa X cuando existe carga X: los apoyos que reaccionan están en su base.
En `pdelta-cero/H` se usa Y, la dirección transversal de su columna en X.
La tabla muestra el índice del eje (0=X, 1=Y); sin carga lateral indica N/A.
No se transforma un valor absoluto en un resultado firmado.

Cobertura geométrica limitada a los casos presentes: el caso espacial activa
Fz y My. Ninguna entrada actual excita Mx; verificar su cero no acredita un
benchmark de torsión. Cobertura de Mx no nulo: **PENDIENTE**.

## DESCONOCIDO / PENDIENTE por caso existente

Cada caso pendiente conserva su nombre exacto de ORACLES.md y el motivo.
- `modal-cortante-1`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-cortante-2`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-cortante-3`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-cortante-4`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-cortante-5`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-variable-2x2`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-cadena-libre-fija`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-apoyada-malla4`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-apoyada-malla12`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-voladizo-malla4`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-voladizo-malla12`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `modal-voladizo-rotado`: PENDIENTE — modo propio, sin par simultáneo de cargas estáticas y reacciones.
- `pdelta-compresion-020`: PENDIENTE — reacciones laterales incrementales con preesfuerzo; balance físico completo de gravedad y geometría desplazada por definir.
- `pdelta-compresion-050`: PENDIENTE — reacciones laterales incrementales con preesfuerzo; balance físico completo de gravedad y geometría desplazada por definir.
- `pdelta-compresion-080`: PENDIENTE — reacciones laterales incrementales con preesfuerzo; balance físico completo de gravedad y geometría desplazada por definir.
- `pdelta-compresion-090`: PENDIENTE — reacciones laterales incrementales con preesfuerzo; balance físico completo de gravedad y geometría desplazada por definir.
- `pdelta-traccion`: PENDIENTE — reacciones laterales incrementales con preesfuerzo; balance físico completo de gravedad y geometría desplazada por definir.
- `pdelta-inestable`: PENDIENTE — estado inestable, sin desplazamientos ni reacciones laterales.
- `pdelta-piso-005`: PENDIENTE — reacciones laterales incrementales con preesfuerzo; balance físico completo de gravedad y geometría desplazada por definir.
- `pdelta-piso-010`: PENDIENTE — reacciones laterales incrementales con preesfuerzo; balance físico completo de gravedad y geometría desplazada por definir.
- `pdelta-iteracion-geometrica`: PENDIENTE — iteración escalar artificial; no hay geometría estructural ni balance 3D de momentos.
- `espectro-srss-2pisos`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-desplazamiento-modal`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-interpolacion-lineal`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-interpolacion-loglog`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-cqc-separados`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-cqc-cercanos`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-cqc-diagonal`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-direccional-ponderado`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `espectro-direccional-srss`: PENDIENTE — respuesta/interpolación/combinación espectral, sin estado estático firmado único.
- `Hetenyi-centro-2d`: PENDIENTE — el arnés no expone fuerzas y momentos completos del lecho; apoyos axiales/torsionales no representan el suelo.
- `Hetenyi-extremo-2d`: PENDIENTE — el arnés no expone fuerzas y momentos completos del lecho; apoyos axiales/torsionales no representan el suelo.
- `rigida-centro-2d`: PENDIENTE — el arnés no expone fuerzas y momentos completos del lecho; apoyos axiales/torsionales no representan el suelo.
- `Hetenyi-centro-3d-dos-planos`: PENDIENTE — el arnés no expone fuerzas y momentos completos del lecho; apoyos axiales/torsionales no representan el suelo.
- `Hetenyi-extremo-3d-dos-planos`: PENDIENTE — el arnés no expone fuerzas y momentos completos del lecho; apoyos axiales/torsionales no representan el suelo.
- `rigida-centro-3d-dos-planos`: PENDIENTE — el arnés no expone fuerzas y momentos completos del lecho; apoyos axiales/torsionales no representan el suelo.
- `Terzaghi-dren-superior`: PENDIENTE — consolidación: presión, asiento y flujo, sin reacciones estructurales globales.
- `Terzaghi-dren-inferior`: PENDIENTE — consolidación: presión, asiento y flujo, sin reacciones estructurales globales.
- `Terzaghi-doble-drenaje`: PENDIENTE — consolidación: presión, asiento y flujo, sin reacciones estructurales globales.
- `Terzaghi-rampa`: PENDIENTE — consolidación: presión, asiento y flujo, sin reacciones estructurales globales.
- `Terzaghi-rampa-descarga`: PENDIENTE — consolidación: presión, asiento y flujo, sin reacciones estructurales globales.
- `multicapa-flujo-interfaz`: PENDIENTE — consolidación: presión, asiento y flujo, sin reacciones estructurales globales.
- `multicapa-no-transformable`: PENDIENTE — consolidación: presión, asiento y flujo, sin reacciones estructurales globales.
- `py-matlock-estatico`: PENDIENTE — curva constitutiva local, sin estructura global resuelta.
- `py-reese-superficial`: PENDIENTE — curva constitutiva local, sin estructura global resuelta.
- `py-reese-profundo`: PENDIENTE — curva constitutiva local, sin estructura global resuelta.

Además, arena t-z, arcilla t-z, punta q-z y las dos tablas axiales de usuario:
**PENDIENTE**, sin reacciones globales; ids VAL propios **DESCONOCIDOS**.

## Casos VAL del plan sin cobertura ejecutable en este alcance

VAL-002, VAL-003, VAL-004 y VAL-006..VAL-041: **PENDIENTE** como casos
independientes del informe para esta entrega. La presencia de una fórmula
semejante en ENG-VAL-001, o de documentación/fixtures en otras áreas del
repositorio, no establece correspondencia ni valida esos ids en EQ-ALL.
VAL-039 (casos patológicos, MAM-851, #244) y VAL-040 (invariancia, MAM-852,
#243) ya están en main, pero siguen **PENDIENTE** en EQ-ALL: el inventario de
esta entrega se hizo antes de su fusión y sus casos no se revisaron para la
tabla. Gran parte de VAL-039 son entradas que el Engine rechaza con código de
error, sin reacciones que sumar; qué casos de VAL-039 y VAL-040 admiten la suma
ΣR+ΣF=0: **DESCONOCIDO**. Correspondencia uno a uno entre ids del plan y
fixtures: **DESCONOCIDO** desde este inventario. No se rellenan con datos supuestos.

## Reproducción y estado de comprobaciones

Desde la raíz del repositorio:

```sh
cargo test --offline -p engine-validation --test equilibrium_global
```

Para mostrar la única tabla aun cuando la prueba pase:

```sh
cargo test --offline -p engine-validation --test equilibrium_global -- --nocapture
```

Verificación externa (target limpio, sin red, 2026-10-29): `cargo fmt --all --
--check` OK (el verificador aplicó `cargo fmt` a la prueba nueva: solo saltos
de línea); `cargo test --offline --workspace` exit 0 (886 passed, 0 failed, sobre main 62593f69);
`determinism_guard` 37/37; `cargo clippy --offline --workspace --all-targets
-- -D warnings` OK; `clean-room/check.sh` OK; `git diff --check` OK.

## Lo que EQ-ALL no detecta por construcción

Las reacciones se recuperan como K·u − F, de modo que su suma con las cargas
permite a EQ-ALL detectar errores en el residuo del solve y en la resultante
del vector de cargas, pero no errores de K que preserven los modos de cuerpo
rígido. EQ-ALL no detecta errores que conservan la resultante: EI o EA
erróneos, signo de los
momentos de empotramiento perfecto, o un reparto equivocado de las cargas
entre nudos que mantenga fuerza y momento totales. Esos errores los cubren,
en su caso, las comparaciones analíticas de ENG-VAL-001/ENG-VAL-005.
Tampoco comprueba los esfuerzos internos de barra ni la magnitud de los
desplazamientos (N1/N2, J4 engine#245 r2).

## Mutaciones

Mutaciones reversibles del verificador; cada archivo se restauró y se
comprobó idéntico con `cmp`. Comando: `cargo test --offline -p
engine-validation --test equilibrium_global`.

| Mutación | Cambio aplicado | Resultado | Evidencia |
| --- | --- | --- | --- |
| M1 | Brazo de momento: `x * fy` → `-x * fy` en la suma de momentos | Falla | `voladizo-punta-L4/Q` componente 5, residuo relativo 1.19 |
| M2 | Resultante de carga de barra ×1.01 (`length` → `1.01 * length`) | Falla | `voladizo-uniforme-L4/Q` componente 1, residuo 9.90e-3 |
| M3 | Omitir las reacciones del primer nodo recuperado | Falla | aserción de asentamiento (reacciones nulas) en `barra-asentamiento-positivo/Q`, antes de la suma |
| M4 | Producción: `combination.rs` factor de reacciones ×1.000001 | Falla | la detecta primero la comprobación propia del arnés (ENG-E-0012, superposición), no la tabla EQ-ALL |
| M5 | Reacciones recuperadas ×(1+1e-8) | Falla | `voladizo-punta-L4/Q` componente 1, residuo 1.0e-8 > 1e-10 |
| M6 | Reacciones recuperadas ×(1+1e-12) | Pasa | por debajo de rtol 1e-10, esperado (límite de sensibilidad) |

DESCONOCIDO: una mutación de producción que solo detecte EQ-ALL y no el arnés.
