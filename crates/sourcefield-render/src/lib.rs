//! Deterministic, data-driven SVG presentation for SOURCEFIELD.
#![deny(missing_docs)]

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

mod geometry;
mod ornaments;

use anyhow::{Context, Result};
use geometry::{Curve, Point};
use sourcefield_core::{EdgeKind, Node, NodeKind, ProfileState, SnapshotMode, Visibility};

/// Color scheme for an independently usable SVG artifact.
#[derive(Debug, Clone, Copy)]
pub enum Theme {
    /// Low-luminance profile presentation.
    Dark,
    /// High-luminance presentation with dark readable labels.
    Light,
}

/// Paths emitted by [`write_outputs`].
#[derive(Debug)]
pub struct OutputFiles {
    /// Animated dark image.
    pub dark_svg: PathBuf,
    /// Animated light image.
    pub light_svg: PathBuf,
    /// Dark image without animation declarations.
    pub static_svg: PathBuf,
    /// State shared by the browser and simulator.
    pub state_json: PathBuf,
}

/// Semantic colors keep light-theme text independent of decorative opacity.
#[derive(Clone, Copy)]
struct Palette {
    background: &'static str,
    secondary: &'static str,
    surface: &'static str,
    text: &'static str,
    muted: &'static str,
    quiet: &'static str,
    grid: &'static str,
    mint: &'static str,
    purple: &'static str,
    amber: &'static str,
    blue: &'static str,
}

impl Theme {
    /// Resolve explicit contrast-aware values rather than inverting the dark image.
    fn palette(self) -> Palette {
        match self {
            Self::Dark => Palette {
                background: "#050911",
                secondary: "#0c1222",
                surface: "#0d1425",
                text: "#edf2ff",
                muted: "#a2acc8",
                quiet: "#93a3bf",
                grid: "#3f4e6a",
                mint: "#4de7c2",
                purple: "#a898ff",
                amber: "#ffbd7a",
                blue: "#7acfff",
            },
            Self::Light => Palette {
                background: "#f5f7fc",
                secondary: "#e9eef8",
                surface: "#ffffff",
                text: "#11182b",
                muted: "#47536d",
                quiet: "#53617b",
                grid: "#8592aa",
                mint: "#006c59",
                purple: "#6243b5",
                amber: "#92500b",
                blue: "#17628e",
            },
        }
    }
}

/// Write all presentation flavors and their shared serialized state.
///
/// Existing identical files are retained so a repeated build does not change timestamps.
/// Filesystem failures are returned with their destination path.
pub fn write_outputs(state: &ProfileState, directory: impl AsRef<Path>) -> Result<OutputFiles> {
    let directory = directory.as_ref();
    fs::create_dir_all(directory)
        .with_context(|| format!("create output directory {}", directory.display()))?;

    let files = OutputFiles {
        dark_svg: directory.join("sourcefield.dark.svg"),
        light_svg: directory.join("sourcefield.light.svg"),
        static_svg: directory.join("sourcefield.static.svg"),
        state_json: directory.join("profile-state.json"),
    };

    write_changed(&files.dark_svg, &render_svg(state, Theme::Dark, true))?;
    write_changed(&files.light_svg, &render_svg(state, Theme::Light, true))?;
    write_changed(&files.static_svg, &render_svg(state, Theme::Dark, false))?;
    write_changed(&files.state_json, &serde_json::to_string_pretty(state)?)?;
    Ok(files)
}

/// Preserve byte-identical artifacts without hiding read failures.
fn write_changed(path: &Path, value: &str) -> Result<()> {
    match fs::read(path) {
        Ok(existing) if existing == value.as_bytes() => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    }

    fs::write(path, value).with_context(|| format!("write {}", path.display()))
}

/// Render the approved field composition from public state.
///
/// Labels never move; motion is restricted to decorative rings and ownership traces.
/// `motion = false` omits animation rules entirely. Configuration controls optional modules.
/// State produced by `sourcefield_core` supplies validated coordinates and HTTPS links.
pub fn render_svg(state: &ProfileState, theme: Theme, motion: bool) -> String {
    let palette = theme.palette();
    let mut output = String::with_capacity(48_000);
    let width = state.canvas.width;
    let height = state.canvas.height;
    let sx = width as f32 / 1800.0;
    let sy = height as f32 / 1680.0;
    let nodes = state
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    let _ = write!(
        output,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" \
         height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\" \
         aria-labelledby=\"title description\"><title id=\"title\">{} - \
         SOURCEFIELD</title><desc id=\"description\">{}. Project connections \
         group ownership, not implementation dependencies.</desc>",
        xml(&state.profile.username),
        xml(&state.profile.tagline)
    );
    definitions(&mut output, palette, motion, state);
    let _ = write!(
        output,
        "<g clip-path=\"url(#frame)\"><rect width=\"{width}\" \
         height=\"{height}\" fill=\"url(#bg)\"/><rect width=\"{width}\" \
         height=\"{height}\" fill=\"url(#grid)\"/>"
    );

    // Normalize decorative typography only; semantic node coordinates remain in canvas units.
    let _ = write!(output, r#"<g transform="scale({sx} {sy})">"#);
    for index in 0..85 {
        let x = 35 + (index * 193 + 89) % 1730;
        let y = 235 + (index * 137 + 31) % 665;
        let _ = write!(
            output,
            r#"<circle cx="{x}" cy="{y}" r=".8" fill="{}" opacity=".22"/>"#,
            palette.quiet
        );
    }

    header(&mut output, state, palette);
    output.push_str("</g>");
    field(&mut output, state, &nodes, palette);
    let _ = write!(output, r#"<g transform="scale({sx} {sy})">"#);
    publications(&mut output, state, &nodes, palette);
    personal(&mut output, state, palette);
    output.push_str("</g></g></svg>");
    output
}

/// Emit self-contained SVG resources; no network fonts or executable SVG content are required.
fn definitions(output: &mut String, p: Palette, motion: bool, state: &ProfileState) {
    let width = state.canvas.width;
    let height = state.canvas.height;
    let seconds = state.canvas.motion_seconds;

    let _ = write!(
        output,
        "<defs><linearGradient id=\"bg\" x2=\"1\" y2=\"1\"><stop \
         stop-color=\"{}\"/><stop offset=\".48\" stop-color=\"{}\"/><stop \
         offset=\"1\" stop-color=\"{}\"/></linearGradient><pattern id=\"grid\" \
         width=\"32\" height=\"32\" patternUnits=\"userSpaceOnUse\"><path \
         d=\"M32 0H0V32\" fill=\"none\" stroke=\"{}\" stroke-opacity=\".16\" \
         stroke-width=\".6\"/></pattern><filter id=\"glow\" x=\"-100%\" \
         y=\"-100%\" width=\"300%\" height=\"300%\"><feGaussianBlur \
         stdDeviation=\"2.4\" result=\"b\"/><feMerge><feMergeNode in=\"b\"/>\
         <feMergeNode in=\"SourceGraphic\"/></feMerge></filter><clipPath \
         id=\"frame\"><rect width=\"{width}\" height=\"{height}\" rx=\"28\"/>\
         </clipPath>",
        p.background, p.secondary, p.background, p.grid
    );
    for (name, color) in [("mint", p.mint), ("purple", p.purple), ("amber", p.amber)] {
        let _ = write!(
            output,
            "<radialGradient id=\"{name}\"><stop stop-color=\"{color}\" \
             stop-opacity=\".13\"/><stop offset=\"1\" stop-color=\"{color}\" \
             stop-opacity=\"0\"/></radialGradient>"
        );
    }

    for (id, first, second) in [
        ("personal-gradient", p.mint, p.purple),
        ("organization-gradient", p.amber, p.blue),
    ] {
        let _ = write!(
            output,
            "<linearGradient id=\"{id}\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\">\
             <stop offset=\"0\" stop-color=\"{first}\"/>\
             <stop offset=\"1\" stop-color=\"{second}\"/></linearGradient>"
        );
    }

    // The bridge intentionally keeps the original artwork's four color stops.
    output.push_str(concat!(
        "<linearGradient id=\"bridge-gradient\"><stop offset=\"0\" ",
        "stop-color=\"#4DE7C2\" stop-opacity=\".15\"/><stop offset=\".44\" ",
        "stop-color=\"#8B7CFF\"/><stop offset=\".55\" stop-color=\"#D582FF\"/>",
        "<stop offset=\"1\" stop-color=\"#FFB86B\"/></linearGradient>"
    ));

    let _ = write!(
        output,
        "</defs><style>text{{font-family:-apple-system,BlinkMacSystemFont,'Segoe \
         UI',sans-serif}}.mono{{font-family:Menlo,Consolas,\
         monospace}}.project:focus,.package:focus{{outline:2px solid {};\
         outline-offset:4px}}.project:hover text,.package:hover \
         text{{fill:{}}}.paused \
         *{{animation-play-state:paused!important}}</style>",
        p.purple, p.text
    );
    if motion {
        let duration = seconds.max(12);
        let orbit = duration + 26;

        // Keep standalone delays in a style block: Pages removes it before DOM parsing,
        // then restores numeric timing from data attributes without CSP inline-style violations.
        output.push_str("<style>");
        for (kind, satellite, step) in [
            (NodeKind::Project, "personal-project", 0.8),
            (NodeKind::Package, "nuget-package", 0.65),
        ] {
            let count = state
                .nodes
                .iter()
                .filter(|node| node.kind == kind && node.show_in_readme)
                .count();

            for index in 0..count {
                let delay = -(index as f64 * step);
                let value = if kind == NodeKind::Project {
                    format!("{delay:.1}")
                } else {
                    format!("{delay:.2}")
                };
                let _ = write!(
                    output,
                    ".signal[data-satellite=\"{satellite}\"][data-signal-delay=\"{value}\"]\
                     {{animation-delay:{value}s}}"
                );
            }
        }

        output.push_str("</style>");

        let _ = write!(
            output,
            "<style>.rotate{{animation:orbit {orbit}s linear infinite;\
             transform-origin:0 0}}.reverse{{animation-direction:reverse}}.flow{{stro\
             ke-dasharray:3 21;animation:flow {duration}s linear \
             infinite}}.pulse{{animation:pulse 6s ease-in-out infinite}}\
             .scan{{animation-duration:24s}}.signal{{animation:signal 3.6s ease-in-out infinite}}\
             @keyframes signal{{0%,100%{{opacity:.18}}45%{{opacity:1}}}}@keyframes \
             orbit{{to{{transform:rotate(360deg)}}}}@keyframes \
             flow{{to{{stroke-dashoffset:-240}}}}@keyframes pulse{{0%,\
             100%{{opacity:.25}}50%{{opacity:.85}}}}@media(prefers-reduced-motion:red\
             uce){{.rotate,.flow,.pulse,.signal{{animation:none!important}}}}</style>"
        );
    }
}

/// Layout chrome uses normalized design coordinates so canvas resizing cannot clip its footer.
fn header(output: &mut String, state: &ProfileState, p: Palette) {
    text(
        output,
        (64.0, 56.0),
        "SOURCEFIELD",
        12,
        p.quiet,
        "start",
        "mono",
    );
    text(
        output,
        (64.0, 120.0),
        &state.profile.username,
        52,
        p.text,
        "start",
        "identity",
    );
    text(
        output,
        (66.0, 156.0),
        &state.profile.tagline,
        19,
        p.muted,
        "start",
        "",
    );
    let status = match state.mode {
        SnapshotMode::Live => "LIVE",
        SnapshotMode::Preview => "PREVIEW",
        SnapshotMode::Partial => "PARTIAL",
        SnapshotMode::Fallback => "FALLBACK",
    };

    text(output, (1736.0, 57.0), status, 11, p.quiet, "end", "mono");
    if state.canvas.show_technology_labels {
        text(
            output,
            (1736.0, 112.0),
            &state.presentation.main_stack.join(" / "),
            18,
            p.text,
            "end",
            "mono",
        );
        text(
            output,
            (1736.0, 143.0),
            &state.presentation.supporting_stack.join(" / "),
            13,
            p.muted,
            "end",
            "mono",
        );
    }

    if state.canvas.show_state_hash {
        text(
            output,
            (1736.0, 179.0),
            &state.semantic_hash,
            10,
            p.quiet,
            "end",
            "mono",
        );
    }

    path(output, "M64 194H1736", p.grid, ".45", "");
}

/// Draw ownership relationships using the same coordinates exposed to browser selection.
fn field(output: &mut String, state: &ProfileState, nodes: &BTreeMap<&str, &Node>, p: Palette) {
    let scale = state.canvas.width as f32 / 1800.0;
    for domain in state
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Domain && node.show_in_readme)
    {
        let color = node_color(domain, p);
        let organization = domain.scope.as_deref() == Some("organization");
        let heading = if organization {
            domain.label.to_uppercase()
        } else {
            "PERSONAL PROJECTS".to_string()
        };

        let x = if organization {
            state.canvas.width as f32 - 266.0 * scale
        } else {
            68.0 * scale
        };

        text(
            output,
            (x, 251.0 * state.canvas.height as f32 / 1680.0),
            &heading,
            11,
            color,
            "start",
            "mono",
        );
        let gradient = if domain.scope.as_deref() == Some("organization") {
            "amber"
        } else {
            "mint"
        };

        let _ = write!(
            output,
            r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="url(#{gradient})"/>"#,
            domain.x,
            domain.y,
            480.0 * scale,
            400.0 * state.canvas.height as f32 / 1680.0
        );
        if state.canvas.show_activity_orbit {
            let _ = write!(
                output,
                "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"none\" \
                 stroke=\"{color}\" stroke-opacity=\".25\" class=\"flow\"/>",
                domain.x,
                domain.y,
                373.0 * scale,
                307.0 * state.canvas.height as f32 / 1680.0
            );
        }
    }

    let mut domains = state
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Domain && node.show_in_readme)
        .collect::<Vec<_>>();

    domains.sort_by(|a, b| a.x.total_cmp(&b.x));

    let ports = departure_ports(state, &domains);
    for pair in domains.windows(2) {
        let left = pair[0];
        let right = pair[1];
        let departure = ports.get(&(left.id.as_str(), right.id.as_str())).copied();
        let d = connection_path(left, right, state, departure, true);

        if let Some(d) = d {
            // This bridge joins ownership fields; it does not assert a code dependency.
            output.push_str("<g data-connection=\"domain-bridge\">");
            path(output, &d, "url(#bridge-gradient)", ".20", "");
            path(output, &d, "url(#bridge-gradient)", ".65", "flow");
            output.push_str("</g>");
        }
    }

    let mut edges = state
        .edges
        .iter()
        .filter(|edge| edge.show_in_readme && edge.kind == EdgeKind::Contains)
        .collect::<Vec<_>>();

    edges.sort_by(|a, b| (&a.from, &a.to).cmp(&(&b.from, &b.to)));

    for edge in edges {
        let (Some(from), Some(to)) = (nodes.get(edge.from.as_str()), nodes.get(edge.to.as_str()))
        else {
            continue;
        };

        if from.kind != NodeKind::Domain
            || to.kind != NodeKind::Project
            || !from.show_in_readme
            || !to.show_in_readme
        {
            continue;
        }

        let departure = ports.get(&(from.id.as_str(), to.id.as_str())).copied();
        let Some(d) = connection_path(from, to, state, departure, false) else {
            continue;
        };

        let _ = write!(
            output,
            r#"<g data-edge-id="{}:{}" data-from="{}" data-to="{}">"#,
            xml(&edge.from),
            xml(&edge.to),
            xml(&edge.from),
            xml(&edge.to)
        );
        path(output, &d, node_color(from, p), ".16", "");
        path(output, &d, node_color(from, p), ".55", "flow");
        output.push_str("</g>");
    }

    for node in nodes.values().filter(|node| {
        node.show_in_readme && matches!(node.kind, NodeKind::Domain | NodeKind::Project)
    }) {
        project(output, node, state, p);
    }
}

/// Fit account departure ports to visible ownership neighbors and decorative bridges.
fn departure_ports<'a>(
    state: &'a ProfileState,
    domains: &[&'a Node],
) -> BTreeMap<(&'a str, &'a str), f64> {
    let sx = state.canvas.width as f64 / 1800.0;
    let sy = state.canvas.height as f64 / 1680.0;
    let mut result = BTreeMap::new();

    for domain in domains
        .iter()
        .filter(|node| node.scope.as_deref() != Some("organization"))
    {
        let mut neighbors = state
            .nodes
            .iter()
            .filter(|node| {
                node.show_in_readme
                    && node.kind == NodeKind::Project
                    && state.edges.iter().any(|edge| {
                        edge.show_in_readme
                            && edge.kind == EdgeKind::Contains
                            && edge.from == domain.id
                            && edge.to == node.id
                    })
            })
            .collect::<Vec<_>>();

        for pair in domains.windows(2).filter(|pair| pair[0].id == domain.id) {
            neighbors.push(pair[1]);
        }

        let mut angles = neighbors
            .into_iter()
            .map(|node| {
                let angle =
                    ((node.y - domain.y) as f64 / sy).atan2((node.x - domain.x) as f64 / sx);
                (angle, node.id.as_str())
            })
            .collect::<Vec<_>>();
        angles.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(b.1)));
        let fitted = geometry::ports(&angles.iter().map(|entry| entry.0).collect::<Vec<_>>());

        for ((_, target), angle) in angles.into_iter().zip(fitted) {
            result.insert((domain.id.as_str(), target), angle);
        }
    }

    result
}

/// Work in design coordinates so radial connections also match scaled elliptical ornaments.
fn connection_path(
    from: &Node,
    to: &Node,
    state: &ProfileState,
    departure: Option<f64>,
    bridge: bool,
) -> Option<String> {
    let sx = state.canvas.width as f64 / 1800.0;
    let sy = state.canvas.height as f64 / 1680.0;
    let a = Point(from.x as f64 / sx, from.y as f64 / sy);
    let b = Point(to.x as f64 / sx, to.y as f64 / sy);
    let controls = if bridge {
        [
            Point(a.0 + 304.0, a.1 - 197.0),
            Point(b.0 - 267.0, b.1 + 219.0),
        ]
    } else {
        [
            Point(a.0 + (b.0 - a.0) * 0.55, a.1 - 80.0),
            Point(b.0 - 40.0, b.1 + 65.0),
        ]
    };

    Curve::between(
        (a, from.radius as f64 / sx),
        (b, to.radius as f64 / sx),
        controls,
        departure,
    )
    .map(|curve| curve.path(sx, sy))
}

/// Assign glyph colors from declared visual roles, never from project identities.
fn node_color(node: &Node, p: Palette) -> &'static str {
    match node
        .scope
        .as_deref()
        .filter(|scope| *scope == "organization")
        .or(node.visual.as_deref())
    {
        Some("spatial" | "dotfiles" | "provider") => p.blue,
        Some("trace" | "theme" | "lab") => p.purple,
        Some("organization" | "migrations" | "packages") => p.amber,
        _ => p.mint,
    }
}

/// Keep ornaments inside a translated child, leaving labels stable during rotation.
fn project(output: &mut String, node: &Node, state: &ProfileState, p: Palette) {
    let sx = state.canvas.width as f32 / 1800.0;
    let sy = state.canvas.height as f32 / 1680.0;
    let actual = (node.x, node.y);
    let mut design_node = node.clone();
    design_node.x = 0.0;
    design_node.y = 0.0;
    design_node.radius /= sx;
    let node = &design_node;
    let color = node_color(node, p);
    let domain = node.kind == NodeKind::Domain;
    let kind = if domain { "domain" } else { "project" };
    let _ = write!(
        output,
        "<g class=\"project\" tabindex=\"0\" role=\"group\" aria-label=\"{}: \
         {}\" data-node-id=\"{}\" data-node-kind=\"{kind}\" data-domain=\"{}\" \
         data-x=\"{}\" data-y=\"{}\">",
        xml(&node.label),
        xml(&node.summary),
        xml(&node.id),
        xml(node.domain.as_deref().unwrap_or("")),
        actual.0,
        actual.1
    );
    let _ = write!(
        output,
        r#"<g transform="translate({} {}) scale({sx} {sy})">"#,
        actual.0, actual.1
    );
    if state.canvas.show_details && !node.details.is_empty() {
        let _ = write!(output, "<title>{}</title>", xml(&node.details.join("; ")));
    }

    let linked = open_link(output, node.url.as_deref());
    let radius = node.radius;
    output.push_str(
        "<g data-node-decoration=\"true\" transform=\"translate(0 0)\" aria-hidden=\"true\">",
    );
    ornaments::render(output, node, state, p);

    output.push_str("</g>");
    let mut y = node.y + if domain { 91.0 } else { radius + 25.0 };
    if let Some(prefix) = &node.label_prefix {
        text(output, (node.x, y), prefix, 12, p.quiet, "middle", "mono");
        y += 26.0;
    }

    text(
        output,
        (node.x, y),
        &node.surface_label,
        if domain { 21 } else { 23 },
        p.text,
        "middle",
        "node-label",
    );
    if domain {
        let projects = state
            .nodes
            .iter()
            .filter(|other| {
                other.show_in_readme
                    && other.kind == NodeKind::Project
                    && other.domain == node.domain
            })
            .collect::<Vec<_>>();

        let public = projects
            .iter()
            .filter(|other| other.visibility == Some(Visibility::Public))
            .count();
        let private = projects.len() - public;
        let summary = if public == 0 {
            format!("{private} private projects")
        } else {
            format!("{public} public / {private} private")
        };

        text(
            output,
            (node.x, y + 23.0),
            &summary,
            12,
            p.muted,
            "middle",
            "mono",
        );
    } else {
        for (index, line) in node.summary.lines().enumerate() {
            y += if index == 0 { 24.0 } else { 20.0 };
            text(
                output,
                (node.x, y),
                line,
                14,
                p.muted,
                "middle",
                "node-summary",
            );
        }

        if state.canvas.show_technology_labels {
            text(
                output,
                (node.x, y + 23.0),
                &node.display_stack.join(" / "),
                12,
                color,
                "middle",
                "mono",
            );
        }
    }

    if linked {
        output.push_str("</a>");
    }

    output.push_str("</g></g>");
}

/// Group linked package rows by their publication parent rather than inferred name prefixes.
fn publications(
    output: &mut String,
    state: &ProfileState,
    nodes: &BTreeMap<&str, &Node>,
    p: Palette,
) {
    let groups = nodes
        .values()
        .filter(|node| node.kind == NodeKind::Publication && node.show_in_readme)
        .collect::<Vec<_>>();

    let packages = nodes
        .values()
        .filter(|node| node.kind == NodeKind::Package && node.show_in_readme)
        .count();
    output.push_str("<g data-module=\"publications\">");
    path(output, "M64 937H1736", p.grid, ".5", "");
    text(
        output,
        (66.0, 977.0),
        &format!("NUGET / {packages} PACKAGES"),
        11,
        p.quiet,
        "start",
        "mono",
    );
    let sx = state.canvas.width as f32 / 1800.0;
    let sy = state.canvas.height as f32 / 1680.0;

    for (index, group) in groups.iter().enumerate() {
        let x = group.x / sx;
        let color = if index % 2 == 0 { p.blue } else { p.amber };
        text(
            output,
            (x, group.y / sy),
            &group.label,
            18,
            color,
            "start",
            "",
        );
        let mut children = state
            .edges
            .iter()
            .filter(|edge| edge.from == group.id && edge.kind == EdgeKind::Publishes)
            .filter_map(|edge| nodes.get(edge.to.as_str()).copied())
            .filter(|node| node.kind == NodeKind::Package && node.show_in_readme)
            .collect::<Vec<_>>();

        children.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.id.cmp(&b.id)));
        let mut previous = None;
        for node in &children {
            let x = node.x / sx;
            let y = node.y / sy;
            let start = previous.unwrap_or(group.y / sy + 17.0);

            if start < y - 16.0 {
                let d = format!("M{x} {start}V{}", y - 16.0);
                path(output, &d, p.quiet, ".28", "package-connector");
            }

            previous = Some(y + 16.0);
        }

        for (row, node) in children.into_iter().enumerate() {
            let direction = if row % 2 == 1 {
                "rotate reverse"
            } else {
                "rotate"
            };
            let x = node.x / sx - 19.0;
            let y = node.y / sy;
            let _ = write!(
                output,
                "<g class=\"package\" data-node-id=\"{}\" data-node-kind=\"package\" \
                 data-domain=\"{}\" tabindex=\"0\" role=\"group\" aria-label=\"{} on \
                 NuGet\">",
                xml(&node.id),
                xml(node.domain.as_deref().unwrap_or("")),
                xml(&node.label)
            );
            let linked = open_link(output, node.url.as_deref());
            let _ = write!(
                output,
                "<g transform=\"translate({} {y})\" aria-hidden=\"true\"><g \
                 class=\"{direction}\" data-package-row=\"{row}\"><circle r=\"16\" fill=\"none\" \
                 stroke=\"{color}\" stroke-dasharray=\"2 6\"/></g>",
                x + 19.0
            );
            path(output, "M0 -7L6 -3V4L0 8L-6 4V-3Z", color, ".9", "");
            output.push_str("</g>");
            text(
                output,
                (x + 53.0, y + 5.0),
                &node.label,
                15,
                p.text,
                "start",
                "mono",
            );
            text(
                output,
                (x + 53.0, y + 30.0),
                &node.summary,
                14,
                p.muted,
                "start",
                "",
            );
            if linked {
                output.push_str("</a>");
            }

            output.push_str("</g>");
        }
    }

    output.push_str("</g>");
}

/// Render curated personal context without exposing repository implementation metadata.
fn personal(output: &mut String, state: &ProfileState, p: Palette) {
    path(output, "M64 1337H1736", p.grid, ".5", "");
    path(output, "M615 1373V1596M1201 1373V1596", p.grid, ".4", "");
    output.push_str("<g data-module=\"interests\">");
    if state.canvas.show_interests_in_readme {
        text(
            output,
            (66.0, 1381.0),
            "OTHER INTERESTS",
            11,
            p.quiet,
            "start",
            "mono",
        );
        let interests = state
            .interests
            .iter()
            .filter(|interest| interest.show_in_readme);

        for (row, interest) in interests.enumerate() {
            let y = 1422.0 + row as f32 * 46.0;
            text(
                output,
                (66.0, y),
                &interest.label,
                20,
                if row == 0 { p.text } else { p.purple },
                "start",
                "",
            );
            if row > 0 {
                text(
                    output,
                    (66.0, y + 25.0),
                    &interest.summary,
                    15,
                    p.muted,
                    "start",
                    "",
                );
            }
        }
    }

    text(
        output,
        (66.0, 1542.0),
        &state.presentation.platforms.join(" / "),
        19,
        p.text,
        "start",
        "",
    );
    text(
        output,
        (66.0, 1572.0),
        &state.presentation.platform_note,
        15,
        p.muted,
        "start",
        "",
    );
    output.push_str("</g><g data-module=\"learning\">");
    text(
        output,
        (647.0, 1381.0),
        "LEARNING",
        11,
        p.quiet,
        "start",
        "mono",
    );
    for (index, learning) in state.learning.iter().enumerate() {
        let linked = open_link(output, Some(&learning.url));
        text(
            output,
            (647.0, 1424.0 + index as f32 * 40.0),
            &learning.label,
            22,
            p.purple,
            "start",
            "",
        );
        if linked {
            output.push_str("</a>");
        }
    }

    for (index, line) in state.presentation.learning_note.lines().enumerate() {
        text(
            output,
            (647.0, 1510.0 + index as f32 * 30.0),
            line,
            15,
            p.muted,
            "start",
            "",
        );
    }

    output.push_str("</g><g data-module=\"hardware\">");
    text(
        output,
        (1233.0, 1381.0),
        "AT HOME",
        11,
        p.quiet,
        "start",
        "mono",
    );
    for (index, hardware) in state.presentation.hardware.iter().enumerate() {
        let y = 1422.0 + index as f32 * 67.0;
        text(
            output,
            (1233.0, y),
            &hardware.label,
            20,
            p.text,
            "start",
            "",
        );
        text(
            output,
            (1233.0, y + 27.0),
            &hardware.detail,
            15,
            p.muted,
            "start",
            "mono",
        );
    }

    output.push_str("</g>");
    path(output, "M64 1620H1736", p.grid, ".4", "");
    text(
        output,
        (64.0, 1651.0),
        &format!("github.com/{}", state.profile.username),
        12,
        p.quiet,
        "start",
        "mono",
    );
    text(
        output,
        (1736.0, 1651.0),
        &format!("github.com/{}", state.profile.organization),
        12,
        p.quiet,
        "end",
        "mono",
    );
}

/// Encode text and attributes through one escaping boundary.
fn text(
    output: &mut String,
    at: (f32, f32),
    value: &str,
    size: u32,
    color: &str,
    anchor: &str,
    class: &str,
) {
    let weight = if matches!(class, "identity" | "node-label") {
        600
    } else {
        400
    };

    let _ = write!(
        output,
        "<text x=\"{}\" y=\"{}\" text-anchor=\"{anchor}\" font-size=\"{size}\" \
         fill=\"{color}\" font-weight=\"{weight}\" class=\"{class}\">{}</text>",
        at.0,
        at.1,
        xml(value)
    );
}

/// Emit decorative paths whose geometry is generated internally.
fn path(output: &mut String, d: &str, color: &str, opacity: &str, class: &str) {
    let _ = write!(
        output,
        "<path d=\"{d}\" fill=\"none\" stroke=\"{color}\" \
         stroke-opacity=\"{opacity}\" stroke-width=\"1.2\" stroke-linecap=\"butt\" class=\"{class}\"/>"
    );
}

/// Refuse executable or protocol-relative links even for manually constructed state.
fn open_link(output: &mut String, url: Option<&str>) -> bool {
    let Some(url) =
        url.filter(|url| url.starts_with("https://") && !url.chars().any(char::is_control))
    else {
        return false;
    };

    let _ = write!(
        output,
        r#"<a href="{}" target="_blank" rel="noopener noreferrer">"#,
        xml(url)
    );
    true
}

/// Escape all XML metacharacters, including quotes used by accessible labels.
fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build fixtures through the production model so renderer tests exercise schema changes.
    fn fixture() -> (sourcefield_core::Config, ProfileState) {
        let config = sourcefield_core::load_config(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/profile.toml"),
        )
        .expect("approved configuration must load");

        let state = sourcefield_core::build_state(
            &config,
            &sourcefield_core::Snapshot::default(),
            "fixture",
        )
        .expect("fixture state must build");

        (config, state)
    }

    #[test]
    fn approved_inventory_and_copy_are_rendered() {
        let (_, state) = fixture();

        let svg = render_svg(&state, Theme::Dark, true);

        assert_eq!(svg.matches("data-node-kind=\"project\"").count(), 8);
        assert_eq!(svg.matches("data-node-kind=\"package\"").count(), 6);
        assert!(svg.contains("6 MCP endpoints"));
        assert!(svg.contains("encrypted storage and backups"));
        assert!(svg.contains("AMD Ryzen"));
        assert!(svg.contains("AI Max+ 395"));
        assert!(!svg.contains("SOURCEFIELD / 03"));
    }

    #[test]
    fn hidden_renamed_added_and_reordered_projects_are_data_driven() {
        let (mut config, _) = fixture();
        config.projects[0].show_in_readme = false;
        let hidden = config.projects[0].surface_label.clone();
        config.projects[1].surface_label = "A <renamed> project".into();
        let mut added = config.projects[1].clone();
        added.id = "additional-project".into();
        added.label = "Additional project".into();
        added.surface_label = "Additional project".into();
        added.anchor = [0.48, 0.42];
        config.projects.push(added);
        let state = sourcefield_core::build_state(
            &config,
            &sourcefield_core::Snapshot::default(),
            "fixture",
        )
        .unwrap();
        config.projects.reverse();
        let reversed = sourcefield_core::build_state(
            &config,
            &sourcefield_core::Snapshot::default(),
            "fixture",
        )
        .unwrap();

        let svg = render_svg(&state, Theme::Dark, false);
        let reversed_svg = render_svg(&reversed, Theme::Dark, false);

        assert!(!svg.contains(&format!(">{hidden}</text>")));
        assert!(svg.contains("A &lt;renamed&gt; project"));
        assert!(svg.contains("Additional project"));
        assert_eq!(svg.matches("data-node-kind=\"project\"").count(), 8);
        assert_eq!(svg, reversed_svg);
    }

    #[test]
    fn static_output_has_no_animation_rules_and_light_labels_are_dark() {
        let (_, state) = fixture();

        let static_svg = render_svg(&state, Theme::Dark, false);
        let animated_svg = render_svg(&state, Theme::Dark, true);
        let light_svg = render_svg(&state, Theme::Light, false);

        assert!(!static_svg.contains("@keyframes"));
        assert!(!static_svg.contains("animation:"));
        assert!(animated_svg.contains("prefers-reduced-motion"));
        assert!(animated_svg.contains("animation-play-state:paused"));
        assert!(light_svg.contains("fill=\"#11182b\""));
        assert!(light_svg.contains("stop-color=\"#f5f7fc\""));
    }

    #[test]
    fn optional_content_and_canvas_dimensions_are_consumed() {
        let (_, mut state) = fixture();
        state.canvas.width = 2400;
        state.canvas.height = 2000;
        state.canvas.show_state_hash = false;
        state.canvas.show_interests_in_readme = false;
        state.canvas.show_technology_labels = false;
        state.canvas.show_activity_orbit = false;

        let svg = render_svg(&state, Theme::Dark, false);

        assert!(svg.contains("viewBox=\"0 0 2400 2000\""));
        assert!(!svg.contains("OTHER INTERESTS"));
        assert!(!svg.contains("6 MCP endpoints"));
        assert!(!svg.contains(&state.semantic_hash));
        assert!(!svg.contains("stroke-opacity=\".25\" class=\"flow\""));
    }

    #[test]
    fn repeated_outputs_preserve_mtime_and_invalid_destination_fails() {
        let (_, state) = fixture();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("sourcefield-render-{}-{nonce}", std::process::id()));
        let files = write_outputs(&state, &directory).unwrap();
        let before = fs::metadata(&files.dark_svg).unwrap().modified().unwrap();

        let second = write_outputs(&state, &directory).unwrap();
        let after = fs::metadata(&second.dark_svg).unwrap().modified().unwrap();
        let invalid = write_outputs(&state, &files.dark_svg);

        assert_eq!(before, after);
        assert!(invalid.is_err());
        assert_eq!(
            fs::read_to_string(&files.dark_svg).unwrap(),
            render_svg(&state, Theme::Dark, true)
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn details_require_explicit_rendering_flag() {
        let (_, mut state) = fixture();
        let project = state
            .nodes
            .iter_mut()
            .find(|node| node.kind == NodeKind::Project)
            .unwrap();
        project.details = vec!["Explicit public detail".into()];
        state.canvas.show_details = false;

        let hidden = render_svg(&state, Theme::Dark, false);
        state.canvas.show_details = true;
        let shown = render_svg(&state, Theme::Dark, false);

        assert!(!hidden.contains("Explicit public detail"));
        assert!(shown.contains("<title>Explicit public detail</title>"));
    }

    /// WCAG relative luminance for the fixed six-digit palette colors.
    fn luminance(color: &str) -> f64 {
        let channels = [1, 3, 5].map(|start| {
            let value = u8::from_str_radix(&color[start..start + 2], 16).unwrap() as f64 / 255.0;

            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });

        channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722
    }

    #[test]
    fn text_palettes_meet_normal_text_contrast_on_both_gradient_stops() {
        for theme in [Theme::Dark, Theme::Light] {
            let palette = theme.palette();

            for background in [palette.background, palette.secondary] {
                for foreground in [
                    palette.text,
                    palette.muted,
                    palette.quiet,
                    palette.mint,
                    palette.purple,
                    palette.amber,
                    palette.blue,
                ] {
                    let a = luminance(background);
                    let b = luminance(foreground);
                    let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);

                    assert!(ratio >= 4.5, "{foreground} on {background}: {ratio}");
                }
            }
        }
    }

    #[test]
    fn xml_preserves_text_without_markup() {
        let value = "<&>\"'";

        let escaped = xml(value);

        assert_eq!(escaped, "&lt;&amp;&gt;&quot;&apos;");
    }

    #[test]
    fn links_accept_https_and_reject_executable_schemes() {
        let mut output = String::new();

        let rejected = open_link(&mut output, Some("javascript:alert(1)"));
        let accepted = open_link(&mut output, Some("https://example.com/?a=1&b=2"));

        assert!(!rejected);
        assert!(accepted);
        assert!(!output.contains("javascript:"));
        assert!(output.contains("a=1&amp;b=2"));
    }
}
