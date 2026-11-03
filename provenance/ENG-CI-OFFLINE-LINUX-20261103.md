# Clean-room log · ENG-CI-OFFLINE-LINUX-20261103

- Author: Codex, gpt-6.1-sol/high; 2026-11-03 (America/Guayaquil).
- Role: sole offline CI patch author; user expressly authorized read-only
  authorship, with literal application and validation by the coordinator.
- Supplied checkout: Mamut1195/mamut-engine, branch
  `t3/public-ci-runner-migration`, clean baseline
  `d815b55bf6588598580e6aaffa03b7aed5f33f6c`.
- Launcher configuration supplied by the user:
  `codex exec --ignore-user-config -c web_search=disabled -c sandbox_workspace_write.network_access=false --disable apps -s read-only -m gpt-6.1-sol -c model_reasoning_effort=high -C <sameworktree> --json --output-last-message <executiondir-author.patch> -`,
  with UTF-8 prompt stdin. No tools or commands were invoked by this author.

The authorized deliverable changes only `.github/workflows/rust.yml` and this
log. It replaces the five direct Cargo gates with one script invocation after
toolchain installation and Changelog, before the existing bounded cleanup.
Setup, provisioning, metadata checks, triggers, permissions, concurrency, job
identity, runner, timeout, checkout pin and credential handling are preserved.
No action, upload, cache, release, publication or deployment is added.
`clean-room.yml`, protected-review and owner own-PR gates/exceptions are unchanged.

Permitted sources were only the user-supplied checkout contents
(`rust.yml`, `docs/CLEAN_ROOM.md`, `provenance/TEMPLATE.md`) and the expressly
authorized same-project local script snapshot:
`C:\Users\Jonnathan\.t3\worktrees\mamut-engine\t3-78d38825\scripts\check-offline-linux.sh`.
Its supplied SHA-256 is
`336b6514183c3a6087240b29a0b63077c969134eb7aa823eac3c16f5d4c1e445`.
This records inspection of supplied text, not execution, an independently
computed hash or a workflow compatibility pin. No other sibling files or
history were inspected.

Only the Cargo/test gate phase is isolated, not the entire CI job.
Installing Rust 1.99.0 and its components/target needs network access;
Changelog metadata, provisioning version commands and launcher bootstrap
checks precede the namespace and do not establish isolated gate success.
The script also performs a separate toolchain gate inside the namespace.

The inspected launcher requires a nonroot outer user, uses `sudo unshare --net`,
then `setpriv` to drop capabilities, clear supplementary groups and enforce
no-new-privileges. It launches with `env -i` and fresh HOME, CARGO_HOME, TMPDIR
and target paths. Standard hosted Ubuntu 24.04 passwordless sudo is a
prerequisite. A normal local WSL user requires a sudo password; root validation
of the inner phase is separate. This patch changes no sudo configuration or
privileges and adds no workflow sudo command.

The namespace gate delegates to `scripts/check-offline-namespace.py`; required
runtime evidence covers the kernel network namespace, interfaces, routes and
TCP ENETUNREACH probes. That helper was not supplied or independently inspected
here. The supplied script orders namespace/toolchain checks, vendor validation,
tamper tests and feature inspection before fmt, clippy, test, doc and wasm.
Its resolving/build gates use `--frozen`. Formatting runs
`cargo fmt --all -- --check` under `CARGO_NET_OFFLINE=true` inside the network
namespace; the fmt command itself has no `--frozen` flag.

Evidence goes beneath `$CI_JOB_TEMP/offline-evidence` and is erased by the
existing audited final cleanup. On an actual run, tee output remains in GitHub
job logs; no artifact upload or cache is added.

The script is intentionally absent from this checkout. Root must compose
B/vendor/hardening and the script before enabling Actions, which remain
disabled pending integration, source review and final review. The script may
receive the parent-authorized IPv6 parser correction before integration;
this author neither changes nor reviews that correction. The invocation
contract remains one absolute evidence-directory argument.

Validation at author handoff: patch not applied; all execution NOT RUN.
Coordinator validation is planned/PENDING: actionlint 1.7.12, Bash syntax checks
of extracted run blocks and the supplied script via stdin, semantic comparison
of the five-step replacement and preserved workflow fields/cleanup, unchanged
protected-review content, `git diff --check` and a two-path diff boundary.
Actual results belong in the coordinator's outside-checkout handoff after
commit. Main integration, full runtime, kernel namespace execution, Cargo gates
and hosted CI were NOT RUN here. Separate root inner tests and prior CI evidence
are not PASS evidence for this wiring. Independent review is PENDING.
The root-managed port of the d815 docs-public snapshot remains pending;
this log assigns it no integrated public SHA.

Third-party code copied: none. Own numerical derivations: not applicable.
No numerical, fixture, tolerance, schema, license, version or mapping changes.
No changelog fragment is required for this workflow/provenance-only scope.
No network, web, apps, MCP, external documentation, prohibited engine code,
oracle reports or oracle outputs were consulted.

Declaration: "No consulté código fuente de OpenSees, Frame3DD, Stabileo ni
derivados para este ticket."

Clean-room reviewer: PENDING.
