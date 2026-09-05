# Architecture

SOURCEFIELD is a five-crate Rust workspace. The generator owns public state and SVG output;
the browser consumes that output and a Rust/WASM simulation, with a JavaScript fallback.

| Crate | Responsibility |
| --- | --- |
| sourcefield-core | Configuration, validation, deterministic graph/state and explicit source provenance |
| sourcefield-collector | GitHub/NuGet collection and isolated opt-in private aggregate |
| sourcefield-render | Data-driven dark, light and static SVG presentation |
| sourcefield-cli | Collection/generation commands, output validation, no-op writes and history |
| sourcefield-wasm | Browser simulation with declared canvas dimensions and validated input |

## Public model

`config/profile.toml` is the curated content source. It separates project ownership, repository
visibility, technology roles, package publications, learning interests and personal setup.
The public presentation uses full project/package identities plus optional visual prefixes and
short labels. Its two domain fields group ownership; they do not claim implementation dependencies.
Private components are not inferred or read from other repositories during generation.

The approved composition contains eight projects and six NuGet publications, with US English copy.
Layout and presentation are generated from configuration/state instead of a table of project IDs
inside rendering code. Node coordinates and canvas dimensions are shared by SVG and simulation.
The default field is 1800 by 1680 logical units; responsive browser content has a semantic path
for narrow viewports instead of forcing long graph labels into a small canvas.

## Determinism and freshness

Semantic state normalizes unordered metadata. Fetch timestamps do not themselves constitute a
profile change. Source status remains explicit: missing data is not zero, partial totals are not
complete totals, and fallback versions are not current registry evidence. Rendering can still
change when the renderer changes, even when the collected semantic state is unchanged.

History preserves the timestamp of a stored snapshot. Invalid history is rejected rather than
silently reset. The pipeline validates history paths and retains unrelated files; only known
obsolete snapshots are candidates for pruning. Generation is separate from committing or deploying.

## Browser and delivery

The default Field view uses the approved SVG composition with fixed labels, circles and radial
connections. Independent ring and signal animations provide motion; Systems
and Capabilities expose the graph through the simulation. Package links and a semantic node list
remain keyboard accessible. Pause and system reduced-motion settings govern all animation paths.
Runtime status distinguishes real WASM from the JavaScript fallback. SVG signal delays use data
attributes and local styles; the page restores numeric timing after sanitization and animation
recreation, without admitting inline style declarations.

README SVGs are images: links inside them are not interactive in that context. The README links to
Pages and includes accessible text/package links outside the image. Pages uses local modules and
CSP; it does not require a third-party CDN. See [Primary sources](SOURCES.md).

The Python preview entry point calls the canonical Rust generator. It does not maintain a second
semantic hash or independent SVG template. WASM glue is generated at build time and remains ignored;
the handwritten fallback has its own stable source file.
