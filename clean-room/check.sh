#!/usr/bin/env bash
# Solo requiere Bash y Git. Incluye cambios locales de archivos conocidos por Git.
set -euo pipefail
export LC_ALL=C

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

# -z preserva rutas con espacios, tabuladores, saltos de línea y caracteres Unicode.
while IFS= read -r -d '' path; do
    [[ "$path" == crates/* ]] && touches_crates=true
done < <(git diff --no-ext-diff --no-renames --name-only -z "$base" --)

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
