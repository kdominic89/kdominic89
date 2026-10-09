# Validation

Verified locally on October 9, 2026 with the published Sourcefield `v0.1.2` CLI and matching
browser runtime at `cd2e2b6779c82a3f64346a47d40da4967fcc0dc5`. The existing shared installation
was authenticated against the immutable release, both archive digests, and GitHub attestations
for the Doka upgrade. This consumer reuses that exact pair and verbatim release lock; no additional
installation, dependency, upstream build, or implementation suite was introduced.

## Retained inputs and captions

The starting point is the actual published personal revision
`928fa496b98859319b1f3098e7e2d8651c31ff56`. The previously clean local branch was behind generated
refreshes and was fast-forwarded to it before this upgrade. No published history was rewritten.

The personal account domain now selects `owner-repositories`. The canonical Doka manifest selects
`selected-projects`; the personal authored configuration contains no duplicate organization caption.
The exact public manifest is captured from `doka-labs/.github` at
`84b8d8766fbb5b3c4dc162a276aafd1b721ba49d`, SHA-256
`05c1f97d5fc4e1340a092dab337527f8eed6b4585032b074ae985e889e478aef`.
Its bytes were verified against that public Git commit/blob before explicit isolated offline adoption.
The import's authored ref stays `main`. Normal online updates resolve it again; offline replay uses
its captured bytes.

The staged output is Preview `1D9A389991A0A7E3`: 63 nodes, 92 edges, ten projects, ten packages,
and the approved 1800 by 1885 canvas. It retains the previously published, explicitly opted-in
aggregate: `12 public / 9 private repos`. The ordinary public-only candidate instead shows
`12 public repos`. Doka shows `3 public / 1 private repos` in both personal candidates and the
actual published organization profile. RelationalLab remains private-abstract and unlinked.
Visible SVG captions and their accessible labels agree. The selected project circles do not change.

Private collection remains false in authored configuration. The existing manual boolean, repository
variable, optional named `PROFILE_TOKEN`, schedule, permissions, and publication ordering are
unchanged. An explicit offline private preview also requires a nonempty credential; an existing
authenticated owner credential was held only in memory for that process. It was neither logged nor
persisted, and offline execution performed no observation collection. The credential-free public
wrapper and locked replay were checked separately.

The raw source capture remains byte-identical, including its genuine retrieval timestamp
`2026-10-08T14:15:32.105853809+00:00`, source statuses, package versions, and approved aggregate.
README, authored layout, offline seed, all 24 current archives, and the complete history index remain
byte-identical to the published baseline. Identity, stacks, hardware, learning, private abstractions,
all 30 technologies, 15 organization affinities, and three canonical bindings are preserved.

The effective input is explicitly undated Preview. Its `1970-01-01T00:00:00Z` generation timestamp
is a deterministic sentinel, not a collection date. Schema-2 provenance binds exactly six captured
inputs and current authored bytes; every digest and generated-file ownership entry was checked.
Locked replay without credentials reproduces the already recorded private Preview exactly.

## Executed checks

| Check | Result |
| --- | --- |
| Existing consumer tests | All 42 covered successfully: 41 passed in the suite; one local-proxy test was blocked by the socket sandbox and passed alone with local socket permission. Zero skipped. |
| Test changes | Existing release expectations updated; two absent-date tests now explicitly construct temporary legacy captures. No test added or removed. |
| Release lock and workflow/tooling identity | Verbatim authenticated lock; update, validation checkout, and validation identity argument all match the selected source SHA. |
| Native validation and artifact checks | Passed for retained-private and public-only candidates; the published Doka artifact also passes with matching restored runtime. |
| Workflow syntax | Update and validation both pass actionlint. |
| Provenance, ownership, and locked replay | Six input hashes and all owned files match; replay preserves every tracked candidate byte. |
| Preserved data | Current source, README, layout, offline seed, 24 archives, and index match the published baseline byte for byte. |
| Real Chromium | Private, public-only, and published Doka views pass WASM/ARIA, desktop dark and mobile light, no horizontal overflow, and reduced motion. |
| Affected browser behavior | Mobile keyboard opening and reachable close, pause, JavaScript fallback, one retained historical view and return to the current captions all pass. |
| README comparison | Before/after screenshots at 820 and 343 content pixels; captions do not overlap project text. |
| Repository Actions policy | Enabled and allows all actions; no additional reusable-workflow allowance or settings write is required. |

Commands actually executed, with external candidate/installation paths abbreviated:

```sh
SOURCEFIELD_INSTALLATION=/path/to/existing/authenticated-installation \
  python3 -B -m unittest discover -s tests -p 'test_*.py' -v

# Only the socket-blocked case was repeated with local test-server permission.
python3 -B -m unittest \
  test_consumer_generation.InstalledGenerationTests.test_locked_replay_restores_outputs_with_remote_transport_unavailable -v

sourcefield generate --root /path/to/candidate --offline --no-history \
  --fallback-snapshot assets/source-snapshot.json --runtime /path/to/matching/runtime \
  --readme README.md --private-counts

sourcefield generate --root /path/to/candidate --offline --locked --no-history \
  --runtime /path/to/matching/runtime --readme README.md

sourcefield validate --root /path/to/candidate

python3 -B "$SOURCEFIELD_SOURCE/scripts/check_pin.py" \
  --lock sourcefield.lock.json --workflow .github/workflows/update-profile.yml \
  --own-commit cd2e2b6779c82a3f64346a47d40da4967fcc0dc5

python3 -B "$SOURCEFIELD_SOURCE/scripts/consumer_candidate.py" \
  --source /path/to/prepared-consumer --destination /path/to/public-preview \
  --installation /path/to/existing/authenticated-installation \
  --readmes '["README.md"]' --offline

python3 -B "$SOURCEFIELD_SOURCE/scripts/validate_artifact.py" \
  --root /path/to/candidate --workflow-root /path/to/prepared-consumer --require-wasm

actionlint .github/workflows/update-profile.yml .github/workflows/validate.yml
```

The initial isolated capture edit was correctly rejected as modified owned input; its verified
public bytes were then adopted once with `--adopt-existing` in that external preparation tree.
Subsequent ordinary generation, replay, and corruption tests keep the normal ownership guard.
An external browser probe initially expected the abbreviation `JS`; the deliberate runtime label
is `JAVASCRIPT FALLBACK`. Correcting that probe required no product runtime edit. Original failures,
final logs, exact input/output hashes, command arguments, and screenshots remain in the external
`reviews/git-profile` evidence directory.

## Publication and rollback

Local adoption is ready for commit review. Commit, push, the upgraded hosted `Validate SOURCEFIELD`,
`Update SOURCEFIELD`, and Pages publication are separate steps; local acceptance does not certify
those hosted results. No GitHub settings were changed and neither other product repository was edited.

A later live update uses the unchanged private intent/token selection and current public APIs.
It can change observations and rolling history naturally. This offline upgrade does not establish
fresh live data or invent missing historical dates. Initial migration converted published legacy
history; the retained set now also contains genuine later refreshes.

Sourcefield v0.1.1 rejects caption fields in authored configuration, states, and history archives.
Rollback requires the complete compatible revision, data, ownership, matched CLI/runtime and pins.
An older generator must also use a compatible fixed Doka manifest revision before live collection;
changing only the lock while continuing to import the new canonical `main` is insufficient.
