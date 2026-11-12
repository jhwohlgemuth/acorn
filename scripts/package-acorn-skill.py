#!/usr/bin/env python3
"""Stage the tracked ACORN skill with reproducible bytes and release metadata."""
import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import posixpath
import re
import subprocess
from urllib.parse import unquote, urlsplit
import zipfile

SOURCE = Path("crates/acorn-cli/assets/skills/acorn")


def digest(content):
    return hashlib.sha256(content).hexdigest()


def git(root, *args):
    return subprocess.check_output(["git", "-C", str(root), *args])


def package(files, version, revision, dirty):
    """Build a complete archive after checking local Markdown dependencies."""
    if "SKILL.md" not in files:
        raise ValueError("Missing SKILL.md")
    for name, content in files.items():
        path = PurePosixPath(name)
        if path.is_absolute() or ".." in path.parts or "\\" in name:
            raise ValueError("Invalid archive path: " + name)
        if any(part.startswith(".") or part in {"__pycache__", "node_modules"} for part in path.parts):
            raise ValueError("Unpublishable skill file: " + name)
        if path.suffix.lower() in {".pyc", ".pem", ".key"}:
            raise ValueError("Unpublishable skill file: " + name)
        if path.suffix.lower() == ".md":
            text = content.decode("utf-8")
            targets = re.findall(r"\]\(<?([^\s)>]+)>?(?:\s+[^)]*)?\)", text)
            targets += re.findall(r"^\s*\[[^]]+\]:\s*<?([^\s>]+)>?", text, re.MULTILINE)
            for target in targets:
                url = urlsplit(target)
                if url.scheme or url.netloc or not url.path:
                    continue
                relative = PurePosixPath(posixpath.normpath(str(path.parent / unquote(url.path))))
                if relative.is_absolute() or ".." in relative.parts or str(relative) not in files:
                    raise ValueError("Missing or out-of-bundle reference: " + name + " -> " + target)
    hashes = {name: digest(content) for name, content in sorted(files.items())}
    skill_revision = digest(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode())
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, content in sorted(files.items()):
            entry = zipfile.ZipInfo("acorn/" + name, (1980, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, content)
    archive_bytes = stream.getvalue()
    manifest = {
        "appVersion": version,
        "archive": {"file": "acorn-skill.zip", "sha256": digest(archive_bytes)},
        "compatibility": {"browser": "preview", "cli": version, "desktop": "planned"},
        "definition": {"file": "SKILL.md", "sha256": hashes["SKILL.md"]},
        "files": hashes,
        "schemaVersion": 1,
        "skillRevision": skill_revision,
        "source": "https://code.ornl.gov/research-enablement/acorn",
        "sourceDirty": dirty,
        "sourcePath": SOURCE.as_posix(),
        "sourceRevision": revision,
    }
    return {
        **{"acorn/" + name: content for name, content in files.items()},
        "SKILL.md": files["SKILL.md"],
        "acorn-skill.zip": archive_bytes,
        "manifest.json": (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(),
    }


def stage(output, artifacts, check=False):
    """Reject stale staging or replace only this packager's dedicated directory."""
    if output.is_symlink():
        raise ValueError("Output must not be a symlink")
    existing = tuple(output.rglob("*")) if output.exists() else ()
    if any(path.is_symlink() for path in existing):
        raise ValueError("Staging contains a symlink")
    actual = {path.relative_to(output).as_posix(): path.read_bytes() for path in existing if path.is_file()}
    if check:
        if actual != artifacts:
            raise ValueError("Skill staging is stale: regenerate for the current version and source")
    else:
        for name in sorted(set(actual) - set(artifacts)):
            (output / name).unlink()
        for name, content in artifacts.items():
            destination = output / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(content)


def workspace_version(root):
    text = (root / "Cargo.toml").read_text()
    section = re.search(r"^\[workspace\.package\]\s*\n(.*?)(?=^\[|\Z)", text, re.MULTILINE | re.DOTALL)
    version = re.search(r'^version\s*=\s*"([^"]+)"', section[1], re.MULTILINE) if section else None
    if not version:
        raise ValueError("Missing workspace.package.version")
    return version[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Reject stale or missing staging")
    parser.add_argument("--output", type=Path, default=Path("target/acorn-skill"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = root / args.output
    if not output.resolve().is_relative_to((root / "target").resolve()) or output.name != "acorn-skill":
        parser.error("Output must be an acorn-skill directory beneath target/")
    try:
        names = git(root, "ls-files", "-z", "--", str(SOURCE)).decode().split("\0")
        paths = tuple(Path(name) for name in names if name)
        if any((root / path).is_symlink() or any(parent.is_symlink() for parent in (root / path).parents) for path in paths):
            raise ValueError("Skill source must not contain symlinks")
        files = {path.relative_to(SOURCE).as_posix(): (root / path).read_bytes() for path in paths}
        dirty = bool(git(root, "status", "--porcelain", "--", str(SOURCE), "Cargo.toml"))
        artifacts = package(files, workspace_version(root), git(root, "rev-parse", "HEAD").decode().strip(), dirty)
        stage(output, artifacts, args.check)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, str(error) + "\n")
    print("Skill bundle verified" if args.check else "Skill bundle staged")


if __name__ == "__main__":
    main()
