# Clean-room log · ENG-CORE-014

- Ticket / PR: MAM-1045 · ENG-CORE-014. Errores de referencia con la entidad ausente.
- Author(s): agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO, 2026-11-10 (America/Guayaquil).
- Role: implementer
- Contract implemented: contexto de `ENG-E-0003` en el diagnóstico, en `results/1`,
  `validate-report/1` y `cli-error/1`. El código estable no cambia.
  `docs/contracts/error-codes-1.json` sigue en 32 entradas.
- Permitted sources used: ficha `docs/tickets/12-errores-de-referencia.md`;
  catálogo y mensajes ya publicados en este repositorio; esquemas `results/1`,
  `validate-report/1` y `cli-error/1`; fachada Python del propio binding.
  No hay cita bibliográfica nueva. No se inventa una.
- Own derivations: la distancia de Levenshtein y el corte `max(1, longitud / 2)`
  descritos en la ficha. No hay magnitud ni sistema de unidades en el mensaje.
- Third-party code copied: none
- Validation: tests del modelo, uno por nudo, sección, ley, caso, apoyo y carga;
  un documento `results/1` de referencia rota comprobado contra su esquema;
  atributos Python `entity_kind`, `missing_id` y `referenced_by`.
- Comparison with oracle (sibling repo, informative): no
- Declaration: "No consulté código ni salidas de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK para este ticket."
- Session: no se abrió provenance/ENG-BENCH-001-SMOKE-VPS.md, la carpeta de auditoría restringida ni registros de Codex.
- Clean-room reviewer: PENDING

## Decisión de idioma

El catálogo traduce `diagnostic.ENG-E-0003` solo cuando la causa es la frase
exacta. La causa con el id, el tipo y la cita no coincide, así que queda en
inglés también con `lang=es` y `lang=pt`. `explain("ENG-E-0003")` sigue
traducido porque esa entrada sí está en el catálogo.

## Decisión de esquema

Las propiedades `entity_kind`, `missing_id`, `referenced_by` y `suggestions`
son opcionales y no entran en `required`. Un documento anterior, sin esas
claves, sigue validando. Las claves se omiten cuando no hay referencia rota,
de modo que un `results/1` de cálculo correcto no cambia de bytes. No se abre
una versión mayor.

No se regeneraron las huellas de determinismo de `docs/DETERMINISM.md` ni
`crates/engine-api/fixtures/results/fingerprints.json`.

El inventario `schemas/index.json` sí cambia, porque el documento del esquema
ganó esas propiedades opcionales:

| Esquema | Bytes antes | SHA-256 antes | Bytes | SHA-256 |
| --- | --- | --- | --- | --- |
| `results/1` | 16350 | `f82f5b783a1ede2e8d8028ced5255a8b0c440a20bb8591d9ae635a99d68995a0` | 17005 | `476a2ec74504c03f668ae99208ad9a4baa18b3dcee3c0a5b6f05ba7dd49ccd4a` |
| `validate-report/1` | 1484 | `caba2efc40d6584141ca8a85582ac9ca5f75e942fc4accf34ceee84f414dc14c` | 2147 | `9ff3ec9c09b0d5cbb5e903913ed26e261cc8afd2f9bf28281fca8561c5e945cc` |
| `cli-error/1` | 4659 | `9662d98f5bb5bdd5bbd4851a517c29a938a4936d700a62a4d6e154e8f7223eff` | 5514 | `f54fdaaa4014d648163b160372b9968e2a108f76c72fe017d1ae0114c25bb567` |

`mamut-engine describe --json` publica ese inventario. Por eso
`crates/engine-cli/tests/snapshots/describe-1.stdout.sha256` pasa de
`7a76a5bbfe58cb284e4cf6c9f64e8da4eb3c3854e1e582a6d7bfded3c358d049` a
`5fe8373ba21e5e02084b539fa3b20869caaa57937982a5fdb9ffc40ed99c95a4`.
