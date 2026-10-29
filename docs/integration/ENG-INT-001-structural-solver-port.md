# ENG-INT-001 (MAM-60) · Especificación del adaptador para `StructuralSolverPort` (Structures)

Estado: **borrador de especificación**. No hay código del adaptador en este repo.
La implementación del adaptador va al **backlog de Structures**
(repo `Mamut1195/mamut-structures`); este documento solo fija el mapeo contra
los contratos publicados del Engine.

## 0. Alcance, fuentes y reglas

- Fuente del lado Structures: **solo la interfaz pública** del puerto,
  `packages/structural-core/src/solver-port/index.ts` del repo
  `Mamut1195/mamut-structures` (leída en solo lectura). No se leyó el
  adaptador Stabileo de Structures ni código de Stabileo (clean-room,
  ENG-GOV-004). Los tipos de dominio que el puerto importa
  (`CanonicalStructuralModel`, `SolverRunArtifact`, `ValidationResult`,
  `MappedDiagnostic`) **no se leyeron**: sus campos son DESCONOCIDO.
- Fuentes del lado Engine: [`schemas/engine-input/1.json`](../../schemas/engine-input/1.json)
  (`urn:mamut:engine-input:1`), [`schemas/results/1.json`](../../schemas/results/1.json)
  (`urn:mamut:results:1`), [`schemas/capabilities/1.json`](../../schemas/capabilities/1.json),
  [`schemas/cli-run/1.json`](../../schemas/cli-run/1.json),
  [`schemas/cli-error/1.json`](../../schemas/cli-error/1.json),
  el registro [`docs/contracts/error-codes-1.json`](../contracts/error-codes-1.json),
  la guía [`docs/GUIA_CLI_AGENTES.md`](../GUIA_CLI_AGENTES.md) y el registro de
  métodos de `crates/engine-api/src/methods.rs`.
- Transporte: para Harness se cita una decisión de Jonathan (2026-10-20) de
  hablar con el Engine **solo por CLI**; esa decisión es DESCONOCIDO desde el
  repo (no consta en él). Para Structures el transporte (CLI o WASM en
  proceso) está **SIN decidir**; ver §6.
- Sin valores numéricos de OpenSees, Frame3DD ni Stabileo. Nunca se empaquetan
  OpenSees ni Stabileo.

## 1. Operaciones del puerto → operaciones del Engine

| Puerto (`StructuralSolverPort`) | Engine | Notas |
| --- | --- | --- |
| `capabilities(): SolverCapabilities` | `mamut-engine capabilities` → `capabilities/1`; `--version --json` → `version/1` | Tabla en §3. `--version` sin `--json` imprime texto plano (`<engineId> <engine_version>`), no `version/1`. `version/1` solo contiene `engine_version` y `contracts`; no trae nombre ni SHA, así que `solverName`/`solverVersion` no salen de ahí (ver §2.3). |
| `validate(model): ValidationResult` | `mamut-engine validate --input <doc>` → `validate-report/1` (exit 0 `ok`, 2 `invalid`) | Forma de `ValidationResult`: DESCONOCIDO. La validación del Engine es `topology` (`capabilities/1.validation`): solo comprueba topología, no estabilidad. Ejemplo: una viga sin apoyos da exit 0 en `validate` y exit 4 (`ENG-E-0006`) en `run`. El adaptador no debe usar `validate` para predecir estabilidad. |
| `solve(request): Promise<SolverRunArtifact>` | `mamut-engine run --input <doc>` → `cli-run/1` con `engineResults` (`results/1`) | Un solo análisis por documento (`analyses`). |
| `solveBatch(request)` | Sin equivalente: N invocaciones `run`, una por escenario, orquestadas por el adaptador | Concurrencia, reintentos (`maxRetries`) y `onProgress` son del adaptador. |
| `cancel(reason?)` | `SIGINT`/`SIGTERM` al proceso o marcador de cancelación → exit 5 | El `reason` no se transmite al Engine. |

## 2. Mapeo de datos

### 2.1 `SolveRequest` / `ScenarioSolveRequest` → `engine-input/1`

`engine-input/1` exige en la raíz: `schema`, `units`, `model`, `cases`,
`combinations`, `analyses`, `options`.

| Campo del puerto | Destino en `engine-input/1` | Estado |
| --- | --- | --- |
| `model: CanonicalStructuralModel` | `model` (`axes: "Z-up"`, `nodes`, `members`, `sections`, `laws`, `supports`, `masses`; opcionales `prescribed`, `diaphragms`, `springs`) más `units`, `cases`, `combinations` | Mapeo campo a campo: **DESCONOCIDO** (no se leyó el tipo). Pendiente de revisión con Structures. |
| `analysis: AnalysisKind` | `analyses[].method` | Ver tabla 2.2 |
| `scenarioId` | Sin campo en `engine-input/1` | El adaptador lo conserva fuera del documento. |
| `scenarioSeed` | Sin campo | DESCONOCIDO si debe influir en el modelo antes de serializar. |
| `necDatasetVersion`, `hazardDatasetVersion` | Sin campo | Lado Structures; el Engine no los consume. |
| `signal: AbortSignal` | Cancelación del proceso (exit 5) | — |
| `BatchSolveRequest.timeoutMs` | `--timeout` (segundos, máx. `MAX_TIMEOUT_SECONDS` = 31 536 000) → exit 6 | Conversión ms→s a cargo del adaptador. `--timeout` exige 0 < s ≤ 31 536 000: `run --timeout 0` da `CLI-E-USAGE` con exit 2. Para «sin límite» el adaptador debe omitir `--timeout`; aun así `MAMUT_ENGINE_TIMEOUT` (mismo rango, el flag prevalece) sí aplica si está definida, así que el adaptador debe quitarla del entorno si quiere ejecutar sin límite. |

Límites del schema que el adaptador debe respetar: `model.nodes` máx. 256,
`model.members` máx. 1024, `springs` máx. 1024 (resto en `describe --json`).
Unidades SI (m, kg, s, N, Pa, rad; momentos N·m).

### 2.2 `AnalysisKind` → `analyses[].method`

| `AnalysisKind` | `method` | Estado en el registro del Engine |
| --- | --- | --- |
| `linearStatic` | `linearStatic` | Validated |
| `pDelta` | `pDelta` (requiere `options`) | Stable (evidencia ENG-BENCH-002) |
| `modal` | `modal` (requiere `options`) | Validated |
| `responseSpectrum` | `responseSpectrum` (requiere `options`) | Validated |
| `kinematic` | Sin método equivalente | No cubre |

### 2.3 `results/1` → `SolverRunArtifact` / `ScenarioSolveOutcome`

`results/1` contiene: `schema`, `engine_version`, `git_commit`, `target`,
`contracts`, `methods`, `input_sha256`, `status`, `nodes`, `members`, `modes`,
`cases`, `envelopes`, `combinations`, `convergence`, `diagnostics`,
`geotechnical`.

| Campo del puerto | Origen en el Engine | Estado |
| --- | --- | --- |
| `artifact: SolverRunArtifact` | `cli-run/1.engineResults` (`results/1`) | Mapeo campo a campo: **DESCONOCIDO** (no se leyó el tipo). |
| `diagnostics: MappedDiagnostic[]` | `results/1.diagnostics` y `cli-error/1.diagnostics` | Forma de `MappedDiagnostic`: DESCONOCIDO. Códigos `ENG-E-…`, `API-E-…`, `API-W-…`. |
| `solverName` / `solverVersion` / `solverBuildSha` | Nombre: `engineId` (texto de `--version` y `cli-error/1`); versión: `results/1.engine_version` (o `version/1.engine_version`); SHA: `results/1.git_commit` | `version/1` no contiene nombre ni SHA. |
| `adapterVersion` | Lado Structures | — |
| `attempts` | Lado adaptador | — |
| `error.code` / `error.message` | `cli-error/1.code` y mensaje | No parsear el texto del mensaje (guía §1). |

## 3. `SolverCapabilities` frente a V1

Criterio: «cubre» = método `Validated` o `Stable` con evidencia en el registro;
«experimental» = método con estado `Experimental`. Fuente: la descripción de
[`schemas/engine-input/1.json`](../../schemas/engine-input/1.json) (línea 5) y
el registro de `crates/engine-api/src/methods.rs`: solo `nonlinearStatic`,
`soilProfile` y `consolidation1d` son `Experimental` (pendientes de ENG-VAL-006,
requieren `options.allow_experimental = true`); `pDelta` es `Stable`
(ENG-BENCH-002), no `Validated` (Validated: PENDIENTE, sin ticket asignado). El
estado de ENG-VAL-006 es DESCONOCIDO desde el repo.

| Capacidad | Engine V1 | Base |
| --- | --- | --- |
| `linearStatic` | cubre | `linearStatic` Validated |
| `pDelta` | cubre (Stable; ENG-BENCH-002; Validated: PENDIENTE (sin ticket asignado)) | `pDelta` Stable, no Validated |
| `modal` | cubre | `modal` Validated |
| `responseSpectrum` | cubre | `responseSpectrum` Validated |
| `kinematic` | no cubre | Sin método |
| `shells` | no cubre | Elementos: `frame2d/3d.euler_elastic`, `truss2d/3d.linear`, `frame2d/3d.winkler`, `spring.linear`, `spring.nonlinear` |
| `nonlinear` | experimental | `nonlinearStatic` Experimental; requiere `options.allow_experimental = true` |
| `fiberBeamColumn` | no cubre | Sin elemento de fibras |
| `constraints` | DESCONOCIDO | Existen `diaphragms`, `prescribed` y releases; si equivalen a lo que el puerto entiende por `constraints` es DESCONOCIDO |

Qué capacidades de Stabileo usa hoy Structures: **DESCONOCIDO** (exigiría leer
el adaptador Stabileo; fuera de alcance por clean-room/ENG-GOV-004). Este
documento solo puede compararse con los flags declarados por el puerto.

## 4. `ScenarioSolveStatus` ↔ salidas de la CLI

| Exit | `outcome` | `results/1.status` posibles | `ScenarioSolveStatus` propuesto |
| --- | --- | --- | --- |
| 0 | `ok` | `ok` | `solved` |
| 1 | `warning` | `warning` | `solver-warning` (resultados completos) |
| 2 | `usage`, `invalid_input` | `invalid` | `invalid-model` (`usage` es error del adaptador → `failed`) |
| 3 | `unsupported` | `unsupported` | `invalid-model` o `failed`: DESCONOCIDO cuál espera Structures |
| 4 | `not_converged` | `notConverged`, `unstable` | `non-convergent` |
| 5 | `cancelled` | `cancelled` | `cancelled` |
| 6 | `timeout` | `timeout` | `timeout` |
| 70 | `internal` | `internalError` | `failed` |

Binario ausente o no ejecutable: `SolverUnavailableError` (`SOLVER_UNAVAILABLE`),
lado adaptador. Cancelación: `SolverCancelledError` (`SOLVER_CANCELLED`).

### 4.1 Códigos de error reales (registro `error-codes-1.json`)

| Código | Exit | Descripción |
| --- | --- | --- |
| `ENG-E-0001` | 2 | Unknown unit. |
| `ENG-E-0002` | 2 | Duplicate entity identifier. |
| `ENG-E-0003` | 2 | Missing entity reference. |
| `ENG-E-0004` | 2 | Node is not connected to any member. |
| `ENG-E-0005` | 2 | Member has zero length. |
| `ENG-E-0006` | 4 | System is singular or unstable. |
| `ENG-E-0007` | 4 | Numerical procedure did not converge. |
| `ENG-E-0008` | 2 | Non-finite input or arithmetic result. |
| `ENG-E-0009` | 2 | Incompatible contract name or major version. |
| `ENG-E-0010` | 2 | Invalid dimensions, sizes or operation parameters. |
| `ENG-E-0011` | 2 | Degree of freedom is unavailable in this context. |
| `ENG-E-0012` | 70 | Engine encountered an inconsistent internal state. |
| `ENG-E-0013` | 3 | Component does not support the requested capability. |
| `ENG-E-0014` | 4 | Massless degrees of freedom have non-positive-definite stiffness. |
| `API-E-INPUT` | 2 | Invalid JSON or engine input structure. |
| `API-I-CANCELLED` | 5 | Cooperative cancellation was requested. |
| `CLI-E-USAGE`, `CLI-E-DOCUMENT`, `CLI-E-VALIDATION` | 2 | Fallos de CLI sin diagnóstico del Engine |
| `CLI-E-UNSUPPORTED` | 3 | — |
| `CLI-E-NOT-CONVERGED` | 4 | — |
| `CLI-E-CANCELLED` | 5 | — |
| `CLI-E-TIMEOUT` | 6 | — |
| `CLI-E-METHOD`, `CLI-E-INTERNAL` | 70 | — |

El adaptador debe decidir por **exit code**, luego por `code`; nunca por el texto.

## 5. Bloqueos y dependencias

- MAM-9 (ENG-API-001 · Fachada engine-api): estado DESCONOCIDO desde el repo
  (según Linear, Done).
- MAM-89 (ENG-RES-001 · Resultados estructurales y postproceso): estado
  DESCONOCIDO desde el repo (según Linear, Done).
- ENG-VAL-006 (MAM-604): estado DESCONOCIDO desde el repo; condiciona qué pasa
  de `Experimental` a otro estado.
- Reemplazo de Stabileo en Structures: se cita una decisión de Jonathan
  (2026-10-20), DESCONOCIDO desde el repo; Stabileo espera a ENG-GOV-004.
- Implementación del adaptador: **backlog de Structures**, no en este repo.

## 6. PENDIENTE (SIN decidir)

- **Licencia / AGPL por WASM**: si Structures enlaza el motor en proceso
  (`engine-wasm`), queda bajo AGPL-3.0 o necesita la licencia comercial.
  **SIN decidir**; ver ENG-GOV-004. Este documento no propone decisión.
- Transporte CLI frente a WASM para Structures: SIN decidir.

## 7. DESCONOCIDOS

1. Campos de `CanonicalStructuralModel`, `SolverRunArtifact`, `ValidationResult`, `MappedDiagnostic`.
2. Capacidades de Stabileo que Structures usa hoy.
3. Qué entiende el puerto por `constraints`.
4. Estado esperado ante exit 3 (`unsupported`).
5. Uso de `scenarioSeed`. (`timeoutMs = 0`: `--timeout 0` es exit 2; ver §2.1.)
6. Decisión de Jonathan del 2026-10-20, estado de MAM-9, MAM-89 y ENG-VAL-006:
   DESCONOCIDO desde el repo.
