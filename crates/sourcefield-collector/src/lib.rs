//! Collect configured public metadata with explicit provenance and redacted failures.
#![deny(missing_docs)]

use std::collections::BTreeSet;

use chrono::{Duration, Utc};
use reqwest::{
    Client, StatusCode,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::Deserialize;
use serde_json::{Value, json};
use thiserror::Error;

use sourcefield_core::{
    AccountSnapshot, ActivityDay, Config, ContributionSnapshot, DataStatus, PackageSnapshot,
    RepositorySnapshot, Snapshot, SnapshotMode, SourceStatus,
};

const GITHUB_API: &str = "https://api.github.com";
const GITHUB_GRAPHQL: &str = "https://api.github.com/graphql";
const NUGET_INDEX: &str = "https://api.nuget.org/v3/index.json";

#[derive(Debug, Error)]
/// Bounded errors; upstream response bodies are never retained.
pub enum CollectorError {
    /// Transport or response decoding failed.
    #[error("HTTP transport or decoding failed")]
    Client(#[from] reqwest::Error),
    /// An upstream returned a non-success status.
    #[error("upstream HTTP status {status}")]
    Http {
        /// Status without URL or response payload.
        status: StatusCode,
    },
    /// GraphQL rejected the request.
    #[error("GitHub GraphQL returned errors")]
    GraphQl,
    /// An expected response field was absent or invalid.
    #[error("unexpected API response: {0}")]
    Response(String),
}

#[derive(Clone)]
/// Public metadata collector with isolated optional aggregate credentials.
pub struct Collector {
    client: Client,
    token: Option<String>,
    private_token: Option<String>,
    github_api: String,
    graphql_api: String,
    nuget_api: String,
}

impl Collector {
    /// Create a collector using a public-metadata token, if supplied.
    pub fn new(token: Option<String>) -> Result<Self, CollectorError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("sourcefield-profile/0.3"),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(24))
            .build()?;

        Ok(Self {
            client,
            token: token.filter(|value| !value.trim().is_empty()),
            private_token: None,
            github_api: GITHUB_API.into(),
            graphql_api: GITHUB_GRAPHQL.into(),
            nuget_api: NUGET_INDEX.into(),
        })
    }

    /// Set credentials used exclusively for the explicitly requested owned-private count.
    pub fn with_private_token(mut self, token: Option<String>) -> Self {
        self.private_token = token.filter(|value| !value.trim().is_empty());
        self
    }

    /// Collect configured sources, marking every incomplete source and redacting failures.
    pub async fn collect(
        &self,
        config: &Config,
        private_counts: bool,
    ) -> Result<Snapshot, CollectorError> {
        let mut snapshot = Snapshot {
            mode: SnapshotMode::Live,
            fetched_at: Utc::now().to_rfc3339(),
            ..Snapshot::default()
        };

        snapshot.user = self
            .fetch_account(&config.collection.github_user, AccountKind::User)
            .await?;
        record(&mut snapshot, "github:user", true);

        for organization in &config.collection.github_organizations {
            match self
                .fetch_account(organization, AccountKind::Organization)
                .await
            {
                Ok(account) => {
                    snapshot.organizations.push(account);
                    record(&mut snapshot, &format!("github:org:{organization}"), true);
                }

                Err(_) => record(&mut snapshot, &format!("github:org:{organization}"), false),
            }
        }

        if config.collection.discover_public_repositories {
            let mut repositories_complete = true;
            let mut repositories = self
                .list_repositories(
                    &config.collection.github_user,
                    AccountKind::User,
                    config.collection.repository_limit,
                )
                .await?;

            for organization in &config.collection.github_organizations {
                match self
                    .list_repositories(
                        organization,
                        AccountKind::Organization,
                        config.collection.repository_limit,
                    )
                    .await
                {
                    Ok(mut values) => repositories.append(&mut values),
                    Err(_) => repositories_complete = false,
                }
            }

            repositories.retain(|repository| {
                (config.collection.include_forks || !repository.fork)
                    && (config.collection.include_archived || !repository.archived)
            });
            repositories.sort_by(|left, right| left.full_name.cmp(&right.full_name));
            repositories.dedup_by(|left, right| left.full_name == right.full_name);
            snapshot.repositories = repositories;
            record(&mut snapshot, "github:repositories", repositories_complete);
        }

        if config.collection.collect_contributions {
            match self
                .fetch_contributions(&config.collection.github_user)
                .await
            {
                Ok(contributions) => {
                    snapshot.contributions = Some(contributions);
                    record(&mut snapshot, "github:contributions", true);
                }

                _ => record(&mut snapshot, "github:contributions", false),
            }
        }

        if private_counts {
            match self
                .fetch_private_count(&config.collection.github_user)
                .await
            {
                Ok(count) => {
                    snapshot.private_repository_count = Some(count);
                    record(&mut snapshot, "github:private-count", true);
                }

                Err(_) => record(&mut snapshot, "github:private-count", false),
            }
        }

        snapshot.packages = self.collect_packages(config, &mut snapshot.warnings).await;
        for package in config
            .publications
            .iter()
            .flat_map(|publication| &publication.packages)
        {
            let complete = snapshot.packages.iter().any(|value| {
                value.id.eq_ignore_ascii_case(&package.id)
                    && value.version.is_some()
                    && value.total_downloads.is_some()
            });

            record(
                &mut snapshot,
                &format!("nuget:{}", package.id.to_ascii_lowercase()),
                complete,
            );
        }

        Ok(snapshot)
    }

    /// Count token-visible private repositories owned by the configured user.
    /// Organization repositories are excluded.
    async fn fetch_private_count(&self, login: &str) -> Result<u32, CollectorError> {
        let token = self
            .private_token
            .as_ref()
            .ok_or_else(|| CollectorError::Response("private-count credentials missing".into()))?;

        let query = concat!(
            "query($login:String!){user(login:$login){",
            "repositories(first:1,privacy:PRIVATE,ownerAffiliations:[OWNER]){totalCount}}}"
        );

        let body = json!({"query": query, "variables": {"login": login}});
        let response = self
            .client
            .post(&self.graphql_api)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .json(&body)
            .send()
            .await?;

        let value: Value = decode(response, &self.graphql_api).await?;
        if value.get("errors").is_some() {
            return Err(CollectorError::GraphQl);
        }

        value
            .pointer("/data/user/repositories/totalCount")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| CollectorError::Response("invalid private count".into()))
    }

    async fn fetch_account(
        &self,
        login: &str,
        kind: AccountKind,
    ) -> Result<AccountSnapshot, CollectorError> {
        let endpoint = match kind {
            AccountKind::User => format!("/users/{login}"),
            AccountKind::Organization => format!("/orgs/{login}"),
        };

        let value: GithubAccount = self.get_github(&endpoint).await?;
        Ok(AccountSnapshot {
            login: value.login,
            display_name: value.name,
            profile_url: Some(value.html_url),
            public_repositories: value.public_repos,
            followers: match (kind, value.followers) {
                (AccountKind::User, None) => {
                    return Err(CollectorError::Response(
                        "missing user follower count".into(),
                    ));
                }
                (_, followers) => followers.unwrap_or(0),
            },
        })
    }

    async fn list_repositories(
        &self,
        login: &str,
        kind: AccountKind,
        limit: usize,
    ) -> Result<Vec<RepositorySnapshot>, CollectorError> {
        let base = match kind {
            AccountKind::User => format!("/users/{login}/repos?type=owner&sort=pushed"),
            AccountKind::Organization => format!("/orgs/{login}/repos?type=public&sort=pushed"),
        };

        let mut repositories = Vec::new();
        let maximum = limit.clamp(1, 1_000);

        for page in 1..=10 {
            let endpoint = format!("{base}&per_page=100&page={page}");
            let values: Vec<GithubRepository> = self.get_github(&endpoint).await?;
            let page_size = values.len();
            repositories.extend(values.into_iter().map(RepositorySnapshot::from));
            if page_size < 100 || repositories.len() >= maximum {
                break;
            }
        }

        repositories.truncate(maximum);
        Ok(repositories)
    }

    async fn collect_packages(
        &self,
        config: &Config,
        warnings: &mut Vec<String>,
    ) -> Vec<PackageSnapshot> {
        let mut seen = BTreeSet::new();
        let mut packages = Vec::new();
        if config
            .publications
            .iter()
            .all(|publication| publication.packages.is_empty())
        {
            return packages;
        }

        // NuGet explicitly requires discovery: search hosts can move independently of the index.
        let endpoint = match self.nuget_search_endpoint().await {
            Ok(endpoint) => endpoint,
            Err(_) => {
                warnings.push("NuGet service discovery unavailable".into());

                return packages;
            }
        };

        for package in config
            .publications
            .iter()
            .flat_map(|publication| publication.packages.iter())
        {
            if !seen.insert(package.id.to_ascii_lowercase()) {
                continue;
            }

            let request = self.client.get(&endpoint).query(&[
                ("q", format!("packageid:{}", package.id)),
                ("prerelease", "false".to_string()),
                ("semVerLevel", "2.0.0".to_string()),
                ("take", "1".to_string()),
            ]);

            match request.send().await {
                Ok(response) => {
                    let url = response.url().to_string();
                    match decode::<NugetSearchResponse>(response, &url).await {
                        Ok(result) => {
                            if let Some(found) = result
                                .data
                                .into_iter()
                                .find(|value| value.id.eq_ignore_ascii_case(&package.id))
                            {
                                packages.push(PackageSnapshot {
                                    id: package.id.clone(),
                                    version: Some(found.version),
                                    total_downloads: found.total_downloads,
                                    updated_at: None,
                                    url: Some(package.url.clone()),
                                });
                            } else {
                                warnings.push(format!(
                                    "NuGet returned no exact match for {}",
                                    package.id
                                ));
                                packages.push(PackageSnapshot {
                                    id: package.id.clone(),
                                    url: Some(package.url.clone()),
                                    ..PackageSnapshot::default()
                                });
                            }
                        }

                        Err(_) => {
                            warnings.push(format!("NuGet metadata unavailable: {}", package.id));
                        }
                    }
                }

                Err(_) => warnings.push(format!("NuGet metadata unavailable: {}", package.id)),
            }
        }

        packages.sort_by(|left, right| left.id.cmp(&right.id));
        packages
    }

    /// Resolve the versioned NuGet search resource from the authoritative service index.
    async fn nuget_search_endpoint(&self) -> Result<String, CollectorError> {
        let response = self.client.get(&self.nuget_api).send().await?;
        let index: Value = decode(response, &self.nuget_api).await?;
        let endpoint = index
            .get("resources")
            .and_then(Value::as_array)
            .and_then(|resources| {
                resources.iter().find(|resource| {
                    resource
                        .get("@type")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| {
                            kind == "SearchQueryService" || kind.starts_with("SearchQueryService/")
                        })
                })
            })
            .and_then(|resource| resource.get("@id"))
            .and_then(Value::as_str)
            .ok_or_else(|| CollectorError::Response("NuGet search resource missing".into()))?;

        let parsed = reqwest::Url::parse(endpoint)
            .map_err(|_| CollectorError::Response("invalid NuGet resource URL".into()))?;

        let index_url = reqwest::Url::parse(&self.nuget_api)
            .map_err(|_| CollectorError::Response("invalid NuGet index URL".into()))?;

        if parsed.scheme() != "https"
            && !(index_url.scheme() == "http"
                && index_url.host_str() == Some("127.0.0.1")
                && parsed.host_str() == Some("127.0.0.1"))
        {
            return Err(CollectorError::Response(
                "NuGet resource requires HTTPS".into(),
            ));
        }

        Ok(endpoint.to_string())
    }

    async fn fetch_contributions(
        &self,
        login: &str,
    ) -> Result<ContributionSnapshot, CollectorError> {
        if self.token.is_none() {
            return Err(CollectorError::Response(
                "contribution credentials missing".into(),
            ));
        }

        let to = Utc::now();
        let from = to - Duration::days(364);
        let query = r#"
query SourcefieldProfile($login: String!, $from: DateTime!, $to: DateTime!) {
  user(login: $login) {
    contributionsCollection(from: $from, to: $to) {
      totalCommitContributions
      totalIssueContributions
      totalPullRequestContributions
      totalPullRequestReviewContributions
      restrictedContributionsCount
      contributionCalendar {
        totalContributions
        weeks {
          contributionDays {
            date
            contributionCount
            contributionLevel
          }
        }
      }
    }
  }
}

"#;

        let body = json!({
            "query": query,
            "variables": {
                "login": login,
                "from": from.to_rfc3339(),
                "to": to.to_rfc3339(),
            }
        });

        let response = self
            .authorized(self.client.post(&self.graphql_api).json(&body))
            .send()
            .await?;

        let value: Value = decode(response, &self.graphql_api).await?;
        if value.get("errors").is_some() {
            return Err(CollectorError::GraphQl);
        }

        let user = value
            .pointer("/data/user")
            .ok_or_else(|| CollectorError::Response("missing GraphQL user".to_string()))?;

        let collection = user.get("contributionsCollection").ok_or_else(|| {
            CollectorError::Response("missing contributionsCollection".to_string())
        })?;

        for path in [
            "/contributionCalendar/totalContributions",
            "/totalCommitContributions",
            "/totalIssueContributions",
            "/totalPullRequestContributions",
            "/totalPullRequestReviewContributions",
            "/restrictedContributionsCount",
        ] {
            if collection
                .pointer(path)
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .is_none()
            {
                return Err(CollectorError::Response(
                    "invalid contribution counter".into(),
                ));
            }
        }

        if collection
            .pointer("/contributionCalendar/weeks")
            .and_then(Value::as_array)
            .is_none()
        {
            return Err(CollectorError::Response(
                "missing contribution calendar".into(),
            ));
        }

        let mut days = Vec::new();
        for week in collection
            .pointer("/contributionCalendar/weeks")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let values = week
                .get("contributionDays")
                .and_then(Value::as_array)
                .ok_or_else(|| CollectorError::Response("missing contribution days".into()))?;

            for day in values {
                let date = day
                    .get("date")
                    .and_then(Value::as_str)
                    .filter(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok())
                    .ok_or_else(|| CollectorError::Response("invalid contribution date".into()))?;

                let count = day
                    .get("contributionCount")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or_else(|| CollectorError::Response("invalid contribution count".into()))?;

                let level = day
                    .get("contributionLevel")
                    .and_then(Value::as_str)
                    .filter(|value| {
                        matches!(
                            *value,
                            "NONE"
                                | "FIRST_QUARTILE"
                                | "SECOND_QUARTILE"
                                | "THIRD_QUARTILE"
                                | "FOURTH_QUARTILE"
                        )
                    })
                    .ok_or_else(|| CollectorError::Response("invalid contribution level".into()))?;

                days.push(ActivityDay {
                    date: date.to_string(),
                    count,
                    level: contribution_level(level),
                });
            }
        }

        days.sort_by(|left, right| left.date.cmp(&right.date));

        let contributions = ContributionSnapshot {
            total: u32_at(collection.pointer("/contributionCalendar/totalContributions")),
            commits: u32_at(collection.get("totalCommitContributions")),
            issues: u32_at(collection.get("totalIssueContributions")),
            pull_requests: u32_at(collection.get("totalPullRequestContributions")),
            reviews: u32_at(collection.get("totalPullRequestReviewContributions")),
            restricted: u32_at(collection.get("restrictedContributionsCount")),
            days,
        };

        Ok(contributions)
    }

    async fn get_github<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
    ) -> Result<T, CollectorError> {
        let url = format!("{}{endpoint}", self.github_api);
        let response = self.authorized(self.client.get(&url)).send().await?;
        decode(response, &url).await
    }

    fn authorized(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.token {
            request.header(AUTHORIZATION, format!("Bearer {token}"))
        } else {
            request
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AccountKind {
    User,
    Organization,
}

#[derive(Debug, Deserialize)]
struct GithubAccount {
    login: String,
    name: Option<String>,
    html_url: String,
    public_repos: u32,
    #[serde(default)]
    followers: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GithubOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GithubRepository {
    owner: GithubOwner,
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    language: Option<String>,
    #[serde(default)]
    topics: Vec<String>,
    #[serde(default)]
    stargazers_count: u32,
    #[serde(default)]
    forks_count: u32,
    #[serde(default)]
    archived: bool,
    #[serde(default)]
    fork: bool,
    pushed_at: Option<String>,
}

impl From<GithubRepository> for RepositorySnapshot {
    fn from(value: GithubRepository) -> Self {
        Self {
            owner: value.owner.login,
            name: value.name,
            full_name: value.full_name,
            url: value.html_url,
            description: value.description,
            primary_language: value.language,
            topics: value.topics,
            stars: value.stargazers_count,
            forks: value.forks_count,
            archived: value.archived,
            fork: value.fork,
            pushed_at: value.pushed_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NugetSearchResponse {
    #[serde(default)]
    data: Vec<NugetPackage>,
}

#[derive(Debug, Deserialize)]
struct NugetPackage {
    id: String,
    version: String,
    #[serde(default, rename = "totalDownloads")]
    total_downloads: Option<u64>,
}

async fn decode<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    _url: &str,
) -> Result<T, CollectorError> {
    let status = response.status();
    if !status.is_success() {
        return Err(CollectorError::Http { status });
    }

    Ok(response.json::<T>().await?)
}

fn contribution_level(value: &str) -> u8 {
    match value {
        "FIRST_QUARTILE" => 1,
        "SECOND_QUARTILE" => 2,
        "THIRD_QUARTILE" => 3,
        "FOURTH_QUARTILE" => 4,
        _ => 0,
    }
}

fn u32_at(value: Option<&Value>) -> u32 {
    value.and_then(Value::as_u64).unwrap_or_default() as u32
}

/// Publish a stable source code instead of untrusted upstream diagnostics.
fn record(snapshot: &mut Snapshot, source: &str, complete: bool) {
    snapshot.sources.push(SourceStatus {
        source: source.into(),
        status: if complete {
            DataStatus::Live
        } else {
            DataStatus::Missing
        },
    });
    if !complete {
        snapshot.mode = SnapshotMode::Partial;
        snapshot
            .warnings
            .push(format!("Source unavailable: {source}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
        thread,
    };

    /// Minimal real HTTP fixture; captured requests prove credential and query boundaries.
    fn fixture(
        responses: Vec<(u16, String)>,
    ) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let fixture_endpoint = endpoint.clone();
        let worker = thread::spawn(move || {
            for (status, body) in responses {
                let body = body.replace("FIXTURE_ENDPOINT", &fixture_endpoint);

                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                loop {
                    let mut chunk = [0; 4096];
                    let count = stream.read(&mut chunk).unwrap();
                    if count == 0 {
                        break;
                    }

                    bytes.extend_from_slice(&chunk[..count]);
                    if let Some(end) = bytes.windows(4).position(|value| value == b"\r\n\r\n") {
                        let header = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                        let length = header
                            .lines()
                            .find_map(|line| {
                                line.strip_prefix("content-length:")
                                    .and_then(|value| value.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);

                        if bytes.len() >= end + 4 + length {
                            break;
                        }
                    }
                }

                captured
                    .lock()
                    .unwrap()
                    .push(String::from_utf8(bytes).unwrap());
                write!(
                    stream,
                    concat!(
                        "HTTP/1.1 {} Fixture\r\nContent-Type: application/json\r\n",
                        "Content-Length: {}\r\nConnection: close\r\n\r\n{}"
                    ),
                    status,
                    body.len(),
                    body
                )
                .unwrap();
            }
        });
        (endpoint, requests, worker)
    }

    fn config() -> Config {
        let mut config = sourcefield_core::load_config(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config/profile.toml"
        ))
        .unwrap();

        config.collection.github_organizations.clear();
        config.collection.discover_public_repositories = false;
        config.collection.collect_contributions = false;
        config.publications.clear();
        config
    }

    fn account() -> String {
        json!({
            "login": "owner",
            "name": null,
            "html_url": "https://github.com/owner",
            "public_repos": 2,
            "followers": 0
        })
        .to_string()
    }

    #[tokio::test]
    async fn complete_public_collection_is_live_and_does_not_request_private_data() {
        let (endpoint, requests, worker) = fixture(vec![(200, account())]);
        let mut collector = Collector::new(None).unwrap();
        collector.github_api = endpoint;

        let snapshot = collector.collect(&config(), false).await.unwrap();

        worker.join().unwrap();
        assert_eq!(snapshot.mode, SnapshotMode::Live);
        assert_eq!(snapshot.private_repository_count, None);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn organization_failure_is_partial_and_response_secrets_are_not_persisted() {
        let (endpoint, _, worker) = fixture(vec![
            (200, account()),
            (403, "SECRET-TOKEN user@example.invalid".into()),
        ]);

        let mut collector = Collector::new(None).unwrap();
        collector.github_api = endpoint;
        let mut config = config();
        config
            .collection
            .github_organizations
            .push("organization".into());

        let snapshot = collector.collect(&config, false).await.unwrap();

        worker.join().unwrap();
        assert_eq!(snapshot.mode, SnapshotMode::Partial);
        let public = serde_json::to_string(&snapshot).unwrap();
        assert!(!public.contains("SECRET-TOKEN"));
        assert!(!public.contains("user@example.invalid"));
        assert!(public.contains("github:org:organization"));
    }

    #[tokio::test]
    async fn private_count_is_independent_and_uses_only_isolated_credentials() {
        let (endpoint, requests, worker) = fixture(vec![
            (200, account()),
            (
                200,
                json!({"data":{"user":{"repositories":{"totalCount":7}}}}).to_string(),
            ),
        ]);

        let mut collector = Collector::new(Some("public-secret".into()))
            .unwrap()
            .with_private_token(Some("private-secret".into()));

        collector.github_api = endpoint.clone();
        collector.graphql_api = endpoint;

        let snapshot = collector.collect(&config(), true).await.unwrap();

        worker.join().unwrap();
        assert_eq!(snapshot.private_repository_count, Some(7));
        assert!(snapshot.contributions.is_none());
        let requests = requests.lock().unwrap();
        assert!(requests[0].contains("public-secret"));
        assert!(!requests[0].contains("private-secret"));
        assert!(requests[1].contains("private-secret"));
        assert!(!requests[1].contains("public-secret"));
        assert!(requests[1].contains("ownerAffiliations:[OWNER]"));
        assert!(!requests[1].contains("contributionsCollection"));
    }

    #[tokio::test]
    async fn malformed_private_count_stays_unknown_and_partial() {
        let (endpoint, _, worker) = fixture(vec![
            (200, account()),
            (
                200,
                json!({"data":{"user":{"repositories":{"totalCount":-1}}}}).to_string(),
            ),
        ]);

        let mut collector = Collector::new(None)
            .unwrap()
            .with_private_token(Some("secret".into()));

        collector.github_api = endpoint.clone();
        collector.graphql_api = endpoint;

        let snapshot = collector.collect(&config(), true).await.unwrap();

        worker.join().unwrap();
        assert_eq!(snapshot.private_repository_count, None);
        assert_eq!(snapshot.mode, SnapshotMode::Partial);
    }

    #[tokio::test]
    async fn missing_contribution_token_marks_requested_source_missing() {
        let (endpoint, requests, worker) = fixture(vec![(200, account())]);
        let mut collector = Collector::new(None).unwrap();
        collector.github_api = endpoint;
        let mut config = config();
        config.collection.collect_contributions = true;

        let snapshot = collector.collect(&config, false).await.unwrap();

        worker.join().unwrap();
        assert_eq!(snapshot.mode, SnapshotMode::Partial);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn nuget_discovery_and_exact_match_are_required() {
        for exact in [true, false] {
            let index = json!({
                "resources": [{
                    "@type": "SearchQueryService/3.5.0",
                    "@id": "FIXTURE_ENDPOINT/query"
                }]
            })
            .to_string();

            let id = if exact {
                "Doka.Caching.MySql"
            } else {
                "Different.Package"
            };

            let response =
                json!({"data":[{"id":id,"version":"10.3.0","totalDownloads":42}]}).to_string();

            let (endpoint, requests, worker) =
                fixture(vec![(200, account()), (200, index), (200, response)]);

            let mut collector = Collector::new(None).unwrap();
            collector.github_api = endpoint.clone();
            collector.nuget_api = endpoint;
            let mut config = config();
            let full = sourcefield_core::load_config(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../config/profile.toml"
            ))
            .unwrap();

            let mut publication = full.publications[0].clone();
            publication
                .packages
                .retain(|package| package.id == "Doka.Caching.MySql");
            assert_eq!(publication.packages.len(), 1);
            config.publications.push(publication);

            let snapshot = collector.collect(&config, false).await.unwrap();

            worker.join().unwrap();
            assert_eq!(requests.lock().unwrap().len(), 3);
            assert_eq!(
                snapshot.mode,
                if exact {
                    SnapshotMode::Live
                } else {
                    SnapshotMode::Partial
                }
            );
            assert_eq!(
                snapshot.packages[0].version.as_deref(),
                if exact { Some("10.3.0") } else { None }
            );
        }
    }

    #[tokio::test]
    async fn contribution_response_requires_numeric_fields_and_redacts_graphql_errors() {
        let collection = json!({
            "totalCommitContributions": 1,
            "totalIssueContributions": 0,
            "totalPullRequestContributions": 0,
            "totalPullRequestReviewContributions": 0,
            "restrictedContributionsCount": 0,
            "contributionCalendar": {
                "totalContributions": 1,
                "weeks": [
                    {
                        "contributionDays": [
                            {
                                "date": "2026-09-05",
                                "contributionCount": 1,
                                "contributionLevel": "FIRST_QUARTILE"
                            }
                        ]
                    }
                ]
            }
        });

        let success = json!({"data":{"user":{"contributionsCollection":collection}}}).to_string();
        for (body, valid) in [
            (success.clone(), true),
            (success.replace("2026-09-05", "invalid-date"), false),
            (success.replace("FIRST_QUARTILE", "UNKNOWN"), false),
            (
                json!({"errors":[{"message":"secret-payload"}]}).to_string(),
                false,
            ),
            (
                json!({"data":{"user":{"contributionsCollection":{}}}}).to_string(),
                false,
            ),
        ] {
            let (endpoint, _, worker) = fixture(vec![(200, body)]);
            let mut collector = Collector::new(Some("token".into())).unwrap();
            collector.graphql_api = endpoint;

            let result = collector.fetch_contributions("owner").await;

            worker.join().unwrap();
            assert_eq!(result.is_ok(), valid);
            if let Err(error) = result {
                assert!(!format!("{error:?}").contains("secret-payload"));
            }
        }
    }
    #[tokio::test]
    async fn missing_user_counter_is_not_reported_as_zero() {
        let mut response: Value = serde_json::from_str(&account()).unwrap();
        response.as_object_mut().unwrap().remove("followers");
        let (endpoint, _, worker) = fixture(vec![(200, response.to_string())]);
        let mut collector = Collector::new(None).unwrap();
        collector.github_api = endpoint;

        let result = collector.collect(&config(), false).await;

        worker.join().unwrap();
        assert!(result.is_err());
    }
}
