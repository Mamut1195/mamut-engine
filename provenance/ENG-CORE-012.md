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

## Fase 2 · 2026-11-10

Jonathan, 01:12 ECT, a través de J0: «Apruebo que MAM-1043 cambie el contrato
de entrada para declarar unidades SI o EE. UU.»

El cambio no es aditivo. `units` pasa de constante `"SI"` a la enumeración
`"SI"` | `"US"`, así que MAM-990 pide `engine-input/2`. La versión 1 no se
reescribe. La migración está en `docs/migration/engine-input-2.md`.
`"US"` en esta fase es solo kip-in. kip-ft, lb-in, lb-ft y los demás
presets quedan para otro ticket. La fachada convierte a SI antes del núcleo.
Un texto distinto de `"SI"` y `"US"` en la versión 2 es `ENG-E-0001` y nombra
el texto recibido. Si falta `units`, el rechazo nombra el campo `units`.
`error-codes-1.json` sigue en 32 entradas. `engine-input/1` sigue validando.

La equivalencia US frente a SI no promete los mismos bits. La prueba de la
cercha usa tolerancia relativa 1e-12. En esa cercha los resultados
coincidieron salvo `input_sha256`; es una observación, no el contrato.

El digesto de `describe --json` pasa de
`7a76a5bbfe58cb284e4cf6c9f64e8da4eb3c3854e1e582a6d7bfded3c358d049` a
`201d3d0ba755bd39bccc5ebba5bfbdc268ca3af5de018a57ad5862588f7cd0c8`.
La causa es que entra `engine-input/2` en el catálogo de esquemas y en
`capabilities`. Los 16 digestos de determinismo no cambian.
`schemas/engine-input/1.json` no cambia. El digesto nuevo es provisional:
#28 (MAM-1045) se fusiona antes y también cambia esa instantánea; después
este PR integra main y la regenera.

## Pendiente

F3 y F4 de la ficha. La firma clean-room sigue en PENDING.
Una sesión de agente no firma esa línea. La aceptación final de esta F2
la dan J0, después de la auditoría de J4, y Jonathan. El PR queda en borrador.
