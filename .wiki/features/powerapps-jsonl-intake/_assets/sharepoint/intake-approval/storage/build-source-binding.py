"""Bind a native TEST ONLY materializer package to its trusted source list."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile


ROOT = Path(__file__).resolve().parent
SCHEMA = json.loads((ROOT / "schema.json").read_text())


def batch_actions(definition: dict) -> dict:
    actions = definition["properties"]["definition"]["actions"]
    scope = actions["Materialization_source_probe_PA36A"]["actions"]
    request = scope["Request_guard_mat_PA36A"]["actions"]
    return request["Batch_guard_PA36A"]


def bind(definition: dict, source_id: str) -> dict:
    guard = batch_actions(definition)
    condition = guard["expression"]["and"][0]["equals"]
    old_guard = condition[0]
    if not old_guard.startswith("@and("):
        raise ValueError("Unexpected materializer batch guard")
    ready = "equals(first(body('Find_batch_mat_PA36A')?['value'])?['State']?['Value'],'Ready')"
    if ready not in old_guard:
        materializing = "equals(first(body('Find_batch_mat_PA36A')?['value'])?['State']?['Value'],'Materializing')"
        if materializing not in old_guard:
            raise ValueError("Materializer batch guard lacks Materializing state")
        old_guard = old_guard.replace(materializing, materializing + "," + ready, 1)
    binding = (
        "or(equals(first(body('Find_batch_mat_PA36A')?['value'])?['SourceListId'],'"
        + source_id
        + "'),and(equals(first(body('Find_batch_mat_PA36A')?['value'])?['State']?['Value'],'Draft'),"
        + "empty(first(body('Find_batch_mat_PA36A')?['value'])?['SourceListId'])))"
    )
    if "SourceListId" not in old_guard:
        condition[0] = old_guard[:-1] + "," + binding + ")"
    elif source_id not in old_guard:
        raise ValueError("Batch guard already bound to another source")
    else:
        condition[0] = old_guard

    limit = guard["actions"]["Source_limit_PA36A"]["actions"]
    materialization = limit["Materialize_review_items_PA36A"]["actions"]
    valid = materialization["Source_valid_PA36A"]["actions"]
    commit = valid["Existing_subset_PA36A"]["actions"]
    updates = (
        commit["Mark_materializing_PA36A"],
        commit["Reconciled_PA36A"]["actions"]["Mark_ready_PA36A"],
    )
    for action in updates:
        parameters = action["inputs"]["parameters"]
        previous = parameters.get("item/SourceListId")
        if previous is not None and previous != source_id:
            raise ValueError("Batch update already bound to another source")
        parameters["item/SourceListId"] = source_id
    return definition


def build(template: Path, output: Path, source: str) -> Path:
    source_id = SCHEMA[source]["listId"]
    with ZipFile(template) as package:
        files = {name: package.read(name) for name in package.namelist()}
    definition_path = next(name for name in files if name.endswith("/definition.json"))
    flow = json.loads(files[definition_path])
    expected_name = (
        "Materialize Approval Batch Fixture TEST ONLY"
        if source == "sourceFixtures"
        else "Materialize Approval Batch TEST ONLY"
    )
    if flow["properties"]["displayName"] != expected_name:
        raise ValueError("Template is not the expected TEST ONLY materializer")
    files[definition_path] = (json.dumps(bind(flow, source_id), indent=2) + "\n").encode()
    output.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(output, "w", ZIP_DEFLATED) as package:
        for name in sorted(files):
            package.writestr(name, files[name])
    return output


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("template", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--source", choices=("source", "sourceFixtures"), required=True)
    arguments = parser.parse_args()
    print(build(arguments.template, arguments.output, arguments.source))
