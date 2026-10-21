#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
export GIT_AUTHOR_DATE='2026-10-21T00:00:00Z' GIT_COMMITTER_DATE='2026-10-21T00:00:00Z'
unset SOURCE_DATE_EPOCH CHANGELOG_RELEASE GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
source_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
repo_count=0
checks=0

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
pass() { checks=$((checks + 1)); printf 'OK: %s\n' "$*"; }
new_repo() {
    repo_count=$((repo_count + 1))
    repo="$scratch/repo-$repo_count"
    git -c init.defaultBranch=main init --quiet "$repo"
    cd -- "$repo"
    git config user.name 'Changelog test'
    git config user.email 'changelog@example.invalid'
    git config core.hooksPath /dev/null
    git config commit.gpgSign false
    git config core.autocrlf false
    mkdir scripts changelog.d crates bindings
    cp -- "$source_root/scripts/changelog-assemble.sh" "$source_root/scripts/changelog-check.sh" scripts/
    printf '# Fragmentos\n' > changelog.d/README.md
    printf '# Changelog\n\n## Unreleased\n' > CHANGELOG.md
    printf 'base\n' > crates/example.txt
    printf 'base\n' > bindings/example.txt
    git add .
    git commit --quiet -m base
    git branch base
}
commit_all() { git add -A; git commit --quiet -m "$1"; }
expect_fail() {
    local expected=$1
    shift
    if "$@" > "$scratch/stdout" 2> "$scratch/stderr"; then
        fail "se esperaba un error: $*"
    fi
    local message
    message=$(< "$scratch/stderr")
    [[ $message == *"$expected"* ]] || fail "mensaje esperado '$expected', recibido '$message'"
}
assemble() { bash scripts/changelog-assemble.sh "$@"; }
check() { bash scripts/changelog-check.sh base; }

new_repo
cat > CHANGELOG.md <<'EOF'
# Changelog

## Unreleased

- Entrada antigua.
  Con continuación.

Otro párrafo legado.

## 0.1.0 - 2026-01-01

- Versión anterior.
EOF
printf 'Alta veinte\ncontinuación\n  ya indentada\n\nsegundo párrafo\n' > changelog.d/MAM-20.added.md
printf '%s\n' '- Alta dos' > changelog.d/MAM-2.added.md
printf 'Cambio\n\tya con tabulador\n' > changelog.d/MAM-1.changed.md
printf 'Corrección' > changelog.d/MAM-3.fixed.md
printf 'Pruebas\n' > changelog.d/MAM-4.validation.md
printf 'Seguridad\n' > changelog.d/MAM-5.security.md
cat > "$scratch/expected" <<'EOF'
# Changelog

## Unreleased

## 1.2.3 - 2026-10-21

### Añadido

- Alta dos

- Alta veinte
  continuación
  ya indentada

  segundo párrafo

### Cambiado

- Cambio
	ya con tabulador

### Corregido

- Corrección

### Validación

- Pruebas

### Seguridad

- Seguridad

### Otros (legado)

- Entrada antigua.
  Con continuación.

Otro párrafo legado.

## 0.1.0 - 2026-01-01

- Versión anterior.
EOF
cp CHANGELOG.md "$scratch/original"
cp -R changelog.d "$scratch/original-fragments"
before=$(git status --porcelain=v1 --untracked-files=all)
assemble --dry-run 1.2.3 2026-10-21 > "$scratch/preview-one"
assemble 1.2.3 2026-10-21 --dry-run > "$scratch/preview-two"
cmp "$scratch/preview-one" "$scratch/preview-two"
cmp "$scratch/expected" "$scratch/preview-one"
cmp "$scratch/original" CHANGELOG.md
[[ $(git status --porcelain=v1 --untracked-files=all) == "$before" ]] || fail 'dry-run modificó archivos'
for file in "$scratch/original-fragments"/*; do cmp "$file" "changelog.d/${file##*/}"; done
pass 'dry-run sin mutaciones; orden, formato, legado y versiones anteriores exactos'
commit_all release-inputs
assemble 1.2.3 2026-10-21
cmp "$scratch/expected" CHANGELOG.md
remaining=(changelog.d/*)
[[ ${#remaining[@]} == 1 && ${remaining[0]} == changelog.d/README.md ]] || fail 'no eliminó solo los fragmentos'
cp CHANGELOG.md "$scratch/assembled-one"
cp "$scratch/original" CHANGELOG.md
cp -- "$scratch/original-fragments/"* changelog.d/
# Reverse file creation order and change timezone; epoch must still mean UTC.
rm changelog.d/MAM-2.added.md
cp "$scratch/original-fragments/MAM-2.added.md" changelog.d/
SOURCE_DATE_EPOCH=1792540800 TZ=Pacific/Honolulu assemble 1.2.3
cmp "$scratch/assembled-one" CHANGELOG.md
pass 'dos ensamblados idénticos byte a byte; fecha desde SOURCE_DATE_EPOCH en UTC'
expect_fail 'ya existe' assemble 1.2.3 2026-10-21
pass 'rechazo de versión duplicada'

new_repo
printf 'Contenido\n' > changelog.d/MAM-1.added.md
expect_fail 'DATE o define SOURCE_DATE_EPOCH' assemble 1.0
SOURCE_DATE_EPOCH=invalid expect_fail 'SOURCE_DATE_EPOCH' assemble 1.0
expect_fail 'DATE inválida' assemble 1.0 2026-02-30
expect_fail 'AAAA-MM-DD' assemble 1.0 tomorrow
cp CHANGELOG.md "$scratch/original"
for bad in MAM-x.added.md MAM-1.other.md MAM-1.added.txt .gitkeep; do
    printf 'Inválido\n' > "changelog.d/$bad"
    expect_fail "nombre de fragmento inválido: changelog.d/$bad" assemble 1.0 2026-10-21
    cmp CHANGELOG.md "$scratch/original"
    [[ -f changelog.d/MAM-1.added.md ]] || fail 'un error eliminó fragmentos'
    rm -- "changelog.d/$bad"
done
mkdir changelog.d/MAM-2.fixed.md
expect_fail 'archivo regular' assemble --dry-run 1.0 2026-10-21
rmdir changelog.d/MAM-2.fixed.md
ln -s MAM-1.added.md changelog.d/MAM-2.fixed.md
expect_fail 'archivo regular' assemble 1.0 2026-10-21
rm changelog.d/MAM-2.fixed.md
for empty in '' $' \t\n\n'; do
    printf '%s' "$empty" > changelog.d/MAM-2.fixed.md
    expect_fail 'fragmento vacío' assemble 1.0 2026-10-21
done
rm changelog.d/MAM-2.fixed.md
printf '# Sin Unreleased\n' > CHANGELOG.md
expect_fail 'falta el encabezado' assemble 1.0 2026-10-21
printf '## Unreleased\n\n## Unreleased\n' > CHANGELOG.md
expect_fail 'más de una sección' assemble 1.0 2026-10-21
pass 'fechas, nombres, archivos vacíos, enlaces y encabezados inválidos sin consumo de fragmentos'

new_repo
assemble 1.0 2026-10-21
printf '# Changelog\n\n## Unreleased\n\n## 1.0 - 2026-10-21\n\n' > "$scratch/expected-empty"
cmp "$scratch/expected-empty" CHANGELOG.md
pass 'release sin fragmentos ni legado'

new_repo
printf 'Entrada\n' > changelog.d/MAM-1.added.md
commit_all release-inputs
cp CHANGELOG.md "$scratch/original"
cp changelog.d/MAM-1.added.md "$scratch/original-fragment"
git rm --quiet --cached CHANGELOG.md
expect_fail 'CHANGELOG.md debe estar versionado en Git' assemble 1.0 2026-10-21
cmp "$scratch/original" CHANGELOG.md
cmp "$scratch/original-fragment" changelog.d/MAM-1.added.md
git add CHANGELOG.md
pass 'ensamblado rechaza CHANGELOG.md sin seguimiento sin modificar archivos'

for dirty in unstaged staged untracked; do
    case "$dirty" in
        unstaged) printf 'cambio\n' >> crates/example.txt ;;
        staged) git add crates/example.txt ;;
        untracked)
            git restore --staged --worktree crates/example.txt
            printf 'sin seguimiento\n' > notes.md
            git config status.showUntrackedFiles no
            ;;
    esac
    before=$(git status --porcelain --untracked-files=all)
    expect_fail 'el árbol de trabajo Git debe estar limpio' assemble 1.0 2026-10-21
    cmp "$scratch/original" CHANGELOG.md
    cmp "$scratch/original-fragment" changelog.d/MAM-1.added.md
    [[ $(git status --porcelain --untracked-files=all) == "$before" ]] || fail 'el rechazo modificó el árbol'
done
pass 'ensamblado rechaza cambios sin preparar, preparados y sin seguimiento sin mutaciones'

mv .git "$scratch/nonrepo-git"
expect_fail 'se requiere un árbol de trabajo Git' assemble 1.0 2026-10-21
cmp "$scratch/original" CHANGELOG.md
cmp "$scratch/original-fragment" changelog.d/MAM-1.added.md
assemble --dry-run 1.0 2026-10-21 > "$scratch/nonrepo-preview"
pass 'ensamblado requiere un árbol Git; dry-run funciona fuera de Git'

new_repo
printf 'documentación\n' > notes.md
commit_all docs
check
expect_fail 'uso:' bash scripts/changelog-check.sh
expect_fail "BASE_REF no puede empezar por '-': -HEAD" bash scripts/changelog-check.sh -HEAD
output_path="$scratch/injected-output"
expect_fail "BASE_REF no puede empezar por '-': --output=$output_path" bash scripts/changelog-check.sh "--output=$output_path"
[[ ! -e $output_path ]] || fail 'BASE_REF creó un archivo con --output'
expect_fail 'BASE_REF no resuelve a un commit: no-such-ref' bash scripts/changelog-check.sh no-such-ref
pass 'check: documentación exenta y referencia/argumentos inválidos rechazados'

for area in crates bindings; do
    new_repo
    printf 'cambio\n' >> "$area/example.txt"
    commit_all code
    expect_fail 'requieren al menos un fragmento nuevo' check
    CHANGELOG_RELEASE=1 check
    CHANGELOG_RELEASE=0 expect_fail 'requieren al menos un fragmento nuevo' check
    printf 'Entrada\n' > changelog.d/MAM-10.fixed.md
    commit_all fragment
    check
    # The check must use committed content, not an uncommitted edit.
    printf '' > changelog.d/MAM-10.fixed.md
    check
    printf 'Entrada\n' > changelog.d/MAM-10.fixed.md
    printf 'Edición directa\n' >> CHANGELOG.md
    commit_all direct-changelog
    expect_fail 'no debe editar CHANGELOG.md' check
    CHANGELOG_RELEASE=1 check
done
pass 'check: crates y bindings exigen fragmento, rechazan CHANGELOG y permiten release'

new_repo
printf 'Original\n' > changelog.d/MAM-10.added.md
commit_all existing-fragment
git branch -f base HEAD
printf 'cambio\n' >> crates/example.txt
printf 'Modificado\n' > changelog.d/MAM-10.added.md
commit_all modified-fragment
expect_fail 'requieren al menos un fragmento nuevo' check
pass 'check: modificar un fragmento existente no cuenta como añadido'

for candidate in MAM-bad.fixed.md MAM-11.unknown.md MAM-12.fixed.md; do
    new_repo
    printf 'cambio\n' >> crates/example.txt
    printf ' \t\n' > "changelog.d/$candidate"
    commit_all invalid-fragment
    if [[ $candidate == MAM-12.fixed.md ]]; then
        expect_fail 'fragmento vacío' check
    else
        expect_fail 'nombre de fragmento inválido' check
    fi
done
new_repo
printf 'cambio\n' >> bindings/example.txt
ln -s ../bindings/example.txt changelog.d/MAM-12.fixed.md
commit_all symlink-fragment
expect_fail 'archivo regular' check
pass 'check: nombres inválidos, fragmentos vacíos y enlaces rechazados'

new_repo
git checkout --quiet -b first
printf 'Primera rama\n' > changelog.d/MAM-20.added.md
commit_all first-fragment
git checkout --quiet -b second base
printf 'Segunda rama\n' > changelog.d/MAM-21.added.md
commit_all second-fragment
git merge --quiet --no-edit first
[[ -f changelog.d/MAM-20.added.md && -f changelog.d/MAM-21.added.md ]] || fail 'faltan fragmentos tras merge'
[[ -z $(git ls-files --unmerged) ]] || fail 'merge con conflictos'
assemble 1.0 2026-10-21
printf '# Changelog\n\n## Unreleased\n\n## 1.0 - 2026-10-21\n\n### Añadido\n\n- Primera rama\n\n- Segunda rama\n\n' > "$scratch/expected-merge"
cmp "$scratch/expected-merge" CHANGELOG.md
pass 'dos ramas añaden fragmentos, fusionan sin conflictos y ensamblan ambas entradas'

printf 'Changelog: %s grupos de pruebas OK.\n' "$checks"
