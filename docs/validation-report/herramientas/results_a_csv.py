#!/usr/bin/env python3
"""Convierte un documento results/1 del Engine a datos/<caso>/engine.csv.
Uso: results_a_csv.py <results.json> <caso> <colx> <coly> <ruta_x> <ruta_y>
Las rutas son listas JSON con punto (p.ej. 'cases.*.input.P'). Solo salida del Engine; nunca OpenSees."""
import json,sys,csv
def get(d,path):
    out=[d]
    for k in path.split('.'):
        nxt=[]
        for o in out:
            if k=='*': nxt+= (list(o.values()) if isinstance(o,dict) else list(o))
            else: nxt.append(o[int(k)] if isinstance(o,list) else o[k])
        out=nxt
    return out
r=json.load(open(sys.argv[1])); xs=get(r,sys.argv[5]); ys=get(r,sys.argv[6])
with open(f'datos/{sys.argv[2]}/engine.csv','w',newline='') as f:
    w=csv.writer(f); w.writerow([sys.argv[3],sys.argv[4]]); [w.writerow([x,y]) for x,y in zip(xs,ys)]
