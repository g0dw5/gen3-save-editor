"""Regression checks for source/artifact version alignment and release staging."""
import json
import hashlib
from pathlib import Path
import shutil
import sys
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
            f"## {self.version} — Unreleased / 未发布\n\nEnglish / 中文\n",
            encoding="utf-8")

    def test_development_allowed_but_not_release_tag(self):
        with self.assertRaisesRegex(ValueError, "dated"):
            release.version_and_notes(self.root, f"v{self.version}")

    def test_mismatched_tag_and_manifests_rejected(self):
        with self.assertRaisesRegex(ValueError, "does not match"):
            release.version_and_notes(self.root, "v999.0.0")
        path = self.root / "apps/desktop/tauri.conf.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        data["version"] = "999.0.0"
        path.write_text(json.dumps(data), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "Version mismatch"):
            release.version_and_notes(self.root)

    def test_dated_release_extracts_only_its_notes(self):
        path = self.root / "CHANGELOG.md"
        path.write_text(f"## {self.version} — 2026-01-01\n\nEnglish / 中文\n\n"
                        "## 0.0.1 — 2025-01-01\n\nPrevious\n", encoding="utf-8")
        _, notes = release.version_and_notes(self.root, f"v{self.version}")
        self.assertEqual(notes, "English / 中文\n")

    def test_read_notes_with_legacy_windows_text_defaults(self):
        path = self.root / "CHANGELOG.md"
        path.write_text(f"## {self.version} — 2026-10-07\n\nRelease / 发布说明\n",
                        encoding="utf-8")
        read_text = Path.read_text

        def legacy_read(path, encoding=None, errors=None):
            return read_text(path, encoding=encoding or "cp1252", errors=errors)

        with patch.object(Path, "read_text", legacy_read):
            _, notes = release.version_and_notes(self.root, f"v{self.version}")
        self.assertEqual(notes, "Release / 发布说明\n")

    def test_cli_notes_are_utf8_under_legacy_windows_text_defaults(self):
        output = self.root / "notes.md"
        notes = "Release / 发布说明\n"
        write_text = Path.write_text

        def legacy_write(path, data, encoding=None, errors=None):
            return write_text(path, data, encoding=encoding or "cp1252",
                              errors=errors)

        with patch.object(release, "version_and_notes", return_value=(self.version, notes)), \
                patch.object(Path, "write_text", legacy_write), \
                patch.object(sys, "argv", ["release.py", "check", "--notes", str(output)]):
            release.main()
        self.assertEqual(output.read_bytes().decode("utf-8"), notes)

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
        info = json.loads((output / "build-info-windows-x64.json").read_text(encoding="utf-8"))
        self.assertEqual(info["commit"], "test-sha")
        self.assertEqual(info["signing"], "unsigned")
        self.assertEqual(len(info["files"]), 2)
        import zipfile
        archive = next(output.glob("*-docs.zip"))
        with zipfile.ZipFile(archive) as docs:
            self.assertIn("README.md", docs.namelist())
            self.assertIn("README.zh-CN.md", docs.namelist())
            self.assertIn("cheats.md", docs.namelist())
            self.assertFalse(any("research/" in p or "verification/" in p for p in docs.namelist()))
            self.assertEqual(set(docs.namelist()), set(release.DOCS.values()))
        with self.assertRaisesRegex(ValueError, "empty staging"):
            release.stage(self.root, output, "windows-x64")

    def dual_platform_assets(self):
        assets = self.root / "assets"
        assets.mkdir()
        for platform, subdir, name in [
            ("windows-x64", "nsis", "fixture-setup.exe"),
            ("macos-arm64", "dmg", "fixture.dmg"),
        ]:
            bundle = self.root / platform
            (bundle / subdir).mkdir(parents=True)
            (bundle / subdir / name).write_bytes(b"installer fixture")
            output = self.root / (platform + "-staged")
            with patch.object(release.subprocess, "check_output", return_value="approved-source"):
                release.stage(bundle, output, platform)
            for artifact in output.iterdir():
                shutil.copyfile(artifact, assets / artifact.name)
        self.refresh_checksums(assets)
        return assets

    @staticmethod
    def refresh_checksums(assets):
        lines = [f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n"
                 for path in sorted(assets.iterdir()) if path.name != "SHA256SUMS.txt"]
        (assets / "SHA256SUMS.txt").write_text("".join(lines), encoding="utf-8")

    def test_verify_accepts_complete_approved_dual_platform_set(self):
        release.verify_artifacts(self.dual_platform_assets(), self.version, "approved-source")

    def test_verify_rejects_partial_platform_set(self):
        assets = self.dual_platform_assets()
        (assets / f"Gen3RomHackEditor-{self.version}-macos-arm64.dmg").unlink()
        with self.assertRaisesRegex(ValueError, "both platform"):
            release.verify_artifacts(assets, self.version, "approved-source")

    def test_verify_rejects_tampered_installer(self):
        assets = self.dual_platform_assets()
        (assets / f"Gen3RomHackEditor-{self.version}-windows-x64-setup.exe").write_bytes(b"altered")
        with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
            release.verify_artifacts(assets, self.version, "approved-source")

    def test_verify_rejects_other_source_even_with_matching_checksums(self):
        assets = self.dual_platform_assets()
        path = assets / "build-info-windows-x64.json"
        info = json.loads(path.read_text(encoding="utf-8"))
        info["commit"] = "unapproved-source"
        path.write_text(json.dumps(info), encoding="utf-8")
        self.refresh_checksums(assets)
        with self.assertRaisesRegex(ValueError, "Source provenance mismatch"):
            release.verify_artifacts(assets, self.version, "approved-source")


if __name__ == "__main__":
    unittest.main()
