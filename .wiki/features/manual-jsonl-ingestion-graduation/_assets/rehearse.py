#!/usr/bin/env python3
"""Rehearse the captured, synthetic intake in a new local directory only."""
import argparse
import datetime
import difflib
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, required=True, help="New directory; existing paths are refused")
    parser.add_argument("--acorn", type=Path, required=True)
    parser.add_argument("--chrome", type=Path, required=True)
    parser.add_argument("--git", default="git")
    parser.add_argument("--approve-synthetic", action="store_true", help="Approve only this frozen TEST ONLY fixture")
    args = parser.parse_args()
    started = time.monotonic()
    source = Path(__file__).resolve().parent
    manifest = json.loads((source / "source-manifest.json").read_text())
    snapshot = (source / "inbox-snapshot.jsonl").read_bytes()
    selected = (source / "selected.jsonl").read_bytes()
    assert sha(snapshot) == "76f0e36f637b5611cca8aa6c69ce5db13be00a0b5e17b2dbfe71c20a38177384"
    assert sha(selected) == "c8496e27ce6379296d19c194305b2f17f028f2e3314230de9ce8adc7b28a2c58"
    assert snapshot.splitlines(keepends=True)[5] == selected
    envelope = json.loads(selected)
    assert envelope["submissionId"] == "4313fcfc-0201-441e-8bda-c77718a31dcf"
    root = args.workspace.resolve()
    root.mkdir(parents=True, exist_ok=False)
    index = root / "acorn/index.json"
    index.parent.mkdir()
    shutil.copyfile(source / "baseline.json", index)
    (root / "selected.jsonl").write_bytes(selected)
    config = root / "intake.json"
    config.write_text(json.dumps({"schemaVersion": "2.0.0", "folders": [{
        "id": "acorn", "members": ["o9w@ornl.gov"], "path": index.as_uri()
    }]}, indent=2) + "\n")
    transcript = []
    acorn = str(args.acorn.resolve())

    def run(label, *arguments, success=True, confirm=False, graduation=False):
        command = [acorn, "--offline", *map(str, arguments)]
        interactive = confirm and not args.approve_synthetic
        if interactive and graduation:
            command.append("--interactive")
        elif confirm and not interactive and not graduation:
            command.append("--yes")
        result = subprocess.run(command, capture_output=not interactive, text=True, timeout=None if interactive else 180)
        output = (result.stdout or "") + (result.stderr or "")
        (root / (label + ".txt")).write_text(output if not interactive else "Confirmation and receipt shown in the operator terminal.\n")
        transcript.append({"step": label, "command": command, "exitCode": result.returncode,
                           "confirmation": "terminal buttons" if interactive else "--yes" if "--yes" in command else "reviewed IDs and digest" if graduation else "none"})
        if interactive and result.returncode != 0:
            raise SystemExit("Stopped at reviewer checkpoint; inspect evidence in " + str(root))
        assert (result.returncode == 0) == success, result.stderr
        return result.stdout or ""

    baseline = index.read_bytes()
    stage = json.loads(run("01-stage", "intake", "stage", "--batch", "--config", config, "--source", root / "selected.jsonl"))
    assert stage["canApply"] and stage["accepted"] == 1
    inspected = run("02-inspect", "intake", "inspect", "--config", config, "--batch-id", stage["batchId"])
    assert index.read_bytes() == baseline
    print(inspected)
    print("Review the one-entry diff and its route to this disposable local index.", flush=True)
    run("03-apply", "intake", "apply", "--config", config, "--batch-id", stage["batchId"], confirm=True)
    ingested = index.read_bytes()
    (root / "ingested.json").write_bytes(ingested)
    run("04-apply-replay", "intake", "apply", "--config", config, "--batch-id", stage["batchId"], "--yes")
    assert index.read_bytes() == ingested
    preview = json.loads(run("05-candidates", "intake", "run", "--index", index, "--dry-run"))
    assert len(preview["candidates"]) == 1
    candidate = preview["candidates"][0]
    assert candidate["target"] == "contributors" and candidate["value"] == "Example, Avery (TEST ONLY)"
    assert index.read_bytes() == ingested
    print(json.dumps(preview, indent=2))
    print("Review the contributor candidate; the source entry will remain archived.", flush=True)
    graduate_args = ("intake", "run", "--index", index, "--accept", candidate["entryIdentifier"], "--expected-digest", preview["baseDigest"])
    run("06-graduate", *graduate_args, confirm=True, graduation=True)
    graduated = index.read_bytes()
    expected = json.loads(ingested)
    expected["meta"]["contributors"] = [{"name": "Example, Avery (TEST ONLY)", "identifiers": [], "affiliations": [], "roles": []}]
    expected["logbook"]["entries"][0]["archived"] = True
    assert json.loads(graduated) == expected
    backup = Path(str(index) + ".graduation-" + sha(ingested) + ".before")
    assert backup.read_bytes() == ingested
    (root / "graduation.diff").write_text("".join(difflib.unified_diff(ingested.decode().splitlines(True), graduated.decode().splitlines(True), fromfile="ingested.json", tofile="graduated.json")))
    run("07-graduate-replay", *graduate_args, success=False)
    assert index.read_bytes() == graduated
    rejected = {**envelope, "projectId": "unregistered-demo-project"}
    (root / "rejected.jsonl").write_text(json.dumps(rejected) + "\n")
    denied = json.loads(run("08-rejected-stage", "intake", "stage", "--batch", "--config", config, "--source", root / "rejected.jsonl", success=False))
    assert not denied["canApply"] and denied["rejected"] == 1
    run("09-rejected-apply", "intake", "apply", "--config", config, "--batch-id", denied["batchId"], "--yes", success=False)
    assert index.read_bytes() == graduated
    report_config = root / "report.json"
    report_config.write_text(json.dumps({"schemaVersion": "2.0.0", "folders": [{"id": "acorn", "path": "acorn/index.json"}],
                                       "options": {"folders": {"root": root.as_uri() + "/"}}}, indent=2) + "\n")
    git = shutil.which(args.git) or args.git
    git_env = {**os.environ, "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}
    for arguments in [("init", "--initial-branch=main"), ("add", "acorn/index.json", "report.json", "selected.jsonl"),
                      ("-c", "user.name=ACORN Synthetic Demo", "-c", "user.email=demo@example.org", "commit", "-m", "Reviewed TEST ONLY intake and graduation")]:
        subprocess.run([git, "-C", str(root), *arguments], check=True, capture_output=True, env=git_env)
    commit = subprocess.check_output([git, "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
    report_args = ("report", "--config", report_config, "--root", root, "--project", "acorn", "--commit", commit,
                   "--from", "2026-10-24T00:00:00Z", "--through", "2026-10-25T00:00:00Z",
                   "--chrome-path", args.chrome.resolve(), "--dry-run", "--combine")
    run("10-report", *report_args, "--output", root / "report")
    run("11-report-repeat", *report_args, "--output", root / "report-repeat")
    assert index.read_bytes() == graduated
    assert (root / "report/report.pdf").read_bytes().startswith(b"%PDF")
    evidence = {"mode": "local-only-recorded-source", "remotePublication": False,
                "capturedSource": manifest, "localCommit": commit, "entryIdentifier": candidate["entryIdentifier"],
                "approvedBy": "operator --approve-synthetic" if args.approve_synthetic else "interactive ACORN Yes/No confirmations",
                "configSha256": sha(config.read_bytes()), "baselineSha256": sha(baseline),
                "ingestedSha256": sha(ingested), "graduatedSha256": sha(graduated),
                "pdfSha256": sha((root / "report/report.pdf").read_bytes()),
                "window": {"from": "2026-10-24T00:00:00Z", "through": "2026-10-25T00:00:00Z"},
                "completedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                "elapsedSeconds": round(time.monotonic() - started, 2), "steps": transcript}
    (root / "evidence.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print("Verified 11 steps. Local PDF: " + str(root / "report/report.pdf"))
    print("Evidence: " + str(root / "evidence.json"))


if __name__ == "__main__":
    main()
