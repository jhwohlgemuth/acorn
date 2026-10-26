"""Build native designer clipboard actions for the isolated SharePoint pilot."""
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

ROOT = Path(__file__).resolve().parent
SITE = "https://ornl.sharepoint.com/sites/ResearchEnablement"
LIST = "a904a6c6-5dd0-427b-9575-8167c4c0c412"
ALL = ["Succeeded", "Failed", "Skipped", "TimedOut"]
SECURE = {"secureData": {"properties": ["inputs", "outputs"]}}
SECURE_INPUTS = {"secureData": {"properties": ["inputs"]}}


def after(name, statuses=None):
    return {name: statuses or ["Succeeded"]}


def compose(value, dependency=None):
    return {"type": "Compose", "inputs": value,
            "runAfter": after(dependency) if dependency else {},
            "runtimeConfiguration": SECURE_INPUTS}


def condition(expression, actions, otherwise=None, dependency=None):
    return {"type": "If", "expression": expression, "actions": actions,
            "else": {"actions": otherwise or {}},
            "runAfter": after(dependency) if dependency else {}}


def sharepoint(operation, parameters, dependency=None, statuses=None):
    return {"type": "OpenApiConnection", "inputs": {
        "host": {"connectionName": "shared_sharepointonline", "operationId": operation,
                 "apiId": "/providers/Microsoft.PowerApps/apis/shared_sharepointonline"},
        "parameters": {"dataset": SITE, **parameters},
        "authentication": "@parameters('$authentication')",
        "retryPolicy": {"type": "none"}},
        "runAfter": after(dependency, statuses) if dependency else {},
        "runtimeConfiguration": SECURE}


def clipboard(name, definition):
    # The classic designer drops empty string literals while parsing clipboard
    # expressions. string(null) retains the same Workflow Definition Language value.
    definition = json.loads(json.dumps(definition).replace("''", "string(null)"))
    references = {"shared_sharepointonline": {
        "connectionName": "shared-sharepointonl-4aa12f84-9554-41c4-a184-b21799bd4104",
        "source": "Embedded",
        "id": "/providers/Microsoft.PowerApps/apis/shared_sharepointonline",
        "tier": "NotSpecified", "apiName": "sharepointonline",
        "isProcessSimpleApiReferenceConversionAlreadyDone": True}}
    return {"id": str(uuid5(NAMESPACE_URL, f"{SITE}/{name}")),
            "brandColor": "#8C6CFF", "connectionReferences": references,
            "connectorDisplayName": "Control", "isTrigger": False,
            "operationName": name, "operationDefinition": definition}


source = json.loads((ROOT.parent / "solution-export/pa17-native-persistence-v2-verified.json").read_text())
source = json.loads(json.dumps(source).replace("_PA17v2", "_SP"))["actions"]
validation_names = ["Raw_submission_SP", "Parse_submission_SP", "Check_timestamps_SP",
                    "Blank_details_SP", "Caller_email_SP"]
validation = {name: {k: v for k, v in source[name].items() if k != "metadata"}
              for name in validation_names}
for value in validation.values():
    value.setdefault("runAfter", {})

query = {"table": LIST, "$filter": "@concat('SubmissionId eq ',decodeUriComponent('%27'),body('Parse_submission_SP')?['submissionId'],decodeUriComponent('%27'))", "$top": 2}
payload = "base64(outputs('Raw_submission_SP'))"
item = {"Title": "@body('Parse_submission_SP')?['submissionId']",
        "SubmissionId": "@body('Parse_submission_SP')?['submissionId']",
        "PayloadBase64A": f"@take({payload},45000)",
        "PayloadBase64B": f"@skip({payload},45000)",
        "ProjectId": "@body('Parse_submission_SP')?['projectId']",
        "Summary": "@body('Parse_submission_SP')?['entry']?['summary']",
        "SubmitterEmail": "@outputs('Caller_email_SP')", "ExportStatus": "Pending"}
receipt = "first(body('Read_receipt_SP')?['value'])"
file_url = "@concat('https://ornl.sharepoint.com/sites/ResearchEnablement/Sponsor%20Highlights%20JSONL%20Pilot/',body('Parse_submission_SP')?['submissionId'],'.jsonl')"
export_actions = {
    "Accepted_receipt_SP": compose(f"@{receipt}?['ID']"),
    "Create_JSONL_SP": sharepoint("HttpRequest", {
        "parameters/method": "POST",
        "parameters/uri": "@concat('_api/web/GetFolderByServerRelativeUrl(',decodeUriComponent('%27'),'/sites/ResearchEnablement/Sponsor Highlights JSONL Pilot',decodeUriComponent('%27'),')/Files/add(url=',decodeUriComponent('%27'),body('Parse_submission_SP')?['submissionId'],'.jsonl',decodeUriComponent('%27'),',overwrite=false)')",
        "parameters/headers": {"Accept": "application/json;odata=nometadata", "Content-Type": "application/octet-stream"},
        "parameters/body": "@concat(outputs('Raw_submission_SP'),decodeUriComponent('%0A'))"
    }, "Accepted_receipt_SP"),
    "Read_JSONL_SP": sharepoint("GetFileContentByPath", {
        "path": "@concat('/Sponsor Highlights JSONL Pilot/',body('Parse_submission_SP')?['submissionId'],'.jsonl')",
        "inferContentType": False
    }, "Create_JSONL_SP", ALL),
    "Exact_export_SP": condition({"equals": [
        "@body('Read_JSONL_SP')?['$content']",
        "@base64(concat(outputs('Raw_submission_SP'),decodeUriComponent('%0A')))"
    ]}, {
        "Mark_exported_SP": sharepoint("PatchItem", {
            "table": LIST, "id": "@outputs('Accepted_receipt_SP')",
            **{f"item/{key}": value for key, value in item.items()},
            "item/ExportStatus": "Exported", "item/ExportFileUrl": file_url
        })
    }, dependency="Read_JSONL_SP")
}
persistence = {
    "Find_receipt_SP": sharepoint("GetItems", query),
    "Create_if_missing_SP": condition({"equals": ["@length(body('Find_receipt_SP')?['value'])", 0]}, {
        "Create_receipt_SP": sharepoint("PostItem", {"table": LIST, **{f"item/{key}": value for key, value in item.items()}})
    }, dependency="Find_receipt_SP"),
    "Read_receipt_SP": sharepoint("GetItems", query, "Create_if_missing_SP", ALL),
    "One_receipt_SP": condition({"equals": ["@length(body('Read_receipt_SP')?['value'])", 1]}, {
        "Exact_receipt_SP": condition({"equals": [
            f"@concat({receipt}?['PayloadBase64A'],coalesce({receipt}?['PayloadBase64B'],''))",
            f"@{payload}"
        ]}, export_actions, {"Conflict_SP": compose("conflict")})
    }, dependency="Read_receipt_SP")
}
validation["Matching_projects_SP"] = {
    "type": "Query", "inputs": {"from": "@body('Get_items')?['value']",
    "where": "@equals(item()?['ProjectId'],body('Parse_submission_SP')?['projectId'])"},
    "runAfter": after("Caller_email_SP"), "runtimeConfiguration": SECURE}
guard = source["Envelope_allowed_SP"]["expression"]
guard["and"].extend([{"equals": ["@length(body('Matching_projects_SP'))", 1]},
                      {"lessOrEquals": ["@length(body('Get_items')?['value'])", 1000]}])
validation["Envelope_allowed_SP"] = condition(guard, persistence, {"Rejected_SP": compose("rejected")}, "Matching_projects_SP")
scope = {"type": "Scope", "actions": validation, "runAfter": after("Get_items")}
response_fields = {
    "status": "@if(equals(actions('Accepted_receipt_SP')?['status'],'Succeeded'),'received',if(equals(actions('Conflict_SP')?['status'],'Succeeded'),'conflict',if(or(equals(actions('Rejected_SP')?['status'],'Succeeded'),equals(actions('Parse_submission_SP')?['status'],'Failed'),equals(actions('Check_timestamps_SP')?['status'],'Failed')),'rejected','unresolved')))",
    "submissionId": "@if(equals(actions('Parse_submission_SP')?['status'],'Succeeded'),body('Parse_submission_SP')?['submissionId'],'')",
    "exportStatus": "@if(equals(actions('Mark_exported_SP')?['status'],'Succeeded'),'exported','pending')",
    "exportFileUrl": f"@if(equals(actions('Mark_exported_SP')?['status'],'Succeeded'),{file_url[1:]},'')",
    "receiptUrl": "@if(equals(actions('Accepted_receipt_SP')?['status'],'Succeeded'),concat('https://ornl.sharepoint.com/sites/ResearchEnablement/Lists/Sponsor%20Highlights%20Intake%20Pilot/DispForm.aspx?ID=',string(outputs('Accepted_receipt_SP'))),'')"
}
response = {"type": "Response", "kind": "PowerApp", "inputs": {"statusCode": 200,
    "schema": {"type": "object", "properties": {
        key.lower(): {"type": "string", "title": key, "x-ms-dynamically-added": True}
        for key in response_fields}, "additionalProperties": {}},
    "body": {key.lower(): "@{" + value[1:] + "}" for key, value in response_fields.items()}},
    "runAfter": after("SharePoint_intake_SP", ALL)}
for name, definition in [("SharePoint_intake_SP", scope), ("SharePoint_response_SP", response)]:
    (ROOT / f"{name}.clipboard.json").write_text(json.dumps(clipboard(name, definition), indent=2) + "\n")
