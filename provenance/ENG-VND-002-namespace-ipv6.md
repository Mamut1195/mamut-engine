# MAM-938 IPv6 namespace gate regression

Date: 2026-11-03.
Author attribution supplied by coordinator: Sol / gpt-6.1-sol / high.
Source HEAD before fix, supplied by coordinator: 694f00b8220ae85f03973970361869341e593fa3; base label: originalc64.

## Original failure

ROOT reported integrated candidate 5a172 entered a real kernel network namespace with UID 1000, all capability sets zero, and only loopback, DOWN. The manual inner namespace gate failed with the exact message `IPv6 default route present`, before Cargo. All later gates (toolchain, vendor, tamper, features, fmt, clippy, test, doc, wasm) were NOT RUN. This does not establish a Cargo regression.

Original user-supplied references remain `execution/mam938-namespace-validation.log` and `namespace-inner-receipt`. Their exact absolute root paths were not supplied or inspected by this author.

## Authored change and sources

Implementation sources used: the coordinator-provided shell source and route-format/flag contract only. No external kernel headers were read. No tools, filesystem reads, shell, Cargo, browser, network, vendor sources, prohibited engine sources, or oracle outputs were used by this author. Validation information below consists solely of coordinator-supplied execution reports; this author did not inspect the artifacts.

The shell delegates its namespace proof to an import-safe stdlib module. Every nonblank IPv6 record receives strict ten-field validation; blank lines are ignored and malformed records fail closed without record contents in errors. Under supplied constants RTF_UP=0x1 and RTF_REJECT=0x200, a destination default is rejected when UP and not REJECT, regardless of source prefix. REJECT defaults and defaults without UP are accepted as unusable. Other namespace checks, including the unchanged IPv4 check and exact numeric TCP ENETUNREACH requirement, retain their behavior.

Tests use synthetic route metadata only and do not execute or mock the namespace proof. The coordinator reports applying the exact four-file patch and proving all shell text outside `verify_namespace` unchanged. Scope remains bounded to the four authorized paths, with no Cargo or vendor changes.

## Coordinator-executed validation

External evidence bundle supplied: `MAM-938-pr-b-t3-78d38825/`. References below are supplied artifact names, not independently inspected absolute paths.

- Original failure reproduced before patch through `wsl -d Ubuntu -u root -- bash <external>/ipv6-namespace-fix/run-original-proof.sh`, using the original embedded proof extracted verbatim. Label: `PROOF_ONLY_ORIGINAL_NO_CARGO`. The actual kernel table contained two all-zero destination/prefix-zero sentinel records with flags `00200200` on `lo`. The proof failed with `IPv6 default route present`, child exit 1; the wrapper expected that failure. Evidence: `ipv6-namespace-fix/original-namespace-proof.log` and companion `.exit` recording 1. No Cargo ran.
- Windows Python 3.13: `python -I -m unittest discover -s scripts/tests -p test_offline_namespace.py -v` exited 0; coordinator reported 16 tests plus subtests, OK. Evidence: `unit-windows.log`.
- Ubuntu WSL Python 3.12: the same unittest invocation using an absolute `scripts/tests` path exited 0; coordinator reported 16 tests plus subtests, OK. Evidence: `unit-ubuntu.log`.
- Ubuntu `bash -n scripts/check-offline-linux.sh` exited 0. Git diff whitespace checking restricted to the four authorized paths exited 0.
- Patched real-kernel proof exited 0 under label `PROOF_ONLY_PATCHED_NO_CARGO`. Evidence: `ipv6-namespace-fix/patched-namespace-proof.log`, companion `.exit` recording 0, and runner `run-patched-proof.sh`. The module completed its assertions and logged `PASS kernel network namespace` and `TCP errno=ENETUNREACH`.

Both real-kernel proofs used explicit root-authorized WSL root execution to launch `unshare --net`, followed by `setpriv` to UID 1000/GID 1000, cleared supplementary groups, all five capability sets zero, NoNewPrivs=1, and an `env -i` minimal environment. The patched proof verified a different kernel network namespace, only loopback DOWN, no IPv4 default route, no usable IPv6 default route, and exact ENETUNREACH from numeric TCP. Actual fallback flags `00200200` were accepted. Namespace inode identifiers are run metadata, not durable expected values.

## Remaining status

Full Cargo suite, full outer runner requiring passwordless sudo, public GitHub CI, and CF: NOT RUN. The standalone namespace proof is proof-only and does not establish full E2E PASS.

WSL normal-user `sudo -n` required a password. No password was requested and no policy was changed. The explicitly authorized root launch retained the dropped-process controls described above.

Overall status remains PROVISIONAL pending ROOT reruns of the exact integrated commit after porting. The forthcoming independent fix commit SHA belongs in `HANDOFF.json` after commit; no SHA is invented here. No release or ticket completion is asserted.
