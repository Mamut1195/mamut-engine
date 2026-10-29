# Clean-room log · ENG-VAL-039

- Ticket / PR: MAM-851 · VAL-039; rama `j2/MAM-851-patologicos`.
- Author(s): Codex (gpt-6.1-sol, high) · 2026-10-29 (America/Guayaquil).
- Role: J2 especifica (task file del ticket); Codex (gpt-6.1-sol, high) implementó
  las pruebas y este registro; un helper ejecutor de J2 verificó fuera del sandbox
  (pruebas, mutaciones, chequeos) y redactó la segunda revisión de este registro
  y el ajuste de la prueba en serie (H4).
- Contract implemented: `engine-input/1`, `results/1`, `cli-error/1`;
  códigos y salidas de `docs/contracts/error-codes-1.json`.
- Permitted sources used: solo derivación propia [S], desarrollada abajo.
  Sin red, datos numéricos ajenos, comparaciones ni ejecuciones de terceros.
- Own derivations: [S], sección «Derivaciones y criterios» de este documento.
- Third-party code copied: none.
- Validation: analítica propia [S], equilibrio, finitud, escalado, diagnóstico
  determinista y ejecución real de API y CLI contra el mismo Engine.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: «No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket.»
- Clean-room reviewer: PENDIENTE.

## Alcance y archivos

Se revisaron los patrones locales de `crates/engine-api/tests/run.rs`,
`methods.rs`, `dynamic.rs`, `facade.rs`, `support/`, sus entradas JSON y las
pruebas de `crates/engine-cli/tests/cli.rs`. Son referencias de interfaz y
organización de pruebas; todos los modelos y criterios de esta validación
proceden de [S]. No se modifica producción ni se añaden dependencias.

Archivos nuevos:

- `crates/engine-api/tests/val039_pathological.rs` (10 pruebas).
- `crates/engine-cli/tests/val039_cli.rs` (3 pruebas de proceso).
- `crates/engine-api/tests/support/val039.rs` (constructores compartidos).
- `crates/engine-api/tests/fixtures/val039/cantilever.json` (modelo mínimo).
- `provenance/ENG-VAL-039.md` (este registro).
- `changelog.d/MAM-851.validation.md`.

## Qué se probó

Los nombres siguientes corresponden a `val039_pathological`, salvo los tres
nombres con prefijo `CLI:`. Cada error exige exactamente un diagnóstico,
severidad `error`, causa no vacía, código literal, status y exit code. La API
repite la ejecución y compara todo el `Outcome`, incluidos los bytes del
resultado; valida `results/1` con el comprobador existente. Se rechazan
respuestas parciales en errores. La CLI verifica el código del proceso,
stdout vacío y el mismo código en el sobre, el error y `engineResults`.
Ambas suites comprueban la pertenencia y el exit code del registro.

| Caso [S] | Código esperado / resultado | Prueba |
| --- | --- | --- |
| Viga sin apoyos y barra hipostática | `ENG-E-0006`, unstable, exit 4 | `mechanisms_and_massless_mechanism_have_exact_codes_and_no_partial_results` |
| E distintos en dos barras ortogonales del mismo sistema: extremos 1e-6 y 1e12, ambos órdenes; contraste intermedio | Éxito, finitud, equilibrio y δ=FL/EA | `extreme_moduli_in_one_model_remain_finite_and_balanced` |
| Dos barras en serie con esos extremos | Orden E mayor junto al apoyo: éxito, equilibrio y δ analítica. Orden contrario: `ENG-E-0010` (mal condicionamiento), invalid, exit 2, sin respuestas (H4, corregido en MAM-921) | `extreme_moduli_in_series_have_balanced_response_or_clean_rejection`, `h4_soft_bar_next_to_support_reports_ill_conditioning_not_mechanism`, `h4_reclassification_threshold_is_pinned_on_both_sides` |
| Longitud cero con dos identificadores de nudo distintos | `ENG-E-0005`, invalid, exit 2 | `topology_and_contract_errors_have_exact_codes_and_stable_diagnostics` |
| Longitud 1e-12 | `ENG-E-0005`, invalid, exit 2 | `near_zero_lengths_have_defined_rejection_or_finite_equilibrium` |
| Longitud 1e-9 | Éxito, finitud, equilibrio y flecha analítica relativa | `near_zero_lengths_have_defined_rejection_or_finite_equilibrium` |
| Compresión pDelta: 1.2, 2 y 10 veces Pcr propia, columna de cuatro elementos | `ENG-E-0006`, unstable, exit 4, sin respuestas | `supercritical_pdelta_columns_report_instability_without_responses` |
| Dos modos iguales en direcciones X e Y simétricas | Dos frecuencias finitas, λ=20, masa efectiva total por dirección y bytes repetibles | `repeated_modes_in_orthogonal_directions_are_finite_and_deterministic` |
| Masa solo en Uy de punta, Ux y Rz libres sin masa | Éxito, un modo finito, λ=3EI/(L³m), recuperación de giro | `massless_axial_and_rotation_dofs_are_condensed_to_a_finite_mode` |
| Masa en Ux y mecanismo sin masa en Uy | `ENG-E-0014`, unstable, exit 4 | `mechanisms_and_massless_mechanism_have_exact_codes_and_no_partial_results` |
| JSON truncado, documentos concatenados y clave JSON repetida | `API-E-INPUT`, invalid, exit 2; código que el registro asigna explícitamente a sintaxis/estructura | `malformed_json_and_missing_fields_use_the_registered_input_structure_code` |
| Campos schema, model, cases, analyses o units ausentes | `API-E-INPUT`, invalid, exit 2 | `malformed_json_and_missing_fields_use_the_registered_input_structure_code` |
| Id repetido de nudo, miembro, sección o ley | `ENG-E-0002`, invalid, exit 2 | `topology_and_contract_errors_have_exact_codes_and_stable_diagnostics` |
| Sección, ley o nudo de apoyo inexistente | `ENG-E-0003`, invalid, exit 2 | `topology_and_contract_errors_have_exact_codes_and_stable_diagnostics` |
| Nudo suelto | `ENG-E-0004`, invalid, exit 2 | `topology_and_contract_errors_have_exact_codes_and_stable_diagnostics` |
| Contrato engine-input/2 | `ENG-E-0009`, invalid, exit 2 | `topology_and_contract_errors_have_exact_codes_and_stable_diagnostics` |
| Fuerza axial SI: 1e-9 y 1e9 respecto de 1 N | Éxito, equilibrio y escalado de desplazamientos, reacciones y fuerzas locales | `extreme_si_loads_scale_displacements_reactions_and_member_forces` |
| Errores de topología y versión anteriores | Mismos códigos y exit 2 | CLI: `topology_and_contract_errors_exit_two_with_exact_engine_codes` |
| Mecanismos estáticos, mecanismo modal sin masa y columna supercrítica | `ENG-E-0006` / `ENG-E-0014`, exit 4 | CLI: `mechanisms_massless_mechanism_and_supercritical_column_exit_four` |
| JSON truncado y model ausente | `API-E-INPUT`, exit 2 | CLI: `malformed_json_and_missing_model_exit_two_with_input_structure_code` |

La barra en serie es físicamente estable en ambos órdenes. Desde MAM-921 el
orden con E menor junto al apoyo da `ENG-E-0010` con causa «Ill-conditioned
stiffness» en lugar de `ENG-E-0006` (ver H4).
La longitud 1e-12 se trata como nula en el comportamiento observado; no se
infiere de dos muestras un umbral universal para todas las escalas.

## Derivaciones y criterios

[S] Se eligen E=200 Pa, A=2 m², I=3 m⁴, L=4 m y, cuando corresponde,
m=5 kg, sin datos externos. La barra tiene N=EAδ/L=F, por tanto δ=FL/EA.
Dos barras en serie suman flexibilidades: δ=FL/(E₁A)+FL/(E₂A).
Dos barras ortogonales independientes con igual masa y rigidez tienen
K=(EA/L)Id y M=mId; det(K−λM)=0 da dos raíces λ=EA/(Lm)=20 s⁻².
Sus frecuencias son sqrt(λ)/(2π); cada dirección recupera su masa de punta.
Se compara el resultado completo en cinco ejecuciones iguales; no se impone
una orientación universal de los vectores dentro de un subespacio repetido.

[S] En el voladizo, EI v''=−F(L−x), v(0)=v'(0)=0 implica
v(L)=−FL³/(3EI). Al eliminar el giro libre en el problema modal se obtiene
k=3EI/L³, λ=k/m y θ(L)=3v(L)/(2L). Para la columna comprimida,
EI v''''+P v''=0. Las condiciones del empotramiento y extremo libre
(v(0)=v'(0)=0, v''(L)=0, EI v'''(L)+Pv'(L)=0) dan
cos(sqrt(P/EI)L)=0. La primera raíz es π/2: K=2 y
Pcr=π²EI/(KL)². El constructor calcula esa expresión y aplica P/Pcr>1;
no toma una carga crítica tabulada ni pretende medir el error de discretización.

[S] La comprobación estática suma fuerzas y momentos en los tres ejes:
ΣR+ΣF=0 y Σ(r×R+M)+Σ(r×F)=0, incluyendo todos los nudos. El umbral
es 1e-10 multiplicado por la suma de magnitudes de cargas y sus momentos.
La comparación analítica usa tolerancia relativa 1e-10 sin un suelo de unidad,
para que una flecha diminuta no pueda pasar por ser aproximadamente cero.
Se exige que los desplazamientos, reacciones y fuerzas locales sean números
finitos; el esquema de resultados protege también los campos numéricos.

[S] El contrato estructural solo admite SI; las cantidades extremas se expresan
en N, m, Pa, kg y s según corresponda. Multiplicar F por s deja K invariante:
u(sF)=s u(F), R(sF)=s R(F) y N(sF)=s N(F). La prueba usa s=1e-9 y s=1e9;
no declara una conversión a sistemas de unidades no admitidos.

## Hallazgos

Cada hallazgo: no se corrige aquí; se reporta para decisión.

No hubo pánicos ni resultados no finitos o sin equilibrio en las ejecuciones
exploradas que terminaron con éxito. Los siguientes diagnósticos requieren
revisión; no se añade una prueba que apruebe o falle esos comportamientos.
Los comandos se ejecutan desde la raíz, sin red, con `cargo` directo y el target
heredado. Los extractos muestran únicamente campos de las salidas reales.

### H1 · Unidad desconocida clasificada con código general de entrada

Esperado por el ticket y registro: `ENG-E-0001`. Observado en API y CLI:
`API-E-INPUT`, exit 2, status invalid, causa `InvalidInput: Invalid tag: Units`.
Se descartan resultados, pero no se entrega el código específico de unidad.

- Comando: script H1–H3 más abajo (entrada `units: "kN-mm"`).
- Salida: `H1 unidades exit=2 status=invalid code=API-E-INPUT`.
- Esperado vs recibido: `ENG-E-0001` (exit 2) vs `API-E-INPUT` (exit 2).
- No se corrige aquí; se reporta para decisión.

### H2 · No finitos clasificados con código general de entrada

Se exploraron NaN, Infinity, -Infinity y el número JSON 1e999 dentro de la
fuerza nodal. Todos entregan `API-E-INPUT`, exit 2, sin resultados; no aparece
`ENG-E-0008` (entrada o aritmética no finita en el registro). Los tres primeros
no son números válidos en JSON: se documenta la clasificación observada,
sin exigir que el parser admita esos tokens. El caso 1e999 sí tiene forma de
número JSON y excede f64. Por prudencia ante la regla del ticket sobre códigos
generales, ninguno de estos casos se incorpora como prueba de aceptación.
La causa de -Infinity habla del rango de enteros JCS en esta entrada; también
queda pendiente la claridad de ese mensaje.

- Comando: script H1–H3 más abajo (tokens NaN, Infinity, -Infinity, 1e999).
- Salida: `code=API-E-INPUT`, exit 2, en los cuatro casos.
- Esperado vs recibido: `ENG-E-0008` (exit 2) vs `API-E-INPUT` (exit 2).
- No se corrige aquí; se reporta para decisión.

### H3 · Carga transversal del orden de MN rechazada como parámetro inválido

Un voladizo con carga transversal en la punta admite una solución lineal
propia finita (v = FL³/(3EI)), pero con algunas cargas a partir de ~1e6 N, de
forma no monótona (ver las viñetas de abajo), el Engine devuelve `ENG-E-0010`, exit 2, status invalid. La causa es un residuo
numérico en la recuperación, no una dimensión o parámetro inválido según la
descripción del código. La salida descarta todas las respuestas; no se afirma
que entregue valores basura. Se registra como límite de robustez y clasificación
por revisar, sin consagrar ese rechazo en una prueba. La prueba axial de
escalado axial cubre los mismos órdenes de carga con resultado correcto; el
rechazo afecta a la recuperación del giro Rz en flexión.

Umbral medido (barrido de F en el fixture `val039/cantilever.json`, E=200 Pa,
A=2 m², I=3 m⁴, y en un voladizo de acero realista E=2e11 Pa, A=1e-2 m²,
I=1e-4 m⁴, ambos L=4 m):

- Fixture: resuelve hasta F=−3e5 N; rechaza en F=−1e6 N (residuo
  2.79e-9 > 1e-9); vuelve a resolver en F=−3e6 N y rechaza en −1e7, −1e8 y
  −1e9 N. El umbral no es monótono; causa no determinada.
- Acero: resuelve hasta F=−1e6 N (uy=−1.0666666666666664 m, teórico
  −1.0666666666666667 m); rechaza desde F=−3e6 N (residuo 3.73e-9) y en
  −1e7 N (1.49e-8), −1e8 y −1e9 N.
- El residuo relativo crece aproximadamente proporcional a F, lo que indica
  un umbral de recuperación (1e-9) no invariante a la escala de la carga.

Comando del barrido (desde la raíz, sin red):

```sh
python3 - <<'PY'
import copy, json, subprocess
from pathlib import Path
base = json.loads(Path('crates/engine-api/tests/fixtures/val039/cantilever.json').read_text())
def run(v):
    r = subprocess.run(['cargo', 'run', '--offline', '--quiet', '-p', 'engine-cli', '--', 'run', '--input', '-'],
                       input=json.dumps(v), text=True, capture_output=True)
    if r.returncode == 0:
        d = json.loads(r.stdout)
        d = d['engineResults'][0] if 'engineResults' in d else d
        return 0, d, None
    d = json.loads(r.stderr)['engineResults'][0]
    return r.returncode, d, d['diagnostics'][0]
def cantilever(F, E=None, A=None, I=None):
    v = copy.deepcopy(base)
    if E is not None:
        v['model']['laws'][0]['parameters'][0]['value'] = E
        v['model']['sections'][0]['properties'] = [{'name': 'A', 'value': A}, {'name': 'I', 'value': I}]
    v['cases'][0]['loads'][0]['values'][0]['value'] = F
    return v
for label, E, A, I in [('fixture E=200 A=2 I=3', 200.0, 2.0, 3.0), ('acero E=2e11 A=1e-2 I=1e-4', 2e11, 1e-2, 1e-4)]:
    for F in [-1e3, -1e4, -1e5, -3e5, -1e6, -3e6, -1e7, -1e8, -1e9]:
        code, d, diag = run(cantilever(F, E, A, I))
        teo = F * 4.0**3 / (3 * E * I)
        if code == 0:
            print(f'{label} F={F:g} exit=0 uy2={d["cases"][0]["nodes"][1]["displacements"][1]!r} teorico={teo!r}')
        else:
            print(f'{label} F={F:g} exit={code} code={diag["code"]} cause={diag["cause"]}')
PY
```

Salida observada:

```text
fixture E=200 A=2 I=3 F=-1000 exit=0 uy2=-35.55555555555556 teorico=-35.55555555555556
fixture E=200 A=2 I=3 F=-10000 exit=0 uy2=-355.55555555555554 teorico=-355.55555555555554
fixture E=200 A=2 I=3 F=-100000 exit=0 uy2=-3555.555555555555 teorico=-3555.5555555555557
fixture E=200 A=2 I=3 F=-300000 exit=0 uy2=-10666.666666666666 teorico=-10666.666666666666
fixture E=200 A=2 I=3 F=-1e+06 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 2.7939677238464355e-9 > 1e-9.
fixture E=200 A=2 I=3 F=-3e+06 exit=0 uy2=-106666.66666666666 teorico=-106666.66666666667
fixture E=200 A=2 I=3 F=-1e+07 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 2.9802322387695313e-8 > 1e-9.
fixture E=200 A=2 I=3 F=-1e+08 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 2.384185791015625e-7 > 1e-9.
fixture E=200 A=2 I=3 F=-1e+09 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 1.9073486328125e-6 > 1e-9.
acero E=2e11 A=1e-2 I=1e-4 F=-1000 exit=0 uy2=-0.0010666666666666665 teorico=-0.0010666666666666667
acero E=2e11 A=1e-2 I=1e-4 F=-10000 exit=0 uy2=-0.010666666666666665 teorico=-0.010666666666666666
acero E=2e11 A=1e-2 I=1e-4 F=-100000 exit=0 uy2=-0.10666666666666665 teorico=-0.10666666666666667
acero E=2e11 A=1e-2 I=1e-4 F=-300000 exit=0 uy2=-0.31999999999999995 teorico=-0.32
acero E=2e11 A=1e-2 I=1e-4 F=-1e+06 exit=0 uy2=-1.0666666666666664 teorico=-1.0666666666666667
acero E=2e11 A=1e-2 I=1e-4 F=-3e+06 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 3.725290298461914e-9 > 1e-9.
acero E=2e11 A=1e-2 I=1e-4 F=-1e+07 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 1.4901161193847656e-8 > 1e-9.
acero E=2e11 A=1e-2 I=1e-4 F=-1e+08 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 1.1920928955078125e-7 > 1e-9.
acero E=2e11 A=1e-2 I=1e-4 F=-1e+09 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Rz: relative residual 9.5367431640625e-7 > 1e-9.
```

- Esperado vs recibido: solución finita con equilibrio (exit 0), o un código que
  describa la pérdida de precisión, vs `ENG-E-0010` (exit 2, «dimensiones o
  parámetros inválidos»).
- No se corrige aquí; se reporta para decisión.

Comando reproducible para H1–H3, sin dependencias adicionales:

```sh
python3 - <<'PY'
import copy
import json
import subprocess
from pathlib import Path
base = json.loads(Path('crates/engine-api/tests/fixtures/val039/cantilever.json').read_text())
inputs = []
v = copy.deepcopy(base)
v['units'] = 'kN-mm'
inputs.append(('H1 unidades', json.dumps(v)))
for token in ['NaN', 'Infinity', '-Infinity', '1e999']:
    raw = json.dumps(base).replace('-12.0', token)
    inputs.append(('H2 ' + token, raw))
v = copy.deepcopy(base)
v['cases'][0]['loads'][0]['values'][0]['value'] = -1e9
inputs.append(('H3 carga transversal', json.dumps(v)))
for name, raw in inputs:
    result = subprocess.run(
        ['cargo', 'run', '--offline', '--quiet', '-p', 'engine-cli', '--',
         'run', '--input', '-'], input=raw, text=True, capture_output=True)
    envelope = json.loads(result.stderr)
    d = envelope['engineResults'][0]
    print(name, 'exit=' + str(result.returncode), 'status=' + d['status'],
          'code=' + d['diagnostics'][0]['code'])
    print(d['diagnostics'][0]['cause'])
    print('cases=' + str(d['cases']), 'modes=' + str(d['modes']))
PY
```

Salida observada:

```text
H1 unidades exit=2 status=invalid code=API-E-INPUT
InvalidInput: Invalid tag: Units
cases=[] modes=[]
H2 NaN exit=2 status=invalid code=API-E-INPUT
InvalidInput: expected value at line 1 column 720
cases=[] modes=[]
H2 Infinity exit=2 status=invalid code=API-E-INPUT
InvalidInput: expected value at line 1 column 720
cases=[] modes=[]
H2 -Infinity exit=2 status=invalid code=API-E-INPUT
InvalidInput: JSON integer is outside the JCS safe range: |n| <= 9007199254740991 (2^53-1) is required
cases=[] modes=[]
H2 1e999 exit=2 status=invalid code=API-E-INPUT
InvalidInput: JSON number is outside the finite f64 range
cases=[] modes=[]
H3 carga transversal exit=2 status=invalid code=ENG-E-0010
Inconsistent recovery at node 2 / Rz: relative residual 1.9073486328125e-6 > 1e-9.
cases=[] modes=[]
```

### H4 · Falso singular en dos barras en serie con E = 1e-6 junto al apoyo

Dos barras de armadura colineales (L = 4 m cada una, A = 2 m²), apoyo fijo en
el nudo 1, guías Uy en los nudos 2 y 3, fuerza Ux = 1 N en el nudo 3. La
estructura es estable en ambos órdenes y [S] da
δ3 = FL/(E₁A) + FL/(E₂A) ≈ 2e6 m en los dos. Con E₁ = 1e12 (junto al apoyo) y
E₂ = 1e-6 el Engine resuelve exacto; con E₁ = 1e-6 y E₂ = 1e12 devuelve
`ENG-E-0006` («sistema singular o inestable»), un falso singular por contraste
de rigidez. La primera versión de la prueba fijaba ese código; ahora solo exige
un rechazo limpio (sin pánico, sin respuestas parciales, código registrado,
repetible) y no consagra `ENG-E-0006`.

Comando reproducible (desde la raíz, sin red):

```sh
python3 - <<'PY'
import copy, json, subprocess
from pathlib import Path
base = json.loads(Path('crates/engine-api/tests/fixtures/val039/cantilever.json').read_text())
def series(e1, e2):
    v = copy.deepcopy(base)
    for n in v['model']['nodes']:
        n['dofs'] = ['Ux', 'Uy']
    v['model']['members'][0]['element'] = 'truss2d.linear'
    v['model']['sections'][0]['properties'] = [{'name': 'A', 'value': 2.0}]
    v['model']['nodes'].append({'id': 3, 'dim': 'D2', 'coordinates': [8.0, 0.0, 0.0], 'dofs': ['Ux', 'Uy']})
    v['model']['members'].append({'id': 8, 'nodes': [2, 3], 'element': 'truss2d.linear', 'section': 's', 'law': 'other'})
    v['model']['laws'][0]['parameters'][0]['value'] = e1
    v['model']['laws'].append({'name': 'other', 'parameters': [{'name': 'E', 'value': e2}, {'name': 'rho', 'value': 0.0}]})
    v['model']['supports'] = [{'id': 'base', 'node': 1, 'restrained': ['Ux', 'Uy']},
                              {'id': 'guide', 'node': 2, 'restrained': ['Uy']},
                              {'id': 'guide3', 'node': 3, 'restrained': ['Uy']}]
    v['cases'][0]['loads'] = [{'node': 3, 'values': [{'dof': 'Ux', 'value': 1.0}]}]
    return v
for e1, e2 in [(1e-6, 1e12), (1e12, 1e-6)]:
    r = subprocess.run(['cargo', 'run', '--offline', '--quiet', '-p', 'engine-cli', '--', 'run', '--input', '-'],
                       input=json.dumps(series(e1, e2)), text=True, capture_output=True)
    if r.returncode == 0:
        d = json.loads(r.stdout)['engineResults'][0] if 'engineResults' in r.stdout else json.loads(r.stdout)
        print(f'E1={e1:g} E2={e2:g} exit=0 ux3={d["cases"][0]["nodes"][2]["displacements"][0]!r} esperado={4/(2*e1)+4/(2*e2)!r}')
    else:
        d = json.loads(r.stderr)['engineResults'][0]
        print(f'E1={e1:g} E2={e2:g} exit={r.returncode} status={d["status"]} code={d["diagnostics"][0]["code"]}')
        print(d['diagnostics'][0]['cause'])
PY
```

Salida observada:

```text
E1=1e-06 E2=1e+12 exit=4 status=unstable code=ENG-E-0006
Reference LDLᵀ: singular pivot at DOF 0. Node 2, DOF Ux (global 2).
E1=1e+12 E2=1e-06 exit=0 ux3=2000000.0 esperado=2000000.0
```

- Esperado vs recibido: solución finita δ3 ≈ 2e6 m (exit 0), o un código que
  identifique el mal condicionamiento, vs `ENG-E-0006` (exit 4, inestabilidad).
El mismo defecto cambia de código según el contraste de rigideces: con
contrastes extremos sale `ENG-E-0006` (pivote singular), con contrastes
intermedios `ENG-E-0010` («Inconsistent recovery … Ux»), y con un contraste
realista (2e5 / 2e11 Pa) resuelve bien. Barrido (mismo modelo, F = 1 N):

```sh
python3 - <<'PY'
import copy, json, subprocess
from pathlib import Path
base = json.loads(Path('crates/engine-api/tests/fixtures/val039/cantilever.json').read_text())
def series(e1, e2):
    v = copy.deepcopy(base)
    for n in v['model']['nodes']:
        n['dofs'] = ['Ux', 'Uy']
    v['model']['members'][0]['element'] = 'truss2d.linear'
    v['model']['sections'][0]['properties'] = [{'name': 'A', 'value': 2.0}]
    v['model']['nodes'].append({'id': 3, 'dim': 'D2', 'coordinates': [8.0, 0.0, 0.0], 'dofs': ['Ux', 'Uy']})
    v['model']['members'].append({'id': 8, 'nodes': [2, 3], 'element': 'truss2d.linear', 'section': 's', 'law': 'other'})
    v['model']['laws'][0]['parameters'][0]['value'] = e1
    v['model']['laws'].append({'name': 'other', 'parameters': [{'name': 'E', 'value': e2}, {'name': 'rho', 'value': 0.0}]})
    v['model']['supports'] = [{'id': 'base', 'node': 1, 'restrained': ['Ux', 'Uy']},
                              {'id': 'guide', 'node': 2, 'restrained': ['Uy']},
                              {'id': 'guide3', 'node': 3, 'restrained': ['Uy']}]
    v['cases'][0]['loads'] = [{'node': 3, 'values': [{'dof': 'Ux', 'value': 1.0}]}]
    return v
for e1, e2 in [(1e-6, 1e12), (1e-3, 1e12), (1e-6, 1e9), (1.0, 1e12), (1e-6, 1e6), (2e5, 2e11), (1e12, 1e-6)]:
    r = subprocess.run(['cargo', 'run', '--offline', '--quiet', '-p', 'engine-cli', '--', 'run', '--input', '-'],
                       input=json.dumps(series(e1, e2)), text=True, capture_output=True)
    if r.returncode == 0:
        d = json.loads(r.stdout)
        d = d['engineResults'][0] if 'engineResults' in d else d
        print(f'E1={e1:g} E2={e2:g} exit=0 ux3={d["cases"][0]["nodes"][2]["displacements"][0]!r} teorico={4/(2*e1)+4/(2*e2)!r}')
    else:
        d = json.loads(r.stderr)['engineResults'][0]
        print(f'E1={e1:g} E2={e2:g} exit={r.returncode} code={d["diagnostics"][0]["code"]} cause={d["diagnostics"][0]["cause"]}')
PY
```

Salida observada:

```text
E1=1e-06 E2=1e+12 exit=4 code=ENG-E-0006 cause=Reference LDLᵀ: singular pivot at DOF 0. Node 2, DOF Ux (global 2).
E1=0.001 E2=1e+12 exit=4 code=ENG-E-0006 cause=Reference LDLᵀ: singular pivot at DOF 0. Node 2, DOF Ux (global 2).
E1=1e-06 E2=1e+09 exit=4 code=ENG-E-0006 cause=Reference LDLᵀ: singular pivot at DOF 0. Node 2, DOF Ux (global 2).
E1=1 E2=1e+12 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Ux: relative residual 3.338733354170781e-5 > 1e-9.
E1=1e-06 E2=1e+06 exit=2 code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Ux: relative residual 9.955057888669192e-6 > 1e-9.
E1=200000 E2=2e+11 exit=0 ux3=1.0000009998390996e-05 teorico=1.0000010000000002e-05
E1=1e+12 E2=1e-06 exit=0 ux3=2000000.0 teorico=2000000.0
```

- Estado inicial (MAM-851): no se corrigió; se reportó para decisión.

#### H4 · Corrección (MAM-921, 2026-10-29)

Causa raíz: con k₁ = E₁A/L = 5e-7 y k₂ = 5e11, la diagonal global del nudo 2
Ux es k₁ + k₂ y en f64 `5e11 + 5e-7 == 5e11`: la rigidez de la barra blanda se
pierde por redondeo al ensamblar. La K ensamblada es numéricamente singular,
así que una solución finita no es alcanzable con esa matriz; el defecto era de
clasificación (se informaba un mecanismo inexistente).

Cambio: el ensamblaje guarda, por DOF global, la menor diagonal positiva de
elemento. Cuando el backend devuelve `ENG-E-0006` en un DOF cuya contribución
de elemento c cumple c ≤ 64 ε max(n,1) |K_gg| (la misma constante 64 ε max(n,1)
que el pivote de referencia, pero aplicada a otra magnitud: el backend la compara
con el pivote de la matriz equilibrada S·A·S y la reclasificación con el cociente
bruto c/|K_gg|, así que las dos fronteras no coinciden; ε = `f64::EPSILON`,
n = orden de K reducida), el sistema
devuelve `ENG-E-0010` (invalid, exit 2) «Ill-conditioned stiffness …» con nudo y
DOF global. Sin absorción el diagnóstico sigue siendo `ENG-E-0006`. No cambia
ningún valor de K, M ni Kg ni el orden de operaciones.

Barrido repetido con el mismo comando tras el cambio:

```text
E1=1e-06 E2=1e+12 exit=2 status=invalid code=ENG-E-0010 cause=… Ill-conditioned stiffness: element stiffness 5e-7 at node 2, DOF Ux (global 2) is below f64 resolution of the assembled diagonal 5e11 (contrast 9.999999999999999e-19); …
E1=0.001 E2=1e+12 exit=2 status=invalid code=ENG-E-0010 cause=… element stiffness 5e-4 … assembled diagonal 5.000000000000005e11 (contrast 9.99999999999999e-16); …
E1=1e-06 E2=1e+09 exit=2 status=invalid code=ENG-E-0010 cause=… element stiffness 5e-7 … assembled diagonal 5.000000000000005e8 (contrast 9.99999999999999e-16); …
E1=1 E2=1e+12 exit=2 status=invalid code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Ux: relative residual 3.338733354170781e-5 > 1e-9.
E1=1e-06 E2=1e+06 exit=2 status=invalid code=ENG-E-0010 cause=Inconsistent recovery at node 2 / Ux: relative residual 9.955057888669192e-6 > 1e-9.
E1=200000 E2=2e+11 exit=0 ux3=1.0000009998390996e-05 teorico=1.0000010000000002e-05
E1=1e+12 E2=1e-06 exit=0 ux3=2000000.0 teorico=2000000.0
```

Margen (n = 2): umbral 64 ε · 2 ≈ 2.84e-14. Los contrastes 1e-15 quedan ≈ 28
veces por debajo; el contraste 1e-12 (E₁ = 1) queda ≈ 35 veces por encima y
sigue en `ENG-E-0010` «Inconsistent recovery» (sin cambio).

Banda residual de `ENG-E-0006` (limitación conocida, revisión J4 r1, H1). Barrido
propio con el mismo modelo, contraste = E₁/E₂ en pasos de 1e-17, binario del
head de este PR:

| E₂ | 2.700e-14 … 2.950e-14 | [1.000e-14, 2.842e-14]: casos que no dan «Ill-conditioned» |
| --- | --- | --- |
| 1e12 | «Ill-conditioned» hasta 2.842e-14; `ENG-E-0006` en 2.843e-14 … 2.862e-14; «Inconsistent recovery» desde 2.863e-14 | 0 de 1843 |
| 2e11 | «Ill-conditioned» hasta 2.830e-14; «Inconsistent recovery» desde 2.831e-14; sin `ENG-E-0006` | 12 de 1843 (2.831e-14 … 2.842e-14, «Inconsistent recovery») |
| 1e9 | `ENG-E-0006` en 2.843e-14 y en 2.856e-14 … 2.878e-14; «Inconsistent recovery» desde 2.879e-14 | 407 de 1843 en `ENG-E-0006`, el primero en 1.056e-14 |
| 1e6 | alterna «Ill-conditioned» / `ENG-E-0006` desde 2.719e-14; «Inconsistent recovery» desde 2.858e-14 | 431 de 1843 en `ENG-E-0006`, el primero en 1.077e-14 |

En la franja alrededor del umbral los números coinciden con los de J4. Por
debajo del umbral, sin embargo, mido una banda más ancha que J4 no informó: con
E₂ = 1e9 y 1e6, cerca de un 22 % de los contrastes entre 1e-14 y 2.84e-14 siguen
dando `ENG-E-0006`. En esos casos la contribución de la barra blanda no se pierde
del todo al sumar (p. ej. E₂ = 1e9, contraste 2e-14: (k₁+k₂)−k₂ = 1.0014e-5 frente
a k₁ = 1e-5) y el backend señala el pivote del nudo 3 Ux («singular pivot at DOF 1.
Node 3, DOF Ux (global 4)»). Ese nudo solo recibe la barra rígida, así que no hay
absorción en el DOF señalado y no se reclasifica. Causa: el criterio solo examina
el DOF que nombra el backend. Queda como limitación; no se cambia el criterio en
este PR. Por encima de ≈2.88e-14 todos los casos medidos dan «Inconsistent
recovery» (`ENG-E-0010`).
Un modelo con un mecanismo real y además una rigidez absorbida en el mismo DOF
singular se clasificaría como mal condicionamiento: DESCONOCIDO si ocurre en
modelos reales.

Alcance (J4 r1, H3): la reclasificación vive en el traductor común de errores
del sistema, así que también se aplica a modos y a la factorización de la
tangente pDelta / no lineal, siempre con la K lineal (diagonales de elemento y
K_gg). Un pandeo real en un DOF con rigidez absorbida pasaría de `ENG-E-0006` a
`ENG-E-0010`, en contra de `docs/METHODS_GUIDE.md` («If the tangent is singular,
unstable/ENG-E-0006»). No se ha probado: DESCONOCIDO si ocurre en modelos reales.
La columna supercrítica de VAL-039 sigue en `ENG-E-0006`.

Mensaje (J4 r1, H4): en la CLI la causa empieza por el texto genérico del
registro para `ENG-E-0010` («The specified dimension is not valid for the
operation.») seguido de «Ill-conditioned stiffness …». Es cosmético: el código,
el exit y el outcome del registro no cambian y el catálogo no se toca. «Inconsistent
recovery» ya usaba `ENG-E-0010` con el mismo prefijo.

Pruebas: `h4_soft_bar_next_to_support_reports_ill_conditioning_not_mechanism`
(los tres contrastes absorbidos dan `ENG-E-0010` «Ill-conditioned»),
`true_mechanism_still_reports_singular_system` (mecanismo real sigue en
`ENG-E-0006`), `extreme_moduli_in_series_have_balanced_response_or_clean_rejection`
(ahora exige `ENG-E-0010`) y, en `engine-assembly`,
`absorbed_element_diagonal_reports_ill_conditioning_with_global_entities`.
Mutación (r1): anular la reclasificación hizo fallar esas tres pruebas de
regresión (2 de 12 en val039_pathological, 1 de 11 en assembly).

Umbral fijado (J4 r1, H2): `h4_reclassification_threshold_is_pinned_on_both_sides`
usa E₂ = 1e12 con contraste 2.0e-14 («Ill-conditioned»), 2.85e-14 (banda
residual, sigue en `ENG-E-0006`) y 4.0e-14 («Inconsistent recovery»). Las
mutaciones de la tolerancia ×0.5, ×2 y ×1e6 hacen fallar esa prueba (1 de 13 en
val039_pathological); assembly sigue 11 de 11. Con el cambio,
`cargo test --offline --workspace` pasa completo: 889 passed, 0 failed
(guard de determinismo 37/37).

## Qué NO se probó · DESCONOCIDO

- DESCONOCIDO: exhaustividad del espacio de entradas, otras topologías,
  modelos D3, liberaciones, diafragmas, resortes, no linealidad material y
  matrices de mayor tamaño. No se ejecutó fuzzing ni una campaña de recursos.
- DESCONOCIDO: límites universales de condicionamiento o longitud; barridos
  completos de E, A e I por separado, overflow aritmético posterior al parseo,
  underflow y cambios simultáneos de geometría, masa y unidades geotécnicas.
- DESCONOCIDO: repetibilidad entre arquitecturas, compiladores o backends;
  la evidencia se limita al backend disponible y ejecuciones iguales aquí.
- DESCONOCIDO: carga crítica discreta exacta y convergencia de malla; esta
  prueba solo exige rechazo claro por encima de la crítica continua propia.
- DESCONOCIDO: tratamiento específico correcto de unidades desconocidas y
  entradas no finitas; H1–H2 solo documentan la salida actual.

## Verificación

Ejecutado por el helper de J2 fuera del sandbox, `cargo` vía el serializador del
target compartido, `--offline`:

- `cargo test -p engine-api --test val039_pathological`: 10 aprobadas.
- `cargo test -p engine-cli --test val039_cli`: 3 aprobadas.
- `cargo test --workspace`: todas aprobadas (871 pruebas, 0 fallos), incluido el
  guardia `engine-linalg/tests/determinism_guard.rs`
  (`engine_crates_use_only_the_central_scalar_module`: ok).
- `cargo fmt --all -- --check`: limpio.
- `cargo clippy --workspace --all-targets -- -D warnings`: OK.
- `bash clean-room/check.sh origin/main clean-room`: `Clean-room: OK`.
- `bash scripts/changelog-check.sh origin/main`: OK.
- `git diff --check`: sin salida.

Funciones escalares: las pruebas usan `engine_core::math::{sqrt, powi}`; el
archivo de soporte, incluido también por las pruebas de `engine-cli` (sin
dependencia de `engine-core`), usa productos en lugar de `powi`. Control: volver
a escribir `20.0_f64.sqrt()` hace fallar el guardia (36 aprobadas, 1 fallo).

### Mutaciones

Cada mutación se aplicó sobre el archivo, se ejecutó la suite indicada y se
restauró el archivo original (comprobado con `cmp`); tras restaurar, 10/10 y
3/3 aprobadas.

| # | Archivo | Cambio exacto | Suite | Resultado |
| --- | --- | --- | --- | --- |
| M1 | `crates/engine-api/tests/val039_pathological.rs` | primer `"ENG-E-0006"` → `"ENG-E-0007"` | val039_pathological | 1 falla (`supercritical_pdelta_columns_report_instability_without_responses`), 9 pasan |
| M2 | `crates/engine-api/tests/val039_pathological.rs` | `math::sqrt(20.0)` → `math::sqrt(21.0)` (frecuencia de modos repetidos) | val039_pathological | 1 falla (`repeated_modes_in_orthogonal_directions_are_finite_and_deterministic`), 9 pasan |
| M3 | `crates/engine-cli/tests/val039_cli.rs` | `rejected(&input.to_string(), code, 4)` → `…, 2)` | val039_cli | 1 falla (`mechanisms_massless_mechanism_and_supercritical_column_exit_four`), 2 pasan |
| M4 | `crates/engine-cli/tests/val039_cli.rs` | `rejected("{", "API-E-INPUT", 2)` → `"CLI-E-DOCUMENT"` | val039_cli | 1 falla (`malformed_json_and_missing_model_exit_two_with_input_structure_code`), 2 pasan |
| M5 | `crates/engine-api/tests/val039_pathological.rs` | δ en serie, línea 310: `4.0 / (2.0 * e2)` → `4.0 / (3.0 * e2)` (no la línea 291, barras ortogonales) | val039_pathological | 1 falla (`extreme_moduli_in_series_have_balanced_response_or_clean_rejection`), 9 pasan |
| M6 | `crates/engine-api/tests/val039_pathological.rs` | `assert!(e1 < e2, …)` → `assert!(e1 > e2, …)` | val039_pathological | 1 falla (`extreme_moduli_in_series_have_balanced_response_or_clean_rejection`), 9 pasan |

Variante descartada de M5: cambiar el término `e1` (`2.0 * e1` → `3.0 * e1`)
sobrevive, porque en el orden que resuelve (E₁ = 1e12) ese término vale ~1e-12
frente a 2e6, por debajo de la tolerancia 1e-10. No indica una prueba vacua.

No hubo bloqueos de ejecución por el sandbox.
