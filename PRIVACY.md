# Public content and privacy

This profile intentionally publishes Dominic K.'s identity, interests, learning links, setup,
technology labels, public projects, and approved descriptions of private projects. A
`private-abstract` entry is curated public content, not anonymization. Its approved label, purpose,
and stack may be visible; its private repository URL and implementation details are not inputs.
The generator cannot establish editorial approval for arbitrary prose, so review content before
publication.

## Collection and private aggregate

The configured sources provide public GitHub account/repository and contribution information,
doka-labs organization metadata, and NuGet packages in the approved owner/families. Shared doka-labs
content is imported from the public organization-owned manifest with captured commit/digest
provenance. Importing that manifest does not authorize access to other repositories or private code.

Normal public collection uses `GH_TOKEN` or `GITHUB_TOKEN`. The optional private aggregate uses
the separate `PROFILE_TOKEN`. It counts only the configured user's owned private repositories
visible to that token. Organization repositories and collaborations are excluded; token permissions
can limit visibility. No private names, descriptions, URLs, file paths, or source are collected for
this option.

| Caller selection | `PROFILE_TOKEN` | Behavior |
| --- | --- | --- |
| Manual input false and variable absent/false | Absent or present | Public collection only |
| Manual `include_private_count` true or `SOURCEFIELD_PRIVATE_COUNTS=true` | Nonempty | Request only the private aggregate alongside public collection |
| Aggregate selected | Absent or empty | Warn and continue strict public collection without the aggregate |

The caller forwards a typed boolean and the optional named secret to the reusable workflow.
Credentials alone do not enable enumeration. The native CLI's explicit flag, configuration, and
environment opt-ins are separate deliberate inputs; this consumer's normal configuration disables
the private aggregate. The caller's missing-token continuation does not weaken strict public-source
validation or silently override an independently enabled native policy.

Supply tokens only through the environment or an explicitly selected workflow secret. Never place
credentials in configuration, captures, fixtures, diagnostics, public output, or history. Do not
use `secrets: inherit`. Bounded diagnostics and scoped source statuses help prevent disclosure;
missing, partial, Preview, or fallback observations must not become fabricated zeroes or live data.

The wrapper deliberately selects `assets/source-snapshot.json` for retained observations and
fallback. Offline generation and locked replay preserve an existing approved capture byte for byte,
including its collection date, source statuses, and prior observations. A prior approved private
aggregate may remain in that raw capture. Disabling the current aggregate does not erase it.

The separate `assets/render-snapshot.json` contains the effective policy-filtered input. Offline
Preview is undated there, and current rendering omits a private count when policy does not enable
it. Locked replay uses the recorded effective input, independent of today's credential selection.
Keep both snapshots and the matching generation record; review retained captures as public content.
Credentials alone never enable private collection. A first online refresh can collect without a prior
capture or seed; missing offline/replay captures fail, and malformed existing captures remain fatal.

## Publication and retention

Generated assets, managed README content, the complete Pages site, captured public inputs, and
retained history are public. Source status preserves uncertainty and dated observations. An aggregate
can itself be information the owner chooses to disclose even though it contains no names.

Removing content from current configuration does not remove earlier history, existing Git commits,
deployed copies, or separate recovery material. Review every affected surface when withdrawing
information. Rolling `history_limit = 24` controls live retention; offline/replay modes preserve
history. Recovery copies need their own access control.

The browser uses local application resources without embedded analytics or external fonts.
Following a project, package, or learning link contacts its destination service. Validators check
structure and common unsafe input; a passing validator is not proof that arbitrary prose contains
no secret. Report privacy or collection defects through [Security](SECURITY.md).
