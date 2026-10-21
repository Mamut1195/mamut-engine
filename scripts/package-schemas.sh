#!/usr/bin/env bash
# Solo produce un archivo local; no publica ni accede a la red.
# Ignora SOURCE_DATE_EPOCH: el mtime de las entradas del TAR se fija en 0.
set -euo pipefail
if [[ $# != 1 ]]; then
    printf 'Uso: bash scripts/package-schemas.sh DIRECTORIO_SALIDA\n' >&2
    exit 2
fi
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
python3 "$script_dir/generate-schemas.py" --package "$1"
