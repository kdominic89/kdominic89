"""Exercise the installed CLI at the personal consumer's file and remote import boundaries."""

from contextlib import contextmanager
import hashlib
import html
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from threading import Thread
import tomllib
import unittest

from test_consumer_contract import (
    APPROVED_PROJECT_LAYOUT, DOKA_AFFINITIES, DOKA_SHARED_CAPABILITIES, ROOT, SQL_SERVER_PACKAGE, TECHNOLOGY_IDS,
    captured_organization, read_json, read_toml,
)


REPLAY_INPUTS = {
    "resolved-config.json", "source-snapshot.json", "render-snapshot.json",
    "import-capture.json", "layout.json", "profile-state.json",
}
AUTHORED_DRIFT_DIAGNOSTIC = (
    "generation provenance authored inputs changed: configuration bytes differ from the recorded generation"
)
GENERATOR_MISMATCH_DIAGNOSTIC = "generation provenance generator identity mismatch: use the recorded generator build"


class UnavailableProxy(BaseHTTPRequestHandler):
    """Reject remote transport so replay cannot succeed by quietly fetching current inputs."""

    def do_CONNECT(self):
        """Record and reject an attempted HTTPS tunnel."""
        self.server.requests.append(self.path)
        self.send_error(503, "Remote canonical source unavailable")

    def do_GET(self):
        """Record and reject an attempted plain HTTP fetch."""
        self.server.requests.append(self.path)
        self.send_error(503, "Remote canonical source unavailable")

    def log_message(self, format_string, *arguments):
        """Keep the test output free of proxy access logs."""
        return


@contextmanager
def unavailable_remote_transport():
    """Provide a denied remote transport and count any attempted fetches."""
    server = HTTPServer(("127.0.0.1", 0), UnavailableProxy)
    server.requests = []
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    proxy = f"http://127.0.0.1:{server.server_port}"

    try:
        yield {"HTTP_PROXY": proxy, "HTTPS_PROXY": proxy, "ALL_PROXY": proxy, "NO_PROXY": "",
               "http_proxy": proxy, "https_proxy": proxy, "all_proxy": proxy, "no_proxy": ""}, server.requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def project_layout(state):
    """Select the project positions, ring sizes, and weights visible to this consumer."""
    return {node["id"]: ([node["x"], node["y"]], node["radius"], node["weight"])
            for node in state["nodes"] if node["kind"] == "project"}


def project_rows(readme):
    """Select the generated project table rows without depending on implementation modules."""
    block = readme.split("<!-- sourcefield:projects:start -->", 1)[1].split("<!-- sourcefield:projects:end -->", 1)[0]

    return [line for line in block.splitlines() if line.startswith("| ")][2:]


@unittest.skipUnless(os.environ.get("SOURCEFIELD_INSTALLATION"),
                     "Set SOURCEFIELD_INSTALLATION to execute the CLI integration tests")
class InstalledGenerationTests(unittest.TestCase):
    """Generate isolated copies using an explicitly selected complete native and runtime installation."""

    @classmethod
    def setUpClass(cls):
        """Fail explicitly when an opted-in installation is incomplete."""
        cls.installation = Path(os.environ["SOURCEFIELD_INSTALLATION"]).resolve()
        for relative in ("sourcefield", "runtime/runtime-manifest.json", "runtime/pkg/sourcefield_wasm_bg.wasm"):
            if not (cls.installation / relative).is_file():
                raise RuntimeError(f"Sourcefield installation is incomplete: {relative}")

    def setUp(self):
        """Copy actual consumer inputs, outputs, history, README, and ownership into an isolated fixture."""
        temporary = tempfile.TemporaryDirectory(prefix="personal-consumer-contract-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for relative in ("config", "assets", "docs"):
            ignore = shutil.ignore_patterns("pkg") if relative == "docs" else None
            shutil.copytree(ROOT / relative, self.root / relative, ignore=ignore)

        for relative in ("README.md", ".sourcefield-owned.json"):
            shutil.copy2(ROOT / relative, self.root / relative)

    def _invoke(self, operation, *arguments, environment=None):
        """Run the installed CLI with no inherited collection credentials and a bounded timeout."""
        selected_environment = os.environ.copy()
        for name in ("GH_TOKEN", "GITHUB_TOKEN", "PROFILE_TOKEN", "SOURCEFIELD_PRIVATE_COUNTS"):
            selected_environment.pop(name, None)

        selected_environment.update(environment or {})
        command = [str(self.installation / "sourcefield"), operation, "--root", str(self.root), *arguments]

        return subprocess.run(command, cwd=self.root, env=selected_environment, capture_output=True,
                              text=True, timeout=90, check=False)

    def _command(self, *extra, environment=None):
        """Generate offline using the actual public capture, one README, and explicit history preservation."""
        return self._invoke(
            "generate", "--offline", "--fallback-snapshot", "assets/source-snapshot.json",
            "--runtime", str(self.installation / "runtime"), "--no-history", "--readme", "README.md",
            *extra, environment=environment,
        )

    def _generate(self, *extra, environment=None):
        """Require successful fixture generation before reading its semantic state."""
        result = self._command(*extra, environment=environment)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

        return read_json(self.root / "assets/profile-state.json")

    def _validate(self):
        """Validate the consumer's current authored bytes and generated artifact set."""
        return self._invoke("validate")

    def _owned_outputs(self):
        """Capture all managed output bytes, including intentionally missing files, for atomicity checks."""
        ownership = read_json(self.root / ".sourcefield-owned.json")
        paths = {*ownership["files"], "README.md", ".sourcefield-owned.json"}

        return {relative: (self.root / relative).read_bytes() if (self.root / relative).is_file() else None
                for relative in sorted(paths)}

    def _history(self):
        """Capture every actual retained archive and its index byte for byte."""
        return {path.name: path.read_bytes() for path in (self.root / "docs/history").iterdir() if path.is_file()}

    def _assert_generation_record(self):
        """Require the schema 2 record to bind current authored bytes and exactly six captured input roles."""
        record = read_json(self.root / "assets/generation-record.json")
        inputs = {name: hashlib.sha256((self.root / "assets" / name).read_bytes()).hexdigest()
                  for name in REPLAY_INPUTS}
        self.assertEqual(record["schema_version"], 2)
        self.assertEqual(record["inputs"], inputs)
        self.assertEqual(record["authored_config_sha256"],
                         hashlib.sha256((self.root / "config/profile.toml").read_bytes()).hexdigest())

        return record

    def _replace_profile(self, original, replacement):
        """Change exactly one known authored field in the isolated consumer profile."""
        path = self.root / "config/profile.toml"
        content = path.read_text(encoding="utf-8")
        self.assertEqual(content.count(original), 1, original)
        path.write_text(content.replace(original, replacement, 1), encoding="utf-8")

    def _write_capture(self, capture):
        """Write one adversarial captured-input fixture without touching other consumer files."""
        (self.root / "assets/import-capture.json").write_text(json.dumps(capture) + "\n", encoding="utf-8")

    def test_offline_regeneration_is_deterministic_and_preserves_all_history(self):
        """Repeated offline generation keeps managed bytes and all 24 prior states stable."""
        # Arrange
        history = self._history()
        self._generate()
        outputs = self._owned_outputs()

        # Act
        self._generate()
        validation = self._validate()

        # Assert
        self.assertEqual(validation.returncode, 0, validation.stdout + validation.stderr)
        self.assertEqual(self._owned_outputs(), outputs)
        self.assertEqual(self._history(), history)
        self.assertEqual(len(read_json(self.root / "docs/history/index.json")["states"]), 24)

    def test_offline_preview_preserves_absent_source_date_and_records_distinct_effective_input(self):
        """Preview preserves the actual undated Live capture and binds its distinct effective rendering input."""
        # Arrange
        source_path = self.root / "assets/source-snapshot.json"
        source_bytes = source_path.read_bytes()
        captured = read_json(source_path)
        history = self._history()

        # Act
        state = self._generate()

        # Assert
        self.assertEqual(captured["mode"], "live")
        self.assertEqual(captured["fetched_at"], "")
        self.assertIsNone(captured["private_repository_count"])
        self.assertEqual(source_path.read_bytes(), source_bytes)
        current = read_json(source_path)
        self.assertEqual(current["fetched_at"], captured["fetched_at"])
        self.assertEqual(current["sources"], captured["sources"])
        render_path = self.root / "assets/render-snapshot.json"
        effective = read_json(render_path)
        self.assertEqual(effective["mode"], "preview")
        self.assertEqual(effective["fetched_at"], "")
        self.assertIsNone(effective["private_repository_count"])
        self.assertEqual(state["mode"], "preview")
        self.assertNotEqual(render_path.read_bytes(), source_bytes)
        self._assert_generation_record()
        self.assertEqual(self._history(), history)

    def test_locked_replay_preserves_absent_source_date_and_recorded_effective_preview(self):
        """Replay retains the actual missing retrieval date, captured Preview, and original history."""
        # Arrange
        source_path = self.root / "assets/source-snapshot.json"
        source_bytes = source_path.read_bytes()
        captured = read_json(source_path)
        history = self._history()
        self._generate()
        render_path = self.root / "assets/render-snapshot.json"
        render_bytes = render_path.read_bytes()
        outputs = self._owned_outputs()

        # Act
        state = self._generate("--locked")

        # Assert
        self.assertEqual(captured["mode"], "live")
        self.assertEqual(captured["fetched_at"], "")
        self.assertEqual(source_path.read_bytes(), source_bytes)
        self.assertEqual(read_json(source_path)["fetched_at"], captured["fetched_at"])
        self.assertEqual(read_json(source_path)["mode"], captured["mode"])
        self.assertEqual(read_json(source_path)["sources"], captured["sources"])
        self.assertEqual(render_path.read_bytes(), render_bytes)
        self.assertEqual(read_json(render_path)["mode"], "preview")
        self.assertEqual(read_json(render_path)["fetched_at"], "")
        self.assertIsNone(read_json(render_path)["private_repository_count"])
        self.assertEqual(state["mode"], "preview")
        self._assert_generation_record()
        self.assertEqual(self._owned_outputs(), outputs)
        self.assertEqual(self._history(), history)

    def test_generated_graph_has_thirty_technologies_and_fifteen_imported_affinity_edges(self):
        """Resolved shared technology aliases preserve the consumer's complete graph relations."""
        # Arrange
        expected = {f"technology:{identifier}" for identifier in TECHNOLOGY_IDS}

        # Act
        state = self._generate()

        # Assert
        technologies = [node["id"] for node in state["nodes"] if node["kind"] == "technology"]
        relations = [edge for edge in state["edges"]
                     if edge["kind"] in {"affinity", "shared-capability"} and edge["from"] == "domain:doka-labs"]
        self.assertEqual(len(technologies), 30)
        self.assertEqual(set(technologies), expected)
        self.assertEqual(len(relations), 15)
        self.assertEqual({edge["to"] for edge in relations},
                         {f"technology:{identifier}" for identifier in DOKA_AFFINITIES})
        self.assertEqual({edge["to"] for edge in relations if edge["kind"] == "shared-capability"},
                         {f"technology:{identifier}" for identifier in DOKA_SHARED_CAPABILITIES})
        self.assertEqual({edge["to"] for edge in relations if edge["kind"] == "affinity"},
                         {f"technology:{identifier}" for identifier in DOKA_AFFINITIES - DOKA_SHARED_CAPABILITIES})

    def test_private_projects_are_unlinked_in_state_readme_and_captured_body(self):
        """All six private abstract projects retain approved prose and stacks without repository URLs."""
        # Arrange
        profile = read_toml(self.root / "config/profile.toml")
        saved, organization = captured_organization(self.root)
        private = [item for item in [*profile["projects"], *organization["projects"]]
                   if item["visibility"] == "private-abstract"]

        # Act
        state = self._generate()

        # Assert
        readme = (self.root / "README.md").read_text(encoding="utf-8")
        rows = project_rows(readme)
        private_nodes = [node for node in state["nodes"]
                         if node["kind"] == "project" and node["visibility"] == "private-abstract"]
        self.assertEqual(len(private_nodes), 6)
        self.assertEqual({node["label"] for node in private_nodes}, {item["label"] for item in private})
        for item in private:
            summary = html.escape(item["summary"].replace("\n", " "), quote=False)
            expected_row = f"| {item['label']} | {summary} | {' / '.join(item['display_stack'])} |"
            self.assertIn(expected_row, rows)
            self.assertNotIn(f"[{item['label']}]", readme)
            node = next(node for node in private_nodes if node["label"] == item["label"])
            self.assertIsNone(node["url"])
            self.assertEqual(node["display_stack"], item["display_stack"])

        captured_private = next(item for item in tomllib.loads(saved["manifest"])["projects"]
                                if item["id"] == "relational-lab")
        self.assertFalse({"url", "repository"} & captured_private.keys())
        self.assertFalse(any(item.get("url") or item.get("repository") for item in private))
        resolved_private = [item for item in read_json(self.root / "assets/resolved-config.json")["projects"]
                            if item["visibility"] == "private-abstract"]
        self.assertEqual(len(resolved_private), 6)
        self.assertFalse(any(item.get("url") or item.get("repository") for item in resolved_private))
        self.assertEqual({item["label"]: item["display_stack"] for item in resolved_private},
                         {item["label"]: item["display_stack"] for item in private})

    def test_readme_uses_personal_then_authored_canonical_project_order(self):
        """Canonical composition retains the reviewed table order and public repository destinations."""
        # Arrange
        profile = read_toml(self.root / "config/profile.toml")
        _, organization = captured_organization(self.root)
        expected = [item for item in [*profile["projects"], *organization["projects"]] if item["show_in_readme"]]

        # Act
        self._generate()

        # Assert
        readme = (self.root / "README.md").read_text(encoding="utf-8")
        rows = project_rows(readme)
        self.assertIn("| Project | What it does | Stack |", readme)
        self.assertEqual(len(rows), len(expected))
        for row, item in zip(rows, expected, strict=True):
            self.assertIn(item["label"], row)
            self.assertIn(" / ".join(item["display_stack"]), row)
            if "repository" in item:
                self.assertIn(f"[{item['label']}](https://github.com/{item['repository']})", row)

    def test_captured_canonical_geometry_packages_and_motifs_reach_generation(self):
        """Remote canonical facts produce the SQL Server fifth row and both approved new icon motifs."""
        # Arrange
        profile = read_toml(self.root / "config/profile.toml")
        saved, organization = captured_organization(self.root)

        # Act
        state = self._generate()

        # Assert
        nodes = {node["id"]: node for node in state["nodes"]}
        self.assertEqual(project_layout(state), APPROVED_PROJECT_LAYOUT)
        for identifier, anchor in profile["layout"]["overrides"].items():
            self.assertEqual([nodes[identifier]["x"], nodes[identifier]["y"]], anchor, identifier)

        packages = {f"package:{item['id']}" for publication in organization["publications"]
                    for item in publication["packages"]}
        self.assertEqual(len(packages), 10)
        self.assertTrue(packages.issubset(nodes))
        package = nodes[f"package:{SQL_SERVER_PACKAGE}"]
        self.assertEqual(package["surface_label"], "SQL Server")
        self.assertEqual([package["x"], package["y"]], [666.0, 1427.0])
        self.assertEqual(nodes["project:doka-labs/safe-migrations"]["icon"], "builtin:database-safe")
        self.assertEqual(nodes["project:sourcefield"]["icon"], "builtin:sourcefield")
        self.assertEqual(nodes["project:sourcefield"]["url"], "https://github.com/kdominic89/sourcefield")
        self.assertEqual(state["canvas"]["height"], 1885.0)
        self.assertEqual(captured_organization(self.root)[0], saved)
        resolved = read_json(self.root / "assets/resolved-config.json")
        imported = next(domain for domain in resolved["domains"] if domain["id"] == "doka-labs")
        self.assertEqual(imported["maintainer"], organization["maintainer"])

    def test_reduced_history_limit_preserves_every_archive_and_index_byte(self):
        """Offline no-history generation cannot prune the actual 24 archives when retention is lowered."""
        # Arrange
        self._generate()
        history = self._history()
        self._replace_profile("history_limit = 24", "history_limit = 1")

        # Act
        self._generate()
        validation = self._validate()

        # Assert
        self.assertEqual(validation.returncode, 0, validation.stdout + validation.stderr)
        self.assertEqual(self._history(), history)
        self.assertEqual(len(read_json(self.root / "docs/history/index.json")["states"]), 24)

    def test_locked_replay_restores_outputs_with_remote_transport_unavailable(self):
        """Replay uses the exact capture while no remote source can be fetched and retained history stays intact."""
        # Arrange
        self._generate()
        outputs = self._owned_outputs()
        history = self._history()
        for relative in ("assets/sourcefield.dark.svg", "docs/profile-state.json", "docs/app.js"):
            (self.root / relative).unlink()

        # Act
        with unavailable_remote_transport() as (environment, requests):
            self._generate("--locked", environment=environment)
            validation = self._validate()

        # Assert
        self.assertEqual(validation.returncode, 0, validation.stdout + validation.stderr)
        self.assertEqual(requests, [])
        self.assertEqual(self._owned_outputs(), outputs)
        self.assertEqual(self._history(), history)

    def test_locked_replay_preserves_archives_after_reduced_history_limit(self):
        """Captured replay cannot apply live retention to the 24 archives under a newly reduced cap."""
        # Arrange
        self._generate()
        history = self._history()
        self._replace_profile("history_limit = 24", "history_limit = 1")
        self._generate()
        outputs = self._owned_outputs()

        # Act
        self._generate("--locked")

        # Assert
        self.assertEqual(self._history(), history)
        self.assertEqual(self._owned_outputs(), outputs)
        self.assertEqual(len(read_json(self.root / "docs/history/index.json")["states"]), 24)

    def test_fresh_checkout_restores_missing_owned_generated_assets(self):
        """Generation restores omitted runtime and generated assets from actual captured consumer inputs."""
        # Arrange
        self._generate()
        outputs = self._owned_outputs()
        retained = {"assets/import-capture.json", "assets/layout.json", "assets/source-snapshot.json"}
        missing = [relative for relative in outputs
                   if relative not in retained and not relative.startswith("docs/history/")
                   and relative not in {"README.md", ".sourcefield-owned.json"}]
        for relative in missing:
            (self.root / relative).unlink()

        # Act
        self._generate()
        validation = self._validate()

        # Assert
        self.assertGreater(len(missing), 10)
        self.assertEqual(validation.returncode, 0, validation.stdout + validation.stderr)
        self.assertEqual(self._owned_outputs(), outputs)
        self.assertTrue((self.root / "docs/pkg/sourcefield_wasm_bg.wasm").is_file())

    def test_changed_authored_bytes_reject_locked_replay_without_output_changes(self):
        """A comment-only consumer change still crosses the recorded authored input boundary."""
        # Arrange
        self._generate()
        path = self.root / "config/profile.toml"
        path.write_bytes(path.read_bytes() + b"\n# Consumer-authored drift.\n")
        outputs = self._owned_outputs()

        # Act
        result = self._command("--locked")

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(AUTHORED_DRIFT_DIAGNOSTIC, result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_stale_valid_authored_headline_rejects_standalone_validation(self):
        """Validation cannot substitute an older resolved profile for current valid authored content."""
        # Arrange
        self._generate()
        profile = read_toml(self.root / "config/profile.toml")
        self._replace_profile(f"headline = {json.dumps(profile['profile']['headline'])}",
                              'headline = "Consumer-authored updated headline."')
        outputs = self._owned_outputs()

        # Act
        result = self._validate()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(AUTHORED_DRIFT_DIAGNOSTIC, result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_missing_remote_capture_rejects_offline_generation_without_output_changes(self):
        """Public observation fallback cannot replace the required canonical organization capture."""
        # Arrange
        self._generate()
        (self.root / "assets/import-capture.json").unlink()
        outputs = self._owned_outputs()

        # Act
        result = self._command()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("offline remote imports require captured inputs", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_missing_selected_retained_snapshot_rejects_generation_despite_available_seed(self):
        """An authored seed cannot replace explicitly selected retained source data that is missing."""
        # Arrange
        self._generate()
        seed = self.root / "config/offline-snapshot.json"
        seed_bytes = seed.read_bytes()
        retained = self.root / "assets/source-snapshot.json"
        retained.unlink()
        outputs = self._owned_outputs()

        # Act
        result = self._command()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(retained.exists())
        self.assertEqual(seed.read_bytes(), seed_bytes)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_tampered_remote_manifest_rejects_offline_generation_without_output_changes(self):
        """Canonical manifest bytes must match their saved digest even outside locked replay."""
        # Arrange
        self._generate()
        capture = read_json(self.root / "assets/import-capture.json")
        capture["imports"][0]["manifest"] += "\n# Corrupt captured manifest.\n"
        self._write_capture(capture)
        outputs = self._owned_outputs()

        # Act
        result = self._command()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("captured manifest digest mismatch", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_missing_remote_commit_rejects_offline_generation_without_output_changes(self):
        """A remote capture cannot publish without its real resolved commit identity."""
        # Arrange
        self._generate()
        capture = read_json(self.root / "assets/import-capture.json")
        capture["imports"][0]["commit"] = None
        self._write_capture(capture)
        outputs = self._owned_outputs()

        # Act
        result = self._command()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("remote capture has no commit", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_wrong_captured_repository_rejects_generation_without_output_changes(self):
        """Canonical provenance cannot claim a different repository from the authored source."""
        # Arrange
        self._generate()
        capture = read_json(self.root / "assets/import-capture.json")
        capture["imports"][0]["repository"] = "unrelated/organization"
        self._write_capture(capture)
        outputs = self._owned_outputs()

        # Act
        result = self._command()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("captured repository differs", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_tampered_capture_rejects_locked_replay_without_output_changes(self):
        """The replay generation record detects changed canonical capture bytes."""
        # Arrange
        self._generate()
        path = self.root / "assets/import-capture.json"
        path.write_bytes(path.read_bytes() + b"\n")
        outputs = self._owned_outputs()

        # Act
        result = self._command("--locked")

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("generation provenance input digest mismatch: import-capture.json", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_tampered_effective_render_snapshot_rejects_locked_replay_without_output_changes(self):
        """The schema 2 record rejects changed effective rendering bytes independently of the durable source."""
        # Arrange
        self._generate()
        source_bytes = (self.root / "assets/source-snapshot.json").read_bytes()
        render_path = self.root / "assets/render-snapshot.json"
        render_path.write_bytes(render_path.read_bytes() + b"\n")
        outputs = self._owned_outputs()

        # Act
        result = self._command("--locked")

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("generation provenance input digest mismatch: render-snapshot.json", result.stderr)
        self.assertEqual((self.root / "assets/source-snapshot.json").read_bytes(), source_bytes)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_generator_identity_mismatch_rejects_standalone_validation_without_output_changes(self):
        """Validation reports generator identity drift separately from authored input or capture corruption."""
        # Arrange
        self._generate()
        path = self.root / "assets/generation-record.json"
        record = self._assert_generation_record()
        record["generator_fingerprint"] = "0" * 64
        path.write_text(json.dumps(record) + "\n", encoding="utf-8")
        outputs = self._owned_outputs()

        # Act
        result = self._validate()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(GENERATOR_MISMATCH_DIAGNOSTIC, result.stderr)
        self.assertNotIn(AUTHORED_DRIFT_DIAGNOSTIC, result.stderr)
        self.assertNotIn("generation provenance input digest mismatch", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_missing_generation_record_rejects_standalone_validation(self):
        """Deleting provenance cannot downgrade captured validation to the direct-authored path."""
        # Arrange
        self._generate()
        (self.root / "assets/generation-record.json").unlink()
        outputs = self._owned_outputs()

        # Act
        result = self._validate()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("generation-record.json", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_missing_remote_capture_rejects_standalone_validation(self):
        """Standalone validation requires the remote capture that produced the resolved consumer."""
        # Arrange
        self._generate()
        (self.root / "assets/import-capture.json").unlink()
        outputs = self._owned_outputs()

        # Act
        result = self._validate()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("validate generation provenance", result.stderr)
        self.assertIn("import-capture.json", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_tampered_remote_capture_rejects_standalone_validation(self):
        """Standalone validation detects changed capture bytes before consuming resolved output."""
        # Arrange
        self._generate()
        path = self.root / "assets/import-capture.json"
        path.write_bytes(path.read_bytes() + b"\n")
        outputs = self._owned_outputs()

        # Act
        result = self._validate()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("generation provenance input digest mismatch: import-capture.json", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_resolved_input_tampering_rejects_validation_even_with_updated_record_digest(self):
        """Validation recomposes captured canonical facts instead of trusting a rehashed resolved file."""
        # Arrange
        self._generate()
        resolved_path = self.root / "assets/resolved-config.json"
        resolved = read_json(resolved_path)
        resolved["profile"]["headline"] = "Unrelated resolved profile."
        resolved_path.write_text(json.dumps(resolved) + "\n", encoding="utf-8")
        record_path = self.root / "assets/generation-record.json"
        record = read_json(record_path)
        record["inputs"]["resolved-config.json"] = hashlib.sha256(resolved_path.read_bytes()).hexdigest()
        record_path.write_text(json.dumps(record) + "\n", encoding="utf-8")
        outputs = self._owned_outputs()

        # Act
        result = self._validate()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("captured inputs resolve differently from the recorded configuration", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_invalid_authored_url_rejects_generation_without_output_changes(self):
        """A malformed destination cannot be partially promoted into the actual consumer output set."""
        # Arrange
        self._generate()
        profile = read_toml(self.root / "config/profile.toml")
        self._replace_profile(f"pages_url = {json.dumps(profile['profile']['pages_url'])}",
                              'pages_url = "javascript:alert(1)"')
        outputs = self._owned_outputs()

        # Act
        result = self._command()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("HTTPS link", result.stderr)
        self.assertEqual(self._owned_outputs(), outputs)

    def test_malformed_authored_toml_rejects_generation_without_output_changes(self):
        """A parse failure preserves every managed artifact rather than publishing a partial candidate."""
        # Arrange
        self._generate()
        (self.root / "config/profile.toml").write_text("schema_version = 1\n[profile\n", encoding="utf-8")
        outputs = self._owned_outputs()

        # Act
        result = self._command()

        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("parse", result.stderr.lower())
        self.assertEqual(self._owned_outputs(), outputs)


if __name__ == "__main__":
    unittest.main()
