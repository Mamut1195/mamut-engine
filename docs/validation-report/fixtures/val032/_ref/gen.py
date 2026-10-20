# Generador VAL-032 (E1–E5, E2b y E5b). Determinista; sin red; sin salidas de oráculos.
import json, math, hashlib, pathlib, copy
from bench_translate import make_bench, translate, remap_document, require_exact, engine_gaps
OUT = pathlib.Path(__file__).resolve().parent.parent
# Identidad de la copia revisada; generar no requiere acceso al esquema externo.
BENCH_MODEL_SPEC = dict(pr=183, sha="797535d86af8fe6f57f935fbaa978e14b15cfe89",
    schema_sha256="4e4aa7cd2278767a0f9b2899a77657a980230539bfff3ea3a18fb7f6d0e8b005",
    estado="borrador")
g = 9.80665
fc = 28.0                      # MPa
E = 4700*math.sqrt(fc)*1e6     # Pa, ACI 318-25 19.2.2.1(b)
nu = 0.2; G = E/(2*(1+nu))
SLAB_A = 10.0  # m², penalización axial común a E5b y al diafragma de E3 en Frame3DD
FRAME3DD_FORMAT = ("Frame3DD imprime en coma fija, no en notación científica. La salida del oráculo debe alcanzar ≥10 cifras significativas ajustando las unidades (p.ej. mm, kN); Q2 debe comprobarlo en una corrida en seco de formato antes de congelar. Si no se alcanza, E5b frente a Frame3DD y la parte afectada de E3 son solo diagnóstico, no criterio.")
def rect(b, h):  # b ancho (horizontal local y), h canto (vertical local z)
    A = b*h; Iy = b*h**3/12; Iz = h*b**3/12
    a, c = max(b, h), min(b, h)
    J = a*c**3*(1/3 - 0.21*(c/a)*(1 - c**4/(12*a**4)))
    return dict(A=A, Iy=Iy, Iz=Iz, J=J)
# NEC-SE-DS 2015 §3.3.1 pág. 34; parámetros contrastados por J4 (ver README).
NEC = dict(ubicacion="Sierra", zona="V", Z=0.40, suelo="D", Fa=1.20, Fd=1.19, Fs=1.28, eta=2.48, r=1.0, R=8.0, I=1.0,
           phiP=1.0, phiE=1.0, amortiguamiento=0.05)
T0 = 0.10*NEC['Fs']*NEC['Fd']/NEC['Fa']
Tc = 0.55*NEC['Fs']*NEC['Fd']/NEC['Fa']
def Sa(T):  # en g, espectro elástico NEC-SE-DS 2015 §3.3.1, pág. 34
    p = NEC['eta']*NEC['Z']*NEC['Fa']
    if T < T0:
        return NEC['Z']*NEC['Fa']*(1+(NEC['eta']-1)*T/T0)
    return p if T <= Tc else p*(Tc/T)**NEC['r']
periods = sorted(set([round(0.02*i, 2) for i in range(0, 201)] + [T0, Tc]))
SA_TABLE = [[T, Sa(T)*g] for T in periods]   # m/s², elástico (I/R no aplicado)
def table():
    return dict(pairs=SA_TABLE, units="periodSecondsAccelerationMetersPerSecondSquared",
                damping=NEC['amortiguamiento'], interpolation="linear", extrapolation="error")

LX = [0, 5, 10, 15]; LY = [0, 5, 10]; H = 3.0
qD, qL = 6000.0, 2000.0          # Pa (D incluye losa, vigas, acabados; L oficinas)
area = (LX[-1]-LX[0])*(LY[-1]-LY[0])
def trib(x, y):
    tx = 2.5 if x in (LX[0], LX[-1]) else 5.0
    ty = 2.5 if y in (LY[0], LY[-1]) else 5.0
    return tx*ty
ALL = ["Ux","Uy","Uz","Rx","Ry","Rz"]
def nid(k, i, j): return 100*k + 4*j + i + 1
def build(nst, col, beam, ecc):
    nodes, members, sup, masses, dia = [], [], [], [], []
    mid = 0
    for k in range(nst+1):
        for j, y in enumerate(LY):
            for i, x in enumerate(LX):
                nodes.append(dict(id=nid(k,i,j), dim="D3", coordinates=[x, y, k*H], dofs=ALL))
                if k == 0: sup.append(dict(id=f"base_{nid(k,i,j)}", node=nid(k,i,j), restrained=ALL))
    m = qD*area/g; xc = LX[-1]/2+ecc[0]; yc = LY[-1]/2+ecc[1]
    Jm = m*((LX[-1])**2+(LY[-1])**2)/12
    for k in range(1, nst+1):
        floor = [n for n in nodes if n['coordinates'][2] == k*H]
        candidates = [n for n in floor if n['coordinates'][0] != xc and n['coordinates'][1] != yc] 
        master = min(candidates, key=lambda n: ((n['coordinates'][0]-xc)**2 +
                                          (n['coordinates'][1]-yc)**2, -n['coordinates'][0], n['coordinates'][1], n['id']))
        mn = master['id']
        # Bilineal: pesos no negativos y primer momento exacto en el CM real.
        x0 = max(x for x in LX if x <= xc); x1 = min(x for x in LX if x >= xc)
        y0 = max(y for y in LY if y <= yc); y1 = min(y for y in LY if y >= yc)
        wx = {x0: 1.0} if x0 == x1 else {x0: (x1-xc)/(x1-x0), x1: (xc-x0)/(x1-x0)}
        wy = {y0: 1.0} if y0 == y1 else {y0: (y1-yc)/(y1-y0), y1: (yc-y0)/(y1-y0)}
        distributed = []
        for n in floor:
            x, y, _ = n['coordinates']
            w = m*wx.get(x, 0.0)*wy.get(y, 0.0)
            if w > 0:
                distributed.append((n, w))
        assert math.isclose(sum(w for _, w in distributed), m, rel_tol=1e-14)
        for axis, cm in enumerate((xc, yc)):
            assert math.isclose(sum(w*n['coordinates'][axis] for n, w in distributed), m*cm, rel_tol=1e-14)
        mx, my, _ = master['coordinates']
        polar = sum(w*((n['coordinates'][0]-mx)**2+(n['coordinates'][1]-my)**2) for n, w in distributed)
        extra = Jm + m*((xc-mx)**2+(yc-my)**2) - polar
        assert extra >= 0, extra
        values = {n['id']: [dict(dof='Ux', value=w), dict(dof='Uy', value=w)] for n, w in distributed}
        values.setdefault(mn, []).append(dict(dof='Rz', value=extra))
        for node, vv in values.items():
            masses.append(dict(id=f"piso_{k}_nodo_{node}", node=node, values=vv))
        dia.append(dict(master=mn, slaves=[n['id'] for n in floor if n['id'] != mn]))
    for k in range(1, nst+1):
        for j in range(3):
            for i in range(4):
                mid += 1; members.append(dict(id=mid, nodes=[nid(k-1,i,j), nid(k,i,j)], section="columna", law="hormigon", element="frame3d.euler_elastic", orientation=[1,0,0]))
        for j in range(3):
            for i in range(3):
                mid += 1; members.append(dict(id=mid, nodes=[nid(k,i,j), nid(k,i+1,j)], section="viga", law="hormigon", element="frame3d.euler_elastic", orientation=[0,0,1]))
        for j in range(2):
            for i in range(4):
                mid += 1; members.append(dict(id=mid, nodes=[nid(k,i,j), nid(k,i,j+1)], section="viga", law="hormigon", element="frame3d.euler_elastic", orientation=[0,0,1]))
    secs = [dict(id="columna", properties=[dict(name=n, value=v) for n, v in rect(*col).items()]),
            dict(id="viga", properties=[dict(name=n, value=v) for n, v in rect(*beam).items()])]
    laws = [dict(name="hormigon", parameters=[dict(name="E",value=E),dict(name="G",value=G),dict(name="rho",value=0.0)])]
    model = dict(axes="Z-up", nodes=nodes, members=members, sections=secs, laws=laws, supports=sup, masses=masses, diaphragms=dia)
    return model, m, Jm, (xc, yc)
def grav(nst, q, name):
    loads = []
    for k in range(1, nst+1):
        for j, y in enumerate(LY):
            for i, x in enumerate(LX):
                loads.append(dict(node=nid(k,i,j), values=[dict(dof="Uz", value=-q*trib(x,y))]))
    return dict(name=name, loads=loads)
def elf(nst, w_floor):  # NEC-SE-DS §6.3.2 / §6.3.3 / §6.3.5
    hn = nst*H; Ta = 0.055*hn**0.9
    k = 1.0 if Ta <= 0.5 else (0.75+0.5*Ta if Ta <= 2.5 else 2.0)
    Cs = NEC['I']*Sa(Ta)/(NEC['R']*NEC['phiP']*NEC['phiE'])
    W = w_floor*nst; V = Cs*W
    s = sum(w_floor*(i*H)**k for i in range(1, nst+1))
    F = [V*w_floor*(i*H)**k/s for i in range(1, nst+1)]
    return dict(hn=hn, Ta=Ta, k=k, Sa_g=Sa(Ta), Cs=Cs, W=W, V=V, F=F)
def lat(model, F, dof, name, cm):
    coords = {n['id']: n['coordinates'] for n in model['nodes']}
    loads = []
    for i, dia in enumerate(model['diaphragms']):
        master = dia['master']; x, y, _ = coords[master]
        moment = (cm[0]-x)*F[i] if dof == 'Uy' else -(cm[1]-y)*F[i]
        loads.append(dict(node=master, values=[dict(dof=dof, value=F[i]), dict(dof='Rz', value=moment)]))
    return dict(name=name, loads=loads)
def doc(model, cases, combos, analysis):
    return {"schema":"engine-input/1","units":"SI","model":model,"cases":cases,"combinations":combos,"analyses":[analysis],"options":{}}
def modal(n): return dict(method="modal", options=dict(mode_count=n, mass_kind="lumped", directions=[[1,0,0],[0,1,0]]))
def spec(n, comb, d):
    return specs(n, comb, [d])
def specs(n, comb, directions):
    return dict(method="responseSpectrum", options=dict(mode_count=n, mass_kind="lumped", modal_combination=comb,
        directional_combination="srss", directional_factors=[1.0]*len(directions),
        directions=[dict(direction=d, spectra=[table()]) for d in directions]))
COL, BEAM = (0.50, 0.50), (0.30, 0.50)
common = dict(material=dict(fc_MPa=fc, E_Pa=E, nu=nu, G_Pa=G, cita="ACI 318-25 §19.2.2.1(b): Ec=4700√f'c (MPa)"),
    rigidez="Secciones brutas (sin agrietamiento NEC §6.1.6) — modelo de verificación, no de diseño",
    planta=dict(LX=LX, LY=LY, altura_piso=H), cargas=dict(D_Pa=qD, L_Pa=qL, fuente_masa="m_piso = D·A/g, NEC-SE-DS §6.1.7 (W=D, sin fracción de L)"))
fixtures = {}
def fixture(fid, title, tests, model, info, runs, tol, extra=None):
    nn = len(model['nodes']); assert nn <= 256
    d = {"schema":"val032-fixture/1","schema_status":"BORRADOR — envoltura VAL-032; documentos bench-model/1 separados conformes a #183 (797535d8); ver README §Brechas",
         "id":fid, "version":"1.0.1" if fid in ("E2b", "E5b") else "1.1.1", "case":"VAL-032", "linear":"MAM-847", "parent":"MAM-604", "title":title, "tests":tests,
         "units":"SI", "common":common, "info":info, "node_count":nn, "member_count":len(model['members']),
         "model":model, "runs":runs, "tolerances_preregistered":tol + tolmk([
             ("períodos disponibles T1..T15 (s)", "Engine vs OpenSees/Frame3DD", 1e-5, "relativa",
              "Aplica cuando el run entrega períodos; mismo modelo y masas."),
             ("desplazamientos lineales disponibles (CM y nudos)", "Engine vs OpenSees/Frame3DD", 1e-4, "relativa",
              "Todos los modelos; desplazamientos y derivas P-Δ tienen las comparaciones específicas por formulación de E3."),
             ("equivalencia de formato: todas las magnitudes disponibles", "mismo Engine: entrada actual vs remapeo a bench-model/1", 1e-12, "relativa",
              "Mismos números y orden del modelo. El 1e-3 de Q3 solo controla equivalencia de formato; no es criterio de verificación.")])}
    if fid == 'E5b':
        d['common'] = copy.deepcopy(common)
        d['common']['cargas']['fuente_masa'] = "ninguna: masses=[]; fixture exclusivamente estático"
    if extra: d.update(extra)
    fixtures[fid] = d

def tolmk(rows):
    return [dict(metric=a, comparison=b, tol=c, kind=dk, justification=e,
        status=override[0] if override else ("diagnóstico, no criterio (#84)" if c is None else "criterio"))
        for a,b,c,dk,e,*override in rows]
EQ = ("equilibrio global ΣR + ΣF = 0 (fuerzas y momentos)", "Engine interno", 1e-9, "relativa a ΣF", "Identidad de equilibrio; solo error de redondeo del sistema lineal (ASME V&V 10, verificación de código).")
# E4 simétrico
m4, mm, Jm, cm = build(5, COL, BEAM, (0, 0))
W1 = qD*area
e4 = elf(5, W1)
fixture("E4", "Pórtico 3D simétrico regular de 5 pisos (control de desacople)",
  ["modos desacoplados X/Y/torsión", "participación cruzada nula", "períodos"], m4,
  dict(masa_piso_kg=mm, inercia_Rz_kgm2=Jm, CM=cm, excentricidad=[0,0]),
  [dict(id="E4-modal", engine_input=doc(m4, [], [], modal(15)))],
  tolmk([("T1..T15 (s)", "Engine vs OpenSees/Frame3DD", 1e-5, "relativa", "Mismo modelo lineal Euler, masa concentrada idéntica; diferencias solo del autosolver."),
         ("fracción de masa efectiva cruzada (X en modos Y y Y en modos X)", "valor absoluto", 1e-8, "absoluta", "Simetría exacta CM=CR: la participación cruzada es analíticamente cero."),
         ("masa efectiva acumulada modos 1–6 / masa libre, en X e Y", "Engine vs oráculos", 1e-6, "absoluta", "Control no trivial de participación modal; acumulación por modo pendiente de corrida.")]))
# E1 excéntrico
m1, mm, Jm, cm1 = build(5, COL, BEAM, (1.5, 0.5))
fixture("E1", "Pórtico 3D de 5 pisos con diafragma rígido y excentricidad de masa (torsión, modos acoplados)",
  ["acoplamiento traslación-torsión", "períodos y participación"], m1,
  dict(masa_piso_kg=mm, inercia_Rz_kgm2=Jm, CM=cm1, excentricidad=[1.5, 0.5], nota="e=0.10·Lx, 0.05·Ly"),
  [dict(id="E1-modal", engine_input=doc(m1, [], [], modal(15)))],
  tolmk([("T1..T15 (s)", "Engine vs OpenSees/Frame3DD", 1e-5, "relativa", "Modelo lineal idéntico."),
         ("fracciones de masa efectiva por modo y dirección", "Engine vs oráculos", 1e-5, "absoluta", "Modos acoplados bien separados; sensible sólo si hay modos casi repetidos (se vigila).")]))
# E2 espectro sobre la geometría E1
fixture("E2", "Modal espectral NEC-SE-DS (CQC y SRSS) sobre E1",
  ["cortante basal", "desplazamientos y derivas de piso", "CQC vs SRSS con modos acoplados"], m1,
  dict(masa_piso_kg=mm, CM=cm1, T0_s=T0, Tc_s=Tc, postproc="_ref/postproc_derivas.py", nota_derivas="La deriva espectral se calcula combinando derivas modales (no restando desplazamientos combinados); el oráculo debe hacer lo mismo."),
  [dict(id="E2-cqc-X", engine_input=doc(m1, [], [], spec(15, "cqc", [1,0,0]))),
   dict(id="E2-srss-X", engine_input=doc(m1, [], [], spec(15, "srss", [1,0,0]))),
   dict(id="E2-cqc-Y", engine_input=doc(m1, [], [], spec(15, "cqc", [0,1,0])))],
  tolmk([("cortante basal V (N) por dirección", "Engine vs oráculo con análisis por espectro de respuesta", 1e-4, "relativa", "Misma tabla Sa(T) con interpolación lineal; diferencias de períodos ≤1e-5 se amplifican poco."),
         ("desplazamiento de techo en el CM real (reconstruido desde el maestro)", "Engine vs oráculos", 1e-4, "relativa", "Idem."),
         ("derivas de piso CM/esquinas CQC y SRSS", "Engine vs oráculos vía postproc común", 1e-4, "relativa", "Combinar derivas modales firmadas con _ref/postproc_derivas.py, no restar máximos."),
         ("Sa(T_i) interpolada", "Engine vs Sa(T) analítica §3.3.1", 5e-4, "relativa", "Error de interpolación lineal en la rama 1/T con paso 0.02 s.")]),
  extra=dict(nec_se_ds=dict(parametros=NEC, T0_s=T0, Tc_s=Tc,
             cita="NEC-SE-DS 2015 §3.3.1 pág. 34 (Sierra, espectro elástico); ver páginas contrastadas por J4 en README",
             no_verificado=["I=1.0 contra Tabla 6 pág. 39", "NEC-SE-CG §3.4.3, números y factores de combinaciones"],
             nota="Tabla en m/s² = Sa(g)·9.80665, ELÁSTICA (I/R no aplicados). T0 y Tc exactos incluidos.",
             tabla_Sa=SA_TABLE)))
# E2b: torsión accidental, variante de masas de E4; rigidez idéntica.
m2b, mm2b, J2b, cm2b = build(5, COL, BEAM, (0.75, 0.75))
fixture("E2b", "Torsión accidental: CM +0.05·Lmax en X e Y (geometría E4)",
  ["CQC X/Y", "SRSS direccional X+Y", "cortante basal", "derivas CM/esquinas y Ax"], m2b,
  dict(masa_piso_kg=mm2b, inercia_Rz_kgm2=J2b, CM=cm2b, excentricidad=[0.75,0.75],
       cita="NEC-SE-DS 2015 §6.3.6–6.3.7 págs. 66–67; rama accidental +5% en ambos ejes, no envolvente ±",
       postproc="_ref/postproc_derivas.py", corners_xy=[[0,0],[15,10]], T0_s=T0, Tc_s=Tc,
       Ax="(δmax/(1.2·δprom))²; δmax y δprom de desplazamientos de nivel en las dos esquinas; por componente, sin límites adicionales"),
  [dict(id="E2b-cqc-X", engine_input=doc(m2b, [], [], spec(15, "cqc", [1,0,0]))),
   dict(id="E2b-cqc-Y", engine_input=doc(m2b, [], [], spec(15, "cqc", [0,1,0]))),
   dict(id="E2b-cqc-XY-srss", engine_input=doc(m2b, [], [], specs(15, "cqc", [[1,0,0],[0,1,0]])))],
  tolmk([("cortante basal por dirección y SRSS direccional", "Engine vs oráculos", 1e-4, "relativa", "Misma tabla y factores [1,1]."),
         ("derivas CM/esquinas CQC/SRSS y Ax por piso", "Engine vs oráculos vía postproc común", 1e-4, "relativa", "Mismo _ref/postproc_derivas.py para todos los programas."),
         ("Sa(T_i) interpolada", "Engine vs Sa(T) analítica §3.3.1", 5e-4, "relativa", "Tabla cada 0.02 s, T0 y Tc exactos.")]),
  extra=dict(nec_se_ds=copy.deepcopy(fixtures['E2']['nec_se_ds'])))
# E3 P-Delta
COL3 = (0.25, 0.25)
QG_EXTRA = 16000.0  # Pa, carga muerta artificial solo G: no fuente de masa ni ELF
m3, mm, Jm, cm3 = build(5, COL3, BEAM, (0, 0))
e3 = elf(5, W1)
G3 = grav(5, qD+qL+QG_EXTRA, "G"); EX3 = lat(m3, e3['F'], "Ux", "EX", cm3)
# Desplazamientos de primer orden calculados con numpy en _ref/val032_calibrate.py.
# Modelo J2 independiente: frame3d Euler + transformación exacta de diafragmas XY.
# No son respuestas del Engine ni de oráculos. Recalibrar si cambia rigidez o EX.
DELTA3 = [0.017584054251960253, 0.0185858780281215, 0.01616030677471958,
          0.012299932401153263, 0.007040564535673727]
STABILITY3 = []
for i, delta in enumerate(DELTA3, 1):
    p = (6-i)*(qD+qL+QG_EXTRA)*area
    v = sum(e3['F'][i-1:])
    STABILITY3.append(dict(piso=i, Delta_i_m=delta, P_i_N=p, V_i_N=v, h_i_m=H, theta_i=p*delta/(v*H)))
assert 0.10 <= max(r['theta_i'] for r in STABILITY3) <= 0.25
assert sum(r['theta_i'] >= 0.10 for r in STABILITY3) >= 3
fixture("E3", "P-Δ de piso con factor de estabilidad θ (5 pisos, columnas 0.25×0.25)",
  ["amplificación P-Δ", "θ_i = P_i Δ_i /(V_i h_i), NEC-SE-DS §6.3.8"], m3,
  dict(ELF=e3, D_adicional_solo_G_Pa=QG_EXTRA, D_total_gravedad_Pa=qD+QG_EXTRA,
       gravedad="G = (D + D_adicional) + L = 24 kPa, sin mayorar, en nudos de columna por área tributaria; D_adicional=16 kPa solo G, sin modificar masa ni ELF",
       CM=cm3, lateral="EX: fuerzas ELF y momento r×F en maestros conectados; Δ_i de primer orden en el CM; P_i acumula G desde piso i hasta cubierta",
       formulacion_pdelta="Engine pDelta usa la matriz geométrica consistente Hermite (coeficientes N/(30L)·[36, 3L, 4L², −L²]; crates/engine-elements-frame3d/src/geometric.rs): P-Δ + P-δ por elemento, no P-Δ puro de cuerda.",
       configuracion_frame3dd=dict(obligatoria=True, shear=0, geom=1, elementos_por_columna=1,
           tolerancia_newton_raphson_max=1e-9,
           descripcion="Kg consistente: shear=0 sin deformación por cortante; geom=1 con rigidez geométrica activada; 1 elemento por columna igual que el Engine; tolerancia de Newton-Raphson ≤1e-9.",
           diafragma=dict(A_m2=SLAB_A, E_Pa=E, EA_N=E*SLAB_A,
               representacion="Frame3DD no ofrece constraints de diafragma: se debe declarar su representación con la penalización de diagonales axiales de E5b, misma disposición y mismo EA/A. Efecto medido ≤5.3e-6 (J4 r2)."),
           torsion_alabeo="Frame3DD incluye términos geométricos de torsión-alabeo que el Engine no tiene; en E3 (carga X, planta simétrica) se esperan despreciables, no cuantificado."),
       requisitos_formato_frame3dd=FRAME3DD_FORMAT,
       estabilidad_previa=dict(etiqueta="estimación previa con modelo independiente de J2, no oráculo",
           metodo="frame3d Euler + diafragma XY; numpy en _ref/val032_calibrate.py; primer orden bajo EX", pisos=STABILITY3)),
  [dict(id="E3-lineal", engine_input=doc(m3, [G3, EX3], [], dict(method="linearStatic"))),
   dict(id="E3-pdelta", engine_input=doc(m3, [G3, EX3], [], dict(method="pDelta", options=dict(gravity=dict(case="G",factor=1.0), lateral=dict(case="EX",factor=1.0), tolerance=1e-10, max_iterations=100, displacement_scale=0.01, rotation_scale=0.001))))],
  tolmk([EQ,
         ("desplazamientos laterales y derivas P-Δ en CM reales (reconstruidos desde maestros)", "Engine vs oráculo con Kg consistente (Frame3DD geom. no lineal)", 5e-3, "relativa", "Configuración obligatoria según info.configuracion_frame3dd; comparación pendiente de corrida. " + FRAME3DD_FORMAT),
         ("desplazamientos laterales y derivas P-Δ en CM reales (reconstruidos desde maestros)", "Engine vs oráculo con transformación geométrica P-Δ (solo cuerda)", 5e-2, "relativa", "Diagnóstico: diferencia de formulación P-Δ + P-δ por elemento frente a solo cuerda, 0.5–3.2 % por piso medida por J4.", "diagnóstico, no criterio (diferencia de formulación)"),
         ("factor de amplificación de deriva Δ_PΔ/Δ_lin por piso", "Engine vs 1/(1−θ_i) con θ de primer orden", 2e-2, "relativa", "Comparar amplificaciones de las derivas de entrepiso; 1/(1−θ) es aproximación de piso, pendiente de corrida."),
         ("θ_i de primer orden", "Engine vs oráculos", 1e-4, "relativa", "Lineal; mismo cálculo.")]))
# E5 combinaciones
D5, L5 = grav(5, qD, "D"), grav(5, qL, "L")
EX5, EY5 = lat(m4, e4['F'], "Ux", "EX", cm), lat(m4, e4['F'], "Uy", "EY", cm)
combos = []
def cb(i, t): combos.append(dict(id=i, terms=[dict(case=c, factor=f) for c, f in t]))
cb("C1_1.4D", [("D",1.4)]); cb("C2_1.2D+1.6L", [("D",1.2),("L",1.6)])
for e in ("EX","EY"):
    for s, sg in (("+",1.0),("-",-1.0)):
        cb(f"C5_1.2D+L{s}{e}", [("D",1.2),("L",1.0),(e,sg)]); cb(f"C7_0.9D{s}{e}", [("D",0.9),(e,sg)])
fixture("E5", "Combinaciones gravedad + sismo X/Y con envolventes (geometría E4)",
  ["combinaciones lineales", "envolventes max/min (post-proceso)"], m4,
  dict(ELF=e4, combinaciones_cita="NEC-SE-CG 2015 §3.4.3: numeración 1, 2, 5, 7 y factores NO VERIFICADOS", envolvente="max/min sobre las 10 combinaciones, calculada fuera del Engine (engine-input/1 no declara envolventes en linearStatic)"),
  [dict(id="E5-combinaciones", engine_input=doc(m4, [D5, L5, EX5, EY5], combos, dict(method="linearStatic")))],
  tolmk([EQ,
         ("reacciones por combinación", "Engine vs oráculos", 1e-6, "relativa", "Superposición lineal exacta."),
         ("fuerzas y momentos de barra N,Vy,Vz,T,My,Mz: casos, combinaciones y envolventes", "diagnóstico, no criterio (#84)", None, "diagnóstico, no criterio (#84)", "Recuperación con diafragma sin cobertura: API-W-DIAPHRAGM-RECOVERY; no determina aceptación."),
         ("combinación = Σ factor·caso (desplazamientos y reacciones)", "Engine interno", 1e-12, "relativa", "Identidad algebraica; cualquier recuperación de barra queda solo como diagnóstico #84.")]))
# E5b: E4 sin masas ni MPC; losa de rigidez finita mediante celosía.
m5b = copy.deepcopy(m4)
m5b['masses'] = []
m5b['diaphragms'] = []
m5b['sections'].append(dict(id="losa_diagonal", properties=[dict(name="A", value=SLAB_A)]))
m5b['laws'].append(dict(name="losa_elastica", parameters=[dict(name="E",value=E), dict(name="rho",value=0.0)]))
mid = max(b['id'] for b in m5b['members'])
for k in range(1,6):
    for j in range(2):
        for i in range(3):
            for ends in ([nid(k,i,j),nid(k,i+1,j+1)], [nid(k,i+1,j),nid(k,i,j+1)]):
                mid += 1
                m5b['members'].append(dict(id=mid, nodes=ends, section="losa_diagonal", law="losa_elastica", element="truss3d.linear"))
def lat_tributary(F, dof, name):
    return dict(name=name, loads=[dict(node=nid(k,i,j), values=[dict(dof=dof, value=F[k-1]*trib(x,y)/area)])
        for k in range(1,6) for j,y in enumerate(LY) for i,x in enumerate(LX)])
fixture("E5b", "Combinaciones sobre E4 sin diafragmas: losa mediante diagonales truss3d.linear",
  ["fuerzas y momentos de barra como criterio", "combinaciones y envolventes", "equilibrio global"], m5b,
  dict(ELF=e4, carga_lateral="EX/EY repartidas por área tributaria; mismas resultantes y CM que E4",
       losa=dict(A_m2=SLAB_A, E_Pa=E, EA_N=E*SLAB_A, diagonales_por_pano=2, diagonales_totales=60,
           justificacion="Rigidez finita de penalización: EA/Ldiag≈3.52e10 N/m, ≈611 veces 12EcIcol/h³ de una columna y ≈13000 veces 12EcIz_viga/5³. Solo rigidez axial en plano; no es losa física ni igualdad exacta con MPC. Los cruces no añaden nudo."),
       requisitos_oraculos="La salida de los oráculos debe imprimir ≥10 cifras significativas para ser comparable como criterio. " + FRAME3DD_FORMAT + " Si un oráculo modela truss como barra con I pequeña, debe declarar esa aproximación y la I utilizada.",
       combinaciones_cita=fixtures['E5']['info']['combinaciones_cita'], envolvente=fixtures['E5']['info']['envolvente']),
  [dict(id="E5b-combinaciones", engine_input=doc(m5b,
      [D5,L5,lat_tributary(e4['F'],"Ux","EX"),lat_tributary(e4['F'],"Uy","EY")], combos, dict(method="linearStatic")))],
  tolmk([EQ,
         ("fuerzas y momentos de barra N,Vy,Vz,T,My,Mz por caso, combinación y envolvente (truss: solo N)", "Engine vs oráculos", 1e-6, "relativa con piso absoluto por componente", "|Δ| ≤ 1e-6·|ref| o |Δ| ≤ 1e-6·max|componente del mismo tipo en el caso|. El máximo se toma en la referencia, por separado para N, Vy, Vz, T, My y Mz, sin mezclar casos; para combinaciones y envolventes se usa su propio conjunto de referencia. Mismas estaciones de extremo y convención local; no depende de #84."),
         ("reacciones por combinación", "Engine vs oráculos", 1e-6, "relativa", "Superposición lineal."),
         ("combinación = Σ factor·caso", "Engine interno", 1e-12, "relativa", "Identidad algebraica.")]))

# El formato condiciona los criterios de E3 y E5b frente a Frame3DD (S1 de J4 r2),
# también los heredados de fixture(); conserva los umbrales preregistrados para cada métrica.
for fid in ('E3', 'E5b'):
    for tolerance in fixtures[fid]['tolerances_preregistered']:
        comparison = tolerance['comparison']
        if ('oráculo' in comparison or 'Frame3DD' in comparison) and FRAME3DD_FORMAT not in tolerance['justification']:
            tolerance['justification'] += ' ' + FRAME3DD_FORMAT

# Contrato común del postproceso: ambos lados usan este mismo archivo y tabla.
for fid in ('E2', 'E2b'):
    fixtures[fid]['info']['postproc_contract'] = dict(schema="val032-modal/1", script="_ref/postproc_derivas.py",
        Sa_table="nec_se_ds_tabla_Sa.csv (columnas T_s y Sa_m_s2 = pairs)", xi=NEC['amortiguamiento'],
        corners_xy=[[0,0],[15,10]], consumidores="Engine y oráculos; misma normalización de phi_n y Gamma_n; ver README")

for fid, fixture_data in fixtures.items():
    model = fixture_data['model']
    coords = {n['id']: n['coordinates'] for n in model['nodes']}
    connected = {n for member in model['members'] for n in member['nodes']}
    assert set(coords) <= connected
    assert all(coords[s['node']][2] == 0 for s in model['supports'])
    floors = []
    for k, dia in enumerate(model['diaphragms'], 1):
        assert dia['master'] not in dia['slaves'] and len(dia['slaves']) == 11
        assert all(coords[n][2] == k*H > 0 for n in [dia['master'], *dia['slaves']])
        weights = [dict(node=a['node'], w_kg=next(v['value'] for v in a['values'] if v['dof'] == 'Ux'))
                   for a in model['masses'] if coords[a['node']][2] == k*H and any(v['dof'] == 'Ux' for v in a['values'])]
        extra = sum(v['value'] for a in model['masses'] if coords[a['node']][2] == k*H
                    for v in a['values'] if v['dof'] == 'Rz')
        floors.append(dict(piso=k, master=dia['master'], coordinates=coords[dia['master']],
                           pesos=weights, Rz_extra_kgm2=extra))
    fixture_data['info']['diafragma_r3'] = dict(pisos=floors,
        tratamiento=("Maestro conectado más cercano al CM entre nodos con x≠x_cm e y≠y_cm para evitar momento transportado nulo y cancelación en recuperación #84. Desempate: mayor x, menor y, menor id; maestro excluido de esclavos; bases fuera del diafragma; Uz/Rx/Ry libres en todos los nodos de piso."),
        masa="Ux=Uy=w_i≥0, Σw_i=m, Σw_i·r_i=m·r_cm. Rz_extra=Iz_cm+m·|r_cm-r_master|²−Σw_i·|r_i-r_master|²≥0. Solo m·d² en Iz no conserva el acoplamiento con masa diagonal; el reparto sí es exacto (prueba ≤1e-10)." if floors else "Sin masas ni diafragmas; modelo físico conservado.",
        cargas="Fuerza lateral conservada, momento adicional Mz=(r_cm-r_master)×F; E5b conserva reparto tributario.",
        evidencia="_ref/calibrate_r3.json")
    for run in fixture_data['runs']:
        method = run['engine_input']['analyses'][0]['method']
        blocked = method in ('modal', 'responseSpectrum')
        if blocked:
            state, status, cause = ("bloqueado por ENG-E-0006", "unstable",
                "El Engine no condensa los 180 GDL Uz/Rx/Ry libres sin masa; falla en Uz del nodo 101. Issue del Engine pendiente de abrir.")
        elif run['id'] == 'E3-pdelta':
            state, status, cause = ("bloqueado por ENG-E-0007", "notConverged",
                "Defecto del Engine (pDelta + diafragma): las fuerzas geométricas y el residual no se proyectan por la transformación del diafragma; el residual no baja de 1.14e6. Issue del Engine pendiente de abrir; corrección en ticket aparte del Engine.")
        else:
            state, status, cause = "Warning", "warning", "Sin masa artificial ni apoyos de piso añadidos."
        run['info'] = dict(estado=state, status_engine=status, seguimiento="MAM-847", causa=cause)
        run['info']['warnings'] = []
        if model['diaphragms']:
            run['info']['warnings'].append(dict(codigo="API-W-DIAPHRAGM-RECOVERY", cantidad=1,
                causa="#84: recuperación con diafragmas sin cobertura de pruebas del dominio; revisar equilibrio y resultados."))
        if run['engine_input']['combinations']:
            run['info']['warnings'].append(dict(codigo="API-W-COMBINED-PRESCRIBED", cantidad=10,
                causa="#85: cada combinación suma factor·δ de los prescritos. El Engine avisa también para los empotramientos de base con δ=0; no hay asentamientos impuestos."))
        run['info']['evidencia_engine'] = "_ref/engine_r3.json"
    fixture_data['info']['estado_engine'] = [dict(run=r['id'], **r['info']) for r in fixture_data['runs']]

def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=1, allow_nan=False)+"\n", encoding="utf-8")
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

# Campos obligatorios de espectro: copiados de la referencia física común.
# En E3/E5/E5b no solicitan análisis dinámico: no crean runs adicionales.
REFERENCE_SPECTRUM = dict(zone=NEC['zona'], Z=NEC['Z'], soil=NEC['suelo'],
    R=NEC['R'], eta=NEC['eta'], Fa=NEC['Fa'], Fd=NEC['Fd'], Fs=NEC['Fs'],
    zeta=NEC['amortiguamiento'], g=g, r=NEC['r'],
    directions=[dict(axis="X", vector=[1,0,0])],
    directional_combination=dict(method="srss", factors=[1.0]),
    interpolation="linear", extrapolation="error", Sa=dict(unit="m/s^2", pairs=SA_TABLE))

idx = {"schema":"val032-index/1","case":"VAL-032","linear":"MAM-847","generator":"_ref/gen.py",
       "bench_model_spec":BENCH_MODEL_SPEC,"entries":[],"bench_models":[],"bench_model_omissions":[],"herramientas":[],
       "bench_model_translation":dict(comparison="exacta; documentos completos y listas ordenadas",
           translator="_ref/bench_translate.py", reference_spectrum=REFERENCE_SPECTRUM,
           id_policy="Biyectiva por namespace; nodes/members según orden original, contiguos desde 1. "
                     "Secciones conservadas; leyes/apoyos/masas renombrados como translate.rs. "
                     "Casos, combinaciones y sus referencias mantienen identidad.",
           engine_execution=False, checks=[])}
# Patch r3: retirar versiones anteriores solo de estos siete fixtures.
for fid in ("E1","E2","E2b","E3","E4","E5","E5b"):
    d = fixtures[fid]
    path = OUT/f"{fid}-v{d['version']}.json"
    for previous in OUT.glob(f"{fid}-v*.json"):
        if previous != path:
            previous.unlink()
    write_json(path, d)
    idx["entries"].append(dict(id=fid, version=d['version'], path=path.name, sha256=sha(path),
        nodes=d['node_count'], members=d['member_count'], runs=[r['id'] for r in d['runs']]))
    bench, bijections, selections = make_bench(d, REFERENCE_SPECTRUM)
    gaps = []
    for run, selection in zip(d['runs'], selections):
        translated = translate(bench, selection['method'], selection.get('variant'))
        require_exact(translated, remap_document(run['engine_input'], bijections))
        for gap in engine_gaps(translated):
            if gap not in gaps:
                gaps.append(gap)
    check = dict(id=fid, id_bijections=bijections, runs=selections,
                 exact_translation=True, engine_limits="incompatible" if gaps else "compatible")
    idx['bench_model_translation']['checks'].append(check)
    bench_path = OUT/'bench-model'/f'{fid}.json'
    check['engine_gaps'] = gaps
    bench_path.parent.mkdir(exist_ok=True)
    write_json(bench_path, bench)
    idx['bench_models'].append(dict(id=fid, path=str(bench_path.relative_to(OUT)), sha256=sha(bench_path),
        estado_engine="; ".join(f"bloqueado por {code}" for code in dict.fromkeys(gap['codigo'] for gap in gaps)) if gaps else "ejecutable",
        seguimiento="MAM-847"))
csv_path = OUT/"nec_se_ds_tabla_Sa.csv"
csv_bytes = ("T_s,Sa_g,Sa_m_s2\n"+"".join(f"{T},{Sa(T)},{v}\n" for T, v in SA_TABLE)).encode('utf-8')
if csv_path.exists():
    if csv_path.read_bytes() != csv_bytes:
        raise ValueError('El generador alteraría el CSV existente')
else:
    csv_path.write_bytes(csv_bytes)
for rel in ("_ref/gen.py", "_ref/validate_bench.py", "_ref/bench_translate.py", "_ref/postproc_derivas.py", "_ref/val032_calibrate.py", "_ref/calibrate_r3.json", "_ref/record_engine.py", "_ref/engine_r3.json", "_ref/engine-input-1.json", "nec_se_ds_tabla_Sa.csv"):
    idx['herramientas'].append(dict(path=rel, sha256=sha(OUT/rel)))
idx['calibration_r3'] = dict(path="_ref/calibrate_r3.json", sha256=sha(OUT/'_ref/calibrate_r3.json'),
    tolerancia_relativa=1e-10, max_error_relativo_por_edificio={e['id']: e['max_error_relativo']
        for e in json.loads((OUT/'_ref/calibrate_r3.json').read_text())['edificios']})
idx['engine_verification_r3'] = dict(path="_ref/engine_r3.json", sha256=sha(OUT/'_ref/engine_r3.json'),
    test="crates/engine-api/tests/val032_runs.rs", seguimiento="MAM-847",
    fuentes=[dict(path=str(p.relative_to(OUT.parents[3])), sha256=sha(p)) for p in (
        OUT.parents[3]/'crates/engine-api/tests/val032_runs.rs',
        OUT.parents[3]/'crates/engine-analysis/src/pdelta.rs',
        OUT.parents[3]/'crates/engine-analysis/src/modal.rs')])
write_json(OUT/"index.json", idx)
print(json.dumps(dict(E=E,T0=T0,Tc=Tc,theta3=STABILITY3,
    modelos=[dict(id=e['id'],nudos=e['nodes'],barras=e['members']) for e in idx['entries']],
    bench_model_emitidos=[e['id'] for e in idx['bench_models']],
    bench_model_omitidos=[e['id'] for e in idx['bench_model_omissions']]), ensure_ascii=False, indent=1))
