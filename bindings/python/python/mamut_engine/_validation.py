"""Validación estructural previa al FFI; no contiene física ni solver."""
from __future__ import annotations

import json
import math
import re
from typing import Any, cast

from .errors import InvalidInputError, Unsupported
from .types import JSONDocument, JSONValue

MAX_BYTES = 8 * 1024 * 1024
MAX_SAFE_INTEGER = 2**53 - 1


def invalid(message: str, path: str = "$") -> InvalidInputError:
    return InvalidInputError(f"{path}: {message}", details={"path": path})


def _pairs(pairs: list[tuple[str, JSONValue]]) -> JSONDocument:
    result: JSONDocument = {}
    for key, value in pairs:
        if key in result:
            raise invalid(f"clave duplicada: {key}")
        result[key] = value
    return result


def _constant(value: str) -> None:
    raise invalid(f"literal no finito: {value}")


def check_tree(value: object, path: str = "$", depth: int = 0, *, bounded: bool = True) -> None:
    """Rechaza ciclos/profundidad, no-JSON, enteros fuera de rango y NaN/Inf."""
    if type(value) in (dict, list):
        if bounded and depth >= 64:
            raise invalid("profundidad máxima 64", path)
        if isinstance(value, dict):
            for key, child in value.items():
                if type(key) is not str:
                    raise invalid("las claves deben ser str", path)
                check_tree(child, f"{path}.{key}", depth + 1, bounded=bounded)
        else:
            if bounded and len(value) > 16384:
                raise invalid("máximo 16384 elementos por array", path)
            for index, child in enumerate(value):
                check_tree(child, f"{path}[{index}]", depth + 1, bounded=bounded)
    elif type(value) is float:
        if not math.isfinite(value):
            raise invalid("se requiere número finito", path)
    elif type(value) is int:
        if bounded and not -MAX_SAFE_INTEGER <= value <= MAX_SAFE_INTEGER:
            raise invalid("entero fuera del rango seguro JCS: |n| <= 9007199254740991 (2^53-1)", path)
    elif value is not None and type(value) not in (bool, str):
        raise invalid("se requiere un valor JSON (sin coerciones)", path)


def parse(source: str, *, bounded: bool = True) -> JSONDocument:
    if not isinstance(source, str):
        raise invalid("se requiere JSON str")
    try:
        if bounded and len(source.encode("utf-8")) > MAX_BYTES:
            raise invalid("máximo 8 MiB de entrada")
        value = json.loads(source, object_pairs_hook=_pairs, parse_constant=_constant)
        check_tree(value, bounded=bounded)
    except (ValueError, RecursionError, UnicodeError) as err:
        raise invalid(str(err)) from None
    if not isinstance(value, dict):
        raise invalid("la raíz debe ser un objeto")
    return cast(JSONDocument, value)


def dumps(value: object) -> str:
    check_tree(value)
    try:
        result = json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":"))
        if len(result.encode("utf-8")) > MAX_BYTES:
            raise invalid("máximo 8 MiB de entrada")
        return result
    except (ValueError, UnicodeError) as err:
        raise invalid(str(err)) from None


def contract(value: JSONDocument, expected: str) -> None:
    if "schema" not in value or not isinstance(value["schema"], str):
        raise invalid("schema debe ser str")
    if value["schema"] != expected:
        raise Unsupported(f"Contrato no soportado: {value['schema']}", code="ENG-E-0009")


def structure(value: Any, spec: dict[str, Any], root: dict[str, Any],
              path: str = "$", *, extensions: bool = False) -> None:
    """Subconjunto estructural de los schemas propios; sin resolver referencias externas.

    La versión instalada publica la forma de los argumentos, de modo que los
    campos opcionales nuevos no requieren otra lista rígida en la fachada.
    """
    if "$ref" in spec:
        structure(value, root["$defs"][spec["$ref"].rsplit("/", 1)[1]], root,
                  path, extensions=extensions)
        return
    if "oneOf" in spec:
        for branch in spec["oneOf"]:
            try:
                structure(value, branch, root, path, extensions=extensions)
                return
            except InvalidInputError:
                pass
        raise invalid("ninguna variante del contrato coincide", path)
    kind = spec.get("type")
    kinds = kind if isinstance(kind, list) else [kind]
    valid = {"object": type(value) is dict, "array": type(value) is list,
             "string": type(value) is str, "integer": type(value) is int,
             "number": type(value) in (int, float), "boolean": type(value) is bool,
             "null": value is None, None: True}
    if not any(valid[k] for k in kinds):
        raise invalid(f"tipo requerido: {kind}", path)
    if "const" in spec and value != spec["const"]:
        raise invalid(f"valor requerido: {spec['const']}", path)
    if "enum" in spec and value not in spec["enum"]:
        raise invalid(f"valor fuera de {spec['enum']}", path)
    if isinstance(value, str) and "pattern" in spec and re.search(spec["pattern"], value) is None:
        raise invalid("texto fuera del patrón del contrato", path)
    if type(value) in (int, float):
        for key, fails in (("minimum", lambda n: value < n),
                           ("maximum", lambda n: value > n),
                           ("exclusiveMinimum", lambda n: value <= n),
                           ("exclusiveMaximum", lambda n: value >= n)):
            if key in spec and fails(spec[key]):
                raise invalid(f"límite {key}: {spec[key]}", path)
    if isinstance(value, list):
        if len(value) < spec.get("minItems", 0) or len(value) > spec.get("maxItems", len(value)):
            raise invalid("tamaño de array fuera del contrato", path)
        for i, child in enumerate(value):
            prefix = spec.get("prefixItems", [])
            child_spec = prefix[i] if i < len(prefix) else spec.get("items", {})
            structure(child, child_spec, root, f"{path}[{i}]", extensions=extensions)
    if isinstance(value, dict):
        for key in spec.get("required", []):
            if key not in value:
                raise invalid(f"falta campo requerido: {key}", path)
        props = spec.get("properties", {})
        for key, child in value.items():
            if key in props:
                structure(child, props[key], root, f"{path}.{key}", extensions=extensions)
            elif spec.get("additionalProperties") is False and not extensions:
                raise invalid(f"campo desconocido: {key}", path)
        for branch in spec.get("allOf", []):
            condition = branch.get("if", {}).get("properties", {})
            if all(value.get(k) == s.get("const") for k, s in condition.items()):
                structure(value, branch.get("then", {}), root, path, extensions=extensions)
