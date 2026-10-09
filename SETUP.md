# Setup

This is the personal GitHub profile repository `kdominic89/kdominic89`. GitHub displays the root
`README.md`. The interactive profile is [available on Pages](https://kdominic89.github.io/kdominic89/).
Personal content and presentation belong in `config/profile.toml`; doka-labs facts are imported
from its public canonical manifest.

## Repository automation

Select **Settings > Pages > Build and deployment > GitHub Actions** and review the `github-pages`
environment. The repository Actions policy was inspected on October 9, 2026 and permits all actions;
no new allowlist entry is required under that policy. If selected-actions policy is introduced later,
allow the exact reusable workflow below together with the configured official Actions:

```text
kdominic89/sourcefield/.github/workflows/generate.yml@cd2e2b6779c82a3f64346a47d40da4967fcc0dc5
```

`Update SOURCEFIELD` retains the 03:17 UTC daily schedule and manual dispatch. Its full source SHA
matches `sourcefield.lock.json`, selecting immutable `v0.1.2`. The generator produces a candidate
under read-only permissions. Strict live collection and required remote imports must succeed before
publication. The caller uploads the complete Pages artifact before applying owned output against
the expected repository HEAD. Pages deploys that same artifact after successful repository publication.

Pushes to main and pull requests run read-only `Validate SOURCEFIELD`: matching lock/tooling,
authenticated release installation, consumer tests, isolated offline generation and artifact checks.
Use **Actions > Update SOURCEFIELD > Run workflow** for refresh, then check publication, Pages and
the served identity. Local checks do not establish hosted success.

## Optional private aggregate

Normal collection is public. Enable only the owned private repository total through the manual
`include_private_count` boolean or repository variable `SOURCEFIELD_PRIVATE_COUNTS=true`. Configure
the optional named Actions secret `PROFILE_TOKEN` only if this aggregate is wanted. A token alone
does not enable it; selected intent without a token warns and continues public collection.
Keep `collect_private_repository_count=false` in the normal authored configuration.
See [Privacy](PRIVACY.md) for the exact disclosure boundary.

The account caption uses `owner-repositories`; it shows available public totals and the approved
private aggregate only when enabled. The imported Doka caption uses its canonical `selected-projects`
setting, so RelationalLab is counted as a private abstraction in both profiles. Counts never add
project circles automatically. Caption wording belongs in configuration, not generator code.

## Optional local preview

Use Python 3.11+ and shared tooling checked out at exactly `cd2e2b6779c82a3f64346a47d40da4967fcc0dc5`.
Reuse one matching verified Sourcefield installation across profile repositories. No per-repository
manual installation or Rust build is required. If this machine has no matching installation, bootstrap
once into a new directory with GitHub CLI release/attestation verification support:

```sh
SOURCEFIELD_SOURCE=/path/to/pinned/sourcefield
export SOURCEFIELD_INSTALLATION=/path/to/shared/sourcefield-v0.1.2
CONSUMER_REPOSITORY=/path/to/kdominic89

python3 -B "$SOURCEFIELD_SOURCE/scripts/bootstrap_release.py" \
  --lock "$CONSUMER_REPOSITORY/sourcefield.lock.json" \
  --destination "$SOURCEFIELD_INSTALLATION"
```

Keep existing installations intact. Verify the source checkout with `git rev-parse HEAD`; bootstrap
authenticates the immutable release, both asset digests and repository-bound attestations.

```sh
python3 -B "$SOURCEFIELD_SOURCE/scripts/check_pin.py" \
  --lock "$CONSUMER_REPOSITORY/sourcefield.lock.json" \
  --workflow "$CONSUMER_REPOSITORY/.github/workflows/update-profile.yml"

python3 -B "$SOURCEFIELD_SOURCE/scripts/consumer_candidate.py" \
  --source "$CONSUMER_REPOSITORY" --destination /path/to/new-external-preview \
  --installation "$SOURCEFIELD_INSTALLATION" --readmes '["README.md"]' --offline

python3 -B -m http.server 8000 --bind 127.0.0.1 \
  --directory /path/to/new-external-preview/docs
```

Open `http://127.0.0.1:8000/` for module/WASM loading. The new candidate directory must not exist.
Use a reviewed committed input set for the wrapper. It begins from the selected HEAD checkout;
staging deletions alone does not remove those HEAD files. Review structural pre-commit changes
in an explicit external copy or complete local fixture before using this publication path.
It selects `assets/source-snapshot.json` explicitly. Offline generation preserves existing capture
bytes, source status and retrieval date, including an absent legacy date. It performs no new collection
and does not substitute the authoring seed when the retained capture is missing.

`assets/render-snapshot.json` separately records undated, policy-filtered Preview input. Track both
snapshots and the generation record. Envelope schema 2 binds six inputs; locked offline replay
checks them and the generator identity and uses the recorded effective input without selecting new
credentials. Regenerate with the matched pair after a generator upgrade. An explicit local
`--private-counts` preview also requires a nonempty `PROFILE_TOKEN`; without it, the request warns
and continues public-only. Replay of already recorded inputs requires neither flag nor credential.

Omit `--offline` for a strict online refresh. The first online collection may start without a prior
capture or authoring seed. Malformed existing captures fail; an undated legacy capture or Preview
seed cannot authorize fallback after failed collection. Offline/replay preserve all indexed history;
allowed live updates apply rolling retention 24. See [Maintenance](MAINTAINING.md).

## Consumer checks

From the consumer repository with `SOURCEFIELD_INSTALLATION` exported, run:

```sh
python3 -B -m unittest discover -s tests -p 'test_*.py'
```

Integration tests require the complete matching installation; inspect the result for skipped tests.
Review dark/light/static images at README and mobile widths, current and historical browser views,
WASM/fallback, keyboard/touch, pause and reduced motion. Implementation tests belong upstream.
