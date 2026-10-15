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

# Cada fixture tiene su propio verificador de base, con el hash de su distribución
# inicial. Las mutaciones posteriores NO pueden actualizar esa ancla.
pin_vendor_checker() {
    local r="$tmp/$1" digest
    digest=$(sha256sum "$r/vendor/mamut-workflows/MANIFEST.sha256")
    sed "s/^EXPECTED_WORKFLOWS_MANIFEST_SHA256=.*/EXPECTED_WORKFLOWS_MANIFEST_SHA256=${digest%% *}/" \
        "$src/check.sh" > "$r/base-check.sh"
}
vendor_manifest() {
    (cd "$tmp/$1/vendor/mamut-workflows" &&
        find . -type f ! -path './MANIFEST.sha256' -print0 |
        sort -z | xargs -0 sha256sum > MANIFEST.sha256)
}
new_vendor() { # Distribución mínima válida; el identificador queda cubierto por SHA-256.
    new_repo "$1"
    local v="$tmp/$1/vendor/mamut-workflows" file
    mkdir -p "$v/examples/open-local" "$tmp/$1/vendor/examples/open-local"
    for file in catalog.json workflow.json; do
        echo fixture > "$v/examples/open-local/$file"
        cp "$v/examples/open-local/$file" "$tmp/$1/vendor/examples/open-local/$file"
    done
    echo license > "$v/LICENSE"
    echo 'upstream 1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d' > "$v/NOTICE"
    echo 'x = geomTransf(1)' > "$v/payload.rs"
    vendor_manifest "$1"
    git -C "$tmp/$1" add vendor
    pin_vendor_checker "$1"
}
vendor_expect() {
    local checker="$src/check.sh"
    [[ ! -f "$tmp/$1/base-check.sh" ]] || checker="$tmp/$1/base-check.sh"
    expect "$1" "$2" bash "$checker" HEAD "$tmp/$1/rules"
}

new_vendor vendor_ok; vendor_expect vendor_ok 0
new_vendor vendor_external_fixture
printf changed >> "$tmp/vendor_external_fixture/vendor/examples/open-local/catalog.json"
vendor_expect vendor_external_fixture 1
new_vendor vendor_external_only
# Una modificación solo en el fixture externo también activa verify_workflows.
git -C "$tmp/vendor_external_only" add vendor
git -C "$tmp/vendor_external_only" commit -qm vendor
printf changed >> "$tmp/vendor_external_only/vendor/examples/open-local/workflow.json"
vendor_expect vendor_external_only 1
new_vendor vendor_external_symlink
mv "$tmp/vendor_external_symlink/vendor/examples" "$tmp/vendor_external_symlink/copy-examples"
ln -s ../copy-examples "$tmp/vendor_external_symlink/vendor/examples"
vendor_expect vendor_external_symlink 1
# Ataque r31: payload cambiado y manifiesto coherente, pero distinto del ancla de base.
new_vendor vendor_rehashed
echo changed >> "$tmp/vendor_rehashed/vendor/mamut-workflows/payload.rs"
vendor_manifest vendor_rehashed
(cd "$tmp/vendor_rehashed/vendor/mamut-workflows" && sha256sum --check --strict MANIFEST.sha256 >/dev/null) || exit 1
vendor_expect vendor_rehashed 1
new_vendor vendor_edit; git -C "$tmp/vendor_edit" commit -qm vendor
echo changed >> "$tmp/vendor_edit/vendor/mamut-workflows/payload.rs"
vendor_expect vendor_edit 1
# Incluso extras no rastreados o ignorados deben aparecer en el inventario físico.
new_vendor vendor_extra; echo extra > "$tmp/vendor_extra/vendor/mamut-workflows/extra"
vendor_expect vendor_extra 1
new_vendor vendor_ignored; echo '*.hidden' > "$tmp/vendor_ignored/.gitignore"; echo extra > "$tmp/vendor_ignored/vendor/mamut-workflows/extra.hidden"
vendor_expect vendor_ignored 1
new_vendor vendor_missing; rm "$tmp/vendor_missing/vendor/mamut-workflows/payload.rs"
vendor_expect vendor_missing 1
new_vendor vendor_deleted; git -C "$tmp/vendor_deleted" commit -qm vendor
git -C "$tmp/vendor_deleted" rm -qr vendor
vendor_expect vendor_deleted 1
new_vendor vendor_no_manifest; rm "$tmp/vendor_no_manifest/vendor/mamut-workflows/MANIFEST.sha256"
vendor_expect vendor_no_manifest 1
new_vendor vendor_empty; : > "$tmp/vendor_empty/vendor/mamut-workflows/MANIFEST.sha256"
vendor_expect vendor_empty 1
new_vendor vendor_malformed; echo invalid >> "$tmp/vendor_malformed/vendor/mamut-workflows/MANIFEST.sha256"
vendor_expect vendor_malformed 1
new_vendor vendor_duplicate; head -n 1 "$tmp/vendor_duplicate/vendor/mamut-workflows/MANIFEST.sha256" >> "$tmp/vendor_duplicate/vendor/mamut-workflows/MANIFEST.sha256"
vendor_expect vendor_duplicate 1
new_vendor vendor_traversal
(cd "$tmp/vendor_traversal/vendor/mamut-workflows" && sha256sum ./../../README.md >> MANIFEST.sha256)
vendor_expect vendor_traversal 1
new_vendor vendor_symlink; ln -s payload.rs "$tmp/vendor_symlink/vendor/mamut-workflows/link"
vendor_expect vendor_symlink 1
new_vendor vendor_listed_symlink
(cd "$tmp/vendor_listed_symlink/vendor/mamut-workflows" && ln -s payload.rs link && sha256sum ./link >> MANIFEST.sha256)
vendor_expect vendor_listed_symlink 1
new_vendor vendor_dir_symlink; ln -s ../.. "$tmp/vendor_dir_symlink/vendor/mamut-workflows/escape"
vendor_expect vendor_dir_symlink 1
new_vendor vendor_root_symlink
mv "$tmp/vendor_root_symlink/vendor/mamut-workflows" "$tmp/vendor_root_symlink/copy"
ln -s ../copy "$tmp/vendor_root_symlink/vendor/mamut-workflows"
git -C "$tmp/vendor_root_symlink" add -A vendor
vendor_expect vendor_root_symlink 1
# Metadatos obligatorios aun si los hashes y el inventario coinciden.
new_vendor vendor_no_commit
(cd "$tmp/vendor_no_commit/vendor/mamut-workflows" && echo attribution > NOTICE)
vendor_manifest vendor_no_commit
pin_vendor_checker vendor_no_commit
vendor_expect vendor_no_commit 1
new_vendor vendor_old_commit
(cd "$tmp/vendor_old_commit/vendor/mamut-workflows" && echo 'upstream 4cb9ecd0e1117d86444149b17a6b34d1283d9455' > NOTICE)
vendor_manifest vendor_old_commit
pin_vendor_checker vendor_old_commit
vendor_expect vendor_old_commit 1
new_vendor vendor_short_commit
(cd "$tmp/vendor_short_commit/vendor/mamut-workflows" && echo 'upstream 1eeb91a' > NOTICE)
vendor_manifest vendor_short_commit
pin_vendor_checker vendor_short_commit
vendor_expect vendor_short_commit 1
new_vendor vendor_readme
(cd "$tmp/vendor_readme/vendor/mamut-workflows" && mv NOTICE README.md)
vendor_manifest vendor_readme
pin_vendor_checker vendor_readme
vendor_expect vendor_readme 0
new_vendor vendor_no_license
(cd "$tmp/vendor_no_license/vendor/mamut-workflows" && rm LICENSE)
vendor_manifest vendor_no_license
pin_vendor_checker vendor_no_license
vendor_expect vendor_no_license 1
# Nunca ejecutar el script del PR, aunque prometa validar la distribución.
new_vendor vendor_untrusted_script
mkdir "$tmp/vendor_untrusted_script/scripts"
echo 'exit 0' > "$tmp/vendor_untrusted_script/scripts/check-vendor.sh"
echo changed >> "$tmp/vendor_untrusted_script/vendor/mamut-workflows/payload.rs"
vendor_expect vendor_untrusted_script 1
# La excepción solo cubre el prefijo exacto; las rutas prohibidas se revisan antes.
for name in otro mamut-workflows-x; do
    new_repo "vendor_$name"; mkdir -p "$tmp/vendor_$name/vendor/$name"
    echo 'geomTransf(1)' > "$tmp/vendor_$name/vendor/$name/a.rs"
    git -C "$tmp/vendor_$name" add vendor
    vendor_expect "vendor_$name" 1
done
new_vendor vendor_forbidden_path
printf 'vendor/mamut-workflows/payload.rs\n' > "$tmp/vendor_forbidden_path/rules/forbidden_paths.txt"
vendor_expect vendor_forbidden_path 1
printf '%d pruebas, %d fallos\n' "$n" "$fails"
[[ $fails == 0 ]]
