# Maintenance

Edit personal content, technology affinities, interests, setup, collection policy, and layout in
`config/profile.toml`. Shared doka-labs project and NuGet facts belong in
`doka-labs/.github:config/organization.toml`. The remote import brings those facts into this
profile's own presentation. Edit neither generated artifacts nor managed README sections as the
content source.

## Content refresh

Keep Dominic K.'s approved identity and personal voice, all 30 technology definitions, and their
15 doka-labs affinities unless a deliberate content change is reviewed. The three canonical
technology bindings must continue to match local labels/categories. Layout overrides use full
stable graph IDs; remove obsolete overrides when removing a project. A new project should not move
existing authored anchors or silently replace approved copy.

Keep canonical maintainer attribution accurate. When a maintainer is supplied, its role must be
nonempty after trimming whitespace; do not invent or silently drop that attribution. The same
admission rule applies to retained archives from older generators. Omitted optional maintainer
attribution is valid; supplied attribution requires a nonblank role. Inspect history before upgrading
instead of silently rewriting or discarding an archive.

Private-project labels, summaries, and stacks are public abstractions. Review edits for disclosure
and keep them unlinked. NuGet discovery stays inside the configured owner and package families.
Versions and counts are collected observations; do not copy dated values into permanent prose.
Adding a new family requires canonical content changes. Existing-family publication can be discovered
automatically without inventing unpublished package identities.

Select `--readme README.md` or the candidate tool's `--readmes '["README.md"]'` explicitly. Retain
one complete Projects marker pair and one complete packages marker pair. The generated table keeps
approved stack text and canonical row order; authored content outside the markers remains separate.
See [Setup](SETUP.md) for preview in a new external directory.

Run consumer tests, the shared artifact validator, and actual-browser inspection. Check both themes,
static rendering, README/mobile readability, source status, all retained history, pause, reduced
motion, and accessible links. Use the matching installation rather than compiling copied generator
code here. Review browser evidence as well as command results.

## Imports, collection, and history

Online refresh resolves the required remote manifest at one commit and records its exact bytes and
digest. A failed required import stops generation independently of observation fallback. Never replace
the actual canonical manifest with a personal-derived extraction to make composition pass.

Keep both observation snapshots, import captures, and generation records tracked. The wrapper
selects `--fallback-snapshot assets/source-snapshot.json`. Offline preview and locked replay preserve
an existing source capture byte for byte, including its date, source statuses, and approved facts.
The effective rendering input is separate in `assets/render-snapshot.json`: offline Preview is
undated and applies current private-count policy there. Previously approved raw observations may
retain a prior aggregate while current rendering omits it. Locked replay reuses the recorded effective
input rather than today's credential selection.

Generation-record envelope schema 2 binds six fixed inputs, including both snapshots. Only envelope
schema 2 is supported; older generator records require regeneration with the matching pair.
Validation diagnostics distinguish record schema, generator identity, authored-input drift, and
capture corruption. After an implementation upgrade, regenerate with the matching pair before
validating or replaying.

Missing offline/replay observations or required import captures fail without seed substitution.
A first online refresh may collect without prior observations or the authoring seed; malformed existing
captures, unsupported snapshot schemas, and filesystem errors remain fatal. `config/offline-snapshot.json`
serves initial direct
native offline authoring. A Preview seed or undated observation is not usable live fallback evidence.

Strict live failures stop publication. Investigate the named incomplete source, limits, credentials,
or trust configuration instead of deleting warnings or labeling old data live. The optional private
aggregate retains the intent/token separation in [Privacy](PRIVACY.md). Missing optional caller
credentials warn and continue public collection; actual required public-source failures still fail.

The migrated legacy source capture has no recorded retrieval date. That absence remains empty;
it cannot authorize live fallback. A successful future live refresh will record genuine collection
provenance. Offline Preview and migrated history never fabricate a missing date.

The 24 migration archives came from captured published history. Their facts, hashes, and timestamps
must survive conversion. Offline, locked replay, and `--no-history` preserve the indexed archives and
index byte for byte, even when the configured limit is lowered. Allowed live history updates apply
rolling `history_limit = 24` retention, including when the current semantic hash is already archived.
Rolling history is not an all-time archive.

## Generator upgrades

Obtain a new authenticated lock from an actual reviewed immutable release. Use its source revision's
`scripts/check_pin.py --prepare` with the new lock, current update workflow, and a new external review
directory. Inspect the prepared lock and workflow together, and update the validation workflow's
tool checkout and identity checks to that same source SHA. Review [Action pins](ACTION-PINS.md).

Bootstrap a new matching installation, then verify candidates and replay boundaries before adopting
the change. Update only this repository's exact reusable-workflow allowlist entry if its selected-actions
policy requires it. Ordinary profile refreshes do not upgrade the generator. Do not substitute a
development version, fabricated archive digest, or old release for the selected implementation.

Raw runtime verification precedes profile-specific HTML, favicon, and webmanifest projection. The
deployed manifest retains source identity and records projected output digests. Do not hand-edit
runtime files or claim every deployed byte is identical to the raw bundle. Generator, renderer,
collector, runtime, and release implementation changes belong upstream.

## Publication and recovery

Keep cron `17 3 * * *`, explicit single-README selection, and serialized publication. Upload the
complete Pages artifact before repository mutation. Check the expected source HEAD before applying
owned files. Deployment must depend on successful publication of the same candidate. Inspect a
stale revision or push conflict and generate from the correct source before retrying publication.

If repository publication succeeded but deployment failed, retry deployment of the saved artifact
identity. Do not recollect observations just to retry deployment. Verify the served page after the
deployment; a local test result does not certify hosted publication.

Keep original working files, matching runtime, captured history, and recovery material outside the
repository under suitable access control. For interrupted generation, preserve the transaction
journal, confirm no writer remains, and follow the reviewed upstream recovery procedure with the
exact abandoned lock token. Do not delete locks blindly or mix old runtime with new state.
Restore the complete matching configuration, data, history index, and runtime set, then validate it.
Include the retained source snapshot, effective render snapshot, and matching generation record.
Actions artifacts are temporary copies; retain recovery evidence separately when it must survive
artifact expiration.
