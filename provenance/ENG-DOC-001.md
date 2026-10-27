# ENG-DOC-001 · MAM-870 · Guía «CLI para agentes» y ejemplos ejecutables

Fecha: **2026-10-23**. Trabajo local sin red, rama `j2/MAM-870-guia-cli-agentes`
creada desde `origin/main` (`00189ad`, que incluye #205, #208, #210, #217 y #218).

## Alcance

- [GUIA_CLI_AGENTES.md](../docs/GUIA_CLI_AGENTES.md): receta de tuberías,
  tabla de códigos de salida, manejo de errores y límites, escrita solo con lo
  que hay en `main`. No documenta `--lang` ni el catálogo i18n de #219 (MAM-836,
  abierto): el ticket no se da por dependiente de él. Si #219 se fusiona, la guía
  y su prueba siguen válidas salvo cambios de texto en `describe --json`, que la
  prueba compara como subconjunto.
- [two-storey-frame.json](../docs/examples/agent-cli/two-storey-frame.json):
  pórtico plano D2 de dos pisos (SI, 6 nodos, 6 barras, caso `wind`) construido
  para este ticket; sin datos externos.
- [agent_guide.rs](../crates/engine-cli/tests/agent_guide.rs): cinco pruebas.
- Un enlace desde el README de engine-cli (sección «Agent discovery and API
  queries»). No se tocan `src/`, contratos, esquemas ni golden.

## Qué se verifica en CI

`cargo test -p engine-cli --test agent_guide` hace lo siguiente con el binario
real:

1. Ejecuta los 17 bloques `agent-example` de la guía en orden, con un único
   `TMPDIR` compartido, y compara código de salida y stdout exactos con el bloque
   `agent-output` de mismo id. Usa solo `sh`, utilidades POSIX y `python3`.
2. Comprueba que los enlaces relativos y las anclas internas resuelven.
3. Compara la tabla de códigos de salida con `exitCodes` de `describe --json` y
   la tabla de códigos `CLI-E-*` con `docs/contracts/error-codes-1.json`, en
   ambos sentidos.
4. Compara la tabla de límites con `limits` de `describe --json`.
5. Ejecuta el modelo de ejemplo y contrasta el desplazamiento Ux de la cubierta
   (nodo 5, caso `wind`) con la estimación a mano de edificio de cortante
   (k = 2·12·E·I/h³ por piso; Ux ≈ 5,5827e-3 m) con tolerancia del 1 %. El motor
   da 5,610851e-3 m (0,5 % de diferencia).

La salida de los ejemplos es determinista (mismos bytes con la misma entrada,
versión y objetivo). El ejemplo de timeout usa `--timeout 0.000001` y dio exit 6
en 100 de 100 ejecuciones locales; el de cancelación usa
`MAMUT_ENGINE_CANCEL_FILE` con un archivo preexistente. El comportamiento de
SIGINT/SIGTERM (exit 5, `CLI-E-CANCELLED`) se comprobó a mano con un modelo
pesado y no forma parte de las pruebas por depender de tiempos. Lo mismo ocurre
con el cierre anticipado de la tubería (exit 70), que la guía documenta como
dependiente del momento.

## Prueba de usabilidad registrada

Criterio del ticket: un agente sin contexto completa validar → ejecutar → leer
el desplazamiento de cubierta usando solo `describe --json` y la guía.

- Agente: una sesión nueva de Codex (gpt-6.1-sol, esfuerzo alto, sin red) en un
  directorio de trabajo aparte que contenía únicamente el binario, la guía y el
  modelo de ejemplo; sin acceso a código fuente, README ni esta ficha.
- Resultado: validar → exit 0 (`status: ok`); ejecutar → exit 0; leyó
  `engineResults[0]` → caso `wind` → nodo 5 → `displacements[0]`:
  **+5,610850782776847e-3 m**, igual al valor de la guía. Fricciones declaradas:
  ninguna. Terminó en menos de un minuto.
- Desviaciones y límites: el agente imprimió también el JSON del modelo (es la
  entrada de la tarea) y el prompt le daba la ruta del modelo y el nombre del
  caso. Es una sola sesión; no sustituye una prueba con personas ni con otros
  agentes, y conviene repetirla si la guía cambia de forma sustancial.

## Notas de revisión

- Convención de ejes: el motor fija el plano D2 como X–Y pero no declara Y como
  vertical; la guía lo advierte y el modelo usa Y como cota por convención del
  ejemplo (ver CONVENCION_EJES).
- Conflicto posible con #219: ambos PR tocan `crates/engine-cli/README.md`. Este
  PR añade solo tres líneas bajo «Agent discovery and API queries»; los
  hunks de #219 observados están en otras zonas, por lo que no se espera
  conflicto de texto, pero sí que revisar tras la fusión de cualquiera de los dos.
- Registro de cambios: `changelog.d/MAM-870.added.md`.
