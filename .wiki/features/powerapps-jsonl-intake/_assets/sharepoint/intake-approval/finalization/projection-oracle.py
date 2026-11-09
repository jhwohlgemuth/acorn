"""Local fixture oracle for PA-37A projection identity and correction mapping.

The cloud flow must match these outcomes before this phase can be accepted.
"""

from __future__ import annotations

import copy
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from uuid import UUID


@dataclass(frozen=True)
class ReviewRow:
    source_item_id: int
    source_json: str
    draft_json: str | None
    decision: str
    reason: str
    validation_state: str
    approved_submission_id: str | None = None


@dataclass(frozen=True)
class Candidate:
    source_item_id: int
    source_submission_id: str
    approved_submission_id: str
    payload_json: str
    approved_supersedes: str | None


class ProjectionError(ValueError):
    pass


def _uuid(value: str) -> str:
    try:
        return str(UUID(value))
    except (TypeError, ValueError) as error:
        raise ProjectionError(f"Invalid UUID: {value}") from error


def _object(value: str) -> dict:
    try:
        result = json.loads(value)
    except json.JSONDecodeError as error:
        raise ProjectionError("Invalid JSON payload") from error
    if not isinstance(result, dict):
        raise ProjectionError("Submission must be a JSON object")
    return result


def _entry(value: dict) -> None:
    entry = value.get("entry")
    if not isinstance(entry, dict):
        raise ProjectionError("Missing entry")
    summary = entry.get("summary")
    if not isinstance(summary, str) or not 1 <= len(summary.strip()) <= 1200:
        raise ProjectionError("Invalid summary")
    if not isinstance(entry.get("timestamp"), str) or not entry["timestamp"]:
        raise ProjectionError("Missing entry timestamp")
    if "health" in entry:
        health = entry["health"]
        if not isinstance(health, dict) or health.get("status") not in {"on-track", "ahead-of-plan", "at-risk", "delayed", "paused", "complete"}:
            raise ProjectionError("Invalid health")
        significance = entry.get("significance")
        if not isinstance(significance, str) or not 1 <= len(significance.strip()) <= 1000:
            raise ProjectionError("Invalid status significance")
        reason = health.get("reason")
        if reason is not None and (health["status"] not in {"at-risk", "delayed", "paused"} or not isinstance(reason, str) or not 1 <= len(reason.strip()) <= 1000):
            raise ProjectionError("Invalid health reason")
        evidence = entry.get("evidence")
        if evidence is not None and (not isinstance(evidence, str) or not 1 <= len(evidence.strip()) <= 1000):
            raise ProjectionError("Invalid evidence")
        risk = entry.get("risk")
        if risk is not None and (not isinstance(risk, dict) or not isinstance(risk.get("kind"), str) or not isinstance(risk.get("explanation"), str) or not 1 <= len(risk["explanation"].strip()) <= 1000):
            raise ProjectionError("Invalid risk")
        reporting_range = entry.get("range", value.get("range"))
        if not isinstance(reporting_range, dict) or not isinstance(reporting_range.get("left"), str) or not isinstance(reporting_range.get("right"), str) or reporting_range["left"] >= reporting_range["right"]:
            raise ProjectionError("Invalid status reporting range")
    else:
        if entry.get("category") not in {"administration", "collaboration", "data", "engagement", "funding", "infrastructure", "method", "other", "personnel", "publication", "research", "software"}:
            raise ProjectionError("Invalid activity category")
        if entry.get("state") not in {"completed", "in-progress", "upcoming"}:
            raise ProjectionError("Invalid activity state")
        if entry.get("eventType") not in {None, "conference", "exhibition", "meeting", "other", "presentation", "tour", "training", "workshop"}:
            raise ProjectionError("Invalid event type")
        items = entry.get("items")
        if items is not None and (not isinstance(items, list) or not 1 <= len(items) <= 20 or any(not isinstance(item, str) or not 1 <= len(item.strip()) <= 1000 for item in items)):
            raise ProjectionError("Invalid supporting items")
        if entry.get("classification") not in {None, "unclassified"} and (entry["category"] != "administration" or entry.get("eventType") != "other"):
            raise ProjectionError("Invalid classified activity")


def prepare(rows: list[ReviewRow], prior_approved: dict[str, str]) -> list[Candidate]:
    if not 1 <= len(rows) <= 5000:
        raise ProjectionError("Batch must contain 1 to 5000 items")
    ordered = sorted(rows, key=lambda row: row.source_item_id)
    if len({row.source_item_id for row in ordered}) != len(ordered):
        raise ProjectionError("Duplicate source item ID")
    source_ids = [_uuid(_object(row.source_json).get("submissionId")) for row in ordered]
    if len(set(source_ids)) != len(source_ids):
        raise ProjectionError("Duplicate source submission ID")
    mapping = {_uuid(source): _uuid(approved) for source, approved in prior_approved.items()}
    candidates = []
    used_approved = set(mapping.values())
    for row in ordered:
        source = _object(row.source_json)
        source_id = _uuid(source.get("submissionId"))
        if row.validation_state != "Valid" or row.decision not in {"Approve", "Exclude"}:
            raise ProjectionError(f"Unresolved or invalid item: {source_id}")
        if row.decision == "Exclude":
            if not row.reason.strip():
                raise ProjectionError(f"Excluded item lacks reason: {source_id}")
            continue
        draft_text = row.draft_json if row.draft_json is not None else row.source_json
        draft = _object(draft_text)
        _entry(draft)
        if draft.get("projectId") != source.get("projectId") or draft.get("submitter") != source.get("submitter") or draft.get("submittedAt") != source.get("submittedAt"):
            raise ProjectionError(f"Immutable identity changed: {source_id}")
        if draft.get("submissionId") != source_id or draft.get("revision") != source.get("revision"):
            raise ProjectionError(f"Immutable revision changed: {source_id}")
        projected = copy.deepcopy(draft)
        revision = projected.get("revision")
        approved_supersedes = None
        if revision is not None:
            if not isinstance(revision, dict):
                raise ProjectionError(f"Invalid revision: {source_id}")
            predecessor = _uuid(revision.get("supersedesSubmissionId"))
            root = _uuid(revision.get("rootSubmissionId"))
            if predecessor not in mapping or root not in mapping:
                raise ProjectionError(f"Missing or excluded predecessor: {source_id}")
            approved_supersedes = mapping[predecessor]
            revision["supersedesSubmissionId"] = approved_supersedes
            revision["rootSubmissionId"] = mapping[root]
        unchanged = draft_text == row.source_json and projected.get("revision") == source.get("revision")
        approved_id = source_id if unchanged else _uuid(row.approved_submission_id)
        if not unchanged and approved_id in source_ids:
            raise ProjectionError(f"Changed item reused a source UUID: {approved_id}")
        if approved_id in used_approved:
            raise ProjectionError(f"Duplicate approved submission ID: {approved_id}")
        used_approved.add(approved_id)
        projected["submissionId"] = approved_id
        payload_json = row.source_json if unchanged else json.dumps(projected, separators=(",", ":"), ensure_ascii=False)
        candidates.append(Candidate(row.source_item_id, source_id, approved_id, payload_json, approved_supersedes))
        mapping[source_id] = approved_id
    return candidates


def _self_test() -> None:
    root = {"submissionId": "a3500000-0000-4000-8000-000000000001", "projectId": "acorn", "submittedAt": "2026-11-08T22:30:00Z", "submitter": {"email": "review-fixture@example.org"}, "entry": {"summary": "Root", "category": "software", "state": "in-progress", "timestamp": "2026-11-08T22:30:00Z"}}
    root_text = json.dumps(root, separators=(",", ":"))
    unchanged = prepare([ReviewRow(1, root_text, None, "Approve", "", "Valid")], {})
    assert unchanged[0].approved_submission_id == root["submissionId"]
    assert unchanged[0].payload_json == root_text
    correction = copy.deepcopy(root)
    correction["submissionId"] = "a3500000-0000-4000-8000-000000000002"
    correction["revision"] = {"rootSubmissionId": root["submissionId"], "supersedesSubmissionId": root["submissionId"]}
    changed = copy.deepcopy(root)
    changed["entry"]["summary"] = "Edited root"
    rows = [
        ReviewRow(2, json.dumps(correction, separators=(",", ":")), None, "Approve", "", "Valid", "b3500000-0000-4000-8000-000000000002"),
        ReviewRow(1, root_text, json.dumps(changed, separators=(",", ":")), "Approve", "", "Valid", "b3500000-0000-4000-8000-000000000001"),
    ]
    candidates = prepare(rows, {})
    assert [item.source_item_id for item in candidates] == [1, 2]
    assert candidates[1].approved_supersedes == candidates[0].approved_submission_id
    assert json.loads(candidates[1].payload_json)["revision"]["rootSubmissionId"] == candidates[0].approved_submission_id
    assert prepare(rows, {}) == candidates
    prior = {root["submissionId"]: "b3500000-0000-4000-8000-000000000001"}
    prior_candidate = prepare([rows[0]], prior)
    assert prior_candidate[0].approved_supersedes == prior[root["submissionId"]]
    try:
        prepare([rows[0], ReviewRow(1, rows[1].source_json, None, "Exclude", "Duplicate", "Valid")], {})
    except ProjectionError as error:
        assert "predecessor" in str(error)
    else:
        raise AssertionError("Excluded root was accepted")
    for invalid in (
        [rows[0]],
        [ReviewRow(1, root_text, json.dumps(changed, separators=(",", ":")), "Approve", "", "Valid", root["submissionId"])],
        [ReviewRow(1, root_text, None, "Unresolved", "", "Valid")],
        [ReviewRow(1, root_text, None, "Exclude", "", "Valid")],
    ):
        try:
            prepare(invalid, {})
        except ProjectionError:
            pass
        else:
            raise AssertionError(f"Invalid projection was accepted: {invalid}")
    status = copy.deepcopy(root)
    status["entry"] = {"summary": "On track", "health": {"status": "on-track"}, "significance": "Milestone retained", "timestamp": "2026-11-08T22:30:00Z", "range": {"left": "2026-11-01T00:00:00Z", "right": "2026-12-01T00:00:00Z"}}
    assert prepare([ReviewRow(1, json.dumps(status), None, "Approve", "", "Valid")], {})[0].approved_submission_id == root["submissionId"]


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        _self_test()
    else:
        raise SystemExit(f"Usage: {Path(sys.argv[0]).name} --self-test")
