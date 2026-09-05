# Contributing

Use US English for source, documentation, tests, and public copy. The profile content in
`config/profile.toml` is deliberately curated; private projects expose only approved names,
purpose, and stack. Do not expand that content by inspecting their implementation.

## Formatting and documentation

The `.editorconfig` is based on the shared ai-devsecops configuration. Rust uses four spaces
and a 100-column limit; JavaScript/Python use four spaces and 120 columns; TOML, JSON, YAML,
HTML, CSS and shell use two spaces. `rustfmt.toml` uses stable settings.

Leave a blank line after a completed control-flow block when another logical statement follows.
Keep `else`/`else if` attached to their corresponding `if`; the blank line goes after the complete
chain. Likewise separate a multiline variable initialization from the following operation.
Use a blank line between test setup, the operation under test, and assertions. These semantic
boundaries require review; rustfmt does not infer them from program intent.

All public Rust APIs, including fields and enum variants, have rustdoc. Internal functions and
types have documentation when it helps IDE users understand contracts. JavaScript and Python
helpers follow the same principle with JSDoc/docstrings. Comments explain a non-obvious reason,
constraint or invariant; avoid merely restating the next line of code. Do not suppress a lint
instead of addressing its cause.

## Verification

```sh
./scripts/generate-preview.sh
./scripts/verify.sh
./scripts/build-wasm.sh
python3 scripts/validate_artifact.py --require-wasm
```

Write positive and negative tests for meaningful contracts: malformed input, partial collection,
unsafe links, deterministic output, history corruption, and browser interactions. Keep Arrange,
Act, and Assert visibly separate. Documentation tests are run separately from all-target tests.

For page or rendering changes, also run `make verify-browser` with an existing Playwright/Chromium
runtime after building WASM. This checks the loaded page rather than the Node helper host. See
[Real-browser setup](SETUP.md#real-browser-verification) for runtime overrides and evidence output.

Do not add dependencies, publish artifacts, or mutate Git history as an incidental cleanup.
Generated native binaries, WASM modules/glue, caches, screenshots, and distribution archives stay
out of version control. Generated SVG/JSON are intentional public text artifacts.
