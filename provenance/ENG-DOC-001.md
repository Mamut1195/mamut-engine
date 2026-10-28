# ENG-DOC-001 · MAM-870 · «CLI for agents» guide and executable examples

Date: **2026-10-23**. Local work without network, branch `j2/MAM-870-guia-cli-agentes`
created from `origin/main` (`00189ad`, which includes #205, #208, #210, #217 and #218).

## Scope

- [GUIA_CLI_AGENTES.md](../docs/GUIA_CLI_AGENTES.md): pipeline recipe,
  exit code table, error handling and limits, written only with what
  exists in `main`. Does not document `--lang` or the #219 i18n catalog (MAM-836,
  open): the ticket is not considered dependent on it. If #219 merges, the guide
  and its test remain valid except for text changes in `describe --json`, which the
  test compares as a subset.
- [two-storey-frame.json](../docs/examples/agent-cli/two-storey-frame.json):
  two-storey D2 plane frame (SI, 6 nodes, 6 bars, case `wind`) built
  for this ticket; no external data.
- [agent_guide.rs](../crates/engine-cli/tests/agent_guide.rs): five tests.
- A link from the engine-cli README («Agent discovery and API
  queries» section). `src/`, contracts, schemas and goldens are untouched.

## What is verified in CI

`cargo test -p engine-cli --test agent_guide` does the following with the real
binary:

1. Executes the guide's 17 `agent-example` blocks in order, with a single
   shared `TMPDIR`, and compares exact exit code and stdout with the
   `agent-output` block of the same id. Uses only `sh`, POSIX utilities and `python3`.
2. Checks that relative links and internal anchors resolve.
3. Compares the exit code table with `exitCodes` from `describe --json` and
   the `CLI-E-*` code table with `docs/contracts/error-codes-1.json`, in
   both directions.
4. Compares the limits table with `limits` from `describe --json`.
5. Runs the example model and compares roof Ux displacement
   (node 5, case `wind`) with the shear-building hand estimate
   (k = 2·12·E·I/h³ per storey; Ux ≈ 5,5827e-3 m) with a tolerance of 1 %. The engine
   gives 5,610851e-3 m (0,5 % difference).

Example output is deterministic (same bytes with the same input,
version and target). The timeout example uses `--timeout 0.000001` and gave exit 6
in 100 of 100 local runs; the cancellation example uses
`MAMUT_ENGINE_CANCEL_FILE` with a preexisting file. SIGINT/SIGTERM
behavior (exit 5, `CLI-E-CANCELLED`) was checked manually with a heavy
model and is not part of the tests because it depends on timing. The same applies
to early pipe closure (exit 70), which the guide documents as
timing-dependent.

## Recorded usability test

Ticket criterion: an agent without context completes validate → run → read
the roof displacement using only `describe --json` and the guide.

- Agent: a new Codex session (gpt-6.1-sol, high effort, without network) in a
  separate working directory containing only the binary, guide and
  example model; no access to source code, README or this record.
- Result: validate → exit 0 (`status: ok`); run → exit 0; read
  `engineResults[0]` → case `wind` → node 5 → `displacements[0]`:
  **+5,610850782776847e-3 m**, equal to the guide's value. Reported friction:
  none. Finished in less than a minute.
- Deviations and limits: the agent also printed the model JSON (it is the
  task input) and the prompt gave it the model path and case
  name. This is a single session; it does not replace a test with people or other
  agents, and repeating it is advisable if the guide changes substantially.

## Review notes

- Axis convention: the engine sets the D2 plane as X–Y but does not declare Y as
  vertical; the guide warns of this and the model uses Y as elevation by the example's
  convention (see CONVENCION_EJES).
- Possible conflict with #219: both PRs touch `crates/engine-cli/README.md`. This
  PR adds only three lines under «Agent discovery and API queries»; the
  observed #219 hunks are elsewhere, so no text conflict is expected,
  but review is required after either one merges.
- Change record: `changelog.d/MAM-870.added.md`.
