#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C

fail() { printf 'Changelog: %s\n' "$*" >&2; exit 1; }
usage='uso: scripts/changelog-assemble.sh [--dry-run] VERSION [DATE (AAAA-MM-DD)]'
dry_run=0
args=()
for arg in "$@"; do
    if [[ $arg == --dry-run ]]; then
        dry_run=1
    else
        args+=("$arg")
    fi
done
(( ${#args[@]} >= 1 && ${#args[@]} <= 2 )) || fail "$usage"
version=${args[0]}
[[ $version =~ ^[A-Za-z0-9][A-Za-z0-9.+_-]*$ ]] || fail 'VERSION inválida (usa un identificador sin espacios).'
if (( ${#args[@]} == 2 )); then
    release_date=${args[1]}
else
    [[ ${SOURCE_DATE_EPOCH:-} =~ ^[0-9]+$ ]] || fail 'indica DATE o define SOURCE_DATE_EPOCH; no se usa el reloj del sistema.'
    release_date=$(date -u --date="@${SOURCE_DATE_EPOCH}" +%F) || fail 'SOURCE_DATE_EPOCH inválido.'
fi
[[ $release_date =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || fail 'DATE debe tener formato AAAA-MM-DD.'
normalized_date=$(date -u --date="$release_date" +%F) || fail 'DATE inválida.'
[[ $normalized_date == "$release_date" ]] || fail 'DATE inválida.'

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$root"
[[ -f CHANGELOG.md && ! -L CHANGELOG.md ]] || fail 'falta un CHANGELOG.md regular.'
[[ -d changelog.d && ! -L changelog.d ]] || fail 'falta el directorio changelog.d.'
shopt -s nullglob dotglob
fragments=()
for file in changelog.d/*; do
    name=${file##*/}
    [[ $name != README.md ]] || continue
    [[ $name =~ ^MAM-[0-9]+\.(added|changed|fixed|validation|security)\.md$ ]] || fail "nombre de fragmento inválido: $file"
    [[ -f $file && ! -L $file ]] || fail "el fragmento debe ser un archivo regular: $file"
    has_text=0
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ [^[:space:]] ]]; then has_text=1; break; fi
    done < "$file"
    (( has_text )) || fail "fragmento vacío: $file"
    fragments+=("$file")
done

# Locate Unreleased and its next peer heading without interpreting its Markdown.
unreleased=0
next_section=0
line_number=0
while IFS= read -r line || [[ -n $line ]]; do
    line_number=$((line_number + 1))
    if [[ $line == '## Unreleased' ]]; then
        (( unreleased == 0 )) || fail 'hay más de una sección ## Unreleased.'
        unreleased=$line_number
    elif (( unreleased > 0 && next_section == 0 )) && [[ $line == '## '* ]]; then
        next_section=$line_number
    fi
    [[ $line != "## $version - "* ]] || fail "la versión $version ya existe."
done < CHANGELOG.md
(( unreleased > 0 )) || fail 'falta el encabezado ## Unreleased.'
(( next_section > 0 )) || next_section=$((line_number + 1))

if (( ! dry_run )); then
    [[ $(git rev-parse --is-inside-work-tree 2>/dev/null) == true ]] || fail 'se requiere un árbol de trabajo Git.'
    git ls-files --error-unmatch -- CHANGELOG.md >/dev/null 2>&1 || fail 'CHANGELOG.md debe estar versionado en Git.'
    tree_status=$(git status --porcelain --untracked-files=all --ignore-submodules=none) || fail 'no se pudo comprobar el estado del árbol de trabajo Git.'
    [[ -z $tree_status ]] || fail 'el árbol de trabajo Git debe estar limpio (incluidos archivos sin seguimiento).'
fi

scratch=$(mktemp -d)
publish=''
cleanup() {
    rm -rf -- "$scratch"
    if [[ -n $publish ]]; then rm -f -- "$publish"; fi
}
trap cleanup EXIT
legacy=()
line_number=0
while IFS= read -r line || [[ -n $line ]]; do
    line_number=$((line_number + 1))
    if (( line_number > unreleased && line_number < next_section )); then
        legacy+=("$line")
    fi
done < CHANGELOG.md
# Trim only boundary whitespace; retain the legacy entries and their order.
start=0
end=${#legacy[@]}
while (( start < end )) && [[ ! ${legacy[start]} =~ [^[:space:]] ]]; do start=$((start + 1)); done
while (( end > start )) && [[ ! ${legacy[end-1]} =~ [^[:space:]] ]]; do end=$((end - 1)); done

render_fragment() {
    local file=$1 line first=1
    while IFS= read -r line || [[ -n $line ]]; do
        if (( first )); then
            [[ $line == '- '* ]] || line="- $line"
            first=0
        elif [[ -n $line && $line != ' '* && $line != $'\t'* ]]; then
            line="  $line"
        fi
        printf '%s\n' "$line"
    done < "$file"
    printf '\n'
}

{
    head -n "$unreleased" CHANGELOG.md
    printf '\n## %s - %s\n\n' "$version" "$release_date"
    types=(added changed fixed validation security)
    labels=(Añadido Cambiado Corregido Validación Seguridad)
    for index in "${!types[@]}"; do
        selected=()
        for file in "${fragments[@]}"; do
            if [[ $file == *."${types[index]}".md ]]; then selected+=("$file"); fi
        done
        (( ${#selected[@]} > 0 )) || continue
        printf '### %s\n\n' "${labels[index]}"
        # Names are validated, so newline-separated sorting is unambiguous.
        # After the hyphen comes the numeric ID; break numeric ties lexically.
        while IFS= read -r file; do render_fragment "$file"; done < <(
            printf '%s\n' "${selected[@]}" | sort -t '-' -k2,2n -k1
        )
    done
    if (( end > start )); then
        printf '### Otros (legado)\n\n'
        for ((index=start; index<end; index++)); do printf '%s\n' "${legacy[index]}"; done
        printf '\n'
    fi
    tail -n "+$next_section" CHANGELOG.md
} > "$scratch/CHANGELOG.md"

if (( dry_run )); then
    cat -- "$scratch/CHANGELOG.md"
else
    publish=$(mktemp "$root/.changelog-assemble.XXXXXX")
    chmod --reference=CHANGELOG.md "$publish"
    cat -- "$scratch/CHANGELOG.md" > "$publish"
    mv -- "$publish" CHANGELOG.md
    publish=''
    if (( ${#fragments[@]} > 0 )); then rm -- "${fragments[@]}"; fi
    printf 'Changelog: versión %s ensamblada; %s fragmentos eliminados.\n' "$version" "${#fragments[@]}" >&2
fi
