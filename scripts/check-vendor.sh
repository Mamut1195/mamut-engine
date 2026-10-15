#!/usr/bin/env bash
# ENG-VND-001: verify the complete vendored file inventory and SHA-256 hashes.
set -euo pipefail
export LC_ALL=C
PINNED_COMMIT=1eeb91a1ca9128ad4a02ead2ab66b6b25aa08f3d

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
cd -- "$script_dir/../vendor/mamut-workflows"

scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT

find . \( -type f -o -type l \) ! -path './MANIFEST.sha256' -print |
    sort > "$scratch/actual"
sed -n 's/^[0-9a-f]\{64\}  //p' MANIFEST.sha256 |
    sort > "$scratch/expected"
if [[ ! -s "$scratch/expected" ]]; then
    printf 'Vendor: empty or invalid manifest\n' >&2
    exit 1
fi
diff -u "$scratch/expected" "$scratch/actual"
sha256sum --check --strict MANIFEST.sha256
for file in README.md NOTICE; do
    grep -Fq -- "$PINNED_COMMIT" "$file" || {
        printf 'Vendor: pinned commit missing from %s\n' "$file" >&2
        exit 1
    }
done
[[ ! -L ../examples && ! -L ../examples/open-local ]]
for file in catalog.json workflow.json; do
    [[ -f "../examples/open-local/$file" && ! -L "../examples/open-local/$file" ]]
    cmp -- "examples/open-local/$file" "../examples/open-local/$file"
done
printf 'Vendor: OK\n'
