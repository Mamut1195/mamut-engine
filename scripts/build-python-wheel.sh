#!/usr/bin/env bash
# Builds and consumes locally; never downloads tools or publishes.
set -euo pipefail
export LC_ALL=C TZ=UTC
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export PIP_NO_INDEX=1 PIP_DISABLE_PIP_VERSION_CHECK=1
export SOURCE_DATE_EPOCH=1767225600 # 2026-01-01T00:00:00Z; versioned constant.

repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
python=$(command -v "${PYTHON:-python3}")
maturin=$(command -v "${MATURIN:-maturin}")
: "${CARGO_TARGET_DIR:?Export CARGO_TARGET_DIR before running this script}"
mkdir -p "$CARGO_TARGET_DIR"
target=$(cd "$CARGO_TARGET_DIR" && pwd)
out=${1:-$target/python-dist}
if [[ $# -gt 1 || -e "$out" ]]; then
    printf 'Usage: %s [new-output-directory]\nThe output must not exist.\n' "$0" >&2
    exit 2
fi
channel=$("$python" -c 'import tomllib; print(tomllib.load(open("rust-toolchain.toml", "rb"))["toolchain"]["channel"])')
export RUSTUP_TOOLCHAIN="$channel"
rustc_version=$(rustc --version)
if [[ $(awk '{print $2}' <<< "$rustc_version") != "$channel" ]]; then
    printf 'Rust %s must be installed locally; adjust PATH to that toolchain. It will not be downloaded.\n' "$channel" >&2
    exit 1
fi
maturin_version=$("$maturin" --version)
[[ "$maturin_version" == 'maturin 1.15.0' ]] || {
    printf 'Local maturin 1.15.0 is required (MATURIN=executable).\n' >&2; exit 1;
}
scratch=$(mktemp -d "$target/python-wheel.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
helper="$repo/scripts/python-wheel-artifacts.py"
"$python" -I -B -m unittest discover -s "$repo/scripts/tests" -v

cargo metadata --offline --locked --format-version 1 \
    --manifest-path bindings/python/Cargo.toml --features pyo3/extension-module \
    > "$scratch/metadata.json"

# Two separate builds, without reusing objects between them.
for build in one two; do
    # Cargo-encoded RUSTFLAGS: relative, reproducible panic/debuginfo paths;
    # the separator preserves path prefixes containing spaces.
    flags=("--remap-path-prefix=$repo=."
           "--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=cargo-home"
           "--remap-path-prefix=$scratch/$build=build")
    encoded=$(printf '%s\037' "${flags[@]}")
    CARGO_ENCODED_RUSTFLAGS=${encoded%$'\037'} \
        "$maturin" build --release --offline --locked --strip \
        --manifest-path bindings/python/Cargo.toml --interpreter "$python" \
        --target-dir "$scratch/$build/target" --out "$scratch/$build/wheels"
    wheels=("$scratch/$build/wheels/"*.whl)
    [[ ${#wheels[@]} == 1 && -f "${wheels[0]}" ]]
    "$python" "$helper" check-wheel "${wheels[0]}"
done
shopt -s nullglob
first=("$scratch/one/wheels/"*.whl)
second=("$scratch/two/wheels/"*.whl)
[[ ${#first[@]} == 1 && ${#second[@]} == 1 ]]
wheel_name=${first[0]##*/}
"$python" "$helper" compare "${first[0]}" "${second[0]}" > "$scratch/REPRODUCIBILITY.json"

# Generates and verifies everything before exposing the final directory.
mkdir "$scratch/dist"
cp "${first[0]}" "$scratch/dist/"
cp "$scratch/REPRODUCIBILITY.json" "$scratch/dist/"
"$python" "$helper" artifacts --metadata "$scratch/metadata.json" \
    --wheel "$scratch/dist/$wheel_name" --root "$repo" \
    --rustc-version "$rustc_version" --maturin-version "$maturin_version"

# Independent engine-api reference for tests on the installed wheel.
mkdir "$scratch/references"
for fixture in crates/engine-api/fixtures/input/valid/*.json \
    crates/engine-api/tests/fixtures/determinism/*.json; do
    cargo run --offline --locked --quiet -p mamut-engine-py --example facade_reference -- \
        "$fixture" > "$scratch/references/${fixture##*/}"
done

"$python" -I -m venv "$scratch/consumer"
venv="$scratch/consumer"
# Only venv binaries in PATH: no Rust, Cargo, or rustup.
(
    cd "$scratch"
    export PATH="$venv/bin"
    unset PYTHONPATH PYTHONHOME
    export MAMUT_RUST_REFERENCES="$scratch/references"
    export MAMUT_RUST_REFERENCE="$scratch/references/cantilever-linear-static.json"
    "$venv/bin/python" -I -m pip install --no-index --no-deps --no-cache-dir \
        "$scratch/dist/$wheel_name"
    "$venv/bin/python" -I "$helper" smoke "$scratch/dist/COMPATIBILITY.json" \
        > "$scratch/dist/SMOKE.json"
    "$venv/bin/python" -I -B -m unittest discover -s "$repo/bindings/python/tests" -v
)
"$python" "$helper" checksums "$scratch/dist"
mkdir -p "$(dirname "$out")"
mv "$scratch/dist" "$out"
printf 'Offline verification complete; local artifacts: %s\n' "$out"
