"""Fault-injection execution of the generated publish graph; no network calls."""
import base64
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("exporter", ROOT.parent / "build-logbook-export.py")
BUILDER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILDER)
EXPECTED = '{"submissionId":"pa26-synthetic","note":"café \\n Ω"}\n'.encode()
OLD = b'{"submissionId":"previous-synthetic"}\n'
ALL_STATUSES = ("Succeeded", "Failed", "TimedOut", "Skipped")


def publish_graph(definition):
    return definition["actions"]["Complete_valid_rows_LB"]["actions"]["Within_byte_limit_LB"]["actions"]


def digest(value):
    return hashlib.sha256(value).hexdigest() if value is not None else None


def simulate(graph, prior_status="Succeeded", code=200, initial=OLD,
             write_status="Succeeded", commit=True, read_status="Succeeded",
             read_override=None, prior_content=True, prior_length=None,
             read_content=True, read_length=None, expected=EXPECTED):
    outcomes = {}
    trace = []
    state = {"file": initial, "writes": 0, "verified": False, "error": None}

    def content(value):
        return base64.b64encode(value).decode() if value is not None else None

    def normalized(status, code, payload, present, length):
        if status == "Succeeded" and code == 200 and str(length) == "0" and not present:
            return ""
        return content(payload) if status == "Succeeded" and present else None

    def value(expression):
        if not isinstance(expression, str) or not expression.startswith("@"):
            return expression
        values = {
            "@actions('Read_previous_logbook_LB')?['status']": prior_status,
            "@outputs('Read_previous_logbook_LB')?['statusCode']": code,
            "@body('Read_previous_logbook_LB')?['$content']":
                content(initial) if prior_status == "Succeeded" and prior_content else None,
            "@actions('Read_published_logbook_LB')?['status']": read_status,
            "@body('Read_published_logbook_LB')?['$content']":
                content(read_override if read_override is not None else state["file"])
                if read_status == "Succeeded" else None,
            "@base64(outputs('Combined_JSONL_LB'))": content(expected),
            BUILDER.file_content('Read_previous_logbook_LB'):
                normalized(prior_status, code, initial, prior_content, prior_length),
            BUILDER.file_content('Read_published_logbook_LB'):
                normalized(read_status, 200, read_override if read_override is not None else state["file"], read_content, read_length),
        }
        return values[expression]

    def condition(expression):
        operator, operands = next(iter(expression.items()))
        if operator == "and":
            return all(condition(item) for item in operands)
        if operator == "or":
            return any(condition(item) for item in operands)
        if operator == "not":
            return not condition(operands)
        if operator == "equals":
            return value(operands[0]) == value(operands[1])
        raise AssertionError(f"Unsupported expression: {expression}")

    def execute(actions):
        pending = dict(actions)
        local = {}
        while pending and state["error"] is None:
            ready = [name for name, action in pending.items()
                     if all(dep in outcomes for dep in action.get("runAfter", {}))]
            if not ready:
                raise AssertionError(f"Unresolved dependencies: {pending.keys()}")
            for name in ready:
                action = pending.pop(name)
                if not all(outcomes[dep] in statuses
                           for dep, statuses in action.get("runAfter", {}).items()):
                    status = "Skipped"
                elif action["type"] == "If":
                    branch = action["actions"] if condition(action["expression"]) else action["else"]["actions"]
                    status = execute(branch)
                elif action["type"] == "Terminate":
                    state["error"] = action["inputs"]["runError"]["code"]
                    status = "Failed"
                elif name == "Read_previous_logbook_LB":
                    status = prior_status
                elif name == "Read_published_logbook_LB":
                    status = read_status
                elif name == "Write_logbook_LB":
                    state["writes"] += 1
                    uri = action["inputs"]["parameters"]["parameters/uri"]
                    # A missing-file creation must never overwrite a file racing into existence.
                    assert "overwrite=@{if(" in uri
                    if commit:
                        state["file"] = expected
                    status = write_status
                elif name == "Export_receipt_LB":
                    state["verified"] = True
                    status = "Succeeded"
                else:
                    raise AssertionError(f"Unexpected action: {name}")
                outcomes[name] = status
                local[name] = status
                trace.append({"action": name, "status": status})
                if state["error"]:
                    break
        # Failed predecessors handled by downstream actions do not poison the scope.
        dependencies = {dep for action in actions.values() for dep in action.get("runAfter", {})}
        leaves = [status for name, status in local.items() if name not in dependencies]
        return "Failed" if any(status in ("Failed", "TimedOut") for status in leaves) else "Succeeded"

    execute(graph)
    return {**state, "file": None, "final_sha256": digest(state["file"]), "trace": trace}


class ExporterTests(unittest.TestCase):
    def setUp(self):
        self.graph = publish_graph(BUILDER.build()["operationDefinition"])

    def test_changed_and_unchanged(self):
        for initial, writes in [(OLD, 1), (EXPECTED, 0), (b"", 1)]:
            with self.subTest(initial=initial):
                result = simulate(self.graph, initial=initial)
                self.assertEqual(result["writes"], writes)
                self.assertTrue(result["verified"])

    def test_confirmed_missing_creation(self):
        result = simulate(self.graph, prior_status="Failed", code=404, initial=None)
        self.assertEqual(result["writes"], 1)
        self.assertTrue(result["verified"])

    def test_empty_success_body_omitted(self):
        created = simulate(self.graph, expected=b"", read_content=False, read_length="0")
        self.assertEqual(created["writes"], 1)
        self.assertTrue(created["verified"])
        repeated = simulate(self.graph, expected=b"", initial=b"", prior_content=False,
                            prior_length=0, read_content=False, read_length="0")
        self.assertEqual(repeated["writes"], 0)
        self.assertTrue(repeated["verified"])
        for status, code, length in [("Failed", 200, "0"), ("TimedOut", 200, "0"),
                                     ("Succeeded", 204, "0"), ("Succeeded", 200, None),
                                     ("Succeeded", 200, "1")]:
            result = simulate(self.graph, expected=b"", initial=b"", prior_content=False,
                              prior_status=status, code=code, prior_length=length)
            self.assertEqual(result["writes"], 0)
            self.assertFalse(result["verified"])

    def test_failed_read_never_writes(self):
        for status, code in [("Failed", 403), ("Failed", 500), ("Failed", 503),
                             ("Failed", None), ("TimedOut", None), ("TimedOut", 404),
                             ("Skipped", None), ("Succeeded", 200)]:
            with self.subTest(status=status, code=code):
                result = simulate(self.graph, prior_status=status, code=code, prior_content=False)
                self.assertEqual(result["writes"], 0)
                self.assertFalse(result["verified"])
                self.assertEqual(result["final_sha256"], digest(OLD))
                self.assertEqual(result["error"], "LogbookPreviousReadUnavailable")

    def test_lost_write_response_reconciles_both_outcomes(self):
        for status in ("Failed", "TimedOut"):
            for commit in (False, True):
                with self.subTest(status=status, commit=commit):
                    result = simulate(self.graph, write_status=status, commit=commit)
                    self.assertEqual(result["writes"], 1)
                    self.assertEqual(result["verified"], commit)
                    self.assertEqual(result["final_sha256"], digest(EXPECTED if commit else OLD))
                    self.assertEqual(result["error"], None if commit else "LogbookOutcomeUnresolved")

    def test_readback_failure_never_claims_preservation(self):
        for status in ("Failed", "TimedOut", "Skipped"):
            with self.subTest(status=status):
                result = simulate(self.graph, read_status=status)
                self.assertEqual(result["writes"], 1)
                self.assertFalse(result["verified"])
                self.assertEqual(result["final_sha256"], digest(EXPECTED))
                self.assertEqual(result["error"], "LogbookOutcomeUnresolved")

    def test_readback_mismatch_and_later_snapshot_no_rollback(self):
        result = simulate(self.graph, read_override=b'{"later":"snapshot"}\n')
        self.assertEqual(result["writes"], 1)
        self.assertFalse(result["verified"])
        self.assertEqual(result["error"], "LogbookOutcomeUnresolved")

    def test_retries_disabled_and_creation_non_overwrite(self):
        publish = self.graph["Readable_or_missing_LB"]["actions"]
        write = publish["Changed_logbook_LB"]["actions"]["Write_logbook_LB"]
        self.assertEqual(write["inputs"]["retryPolicy"], {"type": "none"})
        self.assertIn("'Succeeded'),'true','false')", write["inputs"]["parameters"]["parameters/uri"])
        self.assertEqual(publish["Read_published_logbook_LB"]["runAfter"],
                         {"Changed_logbook_LB": ["Succeeded", "Failed", "TimedOut"]})


if __name__ == "__main__":
    unittest.main()
