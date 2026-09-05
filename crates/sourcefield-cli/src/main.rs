//! Collect, validate, and publish reproducible profile artifacts.

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sourcefield_collector::Collector;
use sourcefield_core::{
    DataStatus, ProfileState, Snapshot, SnapshotMode, build_state, load_config, validate_config,
    validate_state,
};
use sourcefield_render::{Theme, render_svg, write_outputs};

#[derive(Debug, Parser)]
#[command(name = "sourcefield")]
#[command(about = "Generate a deterministic GitHub profile topology")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Collect public GitHub and NuGet signals, analyze the topology and render all assets.
    Generate {
        #[arg(long, default_value = "config/profile.toml")]
        config: PathBuf,
        #[arg(long, default_value = "config/offline-snapshot.json")]
        fallback_snapshot: PathBuf,
        #[arg(long, default_value = "assets")]
        assets: PathBuf,
        #[arg(long, default_value = "docs")]
        docs: PathBuf,
        #[arg(long)]
        offline: bool,
        #[arg(long)]
        strict_live: bool,
        #[arg(long)]
        private_counts: bool,
        #[arg(long)]
        no_history: bool,
    },
    /// Validate configuration, semantic state and generated SVG assets.
    Validate {
        #[arg(long, default_value = "config/profile.toml")]
        config: PathBuf,
        #[arg(long, default_value = "assets/profile-state.json")]
        state: PathBuf,
        #[arg(long, default_value = "assets")]
        assets: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate {
            config,
            fallback_snapshot,
            assets,
            docs,
            offline,
            strict_live,
            private_counts,
            no_history,
        } => {
            generate(GenerateOptions {
                config_path: &config,
                fallback_snapshot_path: &fallback_snapshot,
                assets_dir: &assets,
                docs_dir: &docs,
                offline,
                strict_live,
                private_counts,
                no_history,
            })
            .await
        }

        Command::Validate {
            config,
            state,
            assets,
        } => validate(&config, &state, &assets),
    }
}

/// Explicit generation policy replaces positional Boolean arguments.
struct GenerateOptions<'a> {
    config_path: &'a Path,
    fallback_snapshot_path: &'a Path,
    assets_dir: &'a Path,
    docs_dir: &'a Path,
    offline: bool,
    strict_live: bool,
    private_counts: bool,
    no_history: bool,
}

async fn generate(options: GenerateOptions<'_>) -> Result<()> {
    let GenerateOptions {
        config_path,
        fallback_snapshot_path,
        assets_dir,
        docs_dir,
        offline,
        strict_live,
        private_counts,
        no_history,
    } = options;

    if offline && strict_live {
        bail!("--offline and --strict-live are mutually exclusive");
    }

    let config = load_config(config_path).context("load profile configuration")?;
    validate_config(&config).context("validate profile configuration")?;
    let fallback = read_json::<Snapshot>(fallback_snapshot_path)
        .context("load checked-in fallback snapshot")?;

    let private_counts_requested = private_counts
        || env_flag("SOURCEFIELD_PRIVATE_COUNTS")
        || config.collection.collect_private_repository_count;

    let profile_token = first_non_empty_env(&["PROFILE_TOKEN"]);
    let include_private_counts = private_counts_requested && profile_token.is_some();
    let private_count_requested_without_token = private_counts_requested && profile_token.is_none();

    let mut snapshot = if offline {
        let mut snapshot = fallback.clone();

        snapshot.mode = SnapshotMode::Preview;
        snapshot.fetched_at.clear();
        for source in &mut snapshot.sources {
            source.status = DataStatus::Preview;
        }

        snapshot.warnings.push(
            "Offline preview: public signals are replaced on the first successful live workflow."
                .to_string(),
        );
        snapshot
    } else {
        let token = first_non_empty_env(&["GH_TOKEN", "GITHUB_TOKEN"]);
        let collector = Collector::new(token)
            .context("create public signal collector")?
            .with_private_token(profile_token.clone());

        match collector.collect(&config, include_private_counts).await {
            Ok(snapshot) => {
                require_live(&snapshot, strict_live)?;
                snapshot
            }

            Err(_) if !strict_live => {
                let mut snapshot = fallback.clone();
                snapshot.mode = SnapshotMode::Fallback;
                for source in &mut snapshot.sources {
                    source.status = DataStatus::Fallback;
                }

                snapshot.fetched_at.clear();
                snapshot
                    .warnings
                    .push("Live collection failed; using the dated fallback snapshot.".to_string());
                snapshot
            }

            Err(error) => return Err(error).context("collect live public signals"),
        }
    };

    apply_private_count_policy(
        &mut snapshot,
        private_counts_requested,
        profile_token.is_some(),
    );

    if private_count_requested_without_token {
        snapshot.warnings.push(
            concat!(
                "Aggregate private repository counts were requested but PROFILE_TOKEN is not ",
                "configured; the private signal remains sealed."
            )
            .to_string(),
        );
    }

    require_live(&snapshot, strict_live && private_counts_requested)?;

    let mut state = build_state(&config, &snapshot, Utc::now().to_rfc3339())
        .context("build semantic profile state")?;

    validate_state(&state).context("validate semantic profile state")?;

    preserve_generation_time(&mut state, &assets_dir.join("profile-state.json"))?;
    // Fail before changing outputs when an existing archive cannot be trusted.
    load_history(&docs_dir.join("history"))?;

    let _generation_lock = DirectoryLock::acquire(docs_dir)?;
    let files = write_outputs(&state, assets_dir).context("write SVG and state assets")?;
    // Retrieval time is operational noise; provenance mode and observed values remain verifiable.
    snapshot.fetched_at.clear();
    fs::write(
        assets_dir.join("source-snapshot.json"),
        serde_json::to_string_pretty(&snapshot)?,
    )
    .context("write public source inputs for reproducible validation")?;
    fs::create_dir_all(docs_dir).context("create GitHub Pages directory")?;
    for path in [&files.dark_svg, &files.light_svg, &files.static_svg] {
        let name = path.file_name().context("SVG filename missing")?;
        fs::copy(path, docs_dir.join(name)).context("copy SVG into GitHub Pages")?;
    }

    fs::copy(&files.state_json, docs_dir.join("profile-state.json"))
        .context("copy state into GitHub Pages")?;
    fs::write(docs_dir.join(".nojekyll"), "").context("write .nojekyll")?;
    fs::write(
        docs_dir.join("build-meta.json"),
        serde_json::to_string_pretty(&BuildMeta::from_state(&state))?,
    )
    .context("write build metadata")?;

    if !no_history && state.mode == SnapshotMode::Live {
        update_history(&state, docs_dir, config.collection.history_limit)
            .context("update interactive state history")?;
    } else {
        ensure_empty_history_index(docs_dir).context("ensure history index")?;
    }

    validate(config_path, &files.state_json, assets_dir)?;

    println!("SOURCEFIELD {}", state.semantic_hash);
    println!("  mode:     {:?}", state.mode);
    println!("  nodes:    {}", state.nodes.len());
    println!("  edges:    {}", state.edges.len());
    println!("  packages: {}", state.stats.package_count);
    println!("  dark:     {}", files.dark_svg.display());
    println!("  light:    {}", files.light_svg.display());
    println!("  static:   {}", files.static_svg.display());
    println!("  state:    {}", files.state_json.display());
    Ok(())
}

fn validate(config_path: &Path, state_path: &Path, assets_dir: &Path) -> Result<()> {
    let config = load_config(config_path).context("load profile configuration")?;
    validate_config(&config).context("validate profile configuration")?;
    let state = read_json::<ProfileState>(state_path).context("load semantic state")?;
    validate_state(&state).context("validate semantic state")?;
    let snapshot: Snapshot = read_json(&assets_dir.join("source-snapshot.json"))
        .context("load recorded public source inputs")?;

    let reconstructed = build_state(&config, &snapshot, state.generated_at.clone())?;
    if serde_json::to_value(&reconstructed)? != serde_json::to_value(&state)? {
        bail!("semantic state does not match configuration and recorded source inputs");
    }

    for name in [
        "sourcefield.dark.svg",
        "sourcefield.light.svg",
        "sourcefield.static.svg",
    ] {
        let path = assets_dir.join(name);
        let svg = fs::read_to_string(&path)
            .with_context(|| format!("read generated SVG {}", path.display()))?;

        let theme = if name == "sourcefield.light.svg" {
            Theme::Light
        } else {
            Theme::Dark
        };

        let motion = name != "sourcefield.static.svg";
        validate_svg(&svg, &state, theme, motion)?;
        if svg.len() > 900_000 {
            bail!("{} exceeds the 900 KB README asset budget", path.display());
        }
    }

    let configured_packages = config
        .publications
        .iter()
        .flat_map(|publication| publication.packages.iter())
        .map(|package| package.id.as_str())
        .collect::<BTreeSet<_>>();

    if configured_packages.len() != state.stats.package_count as usize {
        bail!("package count in state does not match configuration");
    }

    println!(
        "Validated {} nodes, {} edges and all generated SVG assets.",
        state.nodes.len(),
        state.edges.len()
    );
    Ok(())
}

/// Validate generated provenance, not arbitrary SVG via an incomplete payload denylist.
fn validate_svg(svg: &str, state: &ProfileState, theme: Theme, motion: bool) -> Result<()> {
    if svg != render_svg(state, theme, motion) {
        bail!("SVG differs from the canonical renderer for the validated state");
    }

    Ok(())
}

/// Clear even dated fallback counts unless both explicit intent and credentials are present.
fn apply_private_count_policy(snapshot: &mut Snapshot, requested: bool, token_present: bool) {
    if !requested || !token_present {
        snapshot.private_repository_count = None;
    }
}

/// Strict mode accepts only complete live collection, without fallback substitution.
fn require_live(snapshot: &Snapshot, strict: bool) -> Result<()> {
    if strict && (snapshot.mode != SnapshotMode::Live || !snapshot.warnings.is_empty()) {
        bail!("strict-live requires every requested source to be complete and live");
    }

    Ok(())
}

/// Preserve content timestamps when semantic inputs and observable provenance are unchanged.
fn preserve_generation_time(state: &mut ProfileState, path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let previous_json: serde_json::Value = read_json(path)?;
    match previous_json
        .get("schema")
        .and_then(serde_json::Value::as_u64)
    {
        // Schema 1 is the explicitly superseded renderer contract, not a timestamp source.
        Some(1) => return Ok(()),
        Some(version) if version == u64::from(state.schema) => {}
        _ => bail!("existing state has an unsupported or missing schema"),
    }

    let previous: ProfileState = serde_json::from_value(previous_json)
        .context("parse current-schema state before preserving its generation timestamp")?;
    validate_state(&previous).context("validate existing current-schema state")?;
    let mut candidate = state.clone();
    candidate.generated_at.clone_from(&previous.generated_at);
    if serde_json::to_value(&candidate)? == serde_json::to_value(&previous)? {
        state.generated_at = previous.generated_at;
    }

    Ok(())
}

/// Reject traversal, links, duplicate entries and inconsistent immutable archive metadata.
fn load_history(directory: &Path) -> Result<HistoryIndex> {
    if directory.is_symlink() {
        bail!("history directory must not be a symlink");
    }

    let path = directory.join("index.json");
    if path.is_symlink() {
        bail!("history index must not be a symlink");
    }

    if !path.exists() {
        return Ok(HistoryIndex::default());
    }

    let index: HistoryIndex = read_json(&path)?;
    let mut hashes = BTreeSet::new();
    for entry in &index.states {
        validate_history_name(&entry.hash, &entry.file)?;
        if !hashes.insert(&entry.hash) {
            bail!("history contains duplicate hashes");
        }

        let archive = directory.join(&entry.file);
        if archive.is_symlink() {
            bail!("history archive must not be a symlink");
        }

        let state: ProfileState = read_json(&archive)?;
        validate_state(&state).context("validate archived semantic state")?;
        if state.semantic_hash != entry.hash
            || state.generated_at != entry.generated_at
            || state.nodes.len() != entry.node_count
            || state.edges.len() != entry.edge_count
        {
            bail!("history metadata does not match archived state");
        }

        chrono::DateTime::parse_from_rfc3339(&entry.generated_at)
            .context("invalid archive timestamp")?;
    }

    Ok(index)
}

fn validate_history_name(hash: &str, file: &str) -> Result<()> {
    if hash.len() != 16
        || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        || file != format!("{}.json", hash.to_ascii_lowercase())
    {
        bail!("invalid history hash or filename");
    }

    Ok(())
}

/// Archive semantic content once; retain only indexed files after committing the new index.
fn update_history(state: &ProfileState, docs_dir: &Path, limit: usize) -> Result<()> {
    let history_dir = docs_dir.join("history");
    let _history_lock = DirectoryLock::acquire(&history_dir)?;
    let mut index = load_history(&history_dir)?;
    let state_file = format!("{}.json", state.semantic_hash.to_ascii_lowercase());
    validate_history_name(&state.semantic_hash, &state_file)?;
    chrono::DateTime::parse_from_rfc3339(&state.generated_at)?;
    fs::create_dir_all(&history_dir)?;

    if !index
        .states
        .iter()
        .any(|entry| entry.hash == state.semantic_hash)
    {
        let state_path = history_dir.join(&state_file);
        if state_path.exists() || state_path.is_symlink() {
            bail!("unindexed archive already exists; preserve it for manual recovery");
        }

        use std::io::Write;

        let mut archive = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&state_path)?;

        archive.write_all(serde_json::to_string_pretty(state)?.as_bytes())?;
        archive.sync_all()?;
        index.states.push(HistoryEntry {
            hash: state.semantic_hash.clone(),
            generated_at: state.generated_at.clone(),
            file: state_file,
            node_count: state.nodes.len(),
            edge_count: state.edges.len(),
        });
    }

    index.states.sort_by(|left, right| {
        // RFC3339 offsets can differ; compare instants rather than lexical text.
        let left_time = chrono::DateTime::parse_from_rfc3339(&left.generated_at).ok();
        let right_time = chrono::DateTime::parse_from_rfc3339(&right.generated_at).ok();
        right_time
            .cmp(&left_time)
            .then_with(|| left.hash.cmp(&right.hash))
    });
    let removed = index.states.split_off(index.states.len().min(limit.max(1)));
    let index_path = history_dir.join("index.json");
    write_atomic(
        &index_path,
        serde_json::to_string_pretty(&index)?.as_bytes(),
    )?;
    // Unindexed files may belong to an interrupted write or a user; never sweep them.
    for entry in removed {
        fs::remove_file(history_dir.join(entry.file))?;
    }

    Ok(())
}

/// Reject concurrent writers instead of silently losing another generation's index update.
struct DirectoryLock(PathBuf);

impl DirectoryLock {
    fn acquire(directory: &Path) -> Result<Self> {
        if directory.is_symlink() {
            bail!("output directory must not be a symlink");
        }

        fs::create_dir_all(directory)?;
        let path = directory.join(".sourcefield.lock");
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .context("output is locked; inspect an interrupted writer before removing its lock")?;
        Ok(Self(path))
    }
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.0) {
            eprintln!("could not release output lock: {error}");
        }
    }
}

/// Create a same-directory temporary file exclusively before the atomic replacement.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;

    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn ensure_empty_history_index(docs_dir: &Path) -> Result<()> {
    let directory = docs_dir.join("history");
    load_history(&directory)?;
    fs::create_dir_all(&directory)?;
    let path = directory.join("index.json");
    if !path.exists() {
        fs::write(path, "{\n  \"states\": []\n}\n")?;
    }

    Ok(())
}

fn first_non_empty_env(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| env::var(name).ok().filter(|value| !value.trim().is_empty()))
}

fn env_flag(name: &str) -> bool {
    env::var(name).is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read JSON file {}", path.display()))?;

    serde_json::from_str(&text).with_context(|| format!("parse JSON file {}", path.display()))
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct HistoryIndex {
    #[serde(default)]
    states: Vec<HistoryEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryEntry {
    hash: String,
    generated_at: String,
    file: String,
    node_count: usize,
    edge_count: usize,
}

#[derive(Debug, Serialize)]
struct BuildMeta<'a> {
    generator: &'static str,
    semantic_hash: &'a str,
    generated_at: &'a str,
    mode: SnapshotMode,
    node_count: usize,
    edge_count: usize,
}

impl<'a> BuildMeta<'a> {
    fn from_state(state: &'a ProfileState) -> Self {
        Self {
            generator: "sourcefield-rust",
            semantic_hash: &state.semantic_hash,
            generated_at: &state.generated_at,
            mode: state.mode,
            node_count: state.nodes.len(),
            edge_count: state.edges.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TemporaryDirectory(PathBuf);

    impl TemporaryDirectory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = env::temp_dir().join(format!(
                "sourcefield-cli-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));

            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn state() -> ProfileState {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = load_config(root.join("config/profile.toml")).unwrap();
        let snapshot: Snapshot = read_json(&root.join("config/offline-snapshot.json")).unwrap();
        build_state(&config, &snapshot, "2026-09-05T12:00:00Z").unwrap()
    }

    #[test]
    fn strict_live_accepts_complete_and_rejects_partial_preview_fallback_and_warnings() {
        let mut snapshot = Snapshot {
            mode: SnapshotMode::Live,
            ..Snapshot::default()
        };

        assert!(require_live(&snapshot, true).is_ok());
        for mode in [
            SnapshotMode::Partial,
            SnapshotMode::Preview,
            SnapshotMode::Fallback,
        ] {
            snapshot.mode = mode;
            assert!(require_live(&snapshot, true).is_err());
            assert!(require_live(&snapshot, false).is_ok());
        }

        snapshot.mode = SnapshotMode::Live;
        snapshot
            .warnings
            .push("requested private count unavailable".into());
        assert!(require_live(&snapshot, true).is_err());
    }

    #[test]
    fn identical_state_preserves_timestamp_but_content_change_does_not() {
        let directory = TemporaryDirectory::new();
        let original = state();
        let path = directory.0.join("state.json");
        fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
        let mut current = original.clone();
        current.generated_at = "2026-09-06T12:00:00Z".into();

        preserve_generation_time(&mut current, &path).unwrap();

        assert_eq!(current.generated_at, original.generated_at);
        current.warnings.push("new source condition".into());
        current.generated_at = "2026-09-06T12:00:00Z".into();
        preserve_generation_time(&mut current, &path).unwrap();
        assert_ne!(current.generated_at, original.generated_at);
    }

    #[test]
    fn repeated_history_preserves_original_timestamp_and_bytes() {
        let directory = TemporaryDirectory::new();
        let mut current = state();
        update_history(&current, &directory.0, 2).unwrap();
        let path = directory.0.join("history/index.json");
        let original = fs::read(&path).unwrap();
        current.generated_at = "2026-09-06T12:00:00Z".into();

        update_history(&current, &directory.0, 2).unwrap();

        assert_eq!(fs::read(path).unwrap(), original);
        assert!(load_history(&directory.0.join("history")).is_ok());
    }

    #[test]
    fn corrupted_index_fails_without_deleting_or_creating_archives() {
        let directory = TemporaryDirectory::new();
        let history = directory.0.join("history");
        fs::create_dir(&history).unwrap();
        fs::write(history.join("index.json"), "{broken").unwrap();
        fs::write(history.join("unrelated.json"), "preserve").unwrap();

        let result = update_history(&state(), &directory.0, 1);

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(history.join("index.json")).unwrap(),
            "{broken"
        );
        assert_eq!(
            fs::read_to_string(history.join("unrelated.json")).unwrap(),
            "preserve"
        );
        assert_eq!(fs::read_dir(history).unwrap().count(), 2);
    }

    #[test]
    fn retention_orders_instants_and_preserves_unindexed_files() {
        let directory = TemporaryDirectory::new();
        let mut first = state();
        first.semantic_hash = "1111111111111111".into();
        first.generated_at = "2026-09-05T13:00:00+02:00".into();
        update_history(&first, &directory.0, 2).unwrap();
        let history = directory.0.join("history");
        fs::write(history.join("unrelated.json"), "preserve").unwrap();
        let mut second = first.clone();
        second.semantic_hash = "2222222222222222".into();
        second.generated_at = "2026-09-05T12:00:00Z".into();

        update_history(&second, &directory.0, 1).unwrap();

        let index = load_history(&history).unwrap();
        assert_eq!(index.states.len(), 1);
        assert_eq!(index.states[0].hash, second.semantic_hash);
        assert!(!history.join("1111111111111111.json").exists());
        assert!(history.join("unrelated.json").exists());
    }

    #[test]
    fn traversal_and_wrong_archive_metadata_are_rejected() {
        assert!(validate_history_name("0123456789ABCDEF", "0123456789abcdef.json").is_ok());
        assert!(validate_history_name("../../escape", "../../escape.json").is_err());
        assert!(validate_history_name("0123456789ABCDEF", "../0123456789abcdef.json").is_err());

        let directory = TemporaryDirectory::new();
        let original = state();
        update_history(&original, &directory.0, 1).unwrap();
        let history = directory.0.join("history");
        let archive = history.join(format!(
            "{}.json",
            original.semantic_hash.to_ascii_lowercase()
        ));

        let mut tampered = original;
        tampered.generated_at = "2020-01-01T00:00:00Z".into();
        fs::write(archive, serde_json::to_vec(&tampered).unwrap()).unwrap();

        assert!(load_history(&history).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_archive_is_rejected_and_target_preserved() {
        let directory = TemporaryDirectory::new();
        let outside = TemporaryDirectory::new();
        let original = state();
        update_history(&original, &directory.0, 1).unwrap();
        let history = directory.0.join("history");
        let archive = history.join(format!(
            "{}.json",
            original.semantic_hash.to_ascii_lowercase()
        ));

        let target = outside.0.join("target.json");
        fs::rename(&archive, &target).unwrap();
        std::os::unix::fs::symlink(&target, &archive).unwrap();

        assert!(update_history(&original, &directory.0, 1).is_err());
        assert!(target.exists());
    }

    #[test]
    fn canonical_svg_passes_but_event_handler_and_css_injections_fail() {
        let current = state();
        let svg = render_svg(&current, Theme::Dark, true);

        assert!(validate_svg(&svg, &current, Theme::Dark, true).is_ok());
        for payload in [
            " onload=\"alert(1)\"",
            " style=\"fill:url(https://evil.invalid/x)\"",
        ] {
            let tampered = svg.replacen("<svg", &format!("<svg{payload}"), 1);
            assert!(validate_svg(&tampered, &current, Theme::Dark, true).is_err());
        }
    }

    #[test]
    fn writer_lock_rejects_concurrent_writer_and_releases_on_drop() {
        let directory = TemporaryDirectory::new();
        let first = DirectoryLock::acquire(&directory.0).unwrap();

        assert!(DirectoryLock::acquire(&directory.0).is_err());
        drop(first);

        assert!(DirectoryLock::acquire(&directory.0).is_ok());
    }

    #[tokio::test]
    async fn offline_generation_is_byte_identical_and_validation_rejects_state_tampering() {
        let directory = TemporaryDirectory::new();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = root.join("config/profile.toml");
        let fallback = root.join("config/offline-snapshot.json");
        let assets = directory.0.join("assets");
        let docs = directory.0.join("docs");
        fs::create_dir(&assets).unwrap();
        let mut legacy = serde_json::to_value(state()).unwrap();
        legacy["schema"] = serde_json::json!(1);
        legacy["canvas"]
            .as_object_mut()
            .unwrap()
            .remove("show_activity_orbit");
        fs::write(
            assets.join("profile-state.json"),
            serde_json::to_vec(&legacy).unwrap(),
        )
        .unwrap();

        let options = || GenerateOptions {
            config_path: &config,
            fallback_snapshot_path: &fallback,
            assets_dir: &assets,
            docs_dir: &docs,
            offline: true,
            strict_live: false,
            private_counts: false,
            no_history: false,
        };

        generate(options()).await.unwrap();
        let paths = [
            assets.join("profile-state.json"),
            assets.join("source-snapshot.json"),
            assets.join("sourcefield.dark.svg"),
            assets.join("sourcefield.light.svg"),
            assets.join("sourcefield.static.svg"),
            docs.join("profile-state.json"),
            docs.join("sourcefield.dark.svg"),
            docs.join("build-meta.json"),
            docs.join("history/index.json"),
        ];

        let original: Vec<_> = paths.iter().map(|path| fs::read(path).unwrap()).collect();

        generate(options()).await.unwrap();

        let repeated: Vec<_> = paths.iter().map(|path| fs::read(path).unwrap()).collect();
        assert_eq!(original, repeated);
        let path = assets.join("profile-state.json");
        let mut tampered: ProfileState = read_json(&path).unwrap();
        tampered.nodes[0].label.push_str(" changed");
        fs::write(&path, serde_json::to_vec(&tampered).unwrap()).unwrap();
        assert!(validate(&config, &path, &assets).is_err());
    }
    #[test]
    fn previous_schema_is_rebuilt_but_malformed_current_schema_is_rejected() {
        let directory = TemporaryDirectory::new();
        let path = directory.0.join("state.json");
        let mut current = state();
        let expected_time = current.generated_at.clone();
        let mut legacy = serde_json::to_value(&current).unwrap();
        legacy["schema"] = serde_json::json!(1);
        legacy["generated_at"] = serde_json::json!("2020-01-01T00:00:00Z");
        legacy["canvas"]
            .as_object_mut()
            .unwrap()
            .remove("show_activity_orbit");
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();

        preserve_generation_time(&mut current, &path).unwrap();

        assert_eq!(current.generated_at, expected_time);
        legacy["schema"] = serde_json::json!(current.schema);
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(preserve_generation_time(&mut current, &path).is_err());
        legacy["schema"] = serde_json::json!(999);
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(preserve_generation_time(&mut current, &path).is_err());
        fs::write(&path, "{broken").unwrap();
        assert!(preserve_generation_time(&mut current, &path).is_err());
    }
    #[test]
    fn archived_geometry_and_edge_references_are_validated_before_retention() {
        for invalid_geometry in [true, false] {
            let directory = TemporaryDirectory::new();
            let original = state();
            update_history(&original, &directory.0, 1).unwrap();
            let history = directory.0.join("history");
            let archive = history.join(format!(
                "{}.json",
                original.semantic_hash.to_ascii_lowercase()
            ));
            let index_before = fs::read(history.join("index.json")).unwrap();
            let mut invalid = original.clone();
            if invalid_geometry {
                invalid.nodes[0].x = -1.0;
            } else {
                invalid.edges[0].from = "missing-node".into();
            }

            let bytes = serde_json::to_vec(&invalid).unwrap();
            fs::write(&archive, &bytes).unwrap();

            assert!(update_history(&original, &directory.0, 1).is_err());
            assert_eq!(fs::read(&archive).unwrap(), bytes);
            assert_eq!(fs::read(history.join("index.json")).unwrap(), index_before);
        }
    }
    #[test]
    fn explicit_private_opt_in_survives_config_default_but_disabled_fallback_is_cleared() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut config = load_config(root.join("config/profile.toml")).unwrap();
        config.collection.collect_private_repository_count = false;
        let mut snapshot = Snapshot {
            private_repository_count: Some(27),
            ..Snapshot::default()
        };

        apply_private_count_policy(&mut snapshot, true, true);
        let enabled = build_state(&config, &snapshot, "2026-09-05T12:00:00Z").unwrap();

        assert_eq!(enabled.stats.private_repository_count, Some(27));

        snapshot.mode = SnapshotMode::Fallback;
        apply_private_count_policy(&mut snapshot, false, true);
        let disabled = build_state(&config, &snapshot, "2026-09-05T12:00:00Z").unwrap();

        assert_eq!(disabled.stats.private_repository_count, None);
        snapshot.private_repository_count = Some(27);
        apply_private_count_policy(&mut snapshot, true, false);
        assert_eq!(snapshot.private_repository_count, None);
    }
}
