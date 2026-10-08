"""Protect the personal consumer's approved content without requiring a generator."""

import hashlib
import json
from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
SQL_SERVER_PACKAGE = "Doka.EntityFrameworkCore.SafeMigrations.SqlServer"
REMOTE_SOURCE = {
    "kind": "remote",
    "repository": "doka-labs/.github",
    "ref": "main",
    "path": "config/organization.toml",
}
TECHNOLOGY_IDS = {
    "age", "angular", "csharp", "css", "docker", "dotnet", "go", "google-drive", "html", "javascript",
    "linux", "lua", "macos", "mariadb", "mcp", "mysql", "ollama", "php", "postgresql", "python",
    "roslyn", "rust", "rust-analyzer", "shell", "sql-dialects", "sqlcipher", "sqlite", "swift", "typescript", "wasm",
}
DOKA_AFFINITIES = {
    "angular", "csharp", "css", "docker", "dotnet", "html", "javascript", "linux", "mariadb", "mysql",
    "postgresql", "roslyn", "sql-dialects", "swift", "typescript",
}
DOKA_SHARED_CAPABILITIES = {"angular", "css", "docker", "html", "linux", "roslyn", "sql-dialects", "typescript"}
APPROVED_PROJECT_LAYOUT = {
    "project:ai-devops": ([315.0, 298.0], 55.0, 0.8),
    "project:budget-board": ([159.0, 529.0], 43.0, 0.8),
    "project:state-trace": ([607.0, 688.0], 48.0, 0.8),
    "project:dotconfig": ([227.0, 754.0], 36.0, 0.8),
    "project:vscode-theme": ([704.0, 366.0], 37.0, 0.8),
    "project:sourcefield": ([547.0, 267.0], 47.0, 0.8),
    "project:doka-labs/mysql": ([1090.0, 318.0], 48.0, 0.8),
    "project:doka-labs/nested-set": ([1525.0, 726.0], 48.0, 0.8),
    "project:doka-labs/relational-lab": ([1080.0, 766.0], 42.0, 0.8),
    "project:doka-labs/safe-migrations": ([1530.0, 358.0], 51.0, 0.8),
}
PRIVATE_PROJECT_STACKS = {
    "ai-devops": ["Rust", "6 MCP endpoints"],
    "budget-board": ["Go", "SQLCipher", "age"],
    "state-trace": ["Rust", "Swift", "SQLite"],
    "dotconfig": ["Zsh", "Neovim", "Git", "Starship"],
    "vscode-theme": ["TypeScript", "JSON"],
}


def read_toml(path):
    """Read authored TOML using the standard library."""
    return tomllib.loads(path.read_text(encoding="utf-8"))


def read_json(path):
    """Read emitted JSON without importing Sourcefield implementation code."""
    return json.loads(path.read_text(encoding="utf-8"))


def captured_organization(root=ROOT):
    """Read the remote canonical manifest retained inside this consumer."""
    capture = read_json(root / "assets/import-capture.json")
    saved = next(item for item in capture["imports"] if item["id"] == "doka-labs")

    return saved, tomllib.loads(saved["manifest"])


class AuthoredPersonalTests(unittest.TestCase):
    """Keep remote composition, full technology coverage, private content, and layout explicit."""

    def test_thin_consumer_has_no_copied_generator_or_build_files(self):
        """A generated consumer cannot retain generator code or build inputs; empty directories are allowed."""
        # Arrange
        build_files = (
            "Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml", "rustfmt", "rustfmt.toml",
            ".rustfmt.toml", "Makefile", ".cargo/config", ".cargo/config.toml",
        )
        generator_directories = ("crates", "scripts", "runtime")

        # Act
        retained = [name for name in build_files if (ROOT / name).is_file()]
        retained.extend(str(path.relative_to(ROOT)) for name in generator_directories
                        for path in (ROOT / name).rglob("*") if path.is_file())

        # Assert
        self.assertEqual(sorted(retained), [], "Thin consumer retained generator or build files")

    def test_approved_hardware_descriptions_remain_unchanged(self):
        """Migration retains all three approved hardware descriptions in their presentation order."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        hardware = profile["presentation"]["hardware"]

        # Assert
        self.assertEqual(hardware, [
            {"label": "MacBook Pro", "detail": "M5 Pro / 64 GB"},
            {"label": "AMD Ryzen\u2122 AI Max+ 395", "detail": "128 GB unified memory"},
            {"label": "Proxmox host", "detail": "For self-hosting."},
        ])

    def test_approved_interests_retain_their_public_labels_and_order(self):
        """The two authored interests retain their exact public text and README visibility."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        interests = profile["interests"]

        # Assert
        self.assertEqual(interests, [
            {"id": "other", "label": "AI, self-hosting, Grafana, security.",
             "show_in_readme": True, "summary": "Personal interests."},
            {"id": "functional-programming", "label": "Elixir / Erlang / OCaml",
             "show_in_readme": True, "summary": "Languages I want to explore."},
        ])

    def test_approved_learning_destinations_and_membership_note_remain_unchanged(self):
        """Both learning memberships keep their destinations, interest binding, and qualification."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        learning = profile["learning"]
        note = profile["presentation"]["learning_note"]

        # Assert
        self.assertEqual(learning, [
            {"id": "hacking-akademie", "interest": "other", "label": "Hacking Akademie",
             "url": "https://hacking-akademie.de/"},
            {"id": "cybersec-academy", "interest": "other", "label": "Cybersec Academy",
             "url": "https://cybersec-academy.de/"},
        ])
        self.assertEqual(note, "Member of both.\nLearning when I have the time.")

    def test_approved_personal_presentation_preserves_stack_platforms_and_identity(self):
        """The consumer keeps its reviewed stack order, platform note, identity, and display settings."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        presentation = profile["presentation"]
        identity = profile["profile"]
        render = profile["render"]

        # Assert
        self.assertEqual(presentation["main_stack"], ["C#", ".NET", "Rust", "Go", "TypeScript", "Angular"])
        self.assertEqual(presentation["supporting_stack"], ["Python", "PHP", "Lua", "Shell", "HTML", "CSS", "SQL"])
        self.assertEqual(presentation["platforms"], ["macOS", "Windows", "Linux"])
        self.assertEqual(presentation["platform_note"], "MacBook for work and personal use.")
        self.assertEqual(identity["display_name"], "Dominic K.")
        self.assertEqual(identity["headline"], ".NET, developer tools, and open source.")
        self.assertEqual(identity["tagline"], "Dominic K. \u00b7 .NET, developer tools, and open source.")
        self.assertEqual(identity["pages_url"], "https://kdominic89.github.io/kdominic89/")
        self.assertEqual(identity["source_url"], "https://github.com/kdominic89/kdominic89")
        self.assertEqual(render, {
            "detail_level": "abstract", "height": 1885, "motion_seconds": 32, "show_activity_orbit": True,
            "show_interests_in_readme": True, "show_state_hash": True, "show_technology_labels": True, "width": 1800,
        })

    def test_profile_uses_one_remote_canonical_import_without_duplicated_projects(self):
        """Personal presentation consumes public canonical organization facts once."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        imports = profile["imports"]

        # Assert
        self.assertEqual(imports, [{"id": "doka-labs", "source": REMOTE_SOURCE}])
        self.assertEqual(profile["profile"]["username"], "kdominic89")
        self.assertEqual(profile["profile"]["variant"], "personal")
        self.assertEqual(profile["profile"]["organization"], "doka-labs")
        self.assertEqual([domain["id"] for domain in profile["domains"]], ["personal"])
        self.assertEqual({project["domain"] for project in profile["projects"]}, {"personal"})
        self.assertEqual(profile["shared_technologies"], {
            "doka-labs/csharp": "csharp", "doka-labs/dotnet": "dotnet", "doka-labs/javascript": "javascript",
        })
        self.assertFalse((ROOT / "config/organization.toml").exists())
        self.assertFalse({"publications", "maintainer"} & profile.keys())
        self.assertEqual(profile["collection"]["github_user"], "kdominic89")
        self.assertEqual(profile["collection"]["github_organizations"], ["doka-labs"])
        self.assertTrue(profile["collection"]["collect_contributions"])
        self.assertFalse(profile["collection"]["collect_private_repository_count"])

    def test_all_thirty_technologies_and_fifteen_doka_affinities_are_authored(self):
        """Imported-domain admission must preserve the complete personal technology inventory."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        technologies = {item["id"]: item for item in profile["technologies"]}
        affinities = {identifier for identifier, item in technologies.items() if "doka-labs" in item["affinities"]}

        # Assert
        self.assertEqual(len(profile["technologies"]), 30)
        self.assertEqual(set(technologies), TECHNOLOGY_IDS)
        self.assertEqual(affinities, DOKA_AFFINITIES)
        self.assertEqual(len(affinities), 15)

    def test_approved_personal_and_imported_project_geometry_remains_explicit(self):
        """Migration preserves all prior project rings and adds the approved Sourcefield ring."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        layout = {f"project:{item['id']}": (item["anchor"], item["radius"], item["weight"])
                  for item in profile["projects"]}
        layout.update({identifier: (profile["layout"]["overrides"][identifier],
                                    profile["layout"]["radii"][identifier],
                                    profile["layout"]["weights"][identifier])
                       for identifier in APPROVED_PROJECT_LAYOUT if identifier.startswith("project:doka-labs/")})

        # Assert
        self.assertEqual(layout, APPROVED_PROJECT_LAYOUT)
        self.assertEqual(profile["layout"]["overrides"][f"package:{SQL_SERVER_PACKAGE}"], [666.0, 1427.0])
        self.assertEqual(profile["render"]["height"], 1885)

    def test_five_private_projects_preserve_public_abstract_stacks_without_destinations(self):
        """Private projects disclose only the authored public description, stack, and components."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        private = {item["id"]: item for item in profile["projects"] if item["visibility"] == "private-abstract"}

        # Assert
        self.assertEqual({identifier: item["display_stack"] for identifier, item in private.items()},
                         PRIVATE_PROJECT_STACKS)
        for identifier, item in private.items():
            self.assertFalse({"url", "repository"} & item.keys(), identifier)
            self.assertTrue(item["summary"].strip(), identifier)
            self.assertTrue(item["show_in_readme"], identifier)

        self.assertEqual(len(private["ai-devops"]["components"]), 6)
        self.assertEqual({component["id"] for component in private["ai-devops"]["components"]}, {
            "graph-mcp", "memory-mcp", "knowledge-mcp", "experience-mcp", "workflow-mcp", "telemetry-mcp",
        })

    def test_sourcefield_is_the_sixth_personal_project_with_approved_public_motif(self):
        """The new project is an active public Rust and WebAssembly consumer."""
        # Arrange
        profile = read_toml(ROOT / "config/profile.toml")

        # Act
        sourcefield = next(item for item in profile["projects"] if item["id"] == "sourcefield")

        # Assert
        self.assertEqual(len(profile["projects"]), 6)
        self.assertEqual(sourcefield["visibility"], "public")
        self.assertEqual(sourcefield["status"], "active")
        self.assertEqual(sourcefield["label"], "Sourcefield")
        self.assertEqual(sourcefield["surface_label"], "Sourcefield")
        self.assertEqual(sourcefield["summary"], "GitHub profile diagrams")
        self.assertEqual(sourcefield["repository"], "kdominic89/sourcefield")
        self.assertEqual(sourcefield["icon"], "builtin:sourcefield")
        self.assertEqual(sourcefield["display_stack"], ["Rust", "WebAssembly"])
        self.assertEqual(sourcefield["implemented_with"], ["rust"])
        self.assertTrue(sourcefield["show_in_readme"])

    def test_remote_capture_binds_canonical_facts_to_repository_commit_and_manifest_bytes(self):
        """The public canonical capture contains four projects, ten packages, and truthful provenance."""
        # Arrange
        saved, organization = captured_organization()

        # Act
        packages = {item["id"]: item for publication in organization["publications"]
                    for item in publication["packages"]}
        projects = {item["id"]: item for item in organization["projects"]}

        # Assert
        self.assertEqual(saved["source"], REMOTE_SOURCE)
        self.assertEqual(saved["repository"], REMOTE_SOURCE["repository"])
        self.assertRegex(saved["commit"], r"^[0-9a-f]{40}$")
        self.assertEqual(saved["digest"], hashlib.sha256(saved["manifest"].encode("utf-8")).hexdigest())
        self.assertEqual(organization["id"], "doka-labs")
        self.assertEqual(organization["maintainer"], {
            "username": "kdominic89", "role": "Administrator & Core Maintainer", "url": "https://github.com/kdominic89",
        })
        self.assertEqual(set(projects), {"mysql", "safe-migrations", "relational-lab", "nested-set"})
        self.assertEqual(len(packages), 10)
        self.assertEqual(projects["safe-migrations"]["icon"], "builtin:database-safe")
        self.assertIn("SQL Server", projects["safe-migrations"]["summary"])
        self.assertEqual(packages[SQL_SERVER_PACKAGE]["summary"], "SQL Server adapter")
        self.assertEqual(packages[SQL_SERVER_PACKAGE]["url"],
                         f"https://www.nuget.org/packages/{SQL_SERVER_PACKAGE}/")
        self.assertFalse({"url", "repository"} & projects["relational-lab"].keys())

    def test_all_twenty_four_historical_states_remain_present(self):
        """The actual consumer retains the full pre-migration history rather than synthetic substitutes."""
        # Arrange
        directory = ROOT / "docs/history"

        # Act
        index = read_json(directory / "index.json")
        archives = {path.name for path in directory.glob("*.json")} - {"index.json"}

        # Assert
        self.assertEqual(index["schema_version"], 3)
        self.assertEqual(len(index["states"]), 24)
        self.assertEqual({item["file"] for item in index["states"]}, archives)
        self.assertEqual(len(archives), 24)
        for item in index["states"]:
            self.assertEqual(item["file"], f"{item['hash'].lower()}.json")
            self.assertEqual(read_json(directory / item["file"])["semantic_hash"], item["hash"])


if __name__ == "__main__":
    unittest.main()
