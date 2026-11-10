# CLI for agents · ENG-DOC-001 · MAM-870

This page is for an AI agent, or any script, that drives the `mamut-engine`
binary from a terminal without prior context. With `describe --json` and this
page it can **validate** an engine-input/1 model, **run** it and **read a roof
displacement** from the result. Every command below was run against the real
binary; the blocks marked `agent-example` are re-run by a test (see
[§8](#8-how-this-guide-is-verified)).

Scope: the CLI as it is on `main` (MAM-863, 864, 865, 866, 867 and 868 merged).
**Not covered**: the MCP server (`mcp`, whose tools still return fingerprint
manifests only), the Python and WebAssembly bindings, and published binaries
(none yet). Build the binary from the repository root with
`cargo build --release -p engine-cli --offline`; it is
`target/release/mamut-engine`. The detailed contract is the
[engine-cli README](../crates/engine-cli/README.md).

Conventions in the examples: run them from the repository root, with
`mamut-engine` on `PATH`; only `sh` (POSIX) and `python3` are used, so nothing
else has to be installed. Temporary files go to `$TMPDIR`.

## 1. The five rules

1. **One command, one JSON document.** A successful command writes a single
   compact JSON document and a final newline to **stdout**. **stderr** carries
   progress events (only with `--progress ndjson`) and, on failure, one error
   envelope. Nothing else is printed.
2. **Read the exit code first, then parse.** The exit code is the contract
   ([§5](#5-exit-codes-and-errors)). It is `0` only when the result is complete
   and has no warnings.
3. **Never parse message text.** Branch on the exit code, on `schema`, and on
   `code` (`ENG-E-NNNN`, `API-[EWI]-…`, `CLI-E-…`). The `message`, `cause` and
   `hint` strings are for people. In particular, the `description` strings of
   `describe --json` are currently written in Spanish; use only their field
   names and ids.
4. **The output is deterministic.** The same input, engine version and build
   target give identical bytes (see [§4.6](#46-check-determinism-and-schema-integrity)).
   Output contains no clock, host name or local path.
5. **The CLI reads nothing implicit.** No network, no locale, no configuration
   file. The only environment variables are the three listed by
   `describe --json` under `environment`.

Where the answer lands depends on the command:

| Command | Exit 0 | Exit 1 | Exit 2 and above |
| --- | --- | --- | --- |
| `validate --input` | stdout: `validate-report/1`, `status: "ok"` | stdout: `validate-report/1`, `status: "warning"` (reserved: no current check produces it) | Rejected model (2): stdout `validate-report/1`, `status: "invalid"`, stderr empty. Usage, unreadable input, cancellation or write failure: stderr `cli-error/1`, stdout empty |
| `run` | stdout: `cli-run/1`, `outcome: "ok"` | stdout: `cli-run/1`, `outcome: "warning"`, **full results** | stdout empty; stderr: `cli-error/1` as its last line |
| `describe --json`, `capabilities`, `schema`, `explain`, `--version` | stdout | not used | stderr `cli-error/1` (code 2 for an unknown id or bad arguments) |

## 2. Discover the CLI with `describe --json`

```sh agent-example id=version exit=0
mamut-engine --version
```

```text agent-output id=version
mamut-engine <VERSION>
```

`mamut-engine describe --json` prints the whole contract (`cli-describe/1`,
about 10 KB). Its top-level fields:

| Field | What an agent uses it for |
| --- | --- |
| `commands[]` | Each command: `name`, `flags[]` (`name`, `required`, `value` such as `path\|-`), `arguments`, and `output`, the id of the document it prints |
| `globalFlags[]` | Flags valid before any command (`--timeout`) |
| `environment[]` | The three environment variables and the commands they affect |
| `inputs[]` | Accepted documents: `id`, `transport` (`--input path\|-`) and `maxBytes` |
| `outputs[]` | Documents the CLI emits: `id`, `channel` (`stdout` or `stderr`) and `format` |
| `exitCodes` | Map from exit code (string) to name |
| `limits` | Public limits ([§6](#6-limits)) |
| `schemas[]` | Contract inventory: `id`, `$id`, `sha256` of the published bytes, and `availableViaSchema` |
| `argumentSyntax` | How to write arguments ([§7](#7-pitfalls-found-while-testing)) |

```sh agent-example id=discover-commands exit=0
mamut-engine describe --json > "$TMPDIR/describe.json"
python3 - "$TMPDIR/describe.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
print(sorted(c["name"] for c in d["commands"]))
print(json.dumps(d["exitCodes"], sort_keys=True))
print(sorted(d["limits"])[:3], "...")
PY
```

```text agent-output id=discover-commands
['--version', 'capabilities', 'compile', 'describe', 'explain', 'help', 'mcp', 'methods', 'run', 'schema', 'validate']
{"0": "ok", "1": "warning", "2": "invalid", "3": "unsupported", "4": "not_converged", "5": "cancelled", "6": "timeout", "70": "internal"}
['MAX_CASES', 'MAX_COMBINATIONS', 'MAX_CONSOLIDATION_COST'] ...
```

(`describe` without `--json` prints the historical description of the base
workflow CLI; agents should always pass `--json`. `--help --json` is equivalent.)

Other read-only queries, all with exit 0 and no input document:

| Command | Prints |
| --- | --- |
| `capabilities` | `capabilities/1`: supported elements, laws, analyses and method descriptors |
| `schema <id>` | The exact bytes of a JSON Schema, for the ids with `availableViaSchema: true`: `capabilities/1`, `engine-input/1`, `method-descriptor/1`, `results/1` and `validate-report/1` |
| `explain <code>` | A human explanation of an `ENG-E-NNNN` or `API-…` code; `explain <code> --json` prints `{"code","explanation"}` |

The other contracts in `schemas[]` (`cli-run/1`, `cli-error/1`,
`cli-describe/1`, `explain/1`, `progress/1`, `version/1`) are published as files
in the repository's `schemas/` tree; `schema <id>` rejects them:

```sh agent-example id=schema-unavailable exit=2
mamut-engine schema cli-run/1 2>"$TMPDIR/err.json" >"$TMPDIR/out.json"
rc=$?
python3 - "$TMPDIR/err.json" "$TMPDIR/out.json" <<'PY'
import json, os, sys
e = json.loads(open(sys.argv[1]).read().splitlines()[-1])
print(e["schema"], e["code"], e["exitCode"], "stdout bytes:", os.path.getsize(sys.argv[2]))
PY
exit $rc
```

```text agent-output id=schema-unavailable
cli-error/1 CLI-E-USAGE 2 stdout bytes: 0
```

To write a model, fetch the schema: `mamut-engine schema engine-input/1`. To know
the shape of a result, fetch `mamut-engine schema results/1`.

## 3. Walkthrough: validate, run, read the roof displacement

The model is [two-storey-frame.json](examples/agent-cli/two-storey-frame.json),
an engine-input/1 document. Units are fixed by `"units": "SI"`: metres, newtons,
pascals, radians. It is a one-bay, two-storey steel frame in the model plane
(D2, nodes carry `Ux`, `Uy`, `Rz`):

| Nodes | Position (x, y) in m | Role |
| --- | --- | --- |
| 1, 2 | (0, 0), (6, 0) | Base, fully restrained |
| 3, 4 | (0, 3.5), (6, 3.5) | First floor |
| 5, 6 | (0, 7), (6, 7) | Roof |

Columns are members 11, 12, 21 and 22 (`E` = 2×10¹¹ Pa, `I` = 8×10⁻⁵ m⁴); beams
31 and 32 are 1000 times stiffer in bending. Load case `wind` applies 10 kN in
`+Ux` at node 3 and 20 kN in `+Ux` at node 5, with analysis `linearStatic`.

> **Y is not "up" for the engine.** For D2 models the engine fixes the plane as
> X–Y and does not interpret Y as vertical (the elevation convention is pending,
> see [AXES_CONVENTION.md](AXES_CONVENTION.md)). This guide draws storeys along
> `y` and defines "roof" as the nodes with the largest `y`. That is a convention
> of this example, not an engine concept. With no gravity load, it does not
> change the physics. The model also declares `"axes": "Z-up"` because that is
> the only value `engine-input/1` accepts; for D2 models it does not make `y`
> vertical.

**Step 1: validate.** `validate --input <path|->` checks syntax, schema,
conventions, limits and topology. It does not assemble or solve the model, so a
valid report does not guarantee that `run` succeeds (a mechanism passes
validation and fails in `run` with exit 4).

```sh agent-example id=validate-ok exit=0
mamut-engine validate --input docs/examples/agent-cli/two-storey-frame.json
```

```text agent-output id=validate-ok
{"diagnostics":[],"operation":"validate","schema":"validate-report/1","status":"ok"}
```

A rejected model gives exit 2 and a report with the diagnostics on stdout.
Repository fixture with a member that references a missing node:

```sh agent-example id=validate-invalid exit=2
mamut-engine validate --input crates/engine-api/fixtures/input/invalid/missing-reference.json
```

```text agent-output id=validate-invalid
{"diagnostics":[{"cause":"The node is not connected to any member.","code":"ENG-E-0004","entities":[{"id":2,"kind":"node"}],"hint":"Node 2","severity":"error"},{"cause":"The referenced entity does not exist. Missing node 99, referenced by member 7 at end 1 (members[0].nodes[1]).","code":"ENG-E-0003","entities":[{"id":7,"kind":"member"}],"entity_kind":"node","hint":"Member 7, end j: missing node 99","missing_id":99,"referenced_by":"members[0].nodes[1]","severity":"error"}],"operation":"validate","schema":"validate-report/1","status":"invalid"}
```

Each diagnostic has `code`, `severity` (`info`, `warning`, `error`), `entities`
(what it points to), `cause` and `hint`. A broken reference keeps `ENG-E-0003`
and adds `entity_kind`, `missing_id` and `referenced_by`. `suggestions` is
present only when a nearby id of the same kind exists.

**Step 2: run.** Redirect stdout to a file and keep the exit code. The default
`run` needs no catalog or workflow: it uses the embedded ones.

```sh agent-example id=run-save exit=0
mamut-engine run --input docs/examples/agent-cli/two-storey-frame.json > "$TMPDIR/frame-run.json"
rc=$?
echo "exit=$rc"
python3 - "$TMPDIR/frame-run.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
print(d["schema"] if "schema" in d else "(no schema field)", d["outcome"], d["exitCode"], len(d["engineResults"]))
PY
exit $rc
```

```text agent-output id=run-save
exit=0
(no schema field) ok 0 1
```

The `cli-run/1` envelope has no `schema` field (the contract name is the
command's `output` id). It carries `type`, `engineId`, `engineVersion`,
`operation: "run"`, `outcome`, `exitCode`, `data` (an execution manifest with
SHA-256 fingerprints, **not** the calculated values) and `engineResults`: one
`results/1` document per executed step, in order. For `run` there is exactly one.

**Step 3: read the result.** In `engineResults[0]`:

* `nodes[]` describes the model: `id` and `coordinates` `[x, y, z]`.
* `cases[]` has one entry per load case (and `combinations[]` one per
  combination), each with `nodes[]`: `id`, `displacements`, `prescribed_displacements`
  and `reactions`.
* `displacements` and `reactions` are always six numbers in the fixed order
  **`[Ux, Uy, Uz, Rx, Ry, Rz]`** (translations in m or forces in N, rotations in
  rad or moments in N·m). A D2 node uses indices 0, 1 and 5.
* `status` repeats the outcome (`ok`, `warning`, `invalid`, `unstable`, …) and
  `diagnostics[]` lists warnings.

The roof displacement is `Ux` (index 0) of the nodes with the largest `y`, in
load case `wind`:

```sh agent-example id=roof-displacement exit=0
python3 - "$TMPDIR/frame-run.json" <<'PY'
import json, sys
r = json.load(open(sys.argv[1]))["engineResults"][0]
top = max(n["coordinates"][1] for n in r["nodes"])
roof = {n["id"] for n in r["nodes"] if n["coordinates"][1] == top}
case = next(c for c in r["cases"] if c["case"] == "wind")
for n in sorted(case["nodes"], key=lambda n: n["id"]):
    if n["id"] in roof:
        print(f"roof node {n['id']}: Ux = {n['displacements'][0]:.6e} m")
PY
```

```text agent-output id=roof-displacement
roof node 5: Ux = 5.610851e-03 m
roof node 6: Ux = 5.604855e-03 m
```

The roof moves about **5.61 mm**. The two roof nodes differ by 6 µm because `Ux`
can differ along a horizontal beam only through its axial deformation.
Sanity check against a hand estimate: if the beams were
rigid, each storey would have stiffness 2 · 12 E I / h³, and the roof
displacement would be `(30 kN + 20 kN) / k`, because storey 1 carries 30 kN and
storey 2 carries 20 kN:

```sh agent-example id=hand-check exit=0
python3 - <<'PY'
E, I, h = 2.0e11, 8.0e-5, 3.5
k = 2 * 12 * E * I / h**3
roof = 30e3 / k + 20e3 / k
print(f"storey stiffness {k:.4e} N/m, shear-building roof estimate {roof:.4e} m")
PY
```

```text agent-output id=hand-check
storey stiffness 8.9563e+06 N/m, shear-building roof estimate 5.5827e-03 m
```

The engine value is within 1 % of the estimate.

## 4. Recipes

### 4.1 Pipes and stdin

`--input -` reads the model from stdin to end of input (limit 8 MiB). It is the
way to chain tools without temporary files:

```sh agent-example id=pipe exit=0
cat docs/examples/agent-cli/two-storey-frame.json | mamut-engine validate --input -
cat docs/examples/agent-cli/two-storey-frame.json | mamut-engine run --input - | python3 -c '
import json, sys
r = json.load(sys.stdin)["engineResults"][0]
print("nodes in result:", len(r["nodes"]), "status:", r["status"])'
```

```text agent-output id=pipe
{"diagnostics":[],"operation":"validate","schema":"validate-report/1","status":"ok"}
nodes in result: 6 status: ok
```

Only **one** source can come from stdin: `--input -` together with `--catalog -`
or `--workflow -` is a usage error (exit 2). A shell pipeline reports the exit
code of its last command, so capture the engine's code from a file redirection
(as in [§3](#3-walkthrough-validate-run-read-the-roof-displacement)) or use
`set -o pipefail` in a shell that has it.

### 4.2 Branch on the exit code

```sh agent-example id=branch exit=0
mamut-engine run --input docs/examples/agent-cli/two-storey-frame.json \
  >"$TMPDIR/out.json" 2>"$TMPDIR/err.ndjson"
rc=$?
case $rc in
  0) echo "ok: read $TMPDIR/out.json" >/dev/null; echo "complete result" ;;
  1) echo "warning: result is complete; inspect engineResults[0].diagnostics" ;;
  2) echo "invalid usage or input: correct the command or the model (stderr)" ;;
  3) echo "unsupported element or capability: do not retry unchanged" ;;
  4) echo "unstable or not converged: change the model" ;;
  5) echo "cancelled" ;;
  6) echo "timeout: raise --timeout or simplify" ;;
  *) echo "internal error ($rc): report it" ;;
esac
```

```text agent-output id=branch
complete result
```

### 4.3 Read a failure

On failure, `run` leaves stdout empty and the **last line of stderr** is a
`cli-error/1` envelope. With `--progress ndjson` it is preceded by event lines,
so always take the last line. Its fields: `exitCode`, `outcome` (`usage`,
`invalid_input`, `unsupported`, `not_converged`, `cancelled`, `timeout`,
`internal`), `code`, `diagnostics[]` (same shape as above), `operation` and
`messageKey`. When the engine itself rejected the model it also carries
`engineResults` with the failed `results/1` document. The next block makes a
mechanism by removing the supports:

```sh agent-example id=exit-unstable exit=4
python3 - <<'PY' > "$TMPDIR/mechanism.json"
import json
m = json.load(open("docs/examples/agent-cli/two-storey-frame.json"))
m["model"]["supports"] = []
print(json.dumps(m))
PY
mamut-engine run --input "$TMPDIR/mechanism.json" >"$TMPDIR/out.json" 2>"$TMPDIR/err.ndjson"
rc=$?
python3 - "$TMPDIR/err.ndjson" "$TMPDIR/out.json" <<'PY'
import json, os, sys
e = json.loads(open(sys.argv[1]).read().splitlines()[-1])
print(e["schema"], e["code"], e["exitCode"], e["outcome"], "stdout bytes:", os.path.getsize(sys.argv[2]))
PY
exit $rc
```

```text agent-output id=exit-unstable
cli-error/1 ENG-E-0006 4 not_converged stdout bytes: 0
```

`explain` turns the code into advice:

```sh agent-example id=explain exit=0
mamut-engine explain ENG-E-0006
```

```text agent-output id=explain
The equations do not determine a unique solution. Check supports, connectivity, releases, and stiffness for unresisted movement.
```

Unsupported (exit 3) is the same flow: set `model.members[0].element` to an
unknown name and `run` fails with code `ENG-E-0013`.

### 4.4 Warnings keep the results

Exit 1 is not a failure. The engine finished, and `engineResults` is complete,
but a diagnostic with `severity: "warning"` needs review. A combination that
scales a load case (factor other than 1) of this model raises
`API-W-COMBINED-PRESCRIBED`:

```sh agent-example id=exit-warning exit=1
python3 - <<'PY' > "$TMPDIR/scaled.json"
import json
m = json.load(open("docs/examples/agent-cli/two-storey-frame.json"))
m["combinations"] = [{"id": "scaled", "terms": [{"case": "wind", "factor": 2.0}]}]
print(json.dumps(m))
PY
mamut-engine run --input "$TMPDIR/scaled.json" > "$TMPDIR/out.json"
rc=$?
python3 - "$TMPDIR/out.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
r = d["engineResults"][0]
print(d["outcome"], d["exitCode"], r["status"], [x["code"] for x in r["diagnostics"]], len(r["combinations"]))
PY
exit $rc
```

```text agent-output id=exit-warning
warning 1 warning ['API-W-COMBINED-PRESCRIBED'] 1
```

### 4.5 Progress, timeout and cancellation

`--progress ndjson` (only for `run`) writes one JSON object per line to stderr:
`started` and `finished` events of the workflow step (field `phase`) and
`progress/1` lines (field `schema`) with `stage`, `completed` and `total`. They
contain no timestamps. stdout is byte-identical with or without the flag.

```sh agent-example id=progress exit=0
mamut-engine run --input docs/examples/agent-cli/two-storey-frame.json --progress ndjson 2>&1 >/dev/null
```

```text agent-output id=progress
{"capability_slot":"engine","index":0,"method_id":"engine.run","method_version":1,"phase":"started"}
{"completed":1,"index":0,"operation":"run","schema":"progress/1","stage":"parse","total":6}
{"completed":2,"index":0,"operation":"run","schema":"progress/1","stage":"model","total":6}
{"completed":3,"index":0,"operation":"run","schema":"progress/1","stage":"assembly","total":6}
{"completed":4,"index":0,"operation":"run","schema":"progress/1","stage":"case","total":6}
{"completed":5,"index":0,"operation":"run","schema":"progress/1","stage":"combinations","total":6}
{"completed":6,"index":0,"operation":"run","schema":"progress/1","stage":"serialization","total":6}
{"capability_slot":"engine","index":0,"method_id":"engine.run","method_version":1,"phase":"finished"}
```

`total` is a maximum: a run may finish before `completed` reaches it.

`--timeout <seconds>` (a number with 0 < s ≤ 31 536 000) or
`MAMUT_ENGINE_TIMEOUT` stops a run cooperatively with exit 6 and
`CLI-E-TIMEOUT`. Write the flag before the command, as documented
(`--timeout 30 run …`); values that are zero, negative or not numbers are usage
errors, and the flag wins over the variable. A deliberately tiny value shows the
failure mode:

```sh agent-example id=exit-timeout exit=6
mamut-engine --timeout 0.000001 run --input docs/examples/agent-cli/two-storey-frame.json 2>"$TMPDIR/err.json"
rc=$?
python3 -c 'import json,sys; e=json.loads(open(sys.argv[1]).read().splitlines()[-1]); print(e["code"], e["exitCode"])' "$TMPDIR/err.json"
exit $rc
```

```text agent-output id=exit-timeout
CLI-E-TIMEOUT 6
```

Cooperative cancellation: if the file named by `MAMUT_ENGINE_CANCEL_FILE`
exists, the run stops with exit 5 and `CLI-E-CANCELLED`. The CLI never creates
or deletes that file. `SIGINT` and `SIGTERM` also give exit 5.

```sh agent-example id=exit-cancelled exit=5
: > "$TMPDIR/cancel.flag"
MAMUT_ENGINE_CANCEL_FILE="$TMPDIR/cancel.flag" mamut-engine run --input docs/examples/agent-cli/two-storey-frame.json 2>"$TMPDIR/err.json"
rc=$?
python3 -c 'import json,sys; e=json.loads(open(sys.argv[1]).read().splitlines()[-1]); print(e["code"], e["exitCode"])' "$TMPDIR/err.json"
exit $rc
```

```text agent-output id=exit-cancelled
CLI-E-CANCELLED 5
```

### 4.6 Check determinism and schema integrity

Two runs of the same model give the same bytes, so a SHA-256 of stdout is a
cache key or a regression check. The `sha256` in `describe --json` is the digest
of the exact bytes of each published schema; `schema <id>` returns those bytes.

```sh agent-example id=determinism exit=0
mamut-engine run --input docs/examples/agent-cli/two-storey-frame.json > "$TMPDIR/a.json"
mamut-engine run --input docs/examples/agent-cli/two-storey-frame.json > "$TMPDIR/b.json"
cmp "$TMPDIR/a.json" "$TMPDIR/b.json" && echo "run output identical"
mamut-engine describe --json > "$TMPDIR/describe.json"
mamut-engine schema results/1 > "$TMPDIR/results-1.schema.json"
python3 - "$TMPDIR/describe.json" "$TMPDIR/results-1.schema.json" <<'PY'
import hashlib, json, sys
d = json.load(open(sys.argv[1]))
want = next(s["sha256"] for s in d["schemas"] if s["id"] == "results/1")
got = hashlib.sha256(open(sys.argv[2], "rb").read()).hexdigest()
print("results/1 schema matches inventory:", want == got)
PY
```

```text agent-output id=determinism
run output identical
results/1 schema matches inventory: True
```

The raw bytes are only guaranteed identical for the same build target: the
results carry `target` and `git_commit` as provenance
([DETERMINISM.md](DETERMINISM.md)).

## 5. Exit codes and errors

| Exit | `exitCodes` name | `outcome` | Meaning | What the agent should do |
| --- | --- | --- | --- | --- |
| 0 | `ok` | `ok` | Complete result, no warnings | Read `engineResults` |
| 1 | `warning` | `warning` | Complete result with warnings | Read results, review `diagnostics` |
| 2 | `invalid` | `usage`, `invalid_input` | Wrong command or arguments, unreadable or oversized document, rejected model | Fix the command or the model; do not retry unchanged |
| 3 | `unsupported` | `unsupported` | The model asks for an element, law or capability the engine does not have | Change the model; see `capabilities` |
| 4 | `not_converged` | `not_converged` | Singular or unstable system, or an iteration did not converge | Check supports, connectivity and parameters |
| 5 | `cancelled` | `cancelled` | Cancellation marker, `SIGINT` or `SIGTERM` | Not an error of the model |
| 6 | `timeout` | `timeout` | The `--timeout` deadline expired | Raise the deadline or simplify |
| 70 | `internal` | `internal` | Internal failure, or stdout or stderr could not be written (for example the reader closed the pipe) | Report it; do not retry blindly |

The `cli-error/1` fallback codes, all registered and frozen in
[error-codes-1.json](contracts/error-codes-1.json):

| Code | Exit | When |
| --- | --- | --- |
| `CLI-E-USAGE` | 2 | Invalid command or arguments, including an invalid `--timeout` |
| `CLI-E-DOCUMENT` | 2 | The input document is missing, unreadable, too large, not UTF-8 or not JSON |
| `CLI-E-VALIDATION` | 2 | The workflow contract or parameters are invalid, without an engine diagnostic |
| `CLI-E-UNSUPPORTED` | 3 | Unsupported provider or method, without an engine diagnostic |
| `CLI-E-NOT-CONVERGED` | 4 | Not converged or unstable, without an engine diagnostic |
| `CLI-E-CANCELLED` | 5 | Cooperative cancellation |
| `CLI-E-TIMEOUT` | 6 | Deadline expired |
| `CLI-E-METHOD` | 70 | Method failure without a classification |
| `CLI-E-INTERNAL` | 70 | Internal state or output I/O |

When the engine rejects a model, the envelope's `code` is the engine's own
(`ENG-E-NNNN` or `API-E-…`) and `diagnostics[]` has the details. Warnings are
`API-W-…`. Only `ENG-…` and `API-…` codes can be passed to `explain`; asking for
a `CLI-E-…` code is a usage error. The registry has 32 codes with the exit code
of each.

`pointer` in the envelope is currently always `null`. The error registry is
append-only: a published code is never reused or deleted.

## 6. Limits

All are exposed in `describe --json` under `limits` and `inputs`. Exceeding a
limit is an input error (exit 2), not a crash.

| Limit | Value | Applies to |
| --- | --- | --- |
| `MAX_INPUT_BYTES` | 8 388 608 (8 MiB) | The model document; exactly that size is accepted |
| `MAX_DOCUMENT_BYTES` | 16 777 216 (16 MiB) | Catalog and workflow documents |
| `MAX_NODES` | 256 | `model.nodes` |
| `MAX_MEMBERS` | 1024 | `model.members` |
| `MAX_CASES` | 64 | `cases` |
| `MAX_COMBINATIONS` | 128 | `combinations` |
| `MAX_LOADS` | 16 384 | Loads and member loads of all cases |
| `MAX_JSON_DEPTH` | 64 | Nesting of objects and arrays |
| `MAX_JSON_ARRAY_ITEMS` | 16 384 | Any auxiliary array |
| `MAX_TIMEOUT_SECONDS` | 31 536 000 | Largest `--timeout` |
| `MAX_CONSOLIDATION_*` | see `describe --json` | Work, cost, segments and substeps of `consolidation1d` |

The same defect can carry a different code depending on the command: a model
with 257 nodes is `ENG-E-0009` in the `validate` report (parse, structure and
limit errors are normalized to that code) and `API-E-INPUT` in the `run`
envelope. Both mention `MAX_NODES` in `cause` and both exit 2. Decide by exit
code, then by `cause` if you need the detail.

A text label in `units` other than `"SI"` yields `ENG-E-0001` in both
`validate` and `run`, with exit 2 and no results. Missing or non-text `units`
values remain structure errors (`ENG-E-0009` in `validate`, `API-E-INPUT` in `run`).
The cause is `UnknownUnit: Unknown unit system; only "SI" is accepted` and
omits the received label for security. After JSON parsing and the `schema`
check, the unit check precedes structural validation, so an unknown text label
also takes precedence when model fields are missing or malformed.

Use `--timeout` to bound the run time. Experimental analyses
(`nonlinearStatic`, `soilProfile`, `consolidation1d`) are rejected with exit 2
unless the job sets `"options": {"allow_experimental": true}`.

## 7. Pitfalls found while testing

Each row was observed with the real binary; all of them exit 2 with
`CLI-E-USAGE` unless stated.

| Mistake | Correct form |
| --- | --- |
| `--input=model.json` | `--input model.json` (a space; `--flag=value` is rejected) |
| `run -- --input model.json` | The `--` separator is rejected anywhere |
| `--input a.json --input b.json` | Repeated flags are rejected |
| `--input - --catalog -` | Only one stdin source per invocation |
| `run` without `--input` and without `MAMUT_ENGINE_INPUT` | Exit 2 with `CLI-E-DOCUMENT`: there is no model to read |
| `--timeout 0` or `--timeout -1` | The value must satisfy 0 < s ≤ 31 536 000 |
| `schema cli-run/1`, `explain CLI-E-USAGE` | Only API schemas and ENG/API codes are available |
| `explain --json ENG-E-0006` | `explain ENG-E-0006 --json` |
| Reading `validate` errors from stderr | A rejected model is reported on **stdout** by `validate`, but on **stderr** by `run` |
| `mamut-engine run … \| head -c 10` | If the reader closes the pipe before the write, the CLI exits 70 with `CLI-E-INTERNAL` (timing dependent); read all of stdout |
| `displacements[1]` for a horizontal displacement | Order is `[Ux, Uy, Uz, Rx, Ry, Rz]`: `Ux` is index 0 |
| Treating `data` as results | `data` holds fingerprints; the values are in `engineResults` |

`MAMUT_ENGINE_INPUT=<path>` still works as a compatibility alternative to
`--input`, but `--input` takes precedence and is the documented way.

## 8. How this guide is verified

* `cargo test -p engine-cli --test agent_guide` extracts the blocks of this page
  and runs the real binary. A block marked `agent-example` is executed with `sh`
  (from the repository root, with `$TMPDIR` set to a private directory); the
  test fails if its exit code differs from `exit=` or if its stdout differs from
  the following `agent-output` block (`<VERSION>` stands for the engine version).
  It also checks that relative links resolve, that the exit-code and error-code
  tables agree with `describe --json` and the registry, and that the example
  model validates and runs.
* The acceptance criterion, "an agent without context completes validate, run
  and reading the roof displacement using only `describe --json` and this
  guide", was exercised as a usability test with one agent session that had seen the model JSON and the case name;
  the record is in [provenance/ENG-DOC-001.md](../provenance/ENG-DOC-001.md).
