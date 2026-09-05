#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# A single generator keeps offline and live rendering semantics identical.
python3 scripts/bootstrap_preview.py "$@"
python3 scripts/validate_artifact.py --approved-profile
