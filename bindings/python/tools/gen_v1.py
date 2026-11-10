"""Generador de dataclasses para los esquemas engine-input/1 y results/1.

Usa solo la biblioteca estándar. Lee los dos JSON del repositorio y escribe
el módulo ``mamut_engine.v1``. La salida depende solo del texto de los
esquemas: dos ejecuciones producen los mismos bytes.

Una definición contada es una clave de ``$defs`` o el documento raíz. Las
ramas anónimas de un ``oneOf`` y los objetos o enums escritos dentro de una
propiedad se generan para que esa definición quede tipada, y no suman en el
denominador.

No se copia el texto descriptivo de los esquemas. Ese texto nombra magnitudes
y sistemas, y las unidades tipadas pertenecen a MAM-1046.
"""

from __future__ import annotations

import json
import keyword
import math
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TypeAlias, cast

JsonScalar: TypeAlias = None | bool | int | float | str
JsonValue: TypeAlias = JsonScalar | list["JsonValue"] | dict[str, "JsonValue"]

_CAMEL_BOUNDARY = re.compile(r"([a-z0-9])([A-Z])")
_NON_IDENTIFIER = re.compile(r"[^0-9A-Za-z]+")

_NODE_KEYWORDS = frozenset(
    {
        "$comment",
        "$ref",
        "additionalProperties",
        "allOf",
        "const",
        "default",
        "deprecated",
        "description",
        "enum",
        "examples",
        "exclusiveMaximum",
        "exclusiveMinimum",
        "items",
        "maxItems",
        "maxLength",
        "maximum",
        "minItems",
        "minLength",
        "minimum",
        "oneOf",
        "pattern",
        "properties",
        "required",
        "title",
        "type",
    }
)
_ROOT_KEYWORDS = _NODE_KEYWORDS | {"$defs", "$id", "$schema"}
_PRIMITIVES = frozenset({"string", "integer", "number", "boolean", "null"})
_PRIMITIVE_TYPES = {
    "string": "str",
    "integer": "int",
    "number": "float",
    "boolean": "bool",
    "null": "None",
}

_ENGINE_INPUT = "engine-input/1"
_RESULTS = "results/1"


class SchemaShapeError(ValueError):
    """El esquema tiene una forma que este generador no sabe expresar."""


@dataclass(frozen=True, slots=True)
class RefNode:
    """Referencia local ``#/$defs/nombre``."""

    target: str


@dataclass(frozen=True, slots=True)
class EnumNode:
    """Enumeración cerrada de cadenas."""

    values: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class ConstNode:
    """Constante de cadena del esquema."""

    value: str


@dataclass(frozen=True, slots=True)
class PrimitiveNode:
    """Uno o más tipos primitivos del esquema."""

    names: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class ArrayNode:
    """Arreglo homogéneo."""

    item: SchemaNode


@dataclass(frozen=True, slots=True)
class PropertySpec:
    """Una propiedad de un objeto, con su valor por defecto si el esquema lo escribe."""

    name: str
    schema: SchemaNode
    default: JsonScalar | None
    has_default: bool


@dataclass(frozen=True, slots=True)
class ObjectNode:
    """Objeto de propiedades cerradas."""

    properties: tuple[PropertySpec, ...]
    required: frozenset[str]


@dataclass(frozen=True, slots=True)
class OneOfNode:
    """Unión de ramas, en el orden del esquema."""

    branches: tuple[SchemaNode, ...]


SchemaNode: TypeAlias = (
    RefNode | EnumNode | ConstNode | PrimitiveNode | ArrayNode | ObjectNode | OneOfNode
)


@dataclass(frozen=True, slots=True)
class SchemaDocument:
    """Un esquema ya leído, con el raíz y las definiciones en orden de aparición."""

    name: str
    root: ObjectNode
    definitions: tuple[tuple[str, SchemaNode], ...]


@dataclass(frozen=True, slots=True)
class SchemaCoverage:
    """Símbolos con los que se cubrió cada definición contada."""

    schema_name: str
    definition_symbols: tuple[tuple[str, str], ...]
    root_symbol: str


@dataclass(frozen=True, slots=True)
class BuiltModule:
    """Texto del módulo y la cobertura que lo produjo."""

    text: str
    coverage: tuple[SchemaCoverage, SchemaCoverage]


def repository_root() -> Path:
    """Raíz del repositorio, tres niveles por encima de este archivo."""

    return Path(__file__).resolve().parents[3]


def output_path() -> Path:
    """Ruta del módulo generado, relativa a la raíz del repositorio."""

    return (
        repository_root()
        / "bindings"
        / "python"
        / "python"
        / "mamut_engine"
        / "v1.py"
    )


def render() -> str:
    """Devuelve el texto completo del módulo, con un salto de línea final."""

    return build().text


def coverage() -> tuple[SchemaCoverage, SchemaCoverage]:
    """Devuelve la cobertura de engine-input/1 y, después, la de results/1."""

    return build().coverage


def build() -> BuiltModule:
    """Lee los dos esquemas y construye el módulo y su cobertura."""

    engine_input = _load_schema(_ENGINE_INPUT, ("engine-input", "1.json"))
    results = _load_schema(_RESULTS, ("results", "1.json"))
    input_module = _PythonModule(engine_input)
    result_module = _PythonModule(results)
    shared = set(input_module.public_names) & set(result_module.public_names)
    if shared:
        names = ", ".join(sorted(shared))
        raise SchemaShapeError(f"nombres Python repetidos entre esquemas: {names}")
    public_names = [*input_module.public_names, *result_module.public_names]
    pieces = [
        _section(_ENGINE_INPUT),
        *input_module.blocks,
        _section(_RESULTS),
        *result_module.blocks,
        _all_block(public_names),
    ]
    text = _MODULE_DOCSTRING + "\n\n" + _IMPORT_BLOCK.rstrip("\n") + "\n\n\n" + "\n\n\n".join(pieces)
    return BuiltModule(
        text=_finalize(text),
        coverage=(input_module.coverage, result_module.coverage),
    )


def main(argv: list[str] | None = None) -> int:
    """Escribe el módulo generado. No acepta argumentos."""

    arguments = list(sys.argv[1:] if argv is None else argv)
    if arguments:
        print("Uso: gen_v1.py", file=sys.stderr)
        return 2
    output_path().write_text(render(), encoding="utf-8", newline="\n")
    return 0


class _PythonModule:
    """Acumula las clases, enums y alias de un esquema, en orden estable."""

    def __init__(self, document: SchemaDocument) -> None:
        self._document = document
        self._definitions = dict(document.definitions)
        self.blocks: list[str] = []
        self.public_names: list[str] = []
        self._claimed: set[str] = set()
        definition_symbols: list[tuple[str, str]] = []
        for key, _node in document.definitions:
            symbol = _definition_symbol(document.name, key)
            self._claim(symbol)
            definition_symbols.append((key, symbol))
            self.blocks.append(self._render_definition(symbol, key))
        root_symbol = _root_symbol(document.name)
        self._claim(root_symbol)
        self.blocks.append(
            self._object_block(
                root_symbol,
                document.root,
                f"Documento raíz del esquema {document.name}.",
            )
        )
        self.coverage = SchemaCoverage(
            schema_name=document.name,
            definition_symbols=tuple(definition_symbols),
            root_symbol=root_symbol,
        )

    def _claim(self, name: str) -> None:
        if name in self._claimed:
            raise SchemaShapeError(
                f"{self._document.name}: nombre Python repetido: {name}"
            )
        self._claimed.add(name)
        self.public_names.append(name)

    def _render_definition(self, symbol: str, key: str) -> str:
        node = self._definitions[key]
        match node:
            case ObjectNode() as node:
                return self._object_block(
                    symbol,
                    node,
                    f"Objeto ``{key}`` del esquema {self._document.name}.",
                )
            case EnumNode() as node:
                return _enum_block(
                    symbol,
                    node.values,
                    (
                        "Conjunto cerrado de cadenas. "
                        f"Definición ``{key}`` del esquema {self._document.name}."
                    ),
                )
            case OneOfNode() as node:
                return self._union_block(symbol, node, key)
            case PrimitiveNode() as node:
                return self._alias_definition(symbol, key, node)
            case ArrayNode() as node:
                return self._alias_definition(symbol, key, node)
            case ConstNode() as node:
                return self._alias_definition(symbol, key, node)
            case RefNode() as node:
                return self._alias_definition(symbol, key, node)

    def _alias_definition(self, symbol: str, key: str, node: SchemaNode) -> str:
        expression = self._type_expression(node, symbol, None, item=False)
        return _alias_block(
            symbol,
            expression,
            f"Alias de la definición ``{key}`` del esquema {self._document.name}.",
        )

    def _union_block(self, symbol: str, node: OneOfNode, key: str) -> str:
        branch_names: list[str] = []
        branch_blocks: list[str] = []
        for index, branch in enumerate(node.branches, start=1):
            if isinstance(branch, ObjectNode):
                branch_symbol = _branch_symbol(symbol, branch, index)
                self._claim(branch_symbol)
                branch_blocks.append(
                    self._object_block(
                        branch_symbol,
                        branch,
                        _branch_docstring(symbol, branch),
                    )
                )
                branch_names.append(branch_symbol)
                continue
            branch_names.append(self._type_expression(branch, symbol, None, item=False))
        expression = _format_union(branch_names)
        if key == "analysis" and self._document.name == _ENGINE_INPUT:
            comment = (
                "Unión discriminada por method. "
                f"Definición ``{key}`` del esquema {self._document.name}."
            )
        else:
            comment = (
                f"Unión de la definición ``{key}`` del esquema {self._document.name}."
            )
        return "\n\n\n".join([*branch_blocks, _alias_block(symbol, expression, comment)])

    def _object_block(self, symbol: str, node: ObjectNode, summary: str) -> str:
        fields = [_ordered_property(node, name) for name in _property_order(node)]
        field_lines = [self._field_line(symbol, node.required, prop) for prop in fields]
        lines = [
            "@dataclass(frozen=True, slots=True, kw_only=True)",
            f"class {symbol}:",
            f"    {_docstring(summary)}",
        ]
        if field_lines:
            lines.append("")
            lines.extend(field_lines)
        else:
            lines.append("")
            lines.append("    pass")
        return "\n".join(lines)

    def _field_line(self, owner: str, required: frozenset[str], prop: PropertySpec) -> str:
        annotation = self._type_expression(prop.schema, owner, prop.name, item=False)
        default_literal: str | None = None
        if prop.name not in required:
            if prop.has_default:
                default_literal = _python_literal(prop.default)
            else:
                if not _annotation_allows_none(annotation):
                    annotation = _format_union([annotation, "None"])
                default_literal = "None"
        return _format_field(prop.name, annotation, default_literal)

    def _type_expression(
        self,
        node: SchemaNode,
        owner: str,
        property_name: str | None,
        *,
        item: bool,
    ) -> str:
        match node:
            case RefNode() as ref:
                if ref.target not in self._definitions:
                    raise SchemaShapeError(
                        f"{self._document.name}: referencia desconocida: {ref.target}"
                    )
                return _definition_symbol(self._document.name, ref.target)
            case EnumNode() as enum_node:
                if property_name is None:
                    raise SchemaShapeError(
                        f"{self._document.name}: enum sin propiedad en {owner}"
                    )
                stem = _item_stem(property_name) if item else property_name
                symbol = owner + _pascal_case(stem)
                self._claim(symbol)
                if item:
                    summary = (
                        "Conjunto cerrado de cadenas de cada elemento de la "
                        f"propiedad ``{property_name}`` de ``{owner}``."
                    )
                else:
                    summary = (
                        "Conjunto cerrado de cadenas de la propiedad "
                        f"``{property_name}`` de ``{owner}``."
                    )
                self.blocks.append(_enum_block(symbol, enum_node.values, summary))
                return symbol
            case ConstNode(value=value):
                return f"Literal[{_python_string(value)}]"
            case PrimitiveNode(names=names):
                return _format_union([_PRIMITIVE_TYPES[name] for name in names])
            case ArrayNode(item=item_node):
                inner = self._type_expression(item_node, owner, property_name, item=True)
                return f"list[{inner}]"
            case ObjectNode() as object_node:
                if property_name is None:
                    raise SchemaShapeError(
                        f"{self._document.name}: objeto anónimo sin propiedad en {owner}"
                    )
                stem = _item_stem(property_name) if item else property_name
                symbol = owner + _pascal_case(stem)
                self._claim(symbol)
                if item:
                    summary = (
                        f"Objeto de cada elemento de la propiedad ``{property_name}`` "
                        f"en ``{owner}``."
                    )
                else:
                    summary = f"Objeto de la propiedad ``{property_name}`` en ``{owner}``."
                self.blocks.append(self._object_block(symbol, object_node, summary))
                return symbol
            case OneOfNode(branches=branches):
                parts: list[str] = []
                for index, branch in enumerate(branches, start=1):
                    if isinstance(branch, ObjectNode):
                        symbol = _branch_symbol(
                            owner + _pascal_case(property_name or "Union"),
                            branch,
                            index,
                        )
                        self._claim(symbol)
                        self.blocks.append(
                            self._object_block(
                                symbol,
                                branch,
                                _branch_docstring(symbol, branch),
                            )
                        )
                        parts.append(symbol)
                    else:
                        parts.append(
                            self._type_expression(branch, owner, property_name, item=item)
                        )
                return _format_union(parts)


def _load_schema(name: str, relative_parts: tuple[str, ...]) -> SchemaDocument:
    path = repository_root().joinpath("schemas", *relative_parts)
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as error:
        relative = "/".join(("schemas", *relative_parts))
        raise SchemaShapeError(f"no se pudo leer {relative}") from error
    parsed: object = json.loads(text, object_pairs_hook=_reject_duplicate_keys)
    document = _expect_json(parsed, name)
    if not isinstance(document, dict):
        raise SchemaShapeError(f"{name}: el esquema no es un objeto JSON")
    return _parse_document(name, document)


def _reject_duplicate_keys(pairs: list[tuple[object, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if not isinstance(key, str):
            raise SchemaShapeError("clave JSON que no es texto")
        if key in result:
            raise SchemaShapeError(f"clave JSON duplicada: {key}")
        result[key] = value
    return result


def _expect_json(value: object, where: str) -> JsonValue:
    if value is None or isinstance(value, str):
        return value
    if isinstance(value, bool) or isinstance(value, int):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            raise SchemaShapeError(f"{where}: número no finito")
        return value
    if isinstance(value, list):
        items = cast(list[object], value)
        return [_expect_json(item, where) for item in items]
    if isinstance(value, dict):
        mapping = cast(dict[object, object], value)
        result: dict[str, JsonValue] = {}
        for key, item in mapping.items():
            if not isinstance(key, str):
                raise SchemaShapeError(f"{where}: clave que no es texto")
            result[key] = _expect_json(item, f"{where}/{key}")
        return result
    raise SchemaShapeError(f"{where}: valor JSON no soportado")


def _parse_document(name: str, value: dict[str, JsonValue]) -> SchemaDocument:
    unknown = set(value) - _ROOT_KEYWORDS
    if unknown:
        listed = ", ".join(sorted(unknown))
        raise SchemaShapeError(f"{name}: palabras del esquema no reconocidas: {listed}")
    definitions_value = value.get("$defs", {})
    if not isinstance(definitions_value, dict):
        raise SchemaShapeError(f"{name}: $defs no es un objeto")
    definitions: list[tuple[str, SchemaNode]] = []
    for key, body in definitions_value.items():
        _require_identifier(key, f"{name}/$defs")
        if not isinstance(body, dict):
            raise SchemaShapeError(f"{name}: la definición {key} no es un objeto")
        definitions.append((key, _parse_schema(f"{name}/$defs/{key}", body)))
    root_body = {key: item for key, item in value.items() if key not in {"$defs", "$id", "$schema"}}
    root = _parse_schema(name, root_body)
    if not isinstance(root, ObjectNode):
        raise SchemaShapeError(f"{name}: el documento raíz no es un objeto")
    declared = _schema_constant(root)
    if declared != name:
        raise SchemaShapeError(
            f"{name}: la constante schema del documento es {declared!r}"
        )
    return SchemaDocument(name=name, root=root, definitions=tuple(definitions))


def _parse_schema(where: str, value: dict[str, JsonValue]) -> SchemaNode:
    unknown = set(value) - _NODE_KEYWORDS
    if unknown:
        listed = ", ".join(sorted(unknown))
        raise SchemaShapeError(f"{where}: palabras no reconocidas: {listed}")
    if "$ref" in value:
        extra = set(value) - {"$ref", "description", "title", "$comment"}
        if extra:
            raise SchemaShapeError(f"{where}: $ref mezcla otras palabras de tipo")
        return RefNode(_parse_ref(where, value["$ref"]))
    _reject_structural_allof(where, value.get("allOf"))
    if "enum" in value:
        return _parse_enum(where, value)
    if "const" in value:
        return ConstNode(_expect_string(value["const"], f"{where}/const"))
    if "oneOf" in value:
        if "type" in value or "properties" in value:
            raise SchemaShapeError(f"{where}: oneOf mezcla type o properties")
        return _parse_one_of(where, value["oneOf"])
    if "type" not in value:
        raise SchemaShapeError(f"{where}: falta type, enum, const, oneOf o $ref")
    names = _parse_type_names(where, value["type"])
    if names == ("object",):
        return _parse_object(where, value)
    if names == ("array",):
        return ArrayNode(_parse_items(where, value.get("items")))
    if "object" in names or "array" in names:
        raise SchemaShapeError(f"{where}: unión con object o array fuera de oneOf")
    return PrimitiveNode(names)


def _parse_ref(where: str, value: JsonValue) -> str:
    text = _expect_string(value, where)
    prefix = "#/$defs/"
    if not text.startswith(prefix) or not text.removeprefix(prefix):
        raise SchemaShapeError(f"{where}: referencia no local: {text}")
    target = text.removeprefix(prefix)
    if "/" in target:
        raise SchemaShapeError(f"{where}: referencia anidada: {text}")
    return target


def _parse_enum(where: str, value: dict[str, JsonValue]) -> EnumNode:
    if "type" in value:
        names = _parse_type_names(where, value["type"])
        if names != ("string",):
            raise SchemaShapeError(f"{where}: enum que no es de cadenas")
    raw = value["enum"]
    if not isinstance(raw, list) or not raw:
        raise SchemaShapeError(f"{where}: enum vacío")
    values = tuple(_expect_string(item, f"{where}/enum") for item in raw)
    if len(set(values)) != len(values):
        raise SchemaShapeError(f"{where}: enum con valores repetidos")
    return EnumNode(values)


def _parse_one_of(where: str, value: JsonValue) -> OneOfNode:
    if not isinstance(value, list) or not value:
        raise SchemaShapeError(f"{where}: oneOf vacío")
    branches: list[SchemaNode] = []
    for index, branch in enumerate(value, start=1):
        if not isinstance(branch, dict):
            raise SchemaShapeError(f"{where}/oneOf/{index}: la rama no es un objeto")
        branches.append(_parse_schema(f"{where}/oneOf/{index}", branch))
    return OneOfNode(tuple(branches))


def _parse_object(where: str, value: dict[str, JsonValue]) -> ObjectNode:
    if value.get("additionalProperties") is not False:
        raise SchemaShapeError(f"{where}: se esperaba additionalProperties false")
    raw_properties = value.get("properties", {})
    if not isinstance(raw_properties, dict):
        raise SchemaShapeError(f"{where}: properties no es un objeto")
    properties: list[PropertySpec] = []
    for name, body in raw_properties.items():
        _require_identifier(name, where)
        if not isinstance(body, dict):
            raise SchemaShapeError(f"{where}/{name}: la propiedad no es un objeto")
        default, has_default = _parse_default(f"{where}/{name}", body.get("default"), "default" in body)
        properties.append(
            PropertySpec(
                name=name,
                schema=_parse_schema(f"{where}/{name}", body),
                default=default,
                has_default=has_default,
            )
        )
    required = _parse_required(where, value.get("required", []), {prop.name for prop in properties})
    return ObjectNode(properties=tuple(properties), required=required)


def _parse_items(where: str, value: JsonValue | None) -> SchemaNode:
    if not isinstance(value, dict):
        raise SchemaShapeError(f"{where}: el arreglo no declara items")
    return _parse_schema(f"{where}/items", value)


def _parse_default(where: str, value: JsonValue | None, present: bool) -> tuple[JsonScalar | None, bool]:
    if not present:
        return None, False
    if isinstance(value, (list, dict)):
        raise SchemaShapeError(f"{where}: default estructurado sin representación")
    return value, True


def _parse_required(where: str, value: JsonValue, names: set[str]) -> frozenset[str]:
    if not isinstance(value, list):
        raise SchemaShapeError(f"{where}: required no es una lista")
    required: list[str] = []
    for item in value:
        name = _expect_string(item, f"{where}/required")
        if name not in names:
            raise SchemaShapeError(f"{where}: required nombra una propiedad ausente: {name}")
        if name in required:
            raise SchemaShapeError(f"{where}: required repetido: {name}")
        required.append(name)
    return frozenset(required)


def _parse_type_names(where: str, value: JsonValue) -> tuple[str, ...]:
    raw_names: tuple[str, ...]
    if isinstance(value, str):
        raw_names = (value,)
    elif isinstance(value, list) and value and all(isinstance(item, str) for item in value):
        raw_names = tuple(item for item in value if isinstance(item, str))
    else:
        raise SchemaShapeError(f"{where}: type no es una cadena ni una lista de cadenas")
    if len(set(raw_names)) != len(raw_names):
        raise SchemaShapeError(f"{where}: type repetido")
    unknown = [name for name in raw_names if name not in _PRIMITIVES and name not in {"object", "array"}]
    if unknown:
        raise SchemaShapeError(f"{where}: type desconocido: {', '.join(unknown)}")
    return raw_names


def _reject_structural_allof(where: str, value: JsonValue | None) -> None:
    if value is None:
        return
    if not isinstance(value, list) or not value:
        raise SchemaShapeError(f"{where}: allOf vacío")
    for index, branch in enumerate(value, start=1):
        if not isinstance(branch, dict):
            raise SchemaShapeError(f"{where}: allOf/{index} no es un objeto")
        extra = set(branch) - {"if", "then", "else"}
        if extra or "if" not in branch or "then" not in branch:
            raise SchemaShapeError(
                f"{where}: allOf que compone tipos; este generador no lo reescribe"
            )


def _schema_constant(root: ObjectNode) -> str:
    for prop in root.properties:
        if prop.name == "schema" and isinstance(prop.schema, ConstNode):
            return prop.schema.value
    raise SchemaShapeError("el documento no declara la constante schema")


def _definition_symbol(schema_name: str, key: str) -> str:
    if schema_name == _ENGINE_INPUT and key == "analysis":
        return "Analysis"
    return _schema_prefix(schema_name) + _pascal_case(key)


def _root_symbol(schema_name: str) -> str:
    if schema_name == _ENGINE_INPUT:
        return "EngineInputDocument"
    if schema_name == _RESULTS:
        return "ResultsDocument"
    raise SchemaShapeError(f"esquema no pedido: {schema_name}")


def _schema_prefix(schema_name: str) -> str:
    if schema_name == _ENGINE_INPUT:
        return "Input"
    if schema_name == _RESULTS:
        return "Result"
    raise SchemaShapeError(f"esquema no pedido: {schema_name}")


def _branch_symbol(parent: str, node: ObjectNode, index: int) -> str:
    method = _method_constant(node)
    if method is not None:
        return _pascal_case(method) + "Analysis"
    required = [prop.name for prop in node.properties if prop.name in node.required]
    if not required:
        return f"{parent}Branch{index}"
    return parent + "".join(_pascal_case(name) for name in required)


def _method_constant(node: ObjectNode) -> str | None:
    for prop in node.properties:
        if prop.name == "method" and isinstance(prop.schema, ConstNode):
            return prop.schema.value
    return None


def _branch_docstring(parent: str, node: ObjectNode) -> str:
    if _method_constant(node) is not None:
        return f"Rama de ``{parent}``. El campo method es la constante del esquema."
    return f"Rama de la unión ``{parent}``."


def _property_order(node: ObjectNode) -> tuple[str, ...]:
    required = [prop.name for prop in node.properties if prop.name in node.required]
    optional = [prop.name for prop in node.properties if prop.name not in node.required]
    return tuple(required + optional)


def _ordered_property(node: ObjectNode, name: str) -> PropertySpec:
    for prop in node.properties:
        if prop.name == name:
            return prop
    raise SchemaShapeError(f"propiedad ausente: {name}")


def _item_stem(property_name: str) -> str:
    """Nombre del elemento de un arreglo a partir del nombre de la propiedad.

    Si el nombre termina en una ese que no es doble, se quita esa ese:
    ``methods`` nombra a cada ``method``. Si no, se añade ``_item``.
    """

    plural = (
        property_name.endswith("s")
        and not property_name.endswith("ss")
        and len(property_name) > 1
    )
    if plural:
        return property_name[:-1]
    return property_name + "_item"


def _pascal_case(text: str) -> str:
    broken = _CAMEL_BOUNDARY.sub(r"\1_\2", text)
    pieces = [piece for piece in _NON_IDENTIFIER.split(broken) if piece]
    if not pieces:
        raise SchemaShapeError(f"nombre vacío a partir de {text!r}")
    return "".join(piece[0].upper() + piece[1:] for piece in pieces)


def _enum_member(value: str) -> str:
    broken = _CAMEL_BOUNDARY.sub(r"\1_\2", value)
    pieces = [piece for piece in _NON_IDENTIFIER.split(broken) if piece]
    if not pieces:
        raise SchemaShapeError(f"valor de enum vacío: {value!r}")
    name = "_".join(piece.upper() for piece in pieces)
    if name[0].isdigit():
        name = "VALUE_" + name
    if keyword.iskeyword(name):
        name = name + "_"
    return name


def _require_identifier(name: str, where: str) -> None:
    if not name.isidentifier() or keyword.iskeyword(name):
        raise SchemaShapeError(f"{where}: {name!r} no puede ser un nombre Python")


def _expect_string(value: JsonValue, where: str) -> str:
    if isinstance(value, str):
        return value
    raise SchemaShapeError(f"{where}: se esperaba una cadena")


def _python_string(value: str) -> str:
    escaped = (
        value.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
    )
    return f'"{escaped}"'


def _python_literal(value: JsonScalar | None) -> str:
    match value:
        case None:
            return "None"
        case bool() as flag:
            return "True" if flag else "False"
        case str() as text:
            return _python_string(text)
        case int() as number:
            return str(number)
        case float() as number:
            return repr(number)


def _format_union(parts: list[str]) -> str:
    if not parts:
        raise SchemaShapeError("unión vacía")
    if len(parts) == 1:
        return parts[0]
    single_line = " | ".join(parts)
    if len(parts) == 2 and "\n" not in single_line and len(single_line) <= 76:
        return single_line
    lines = ["(", f"    {parts[0]}"]
    lines.extend(f"    | {part}" for part in parts[1:])
    lines.append(")")
    return "\n".join(lines)


def _annotation_allows_none(annotation: str) -> bool:
    if annotation == "None":
        return True
    compact = annotation.replace("\n", " ")
    return any(part.strip() == "None" for part in compact.split("|"))


def _format_field(name: str, annotation: str, default_literal: str | None) -> str:
    suffix = "" if default_literal is None else f" = {default_literal}"
    if "\n" not in annotation:
        line = f"    {name}: {annotation}{suffix}"
        if len(line) <= 88:
            return line
    indented = "\n".join(
        "    " + line if line else ""
        for line in annotation.split("\n")
    )
    return f"    {name}: {indented}{suffix}"


def _docstring(text: str) -> str:
    if '"""' in text or "\n" in text:
        raise SchemaShapeError("docstring sin comillas triples ni saltos propios")
    words = text.split(" ")
    lines: list[str] = []
    current = ""
    for word in words:
        candidate = word if current == "" else f"{current} {word}"
        if len(candidate) <= 78 or current == "":
            current = candidate
            continue
        lines.append(current)
        current = word
    if current:
        lines.append(current)
    if len(lines) == 1:
        return f'"""{lines[0]}"""'
    body = "\n    ".join(lines)
    return f'"""{body}"""'


def _enum_block(symbol: str, values: tuple[str, ...], summary: str) -> str:
    lines = [
        "@final",
        f"class {symbol}(StrEnum):",
        f"    {_docstring(summary)}",
        "",
    ]
    seen: set[str] = set()
    for value in values:
        member = _enum_member(value)
        if member in seen:
            raise SchemaShapeError(f"{symbol}: miembro de enum repetido: {member}")
        seen.add(member)
        literal = _python_string(value)
        assignment = f"    {member} = {literal}"
        if len(assignment) <= 88:
            lines.append(assignment)
        else:
            lines.append(f"    {member} = (")
            lines.append(f"        {literal}")
            lines.append("    )")
    return "\n".join(lines)


def _alias_block(symbol: str, expression: str, comment: str) -> str:
    return f"# {comment}\n{symbol}: TypeAlias = {expression}"


def _all_block(names: list[str]) -> str:
    body = ",\n    ".join(f'"{name}"' for name in names)
    return f"__all__ = [\n    {body},\n]"


def _section(title: str) -> str:
    rule = "-" * len(title)
    return f"# {title}\n# {rule}"


def _finalize(text: str) -> str:
    lines = [line.rstrip() for line in text.split("\n")]
    while lines and lines[-1] == "":
        lines.pop()
    return "\n".join(lines) + "\n"


_MODULE_DOCSTRING = '''\
"""Documentos tipados de los esquemas engine-input/1 y results/1.

Este módulo lo escribe el generador de la biblioteca estándar. No se edita
a mano. Si un esquema cambia, hay que volver a generarlo: el test del
repositorio compara los bytes.

Las dataclasses son frozen, slots y kw_only. Cada enum de cadenas es un
StrEnum cerrado. La unión Analysis está discriminada por method.

Un campo opcional vale None cuando el documento omite la propiedad. Si el
esquema permite null, None también representa ese null. None no inventa una
magnitud ni un valor que el esquema no haya escrito.
"""'''

_IMPORT_BLOCK = '''\
from __future__ import annotations

import sys
from dataclasses import dataclass
from enum import Enum
from typing import Literal, TypeAlias, final

if sys.version_info >= (3, 11):
    from enum import StrEnum
else:

    class StrEnum(str, Enum):
        """Enumeración de cadenas cerrada para Python 3.10.

        La biblioteca estándar llama StrEnum a esta clase desde Python 3.11.
        El paquete declara Python 3.10, donde ese nombre todavía no existe.
        Esta clase no define una metaclase propia.
        """

        def __str__(self) -> str:
            value = self.value
            if isinstance(value, str):
                return value
            return str(value)
'''


if __name__ == "__main__":
    raise SystemExit(main())
