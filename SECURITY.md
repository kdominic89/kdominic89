# Security

Report suspected vulnerabilities in this profile's configuration or publication workflow through
[GitHub private vulnerability reporting](https://github.com/kdominic89/kdominic89/security/advisories/new)
when that repository feature is available. Do not put credentials, private-project information,
or sensitive exploit payloads in public issues.

For Sourcefield CLI, collection, imports, generation, installation, or browser-runtime defects,
follow the upstream [security policy](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/SECURITY.md).
Include the exact release/source revision, minimal reproduction, input an attacker controls,
prerequisites, and expected impact. Use synthetic inputs and redact sensitive captures. This
consumer does not establish a separate response-time or support guarantee.

The consumer controls which facts become public, optional collection credentials, workflow
permissions, and repository/Pages publication. Relevant boundaries include authored configuration,
remote canonical content, API responses, captures/history, generated SVG/JSON, runtime archives,
and browser state. Approved private abstractions are public by design; see [Privacy](PRIVACY.md).

Use authenticated matching CLI/browser assets and tooling from the lock's source revision. The
release lock, reusable-workflow reference, and validation-tool checkout must agree. Raw runtime
verification happens before profile metadata projection; the deployed manifest records the projected
digests while retaining source identity. Provenance proves selected origin, not freedom from defects.

Input/reference validation, output escaping, bounded diagnostics, filesystem ownership checks, and
local browser resources provide defense in depth. Review arbitrary authored prose, credentials,
permissions, and generated output before publishing. Never add secrets to fixtures or captures,
disable certificate verification, or grant collection credentials as an implicit private opt-in.

Generation has read-only repository permissions. Publication checks expected HEAD and applies owned
files; deployment depends on successful publication of the same candidate. Keep those checks and
publication serialization intact. Preserve interrupted transaction evidence and follow reviewed
recovery rather than manually mixing file sets. Runtime output and caches stay ignored while the
complete matched runtime is included in the Pages artifact.
