# Security

Report suspected vulnerabilities through the repository's GitHub security advisory mechanism when
available. Avoid placing credentials, private project source, or exploit payloads in public issues.

The primary boundaries are public configuration, remote API responses, retained history, generated
SVG, and browser state. Approved private-project summaries are public by design; see [Privacy](PRIVACY.md).

Configuration and state validation reject malformed identifiers, unsafe links and inconsistent
references. Rendering escapes text. Browser code validates fetched state and uses a restrictive
content security policy with local resources. Public collection errors are bounded and redacted.
These controls are defense in depth, not a guarantee that arbitrary unreviewed prose is safe to publish.

Never add tokens to configuration or fixtures. Keep the optional private-count credential separate
from normal public collection. Review workflow permissions and immutable action references before
publishing. Native binaries, WASM build output and caches are not source-controlled.
