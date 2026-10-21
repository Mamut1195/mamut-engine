"""MAM-860: referencia propia E3, Euler 3D con pretensión inicial congelada.

Derivación energética propia: U=1/2 int[EA*u'^2+GJ*phi'^2+
EIy*w''^2+EIz*v''^2+N*(v'^2+w'^2)] dx. Interpolación axial/torsional
lineal y transversal Hermite cúbica. La integración exacta de H' H' da
la matriz geométrica estándar N/(30L), también descrita en McGuire,
Gallagher y Ziemian, Matrix Structural Analysis, 2nd ed. (2000), cap. 9
(análisis de segundo orden). No se copia ningún listado de ese libro.

Solo lee la entrada JSON; no importa ni ejecuta Rust ni otros motores.
NumPy preinstalado (BSD-3-Clause); sin red, sin consultar su implementación.
Los nudos interiores de cada columna no tienen cargas ni restricciones:
su eliminación por complemento de Schur es exactamente el ensamblaje de
n elementos, no una aproximación de resorte P/L. Las vigas no se subdividen.
"""
import hashlib
import json
import math
import os
from pathlib import Path
import sys

# Un solo hilo fija el orden de las reducciones en el entorno offline de generación.
os.environ["OPENBLAS_NUM_THREADS"] = "1"
import numpy as np

HERE = Path(__file__).resolve().parent
DOFS = ("Ux", "Uy", "Uz", "Rx", "Ry", "Rz")


def local_stiffness(length, section, law, normal=0.0):
    """Segunda variación de U, N positivo en tracción; orden u,v,w,rx,ry,rz."""
    l = length
    matrix = np.zeros((12, 12))
    for a, b, rigidity in ((0, 6, law["E"] * section["A"]),
                          (3, 9, law["G"] * section["J"])):
        matrix[np.ix_([a, b], [a, b])] = rigidity / l * np.array([[1, -1], [-1, 1]])
    elastic = np.array([[12, 6*l, -12, 6*l],
                        [6*l, 4*l*l, -6*l, 2*l*l],
                        [-12, -6*l, 12, -6*l],
                        [6*l, 2*l*l, -6*l, 4*l*l]]) / l**3
    # H=(1-3s²+2s³, L(s-2s²+s³), 3s²-2s³, L(-s²+s³)).
    # int_0^L H'_a H'_b dx, evaluada algebraicamente.
    geometric = normal / (30*l) * np.array([
        [36, 3*l, -36, 3*l], [3*l, 4*l*l, -3*l, -l*l],
        [-36, -3*l, 36, -3*l], [3*l, -l*l, -3*l, 4*l*l]])
    for indices, inertia, signs in (([1, 5, 7, 11], "Iz", [1, 1, 1, 1]),
                                    ([2, 4, 8, 10], "Iy", [1, -1, 1, -1])):
        matrix[np.ix_(indices, indices)] = (
            law["E"] * section[inertia] * elastic + geometric) * np.outer(signs, signs)
    return matrix


def column_stiffness(length, section, law, normal, count):
    """Ensamblaje explícito de count elementos y condensación de nudos internos."""
    if count == 1:
        return local_stiffness(length, section, law, normal)
    matrix = np.zeros((6*(count+1), 6*(count+1)))
    element = local_stiffness(length/count, section, law, normal)
    for i in range(count):
        matrix[6*i:6*i+12, 6*i:6*i+12] += element
    ends = list(range(6)) + list(range(6*count, 6*count+6))
    inner = list(range(6, 6*count))
    coupling = matrix[np.ix_(ends, inner)]
    condensed = matrix[np.ix_(ends, ends)] - coupling @ np.linalg.solve(
        matrix[np.ix_(inner, inner)], coupling.T)
    return (condensed + condensed.T)/2


def prepare(model):
    coords = {n["id"]: np.array(n["coordinates"], dtype=float) for n in model["nodes"]}
    ids = sorted(coords)
    offset = {node: 6*i for i, node in enumerate(ids)}
    nd = 6*len(ids)
    sections = {s["id"]: {p["name"]: p["value"] for p in s["properties"]}
                for s in model["sections"]}
    laws = {s["name"]: {p["name"]: p["value"] for p in s["parameters"]}
            for s in model["laws"]}
    bars = []
    for member in model["members"]:
        assert member["element"] == "frame3d.euler_elastic"
        assert not member.get("releases")
        a, b = member["nodes"]
        delta = coords[b] - coords[a]
        length = float(np.linalg.norm(delta))
        x = delta/length
        y = np.cross(member["orientation"], x)
        y /= np.linalg.norm(y)
        rotation = np.array([x, y, np.cross(x, y)])
        transform = np.kron(np.eye(4), rotation)
        indices = [offset[n]+j for n in (a, b) for j in range(6)]
        column = bool(np.all(delta[:2] == 0) and delta[2] != 0)
        bars.append(dict(id=member["id"], length=length, section=sections[member["section"]],
                         law=laws[member["law"]], indices=indices,
                         transform=transform, column=column))
    fixed = {offset[s["node"]]+DOFS.index(d) for s in model["supports"] for d in s["restrained"]}
    assert all(not s.get("prescribed") for s in model["supports"])
    slaves = {s: d["master"] for d in model["diaphragms"] for s in d["slaves"]}
    free = [offset[n]+j for n in ids for j in range(6)
            if offset[n]+j not in fixed and not (n in slaves and j in (0, 1, 5))]
    reduced = {d: i for i, d in enumerate(free)}
    t = np.zeros((nd, len(free)))
    for d, i in reduced.items():
        t[d, i] = 1
    for slave, master in slaves.items():
        dx, dy, _ = coords[slave] - coords[master]
        ux, uy, rz = (reduced[offset[master]+j] for j in (0, 1, 5))
        t[offset[slave], ux] = 1
        t[offset[slave], rz] = -dy
        t[offset[slave]+1, uy] = 1
        t[offset[slave]+1, rz] = dx
        t[offset[slave]+5, rz] = 1
    return coords, offset, bars, t


def assemble(bars, nd, normals, count):
    matrix = np.zeros((nd, nd))
    for bar, normal in zip(bars, normals):
        local = column_stiffness(bar["length"], bar["section"], bar["law"], normal,
                                 count if bar["column"] else 1)
        transform = bar["transform"]
        matrix[np.ix_(bar["indices"], bar["indices"])] += transform.T @ local @ transform
    return matrix


def generate(source_path):
    source = source_path.read_bytes()
    fixture = json.loads(source)
    runs = [r for r in fixture["runs"] if r["id"] == "E3-pdelta"]
    assert len(runs) == 1
    document = runs[0]["engine_input"]
    model = document["model"]
    coords, offset, bars, t = prepare(model)
    nd = t.shape[0]
    options = document["analyses"][0]["options"]
    cases = {c["name"]: c["loads"] for c in document["cases"]}

    def force(option):
        f = np.zeros(nd)
        for load in cases[option["case"]]:
            for value in load["values"]:
                f[offset[load["node"]]+DOFS.index(value["dof"])] += option["factor"] * value["value"]
        return f

    fg, fl = force(options["gravity"]), force(options["lateral"])
    elastic = assemble(bars, nd, [0.0]*len(bars), 1)
    kr = t.T @ elastic @ t
    ug = t @ np.linalg.solve(kr, t.T @ fg)
    linear = t @ np.linalg.solve(kr, t.T @ fl)
    normals = []
    for bar in bars:
        local = bar["transform"] @ ug[bar["indices"]]
        normals.append(float(bar["law"]["E"] * bar["section"]["A"] / bar["length"]
                             * (local[6]-local[0])))
    floors = sorted(model["diaphragms"], key=lambda d: coords[d["master"]][2])

    def floor_rows(displacement):
        rows = []
        last_z = last_u = last_linear = 0.0
        for i, dia in enumerate(floors, 1):
            master = dia["master"]
            _, my, z = coords[master]
            masses = [(m["node"], v["value"]) for m in model["masses"] for v in m["values"]
                      if v["dof"] == "Ux" and coords[m["node"]][2] == z]
            cm = sum(coords[n][:2]*w for n, w in masses)/sum(w for _, w in masses)
            d = offset[master]
            ux = float(displacement[d] - (cm[1]-my)*displacement[d+5])
            ux_linear = float(linear[d] - (cm[1]-my)*linear[d+5])
            drift, drift_linear = ux-last_u, ux_linear-last_linear
            above = [offset[n] for n in coords if coords[n][2] >= z]
            p, v = -float(sum(fg[d+2] for d in above)), float(sum(fl[d] for d in above))
            h = float(z-last_z)
            theta = p*drift_linear/(v*h)
            amp = drift/drift_linear
            rows.append(dict(floor=i, master=master, cm_m=cm.tolist(), height_m=h,
                             gravity_above_N=p, shear_above_N=v,
                             linear_drift_m=drift_linear, pdelta_drift_m=drift,
                             amplification=amp, theta_linear=theta,
                             theta_pdelta=p*drift/(v*h), chord_excess=amp*(1-theta)-1))
            last_z, last_u, last_linear = z, ux, ux_linear
        return rows

    meshes = []
    for count in (1, 8, 16):
        # Sin cargas interiores: el N de gravedad es constante en cada columna.
        # La condensación elástica debe reproducir el mismo problema de gravedad.
        refined_elastic = assemble(bars, nd, [0.0]*len(bars), count)
        elastic_error = float(np.max(np.abs(refined_elastic-elastic))/np.max(np.abs(elastic)))
        assert elastic_error < 1e-10
        total = assemble(bars, nd, normals, count)
        reduced = t.T @ total @ t
        eigmin = float(np.linalg.eigvalsh(reduced)[0])
        assert eigmin > 0
        displacement = t @ np.linalg.solve(reduced, t.T @ fl)
        residual = float(np.max(np.abs(t.T @ (total @ displacement-fl)))/np.max(np.abs(t.T @ fl)))
        assert residual < 1e-10
        meshes.append(dict(elements_per_column=count,
                           element_count=sum(count if b["column"] else 1 for b in bars),
                           node_count=len(coords)+(count-1)*sum(b["column"] for b in bars),
                           elastic_condensation_relative_error=elastic_error,
                           min_reduced_total_eigenvalue=eigmin,
                           projected_relative_residual=residual, floors=floor_rows(displacement)))
    convergence = []
    for a, b in zip(meshes[1]["floors"], meshes[2]["floors"]):
        row = dict(floor=a["floor"])
        for metric in ("amplification", "pdelta_drift_m"):
            error = abs(a[metric]/b[metric]-1)
            assert error < 2e-6
            row[metric+"_relative_error_8_vs_16"] = error
        convergence.append(row)
    return dict(schema="e3-pdelta-reference/1", ticket="MAM-860",
                source=dict(kind="derivación energética propia e integración algebraica Hermite",
                            citation="McGuire, Gallagher y Ziemian, Matrix Structural Analysis, 2.ª ed. (2000), cap. 9: análisis de segundo orden; sin copiar listados.",
                            equation="Kg_ab = N integral_0^L H'_a H'_b dx; N = EA/L (ug_bx-ug_ax), positivo a tracción"),
                fixture=source_path.name, fixture_sha256=hashlib.sha256(source).hexdigest(),
                script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                run_id="E3-pdelta", definition="Kg Hermite consistente, N de gravedad lineal congelada; respuesta lateral EX; derivas Ux en CM; amplificación respecto de la misma solución lineal (invariante al refinamiento).",
                axial_forces_N=[dict(member=b["id"], normal_N=n) for b, n in zip(bars, normals)],
                meshes=meshes, convergence_8_vs_16=convergence)


def selftest():
    """Dos comprobaciones analíticas ajenas a las respuestas del Engine."""
    section = dict(A=1.0, Iy=1.0, Iz=1.0, J=1.0)
    law = dict(E=600.0, G=240.0)
    length = 2.0
    p = 0.2 * math.pi**2 * law["E"]/(4*length**2)
    k = math.sqrt(p/law["E"])
    # EI y''''+P y''=0: voladizo comprimido, fuerza transversal unitaria.
    exact = (math.tan(k*length)/k-length)/p
    errors = []
    for count in (1, 8, 16):
        matrix = column_stiffness(length, section, law, -p, count)
        u = np.linalg.solve(matrix[6:, 6:], np.array([0., 1., 0., 0., 0., 0.]))
        errors.append(abs(u[1]/exact-1))
    assert errors[2] < 1e-7 and errors[2] < errors[1] < errors[0]
    # Cuadratura exacta independiente de la matriz geométrica cerrada.
    points, weights = np.polynomial.legendre.leggauss(3)
    quadrature = np.zeros((4, 4))
    for point, weight in zip(points, weights):
        s = (point+1)/2
        dh = np.array([(-6*s+6*s*s)/length, 1-4*s+3*s*s,
                       (6*s-6*s*s)/length, -2*s+3*s*s])
        quadrature += -p*length*weight/2*np.outer(dh, dh)
    kg = local_stiffness(length, section, law, -p)-local_stiffness(length, section, law)
    assert np.max(np.abs(kg[np.ix_([1, 5, 7, 11], [1, 5, 7, 11])]-quadrature)) < 1e-12
    print("2 comprobaciones analíticas passed, 0 failed; errores voladizo:", errors)


def main():
    source = HERE.parent / "E3-v1.1.3.json"
    result = generate(source)
    output = HERE / "e3_pdelta_reference.json"
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2, allow_nan=False)+"\n", encoding="utf-8")
    print(json.dumps(result["convergence_8_vs_16"], indent=2))


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        selftest()
    else:
        assert not sys.argv[1:], "uso: e3_pdelta_reference.py [--selftest]"
        main()
