"""Release invariants for the complete ACORN skill bundle."""
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

SPEC = importlib.util.spec_from_file_location("package_skill", Path(__file__).parents[1] / "package-acorn-skill.py")
PACKAGER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGER)


class PackageTests(unittest.TestCase):
    def test_archive_is_complete_and_reproducible(self):
        files = {"SKILL.md": b"See [guide](references/guide.md).", "references/guide.md": b"[Back](../SKILL.md)\n"}
        first = PACKAGER.package(files, "1.2.3", "a" * 40, False)
        second = PACKAGER.package(dict(reversed(list(files.items()))), "1.2.3", "a" * 40, False)
        self.assertEqual(first, second)
        with zipfile.ZipFile(io.BytesIO(first["acorn-skill.zip"])) as archive:
            self.assertEqual(archive.namelist(), ["acorn/SKILL.md", "acorn/references/guide.md"])
            for name, content in files.items():
                self.assertEqual(archive.read("acorn/" + name), content)
        manifest = json.loads(first["manifest.json"])
        self.assertEqual(manifest["archive"]["sha256"], PACKAGER.digest(first["acorn-skill.zip"]))
        self.assertEqual(manifest["definition"]["sha256"], PACKAGER.digest(files["SKILL.md"]))
        self.assertEqual(first["SKILL.md"], files["SKILL.md"])
    def test_changed_source_changes_skill_revision(self):
        first = PACKAGER.package({"SKILL.md": b"one"}, "1.2.3", "a" * 40, False)
        second = PACKAGER.package({"SKILL.md": b"two"}, "1.2.3", "a" * 40, True)
        self.assertNotEqual(json.loads(first["manifest.json"])["skillRevision"], json.loads(second["manifest.json"])["skillRevision"])
        self.assertTrue(json.loads(second["manifest.json"])["sourceDirty"])
    def test_missing_and_escaping_references_fail(self):
        for text in [b"[guide](missing.md)", b"[guide](../secret)", b"[guide](/secret)", b"[guide][ref]\n[ref]: absent.md", b"[guide](%2e%2e/secret)"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                PACKAGER.package({"SKILL.md": text}, "1.2.3", "a" * 40, False)
    def test_missing_definition_and_private_files_fail(self):
        for files in [{}, {"SKILL.md": b"ok", ".env": b"secret"}, {"SKILL.md": b"ok", "key.pem": b"secret"}, {"SKILL.md": b"ok", "__pycache__/cache": b"cache"}]:
            with self.subTest(files=list(files)), self.assertRaises(ValueError):
                PACKAGER.package(files, "1.2.3", "a" * 40, False)
    def test_stale_version_source_extra_and_corrupt_bytes_fail(self):
        artifacts = PACKAGER.package({"SKILL.md": b"one"}, "1.2.3", "a" * 40, False)
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / "acorn-skill"
            PACKAGER.stage(output, artifacts)
            PACKAGER.stage(output, artifacts, check=True)
            changed = PACKAGER.package({"SKILL.md": b"one"}, "1.2.4", "a" * 40, False)
            with self.assertRaises(ValueError):
                PACKAGER.stage(output, changed, check=True)
            (output / "unexpected.txt").write_text("stale")
            with self.assertRaises(ValueError):
                PACKAGER.stage(output, artifacts, check=True)
            PACKAGER.stage(output, artifacts)
            self.assertFalse((output / "unexpected.txt").exists())
            (output / "SKILL.md").write_bytes(b"corrupt")
            with self.assertRaises(ValueError):
                PACKAGER.stage(output, artifacts, check=True)
    def test_symlink_staging_fails_without_touching_target(self):
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / "acorn-skill"
            output.mkdir()
            target = Path(folder) / "user-file"
            target.write_bytes(b"preserve")
            (output / "SKILL.md").symlink_to(target)
            with self.assertRaises(ValueError):
                PACKAGER.stage(output, {"SKILL.md": b"new"})
            self.assertEqual(target.read_bytes(), b"preserve")
    def test_workspace_version_ignores_other_package_versions(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.6.3"\n[dependencies]\nversion = "2"\n')
            self.assertEqual(PACKAGER.workspace_version(root), "0.6.3")


if __name__ == "__main__":
    unittest.main()
