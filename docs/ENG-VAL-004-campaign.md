# ENG-VAL-004: candidate campaign

The provisional coordinator observations are in
[ENG-VAL-004-local-report.md](ENG-VAL-004-local-report.md). They do not
establish a performance budget or maximum supported model size.

From the checkout root in Windows PowerShell, with Rust 1.99.0, its
wasm32-unknown-unknown target and wasm-bindgen CLI 0.2.129 already installed,
build offline and generate the Node module:

```powershell
$env:CARGO_NET_OFFLINE = 'true'
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
rustc --version
wasm-bindgen --version
cargo build --offline --locked --release -p engine-cli
if ($LASTEXITCODE -ne 0) { throw 'Native build failed' }
cargo build --offline --locked --release -p engine-wasm --target wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) { throw 'WASM build failed' }
$wasmDir = Join-Path $env:TEMP 'eng-val-004-wasm'
wasm-bindgen --target nodejs --out-dir $wasmDir "$env:CARGO_TARGET_DIR\wasm32-unknown-unknown\release\engine_wasm.wasm"
if ($LASTEXITCODE -ne 0) { throw 'wasm-bindgen failed' }
$binary = Join-Path $env:CARGO_TARGET_DIR 'release\mamut-engine.exe'
```

Run one native case at a time, selecting an ID from the fixture index and an
analysis. This permits recording failures without losing subsequent cases:

```powershell
$index = Get-Content .\crates\engine-bench\fixtures\index.json -Raw | ConvertFrom-Json
$index.models | Select-Object id, nodes, members
$model = $index.models[0]
$analysis = 'static' # also modal or spectrum
.\scripts\eng-val-004-native.ps1 -Binary $binary -Model $model.id -Analysis $analysis -Repetitions 1 -TimeoutSeconds 60 -MaxObservedWorkingSetBytes 1GB |
  Out-File -Encoding utf8 -FilePath "$env:TEMP\eng-val-004-native.json"
if (-not $?) { throw 'Native case failed; retain the error separately' }
```

Repeat for the desired indexed models and analyses, changing output names to
retain each result. The script verifies input hashes, status and exact stdout
repeatability across repetitions; errors abort the invocation. The 1 GiB
threshold samples WorkingSet64 every 10 ms: it is not a hard memory cap and
can miss a peak. The 60-second timeout is the operational limit used in the
reported campaign. The output records checkout and binary identification;
`rustc_at_execution` does not prove which compiler built a supplied binary.

For a Node/WASM case, use the indexed input path and the same native binary:

```powershell
$inputPath = $model.engine_input.$analysis.path
node .\scripts\eng-val-004-wasm.cjs (Join-Path $wasmDir 'engine_wasm.js') $inputPath $binary
if ($LASTEXITCODE -ne 0) { throw 'WASM case failed' }
```

The runner starts native and WASM children with individual 120-second
timeouts. The reported campaign additionally enforced an external 60-second
limit per case. Its V8 old-space limit is not a cap on WASM linear memory or
total RSS; the runner has no full RSS cap. It reports worker RSS before and
after the call, not peak RSS; exported linear memory was unavailable in the
observed runs. Native elapsed time includes process startup, input transport
and output capture, while WASM call time measures the API call inside its
worker. These timings have different boundaries.

The runner parses both results as JavaScript JSON values and compares their
canonical structures after normalizing the provenance target only. JavaScript
number parsing is part of this boundary; this is not a byte comparison.
Record host conditions, overrides, errors and artifact hashes alongside results.
Temporary logs can be transient and have no publication guarantee.
