#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C

fail() { printf 'Changelog: %s\n' "$*" >&2; exit 1; }
(( $# == 1 )) || fail 'uso: scripts/changelog-check.sh BASE_REF'
[[ $1 != -* ]] || fail "BASE_REF no puede empezar por '-': $1"
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$root"
base=$(git rev-parse --verify --quiet --end-of-options "$1^{commit}") || fail "BASE_REF no resuelve a un commit: $1"
if [[ ${CHANGELOG_RELEASE:-} == 1 ]]; then
    printf 'Changelog: exempt under CHANGELOG_RELEASE=1 (release PR).\n'
    exit 0
fi
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
git diff --no-ext-diff --no-renames --name-status -z "$base...HEAD" -- > "$scratch/diff" || fail "could not compare $base...HEAD."
code_changed=0
changelog_changed=0
added=()
while IFS= read -r -d '' status && IFS= read -r -d '' path; do
    case "$path" in
        crates/*|bindings/*) code_changed=1 ;;
        CHANGELOG.md) changelog_changed=1 ;;
        changelog.d/*)
            if [[ $status == A && $path != changelog.d/README.md ]]; then added+=("$path"); fi
            ;;
    esac
done < "$scratch/diff"
if (( ! code_changed )); then
    printf 'Changelog: no changes in crates/ or bindings/; no fragment required.\n'
    exit 0
fi
(( ! changelog_changed )) || fail 'un PR que modifica crates/ o bindings/ no debe editar CHANGELOG.md; añade un fragmento en changelog.d/.'
(( ${#added[@]} > 0 )) || fail 'cambios en crates/ o bindings/ requieren al menos un fragmento nuevo changelog.d/MAM-ID.tipo.md.'
for path in "${added[@]}"; do
    [[ $path =~ ^changelog\.d/MAM-[0-9]+\.(added|changed|fixed|validation|security)\.md$ ]] || fail "nombre de fragmento inválido: $path"
    # Check the committed blob, not the working copy or an external symlink target.
    mode=$(git ls-tree HEAD -- "$path")
    [[ $mode == '100644 '* || $mode == '100755 '* ]] || fail "el fragmento debe ser un archivo regular: $path"
    content=$(git show "HEAD:$path") || fail "could not read the fragment: $path"
    [[ $content =~ [^[:space:]] ]] || fail "fragmento vacío: $path"
done
printf 'Changelog: OK (%s new fragments).\n' "${#added[@]}"
