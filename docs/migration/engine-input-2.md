# Migración de engine-input/1 a engine-input/2

MAM-1043, fase 2. Fecha de la decisión: 2026-11-10, 01:12 ECT.
Jonathan, a través de J0, aprobó que el contrato de entrada declare
unidades SI o de EE. UU.

## Por qué cambia el número mayor

MAM-990: un cambio incompatible exige versión nueva. Lo incompatible es
quitar o renombrar un campo, cambiar su tipo o su significado, o volver
obligatorio un campo que era opcional.

`units` en `engine-input/1` es obligatorio y vale exactamente `"SI"`.
Abrir ese campo a `"US"` cambia el tipo (de constante a enumeración) y el
significado (un documento aceptado ya no afirma, por el solo hecho de ser
válido, que sus números están en newtons y metros). No es un campo
opcional nuevo. No cabe en `engine-input/1`.

El esquema nuevo es `engine-input/2` (`urn:mamut:engine-input:2`).
`schemas/engine-input/1.json` no se modifica. Un documento que ya valida
en la versión 1 sigue validando y produce el mismo resultado.

## Qué hay que cambiar en un documento

1. `schema` pasa de `"engine-input/1"` a `"engine-input/2"`.
2. `units` sigue siendo obligatorio. Vale `"SI"` o `"US"`. Si falta, o si
   no es uno de esos dos textos, el documento se rechaza. No se rellena
   con SI.
3. Con `"SI"`, los números significan lo mismo que en la versión 1:
   m, N, N·m, Pa, kg, kg/m³, 1/K y K en los incrementos de temperatura.
   Los giros siguen en radianes.
4. Con `"US"`, los números del conjunto estructural están en kip-in y la
   fachada los convierte a SI antes del cálculo. El núcleo y `results/1`
   siguen en SI.

No hace falta migrar un archivo que deba seguir igual. Se queda en
`engine-input/1` con `units` igual a `"SI"`.

## Conjunto `"US"` (kip-in)

| Magnitud del documento | Unidad de entrada | Unidad interna |
| --- | --- | --- |
| Coordenadas | in | m |
| Fuerza nodal, Ux Uy Uz | kip | N |
| Momento nodal, Rx Ry Rz | kip·in | N·m |
| Masa nodal de traslación | kip·s²/in | kg |
| Inercia nodal de giro | kip·s²·in | kg·m² |
| Carga de barra, fuerza/longitud | kip/in | N/m |
| Carga de barra, momento/longitud | kip | N |
| Desplazamiento prescrito de traslación | in | m |
| Giro prescrito | rad | rad |
| E, G, fy, C y la tensión de una polilínea | ksi | Pa |
| A y áreas de corte | in² | m² |
| I, Iy, Iz, J | in⁴ | m⁴ |
| rho | kip·s²/in⁴ | kg/m³ |
| b de endurecimiento, ν | adimensional | igual |
| α de temperatura | 1/°F | 1/K |
| ΔT | intervalo °F | K |
| k, ky, kz de Winkler o de sección | ksi | Pa |
| k de resorte de traslación | kip/in | N/m |
| k de resorte de giro | kip·in/rad | N·m/rad |
| `displacement_scale` | in | m |
| `force_scale` | kip | N |
| `energy_scale` | kip·in | N·m |
| `gravity` de una fuente de masa | in/s² | m/s² |

Un nombre de parámetro de sección o de ley que no esté en esa lista se
rechaza. No se deja en unidades de EE. UU. dentro del núcleo.

Estas magnitudes no se reescalan, porque ya nombran su unidad o no tienen
dimensión:

- factores de combinación, tolerancias, direcciones y la orientación;
- el giro, que en los dos sistemas está en radianes;
- la tabla de espectro: su campo `units` ya dice la unidad de cada par;
- el perfil geotécnico y la consolidación: el nombre del campo lleva la
  unidad (`thickness_m`, `gamma_kn_m3`, `delta_stress_kpa`, y el resto
  del mismo estilo).

`mamut-toml/1` sigue produciendo `engine-input/1`. La fachada Python
sigue aceptando ese contrato; las unidades tipadas de Python son MAM-1046.
El motor Rust acepta los dos esquemas en `validate`, `run` y `prepare`.
