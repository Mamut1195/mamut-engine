"""Entrada compatible: referencia canónica MAM-860 de 1, 8 y 16 elementos.

La generación y los datos viven junto al fixture VAL-032, en docs/validation-report/
fixtures/val032/_ref/e3_pdelta_reference.{py,json}. Sin copia local de resultados.
"""
from pathlib import Path
import runpy

if __name__ == "__main__":
    script = (Path(__file__).resolve().parents[5]
              / "docs/validation-report/fixtures/val032/_ref/e3_pdelta_reference.py")
    runpy.run_path(str(script), run_name="__main__")
