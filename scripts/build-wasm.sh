#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! command -v wasm-pack >/dev/null 2>&1; then
  printf '%s\n' 'wasm-pack 0.15.0 is required; install with cargo install wasm-pack --version 0.15.0 --locked.' >&2
  exit 1
fi

if [[ "$(wasm-pack --version)" != 'wasm-pack 0.15.0' || ! -f Cargo.lock ]]; then
  printf '%s\n' 'wasm-pack 0.15.0 and the committed Cargo.lock are required.' >&2
  exit 1
fi

# Trailing arguments are forwarded to cargo; dependency resolution must stay locked.
wasm-pack build crates/sourcefield-wasm \
  --target web \
  --release \
  --out-dir ../../docs/pkg \
  --out-name sourcefield_wasm \
  --locked

python3 scripts/validate_artifact.py --approved-profile --require-wasm
