"""Validate the callback contract with jsonschema 4.26.0 (Draft 4)."""

import json
from pathlib import Path

from jsonschema import Draft4Validator


def cases(fixtures):
    """Exercise actual event fixtures and incompatible wire representations."""
    for fixture in fixtures:
        body = fixture["body"]
        event = body["event"]
        yield event, body, True
        yield f"{event}: unknown field", {**body, "unexpected": True}, False
        yield f"{event}: wrong project type", {**body, "project_id": 42}, False
        for field in body:
            yield (
                f"{event}: missing {field}",
                {key: value for key, value in body.items() if key != field},
                False,
            )
    yield "unknown event", {"event": "unknown", "project_id": "acorn"}, False
    yield "camelCase fields", {
        "event": "review-needed",
        "mergeRequestUrl": "https://example.invalid/review/1",
        "projectId": "acorn",
    }, False
    yield "null body", None, False
    yield "array body", [], False


def main():
    """Return nonzero if any accepted/rejected contract case disagrees."""
    root = Path(__file__).resolve().parent
    schema = json.loads((root / "callback-event.schema.json").read_text())
    fixtures = json.loads((root / "callback-event-fixtures.json").read_text())
    Draft4Validator.check_schema(schema)
    validator = Draft4Validator(schema)
    results = [
        {"case": name, "passed": validator.is_valid(body) == expected}
        for name, body, expected in cases(fixtures)
    ]
    print(json.dumps(results, indent=2))
    return 0 if all(result["passed"] for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
