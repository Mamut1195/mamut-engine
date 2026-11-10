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
  2026-11-10.
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

## Casos fijados antes de ejecutar

| Caso / fuente / cita | Parámetros SI | Entrada e historia | Esperado analítico | Tolerancia y justificación |
| --- | --- | --- | --- | --- |
| Barra biapoyada con giro impuesto. Ecuaciones (9)–(13). N de la ecuación (8) de `docs/fichas/rectangle.fiber_bending.md`; Mz es su opuesto porque κz = −1/256. | La sección de control de esa ficha. L = 4 m. | q de la ecuación (10). Historia de 12 ceros. Estación ξ = 1/2. | N, Vy = 0 y Mz de la ecuación (12). Fuerza local de la ecuación (13). | El mayor entre 1×10⁻⁹ y 1×10⁻¹² por el valor absoluto esperado. Es el criterio ya usado por la sección para la holgura entre la fracción y la suma f64. El corte usa el piso absoluto porque el esperado es cero. |
| Armadura asimétrica. Ecuación (14). Mismo campo. `ElasticFrame` y la barra de Euler con el mismo κz = v″, Mz = EI v″. | Barras 1/1024 m² y 4/1024 m². Euler: E, A e I explícitos. | q de (10). Historia nula. | (14) y (13) para la barra de fibras. EA ε0 y EI v″ para `ElasticFrame` y para la estación de Euler. En ambos, Mz de estación = −f_θ,i. | La misma holgura. |
| Curvatura lineal. Ecuación (15). Integral de Boole, independiente de (2)–(4). | Sección simétrica de control. | q de (15). Historia nula. | La integral de Boole de (8). | Holgura de redondeo f64 de esa integral, piso 1×10⁻⁶ N o N·m, por encima del último bit de sumas del orden de 1×10⁶. |
| Historia solo en la estación izquierda. Apartado 11. | Igual. | ε_p = 1/10000 en la barra superior izquierda. | Cuadratura (2)–(4) con esa historia en la primera estación. | La misma holgura de 1×10⁻⁶. |

Muestra lisa para la diferencia finita del kit: solo el desplazamiento de
la ecuación (10), h = 1×10⁻⁶. El origen no entra en esa muestra.

Comprobaciones aparte, sin otro esperado de tensión: historia de longitud
incorrecta, desplazamiento no finito, D3, Z distinto de cero, luz por
debajo de 1×10⁻¹² m, estación fuera de [0, 1], carga de vano distinta
de cero (`ENG-E-0013`), y más de 64 estaciones pedidas (`ENG-E-0010`).

## Integración y evidencia

- Exportaciones: `FiberStationBar`, kind `frame2d.fiber_stations`,
  marcado experimental en el código. Sin fábrica JSON y sin registro
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
