#!/usr/bin/env bash
# Only produces a local archive; does not publish or access the network.
# Ignores SOURCE_DATE_EPOCH: the mtime of TAR entries is set to 0.
set -euo pipefail
if [[ $# != 1 ]]; then
    printf 'Usage: bash scripts/package-schemas.sh OUTPUT_DIRECTORY\n' >&2
    exit 2
fi
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
python3 "$script_dir/generate-schemas.py" --package "$1"
