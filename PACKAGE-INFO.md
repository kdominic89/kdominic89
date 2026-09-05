# Distribution contents

The source archive contains the Rust workspace, lockfile, configuration, source scripts, documentation,
intentional SVG/JSON assets and browser source. The archive has deterministic ordering/timestamps and
an internal checksum manifest. An adjacent checksum identifies the archive itself.

Build caches, model caches, `.git`, native binaries, generated WASM/glue, editor/OS metadata and prior
archives are excluded. No cleanup of unrelated local files is needed to package the source.

The Pages deployment is a separate artifact: build WASM and deploy `docs/` with its generated module.
A source archive does not claim that the browser module has been compiled.

Checksums belong to an immutable archive or review snapshot. There is no rolling checksum file in
the source root that becomes stale when the daily profile update changes generated assets.
