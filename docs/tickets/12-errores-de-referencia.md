# 12 · Errores de referencia con la entidad que falta

- Estado: abierto. Puede empezar ya, en paralelo.
- Ticket: MAM-1045. Bitácora: `provenance/ENG-CORE-014.md`.
- Código estable: `ENG-E-0003` no cambia. `docs/contracts/error-codes-1.json` sigue en 32 entradas. Solo se añade contexto.
- Reglas comunes: [README.md](README.md).

## Qué construir

Cuando una relación apunta a una entidad que no está en el modelo, el diagnóstico
`ENG-E-0003` nombra esa entidad y el sitio que la cita.

- El mensaje y los campos dicen el tipo y el id buscados.
- También dicen quién cita: tipo, id y campo. El camino usa el índice 0 de la
  colección del modelo, por ejemplo `members[3].nodes[1]`. El 3 es la posición
  de la barra, no su id. El 1 es el extremo.
- Si hay ids del mismo tipo suficientemente parecidos, el mensaje y
  `suggestions` los listan. El algoritmo de abajo es determinista y no depende
  del idioma ni del orden de un mapa.
- Rust, el JSON de error, `results/1` y la excepción Python exponen
  `entity_kind`, `missing_id` y `referenced_by`. Python también expone
  `suggestions` como lista, vacía cuando no hay sugerencia.

Los campos nuevos de un esquema versionado son opcionales y no entran en
`required`. `additionalProperties` sigue en false. Un documento ya publicado,
sin esas claves, sigue validando. Es el mismo criterio que `git_commit` y
`target` en `results/1`. Las claves solo aparecen en un diagnóstico de
referencia. Un resultado sin ese diagnóstico no cambia de bytes.

El mismo diagnóstico viaja en `validate-report/1` y en `cli-error/1`. Esos
esquemas reciben las mismas propiedades opcionales. No se abre una versión
mayor.

## Ids parecidos

1. Cada id se escribe en texto canónico. Un entero usa decimal, sin ceros a la
   izquierda. Un nombre es la secuencia exacta de escalares Unicode, sin
   recortar ni cambiar mayúsculas.
2. La distancia es Levenshtein sobre esos escalares. Insertar, borrar o
   sustituir cuesta 1. Una transposición no es una operación distinta: `ab`
   frente a `ba` cuesta 2.
3. Un id existente entra si la distancia es mayor que 0 y no supera
   `max(1, longitud / 2)`, con división entera de la longitud del id buscado.
4. El orden es distancia creciente y, a igual distancia, el orden de los bytes
   UTF-8 del texto canónico. El mismo texto se lista una vez.
5. Como máximo tres. Si ninguno entra, no hay sugerencia y el JSON omite
   `suggestions`.

## Idioma

El catálogo traduce `diagnostic.ENG-E-0003` solo cuando la causa es exactamente
la frase registrada. La causa con el contexto no coincide con esa frase, así
que queda en inglés en `en`, `es` y `pt`. No se inventa una plantilla fuera
del catálogo.

`explain("ENG-E-0003")` sigue en inglés, español y portugués, porque esa
explicación sí está en el catálogo. El mensaje enriquecido no nombra
magnitudes ni supone un sistema de unidades.

## Hecho cuando

- Una barra con un nudo ausente produce un mensaje que incluye `node`, el id
  ausente, el id de la barra y el índice del extremo.
- Hay un test para sección, ley, caso, apoyo y carga.
- En Python, `entity_kind`, `missing_id` y `referenced_by` van anotados.
- Hay fragmento de changelog. El revisor clean-room queda en PENDING.
