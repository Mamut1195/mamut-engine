# Comprobación clean-room

Ejecutar desde el repo: `bash clean-room/check.sh <base-del-PR>`.
Compara desde el ancestro común de la base y `HEAD` hasta el árbol de trabajo;
en CI el checkout corresponde al HEAD del PR. Incluye cambios locales de archivos
conocidos por Git; para archivos nuevos usar `git add -N` antes de probar.

Las rutas son globs de Bash relativos a la raíz (`*` incluye `/`), derivados
solo de `docs/CLEAN_ROOM.md` §5. Las ubicaciones no especificadas quedan como
comentarios `PENDIENTE/DESCONOCIDO` y no se comprueban automáticamente.
Los identificadores proceden solo de la lista inicial de §6: coincidencia exacta,
sensible a mayúsculas, en líneas agregadas, con límites de palabra ASCII
(`A-Z`, `a-z`, `0-9`, `_`). Se excluyen `clean-room/*`, `docs/CLEAN_ROOM.md`
y `provenance/*` de esa comprobación. Los renombres se tratan como baja y alta;
los binarios no aportan líneas de texto para comprobar identificadores.

Cualquier cambio en `crates/`, incluidas bajas, requiere un
`provenance/ENG-*.md` añadido o modificado; `provenance/TEMPLATE.md` no cuenta.
El script devuelve 0 si pasa, 1 ante infracciones y otro código ante errores.

## Seguridad del CI

El workflow corre con `pull_request` y `contents: read`, con acciones fijadas por SHA.
El validador (`check.sh`) y las listas se toman de la rama **base** (carpeta
`trusted-base`), no del PR: un PR no puede vaciar las listas para pasar. Listas vacías
o un glob que abarque todo el repo (`*`, `/*`) dan error (código 2); un `/` inicial
ancla el patrón a la raíz. El job `protected-review` falla si el PR modifica
`clean-room/**`, `.github/workflows/**`, `.github/CODEOWNERS` o `docs/CLEAN_ROOM.md`
sin aprobación vigente de `@Mamut1195` (ver `.github/CODEOWNERS`). Pruebas del
validador: `bash clean-room/tests/run.sh`.

## Salidas de oráculos (cierra #173 N2)

**La regla es CLEAN_ROOM.md §5.1** (opción A): las salidas de OpenSees, Frame3DD y Stabileo
viven solo en `results/` de mamut-engine-oracles, y solo las leen Q2 (operador) y J4; J2, J0 y
las sesiones autoras no ven los valores. Esta sección solo describe el chequeo; si difiere de
§5.1, manda §5.1.

Regla propia del chequeo (no está en §5.1): en este repo, de una corrida de oráculo solo puede
quedar "humo ok" más un puntero a mamut-engine-oracles; nada de entradas ni salidas.

`check.sh` revisa las rutas agregadas o modificadas del PR (archivo completo).
`bash clean-room/check.sh --all` barre todo el árbol rastreado solo con esta regla.

- **Rutas:** `*.tcl`, `*.tcl.*`, `*.3dd`; `opensees*.csv`, `opensees*.csv.*` (p. ej. `.csv.gz`),
  `opensees*.txt`, `opensees*.json`, `opensees*.out`; `frame3dd*.out|.plt`, `f3dd*.out|.plt` y
  `*.out`/`*.plt` bajo carpetas `*frame3dd*` o `*f3dd*`; `results/` en la raíz y `results/` bajo una
  carpeta de oráculo. Sin distinguir mayúsculas. `crates/**/fixtures/results/` del Engine se permite.
- **Contenido de `provenance/` y `docs/`:** una línea está en contexto de oráculo si ella o
  **cualquiera de sus encabezados Markdown ancestros** (pila por nivel) nombra OpenSees, Open Sees,
  Frame3DD, F3DD, Stabileo u `OS` (en mayúsculas, como palabra). Las líneas `#` dentro de bloques
  de código (```/~~~) no son encabezados. En contexto, se marca la línea si tiene:
  - una asignación con número (`u = …`, `E = …`, `P = …`): entradas o salidas del modelo;
  - una palabra de salida o entrada (obtenido, salida, output, resultado, result, entrada, input)
    y una cifra decimal o con exponente, también con coma decimal o seguida de `.`, `,`, `;`, `:`
    al final de la oración;
  - una fila de tabla (`| … |`) con una cifra decimal o con exponente.
- **Metadatos permitidos** (se quitan antes de buscar cifras): tokens hexadecimales delimitados de
  32 o más caracteres, o de 7 o más con alguna letra a–f (sha256, commits); una mantisa decimal no
  se quita. También versiones `x.y.z`, fechas, horas y referencias `§n.m`.
- **Tolerancias (exención estrecha):** la palabra `tol`, `rtol`, `atol`, `toleranc…`, `umbral`
  o `criterio` cuenta solo como palabra completa en la **misma línea** (nunca en encabezados;
  "Toledo" no exime). Entonces se quita solo `rtol|atol|tol` seguido de `=`, `:`, `<=` o `≤` y un
  valor X ≤ 1e-2; "≤ X" o "X %" sin identificador tol no eximen. El resto de la línea se sigue
  revisando, y una línea con palabra de salida nunca queda exenta.
- **Tablas:** el texto de la fila de encabezado da contexto de oráculo a sus filas. En las columnas
  cuyo encabezado es tol, rtol, atol, toleranc… o criterio solo se quita un valor admisible
  (≤ 1e-2 o ≤ 5 %); cualquier otro valor de la celda se sigue revisando.
- **Asignación:** `=`, `:=`, `:`, `≤`, `≥`, `<`, `>`, `≈` y `~` entre un identificador y un número.
- **Palabras de salida** con límite de inicio de palabra; `_` cuenta como límite ("u_obtenido" sí, "concentrada" no).
- **CI:** con `CI=true`, la falta de `43b5f45` en el historial (clon superficial) es fallo duro.
- **Mensajes:** solo ruta y línea; nunca el contenido, para no copiar cifras en logs de CI.
- **Tests:** usan cifras sintéticas. El caso del incidente lee el archivo de `43b5f45` desde el
  historial de git en tiempo de prueba, sin copiarlo al repo; si el historial no está, se avisa y
  queda la réplica sintética.
