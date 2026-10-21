#!/usr/bin/env python3
"""Genera datos/<caso>/referencia.csv con soluciones analíticas propias.
Derivaciones propias (clean-room); fuentes solo citadas. Uso: python3 herramientas/gen_referencia.py"""
import math, os, csv
D=os.path.join(os.path.dirname(__file__),'..','datos')
def w(case,header,rows):
    os.makedirs(f'{D}/{case}',exist_ok=True)
    with open(f'{D}/{case}/referencia.csv','w',newline='') as f:
        c=csv.writer(f); c.writerow(header)
        for r in rows: c.writerow([f'{v:.10g}' for v in r])
KIP=4.4482216152605; IN=0.0254
E=29000*KIP*1e3/IN**2; I=484*IN**4; L=336*IN; EI=E*I   # W14x48, eje fuerte (AISC 360-16 C2)
# VAL-018: biapoyada, carga uniforme w + axial P (Euler-Bernoulli, sin cortante)
wq=0.200*KIP*1e3/(12*IN)
rows=[]
for Pk in (0,150,300,450):
    P=Pk*KIP*1e3; u=L/2*math.sqrt(P/EI) if P>0 else 0
    M0=wq*L**2/8; d0=5*wq*L**4/(384*EI)
    fM=1 if u==0 else 2*(1/math.cos(u)-1)/u**2
    fd=1 if u==0 else 12*(2/math.cos(u)-2-u**2)/(5*u**4)
    rows.append((P/1e3,M0*fM/1e3,d0*fd*1e3))
w('VAL-018',['P_kN','M_kNm','d_mm'],rows)
# VAL-019: voladizo con H en la punta + axial P
H=1.0*KIP*1e3; rows=[]
for Pk in (0,100,150,200):
    P=Pk*KIP*1e3
    if P==0: M=H*L; d=H*L**3/(3*EI)
    else:
        k=math.sqrt(P/EI); M=H*math.tan(k*L)/k; d=H*(math.tan(k*L)-k*L)/(P*k)
    rows.append((P/1e3,M/1e3,d*1e3))
w('VAL-019',['P_kN','M_kNm','d_mm'],rows)
# VAL-020: factores de amplificación de viga-columna biapoyada con carga uniforme
rows=[]
for i in range(1,31):
    u=i*0.05  # u = (L/2) sqrt(P/EI) < pi/2
    rows.append((u,2*(1/math.cos(u)-1)/u**2,12*(2/math.cos(u)-2-u**2)/(5*u**4)))
w('VAL-020',['u','chiM','chiD'],rows)
# VAL-021: 1 GDL, u = Sa/omega^2 con Sa = 1 m/s^2 (espectro unitario)
rows=[(T,(T/(2*math.pi))**2*1.0*1e3) for T in [0.1*i for i in range(1,31)]]
w('VAL-021',['T_s','u_mm'],rows)
# VAL-024: grado medio de consolidación de Terzaghi, capa homogénea (serie de Fourier)
def U(Tv):
    s=0
    for m in range(200):
        M=math.pi*(2*m+1)/2; s+=2/M**2*math.exp(-M**2*Tv)
    return 1-s
w('VAL-024',['Tv','U'],[(Tv,U(Tv)) for Tv in [0.005*1.25**i for i in range(30)]])
