# Public content and privacy

Project names, short descriptions, and technology stacks in `config/profile.toml` are deliberately
public, including entries describing private repositories. `private-abstract` identifies a curated
public description of a private project; it is not anonymization.

Private implementation details, internal architecture, file paths, source code, credentials, and
unapproved metadata do not belong in this configuration or the generated profile. The generator
cannot decide which prose the owner has approved. Review content changes before publication.

## Collection

Public GitHub repositories and NuGet package metadata are collected from the configured services.
The normal GitHub credential is `GH_TOKEN` or `GITHUB_TOKEN`. No private source content is collected.
An explicitly enabled private-count option uses `PROFILE_TOKEN` only for an aggregate count;
this counts the configured user's owned private repositories visible to that token.
Organization repositories and collaborations are excluded; token permissions can restrict visibility.
The option is off by default. It does not publish private repository names, descriptions, or URLs.

Source status distinguishes available, missing, partial, preview, and fallback information.
A failed request must not become a fabricated zero or an unmarked old version. Public diagnostics
use bounded messages rather than upstream exception text, response bodies, or authorization data.

## Publication and history

Everything under `assets/` and the deployed `docs/` directory is public. History snapshots retain
previously published content. Removing a detail from current configuration alone does not remove
it from history or from an existing Git commit. Any later withdrawal must review all publication
surfaces deliberately.

The browser loads local application resources and does not embed third-party analytics or fonts.
Following external links contacts GitHub, NuGet or the named learning provider through the browser.

Validators catch structural violations and common unsafe content; passing them does not establish
that every manually written description has editorial approval or that no possible secret exists.
