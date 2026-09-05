use std::collections::BTreeSet;

use thiserror::Error;

use crate::{Config, NodeKind, ProfileState, Visibility};

/// A configuration or generated state violates the public graph contract.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Only the currently supported configuration schema can be interpreted.
    #[error("configuration version must be 1")]
    UnsupportedVersion,
    /// Identifiers must be unique within their graph namespace.
    #[error("duplicate identifier: {0}")]
    DuplicateId(String),
    /// A graph object references a domain that is not configured.
    #[error("unknown domain '{domain}' referenced by {owner}")]
    UnknownDomain {
        /// Missing domain identifier.
        domain: String,
        /// Object containing the invalid reference.
        owner: String,
    },
    /// A graph object references a technology that is not configured.
    #[error("unknown technology '{technology}' referenced by {owner}")]
    UnknownTechnology {
        /// Missing technology identifier.
        technology: String,
        /// Object containing the invalid reference.
        owner: String,
    },
    /// An edge references an absent endpoint.
    #[error("edge references missing node: {0}")]
    MissingNode(String),
    /// The design canvas is below the supported minimum.
    #[error("canvas must be at least 1200 x 640")]
    CanvasTooSmall,
    /// A field contains an unsafe, empty, or otherwise unsupported value.
    #[error("invalid value: {0}")]
    InvalidValue(String),
    /// Curated private project metadata cannot contain a repository URL.
    #[error("private project must not contain a repository URL: {0}")]
    PrivateUrl(String),
}

/// Check graph references, finite geometry, safe links, and the curated private-content boundary.
///
/// Project identities and endpoint counts are content, not schema invariants. This permits
/// legitimate additions and renames without changing validation code.
pub fn validate_config(config: &Config) -> Result<(), ValidationError> {
    if config.version != 1 {
        return Err(ValidationError::UnsupportedVersion);
    }

    if config.render.width < 1200 || config.render.height < 640 {
        return Err(ValidationError::CanvasTooSmall);
    }

    if config.render.motion_seconds == 0
        || !matches!(config.render.detail_level.as_str(), "abstract" | "detailed")
        || config.collection.repository_limit == 0
        || config.collection.repository_limit > 1000
        || config.collection.history_limit == 0
    {
        return Err(ValidationError::InvalidValue(
            "render or collection settings".into(),
        ));
    }

    safe_url(&config.profile.pages_url)?;
    safe_url(&config.profile.source_url)?;
    account(&config.profile.username)?;
    account(&config.profile.organization)?;
    account(&config.collection.github_user)?;
    for organization in &config.collection.github_organizations {
        account(organization)?;
    }

    let mut ids = BTreeSet::new();
    let domains = config
        .domains
        .iter()
        .map(|domain| domain.id.as_str())
        .collect::<BTreeSet<_>>();
    let technologies = config
        .technologies
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    for domain in &config.domains {
        insert(&mut ids, &format!("domain:{}", domain.id))?;
        account(&domain.owner)?;
        anchor(domain.anchor, config)?;
    }

    for technology in &config.technologies {
        insert(&mut ids, &format!("technology:{}", technology.id))?;
        for affinity in &technology.affinities {
            domain_reference(&domains, affinity, &technology.id)?;
        }
    }

    for project in &config.projects {
        insert(&mut ids, &format!("project:{}", project.id))?;
        domain_reference(&domains, &project.domain, &project.id)?;
        anchor(project.anchor, config)?;
        if !project.weight.is_finite()
            || !(0.0..=1.0).contains(&project.weight)
            || project
                .radius
                .is_some_and(|radius| !radius.is_finite() || !(1.0..=200.0).contains(&radius))
        {
            return Err(ValidationError::InvalidValue(
                "project weight or radius".into(),
            ));
        }

        if let Some(repository) = &project.repository {
            if project.visibility == Visibility::PrivateAbstract {
                return Err(ValidationError::PrivateUrl(project.id.clone()));
            }

            repository_name(repository)?;
        }

        for technology in project
            .implemented_with
            .iter()
            .chain(&project.integrates)
            .chain(&project.targets)
        {
            technology_reference(&technologies, technology, &project.id)?;
        }

        for component in &project.components {
            insert(
                &mut ids,
                &format!("component:{}:{}", project.id, component.id),
            )?;
            for technology in component.integrates.iter().chain(&component.targets) {
                technology_reference(&technologies, technology, &component.id)?;
            }
        }
    }

    let mut packages = BTreeSet::new();
    for publication in &config.publications {
        insert(&mut ids, &format!("publication:{}", publication.id))?;
        domain_reference(&domains, &publication.domain, &publication.id)?;
        anchor(publication.anchor, config)?;
        for technology in &publication.technologies {
            technology_reference(&technologies, technology, &publication.id)?;
        }

        for package in &publication.packages {
            insert(&mut packages, &package.id.to_ascii_lowercase())?;
            safe_url(&package.url)?;
            if let Some(value) = package.anchor {
                anchor(value, config)?;
            }
        }
    }

    let interests = config
        .interests
        .iter()
        .map(|interest| interest.id.as_str())
        .collect::<BTreeSet<_>>();
    for interest in &config.interests {
        insert(&mut ids, &format!("interest:{}", interest.id))?;
    }

    for learning in &config.learning {
        insert(&mut ids, &format!("learning:{}", learning.id))?;
        safe_url(&learning.url)?;
        if !interests.contains(learning.interest.as_str()) {
            return Err(ValidationError::InvalidValue(
                "learning interest reference".into(),
            ));
        }
    }

    Ok(())
}

/// Validate state before it reaches renderers or browser consumers.
///
/// HTTPS alone is deliberately required for links; arbitrary schemes could execute code when
/// a state node is rendered as an anchor. Text still requires output-context escaping.
pub fn validate_state(state: &ProfileState) -> Result<(), ValidationError> {
    if state.canvas.width < 1200 || state.canvas.height < 640 || state.canvas.motion_seconds == 0 {
        return Err(ValidationError::CanvasTooSmall);
    }

    safe_url(&state.profile.pages_url)?;
    safe_url(&state.profile.source_url)?;
    let mut node_ids = BTreeSet::new();
    for node in &state.nodes {
        insert(&mut node_ids, &node.id)?;
        if !node.x.is_finite()
            || !node.y.is_finite()
            || !node.radius.is_finite()
            || !node.weight.is_finite()
            || node.radius <= 0.0
            || node.x < 0.0
            || node.y < 0.0
            || node.x > state.canvas.width as f32
            || node.y > state.canvas.height as f32
        {
            return Err(ValidationError::InvalidValue("node geometry".into()));
        }

        if let Some(url) = &node.url {
            if node.visibility == Some(Visibility::PrivateAbstract)
                && matches!(node.kind, NodeKind::Project | NodeKind::Component)
            {
                return Err(ValidationError::PrivateUrl(node.id.clone()));
            }

            safe_url(url)?;
        }
    }

    for edge in &state.edges {
        if !node_ids.contains(&edge.from) {
            return Err(ValidationError::MissingNode(edge.from.clone()));
        }

        if !node_ids.contains(&edge.to) {
            return Err(ValidationError::MissingNode(edge.to.clone()));
        }

        if !edge.weight.is_finite() || !(0.0..=1.0).contains(&edge.weight) {
            return Err(ValidationError::InvalidValue("edge weight".into()));
        }
    }

    for package in &state.packages {
        if let Some(url) = &package.url {
            safe_url(url)?;
        }
    }

    for learning in &state.learning {
        safe_url(&learning.url)?;
    }

    Ok(())
}

/// Restrict authored outbound links to unambiguous absolute HTTPS URLs without credentials.
fn safe_url(value: &str) -> Result<(), ValidationError> {
    let valid = value.strip_prefix("https://").is_some_and(|rest| {
        let host = rest.split('/').next().unwrap_or_default();
        !host.is_empty()
            && host.contains('.')
            && host
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte))
            && !value.chars().any(|character| {
                character.is_control()
                    || character.is_whitespace()
                    || matches!(character, '\\' | '<' | '>' | '"' | '\'')
            })
    });

    if !valid {
        return Err(ValidationError::InvalidValue("HTTPS link".into()));
    }

    Ok(())
}

/// GitHub identities are path segments, never URL fragments or arbitrary paths.
fn account(value: &str) -> Result<(), ValidationError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(ValidationError::InvalidValue("GitHub account".into()));
    }

    Ok(())
}

/// Validate the owner/repository shorthand before constructing its public URL.
fn repository_name(value: &str) -> Result<(), ValidationError> {
    let Some((owner, name)) = value.split_once('/') else {
        return Err(ValidationError::InvalidValue("repository name".into()));
    };

    account(owner)?;
    if name.is_empty()
        || matches!(name, "." | "..")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
    {
        return Err(ValidationError::InvalidValue("repository name".into()));
    }

    Ok(())
}

/// Reject invalid authored coordinates rather than silently altering the intended composition.
fn anchor(value: [f32; 2], config: &Config) -> Result<(), ValidationError> {
    if !value[0].is_finite()
        || !value[1].is_finite()
        || value[0] < 0.0
        || value[1] < 0.0
        || value[0] > config.render.width as f32
        || value[1] > config.render.height as f32
    {
        return Err(ValidationError::InvalidValue("anchor".into()));
    }

    Ok(())
}

fn domain_reference(
    domains: &BTreeSet<&str>,
    domain: &str,
    owner: &str,
) -> Result<(), ValidationError> {
    if !domains.contains(domain) {
        return Err(ValidationError::UnknownDomain {
            domain: domain.into(),
            owner: owner.into(),
        });
    }

    Ok(())
}

fn technology_reference(
    technologies: &BTreeSet<&str>,
    technology: &str,
    owner: &str,
) -> Result<(), ValidationError> {
    if !technologies.contains(technology) {
        return Err(ValidationError::UnknownTechnology {
            technology: technology.into(),
            owner: owner.into(),
        });
    }

    Ok(())
}

fn insert(ids: &mut BTreeSet<String>, id: &str) -> Result<(), ValidationError> {
    if id.is_empty() || id.ends_with(':') || id.chars().any(char::is_control) {
        return Err(ValidationError::InvalidValue("identifier".into()));
    }

    if !ids.insert(id.to_string()) {
        return Err(ValidationError::DuplicateId(id.to_string()));
    }

    Ok(())
}
