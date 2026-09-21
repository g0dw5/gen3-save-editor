"""Validate release versions and stage explicitly selected public artifacts."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import zipfile


ROOT = Path(__file__).resolve().parents[1]
DOCS = {
    "README.md": "README.md",
    "README.zh-CN.md": "README.zh-CN.md",
    "CHANGELOG.md": "CHANGELOG.md",
    "LICENSE": "LICENSE",
    "THIRD_PARTY_NOTICES.md": "THIRD_PARTY_NOTICES.md",
    "ui/data/README.md": "OFFICIAL_STATS_NOTICE.md",
    "docs/windows-build.md": "windows-build.md",
}


def version_and_notes(root=ROOT, tag=None):
    version = json.loads((root / "package.json").read_text())["version"]
    lock = json.loads((root / "package-lock.json").read_text())
    tauri = json.loads((root / "apps/desktop/tauri.conf.json").read_text())
    cargo = (root / "Cargo.toml").read_text()
    cargo_version = re.search(r'\[workspace.package\]\s*version = "([^"]+)"', cargo)
    if not cargo_version:
        raise ValueError("Missing Cargo workspace version")
    versions = [lock["version"], lock["packages"][""]["version"],
                tauri["version"], cargo_version[1]]
    if any(v != version for v in versions):
        raise ValueError(f"Version mismatch: npm={version}, other manifests={versions}")
    if tag is not None and tag != f"v{version}":
        raise ValueError(f"Tag {tag} does not match v{version}")
    log = (root / "CHANGELOG.md").read_text()
    match = re.search(rf"^## {re.escape(version)} — ([^\n]+)\n(.*?)(?=^## |\Z)",
                      log, re.M | re.S)
    if not match:
        raise ValueError(f"Missing changelog entry for {version}")
    if tag is not None and not re.fullmatch(r"\d{4}-\d{2}-\d{2}", match[1]):
        raise ValueError("A release tag requires a dated, released changelog entry")
    return version, match[2].strip() + "\n"


def stage(bundle, output, platform):
    version, _ = version_and_notes()
    patterns = {"macos-arm64": ("dmg/*.dmg", ".dmg"),
                "windows-x64": ("nsis/*-setup.exe", "-setup.exe")}
    pattern, suffix = patterns[platform]
    candidates = list(bundle.glob(pattern))
    if len(candidates) != 1:
        raise ValueError(f"Expected one {platform} installer, found {candidates}")
    output.mkdir(parents=True, exist_ok=True)
    if any(output.iterdir()):
        raise ValueError("Use an empty staging directory; never mix release artifacts")
    name = f"Gen3RomHackEditor-{version}-{platform}"
    installer = output / (name + suffix)
    shutil.copyfile(candidates[0], installer)
    with zipfile.ZipFile(output / (name + "-docs.zip"), "w",
                         zipfile.ZIP_DEFLATED) as archive:
        for source, destination in DOCS.items():
            archive.write(ROOT / source, destination)
    manifest = {
        "version": version,
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT,
                                         text=True).strip(),
        "platform": platform,
        "signing": "unsigned / not notarized",
        "runtime_tested_by_this_workflow": False,
        "files": {f.name: hashlib.sha256(f.read_bytes()).hexdigest()
                  for f in sorted(output.iterdir())},
    }
    (output / f"build-info-{platform}.json").write_text(
        json.dumps(manifest, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check")
    check.add_argument("--tag")
    check.add_argument("--notes", type=Path)
    package = commands.add_parser("stage")
    package.add_argument("--bundle", type=Path, required=True)
    package.add_argument("--output", type=Path, required=True)
    package.add_argument("--platform", choices=["macos-arm64", "windows-x64"],
                         required=True)
    args = parser.parse_args()
    if args.command == "check":
        version, notes = version_and_notes(tag=args.tag)
        if args.notes:
            args.notes.write_text(notes)
        print(version)
    else:
        stage(args.bundle, args.output, args.platform)


if __name__ == "__main__":
    main()
