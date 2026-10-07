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
    "docs/USER-GUIDE.md": "README.md",
    "docs/USER-GUIDE.zh-CN.md": "README.zh-CN.md",
    "docs/cheats.md": "cheats.md",
    "CHANGELOG.md": "CHANGELOG.md",
    "LICENSE": "LICENSE",
    "THIRD_PARTY_NOTICES.md": "THIRD_PARTY_NOTICES.md",
    "ui/data/README.md": "OFFICIAL_STATS_NOTICE.md",
}


def version_and_notes(root=ROOT, tag=None):
    version = json.loads((root / "package.json").read_text(encoding="utf-8"))["version"]
    lock = json.loads((root / "package-lock.json").read_text(encoding="utf-8"))
    tauri = json.loads((root / "apps/desktop/tauri.conf.json").read_text(encoding="utf-8"))
    cargo = (root / "Cargo.toml").read_text(encoding="utf-8")
    cargo_version = re.search(r'\[workspace.package\]\s*version = "([^"]+)"', cargo)
    if not cargo_version:
        raise ValueError("Missing Cargo workspace version")
    versions = [lock["version"], lock["packages"][""]["version"],
                tauri["version"], cargo_version[1]]
    if any(v != version for v in versions):
        raise ValueError(f"Version mismatch: npm={version}, other manifests={versions}")
    if tag is not None and tag != f"v{version}":
        raise ValueError(f"Tag {tag} does not match v{version}")
    log = (root / "CHANGELOG.md").read_text(encoding="utf-8")
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
        "signing": ("ad hoc signed / not notarized" if platform == "macos-arm64"
                    else "unsigned"),
        "runtime_tested_by_this_workflow": False,
        "files": {f.name: hashlib.sha256(f.read_bytes()).hexdigest()
                  for f in sorted(output.iterdir())},
    }
    (output / f"build-info-{platform}.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def verify_artifacts(assets, version, commit):
    """Require a complete, untampered two-platform set from the approved source."""
    suffixes = {"macos-arm64": ".dmg", "windows-x64": "-setup.exe"}
    expected = {"SHA256SUMS.txt"}
    for platform, suffix in suffixes.items():
        prefix = f"Gen3RomHackEditor-{version}-{platform}"
        expected.update([prefix + suffix, prefix + "-docs.zip", f"build-info-{platform}.json"])
    if {f.name for f in assets.iterdir()} != expected:
        raise ValueError("Expected exactly both platform installers, docs, manifests and checksums")
    hashes = {}
    for line in (assets / "SHA256SUMS.txt").read_text(encoding="utf-8").splitlines():
        digest, name = line.split("  ", 1)
        if name not in expected or name in hashes or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError("Invalid or duplicate checksum entry")
        path = assets / name
        if path.is_symlink() or not path.is_file():
            raise ValueError("Release assets must be regular files")
        if hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise ValueError(f"Checksum mismatch: {name}")
        hashes[name] = digest
    if set(hashes) != expected - {"SHA256SUMS.txt"}:
        raise ValueError("Incomplete checksum list")
    for platform, suffix in suffixes.items():
        prefix = f"Gen3RomHackEditor-{version}-{platform}"
        info = json.loads((assets / f"build-info-{platform}.json").read_text(encoding="utf-8"))
        if (info["version"], info["commit"], info["platform"]) != (version, commit, platform):
            raise ValueError(f"Source provenance mismatch: {platform}")
        names = {prefix + suffix, prefix + "-docs.zip"}
        if set(info["files"]) != names or any(hashes[name] != info["files"][name] for name in names):
            raise ValueError(f"Manifest checksum mismatch: {platform}")
        with zipfile.ZipFile(assets / (prefix + "-docs.zip")) as docs:
            if set(docs.namelist()) != set(DOCS.values()) or docs.testzip() is not None:
                raise ValueError(f"Invalid player documentation: {platform}")


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
    verify = commands.add_parser("verify")
    verify.add_argument("--assets", type=Path, required=True)
    verify.add_argument("--commit", required=True)
    verify.add_argument("--tag", required=True)
    args = parser.parse_args()
    if args.command == "check":
        version, notes = version_and_notes(tag=args.tag)
        if args.notes:
            args.notes.write_text(notes, encoding="utf-8")
        print(version)
    elif args.command == "stage":
        stage(args.bundle, args.output, args.platform)
    else:
        version, _ = version_and_notes(tag=args.tag)
        verify_artifacts(args.assets, version, args.commit)
        print(f"Verified both platforms for {args.tag} at {args.commit}")


if __name__ == "__main__":
    main()
