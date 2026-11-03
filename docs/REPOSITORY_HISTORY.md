# Repository history and migration

Snapshot dated **2026-11-03**, based on the parent coordinator's verification.
This offline documentation followup did not query GitHub or check external links.

- **Original repository:** `Mamut1195/mamut-engine-private-archive-20261103`,
  GitHub repository ID `1412637886`, verified **PRIVATE** by the parent.
  Renamed from `Mamut1195/mamut-engine`, it preserves all original PRs,
  issues, logs and 84 branches.
- **New repository:** `Mamut1195/mamut-engine`, GitHub repository ID
  `1444444923`. It reuses the original address but is a different repository.
  The parent reports sanitized candidate history pushed to its `main`;
  it currently remains **PRIVATE**, with Actions disabled and 0 runners.
  **PUBLIC publication remains pending independent review.**

The migration strategy preserves the full original repository privately
and sanitizes copied history for publication. Archive access remains
subject to existing private permissions.

PR and issue numbers and commit SHAs from before the migration on
2026-11-03 must be looked up in the private archive. Reuse of the original
URL does not transfer historical PR or issue numbering to the new repository.
For example, historical
[PR #254](https://github.com/Mamut1195/mamut-engine-private-archive-20261103/pull/254)
belongs to the private archive. Use these archive URL templates:

- PRs: `https://github.com/Mamut1195/mamut-engine-private-archive-20261103/pull/<number>`
- Issues: `https://github.com/Mamut1195/mamut-engine-private-archive-20261103/issues/<number>`

No old-to-new commit map has been provided; do not infer SHA mappings.
The root coordinator may provide a machine-readable map later.

Branches or history from old clones **must not be merged or pushed into
the sanitized repository or future public history**, because this can
reintroduce removed blobs. Start from a fresh checkout of the new repository
or a clean base in its sanitized history. Transfer changes through controlled
clean patches or cherry-picks, with the root coordinator's review of the
resulting history; do not merge wholesale old history.
