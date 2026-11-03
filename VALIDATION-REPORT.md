# Validation

Verified locally on November 3, 2026 using the published Sourcefield `v0.1.3` native CLI
and matching browser runtime at `ef93cf245ab5dd510132e61ec3cfce64ffafa17a`. This reuses the
shared installation authenticated earlier in the same session for doka-labs, including its
immutable release, lock, asset digests and repository/source-bound attestations. No separate
per-profile build or installation was needed for the upgrade.

## Prepared upgrade and preserved profile

The freshly fetched personal `main` matched `origin/main` before changes. The release lock,
reusable-workflow source, validation checkout, validation identity argument and existing workflow
tests all select the same v0.1.3 pair. Manual/variable private-count intent, the optional named
`PROFILE_TOKEN`, authored default-off collection policy, one README, 03:17 UTC schedule,
permissions, serialization and checked Git/Pages publication ordering are unchanged.

The real repository's authored profile, captured organization manifest, observations, layout,
README, generated output and 24 retained archives plus index remain byte-identical. The existing
operator `.editorconfig` preference is included verbatim in the proposed commit.

Verification generated a separate offline candidate. It has 67 nodes, 96 edges,
ten projects and 14 observed packages. Its complete nodes, edges, profile identity and canvas
match the existing output. Both old and new published generators produce the same complete
offline semantic state, including Preview hash `28202A128F89C622`. Relative to the existing
Live state, offline conversion changes only generated time, mode, source status, warnings and
semantic hash; it does not alter the preserved graph or collect fresh observations.

The candidate retains `12 public repos` for the personal account and `3 public / 1 private repos`
for doka-labs, with matching accessible labels. No private aggregate is fabricated. All six
private project abstractions remain unlinked. Both approved motifs, stacks, hardware, learning,
interests, all 30 technologies, 15 organization affinities and three canonical bindings remain intact.
The captured canonical manifest bytes also match the actual public doka-labs source inspected
during this upgrade; offline generation preserves its original captured commit and digest.

Raw observation and import captures, layout, offline seed, README and every history file remain
byte-identical in the candidate. Its effective rendering input is explicitly undated Preview;
the schema 2 record binds six inputs and the new generator identity. Locked replay reproduced
all 76 candidate files byte for byte without collection credentials.

## Existing height-test correction

The configured `render.height = 1885` is a minimum. Additional discovered package rows already
expanded the checked-in canvas to 1975 pixels before this upgrade. The old test incorrectly
asserted that generated height equaled the configured minimum. The failure was reproduced
with authenticated published v0.1.2 and v0.1.3 binaries before changing the test.

That existing case now compares the complete generated canvas with the checked-in approved
canvas and separately enforces the authored minimum. It passes with both releases. No authored
dimensions or project positions changed, and no test case was added, removed or skipped.
The existing workflow contract's release/SHA expectations were updated to the selected pair.

## Executed verification

| Check | Result |
| --- | --- |
| Authenticated shared v0.1.3 installation reuse and exact lock bytes | Passed |
| Old/new released height controls before and after test correction | Reproduced the same old failure; corrected case passes with both releases |
| Existing personal consumer suite with released v0.1.3 | 42 passed; zero skipped |
| Release pin checks and both workflows with actionlint | Passed |
| Current public canonical manifest versus captured manifest | Exact byte match |
| External offline candidate and complete artifact validation | Passed; real WASM present |
| Graph, geometry, README, captures, layout and history preservation | Passed by exact comparisons |
| Full old/new offline semantic state | Identical |
| Locked replay | All 76 files reproduced byte for byte |
| Actual Chromium gate | Passed: desktop/mobile, dark/light, WASM/fallback, links, keyboard/touch, pause, reduced motion, inspector close and history handling |

Representative commands actually executed, with external paths abbreviated:

```sh
SOURCEFIELD_INSTALLATION=/path/to/verified/shared-installation \
  python3 -B -m unittest discover -s tests -p 'test_*.py' -v

python3 -B "$SOURCEFIELD_SOURCE/scripts/check_pin.py" \
  --lock sourcefield.lock.json --workflow .github/workflows/update-profile.yml \
  --own-commit ef93cf245ab5dd510132e61ec3cfce64ffafa17a

python3 -B "$SOURCEFIELD_SOURCE/scripts/consumer_candidate.py" \
  --source /path/to/consumer --destination /path/to/new/candidate \
  --installation /path/to/verified/shared-installation --readmes '["README.md"]' --offline

python3 -B "$SOURCEFIELD_SOURCE/scripts/validate_artifact.py" \
  --root /path/to/candidate --workflow-root /path/to/consumer --require-wasm

sourcefield generate --root /path/to/candidate --offline --locked --no-history \
  --fallback-snapshot assets/source-snapshot.json \
  --runtime /path/to/verified/shared-installation/runtime --readme README.md

actionlint .github/workflows/update-profile.yml .github/workflows/validate.yml

node "$SOURCEFIELD_SOURCE/scripts/verify-browser.mjs" \
  --site /path/to/candidate/docs --playwright-module /path/to/playwright/index.mjs \
  --browser /path/to/installed/chromium --output /path/to/browser-evidence
```

Exact commands, original failure logs, successful controls, file comparisons and browser screenshots
are retained externally under reviews/git-profile. No upstream Rust/build suite was repeated for
this consumer upgrade. Verification used the existing Node and Chromium installations with the
browser sandbox enabled; no credentials were persisted or private live collection requested.

## Publication boundary

The repository Actions policy was read on November 3, 2026 and permits all actions. No additional
workflow allowlist entry, new secret or remote settings write is required for this version change.
The existing private aggregate remains optional and requires its existing explicit intent/token
combination; this offline verification does not prove fresh private or public collection.

Commit, push, hosted Validate SOURCEFIELD, the next Update SOURCEFIELD refresh and Pages deployment
remain separate steps. The pin update preserves the current generated record and published files;
the next successful strict live refresh regenerates them with the new pair. Local acceptance does
not establish a hosted run or public deployment. Doka-labs and Sourcefield product files were not edited.

Rollback to v0.1.1 still requires a complete compatible configuration/data/history/runtime revision
and a compatible fixed canonical manifest. That older generator rejects caption fields; changing
only the lock while retaining the new captured or remote manifest is insufficient.
