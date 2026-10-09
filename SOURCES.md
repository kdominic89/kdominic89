# Primary sources

Reviewed on October 9, 2026 against the actual published Sourcefield v0.1.2 source,
matched authenticated installation, current public canonical manifest and preserved personal
authorship/history. Contracts and actual test evidence remain separate; see [Validation](VALIDATION-REPORT.md).

## Consumer contracts

The upstream links below select the exact `source_commit` in the authenticated consumer lock.
The canonical manifest link separately selects its captured organization commit.

| Contract | Primary source |
| --- | --- |
| Matched CLI/browser assets, authenticated installation, caller/private-count wiring, and publication | [Sourcefield distribution](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/docs/distribution.md) |
| Explicit destinations, validation, offline/locked replay, migration, history, and recovery | [Sourcefield operations](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/docs/operations.md) |
| Native lifecycle, actual-browser gates, and limits of mocked wrapper checks | [Sourcefield verification](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/docs/verification.md) |
| Remote imports, canonical order, shared technology bindings, local affinities, and stack text | [Sourcefield configuration](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/docs/configuration.md) |
| Built-in Sourcefield and database-safe icons | [Sourcefield icons](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/docs/icons.md) |
| Caller-owned schedule, permissions, checked publication, and deployment order | [Sourcefield consumer template](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/docs/consumer-workflow.yml.template) |
| CLI/runtime and input trust boundaries | [Sourcefield security policy](https://github.com/kdominic89/sourcefield/blob/cd2e2b6779c82a3f64346a47d40da4967fcc0dc5/SECURITY.md) |
| Personal root README placement | [GitHub profile README](https://docs.github.com/en/account-and-profile/how-tos/setting-up-and-managing-your-github-profile/customizing-your-profile/managing-your-profile-readme) |
| Typed reusable-workflow inputs, named secrets, and SHA references | [GitHub reusable workflows](https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows) |
| Selected Actions/reusable-workflow policy | [GitHub Actions settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository) |
| Pages artifacts and deployment jobs | [GitHub Pages custom workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages) |
| Public contributions and owned private repository aggregate | [GitHub GraphQL User](https://docs.github.com/en/graphql/reference/objects#user) |
| NuGet owner/family discovery, prereleases, and pagination | [NuGet SearchQueryService](https://learn.microsoft.com/en-us/nuget/api/search-query-service-resource) |
| Published package versions | [NuGet PackageBaseAddress](https://learn.microsoft.com/en-us/nuget/api/package-base-address-resource) |

## Authored facts and captured inputs

Dominic K.'s identity, headline, professional context, hardware capacity, interests, memberships,
private-project abstractions, and technology affinities are owner-approved facts in
`config/profile.toml`. They are not inferred from package metadata or private source.

Shared organization facts come from the actual public
[doka-labs canonical manifest](https://github.com/doka-labs/.github/blob/84b8d8766fbb5b3c4dc162a276aafd1b721ba49d/config/organization.toml).
The reviewed personal capture resolves that source to commit
`84b8d8766fbb5b3c4dc162a276aafd1b721ba49d`, with manifest SHA-256
`05c1f97d5fc4e1340a092dab337527f8eed6b4585032b074ae985e889e478aef`.
The capture's manifest bytes and digest establish its exact input; the ref remains a deliberate
online refresh choice. A personal-derived extraction is not a replacement for that canonical source.

Canonical content includes the approved SafeMigrations safe motif and
[SQL Server adapter](https://www.nuget.org/packages/Doka.EntityFrameworkCore.SafeMigrations.SqlServer/).
Registry package URLs identify publication; dated version/count observations do not establish a
permanent current version. The
[official SQL Server version list](https://api.nuget.org/v3-flatcontainer/doka.entityframeworkcore.safemigrations.sqlserver/index.json)
is the version source. Versions belong in collected observations rather than authored stack text.

The initial 24 legacy archives used for migration were captured from published history. The original local
working index was empty; its existence does not prove those archives were stored locally. Keep that
provenance distinction in migration and recovery evidence. Later live refreshes apply rolling retention;
the current 24 indexed states and genuine source dates are preserved during this upgrade.

Official Action identities are recorded in [Action pins](ACTION-PINS.md). The consumer's own
configuration and local checks are described in [Architecture](ARCHITECTURE.md) and
[Maintenance](MAINTAINING.md).
