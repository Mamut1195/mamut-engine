"""Proyección tabular de results/1; copia y etiquetas, sin cálculo físico."""
from __future__ import annotations

from dataclasses import dataclass
import math
from typing import Any, cast

from . import _validation as checks
from .errors import MissingNumPyError, Unsupported
from .types import ResultsDocument

DOFS = ("Ux", "Uy", "Uz", "Rx", "Ry", "Rz")
DISPLACEMENT_UNITS = ("m", "m", "m", "rad", "rad", "rad")
FORCE_UNITS = ("N", "N", "N", "N*m", "N*m", "N*m")
LOCAL_FIELDS = ("N", "Vy", "Vz", "T", "My", "Mz")
# Orden publicado por los elementos de engine-api; no se infiere del ancho.
ELEMENT_DOFS = {
    "frame2d.euler_elastic": ("Ux", "Uy", "Rz"),
    "frame3d.euler_elastic": DOFS,
    "frame2d.winkler": ("Ux", "Uy", "Rz"),
    "frame3d.winkler": DOFS,
    "truss2d.linear": ("Ux", "Uy"),
    "truss3d.linear": ("Ux", "Uy", "Uz"),
}


@dataclass(frozen=True)
class ResultMatrix:
    """Tabla (filas, columnas), listas o ndarray f64 C con memoria propia.

    row_ids son enteros exactos, nunca índices ni floats. row_indices distingue
    estaciones del mismo miembro. dofs y units etiquetan cada columna; frame
    declara el sistema de referencia. values es Any para mantener NumPy opcional
    también al resolver anotaciones en runtime.
    """

    values: Any
    shape: tuple[int, int]
    row_ids: tuple[int, ...]
    row_indices: tuple[int, ...]
    dofs: tuple[str, ...]
    units: tuple[str, ...]
    frame: str


@dataclass(frozen=True)
class ResultBlock:
    """Tablas de una ubicación exacta de document, sin mezclar casos/modos."""

    path: tuple[str | int, ...]
    matrices: dict[str, ResultMatrix]


@dataclass(frozen=True)
class ArrayResults:
    """Export independiente; document conserva estado y todos los campos JSON.

    El estado textual, diagnósticos, identidades, conectividad, GDL activos,
    prescritos nullable, extremos y convergencia permanecen en document.
    """

    document: ResultsDocument
    blocks: tuple[ResultBlock, ...]

    @property
    def status(self) -> str:
        """Estado original, sin codificarlo como un número físico."""
        return self.document["status"]

    @property
    def partial(self) -> bool:
        """Mismo criterio de completitud que Results.partial."""
        return self.status not in ("ok", "warning")


def export(document: ResultsDocument, *, numpy: bool) -> ArrayResults:
    """Recibe una copia del snapshot; ninguna tabla referencia sus listas."""
    np: Any = None
    if numpy:
        try:
            import numpy as np
        except ModuleNotFoundError as exc:
            if exc.name != "numpy":
                raise
            raise MissingNumPyError(
                "Results.to_numpy requiere el extra mamut-engine[numpy]",
                code="PY-E-NUMPY",
            ) from exc

    def table(rows: list[list[float]], ids: list[int], indices: list[int],
              dofs: tuple[str, ...], units: tuple[str, ...], frame: str) -> ResultMatrix:
        if len(rows) != len(ids) or len(rows) != len(indices):
            raise checks.invalid("filas e identidades incompatibles")
        copied: list[list[float]] = []
        for row in rows:
            if len(row) != len(dofs):
                raise checks.invalid("shape de resultado incompatible con sus GDL")
            if any(isinstance(v, bool) or type(v) not in (int, float) for v in row):
                raise checks.invalid("se requieren valores f64 finitos")
            try:
                floats = [float(v) for v in row]
            except OverflowError:
                raise checks.invalid("valor no representable como f64 finito") from None
            if any(not math.isfinite(v) for v in floats):
                raise checks.invalid("se requieren valores f64 finitos")
            copied.append(floats)
        shape = (len(rows), len(dofs))
        values = (np.asarray(copied, dtype=np.float64, order="C")
                  .reshape(shape).copy(order="C")) if numpy else copied
        return ResultMatrix(values, shape, tuple(ids), tuple(indices), dofs, units, frame)

    identities = {m["id"]: m for m in document["members"]}
    node_ids = {n["id"] for n in document["nodes"]}
    if (len(identities) != len(document["members"])
            or len(node_ids) != len(document["nodes"])):
        raise checks.invalid("IDs de identidad duplicados")
    blocks: list[ResultBlock] = []

    def block(data: dict[str, Any], path: tuple[str | int, ...],
              *, modal: bool = False) -> None:
        nodes = data["nodes"]
        ids = [n["id"] for n in nodes]
        if len(set(ids)) != len(ids) or any(i not in node_ids for i in ids):
            raise checks.invalid("IDs nodales duplicados o sin identidad")
        indices = [0] * len(ids)
        units = (("kg^-1/2",) * 3 + ("m^-1*kg^-1/2",) * 3
                 if modal else DISPLACEMENT_UNITS)
        matrices = {
            "displacements": table([n["displacements"] for n in nodes], ids, indices,
                                   DOFS, units, "global"),
            "reactions": table([n["reactions"] for n in nodes], ids, indices,
                               DOFS, FORCE_UNITS, "global"),
        }
        members = data.get("members", [])
        seen: set[int] = set()
        groups: dict[tuple[str, tuple[str, ...]], list[dict[str, Any]]] = {}
        station_rows: list[list[float]] = []
        positions: list[list[float]] = []
        station_ids: list[int] = []
        station_indices: list[int] = []
        for member in members:
            mid = member["id"]
            if mid in seen or mid not in identities:
                raise checks.invalid("ID de elemento duplicado o sin identidad")
            seen.add(mid)
            kind = identities[mid]["element"]
            if kind in ELEMENT_DOFS:
                labels = tuple(f"{end}.{dof}" for end in ("i", "j")
                               for dof in ELEMENT_DOFS[kind])
                group_name = kind
            elif kind in ("spring.linear", "spring.nonlinear"):
                labels = tuple(identities[mid].get("end_dofs", []))
                if (not labels or any(label not in {f"{end}.{dof}"
                        for end in ("i", "j") for dof in DOFS} for label in labels)):
                    raise checks.invalid("resorte sin etiquetas locales válidas")
                # Resortes con distintos GDL o conexión requieren tablas distintas.
                group_name = kind + "/" + "_".join(labels)
            else:
                raise Unsupported(f"Etiquetas de elemento no soportadas: {kind}",
                                  code="ENG-E-0013")
            groups.setdefault((group_name, labels), []).append(member)
            for index, station in enumerate(member["stations"]):
                station_ids.append(mid)
                station_indices.append(index)
                station_rows.append(station["forces_local"])
                positions.append([station["position"]])
        for (kind, labels), group in groups.items():
            force_units = tuple("N*m" if label.split(".", 1)[1].startswith("R") else "N"
                                for label in labels)
            matrices[f"end_forces_local/{kind}"] = table(
                [m["end_forces_local"] for m in group], [m["id"] for m in group],
                [0] * len(group), labels, force_units, "local")
        if "members" in data:
            matrices["forces_local"] = table(station_rows, station_ids, station_indices,
                                             LOCAL_FIELDS, FORCE_UNITS, "local")
            matrices["positions"] = table(positions, station_ids, station_indices,
                                          ("x/L",), ("1",), "local")
        blocks.append(ResultBlock(path, matrices))

    raw = cast(dict[str, Any], document)
    for index, case in enumerate(raw["cases"]):
        block(case, ("cases", index))
    for index, combination in enumerate(raw["combinations"]):
        block(combination["results"], ("combinations", index, "results"))
    for index, mode in enumerate(raw["modes"]):
        block({"nodes": mode["shape"]}, ("modes", index, "shape"), modal=True)
        for response_index, response in enumerate(mode["spectrum_responses"]):
            block(response, ("modes", index, "spectrum_responses", response_index))
    for index, envelope in enumerate(raw["envelopes"]):
        block(envelope, ("envelopes", index))
    buckling = raw.get("buckling")
    if buckling:
        for index, mode in enumerate(buckling["modes"]):
            block({"nodes": mode["shape"]}, ("buckling", "modes", index, "shape"))
    return ArrayResults(document, tuple(blocks))
