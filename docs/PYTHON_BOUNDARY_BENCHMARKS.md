# ENG-PY-004 · offline boundary performance plan

Manual benchmark, outside CI, without network access or external reference engines. It measures the same
Rust engine and the same document for each size; Python only replicates the project's
own `cantilever-linear-static.json` fixture and transports results.
It implements no physical equations. The fixture's analytical reference and
independent references remain in the binding tests.

## Sizes and baselines

| Default size | Independent cantilevers | Nodes | Members | Active/free DOFs |
| --- | ---: | ---: | ---: | ---: |
| Small | 1 | 2 | 1 | 6/3 |
| Medium | 16 | 32 | 16 | 96/48 |
| Large | 64 | 128 | 64 | 384/192 |

These sizes are relative to the bounded V1 facade and its reference backend;
they do not represent a large production structure. Independent replicas
keep the problem well-defined without adding laws, connections or methods.
Coordinate translation and ID renumbering do not compute responses.

- `rust_api`: `boundary_benchmark` example, multiple direct calls to
  `engine_api::run` within the same release process. Excludes startup and input
  reading; includes engine parsing, analysis and JSON serialization.
- `cli_process`: release `mamut-engine` binary, the project's own workflow/catalog.
  Includes process, reading, dispatch and CLI envelope. Comparative envelope
  parsing occurs outside the measurement; this is not a pure FFI measurement.
- `native_json`: `_native.run` on the same string, including the PyO3 crossing,
  detach/reacquisition and copying JSON to the interpreter.
- `facade_run`: complete public API, including prevalidation, JCS fingerprint
  and reading/validation of `results/1`, in addition to Rust analysis.
- `parse_results`: `Results.from_json` of an already computed output.
- `export_lists` and `export_numpy`: export of an already computed Results,
  including the input stale/fingerprint check and full document copy.
  `export_numpy` includes list construction and an explicit ndarray copy;
  NumPy is imported before timing. Without NumPy that metric is skipped.

Byte equality between Rust/FFI/facade and the document contained
in CLI is checked before measurement; mismatches abort. `native_minus_rust_ns` approximates
the boundary crossing and JSON copy cost; `facade_minus_rust_ns` includes the entire
facade. These are differences of medians from separate processes, with noise and potentially
negative values; they must not be interpreted as exact internal profiles.
Phases are not additive: some include shared fingerprints/copies.

## Reproduction

Provision Rust 1.99.0, maturin 1.15.0, pytest and, optionally,
NumPy in advance. From the root, export `CARGO_TARGET_DIR` to the authorized directory and
`MATURIN` to the installed executable. The wheel output must be new:

```bash
export PATH="$RUST_TOOLCHAIN_BIN:$PATH"
export CARGO_NET_OFFLINE=true PIP_NO_INDEX=1
bash scripts/build-python-wheel.sh "$CARGO_TARGET_DIR/python-dist-nuevo"
python3 -m venv --system-site-packages "$CARGO_TARGET_DIR/python-bench-venv"
"$CARGO_TARGET_DIR/python-bench-venv/bin/python" -m pip install \
  --no-index --no-deps "$CARGO_TARGET_DIR/python-dist-nuevo/"*.whl
cargo build --release --offline --locked -p engine-cli
cargo build --release --offline --locked -p mamut-engine-py --example boundary_benchmark
"$CARGO_TARGET_DIR/python-bench-venv/bin/python" scripts/benchmark-python-boundary.py \
  --rust-baseline "$CARGO_TARGET_DIR/release/examples/boundary_benchmark" \
  --cli "$CARGO_TARGET_DIR/release/mamut-engine" \
  --work-dir "$CARGO_TARGET_DIR/python-bench-inputs-nuevo" \
  --repeats 7 --warmups 2 > "$CARGO_TARGET_DIR/python-boundary-report.json"
```

The input directory must be new. `--sizes` allows a selection of
1 to 64 replicas. Neither Cargo nor pip runs within the timed interval. The JSON
report contains sizes, input/output bytes, result hashes, Python/NumPy/engine
versions, architecture, individual samples, minimum, median and
maximum in ns. Keep Rust/maturin versions, profile,
flags, CPU, machine load and tested commit/diff alongside the report. Keep the same
tools and flags between baselines and wheel; no new internal parallelism.
To compare changes, repeat at least three full runs with an idle machine
and compare dispersion as well as medians; do not impose CI thresholds.

Default warmup is 2 and each metric has 7 samples.
Each result is destroyed outside the interval. The script does not measure
peak RSS: ndarray.nbytes would omit lists/JSON/temporary copies, so
it is not used as a total-memory approximation. NumPy's first import time
is also excluded. No speedup is promised and the GIL is not relaxed to
improve figures. Local evidence and its limits are recorded in
[provenance](../provenance/ENG-PY-004.md).

## Wheel tests with NumPy

The builder runs stdlib tests in an isolated consumer without NumPy.
To also run pytest against the wheel, use the previous temporary venv with
`--system-site-packages` (provisioned pytest/NumPy). Generate independent
references with the Rust example, bypassing PyO3:

```bash
mkdir "$CARGO_TARGET_DIR/python-test-references"
for fixture in crates/engine-api/fixtures/input/valid/*.json \
  crates/engine-api/tests/fixtures/determinism/*.json; do
  cargo run --offline --locked --quiet -p mamut-engine-py --example facade_reference -- \
    "$fixture" > "$CARGO_TARGET_DIR/python-test-references/${fixture##*/}"
done
export MAMUT_RUST_REFERENCES="$CARGO_TARGET_DIR/python-test-references"
export MAMUT_RUST_REFERENCE="$MAMUT_RUST_REFERENCES/cantilever-linear-static.json"
export PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 PYTHONDONTWRITEBYTECODE=1
"$CARGO_TARGET_DIR/python-bench-venv/bin/python" -I -B -m pytest \
  -q -p no:cacheprovider bindings/python/tests
```

`numpy_test.py` is collected with pytest's standard `*_test.py` pattern and uses
`pytest.importorskip("numpy")`; it is not loaded in unittest, which collects `test_*.py`
and does not require pytest. `test_arrays.py` tests lists and simulates the absence of
NumPy with a blocking importer in a fresh process, without uninstalling
system packages. The simulation checks lazy importing, analysis,
basic export and `MissingNumPyError`.
