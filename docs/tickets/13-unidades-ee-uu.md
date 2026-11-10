# 13 · Unidades usuales de EE. UU.

- Estado: F1 es el corte de catálogo y conversión. F2, F3 y F4 siguen abiertas.
- Relación: MAM-1043, ENG-CORE-012. No cierra MAM-12 ni MAM-107.
- Aprobación: Jonathan, 2026-11-10 a las 01:12 ECT, a través de J0: «Apruebo que MAM-1043 cambie el contrato de entrada para declarar unidades SI o EE. UU.» F2 ya no espera otra decisión. El cambio pasa por MAM-990. F1 no cambia el contrato.
- Reglas comunes: [README.md](README.md).

## Reglas que no se relajan

- La unidad va siempre explícita. Nunca se supone SI.
- La conversión es exacta y ocurre en la frontera. El núcleo calcula en SI.
- Cada unidad nueva tiene prueba de ida y vuelta.
- La masa en un sistema de fuerza se declara. No se deduce la gravedad.
- Una unidad desconocida es un error con código estable, y el mensaje nombra el texto recibido. El precedente es MAM-918 (`ENG-E-0001`).

## Qué ya existe

MAM-12 (ENG-CORE-002) cierra las unidades del núcleo en SI.
`crates/engine-core/src/units` acepta N, kN, MN, m, cm, mm, N·m, kN·m,
MN·m, Pa, kPa, MPa, GPa, rad, deg y la unidad adimensional «1».
`engine-input/1` solo admite `units: "SI"`.
Cualquier otro texto da `ENG-E-0001`.

## Anclas exactas

La tabla de F1 sale de estas relaciones, sin redondeo intermedio:

- 1 in = 0,0254 m;
- 1 lbm = 0,45359237 kg;
- 1 lbf = 0,45359237 × 9,80665 N;
- 1 kip = 1000 lbf;
- 1 ft = 12 in;
- un intervalo Δ°F = 5/9 K.

El desplazamiento de la escala Fahrenheit se trata aparte de ese factor,
y solo cuando la temperatura es absoluta. Las citas están en
[docs/UNITS.md](../UNITS.md). Si una página no se pudo verificar, allí dice
«cita desconocida».

## Fases

### F1 · Catálogo y conversión

Tabla única, versionada, con factores exactos hacia el SI canónico.
Catálogo cerrado por dimensión: longitud, fuerza, momento, tensión,
fuerza/longitud, fuerza/área, masa, densidad, aceleración, temperatura
y coeficiente térmico.

Símbolos SI: N, kN, MN, mm, cm, m, Pa, kPa, MPa, GPa, kN/m, kN/m², kg, t,
kg/m³, m/s², °C, 1/°C.

Símbolos de EE. UU.: lbf, kip, in, ft, psi, ksi, psf, ksf, plf, klf,
lbf·in, lbf·ft, kip·in, kip·ft, lbm, kip·s²/in, pcf, in/s², ft/s², g,
°F, 1/°F.

El momento necesita un ancla SI. El núcleo ya calcula en N·m, y esa unidad
entra en la tabla como ancla. No es un símbolo nuevo del contrato.
Jonathan puede rechazarla; la decisión queda abierta.

Los presets cerrados N-m, kN-m, N-mm, kip-in, kip-ft, lb-in y lb-ft se
definen como tipo. No se publican en `engine-input/1` ni en ningún esquema.
Donde el conjunto coherente no tiene una unidad de masa dentro del catálogo,
el preset deja la masa vacía. No la rellena con g.

F1 no toca esquemas, goldens ni el valor `units: "SI"`.

### F2 · Contrato de entrada

El documento declara el sistema con el campo obligatorio `units`. Los
valores son `"SI"` o `"US"`. No hay omisión ni otro valor que se lea como
SI.

No es aditivo. En `engine-input/1`, `units` es la constante `"SI"`.
Pasarlo a una enumeración cambia el tipo y el significado: un texto que
hoy es `ENG-E-0001` pasaría a ser válido, y quien solo comprueba
`engine-input/1` leería números en kip como si fueran newtons. MAM-990
pide versión mayor para eso. El contrato nuevo es `engine-input/2`.
`engine-input/1` no se reescribe.

`"US"` es el conjunto kip-in de la tabla: longitud en in, fuerza en kip,
momento en kip·in, tensión en ksi, masa en kip·s²/in. La conversión a SI
es exacta y ocurre en la frontera, antes del núcleo. Los giros siguen en
radianes. La migración está en
[docs/migration/engine-input-2.md](../migration/engine-input-2.md).

### F3 · `units_out` en resultados e informes

`UnitsOut` ya existe y guarda la unidad SI de cada campo. F3 elige si la
salida sigue en SI, con la unidad declarada, o si un informe puede pedir
el sistema de entrada. No se hace en F1.

### F4 · Paridad y tres fixtures

Misma respuesta, dentro de la tolerancia del caso, entre un modelo en
kip-in y el mismo modelo en kN-m, en lineal, modal y P-Delta.
Tres fixtures de EE. UU. con respuesta analítica, cada uno con `source`
y cita. No se calibran contra otro programa.

## Ecuaciones normativas

Cada ecuación declara su sistema nativo. La conversión, si hace falta,
es anterior a la ecuación. La ecuación no esconde un factor.

| Ecuación | Dónde está | Sistema nativo |
| --- | --- | --- |
| Equilibrio y elasticidad del contrato estructural: m, N, Pa, N·m, kg, rad. | `crates/engine-contracts/src/lib.rs` | SI. |
| Temperatura uniforme de la cercha, N = E A α ΔT. α en 1/K y ΔT en K. | `provenance/ENG-LOAD-TEMP-972.md` y el test de la cercha. F1 no la reescribe. | SI. El catálogo convierte 1/°C y 1/°F en la frontera. |
| Coeficiente térmico. Se conserva ε = α ΔT. Con ΔT_K = ΔT_°F × 5/9 queda α_K = α_°F × 9/5. Derivación de esta ficha, no de un libro. | Tabla de F1. | La barra sigue en 1/K. El 9/5 es de frontera. |
| Pandeo lineal, K + λ Kg = 0. | `crates/engine-analysis` y el ticket 04. | SI. La página de Timoshenko y Gere sigue como cita desconocida. No se inventa. |
| `concrete.kent_park`, (A.12) del informe BARC/2009/E/026, apéndice A. El 1000 del denominador está impreso en psi. | `docs/fichas/concrete.kent_park.md` | psi en (A.12). La ley recibe Pa. El factor ya escrito, 1 lbf = 4,4482216152605 N y 1 in = 0,0254 m, coincide con el producto de las anclas. F1 no reescribe la ley. |
| Perfil geotécnico: profundidad en m hacia abajo, peso específico en kN/m³, tensión en kPa. | `docs/V1.md`, sección 1. | SI con prefijos explícitos. kN/m³ no está en el catálogo de F1 y sigue fuera. |
| P-Delta y modal del núcleo. | Cálculo interno en SI. La paridad kip-in frente a kN-m es F4. | SI. `provenance/ENG-ANL-003.md` ya anotó un caso en kip·in con los mismos factores de kip e inch. F1 no reejecuta ese caso. |
| Comentario C2 de AISC 360-16, citado en `docs/DETERMINISM.md` como antecedente de tolerancias. | No se copia la especificación y no se añade su ecuación. | Si una ecuación de esa especificación entra más adelante, declara su sistema (kip-in en ese comentario) antes de convertirse. |

## Hecho cuando

F1:

- La tabla versionada y el catálogo están en `engine-core`, sin depender de un crate nuevo.
- La ida y vuelta SI ↔ EE. UU. de cada unidad queda dentro de 1e-15 en magnitudes de 1e-6 a 1e9, con semilla determinista.
- Una unidad desconocida devuelve `ENG-E-0001` y el mensaje contiene el texto recibido.
- Mezclar lbm y lbf sin una aceleración declarada es un error. No aparece 9,80665 por omisión.
- `units: "SI"` y los esquemas no cambian. Los goldens y los digestos de determinismo quedan iguales.
- Hay `docs/UNITS.md`, `provenance/ENG-CORE-012.md` y fragmento en `changelog.d/`. El revisor clean-room queda en PENDING.

F2:

- `engine-input/2` exige `units` igual a `"SI"` o `"US"`, sin valor por omisión.
- `"US"` se convierte a SI en la frontera con los factores de `units-table/1`.
- La ida y vuelta de esa frontera queda dentro de 1e-15.
- `engine-input/1` y los goldens de documentos SI no cambian.
- La migración está escrita en `docs/migration/engine-input-2.md`.

F3: resultados e informes publican la unidad de salida acordada.

F4: la paridad kip-in / kN-m pasa en lineal, modal y P-Delta, y los tres fixtures analíticos están en el repositorio con `source` y cita.

MAM-107 no se cierra. No se aprueba un presupuesto de tiempo o memoria.
