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
