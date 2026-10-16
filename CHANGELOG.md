# Changelog

## Unreleased

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
