"""Referencia numpy independiente: frame3d Euler y diafragma XY exacto.

La condensación estática modal de GDL sin masa es exclusiva de esta herramienta;
no modifica el Engine ni habilita sus runs bloqueados. SI; sin red ni oráculos.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import numpy as np

DOFS = ['Ux', 'Uy', 'Uz', 'Rx', 'Ry', 'Rz']
ROOT = Path(__file__).resolve().parent.parent
TOL = 1e-10


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


class Reference:
    def __init__(self, model):
        self.model = model
        self.__dict__.update(assemble(model))
        self.eigmin = float(np.linalg.eigvalsh(self.kr)[0])
        assert self.eigmin > 0, self.eigmin

    def force(self, loads):
        force = np.zeros(self.nd)
        for load in loads:
            for value in load['values']:
                force[6*self.ni[load['node']]+DOFS.index(value['dof'])] += value['value']
        return force

    def static(self, loads):
        force = self.force(loads)
        rhs = self.constraint.T @ force
        reduced = np.linalg.solve(self.kr, rhs)
        u = self.constraint @ reduced
        residual = float(np.linalg.norm(self.kr @ reduced-rhs)/max(1., np.linalg.norm(force)))
        assert residual < 1e-9, residual
        reactions = self.stiffness @ u-force
        applied = np.zeros(6); base = np.zeros(6)
        for n in self.ids:
            offset = 6*self.ni[n]
            f = force[offset:offset+6]
            r = np.array([reactions[offset+j] if offset+j in self.fixed else 0 for j in range(6)])
            applied[:3] += f[:3]; applied[3:] += f[3:]+np.cross(self.coords[n], f[:3])
            base[:3] += r[:3]; base[3:] += r[3:]+np.cross(self.coords[n], r[:3])
        equilibrium = max(float(np.linalg.norm((base+applied)[part])/max(1., np.linalg.norm(applied[part])))
                          for part in (slice(0,3), slice(3,6)))
        assert equilibrium < 1e-9, equilibrium
        displacement = {n: u[6*self.ni[n]:6*self.ni[n]+6] for n in self.ids}
        bases = sorted(s['node'] for s in self.model['supports'] if self.coords[s['node']][2] == 0)
        reaction = np.array([reactions[6*self.ni[n]:6*self.ni[n]+6] for n in bases])
        return displacement, reaction, residual, equilibrium

    def periods(self):
        # M puede acoplar Ux/Uy/Rz: partición por filas nulas, no diagonalización aproximada.
        massive = np.flatnonzero(np.any(self.mr != 0, axis=1))
        massless = np.flatnonzero(~np.any(self.mr != 0, axis=1))
        if not len(massive):
            return [], len(massless)
        kaa = self.kr[np.ix_(massive, massive)]
        kab = self.kr[np.ix_(massive, massless)]
        kbb = self.kr[np.ix_(massless, massless)]
        condensed = kaa-kab @ np.linalg.solve(kbb, kab.T) if len(massless) else kaa
        mass = self.mr[np.ix_(massive, massive)]
        chol = np.linalg.cholesky(mass)
        eigenmatrix = np.linalg.solve(chol, np.linalg.solve(chol, condensed).T).T
        eigenvalues = np.linalg.eigvalsh((eigenmatrix+eigenmatrix.T)/2)
        assert np.all(eigenvalues > 0)
        return (2*np.pi/np.sqrt(eigenvalues[:15])).tolist(), len(massless)


def solve(model, loads):
    ref = Reference(model)
    u, _, residual, equilibrium = ref.static(loads)
    return u, ref.eigmin, residual, equilibrium


def cm_of(model):
    coords = {n['id']: n['coordinates'] for n in model['nodes']}
    dia = model['diaphragms'][0]
    level = coords[dia['master']][2]
    masses = [(coords[m['node']], v['value']) for m in model['masses']
              for v in m['values'] if v['dof'] == 'Ux' and coords[m['node']][2] == level]
    total = sum(w for _, w in masses)
    return [sum(r[j]*w for r, w in masses)/total for j in (0,1)]


def floor_responses(ref, displacement, cm):
    rows = []
    for dia in ref.model['diaphragms']:
        master = dia['master']; x, y, _ = ref.coords[master]
        u, v, _, _, _, rz = displacement[master]
        rows.append([[u-(py-y)*rz, v+(px-x)*rz, rz] for px, py in (cm, (0,0), (15,10))])
    rows = np.array(rows)
    drift = np.diff(rows, axis=0, prepend=np.zeros_like(rows[:1]))
    return rows, drift


def relative(actual, previous):
    """Máximo por componente: ||nuevo−anterior||∞/||anterior||∞.

    Componentes físicamente nulas (<1e-12 del máximo global) usan ese piso
    numérico como denominador. Se registran también el error absoluto y escala.
    """
    a, b = np.asarray(actual, dtype=float), np.asarray(previous, dtype=float)
    if a.size == 0:
        return dict(rel=0.0, abs=0.0, escala=0.0)
    # Escala por componente (Ux/Uy/Rz o seis componentes de reacción).
    axes = tuple(range(b.ndim-1))
    scale = np.max(np.abs(b), axis=axes) if axes else np.abs(b)
    error = np.max(np.abs(a-b), axis=axes) if axes else np.abs(a-b)
    # Componentes nulas se juzgan contra la escala de la respuesta completa.
    denominator = np.where(scale > np.max(scale)*1e-12, scale, max(float(np.max(scale)), 1e-30))
    result = dict(rel=float(np.max(error/denominator)), abs=float(np.max(error)), escala=float(np.max(scale)))
    assert result['rel'] <= TOL, result
    return result


def translated_elf(model, cm, forces, dof):
    coords = {n['id']: n['coordinates'] for n in model['nodes']}
    loads = []
    for dia, force in zip(model['diaphragms'], forces):
        master = dia['master']; x, y, _ = coords[master]
        moment = (cm[0]-x)*force if dof == 'Uy' else -(cm[1]-y)*force
        loads.append(dict(node=master, values=[dict(dof=dof, value=force), dict(dof='Rz', value=moment)]))
    return loads


def polar_checks(model, old_model):
    coords = {n['id']: n['coordinates'] for n in model['nodes']}
    cm = cm_of(old_model)
    rows = []
    connected = {n for b in model['members'] for n in b['nodes']}
    assert set(coords) <= connected
    assert all(coords[s['node']][2] == 0 for s in model['supports'])
    for i, (dia, old) in enumerate(zip(model['diaphragms'], old_model['diaphragms']), 1):
        master = dia['master']; mx, my, z = coords[master]
        assert master not in dia['slaves'] and len(dia['slaves']) == 11
        assert all(coords[n][2] == z > 0 for n in [master, *dia['slaves']])
        masses = [(m, v) for m in model['masses'] if coords[m['node']][2] == z for v in m['values']]
        weights = [(m['node'], v['value']) for m, v in masses if v['dof'] == 'Ux']
        assert all(w >= 0 for _, w in weights)
        assert [(m['node'], v['value']) for m, v in masses if v['dof'] == 'Uy'] == weights
        assert all(v['dof'] in ('Ux','Uy','Rz') for _, v in masses)
        original = {v['dof']: v['value'] for m in old_model['masses'] if m['node'] == old['master'] for v in m['values']}
        total = sum(w for _, w in weights)
        for j in (0,1):
            assert math.isclose(sum(coords[n][j]*w for n, w in weights), original['Ux']*cm[j], rel_tol=1e-14)
        assert math.isclose(total, original['Ux'], rel_tol=1e-14)
        extra = sum(v['value'] for _, v in masses if v['dof'] == 'Rz')
        polar = sum(w*((coords[n][0]-mx)**2+(coords[n][1]-my)**2) for n, w in weights)
        target = original['Rz']+total*((cm[0]-mx)**2+(cm[1]-my)**2)
        assert extra >= 0 and math.isclose(polar+extra, target, rel_tol=1e-14)
        # Matriz 3×3 referida al nuevo maestro; comprueba también términos
        # cruzados, ausentes si solo se coloca m y se corrige Iz con m*d².
        actual = np.zeros((3,3))
        for item, value in masses:
            x, y, _ = coords[item['node']]
            row = {'Ux': [1,0,-(y-my)], 'Uy': [0,1,x-mx], 'Rz': [0,0,1]}[value['dof']]
            actual += value['value']*np.outer(row, row)
        rx, ry = cm[0]-mx, cm[1]-my
        target_matrix = np.array([[total,0,-total*ry], [0,total,total*rx], [-total*ry,total*rx,target]])
        matrix_error = float(np.max(np.abs(actual-target_matrix))/np.max(np.abs(target_matrix)))
        naive = np.diag([total,total,target])
        naive_error = float(np.max(np.abs(naive-target_matrix))/np.max(np.abs(target_matrix)))
        assert matrix_error <= TOL and naive_error > TOL
        rows.append(dict(piso=i, master=master, cm=cm,
                         matriz_masa_master=actual.tolist(), matriz_masa_objetivo=target_matrix.tolist(),
                         error_rel_matriz=matrix_error, error_rel_solo_m_d2=naive_error, pesos=[dict(node=n, w_kg=w) for n,w in weights],
                         Rz_extra_kgm2=extra, J_total_master_kgm2=target, J_reconstruido_kgm2=polar+extra))
    return rows


def calibrate_r3(previous_dir, entries=None):
    if entries is None:
        entries = json.loads((ROOT/'index.json').read_text())['entries']
    force_doc = json.loads((previous_dir/'v32run-E5-combinaciones.json').read_text())
    forces = [load['values'][0]['value'] for load in next(c for c in force_doc['cases'] if c['name'] == 'EX')['loads']]
    evidence = dict(schema='val032-calibrate-r3/1', tolerancia_relativa=TOL,
        metodo='Referencia numpy independiente, frame3d Euler, transformación exacta XY; modal con condensación estática de GDL sin masa, solo en referencia.',
        definicion_error='Máximo por componente: norma infinito de diferencia / norma infinito anterior; componentes nulas (<1e-12 del máximo) usan escala máxima de la respuesta. Períodos: relativo por modo.',
        edificios=[])
    for entry in entries:
        fixture = json.loads((ROOT/entry['path']).read_text())
        run = fixture['runs'][0]
        prior_path = previous_dir/f"v32run-{run['id']}.json"
        prior_bytes = prior_path.read_bytes(); prior = json.loads(prior_bytes)
        sources = []
        for current_run in fixture['runs']:
            source = previous_dir/f"v32run-{current_run['id']}.json"
            data = source.read_bytes(); document = json.loads(data)
            assert document['model'] == prior['model'], current_run['id']
            assert document['cases'] == prior['cases'], current_run['id']
            assert document['combinations'] == prior['combinations'], current_run['id']
            sources.append(dict(run=current_run['id'], path=source.name, sha256=hashlib.sha256(data).hexdigest()))
        old, new = Reference(prior['model']), Reference(fixture['model'])
        checks = {}
        periods_old, massless_old = old.periods(); periods_new, massless_new = new.periods()
        if periods_old:
            assert len(periods_old) == len(periods_new) == 15
            mode_error = [abs(a-b)/abs(b) for a,b in zip(periods_new, periods_old)]
            assert max(mode_error) <= TOL
            checks['T1_T15'] = dict(anterior_s=periods_old, nuevo_s=periods_new, error_rel_por_modo=mode_error, rel=max(mode_error))
        else:
            assert entry['id'] == 'E5b' and not periods_new
            checks['modal'] = dict(estado='no aplica: E5b sin masas, exclusivamente estático', rel=0.0)
        floors = polar_checks(fixture['model'], prior['model']) if periods_old else []
        if floors:
            cm = cm_of(prior['model'])
            for dof in ('Ux','Uy'):
                uo, _, _, _ = old.static(translated_elf(prior['model'], cm, forces, dof))
                un, _, _, _ = new.static(translated_elf(fixture['model'], cm, forces, dof))
                ao, bo = floor_responses(old, uo, cm); an, bn = floor_responses(new, un, cm)
                checks[f'ELF_{dof}_desplazamientos_CM_esquinas'] = relative(an, ao)
                checks[f'ELF_{dof}_derivas_CM_esquinas'] = relative(bn, bo)
        # Casos estáticos reales y combinaciones, conservando las cargas de ambos JSON.
        old_cases = {c['name']: c for c in prior['cases']}
        actual_cases = {c['name']: c for c in run['engine_input']['cases']}
        reactions_old, reactions_new = {}, {}
        for name, case in actual_cases.items():
            uo, ro, _, _ = old.static(old_cases[name]['loads'])
            un, rn, _, _ = new.static(case['loads'])
            reactions_old[name], reactions_new[name] = ro, rn
            if entry['id'] in ('E5','E5b'):
                checks[f'reacciones_{name}'] = relative(rn, ro)
            if name in ('G', 'D', 'L'):
                checks[f'{name}_desplazamientos_nodos_piso'] = relative(
                    [un[n] for n in new.ids if new.coords[n][2] > 0],
                    [uo[n] for n in new.ids if new.coords[n][2] > 0])
            elif floors:
                ao, bo = floor_responses(old, uo, cm); an, bn = floor_responses(new, un, cm)
                checks[f'{name}_desplazamientos_CM_esquinas'] = relative(an, ao)
                checks[f'{name}_derivas_CM_esquinas'] = relative(bn, bo)
            else:
                checks[f'{name}_desplazamientos_nodos_piso'] = relative(
                    [un[n] for n in new.ids if new.coords[n][2] > 0],
                    [uo[n] for n in old.ids if old.coords[n][2] > 0])
        for combo in run['engine_input']['combinations']:
            ro = sum(t['factor']*reactions_old[t['case']] for t in combo['terms'])
            rn = sum(t['factor']*reactions_new[t['case']] for t in combo['terms'])
            checks['reacciones_'+combo['id']] = relative(rn, ro)
        if 'estabilidad_previa' in fixture['info']:
            un, _, _, _ = new.static(actual_cases['EX']['loads'])
            _, drift = floor_responses(new, un, cm)
            for row, delta in zip(fixture['info']['estabilidad_previa']['pisos'], drift[:,0,0]):
                assert math.isclose(row['Delta_i_m'], float(delta), rel_tol=TOL)
        evidence['edificios'].append(dict(id=entry['id'], fuente_anterior=prior_path.name,
            sha256_anterior=hashlib.sha256(prior_bytes).hexdigest(), fuentes_anteriores=sources,
            sha256_engine_input_nuevo=hashlib.sha256(json.dumps(run['engine_input'], sort_keys=True, separators=(',',':')).encode()).hexdigest(),
            gdl_sin_masa_anterior=massless_old, gdl_sin_masa_nuevo=massless_new,
            pisos=floors, comprobaciones=checks, max_error_relativo=max(c['rel'] for c in checks.values())))
    return evidence


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixture', nargs='?', type=Path)
    parser.add_argument('--r3', action='store_true')
    parser.add_argument('--previous-dir', type=Path, default=Path('/tmp'))
    parser.add_argument('--output', type=Path, default=ROOT/'_ref/calibrate_r3.json')
    args = parser.parse_args()
    if args.r3:
        evidence = calibrate_r3(args.previous_dir)
        args.output.write_text(json.dumps(evidence, ensure_ascii=False, indent=1, allow_nan=False)+'\n')
        print(json.dumps({e['id']: e['max_error_relativo'] for e in evidence['edificios']}, indent=1))
    else:
        fixture = json.loads(args.fixture.read_text())
        case = next(c for c in fixture['runs'][0]['engine_input']['cases'] if c['name'] == 'EX')
        ref = Reference(fixture['model'])
        u, _, residual, equilibrium = ref.static(case['loads'])
        _, drift = floor_responses(ref, u, cm_of(fixture['model']))
        rows = []
        for row, delta in zip(fixture['info']['estabilidad_previa']['pisos'], drift[:,0,0]):
            assert math.isclose(row['Delta_i_m'], float(delta), rel_tol=TOL)
            rows.append(dict(row, Delta_i_m=float(delta), theta_i=row['P_i_N']*float(delta)/(row['V_i_N']*row['h_i_m'])))
        print(json.dumps(dict(pisos=rows, lambda_min=ref.eigmin, residual_rel=residual, equilibrio_rel=equilibrium), indent=1))


if __name__ == '__main__':
    main()
