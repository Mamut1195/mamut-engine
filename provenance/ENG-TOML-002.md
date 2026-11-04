# Clean-room log · ENG-TOML-002

- Ticket / PR: MAM-940, MAM-941, and MAM-943. `mamut-engine convert` for mamut-toml/1.
- Author(s): implementation agent, Grok 4.7, 2026-11-03 and 2026-11-04 (America/Guayaquil). Provenance note added on 2026-11-04.
- Role: implementer.
- Contract implemented: mamut-toml/1 as the TOML representation of engine-input/1, following docs/formats/mamut-toml-1.md and ADR-006. Schemas were not changed.
- Permitted sources used: the repository specification, ADR-006, provenance/ENG-TOML-001.md, and crates/engine-api/schemas/engine-input-1.json. No book page was required for this mapping.
- Own derivations: none. The converter applies the mapping already written in the specification.
- Third-party code copied: none.
- Validation: `cargo test --offline --locked -p engine-cli` reported exit 0 by the implementing session (124 tests on the analysis commit). This note does not repeat fixture numbers.
- Comparison with oracle (sibling repo, informative): no.
- Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4).

## Decisions and evidence

The structural core, the load and case collections, and the specified analysis branches are converted in both directions. `spring.nonlinear` is accepted only with `nonlinearStatic`. `nonlinearStatic`, `soilProfile`, and `consolidation1d` require `allow_experimental`. Soil and consolidation require an empty structural model and do not fill missing geotechnical values.

U5, U7, U9, U10, and U11 stay open. A support `type` is still rejected. Diagnostics do not carry line or column.

The converter mapping is unchanged by the clippy cleanup on 2026-11-04: `json_to_toml` returns the export result directly, the assembly arguments travel in one struct, section records use a named tuple, and JSON detection trims whitespace with a character array. The convert test joins `temp_dir()` directly. No numerical result changed.

## Pending work

J4 review of this log. The five open points above remain Jonathan's decisions.
