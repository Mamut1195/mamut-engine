"""Referencia propia NumPy: E3 P-Delta con pretensión inicial congelada.

assemble deriva de val032_calibrate.py (rama local de VAL-032, ver README).
La extensión integra N*(dv/dx)^2 con Hermite y resuelve directamente K+Kg.
No ejecuta ni importa el Engine; solo lee el fixture E3 de main. SI, float64, sin red.
"""
import hashlib
import json
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent

DOFS = ['Ux', 'Uy', 'Uz', 'Rx', 'Ry', 'Rz']


def assemble(model):
    coords = {n['id']: np.array(n['coordinates'], dtype=float) for n in model['nodes']}
    ids = sorted(coords)
    ni = {n: i for i, n in enumerate(ids)}
    nd = 6 * len(ids)
    sections = {s['id']: {p['name']: p['value'] for p in s['properties']} for s in model['sections']}
    laws = {s['name']: {p['name']: p['value'] for p in s['parameters']} for s in model['laws']}
    stiffness = np.zeros((nd, nd))
    for bar in model['members']:
        a, b = bar['nodes']
        d = coords[b] - coords[a]
        length = np.linalg.norm(d)
        x = d / length
        sec, law = sections[bar['section']], laws[bar['law']]
        edofs = [6*ni[n]+j for n in (a, b) for j in range(6)]
        if bar['element'] == 'truss3d.linear':
            kg = np.zeros((12, 12))
            block = law['E'] * sec['A'] / length * np.outer(x, x)
            kg[:3, :3] = block
            kg[6:9, 6:9] = block
            kg[:3, 6:9] = -block
            kg[6:9, :3] = -block
        else:
            y = np.cross(bar['orientation'], x)
            y /= np.linalg.norm(y)
            z = np.cross(x, y)
            transform = np.kron(np.eye(4), np.array([x, y, z]))
            local = np.zeros((12, 12))
            for p, q, value in [(0, 6, law['E']*sec['A']/length), (3, 9, law['G']*sec['J']/length)]:
                local[p, p] += value
                local[q, q] += value
                local[p, q] -= value
                local[q, p] -= value
            for dofs, sign, ei in [([1,5,7,11], 1, law['E']*sec['Iz']), ([2,4,8,10], -1, law['E']*sec['Iy'])]:
                l = length
                block = ei/l**3 * np.array([[12,6*l*sign,-12,6*l*sign], [6*l*sign,4*l*l,-6*l*sign,2*l*l], [-12,-6*l*sign,12,-6*l*sign], [6*l*sign,2*l*l,-6*l*sign,4*l*l]])
                local[np.ix_(dofs, dofs)] += block
            kg = transform.T @ local @ transform
        stiffness[np.ix_(edofs, edofs)] += kg
    fixed = {6*ni[s['node']]+DOFS.index(d) for s in model['supports'] for d in s['restrained']}
    slaves = {s: dia['master'] for dia in model.get('diaphragms', []) for s in dia['slaves']}
    free = [6*ni[n]+j for n in ids for j in range(6) if 6*ni[n]+j not in fixed and not (n in slaves and j in (0,1,5))]
    ri = {d: i for i, d in enumerate(free)}
    constraint = np.zeros((nd, len(free)))
    for d, i in ri.items():
        constraint[d, i] = 1
    for s, master in slaves.items():
        dx, dy, _ = coords[s] - coords[master]
        ux, uy, rz = [ri[6*ni[master]+j] for j in (0,1,5)]
        constraint[6*ni[s], ux] = 1
        constraint[6*ni[s], rz] = -dy
        constraint[6*ni[s]+1, uy] = 1
        constraint[6*ni[s]+1, rz] = dx
        constraint[6*ni[s]+5, rz] = 1
    mass = np.zeros(nd)
    for item in model['masses']:
        for value in item['values']:
            mass[6*ni[item['node']]+DOFS.index(value['dof'])] += value['value']
    kr = constraint.T @ stiffness @ constraint
    mr = constraint.T @ (mass[:, None]*constraint)
    return dict(coords=coords, ids=ids, ni=ni, nd=nd, fixed=fixed,
                constraint=constraint, stiffness=stiffness, kr=kr, mr=mr)



def force(ref, loads):
    result = np.zeros(ref['nd'])
    for load in loads:
        for value in load['values']:
            result[6*ref['ni'][load['node']]+DOFS.index(value['dof'])] += value['value']
    return result


def geometric(model, ref, gravity):
    """Segunda variación: N integral H'_a H'_b dx, N=EA/L*(u_b-u_a).

    Tres puntos Gauss integran exactamente el polinomio de grado cuatro.
    No se aproxima Kg por un resorte P/L ni se actualiza N bajo carga lateral.
    """
    sections = {s['id']: {p['name']: p['value'] for p in s['properties']} for s in model['sections']}
    laws = {s['name']: {p['name']: p['value'] for p in s['parameters']} for s in model['laws']}
    matrix = np.zeros_like(ref['stiffness'])
    points, weights = np.polynomial.legendre.leggauss(3)
    axial = []
    for bar in model['members']:
        assert bar['element'] == 'frame3d.euler_elastic'
        assert not bar.get('releases')
        a, b = bar['nodes']
        d = ref['coords'][b] - ref['coords'][a]
        length = np.linalg.norm(d)
        x = d / length
        y = np.cross(bar['orientation'], x)
        y /= np.linalg.norm(y)
        z = np.cross(x, y)
        rotation = np.array([x, y, z])
        transform = np.kron(np.eye(4), rotation)
        edofs = [6*ref['ni'][n]+j for n in (a,b) for j in range(6)]
        ug = transform @ gravity[edofs]
        normal = laws[bar['law']]['E'] * sections[bar['section']]['A'] / length * (ug[6]-ug[0])
        axial.append(float(normal))
        bending = np.zeros((4, 4))
        for point, weight in zip(points, weights):
            s = (point+1)/2
            dh = np.array([(-6*s+6*s*s)/length, 1-4*s+3*s*s,
                           (6*s-6*s*s)/length, -2*s+3*s*s])
            bending += normal * length * weight/2 * np.outer(dh, dh)
        local = np.zeros((12, 12))
        for dofs, signs in [([1,5,7,11], [1,1,1,1]), ([2,4,8,10], [1,-1,1,-1])]:
            local[np.ix_(dofs, dofs)] = bending * np.outer(signs, signs)
        matrix[np.ix_(edofs, edofs)] += transform.T @ local @ transform
    return matrix, axial


def main():
    source = (HERE.parents[4]/'docs/validation-report/fixtures/val032/E3-v1.1.1.json').read_bytes()
    runs = [run for run in json.loads(source)['runs'] if run['id'] == 'E3-pdelta']
    assert len(runs) == 1
    document = runs[0]['engine_input']
    model = document['model']
    options = document['analyses'][0]['options']
    ref = assemble(model)
    t, kr = ref['constraint'], ref['kr']
    cases = {c['name']: c['loads'] for c in document['cases']}
    fg = options['gravity']['factor'] * force(ref, cases[options['gravity']['case']])
    fl = options['lateral']['factor'] * force(ref, cases[options['lateral']['case']])
    # El fixture no tiene asentamientos; todos los apoyos son homogéneos.
    assert all(not s.get('prescribed') for s in model['supports'])
    ug = t @ np.linalg.solve(kr, t.T @ fg)
    kg, axial = geometric(model, ref, ug)
    kgr = t.T @ kg @ t
    total = kr + kgr
    eigmin = float(np.linalg.eigvalsh(total)[0])
    assert eigmin > 0
    q = np.linalg.solve(total, t.T @ fl)
    u = t @ q
    linear = t @ np.linalg.solve(kr, t.T @ fl)
    residual = float(np.max(np.abs(t.T @ ((ref['stiffness']+kg) @ u-fl))) / np.max(np.abs(t.T @ fl)))
    assert residual < 1e-10
    # P_i y V_i acumulan las cargas EX/G desde el piso i hasta cubierta.
    floors = sorted(model['diaphragms'], key=lambda d: ref['coords'][d['master']][2])
    rows = []
    last_z = last_u = last_linear = 0.0
    for i, dia in enumerate(floors, 1):
        master = dia['master']
        mx, my, z = ref['coords'][master]
        masses = [(m['node'], v['value']) for m in model['masses']
                  for v in m['values'] if v['dof'] == 'Ux' and ref['coords'][m['node']][2] == z]
        cm = sum(ref['coords'][n][:2]*w for n,w in masses)/sum(w for _,w in masses)
        offset = 6*ref['ni'][master]
        ux = float(u[offset] - (cm[1]-my)*u[offset+5])
        ux_linear = float(linear[offset] - (cm[1]-my)*linear[offset+5])
        delta, delta_linear = ux-last_u, ux_linear-last_linear
        above = [6*ref['ni'][n] for n in ref['ids'] if ref['coords'][n][2] >= z]
        p = -float(sum(fg[d+2] for d in above))
        v = float(sum(fl[d] for d in above))
        h = float(z-last_z)
        rows.append(dict(floor=i, master=master, cm_m=cm.tolist(), height_m=h,
                         gravity_above_N=p, shear_above_N=v,
                         linear_drift_m=delta_linear, pdelta_drift_m=delta,
                         theta_linear=p*delta_linear/(v*h), theta_pdelta=p*delta/(v*h)))
        last_z, last_u, last_linear = z, ux, ux_linear
    evidence = dict(schema='pdelta-diaphragm-reference/1',
                    fixture_sha256=hashlib.sha256(source).hexdigest(),
                    run_id=runs[0]['id'],
                    definition='theta=P_above*Delta_CM/(V_EX_above*h); Delta=P-Delta lateral independiente, Kg Hermite congelada en gravedad lineal; theta_linear usa Delta lineal.',
                    min_reduced_total_eigenvalue=eigmin,
                    projected_relative_residual=residual,
                    axial_min_N=min(axial), axial_max_N=max(axial), floors=rows)
    (HERE/'reference.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=2, allow_nan=False)+'\n')
    print(json.dumps(evidence, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
