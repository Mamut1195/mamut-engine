"""Comparación offline de dos secciones del voladizo; sin cálculo Python."""
from copy import deepcopy
import json
from pathlib import Path

from mamut_engine.experiments import compare_scenarios, comparison_table, methods

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "crates/engine-api/fixtures/input/valid/cantilever-linear-static.json"
PROFILE = {
    "schema": "mamut.experiment/1", "name": "voladizo-secciones", "version": 1,
    "metrics": {"Uy_punta": {
        "path": ["cases", 0, "nodes", 1, "displacements", 1], "unit": "m",
    }},
}

if __name__ == "__main__":
    base = json.loads(BASE.read_text(encoding="utf-8"))
    larger = deepcopy(base)
    larger["model"]["sections"][0]["properties"][1]["value"] = 6
    rows = compare_scenarios({"original": base, "mayor_inercia": larger}, PROFILE)
    print(json.dumps({"registered_methods": methods(), "comparison": comparison_table(rows)},
                     ensure_ascii=False, indent=2))
