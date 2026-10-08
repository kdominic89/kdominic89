# Contributing

Contributions here cover approved personal content, consumer configuration, documentation, workflows,
and consumer tests. Sourcefield's generator and browser runtime are maintained
[upstream](https://github.com/kdominic89/sourcefield). Use ASCII US English for new text and follow
`.editorconfig`. Preserve existing approved branding copy exactly when its characters are intentional.

Personal facts and presentation belong in `config/profile.toml`. Shared doka-labs facts belong in
the organization-owned canonical manifest. Preserve the remote import, explicit technology bindings,
and consumer-owned affinities and geometry. Keep full project/package identities in links and stacks
in the generated Projects table. Approved private abstractions expose no private repository URLs or
implementation details. Do not inspect private source to embellish their descriptions.

Keep changes limited to their purpose. Add or update tests for meaningful contracts and rejected
inputs, such as composition, privacy selection, stale provenance, history preservation, and workflow
publication. Separate setup, operation, and assertions with blank lines. Run:

```sh
python3 -B -m unittest discover -s tests -p 'test_*.py'
```

Installed-runtime probes require `SOURCEFIELD_INSTALLATION`; report any skips. Preview with the
reviewed shared tooling and matching installation in a separate directory; see [Setup](SETUP.md).
Candidate tooling copies tracked inputs, so review new untracked files explicitly. Changes to
generated behavior belong upstream and require the consumer's release upgrade to use them.

Inspect SVGs, both managed sections of the root README, and actual browser current/history views,
mobile layout, keyboard/touch interaction, pause, and reduced motion when changing content or
presentation. Offline consumer CI does not establish strict live collection or public deployment.

Keep runtime installations, compiled WASM/glue, caches, screenshots, and distribution archives out
of Git. Intentional public SVG/JSON, captured input records, and ownership/history data remain
tracked. Do not repair generated output by hand or add a new dependency as incidental cleanup.
Generator upgrades change the authenticated lock and all matching workflow references together
after review. See [Maintenance](MAINTAINING.md) for that procedure.
