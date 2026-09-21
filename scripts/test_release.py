"""Regression checks for source/artifact version alignment and release staging."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import release


class ReleaseChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in ["package.json", "package-lock.json", "Cargo.toml",
                     "apps/desktop/tauri.conf.json", "CHANGELOG.md"]:
            target = self.root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((release.ROOT / name).read_bytes())
        self.version, _ = release.version_and_notes(self.root)
        (self.root / "CHANGELOG.md").write_text(
            f"## {self.version} — Unreleased / 未发布\n\nEnglish / 中文\n")

    def test_development_allowed_but_not_release_tag(self):
        with self.assertRaisesRegex(ValueError, "dated"):
            release.version_and_notes(self.root, f"v{self.version}")

    def test_mismatched_tag_and_manifests_rejected(self):
        with self.assertRaisesRegex(ValueError, "does not match"):
            release.version_and_notes(self.root, "v999.0.0")
        path = self.root / "apps/desktop/tauri.conf.json"
        data = json.loads(path.read_text())
        data["version"] = "999.0.0"
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, "Version mismatch"):
            release.version_and_notes(self.root)

    def test_dated_release_extracts_only_its_notes(self):
        path = self.root / "CHANGELOG.md"
        path.write_text(f"## {self.version} — 2026-01-01\n\nEnglish / 中文\n\n"
                        "## 0.0.1 — 2025-01-01\n\nPrevious\n")
        _, notes = release.version_and_notes(self.root, f"v{self.version}")
        self.assertEqual(notes, "English / 中文\n")

    def test_staging_requires_exactly_one_installer(self):
        with self.assertRaisesRegex(ValueError, "Expected one"):
            release.stage(self.root, self.root / "out", "windows-x64")
        folder = self.root / "nsis"
        folder.mkdir()
        for name in ["a-setup.exe", "b-setup.exe"]:
            (folder / name).write_bytes(b"fixture")
        with self.assertRaisesRegex(ValueError, "Expected one"):
            release.stage(self.root, self.root / "out", "windows-x64")

    def test_staging_copies_only_allowlisted_files(self):
        folder = self.root / "nsis"
        folder.mkdir()
        (folder / "test-setup.exe").write_bytes(b"installer fixture")
        (folder / "private.gba").write_bytes(b"private ROM fixture")
        with patch.object(release.subprocess, "check_output", return_value="test-sha"):
            release.stage(self.root, self.root / "out", "windows-x64")
        output = self.root / "out"
        self.assertEqual(len(list(output.iterdir())), 3)
        self.assertFalse(any(f.suffix == ".gba" for f in output.iterdir()))
        info = json.loads((output / "build-info-windows-x64.json").read_text())
        self.assertEqual(info["commit"], "test-sha")
        self.assertEqual(len(info["files"]), 2)
        with self.assertRaisesRegex(ValueError, "empty staging"):
            release.stage(self.root, output, "windows-x64")


if __name__ == "__main__":
    unittest.main()
