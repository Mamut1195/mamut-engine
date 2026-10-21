"""Barrido offline del voladizo existente; ejecutar con la wheel instalada."""
import json
from pathlib import Path

from mamut_engine.experiments import comparison_table, sweep

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "crates/engine-api/fixtures/input/valid/cantilever-linear-static.json"
PROFILE = {
    "schema": "mamut.experiment/1", "name": "voladizo-carga", "version": 1,
    "parameters": {"carga_N": {
        "path": ["cases", 0, "loads", 0, "values", 0, "value"],
        "values": [-6, -12, -24],
    }},
    "metrics": {"Uy_punta": {
        "path": ["cases", 0, "nodes", 1, "displacements", 1], "unit": "m",
    }},
}

if __name__ == "__main__":
    rows = sweep(BASE.read_text(encoding="utf-8"), PROFILE)
    print(json.dumps(comparison_table(rows), ensure_ascii=False, indent=2))
