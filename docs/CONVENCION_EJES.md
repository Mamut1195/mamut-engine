# Convención de ejes y signos · ENG-CORE-003

**DECISIÓN PROVISIONAL (J0/J2, 2026-10-10): X-Y con Rz en planos, Z arriba; revocable por Jonathan**

**Estado: DECISIÓN PROVISIONAL (J0/J2, 2026-10-10): X-Y con Rz en planos, Z arriba; revocable por Jonathan.** Este texto y el rustdoc de `engine-model`
registran esa decisión provisional.
Redacción propia basada en mecánica estructural estándar.

## Ejes globales y dimensión

X, Y y Z forman una terna diestra: X × Y = Z. Z es vertical hacia arriba.
Los modelos planos D2 se representan en X-Y, con Z = 0 y componentes Ux, Uy y
Rz. Por tanto, el plano propuesto es horizontal; un pórtico vertical requeriría
D3 o una futura decisión explícita sobre otro plano. No se interpreta Y como
vertical. D3 admite Ux, Uy, Uz, Rx, Ry y Rz.

Cada nodo declara `Dim`. El modelo de dominio permite almacenar nodos de ambas
dimensiones, sin resolver en este ticket la compatibilidad de sus conexiones.
Las coordenadas tienen tres componentes; la condición Z = 0 de D2 queda como
convención de entrada y no se verifica geométricamente aquí. Los GDL de nodos,
apoyos, acciones y masas sí se contrastan con la dimensión del nodo.

## Ejes locales del miembro

El extremo i es el primer nodo y j el segundo. El eje unitario local x apunta
de i a j. En D2 se toma y = (-x_y, x_x, 0) y z = x × y, de modo que z apunta
hacia +Z. En D3, la dirección x no basta para elegir y: el cálculo futuro deberá
recibir una orientación transversal explícita, normalizarla y hacerla ortogonal
a x antes de formar z = x × y. Este crate solo almacena el dominio y no crea
una orientación espacial automática. Invertir i y j cambia la orientación local;
no cambia las direcciones de cargas definidas en ejes globales.

## Acciones, desplazamientos y giros

Una fuerza o desplazamiento nodal positivo sigue su eje global positivo.
Los momentos y giros positivos siguen la regla de la mano derecha alrededor
del eje indicado. En D2, Rz y un momento nodal positivo alrededor de Z son
antihorarios vistos desde +Z hacia el origen. Una carga sobre Ux, Uy o Uz es
una fuerza; una carga sobre Rx, Ry o Rz es un momento.

Las resultantes internas se expresan en ejes locales. N positivo significa
tracción: las fuerzas normales de las dos caras tiran hacia fuera del segmento.
Las masas asociadas a traslaciones son masas concentradas; las asociadas a giros
son inercias rotacionales. Todas las entradas deben usar unidades coherentes;
el dominio no impone un sistema de unidades.

## Signos por cara de sección y por extremo

Una cara positiva tiene normal exterior +x y una cara negativa tiene normal
exterior -x. Los símbolos siguientes son resultantes internas con signo, no
magnitudes siempre positivas. Para un valor positivo se aplican estas direcciones:

| Resultante | Cara positiva (extremo j) | Cara negativa (extremo i) |
| --- | --- | --- |
| Axial N | Fuerza +x | Fuerza -x |
| Cortante Vy | Fuerza +y | Fuerza -y |
| Cortante Vz | Fuerza +z | Fuerza -z |
| Torsor T | Momento alrededor de +x | Momento alrededor de -x |
| Flector My | Momento alrededor de +y | Momento alrededor de -y |
| Flector Mz | Momento alrededor de +z | Momento alrededor de -z |

El sentido de cada momento de la tabla se interpreta con la mano derecha.
En D2 solo intervienen N, Vy y Mz. Por ejemplo, Vy positivo apunta hacia -y
en i y hacia +y en j; Mz positivo es horario en i y antihorario en j vistos
desde +z. En una sección interior, las fuerzas y momentos de caras opuestas
son iguales y contrarios al comparar la misma sección. La tabla expresa una
convención de signos, no obliga a que las resultantes en i y j tengan el mismo
valor en presencia de cargas distribuidas.

Las cargas nodales globales y estas resultantes de caras son conceptos distintos.
Un futuro ensamblaje deberá declarar si entrega acciones sobre el miembro o
acciones del miembro sobre el nodo: pasar de unas a otras invierte los signos.
Este ticket no calcula ni transforma esas acciones.

## Tolerancia geométrica y alcance de la inspección

`LENGTH_TOLERANCE = 1e-12` es absoluta, en la unidad usada para las coordenadas.
Una distancia euclídea en tres componentes menor o igual a ese valor se informa
como longitud nula (`ENG-E-0005`). La distancia se acumula con `hypot` para evitar
el desbordamiento de cuadrados y la pérdida de distancias pequeñas al elevarlas
al cuadrado. No se comprueba longitud si faltan extremos o si sus coordenadas
no son finitas; esos defectos tienen sus propios diagnósticos.

`validate` no realiza análisis estructural ni comprueba estabilidad, positividad
de rigideces, compatibilidad dimensional entre nodos o propiedades obligatorias.
Sus diagnósticos se acumulan en un orden determinista y reutilizan el catálogo
de `engine-core`. La inserción no valida ni elimina duplicados.
