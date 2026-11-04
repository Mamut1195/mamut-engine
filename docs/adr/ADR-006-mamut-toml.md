# ADR-006: representación de entrada mamut-toml/1 en el CLI

- **Ticket:** MAM-939 (T1).
- **Estado:** decisión documental; implementación posterior. Los puntos
  abiertos siguen pendientes de Jonathan.
- **Fecha:** 2026-10-31 (America/Guayaquil).
- **Antecedentes:** plan de entrada TOML v4, v4.1, v4.2 y v4.3;
  [engine-input/1](../../crates/engine-api/schemas/engine-input-1.json).
- **Contrato:** [especificación mamut-toml/1](../formats/mamut-toml-1.md).
- **Procedencia:** [ENG-TOML-001](../../provenance/ENG-TOML-001.md).
- **Numeración:** MAM-993 usa [ADR-005](ADR-005-alias-name-idmap.md); este es ADR-006. No hay choque:
  decidido por el consejo (delegación de Jonathan), 2026-10-31.

## Contexto

engine-input/1 es el contrato JSON propio del motor. La escritura manual
requiere listas de propiedades, coordenadas agrupadas y referencias que
pueden expresarse con tablas TOML más legibles. El formato de entrada debe
cubrir las 35 definiciones del esquema sin introducir física, unidades
alternativas ni un contrato nuevo dentro del motor.

La documentación vigente del CLI publica cli-error/1 y cli-describe/1.
La entrada TOML necesita controles y ubicaciones de errores antes de llegar
a engine-api. Los mensajes no deben reproducir valores ni fragmentos de
entrada. El orden del documento convertido importa tanto para el hash
como para las sumas de ensamblaje.

El plan contiene revisiones que cambian decisiones anteriores. Para este
ticket prevalecen las decisiones vigentes de Jonathan; el resumen de tickets
está desactualizado en alias, salidas y equivalentes. La especificación
separa decisiones cerradas y contradicciones pendientes.

## Decisión

1. Definir mamut-toml/1 como representación de engine-input/1 convertida
   exclusivamente por engine-cli. format = "mamut-toml/1" y units = "SI"
   son obligatorios. [engine] queda rechazado, sin vía de JSON crudo.
   Las referencias son por id o nombre; GDL y componentes de carga
   conservan sus literales. El mapeo incluye 15 formas directas y 20
   derivadas; la raíz se describe aparte.
2. Mantener la Opción 0: cli-error/1 y cli-describe/1 intactos;
   inputs.format sigue const "json". engine-api, engine-wasm y los
   esquemas engine-input/1 y results/1 no cambian. El versionado de
   mamut-toml se remite a MAM-990.
3. Mantener las identidades enteras 0..4294967295 de nudos, miembros y
   springs en el JSON. Para alias de texto de nudos/barras/springs:
   definido en MAM-993, [ADR-005](ADR-005-alias-name-idmap.md).
   Los demás identificadores son textuales; "" es válido.
4. H1 acepta S01/S1 y 0/00 como ids distintos; cmp_natural desempata
   por bytes completos. ENG-E-0002 solo representa el mismo id exacto
   repetido, nunca equivalencia de corridas numéricas.
   H2 limita cada id a 256 bytes, con error sin eco; la ordenación usa
   clave precalculada mediante sort_by_cached_key. Solo cuentan dígitos
   ASCII, con u8::is_ascii_digit, sin char::is_numeric ni conversión a entero.
5. El export usa orden de secciones constante; nudos, barras, apoyos,
   masas y resortes por id natural; secciones y leyes por nombre, por
   bytes; properties y parameters por claves, por bytes. Conserva el
   orden documental de casos, cargas y espectros. La especificación
   detalla las demás colecciones y los 36 casos del orden natural.
   U3 establece dos clases de hash distintas: {4, 4.0, 4e0} y
   {0, 0.0, -0.0}, con igualdad dentro de cada clase y diferencia entre
   clases, según la atribución de la ronda 3 registrada más abajo.
6. Aplicar ALLOWED por claves literales, sin concatenar rutas con punto.
   P1 reserva kind, points y mode frente a parámetros de ley; P2 no
   amplía los tipos de apoyo; P3 mantiene los sufijos de unidad del
   esquema. El id de apoyo omitido es `N<nodo>`; una colisión con uno
   escrito se rechaza sin renombrar ni revelar contenido.
7. Comprobar 8 MiB antes de parsear y profundidad 64 justo después.
   Rechazar duplicados, claves desconocidas, referencias ausentes,
   null, fechas/horas, NaN, infinitos y enteros fuera de ±(2^53−1).
   Datetime/fechas y horas TOML producen CLI-E-VALIDATION/2;
   NaN/±inf y demás no finitos, ENG-E-0008/2 en `run` y ENG-E-0009/2
   en `validate`, como en JSON,
   también al exportar.
   Un float entero solo puede emitirse como Integer dentro de ese rango.
   Comprobar intersección de propiedades de sección y parámetros de
   ley por miembro antes del motor.
8. Producir diagnósticos sin eco con message() saneado y span();
   nunca usar Display, to_string() o Debug del error TOML.
   Escapar punteros según RFC 6901 y limitar sus claves a 64 bytes
   sin romper UTF-8. Sanear también cause/message de ENG-E-0002
   procedente del motor cuando la entrada sea TOML.
9. Reutilizar códigos: documento inválido → CLI-E-DOCUMENT/2;
   NaN/±inf y demás no finitos → ENG-E-0008/2 en `run` y
   ENG-E-0009/2 en `validate`, como en JSON;
   datetime/fechas y horas TOML → CLI-E-VALIDATION/2;
   reglas o colisión de alias → CLI-E-VALIDATION/2;
   flags → CLI-E-USAGE/2; versión no soportada →
   CLI-E-UNSUPPORTED/3; archivo de entrada TOML que falta, error de lectura,
   UTF-8 inválido o tamaño excedido → CLI-E-DOCUMENT/2, igual que JSON;
   E/S interna o de salida y fallo interno → CLI-E-INTERNAL/70, según
   la decisión de Jonathan sobre U8 registrada más abajo.
10. La dependencia prevista es toml =1.1.8+spec-1.1.0,
    default-features=false y capacidades std, serde, parse, display,
    sin unbounded; solo en engine-cli. Su incorporación corresponde
    a MAM-938 y Jonathan edita Cargo. T1 no cambia dependencias.

### Decisiones del consejo · ronda 2

Cada fila queda registrada como **decidido por el consejo (delegación de Jonathan), 2026-10-31, pendiente de revisión de Jonathan**.
Estos puntos salen del registro DESCONOCIDO; se conservan los identificadores
de los demás puntos abiertos.

| Punto cerrado | Decisión |
| --- | --- |
| (b), U2 | Recuento: 32 códigos en total, 14 ENG-E, 9 CLI-E y 9 API (4 API-E, 4 API-W, 1 API-I); 27 son `*-E-*`. Comprobado en error-codes/1 (docs/contracts/error-codes-1.json) en la base actual 4fcde46d45ccff13c2a89f23123720d361d9fe96, con el mismo resultado. |
| (a), U1 | NaN/±inf y demás números no finitos → en `run`, ENG-E-0008, salida 2, como en JSON; en `validate`, diagnóstico ENG-E-0009 en validate-report/1, salida 2, como en JSON (engine-cli, lib.rs, validate_input). Datetime/fechas y horas TOML → CLI-E-VALIDATION, salida 2. |
| (d), U4 | vendor/ existe hoy; vendor-crates/ es futuro. No se crea ni se cambia ninguno en esta entrega. |
| (f), U6 | MAM-993 usa [ADR-005](ADR-005-alias-name-idmap.md); este documento arquitectónico es ADR-006. No hay choque. |

### Decisiones del consejo · ronda 3

Atribución de la decisión: **Decisión del consejo (delegada por Jonathan, relatada por J0, 2026-10-31), pendiente de revisión de Jonathan**.

| Punto | Decisión |
| --- | --- |
| U3 | Dos clases de hash distintas: {4, 4.0, 4e0} y {0, 0.0, -0.0}. Las representaciones de cada clase comparten hash; las dos clases tienen hashes distintos. |

U3 sale del registro de puntos abiertos.

### Decisión de Jonathan · U8

Atribución: **decisión de Jonathan del 2026-10-31, relayada por J0, opción A**.

| Punto | Decisión |
| --- | --- |
| U8 | En TOML, un archivo de entrada que falta, un error de lectura, UTF-8 inválido o un tamaño excedido producen CLI-E-DOCUMENT, salida 2, invalid_input, igual que el código actual y JSON. CLI-E-INTERNAL, salida 70, internal, queda solo para error de E/S interna o de salida (por ejemplo, fallo al escribir la salida) y fallo interno. |

U8 sale del registro de puntos abiertos. No hay cambio de código: la regla
coincide con el comportamiento actual de main para JSON.

## Alternativas consideradas

| Alternativa | Evaluación |
| --- | --- |
| Mantener exclusivamente la escritura JSON | Conserva una única representación, pero no atiende la necesidad de edición manual con tablas y formas cortas. |
| Bloque [engine] con JSON crudo o espejo del contrato | Rechazado por Jonathan: elude el diseño de piezas TOML y duplica vías de entrada. |
| Incluir el lector TOML en engine-api o engine-wasm | Rechazado: amplía dependencias y fronteras ajenas al CLI. |
| Cambiar cli-error/1 y cli-describe/1 para anunciar TOML | No elegido: se mantiene Opción 0 y inputs.format const "json". |
| Ordenar todos los nombres por bytes | No elegido para ids textuales: Jonathan decidió orden natural; se conserva por bytes para nombres de secciones/leyes y claves de parámetros. |
| Rechazar ids equivalentes como S01/S1 | No elegido: H1 exige aceptación y desempate por bytes; ENG-E-0002 no significa equivalencia numérica. |
| Añadir tipos de apoyo, subtabla de parámetros o sufijos de unidades nuevos | Fuera de la primera entrega; se mantienen P1, P2 y P3. |

## Consecuencias y verificación

La representación añade un segundo contrato de entrada en el CLI. Una
tabla exhaustiva y controles de cobertura en ambos sentidos deben detectar
campos del esquema sin forma TOML y formas TOML sin correspondencia.
Las siete ramas de análisis conservan sus campos obligatorios y el control
explícito de métodos experimentales.

El export reordena arrays. El hash de TOML es el del JSON convertido y no
se promete igualdad con el JSON fuente. B2: el ensamblaje suma en orden de
entrada; reordenar contribuciones puede afectar redondeo, resultados y
diagnósticos. Las pruebas deben comparar bytes repetidos de una misma
entrada y estudiar por separado diferencias numéricas entre órdenes.

La cota de 256 bytes y las reservas de parámetros restringen ciertos
documentos que el esquema JSON no limita así. La cobertura de todas las
definiciones no equivale a exportabilidad universal de cualquier JSON.
La ordenación evita desbordamientos de enteros y retokenización repetida,
pero no se publican cifras de rendimiento sin medir.

Esta entrega contiene solo documentación. No se ha escrito código Rust,
ejecutado Cargo, alterado contratos ni dependencias, ni realizado pruebas
funcionales del conversor. La aceptación posterior requiere cobertura del
mapeo, ALLOWED, las 36 filas de orden natural, límites, redondeo y
diagnósticos con centinelas, según la especificación.

## Puntos abiertos

Salvo U11, siguen en **DESCONOCIDO — pendiente de decisión de Jonathan**.
El registro detallado y la evidencia documental están en la sección 10
de la especificación:

- U5: definido en MAM-993, [ADR-005](ADR-005-alias-name-idmap.md).
- U7: prueba de 4096 dígitos frente al límite de 256 bytes por id.
- U9: detalles completos de escritura textual y extracción de defaults.
- U10: listas exactas de GDL para expansión de type según dimensión.
- U11: cerrado en `docs/V1.md`, sección 13. La raíz es el nivel 1. Línea y columna, si hay posición, empiezan en 1.
- Rendimiento y umbral de aceptación para documentos de 8 MiB.
