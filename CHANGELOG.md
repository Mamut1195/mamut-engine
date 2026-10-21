# Changelog

## Unreleased

- #185 · ENG-E-0007: P-Δ proyecta la rigidez geométrica, sus fuerzas y el
  residuo mediante la transformación de diafragmas rígidos; la inercia LDLᵀ
  se evalúa en la rigidez total proyectada. Regresiones de restricciones
  exactas y E3-pdelta con referencia NumPy propia reproducible que lee el
  fixture de main, sin duplicarlo ni cambiar los JSON/hashes de VAL-032.
  VAL-032 acepta E3-pdelta ejecutable con aviso #84 justificado y conserva
  los ocho bloqueos modales exactos de #184 / ENG-E-0006.

- MAM-46 · ENG-GEO-003: generadores estáticos `py.matlock_soft_clay`,
  `py.reese_sand` y tabla simétrica `py.user` en engine-geo, puntos (m, N/m),
  parámetros obligatorios y errores tipados; adaptador de tensión efectiva
  del perfil a Pa. Tres fixtures analíticos compartidos nativo/wasm,
  integración dimensional con multilinear_elastic y bitácora clean-room.
  Sin cambios de los análisis existentes ni exposición JSON del futuro pilote.
- MAM-45 · ENG-GEO-004: tablas axiales inmutables `tz.api_clay`,
  `tz.api_sand`, `qz.api`, `tz.user` y `qz.user` en m/Pa/N; parámetros
  obligatorios, tracción explícita, mesetas y errores tipados. Adaptadores
  a la ley multilineal con conversión de unidades/signos y rechazo claro
  del ablandamiento que esa ley no admite. Fixtures API RP 2GEO y analíticos
  compartidos en nativo/wasm, sin nuevas dependencias. Coyle–Reese queda
  solo vía tablas del usuario, pendiente de la fuente original.
  Seguimiento J4 r1: `Explicit` valida el signo de resistencia en ambas ramas.
- MAM-855 / ENG-VAL-006: incorpora Frame3DD a la comparación externa de
  §5.1 del protocolo clean-room y formaliza la opción A: Q2 opera en sesión
  aislada, J4 audita y J2/J0 solo reciben discrepancias sin valores.
  Seguimiento de revisión: alinea §2/§3 y exige versión y hash por campaña,
  con nuevo registro y auditoría de J4 al cambiar de versión.
  Seguimiento J4 r1 (#178): precisa aprobación directa y antecedentes,
  alcance de la opción A, acceso a `results/` y declaración de la plantilla.

- MAM-82 · #162: quitar rutas locales de provenance ENG-PY-004 y anotar licencia BSD-3-Clause de NumPy en ADR-004

- MAM-82 / ENG-PY-004 · seguimiento #156: conserva la causa al faltar NumPy,
  explicita el rechazo de bool numérico y comprueba la independencia de las
  exportaciones. Pruebas NumPy migradas a unittest con skip opcional para el
  descubrimiento oficial de la wheel.

- MAM-82 / ENG-PY-004: exportación tabular de resultados con listas o copias
  NumPy float64 C, IDs/GDL/unidades/estado y extra opcional de importación
  perezosa. ADR-004 y benchmark offline Rust/CLI/FFI/fachada/copias, sin nuevas
  dependencias Cargo, física Python ni cambios del GIL o resultados del motor.

- MAM-105 · ENG-VAL-005, seguimiento PR #155: fuentes primarias propias con
  anclas verificadas, bibliografía por cotejar separada de la aceptación y
  cobertura calculada desde las tablas/fixtures, con diagnóstico de diferencias.
  Pruebas negativas de citas pendientes y justificaciones vacías; justificación
  específica de los cinco defaults fixture/1, sin cambios numéricos.
  [Provenance](provenance/ENG-VAL-005.md#seguimiento-de-revisión-opencode--pr-155).

- MAM-105 · ENG-VAL-005: matriz y guardia de oráculos libres para los 91 casos
  V1 de engine-validation, incluidos Winkler, consolidación y fixture/1.
  Tolerancias existentes explícitas, sin cambios de valores numéricos;
  ENG-REL-001 depende de ENG-VAL-005 y no exige ENG-VAL-003. OpenSees solo
  como contraste externo adicional, sin incorporar archivos ni resultados.
  [Provenance](provenance/ENG-VAL-005.md).

- MAM-25 · ENG-CORE-011: guía en español para contribuir métodos Rust,
  plantillas de ficha, módulo, conformidad, fixtures, prompt IA restringido,
  provenance y revisión. Piloto `elastic.cubic` con derivación energética propia,
  fábrica/descriptor uniaxial/1 y benchmark analítico nativo; sin modificar
  métodos existentes ni los 16 hashes de determinismo.
- MAM-91 / MAM-79 · #152 (2026-10-20): rutas locales de ENG-SFC-002 y ENG-PY-003 sustituidas por marcadores genéricos; solo documentación.
- MAM-79 / #144: PyO3 y sus cuatro crates auxiliares pasan de 0.25.1 a
  0.29.3 para corregir RUSTSEC-2026-0176 y RUSTSEC-2026-0177. El adaptador
  usa `Python::detach` y conserva la API Python y la feature `abi3-py310`;
  actualización y validación de wheels reproducibles exclusivamente offline.
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
