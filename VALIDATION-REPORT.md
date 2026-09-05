# Verification snapshot

Verified locally on September 5, 2026 for the Rust SOURCEFIELD migration and approved V4 integration.
This is a dated integration record, not a claim that future generated states have already passed.
The release workflow reruns its gates for each update.

| Check | Result |
| --- | --- |
| Rust 1.98.0, locked workspace and all targets | PASS |
| wasm32-unknown-unknown compilation | PASS |
| Native tests | 55 passed: CLI 13, collector 8, core 19, renderer 12, simulator 3 |
| Python positive/negative tooling tests | 13 passed |
| JavaScript interaction and browser-runner tests | 15 passed |
| Rustfmt, strict Clippy, rustdoc including private items | PASS, no suppressed warnings |
| Doctest invocation | PASS; no doctest examples are declared |
| wasm-pack 0.15.0 release build with wasm-opt | PASS, optimization enabled |
| Canonical artifact checks | 56 nodes, 80 edges, eight projects, six NuGet packages |
| Real Chromium browser integration | 12 regression groups plus V4 geometry and animation lifecycle checks passed |
| Desktop dark/light and mobile 390 x 600 | Visually inspected; controls visible after viewport changes |
| Repeated offline generation | 11 current SVG/JSON files byte-identical |
| Original Rust source and approved previews | 64 source files and eight preview files unchanged |

The generated preview state is `FA9C5E60AC766234`, explicitly labeled Preview.
Native HTTP fixtures exercise source failures, strict-live behavior, credential separation and
public-error redaction. They do not claim authenticated production GitHub API execution.
GitHub Actions execution and Pages deployment have not been performed locally.

Browser verification and screenshots are retained with the migration review outside this source
repository. The source archive creates its own SHA256SUMS; it excludes compiler caches and WASM.
See [Primary sources](SOURCES.md) and [Distribution contents](PACKAGE-INFO.md).

V4 matches the approved ornament geometry for all ten visible domain/project nodes and all eight
ownership curves. Browser checks cover nine curves, six evenly spaced private departures, both
NuGet columns, 26 ring directions, 11 staggered signals, and theme/layer/reduced-motion transitions.
Inline signal styles were replaced with local SVG rules and numeric browser timing to retain CSP.
The overview stays fixed while exploratory layers retain simulation. No browser/CSP errors remain.

The portable real-browser entry point is `node scripts/verify-browser.mjs`. It passed with Node.js
26.5.0, Playwright 1.62.1 and that installation's Chromium, using explicit runtime paths rather than
checkout-specific defaults. Its interaction and field suites now live in `tests/integration/`.
Three runner tests cover loopback serving, path isolation, missing tools and failed-launch cleanup.
Both workflows rerun presentation geometry tests after regeneration. Chromium remains an explicit
local/pre-publication check; the GitHub workflows do not install or run Playwright automatically.
