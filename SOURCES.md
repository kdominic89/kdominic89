# Primary sources

Checked on 2026-09-05. These sources ground the implementation contracts; local verification proves
what this checkout actually does. A source citation alone is not evidence that our implementation passes.

| Decision | Primary source |
| --- | --- |
| Pinned dependencies for an executable workspace | [Cargo manifest and lockfile](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html) |
| Native targets and separate doctests | [Cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html) |
| Document public APIs | [rustc missing_docs lint](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#missing-docs) |
| Useful executable API examples | [Rustdoc tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html) |
| Stable formatting settings; semantic blank lines require review | [rustfmt configuration](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md) |
| Explicit browser launch and cleanup with an existing runtime | [Playwright library](https://playwright.dev/docs/library) and [BrowserType.launch](https://playwright.dev/docs/api/class-browsertype#browser-type-launch) |
| Browser module initialization | [wasm-bindgen without a bundler](https://wasm-bindgen.github.io/wasm-bindgen/examples/without-a-bundler.html) |
| Radial cubic endpoints follow their endpoint control handles | [SVG 2 cubic Bezier commands](https://www.w3.org/TR/SVG2/paths.html#PathDataCubicBezierCommands) |
| Original bridge color stops and local paint references | [SVG 2 linear gradients](https://www.w3.org/TR/SVG2/pservers.html#LinearGradients) |
| Independent sibling rings use opposite animation directions | [CSS Animations Level 1](https://www.w3.org/TR/css-animations-1/#animation-direction) |
| SVG images do not provide inline interaction | [SVG 2 secure animated mode](https://www.w3.org/TR/SVG2/conform.html#secure-animated-mode) |
| WASM-specific compilation permission | [Content Security Policy Level 3](https://www.w3.org/TR/CSP3/#directive-script-src) |
| Pause for nonessential automatic motion | [WCAG 2.2 Pause, Stop, Hide](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html) |
| Honor system motion preference | [W3C technique C39](https://www.w3.org/WAI/WCAG22/Techniques/css/C39) |
| Discover NuGet search resources from service index | [NuGet SearchQueryService](https://learn.microsoft.com/en-us/nuget/api/search-query-service-resource) |
| GitHub user aggregate scope | [GitHub GraphQL users](https://docs.github.com/en/graphql/reference/users) |
| Exclusive filesystem creation | [OpenOptions::create_new](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new) |
| Exact processor name | [AMD processor announcement](https://ir.amd.com/news-events/press-releases/detail/1232/amd-announces-expanded-consumer-and-commercial-ai-pc-portfolio-at-ces) |

The local toolchain installation resolved Rust 1.98.0, published 2026-08-20, and the matching
`wasm32-unknown-unknown` standard library. This is the declared and tested compiler floor for this
checkout; the earlier unverified Rust 1.85 claim is not retained. Action sources are in [ACTION-PINS.md](ACTION-PINS.md).

Profile membership, personal hardware capacity and professional context are owner-provided facts.
The published package URLs are authoritative identities, not a claim that their download counts or
versions never change. Private-project descriptions intentionally omit internal implementation detail.

The build tool is wasm-pack 0.15.0, verified against its [official release](https://github.com/wasm-bindgen/wasm-pack/releases/tag/v0.15.0)
and [locked dependencies](https://github.com/wasm-bindgen/wasm-pack/blob/v0.15.0/Cargo.lock).
The workspace locks wasm-bindgen 0.2.128; generated browser glue must match that crate version.
