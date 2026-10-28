# Clean-room log · ENG-VAL-006

- Ticket / PR: ENG-VAL-006, Engine side of the performance comparison. The translator
  to the other engine and any files from that engine remain outside this
  repository, in the sibling repo, under Q2's responsibility.
- Author(s): Cursor Grok 4.7 · 2026-10-20 and 2026-10-21 (America/Guayaquil).
- Role: specifier and implementer of the schema, generator, translator
  to `engine-input/1`, index and Engine measurement orchestrator.
- Contract implemented: `bench-model/1` and `bench-index/1` (JSON Schema
  draft 2020-12). Engine input remains `engine-input/1`.
- Permitted sources used:
  1. ACI 318-19, §19.2.2.1.a: E = 4700·√(f'c) MPa, with f'c in MPa. Here
     f'c = 28 MPa. ν = 0.20 and G = E / (2 (1 + ν)) are constants of this
     model, not an article of the code.
  2. NEC-SE-DS 2015, elastic spectrum shape in g: T0 = 0.10·Fs·Fd/Fa,
     Tc = 0.55·Fs·Fd/Fa, TL = 2.4·Fd; 0..T0 branch linear up to the plateau
     η·Z·Fa; T0..Tc plateau; Tc..TL meseta·(Tc/T)^r; T > TL
     meseta·(Tc/TL)^r·(TL/T)². Coefficients saved, not recalculated by
     the engine: zone V, Z = 0.40, soil D, Fa = 1.20, Fd = 1.19, Fs = 1.02,
     η = 1.80 (Sierra), R = 8, r = 1, ζ = 0.05. Published Sa =
     (elastic ordinate in g) / R · g, with g = 9.80665 m/s². Neither φP
     nor φE is applied. Both engines must read the versioned table; if a
     coefficient from a later edition differs, the table in this commit
     remains the common one.
  3. Saint-Venant approximation for a rectangle:
     J = a·b³·(1/3 − 0.21·(b/a)·(1 − (b/a)⁴/12)), with a the longer side.
     Implemented in `saint_venant_j`. Monograph page not reopened
     in this session.
  4. R. J. Hyndman y Y. Fan, «Sample quantiles in statistical packages»,
     The American Statistician 50(4), 1996: type 6 quantile, h = (n + 1)·p.
     Median, Q1, Q3, IQR, minimum and maximum come only from
     `phase=measure` rows.
- Own derivations: nodal distribution of `{m, Izz}` documented in the
  schema (master Ux = Uy = m, Rz = Izz; Uz and the Rx, Ry inertias are
  distributed so that the mass matrix is positive definite in the free
  diaphragm DOFs). Izz of the uniform slab about the master,
  not the centroid: m·(Lx² + Ly²)/12 + m·d². Lateral load = 0.10·m·g
  along the spectrum axis, constant coefficient of this model.
- Third-party code copied: none. No new crates in the license
  review: `engine-bench` uses `engine-api`, `engine-core`, `mamut-canon`,
  `serde` 1.0.229 and `serde_json` 1.0.154, already present in the workspace.
  The generator's `sqrt`, `powi`, `powf` and `hypot` go through `engine_core::math`.
- Validation: `cargo test -p engine-bench` (17 Rust tests; one of them
  runs the 7 Python tests in `scripts/eng-val-006/test_orchestrator.py`).
  The 3-story frame test runs static, modal and spectrum and
  reads period, static roof Ux and CQC Ux through the index pointers.
  The 8- and 15-story frames are validated against `engine-input/1` without
  solving the dense modal problem in the test. `cargo clippy -p engine-bench
  --all-targets -- -D warnings` and `cargo check -p engine-bench --target
  wasm32-unknown-unknown` passed in this session. The orchestrator was not
  run against the VPS. A minimal local static `rc3d-3s` pass
  (one warmup and one measurement, one thread) wrote a manifest, CSV,
  table and SVG under `target/`, which is not versioned. The exit code
  was 1, the diaphragm warning that the protocol accepts. This image does not
  include `/usr/bin/time`; the pass used GNU time 1.9 through `--time-bin` and
  the orchestrator default remains `/usr/bin/time`.
  No timings are published.
- Comparison with oracle (sibling repo, informative): no. That repository
  was not opened and no file in its language was generated.
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, Frame3DD, LAPACK ni derivados para este ticket."
- Clean-room reviewer: J4 · 2026-10-21 (America/Guayaquil) · SUITABLE on 6068f7c90c64f50877132481d34105ad980ffe74. Declared sources sufficient; no oracle code (OpenSees, CalculiX, Frame3DD, LAPACK) or third-party code; the description of the scalar guard (option A, math::powi = libm::pow, ±1 ulp in beam Iz and J, 12 new sha256) matches diff 3567d8e..6068f7c. The merge requires Jonathan's explicit yes.

## Scope

`crates/engine-bench` publishes the JSON Schema `schemas/bench-model-1.json`
(`$id` `urn:mamut:bench-model:1`) and three deterministic frames:

| id | nodes | members | bench-model sha256 |
| --- | ---: | ---: | --- |
| rc3d-3s | 64 | 120 | `199cfb84d248877d70caa6e1df641582386a0dabfe725d8ec659007d4552d7b3` |
| rc3d-8s | 144 | 320 | `33fa7da8e4af10edf89adf4d19737eab72391618c297a5952cfb059cf5efe960` |
| rc3d-15s | 256 | 600 | `b914e418b36a9ce8085ad31f7caecf1d512946215577ea961ccb0071e998a5b9` |

The `engine-input/1` limit is 256 nodes and 1024 members. The 15-story model
uses the maximum number of nodes. One more story would exceed it and the translator rejects it.

`orientation.kind` is `local_z_vector`: global reference vector for the
local z axis. There is no angle. Columns `[1, 0, 0]`; beams `[0, 0, 1]`.
The spectral direction is a global axis; in the official models, X = `[1, 0, 0]`.
Story mass is `mass {m, Izz}` in kg and kg·m². `Sa.unit` is `m/s^2`.

The `fixtures/index.json` index (`bench-index/1`) lists `models[]` with
`id`, path and sha256 of the bench-model, `engine_input.{static,modal,spectrum}`
with path and sha256 of the already translated input, `roof_node` and
JSON pointers for period, static roof displacement and CQC
roof displacement. The sha256 is of the file's UTF-8 bytes with a final LF and no
BOM, in lowercase hexadecimal. It is not `results/1.input_sha256`. Paths
use `/`. `.gitattributes` enforces LF in those JSON so that a Windows checkout
does not change the hash.

`scripts/eng-val-006/orchestrate.py` prepares the release binary campaign
(3 warmups, 15 measurements, 1 thread and N threads, `taskset`,
BLAS variables, `/usr/bin/time -v`, idle runners before and after
each block). Tables and SVG come only from the CSV. There is no workflow
for this campaign in `.github`.

## Additive extension

After J2's comparison in VAL-032 (PR #180, `8055c02`), `bench-model/1`
accepts, without touching the bytes of the `rc3d-*`:

- `nodal_masses`, which replaces the tributary distribution when the key is present.
- `modal.n` in 1..=16384 and modal combination `cqc` or `srss`.
- Multiple spectral directions, their factors, and `spectrum.variants`.
- `p_delta` with the six `engine-input/1` options. The method exists and
  is stable; no tolerance or iterations are invented.
- Named patterns and `static_combinations` with signed factors.
- Empty diaphragm, zero mass and `truss` members (`truss3d.linear`, only E, A and rho).

The ids remain contiguous from 1: a VAL-032 remapping needs that
bijection. Since ENG-LA-009, modal analysis condenses exactly zero mass rows
with Guyan; the translator still adds no mass. `ENG-E-0006` remains
for negative mass or absence of free mass, and `ENG-E-0014` for a singular `Kss`.
A diaphragm slave does not allow a support (`ENG-E-0013`).

## Scalar guard

J4 approved option (A): `generate.rs` calls `engine_core::math::{sqrt, powi, powf, hypot}`
and `engine-bench` is not excluded from the guard. No other implementation was added.
`math::powi` is `libm::pow`. In the beam, `0.30.powi(3)` goes from
`0x3f9ba5e353f7ced9` (std) to `0x3f9ba5e353f7ced8` (libm), and `(0.30/0.70).powi(4)`
from `0x3fa145d66ad5e25e` to `0x3fa145d66ad5e25d`. The beam JSON changes
`Iz` from `0.001575` to `0.0015749999999999998` and `J` from `0.004603782069970845`
to `0.004603782069970844`. The rest of the official models, including the master
`(1, 1)` chosen with `math::hypot`, remain the same. The twelve files and
`fixtures/index.json` were regenerated with those hashes. `ORIENTATION_KIND`
remains `pub(crate)`. The orchestrator still retains stdout in memory.
