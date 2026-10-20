# Changelog

## Unreleased

- MAM-91: helpers de tests del CLI toleran solo BrokenPipe al escribir stdin de hijos que terminan antes de leerlo; se mantienen los asserts de salida y stderr, sin cambios del CLI.
- MAM-79 / #143: engine-api rechaza literales enteros fuera de ±(2^53−1)
  antes de JCS con `InvalidInput` (`API-E-INPUT`, salida 2, huella null).
  Python aplica el mismo rango simétrico antes del FFI y detecta como stale
  mutaciones hacia enteros inseguros. Regresiones de fronteras y de bool como
  entero; sin cambios de vendor, cálculos ni los 16 hashes de determinismo.
- MAM-91 / #146: pruebas del CLI con EOF para aceptar exactamente MAX_INPUT_BYTES y rechazar MAX+1 como Document/2; mutación del límite verificada y bitácora sin ruta local de compilación.
- MAM-22 / #142: exige exactamente cinco fixtures JSON, elimina rutas locales de la documentación de engine-validation y explicita el alcance de los agentes dentro de su worktree.
- MAM-91 / #141: regresiones del CLI para lectura acotada a MAX+1 y mapeo de ENG-E-0013 en validate a salida 3, verificadas mediante mutaciones.
- MAM-79 / ENG-PY-003: fachada Python pública tipada y documentada sobre
  engine-api, con modelos/resultados versionados, errores completos, controles
  SI/ejes y stale mediante la huella JCS de Rust. Completa procedencia,
  descriptores y convergencia P-Delta del main actual; prueba P-Delta Stable sin
  consentimiento experimental. Incluye regresiones de compatibilidad, stubs y
  comparación de la wheel offline con 16 entradas Rust. Sin física en Python.
- MAM-22 · ENG-CORE-009: contrato JSON fixture/1 con source obligatorio y veto
  recursivo de procedencias prohibidas; cinco casos analíticos propios y runner
  en engine-validation con targets engine-api nativo y wasm32/Node sin npm.
  Informes legibles y JSON determinista, tolerancias justificables e interfaz
  Target para añadir el CLI tras ENG-SFC-002. Véase
  [provenance](provenance/ENG-CORE-009.md).

- MAM-91 / ENG-SFC-002: CLI nativo `mamut-engine` sobre `parse_args`/`dispatch`
  de SDK-WF-012, con métodos `engine.run` y `engine.validate`, resultados completos
  y códigos Engine v1 0/1/2/3/4/5/70 probados ejecutando el binario. Entrada y
  cancelación cooperativa por rutas locales configuradas mediante entorno;
  sin lógica numérica ni dependencias externas nuevas. Se documenta la
  correspondencia de operaciones con describe/methods/validate/run y sus límites.
  Binarios Windows/Linux/macOS en CI pendientes de Jonathan; workflows intactos.
- MAM-582: pDelta pasa a `Stable` y se habilita por defecto, según
  [ENG-BENCH-002 (PR #132)](provenance/ENG-BENCH-002.md): AISC 360-16
  Comm. C2, casos 1 y 2, error ≤0.27% con tolerancias 3% M / 5% Δ. Se conserva `options.allow_experimental` para futuros métodos
  Experimental y su gate tiene cobertura unitaria.
- Se añade Stable al esquema method-descriptor/1; se actualizan pruebas,
  documentación y tres fixtures P-Delta sin consentimiento experimental.
  Se regeneran los dos hashes de cruce afectados por input_sha256 y se verifica
  igualdad nativo/wasm en debug y release. Los cálculos mecánicos no cambian;
  véase la tabla antes/después en [provenance](provenance/ENG-CORE-006.md#mam-582--pdelta-estable--2026-10-15).
- Cerca o por encima del pandeo, P-Delta puede terminar `unstable` o
  `notConverged`: salida 4, sin resultados mecánicos; se conserva el reporte
  de convergencia. El test
  [`pdelta_nonconvergence_and_instability_return_four_with_iteration_report`](crates/engine-api/tests/dynamic.rs)
  comprueba no convergencia al limitar a una iteración (P/Pcr=0,2) e
  inestabilidad por encima del pandeo (P/Pcr=1,2), ambos con salida 4.
