# Registro de procedencia · ENG-TOML-001

- Ticket: MAM-939 (T1).
- Fecha: 2026-10-31 (America/Guayaquil).
- Autoría: redactado con asistencia de IA, bajo las decisiones de Jonathan.
- Rol: redacción documental; sin código.
- Contrato documentado: mamut-toml/1 como representación TOML de
  engine-input/1 en engine-cli.
- Entregables: [especificación](../docs/formats/mamut-toml-1.md) y
  [ADR-006](../docs/adr/ADR-006-mamut-toml.md), además de este registro.
- Modelo de este registro: ENG-API-001, único documento de procedencia
  ajeno a T1 abierto como modelo.

## Fuentes permitidas

- Esquema propio:
  [crates/engine-api/schemas/engine-input-1.json](../crates/engine-api/schemas/engine-input-1.json),
  fuente de campos, tipos, ramas, límites y valores literales.
- TOML 1.1.0 spec: referencia normativa de sintaxis; no se consultó
  una copia por red durante esta tarea.
- Decisiones vigentes de Jonathan comunicadas para MAM-939: prevalecen
  sobre el plan donde lo contradicen.
- Plan de entrada TOML, secciones v4, v4.1, v4.2 y v4.3; resumen de
  tickets TOML como antecedente desactualizado en alias, salidas y
  ENG-E-0002 para equivalentes.
- Documentación propia complementaria:
  [registro de errores](../docs/contracts/error-codes-1.json),
  [README del CLI](../crates/engine-cli/README.md),
  [guía de agentes](../docs/GUIA_CLI_AGENTES.md) y
  ADR-004 como modelo de estructura.

## Derivaciones y alcance

Se contrastó el mapeo del plan con las 35 definiciones del esquema propio:
15 formas directas y 20 derivadas, con la raíz descrita aparte. Se
documentaron ALLOWED, siete métodos, controles anteriores a engine-api,
orden natural, orden canónico, salidas y diagnósticos sin eco.

La batería de 36 casos procede del plan v4.2, con las decisiones vigentes
H1 y H2. Los casos 7, 8 y 33 pasan a orden por bytes sin error.
Se preserva la advertencia B2: el reordenamiento puede afectar el redondeo
porque el ensamblaje suma en orden de entrada.

- Código de terceros copiado: ninguno.
- Código propio escrito o modificado: ninguno.
- Valores tomados de otros programas: ninguno.
- Comparación con oráculos numéricos externos: ninguna.
- Derivaciones mecánicas o cambios de solver: ninguno.
- Acceso a red: ninguno.
- Cargo, commit y git push: no ejecutados.
- Dependencias: ninguna modificada; la incorporación de toml corresponde
  a MAM-938 y Jonathan.
- Declaración: no se consultó código de adaptadores externos ni
  documentación restringida; no se copiaron implementaciones externas.

## Decisiones del consejo · ronda 2

Cada fila queda registrada como **decidido por el consejo (delegación de Jonathan), 2026-10-31, pendiente de revisión de Jonathan**.
Estos puntos salen del registro DESCONOCIDO; se conservan los identificadores
de los demás puntos abiertos.

| Punto cerrado | Decisión |
| --- | --- |
| (b), U2 | Recuento: 32 códigos en total, 14 ENG-E, 9 CLI-E y 9 API (4 API-E, 4 API-W, 1 API-I); 27 son `*-E-*`. Comprobado en error-codes/1 (docs/contracts/error-codes-1.json) en la base actual 4fcde46d45ccff13c2a89f23123720d361d9fe96, con el mismo resultado. |
| (a), U1 | NaN/±inf y demás números no finitos → en `run`, ENG-E-0008, salida 2, como en JSON; en `validate`, diagnóstico ENG-E-0009 en validate-report/1, salida 2, como en JSON (engine-cli, lib.rs, validate_input). Datetime/fechas y horas TOML → CLI-E-VALIDATION, salida 2. |
| (d), U4 | vendor/ existe hoy; vendor-crates/ es futuro. No se crea ni se cambia ninguno en esta entrega. |
| (f), U6 | MAM-993 usa [ADR-005](../docs/adr/ADR-005-alias-name-idmap.md); este documento arquitectónico es ADR-006. No hay choque. |

En la ronda 2 se comprobó el recuento en el registro propio, la
existencia actual de vendor/ y la clasificación de lectura JSON en el
README de engine-cli. En la r2 de J4 el recuento se contrastó con
4fcde46d45ccff13c2a89f23123720d361d9fe96 (véase U2). Se actualizaron los
controles y la tabla de salidas: fechas/horas → CLI-E-VALIDATION/2; no
finitos → ENG-E-0008/2 en `run`, también al exportar, y ENG-E-0009/2 en
`validate` (engine-cli, lib.rs, validate_input), como en JSON. La discrepancia con JSON de U8 quedó resuelta
por decisión de Jonathan (opción A), registrada más abajo.

## Decisiones del consejo · ronda 3

Atribución de la decisión: **Decisión del consejo (delegada por Jonathan, relatada por J0, 2026-10-31), pendiente de revisión de Jonathan**.

| Punto | Decisión |
| --- | --- |
| U3 | Dos clases de hash distintas: {4, 4.0, 4e0} y {0, 0.0, -0.0}. Las representaciones de cada clase comparten hash; las dos clases tienen hashes distintos. |

U3 sale del registro de puntos abiertos.

En la ronda 3 se verificó el código propio del CLI mediante lectura:
read_engine_path y read_engine_input en lib.rs; clasificación en
error.rs:34; contraste con el README de engine-cli, línea 295.
Esa evidencia sustentó la decisión de U8 registrada a continuación.
Se actualizaron únicamente la especificación, ADR-006 y
este registro; se conservaron los demás puntos DESCONOCIDO.

## Decisión de Jonathan · U8

Atribución: **decisión de Jonathan del 2026-10-31, relayada por J0, opción A**.

| Punto | Decisión |
| --- | --- |
| U8 | En TOML, un archivo de entrada que falta, un error de lectura, UTF-8 inválido o un tamaño excedido producen CLI-E-DOCUMENT, salida 2, invalid_input, igual que el código actual y JSON. CLI-E-INTERNAL, salida 70, internal, queda solo para error de E/S interna o de salida (por ejemplo, fallo al escribir la salida) y fallo interno. |

U8 sale del registro de puntos abiertos. No hay cambio de código: la regla
coincide con el comportamiento actual de main para JSON.

## Validación documental y pendientes

Se revisaron el inventario de definiciones y campos obligatorios,
las listas de claves por rama, la presencia de los 36 casos y las
restricciones de texto y alcance de los tres archivos. Esta revisión
documental no acredita funcionamiento de un lector o exportador TOML.
No se ejecutaron pruebas funcionales ni se recalcularon hashes o
resultados físicos del plan.

Revisión humana integral de esta entrega: pendiente; las decisiones del
consejo de las rondas 2 y 3 y la decisión de Jonathan sobre U8 quedan
registradas arriba.
La especificación y el ADR registran como
**DESCONOCIDO — pendiente de decisión de Jonathan**:

- U5: definido en MAM-993, [ADR-005](../docs/adr/ADR-005-alias-name-idmap.md).
- U7: prueba de 4096 dígitos frente a cota de 256 bytes por id.
- U9: escritura textual completa y extracción de defaults.
- U10: expansión exacta de type de apoyo según dimensión.
- U11: conteo de profundidad y base de línea/columna.
- Rendimiento y umbral de la prueba de 8 MiB.

El versionado de mamut-toml se remite a MAM-990.
