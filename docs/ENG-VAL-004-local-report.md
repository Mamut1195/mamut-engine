# ENG-VAL-004: provisional local report

The coordinator ran the campaign on a dirty checkout at base commit
`c64d9977d62f539a3a61e69a078bf4d8bc5526df`. The host was Windows 11
Home 10.0.26200 64-bit, Ryzen 9 5900HX (8 cores, 16 logical processors),
with 33737932800 bytes physical RAM, PowerShell 5.1.26100.9711,
Node v22.19.0 and rustc 1.99.0 (b940084d7). The release binary SHA-256
was `18d905f3f1dc052906227b70f8339dd8e38d604c1bf0ffe066273ce873df0a40`;
the WASM SHA-256 was
`610d5e5f87bacb46f5b6ff051e8261d0b0ad6c41f56ad7214996dc64e1cad7ba`.

There was one measured repetition per case, without warmup or isolation.
These observations are not a benchmark distribution. Native and WASM
campaigns ran sequentially except for an initial extra smoke run overlapping
an active native campaign; the WASM table below is from the separate batch.
Ordinary host contention was possible; host load was not controlled.

| Nodes | Members | Analysis | Native elapsed ms | Sampled peak bytes | Result |
| ---: | ---: | --- | ---: | ---: | --- |
| 64 | 120 | static | 295.2058 | 24522752 | warning |
| 64 | 120 | modal | 463.7857 | 12124160 | warning |
| 64 | 120 | spectrum | 509.3451 | 15319040 | warning |
| 144 | 320 | static | 634.8552 | 52555776 | warning |
| 144 | 320 | modal | 25045.9436 | 26779648 | warning |
| 144 | 320 | spectrum | 19352.9376 | 31150080 | warning |
| 256 | 600 | static | — | — | failed ENG-E-0010 |
| 256 | 600 | modal | — | — | timed out at 60 s |
| 256 | 600 | spectrum | — | — | timed out at 60 s |

The 256-node static failure reported a relative recovery residual of
1.1563264103376117e-9 exceeding 1e-9; no reference or tolerance was changed.
The native measurements above used an earlier script version sampling
PeakWorkingSet64 every 10 ms. The current script samples WorkingSet64 every
10 ms, so its measurement is not exactly the same. Sampling can miss peaks;
the sampled threshold is not a hard cap.

| Nodes | Members | Analysis | WASM call ms | RSS before bytes | RSS after bytes | Result |
| ---: | ---: | --- | ---: | ---: | ---: | --- |
| 64 | 120 | static | 136.6545 | 53043200 | 65871872 | parity true, warning |
| 64 | 120 | modal | 2189.2258 | 52920320 | 60309504 | parity true, warning |
| 64 | 120 | spectrum | 1598.5393 | 53129216 | 63438848 | parity true, warning |
| 144 | 320 | static | 502.4412 | 53313536 | 84217856 | parity true, warning |
| 144 | 320 | modal | — | — | — | external timeout at 60 s |
| 144 | 320 | spectrum | — | — | — | external timeout at 60 s |
| 256 | 600 | static | — | — | — | native failed before WASM execution |
| 256 | 600 | modal | — | — | — | not run |
| 256 | 600 | spectrum | — | — | — | not run |

Parity is equality of parsed JavaScript values after normalization of the
provenance target only. JavaScript number parsing applies; this is a
structural comparison, not a byte comparison. RSS is the WASM worker's
before/after RSS, not a peak; exported WASM linear memory was unavailable.
Native time includes process startup, input transport and output capture;
WASM time covers the API call in the worker. Their boundaries differ. The
WASM runner has 120-second child timeouts and no full RSS cap; the actual
campaign used an external 60-second operational limit.

After script changes, a small run succeeded with warning and valid stdout
JSON. A one-second timeout test and a one-byte sampled-threshold test each
returned rc 1. The existing native/WASM determinism script ran release for
16 cases with ten repetitions and returned rc 0. Its first Node attempt
returned rc 1 because PowerShell redirection produced UTF-16; byte redirection
through cmd corrected this and the rerun returned rc 0. Its hashes were equal
after target-only policy normalization.

Approvals, hardware reference and maximum supported size remain PENDING.
This report is provisional, not final validation. Temporary logs may be
transient and have no publication guarantee.

The source for the candidate was base commit
`c64d9977d62f539a3a61e69a078bf4d8bc5526df` with a dirty, uncommitted six-file candidate.
The original native table runner version was not preserved; its binding is
PROVISIONAL and no runner hash is available for that table.

The four final2 measurements used the pre-P2 native script, preserved in
evidence as `eng-val-004-native.ps1`, with SHA-256
`3fbeb14b4676e326e988a0f567e1a2a91b6c1a3e2670e39d26637cdac5bcfa5e`.
The corrected current native script, preserved as `eng-val-004-native-p2.ps1`,
has SHA-256
`bc2cf9e33cef5789d27d9408bdfde7bb71c06871ae54edaaae6f255b0113cb2e`
and was not used for the final2 campaign measurements.
The final2 WASM script was pre-P2, preserved as `eng-val-004-wasm.cjs`,
with SHA-256
`c2a233505cb053ab00382f8b575b9810b8e417ce2c74e15aeae3df1f04430e3c`.
The corrected current WASM script, preserved as `eng-val-004-wasm-p2.cjs`,
has SHA-256
`6410e27ff03fb9bfa79c446b421caf548144896011271cb61486f55cf41fd3fb`
and was not used for the final2 campaign measurements.
The wasm-bindgen generated Node loader `engine_wasm.js` SHA-256 was
`acf0bef49b2bb2fc08f741fa39ec8727d2840a4e1e5f276aafcd00b5552aa597`.
The native executable and WASM module hashes are recorded above.

The final rerun repeated the four successful cases with the pre-P2 native
and WASM scripts, loader and binaries, sequentially, with one repetition
each and no isolation. The native timeout was 60 seconds and its sampled
working-set threshold was 1 GiB; that threshold was not a hard process memory
cap. The WASM runner timeout was 120 seconds, with no hard process memory cap.

| Nodes | Members | Analysis | Native elapsed ms | Sampled working-set peak bytes | WASM call ms | WASM RSS before bytes | WASM RSS after bytes |
| ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 64 | 120 | static | 180.4987 | 17764352 | 85.1915 | 53866496 | 66596864 |
| 64 | 120 | modal | 509.5932 | 11386880 | 1520.7272 | 53927936 | 60567552 |
| 64 | 120 | spectrum | 490.9484 | 16453632 | 1391.1583 | 53891072 | 63614976 |
| 144 | 320 | static | 447.0801 | 49561600 | 386.4022 | 54112256 | 84738048 |

All four native and WASM runs returned rc 0 and status warning. Comparison
used parsed JavaScript values after normalization of the provenance target only.
Native elapsed time includes process startup, input transport and output capture.
WASM call time covers the API call in the worker; the boundaries differ.
RSS values are worker before/after measurements, not peaks; WASM linear memory
was unavailable. Native sampling can miss peaks. JSON comparison is limited by
JavaScript numeric precision.

The first final-rerun attempt invoked a native script copied into evidence
outside the checkout. All four native runs returned rc 1 because the relative
fixture index was unavailable. Those raw files and the successful WASM outputs
were retained. A second attempt ran the pre-P2 native script in the original
checkout; all cases returned rc 0.

The native P2 regression tests passed six synthetic cases with rc 0. They accept
exit/outcome/status `0/ok/ok` and `1/warning/warning`, and reject
`1/ok/ok`, `0/warning/warning`, `1/warning/ok` and `0/ok/warning`.
The reviewer probe was rejected with rc 1 as expected. The real native Engine
64-node static smoke returned rc 0 and warning/1. Details are in
`p2-regression.stdout.txt` and `p2-gates.txt`; raw tests are also in
`eng-val-004-regression-773f4e5932f643deac4fe22a7a95ec3c`, with smoke output
in `p2-engine-smoke.stdout.json` and probe stderr in
`p2-review-probe.stderr.txt`.

The corrected WASM comparison normalizes only the root `/target` in
`results/1`, preserving nested targets, and rejects nonfinite numbers in both
parsed results before comparison. It checks the WASM target exactly and checks
the native target for plausible non-WASM target syntax, without matching it
against the actual native build target. The Node regression returned rc 0 with
31 passed; evidence is `wasm-p2-regression.stdout.txt` and
`wasm-p2-regression.cjs`. A real WASM plus CLI 64-node static smoke returned
rc 0, equality true and warning; raw output is
`p2-wasm-engine-smoke.stdout.json`.

Evidence is local to `C:/Users/Jonnathan/Projects/mamut-engine-execution-20261103-091746/mam107-evidence`;
it includes `final2-*` raw outputs, `final2-exit-codes.txt` and `SHA256SUMS.txt`.
This path may be unavailable on other machines.

Earlier external timeout monitor console messages were not saved; an
empty timeout output alone does not prove a timeout.
The existing determinism Node stdout is now captured in
`determinism-node.stdout.txt`, with exit rc 0. The reviewer independently
reports determinism PASS for 16 cases and 160 hashes; the coordinator did not
observe that independent run.
No maximum supported size or performance budgets have been approved.
