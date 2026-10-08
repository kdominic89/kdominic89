"""Protect the personal consumer's release, privacy, and publication controls."""

import re
import unittest

from test_consumer_contract import ROOT, read_json


SOURCEFIELD_COMMIT = "b12eb4c72d60fbc075776a6ccc4bc15736db28af"


def mapping_body(content, key, indent=0):
    """Select one indentation-delimited authored mapping without adding a YAML parser dependency."""
    lines = content.splitlines()
    start = lines.index(f"{' ' * indent}{key}:") + 1
    body = []
    for line in lines[start:]:
        if line.strip() and not line.lstrip().startswith("#") and len(line) - len(line.lstrip()) <= indent:
            break

        body.append(line)

    return "\n".join(body)


class PersonalWorkflowTests(unittest.TestCase):
    """Check the consumer-specific choices in the actual thin update and validation callers."""

    def test_daily_and_manual_updates_require_explicit_private_count_intent(self):
        """Daily updates run at 03:17 UTC and private counts require the exact approved opt-in expression."""
        # Arrange
        update = (ROOT / ".github/workflows/update-profile.yml").read_text(encoding="utf-8")

        # Act
        triggers = mapping_body(update, "on")
        private_input = mapping_body(triggers, "include_private_count", 6)
        generate = mapping_body(update, "generate", 2)
        secrets = mapping_body(generate, "secrets", 4)

        # Assert
        self.assertEqual(mapping_body(triggers, "schedule", 2).strip(), "- cron: '17 3 * * *'")
        self.assertEqual({line.strip() for line in private_input.splitlines()}, {
            "description: Include only the aggregate number of private repositories (requires PROFILE_TOKEN)",
            "required: false", "type: boolean", "default: false",
        })
        self.assertIn(
            "include_private_count: ${{ (github.event_name == 'workflow_dispatch' && inputs.include_private_count) "
            "|| vars.SOURCEFIELD_PRIVATE_COUNTS == 'true' }}", generate,
        )
        self.assertEqual(secrets.strip(), "PROFILE_TOKEN: ${{ secrets.PROFILE_TOKEN }}")
        self.assertNotRegex(update, r"(?m)^\s*secrets:\s*inherit\s*$")

    def test_lock_and_both_callers_bind_the_approved_release_and_one_readme(self):
        """Update and read-only validation share the authenticated release and exactly one README destination."""
        # Arrange
        update = (ROOT / ".github/workflows/update-profile.yml").read_text(encoding="utf-8")
        validation = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")
        lock = read_json(ROOT / "sourcefield.lock.json")

        # Act
        generate = mapping_body(update, "generate", 2)
        tooling_checkout = re.search(
            r"(?m)^\s+repository: kdominic89/sourcefield\n\s+ref: (\S+)\n", validation,
        )
        readmes = re.findall(r"(?m)^\s+readmes: '([^']+)'$", generate)
        candidate_readmes = re.findall(r"--readmes '([^']+)'", validation)

        # Assert
        self.assertEqual(lock["schema_version"], 1)
        self.assertEqual(lock["repository"], "kdominic89/sourcefield")
        self.assertEqual(lock["release"], "v0.1.1")
        self.assertEqual(lock["source_commit"], SOURCEFIELD_COMMIT)
        self.assertIn(
            f"uses: kdominic89/sourcefield/.github/workflows/generate.yml@{SOURCEFIELD_COMMIT}", generate,
        )
        self.assertIsNotNone(tooling_checkout)
        self.assertEqual(tooling_checkout.group(1), SOURCEFIELD_COMMIT)
        self.assertIn(f"--own-commit {SOURCEFIELD_COMMIT}", validation)
        self.assertIn("--lock sourcefield.lock.json", validation)
        self.assertIn("--workflow .github/workflows/update-profile.yml", validation)
        self.assertEqual(readmes, ['["README.md"]'])
        self.assertEqual(candidate_readmes, ['["README.md"]'])
        self.assertEqual(mapping_body(validation, "permissions").strip(), "contents: read")
        self.assertNotRegex(validation, r"(?m)^\s*(?:contents|pages|id-token): write\s*$")
        self.assertNotIn("secrets.", validation)
        self.assertLess(validation.index("scripts/check_pin.py"), validation.index("scripts/bootstrap_release.py"))
        self.assertLess(validation.index("scripts/bootstrap_release.py"), validation.index("-m unittest discover"))

    def test_publication_orders_the_same_pages_candidate_after_expected_revision_git_update(self):
        """Pages upload precedes expected-revision Git publication and dependent deployment reuses that artifact."""
        # Arrange
        update = (ROOT / ".github/workflows/update-profile.yml").read_text(encoding="utf-8")

        # Act
        publish = mapping_body(update, "publish", 2)
        deploy = mapping_body(update, "deploy", 2)
        permissions = {line.strip() for line in mapping_body(deploy, "permissions", 4).splitlines()}

        # Assert
        self.assertEqual(mapping_body(update, "permissions").strip(), "contents: read")
        self.assertIn("needs: generate", publish)
        self.assertIn("needs: publish", deploy)
        self.assertEqual(mapping_body(publish, "permissions", 4).strip(), "contents: write")
        self.assertEqual(permissions, {"contents: read", "pages: write", "id-token: write"})
        self.assertIn("if: github.ref == format('refs/heads/{0}', github.event.repository.default_branch)", publish)
        self.assertIn("name: ${{ needs.generate.outputs.artifact }}", publish)
        self.assertIn("ref: ${{ needs.generate.outputs.source_commit }}", publish)
        self.assertIn("persist-credentials: false", publish)
        self.assertIn("ARTIFACT_NAME: github-pages-${{ github.run_id }}-${{ github.run_attempt }}", publish)
        self.assertIn("pages_artifact: ${{ steps.artifact_name.outputs.name }}", publish)
        self.assertIn("name: ${{ steps.artifact_name.outputs.name }}", publish)
        self.assertIn("path: ${{ runner.temp }}/candidate/docs", publish)
        self.assertIn("EXPECTED_COMMIT: ${{ github.sha }}", publish)
        self.assertIn("TARGET_BRANCH: ${{ github.event.repository.default_branch }}", publish)
        self.assertIn('--expected-commit "$EXPECTED_COMMIT" --branch "$TARGET_BRANCH"', publish)
        self.assertLess(publish.index("actions/upload-pages-artifact@"), publish.index("scripts/consumer_publish.py"))
        self.assertLess(publish.index("scripts/consumer_publish.py"), publish.index("git push origin"))
        self.assertIn("artifact_name: ${{ needs.publish.outputs.pages_artifact }}", deploy)
        self.assertNotIn("git push", deploy)
        self.assertNotIn("scripts/consumer_publish.py", deploy)


if __name__ == "__main__":
    unittest.main()
