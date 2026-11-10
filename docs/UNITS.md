# Unidades · tabla `units-table/1`

Fase 1 de MAM-1043 (ENG-CORE-012). La ficha es
[docs/tickets/13-unidades-ee-uu.md](tickets/13-unidades-ee-uu.md).

El núcleo sigue calculando en SI. Esta tabla convierte en la frontera.
No es un esquema: `engine-input/1` sigue aceptando solo `units: "SI"`, y
`crates/engine-core/src/units` sigue rechazando `in`, `kip` o `lbm` en
`Unit::parse`. El catálogo nuevo es `engine_core::units::catalog`, versión
`units-table/1` (`TABLE_VERSION = 1`).

Un factor que cambie pide otra versión de la tabla. Esta fase no reserva
esa versión en el contrato de entrada.

## Anclas

Todo factor de la tabla es un producto exacto de estas relaciones. No hay
un redondeo intermedio guardado a mano.

| Relación | Valor exacto | Fuente |
| --- | --- | --- |
| Pulgada | 1 in = 0,0254 m = 127/5000 m | Yarda internacional de 1959, dividida por 36. |
| Pie | 1 ft = 12 in = 0,3048 m = 381/1250 m | La misma yarda, dividida por 3. No es el pie de agrimensura de ese aviso. |
| Libra-masa | 1 lbm = 0,45359237 kg = 45359237/100000000 kg | Libra avoirdupois internacional de 1959. |
| Gravedad convencional | g_n = 9,80665 m/s² = 196133/20000 m/s² | Declaración 2 de la 3.ª CGPM (1901). |
| Libra-fuerza | 1 lbf = 1 lbm × g_n = 4,4482216152605 N | Producto de las dos filas anteriores. |
| Kip | 1 kip = 1000 lbf | Definición adoptada por MAM-1043. Página de un estándar que bautice «kip»: cita desconocida. |
| Intervalo Fahrenheit | Δ°F = 5/9 K | Definición adoptada por MAM-1043. Documento normativo de la escala Fahrenheit: cita desconocida. |
| Cero Celsius | 0 °C = 273,15 K = 5463/20 K | Brochure del SI, 9.ª edición. |

### Yarda y libra de 1959

National Bureau of Standards, «Refinement of Values for the Yard and the
Pound», Federal Register, vol. 24, 1 de julio de 1959, p. 5348, F.R. Doc.
59-5442. En esta sesión se leyó la reproducción de ese aviso publicada por
NOAA (`FRdoc59-5442`). El aviso deja estos equivalentes exactos, vigentes
el 1 de julio de 1959:

- 1 yard = 0,9144 meter;
- 1 pound (avoirdupois) = 0,45359237 kilogram.

La pulgada y el pie internacional salen de esa yarda: 1 in = yard/36 =
0,0254 m y 1 ft = yard/3 = 0,3048 m. El mismo aviso conserva, para
agrimensura, otro pie. Esta tabla no lo implementa y no lo convierte.

### Gravedad convencional

Conférence Générale des Poids et Mesures, 3.ª reunión (1901), Declaración 2,
«Declaration on the unit of mass and on the definition of weight;
conventional value of g_n», Proceedings of the 3rd CGPM (1901), p. 70.
DOI `10.59161/CGPM1901DECL2E`. El valor adoptado es 980,665 cm/s², esto es
9,80665 m/s². Se consultó la página del BIPM de esa declaración. El texto
oficial es el francés; aquí se usa el valor numérico.

Esa gravedad es una aceleración declarada. No es el peso de un cuerpo en
un lugar, y el catálogo no la inserta cuando el caller no la pasa.

### Brochure del SI

Bureau International des Poids et Mesures, *The International System of
Units (SI)*, 9.ª edición, 2019, DOI `10.59161/AUEZ1291`. Se consultó el
texto en inglés.

- En la sección del kelvin: la temperatura Celsius es t = T − T_0, con
  T_0 = 273,15 K, y t/°C = T/K − 273,15. Un intervalo de 1 °C es igual a
  un intervalo de 1 K. NIST Special Publication 330, edición 2019, numera
  esas dos relaciones como ecuaciones (9) y (10).
- Tabla 7, prefijos: 10³ kilo (k), 10⁶ mega (M), 10⁹ giga (G), 10⁻² centi
  (c), 10⁻³ milli (m). De ahí salen kN, MN, kPa, MPa, GPa, cm y mm.
- Tabla 8, unidades ajenas al SI admitidas con él: 1 t = 1 Mg. La nota (e)
  dice que el CIPM adoptó la tonelada y el símbolo t en 1879. El
  megagramo es 10³ kg por el prefijo mega aplicado al gramo. El símbolo `t`
  de esta tabla es esa tonelada, no la short ton de 2000 lbm.

El extracto de texto de la tabla 8 perdió superíndices. La igualdad
«1 t = 1 Mg» y la nota (e) sí se leyeron. El factor 1000 kg es el del
megagramo, no una cifra copiada de otra fila.

## Tabla

La columna «hacia» es la unidad canónica del SI de esa dimensión. El
factor es exacto. `f64` es solo la evaluación IEEE-754 del cociente; la
ida y vuelta repite ese mismo `f64`.

| Símbolo | Dimensión | Lista | Hacia | Factor exacto |
| --- | --- | --- | --- | --- |
| m | longitud | SI | m | 1 |
| cm | longitud | SI | m | 1/100 |
| mm | longitud | SI | m | 1/1000 |
| in | longitud | US | m | 127/5000 |
| ft | longitud | US | m | 12 × in |
| N | fuerza | SI | N | 1 |
| kN | fuerza | SI | N | 1000 |
| MN | fuerza | SI | N | 10⁶ |
| lbf | fuerza | US | N | lbm × g_n |
| kip | fuerza | US | N | 1000 × lbf |
| N·m | momento | SI | N·m | 1 |
| lbf·in | momento | US | N·m | lbf × in |
| lbf·ft | momento | US | N·m | lbf × ft |
| kip·in | momento | US | N·m | kip × in |
| kip·ft | momento | US | N·m | kip × ft |
| Pa | tensión y fuerza/área | SI | Pa | 1 |
| kPa | tensión y fuerza/área | SI | Pa | 1000 |
| MPa | tensión y fuerza/área | SI | Pa | 10⁶ |
| GPa | tensión y fuerza/área | SI | Pa | 10⁹ |
| kN/m² | tensión y fuerza/área | SI | Pa | 1000 |
| psi | tensión y fuerza/área | US | Pa | lbf / in² |
| ksi | tensión y fuerza/área | US | Pa | 1000 × psi |
| psf | tensión y fuerza/área | US | Pa | lbf / ft² |
| ksf | tensión y fuerza/área | US | Pa | 1000 × psf |
| kN/m | fuerza/longitud | SI | N/m | 1000 |
| plf | fuerza/longitud | US | N/m | lbf / ft |
| klf | fuerza/longitud | US | N/m | kip / ft |
| kg | masa | SI | kg | 1 |
| t | masa | SI | kg | 1000 |
| lbm | masa | US | kg | 45359237/100000000 |
| kip·s²/in | masa | US | kg | kip / in |
| kg/m³ | densidad | SI | kg/m³ | 1 |
| pcf | densidad | US | kg/m³ | lbm / ft³ |
| m/s² | aceleración | SI | m/s² | 1 |
| in/s² | aceleración | US | m/s² | in |
| ft/s² | aceleración | US | m/s² | ft |
| g | aceleración | US | m/s² | g_n |
| °C | temperatura, intervalo | SI | K | 1 |
| °F | temperatura, intervalo | US | K | 5/9 |
| 1/°C | coeficiente térmico | SI | 1/K | 1 |
| 1/°F | coeficiente térmico | US | 1/K | 9/5 |

N·m está en la tabla porque el momento necesita un ancla SI y el núcleo ya
calcula en N·m. El listado SI del encargo no lo repetía. Es una decisión
abierta: Jonathan puede pedir que salga del catálogo público y quede solo
como unidad interna.

N/m, K y 1/K son los nombres canónicos de tres dimensiones. El símbolo
cerrado de esas dimensiones es kN/m, °C y 1/°C. No se acepta el texto
`N/m` ni el texto `K` como unidad del catálogo.

Tensión y fuerza por área comparten dimensión: Pa, kN/m², psi y psf se
convierten entre sí. Separarlas para que psi no convierta a psf queda
pendiente de Jonathan.

### Alias ASCII

El símbolo canónico es el de la tabla. También se acepta, sin cambiar de
unidad:

- N·m: `N*m`, `N.m`, `Nm` (las mismas grafías que ya acepta `Unit::parse`);
- lbf·in, lbf·ft, kip·in, kip·ft: el punto medio sustituido por `*`;
- kN/m², kg/m³, m/s², in/s², ft/s²: el superíndice `²` o `³` escrito como
  `2` o `3`;
- kip·s²/in: `kip·s2/in`, `kip*s2/in`, `kip*s²/in`;
- °C, °F, 1/°C, 1/°F: `degC`, `degF`, `1/degC`, `1/degF`.

No se recortan espacios. No se cambia entre mayúsculas y minúsculas.
`lb`, `lbs`, `pound`, `inch`, `ton`, `tonne`, `gram`, `G`, `T` y `SI`
son desconocidos: `ENG-E-0001`, y el mensaje incluye el texto recibido
entre comillas angulares. `g` es la aceleración g_n, no el gramo. `t` es
la tonelada métrica, no la short ton.

`N-m`, `kN-m`, `N-mm`, `kip-in`, `kip-ft`, `lb-in` y `lb-ft` no son
unidades. Son nombres de preset.

## Temperatura absoluta

El factor 5/9 convierte intervalos. No resta 32.

La lectura absoluta usa otra operación:

- K = °C + 273,15;
- °C = (°F − 32) × 5/9;
- K = (°F − 32) × 5/9 + 273,15.

El desplazamiento guardado en la fila de °F es el cociente exacto
273,15 − 32 × 5/9. En `f64`, sumar ese desplazamiento ya redondeado no
deja 32 °F en 0 °C. La conversión absoluta evalúa la resta de 32 antes de
escalar, que es la definición, y 32 °F, 212 °F y −40 °F caen en 0 °C,
100 °C y −40 °C.

La cota de ida y vuelta de 1e-15 cubre los factores de escala, en
magnitudes de 1e-6 a 1e9, con semilla `0x1043_2026_1110`. No se promete
esa cota para una temperatura absoluta diminuta: el desplazamiento de
273,15 K domina a 1e-6 °F.

## Masa y fuerza

`force_from_mass` solo convierte si el caller pasa una aceleración, con
su magnitud y su unidad. Si no la pasa, el error es `ENG-E-0010` y el
mensaje nombra las dos unidades. No se escribe 9,80665 en ese camino.

Eso incluye mezclar lbm y lbf. También incluye kip·s²/in y kip: la
coherencia F = m a existe, pero la aceleración tiene que declararse
(1 in/s², por ejemplo). Declarar `1 g` junto con 1 lbm devuelve 1 lbf,
porque así está definida la libra-fuerza, no porque el catálogo adivine.

`kip·s²/in` es masa, el cociente kip / (in/s²) expresado en kilogramos.
No sale de dividir un peso entre g.

## Presets

Son un tipo, `UnitPreset`. No están en `engine-input/1` ni en ningún
esquema. El nombre `lb-in` elige lbf e in. No autoriza el texto `lb`.

| Preset | Fuerza | Longitud | Momento | Masa |
| --- | --- | --- | --- | --- |
| N-m | N | m | N·m | kg |
| kN-m | kN | m | (factor 1000 N·m; no hay símbolo kN·m en el catálogo) | t |
| N-mm | N | mm | (factor 0,001 N·m; no hay símbolo N·mm) | sin declarar |
| kip-in | kip | in | kip·in | kip·s²/in |
| kip-ft | kip | ft | kip·ft | sin declarar |
| lb-in | lbf | in | lbf·in | sin declarar |
| lb-ft | lbf | ft | lbf·ft | sin declarar |

«Sin declarar» significa que la masa coherente no está en el catálogo
(haría falta lbf·s²/in, lbf·s²/ft o kip·s²/ft) o que la aceleración
coherente tampoco (mm/s² en N-mm). El hueco no se rellena con lbm ni con g.

La ley `concrete.kent_park` ya convierte (A.12) con
1 lbf = 4,4482216152605 N y 1 in = 0,0254 m. Ese producto es el de esta
tabla. F1 no reescribe la ley.

## Fuera de la tabla

- kN/m³ del perfil geotécnico (`docs/V1.md`, sección 1). Sigue fuera.
- rad, deg y la unidad adimensional `1`, que ya viven en `Unit` y no
  entran en este catálogo.
- El pie de agrimensura.
- La short ton.

## F2 · contrato `engine-input/2`

Jonathan, 2026-11-10 a las 01:12 ECT, a través de J0: «Apruebo que
MAM-1043 cambie el contrato de entrada para declarar unidades SI o EE. UU.»

La frase de MAM-990 del 2026-10-31, trasladada a este encargo, dice que un
campo opcional nuevo cabe en la versión 1 y que solo un cambio incompatible
exige la versión 2: quitar o renombrar un campo, cambiar su tipo o su
significado, o volver obligatorio uno opcional. No hay una copia de MAM-990
en el repositorio.

El cambio no es aditivo. `units` en `engine-input/1` es la constante
`"SI"`. Admitir `"US"` en ese mismo campo cambia el tipo y el significado.
Por eso el contrato nuevo es `engine-input/2`. `engine-input/1` queda como
está. La migración está en
[docs/migration/engine-input-2.md](migration/engine-input-2.md).

`engine-input/2` exige `units` con uno de dos textos, `"SI"` o `"US"`.
No hay valor por omisión. `"SI"` significa las mismas magnitudes que
`engine-input/1`. `"US"` es el conjunto kip-in: in, kip, kip·in, ksi y
kip·s²/in. La frontera convierte a SI antes del núcleo. Los resultados
siguen en SI; `units_out` es F3.

## Propuesta de código, no añadida

`error-codes-1.json` se queda en 32 entradas. F1 reutiliza códigos que ya
existen:

- unidad o preset desconocido: `ENG-E-0001`, y el mensaje nombra el texto;
- dimensiones incompatibles, y masa con fuerza sin aceleración declarada:
  `ENG-E-0010`;
- cantidad no finita: `ENG-E-0008`.

PROPUESTA para Jonathan, sin número y sin fila nueva: un código distinto
para «masa y fuerza mezcladas sin aceleración declarada», si `ENG-E-0010`
le parece demasiado amplio. No se añade en este PR.

## Decisiones abiertas

1. `pcf`: en F1 es lbm/ft³, densidad másica. En la práctica de EE. UU. el
   mismo nombre suele ser peso específico, lbf/ft³. Esta tabla no tiene
   fuerza por volumen y no infiere g. Confirmar o cambiar el significado.
2. Si tensión y fuerza/área deben ser dimensiones incompatibles.
3. Si N·m permanece en el catálogo como ancla o solo como unidad interna.
4. Si el código propuesto de arriba hace falta.
