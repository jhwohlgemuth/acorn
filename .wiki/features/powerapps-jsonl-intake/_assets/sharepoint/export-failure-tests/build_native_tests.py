"""Build an isolated native fault harness from the repaired publish graph.

Only connector boundary observations are injected. Conditions, run-after routing,
and real SharePoint file writes/readbacks execute in Power Automate. Rejection
Terminate actions become markers so later assertions can inspect the file.
"""
import base64
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
from uuid import uuid4

ROOT = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("builder", ROOT.parent / "build-logbook-export.py")
B = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(B)
LIBRARY = "PA19 TEST ONLY JSONL"
OLD = '{"submissionId":"pa26-prior","text":"previous synthetic snapshot"}\n'
NEW = '{"submissionId":"pa26-next","text":"café Ω and escaped \\n newline"}\n'
LATER = '{"submissionId":"pa26-later","text":"later synthetic snapshot"}\n'
CASES = [
    ("denied_403", "Failed", 403, "none", "normal", OLD, False),
    ("server_500", "Failed", 500, "none", "normal", OLD, False),
    ("server_503", "Failed", 503, "none", "normal", OLD, False),
    ("read_timeout", "TimedOut", None, "none", "normal", OLD, False),
    ("timeout_with_404", "TimedOut", 404, "none", "normal", OLD, False),
    ("lost_committed", "Succeeded", 200, "lost_commit", "normal", NEW, True),
    ("lost_uncommitted", "Succeeded", 200, "lost_no_commit", "normal", OLD, False),
    ("readback_failed", "Succeeded", 200, "write", "failed", NEW, False),
    ("readback_mismatch", "Succeeded", 200, "write", "mismatch", LATER, False),
    ("unchanged", "Succeeded", 200, "none", "normal", NEW, True),
]


def compose(inputs, after=None, statuses=None):
    return {"type": "Compose", "inputs": inputs,
            "runAfter": {after: statuses or ["Succeeded"]} if after else {}}


def rewrite(value, replacements):
    if isinstance(value, dict):
        return {replacements.get(k, k): rewrite(v, replacements) for k, v in value.items()}
    if isinstance(value, list):
        return [rewrite(v, replacements) for v in value]
    if isinstance(value, str):
        for old, new in replacements.items():
            value = value.replace(old, new)
    return value


def names(actions):
    return [name for name, action in actions.items()] + [
        child for action in actions.values()
        for branch in (action.get("actions", {}), action.get("else", {}).get("actions", {}))
        for child in names(branch)]


def fixture_write(filename, text, dependency=None):
    return B.action("HttpRequest", {
        "parameters/method": "POST",
        "parameters/uri": "_api/web/GetFolderByServerRelativeUrl('/sites/ResearchEnablement/"
                          + LIBRARY + "')/Files/add(url='" + filename + "',overwrite=true)",
        "parameters/headers": {"Content-Type": "application/octet-stream"},
        "parameters/body": "@base64ToString('" + base64.b64encode(text.encode()).decode() + "')"}, dependency)


def scenario(case, suffix):
    label, prior_status, code, write_mode, read_mode, final, verified = case
    filename = "pa26-" + suffix + "-" + label + ".jsonl"
    definition = B.build("fcae092f-c899-4254-bf13-1108997a0a77", LIBRARY)["operationDefinition"]
    publish = deepcopy(definition["actions"]["Complete_valid_rows_LB"]["actions"]
                       ["Within_byte_limit_LB"]["actions"])
    observed = {"status": prior_status, "statusCode": code,
                "$content": base64.b64encode((NEW if label == "unchanged" else OLD).encode()).decode()
                if prior_status == "Succeeded" else None}
    publish["Read_previous_logbook_LB"] = compose(observed)
    gate = publish["Readable_or_missing_LB"]
    # Native Compose emits the injected connector observation. This changes no guard operators.
    gate["expression"] = rewrite(gate["expression"], {
        "@actions('Read_previous_logbook_LB')?['status']":
        "@outputs('Read_previous_logbook_LB')?['status']"})
    gate["else"]["actions"]["LogbookPreviousReadUnavailable"] = compose("rejected-before-write")
    guarded = gate["actions"]
    changed = guarded["Changed_logbook_LB"]
    write = changed["actions"]["Write_logbook_LB"]
    write["inputs"]["parameters"]["parameters/uri"] = write["inputs"]["parameters"]["parameters/uri"].replace(
        "actions('Read_previous_logbook_LB')", "outputs('Read_previous_logbook_LB')")
    if write_mode == "lost_commit":
        changed["actions"]["Write_logbook_LB"] = {
            "type": "Scope", "runAfter": {}, "actions": {
                "Commit_before_lost_response": write,
                "Lose_write_response": compose("@div(1,0)", "Commit_before_lost_response")}}
    elif write_mode == "lost_no_commit":
        changed["actions"]["Write_logbook_LB"] = compose("@div(1,0)")
    if read_mode == "failed":
        guarded["Read_published_logbook_LB"] = compose("@div(1,0)", "Changed_logbook_LB",
                                                      ["Succeeded", "Failed", "TimedOut"])
    elif read_mode == "mismatch":
        # A later file wins; the exporter must reject without rolling it back.
        guarded["Inject_later_snapshot"] = fixture_write(filename, LATER, "Changed_logbook_LB")
        guarded["Read_published_logbook_LB"]["runAfter"] = {"Inject_later_snapshot": ["Succeeded"]}
    exact = guarded["Exact_logbook_LB"]
    exact["else"]["actions"]["LogbookOutcomeUnresolved"] = compose("unresolved-after-write")
    exact["actions"]["Export_receipt_LB"] = compose("verified-exact-bytes")
    setup = {
        "Seed_prior_snapshot": fixture_write(filename, NEW if label == "unchanged" else OLD),
        "Combined_JSONL_LB": compose("@base64ToString('" + base64.b64encode(NEW.encode()).decode() + "')", "Seed_prior_snapshot"),
        "Publish_under_test": {"type": "Scope", "actions": publish,
                              "runAfter": {"Combined_JSONL_LB": ["Succeeded"]}},
        "Independent_final_read": B.action("GetFileContentByPath", {
            "path": "/" + LIBRARY + "/" + filename, "inferContentType": False}, "Publish_under_test"),
    }
    setup["Observe_outcome"] = compose({
        "actualBase64": "@body('Independent_final_read')?['$content']",
        "expectedBase64": base64.b64encode(final.encode()).decode(),
        "writeStatus": "@actions('Write_logbook_LB')?['status']",
        "receiptStatus": "@actions('Export_receipt_LB')?['status']",
    }, "Independent_final_read")
    expected_write_status = "Skipped" if write_mode == "none" else (
        "Failed" if write_mode.startswith("lost_") else "Succeeded")
    setup["Assert_expected_outcome"] = B.guard({"and": [
        {"equals": ["@body('Independent_final_read')?['$content']", base64.b64encode(final.encode()).decode()]},
        {"equals": ["@actions('Write_logbook_LB')?['status']", expected_write_status]},
        {"equals": ["@actions('Export_receipt_LB')?['status']", "Succeeded" if verified else "Skipped"]},
    ]}, {"Case_passed": compose({"case": label, "writeStatus": expected_write_status,
                                "verified": verified, "file": filename})},
        "NativeAssertionFailed", label + " failed expected file/write/receipt assertions.",
        "Observe_outcome")
    replacements = {
        "logbook.jsonl": filename,
        "body('Read_previous_logbook_LB')": "outputs('Read_previous_logbook_LB')",
    }
    replacements.update({name: name + "_" + label for name in names(setup)})
    # Longest first avoids replacing a shorter action name inside another name.
    return rewrite(setup, dict(sorted(replacements.items(), key=lambda pair: -len(pair[0]))))


def main():
    fixture_path = ROOT / "native-fixtures.json"
    suffix = json.loads(fixture_path.read_text())["suffix"] if fixture_path.exists() else uuid4().hex[:10]
    actions = {}
    previous = None
    for case in CASES:
        name = "Case_" + case[0]
        actions[name] = {"type": "Scope", "actions": scenario(case, suffix),
                         "runAfter": {previous: ["Succeeded"]} if previous else {}}
        previous = name
    clipboard = B.build()
    def visible(value):
        if isinstance(value, dict):
            return {key: visible(item) for key, item in value.items() if key != "runtimeConfiguration"}
        if isinstance(value, list):
            return [visible(item) for item in value]
        return value
    actions = visible(actions)
    clipboard["operationName"] = "PA26_native_fault_matrix"
    clipboard["id"] = str(uuid4())
    clipboard["operationDefinition"] = {"type": "Scope", "actions": actions, "runAfter": {}}
    (ROOT / "native-fault-matrix.clipboard.json").write_text(json.dumps(clipboard, indent=2) + "\n")
    (ROOT / "native-fixtures.json").write_text(json.dumps({
        "suffix": suffix, "old": OLD, "expected": NEW, "later": LATER,
        "cases": [case[0] for case in CASES],
        "injection_limits": "Prior HTTP codes/timeouts are injected observations. Lost responses and readback failure use deliberate native action failure, not a real network timeout.",
    }, indent=2) + "\n")


if __name__ == "__main__":
    main()
