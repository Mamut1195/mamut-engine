# Clean-room log · ENG-CORE-012

- Ticket / PR: MAM-1043 (ENG-CORE-012), fase 1. Catálogo, tabla de conversión
  y documentación. Sin cambio de contrato. El PR queda en borrador.
- Author(s): agente cloud de Cursor (Cursor Agent), modelo: DESCONOCIDO · 2026-11-10 (ECT, America/Guayaquil)
- Role: specifier and implementer
- Contract implemented: ninguno. `engine-input/1` sigue con `units` igual a
  la constante `"SI"`. La tabla nueva es `units-table/1` en
  `crates/engine-core/src/units/catalog.rs`. Los presets son un tipo de Rust
  y no entran en un esquema.
- Permitted sources used: las citas escritas en `docs/UNITS.md`. Aviso del
  National Bureau of Standards en el Federal Register, vol. 24, 1 de julio
  de 1959, p. 5348, F.R. Doc. 59-5442. Declaración 2 de la 3.ª CGPM (1901),
  p. 70, DOI 10.59161/CGPM1901DECL2E. Brochure del SI, 9.ª edición, 2019,
  DOI 10.59161/AUEZ1291, sección del kelvin y tablas 7 y 8. La página externa
  de la escala Fahrenheit y la de la palabra kip quedan como cita desconocida.
  No se copió un listado de un libro ni un PDF al repositorio.
- Own derivations: los factores compuestos son productos de las anclas. El
  paso de 1/°F a 1/K es 9/5 para conservar ε = α ΔT. Están en la ficha
  `docs/tickets/13-unidades-ee-uu.md` y en `docs/UNITS.md`.
- Third-party code copied: none
- Validation: pruebas propias del catálogo. Ida y vuelta de cada unidad con
  cota de error relativo 1e-15, magnitudes de 1e-6 a 1e9, semilla determinista.
  Unidad desconocida en `ENG-E-0001`, con el texto recibido en el mensaje.
  lbm con lbf sin aceleración declarada en `ENG-E-0010`. `error-codes-1.json`
  sigue en 32 entradas. Los goldens y los digestos de determinismo no se
  editaron.
- Comparison with oracle (sibling repo, informative): no
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, ni sus derivados, para este ticket. No hubo exposición en el código ni en las salidas."
- Session: sin exposición a OpenSees, Frame3DD, Stabileo, CalculiX ni LAPACK, en código y en salidas.
- Clean-room reviewer: PENDING

## Pendiente

F2, F3 y F4 de la ficha. La versión del contrato de entrada la decide
Jonathan; este corte no la toma. La firma clean-room sigue en PENDING.
Una sesión de agente no firma esa línea.
