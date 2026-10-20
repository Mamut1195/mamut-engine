"""Puerto Python de engine_document de Q3 #183 (797535d8), sin Rust ni red.

Compara documentos completos, incluidos orden y valores, con una biyección
explícita por espacio de identificadores. Los diagnósticos Engine comprobados
aquí son los límites comunicados para VAL-032; no se ejecuta el solver.
"""
import copy

DOFS = ['Ux', 'Uy', 'Uz', 'Rx', 'Ry', 'Rz']
PDELTA_KEYS = ('gravity', 'lateral', 'tolerance', 'max_iterations',
               'displacement_scale', 'rotation_scale')


def make_bench(fixture, reference):
    """Conversión de datos existentes; reference aporta los campos no solicitados
    por runs estáticos (modal/espectro obligatorios en el esquema)."""
    model = fixture['model']
    node_ids = {n['id']: i for i, n in enumerate(model['nodes'], 1)}
    member_ids = {m['id']: i for i, m in enumerate(model['members'], 1)}
    coordinates = {n['id']: n['coordinates'] for n in model['nodes']}
    grid = {axis: sorted({n['coordinates'][i] for n in model['nodes']})
            for i, axis in enumerate(('x', 'y'))}
    levels = sorted({xyz[2] for xyz in coordinates.values()})
    stories = []
    for k, z in enumerate(levels[1:], 1):
        dia = next((d for d in model['diaphragms'] if coordinates[d['master']][2] == z), None)
        floor_masses = [(coordinates[m['node']], v) for m in model['masses']
                        if coordinates[m['node']][2] == z for v in m['values']]
        total = sum(v['value'] for _, v in floor_masses if v['dof'] == 'Ux')
        cm = [sum(r[j]*v['value'] for r, v in floor_masses if v['dof'] == 'Ux')/total
              for j in (0,1)] if total else [0.0, 0.0]
        # Metadatos físicos por piso: m e Iz en CM; nodal_masses manda al traducir.
        izz = sum(v['value']*((r[1]-cm[1])**2 if v['dof'] == 'Ux' else
                             (r[0]-cm[0])**2 if v['dof'] == 'Uy' else 1.0)
                  for r, v in floor_masses if v['dof'] in ('Ux','Uy','Rz'))
        stories.append(dict(level=k, h=z-levels[k-1], mass=dict(m=total, Izz=izz)))
    laws = {law['name']: {p['name']: p['value'] for p in law['parameters']}
            for law in model['laws']}
    sections = []
    for section in model['sections']:
        names = {m['law'] for m in model['members'] if m['section'] == section['id']}
        if len(names) != 1:
            raise ValueError('Una sección debe tener una sola ley en VAL-032')
        law = laws[names.pop()]
        if law['rho'] != 0:
            raise ValueError('translate.rs fija rho=0; no se puede conservar esta ley')
        item = dict(id=section['id'], E=law['E'])
        if 'G' in law:
            item['G'] = law['G']
        item.update({p['name']: p['value'] for p in section['properties']})
        sections.append(item)
    bench = dict(schema='bench-model/1', id=fixture['id'], units=fixture['units'],
        source=dict(geometry=f"{fixture['id']}: fixture VAL-032, model; README §Modelos",
            material=fixture['common']['material']['cita'], torsion='Roark, rectángulo; README §Base común y validez',
            mass=fixture['common']['cargas']['fuente_masa'],
            spectrum='NEC-SE-DS 2015 §3.3.1 pág. 34; Sa elástica en m/s^2, sin I/R'),
        stories=stories, grid=grid, nodes=[], members=[], sections=sections,
        diaphragms=[], restraints=[], nodal_masses=[],
        load_patterns=dict(gravity=[], lateral=[]),
        modal=dict(n=15, mass_kind='lumped', directions=[[1, 0, 0], [0, 1, 0]]),
        spectrum=copy.deepcopy(reference), combination=dict(method='cqc', zeta=reference['zeta']))
    for node in model['nodes']:
        xyz = node['coordinates']
        bench['nodes'].append(dict(id=node_ids[node['id']], story=levels.index(xyz[2]),
            ix=grid['x'].index(xyz[0]), iy=grid['y'].index(xyz[1]), coordinates=xyz))
    for member in model['members']:
        ends = member['nodes']
        kind = ('truss' if member['element'] == 'truss3d.linear' else
                'column' if coordinates[ends[0]][2] != coordinates[ends[1]][2] else 'beam')
        item = dict(id=member_ids[member['id']], kind=kind,
                    nodes=[node_ids[n] for n in ends], section=member['section'])
        if 'orientation' in member:
            item['orientation'] = dict(kind='local_z_vector', vector=member['orientation'])
        bench['members'].append(item)
    for dia in model['diaphragms']:
        z = coordinates[dia['master']][2]
        bench['diaphragms'].append(dict(level=levels.index(z), z=z,
            master=node_ids[dia['master']], slaves=[node_ids[n] for n in dia['slaves']]))
    bench['restraints'] = [dict(node=node_ids[s['node']], dofs=s['restrained']) for s in model['supports']]
    bench['nodal_masses'] = [dict(node=node_ids[m['node']], values=m['values']) for m in model['masses']]
    selections = []
    for run in fixture['runs']:
        document = run['engine_input']
        analysis = document['analyses'][0]
        method, options = analysis['method'], analysis.get('options', {})
        choice = dict(run=run['id'], method=method)
        if method in ('linearStatic', 'pDelta'):
            patterns = [dict(name=c['name'], actions=[dict(node=node_ids[a['node']], values=a['values'])
                        for a in c['loads']]) for c in document['cases']]
            if bench['load_patterns'].get('patterns', patterns) != patterns:
                raise ValueError('Casos diferentes entre runs estáticos')
            bench['load_patterns']['patterns'] = patterns
        if method == 'linearStatic':
            bench['static_combinations'] = document['combinations']
        elif method == 'pDelta':
            bench['p_delta'] = {k: options[k] for k in PDELTA_KEYS}
        elif method in ('modal', 'responseSpectrum'):
            bench['modal']['n'] = options['mode_count']
            bench['modal']['mass_kind'] = options['mass_kind']
            if method == 'modal':
                bench['modal']['directions'] = options['directions']
            else:
                directions = [dict(axis='XYZ'[d['direction'].index(1)], vector=d['direction'])
                              for d in options['directions']]
                tables = [d['spectra'] for d in options['directions']]
                if any(t != tables[0] for t in tables) or len(tables[0]) != 1:
                    raise ValueError('Espectros distintos entre direcciones')
                table = tables[0][0]
                for field, value in (('Sa', dict(unit='m/s^2', pairs=table['pairs'])),
                        ('zeta', table['damping']), ('interpolation', table['interpolation']),
                        ('extrapolation', table['extrapolation'])):
                    if bench['spectrum'][field] != value:
                        raise ValueError(f'Espectros distintos entre runs: {field}')
                variant = dict(id=run['id'], method=options['modal_combination'],
                    directions=directions, directional_combination=dict(
                        method=options['directional_combination'], factors=options['directional_factors']))
                if 'variants' not in bench['spectrum']:
                    bench['spectrum']['directions'] = directions
                    bench['spectrum']['directional_combination'] = variant['directional_combination']
                    bench['combination']['method'] = variant['method']
                    bench['spectrum']['variants'] = []
                bench['spectrum']['variants'].append(variant)
                choice['variant'] = run['id']
        selections.append(choice)
    # Pares explícitos por namespace; también las cadenas que Q3 genera.
    maps = dict(nodes=node_ids, members=member_ids,
        sections={s['id']: s['id'] for s in model['sections']}, laws={},
        supports={s['id']: f"support-{node_ids[s['node']]}" for s in model['supports']},
        masses={m['id']: f"mass-{node_ids[m['node']]}" for m in model['masses']},
        cases={c['name']: c['name'] for r in fixture['runs'] for c in r['engine_input']['cases']},
        combinations={c['id']: c['id'] for r in fixture['runs']
                      for c in r['engine_input']['combinations']})
    for member in model['members']:
        section = next(s for s in sections if s['id'] == member['section'])
        target = law_name(section, sections)
        previous = maps['laws'].setdefault(member['law'], target)
        if previous != target:
            raise ValueError('La ley original no admite biyección')
    for space, mapping in maps.items():
        if len(set(mapping.values())) != len(mapping):
            raise ValueError(f'{space}: remapeo no biyectivo')
    bijections = {space: [dict(original=a, bench=b) for a, b in mapping.items()]
                  for space, mapping in maps.items()}
    return copy.deepcopy(bench), bijections, selections


def flexure(section):
    present = [key in section for key in ('G', 'Iy', 'Iz', 'J')]
    if any(present) and not all(present):
        raise ValueError('Sección de flexión incompleta')
    return all(present)


def law_name(section, sections):
    first = section
    if flexure(section):
        first = next(s for s in sections
                     if s['E'] == section['E'] and s.get('G') == section['G'])
    return 'elastic-' + first['id']


def translate(bench, method, variant=None):
    """Reproduce engine_document, masas, leyes, casos y análisis de translate.rs.

    El control de esquema se hace aparte. engine_gaps aplica los límites Engine
    relevantes a estos fixtures después de probar equivalencia de la traducción.
    """
    sections = bench['sections']
    for key in ('nodes', 'members'):
        if [v['id'] for v in bench[key]] != list(range(1, len(bench[key])+1)):
            raise ValueError(f'{key}: ids no contiguos desde 1')
    bay = min(b-a for axis in bench['grid'].values()
              for a, b in zip(axis, axis[1:]))
    model = dict(axes='Z-up', nodes=[], members=[], sections=[], laws=[],
                 supports=[], masses=[], diaphragms=[])
    for node in bench['nodes']:
        model['nodes'].append(dict(id=node['id'], dim='D3',
                                  coordinates=node['coordinates'], dofs=DOFS))
    for member in bench['members']:
        section = next(s for s in sections if s['id'] == member['section'])
        frame = flexure(section)
        if frame != (member['kind'] != 'truss'):
            raise ValueError('Sección incompatible con kind')
        item = dict(id=member['id'], nodes=member['nodes'], section=section['id'],
                    law=law_name(section, sections),
                    element='frame3d.euler_elastic' if frame else 'truss3d.linear')
        if frame:
            if member['orientation']['kind'] != 'local_z_vector':
                raise ValueError('Orientación no vectorial')
            item['orientation'] = member['orientation']['vector']
        elif 'orientation' in member:
            raise ValueError('Truss con orientación')
        model['members'].append(item)
    seen = set()
    for section in sections:
        frame = flexure(section)
        keys = ('A', 'Iy', 'Iz', 'J') if frame else ('A',)
        model['sections'].append(dict(id=section['id'], properties=[
            dict(name=k, value=section[k]) for k in keys]))
        key = (section['E'], section.get('G'))
        if frame and key in seen:
            continue
        seen.add(key)
        parameters = [dict(name='E', value=section['E'])]
        if frame:
            parameters.append(dict(name='G', value=section['G']))
        parameters.append(dict(name='rho', value=0.0))
        model['laws'].append(dict(name=law_name(section, sections), parameters=parameters))
    model['supports'] = [dict(id=f"support-{r['node']}", node=r['node'],
                              restrained=r['dofs']) for r in bench['restraints']]
    if 'nodal_masses' in bench:
        declared = bench['nodal_masses']
    else:
        declared = []
        for dia in bench['diaphragms']:
            story = next(s for s in bench['stories'] if s['level'] == dia['level'])
            m, izz = story['mass']['m'], story['mass']['Izz']
            count = len(dia['slaves'])+1
            share, rotary = m/count, m*bay*bay/(12.0*count)
            declared.append(dict(node=dia['master'], values=[
                dict(dof=d, value=v) for d, v in zip(DOFS, (m, m, share, rotary, rotary, izz))]))
            declared.extend(dict(node=n, values=[dict(dof=d, value=v)
                            for d, v in (('Uz', share), ('Rx', rotary), ('Ry', rotary))])
                            for n in dia['slaves'])
    model['masses'] = [dict(id=f"mass-{m['node']}", node=m['node'], values=m['values'])
                       for m in declared]
    model['diaphragms'] = [dict(master=d['master'], slaves=d['slaves'])
                           for d in bench['diaphragms']]
    cases, combinations = [], []
    if method in ('linearStatic', 'pDelta'):
        loads = bench['load_patterns']
        patterns = loads.get('patterns') or [dict(name=k, actions=loads[k])
                                             for k in ('gravity', 'lateral')]
        if loads.get('patterns') and (loads['gravity'] or loads['lateral']):
            raise ValueError('patterns reemplaza gravity/lateral')
        cases = [dict(name=p['name'], loads=p['actions']) for p in patterns]
    if method == 'linearStatic':
        combinations = bench.get('static_combinations', [])
        analysis = dict(method=method)
    elif method == 'pDelta':
        # Acceso obligatorio: nunca suplir ninguna de las seis opciones.
        analysis = dict(method=method, options={k: bench['p_delta'][k] for k in PDELTA_KEYS})
    elif method == 'modal':
        m = bench['modal']
        analysis = dict(method=method, options=dict(mode_count=m['n'],
                        mass_kind=m['mass_kind'], directions=m['directions']))
    elif method == 'responseSpectrum':
        spectrum = bench['spectrum']
        choice = next(v for v in spectrum['variants'] if v['id'] == variant) if variant else dict(
            method=bench['combination']['method'],
            directions=spectrum.get('directions') or [spectrum['direction']],
            directional_combination=spectrum['directional_combination'])
        directions = [dict(direction=d['vector'], spectra=[dict(
            pairs=spectrum['Sa']['pairs'], units='periodSecondsAccelerationMetersPerSecondSquared',
            damping=spectrum['zeta'], interpolation=spectrum['interpolation'],
            extrapolation=spectrum['extrapolation'])]) for d in choice['directions']]
        analysis = dict(method=method, options=dict(mode_count=bench['modal']['n'],
            mass_kind=bench['modal']['mass_kind'], directions=directions,
            modal_combination=choice['method'],
            directional_combination=choice['directional_combination']['method'],
            directional_factors=choice['directional_combination']['factors']))
    else:
        raise ValueError(f'Análisis desconocido: {method}')
    return copy.deepcopy(dict(schema='engine-input/1', units='SI', model=model,
                               cases=cases, combinations=combinations,
                               analyses=[analysis], options={}))


def remap_document(document, bijections):
    """Solo sustituye ids y referencias; ningún valor físico se normaliza."""
    result = copy.deepcopy(document)
    maps = {space: {p['original']: p['bench'] for p in pairs}
            for space, pairs in bijections.items()}
    def mapped(space, value):
        return maps[space][value]
    model = result['model']
    for node in model['nodes']:
        node['id'] = mapped('nodes', node['id'])
    for member in model['members']:
        member['id'] = mapped('members', member['id'])
        member['nodes'] = [mapped('nodes', n) for n in member['nodes']]
        member['law'] = mapped('laws', member['law'])
        member['section'] = mapped('sections', member['section'])
    for section in model['sections']:
        section['id'] = mapped('sections', section['id'])
    for law in model['laws']:
        law['name'] = mapped('laws', law['name'])
    for key in ('supports', 'masses'):
        for item in model[key]:
            item['id'] = mapped(key, item['id'])
            item['node'] = mapped('nodes', item['node'])
    for dia in model['diaphragms']:
        dia['master'] = mapped('nodes', dia['master'])
        dia['slaves'] = [mapped('nodes', n) for n in dia['slaves']]
    for case in result['cases']:
        case['name'] = mapped('cases', case['name'])
        for load in case['loads']:
            load['node'] = mapped('nodes', load['node'])
    for combination in result['combinations']:
        combination['id'] = mapped('combinations', combination['id'])
        for term in combination['terms']:
            term['case'] = mapped('cases', term['case'])
    for analysis in result['analyses']:
        if analysis['method'] == 'pDelta':
            for key in ('gravity', 'lateral'):
                term = analysis['options'][key]
                term['case'] = mapped('cases', term['case'])
    return result


def require_exact(actual, expected, path='$'):
    """Igualdad numérica exacta y orden de listas; informa la primera diferencia."""
    if isinstance(expected, dict):
        if not isinstance(actual, dict) or actual.keys() != expected.keys():
            raise ValueError(f'{path}: claves distintas')
        for key in expected:
            require_exact(actual[key], expected[key], f'{path}.{key}')
    elif isinstance(expected, list):
        if not isinstance(actual, list) or len(actual) != len(expected):
            raise ValueError(f'{path}: longitud distinta')
        for i, (a, e) in enumerate(zip(actual, expected)):
            require_exact(a, e, f'{path}[{i}]')
    elif actual != expected:
        raise ValueError(f'{path}: {actual!r} != {expected!r}')


def engine_gaps(document):
    """Límites Engine pertinentes, sin modificar apoyos ni masas."""
    model = document['model']
    slaves = {n for d in model['diaphragms'] for n in d['slaves']}
    supports = {s['node']: set(s['restrained']) for s in model['supports']}
    gaps = []
    if slaves & supports.keys():
        gaps.append(dict(codigo='ENG-E-0013', referencia='Engine: diafragmas/apoyos',
            detalle='Un esclavo de diafragma tiene apoyo; retirarlo cambiaría el modelo físico.'))
    if document['analyses'][0]['method'] == 'pDelta' and model['diaphragms']:
        gaps.append(dict(codigo='ENG-E-0007', referencia='Engine: pDelta + diafragma',
            detalle='Las fuerzas geométricas y el residual no se proyectan por la transformación del diafragma. '
            'E3-pdelta termina con status notConverged, residual mínimo ≈1.14e6. '
            'Seguimiento MAM-847; issue del Engine pendiente de abrir, corrección en ticket aparte del Engine. '
            'El esquema sí permite el modelo; no se cambian física ni tolerancias.'))
    if document['analyses'][0]['method'] in ('modal', 'responseSpectrum'):
        masses = {(m['node'], v['dof']): v['value']
                  for m in model['masses'] for v in m['values']}
        missing = []
        for node in model['nodes']:
            for dof in node['dofs']:
                if dof in supports.get(node['id'], set()):
                    continue
                if node['id'] in slaves and dof in ('Ux', 'Uy', 'Rz'):
                    continue  # eliminados por el diafragma XY
                if masses.get((node['id'], dof), 0.0) <= 0:
                    missing.append((node['id'], dof))
        if missing:
            gaps.append(dict(codigo='ENG-E-0006', referencia='Engine: masa modal positiva por gdl libre',
                detalle=f'{len(missing)} gdl libres sin masa positiva: Uz/Rx/Ry de los '
                '60 nodos de piso (55 esclavos y 5 maestros). nodal_masses conserva '
                'el reparto exacto Ux/Uy y Rz_extra. El Engine no condensa GDL sin masa; '
                'seguimiento MAM-847, issue del Engine pendiente de abrir. '
                'Añadir masa o restringir esos gdl cambiaría la física; '
                'apoyar esclavos además viola ENG-E-0013. El esquema sí permite el modelo.',
                gdl_sin_masa=[dict(node=n, dof=d) for n, d in missing]))
    return gaps
