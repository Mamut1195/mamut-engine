# Fragmentos del changelog

Cada PR añade uno o más archivos `<MAM-ID>.<type>.md`, por ejemplo
`MAM-861.added.md`. El ID tiene el formato `MAM-` seguido de dígitos.
Los tipos permitidos, en orden de publicación, son:

| Tipo | Sección |
| --- | --- |
| `added` | Añadido |
| `changed` | Cambiado |
| `fixed` | Corregido |
| `validation` | Validación |
| `security` | Seguridad |

El contenido es la entrada Markdown tal como debe publicarse, en una o más
líneas; no debe estar vacío ni contener solo espacios. Ejemplo:

```markdown
- MAM-861: permite preparar versiones con fragmentos por PR.
  Conserva el orden y las entradas anteriores del changelog.
```

Los fragmentos no deben contener encabezados Markdown: una línea `## x`
se publica como `- ## x`, no como un encabezado de sección.

El ensamblador conserva el contenido, añade `- ` si la primera línea no lo
tiene e indenta con dos espacios las continuaciones sin sangría. Conserva las
líneas vacías y las sangrías existentes. Ordena por tipo y por ID numérico
(desempate lexicográfico). Solo ignora `README.md`; no añadas `.gitkeep`.

Desde la raíz del repositorio:

```sh
bash scripts/changelog-assemble.sh --dry-run 0.3.0 2026-10-21
bash scripts/changelog-assemble.sh 0.3.0 2026-10-21
```

`VERSION` es un identificador sin espacios (letras, dígitos, `.`, `+`, `_`, `-`).
La fecha es `AAAA-MM-DD`; si se omite, es obligatorio definir
`SOURCE_DATE_EPOCH` (segundos Unix no negativos), convertido a fecha UTC.
Nunca se usa la fecha actual implícitamente. Una versión ya publicada se rechaza.

La nueva sección se inserta inmediatamente después de `## Unreleased`, que se
conserva vacío. Las entradas anteriores de Unreleased pasan a la nueva versión,
después de los fragmentos, bajo `### Otros (legado)`. Las versiones anteriores
se conservan. Tras escribir el changelog se eliminan los fragmentos con `rm`;
revisa y añade esas eliminaciones al commit de release. `--dry-run` solo imprime
el resultado a stdout y conserva los archivos.
