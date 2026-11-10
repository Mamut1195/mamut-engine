#!/usr/bin/env python3
"""Genera casos/VAL-xxx.tex y los CSV vacíos de engine/opensees. Uso: python3 herramientas/gen_casos.py

VAL-037 y VAL-038 no entran en la lista C. Su texto se escribe a mano.
Al rehacer secciones/casos.tex, PRESERVADOS y COLA solo agregan \\input.
No se lee ningún CSV ni un directorio de resultados, y no se toca la plantilla
que emite la columna de un tercer programa.
"""
import os
B=os.path.join(os.path.dirname(__file__),'..')
# id, familia, título, diagrama, fuente, q2, tolerancia(A,R), ticket, x, y, xlabel, ylabel
C=[
("VAL-001","est","Viga simplemente apoyada con carga $q$ y carga $P$ (B1)","biapoyada","[S] catálogo MAM-253 y \\cite{kassimali}",1,(0,1e-6),"MAM-100","x","y","$x$","$y$"),
("VAL-002","est","Viga continua de dos vanos (B2)","continua","[S] catálogo MAM-253",1,(0,1e-6),"MAM-100","x","y","$x$","$y$"),
("VAL-003","est","Voladizo con carga puntual (B3)","voladizo","[S] catálogo MAM-253",1,(0,1e-6),"MAM-100","x","y","$x$","$y$"),
("VAL-004","est","Pórtico 2D bajo carga lateral (F1)","portico","[S] catálogo MAM-253 (energía, $EA$ finito) y \\cite{kassimali}",1,(0,1e-6),"MAM-100","x","y","$x$","$y$"),
("VAL-005","est","Pórtico 3D con carga biaxial (F2)","portico3d","[S] catálogo MAM-253",1,(0,1e-6),"MAM-100","x","y","$x$","$y$"),
("VAL-006","est","Armaduras 2D/3D (seis casos ENG-VAL-001)","armadura","[S] y \\cite{kassimali} (método de rigidez; ejemplo concreto no conseguido)",1,(0,1e-10),"MAM-100","x","y","$x$","$y$"),
("VAL-007","est","Asentamiento impuesto ($N=EA\\,s/L$)","asentamiento","[S]",1,(0,1e-10),"MAM-100","x","y","$x$","$y$"),
("VAL-008","est","Combinaciones y envolventes","combinacion","[S] (superposición algebraica)",0,(0,1e-12),"MAM-100","x","y","$x$","$y$"),
("VAL-009","est","Articulación en extremo de viga ($R_A=5wL/8$, $M_j=0$)","articulada","[S]",1,(0,1e-10),"MAM-100","x","y","$x$","$y$"),
("VAL-010","est","Diafragma rígido 3D","diafragma","[S] y \\cite{chopra}, cap.~9 (sección no conseguida)",1,(0,1e-8),"MAM-100","x","y","$x$","$y$"),
("VAL-011","est","Voladizo espacial: flexión biaxial y torsión de Saint-Venant","voladizo","[S] ($GJ/L$, $PL^3/3EI$)",1,(0,1e-8),"MAM-100","x","y","$x$","$y$"),
("VAL-012","est","Carga uniforme en barra y peso propio","biapoyada","[S] y \\cite{kassimali}",1,(0,1e-10),"MAM-100","x","y","$x$","$y$"),
("VAL-013","mod","Modal: 1 GDL equivalente (M1)","sdof","[S] catálogo MAM-253 y \\cite{chopra}",1,(0,1e-3),"MAM-103","x","y","modo","$T$ [s]"),
("VAL-014","mod","Modal: 2 GDL con inercia rotacional (M2)","sdof","[S] catálogo MAM-253",1,(0,1e-3),"MAM-103","x","y","modo","$T$ [s]"),
("VAL-015","mod","Modal 3D con modos desacoplados (M3)","portico3d","[S] catálogo MAM-253",1,(0,1e-3),"MAM-103","x","y","modo","$T$ [s]"),
("VAL-016","mod","Participación modal y masa efectiva","sdof","[S] y \\cite{chopra}, cap.~13",1,(0,1e-3),"MAM-103","x","y","modo","$\\sum M^*/M$"),
("VAL-017","pd","P-Delta en voladizo (N1, $B_1/B_2$)","voladizo","[S] catálogo MAM-253 y \\cite{aisc16}",1,(0,1e-2),"MAM-104","x","y","$P$","$\\Delta$"),
("VAL-018","pd","P-Delta: benchmark AISC Caso 1 (viga-columna con carga uniforme)","biapoyadaP","\\cite{aisc16}, Comentario C2, Fig.~C-C2.2, p.~16.1-291; solución cerrada propia \\cite{timoshenko}",1,(0,0.03),"MAM-104","P_kN","M_kNm","$P$ [kN]","$M_\\text{mid}$ [kN\\,m]"),
("VAL-019","pd","P-Delta: benchmark AISC Caso 2 (voladizo con $H$)","voladizoP","\\cite{aisc16}, Comentario C2, Fig.~C-C2.3, p.~16.1-291; solución cerrada propia \\cite{timoshenko}",1,(0,0.03),"MAM-104","P_kN","M_kNm","$P$ [kN]","$M_\\text{base}$ [kN\\,m]"),
("VAL-020","pd","Amplificación analítica de viga-columna $\\chi(u)$","biapoyadaP","\\cite{timoshenko}, cap.~1 (página no conseguida); derivación propia",0,(0,1e-3),"MAM-104","u","chiM","$u=\\tfrac{L}{2}\\sqrt{P/EI}$","$\\chi_M$"),
("VAL-021","esp","Espectro con 1 GDL ($u=S_a/\\omega^2$) y espectro tabulado","sdof","[S] y \\cite{chopra}, cap.~6",0,(0,1e-6),"MAM-107","T_s","u_mm","$T$ [s]","$u$ [mm] ($S_a=1$ m/s$^2$)"),
("VAL-022","esp","Espectro MGDL: SRSS frente a CQC con modos cercanos","sdof","\\cite{wilson81,derkiureghian81}, \\cite{chopra}, cap.~13",1,(0,1e-3),"MAM-107","x","y","modo","respuesta"),
("VAL-023","geo","Viga sobre lecho de Winkler 2D/3D (seis casos GEO-002)","winkler","\\cite{hetenyi} y [S]",1,(0,1e-2),"MAM-105","x","y","$x$ [m]","$w$ [mm]"),
("VAL-024","geo","Consolidación 1D de Terzaghi multicapa (siete casos GEO-006)","suelo","\\cite{terzaghi} y [S] (serie de Fourier propia)",0,(1e-4,0),"MAM-105","Tv","U","$T_v$","$U$"),
("VAL-025","geo","Perfil geostático $\\sigma_v$, $u$ y $\\sigma'_v$","suelo","[S] y \\cite{das}, cap.~9 (página no conseguida)",0,(0,1e-12),"MAM-105","x","y","$z$ [m]","$\\sigma'_v$ [kPa]"),
("VAL-026","nl","Acero bilineal cinemático: ciclo de carga y descarga","resorte","[S] (respuesta cerrada por tramos)",1,(0,1e-6),"MAM-105","x","y","$\\varepsilon$","$\\sigma$"),
("VAL-027","nl","Leyes multilineal y cúbica elástica","resorte","[S] (\\texttt{docs/GUIA\\_METODOS.md} \\S5)",0,(0,1e-12),"MAM-105","x","y","$\\varepsilon$","$\\sigma$"),
("VAL-028","nl","Resorte no lineal y Newton por pasos de carga","resorte","[S] (1 GDL con ley conocida)",1,(0,1e-6),"MAM-105","x","y","$u$","$F$"),
("VAL-029","nl","Determinismo y paridad CLI/Python/WASM","combinacion","[S] (igualdad de bytes)",0,(0,0),"MAM-105","x","y","ejecución","SHA"),
("VAL-030","geo","Curvas $p$-$y$ (Matlock 1970, Reese--Cox--Koop 1974, usuario) --- PR \\#168","winkler","\\cite{matlock70,reese74}",1,(0,0),"MAM-105","x","y","$y$ [mm]","$p$ [kN/m]"),
]
fam={"est":[],"mod":[],"pd":[],"esp":[],"geo":[],"nl":[]}
for (i,f,t,dg,src,q2,(A,R),tk,x,y,xl,yl) in C:
    d=f'{B}/datos/{i}'; os.makedirs(d,exist_ok=True)
    for n,h in (("engine",f"# Engine: generar con herramientas/results_a_csv.py (results/1 -> CSV)\n{x},{y}\n"),):
        open(f'{d}/{n}.csv','w').write(h)
    if q2: open(f'{d}/opensees.csv','w').write(f"# Q2-PLACEHOLDER: archivo provisto SOLO por Q2 desde results/ de mamut-engine-oracles. J2 no lo lee ni lo llena.\n{x},{y}\n")
    if not os.path.exists(f'{d}/referencia.csv'):
        open(f'{d}/referencia.csv','w').write(f"# REF-PENDIENTE: derivar desde el fixture con gen_referencia.py\n{x},{y}\n")
    fam[f].append(i)
    ops_plot = (f"\\directlua{{mamut.serie('{i}','opensees','{x}','{y}','ops','OpenSees (contraste)')}}" if q2 else "")
    q2blk = (f"""% Q2-PLACEHOLDER INICIO ({i})
% Solo Q2 escribe datos/{i}/opensees.csv y el texto de modelo OpenSees de este bloque,
% en su propio PR. J2 no lee ni escribe valores de OpenSees.
\\paragraph{{Modelo de contraste (OpenSees).}} \\QdosPendiente
% Q2-PLACEHOLDER FIN ({i})""" if q2 else "\\paragraph{Contraste OpenSees.} No aplica a este caso (ver plan de validación).")
    open(f'{B}/casos/{i}.tex','w').write(f"""% {i} -- generado por herramientas/gen_casos.py; editar el texto libremente.
\\section{{{i}: {t}}}\\label{{sec:{i}}}
\\begin{{ficha}}
Fuente & {src}\\\\
Ticket & {tk}\\\\
Engine & commit \\texttt{{\\EngineCommit}}\\\\
Binario & SHA-256 \\texttt{{\\EngineBinSHA}}\\\\
Entrada & SHA-256 \\texttt{{\\EntradaSHA{{{i}}}}}\\\\
Tolerancia & $|c-r|\\le A+R|r|$, $A={A:g}$, $R={R:g}$ (clase consolidada; orden de ejecución DESCONOCIDO; el marco también distingue preregistro, DESCONOCIDO y VAL-037 tras la r1 de \\#23)\\\\
Estado de datos & \\directlua{{mamut.estado('{i}')}}\\\\
\\end{{ficha}}

\\paragraph{{Modelo.}}
\\begin{{center}}\\diag{{{dg}}}\\end{{center}}

\\paragraph{{Referencia y derivación.}} \\input{{casos/deriv/{i}}}

\\paragraph{{Reproducción.}}
\\reproducir{{{i}}}

{q2blk}

\\paragraph{{Resultados.}}
\\begin{{center}}
\\begin{{tikzpicture}}
\\begin{{axis}}[grafico, xlabel={{{xl}}}, ylabel={{{yl}}}]
\\directlua{{mamut.serie('{i}','referencia','{x}','{y}','ref','Referencia analítica')}}
\\directlua{{mamut.serie('{i}','engine','{x}','{y}','eng','MAMUT Engine')}}
{ops_plot}
\\end{{axis}}
\\end{{tikzpicture}}
\\end{{center}}
\\begin{{center}}\\small
\\begin{{tabular}}{{rrrrrrl}}\\toprule
{xl} & Ref. & Engine & OpenSees & Error rel. & Tol. & Veredicto\\\\\\midrule
\\directlua{{mamut.tabla('{i}','{x}','{y}',{A!r},{R!r})}}
\\bottomrule\\end{{tabular}}
\\end{{center}}
""")
os.makedirs(f'{B}/casos/deriv',exist_ok=True)
for (i,*_) in C:
    p=f'{B}/casos/deriv/{i}.tex'
    if not os.path.exists(p): open(p,'w').write("Derivación propia pendiente de redactar a partir del fixture del caso (provenance del Engine). % DERIV-PENDIENTE\n")
nom={"est":"Estática lineal","mod":"Análisis modal","pd":"Segundo orden (P-Delta)","esp":"Espectro de respuesta","geo":"Geotecnia","nl":"Leyes no lineales, solucionador y determinismo"}
# Solo \\input. No generan ficha ni columna.
PRESERVADOS={"pd":["VAL-037"]}
# Capítulo propio, inmediatamente después de P-Delta. Tampoco usa la plantilla.
DESPUES={"pd":[("Pandeo lineal",["VAL-038"])]}
with open(f'{B}/secciones/casos.tex','w') as g:
    for f,ids in fam.items():
        g.write(f"\\chapter{{{nom[f]}}}\n"+"".join(f"\\input{{casos/{i}}}\n" for i in ids))
        for extra in PRESERVADOS.get(f,[]):
            g.write(f"\\input{{casos/{extra}}}\n")
        for titulo,extras in DESPUES.get(f,[]):
            g.write(f"\\chapter{{{titulo}}}\n"+"".join(f"\\input{{casos/{i}}}\n" for i in extras))
