#!/usr/bin/env python3
"""Verify MAN-06 using the frozen TEST ONLY record in a new, offline Git repository."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess


def checksum(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n").encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True, help="New directory only")
    parser.add_argument("--acorn", type=Path, required=True)
    parser.add_argument("--git", default="git")
    args = parser.parse_args()
    assets = Path(__file__).resolve().parent
    accepted = (assets / "selected.jsonl").read_bytes()
    provenance = json.loads((assets / "source-manifest.json").read_bytes())
    captured = (assets / "inbox-snapshot.jsonl").read_bytes()
    assert checksum(accepted) == "sha256:" + provenance["selectedSha256"]
    assert checksum(captured) == "sha256:" + provenance["snapshotSha256"]
    assert json.loads(accepted)["entry"]["summary"] == "Example, Avery (TEST ONLY)"
    root = args.workspace.resolve()
    root.mkdir(parents=True, exist_ok=False)
    index = root / "acorn/index.json"
    index.parent.mkdir()
    baseline = (assets / "baseline.json").read_bytes()
    index.write_bytes(baseline)
    source = root / "logbook.jsonl"
    source.write_bytes(captured)
    (root / "selected.jsonl").write_bytes(accepted)
    config = root / "intake.json"
    config.write_bytes(json_bytes({"schemaVersion": "2.0.0", "folders": [
        {"id": "acorn", "members": ["o9w@ornl.gov"], "path": index.as_uri()}
    ]}))
    evidence_dir = root / "evidence"
    evidence_dir.mkdir()
    env = {**os.environ, "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}
    commands = []

    def git(*arguments):
        return subprocess.check_output([args.git, "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgSign=false",
                                        "-c", "user.name=ACORN TEST ONLY", "-c", "user.email=demo@example.org",
                                        "-C", str(root), *arguments], env=env, text=True).strip()

    def run(label, *arguments):
        command = [str(args.acorn.resolve()), "--offline", "intake", *map(str, arguments)]
        result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=180)
        extension = ".txt" if label == "inspect" else ".json"
        (evidence_dir / (label + extension)).write_text(result.stdout)
        (evidence_dir / (label + ".stderr.txt")).write_text(result.stderr)
        commands.append({"command": command, "exitCode": result.returncode})
        assert result.returncode == 0, result.stderr
        return json.loads(result.stdout) if label != "inspect" else result.stdout

    git("init", "--initial-branch=main")
    git("add", "acorn/index.json", "logbook.jsonl")
    git("commit", "-m", "Synthetic original source and canonical baseline")
    original_commit = git("rev-parse", "HEAD")
    staged = run("stage", "stage", "--batch", "--config", config, "--source", root / "selected.jsonl")
    assert staged["accepted"] == 1 and staged["rejected"] == 0 and staged["canApply"]
    run("inspect", "inspect", "--config", config, "--batch-id", staged["batchId"])
    applied = run("apply", "apply", "--config", config, "--batch-id", staged["batchId"], "--yes")
    assert applied["status"] == "complete"
    ingested = index.read_bytes()
    preview = run("preview", "run", "--index", index, "--dry-run")
    assert len(preview["candidates"]) == 1
    candidate = preview["candidates"][0]
    assert candidate["target"] == "contributors" and candidate["value"] == "Example, Avery (TEST ONLY)"
    run("graduate", "run", "--index", index, "--accept", candidate["entryIdentifier"],
        "--expected-digest", preview["baseDigest"])
    graduated = index.read_bytes()
    expected = json.loads(ingested)
    expected["meta"]["contributors"] = [{"name": "Example, Avery (TEST ONLY)", "identifiers": [], "affiliations": [], "roles": []}]
    expected["logbook"]["entries"][0]["archived"] = True
    assert json.loads(graduated) == expected
    assert source.read_bytes() == captured
    assert Path(str(index) + ".graduation-" + checksum(ingested)[7:] + ".before").read_bytes() == ingested
    (evidence_dir / "graduated.json").write_bytes(graduated)
    git("add", "acorn/index.json")
    git("commit", "-m", "Synthetic reviewed manual application and graduation")
    canonical_commit = git("rev-parse", "HEAD")
    # Exercise blank, CRLF, Unicode, rejected, unknown and newly appended bytes.
    envelope = json.loads(accepted)
    new = {**envelope, "submissionId": "11111111-2222-4333-8444-555555555555"}
    rejected = {**envelope, "submissionId": "11111111-2222-4333-8444-666666666666", "projectId": "unknown-project"}
    preserved = b'\n{"unknown":"caf\xc3\xa9"}\r\n{invalid json}\n' + json_bytes(rejected) + json_bytes(new)
    current = captured + preserved
    source.write_bytes(current)
    git("add", "logbook.jsonl")
    git("commit", "-m", "Synthetic intervening inbox append")
    current_commit = git("rev-parse", "HEAD")
    source_digest, index_digest = checksum(current), checksum(graduated)
    proposal = reconcile(captured, current, accepted, source_digest, graduated, index_digest)
    assert proposal == captured.replace(accepted, b"", 1) + preserved
    checks = ["only exact reviewed line removed", "new/rejected/unknown/malformed/blank/CRLF/Unicode bytes preserved",
              "canonical contributor and archived provenance verified", "local apply leaves inbox unchanged"]
    changed_identity = json_bytes({**envelope, "submittedAt": "2026-10-25T00:00:00Z"})
    variants = [
        ("concurrent append invalidates review", current + b"\n", source_digest, graduated),
        ("canonical change invalidates review", current, source_digest, graduated + b"\n"),
        ("missing accepted line refuses cleanup", proposal, checksum(proposal), graduated),
        ("duplicate exact line refuses cleanup", current + accepted, checksum(current + accepted), graduated),
        ("changed accepted bytes refuse cleanup", current.replace(accepted, changed_identity),
         checksum(current.replace(accepted, changed_identity)), graduated),
        ("conflicting identity refuses cleanup", current + changed_identity, checksum(current + changed_identity), graduated),
        ("nonterminated tail refuses cleanup", current + b"{partial", checksum(current + b"{partial"), graduated),
    ]
    for label, content, digest, canonical in variants:
        try:
            reconcile(captured, content, accepted, digest, canonical, index_digest)
        except ValueError:
            checks.append(label)
        else:
            raise AssertionError(label)
        assert source.read_bytes() == current and index.read_bytes() == graduated
    git("switch", "-c", "manual-handback-test-only")
    assert git("rev-parse", "HEAD") == current_commit
    assert checksum(source.read_bytes()) == source_digest and checksum(index.read_bytes()) == index_digest
    source.write_bytes(proposal)
    git("add", "logbook.jsonl")
    git("commit", "-m", "Synthetic exact-line reconciliation; no production authorization")
    handback_commit = git("rev-parse", "HEAD")
    assert git("diff", "--name-only", current_commit, handback_commit) == "logbook.jsonl"
    assert index.read_bytes() == graduated and source.read_bytes() == proposal
    assert accepted not in source.read_bytes() and source.read_bytes().endswith(preserved)
    (evidence_dir / "reconciliation.diff").write_text(git("diff", current_commit, handback_commit) + "\n")
    code_commit = subprocess.check_output([args.git, "-C", str(assets.parents[3]), "rev-parse", "HEAD"], text=True).strip()
    report = {"scope": "offline TEST ONLY rehearsal; not production handback approval", "codeCommit": code_commit,
              "rehearsalScriptSha256": checksum(Path(__file__).read_bytes()),
              "completedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "acornSha256": checksum(args.acorn.resolve().read_bytes()), "capturedSource": provenance,
              "sourceCommits": {"original": original_commit, "current": current_commit, "reconciled": handback_commit},
              "canonicalCommit": canonical_commit, "sourceDigests": {"original": checksum(captured), "current": source_digest,
              "reconciled": checksum(proposal)}, "canonicalDigest": index_digest,
              "acceptedLineWithLfDigest": checksum(accepted), "acceptedEntry": candidate["entryIdentifier"],
              "checks": checks, "commands": commands, "remoteWrites": False, "botStarted": False,
              "productionMergeAuthorized": False, "botDatabaseSeeded": False}
    (evidence_dir / "handback-evidence.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Passed {len(checks)} handback checks; evidence: {evidence_dir / 'handback-evidence.json'}")


def reconcile(original, current, accepted, expected_source, index, expected_index):
    """Build a proposal from exact bytes; never write or interpret a receipt as approval."""
    if checksum(current) != expected_source or checksum(index) != expected_index:
        raise ValueError("Source or canonical index changed after review")
    if not original.endswith(b"\n") or not current.endswith(b"\n"):
        raise ValueError("Nonterminated source requires separate repair")
    original_lines = [line + b"\n" for line in original.split(b"\n")[:-1]]
    current_lines = [line + b"\n" for line in current.split(b"\n")[:-1]]
    if original_lines.count(accepted) != 1 or current_lines.count(accepted) != 1:
        raise ValueError("Accepted bytes missing, changed or duplicated")
    identity = json.loads(accepted)["submissionId"]

    def has_identity(line):
        try:
            value = json.loads(line)
            return isinstance(value, dict) and value.get("submissionId") == identity
        except (UnicodeDecodeError, ValueError):
            return False

    if sum(map(has_identity, original_lines)) != 1 or sum(map(has_identity, current_lines)) != 1:
        raise ValueError("Ambiguous or conflicting submission identity")
    return b"".join(line for line in current_lines if line != accepted)


if __name__ == "__main__":
    main()
