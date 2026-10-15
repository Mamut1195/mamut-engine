#!/usr/bin/env bash
# Solo requiere Bash, Git y coreutils. Incluye cambios locales de archivos conocidos por Git.
set -euo pipefail
export LC_ALL=C
# J4 r31: ancla confiable de la base; no aceptar un manifiesto regenerado por el PR.
EXPECTED_WORKFLOWS_MANIFEST_SHA256=214b71529137733aa14babaefa3eac67365f94bd986407ea7e7218bf68174c68

if [[ $# -lt 1 || $# -gt 2 ]]; then
    printf 'Uso: bash clean-room/check.sh <base-del-PR> [directorio-de-listas]\n' >&2
    exit 2
fi

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"
base=$(git merge-base "$1" HEAD)
# Las listas pueden venir de un checkout confiable (rama base) en CI.
rules_dir=$(cd "${2:-$repo_root/clean-room}" && pwd)

paths=()
identifiers=()
for list in forbidden_paths forbidden_identifiers; do
    while IFS= read -r entry || [[ -n "$entry" ]]; do
        [[ -z "$entry" || "$entry" == \#* ]] && continue
        if [[ "$list" == forbidden_paths ]]; then
            # Un patrón anclado a la raíz ("/x/*") se normaliza a relativo.
            entry=${entry#/}
            # Un glob que abarca todo el repo prohibiría cualquier ruta: error de configuración.
            if [[ -z "$entry" || "$entry" =~ ^\*+$ ]]; then
                printf 'Patrón de ruta demasiado amplio o vacío en %s\n' "$list" >&2
                exit 2
            fi
            paths+=("$entry")
        else
            # La lista inicial contiene solo identificadores ASCII.
            if [[ ! "$entry" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
                printf 'Identificador inválido en %s: %s\n' "$list" "$entry" >&2
                exit 2
            fi
            identifiers+=("$entry")
        fi
    done < "$rules_dir/$list.txt"
done

# Listas vacías pasarían en verde sin comprobar nada: se tratan como error.
if [[ ${#paths[@]} == 0 || ${#identifiers[@]} == 0 ]]; then
    printf 'Listas prohibidas vacías (rutas: %d, identificadores: %d)\n' "${#paths[@]}" "${#identifiers[@]}" >&2
    exit 2
fi

# Verificar el diff antes de leerlo mediante sustitución de procesos.
# Sin detección de renombres: se comprueba todo el contenido del destino nuevo.
git diff --no-ext-diff --no-renames --name-only "$base" -- > /dev/null
touches_crates=false
has_provenance=false
failed=false
touches_vendor=false

# Git entrega rutas relativas canónicas; no ampliar la excepción a alias con ./ o ../.
is_workflows_path() {
    [[ "$1" == vendor/mamut-workflows/* ]] || return 1
    case "/$1/" in *//*|*/./*|*/../*) return 1 ;; esac
}

# Código confiable de la base en CI: nunca ejecutar el verificador del árbol del PR.
verify_workflows() (
    [[ ! -L vendor && ! -L vendor/mamut-workflows ]] || exit 1
    cd -- vendor/mamut-workflows || exit 1
    [[ -f MANIFEST.sha256 && -s MANIFEST.sha256 && ! -L MANIFEST.sha256 ]] || exit 1
    scratch=$(mktemp -d) || exit 1
    trap 'rm -rf -- "$scratch"' EXIT
    shopt -s globstar dotglob nullglob
    # Inventario físico, incluidos archivos ignorados por Git. Rechazar enlaces
    # (también directorios enlazados) evita verificar contenido fuera del vendor.
    for file in **/*; do
        [[ ! -L "$file" ]] || exit 1
        if [[ -f "$file" && "$file" != MANIFEST.sha256 ]]; then
            printf './%s\0' "$file"
        fi
    done > "$scratch/actual"
    while IFS= read -r entry || [[ -n "$entry" ]]; do
        [[ "$entry" =~ ^[0-9a-f]{64}\ \ \./.+$ ]] || exit 1
        file=${entry:66}
        # Solo nombres canónicos internos; sin autorreferencia ni duplicados.
        case "/${file#./}/" in *//*|*/./*|*/../*) exit 1 ;; esac
        [[ "$file" != ./MANIFEST.sha256 ]] || exit 1
        printf '%s\0' "$file"
    done < MANIFEST.sha256 > "$scratch/expected"
    sort -z -o "$scratch/actual" "$scratch/actual" || exit 1
    sort -z -o "$scratch/expected" "$scratch/expected" || exit 1
    cmp -s "$scratch/expected" "$scratch/actual" || exit 1
    sha256sum --check --strict MANIFEST.sha256 || exit 1
    [[ -f LICENSE ]] || exit 1
    # Los tests CLI verbatim resuelven ../../examples bajo vendor/, fuera de esta
    # distribución. Comparar esas copias con los originales cubiertos por el ancla.
    [[ ! -L ../examples && ! -L ../examples/open-local ]] || exit 1
    for file in catalog.json workflow.json; do
        [[ -f "../examples/open-local/$file" && ! -L "../examples/open-local/$file" ]] || exit 1
        cmp -s "examples/open-local/$file" "../examples/open-local/$file" || exit 1
    done
    manifest_sha256=$(sha256sum MANIFEST.sha256) || exit 1
    [[ "${manifest_sha256%% *}" == "$EXPECTED_WORKFLOWS_MANIFEST_SHA256" ]] || exit 1
    for file in README.md NOTICE; do
        [[ -f "$file" ]] || continue
        while IFS= read -r line || [[ -n "$line" ]]; do
            if [[ "$line" == *1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d* ]]; then
                exit 0
            fi
        done < "$file"
    done
    exit 1
)

# -z preserva rutas con espacios, tabuladores, saltos de línea y caracteres Unicode.
while IFS= read -r -d '' path; do
    [[ "$path" == crates/* ]] && touches_crates=true
    if [[ "$path" == vendor/mamut-workflows || "$path" == vendor/examples || "$path" == vendor/examples/* ]] || is_workflows_path "$path"; then
        touches_vendor=true
    fi
done < <(git diff --no-ext-diff --no-renames --name-only -z "$base" --)

if [[ "$touches_vendor" == true ]] && ! verify_workflows; then
    printf 'Integridad vendor/mamut-workflows: manifiesto, inventario, hashes, commit o LICENSE inválidos.\n' >&2
    failed=true
fi

while IFS= read -r -d '' path; do
    if [[ "$path" == provenance/ENG-*.md && "$path" != provenance/TEMPLATE.md ]]; then
        has_provenance=true
    fi
done < <(git diff --no-ext-diff --no-renames --name-only -z --diff-filter=AM "$base" --)

while IFS= read -r -d '' path; do
    for pattern in "${paths[@]}"; do
        if [[ "$path" == $pattern ]]; then
            printf 'Ruta prohibida: %s (patrón: %s)\n' "$path" "$pattern" >&2
            failed=true
        fi
    done

    case "$path" in
        clean-room/*|docs/CLEAN_ROOM.md|provenance/*) continue ;;
    esac
    is_workflows_path "$path" && continue
    patch=$(git diff --no-ext-diff --no-textconv --no-renames --unified=0 "$base" -- ":(literal)$path")
    in_hunk=false
    while IFS= read -r line; do
        if [[ "$line" == @@* ]]; then
            in_hunk=true
            continue
        fi
        # Ignorar cabeceras; una línea añadida que empieza con ++ sí se revisa.
        [[ "$in_hunk" == true && "$line" == +* ]] || continue
        added=${line:1}
        for identifier in "${identifiers[@]}"; do
            if [[ "$added" =~ (^|[^a-zA-Z0-9_])${identifier}([^a-zA-Z0-9_]|$) ]]; then
                printf 'Identificador prohibido: %s en %s: %s\n' "$identifier" "$path" "$added" >&2
                failed=true
            fi
        done
    done <<< "$patch"
done < <(git diff --no-ext-diff --no-renames --name-only -z --diff-filter=AMT "$base" --)

if [[ "$touches_crates" == true && "$has_provenance" == false ]]; then
    printf 'Cambios en crates/ requieren un provenance/ENG-*.md añadido o modificado (TEMPLATE.md no cuenta).\n' >&2
    failed=true
fi

if [[ "$failed" == true ]]; then
    exit 1
fi
printf 'Clean-room: OK\n'
