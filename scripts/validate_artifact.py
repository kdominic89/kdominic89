#!/usr/bin/env python3
"""Validate the public SOURCEFIELD artifact with Python's standard library."""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
import xml.etree.ElementTree as ET
from collections import Counter
from html.parser import HTMLParser
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

EXPECTED_PACKAGES = {
    "Doka.Caching.MySql",
    "Doka.EntityFrameworkCore.MySql",
    "Doka.EntityFrameworkCore.MySql.NetTopologySuite",
    "Doka.EntityFrameworkCore.SafeMigrations",
    "Doka.EntityFrameworkCore.SafeMigrations.MySql",
    "Doka.EntityFrameworkCore.SafeMigrations.PostgreSql",
}

EXPECTED_MCP_IDS = {
    "graph-mcp",
    "memory-mcp",
    "knowledge-mcp",
    "experience-mcp",
    "workflow-mcp",
    "telemetry-mcp",
}

REQUIRED_FILES = {
    "README.md",
    "SETUP.md",
    "ARCHITECTURE.md",
    "PRIVACY.md",
    "SECURITY.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "config/profile.toml",
    "config/offline-snapshot.json",
    "assets/profile-state.json",
    "assets/sourcefield.dark.svg",
    "assets/sourcefield.light.svg",
    "assets/sourcefield.static.svg",
    "docs/index.html",
    "docs/app.css",
    "docs/app.js",
    "docs/profile-state.json",
    "docs/simulation-fallback.js",
    ".github/workflows/update-profile.yml",
    ".github/workflows/validate.yml",
    "CHANGELOG.md",
    "scripts/build-wasm.sh",
    "scripts/capture_previews.py",
    "scripts/package.sh",
    "scripts/serve.sh",
    "scripts/validate.sh",
    "scripts/validate_artifact.py",
    "crates/sourcefield-core/src/lib.rs",
    "crates/sourcefield-collector/src/lib.rs",
    "crates/sourcefield-render/src/lib.rs",
    "crates/sourcefield-cli/src/main.rs",
    "crates/sourcefield-wasm/src/lib.rs",
}


class HtmlResources(HTMLParser):
    """Collect executable and stylesheet references for local-resource validation."""

    def __init__(self) -> None:
        """Initialize resource lists for one HTML document."""
        super().__init__()
        self.scripts: list[str] = []
        self.styles: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        """Record script and stylesheet source attributes."""
        values = dict(attrs)
        if tag == "script" and values.get("src"):
            self.scripts.append(values["src"] or "")

        if tag == "link" and values.get("rel") == "stylesheet" and values.get("href"):
            self.styles.append(values["href"] or "")


def fail(message: str) -> None:
    """Abort the current validation group with a human-readable reason."""
    raise AssertionError(message)


def load_json(path: Path) -> Any:
    """Read JSON and preserve useful file context on malformed input."""
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        fail(f"invalid JSON {path}: {error}")


def local_name(tag: str) -> str:
    """Return an XML name without its namespace prefix."""
    return tag.rsplit("}", 1)[-1]


def require_files(root: Path) -> None:
    """Check required paths without traversing unrelated host caches."""
    missing = sorted(path for path in REQUIRED_FILES if not (root / path).is_file())
    if missing:
        fail("missing required files: " + ", ".join(missing))

    for relative in REQUIRED_FILES:
        path = root / relative
        if path.is_symlink() or any(parent.is_symlink() for parent in path.parents if parent != root.parent):
            fail(f"required file must not traverse a symlink: {relative}")


def validate_config(root: Path) -> dict[str, Any]:
    """Validate generic configuration structure independently of profile identity."""
    with (root / "config/profile.toml").open("rb") as handle:
        config = tomllib.load(handle)

    if config.get("version") != 1:
        fail("config version must be 1")

    for section in ("domains", "technologies", "projects", "publications"):
        identifiers = [item["id"] for item in config.get(section, [])]
        if len(identifiers) != len(set(identifiers)):
            fail(f"duplicate IDs in {section}")

    render = config["render"]
    if render["width"] < 1200 or render["height"] < 640:
        fail("README canvas is too small")

    return config


def validate_approved_profile(config: dict[str, Any]) -> None:
    """Check this profile's editorial contract separately from generic structure."""
    profile = config["profile"]
    if profile["username"] != "kdominic89" or profile["organization"] != "doka-labs":
        fail("profile account/organization mapping is incorrect")

    domains = {item["id"]: item for item in config.get("domains", [])}
    if domains.get("doka-labs", {}).get("kind") != "organization":
        fail("Doka Labs must be modeled as an organization namespace")

    if domains.get("doka-labs", {}).get("owner") != "doka-labs":
        fail("Doka Labs owner mapping is incorrect")

    technologies = {item["id"]: item for item in config.get("technologies", [])}
    angular = technologies.get("angular")
    if not angular:
        fail("Angular technology node is missing")

    if set(angular.get("affinities", [])) != {"personal", "doka-labs"}:
        fail("Angular must have affinities to both namespaces")

    if angular.get("cross_domain") is not True:
        fail("Angular must be marked cross-domain")

    projects = {item["id"]: item for item in config.get("projects", [])}
    ai = projects.get("ai-devops")
    if not ai:
        fail("ai-devops is missing")

    components = {item["id"]: item for item in ai.get("components", [])}
    if set(components) != EXPECTED_MCP_IDS:
        fail("ai-devops requires Graph, Memory, Knowledge, Experience, Workflow and Telemetry MCP endpoints")

    for component_id, component in components.items():
        expected_scope = "global-and-project" if component_id in {"knowledge-mcp", "experience-mcp"} else "project"
        if component.get("scope") != expected_scope:
            fail(f"invalid context scope for {component_id}: expected {expected_scope}")

        if component.get("kind") != "mcp":
            fail(f"{component_id} must be an MCP component")

    budget = projects.get("budget-board")
    if not budget or "go" not in budget.get("implemented_with", []):
        fail("BudgetBoard must be modeled as a Go project")

    budget_text = json.dumps(budget, ensure_ascii=False).lower()
    for signal in ("sqlcipher", "age"):
        if signal not in budget_text:
            fail(f"BudgetBoard configuration is missing approved encryption wording: {signal}")

    stack = projects.get("state-trace")
    if not stack or "rust" not in stack.get("implemented_with", []):
        fail("StateTrace must be modeled as a Rust project")

    if "swift" not in stack.get("implemented_with", []):
        fail("StateTrace must retain its native Swift integration")

    package_ids = {
        package["id"] for publication in config.get("publications", []) for package in publication.get("packages", [])
    }

    if package_ids != EXPECTED_PACKAGES:
        fail("configured Doka Labs package set differs from the approved six packages")

    expected = {
        "ai-devops",
        "budget-board",
        "state-trace",
        "dotconfig",
        "vscode-theme",
        "mysql",
        "safe-migrations",
        "relational-lab",
    }

    if set(projects) != expected:
        fail("approved profile must contain all eight projects")


def validate_state(root: Path, config: dict[str, Any]) -> dict[str, Any]:
    """Check configured identities, geometry and graph references in both public copies."""
    assets_path = root / "assets/profile-state.json"
    docs_path = root / "docs/profile-state.json"
    if assets_path.read_bytes() != docs_path.read_bytes():
        fail("assets/profile-state.json and docs/profile-state.json must be byte-identical")

    state = load_json(assets_path)

    if state.get("schema") != 2:
        fail("state schema must be 2")

    if not re.fullmatch(r"[A-F0-9]{16}", state.get("semantic_hash", "")):
        fail("semantic_hash must be 16 uppercase hexadecimal characters")

    canvas = state.get("canvas", {})
    if canvas.get("width") != config["render"]["width"] or canvas.get("height") != config["render"]["height"]:
        fail("state canvas differs from configuration")

    if state.get("profile", {}).get("username") != config["profile"]["username"]:
        fail("state profile username is incorrect")

    if state.get("profile", {}).get("organization") != config["profile"]["organization"]:
        fail("state organization is incorrect")

    nodes = state.get("nodes")
    edges = state.get("edges")
    if not isinstance(nodes, list) or not nodes:
        fail("state must contain nodes")

    if not isinstance(edges, list) or not edges:
        fail("state must contain edges")

    ids = [node.get("id") for node in nodes]
    duplicates = [item for item, count in Counter(ids).items() if count > 1]
    if duplicates:
        fail("duplicate node IDs: " + ", ".join(map(str, duplicates)))

    id_set = set(ids)

    width = canvas["width"]
    height = canvas["height"]
    for node in nodes:
        if not isinstance(node.get("x"), (int, float)) or not isinstance(node.get("y"), (int, float)):
            fail(f"node {node.get('id')} has invalid coordinates")

        if not (-200 <= node["x"] <= width + 200 and -200 <= node["y"] <= height + 200):
            fail(f"node {node.get('id')} lies implausibly outside the canvas")

        if node.get("visibility") == "private-abstract" and node.get("url") is not None:
            fail(f"private abstract node {node.get('id')} must not expose a URL")

    for edge in edges:
        if edge.get("from") not in id_set or edge.get("to") not in id_set:
            fail(f"edge references unknown node: {edge}")

    expected_packages = {
        p["id"] for publication in config.get("publications", []) for p in publication.get("packages", [])
    }

    state_packages = {item.get("id") for item in state.get("packages", [])}
    package_nodes = {node["id"].removeprefix("package:") for node in nodes if node.get("kind") == "package"}

    if state_packages != expected_packages or package_nodes != expected_packages:
        fail("state package list/package nodes differ from the approved package set")

    if state.get("stats", {}).get("package_count") != len(expected_packages):
        fail("state package_count is incorrect")

    expected_core_nodes = {f"project:{item['id']}" for item in config.get("projects", [])}
    missing = expected_core_nodes - id_set
    if missing:
        fail("state misses core nodes: " + ", ".join(sorted(missing)))

    return state


def validate_svg_payload(raw: str) -> None:
    """Reject executable SVG and external rendering resources, while allowing public links."""
    if re.search(r"<!\s*(?:DOCTYPE|ENTITY)", raw, re.IGNORECASE):
        fail("SVG declarations are forbidden")

    tree = ET.fromstring(raw)
    allowed = {
        "svg",
        "title",
        "desc",
        "defs",
        "style",
        "g",
        "a",
        "text",
        "tspan",
        "path",
        "rect",
        "circle",
        "ellipse",
        "line",
        "polyline",
        "polygon",
        "linearGradient",
        "radialGradient",
        "stop",
        "pattern",
        "clipPath",
        "mask",
        "filter",
        "feGaussianBlur",
        "feMerge",
        "feMergeNode",
        "feColorMatrix",
        "feBlend",
        "feComposite",
        "feFlood",
        "feOffset",
        "animate",
        "animateTransform",
        "animateMotion",
        "mpath",
        "use",
    }

    for element in tree.iter():
        tag = local_name(element.tag)
        if tag not in allowed or not element.tag.startswith("{http://www.w3.org/2000/svg}"):
            fail(f"forbidden SVG element: {tag}")

        for attribute, value in element.attrib.items():
            name = local_name(attribute).lower()
            if name.startswith("on") or name in {"src", "base"}:
                fail(f"forbidden SVG attribute: {name}")

            if name == "href" and any(ord(character) < 32 for character in value):
                fail("control characters in SVG link")

            if name == "href" and value and not value.startswith("#"):
                link = urlsplit(value)
                if tag != "a" or link.scheme != "https" or not link.hostname or link.username or link.password:
                    fail("external SVG resource or unsafe anchor")

            if name == "attributename" and value.lower() not in {"opacity", "transform", "r", "stroke-dashoffset"}:
                fail("animation target is outside the presentation allowlist")

            validate_css_value(value)

        if tag == "style":
            validate_css_value(element.text or "")


def validate_css_value(value: str) -> None:
    """Allow only fragment URL references and prohibit CSS escape-based obfuscation."""
    unsafe = r"[\\]|/\*|@import|(?:expression|image-set|image|cross-fade|paint)\s*\(|javascript:|data:"
    if re.search(unsafe, value, re.IGNORECASE):
        fail("unsafe or obfuscated SVG style value")

    for match in re.finditer(r"url\s*\((.*?)\)", value, re.IGNORECASE | re.DOTALL):
        target = match.group(1).strip().strip("\"'")
        if not re.fullmatch(r"#[A-Za-z_][A-Za-z0-9_.:-]*", target):
            fail("external SVG CSS resource")


def validate_svg(root: Path, config: dict[str, Any]) -> None:
    """Check dimensions, accessible labels, payload policy and motion variants."""
    namespace = "{http://www.w3.org/2000/svg}"
    for name in ("sourcefield.dark.svg", "sourcefield.light.svg", "sourcefield.static.svg"):
        path = root / "assets" / name
        raw = path.read_text(encoding="utf-8")
        if len(raw.encode("utf-8")) > 900_000:
            fail(f"{name} exceeds the 900 KB README budget")

        try:
            tree = ET.fromstring(raw)
        except ET.ParseError as error:
            fail(f"invalid SVG {name}: {error}")
        if local_name(tree.tag) != "svg":
            fail(f"{name} root element is not svg")

        if int(tree.attrib.get("width", "0")) != config["render"]["width"]:
            fail(f"{name} width differs from config")

        if int(tree.attrib.get("height", "0")) != config["render"]["height"]:
            fail(f"{name} height differs from config")

        if tree.find(f"{namespace}title") is None or tree.find(f"{namespace}desc") is None:
            fail(f"{name} requires accessible title and description")

        validate_svg_payload(raw)
        motion = "<animate" in raw or "@keyframes" in raw
        if name.endswith("static.svg") and motion:
            fail("sourcefield.static.svg must be motion-free")

        if not name.endswith("static.svg") and not motion:
            fail(f"{name} is expected to contain declarative motion")


def validate_readme_and_site(root: Path) -> None:
    """Check local runtime references and README fallback links."""
    readme = (root / "README.md").read_text(encoding="utf-8")
    for reference in (
        "assets/sourcefield.dark.svg",
        "assets/sourcefield.light.svg",
        "assets/sourcefield.static.svg",
        "https://kdominic89.github.io/kdominic89/",
    ):
        if reference not in readme:
            fail(f"README misses {reference}")

    if "prefers-reduced-motion" not in readme:
        fail("README must provide a reduced-motion source")

    html_path = root / "docs/index.html"
    html = html_path.read_text(encoding="utf-8")
    parser = HtmlResources()
    parser.feed(html)
    if parser.scripts != ["./app.js"]:
        fail(f"Pages scripts must remain local; found {parser.scripts}")

    if parser.styles != ["./app.css"]:
        fail(f"Pages stylesheet must remain local; found {parser.styles}")

    if "https://" in " ".join(parser.scripts + parser.styles):
        fail("Pages runtime resources must not use a CDN")

    for relative in ("docs/app.js", "docs/app.css", "docs/simulation-fallback.js"):
        if not (root / relative).is_file():
            fail(f"missing browser resource {relative}")


def validate_public_secret_surface(root: Path) -> None:
    """Detect common GitHub token signatures; this is not a general secret detector."""
    patterns = [
        re.compile(r"ghp_[A-Za-z0-9]{20,}"),
        re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
        re.compile(r"gho_[A-Za-z0-9]{20,}"),
    ]

    roots = [root / "assets", root / "docs", root / "config"]
    for base in roots:
        for path in base.rglob("*"):
            if not path.is_file() or path.suffix.lower() in {".png", ".wasm", ".zip"}:
                continue

            text = path.read_text(encoding="utf-8", errors="ignore")
            for pattern in patterns:
                if pattern.search(text):
                    fail(f"possible token found in public artifact: {path.relative_to(root)}")


def validate_workflows(root: Path) -> None:
    """Check workflow source conventions without claiming execution evidence."""
    update = (root / ".github/workflows/update-profile.yml").read_text(encoding="utf-8")
    validate = (root / ".github/workflows/validate.yml").read_text(encoding="utf-8")
    for action in (
        "actions/checkout",
        "actions/configure-pages",
        "actions/upload-pages-artifact",
        "actions/deploy-pages",
    ):
        references = re.findall(rf"uses:\s*{re.escape(action)}@([^\s]+)", update + "\n" + validate)
        if not references or any(not re.fullmatch(r"[0-9a-f]{40}", reference) for reference in references):
            fail(f"workflow requires immutable full commit references for {action}")

    if "cargo generate-lockfile" in update or "cargo generate-lockfile" in validate:
        fail("workflows must consume the committed Cargo.lock")

    if "PROFILE_TOKEN:" not in update:
        fail("optional PROFILE_TOKEN mapping is missing")


def validate_wasm(root: Path, require_wasm: bool) -> str | None:
    """Check the module header; runtime behavior is covered by browser tests."""
    path = root / "docs/pkg/sourcefield_wasm_bg.wasm"
    if not path.is_file():
        if require_wasm:
            fail("compiled WebAssembly is required but docs/pkg/sourcefield_wasm_bg.wasm is missing")

        return (
            "compiled WebAssembly is not checked in; the local JavaScript bootstrap works "
            "immediately and the update workflow builds the Rust module"
        )

    with path.open("rb") as handle:
        payload = handle.read(8)
    if len(payload) < 8 or payload[:4] != b"\x00asm":
        fail("docs/pkg/sourcefield_wasm_bg.wasm is not a valid WebAssembly binary")

    if payload[4:8] != b"\x01\x00\x00\x00":
        fail("docs/pkg/sourcefield_wasm_bg.wasm uses an unsupported WebAssembly binary version")

    return None


def write_report(
    path: Path,
    root: Path,
    state: dict[str, Any] | None,
    passed: list[str],
    warnings: list[str],
    failure: str | None,
) -> None:
    """Write only checks actually executed by this artifact validator."""
    semantic_hash = state.get("semantic_hash", "UNAVAILABLE") if state else "UNAVAILABLE"
    nodes = len(state.get("nodes", [])) if state else 0
    edges = len(state.get("edges", [])) if state else 0
    packages = len(state.get("packages", [])) if state else 0
    result = "FAIL" if failure else "PASS"

    lines = [
        "# Validation report",
        "",
        "This report was generated by `scripts/validate_artifact.py`.",
        "",
        f"- Result: **{result}**",
        f"- Semantic state: `{semantic_hash}`",
        f"- Files validated from: `{root.name}`",
        f"- Passed checks: {len(passed)}",
        f"- Warnings: {len(warnings)}",
        f"- Failures: {1 if failure else 0}",
        "",
        "## Artifact summary",
        "",
        f"- Nodes: {nodes}",
        f"- Edges: {edges}",
        f"- NuGet packages: {packages}",
        "",
        "## Passed",
        "",
    ]

    lines.extend(f"- {item}" for item in passed)
    if not passed:
        lines.append("- None")

    lines.extend(["", "## Warnings", ""])
    lines.extend(f"- {item}" for item in warnings)
    if not warnings:
        lines.append("- None")

    lines.extend(["", "## Failures", ""])
    lines.append(f"- {failure}" if failure else "- None")
    lines.extend(
        [
            "",
            "## Verification boundary",
            "",
            "This command checks artifact structure and selected source policies only. "
            "It does not execute JavaScript, compile Rust, validate archive integrity, or prove "
            "browser behavior. scripts/verify.sh enforces native checks; scripts/build-wasm.sh "
            "builds the release WebAssembly module.",
            "",
        ]
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    """Run artifact checks and optionally persist a scope-limited report."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--report", type=Path)
    parser.add_argument(
        "--require-wasm",
        action="store_true",
        help="Fail when the compiled wasm-pack binary is not present.",
    )
    parser.add_argument(
        "--approved-profile", action="store_true", help="Also enforce the approved personal profile facts."
    )
    args = parser.parse_args()
    root = args.root.resolve()
    report = args.report
    if report is not None and not report.is_absolute():
        report = root / report

    passed: list[str] = []
    warnings: list[str] = []
    state: dict[str, Any] | None = None
    failure: str | None = None

    try:
        require_files(root)
        passed.append(f"all {len(REQUIRED_FILES)} required files exist and do not traverse symlinks")

        config = validate_config(root)
        if args.approved_profile:
            validate_approved_profile(config)

        passed.append("configuration structure is valid")
        if args.approved_profile:
            passed.append("approved profile has eight projects, six MCP endpoints and six NuGet packages")

        state = validate_state(root, config)
        passed.extend(
            [
                "README and Pages use a byte-identical semantic state",
                f"all {len(state['nodes'])} node identifiers are unique and all {len(state['edges'])} edges resolve",
                "private abstract nodes expose no repository URL; manually approved descriptions remain public",
                f"the approved set of {len(state['packages'])} Doka Labs packages is present",
            ]
        )

        validate_svg(root, config)
        passed.append("SVG variants satisfy dimensions, accessible-label and local-resource policy checks")

        validate_readme_and_site(root)
        passed.append("README fallbacks and GitHub Pages runtime resources are local and complete")

        validate_public_secret_surface(root)
        passed.append("no common GitHub token signatures were detected in generated public data")

        validate_workflows(root)
        passed.append("workflow source contains expected actions pinned to full commit IDs and consumes the lockfile")

        wasm_warning = validate_wasm(root, args.require_wasm)
        if wasm_warning:
            warnings.append(wasm_warning)
        else:
            passed.append("the local WebAssembly binary has a valid module header")

    except (AssertionError, KeyError, TypeError, ValueError, OSError, tomllib.TOMLDecodeError) as error:
        failure = str(error)
        if report is not None:
            write_report(report, root, state, passed, warnings, failure)

        print(f"VALIDATION FAILED: {error}", file=sys.stderr)
        return 1

    if report is not None:
        write_report(report, root, state, passed, warnings, None)

    assert state is not None
    print(
        "VALIDATION OK: "
        f"{len(state['nodes'])} nodes, {len(state['edges'])} edges, "
        f"{len(state['packages'])} packages, state {state['semantic_hash']}"
    )
    for warning in warnings:
        print(f"VALIDATION WARNING: {warning}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
