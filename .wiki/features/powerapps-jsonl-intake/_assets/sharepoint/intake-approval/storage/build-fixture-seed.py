"""Build an importable TEST ONLY flow that seeds 100 synthetic source rows per batch."""

import argparse
import base64
import json
import re
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5
from zipfile import ZIP_DEFLATED, ZipFile


ROOT = Path(__file__).resolve().parent
AGGREGATE = ROOT.parents[1] / "aggregate-tests" / "pagination-snapshots.clipboard.json"
FIXTURE_LIST = "746cf260-a4d3-4c8f-ab98-d78fe4a9afa1"
OLD_LIST = "fcae092f-c899-4254-bf13-1108997a0a77"
PREFIX = "36361108-4000-4000-8000-"
TAG = "pa36a-review-fixture-page"
FLOW_NAME = "Seed Review Source Fixtures TEST ONLY"


def build(template: Path, output: Path, start_at: int = 1) -> None:
    if start_at < 1:
        raise ValueError("start_at must be positive")
    package = json.loads(AGGREGATE.read_text())
    seed = package["operationDefinition"]["actions"]["Seed_5000"]
    seed = json.loads(json.dumps(seed).replace("Seed_5000", "Seed_100_batches"))
    seed["foreach"] = "@range(0,int(triggerBody()?['text']))"
    seed["runAfter"] = {}

    payload = {
        "entry": {
            "archived": False,
            "category": "software",
            "eventType": None,
            "global": False,
            "items": ["PA36A TEST ONLY pagination fixture."],
            "origin": "manual",
            "state": "in-progress",
            "summary": "PA36A TEST ONLY pagination fixture",
            "timestamp": "2026-11-08T23:00:00.000Z",
        },
        "projectId": "acorn",
        "submissionId": PREFIX + "000000000001",
        "submittedAt": "2026-11-08T23:00:00.000Z",
        "submitter": {"email": "review-fixture@example.org"},
    }
    encoded = base64.b64encode(json.dumps(payload, separators=(",", ":")).encode()).decode()
    raw = (
        "replace(base64ToString('"
        + encoded
        + "'),'"
        + PREFIX
        + "000000000001',concat('"
        + PREFIX
        + "',formatNumber(item(),'000000000000')))"
    )
    select = seed["actions"]["Generate_items"]
    select["inputs"]["from"] = f"@range(add(mul(items('Seed_100_batches'),100),{start_at}),100)"
    select["inputs"]["select"] = {
        "Title": "@concat('" + TAG + "-',string(item()))",
        "SubmissionId": "@concat('" + PREFIX + "',formatNumber(item(),'000000000000'))",
        "ProjectId": "acorn",
        "PayloadBase64A": "@base64(" + raw + ")",
        "PayloadBase64B": "",
        "ExportStatus": "Pending",
    }
    parts = seed["actions"]["Generate_parts"]["inputs"]["select"]
    match = re.search(r"base64ToString\('([^']+)'\)", parts)
    assert match is not None
    decoded = base64.b64decode(match.group(1)).decode()
    assert decoded.count(OLD_LIST) == 1
    encoded_parts = base64.b64encode(decoded.replace(OLD_LIST, FIXTURE_LIST).encode()).decode()
    seed["actions"]["Generate_parts"]["inputs"]["select"] = parts.replace(match.group(1), encoded_parts, 1)
    serialized = json.dumps(seed).replace(OLD_LIST, FIXTURE_LIST).replace("pa23-20261027-page", TAG)
    seed = json.loads(serialized)
    seed["actions"]["Verify_batch"]["else"]["actions"]["Seed_count_mismatch"]["inputs"] = "@int('PA36A seed count mismatch')"
    assert OLD_LIST not in serialized
    assert OLD_LIST not in base64.b64decode(encoded_parts).decode()
    assert FIXTURE_LIST in base64.b64decode(encoded_parts).decode()
    assert "review-fixture@example.org" in json.dumps(payload)
    assert seed["actions"]["Verify_batch"]["else"]["actions"]

    with ZipFile(template) as source:
        files = {name: source.read(name) for name in source.namelist()}
    definition_name = next(name for name in files if name.endswith("/definition.json"))
    flow = json.loads(files[definition_name])
    new_id = str(uuid5(NAMESPACE_URL, "PA36A/" + FLOW_NAME))
    flow["name"] = new_id
    flow["id"] = "/providers/Microsoft.Flow/flows/" + new_id
    flow["properties"]["displayName"] = FLOW_NAME
    definition = flow["properties"]["definition"]
    definition["triggers"]["manual"]["inputs"]["schema"]["properties"]["text"]["title"] = "BatchCount"
    definition["actions"] = {
        "Seed_100_batches": seed,
        "Seed_done": {
            "type": "Compose",
            "inputs": "@mul(int(triggerBody()?['text']),100)",
            "runAfter": {"Seed_100_batches": ["Succeeded"]},
        },
    }
    files[definition_name] = (json.dumps(flow, indent=2) + "\n").encode()

    manifest = json.loads(files["manifest.json"])
    manifest["details"]["displayName"] = FLOW_NAME
    root_resource = next(value for value in manifest["resources"].values() if value["type"] == "Microsoft.Flow/flows")
    root_resource["details"]["displayName"] = FLOW_NAME
    root_resource["suggestedCreationType"] = "New"
    root_resource["creationType"] = "New"
    files["manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()

    output.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(output, "w", ZIP_DEFLATED) as result:
        for name in sorted(files):
            result.writestr(name, files[name])


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("template", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--start-at", type=int, default=1)
    args = parser.parse_args()
    build(args.template, args.output, args.start_at)
