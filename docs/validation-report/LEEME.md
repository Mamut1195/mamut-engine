# ENG-VAL-006 (MAM-604) — Informe de validación MAMUT Engine V1
Compilar: `latexmk -lualatex informe.tex` (TeX Live con texlive-luatex).
- `herramientas/gen_referencia.py` → `datos/<caso>/referencia.csv` (analítica propia).
- `herramientas/results_a_csv.py` → `datos/<caso>/engine.csv` (salida del Engine).
- `datos/<caso>/opensees.csv` → **Q2-PLACEHOLDER: solo lo llena Q2**, en su propio PR. J2 no lo lee.
- `herramientas/gen_casos.py` regenera `casos/*.tex` de la lista interna (las derivaciones de `casos/deriv/` no se tocan). VAL-037 y VAL-038 quedan fuera de esa lista: al rehacer `secciones/casos.tex` el script solo reinserta sus `\input`, sin regenerar la ficha ni leer datos.
- `lua/mamutdata.lua` lee los CSV al compilar; si no hay datos, la serie queda como «pendiente».
