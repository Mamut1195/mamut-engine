# Ficha de método · <nombre> · <ticket>

Completar antes de implementar. No dejar campos pendientes en una ficha usada
para validar el método. La guía contiene una ficha completa para el piloto.

- Autor, modelo si es IA, fecha y rol:
- Kind propio, contrato y versión del método:
- Módulo/crate y consumidor previsto:
- Métodos existentes revisados y por qué este es distinto:

## Fuentes permitidas

Por cada fuente: autor, título, edición/año, sección/página/ecuación y texto
teórico disponible localmente. Para derivación propia: autor, fecha, título,
ubicación y ecuaciones numeradas. No inventar localizadores; si falta teoría,
declarar la duda antes de implementar. Sin listados de terceros ni tablas copiadas.

## Hipótesis, unidades y dominio

- Física y limitaciones:
- Entradas totales, unidades SI y signos:
- Parámetros, unidades, rangos y validación:
- Estado: orden, longitud, unidades, valores iniciales y estados admisibles:
- Salidas y unidades:
- Dominio numérico, orden de operaciones, overflow/underflow:
- Puntos no diferenciables y convención en cada frontera:

## Derivación propia

1. Ecuaciones de partida y cita de cada una.
2. Desarrollo del algoritmo, sin referenciar otros motores.
3. Derivada consistente respecto a la entrada, a historia aceptada fija.
4. Tangente inicial, invariantes, aceptación y descarte de estado.
5. Límites analíticos y recuperación de casos simples.

## Casos fijados antes de ejecutar

| Caso / source / cita | Parámetros SI | Entrada e historia | Esperados analíticos | Tolerancia y justificación |
| --- | --- | --- | --- | --- |
| <completar> | <completar> | <completar> | <evaluación propia con ecuación> | <completar> |

Separar muestras suaves para FD, fronteras, ciclos y pruebas negativas.
Justificar h con error de truncamiento/redondeo y verificar límites del kit.

## Integración y evidencia

- Exportaciones, descriptor y fábrica:
- Conformidad apropiada al trait:
- Benchmark nativo y, si aplica, fixture/1 de ENG-CORE-009:
- Comprobaciones, tests passed/failed y logs bajo `$CARGO_TARGET_DIR`:
- Cruce de los 16 hashes sin modificar la referencia:
- Bitácora, revisión clean-room y límites de validación:
