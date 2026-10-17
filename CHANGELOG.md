# Changelog

## Unreleased

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
