#!/usr/bin/env python3
"""Derivas espectrales compartidas por Engine y oráculos, solo stdlib.

Uso: python3 postproc_derivas.py entrada.json [-o salida.json] | --selftest
Entrada neutral val032-modal/1 (SI, Sa elástica en m/s², Rz en radianes)::

  {"schema":"val032-modal/1", "xi":0.05, "directions":["X","Y"],
   "floors":[{"id":1,"cm":[8.25,5.75,3.0]}, ...],
   "corners_xy":[[0,0],[15,10]],
   "Sa_table":[[0,4.707192], ...],
   "modes":[{"n":1,"T_n":0.52,"Gamma_n":{"X":1.2,"Y":0.3},
             "phi_n":[{"Ux":0.1,"Uy":0.2,"Rz":0.01}, ...]}, ...],
   "directional_combination":"srss", "directional_factors":[1,1]}

floors: orden ascendente, base empotrada en z=0; phi_n: mismo orden, una
terna por maestro. Gamma_n y phi_n deben usar la MISMA normalización modal;
se conservan sus signos. corners_xy son dos esquinas opuestas del rectángulo
(extremos de x e y), alineadas verticalmente, comunes a todos los pisos.
Sa_table: exactamente la tabla pairs de los fixtures/CSV, sin factores I/R.
xi: amortiguamiento igual para todos los modos. Interpolación lineal, sin
extrapolación. directions selecciona excitaciones X/Y; ambas componentes de
respuesta Ux/Uy se reportan para cada excitación. Los dos campos directional_*
son opcionales y juntos solicitan además SRSS entre excitaciones (después de
CQC/SRSS modal); factores en el mismo orden que directions.

Salida val032-drifts/1: directions[excitación].floors[], CM y corners con
Ux/Uy.{CQC_m,SRSS_m,CQC_ratio,SRSS_ratio}; ratio = deriva / altura de piso.
modal_drifts_m conserva las contribuciones firmadas por modo y componente.
displacements: CM/corners con desplazamientos de nivel (CQC_m/SRSS_m).
Ax[componente][CQC|SRSS] = (delta_max/(1.2*delta_prom))², usando los picos de
DESPLAZAMIENTO DE NIVEL en esquinas; delta_prom = media de los dos picos.
Ax_from_drifts es el cociente análogo de derivas, auxiliar explícito. Se usa la componente
en la dirección de excitación para la revisión NEC §6.3.7; las componentes
cruzadas se entregan también. Ax=null si ambas esquinas dan cero (0/0), sin
aplicar límites o factores reglamentarios adicionales. directional_srss tiene
la misma salida combinada, sin contribuciones modales firmadas entre ejes.

Delta_i,n = (phi_n(i)-phi_n(i-1))*Gamma_n*Sa(T_n)/omega_n² en CM.
En cada esquina se transforma ANTES de restar y combinar:
u=u_m-(y-yc)*Rz; v=v_m+(x-xc)*Rz. En el extremo inferior se usa el CM del
piso inferior en la misma vertical; en la base todos los movimientos son cero.
CQC usa Der Kiureghian, xi igual: con b=omega_j/omega_i,
rho_ij=8*xi²*(1+b)*b**1.5/((1-b²)²+4*xi²*b*(1+b)²).
La forma con min(omega)/max(omega) es simétrica y evita cocientes grandes.
No se restan máximos de desplazamientos ya combinados.
"""
import argparse
import bisect
import copy
import json
import math
from pathlib import Path

COMPONENTS = ('Ux', 'Uy')


def finite(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def interpolate(table, period):
    require(table[0][0] <= period <= table[-1][0], 'T_n fuera de la tabla Sa; extrapolación prohibida')
    i = bisect.bisect_left([row[0] for row in table], period)
    if table[i][0] == period:
        return table[i][1]
    t0, a0 = table[i-1]
    t1, a1 = table[i]
    return a0 + (a1-a0)*(period-t0)/(t1-t0)


def correlation(ti, tj, xi):
    if ti == tj:
        return 1.0
    b = min(ti, tj) / max(ti, tj)
    return 8*xi*xi*(1+b)*b**1.5 / ((1-b*b)**2 + 4*xi*xi*b*(1+b)**2)


def combine(values, rho):
    srss2 = math.fsum(v*v for v in values)
    cqc2 = math.fsum(rho[i][j]*a*b for i, a in enumerate(values) for j, b in enumerate(values))
    require(cqc2 >= -1e-12*srss2, 'Forma cuadrática CQC negativa')
    return {'CQC_m': math.sqrt(max(0.0, cqc2)), 'SRSS_m': math.sqrt(srss2)}


def ratios(response, height):
    return dict(response, CQC_ratio=response['CQC_m']/height, SRSS_ratio=response['SRSS_m']/height)


def ax(corners):
    result = {}
    for component in COMPONENTS:
        result[component] = {}
        for method in ('CQC', 'SRSS'):
            values = [c[component][method+'_m'] for c in corners]
            mean = math.fsum(values)/2
            result[component][method] = (max(values)/(1.2*mean))**2 if mean else None
    return result


def process(data):
    require(data.get('schema') == 'val032-modal/1', 'schema debe ser val032-modal/1')
    xi = data['xi']
    require(finite(xi) and 0 <= xi < 1, 'xi debe estar en [0,1)')
    directions, floors, modes, table = (data[k] for k in ('directions', 'floors', 'modes', 'Sa_table'))
    require(directions and len(set(directions)) == len(directions) and all(d in ('X','Y') for d in directions), 'directions: X/Y únicos')
    require(floors and modes and len({m['n'] for m in modes}) == len(modes), 'Pisos y modos no vacíos, n únicos')
    require(len({f['id'] for f in floors}) == len(floors), 'id de piso repetido')
    previous_z = 0
    for floor in floors:
        cm = floor['cm']
        require(len(cm) == 3 and all(finite(v) for v in cm) and cm[2] > previous_z, 'CM finito y pisos sobre base z=0 en orden ascendente')
        previous_z = cm[2]
    corners = data['corners_xy']
    require(len(corners) == 2 and all(len(c) == 2 and all(finite(v) for v in c) for c in corners), 'Dos esquinas XY finitas')
    require(all(corners[0][j] != corners[1][j] for j in (0,1)), 'Esquinas opuestas del rectángulo')
    require(len(table) >= 2 and all(len(r) == 2 and all(finite(v) for v in r) and r[0] >= 0 and r[1] >= 0 for r in table), 'Tabla Sa inválida')
    require(all(a[0] < b[0] for a, b in zip(table, table[1:])), 'Tabla Sa con T estrictamente creciente')
    for mode in modes:
        require(finite(mode['T_n']) and mode['T_n'] > 0, 'T_n debe ser positivo')
        require(len(mode['phi_n']) == len(floors), 'Una terna phi_n por piso')
        require(all(finite(p[c]) for p in mode['phi_n'] for c in ('Ux','Uy','Rz')), 'phi_n no finita')
        require(all(finite(mode['Gamma_n'][d]) for d in directions), 'Gamma_n no finita')
    rho = [[correlation(a['T_n'], b['T_n'], xi) for b in modes] for a in modes]
    spectral = [interpolate(table, m['T_n'])/(2*math.pi/m['T_n'])**2 for m in modes]
    result = {'schema':'val032-drifts/1', 'units':'SI', 'xi':xi, 'mode_numbers':[m['n'] for m in modes], 'directions':{}}
    for direction in directions:
        rows = []
        for i, floor in enumerate(floors):
            height = floor['cm'][2] - (floors[i-1]['cm'][2] if i else 0)
            points, displacement_points = [], []
            for point in [None] + corners:
                modal_values = {c:[] for c in COMPONENTS}
                modal_displacements = {c:[] for c in COMPONENTS}
                for n, mode in enumerate(modes):
                    top = mode['phi_n'][i]
                    bottom = mode['phi_n'][i-1] if i else dict(Ux=0, Uy=0, Rz=0)
                    delta = {c: top[c]-bottom[c] for c in COMPONENTS}
                    displacement = {c: top[c] for c in COMPONENTS}
                    if point is not None:
                        x, y = point
                        xc, yc = floor['cm'][:2]
                        xb, yb = floors[i-1]['cm'][:2] if i else (xc, yc)
                        delta['Ux'] -= (y-yc)*top['Rz'] - (y-yb)*bottom['Rz']
                        delta['Uy'] += (x-xc)*top['Rz'] - (x-xb)*bottom['Rz']
                        displacement['Ux'] -= (y-yc)*top['Rz']
                        displacement['Uy'] += (x-xc)*top['Rz']
                    scale = mode['Gamma_n'][direction] * spectral[n]
                    for c in COMPONENTS:
                        modal_values[c].append(delta[c]*scale)
                        modal_displacements[c].append(displacement[c]*scale)
                point_result = {c: ratios(combine(modal_values[c], rho), height) for c in COMPONENTS}
                point_result['modal_drifts_m'] = modal_values
                disp_result = {c:combine(modal_displacements[c], rho) for c in COMPONENTS}
                disp_result['modal_displacements_m'] = modal_displacements
                if point is not None:
                    point_result['xy'] = point
                    disp_result['xy'] = point
                points.append(point_result)
                displacement_points.append(disp_result)
            rows.append({'id':floor['id'], 'height_m':height, 'CM':points[0], 'corners':points[1:],
                'displacements':{'CM':displacement_points[0], 'corners':displacement_points[1:]},
                'Ax':ax(displacement_points[1:]), 'Ax_from_drifts':ax(points[1:])})
        result['directions'][direction] = {'floors':rows}
    if 'directional_combination' in data or 'directional_factors' in data:
        require(data.get('directional_combination') == 'srss', 'Solo SRSS direccional')
        factors = data['directional_factors']
        require(len(factors) == len(directions) and all(finite(f) and f >= 0 for f in factors), 'Factores direccionales inválidos')
        rows = []
        for i, floor in enumerate(floors):
            inputs = [result['directions'][d]['floors'][i] for d in directions]
            points = []
            for j in range(3):
                inputs_point = [r['CM'] if j == 0 else r['corners'][j-1] for r in inputs]
                point_result = {c:ratios({method+'_m':math.sqrt(math.fsum((f*p[c][method+'_m'])**2 for f, p in zip(factors, inputs_point))) for method in ('CQC','SRSS')}, inputs[0]['height_m']) for c in COMPONENTS}
                if j:
                    point_result['xy'] = corners[j-1]
                points.append(point_result)
            displacement_points = []
            for j in range(3):
                inputs_point = [r['displacements']['CM'] if j == 0 else r['displacements']['corners'][j-1] for r in inputs]
                disp_result = {c:{method+'_m':math.sqrt(math.fsum((f*p[c][method+'_m'])**2 for f,p in zip(factors,inputs_point))) for method in ('CQC','SRSS')} for c in COMPONENTS}
                if j:
                    disp_result['xy'] = corners[j-1]
                displacement_points.append(disp_result)
            rows.append({'id':floor['id'], 'height_m':inputs[0]['height_m'], 'CM':points[0], 'corners':points[1:],
                'displacements':{'CM':displacement_points[0], 'corners':displacement_points[1:]},
                'Ax':ax(displacement_points[1:]), 'Ax_from_drifts':ax(points[1:])})
        result['directional_srss'] = {'directions':directions, 'factors':factors, 'floors':rows}
    return result


def selftest():
    # Cálculo a mano: omega=(1,2), Sa=(8,4), Gamma_X=(1,2),
    # escalas X=(8,2). Derivas Ux CM pisos 1/2: (8,4)/(16,2).
    # rho12=2*sqrt(2)/153 para xi=1/20 y beta=1/2.
    data = dict(schema='val032-modal/1', xi=0.05, directions=['X','Y'],
        floors=[dict(id=1, cm=[0,0,3]), dict(id=2, cm=[0,0,6])],
        corners_xy=[[-2,-1],[2,1]], Sa_table=[[0,2],[math.pi,4],[2*math.pi,8]],
        modes=[dict(n=1, T_n=2*math.pi, Gamma_n=dict(X=1,Y=2), phi_n=[dict(Ux=1,Uy=2,Rz=0.1),dict(Ux=3,Uy=5,Rz=0.3)]),
               dict(n=2, T_n=math.pi, Gamma_n=dict(X=2,Y=-1), phi_n=[dict(Ux=2,Uy=-1,Rz=-0.2),dict(Ux=3,Uy=1,Rz=-0.1)])],
        directional_combination='srss', directional_factors=[1,1])
    answer = process(data)
    r = 2*math.sqrt(2)/153
    def close(a, b):
        assert math.isclose(a, b, rel_tol=1e-12, abs_tol=1e-12), (a, b)
    close(correlation(2*math.pi, math.pi, .05), r)
    close(interpolate(data['Sa_table'], 1.5*math.pi), 6)
    x1, x2 = answer['directions']['X']['floors']
    close(x1['CM']['Ux']['CQC_m'], math.sqrt(80+64*r))
    close(x1['CM']['Ux']['SRSS_m'], math.sqrt(80))
    close(x2['CM']['Ux']['CQC_m'], math.sqrt(260+64*r))
    close(x2['CM']['Ux']['SRSS_m'], math.sqrt(260))
    # Esquinas Ux del piso 1: (8.8,3.6), (7.2,4.4).
    edges = [math.sqrt(8.8**2+3.6**2+2*8.8*3.6*r), math.sqrt(7.2**2+4.4**2+2*7.2*4.4*r)]
    for edge, expected in zip(x1['corners'], edges):
        close(edge['Ux']['CQC_m'], expected)
    close(x1['Ax']['Ux']['CQC'], (max(edges)/(1.2*sum(edges)/2))**2)
    # Nivel 2: desplazamientos esquina Ux=(26.4,5.8)/(21.6,6.2).
    level_edges = [math.sqrt(26.4**2+5.8**2+2*26.4*5.8*r), math.sqrt(21.6**2+6.2**2+2*21.6*6.2*r)]
    close(x2['Ax']['Ux']['CQC'], (max(level_edges)/(1.2*sum(level_edges)/2))**2)
    close(x2['displacements']['CM']['Ux']['CQC_m'], math.sqrt(612+288*r))
    drift_edges = [math.sqrt(17.6**2+2.2**2+2*17.6*2.2*r), math.sqrt(14.4**2+1.8**2+2*14.4*1.8*r)]
    close(x2['Ax_from_drifts']['Ux']['CQC'], (max(drift_edges)/(1.2*sum(drift_edges)/2))**2)
    # Esquina 0, Uy: (14.4,-1.2); piso 2 esquina 1 Ux: (14.4,1.8).
    close(x1['corners'][0]['Uy']['CQC_m'], math.sqrt(14.4**2+1.2**2-2*14.4*1.2*r))
    close(x2['corners'][1]['Ux']['CQC_m'], math.sqrt(14.4**2+1.8**2+2*14.4*1.8*r))
    # Y: escalas=(16,-1), Ux CM piso 1=(16,-2).
    y = math.sqrt(260-64*r)
    close(answer['directions']['Y']['floors'][0]['CM']['Ux']['CQC_m'], y)
    close(answer['directional_srss']['floors'][0]['CM']['Ux']['CQC_m'], math.sqrt(340))
    close(x1['CM']['Ux']['CQC_ratio'], math.sqrt(80+64*r)/3)
    # Invariancia al signo del autovector, correlación repetida y xi=0.
    flipped = copy.deepcopy(data)
    for p in flipped['modes'][1]['phi_n']:
        for c in ('Ux','Uy','Rz'):
            p[c] *= -1
    for d in ('X','Y'):
        flipped['modes'][1]['Gamma_n'][d] *= -1
    assert process(flipped) == answer
    # CM distintos entre pisos: esquina fija en la misma vertical.
    shifted = copy.deepcopy(data)
    shifted['floors'][0]['cm'] = [1,2,3]
    shifted['floors'][1]['cm'] = [2,3,6]
    shifted_rows = process(shifted)['directions']['X']['floors']
    close(shifted_rows[0]['corners'][0]['Ux']['CQC_m'], math.sqrt(10.4**2+2.8**2+2*10.4*2.8*r))
    close(shifted_rows[1]['corners'][0]['Ux']['CQC_m'], math.sqrt(23.2**2+2.4**2+2*23.2*2.4*r))
    close(correlation(1, 1, 0), 1)
    close(correlation(1, 2, 0), 0)
    close(combine([2,-1], [[1,1],[1,1]])['CQC_m'], 1)
    assert ax([dict(Ux=dict(CQC_m=0,SRSS_m=0),Uy=dict(CQC_m=0,SRSS_m=0))]*2)['Ux']['CQC'] is None
    try:
        interpolate(data['Sa_table'], 7)
    except ValueError:
        pass
    else:
        raise AssertionError('Debe rechazar extrapolación')
    print('selftest OK: 2 modos, CM/esquinas, CQC/SRSS, Ax, SRSS direccional y signos')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('input', nargs='?', help='JSON neutral val032-modal/1')
    parser.add_argument('-o', '--output', help='JSON de salida (por defecto stdout)')
    parser.add_argument('--selftest', action='store_true')
    args = parser.parse_args()
    if args.selftest:
        selftest()
        return
    if not args.input:
        parser.error('Falta entrada.json o --selftest')
    try:
        output = json.dumps(process(json.loads(Path(args.input).read_text(encoding='utf-8'))), ensure_ascii=False, indent=2, allow_nan=False)+'\n'
    except (ValueError, KeyError, TypeError, IndexError) as exc:
        parser.error(str(exc))
    if args.output:
        Path(args.output).write_text(output, encoding='utf-8')
    else:
        print(output, end='')


if __name__ == '__main__':
    main()
