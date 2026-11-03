# VAL-032 r3 · Edificios de validación (MAM-847, padre MAM-604)

Los siete fixtures se regeneran con `_ref/gen.py`, sin red; MAM-860 usa NumPy
preinstalado para la referencia independiente E3. La ronda 2 actualiza los
estados documentales: E1/E2/E4 pasan a **1.1.2**, E2b a **1.0.2** y E3 a
**1.1.3** (tras los criterios internos de 1.1.2). E5 conserva **1.1.1**; E5b
pasa a **1.0.2** por la aclaración de prosa de requisitos de formato. Ningún modelo físico, `runs[].engine_input` ni tolerancia cambia
en esta ronda. Se reexpresa la prosa heredada de precisión de salida (al menos
diez cifras significativas), sin cambiar el requisito ni los criterios, y se
remiten los diagnósticos históricos J4 r2 a este README. Esto evita que la
guardia clean-room interprete instrucciones como salidas de oráculos.
Se sustituyen las versiones anteriores; el CSV no cambia.
Todos los edificios tienen 72 nodos conectados. Los doce runs son ejecutables
con `status: warning`: los ocho modal/espectro tras #188 (#184, Guyan),
E3-pdelta tras #185/#186 y los tres estáticos lineales. Los nueve runs antes
bloqueados conservan únicamente `API-W-DIAPHRAGM-RECOVERY` (#84).

Estado comprobado sobre el merge de `origin/main` `c6dd075` (#188),
2026-10-20. MAM-860 regenera `info.estado`, `info.estado_engine`, estados y
brechas del índice, y sus hashes. Las evidencias `_ref/*r3.json` permanecen
byte a byte: describen el cierre histórico anterior a los arreglos modales y
P-Delta; sus anotaciones de bloqueo no representan el estado actual.

`index.json` registra versiones, hashes de fixtures/herramientas, las evidencias
`_ref/calibrate_r3.json` y `_ref/engine_r3.json`, y los hashes de los fuentes Rust
que respaldan la ejecución. El calibrador compara la definición anterior de
maestro flotante en CM, conservada en `/tmp/v32run-<run>.json` (2a22e37), con la
nueva mediante un modelo numpy independiente. Los períodos T1..T15, las respuestas
lineales de piso y las reacciones exigidas difieren ≤1e-10 relativo. Esta prueba
no habilita los análisis dinámicos del Engine.

## Formato y alcance

La envoltura `val032-fixture/1` contiene el modelo neutral y los doce
`runs[].engine_input` completos. Los siete documentos separados `bench-model/E*.json`
son conformes a **PR #183, SHA `797535d86af8fe6f57f935fbaa978e14b15cfe89`**,
esquema SHA-256 `4e4aa7cd2278767a0f9b2899a77657a980230539bfff3ea3a18fb7f6d0e8b005`.
`bench_model_omissions` está vacío: el esquema permite
representar el edificio. `bench_model_translation.checks[].engine_gaps` está vacío
para los siete modelos; todos figuran como `compatible`.

La traducción exige igualdad numérica exacta y listas ordenadas idénticas, salvo
biyecciones explícitas de ids. `nodal_masses` conserva todos los pesos Ux/Uy y
Rz_extra; no activa el reparto automático de masas del traductor #183.
Los doce documentos originales y traducidos pasan el esquema Engine conservado
en `_ref/engine-input-1.json`. La equivalencia de resultados entre formatos a
1e-12 queda pendiente; 1e-3 de Q3 solo es equivalencia de formato.

Los estados actuales del Engine se comprueban en los documentos `results/1`
de `crates/engine-api/tests/val032_runs.rs`. `_ref/record_engine.py` conserva
el registro histórico de estado, diagnósticos íntegros, hashes de entrada/salida
y convergencia en `engine_r3.json`; no se regenera en esta ronda.
La referencia numpy usa solo geometría, propiedades, masas y cargas de los JSON;
no consume resultados del Engine. No se ejecutaron OpenSees/Frame3DD y sus
criterios de comparación siguen pendientes.

## Base común y validez

- Ubicación supuesta: **Sierra**, η = 2.48. Zona V, Z = 0.40; suelo D,
  Fa = 1.20, Fd = 1.19 y Fs = 1.28. No representa otra ubicación sin revisar el espectro.
- Planta de 15 × 10 m: ejes X = 0, 5, 10, 15; Y = 0, 5, 10 m.
  Cinco pisos de 3 m; 12 columnas por planta y 12 bases empotradas.
- Hormigón f'c = 28 MPa; Ec = 4700√f'c = 24 870.062 MPa
  (ACI 318-25 §19.2.2.1(b)); ν = 0.2; G = Ec/2.4.
- Columnas 0.50 × 0.50 m, salvo E3 (0.25 × 0.25 m); vigas 0.30 × 0.50 m.
  **Secciones brutas, sin reducción por NEC-SE-DS §6.1.6**; Euler-Bernoulli,
  sin deformación por cortante y **sin nudos rígidos**. J torsional por fórmula
  de rectángulo de Roark. Es una verificación de código, no un diseño conforme a NEC.
- Orientación de columnas `[1,0,0]`, vigas `[0,0,1]`: referencia z local;
  Iy = b h³/12 resiste la gravedad. J4 confirmó esta convención en su revisión.
- D = 6 kPa (losa, vigas, acabados), L = 2 kPa; fuerzas nodales por áreas
  tributarias: esquina 6.25, borde 12.5, interior 25 m². No hay cargas de miembro.
  E3 añade 16 kPa de carga muerta artificial exclusivamente en G.
- **rho = 0** en todas las leyes: no hay peso propio ni masa distribuida implícitos.
  Fuente sísmica W = D (§6.1.7), sin fracción de L: m = D·150/g = 91 774.459 kg/piso,
  g = 9.80665 m/s². La carga adicional G de E3 no aporta masa ni cambia el ELF.
  E5b es exclusivamente estático, con **`masses: []`**.
- En los modelos con diafragma, masa Ux/Uy e inercia
  **J_Rz = m(Lx² + Ly²)/12 alrededor del CM**. Se supone una losa uniforme
  desplazada, **también en E1**; no se modela una losa de densidad no uniforme
  ni se añade término de ejes paralelos al J alrededor del propio CM.
- Cada maestro es un nodo existente de columna; no se crean nodos flotantes.
  Uz/Rx/Ry permanecen libres en **todos** los nodos de piso. Solo hay apoyos en
  las 12 bases, que no pertenecen a ningún diafragma. E1/E2 conservan su
  excentricidad física; **no hay torsión accidental salvo en E2b**.
- I = 1.0 **NO VERIFICADO contra Tabla 6, pág. 39**. R = 8, φP = φE = 1,
  ξ = 0.05. La adopción de R no prueba requisitos de detallado o diseño.

## Modelos

| ID | Versión | Nudos | Barras | Qué ejercita / runs |
|---|---|---:|---:|---|
| E1 | 1.1.2 | 72 | 145 | CM = (9, 5.5) m, e = (1.5, 0.5) m; modal, 15 modos acoplados |
| E2 | 1.1.2 | 72 | 145 | E1 + espectro; CQC-X, SRSS-X, CQC-Y |
| E2b | 1.0.2 | 72 | 145 | E4 con CM = (8.25, 5.75) m; CQC-X, CQC-Y y CQC-XY/SRSS direccional, factores [1,1] |
| E3 | 1.1.3 | 72 | 145 | Columnas 0.25 × 0.25 m; G, EX; linearStatic y pDelta |
| E4 | 1.1.2 | 72 | 145 | CM = (7.5, 5) m, control simétrico; modal, 15 modos |
| E5 | 1.1.1 | 72 | 145 | E4; D, L, EX, EY y 10 combinaciones; barras solo diagnóstico (#84) |
| E5b | 1.0.2 | 72 | 205 | E4 sin diafragmas ni masas; 60 diagonales truss, mismos casos y combinaciones; barras como criterio |

Todos los modelos tienen ≤256 nodos. Nodos de pórtico: `100k + 4j + i + 1`.
Los maestros se eligen de esos mismos nodos (tabla siguiente); cada diafragma
contiene un maestro y 11 esclavos. No contiene ninguna base.

## Maestro conectado, masa y carga exactamente equivalentes

Para k=1..5, se elige el nodo conectado más cercano al CM entre los que cumplen
x≠x_cm e y≠y_cm. Desempate: mayor x, menor y, menor id. Esto mantiene momentos
transportados no nulos en X/Y. La elección (10,0) en los modelos centrados evita
los rechazos numéricos por cancelación de la recuperación #84 observados con
maestros en la línea y=5 y con (5,0). No cambia cargas generalizadas ni rigidez.
Los maestros se retiran de la lista de esclavos y desaparecen los apoyos
`master_*`. No se restringen Uz/Rx/Ry de ningún nodo de piso.

m = 91774.45916801355 kg/piso; Iz_cm = m(15²+10²)/12 = 2485558.6024670335 kg·m².
Cada peso w_i se declara tanto en Ux como en Uy; los demás nodos no reciben
masa traslacional. La tabla se repite en los cinco pisos (suma de pesos = m).

| Edificio | Maestro por piso; (x,y) m | Nodos con Ux=Uy y fracción w_i/m | Rz_extra (kg·m²) |
|---|---|---|---:|
| E1, E2 | 107/207/307/407/507; (10,5) | 100k+6: 0.18; +7: 0.72; +10: 0.02; +11: 0.08 | 1911967.8993336158 |
| E2b | 107/207/307/407/507; (10,5) | 100k+6: 0.2975; +7: 0.5525; +10: 0.0525; +11: 0.0975 | 1671059.9440175805 |
| E3, E4, E5 | 103/203/303/403/503; (10,0) | 100k+6: 0.5; +7: 0.5 | 1911967.8993336158 |
| E5b | Sin diafragma | Sin masas; cargas tributarias originales | No aplica |

Se comprueban w_i≥0, Σw_i=m y Σw_i(x_i,y_i)=m(x_cm,y_cm), conservando también
la excentricidad accidental E2b. Con r_i respecto del nuevo maestro:

- J_total_master = Iz_cm + m·|r_cm|².
- Rz_extra = J_total_master − Σw_i·|r_i|² ≥ 0, colocado en Rz del maestro.
- Σw_i·|r_i|² + Rz_extra = J_total_master.

La transformación exacta del Engine produce la matriz de masa de piso
`[[m,0,−m·r_cm_y],[0,m,m·r_cm_x],[−m·r_cm_y,m·r_cm_x,J_total_master]]`.
**La petición m·d² en Iz, por sí sola con m en el nuevo maestro y masa diagonal,
no reproduce los términos de acoplamiento. Solo es exacta con el reparto
anterior.** La evidencia incluye la matriz objetivo/reconstruida y un control
negativo: colocar toda m en el maestro y solo corregir Iz falla a 1e-10.

Toda fuerza lateral anterior en el maestro-CM pasa al nuevo maestro con igual
F y Mz_adicional=(x_cm−x_m)Fy−(y_cm−y_m)Fx. En E3/E5: EX tiene Mz=−5F y EY,
cuando aplica, Mz=−2.5F. E5b conserva su reparto tributario. Para comparar piso,
se reconstruyen Ux/Uy en el CM real desde el maestro; también se comprueban las
dos esquinas. Los adaptadores modales deben hacer la misma reconstrucción.

`_ref/calibrate_r3.json` contiene las fuentes anteriores y sus hashes para los
**doce runs**, masas y matrices por piso, ambos juegos de períodos y los errores
de cada comparación. Su SHA-256 está en `index.json.calibration_r3`. El error
estático es la norma infinito de la diferencia dividida por la norma infinito
anterior, por componente; componentes nulas (<1e-12 de la máxima) usan la escala
máxima de la respuesta. Los períodos se comparan uno por uno. Se verifican ELF
X/Y, desplazamientos/derivas CM y esquinas, E3 lineal y reacciones de todos los
casos y diez combinaciones de E5/E5b. En G/D/L se comparan los seis GDL de todos
los nodos de piso. E5b es exclusivamente estático: períodos no aplican.

| Edificio | Máximo error relativo anterior/nuevo (límite 1e-10) |
|---|---:|
| E1 | 3.138039e-13 |
| E2 | 3.138039e-13 |
| E2b | 2.673897e-13 |
| E3 | 1.009209e-13 |
| E4 | 3.236380e-14 |
| E5 | 1.054503e-11 |
| E5b | 0.000000e+00 |

E2b aplica **+0.05·Lmax = +0.75 m en X y en Y**, NEC-SE-DS §6.3.6–6.3.7,
págs. 66–67. Se prueba esta rama positiva; no es una envolvente reglamentaria
completa de excentricidades ±. Se reportan cortante basal, derivas CM/esquinas
mediante el postproceso común y Ax por piso.

E5b conserva todos los nodos de pórtico y barras originales de E4, elimina
masas y diafragmas, y añade ambas diagonales cruzadas de cada paño y piso
(`truss3d.linear`, sin nudo en los cruces). Tiene ley `losa_elastica` y sección
`losa_diagonal` propias: E = Ec, A = 10 m², EA = 2.4870062324e11 N.
EA/Ldiag ≈ 3.52e10 N/m es unas 611 veces 12EcIcol/h³ de una columna y 13 000
veces la rigidez de flexión en plano de una viga de 5 m. Es una penalización de
rigidez finita para simular losa rígida en plano, sin flexión de losa; no representa
su sección física ni exige igualdad exacta con las fuerzas de E4. EX/EY se
reparten a los 12 nudos por área tributaria y conservan la resultante y su línea
de acción. D/L y las 10 combinaciones son idénticas a E5.

La salida de los oráculos debe alcanzar **≥10 cifras significativas** para ser
comparable como criterio. **Frame3DD imprime en coma fija, no en notación
científica**: se deben ajustar las unidades (p.ej. mm, kN) para alcanzar esa
precisión. **Q2 debe comprobarlo en una corrida en seco de formato antes de
congelar**. Si no se alcanza, **E5b frente a Frame3DD y la parte afectada de E3
son solo diagnóstico, no criterio**. Si un oráculo modela las diagonales truss
como barras con I pequeña, debe declarar esa aproximación y la I utilizada.
La misma corrida en seco de formato de Q2 debe confirmar también si la precisión
de salida limita los criterios lineales 1e-4/1e-5 de E1, E2, E2b, E4 y E5
frente a Frame3DD; queda como seguimiento y no cambia esos umbrales.

Combinaciones declaradas NEC-SE-CG §3.4.3: C1 = 1.4D; C2 = 1.2D + 1.6L;
C5 = 1.2D + L ± EX/EY; C7 = 0.9D ± EX/EY. **La numeración 1, 2, 5, 7 y sus
factores están NO VERIFICADOS**; no había copia NEC-SE-CG en la revisión.
Se omiten Lr/S/R por no existir esos casos aquí. Envolventes: max/min de las 10
combinaciones, fuera del Engine, en las mismas estaciones de extremo.

## Estado del Engine por run

Estados y diagnósticos actuales de los doce documentos reales `results/1` del
test, tras #188 y #185/#186. `_ref/engine_r3.json` conserva la ejecución
anterior a los arreglos. Ningún run devuelve `invalid` ni errores. El test
exige `warning`, los códigos y cantidades revisados, y concordancia con
`runs[].info`, su resumen `info.estado_engine` y el índice (estados y brechas).
También comprueba los hashes de fixtures y fuentes Rust del índice.

| Run | status | Error / bloqueo | Warnings revisados |
|---|---|---|---|
| E1-modal | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E2-cqc-X | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E2-srss-X | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E2-cqc-Y | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E2b-cqc-X | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E2b-cqc-Y | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E2b-cqc-XY-srss | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E3-lineal | warning | Sin errores | 1 × #84 |
| E3-pdelta | warning | Sin errores; converge tras #185 | 1 × #84 |
| E4-modal | warning | Sin errores; Guyan tras #188 | 1 × #84 |
| E5-combinaciones | warning | Sin errores | 1 × #84 y 10 × #85 |
| E5b-combinaciones | warning | Sin errores | 10 × #85 |

### Justificación individual de los 31 Warnings

**#84 = `API-W-DIAPHRAGM-RECOVERY`**, causa literal en `results/1`:
«#84: recuperación con diafragmas sin cobertura de pruebas del dominio; revisar
equilibrio y resultados.» Se emite una vez por documento con diafragma, incluso
antes del fallo del solver. Cada ocurrencia queda justificada abajo; el aviso
no demuestra convergencia ni valida las fuerzas recuperadas de barra.

| Run | Warning | Justificación y consecuencia |
|---|---|---|
| E1-modal | API-W-DIAPHRAGM-RECOVERY (#84) | Tiene diafragmas exactos XY; recuperación sin cobertura del dominio. Guyan permite obtener modos; la recuperación de esfuerzos/reacciones sigue bajo aviso #84. |
| E2-cqc-X | API-W-DIAPHRAGM-RECOVERY (#84) | Diafragmas XY en CQC-X; recuperación sin cobertura. Entrega respuesta espectral; la recuperación de esfuerzos/reacciones sigue bajo aviso #84. |
| E2-srss-X | API-W-DIAPHRAGM-RECOVERY (#84) | Diafragmas XY en SRSS-X; recuperación sin cobertura. Entrega respuesta espectral; la recuperación de esfuerzos/reacciones sigue bajo aviso #84. |
| E2-cqc-Y | API-W-DIAPHRAGM-RECOVERY (#84) | Diafragmas XY en CQC-Y; recuperación sin cobertura. Entrega respuesta espectral; la recuperación de esfuerzos/reacciones sigue bajo aviso #84. |
| E2b-cqc-X | API-W-DIAPHRAGM-RECOVERY (#84) | Diafragmas XY con CM accidental en CQC-X; recuperación sin cobertura. Entrega respuesta espectral y derivas; fuerzas/reacciones siguen bajo aviso #84. |
| E2b-cqc-Y | API-W-DIAPHRAGM-RECOVERY (#84) | Diafragmas XY con CM accidental en CQC-Y; recuperación sin cobertura. Entrega respuesta espectral y derivas; fuerzas/reacciones siguen bajo aviso #84. |
| E2b-cqc-XY-srss | API-W-DIAPHRAGM-RECOVERY (#84) | Diafragmas XY con CM accidental y dos excitaciones; recuperación sin cobertura. Entrega respuesta direccional combinada; fuerzas/reacciones siguen bajo aviso #84. |
| E3-lineal | API-W-DIAPHRAGM-RECOVERY (#84) | G/EX usan diafragmas XY; recuperación sin cobertura. El run lineal entrega resultados; fuerzas de barra requieren revisión de equilibrio y comparación independiente antes de validarse. |
| E3-pdelta | API-W-DIAPHRAGM-RECOVERY (#84) | pDelta usa diafragmas XY; converge tras #185. La regresión independiente compara θ y deriva CM por piso; el aviso sigue exigiendo revisión de equilibrio y recuperación de fuerzas de barra. |
| E4-modal | API-W-DIAPHRAGM-RECOVERY (#84) | El control simétrico también tiene diafragmas XY; recuperación sin cobertura. Entrega modos y participación; la recuperación de esfuerzos/reacciones sigue bajo aviso #84. |
| E5-combinaciones | API-W-DIAPHRAGM-RECOVERY (#84) | Casos y combinaciones con diafragmas XY; recuperación sin cobertura. Todas las fuerzas/momentos de barra, también combinaciones/envolventes, son diagnóstico con tolerancia null; desplazamientos/reacciones conservan sus criterios pendientes. |

**#85 = `API-W-COMBINED-PRESCRIBED`**, causa literal para cada id:
«#85: combinación [id]: prescritos = suma factor·δ; un asentamiento único se
pondera por su factor. Asentamientos repetidos se rechazan.» En E5 y E5b solo
hay empotramientos de base con δ=0; no hay asentamientos impuestos. El test
comprueba todos los prescritos de las diez combinaciones de cada run: solo
ceros o `null` (GDL libre). Cada fila corresponde a un diagnóstico distinto,
con `entities[0].id` igual al id indicado. Los términos siguientes expresan
esa suma de **desplazamientos prescritos nulos**, no las cargas aplicadas.

| Run | Warning | Combinación (entity id) | Justificación individual: δ combinado |
|---|---|---|---|
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C1_1.4D | 1.4·0 = 0; empotramiento conservado. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C2_1.2D+1.6L | 1.2·0 + 1.6·0 = 0; sin asentamiento. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L+EX | 1.2·0 + 0 + 0 = 0; EX no prescribe desplazamientos. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D+EX | 0.9·0 + 0 = 0; EX no prescribe desplazamientos. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L-EX | 1.2·0 + 0 − 0 = 0; −EX no cambia el empotramiento. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D-EX | 0.9·0 − 0 = 0; −EX no cambia el empotramiento. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L+EY | 1.2·0 + 0 + 0 = 0; EY no prescribe desplazamientos. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D+EY | 0.9·0 + 0 = 0; EY no prescribe desplazamientos. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L-EY | 1.2·0 + 0 − 0 = 0; −EY no cambia el empotramiento. |
| E5-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D-EY | 0.9·0 − 0 = 0; −EY no cambia el empotramiento. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C1_1.4D | 1.4·0 = 0; empotramiento conservado, sin MPC. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C2_1.2D+1.6L | 1.2·0 + 1.6·0 = 0; sin asentamiento, sin MPC. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L+EX | 1.2·0 + 0 + 0 = 0; EX tributario solo aplica fuerzas. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D+EX | 0.9·0 + 0 = 0; EX tributario solo aplica fuerzas. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L-EX | 1.2·0 + 0 − 0 = 0; −EX tributario no mueve la base. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D-EX | 0.9·0 − 0 = 0; −EX tributario no mueve la base. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L+EY | 1.2·0 + 0 + 0 = 0; EY tributario solo aplica fuerzas. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D+EY | 0.9·0 + 0 = 0; EY tributario solo aplica fuerzas. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C5_1.2D+L-EY | 1.2·0 + 0 − 0 = 0; −EY tributario no mueve la base. |
| E5b-combinaciones | API-W-COMBINED-PRESCRIBED (#85) | C7_0.9D-EY | 0.9·0 − 0 = 0; −EY tributario no mueve la base. |

No se observaron otros Warnings. El test rechaza códigos/run no enumerados,
exige un único #84 por run con diafragma y exactamente los diez ids de #85
sin duplicados en cada run de combinaciones. Los #85 no eximen las pruebas de
superposición/reacciones ni las comparaciones pendientes con oráculos.

## Espectro NEC-SE-DS 2015

NEC-SE-DS **§3.3.1, pág. 34**, para Sierra (η = 2.48), en unidades de g:

- T < T0: Sa = Z·Fa·[1 + (η−1)T/T0].
- T0 ≤ T ≤ Tc: Sa = η·Z·Fa = 1.1904.
- T > Tc: Sa = η·Z·Fa·(Tc/T)^r, r = 1.

T0 = 0.10·Fs·Fd/Fa = **0.12693333333333331 s** (≈0.1269 s);
Tc = 0.55·Fs·Fd/Fa = **0.6981333333333334 s**.
La tabla tiene **203 puntos**, de 0 a 4 s cada 0.02 s, más T0 y Tc calculados
sin redondear los quiebres. `nec_se_ds_tabla_Sa.csv` contiene T_s, Sa_g y Sa_m_s2;
los pares T_s/Sa_m_s2 son exactamente los mismos que `pairs` de todos los runs
espectrales. Interpolación lineal, extrapolación prohibida. El espectro es
**elástico**, en m/s² = Sa(g)·g, sin I/R; cualquier escalado de diseño debe
aplicarse explícita e idénticamente en ambas fuentes, fuera de este postproceso.

ELF de E3/E5/E5b: Ta = 0.055·15^0.9 = 0.629281 s; k = 1.064641;
Cs = I·Sa/(R φP φE) = 0.1488; W = 4500 kN; V = 669.6 kN.
F_i ≈ 41.243, 86.265, 132.834, 180.436, 228.822 kN.
El peso sísmico de referencia para ELF y estas fuerzas son iguales en E3, E5
y E5b; E5b no almacena masas.

## Estabilidad de E3

**E3-pdelta converge tras #185 (pDelta + diafragma):** la proyección de rigidez,
fuerzas geométricas y residual por la transformación del diafragma resuelve
ENG-E-0007. Termina en 14 iteraciones con `status: warning`, solo #84, igual
que E3-lineal. La regresión `crates/engine-api/tests/pdelta_diaphragm.rs` lee
este mismo fixture y compara amplificación y deriva CM de los cinco pisos
con la referencia propia P-Δ+P-δ de MAM-860: **1e-3 relativo por piso frente
a 8 y 16 elementos por columna**, **1e-9 frente a un elemento**. Los controles
previos de θ y deriva a 1e-4 se refuerzan a 1e-9 para la misma discretización.
Las estimaciones θ de primer orden siguientes siguen siendo lineales y no
se sustituyen por θ P-Δ. Las comparaciones externas siguen pendientes.

NEC-SE-DS §6.3.8, págs. 67–68:
θ_i = P_i Δ_i/(V_i h_i), con P_i = (D_total + L) **sin mayorar** acumulada desde
el piso i hasta cubierta, Δ_i = diferencia de desplazamientos de primer orden
en CM bajo EX, V_i = suma de las fuerzas EX desde i hasta cubierta y h_i = 3 m.
Aquí D_total = D + 16 kPa adicionales solo G; G = 24 kPa. No cambia W ni ELF.

El método `pDelta` del Engine usa la **matriz geométrica consistente Hermite**,
con coeficientes N/(30L)·[36, 3L, 4L², −L²]
(`crates/engine-elements-frame3d/src/geometric.rs`): incluye **P-Δ + P-δ por
elemento**, no P-Δ puro de cuerda. La comparación con un oráculo de solo cuerda
es diagnóstica: la diferencia de formulación se documenta solo en el repo de oráculos.
La configuración obligatoria del oráculo con **Kg consistente (Frame3DD)** es:
**`shear=0`** (sin deformación por cortante), **`geom=1`** (rigidez geométrica
activada), **1 elemento por columna igual que el Engine** y tolerancia de
**Newton-Raphson ≤1e-9**.

Frame3DD no ofrece constraints de diafragma: se debe declarar su representación
con la **penalización de diagonales axiales de E5b**, misma disposición y mismo
EA/A: E = Ec, A = 10 m², EA = 2.4870062324e11 N. El efecto medido es
**≤5.3e-6 (J4 r2)**. Frame3DD incluye términos geométricos de torsión-alabeo que
el Engine no tiene; en E3 (carga X, planta simétrica) se esperan despreciables,
**no cuantificado**. También aplica a E3 la comprobación de ≥10 cifras
significativas mediante ajuste de unidades y corrida en seco de formato de Q2;
si no se alcanza, la parte afectada frente a Frame3DD es solo diagnóstico.

**Estimación previa con modelo independiente de J2, no oráculo**:
frame3d Euler 3D, ensamblaje de rigidez axial/flexión/torsión y transformación
exacta del diafragma XY, resuelto con numpy en `_ref/val032_calibrate.py`.
`gen.py` guarda las Δ de esa calibración y genera P, V y θ a partir de ellas;
si cambia la rigidez o EX hay que recalibrar esas constantes. La confirmación
independiente reconstruye el modelo desde el JSON final y contrasta cada fila.

| Piso | Δ_i (m) | P_i (N) | V_i (N) | θ_i |
|---|---:|---:|---:|---:|
| 1 | 0.017584054252 | 18000000 | 669600.000000 | 0.1575632101 |
| 2 | 0.018585878028 | 14400000 | 628357.381781 | 0.1419768704 |
| 3 | 0.016160306775 | 10800000 | 542092.308134 | 0.1073195534 |
| 4 | 0.012299932401 | 7200000 | 409258.412492 | 0.0721300696 |
| 5 | 0.007040564536 | 3600000 | 228822.173173 | 0.0369224596 |

El máximo ≈0.158 cumple el objetivo ≈0.15, hay 3 pisos ≥0.10 y ninguno >0.25.
El límite 0.25 de este fixture es más conservador que el 0.30 normativo revisado
por J4. La aproximación 1/(1−θ_i) se contrasta con la amplificación de **deriva de
entrepiso**, no con el cociente de desplazamientos absolutos de cada piso.

### Referencia convergida propia de E3 · MAM-860

`_ref/e3_pdelta_reference.py` y su JSON son la referencia interna canónica.
Se derivan desde la energía de una viga Euler 3D:

`U = 1/2 ∫ [EA u′² + GJ φ′² + EIy w″² + EIz v″² + N(v′²+w′²)] dx`.

La interpolación Hermite cúbica da `Kg_ab=N∫H′_aH′_b dx`, con la matriz
consistente estándar `N/(30L)` (McGuire, Gallagher y Ziemian, *Matrix Structural
Analysis*, 2.ª ed., 2000, cap. 9, análisis de segundo orden). La integración es
algebraica propia, sin copiar listados ni consultar implementaciones externas.
Se resuelve primero gravedad lineal y se congela `N=EA/L·(u_g,b,x−u_g,a,x)`
en todos los miembros, positivo a tracción. Después se resuelve directamente
`[Tᵀ(K+Kg)T]q=TᵀF_EX`, con diafragmas XY exactos y base empotrada.

Cada tramo de columna entre pisos se discretiza con **1, 8 y 16 elementos**;
las vigas conservan un elemento y su Kg. Los nudos interiores de las columnas
no tienen cargas ni restricciones: su condensación por complemento de Schur
es exactamente equivalente al modelo refinado completo. La condensación
elástica reproduce K a 1e-10, por lo que la solución lineal de gravedad y el
axial constante por tramo son invariantes al refinamiento. No se actualiza N
con la respuesta lateral. Los límites de nudos del Engine no se aplican a esta
referencia independiente (492 y 972 nudos para 8 y 16 elementos).

Se guardan por malla derivas lineales/P-Δ, amplificación, ambos θ, axiales,
residuo proyectado y estabilidad positiva. La diferencia relativa **8 vs 16
es ≤2e-6 por piso** en deriva y amplificación (máximo ≈1.20e-6), muy inferior
al criterio 1e-3. El modelo de un elemento difiere de la referencia convergida
como máximo ≈4.75e-4. El test Rust ejecuta EX lineal y E3-pdelta, obtiene su
propio denominador lineal y verifica ambos criterios en cada piso, además de
las huellas del fixture y del script.

La fórmula `1/(1−θ_i)` pasa a **diagnóstico unilateral**:
**`0 ≤ amp·(1−θ_i)−1 ≤ 5e-2`**, con θ de primer orden y amplificación de
deriva de entrepiso. El exceso incluye P-δ que la estimación de cuerda omite;
el test exige también que el Engine nunca quede por debajo de ella.
**La tolerancia 5e-3 frente a Frame3DD se mantiene sin cambios**, así como las
demás tolerancias preexistentes ajenas al criterio reemplazado.

```sh
OPENBLAS_NUM_THREADS=1 PIP_NO_INDEX=1 python3 _ref/gen.py
OPENBLAS_NUM_THREADS=1 PIP_NO_INDEX=1 python3 _ref/e3_pdelta_reference.py
PIP_NO_INDEX=1 python3 _ref/e3_pdelta_reference.py --selftest
```

El generador emite exactamente el fixture y los datos versionados; `index.json`
registra los hashes de ambos archivos de referencia. El comando antiguo
`crates/engine-api/tests/fixtures/pdelta_diaphragm/reference.py` delega al mismo
script. Rust consume solo el JSON, sin requerir Python en CI. Véase también
[provenance MAM-860](../../../../provenance/ENG-VAL-032-MAM-860.md).

## Postproceso común de derivas: val032-modal/1

Engine y oráculos **deben usar el mismo `_ref/postproc_derivas.py`**, identificado
por su sha256 en `index.json`. Cada adaptador exporta sus propias formas,
períodos y participaciones, sin mezclar datos entre fuentes.

Entrada JSON neutral (el docstring contiene el contrato completo):

| Campo | Contenido |
|---|---|
| schema | `val032-modal/1` |
| xi | Amortiguamiento común, 0.05 |
| directions | `["X"]`, `["Y"]` o `["X","Y"]`, excitaciones a procesar |
| floors | Lista ordenada desde piso 1: `{"id":1,"cm":[xc,yc,3]}`, base empotrada z=0 |
| corners_xy | Dos esquinas opuestas y alineadas verticalmente: `[[0,0],[15,10]]` |
| Sa_table | Copia íntegra de `pairs` del fixture o T_s/Sa_m_s2 del CSV, en s y m/s² |
| modes | Por modo: `{"n":1,"T_n":T,"Gamma_n":{"X":gx,"Y":gy},"phi_n":[{"Ux":u,"Uy":v,"Rz":rz}, ...]}` |
| directional_combination | Opcional, `srss` para el run bidireccional |
| directional_factors | Si se solicita SRSS direccional: `[1,1]`, en orden de `directions` |

Cada `phi_n` tiene una terna evaluada en el **CM real** de cada piso, con la **misma normalización y
signo de autovector que Gamma_n** de ese modo. Antes de exportar, el adaptador reconstruye la terna del CM desde el maestro
con Ux_cm=Ux_m−(y_cm−y_m)Rz_m, Uy_cm=Uy_m+(x_cm−x_m)Rz_m y Rz_cm=Rz_m.
Se exigen T positivos, tabla creciente, valores finitos, y no se extrapola. No se requiere normalización
específica: el producto phi·Gamma debe ser coherente.

Se interpola Sa(T_n) linealmente de la misma tabla y se calcula, con
ω_n = 2π/T_n, Δ_i,n = [φ_n(i)−φ_n(i−1)]·Γ_n·Sa(T_n)/ω_n².
Se conserva el signo antes de combinar. Para las esquinas se evalúa primero
u_esq = u_m − (y−yc)Rz, v_esq = v_m + (x−xc)Rz, en ambos extremos de la misma
vertical (CM del piso inferior cuando corresponde). La base tiene φ = 0.
Las esquinas opuestas son extremas en x e y para este rectángulo.

SRSS = sqrt(ΣΔ_n²); CQC = sqrt(Σ_nΣ_m ρ_nm Δ_n Δ_m), con Der Kiureghian y ξ igual:
ρ_nm = 8ξ²(1+β)β^(3/2)/[(1−β²)² + 4ξ²β(1+β)²], β = ω_m/ω_n.
Para frecuencias coincidentes ρ = 1; si ξ = 0 y son distintas, ρ = 0.
**No se restan desplazamientos ya combinados**. Para XY se combina después por
SRSS entre excitaciones, con factores [1,1].

Salida `val032-drifts/1`: `directions.X/Y.floors[]`, con `CM`, dos `corners` y
`Ax`. Cada punto tiene respuestas Ux/Uy con `CQC_m`, `SRSS_m` y sus cocientes
respecto a altura (`CQC_ratio`, `SRSS_ratio`), además de contribuciones modales
firmadas `modal_drifts_m`. Se reportan ambas componentes por excitación.
`directional_srss.floors[]` contiene la combinación XY cuando se solicita.
`displacements.CM/corners` entrega también los desplazamientos de nivel.
Ax por componente y método = (δmax/(1.2 δprom))², donde δmax es el mayor de los
dos picos de **desplazamiento de nivel** combinados de esquina y δprom su promedio.
`Ax_from_drifts` identifica el cociente análogo de derivas como dato auxiliar. Para revisar NEC §6.3.7
se usa Ux bajo X o Uy bajo Y. No se imponen límites adicionales al Ax crudo;
si ambas esquinas dan cero se escribe `null` (0/0 indefinido).

```sh
python3 _ref/postproc_derivas.py modal-neutral.json -o derivas.json
python3 _ref/postproc_derivas.py --selftest
```

El autotest usa 2 modos sintéticos calculados a mano: ω = (1,2), ξ = 1/20,
Sa = (8,4), Γ_X = (1,2), ρ12 = 2√2/153. En CM las derivas X del piso 1 son
(8,4) y las del piso 2 (16,2); se contrastan CQC, SRSS, esquinas y Ax. Incluye
signos opuestos, SRSS direccional, interpolación, cambio de signo del autovector,
CM distintos por piso, modos coincidentes y rechazo de extrapolación.

## Tolerancias y Congelación

En cada fixture, `tolerances_preregistered` declara métrica, comparación,
tolerancia, tipo, justificación y estado. Los criterios frente a oráculos son:

- Períodos disponibles (todos los modos calculados): **1e-5 relativo**.
- Desplazamientos lineales y espectrales: **1e-4 relativo**, todos los modelos;
  derivas espectrales CM/esquinas y Ax: **1e-4 relativo**, postproceso común.
- E3: desplazamientos y derivas P-Δ según la formulación del oráculo:

  | Comparación | Tolerancia relativa | Uso |
  |---|---:|---|
  | Engine vs oráculo con Kg consistente (Frame3DD geom. no lineal) | 5e-3 | Criterio condicionado a la configuración obligatoria y precisión de salida indicadas en «Estabilidad de E3» |
  | Engine vs oráculo con transformación geométrica P-Δ (solo cuerda) | 5e-2 | Diagnóstico, no criterio; la diferencia de formulación se documenta solo en el repo de oráculos |

  Amplificación y deriva P-Δ frente a referencia propia convergida de 8 y 16
  elementos por columna: **1e-3 relativo por piso**; implementación independiente
  de un elemento: **1e-9 relativo por piso**. Diagnóstico unilateral de cuerda:
  **0 ≤ amp·(1−θ)−1 ≤ 5e-2**. θ de primer orden frente a oráculos: **1e-4 relativo**.
- Sa interpolada frente a analítica: **5e-4 relativo**.
- E4: fracción cruzada **1e-8 absoluto**; masa efectiva acumulada de modos
  **1–6 en X e Y, 1e-6 absoluto** (fracción de masa libre de su dirección).
  Se reemplaza la suma trivial de los 15 modos.
- Reacciones E5/E5b: **1e-6 relativo**; fuerzas/momentos de barra de E5b:
  **|Δ| ≤ 1e-6·|ref| o |Δ| ≤ 1e-6·max|componente del mismo tipo en el caso|**
  (piso absoluto), mismos ejes locales y estaciones. El máximo se toma en la
  referencia por separado para N, Vy, Vz, T, My y Mz, sin mezclar casos; cada
  combinación y envolvente usa su propio conjunto de referencia. En truss solo
  se compara N. Se requieren **≥10 cifras significativas** en la salida del
  oráculo para que el run sea comparable como criterio. Frame3DD imprime en coma
  fija, no en notación científica: se deben ajustar las unidades (p.ej. mm, kN),
  y Q2 debe comprobar las ≥10 cifras en una corrida en seco de formato antes de
  congelar. Si no se alcanza, E5b frente a Frame3DD y la parte afectada de E3 son
  solo diagnóstico. Se debe declarar si usa barra con I pequeña para representar
  truss. En E5 **todas** las métricas de fuerzas/momentos de
  barra son **«diagnóstico, no criterio (#84)»**, con tolerancia `null`, incluidas
  combinaciones y envolventes.
- Equilibrio global: **1e-9 relativo** (fuerzas y momentos). Superposición
  algebraica interna: **1e-12 relativo**; en E5 solo desplazamientos/reacciones.
- Equivalencia de formato con el mismo Engine: **1e-12 relativo**.

### Congelación

**Las tolerancias quedan fijadas por el commit que las contiene; Q2 debe citar
ese SHA en cada run**, junto con el hash del fixture y del postproceso. No se
crea commit ni se hace push en esta tarea; por tanto no se declara aquí un SHA
de congelación. Antes de congelar, Q2 debe verificar la configuración obligatoria
de Frame3DD en E3 y comprobar ≥10 cifras significativas en la corrida en seco
de formato para E5b y la parte afectada de E3. Las comparaciones con oráculos y la equivalencia de formato
quedan pendientes de corrida; no se relajan tolerancias a partir de sus resultados.
La **masa efectiva acumulada por modo en X/Y está pendiente de corrida**:
Q2 debe reportar cada acumulado y en particular los modos 1–6 de E4.

## NEC contrastado por J4

Según la revisión de solo lectura `/workspace/j4rev/comments/val032-r1.md`
(2026-10-20), contra NEC-SE-DS 2015, páginas impresas:

| Tema | Referencia/páginas de la revisión |
|---|---|
| Z = 0.40, zona V | Tabla 1, §3.1.1, pág. 27 |
| Fa = 1.20, Fd = 1.19, Fs = 1.28, suelo D | Tablas 3–5, págs. 31–32 |
| η de Sierra, r, espectro y Tc | §3.3.1, págs. 33–34; J4 H3 pide la rama T<T0 incorporada en r2 |
| W = D | §6.1.7, pág. 55 |
| Participación modal >90 % | §6.2.2 e, pág. 58; masa acumulada aquí pendiente de corrida |
| V, Ta = Ct·hn^α | §6.3.2–6.3.3, págs. 61–62 |
| R = 8, pórtico especial HA | Tabla 15, §6.3.4, pág. 65 |
| Distribución ELF, k | §6.3.5, pág. 66 |
| Torsión accidental y Ax | §6.3.6–6.3.7, págs. 66–67 |
| θ, carga sin mayorar y límite 0.30 | §6.3.8, págs. 67–68 |

I = 1.0 contra Tabla 6 pág. 39 y las combinaciones NEC-SE-CG §3.4.3
(numeración y factores) siguen **NO VERIFICADOS**. No se releyó aquí la norma.

## Brechas

**Sin bloqueos del Engine en los doce runs.** #188 (#184) habilita la
condensación estática de Guyan para los GDL libres sin masa en modal/espectro,
con recuperación de los modos completos y cinemática de esclavos. Los ocho
runs antes afectados por ENG-E-0006 entregan quince períodos; el test los
contrasta con la referencia propia y las tolerancias originales. #185/#186
resuelve ENG-E-0007 en E3-pdelta. No se agregan masas ni apoyos de piso.
Los avisos #84/#85 se mantienen con las limitaciones documentadas arriba.

| Modelo | bench-model/1 | Ejecución del Engine |
|---|---|---|
| E1 | Emitido, masas nodales explícitas | Modal ejecutable con warning #84; Guyan #188 |
| E2 | Emitido, tres variantes espectrales | Los tres runs ejecutables con warning #84; Guyan #188 |
| E2b | Emitido, tres variantes y SRSS direccional [1,1] | Los tres runs ejecutables con warning #84; Guyan #188 |
| E3 | Emitido, G/EX y seis opciones P-Δ literales | Lineal y pDelta ejecutables con warning #84; P-Δ corregido por #185 |
| E4 | Emitido, modal.n=15 | Modal ejecutable con warning #84; Guyan #188 |
| E5 | Emitido, cuatro casos y diez combinaciones | Warning #84 y diez avisos #85 |
| E5b | Emitido, masas/diafragmas vacíos y 60 truss E/A | Warning: diez avisos #85 |

El borrador #183 representa los siete modelos sin cambiar física; no se usa su
rama de masas implícitas, que añadiría Uz/Rx/Ry y cambiaría el problema modal.
`stories[].mass` informa m e Iz **en CM** por piso; `nodal_masses` determina la
entrada del Engine. E5b tiene `stories[].mass={m:0,Izz:0}` y no solicita modal.
Los campos modal/espectro obligatorios de los tres modelos estáticos son
metadatos comunes inactivos y no crean runs adicionales. R sigue siendo metadato:
la tabla elástica se conserva sin dividir por R ni volver a multiplicar por g.

### E3-pdelta ejecutable tras #185

La corrección de `crates/engine-analysis/src/pdelta.rs` incorpora las
contribuciones de los esclavos en la rigidez y las fuerzas geométricas y en
el residual mediante la transformación del diafragma. No cambia física,
cargas ni tolerancias. `E3-pdelta` está enumerado entre los cuatro runs
ejecutables: el test rechaza errores, estados distintos de OK/Warning y
avisos sin justificar. La prueba específica exige además convergencia y
coincidencia de θ y deriva con la referencia propia independiente.

Los bloqueos ENG-E-0007/notConverged y ENG-E-0006/unstable se conservan
solo en las evidencias históricas r3. La ronda 2 de MAM-860 actualiza `info`
y el índice al estado ejecutable con warning #84; las brechas resueltas se
retiran del índice y del traductor. La evidencia P-Delta y su reproducción
se documentan en [la referencia P-Δ](../../../../crates/engine-api/tests/fixtures/pdelta_diaphragm/README.md)
y [la bitácora de #185](../../../../provenance/ENG-ANL-pdelta-diafragma.md).

### Biyecciones y prueba de traducción

`make_bench` construye una biyección explícita por namespace, registrada completa
en `index.json`. Nodos y barras siguen su orden original → ids contiguos desde 1.
Los maestros son nodos de pórtico. Extremos, diafragmas, apoyos, masas y cargas
usan la misma biyección de nodos. Secciones, casos y combinaciones conservan
identidad. Las leyes `hormigon` y `losa_elastica` se renombran respectivamente a
`elastic-columna` y `elastic-losa_diagonal`; E/G/rho permanecen idénticos.
Apoyos → `support-<nodo bench>`; cada masa nodal → `mass-<nodo bench>`.
`grid` refleja los ejes físicos; orientaciones `local_z_vector` se copian.

El traductor Python reconstruye los **doce runs emitidos**, sin diferencias
numéricas en geometría, secciones, materiales, apoyos, masas, cargas, espectro,
combinaciones y opciones después de aplicar esas biyecciones. El validador
comprueba los siete documentos contra el esquema #183, los doce originales y
traducidos contra el esquema Engine, hashes y brechas declaradas. Esta prueba
es de entradas; los estados reales se verifican separadamente en Rust.

## Verificación reproducible

Desde este directorio, Python autorizado `/workspace/.venv-v32/bin/python`:

```sh
/workspace/.venv-v32/bin/python _ref/gen.py
OPENBLAS_NUM_THREADS=1 /workspace/.venv-v32/bin/python _ref/val032_calibrate.py --r3
/workspace/.venv-v32/bin/python _ref/gen.py
/workspace/.venv-v32/bin/python _ref/validate_bench.py --schema /tmp/pr183n/bench-model-1.json
OPENBLAS_NUM_THREADS=1 /workspace/.venv-v32/bin/python _ref/val032_calibrate.py E3-v1.1.3.json
```

`gen.py` usa stdlib y NumPy para la referencia E3, y conserva las evidencias r3; regenerar la comparación
r3 requiere numpy y las doce copias anteriores `/tmp/v32run-<run>.json`.
La referencia modal condensa los GDL sin masa mediante
K_aa−K_ab·K_bb⁻¹·K_ba y resuelve el problema generalizado con Cholesky de M_aa;
no altera los fixtures. E5b, sin masas, no tiene períodos físicos que comparar.
El generador no depende de esas copias temporales. `validate_bench.py` usa
jsonschema y referencing, rechaza referencias externas y comprueba evidencia,
hashes y las doce traducciones. No ejecuta el Engine.

Desde la raíz del repo, el procedimiento histórico de registro r3 es el siguiente
(no ejecutarlo en esta ronda: reescribe evidencias y hashes congelados).
Sustituir `<dir>` por los directorios de compilación y resultados elegidos;
usar el mismo directorio de resultados en el test y en `record_engine.py`:

```sh
CARGO_TARGET_DIR=<dir> VAL032_RESULTS_DIR=<dir> cargo test --offline -p engine-api --test val032_runs
/workspace/.venv-v32/bin/python docs/validation-report/fixtures/val032/_ref/record_engine.py --results <dir>
/workspace/.venv-v32/bin/python docs/validation-report/fixtures/val032/_ref/gen.py
```

Para verificar el estado actual sin regenerar los JSON ni hashes, basta ejecutar
`cargo test --offline -p engine-api --test val032_runs --test pdelta_diaphragm`.
La exportación opcional `VAL032_RESULTS_DIR=<dir>` permite inspeccionar los
doce documentos actuales sin sobrescribir el registro histórico.

El test recorre `E*.json` relativos a `CARGO_MANIFEST_DIR`, exige los doce runs
únicos, ningún Invalid, `warning` sin errores en los doce runs y estados
documentados coincidentes con los reales, incluido E3-pdelta (#185/#186). Los avisos permitidos están enumerados por run/código y los #85
además por las diez combinaciones; verifica que todos los prescritos sean cero.
La exportación opcional conserva results/1 e input real por run para revisarlos.

Determinismo: ejecutar dos generaciones y comparar byte a byte todos los
fixtures, siete bench-model, CSV, índice y evidencias:

Comprobado en el cierre histórico r3: **18 archivos idénticos en dos generaciones**;
`validate_bench.py --schema /tmp/pr183n/bench-model-1.json` dio OK para los
siete modelos y las doce traducciones. Se comprobaron también los hashes de
fixtures, herramientas, evidencias y fuentes Rust registrados en `index.json`.

```sh
/workspace/.venv-v32/bin/python - <<'PYTHON'
import subprocess
from pathlib import Path
root = Path('.')
def snapshot():
    files = [*root.glob('E*-v*.json'), *root.glob('bench-model/*.json'),
             root/'index.json', root/'nec_se_ds_tabla_Sa.csv',
             root/'_ref/calibrate_r3.json', root/'_ref/engine_r3.json']
    return {str(p): p.read_bytes() for p in files}
subprocess.run(['/workspace/.venv-v32/bin/python', '_ref/gen.py'], check=True, stdout=subprocess.PIPE)
first = snapshot()
subprocess.run(['/workspace/.venv-v32/bin/python', '_ref/gen.py'], check=True, stdout=subprocess.PIPE)
assert snapshot() == first
print('Dos generaciones idénticas byte a byte')
PYTHON
```
