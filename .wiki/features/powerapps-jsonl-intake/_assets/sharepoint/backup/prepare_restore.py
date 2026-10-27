"""Build the PA24 one-time, test-only data restore operation.

Operational lists are read-only inputs. Every write is confined to PA24 storage.
Native ZIPs remain unchanged; this helper is not a native backup package.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SITE = "https://ornl.sharepoint.com/sites/ResearchEnablement"
LIBRARY = "/PA24 TEST ONLY JSONL"
PROJECTS = "c482b248-39fb-45f8-833e-26baf05cc15c"
RECEIPTS = "13fa719a-32ae-45de-b444-95d3a33e8048"
SOURCE_PROJECTS = "88dfa78d-a62f-4e0f-aae5-0d5ea52758ed"
SOURCE_RECEIPTS = "a904a6c6-5dd0-427b-9575-8167c4c0c412"
SECURE = {"secureData": {"properties": ["inputs", "outputs"]}}


def after(name, statuses=None):
    return {name: statuses or ["Succeeded"]} if name else {}


def sp(operation, parameters, dependency=None):
    return {"type": "OpenApiConnection", "runAfter": after(dependency),
            "inputs": {"host": {"apiId": "/providers/Microsoft.PowerApps/apis/shared_sharepointonline",
                                  "connectionName": "shared_sharepointonline", "operationId": operation},
                       "parameters": {"dataset": SITE, **parameters},
                       "authentication": "@parameters('$authentication')",
                       "retryPolicy": {"type": "none"}}, "runtimeConfiguration": SECURE}


def compose(value, dependency=None):
    return {"type": "Compose", "inputs": value, "runAfter": after(dependency)}


def guard(expression, dependency):
    return {"type": "If", "expression": expression, "runAfter": after(dependency),
            "actions": {}, "else": {"actions": {f"{dependency}_guard_failed":
                compose("@int('PA24 restore validation failed')")}}}


def file_create(name, content, dependency=None):
    return sp("HttpRequest", {"parameters/method": "POST",
        "parameters/uri": "@concat('_api/web/GetFolderByServerRelativeUrl(', decodeUriComponent('%27'), "
                          "'/sites/ResearchEnablement/PA24 TEST ONLY JSONL', decodeUriComponent('%27'), "
                          "')/Files/add(url=', decodeUriComponent('%27'), " + name +
                          ", decodeUriComponent('%27'), ',overwrite=false)')",
        "parameters/headers": {"Accept": "application/json;odata=nometadata", "Content-Type": "application/octet-stream"},
        "parameters/body": content}, dependency)


def file_read(name, dependency):
    return sp("GetFileContentByPath", {"path": name, "inferContentType": False}, dependency)


def snapshot(prefix, table, dependency):
    latest = f"{prefix}_latest"
    rows = f"{prefix}_rows"
    payload = f"{prefix}_snapshot"
    write = f"{prefix}_backup_write"
    read = f"{prefix}_backup_read"
    return {
        latest: sp("GetItems", {"table": table, "$orderby": "ID desc", "$top": 1}, dependency),
        rows: sp("GetItems", {"table": table, "$orderby": "ID asc", "$top": 1001,
            "$filter": f"@concat('ID le ', string(coalesce(first(body('{latest}')?['value'])?['ID'], 0)))"}, latest),
        payload: compose({"capturedUtc": "@utcNow()", "sourceList": table,
            "cutoffId": f"@coalesce(first(body('{latest}')?['value'])?['ID'], 0)",
            "rows": f"@body('{rows}')?['value']"}, rows),
        write: file_create(f"'{prefix}-snapshot.json'", f"@string(outputs('{payload}'))", payload),
        read: file_read(f"{LIBRARY}/{prefix}-snapshot.json", write),
        f"{prefix}_backup_exact": guard({"equals": [f"@body('{read}')?['$content']", f"@base64(string(outputs('{payload}')))"]}, read),
        f"{prefix}_restored_input": compose(f"@json(base64ToString(body('{read}')?['$content']))", f"{prefix}_backup_exact"),
    }


def receipt_fields(row):
    fields = ["PayloadBase64A", "PayloadBase64B", "ProjectId", "SubmissionId", "SubmitterEmail", "Summary", "Title"]
    return {f"item/{field}": f"@{row}?['{field}']" for field in fields}


def build():
    actions = {
        "Check_empty_receipts": sp("GetItems", {"table": RECEIPTS, "$top": 1}),
        "Check_empty_projects": sp("GetItems", {"table": PROJECTS, "$top": 1}, "Check_empty_receipts"),
        "Require_empty_targets": guard({"and": [
            {"equals": ["@length(body('Check_empty_receipts')?['value'])", 0]},
            {"equals": ["@length(body('Check_empty_projects')?['value'])", 0]}]}, "Check_empty_projects"),
        **snapshot("Receipts", SOURCE_RECEIPTS, "Require_empty_targets"),
        **snapshot("Projects", SOURCE_PROJECTS, "Receipts_restored_input"),
    }
    actions["Restore_projects"] = {"type": "Foreach", "foreach": "@outputs('Projects_restored_input')?['rows']",
        "runAfter": after("Projects_restored_input"), "runtimeConfiguration": {"concurrency": {"repetitions": 1}},
        "actions": {"Restore_project": sp("PostItem", {"table": PROJECTS, **{
            f"item/{key}": f"@items('Restore_projects')?['{key}']" for key in ["ProjectId", "SourceRevision", "Title"]}})}}
    row = "items('Restore_receipts')"
    raw = "base64ToString(concat(coalesce(items('Restore_receipts')?['PayloadBase64A'], string(null)), coalesce(items('Restore_receipts')?['PayloadBase64B'], string(null))))"
    receipt_actions = {
        "Restore_receipt": sp("PostItem", {"table": RECEIPTS, **receipt_fields(row), "item/ExportStatus": "Pending"}),
        "Restore_receipt_file": file_create("concat(items('Restore_receipts')?['SubmissionId'], '.jsonl')", f"@concat({raw}, decodeUriComponent('%0A'))", "Restore_receipt"),
        "Read_restored_receipt_file": file_read("@concat('/PA24 TEST ONLY JSONL/', items('Restore_receipts')?['SubmissionId'], '.jsonl')", "Restore_receipt_file"),
        "Require_exact_receipt_file": guard({"equals": ["@body('Read_restored_receipt_file')?['$content']", f"@base64(concat({raw}, decodeUriComponent('%0A')))"]}, "Read_restored_receipt_file"),
        "Mark_restored_receipt_exported": sp("PatchItem", {"table": RECEIPTS,
            "id": "@body('Restore_receipt')?['ID']", **receipt_fields(row), "item/ExportStatus": "Exported",
            "item/ExportFileUrl": "@concat('https://ornl.sharepoint.com/sites/ResearchEnablement/PA24%20TEST%20ONLY%20JSONL/', items('Restore_receipts')?['SubmissionId'], '.jsonl')"}, "Require_exact_receipt_file"),
    }
    actions["Restore_receipts"] = {"type": "Foreach", "foreach": "@outputs('Receipts_restored_input')?['rows']",
        "runAfter": after("Restore_projects"), "runtimeConfiguration": {"concurrency": {"repetitions": 1}}, "actions": receipt_actions}
    actions["Read_restored_receipts"] = sp("GetItems", {"table": RECEIPTS, "$orderby": "ID asc", "$top": 1001}, "Restore_receipts")
    actions["Read_restored_projects"] = sp("GetItems", {"table": PROJECTS, "$orderby": "ID asc", "$top": 1001}, "Read_restored_receipts")
    actions["Restored_snapshot"] = compose({"receipts": "@body('Read_restored_receipts')?['value']",
        "projects": "@body('Read_restored_projects')?['value']"}, "Read_restored_projects")
    actions["Write_restored_snapshot"] = file_create("'Restored-snapshot.json'", "@string(outputs('Restored_snapshot'))", "Restored_snapshot")
    actions["Restore_counts"] = compose({"receiptCount": "@length(body('Read_restored_receipts')?['value'])",
        "projectCount": "@length(body('Read_restored_projects')?['value'])",
        "receiptCutoff": "@outputs('Receipts_restored_input')?['cutoffId']",
        "projectCutoff": "@outputs('Projects_restored_input')?['cutoffId']"}, "Write_restored_snapshot")
    return {"type": "Scope", "runAfter": {}, "actions": actions}


if __name__ == "__main__":
    clip = json.loads((ROOT / "restored-intake.clipboard.json").read_text())
    clip.update({"operationName": "PA24_restore_data", "operationDefinition": build()})
    text = json.dumps(clip, indent=2)
    (ROOT / "restore-data.clipboard.json").write_text(text + "\n")
