#!/usr/bin/env bash
# Pruebas del validador. Uso: bash clean-room/tests/run.sh [ruta-a-clean-room] (por defecto, este checkout).
set -uo pipefail
export LC_ALL=C
src=$(cd "${1:-$(dirname "$0")/..}" && pwd)
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
fails=0; n=0

new_repo() { # $1 = nombre; deja listas válidas en $tmp/$1/rules
    local r="$tmp/$1"; mkdir -p "$r/rules"; git -C "$r" init -q
    git -C "$r" config user.email t@t; git -C "$r" config user.name t
    printf 'OpenSees/OpenSees/*\n' > "$r/rules/forbidden_paths.txt"
    printf 'geomTransf\n' > "$r/rules/forbidden_identifiers.txt"
    echo base > "$r/README.md"; git -C "$r" add README.md; git -C "$r" commit -qm base
}
# expect NOMBRE CODIGO_ESPERADO -- comando en el repo $tmp/NOMBRE
expect() {
    local name=$1 want=$2; shift 2; n=$((n+1))
    (cd "$tmp/$name" && "$@" >/dev/null 2>&1); local got=$?
    if [[ $got != "$want" ]]; then printf 'FALLA %s: esperado %s, obtenido %s\n' "$name" "$want" "$got"; fails=$((fails+1)); else printf 'ok    %s\n' "$name"; fi
}
chk() { bash "$src/check.sh" HEAD "$tmp/$1/rules"; }

# Positivo: cambio inocuo
new_repo ok; echo x > "$tmp/ok/a.txt"; git -C "$tmp/ok" add a.txt
expect ok 0 bash "$src/check.sh" HEAD "$tmp/ok/rules"
# Negativo: ruta prohibida
new_repo path; mkdir -p "$tmp/path/OpenSees/OpenSees/src"; echo x > "$tmp/path/OpenSees/OpenSees/src/a.c"; git -C "$tmp/path" add OpenSees
expect path 1 bash "$src/check.sh" HEAD "$tmp/path/rules"
# Negativo: identificador prohibido; positivo: subcadena no coincide
new_repo id; echo 'x = geomTransf(1)' > "$tmp/id/a.rs"; git -C "$tmp/id" add a.rs
expect id 1 bash "$src/check.sh" HEAD "$tmp/id/rules"
new_repo idsub; echo 'x = geomTransfX(1)' > "$tmp/idsub/a.rs"; git -C "$tmp/idsub" add a.rs
expect idsub 0 bash "$src/check.sh" HEAD "$tmp/idsub/rules"
# Negativo: listas vacías (solo comentarios) -> error 2
new_repo emptyp; : > "$tmp/emptyp/rules/forbidden_paths.txt"; echo '# nada' > "$tmp/emptyp/rules/forbidden_paths.txt"
expect emptyp 2 bash "$src/check.sh" HEAD "$tmp/emptyp/rules"
new_repo emptyi; : > "$tmp/emptyi/rules/forbidden_identifiers.txt"
expect emptyi 2 bash "$src/check.sh" HEAD "$tmp/emptyi/rules"
# Glob raíz: "*" y "/*" son demasiado amplios -> error 2
new_repo glob1; printf '*\n' > "$tmp/glob1/rules/forbidden_paths.txt"
expect glob1 2 bash "$src/check.sh" HEAD "$tmp/glob1/rules"
new_repo glob2; printf '/*\n' > "$tmp/glob2/rules/forbidden_paths.txt"
expect glob2 2 bash "$src/check.sh" HEAD "$tmp/glob2/rules"
# Patrón anclado "/vendor/*": bloquea vendor/x pero no otro/vendor/x ni fuera
new_repo anc; printf '/vendor/*\n' > "$tmp/anc/rules/forbidden_paths.txt"; mkdir "$tmp/anc/vendor"; echo x > "$tmp/anc/vendor/a"; git -C "$tmp/anc" add vendor
expect anc 1 bash "$src/check.sh" HEAD "$tmp/anc/rules"
new_repo anc2; printf '/vendor/*\n' > "$tmp/anc2/rules/forbidden_paths.txt"; mkdir "$tmp/anc2/src"; echo x > "$tmp/anc2/src/a"; git -C "$tmp/anc2" add src
expect anc2 0 bash "$src/check.sh" HEAD "$tmp/anc2/rules"
# crates/ sin bitácora -> 1; con bitácora -> 0
new_repo cr; mkdir "$tmp/cr/crates"; echo x > "$tmp/cr/crates/a.rs"; git -C "$tmp/cr" add crates
expect cr 1 bash "$src/check.sh" HEAD "$tmp/cr/rules"
new_repo cr2; mkdir "$tmp/cr2/crates" "$tmp/cr2/provenance"; echo x > "$tmp/cr2/crates/a.rs"; echo p > "$tmp/cr2/provenance/ENG-1.md"; git -C "$tmp/cr2" add crates provenance
expect cr2 0 bash "$src/check.sh" HEAD "$tmp/cr2/rules"
# El PR no puede aflojar las reglas: las listas del PR (en clean-room/) se ignoran si se pasan las de base
new_repo trust; mkdir "$tmp/trust/clean-room"; : > "$tmp/trust/clean-room/forbidden_paths.txt"; : > "$tmp/trust/clean-room/forbidden_identifiers.txt"
mkdir -p "$tmp/trust/OpenSees/OpenSees"; echo x > "$tmp/trust/OpenSees/OpenSees/a"; git -C "$tmp/trust" add clean-room OpenSees
expect trust 1 bash "$src/check.sh" HEAD "$tmp/trust/rules"
printf '%d pruebas, %d fallos\n' "$n" "$fails"
[[ $fails == 0 ]]
