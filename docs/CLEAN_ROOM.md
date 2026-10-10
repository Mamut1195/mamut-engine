# Protocolo clean-room del Engine (ENG-GOV-002)

- **Versión:** 0.1 (borrador para aprobación de Jonathan)
- **Ticket:** ENG-GOV-002 · **Depende de:** ENG-GOV-001
- **Nombre del proyecto:** provisional (DESCONOCIDO el definitivo)
- **Aviso:** este documento es un protocolo operativo, no asesoría legal. El aviso legal y la postura de licencia viven solo en Notion (ENG-GOV-004). Los puntos marcados **PENDIENTE** requieren decisión o revisión externa.

## 1. Objetivo

Que cada línea del motor se pueda rastrear hasta una fuente permitida (libro, paper o derivación propia), con evidencia escrita. El motor se escribe desde cero en Rust.

## 2. Regla de oro

Quien escribe código o especificaciones del motor **no abre** el código fuente de OpenSees, de Stabileo ni de ningún derivado. OpenSees solo se **ejecuta** como caja negra (entran modelos, salen números), de forma interna, desde el repo hermano privado `mamut-engine-oracles`.

## 3. Roles

| Rol | Puede ver | No puede ver | Hace |
|---|---|---|---|
| **Especificador** (persona o IA) | Fuentes permitidas (§4), contratos Rust | Código de OpenSees, Stabileo y las rutas del §5 | Escribe la ficha del método (ecuaciones y citas) y los fixtures de benchmark |
| **Implementador** (persona o IA) | Contrato Rust, ficha del método, plantilla de método, fixtures, bitácora | Lo mismo que el especificador, y otros motores FEM no permisivos | Implementa en Rust y hace pasar conformidad y benchmarks |
| **Operador de oráculo** (interno) | Repo hermano, binario de OpenSees, manual de usuario de OpenSees para armar el modelo | Código fuente de OpenSees. No escribe código ni fixtures del motor | Corre OpenSees como caja negra y compara con nuestro CLI; resultados solo en el repo hermano |
| **Auditor** | Todo, solo para comparar | — | Corre similitud de código y revisa bitácoras. Nunca escribe código del motor ni dice al implementador «cómo lo hace OpenSees» |

Una misma persona o sesión no puede combinar los roles de operador de oráculo o auditor con los de especificador o implementador del mismo método. Quien haya visto código o rutas prohibidas (§5) no puede escribir fichas, fixtures ni código de ningún método; solo puede actuar como auditor u operador de oráculo. Pasar de auditor a especificador exige una sesión nueva y limpia, registrada en la bitácora.

## 4. Fuentes permitidas

- Teoría, ecuaciones y algoritmos en prosa o pseudocódigo: libros de texto, papers y reportes publicados. Los papers de autores de OpenSees están permitidos: es ciencia publicada; lo prohibido es el código.
- Derivaciones propias, documentadas en la ficha del método.
- Documentación (no código) de dependencias con licencia permisiva verificada.
- Soluciones cerradas y ejemplos publicados, citados en cada fixture.
- Libros de referencia: la lista canónica está en Notion (`02_PROTOCOLO_CLEAN_ROOM`, §3). **PENDIENTE:** decidir si se copia al repo.
- No se copian listados de código de libros salvo que su licencia lo permita y quede registrado.
- Documentos con copyright (p. ej. API RP 2GEO): se adquiere copia legal, se implementan ecuaciones, no se copia texto (**PENDIENTE** ENG-GOV-004).
- Código permisivo de terceros: por defecto no se copia. Copiar un fragmento requiere aprobación de Jonathan, registro en `THIRD_PARTY_NOTICES` y nota en la bitácora.
- Dependencias de álgebra lineal: solo con licencia permisiva aceptada por `cargo-deny` (ENG-GOV-005).

## 5. Fuentes y rutas prohibidas (no abrir)

- Repo `OpenSees/OpenSees`, sus forks y derivados (Xara, OpenSeesPy, Tessera, `openseesjs`…), su wiki de desarrolladores y fragmentos de código fuente en foros.
- `lambdaclass/stabileo` (crate `dedaliano-engine`) y `node_modules/stabileo-*`.
- Dentro de MAMUT: `mamut-structures/src/stabileo-adapter`, `motor-fem/engine`, `/workspace/opensees-web/spike/` y la parte de `motor-geo` que usa groundhog.
- Otros motores con licencia GPL, LGPL, AGPL, desconocida o sin licencia (CalculiX, code_aster, OOFEM, carapace, anaStruct). Kratos solo con aprobación.
- El manual de comandos de OpenSees **como especificación**: no define formato ni nombres. Solo lo usa el operador de oráculo.
- Los listados de código de Davis (*Direct Methods for Sparse Linear Systems*, CSparse): se lee la prosa, no se copia el código.

La lista machine-readable vivirá en `clean-room/forbidden_paths.txt` y `clean-room/forbidden_identifiers.txt` (**PENDIENTE**, se crea con ENG-GOV-005, que las consume en CI).

## 6. Nombres y formato propios

- Formato JSON propio (`mamut.engine.model/1`); sin comandos Tcl ni estructura de script.
- No aparecen en schemas, CLI, MCP ni `kind` los identificadores de OpenSees. Lista inicial:
  - Comandos: `geomTransf`, `uniaxialMaterial`, `nDMaterial`, `eleLoad`, `timeSeries`, `fix`, `equalDOF`, `rigidDiaphragm`, `recorder`, `numberer`, `integrator`, `algorithm`, `analyze`.
  - Elementos: `elasticBeamColumn`, `dispBeamColumn`, `forceBeamColumn`, `zeroLength`.
  - Materiales: `Steel01`, `Steel02`, `Concrete01`, `Concrete02`, `PySimple1`, `TzSimple1`, `QzSimple1`, `PressureDependMultiYield`.
  - Solvers: `ProfileSPD`, `BandGeneral`, `UmfPack`, `LoadControl`, `DisplacementControl`.
- Los modelos se nombran por autor y física: `steel.menegotto_pinto`, `concrete.kent_park`, `py.matlock_soft_clay`, `py.reese_sand`.
- Importar `.tcl`/`.py` o exportar scripts a OpenSees: no se planifica antes del memo ENG-GOV-004 (**PENDIENTE**).

## 7. Registro de procedencia

Por ticket, `provenance/ENG-XXX.md` a partir de `provenance/TEMPLATE.md`. La CI (ENG-GOV-005, en desarrollo) rechazará un PR que toque `crates/` sin bitácora; hasta que esté activa y protegida, el revisor lo comprueba a mano.

## 8. Verificación contra oráculos libres

- **Validación principal (en este repo):** soluciones analíticas, libros y ejemplos publicados, versionados como fixtures con `source` y cita.
- **Comparación complementaria:** `mamut-engine-oracles` (privado, uso interno). Corre OpenSees como caja negra; sus resultados **no** entran a los fixtures de este repo, ni es dependencia de código, CI obligatoria o release.
- Ningún método se da por validado solo con OpenSees.
- Uso interno de OpenSees y su viabilidad: **PENDIENTE** del memo ENG-GOV-004.

## 9. Auditoría de similitud

Antes de cada release, el auditor compara el código del motor con OpenSees y Stabileo con una herramienta de similitud y revisa manualmente los casos marcados. Herramienta y umbral: **PENDIENTE**. El resultado se archiva en un repo privado.

## 10. Sesiones de agentes IA

- Cada sesión de implementación empieza con contexto limpio, en un checkout que no contiene rutas prohibidas.
- Los agentes que ya vieron código de OpenSees/Stabileo (p. ej. el del spike del 2026-10-07) no implementan el motor.
- Un modelo pudo haber visto código en su entrenamiento: se mitiga con §9; el riesgo residual lo valora un abogado (**PENDIENTE** legal).
- Registrar en la bitácora el agente y el modelo usados.

**Instrucción estándar (pegar al inicio de cada sesión):**

```text
Vas a implementar <método> en Rust para este repo, bajo protocolo clean-room (docs/CLEAN_ROOM.md).
Fuentes permitidas: solo las citadas en la ficha del método y en docs/CLEAN_ROOM.md §4.
PROHIBIDO: buscar, abrir, citar, recordar o reproducir código fuente de OpenSees, Stabileo
o cualquier derivado o motor de las rutas del §5, y usar sus nombres de comandos, elementos,
materiales o solvers (§6). No navegues a esos repos ni a foros con fragmentos de su código.
Si no puedes implementar algo solo con las fuentes permitidas, DETENTE y reporta la duda;
no improvises desde memoria de otro motor. No copies código de libros ni de crates.
Al terminar, completa provenance/ENG-XXX.md con fuentes, derivaciones y declaración.
```

## 11. Checklist de PR

La plantilla está en `.github/PULL_REQUEST_TEMPLATE.md`. Resumen:

- [ ] Bitácora completa y firmada por un revisor
- [ ] Ninguna ruta prohibida en el diff ni en el historial de la sesión
- [ ] Prueba de nombres prohibidos en verde (ENG-GOV-005)
- [ ] `cargo-deny` verde
- [ ] CLA firmado si el autor es externo (ENG-GOV-007)
- [ ] Fixtures con `source` y cita
- [ ] Conformidad y benchmarks en verde en nativo y wasm
- [ ] «Sí» explícito de Jonathan para el merge

## 12. Pendientes y dudas abiertas

- Nombre definitivo y visibilidad del repo (ENG-GOV-001).
- Herramienta y umbral de similitud.
- Memo legal (ENG-GOV-004) y posterior actualización de este protocolo.
- CLA (ENG-GOV-007) y CI (ENG-GOV-005).
