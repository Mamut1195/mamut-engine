# Ficha de método · frame2d.fiber_stations · issue 6

Completa antes de la implementación. Los axiles, cortes y momentos
esperados de abajo se evaluaron con las ecuaciones de esta ficha y con
el equilibrio ya escrito de `rectangle.fiber_bending`, antes de programar
la barra. Esta sesión es limpia, distinta de las que implementaron los
tickets 01 y 02, y distinta de la del 2026-11-04. No vio salidas numéricas
de un programa estructural externo.

- Autor, modelo si es IA, fecha y rol: agente cloud de Cursor (Cursor Agent),
  modelo: DESCONOCIDO, 2026-11-09 (America/Guayaquil). Rol: especificador e
  implementador. La corrección del signo de κz es de la misma sesión, el
  2026-11-09 (ECT).
- Kind propio, contrato y versión del método: `frame2d.fiber_stations`,
  `element/1` (`ContractId::new("element", 1, 0)`), método `0.1.0`.
- Módulo, crate y consumidor previsto:
  `crates/engine-elements-frame/src/fiber_stations.rs`, crate
  `engine-elements-frame`. El consumidor ya tiene un
  `RectangleFiberBending` y llama a `Element::respond`. El kind no entra
  en la API de entrada del motor. No hay fábrica JSON en 0.1.0: la sección
  de fibras tampoco es un kind de entrada.
- Métodos ya revisados y por qué este es distinto: `frame2d.euler_elastic`
  es la barra de Euler homogénea y lineal, con rigidez EA y EI constantes.
  `frame2d.winkler` añade un lecho lineal. `rectangle.fiber_bending` es
  una sección, no una barra. Esta barra no sustituye a
  `frame2d.euler_elastic`. Muestrea esa sección en varias estaciones y
  suma el trabajo virtual. No añade otra ley de acero ni otra sección.

## Fuentes permitidas

Cinemática y signos ya escritos en este repositorio:

- `docs/AXES_CONVENTION.md`, aprobado el 2026-10-10. En D2, x local va de
  i a j, y = (−sin, cos) y z apunta a +Z. N positivo es tracción. En la
  cara positiva, Mz positivo sigue la mano derecha alrededor de +z.
- `crates/engine-laws/src/section/mod.rs` (`ElasticFrame`):
  ε(y) = ε0 − κz y, s = [N, Mz], con curvatura positiva que produce
  momento positivo. Una página de libro para esa frase no está en esta
  sesión: cita desconocida.
- `crates/engine-elements-frame/src/stiffness.rs`: la matriz elástica de
  la barra de Euler con orden `[u, v, θ]` por nudo. Se usa solo como
  comprobación de que el operador de curvatura de esta ficha reproduce
  4EI/L, 2EI/L y 6EI/L² cuando Mz = EI κz. No se copia un listado.
- `crates/engine-elements-frame/src/recover.rs`: en la cara positiva, sin
  carga repartida, el corte de esa barra es −f_v,i. El comentario de
  equilibrio de esa recuperación es N′ = −qx, V′ = −qy, M′ = −V − mz,
  con M y V de esa recuperación elástica.
- `docs/fichas/rectangle.fiber_bending.md`, ecuaciones (1)–(9). El punto
  de control de la sección es ε0 = −1/1024, κz = 1/256 m⁻¹, con N y Mz
  de su ecuación (8). El hormigón es la parábola (A.9) ya transcrita en
  `docs/fichas/concrete.kent_park.md`. Esta sesión no abrió el informe
  BARC/2009/E/026. Página impresa: cita desconocida.

La cuadratura, el número de estaciones, los pesos y las ecuaciones
(1)–(16) de esta ficha son una derivación propia del 2026-11-09. No se
copió una tabla, una figura ni un listado. Las estaciones y los pesos no
se mueven para igualar un programa externo.

## Hipótesis, unidades y dominio

- Física y límites: barra de Euler–Bernoulli en D2, dos nudos, desplazamientos
  pequeños, sin deformación por cortante y sin inercia de rotación de la
  sección. La respuesta de sección es un `rectangle.fiber_bending` ya
  construido. No hay masa: la sección no trae densidad, y `mass` devuelve
  `None`. No hay rigidez geométrica en 0.1.0: `geometric_stiffness`
  devuelve `None`. Una carga de vano distinta de cero se rechaza con
  `ENG-E-0013`. La carga nula devuelve el vector nulo. D3 y Z distinto de
  cero se rechazan como en la barra de Euler. Esta barra no resuelve el
  pórtico: entrega fuerza interna y tangente para quien ya ensambla.
- Entradas totales, unidades SI y signos: u_e global, por nudo
  `[ux, uy, rz]`, en m y rad. En ejes locales,
  q = [u_i, v_i, θ_i, u_j, v_j, θ_j]. θ positivo es el giro alrededor de
  +z y, en la teoría lineal, coincide con dv/dx. N en N, tracción positiva.
  Mz en N·m, el de la sección. El corte Vy de la cara positiva está en N.
- Parámetros, unidades, rangos y validación: el único dato constitutivo es
  la sección ya validada. La luz sale de las coordenadas. L ≤ 1×10⁻¹² m es
  `ENG-E-0005`. Un desplazamiento, una historia o un producto no finito es
  `ENG-E-0008`. Una historia de otra longitud, una estación fuera de
  [0, 1] o un contexto que no sea D2 con dos nudos es `ENG-E-0010`.
- Estado: la sección guarda 2 números por barra. La barra guarda esa
  historia en cada estación de integración, en el orden izquierda, centro,
  derecha. La longitud es 3 por la longitud de la sección. El estado
  inicial son ceros. `respond` no muta la barra ni la historia aceptada.
- Salidas y unidades: fuerza interna global en N y N·m, tangente simétrica
  en las unidades de la barra de Euler, e historia de prueba. La
  recuperación devuelve las acciones de extremo locales, iguales a la
  fuerza interna local cuando no hay carga de vano, y en cada estación
  pedida `[N, Vy, 0, 0, 0, Mz]`.
- Dominio numérico y orden de las operaciones: el de las ecuaciones
  (4)–(7) y (13)–(16). Cada producto o suma que puede desbordar pasa por
  la comprobación finita del crate. El subdesbordamiento ordinario de f64
  se acepta.
- Puntos no diferenciables: los de cada fibra, heredados de la sección.
  En el punto de control todas las fibras de hormigón están en compresión,
  por debajo del pico, y las dos barras siguen elásticas. Una diferencia
  central con h = 1×10⁻⁶ no cruza esos quiebros. El origen ε = 0 es un
  quiebro del hormigón (tracción con tangente nula, compresión con la
  tangente inicial). No se usa como muestra de diferencia central.

## Derivación propia

1. Estaciones. Sobre el intervalo de referencia t ∈ [−1, 1] se piden tres
   nodos y tres pesos que integren exactamente los polinomios de grado
   ≤ 5. El polinomio de grado 3 ortogonal a 1, t y t² con peso 1 es

   (1) P3(t) = (5 t³ − 3 t) / 2.

   Sus raíces son los nodos, ecuación (2):

   t (5 t² − 3) / 2 = 0 ⇒ t ∈ {−√(3/5), 0, √(3/5)}.

   Por simetría los pesos de los extremos son iguales. Las integrales
   ∫ t^k dt de −1 a 1 valen 2/(k+1) si k es par y 0 si k es impar. Con
   a² = 3/5, los momentos k = 0 y k = 2 dan, ecuación (3):

   2 w_a + w_0 = 2,
   2 w_a (3/5) = 2/3 ⇒ w_a = 5/9, w_0 = 8/9.

   El momento k = 4 cierra: 2 (5/9) (3/5)² = 2/5, que es ∫ t⁴ dt. La
   regla clásica de tres puntos recibe aquí el nombre de cuadratura de
   este grado; la página de un libro no está en la sesión: cita
   desconocida. No se ajustó ningún nodo después de ver un resultado.

2. Paso a la luz. ξ = x/L = (t + 1)/2 y dx = (L/2) dt. Las tres estaciones
   y los pesos sobre la longitud son, ecuación (4):

   ξ = (1 − √(3/5))/2, 1/2, (1 + √(3/5))/2,
   W = (L/2) (5/9), (L/2) (8/9), (L/2) (5/9).

   En aritmética finita el peso central se calcula como 2 − 5/9 − 5/9,
   para que los tres pesos de referencia sumen 2 y la integral de una
   constante sea L. Eso es la primera línea de (3), no un ajuste.

3. Deformación de la directriz, ecuación (5). La axial es lineal y la
   transversal es el cubico de Hermite con θ = dv/dx:

   ε0 = (u_j − u_i) / L,
   H1 = 1 − 3 ξ² + 2 ξ³, H2 = ξ − 2 ξ² + ξ³,
   H3 = 3 ξ² − 2 ξ³, H4 = −ξ² + ξ³,
   v(ξ) = H1 v_i + H2 L θ_i + H3 v_j + H4 L θ_j.

   La sección usa ε(y) = ε0 − κz y y un κz positivo produce Mz positivo.
   Un giro θ positivo alrededor de +z sigue la mano derecha: el punto de
   la sección situado en y se desplaza hacia −x en y·θ. El desplazamiento
   axial de esa fibra es u − y θ. Con θ = dv/dx,

   ε(y) = du/dx − y dθ/dx = ε0 − y v″.

   Igualando con ε(y) = ε0 − κz y queda, ecuación (6):

   κz = v″.

   Un v″ positivo acorta la fibra y > 0: el centro de curvatura queda del
   lado −y y esa fibra está más cerca del centro. La frase anterior de
   esta ficha, que ponía κz = −v″, contradecía a `ElasticFrame`.

   El operador que sale de (5) y (6) es, ecuación (7):

   B_ε = [−1/L, 0, 0, 1/L, 0, 0],
   B_κ,v_i = (−6 + 12 ξ) / L², B_κ,θ_i = (−4 + 6 ξ) / L,
   B_κ,v_j = (6 − 12 ξ) / L², B_κ,θ_j = (−2 + 6 ξ) / L.

   Cada componente de B_κ es la derivada de v″. El producto B_κ,a B_κ,c no
   cambia si se invierten todos los signos, así que ∫ B^T diag(EA, EI) B dx
   sigue reproduciendo la matriz de `stiffness.rs`: 4EI/L en θ_i, 2EI/L
   entre θ_i y θ_j, y 6EI/L² entre v_i y θ_i. La integral se hizo a mano.

4. Trabajo virtual. En cada estación la sección devuelve s = [N, Mz] y la
   tangente k_s de 2×2. La fuerza interna local, en el mismo convenio que
   K u de la barra elástica, y la tangente son, ecuación (8):

   f = Σ W_i B(ξ_i)^T s_i,
   K = Σ W_i B(ξ_i)^T k_s,i B(ξ_i).

   El orden es el de las estaciones de izquierda a derecha. Dentro de cada
   estación se evalúa primero ε0, después κz, después la sección, después
   la fuerza y al final la tangente. La tangente material es simétrica
   cuando k_s lo es: se calcula el triángulo superior y se copia al
   inferior. La historia de prueba se concatena en el mismo orden de
   estaciones.

5. Punto de control. La sección de control es la de las ecuaciones (1),
   (2), (4) y (8) de `docs/fichas/rectangle.fiber_bending.md`: b = 1/4 m,
   h = 1/2 m, cuatro franjas, dos barras de área 1/1024 m² en
   y = ±3/16 m, hormigón de 3000 psi con ρ_s = 0, acero
   E = 200000000000 Pa, fy = 500000000 Pa, b = 0, historia inicial nula.
   El mismo desplazamiento de control de la primera redacción se conserva.
   Con (6) corregida ese campo ya no produce κz = +1/256. Ecuación (9):

   ε0 = (u_j − u_i) / L = −1/1024,
   κz = v″ = −1/256 m⁻¹.

   Con v_i = v_j = 0, (7) es constante si θ_j = −θ_i y
   κz = −2 θ_i / L. Se elige L = 4 m para que los giros y el acortamiento
   sean racionales diádicos, ecuación (10):

   u_i = 0, v_i = 0, θ_i = 1/128 rad,
   u_j = −1/256 m, v_j = 0, θ_j = −1/128 rad.

   La barra va de (0, 0, 0) a (4, 0, 0). Los dos extremos no se desplazan
   en la dirección transversal: es el campo de una barra biapoyada a la
   que se le imponen los giros de extremo y el acortamiento, sin carga
   transversal. No es un voladizo con fuerza en la punta.

6. Equilibrio en la estación de control. Como ε0 y κz son constantes, cada
   estación, y también cualquier x/L de recuperación, ve ε0 = −1/1024 y
   κz = −1/256. La sección de la ficha 02 es simétrica, así que
   N(ε0, −κ) = N(ε0, κ) y Mz(ε0, −κ) = −Mz(ε0, κ). N es el de la ecuación
   (8) de esa ficha y Mz es su opuesto. El momento no cambia con x. Sin
   carga repartida, el corte de ese estado es nulo, ecuación (11):

   dMz/dx = 0 ⇒ Vy = 0.

   La estación de control es la central, ξ = 1/2, que además es una
   estación de integración. Ahí, ecuación (12):

   N = −f′_c · 695875 / 8388608 − E / 524288,
   Vy = 0,
   Mz = −(f′_c · 81875 / 16777216 + 9 E / 33554432).

   f′_c es 3000 psi pasado al pascal con el factor de unidad ya usado por
   `concrete.kent_park`, 4.4482216152605 / 0.0254². E es el módulo del
   acero. El paréntesis es el momento de la ecuación (8) de la ficha de
   la sección, evaluado en κz = +1/256. El signo menos es el de κz = −1/256.
   Estos valores no salen de otro programa.

7. Fuerza nodal del mismo estado. Con N y Mz constantes, (8) integra
   polinomios de grado ≤ 1. La regla (3) los integra exactamente porque
   los pesos suman 2 y la regla es simétrica. ∫ B_κ,θ_i dx = −1 y
   ∫ B_κ,θ_j dx = +1, así que, ecuación (13):

   f = [−N, 0, −Mz, N, 0, Mz].

   Con el Mz negativo de (12), el momento de extremo en i es positivo.
   En x = 0 y con corte nulo, el momento de cara positiva de la barra
   elástica cumple M = −f_θ,i. Esa identidad coincide con el Mz de la
   sección. En esta barra horizontal la fuerza global coincide con la
   local. El corte de la cara positiva se informa como −f_v,i. En este
   estado ese corte es 0, de acuerdo con (11).

8. Tangente en el mismo estado. k_s es la ecuación (9) de la ficha de la
   sección y es constante a lo largo de la luz. (8) la integra con la
   misma regla. La diferencia central de la fuerza, con la historia
   aceptada fija, usa h = 1×10⁻⁶. Ese h está dentro de los límites
   1×10⁻⁹ y 1×10⁻² del kit. Un paso h en un grado de libertad cambia la
   deformación de la fibra extrema en menos de 0.5 h, cerca de
   5×10⁻⁷. La fibra menos comprimida está a 1/4096 del quiebro en
   deformación nula, y la más comprimida está a unos 2.9×10⁻⁴ del pico
   0.002. El acero de control está a unos 7.9×10⁻⁴ de la fluencia. La
   muestra ±h no cambia de rama. Con κz = −1/256 las deformaciones de las
   fibras son las de la ficha 02 intercambiadas de arriba abajo, así que
   las distancias a los quiebros no cambian.

9. Armadura asimétrica, mismo desplazamiento (10), régimen de la parábola
   y del acero elástico. La barra inferior sigue en 1/1024 m². La superior
   pasa a 4/1024 m², en y = +3/16 m. Las deformaciones de las franjas, de
   abajo arriba, son −7/4096, −5/4096, −3/4096 y −1/4096. Todas caen en la
   parábola. El acero mayor es 7E/4096, por debajo de fy. El hormigón
   aporta el mismo axil que en (12) y el momento opuesto al de la sección
   simétrica en κz = +1/256, porque las razones de la parábola se invierten
   con y. El acero suma, ecuación (14):

   N = −f′_c · 695875 / 8388608 − 11 E / 4194304,
   Mz = −f′_c · 81875 / 16777216 − 9 E / 67108864.

   La fuerza nodal es otra vez (13) con este par (N, Mz). `ElasticFrame`
   con la misma (ε0, κz) devuelve N = EA ε0 y Mz = EI κz. No es el mismo
   número que (14): la parábola no es lineal. El convenio sí es el mismo:
   κz = v″ y, con corte nulo, el Mz de estación es −f_θ,i. Una barra de
   Euler con los mismos E, A e I cumple esa identidad con Mz = EI v″.

10. Curvatura lineal, sección simétrica de control. Ecuación (15):

   u_i = v_i = v_j = θ_j = 0, θ_i = 1/256 rad, u_j = −1/256 m, L = 4 m.

   ε0 sigue en −1/1024. κz(ξ) = (−4 + 6ξ) θ_i / L = (3ξ − 2) / 512, desde
   −1/256 hasta 1/512. En todo el vano cada fibra de hormigón sigue en la
   parábola y el acero sigue elástico, así que N y Mz son cuadráticos en
   ξ y el integrando de (8) es cúbico. La regla (3) integra ese polinomio
   de forma exacta. Una regla de Boole con nodos 0, 1/4, 1/2, 3/4 y 1 y
   pesos 7, 32, 12, 32, 7, también exacta hasta grado 5, da el mismo
   integral y no usa los nodos ni los pesos de (2)–(4). Esa integral es
   el esperado. Si se cambia 5/9, 8/9 o ±√(3/5), el elemento deja de
   coincidir con ella.

11. Historia distinta en una sola estación. En el campo (15), la barra
   superior de la estación izquierda guarda ε_p = 1/10000 y el resto de
   la historia sigue en cero. σ = E(ε − ε_p) mientras |σ| < fy, que aquí
   se cumple. El esperado usa las estaciones en el orden izquierda,
   centro, derecha. Si ese orden se permuta, la misma historia cae en
   otra abscisa y la fuerza nodal cambia.

12. Recuperación. Una estación pedida reutiliza la historia aceptada de
   la estación de integración más cercana en ξ. Si hay empate, gana el
   índice menor. El Mz informado es el de la sección en esa abscisa. El
   corte informado es el constante −f_v,i. Cuando Mz varía a lo largo de
   la luz, dMz/dx y ese corte no coinciden punto a punto: el equilibrio
   que la barra impone es el trabajo virtual (8), no el equilibrio fuerte
   en cada x. Con Mz constante, como en (11), sí coinciden. El número de
   estaciones pedidas va de 0 a 64. Pasar de 64 es `ENG-E-0010`. Ese tope
   es el dominio del método 0.1.0, no un presupuesto de tiempo o memoria.

## Seguimiento MAM-1037

Autor de este apartado: agente cloud de Cursor (Cursor Agent), modelo:
DESCONOCIDO, 2026-11-09 (America/Guayaquil, ECT, UTC−5). Rol: especificador
de los esperados. La barra y la sección no cambian de código. Esta sesión
no vio salidas numéricas de un programa estructural externo. Las
reducciones de abajo se hicieron antes de ejecutar el ensayo.

13. Voladizo axial con `StaticNewton`. La barra mide L = 4 m, de
    (0, 0, 0) a (4, 0, 0). El nudo i está empotrado. El nudo j recibe
    solo tracción. La sección es la de control, con una salvedad: el
    acero usa b = 1/20, no b = 0. Hay dos barras de 1/1024 m², así que
    A = 1/512 m². El hormigón en tracción no aporta tensión ni tangente.
    La sección es simétrica en y y la carga no tiene parte transversal,
    así que κz = 0, v_j = 0 y θ_j = 0. El axil de fluencia de las dos
    barras es N★ = fy A. La carga final es P = (3/2) N★ y la tensión
    de cada barra es σ = (3/2) fy.

    Con historia nula y tracción monótona, la rama ya escrita de
    `steel.bilinear_kinematic` da

    σ = fy + b E (ε − fy/E),
    ε_p = (1 − b) (ε − fy/E),
    α = b E (ε − fy/E).

    De σ = (3/2) fy sale ε − fy/E = fy / (2 b E) y α = fy/2. Con
    b = 1/20 y fy/E = 1/400,

    ε = 11/400, Ux_j = ε L = 11/100 m, ε_p = 19/800.

    En binary64 la literal 0.05 es el mismo número que 1/20, y 11/400,
    11/100 y 19/800 coinciden con esa evaluación. ε0 es constante, así
    que las tres estaciones guardan el mismo par [ε_p, α] en las dos
    barras. La rama plástica es lineal de pendiente b E: un paso de
    Newton sobre esa rama cae en este estado. La holgura absoluta de
    Ux es 3×10⁻⁹ m, el techo del residuo 1×10⁻⁸ P partido por la
    rigidez axial b E A / L. La holgura de ε_p es 1×10⁻⁹, por encima de
    (1 − b) veces ese techo partido por L. La de α es 10 Pa, por encima
    de b E veces el techo de deformación. v_j y θ_j se comparan con
    1×10⁻⁹: el campo impar es nulo por simetría, y esa cota queda muy
    por debajo del desplazamiento transversal que autorizaría el mismo
    residuo si la carga no fuera axial.

14. Mismo voladizo, con momento de punta y sin axil ni corte. El
    momento en j es M = 1000 N·m, positivo alrededor de +z. Los
    factores absolutos crecen: 1/4, 1/2, 3/4 y 1. En esta barra ε0 es
    constante y κz es lineal en ξ. El campo de curvatura constante κ,
    con v_i = θ_i = 0, es θ_j = κ L y v_j = κ L² / 2, con
    Ux_j = ε0 L. Si N y Mz son constantes, (13) da la fuerza nodal
    [−N, 0, −Mz, N, 0, Mz]. El nudo libre exige N = 0 y Mz = M, y el
    equilibrio de sólido rígido pone en i la fuerza [0, 0, −M]. El
    corte recuperado es −f_v,i = 0. N(ε0, κ) = 0 y Mz(ε0, κ) = M fijan
    el eje neutro. El apartado 16 resuelve ese par con las fibras y
    deja Ux_j = ε0 L y θ_j = κ L.

    Signo. Con κ > 0 y ε0 = 0 el hormigón de y > 0 está comprimido y
    aporta N < 0, mientras el acero simétrico aporta
    N = 2 E A_s ε0 = 0. Bajar ε0 aumenta la compresión. La raíz N = 0
    está en ε0 > 0, así que Ux_j > 0. En la parábola ascendente, y con
    el acero elástico, M > 0 exige κ > 0, y de ahí v_j > 0 y θ_j > 0.

    Simetría. Las tres estaciones comparten (ε0, κ), así que repiten
    N = 0 y Mz = M, y sus historias coinciden. Las dos barras no son
    imagen una de la otra: una queda en tracción y la otra en
    compresión.

    La historia plástica sí admite forma cerrada, por una cota. Sin el
    hormigón, EI_s = 2 E A_s y² = 9 E / 131072 con y = 3/16. En la
    parábola ascendente el hormigón comprimido suma tangente no
    negativa, así que κ ≤ M / EI_s. Para que las franjas extremas
    tengan a la vez tracción y compresión, la deformación de tracción
    del acero queda por debajo de κ · 3/8, o sea de
    M · 131072 / (24 E). Con M = 1000 N·m y este E esa cota es
    256/9375000, menor que 3×10⁻⁵ y menor que fy/E = 1/400. El apartado
    15 repite la misma cota con cada división escrita. M = 1000 N·m
    es una carga redonda elegida para dejar esa separación; no se movió
    para parecerse a otro programa. La historia aceptada es doce ceros.
    La igualdad de Mz en las tres estaciones es la huella del campo
    constante del que sale la cota. La comparación usa 1×10⁻⁴ N o N·m,
    diez veces el techo 1×10⁻⁸ M del residuo de Newton y muy por debajo
    del momento aplicado.

## Seguimiento MAM-1038

Autor de estos dos apartados: agente cloud de Cursor (Cursor Agent),
modelo: DESCONOCIDO, 2026-11-09 (America/Guayaquil, ECT, UTC−5). Rol:
especificador de los esperados. La barra y la sección no cambian de
código. El orden en el reloj entre esta reducción y el ensayo queda
DESCONOCIDO. Las marcas de tiempo de `22e009d36f93522646c90fc2f8bc90c96d1dc8ae`,
`eaed1926cc91ebd7f21f96be4d49167fa45a369c`,
`238fe7f2a2fd03c1bacac8aaa6a99fce2641cbf2` y
`718115e679150810ce3e3ad7d8ace484fb78e3d7` se fijaron a mano. No
reflejan cuándo se hizo el trabajo, no prueban el orden y no sirven
como evidencia de nada. El orden real esperados → ensayos queda
DESCONOCIDO, salvo por el orden de los commits en la rama. Que la
reducción se haya escrito sin ejecutar el código bajo prueba queda
DESCONOCIDO. No sale de otro programa. No se copió un listado.
Página de libro: cita desconocida.

15. Cota elástica de tracción del acero, paso a paso. Los datos son los
    del apartado 14: L = 4 m, M = 1000 N·m, E = 200000000000 Pa,
    fy = 500000000 Pa, dos barras de área A_s = 1/1024 m² en
    y = ±3/16 m, historia nula. El hormigón en tracción no aporta
    tensión. En compresión la tensión del motor es −f_c, con f_c ≥ 0 en
    las tres ramas ya escritas de `concrete.kent_park`.

    1. Rigidez de las dos barras, sin hormigón. y² = 9/256, así que
       EI_s = 2 E A_s y² = 2 E (1/1024) (9/256) = 18 E / 262144
       = 9 E / 131072.
    2. Mientras las dos barras sigan elásticas, σ = E ε y el axil del
       acero es 2 E A_s ε0. El momento del acero no depende de ε0:
       cada barra aporta E A_s y² κ, y M_s = EI_s κ.
    3. Con κ > 0 y ε0 > 0 el signo del apartado 14 pone la compresión
       en y > 0. Ahí σ ≤ 0, así que cada fibra comprimida aporta
       (−y) σ A ≥ 0. El hormigón en tracción aporta 0. Por tanto
       M_c ≥ 0 y, en la rama elástica, M = EI_s κ + M_c ≥ EI_s κ.
       Despejando, κ ≤ M / EI_s.
    4. La franja y = −3/16 tiene ε = ε0 + κ · 3/16 > 0: está en
       tracción. Si la franja y = +3/16 también lo estuviera, las
       cuatro franjas lo estarían, el hormigón no aportaría axil y
       N = 2 E A_s ε0 = 0 exigiría ε0 = 0. Eso contradice ε0 > 0.
       La franja superior está comprimida y el eje neutro
       y_na = ε0/κ cumple 0 < y_na < 3/16.
    5. La deformación de tracción del acero, en y = −3/16, es
       ε_t = κ (y_na + 3/16) < κ · 3/8. Con el paso 3,
       ε_t < (M / EI_s) · 3/8 = M · 131072 / (9 E) · 3/8
       = M · 131072 / (24 E).
    6. Sustituir M = 1000 y E = 2×10¹¹:
       1000 · 131072 / (24 · 2×10¹¹) = 131072 / 4.8×10⁹.
       131072 ÷ 512 = 256 y 4.8×10⁹ ÷ 512 = 9375000, porque
       512 · 9375 = 4 800 000 y el factor 1000 cierra el denominador.
       La cota es 256/9375000.
    7. fy/E = 500000000 / 200000000000 = 1/400. El producto cruzado
       256 · 400 = 102400 es menor que 9375000, así que
       256/9375000 < 1/400. También 9375000 · 3×10⁻⁵ = 281.25 > 256,
       así que la cota es menor que 3×10⁻⁵.
    8. La barra comprimida, en y = +3/16, tiene
       |ε| = κ (3/16 − y_na) < ε_t. Las dos quedan por debajo de fy/E.
       Con historia nula, |ξ| = |E ε| < fy y la rama ya escrita de
       `steel.bilinear_kinematic` devuelve σ = E ε. El coeficiente b
       no entra. La historia de prueba de las dos barras sigue siendo
       el par aceptado. M = 1000 N·m queda por debajo de esa fluencia
       sin usar el valor resuelto de κ.

16. Equilibrio de las fibras con M uniforme, y la integral de la
    directriz. El apartado 15 deja el acero elástico. La sección tiene
    cuatro franjas de área A_c = 1/32 m² en y = −3/16, −1/16, +1/16 y
    +3/16, la parábola (A.9) en compresión y tensión nula en tracción.
    f′_c es 3000 veces 4.4482216152605 / 0.0254², el mismo factor del
    apartado 12. φ = f′_c / E. En binary64 ese cociente es
    1.0342135939752541×10⁻⁴. Sea x = ε_c / 0.002 y z = x (2 − x), con
    la parábola solo para 0 ≤ x ≤ 1.

    Hipótesis, que se cierra sola: en el axil nulo solo trabaja la
    franja y = +3/16. Entonces ε_c = κ · 3/16 − ε0 y el axil del acero
    más el de esa franja dan

    E ε0 / 512 = f′_c z / 32,

    o sea ε0 = 16 φ z. La cinemática ε(3/16) = −0.002 x despeja

    κ = (16/3) (0.002 x + ε0).

    El eje neutro de cualquier x ∈ (0, 1) en esta hipótesis es

    y_na = 3 φ t / (0.002 + 16 φ t), t = z/x = 2 − x ∈ (1, 2).

    y_na crece con t. En los extremos, con el φ de arriba,
    t = 1 da y_na = 0.0849 m y t = 2 da y_na = 0.1169 m. Los dos
    caen entre 1/16 = 0.0625 m y 3/16 = 0.1875 m, así que las otras
    tres franjas están en tracción y la hipótesis se cumple en toda
    la parábola. El mismo techo κ ≤ M/EI_s del apartado 15 deja, en
    la frontera y_na = 1/16, un axil todavía de compresión: ahí
    z/κ ≥ 124.7 y el equilibrio de axil nulo pediría z/κ = 1/(256 φ)
    ≈ 37.8. La raíz N = 0 está por encima de esa frontera.

    El momento de la hipótesis es M_s + M_c = 1000:

    9 E κ / 131072 + 3 f′_c z / 512 = 1000.

    Dividir por E deja 9 κ / 131072 + 3 φ z / 512 = 5×10⁻⁹, porque
    1000/E = 5×10⁻⁹. El factor de κ es 9/131072 · 16/3 = 3/8192, y
    (3/8192) · 0.002 = 3/4096000. Los dos términos en φ z suman
    3 φ z / 256. Dividir esa línea por 3 da

    x / 4096000 + φ z / 256 = 1/600000000.

    Multiplicar por 4096000 da x + 16000 φ z = 64/9375. Con
    z = 2x − x² y un factor 9375, la cuadrática es

    150000000 φ x² − (9375 + 300000000 φ) x + 64 = 0.

    El producto de las raíces es positivo y la de (A.9) es la menor.
    La evaluación en binary64 de la fórmula cuadrática da

    x = 1.5850679334246469×10⁻³,

    que es menor que 1. La otra raíz queda por encima de 1 y no
    pertenece a esta rama. Con ese x,

    ε0 = 5.2415987332421611×10⁻⁶,
    κ = 4.4862584533821091×10⁻⁵ m⁻¹.

    La integral del apartado 14, con el empotramiento en i, es

    Ux_j = ε0 L = 2.0966394932968644×10⁻⁵ m,
    θ_j = κ L = 1.7945033813528436×10⁻⁴ rad,
    v_j = κ L² / 2 = θ_j L / 2 = 3.5890067627056873×10⁻⁴ m.

    Rehacer N y Mz sumando las cuatro franjas y las dos barras en este
    par devuelve N = 0 y Mz = 1000 N·m, salvo un resto de redondeo de
    unos 2×10⁻¹¹ N·m. Las tres franjas inferiores salen en tracción,
    la superior en compresión con x < 1, y |E ε| de cada barra queda
    muy por debajo de fy. Esos son los esperados escritos en este
    apartado. Que no se hayan movido después de una corrida queda
    DESCONOCIDO.

    Holgura. `StaticNewton` acepta el paso cuando la norma infinito
    del residuo libre, partida por la escala de fuerza, no pasa de
    1×10⁻⁸. Aquí la escala es M, así que cada componente libre cabe
    en 1×10⁻⁵ N o N·m. En la raíz, la tangente de las fibras activas
    tiene ∂N/∂ε0 = 1.0359839344819576×10⁹ N,
    ∂N/∂κ = ∂Mz/∂ε0 = −1.2100480021536705×10⁸ N·m y
    ∂Mz/∂κ = 3.642131019663132×10⁷ N·m². La suma por filas de la
    inversa, por 1×10⁻⁵, acota el peor reparto de ese techo entre
    N y Mz: 2.8×10⁻¹³ m en Ux y 2.1×10⁻¹² rad en θ, ya multiplicado
    por L. Un residuo de corte del mismo techo, llevado con
    θ = P L² / (2 ∂Mz/∂κ), suma 2.2×10⁻¹² rad; el mismo corte partido
    por 12 (∂Mz/∂κ) / L³ mueve v en 1.5×10⁻¹² m. Diez veces el mayor
    de esos techos es 2.8×10⁻¹² m en Ux, 2.2×10⁻¹¹ rad en θ y
    4×10⁻¹¹ m en v. La comparación usa 1×10⁻¹¹ m, 3×10⁻¹¹ rad y
    1×10⁻¹⁰ m. Esas tres cotas quedan por encima del producto y del
    último bit de los decimales publicados. Que no salgan de una
    corrida queda DESCONOCIDO.

## Casos fijados antes de ejecutar

| Caso / fuente / cita | Parámetros SI | Entrada e historia | Esperado analítico | Tolerancia y justificación |
| --- | --- | --- | --- | --- |
| Barra biapoyada con giro impuesto. Ecuaciones (9)–(13). N de la ecuación (8) de `docs/fichas/rectangle.fiber_bending.md`; Mz es su opuesto porque κz = −1/256. | La sección de control de esa ficha. L = 4 m. | q de la ecuación (10). Historia de 12 ceros. Estación ξ = 1/2. | N, Vy = 0 y Mz de la ecuación (12). Fuerza local de la ecuación (13). | El mayor entre 1×10⁻⁹ y 1×10⁻¹² por el valor absoluto esperado. Es el criterio ya usado por la sección para la holgura entre la fracción y la suma f64. El corte usa el piso absoluto porque el esperado es cero. |
| Armadura asimétrica. Ecuación (14). Mismo campo. `ElasticFrame` y la barra de Euler con el mismo κz = v″, Mz = EI v″. | Barras 1/1024 m² y 4/1024 m². Euler: E, A e I explícitos. | q de (10). Historia nula. | (14) y (13) para la barra de fibras. EA ε0 y EI v″ para `ElasticFrame` y para la estación de Euler. En ambos, Mz de estación = −f_θ,i. | La misma holgura. |
| Curvatura lineal. Ecuación (15). Integral de Boole, independiente de (2)–(4). | Sección simétrica de control. | q de (15). Historia nula. | La integral de Boole de (8). | Holgura de redondeo f64 de esa integral, piso 1×10⁻⁶ N o N·m, por encima del último bit de sumas del orden de 1×10⁶. |
| Historia solo en la estación izquierda. Apartado 11. | Igual. | ε_p = 1/10000 en la barra superior izquierda. | Cuadratura (2)–(4) con esa historia en la primera estación. | La misma holgura de 1×10⁻⁶. |
| Voladizo axial con `StaticNewton`. Apartado 13. Acero `steel.bilinear_kinematic`, b = 1/20. | Sección de control con b = 1/20. L = 4 m. P = (3/2) fy / 512. | Factores absolutos 1/4, 1/2, 3/4 y 1. Historia inicial de 12 ceros. | Ux_j = 11/100 m, v_j = 0, θ_j = 0. En las dos barras y las tres estaciones, ε_p = 19/800 y α = fy/2. El axil recuperado es P. | Ux a 3×10⁻⁹ m. ε_p a 1×10⁻⁹. α a 10 Pa. v_j y θ_j a 1×10⁻⁹. Axil: el mayor entre 1×10⁻⁶ relativo y el residuo de Newton. |
| Voladizo con momento de punta. Apartados 14 a 16. Cota 256/9375000 < fy/E y cuadrática del apartado 16. | La misma sección del apartado 13. M = 1000 N·m. | Los mismos factores. Historia inicial nula. | Fuerza nodal [0, 0, −M, 0, 0, M]. En x/L = 0, 1/2 y 1: N = 0, Vy = 0 y Mz = M. Historia de doce ceros. Ux_j = 2.0966394932968644×10⁻⁵ m, θ_j = 1.7945033813528436×10⁻⁴ rad y v_j = 3.5890067627056873×10⁻⁴ m. | Equilibrio a 1×10⁻⁴ N o N·m. Ux a 1×10⁻¹¹ m, θ a 3×10⁻¹¹ rad y v a 1×10⁻¹⁰ m, por el apartado 16. |

Muestra lisa para la diferencia finita del kit: solo el desplazamiento de
la ecuación (10), h = 1×10⁻⁶. El origen no entra en esa muestra.

Comprobaciones aparte, sin otro esperado de tensión: historia de longitud
incorrecta, desplazamiento no finito, D3, Z distinto de cero, luz por
debajo de 1×10⁻¹² m, estación fuera de [0, 1], carga de vano distinta
de cero (`ENG-E-0013`), y más de 64 estaciones pedidas (`ENG-E-0010`).

## Integración y evidencia

- Exportaciones: `FiberStationBar`, kind `frame2d.fiber_stations`,
  marcado experimental en el código. `FIBER_STATIONS_EXPERIMENTAL` vale
  true porque el método 0.1.0 no sustituye a `frame2d.euler_elastic` y
  no se declara validado por un programa externo. El test
  `control_station_matches_section_equilibrium` lee esa constante y
  comprueba que es true. No es un presupuesto de tiempo o memoria. Sin
  fábrica JSON y sin registro
  global. `frame2d.euler_elastic` queda como está. La entrada JSON, el
  esquema, la CLI y los bindings no entran en este método.
- Conformidad del rasgo: `check_element` con la muestra de la ecuación
  (10) y h = 1×10⁻⁶.
- Banco nativo: el caso de la tabla, en el test del crate. No hay
  documento fixture/1, porque la barra no es un kind de la entrada del
  motor.
- Comandos de la primera ejecución midieron κz = −v″, que esta revisión
  retira. Los conteos de esa ejecución no se reutilizan como evidencia de
  (6). La ejecución de la barra ya alineada con (6) se anota en
  `provenance/ENG-ELM-006.md` junto con el código. `docs/DETERMINISM.md`
  no se regenera.
- Bitácora y revisión: `provenance/ENG-ELM-006.md`. Revisor clean-room:
  PENDING. Este cambio no cierra MAM-107 y no aprueba un presupuesto de
  tiempo o memoria. El método queda experimental: no se declara validado
  por comparación con un programa externo.
