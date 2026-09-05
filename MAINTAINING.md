# Maintenance

Update profile content in `config/profile.toml`, then regenerate with `./scripts/generate-preview.sh`.
Review both generated SVG and semantic JSON. A content edit must not introduce private implementation
details or label planned work as shipped. Package versions are collection results, not permanent prose.

Run `./scripts/verify.sh` and build release WASM before preparing publication. Inspect the real browser
with and without WASM, with reduced motion, using keyboard/touch and at a narrow viewport. A green
syntax check alone does not establish the visual or interaction contract.

Keep the pinned Rust toolchain and Cargo.lock consistent. A toolchain bump needs a locked full build,
lints, documentation and tests. Do not silently lower the declared MSRV to an untested compiler.
Resolve action commits from official release pages and update [Action pins](ACTION-PINS.md).

The scheduled workflow collects strict live data. If an upstream source fails, inspect the bounded
source status and retry after resolving the cause; do not edit warnings away or relabel fallback as
live. Investigate corrupt history before modifying it. Do not reset an invalid index automatically.

Archive creation uses the explicit source allowlist in the packaging code. Local caches and generated
binaries stay out. Deployable WASM belongs in the Pages artifact, built from the pinned source.

Generation uses an exclusive `.sourcefield.lock` in the assets directory. If a process crashes,
inspect the lock and confirm that no generator remains active before removing that stale lock.
The CLI does not guess whether another writer is still running.

## Presentation ownership

| Change | Source | Verification |
| --- | --- | --- |
| Public copy, project anchors and radii | `config/profile.toml` | Regenerate, inspect SVG/JSON, run presentation tests |
| Curves, radial endpoints and evenly distributed departures | `crates/sourcefield-render/src/geometry.rs` and composition in `lib.rs` | Rust geometry tests and generated presentation tests |
| Glyphs, hub rings and satellite motifs | `crates/sourcefield-render/src/ornaments.rs` | Dark/light/static inspection and real-browser direction checks |
| Standalone SVG resources and animation rules | `crates/sourcefield-render/src/lib.rs` | Generated SVG validation and real-browser checks |
| Loaded-page animation and timing restoration | `docs/app.css` and `docs/app.js` | Node helper tests plus real-browser theme/layer/reduced-motion checks |

Edit these sources rather than generated SVG files. Standalone SVG and the loaded page use different
style-loading paths because the page sanitizes imported SVG. Changes to animation rules must be checked
on both surfaces; a working standalone image does not prove that its timing survives page loading.

After generation, run `python3 -m unittest discover -s tests -p 'test_presentation.py'` against the
new output. Both CI workflows perform this check after their generation step. `./scripts/verify.sh`
checks Rust, Python and Node helpers but does not launch Chromium. Run `make verify-browser` for the
real browser suite after building WASM; see [Setup](SETUP.md#real-browser-verification). Review its
screenshots as well as its JSON results. Current CI does not install or run Playwright automatically.
