#!/usr/bin/env bash
# ENG-VND-001: verify the complete vendored file inventory and SHA-256 hashes.
set -euo pipefail
export LC_ALL=C

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
printf 'Vendor: OK\n'
