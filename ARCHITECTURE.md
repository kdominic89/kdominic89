# Architecture

This repository is the personal content and publication consumer for Dominic K.'s SOURCEFIELD
profile. [Sourcefield](https://github.com/kdominic89/sourcefield) owns the generator, collectors,
renderer, browser runtime, and simulation. This consumer owns the approved public facts, its layout,
collection choices, validation, and publication. The current acceptance state is recorded in
[Validation](VALIDATION-REPORT.md).

## Ownership

| Surface | Purpose |
| --- | --- |
| `config/profile.toml` | Personal identity, approved private abstractions, technologies, interests, setup, collection policy, presentation, and layout |
| Remote `doka-labs/.github:config/organization.toml` | Canonical doka-labs projects, technology definitions, NuGet groups, package identities, and maintainer attribution |
| `sourcefield.lock.json` | Authenticated immutable release identity and matching native/browser archive digests |
| `.github/workflows/` | Consumer checks, 03:17 UTC refresh, checked repository publication, and Pages deployment |
| `assets/` | Captured inputs, resolved configuration, generation record, state, SVGs, and layout |
| Root generated ownership inventory | Generated replacement/pruning scope and selected authored README updates |
| Generated `docs/` | Complete candidate site and its public state/history |
| `README.md` | Authored personal context with managed Projects and NuGet sections |
| `tests/` | Consumer configuration, privacy, composition, output, and workflow contracts |

## Shared facts, personal presentation

The personal profile imports `config/organization.toml` from the actual public `doka-labs/.github`
repository at the authored `main` ref. A refresh resolves that ref to one commit and records the
repository, requested ref, path, commit, manifest bytes, and SHA-256 digest in
`assets/import-capture.json`. Offline composition uses that capture; it does not fetch a moving ref.
Organization facts are maintained once in their owning repository. Personal project geometry and
presentation remain consumer-owned.

The personal technology inventory retains 30 definitions and 15 doka-labs affinities. Three explicit
bindings share canonical C#, .NET, and JavaScript with their existing personal technology nodes.
Other personal definitions and cross-domain affinities remain local; importing the organization
does not reduce them to its three canonical technologies. Domain affinities describe ownership or
association, while implementation and target references carry their separate meanings.

The Sourcefield public project uses `builtin:sourcefield`. Canonical SafeMigrations supplies
`builtin:database-safe` and the published SQL Server adapter. Approved private projects remain
unlinked public abstractions. The managed Projects table retains their labels, summaries, and
stacks; imported rows follow canonical authored order. NuGet groups keep full IDs in accessible
links while presentation labels can be shorter. Versions and counts are observations.

## Generation and runtime

The reusable generator produces a complete candidate with explicit README selection. It has
read-only repository permissions. Consumer checks use tooling from the same reviewed source
revision and the matching native CLI/browser pair; this repository does not build a copied Rust
workspace or a per-consumer WASM runtime.

Generation first verifies the complete raw runtime from the selected installation. It then projects
profile-specific HTML metadata, favicon, and webmanifest into the site. The deployed
`docs/runtime-manifest.json` retains the runtime source revision and fingerprint and records the
digests of those three projected outputs. The deployed site is therefore a generated runtime
projection, not the raw installation directory to pass to `--runtime`. Application, fallback, and
WASM resources still come from the matched bundle. Do not patch either set by hand.

README SVGs are images; their internal links do not provide README interaction. Text project/package
links and the Pages link remain outside the image. The browser supplies Field, Systems, and
Capabilities views, semantic navigation, history selection, keyboard/touch interaction, pause, and
reduced motion. The personal palette and footer derive from the personal profile variant.

## State and publication

Missing data remains distinct from zero. Preview, partial, and dated fallback observations must not
be labeled live. Semantic normalization prevents collection timestamps alone from inventing a new
state. Captures and generation records support validation of current authored inputs and locked
offline replay; stale authored input or damaged provenance fails validation.

The consumer wrapper selects `assets/source-snapshot.json` as its explicit observation/fallback
input. Offline generation and locked replay retain that existing capture byte for byte, including
its date, source statuses, whitespace, and previously approved observations. The separate
`assets/render-snapshot.json` records the effective, privacy-filtered rendering input. Offline
Preview is undated there; it does not erase the retained Live capture's date or relabel it Preview.
Locked replay uses the recorded effective input rather than today's credential selection.

Generation-record envelope schema 2 binds six inputs: resolved configuration, retained source
snapshot, effective render snapshot, import capture, layout, and current profile state. Only
envelope schema 2 is supported. Older records fail the exact recorded generator identity check;
regenerate with a matching CLI/runtime pair after upgrading. Configuration schema 1 and state schema
3 are unchanged. Validation distinguishes record schema, generator identity, authored-input drift,
and damaged capture bytes.

A first online refresh can collect without a previous observation capture or authoring seed.
Malformed existing captures, unsupported snapshot schemas, and filesystem errors remain fatal.
Offline/replay require the retained
capture and required imports; they never substitute the empty `config/offline-snapshot.json` seed.
That seed remains an initial direct-native-authoring input. Failed strict collection publishes no
output; observation fallback never substitutes for a failed required canonical import.

Migration converts the 24 captured published legacy archives to schema 3 while retaining their
facts, hashes, and timestamps. These archives were captured from published history, not from the
empty original local history index. Offline, locked replay, and `--no-history` preserve indexed
history bytes. Allowed live updates apply rolling `history_limit = 24` retention.

History admission rejects a supplied empty or whitespace-only maintainer role, including in archives
from older generators. Omitted optional maintainer attribution remains valid; when attribution is
supplied, its role is required and nonblank. Inspect older archives before an upgrade; generation
does not silently rewrite or discard them. See [Validation](VALIDATION-REPORT.md)
for the current follow-up checks and the separate previous baseline evidence.

Publication is serialized. The caller uploads the complete Pages artifact, checks the expected
source HEAD, applies only owned output and selected README changes, and publishes the repository.
Pages deployment depends on successful publication of that same candidate. A failed push or stale
revision stops deployment. Recovery and deployment retries are described in
[Maintenance](MAINTAINING.md); installation and preview are described in [Setup](SETUP.md).
