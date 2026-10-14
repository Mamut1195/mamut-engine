# ADR-003: paquete Python nativo V1

- **Tarea:** ENG-PY-001
- **Estado:** Aceptado (Jonathan, 2026-10-14)
- **Fecha:** 2026-10-14 (America/Guayaquil)

## Contexto

MAMUT Engine necesita una superficie local para notebooks, scripts y
experimentación sin duplicar el cálculo estructural. El plan V1 de Jonathan
del 2026-10-13 incorpora Python y mantiene una única implementación numérica
Rust. Sustituye la exclusión anterior de bindings Python para V1; ENG-SFC-007
se reactiva para V1 y ENG-CORE-008 permanece archivado porque proponía otro
motor en Python/NumPy.

Las fuentes son los exports locales `ENG-PY-001-ADR.txt` (objetivo, fronteras
y DoD) y `V1-python-DAG.txt` (arquitectura, DAG y gates), y el plan aprobado
por J0 `engine-cli-py-plan.md`, en `/workspace/fu/tickets/`. Se usan como
datos de diseño. El formato sigue [ADR-001](ADR-001-fundacionales.md) y
[ADR-002](ADR-002-algebra-lineal.md). La aprobación del plan no implica
aprobación de este ADR, implementación, fusión ni publicación.

La base local de esta entrega es `main`/`origin/main`
`fd5319f824b3f11e7816343a3dcd4e1f71c28385`, sin actualización remota. El
workspace contiene 14 crates y todavía no incorpora `engine-api`,
`bindings/python` ni `pyproject.toml`. La
[conciliación ticket↔PR/issue](../TICKETS_PRS.md) registra un corte de GitHub
más reciente que algunas observaciones del plan: #61 y #62 constan fusionados,
#64 abierto y #83 fusionado en los exports. Estos estados no prueban una API
de producto completa ni una CI de wheels verificada.

El [protocolo clean-room](../CLEAN_ROOM.md) sigue vigente. Esta tarea es
exclusivamente documental: especifica fronteras y verificación futura, sin
cambiar algoritmos, contratos, manifests ni dependencias.

## Decisión

1. **Un solo núcleo Rust.** Los crates numéricos son propietarios de los
   cálculos, validación física y métodos oficiales. El core no depende de
   Python, PyO3, NumPy, clientes web ni Harness. CLI, MCP y WASM tampoco
   dependen del binding Python. La ejecución local no exige nube, login,
   Platform ni Harness.
2. **Fachada compartida `engine-api` (ENG-API-001).** Será el contrato público
   Rust para validación, ejecución, capacidades, esquemas, versión y explicación
   de errores. Poseerá la adaptación serde/JSON canónico y la integración de
   `engine-input/1` y `results/1`; los crates numéricos no dependerán de serde
   por esta integración. Python consumirá esa fachada, sin convertir los
   internals de análisis o ensamblaje en otra API pública. El binding real
   espera la fachada; el scaffold puede exponer solo `version()`.
3. **PyO3/maturin en el mismo repositorio.** Proponer `bindings/python/`
   como crate `mamut-engine-py` de tipo `cdylib`, con PyO3 0.25.1 y
   `abi3-py310`, `pyproject.toml` con maturin y paquete
   `python/mamut_engine/{__init__.py, _native.pyi, py.typed}` dentro del
   binding. Se importará como `mamut_engine`; el nombre de distribución
   `mamut-engine` queda sujeto a disponibilidad y aprobación. El binding
   quedará **fuera de `default-members`** con la lista principal explícita.
   Los builds nativo y wasm del motor conservarán una selección sin Python;
   al incorporar el binding habrá que separar las comprobaciones del motor
   de las del paquete, pues `--workspace` también incluye miembros ajenos a
   `default-members`.
4. **API tipada y capacidades explícitas.** La superficie propuesta se
   detalla abajo. Los modelos, análisis, resultados y diagnósticos conservan
   contratos versionados; una capacidad ausente devuelve `Unsupported`, sin
   fallback ni solver Python. Los workflows reutilizan el registro Rust y
   el SDK existente (SDK-WF-003), con orquestación fuera del solver.
5. **Unidades, ejes y trazabilidad compartidos.** La entrada pública usa SI
   explícito, rechaza NaN/Inf y conserva los datos originales. Se mantiene
   [X-Y horizontal, Z arriba](../CONVENCION_EJES.md): D2 tiene Ux/Uy/Rz y D3
   Ux/Uy/Uz/Rx/Ry/Rz. La elevación 2D sigue pendiente; no habrá mapeo
   silencioso a X-Z. Un adaptador deberá declarar y probar cualquier
   conversión aprobada. Resultados y manifest registrarán versión del motor,
   commit, target, contratos, métodos, hash de entrada y diagnósticos. El
   JSON canónico y la normalización de cero se concentran en la fachada.
6. **Compatibilidad separada de la ABI.** El paquete y la fachada siguen
   semver; cada documento declara su `schema`. Una versión mayor desconocida
   se rechaza con error tipado. `capabilities()` identifica los esquemas y
   métodos realmente admitidos. `abi3-py310` propone CPython 3.10 o superior;
   no garantiza por sí solo compatibilidad de contratos, plataformas,
   arquitecturas ni otros intérpretes. Cada combinación distribuida debe
   verificarse en ENG-PY-007.
7. **Memoria, GIL, progreso y cancelación.** Rust conserva el ownership de
   modelos, matrices y resultados durante el cálculo. Las conversiones y
   vistas Python tendrán lifetimes explícitos; NumPy será opcional en
   ENG-PY-004, con validación de shape/dtype y copias cuando no pueda
   garantizarse acceso seguro. `run` libera el GIL durante el trabajo Rust
   donde sea seguro, sin acceder a objetos ni ejecutar callbacks Python
   dentro del solver. Progreso y cancelación se coordinan en la fachada
   entre pasos; los callbacks del adaptador requieren recuperar el GIL fuera
   del cálculo. La cancelación produce `Cancelled`; los resultados parciales
   se identifican y no se presentan como éxito. Los errores de biblioteca
   son fallibles y tipados, sin `panic!`, `unwrap` ni `expect`.
8. **Licencia dual y publicación humana.** El paquete conserva
   **AGPL-3.0 + licencia comercial de MAMPRO SAS**, conforme a la decisión
   del 2026-10-08 ratificada el 2026-10-13 y al alcance solicitado para este
   ADR. Apache-2.0 del SDK genérico no se extiende al motor; tampoco se adopta
   PolyForm. La AGPL permite uso comercial sujeto a sus obligaciones. Los
   textos legales, titularidad formal, CLA y oferta comercial requieren
   revisión en ENG-GOV-001/004/007/008; este ADR no redacta textos legales ni
   resuelve el pendiente histórico de titularidad de ADR-001. **Nada se
   publicará en PyPI sin aprobación expresa de Jonathan.**

### Superficie V1 y responsabilidad

Esta matriz es una especificación pendiente de implementación, no una lista
de funciones Python ya disponibles.

| Superficie Python | Responsabilidad de Rust / contrato | Ticket y límite |
| --- | --- | --- |
| `version()` | Identidad del motor, versión y build | ENG-PY-002: scaffold mínimo; ENG-API-001: identidad común |
| `capabilities()` | Métodos, dimensiones y esquemas admitidos | ENG-API-001 / ENG-SFC-007; solo capacidades verificadas |
| `schema(kind)` | Esquemas `engine-input/1` y `results/1` | ENG-API-001; versión mayor desconocida rechazada |
| `validate(model)` | Validación de entrada y diagnósticos del dominio | ENG-API-001 / ENG-PY-003; validar no equivale a probar estabilidad |
| `run(input, *, cancel=None, progress=None)` | Ejecución y estado por la fachada | ENG-SFC-007 / ENG-PY-003; `Unsupported`, `NotConverged`, `Unstable`, `Cancelled` explícitos |
| `explain(code)` y `EngineError(code, entities, cause)` | Catálogo `ENG-E`, entidades y causa | ENG-API-001 / ENG-PY-003; sin pérdida de contexto |
| Modelos, casos, análisis y `Results` tipados | Contratos Rust y `results/1`, con vistas sobre resultados | ENG-PY-003/004 y ENG-RES-001; sin recomputar resultados en Python |
| Matrices y arrays NumPy opcionales | Datos Rust con shape/dtype/lifetime documentados | ENG-PY-004; sin buffers compartidos inseguros |
| Workflows y experimentación | Registro de métodos y workflows existente | ENG-PY-005; no scripts Python ejecutándose en solvers |

## Alternativas descartadas

- **Segundo motor en Python/NumPy (ENG-CORE-008):** duplicaría cálculo,
  validación y mantenimiento; permanece archivado.
- **Binding directo a internals Rust:** expondría detalles que pueden cambiar
  y fragmentaría los contratos entre Python, CLI, MCP y WASM. Se espera
  `engine-api` para la ejecución científica real.
- **Servicio remoto obligatorio o ejecución mediante CLI como API Python
  principal:** añadiría infraestructura o un protocolo de procesos a la
  experiencia local. El CLI sigue siendo otra superficie de la fachada.
- **Python en el core o en los solvers:** invertiría dependencias, afectaría
  wasm y complicaría ownership, GIL y determinismo.
- **Excluir Python de V1:** contrario al plan de Jonathan del 2026-10-13.
  No se propone compilar Python a Rust ni introducir Lua.
- **Apache-2.0/PolyForm para el motor o publicación automática en PyPI:**
  incompatibles con la licencia dual ratificada y la aprobación requerida.

## Consecuencias

- ENG-PY-001 precede al scaffold ENG-PY-002. ENG-SFC-007 necesita tanto el
  scaffold como ENG-API-001; esta fachada depende de ENG-ANL-001 y
  ENG-CORE-006. El estado y alcance de este último no se deducen de un PR
  con ID en los exports: véase la conciliación.
- Tras los bindings, ENG-PY-003 habilita ENG-PY-004 (NumPy/results) y
  ENG-PY-005 (workflows). ENG-PY-006 verifica paridad después de ambos;
  ENG-PY-008 documenta modelos y workflows también después de ambos.
- ENG-PY-007 puede preparar wheels después del scaffold, sin publicar.
  Paridad, superficies CLI/WASM, benchmarks y gates legales son condiciones
  de release; no impiden diseñar o experimentar con el scaffold. La V1
  integrada sale por ENG-REL-001 después de ENG-PY-006/007/008 y
  ENG-SFC-006, sin presentar una entrega parcial como V1 completa.
- Se proponen wheels abi3 para manylinux2014 x86_64/aarch64, macOS universal2
  y win_amd64, instalables sin Rust y sin internet una vez descargados.
  La matriz efectiva y la CI de wheels siguen pendientes. La fusión de #83
  según el export no acredita por sí sola ese gate.
- ENG-PY-006 deberá probar paridad Rust/CLI/WASM/Python con fixtures
  analíticos, tolerancias justificadas, unidades/signos iguales, hashes y
  estados de error. No se consultará ni copiará código de OpenSees, Stabileo
  o derivados ni se usarán como oráculos en esta entrega.
- ENG-PY-007 debe aportar CI ejecutada, checksums, SBOM,
  `THIRD_PARTY_NOTICES` y textos AGPL/comerciales revisados antes de
  distribuir. No se afirma cumplimiento legal ni compatibilidad de wheels
  por seleccionar PyO3/maturin.
- Los seguimientos GitHub abiertos se conservan como tales, incluidos
  #69–#72, #75–#78 y #84–#86 aunque existan correcciones fusionadas. La
  exposición de capacidades requiere evaluar esos pendientes y la
  conformidad real, sin deducirla de la presencia de código.

## Aceptación (2026-10-14)

Registro de aprobación: Jonathan (cuenta Mamut1195) fusionó personalmente
el PR #91 en GitHub (commit 2a21216, 2026-10-14 14:13 ECT); la cuenta
compartida le impide aprobar como reviewer, por lo que la fusión hecha
por él es la evidencia de aprobación directa (mismo precedente que
CLEAN_ROOM §5.1 / #74 → #79). Ticket Linear: MAM-75 (ENG-PY-001).

Antecedente (los relays solos no cuentan): el 2026-10-14 a las 13:57 ECT
J0 comunicó que Jonathan aprobó ADR-003 (MAM-75 / ENG-PY-001).

## Pendientes

- [x] Aprobación de este ADR por Jonathan — fusión personal del PR #91 (commit 2a21216, 2026-10-14 14:13 ECT; MAM-75). Revisión clean-room independiente pendiente.
- [ ] Confirmar los tickets ambiguos y ENG-CORE-006 en la conciliación.
- [ ] Implementar y verificar fachada, bindings, tipos, memoria y cancelación.
- [ ] Resolver la convención de elevación 2D antes de anunciarla como capacidad.
- [ ] Verificar plataformas/CPython, paridad y CI de wheels con evidencia.
- [ ] Completar los gates legales y obtener aprobación expresa de Jonathan
      para cualquier publicación en PyPI y para el release integrado.
