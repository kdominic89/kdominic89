# Consumer and deployment contents

This repository contains personal configuration, the public organization import declaration,
consumer workflows/tests, authored documentation, the managed root README, and intentional public
text artifacts. Captured manifests/observations, resolved input records, generated ownership,
layout, current state, and rolling history support reproducible output. The generator's Rust
workspace, implementation tests, runtime sources, and release/build scripts belong upstream.

The authenticated `sourcefield.lock.json` selects immutable native and browser archive identities
and digests. One complete shared installation can serve multiple consumers. Tooling must come
from that lock's source revision. See [Setup](SETUP.md) for the normal installation path and
[Validation](VALIDATION-REPORT.md) for this candidate's current release status.

## Generated candidate

Candidate preparation reads current tracked consumer files and preserves independent commit/origin
provenance while generating. New untracked inputs need separate review. Private Git metadata is
removed from the candidate, including on generation failure; credentials and Git objects are not
deployment content. The personal consumer selects only `README.md`.

Generated ownership distinguishes generated files from selected authored README updates. It limits
replacement and pruning; omission of a README does not authorize deletion. Captured public inputs,
generation records, ownership inventory, SVGs, state, and retained history remain intentional tracked
text. Preview/replay does not create live archive evidence.

The wrapper's explicit observation path is `assets/source-snapshot.json`. Offline generation and
locked replay retain its existing bytes, date, source statuses, and approved observations. The
separate `assets/render-snapshot.json` stores effective policy-filtered input; an offline Preview is
undated there. Track both snapshots and the matching generation record. Envelope schema 2 binds six
fixed inputs. Only envelope schema 2 is supported. Older generator records require regeneration with
a matching CLI/runtime pair.

Missing offline/replay captures are errors, without authoring-seed substitution. A first online
refresh can collect without a prior capture or seed; malformed captures and unsupported snapshot
schemas remain fatal.

The complete Pages artifact contains generated `docs/`, matching application/fallback/WASM resources,
profile-specific HTML metadata, favicon, webmanifest, and the deployed runtime manifest. The raw
installation bundle is verified before those three profile projections. The deployed manifest keeps
the original source revision/fingerprint and hashes the projected output bytes; its site is not a
raw runtime installation for future generation.

Upload that complete artifact before checked repository publication. Deployment follows successful
publication of the same candidate and uses its saved artifact identity on a deploy-only retry.
Git source alone omits the ignored runtime files that must be restored from the selected release
for a complete offline site.

## Local and recovery material

`docs/pkg/`, `docs/runtime-manifest.json`, native installations, caches, `dist/`, browser evidence,
transaction journals, and staging directories stay ignored or outside this repository. Build
outputs, debug symbols, distribution archives, credentials, and editor/OS metadata are not source.

Migration recovery preserves the complete original working files, runtime, and captured published
history outside the repository. Restore both snapshots and their matching generation record with
the complete data/runtime set. Retained history has a rolling limit of 24; it is not an all-time
archive. Temporary Actions artifacts do not replace deliberately retained recovery material.
