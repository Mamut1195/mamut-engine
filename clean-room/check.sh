#!/usr/bin/env bash
# Solo requiere Bash, Git y coreutils. Incluye cambios locales de archivos conocidos por Git.
set -euo pipefail
export LC_ALL=C
# J4 r31: ancla confiable de la base; no aceptar un manifiesto regenerado por el PR.
EXPECTED_WORKFLOWS_MANIFEST_SHA256=214b71529137733aa14babaefa3eac67365f94bd986407ea7e7218bf68174c68

# --- Salidas de oráculos (regla de CLEAN_ROOM.md §5.1; cierra #173 N2) -----------------
# Ver clean-room/README.md. Los mensajes indican ruta y línea, nunca el contenido.
oracle_failed=false

# Rutas de salidas o entradas de oráculos. crates/**/fixtures/results/ (Engine) se permite.
is_oracle_output_path() {
    local p=${1,,} base
    base=${p##*/}
    [[ "$p" == *.tcl || "$p" == *.tcl.* || "$p" == *.3dd ]] && return 0
    [[ "$base" == opensees*.csv || "$base" == opensees*.csv.* ]] && return 0
    [[ "$base" == opensees*.txt || "$base" == opensees*.json || "$base" == opensees*.out ]] && return 0
    [[ "$base" == frame3dd*.out || "$base" == frame3dd*.plt || "$base" == f3dd*.out || "$base" == f3dd*.plt ]] && return 0
    [[ "$p" == *frame3dd*/*.out || "$p" == *frame3dd*/*.plt || "$p" == *f3dd*/*.out || "$p" == *f3dd*/*.plt ]] && return 0
    [[ "$p" == results/* ]] && return 0
    [[ "$p" =~ (^|/)[^/]*(opensees|frame3dd|f3dd|stabileo|oracle|oraculo)[^/]*/(.*/)?results/ ]] && return 0
    [[ "$p" =~ (^|/)results/(.*/)?[^/]*(opensees|frame3dd|f3dd|stabileo)[^/]*$ ]] && return 0
    return 1
}

# Nombres del oráculo en minúsculas; "OS" solo en mayúsculas y como palabra.
oracle_name_re='(opensees|open[ _-]sees|frame3dd|frame[ _-]3dd|f3dd|stabileo)'
names_oracle() {
    local low=${1,,}
    [[ "$low" =~ $oracle_name_re ]] && return 0
    [[ "$1" =~ (^|[^A-Za-z0-9_])OS([^A-Za-z0-9_]|$) ]] && return 0
    return 1
}

# Quita metadatos permitidos (sustituye por espacio): hashes (token hex de 32+ caracteres,
# o de 7+ con alguna letra a-f, delimitado), versiones x.y.z, fechas y horas.
strip_metadata() {
    local s=${1,,} pre
    local -a pats=(
        '(^|[^0-9a-z_.+-])([0-9a-f]{32,})([^0-9a-z_.+-]|$)'
        '(^|[^0-9a-z_.+-])([0-9]*[a-f][0-9a-f]*[0-9][0-9a-f]*|[0-9][0-9a-f]*[a-f][0-9a-f]*)([^0-9a-z_.+-]|$)'
        '(^|[^0-9])([0-9]+(\.[0-9]+){2,})'
        '(^|[^0-9])([0-9]{4}-[0-9]{2}-[0-9]{2})'
        '(^|[^0-9])([0-9]{1,2}:[0-9]{2}(:[0-9]{2})?)'
        '(^|[^0-9])(§[[:space:]]*[0-9]+(\.[0-9]+)*)'
    )
    local pat i
    for i in "${!pats[@]}"; do
        pat=${pats[$i]}
        while [[ "$s" =~ $pat ]]; do
            # El token hex con letra debe tener 7+ caracteres; si no, no es hash.
            if [[ $i == 1 && ${#BASH_REMATCH[2]} -lt 7 ]]; then break; fi
            pre=${s%%"${BASH_REMATCH[0]}"*}
            s="${pre}${BASH_REMATCH[1]} ${BASH_REMATCH[3]:-}${s:$(( ${#pre} + ${#BASH_REMATCH[0]} ))}"
        done
    done
    printf '%s' "$s"
}

# Cifra: decimal (punto o coma) o exponente; puede cerrar la oración con . , ; :
float_re='(^|[^0-9a-z_.])[-+]?[0-9]+([.,][0-9]+(e[-+]?[0-9]+)?|e[-+]?[0-9]+)([.,;:]?([[:space:]]|$)|[^0-9a-z_.,;:])'
# Asignación "u = X", "E = 200e9", "P = 1000" (entrada o salida del modelo).
assign_re='(^|[[:space:](,;|])[a-z][a-z0-9_]{0,7}[[:space:]]*(:=|=|<=|>=|≤|≥|≈|~|:|<|>)[[:space:]]*[-+]?[0-9]'
output_word_re='(^|[^a-z0-9])(obtenid|salida|output|resultad|result|entrada|input|devuelv|devolvi|da como)'
# Exención de tolerancias: palabra completa en la misma línea o celda (nunca en encabezados).
tol_word_re='(^|[^a-z0-9_])(tol|rtol|atol|toleranc[a-z]*|umbral|criterio)([^a-z0-9_]|$)'
tol_col_re='^[[:space:]]*(tol|rtol|atol|toleranc[a-z]*|criterio)([^a-z0-9_]|$)'
num_tok='[0-9]*[.,]?[0-9]+(e[-+]?[0-9]+)?'

# ¿X es una tolerancia admisible? (X <= 1e-2, o X <= 5 si es porcentaje)
tol_value_ok() {
    local x=${1//,/.} pct=$2 lim=1e-2
    [[ "$pct" == true ]] && lim=5
    [[ "$x" == .* ]] && x="0$x"
    [[ "$(printf '%s\n%s\n' "$x" "$lim" | sort -g | head -n 1)" == "$x" ]]
}
# Quita de la línea solo los tokens con forma de tolerancia admisible.
strip_tolerances() {
    local s=$1 pre tok val pct pat i
    local -a pats=(
        "(^|[^a-z0-9_])((rtol|atol|tol)[[:space:]]*(=|:|<=|≤)[[:space:]]*($num_tok))"
    )
    # Solo con identificador tol/rtol/atol (R3 de J4 r3); "≤ X" o "X %" sueltos no eximen.
    for i in 0; do
        pat=${pats[$i]}
        local guard=0
        while [[ "$s" =~ $pat ]] && (( guard++ < 50 )); do
            tok=${BASH_REMATCH[2]}
            case $i in
                0) val=${BASH_REMATCH[5]}; pct=false ;;
                1) val=${BASH_REMATCH[4]}; pct=false ;;
                2) val=${BASH_REMATCH[7]:-${BASH_REMATCH[3]}}; pct=true ;;
            esac
            tol_value_ok "$val" "$pct" || break
            pre=${s%%"$tok"*}
            s="${pre} ${s:$(( ${#pre} + ${#tok} ))}"
        done
    done
    printf '%s' "$s"
}

# Celda de una columna de tolerancia: se quita solo un valor admisible (<= 1e-2 o <= 5 %)
# y el resto de la celda se sigue revisando.
strip_tol_cell() {
    local c=$1 t
    t=${c#"${c%%[![:space:]]*}"}; t=${t%"${t##*[![:space:]]}"}
    if [[ "$t" =~ ^($num_tok)[[:space:]]*(%)?$ ]]; then
        if tol_value_ok "${BASH_REMATCH[1]}" "$([[ -n "${BASH_REMATCH[3]}" ]] && echo true || echo false)"; then
            printf ' '; return
        fi
    elif [[ "$t" =~ ^($num_tok)[[:space:]]*(%)?[[:space:]]+(.*)$ ]]; then
        local rest=${BASH_REMATCH[4]}
        if tol_value_ok "${BASH_REMATCH[1]}" "$([[ -n "${BASH_REMATCH[3]}" ]] && echo true || echo false)"; then
            printf ' %s' "$rest"; return
        fi
    fi
    strip_tolerances "$c"
}

scan_oracle_output_file() {
    local path=$1 line low clean ctx n=0 lvl k in_code=false is_row prev_row=false
    local -a tcols=() cells=()
    local thead=''
    local -a heads=('' '' '' '' '' '' '')
    while IFS= read -r line || [[ -n "$line" ]]; do
        n=$((n+1))
        if [[ "$line" =~ ^[[:space:]]{0,3}(\`\`\`|~~~) ]]; then
            [[ "$in_code" == true ]] && in_code=false || in_code=true
            continue
        fi
        if [[ "$in_code" == false && "$line" =~ ^(#{1,6})[[:space:]] ]]; then
            lvl=${#BASH_REMATCH[1]}
            heads[$lvl]=$line
            for ((k = lvl + 1; k <= 6; k++)); do heads[$k]=''; done
            continue
        fi
        is_row=false
        [[ "$line" =~ ^[[:space:]]*\| ]] && is_row=true
        if [[ "$is_row" == true ]]; then
            if [[ "$prev_row" == false ]]; then
                IFS='|' read -r -a tcols <<< "${line,,}"   # encabezado de la tabla
                thead=$line   # su texto da contexto a las filas (R4)
                prev_row=true
            fi
        else
            prev_row=false; tcols=(); thead=''
        fi
        ctx="${heads[*]} $thead $line"
        names_oracle "$ctx" || continue
        clean=$(strip_metadata "$line")
        low=${clean,,}
        if [[ "$is_row" == true && ${#tcols[@]} -gt 0 ]]; then
            # Tablas: solo se eximen las columnas cuyo encabezado es tol/rtol/atol/criterio.
            IFS='|' read -r -a cells <<< "$low"
            for k in "${!cells[@]}"; do
                [[ "${tcols[$k]:-}" =~ $tol_col_re ]] && cells[$k]=$(strip_tol_cell "${cells[$k]}")
            done
            low=$(IFS='|'; printf '%s' "${cells[*]}")
        fi
        # Exención por palabra en la misma línea, solo si no hay palabra de salida.
        if [[ ! "$low" =~ $output_word_re && "$low" =~ $tol_word_re ]]; then
            low=$(strip_tolerances "$low")
        fi
        if [[ "$low" =~ $assign_re ]] \
            || [[ "$low" =~ $output_word_re && "$low" =~ $float_re ]] \
            || [[ "$is_row" == true && "$low" =~ $float_re ]]; then
            printf 'Posible salida o entrada de oráculo en %s:%d (contenido omitido)\n' "$path" "$n" >&2
            oracle_failed=true
        fi
    done < "$path"
}

oracle_check_path() {
    local path=$1
    if is_oracle_output_path "$path"; then
        printf 'Ruta de salida de oráculo prohibida: %s\n' "$path" >&2
        oracle_failed=true
    fi
    case "$path" in
        provenance/*|docs/*)
            if [[ -f "$path" && ! -L "$path" ]]; then scan_oracle_output_file "$path"; fi ;;
    esac
}

# Modo --all: barrido de todo el árbol rastreado (solo la regla de oráculos).
if [[ "${1:-}" == --all ]]; then
    cd "$(git rev-parse --show-toplevel)"
    while IFS= read -r -d '' path; do
        oracle_check_path "$path"
    done < <(git ls-files -z)
    if [[ "$oracle_failed" == true ]]; then exit 1; fi
    printf 'Clean-room (--all, salidas de oráculos): OK\n'
    exit 0
fi

if [[ $# -lt 1 || $# -gt 2 ]]; then
    printf 'Uso: bash clean-room/check.sh <base-del-PR> [directorio-de-listas] | --all\n' >&2
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
touches_vendor_crates=false

# Únicas parejas ruta:identificador permitidas; sin prefijos ni globs.
vendor_identifier_allowlist=(
    'vendor/mamut-workflows/mamut-canon/src/lib.rs:algorithm'
    'vendor/mamut-workflows/mamut-workflow-cli/tests/fixtures/run.txt:algorithm'
    'vendor/mamut-workflows/mamut-workflow-execution/src/mcp.rs:algorithm'
    'vendor/mamut-workflows/mamut-workflow-execution/src/tests.rs:algorithm'
)

scan_identifier_line() {
    local path=$1 content=$2 identifier exception allowed
    for identifier in "${identifiers[@]}"; do
        allowed=false
        for exception in "${vendor_identifier_allowlist[@]}"; do
            [[ "$path:$identifier" == "$exception" ]] && allowed=true
        done
        [[ "$allowed" == true ]] && continue
        if [[ "$content" =~ (^|[^a-zA-Z0-9_])${identifier}([^a-zA-Z0-9_]|$) ]]; then
            printf 'Identificador prohibido: %s en %s: %s\n' "$identifier" "$path" "$content" >&2
            failed=true
        fi
    done
}

# Git entrega rutas relativas canónicas; no aceptar alias con ./ o ../.
is_workflows_path() {
    [[ "$1" == vendor/mamut-workflows/* ]] || return 1
    case "/$1/" in *//*|*/./*|*/../*) return 1 ;; esac
}

# vendor-crates/ es la copia de cargo vendor (terceros, crates.io). No se escanea
# por identificadores: su integridad la comprueba cargo con .cargo-checksum.json
# contra Cargo.lock (--frozen en rust.yml) y su cambio exige revisión del dueño.
is_vendor_crates_path() {
    [[ "$1" == vendor-crates/* ]] || return 1
    case "/$1/" in *//*|*/./*|*/../*) return 1 ;; esac
}

# Sin enlaces simbólicos y con .cargo-checksum.json en cada crate.
verify_vendor_crates() (
    [[ -d vendor-crates && ! -L vendor-crates ]] || exit 1
    shopt -s dotglob nullglob globstar
    for file in vendor-crates/**/*; do
        [[ ! -L "$file" ]] || exit 1
    done
    for dir in vendor-crates/*/; do
        [[ -f "${dir}.cargo-checksum.json" ]] || exit 1
    done
    exit 0
)

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
    [[ "$path" == vendor-crates || "$path" == vendor-crates/* ]] && touches_vendor_crates=true
done < <(git diff --no-ext-diff --no-renames --name-only -z "$base" --)

if [[ "$touches_vendor_crates" == true ]] && ! verify_vendor_crates; then
    printf 'Integridad vendor-crates: enlace simbólico o falta .cargo-checksum.json.\n' >&2
    failed=true
fi

if [[ "$touches_vendor" == true ]]; then
    if verify_workflows; then
        # Escanear todo el inventario físico validado, incluidas líneas sin cambios
        # y archivos ignorados o no rastreados. La integridad precede a la lectura
        # para no seguir enlaces fuera del vendor.
        while IFS= read -r -d '' path; do
            while IFS= read -r line || [[ -n "$line" ]]; do
                scan_identifier_line "$path" "$line"
            done < "$path"
        done < <(
            shopt -s globstar dotglob nullglob
            for file in vendor/mamut-workflows/**/*; do
                [[ -f "$file" ]] && printf '%s\0' "$file"
            done
        )
    else
        printf 'Integridad vendor/mamut-workflows: manifiesto, inventario, hashes, commit o LICENSE inválidos.\n' >&2
        failed=true
    fi
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
    # El vendor ya se ha revisado íntegro arriba, sin limitarse al diff.
    is_workflows_path "$path" && continue
    is_vendor_crates_path "$path" && continue
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
        scan_identifier_line "$path" "$added"
    done <<< "$patch"
done < <(git diff --no-ext-diff --no-renames --name-only -z --diff-filter=AMT "$base" --)

while IFS= read -r -d '' path; do
    oracle_check_path "$path"
done < <(git diff --no-ext-diff --no-renames --name-only -z --diff-filter=AMT "$base" --)
[[ "$oracle_failed" == true ]] && failed=true

if [[ "$touches_crates" == true && "$has_provenance" == false ]]; then
    printf 'Cambios en crates/ requieren un provenance/ENG-*.md añadido o modificado (TEMPLATE.md no cuenta).\n' >&2
    failed=true
fi

if [[ "$failed" == true ]]; then
    exit 1
fi
printf 'Clean-room: OK\n'
