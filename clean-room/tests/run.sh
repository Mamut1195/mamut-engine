#!/usr/bin/env bash
# Validator tests. Usage: bash clean-room/tests/run.sh [ruta-a-clean-room] (defaults to this checkout).
set -uo pipefail
export LC_ALL=C
src=$(cd "${1:-$(dirname "$0")/..}" && pwd)
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
fails=0; n=0

new_repo() { # $1 = name; prepares valid lists in $tmp/$1/rules
    local r="$tmp/$1"; mkdir -p "$r/rules"; git -C "$r" init -q
    git -C "$r" config user.email t@t; git -C "$r" config user.name t
    printf 'OpenSees/OpenSees/*\n' > "$r/rules/forbidden_paths.txt"
    printf 'geomTransf\n' > "$r/rules/forbidden_identifiers.txt"
    echo base > "$r/README.md"; git -C "$r" add README.md; git -C "$r" commit -qm base
}
# expect NAME EXPECTED_CODE -- command in the repo $tmp/NOMBRE
expect() {
    local name=$1 want=$2; shift 2; n=$((n+1))
    (cd "$tmp/$name" && "$@" >"$tmp/$name/check.log" 2>&1); local got=$?
    if [[ $got != "$want" ]]; then printf 'FAIL %s: expected %s, got %s\n' "$name" "$want" "$got"; fails=$((fails+1)); else printf 'ok    %s\n' "$name"; fi
}
chk() { bash "$src/check.sh" HEAD "$tmp/$1/rules"; }

# Positive: harmless change
new_repo ok; echo x > "$tmp/ok/a.txt"; git -C "$tmp/ok" add a.txt
expect ok 0 bash "$src/check.sh" HEAD "$tmp/ok/rules"
# Negative: forbidden path
new_repo path; mkdir -p "$tmp/path/OpenSees/OpenSees/src"; echo x > "$tmp/path/OpenSees/OpenSees/src/a.c"; git -C "$tmp/path" add OpenSees
expect path 1 bash "$src/check.sh" HEAD "$tmp/path/rules"
# Negative: forbidden identifier; positive: substring does not match
new_repo id; echo 'x = geomTransf(1)' > "$tmp/id/a.rs"; git -C "$tmp/id" add a.rs
expect id 1 bash "$src/check.sh" HEAD "$tmp/id/rules"
new_repo idsub; echo 'x = geomTransfX(1)' > "$tmp/idsub/a.rs"; git -C "$tmp/idsub" add a.rs
expect idsub 0 bash "$src/check.sh" HEAD "$tmp/idsub/rules"
# Negative: empty lists (comments only) -> error 2
new_repo emptyp; : > "$tmp/emptyp/rules/forbidden_paths.txt"; echo '# nada' > "$tmp/emptyp/rules/forbidden_paths.txt"
expect emptyp 2 bash "$src/check.sh" HEAD "$tmp/emptyp/rules"
new_repo emptyi; : > "$tmp/emptyi/rules/forbidden_identifiers.txt"
expect emptyi 2 bash "$src/check.sh" HEAD "$tmp/emptyi/rules"
# Root glob: "*" and "/*" are too broad -> error 2
new_repo glob1; printf '*\n' > "$tmp/glob1/rules/forbidden_paths.txt"
expect glob1 2 bash "$src/check.sh" HEAD "$tmp/glob1/rules"
new_repo glob2; printf '/*\n' > "$tmp/glob2/rules/forbidden_paths.txt"
expect glob2 2 bash "$src/check.sh" HEAD "$tmp/glob2/rules"
# Anchored pattern "/vendor/*": blocks vendor/x but not otro/vendor/x or paths outside it
new_repo anc; printf '/vendor/*\n' > "$tmp/anc/rules/forbidden_paths.txt"; mkdir "$tmp/anc/vendor"; echo x > "$tmp/anc/vendor/a"; git -C "$tmp/anc" add vendor
expect anc 1 bash "$src/check.sh" HEAD "$tmp/anc/rules"
new_repo anc2; printf '/vendor/*\n' > "$tmp/anc2/rules/forbidden_paths.txt"; mkdir "$tmp/anc2/src"; echo x > "$tmp/anc2/src/a"; git -C "$tmp/anc2" add src
expect anc2 0 bash "$src/check.sh" HEAD "$tmp/anc2/rules"
# crates/ without a log entry -> 1; with a log entry -> 0
new_repo cr; mkdir "$tmp/cr/crates"; echo x > "$tmp/cr/crates/a.rs"; git -C "$tmp/cr" add crates
expect cr 1 bash "$src/check.sh" HEAD "$tmp/cr/rules"
new_repo cr2; mkdir "$tmp/cr2/crates" "$tmp/cr2/provenance"; echo x > "$tmp/cr2/crates/a.rs"; echo p > "$tmp/cr2/provenance/ENG-1.md"; git -C "$tmp/cr2" add crates provenance
expect cr2 0 bash "$src/check.sh" HEAD "$tmp/cr2/rules"
# The PR cannot relax the rules: PR lists (in clean-room/) are ignored when base lists are supplied
new_repo trust; mkdir "$tmp/trust/clean-room"; : > "$tmp/trust/clean-room/forbidden_paths.txt"; : > "$tmp/trust/clean-room/forbidden_identifiers.txt"
mkdir -p "$tmp/trust/OpenSees/OpenSees"; echo x > "$tmp/trust/OpenSees/OpenSees/a"; git -C "$tmp/trust" add clean-room OpenSees
expect trust 1 bash "$src/check.sh" HEAD "$tmp/trust/rules"

# Each sample has its own base checker, with the hash of its initial
# distribution. Later mutations CANNOT update that anchor.
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
new_vendor() { # Optional $2/$3: sample path and content before pinning the anchor.
    new_repo "$1"
    local v="$tmp/$1/vendor/mamut-workflows" file
    mkdir -p "$v/examples/open-local" "$tmp/$1/vendor/examples/open-local"
    for file in catalog.json workflow.json; do
        echo fixture > "$v/examples/open-local/$file"
        cp "$v/examples/open-local/$file" "$tmp/$1/vendor/examples/open-local/$file"
    done
    echo license > "$v/LICENSE"
    echo 'upstream 1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d' > "$v/NOTICE"
    printf 'algorithm\nfix\n' >> "$tmp/$1/rules/forbidden_identifiers.txt"
    echo 'safe payload' > "$v/payload.rs"
    mkdir -p "$v/mamut-canon/src"
    echo 'pub algorithm: String,' > "$v/mamut-canon/src/lib.rs"
    if [[ $# == 3 ]]; then
        mkdir -p "$(dirname "$v/$2")"
        printf '%s' "$3" >> "$v/$2"
    fi
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
# A change to only the external sample also triggers verify_workflows.
git -C "$tmp/vendor_external_only" add vendor
git -C "$tmp/vendor_external_only" commit -qm vendor
printf changed >> "$tmp/vendor_external_only/vendor/examples/open-local/workflow.json"
vendor_expect vendor_external_only 1
new_vendor vendor_external_symlink
mv "$tmp/vendor_external_symlink/vendor/examples" "$tmp/vendor_external_symlink/copy-examples"
ln -s ../copy-examples "$tmp/vendor_external_symlink/vendor/examples"
vendor_expect vendor_external_symlink 1
# Attack r31: changed payload and consistent manifest, but different from the base anchor.
new_vendor vendor_rehashed
echo changed >> "$tmp/vendor_rehashed/vendor/mamut-workflows/payload.rs"
vendor_manifest vendor_rehashed
(cd "$tmp/vendor_rehashed/vendor/mamut-workflows" && sha256sum --check --strict MANIFEST.sha256 >/dev/null) || exit 1
vendor_expect vendor_rehashed 1
new_vendor vendor_edit; git -C "$tmp/vendor_edit" commit -qm vendor
echo changed >> "$tmp/vendor_edit/vendor/mamut-workflows/payload.rs"
vendor_expect vendor_edit 1
# Even untracked or ignored extras must appear in the physical inventory.
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
# Metadata is required even when hashes and inventory match.
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
# Never run the PR script, even if it promises to validate the distribution.
new_vendor vendor_untrusted_script
mkdir "$tmp/vendor_untrusted_script/scripts"
echo 'exit 0' > "$tmp/vendor_untrusted_script/scripts/check-vendor.sh"
echo changed >> "$tmp/vendor_untrusted_script/vendor/mamut-workflows/payload.rs"
vendor_expect vendor_untrusted_script 1
# In addition to the exit code, require the scan diagnostic: identifier
# rejections must not pass because of an anchor or integrity failure.
vendor_id_expect() {
    vendor_expect "$1" 1
    local output
    output=$(< "$tmp/$1/check.log")
    if [[ "$output" != *"Identificador prohibido: $2 en vendor/mamut-workflows/$3:"* ||
          "$output" == *'Integridad vendor/mamut-workflows:'* ]]; then
        printf 'FAIL %s: missing identifier diagnostic with valid integrity\n' "$1"
        fails=$((fails+1))
    fi
}
# vendor_ok covers canon; each new pair must pass separately.
for file in mamut-workflow-cli/tests/fixtures/run.txt mamut-workflow-execution/src/mcp.rs mamut-workflow-execution/src/tests.rs; do
    name="vendor_allowed_${file//\//_}"
    new_vendor "$name" "$file" algorithm
    vendor_expect "$name" 0
done
# Same identifier in another path, including endings and paths that would share
# a leading portion/glob: they never inherit the exception. NOTICE and README are scanned.
for file in payload.rs mamut-workflow/src/lib.rs mamut-canon/src/other.rs mamut-canon/src/lib.rs.extra mamut-workflow-cli/tests/fixtures/run.txt.extra mamut-workflow-execution/src/other.rs mamut-workflow-execution/src/mcp.rs.extra mamut-workflow-execution/src/tests.rs.extra NOTICE README.md; do
    for identifier in algorithm fix; do
        name="vendor_id_${identifier}_${file//\//_}"
        new_vendor "$name" "$file" "$identifier"
        vendor_id_expect "$name" "$identifier" "$file"
    done
done
# Another identifier in any of the four paths is also forbidden.
for file in mamut-canon/src/lib.rs mamut-workflow-cli/tests/fixtures/run.txt mamut-workflow-execution/src/mcp.rs mamut-workflow-execution/src/tests.rs; do
    name="vendor_other_id_${file//\//_}"
    new_vendor "$name" "$file" fix
    vendor_id_expect "$name" fix "$file"
done
# First pin the final sample state and its anchor; then build a different
# synthetic base and restore the final state WITHOUT changing the anchor.
# This keeps forbidden content out of the added diff lines,
# while inventory, hashes and anchor are valid when running the full scan.
for kind in same_file other_file untracked ignored; do
    name="vendor_full_$kind"
    content=fix # No trailing newline for other_file/untracked/ignored.
    [[ "$kind" != same_file ]] || content=$'fix\n'
    new_vendor "$name" payload.rs "$content"
    v="$tmp/$name/vendor/mamut-workflows"
    file=NOTICE
    [[ "$kind" != same_file ]] || file=payload.rs
    cp "$v/$file" "$tmp/$name/final-file"
    case "$kind" in
        same_file) printf 'fix\n' > "$v/payload.rs" ;;
        other_file|untracked|ignored) echo 'safe baseline' >> "$v/NOTICE" ;;
    esac
    if [[ "$kind" == untracked || "$kind" == ignored ]]; then
        git -C "$tmp/$name" rm --cached -q vendor/mamut-workflows/payload.rs
        if [[ "$kind" == ignored ]]; then
            echo '/vendor/mamut-workflows/payload.rs' > "$tmp/$name/.gitignore"
            git -C "$tmp/$name" add .gitignore
        fi
    fi
    vendor_manifest "$name"
    # Do not add payload.rs again: it must remain untracked/ignored.
    git -C "$tmp/$name" add vendor/mamut-workflows/NOTICE vendor/mamut-workflows/MANIFEST.sha256
    [[ "$kind" != same_file ]] || git -C "$tmp/$name" add vendor/mamut-workflows/payload.rs
    git -C "$tmp/$name" commit -qm vendor
    cp "$tmp/$name/final-file" "$v/$file"
    vendor_manifest "$name"
    vendor_id_expect "$name" fix payload.rs
done
# The exception name is also forbidden outside the four paths.
new_repo vendor_algorithm_outside
printf 'algorithm\n' >> "$tmp/vendor_algorithm_outside/rules/forbidden_identifiers.txt"
echo algorithm > "$tmp/vendor_algorithm_outside/a.rs"
git -C "$tmp/vendor_algorithm_outside" add a.rs
vendor_expect vendor_algorithm_outside 1
# Other vendors are still scanned; forbidden paths are checked first.
for name in otro mamut-workflows-x; do
    new_repo "vendor_$name"; mkdir -p "$tmp/vendor_$name/vendor/$name"
    echo 'geomTransf(1)' > "$tmp/vendor_$name/vendor/$name/a.rs"
    git -C "$tmp/vendor_$name" add vendor
    vendor_expect "vendor_$name" 1
done
new_vendor vendor_forbidden_path
printf 'vendor/mamut-workflows/payload.rs\n' > "$tmp/vendor_forbidden_path/rules/forbidden_paths.txt"
vendor_expect vendor_forbidden_path 1
# Oracle outputs (#173 N2). Synthetic figures; no message prints content.
oracle_file() { # name path content expected
    new_repo "$1"; mkdir -p "$(dirname "$tmp/$1/$2")"; printf '%b' "$3" > "$tmp/$1/$2"; git -C "$tmp/$1" add -- "$2"
    expect "$1" "$4" bash "$src/check.sh" HEAD "$tmp/$1/rules"
}
# Paths
for f in modelos/porticoA.tcl modelos/a.tcl.txt docs/validation-report/datos/c1/opensees.csv datos/OpenSees_E1.CSV \
         datos/opensees-e1.csv.gz runs/opensees.txt runs/opensees.json runs/frame3dd/caso.out frame3dd-e1.out \
         runs/F3DD.out modelos/e1.3dd results/E1/salida.json oracles/opensees/results/a.json; do
    oracle_file "or_path_${f//[^a-zA-Z0-9]/_}" "$f" 'x\n' 1
done
new_repo or_engine_results; mkdir -p "$tmp/or_engine_results/crates/engine-api/fixtures/results" "$tmp/or_engine_results/provenance"
echo x > "$tmp/or_engine_results/crates/engine-api/fixtures/results/fingerprints.json"; echo p > "$tmp/or_engine_results/provenance/ENG-1.md"
git -C "$tmp/or_engine_results" add crates provenance
expect or_engine_results 0 bash "$src/check.sh" HEAD "$tmp/or_engine_results/rules"
oracle_file or_txt_ok docs/notas.txt 'x\n' 0
# Self-contained regression: all report text and figures below are invented.
# Reject a full report and isolated obtained output, with and without a period.
# No historical report or commit is read; metadata allowances remain unchanged.
synthetic_report='## (b) Humo sintetico de OpenSees\n\n- sha256:\n  - `bin/synthetic-oracle`: `cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc`\n- Entrada sintetica: E = 80e9, A = 5e-4, L = 4, P = 1200 (SI).\n- Salida esperada u = PL/(EA) = 1.2e-04; obtenido 8.765432109876543e-04.\n'
oracle_file or_incident_report_synth provenance/ENG-I.md "$synthetic_report" 1
oracle_file or_incident_obtained_dot_synth provenance/ENG-I.md '## (b) Humo sintetico de OpenSees\n- obtenido 8.765432109876543e-04.\n' 1
oracle_file or_incident_obtained_nodot_synth provenance/ENG-I.md '## (b) Humo sintetico de OpenSees\n- obtenido 8.765432109876543e-04\n' 1
unset synthetic_report
inc='## (b) Corrida manual de humo de OpenSees en el VPS\n\n- sha256:\n  - `bin/opensees-py`: `1111111111111111111111111111111111111111111111111111111111111111`\n- Entrada: barra elástica E = 100e9, A = 2e-4, L = 2, P = 500 (SI).\n- Salida esperada u = PL/(EA) = 5e-05; obtenido 1.2345678901234567e-05.\n'
oracle_file or_incident_synth provenance/ENG-S.md "$inc" 1
oracle_file or_l26_dot provenance/ENG-S.md '## (b) Humo de OpenSees\n- obtenido 1.2345678901234567e-05.\n' 1
oracle_file or_l26_nodot provenance/ENG-S.md '## (b) Humo de OpenSees\n- obtenido 1.2345678901234567e-05\n' 1
oracle_file or_l26_plain provenance/ENG-S.md '## Humo de OpenSees\n- obtenido 0.1234567.\n' 1
oracle_file or_nested provenance/ENG-S.md '## Humo de OpenSees\n### Caso 1\n- obtenido 3.25\n' 1
oracle_file or_codehash provenance/ENG-S.md '## Humo de OpenSees\n```\n# comentario\n```\n- obtenido 3.25\n' 1
oracle_file or_codefake docs/c.md '## Engine\n```\n# OpenSees\n```\n- salida 3.25\n' 0
oracle_file or_alias_os docs/a.md '## Corrida OS\n- salida 3.25\n' 1
oracle_file or_alias_f3dd docs/a.md 'Salida de F3DD: 3.25 kN\n' 1
oracle_file or_alias_open docs/a.md 'Resultado de Open Sees: 3.25\n' 1
oracle_file or_table docs/t.md '## Comparación con OpenSees\n| caso | T1 |\n| --- | --- |\n| E1 | 0.731 |\n' 1
oracle_file or_u_eq docs/u.md '## OpenSees\n- u = 0.25\n' 1
oracle_file or_result_en docs/r.md 'OpenSees result: 0.25 m\n' 1
oracle_file or_inputs docs/i.md '## Modelo de OpenSees\n- E = 200e9, L = 3\n' 1
oracle_file or_stabileo docs/s.md 'Stabileo output T1 0.731 s\n' 1
oracle_file or_comma docs/s.md 'Salida de OpenSees: 0,731 s\n' 1
# Allowed
oracle_file or_hash provenance/ENG-Y.md '## Corrida de OpenSees 3.8.0\n- sha256 de la salida: `b1bfa97ceb8f0f5d3c3c49eb498ac4e9dc57f86971f0f54c03074968a0f830a3`\n- openseespy 3.8.0.0, salida en 2026-10-20 13:09\n- commit `43b5f45`, ruta python3.12\n' 0
oracle_file or_redacted provenance/ENG-Z.md '## (b) Humo de OpenSees\n- Humo: ok. Entrada, salida y comando: [redactado: ver mamut-engine-oracles results/x.md].\n' 0
oracle_file or_mention docs/d.md 'No consulté código fuente ni resultados de OpenSees, Stabileo ni Frame3DD.\n' 0
oracle_file or_engine_only provenance/ENG-W.md '## Engine\n- Salida del Engine: 1.25e-03 m\n' 0
oracle_file or_sibling provenance/ENG-W.md '## OpenSees\n- instalado\n## Engine\n- salida 3.25\n' 0
oracle_file or_no_word docs/e.md 'OpenSees se compara con tolerancia relativa\n' 0
oracle_file or_os_lower docs/e.md 'los resultados del motor dan 3.25\n' 0
oracle_file or_url docs/g.md 'Hilo del foro de OpenSees (`viewtopic.php?p=122700`), entrada de licencia\n' 0
oracle_file or_section docs/g.md 'Opción A: Q2 ejecuta los oráculos de §5.1 (OpenSees); entrada en §2.3\n' 0
oracle_file or_tol docs/g.md '## Tolerancias\n| Comparación | Tolerancia relativa | Uso |\n|---|---:|---|\n| Engine vs OpenSees | 5e-3 | Criterio |\n' 0
oracle_file or_tol_rtol docs/g.md '## Tolerancias\n- OpenSees: rtol = 5e-3, atol <= 1e-9\n' 0
oracle_file or_tol_pct docs/g.md 'OpenSees frente al Engine: diferencia de formulación de 0.5–3.2 % por piso, no criterio\n' 0
oracle_file or_tol_out docs/g.md '## OpenSees\n- obtenido 3.25 (dentro de tol)\n' 1
oracle_file or_tol_out2 docs/g.md '## OpenSees\n- obtenido 3.25 (tol 1e-3)\n' 1
oracle_file or_tol_head docs/g.md '## Criterio OpenSees\n- obtenido 3.25\n' 1
oracle_file or_tol_head2 docs/g.md '## Tolerancias de OpenSees\n| caso | T1 |\n|---|---|\n| E1 | 0.731 |\n' 1
oracle_file or_toledo docs/g.md '## Corrida de OpenSees en Toledo\n| caso | T1 |\n|---|---|\n| E1 | 0.731 |\n' 1
oracle_file or_toledo2 docs/g.md 'OpenSees, Toledo: T1 0.731 s; u = 0.25\n' 1
oracle_file or_tol_big docs/g.md 'OpenSees: tol = 0.25 y T1 0.731\n' 1
oracle_file or_tol_rest docs/g.md 'OpenSees: rtol = 1e-3; T1 = 0.731 s; criterio\n' 1
oracle_file or_tol_rest_tab docs/g.md '## OpenSees\n| caso | rtol | T1 |\n|---|---|---|\n| E1 | 1e-3 | 0.731 |\n' 1
oracle_file or_tol_col_only docs/g.md '## OpenSees\n| caso | rtol |\n|---|---|\n| E1 | 1e-3 |\n' 0
oracle_file or_concentrada docs/g.py 'fix("Engine vs OpenSees", 1e-5, "masa concentrada idéntica")\n' 0
oracle_file or_r1_col docs/g.md '## OpenSees\n| caso | tolerancia |\n|---|---|\n| E1 | 731.25 |\n' 1
oracle_file or_r1_col_rel docs/g.md '## OpenSees\n| caso | Tolerancia relativa |\n|---|---|\n| E1 | 731.25 |\n' 1
oracle_file or_r1_col_rest docs/g.md '## OpenSees\n| caso | tolerancia |\n|---|---|\n| E1 | 1e-3 (T1 = 0.731) |\n' 1
oracle_file or_r1_col_ok docs/g.md '## OpenSees\n| caso | tolerancia |\n|---|---|\n| E1 | 1e-3 |\n| E2 | 2 % |\n' 0
oracle_file or_r2_under docs/g.md '## OpenSees\n- u_obtenido 3.25\n' 1
oracle_file or_r3_tolval docs/g.md '## OpenSees\n- u ≤ 0.005 (criterio)\n' 1
oracle_file or_r3_pct docs/g.md '## OpenSees\n- u: 3.2 % (tolerancia)\n' 1
oracle_file or_r3_approx docs/g.md '## OpenSees\n- u ≈ 0.25\n' 1
oracle_file or_r3_tilde docs/g.md '## OpenSees\n- T1 ~ 0.731 s\n' 1
oracle_file or_r4_thead docs/g.md '| caso | OpenSees |\n|---|---|\n| E1 | 0.731 |\n' 1
oracle_file or_r4_thead_engine docs/g.md '| caso | Engine |\n|---|---|\n| E1 | 0.731 |\n' 0
# Mode --all: scans the entire tree, even if the diff does not touch the file
new_repo or_all; mkdir -p "$tmp/or_all/docs"; printf '## OpenSees\n- obtenido 3.25\n' > "$tmp/or_all/docs/x.md"
git -C "$tmp/or_all" add docs; git -C "$tmp/or_all" commit -qm x
expect or_all 0 bash "$src/check.sh" HEAD "$tmp/or_all/rules"
cp -r "$tmp/or_all" "$tmp/or_all_tree"
expect or_all_tree 1 bash "$src/check.sh" --all
new_repo or_all_ok; expect or_all_ok 0 bash "$src/check.sh" --all
# vendor-crates/ (cargo vendor, MAM-938): exento del escaneo de identificadores, pero con integridad
new_vendor_crate() { # $1 = name; crate "c" con .cargo-checksum.json y un identificador prohibido
    new_repo "$1"; mkdir -p "$tmp/$1/vendor-crates/c/src"
    echo '{"files":{},"package":"0"}' > "$tmp/$1/vendor-crates/c/.cargo-checksum.json"
    echo 'x = geomTransf(1)' > "$tmp/$1/vendor-crates/c/src/lib.rs"
}
new_vendor_crate vc_skip; git -C "$tmp/vc_skip" add vendor-crates
expect vc_skip 0 bash "$src/check.sh" HEAD "$tmp/vc_skip/rules"
new_vendor_crate vc_link; ln -s ../../README.md "$tmp/vc_link/vendor-crates/c/src/l.rs"; git -C "$tmp/vc_link" add vendor-crates
expect vc_link 1 bash "$src/check.sh" HEAD "$tmp/vc_link/rules"
new_vendor_crate vc_nosum; rm "$tmp/vc_nosum/vendor-crates/c/.cargo-checksum.json"; git -C "$tmp/vc_nosum" add vendor-crates
expect vc_nosum 1 bash "$src/check.sh" HEAD "$tmp/vc_nosum/rules"
new_vendor_crate vc_outside; echo 'x = geomTransf(1)' > "$tmp/vc_outside/a.rs"; git -C "$tmp/vc_outside" add vendor-crates a.rs
expect vc_outside 1 bash "$src/check.sh" HEAD "$tmp/vc_outside/rules"
new_repo vc_prefix; mkdir -p "$tmp/vc_prefix/vendor-cratesX"; echo 'x = geomTransf(1)' > "$tmp/vc_prefix/vendor-cratesX/a.rs"; git -C "$tmp/vc_prefix" add vendor-cratesX
expect vc_prefix 1 bash "$src/check.sh" HEAD "$tmp/vc_prefix/rules"
printf '%d tests, %d failures\n' "$n" "$fails"
[[ $fails == 0 ]]
