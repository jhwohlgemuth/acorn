"""Fault-injection execution of the generated publish graph; no network calls."""
import base64
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
from copy import deepcopy

ROOT = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("exporter", ROOT.parent / "build-logbook-export.py")
BUILDER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILDER)
EXPECTED = '{"submissionId":"pa26-synthetic","note":"café \\n Ω"}\n'.encode()
OLD = b'{"submissionId":"previous-synthetic"}\n'
ALL_STATUSES = ("Succeeded", "Failed", "TimedOut", "Skipped")


def publish_graph(definition):
    return definition["actions"]["Complete_valid_rows_LB"]["actions"]["Within_byte_limit_LB"]["actions"]


def approved_publish_graph(definition):
    return definition["actions"]["Within_candidate_bounds_PA38"]["actions"]


def digest(value):
    return hashlib.sha256(value).hexdigest() if value is not None else None


def simulate(graph, prior_status="Succeeded", code=200, initial=OLD,
             write_status="Succeeded", commit=True, read_status="Succeeded",
             read_override=None, prior_content=True, prior_length=None,
             read_content=True, read_length=None, expected=EXPECTED,
             race_file=None):
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
                    if race_file is not None:
                        state["file"] = race_file
                    if commit:
                        state["file"] = expected
                    status = write_status
                elif name == "Export_receipt_LB":
                    state["verified"] = True
                    status = "Succeeded"
                elif name in ("Read_published_metadata_PA38", "Exact_metadata_PA38",
                              "Publish_receipt_PA38", "Read_receipt_PA38",
                              "Exact_receipt_PA38"):
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
    return {**state, "file": None, "final_base64": content(state["file"]),
            "final_sha256": digest(state["file"]), "trace": trace}


def approved_fixture():
    root = ROOT.parent / "intake-approval" / "exporter"
    baseline = json.loads((root / "baseline.json").read_text())
    fixture = json.loads((root / "fixtures.json").read_text())
    batch = {key: value for key, value in fixture["batch"].items() if key != "manifestBase64"}
    batch["manifestBytes"] = base64.b64decode(fixture["batch"]["manifestBase64"])
    frozen = {(row["batchId"], row["sourceItemId"]): base64.b64decode(row["contentBase64"])
              for row in fixture["frozenFiles"]}
    return base64.b64decode(baseline["contentBase64"]), batch, frozen, fixture


def replace_manifest(batch, change):
    changed = deepcopy(batch)
    manifest = json.loads(changed["manifestBytes"])
    change(manifest)
    changed["manifestBytes"] = json.dumps(manifest, separators=(",", ":"), ensure_ascii=False).encode()
    changed["projectionDigest"] = BUILDER._fingerprint(changed["manifestBytes"])
    return changed


class ApprovedCandidateTests(unittest.TestCase):
    def setUp(self):
        self.baseline, self.batch, self.frozen, self.fixture = approved_fixture()

    def assert_failure(self, code, baseline=None, batch=None, frozen=None):
        with self.assertRaises(BUILDER.ApprovedExportError) as error:
            BUILDER.approved_candidate(self.baseline if baseline is None else baseline,
                                       [self.batch if batch is None else batch],
                                       self.frozen if frozen is None else frozen)
        self.assertEqual(error.exception.code, code)

    def test_approved_append_replay_and_exclusion(self):
        first = BUILDER.approved_candidate(self.baseline, [self.batch], self.frozen)
        second = BUILDER.approved_candidate(self.baseline, [self.batch], self.frozen)
        self.assertEqual(first, second)
        self.assertTrue(first.startswith(self.baseline))
        self.assertEqual(hashlib.sha256(first).hexdigest(), self.fixture["expectedCandidateSha256"])
        self.assertEqual(len(first), self.fixture["expectedCandidateByteCount"])
        self.assertEqual(len(first.splitlines()), self.fixture["expectedCandidateLineCount"])
        self.assertNotIn(b"44444444-4444-4444-8444-444444444444", first)
        for state in ("Draft", "Materializing", "Ready", "Finalizing", "Failed"):
            with self.subTest(state=state):
                pending = {**self.batch, "state": state}
                self.assertEqual(BUILDER.approved_candidate(self.baseline, [pending], self.frozen), self.baseline)

    def test_approved_batches_sort_and_extend(self):
        second_id = "55555555-5555-4555-8555-555555555555"
        second_batch_id = "66666666-6666-4666-8666-666666666666"
        second_raw = json.dumps({"submissionId": second_id, "entry": {"summary": "Second approved"}},
                                separators=(",", ":")).encode()
        second = replace_manifest(self.batch, lambda manifest: (
            manifest.update(batchId=second_batch_id),
            manifest["included"][0].update(sourceItemId=5, sourceSubmissionId=second_id,
                                            approvedSubmissionId=second_id,
                                            payloadBase64=base64.b64encode(second_raw).decode()),
            manifest["excluded"][0].update(sourceItemId=6),
            manifest.update(mapping={second_id: second_id})))
        second.update(batchId=second_batch_id, approvedAtUtc="2026-11-11T12:00:00Z")
        files = {**self.frozen, (second_batch_id, 5): second_raw}
        first = BUILDER.approved_candidate(self.baseline, [self.batch], files)
        extended = BUILDER.approved_candidate(self.baseline, [second, self.batch], files)
        self.assertEqual(extended, first + second_raw + b"\n")
        self.assertEqual(extended, BUILDER.approved_candidate(self.baseline, [self.batch, second], files))

    def test_bad_baseline_and_overflow(self):
        self.assert_failure("InvalidBaseline", baseline=self.baseline.rstrip(b"\n"))
        self.assert_failure("ExportBoundExceeded", baseline=b'{"submissionId":"x"}\n' * 5001)
        self.assert_failure("ExportBoundExceeded", baseline=b'{"submissionId":"x","pad":"' + b'x' * 10485761 + b'"}\n')

    def test_bad_manifest_and_count(self):
        self.assert_failure("InvalidManifest", batch={**self.batch, "manifestBytes": b"{"})
        self.assert_failure("InvalidManifest", batch={**self.batch, "approvedCount": 2})
        self.assert_failure("InvalidManifest", batch={**self.batch, "projectionDigest": "wrong"})
        with self.assertRaisesRegex(BUILDER.ApprovedExportError, "DuplicateBatchId"):
            BUILDER.approved_candidate(self.baseline, [self.batch, self.batch], self.frozen)

    def test_duplicate_baseline_submission(self):
        self.assert_failure("DuplicateSubmissionId", baseline=self.baseline + self.baseline.splitlines()[0] + b"\n")

    def test_bad_frozen_row_and_identity(self):
        self.assert_failure("FrozenBytesChanged", frozen={})
        invalid = replace_manifest(self.batch, lambda manifest: manifest["included"][0].update(payloadBase64="?"))
        self.assert_failure("InvalidFrozenBytes", batch=invalid)
        invalid = replace_manifest(self.batch, lambda manifest: manifest["included"][0].update(approvedSubmissionId="wrong"))
        self.assert_failure("DuplicateSubmissionId", batch=invalid)
        invalid = replace_manifest(self.batch, lambda manifest: manifest["included"][0].update(decision="Exclude"))
        self.assert_failure("InvalidDecision", batch=invalid)

    def test_bad_order_exclusion_and_ancestry(self):
        invalid = replace_manifest(self.batch, lambda manifest: manifest["excluded"][0].update(reason=""))
        self.assert_failure("InvalidExclusion", batch=invalid)
        invalid = replace_manifest(self.batch, lambda manifest: manifest["excluded"][0].update(sourceItemId=3))
        self.assert_failure("DuplicateSourceItemId", batch=invalid)
        malformed = b'{"submissionId":"11111111-1111-4111-8111-111111111111","revision":{"rootSubmissionId":"missing","supersedesSubmissionId":"missing"}}'
        invalid = replace_manifest(self.batch, lambda manifest: manifest["included"][0].update(payloadBase64=base64.b64encode(malformed).decode(), approvedSupersedes="missing"))
        self.assert_failure("InvalidAncestry", batch=invalid,
                            frozen={(self.batch["batchId"], 3): malformed})

    def test_bad_json_and_row_bound(self):
        malformed = b'{"submissionId":'
        invalid = replace_manifest(self.batch, lambda manifest: manifest["included"][0].update(payloadBase64=base64.b64encode(malformed).decode()))
        self.assert_failure("InvalidJSONLRow", batch=invalid,
                            frozen={(self.batch["batchId"], 3): malformed})
        oversized = b'{"submissionId":"11111111-1111-4111-8111-111111111111","pad":"' + b'x' * 65536 + b'"}'
        invalid = replace_manifest(self.batch, lambda manifest: manifest["included"][0].update(payloadBase64=base64.b64encode(oversized).decode()))
        self.assert_failure("InvalidJSONLRow", batch=invalid,
                            frozen={(self.batch["batchId"], 3): oversized})


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

    def test_competing_writer_preserves_racing_file(self):
        racing = b'{"submissionId":"racing-synthetic"}\n'
        result = simulate(self.graph, prior_status="Failed", code=404, initial=None,
                          write_status="Failed", commit=False, race_file=racing)
        self.assertEqual(result["writes"], 1)
        self.assertFalse(result["verified"])
        self.assertEqual(result["final_base64"], base64.b64encode(racing).decode())
        self.assertEqual(result["error"], "LogbookOutcomeUnresolved")

    def test_approved_publication_reuses_safe_graph(self):
        graph = approved_publish_graph(BUILDER.build_approved()["operationDefinition"])
        baseline, batch, files, _ = approved_fixture()
        candidate = BUILDER.approved_candidate(baseline, [batch], files)
        changed = simulate(graph, initial=baseline, expected=candidate)
        unchanged = simulate(graph, initial=candidate, expected=candidate)
        self.assertEqual(changed["writes"], 1)
        self.assertEqual(unchanged["writes"], 0)
        self.assertTrue(changed["verified"] and unchanged["verified"])
        for status, code in (("Failed", 403), ("Failed", 500), ("TimedOut", None)):
            rejected = simulate(graph, prior_status=status, code=code, initial=baseline,
                                expected=candidate)
            self.assertEqual(rejected["writes"], 0)
            self.assertFalse(rejected["verified"])
            self.assertEqual(rejected["final_sha256"], digest(baseline))

    def test_approved_native_package_has_top_level_initializers(self):
        path = ROOT.parent / "intake-approval" / "exporter" / "generated-definition.json"
        actions = json.loads(path.read_text())
        for name in ("Initialize_seen_ids_PA38", "Initialize_seen_batches_PA38",
                     "Initialize_source_ids_PA38", "Initialize_lines_PA38",
                     "Initialize_included_order_PA38"):
            self.assertEqual(actions[name]["type"], "InitializeVariable")
        scope = actions["Approved_logbook_export_PA38"]
        self.assertEqual(scope["runAfter"], {"Initialize_included_order_PA38": ["Succeeded"]})
        self.assertFalse(any(action["type"] == "InitializeVariable"
                             for action in scope["actions"].values()))
        batch = scope["actions"]["Each_approved_batch_PA38"]["actions"]
        self.assertIn("ItemCount", batch["Valid_manifest_PA38"]["inputs"])
        included = batch["Each_approved_row_PA38"]["actions"]
        self.assertIn("PreviousIncludedSourceIdPA38",
                      included["Valid_frozen_row_PA38"]["inputs"])
        receipt = (scope["actions"]["Within_candidate_bounds_PA38"]["actions"]
                   ["Readable_or_missing_LB"]["actions"]["Exact_logbook_LB"]["actions"])
        self.assertEqual(receipt["Export_receipt_LB"]["runAfter"],
                         {"Exact_metadata_PA38": ["Succeeded"]})
        self.assertIn("Read_published_metadata_PA38",
                      receipt["Export_receipt_LB"]["inputs"]["fileRevision"])
        self.assertEqual(receipt["Publish_receipt_PA38"]["runAfter"],
                         {"Export_receipt_LB": ["Succeeded"]})
        self.assertEqual(receipt["Exact_receipt_PA38"]["runAfter"],
                         {"Read_receipt_PA38": ["Succeeded"]})


if __name__ == "__main__":
    unittest.main()
