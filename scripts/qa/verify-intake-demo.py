#!/usr/bin/env python3
"""Exercise the built CLI against synthetic, disposable intake and report data."""
import argparse
import copy
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys


PROJECTS = ("watershed-sensing", "materials-discovery", "grid-resilience")
REPO = Path(__file__).resolve().parents[2]


def check(condition, message):
    if not condition:
        raise RuntimeError(message)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_rows(path, rows):
    path.write_text("".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows), encoding="utf-8")


def originals():
    return [{
        "submissionId": f"10000000-0000-4000-8000-{number:012d}",
        "projectId": project,
        "submittedAt": "2026-10-26T12:00:00Z",
        "submitter": {"email": "reviewer@example.org"},
        "entry": {
            "timestamp": "2026-10-26T11:00:00Z", "category": "software", "state": "completed",
            "summary": f"TEST ONLY {project}: original highlight",
            "items": ["Synthetic calibration completed.", 'Quoted "value", Unicode café, and newline\nretained.'],
        },
    } for number, project in enumerate(PROJECTS, 1)]


def correction(original):
    revised = copy.deepcopy(original)
    revised.update({
        "submissionId": "20000000-0000-4000-8000-000000000001",
        "submittedAt": "2026-10-26T13:00:00Z",
        "revision": {"rootSubmissionId": original["submissionId"], "supersedesSubmissionId": original["submissionId"]},
    })
    revised["entry"]["summary"] = "TEST ONLY corrected watershed highlight"
    return revised


def workspace(root):
    root.mkdir(parents=True, exist_ok=False)
    for project in PROJECTS:
        shutil.copytree(REPO / "tests/fixtures/report/baseline" / project, root / project)
    write_json(root / "intake.json", {"schemaVersion": "2.0.0", "folders": [
        {"id": project, "members": ["reviewer@example.org", "other@example.org"],
         "path": (root / project / "index.json").as_uri()} for project in PROJECTS
    ]})
    write_json(root / "report.json", {"schemaVersion": "2.0.0", "folders": [
        {"id": project, "path": f"{project}/index.json"} for project in PROJECTS
    ], "options": {"folders": {"root": "file:."}}})
    return root


def indexes(root):
    return {project: (root / project / "index.json").read_bytes() for project in PROJECTS}


def verify(args, root, evidence):
    binary = str(args.acorn.resolve())
    environment = {key: value for key, value in os.environ.items() if not key.startswith("ACORN_")}
    environment.update({"NO_COLOR": "1", "TERM": "dumb"})
    version = subprocess.run([binary, "--version"], check=True, capture_output=True, text=True, encoding="utf-8").stdout.strip()
    evidence.update({"binarySha256": digest(Path(binary).read_bytes()), "version": version})

    def run(folder, label, *arguments, success=True):
        command = [binary, "--offline", "--no-local-database", *map(str, arguments)]
        result = subprocess.run(command, cwd=folder, env=environment, capture_output=True, text=True, encoding="utf-8", timeout=240)
        (root / f"{len(evidence['commands']):03d}-{label}.txt").write_text(result.stdout + result.stderr, encoding="utf-8")
        evidence["commands"].append({"cwd": str(folder.relative_to(root)), "arguments": command[1:], "exitCode": result.returncode})
        check((result.returncode == 0) == success, f"{label}: unexpected exit {result.returncode}\n{result.stderr}")
        return result.stdout

    def stage(folder, rows, name="downloaded.jsonl", success=True):
        source = folder / name
        write_rows(source, rows)
        before = source.read_bytes()
        result = json.loads(run(folder, "stage", "intake", "stage", "--batch", "--config", "intake.json", "--source", name, success=success))
        check(source.read_bytes() == before, "Staging rewrote downloaded receipts")
        return result

    def apply(folder, batch, success=True, approve=True):
        return run(folder, "apply", "intake", "apply", "--config", "intake.json", "--batch-id", batch["batchId"],
                   *(["--yes"] if approve else []), success=success)

    def passed(case):
        evidence["cases"][case] = "PASS"
        print(f"[PASS] {case}", flush=True)

    rows = originals()
    revised = correction(rows[0])
    demo = workspace(root / "demo")
    before = indexes(demo)
    evidence["baselineSha256"] = {project: digest(content) for project, content in before.items()}
    batch = stage(demo, rows)
    check(batch["canApply"] and batch["accepted"] == 3, "Expected three accepted project receipts")
    inspection = run(demo, "inspect", "intake", "inspect", "--config", "intake.json", "--batch-id", batch["batchId"])
    check(all(project in inspection for project in PROJECTS), "Inspection omitted a route")
    check(indexes(demo) == before, "Stage/inspect modified an index")
    passed("INTAKE-01")
    apply(demo, batch, success=False, approve=False)
    check(indexes(demo) == before, "Redirected confirmation modified an index")
    apply(demo, batch)
    for project, row in zip(PROJECTS, rows):
        entries = json.loads(indexes(demo)[project])["logbook"]["entries"]
        check(len(entries) == 2 and entries[-1]["summary"] == row["entry"]["summary"], "Incorrect ingestion")
        check(entries[-1]["items"] == row["entry"]["items"], "Escaped receipt text changed")
    passed("INTAKE-02")
    ingested = indexes(demo)
    apply(demo, batch)
    check(indexes(demo) == ingested, "Repeated apply changed index bytes")
    replay = stage(demo, rows, "replay.jsonl")
    check(replay["replayed"] == 3 and replay["accepted"] == 0, "Repeated source was not recognized")
    apply(demo, replay)
    check(indexes(demo) == ingested, "Replayed source changed index bytes")
    passed("INTAKE-03")
    revision = stage(demo, [revised], "correction.jsonl")
    inspection = run(demo, "inspect-correction", "intake", "inspect", "--config", "intake.json", "--batch-id", revision["batchId"])
    check("Correction of " + rows[0]["submissionId"] in inspection, "Inspection omitted ancestry")
    apply(demo, revision)
    corrected = indexes(demo)
    entries = json.loads(corrected[PROJECTS[0]])["logbook"]["entries"]
    check(len(entries) == 2 and entries[-1]["summary"] == revised["entry"]["summary"], "Correction duplicated or lost entry")
    check(entries[-1]["identifier"] == json.loads(ingested[PROJECTS[0]])["logbook"]["entries"][-1]["identifier"], "Logical identity changed")
    history = stage(demo, rows + [revised], "collected.jsonl")
    check(history["replayed"] == 4 and history["accepted"] == 0, "Full history did not replay")
    apply(demo, history)
    check(indexes(demo) == corrected, "History replay modified corrected indexes")
    fresh = workspace(root / "same-batch")
    apply(fresh, stage(fresh, rows + [revised]))
    check(indexes(fresh) == corrected, "Same-batch and later correction results differ")
    passed("INTAKE-04")
    for case in ("missing", "owner", "route", "placement", "fork", "changed-uuid", "malformed"):
        isolated = workspace(root / f"reject-{case}")
        invalid = copy.deepcopy(revised)
        if case == "owner":
            invalid["submitter"]["email"] = "other@example.org"
        elif case == "route":
            invalid["projectId"] = PROJECTS[1]
        elif case == "placement":
            invalid["entry"]["global"] = True
        elif case == "changed-uuid":
            invalid = copy.deepcopy(rows[0])
            invalid["entry"]["summary"] = "Changed bytes using the same receipt UUID"
        elif case == "malformed":
            invalid = {}
        source = ([] if case == "missing" else rows) + [invalid]
        if case == "fork":
            fork = copy.deepcopy(revised)
            fork["submissionId"] = "30000000-0000-4000-8000-000000000001"
            source.append(fork)
        before = indexes(isolated)
        rejected = stage(isolated, source, success=False)
        check(not rejected["canApply"] and rejected["rejected"] > 0, f"Accepted invalid {case}")
        apply(isolated, rejected, success=False)
        check(indexes(isolated) == before, f"Rejected {case} batch partially wrote indexes")
    passed("INTAKE-05")
    for case in ("source", "index", "membership"):
        isolated = workspace(root / f"stale-{case}")
        staged = stage(isolated, rows)
        if case == "source":
            with (isolated / "downloaded.jsonl").open("a", encoding="utf-8") as stream:
                stream.write("{}\n")
        elif case == "index":
            with (isolated / PROJECTS[0] / "index.json").open("a", encoding="utf-8") as stream:
                stream.write("\n")
        else:
            config = json.loads((isolated / "intake.json").read_text(encoding="utf-8"))
            config["folders"][0]["members"] = ["other@example.org"]
            write_json(isolated / "intake.json", config)
        before = indexes(isolated)
        apply(isolated, staged, success=False)
        check(indexes(isolated) == before, f"Stale {case} caused a write")
    passed("INTAKE-06")
    general = copy.deepcopy(rows[0])
    general.update({"submissionId": "40000000-0000-4000-8000-000000000001", "projectId": "general"})
    general["entry"]["global"] = True
    isolated = workspace(root / "general-gap")
    before = indexes(isolated)
    rejected = stage(isolated, rows + [general], success=False)
    check(not rejected["canApply"] and rejected["rejected"] == 1, "Standalone general must remain an explicit unsupported route")
    apply(isolated, rejected, success=False)
    check(indexes(isolated) == before, "General row was silently dropped/remapped")
    passed("INTAKE-07")
    graduate = workspace(root / "graduation")
    personnel = copy.deepcopy(rows[0])
    personnel["entry"].update({"category": "personnel", "summary": "Example, Avery (TEST ONLY)", "items": ["Synthetic contributor for graduation acceptance."]})
    apply(graduate, stage(graduate, [personnel]))
    index = graduate / PROJECTS[0] / "index.json"
    before = index.read_bytes()
    preview = json.loads(run(graduate, "graduate-preview", "intake", "run", "--index", index, "--dry-run", "--force"))
    check(index.read_bytes() == before and len(preview["candidates"]) == 1, "Graduation preview wrote or omitted the candidate")
    run(graduate, "graduate", "intake", "run", "--index", index, "--force", "--expected-digest", preview["baseDigest"])
    graduated = index.read_bytes()
    activity = json.loads(graduated)
    check(any(person["name"] == "Example, Avery (TEST ONLY)" for person in activity["meta"]["contributors"]), "Contributor not graduated")
    check(activity["logbook"]["entries"][-1]["archived"], "Graduation did not retain archived source")
    check(Path(str(index) + ".graduation-" + digest(before) + ".before").read_bytes() == before, "Graduation backup differs")
    run(graduate, "graduate-stale", "intake", "run", "--index", index, "--force", "--expected-digest", preview["baseDigest"], success=False)
    check(index.read_bytes() == graduated, "Stale graduation changed index")
    passed("INTAKE-08")
    if args.skip_pdf:
        evidence["cases"]["INTAKE-09"] = "SKIP: PDF generation and visual review required for demo acceptance"
    else:
        run(demo, "report", "report", "--config", "report.json", "--from", "2026-10-26", "--through", "2026-10-27",
            "--dry-run", "--combine", *(["--chrome-path", args.chrome.resolve()] if args.chrome else []))
        pdf = demo / ".acorn/reports/report.pdf"
        check(pdf.read_bytes().startswith(b"%PDF"), "Combined PDF missing")
        check(indexes(demo) == corrected, "Report changed project indexes")
        evidence["pdf"] = {"path": str(pdf.relative_to(root)), "sha256": digest(pdf.read_bytes()), "visualReview": "PENDING"}
        passed("INTAKE-09")
    check((demo / "downloaded.jsonl").read_text(encoding="utf-8").splitlines() == [json.dumps(row, ensure_ascii=False) for row in rows], "Original receipts changed")
    confirmation = workspace(root / "confirmation")
    personnel["submissionId"] = "50000000-0000-4000-8000-000000000001"
    evidence["confirmationBatch"] = stage(confirmation, rows + [personnel])["batchId"]
    evidence["cases"].update({"INTAKE-10": "MANUAL: live contributor/SharePoint/Teams smoke test", "INTAKE-11": "MANUAL: interactive confirmation buttons"})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--acorn", type=Path, required=True, help="Exact built executable to test")
    parser.add_argument("--chrome", type=Path, help="Installed Chrome/Chromium; never downloaded in offline mode")
    parser.add_argument("--skip-pdf", action="store_true", help="Run intake baseline only; does not accept the demo")
    parser.add_argument("--workspace", type=Path, help="New directory; existing paths are refused")
    args = parser.parse_args()
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d-%H%M%S-%f")
    root = (args.workspace or REPO / "target/qa-acceptance" / f"intake-{stamp}").resolve()
    root.mkdir(parents=True, exist_ok=False)
    evidence = {"cases": {}, "commands": [], "completed": False, "platform": platform.platform(),
                "runnerSha256": digest(Path(__file__).read_bytes()), "startedAt": stamp}
    try:
        verify(args, root, evidence)
        evidence["completed"] = True
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError) as error:
        evidence["failure"] = str(error)
        print(f"[FAIL] {error}", file=sys.stderr)
    finally:
        write_json(root / "evidence.json", evidence)
        print(f"Evidence and disposable demo: {root}")
    return 0 if evidence["completed"] else 1


if __name__ == "__main__":
    sys.exit(main())
