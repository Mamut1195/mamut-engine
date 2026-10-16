#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
: "${CARGO_TARGET_DIR:?Export CARGO_TARGET_DIR before building}"
wasm_bindgen=${WASM_BINDGEN:-/workspace/tools-bin/wbg/bin/wasm-bindgen}
node_bin=${NODE:-/home/box/.local/bin/node}
[[ $("$wasm_bindgen" --version) == 'wasm-bindgen 0.2.129' ]] || {
    printf 'Requires wasm-bindgen CLI 0.2.129\n' >&2
    exit 1
}
pkg=crates/engine-wasm/pkg
mkdir -p "$pkg/web" "$pkg/node"
cargo build --offline --locked --target wasm32-unknown-unknown --release -p engine-wasm
raw="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/engine_wasm.wasm"
"$wasm_bindgen" "$raw" --target web --out-dir "$pkg/web"
"$wasm_bindgen" "$raw" --target nodejs --out-dir "$pkg/node"
printf '{"type":"module"}\n' > "$pkg/web/package.json"
cargo run --offline --locked --release -p engine-wasm --example v1_native > "$pkg/v1.json"
"$node_bin" crates/engine-wasm/js/test-node.mjs
report="$pkg/size-report.md"
{
    printf '# ENG-SFC-001: release sizes\n\n'
    printf 'Toolchain: `%s`; CLI: `%s`\n\n' "$(rustc --version)" "$("$wasm_bindgen" --version)"
    printf 'Proposed budget: <= 1,000,000 bytes gzip; confirmation PENDING.\n\n'
    printf '| Artifact | Raw bytes | gzip -9 bytes | Against proposed budget |\n'
    printf '| --- | ---: | ---: | --- |\n'
    for file in "$raw" "$pkg/web/engine_wasm_bg.wasm" "$pkg/node/engine_wasm_bg.wasm"; do
        bytes=$(wc -c < "$file")
        compressed=$(gzip -9 -n -c "$file" | wc -c)
        comparison=PASS
        (( compressed <= 1000000 )) || comparison=OVER
        printf '| `%s` | %d | %d | %s |\n' "$file" "$bytes" "$compressed" "$comparison"
    done
    printf '\nSizes measure wasm only; generated JS and the example client are additional.\n'
    printf 'gzip uses -9 -n (no filename or timestamp); no wasm-opt or custom release profile.\n'
} > "$report"
cat "$report"
