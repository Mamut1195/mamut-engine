#!/usr/bin/env python3
"""Validar los bench-model emitidos con JSON Schema 2020-12, siempre sin red.

Uso desde val032: python _ref/validate_bench.py --schema /ruta/bench-model-1.json
Comprueba también la traducción exacta de todos los runs (incluidos candidatos
omitidos), sus biyecciones y los límites Engine documentados, sin ejecutarlo.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

from jsonschema import Draft202012Validator
from referencing import Registry
from referencing.exceptions import NoSuchResource
from bench_translate import make_bench, translate, remap_document, require_exact, engine_gaps

ROOT = Path(__file__).resolve().parent.parent
EXPECTED_SCHEMA_SHA256 = '4e4aa7cd2278767a0f9b2899a77657a980230539bfff3ea3a18fb7f6d0e8b005'


def deny_retrieval(uri):
    # Solo se resuelven referencias internas al esquema pasado por argumento.
    raise NoSuchResource(ref=uri)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--schema', type=Path, required=True)
    args = parser.parse_args()
    try:
        schema_bytes = args.schema.read_bytes()
        schema = json.loads(schema_bytes)
        index = json.loads((ROOT/'index.json').read_text(encoding='utf-8'))
        if (hashlib.sha256(schema_bytes).hexdigest() != EXPECTED_SCHEMA_SHA256
                or index['bench_model_spec']['schema_sha256'] != EXPECTED_SCHEMA_SHA256):
            raise ValueError('El sha256 del esquema no coincide con bench_model_spec.schema_sha256')
        Draft202012Validator.check_schema(schema)
        validator = Draft202012Validator(schema, registry=Registry(retrieve=deny_retrieval))
        entries = index['bench_models']
        omitted = index['bench_model_omissions']
        all_ids = {entry['id'] for entry in index['entries']}
        emitted_ids = [entry['id'] for entry in entries]
        omitted_ids = [entry['id'] for entry in omitted]
        if (len(set(emitted_ids)) != len(emitted_ids)
                or len(set(omitted_ids)) != len(omitted_ids)
                or set(emitted_ids) & set(omitted_ids)
                or set(emitted_ids) | set(omitted_ids) != all_ids):
            raise ValueError('Cada fixture debe estar emitido u omitido exactamente una vez')
        if any(entry['estado'] != 'no_representable' or not entry['brechas'] for entry in omitted):
            raise ValueError('Toda omisión requiere estado no_representable y brechas documentadas')
        indexed = {(ROOT/entry['path']).resolve() for entry in entries}
        present = {path.resolve() for path in (ROOT/'bench-model').glob('*.json')}
        if indexed != present:
            raise ValueError('Los archivos bench-model/*.json y las entradas del índice no coinciden')
        failed = False
        for entry in entries:
            path = (ROOT/entry['path']).resolve()
            data = path.read_bytes()
            if hashlib.sha256(data).hexdigest() != entry['sha256']:
                raise ValueError(f"{entry['path']}: sha256 distinto del índice")
            document = json.loads(data)
            if document.get('id') != entry['id']:
                raise ValueError(f"{entry['path']}: id distinto del índice")
            errors = list(validator.iter_errors(document))
            for error in errors:
                print(f"{entry['path']}:{error.json_path}: {error.message}", file=sys.stderr)
            failed |= bool(errors)
        if failed:
            return 1
        for entry in index['entries'] + index['herramientas']:
            if hashlib.sha256((ROOT/entry['path']).read_bytes()).hexdigest() != entry['sha256']:
                raise ValueError(f"{entry['path']}: hash de fixture/herramienta incorrecto")
        for key in ('calibration_r3', 'engine_verification_r3'):
            evidence = index[key]
            if hashlib.sha256((ROOT/evidence['path']).read_bytes()).hexdigest() != evidence['sha256']:
                raise ValueError(f'{key}: hash de evidencia incorrecto')
        for source in index['engine_verification_r3']['fuentes']:
            if hashlib.sha256((ROOT.parents[3]/source['path']).read_bytes()).hexdigest() != source['sha256']:
                raise ValueError(f"{source['path']}: hash de fuente Rust incorrecto")
        translation = index['bench_model_translation']
        checks = translation['checks']
        if [c['id'] for c in checks] != [e['id'] for e in index['entries']]:
            raise ValueError('Falta una comprobación de traducción por fixture')
        engine_validator = Draft202012Validator(
            json.loads((ROOT/'_ref/engine-input-1.json').read_text()),
            registry=Registry(retrieve=deny_retrieval))
        run_count = 0
        for entry, check in zip(index['entries'], checks):
            fixture = json.loads((ROOT/entry['path']).read_text())
            candidate, bijections, selections = make_bench(fixture, translation['reference_spectrum'])
            require_exact(check['id_bijections'], bijections)
            require_exact(check['runs'], selections)
            # Los candidatos omitidos también caben en el esquema; el rechazo
            # pertinente se debe al Engine y no se disfraza de brecha del esquema.
            validator.validate(candidate)
            bench = candidate
            emitted = next((e for e in entries if e['id'] == entry['id']), None)
            if emitted:
                bench = json.loads((ROOT/emitted['path']).read_text())
                require_exact(bench, candidate)
            gaps = []
            for run, selection in zip(fixture['runs'], selections):
                document = translate(bench, selection['method'], selection.get('variant'))
                require_exact(document, remap_document(run['engine_input'], bijections))
                engine_validator.validate(run['engine_input'])
                engine_validator.validate(document)
                for gap in engine_gaps(document):
                    if gap not in gaps:
                        gaps.append(gap)
                run_count += 1
            if not check['exact_translation'] or check['engine_limits'] != (
                    'incompatible' if gaps else 'compatible'):
                raise ValueError(f"{entry['id']}: resultado de traducción incorrecto")
            if not emitted:
                raise ValueError(f"{entry['id']}: candidato representable no emitido")
            require_exact(check['engine_gaps'], gaps)
            expected_state = '; '.join(f'bloqueado por {code}' for code in
                                      dict.fromkeys(gap['codigo'] for gap in gaps)) if gaps else 'ejecutable'
            if emitted['estado_engine'] != expected_state:
                raise ValueError(f"{entry['id']}: estado Engine incorrecto")
        if entries:
            print(f'{len(entries)} bench-model/1 válidos (Draft 2020-12, sin red).')
        else:
            print(f'0 bench-model/1 emitidos; {len(omitted)} omisiones documentadas. '
                  'Esquema comprobado; ningún modelo estricto validado.')
        print(f'{run_count} runs: traducción exacta salvo biyecciones; '
              f'{len(omitted)} omisiones por límites Engine. Engine no ejecutado.')
        return 0
    except Exception as error:
        print(f'Error de validación: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
