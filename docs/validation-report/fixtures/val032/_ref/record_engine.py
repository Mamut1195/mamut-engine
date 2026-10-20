"""Registrar estados y diagnósticos de results/1 exportados por val032_runs.

No ejecuta solvers; la comparación física independiente está en el calibrador.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def input_digest(document):
    return hashlib.sha256(json.dumps(document, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--results', type=Path, required=True)
    args = parser.parse_args()
    entries = json.loads((ROOT/'index.json').read_text())['entries']
    rows = []
    for entry in entries:
        fixture = json.loads((ROOT/entry['path']).read_text())
        for run in fixture['runs']:
            path = args.results/(run['id']+'.json')
            data = path.read_bytes()
            result = json.loads(data)
            assert result['schema'] == 'results/1'
            assert json.loads((args.results/(run['id']+'.input.json')).read_text()) == run['engine_input']
            modal = run['engine_input']['analyses'][0]['method'] in ('modal', 'responseSpectrum')
            pdelta = run['id'] == 'E3-pdelta'
            assert result['status'] == ('unstable' if modal else 'notConverged' if pdelta else 'warning')
            assert result['status'] == run['info']['status_engine']
            errors = [d['code'] for d in result['diagnostics'] if d['severity'] == 'error']
            assert errors == (['ENG-E-0006'] if modal else ['ENG-E-0007'] if pdelta else [])
            expected_warnings = {}
            if run['engine_input']['model']['diaphragms']:
                expected_warnings['API-W-DIAPHRAGM-RECOVERY'] = 1
            if run['engine_input']['combinations']:
                expected_warnings['API-W-COMBINED-PRESCRIBED'] = 10
            assert Counter(d['code'] for d in result['diagnostics'] if d['severity'] == 'warning') == expected_warnings
            rows.append(dict(run=run['id'], status=result['status'],
                input_sha256=result['input_sha256'], sha256_engine_input=input_digest(run['engine_input']),
                sha256_results_document=hashlib.sha256(data).hexdigest(),
                diagnostics=result['diagnostics'], cases=len(result['cases']),
                combinations=len(result['combinations']), modes=len(result['modes']),
                convergence=result['convergence']))
    evidence = dict(schema='val032-engine-r3/1', seguimiento='MAM-847',
        test='crates/engine-api/tests/val032_runs.rs',
        nota='Documentos results/1 reales; estados y causas copiados sin reinterpretar códigos. Modal/espectro bloqueados por ENG-E-0006; E3-pdelta bloqueado por defecto del Engine (pDelta + diafragma), ENG-E-0007 y notConverged. Seguimiento MAM-847; issues del Engine pendientes de abrir.',
        runs=rows)
    (ROOT/'_ref/engine_r3.json').write_text(json.dumps(evidence, ensure_ascii=False, indent=1, allow_nan=False)+'\n')
    print(f'{len(rows)} documentos results/1 registrados')


if __name__ == '__main__':
    main()
