"""Build importable TEST ONLY reviewer read flows from a Power Apps V2 template."""

import argparse
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5
from zipfile import ZIP_DEFLATED, ZipFile


SITE = "https://ornl.sharepoint.com/sites/ResearchEnablement"
BATCHES = "2f405310-4681-40db-99ff-1bafd4b90676"
ITEMS = "47c8f476-1a5e-4658-9380-39bec3671a24"
AUTH = {
    "value": "@json(decodeBase64(triggerOutputs().headers['X-MS-APIM-Tokens']))['$ConnectionKey']",
    "type": "Raw",
}


def compose(expression, after=None):
    return {"type": "Compose", "inputs": expression, "runAfter": after or {}}


def sharepoint(operation, parameters, after):
    return {
        "type": "OpenApiConnection",
        "inputs": {
            "host": {
                "apiId": "/providers/Microsoft.PowerApps/apis/shared_sharepointonline",
                "connectionName": "shared_sharepointonline",
                "operationId": operation,
            },
            "parameters": {"dataset": SITE, **parameters},
            "authentication": AUTH,
            "retryPolicy": {"type": "none"},
        },
        "runAfter": after,
        "runtimeConfiguration": {"secureData": {"properties": ["inputs", "outputs"]}},
    }


def get_items(table, filter_expression, order, top, after):
    return sharepoint(
        "GetItems",
        {
            "table": table,
            "$filter": filter_expression,
            "$orderby": order,
            "$top": top,
        },
        after,
    )


def select(source, fields, after):
    return {
        "type": "Select",
        "inputs": {"from": source, "select": fields},
        "runAfter": after,
    }


def response(fields, after):
    return {
        "type": "Response",
        "kind": "PowerApp",
        "inputs": {
            "schema": {
                "type": "object",
                "properties": {
                    name: {
                        "title": name,
                        "type": "string" if kind == "TEXT" else "boolean",
                        "x-ms-content-hint": kind,
                        "x-ms-dynamically-added": True,
                    }
                    for name, (kind, _) in fields.items()
                },
                "additionalProperties": {},
            },
            "statusCode": 200,
            "body": {name: value for name, (_, value) in fields.items()},
        },
        "runAfter": after,
        "runtimeConfiguration": {"secureData": {"properties": ["inputs"]}},
    }


def inputs(*names):
    return {
        "type": "object",
        "properties": {
            f"text{('_' + str(index)) if index else ''}": {
                "title": name,
                "type": "string",
                "x-ms-content-hint": "TEXT",
                "x-ms-dynamically-added": True,
            }
            for index, name in enumerate(names)
        },
        "required": [f"text{('_' + str(index)) if index else ''}" for index in range(len(names))],
    }


def authorized_actions():
    return {
        "Caller_email_PA36B": compose(
            "@toLower(if(not(empty(triggerOutputs()?['headers']?['x-ms-user-email-encoded'])),base64ToString(triggerOutputs()?['headers']?['x-ms-user-email-encoded']),coalesce(triggerOutputs()?['headers']?['x-ms-user-email'],string(null))))"
        ),
        "Read_reviewers_PA36B": sharepoint(
            "HttpRequest",
            {
                "parameters/method": "GET",
                "parameters/uri": "_api/web/sitegroups/getbyid(42)/users",
                "parameters/headers": {"Accept": "application/json;odata=nometadata"},
            },
            {"Caller_email_PA36B": ["Succeeded"]},
        ),
        "Caller_membership_PA36B": {
            "type": "Query",
            "inputs": {
                "from": "@body('Read_reviewers_PA36B')?['value']",
                "where": "@equals(toLower(item()?['LoginName']),concat('i:0#.f|membership|',outputs('Caller_email_PA36B')))",
            },
            "runAfter": {"Read_reviewers_PA36B": ["Succeeded"]},
        },
    }


def gate(actions):
    return {
        "type": "If",
        "expression": {
            "equals": [
                "@and(not(empty(outputs('Caller_email_PA36B'))),equals(length(body('Caller_membership_PA36B')),1))",
                True,
            ]
        },
        "actions": actions,
        "else": {
            "actions": {
                "Forbidden_PA36B": {
                    "type": "Terminate",
                    "inputs": {
                        "runStatus": "Failed",
                        "runError": {"code": "Forbidden", "message": "Reviewer access is required."},
                    },
                    "runAfter": {},
                }
            }
        },
        "runAfter": {"Caller_membership_PA36B": ["Succeeded"]},
    }


def batch_guard(after, actions):
    return {
        "type": "If",
        "expression": {
            "and": [
                {"equals": ["@length(body('Find_batch_PA36B')?['value'])", 1]},
                {"equals": ["@first(body('Find_batch_PA36B')?['value'])?['State']?['Value']", "Ready"]},
            ]
        },
        "actions": actions,
        "else": {
            "actions": {
                "Batch_not_ready_PA36B": {
                    "type": "Terminate",
                    "inputs": {
                        "runStatus": "Failed",
                        "runError": {"code": "BatchNotReady", "message": "Batch is unavailable for review."},
                    },
                    "runAfter": {},
                }
            }
        },
        "runAfter": after,
    }


def find_batch(after):
    return get_items(
        BATCHES,
        "@concat('BatchId eq ',decodeUriComponent('%27'),outputs('Batch_id_PA36B'),decodeUriComponent('%27'))",
        "ID asc",
        2,
        after,
    )


def valid_uuid():
    return "@replace(toLower(trim(triggerBody()?['text'])),decodeUriComponent('%27'),concat(decodeUriComponent('%27'),decodeUriComponent('%27')))"


def list_batches():
    rows = "body('Read_batches_PA36B')?['value']"
    fifty = f"take({rows},50)"
    actions = {
        "Cursor_PA36B": compose("@if(empty(triggerBody()?['text']),'',formatDateTime(triggerBody()?['text'],'yyyy-MM-ddTHH:mm:ssZ'))"),
        "Read_batches_PA36B": get_items(
            BATCHES,
            "@if(empty(outputs('Cursor_PA36B')),concat('State eq ',decodeUriComponent('%27'),'Ready',decodeUriComponent('%27')),concat('State eq ',decodeUriComponent('%27'),'Ready',decodeUriComponent('%27'),' and PeriodStartUtc lt ',decodeUriComponent('%27'),outputs('Cursor_PA36B'),decodeUriComponent('%27')))",
            "PeriodStartUtc desc,BatchId desc",
            51,
            {"Cursor_PA36B": ["Succeeded"]},
        ),
        "Select_batches_PA36B": select(
            f"@{fifty}",
            {
                "batchId": "@item()?['BatchId']",
                "periodStartUtc": "@item()?['PeriodStartUtc']",
                "periodEndUtc": "@item()?['PeriodEndUtc']",
                "state": "@item()?['State']?['Value']",
                "sourceCount": "@item()?['SourceCount']",
                "itemCount": "@item()?['ItemCount']",
                "unresolvedCount": "@item()?['UnresolvedCount']",
                "approvedCount": "@item()?['ApprovedCount']",
                "excludedCount": "@item()?['ExcludedCount']",
            },
            {"Read_batches_PA36B": ["Succeeded"]},
        ),
        "Respond_batches_PA36B": response(
            {
                "batchesjson": ("TEXT", "@string(body('Select_batches_PA36B'))"),
                "nextperiod": ("TEXT", f"@if(greater(length({rows}),50),last({fifty})?['PeriodStartUtc'],'')"),
                "hasmore": ("TEXT", f"@string(greater(length({rows}),50))"),
            },
            {"Select_batches_PA36B": ["Succeeded"]},
        ),
    }
    return inputs("BeforePeriodStartUtc"), actions


def read_page():
    rows = "body('Read_page_PA36B')?['value']"
    fifty = f"take({rows},50)"
    filter_expression = (
        "@concat('BatchId eq ',decodeUriComponent('%27'),outputs('Batch_id_PA36B'),"
        "decodeUriComponent('%27'),if(empty(outputs('Cursor_PA36B')),'',"
        "concat(' and ReportSortKey gt ',decodeUriComponent('%27'),outputs('Cursor_PA36B'),"
        "decodeUriComponent('%27'))),if(equals(toLower(triggerBody()?['text_2']),'true'),"
        "concat(' and (Decision eq ',decodeUriComponent('%27'),'Unresolved',decodeUriComponent('%27'),' or ValidationState eq ',decodeUriComponent('%27'),'Invalid',decodeUriComponent('%27'),')'),''))"
    )
    read_actions = {
        "Cursor_PA36B": compose("@replace(trim(triggerBody()?['text_1']),decodeUriComponent('%27'),concat(decodeUriComponent('%27'),decodeUriComponent('%27')))"),
        "Read_page_PA36B": get_items(
            ITEMS, filter_expression, "ReportSortKey asc", 51, {"Cursor_PA36B": ["Succeeded"]}
        ),
        "Select_page_PA36B": select(
            f"@{fifty}",
            {
                "sourceItemId": "@item()?['SourceItemId']",
                "sourceSubmissionId": "@item()?['SourceSubmissionId']",
                "projectId": "@item()?['ProjectId']",
                "reportSortKey": "@item()?['ReportSortKey']",
                "sourceCreatedUtc": "@item()?['SourceCreatedUtc']",
                "sourceSummary": "@json(base64ToString(concat(coalesce(item()?['SourcePayloadBase64A'],''),coalesce(item()?['SourcePayloadBase64B'],''))))?['entry']?['summary']",
                "decision": "@item()?['Decision']?['Value']",
                "validationState": "@item()?['ValidationState']?['Value']",
                "draftVersion": "@item()?['DraftVersion']",
                "hasDraft": "@not(empty(item()?['DraftProjectionBase64A']))",
            },
            {"Read_page_PA36B": ["Succeeded"]},
        ),
        "Respond_page_PA36B": response(
            {
                "batchid": ("TEXT", "@outputs('Batch_id_PA36B')"),
                "itemsjson": ("TEXT", "@string(body('Select_page_PA36B'))"),
                "nextcursor": ("TEXT", f"@if(greater(length({rows}),50),last({fifty})?['ReportSortKey'],'')"),
                "hasmore": ("TEXT", f"@string(greater(length({rows}),50))"),
            },
            {"Select_page_PA36B": ["Succeeded"]},
        ),
    }
    actions = {
        "Batch_id_PA36B": compose(valid_uuid()),
        "Find_batch_PA36B": find_batch({"Batch_id_PA36B": ["Succeeded"]}),
        "Ready_batch_PA36B": batch_guard({"Find_batch_PA36B": ["Succeeded"]}, read_actions),
    }
    return inputs("BatchId", "AfterSortKey", "ProblemsOnly"), actions


def read_item():
    rows = "body('Read_item_PA36B')?['value']"
    read_actions = {
        "Read_item_PA36B": get_items(
            ITEMS,
            "@concat('BatchId eq ',decodeUriComponent('%27'),outputs('Batch_id_PA36B'),decodeUriComponent('%27'),' and SourceItemId eq ',string(outputs('Source_item_id_PA36B')))",
            "ID asc",
            2,
            {},
        ),
        "Unique_item_PA36B": {
            "type": "If",
            "expression": {"equals": [f"@length({rows})", 1]},
            "actions": {
                "Select_item_PA36B": select(
                    f"@{rows}",
                    {
                        "batchId": "@item()?['BatchId']",
                        "sourceItemId": "@item()?['SourceItemId']",
                        "sourceSubmissionId": "@item()?['SourceSubmissionId']",
                        "projectId": "@item()?['ProjectId']",
                        "reportSortKey": "@item()?['ReportSortKey']",
                        "sourcePayloadJson": "@base64ToString(concat(coalesce(item()?['SourcePayloadBase64A'],''),coalesce(item()?['SourcePayloadBase64B'],'')))",
                        "sourceETag": "@item()?['SourceETag']",
                        "sourceRevisionRoot": "@item()?['SourceRevisionRoot']",
                        "sourceSupersedes": "@item()?['SourceSupersedes']",
                        "draftProjectionJson": "@if(empty(item()?['DraftProjectionBase64A']),'',base64ToString(concat(item()?['DraftProjectionBase64A'],coalesce(item()?['DraftProjectionBase64B'],''))))",
                        "draftVersion": "@item()?['DraftVersion']",
                        "decision": "@item()?['Decision']?['Value']",
                        "decisionReason": "@item()?['DecisionReason']",
                        "validationState": "@item()?['ValidationState']?['Value']",
                        "validationErrors": "@item()?['ValidationErrors']",
                        "itemETag": "@item()?['@odata.etag']",
                    },
                    {},
                ),
                "Respond_item_PA36B": response(
                    {"itemjson": ("TEXT", "@string(first(body('Select_item_PA36B')))" )},
                    {"Select_item_PA36B": ["Succeeded"]},
                ),
            },
            "else": {
                "actions": {
                    "Item_not_found_PA36B": {
                        "type": "Terminate",
                        "inputs": {
                            "runStatus": "Failed",
                            "runError": {"code": "ItemNotFound", "message": "Review item is unavailable."},
                        },
                        "runAfter": {},
                    }
                }
            },
            "runAfter": {"Read_item_PA36B": ["Succeeded"]},
        },
    }
    actions = {
        "Batch_id_PA36B": compose(valid_uuid()),
        "Source_item_id_PA36B": compose("@int(triggerBody()?['text_1'])", {"Batch_id_PA36B": ["Succeeded"]}),
        "Find_batch_PA36B": find_batch({"Source_item_id_PA36B": ["Succeeded"]}),
        "Ready_batch_PA36B": batch_guard({"Find_batch_PA36B": ["Succeeded"]}, read_actions),
    }
    return inputs("BatchId", "SourceItemId"), actions


FLOWS = {
    "List Approval Batches TEST ONLY": list_batches,
    "Read Approval Page TEST ONLY": read_page,
    "Read Approval Item TEST ONLY": read_item,
}


def build(template: Path, output: Path):
    with ZipFile(template) as source:
        base_files = {name: source.read(name) for name in source.namelist()}
    original_definition = next(name for name in base_files if name.endswith("/definition.json"))
    root_id = original_definition.split("/")[2]
    output.mkdir(parents=True, exist_ok=True)
    for name, factory in FLOWS.items():
        files = dict(base_files)
        flow = json.loads(files[original_definition])
        new_id = str(uuid5(NAMESPACE_URL, "PA36B/" + name))
        flow["name"] = new_id
        flow["id"] = "/providers/Microsoft.Flow/flows/" + new_id
        flow["properties"]["displayName"] = name
        flow["properties"]["connectionReferences"]["shared_sharepointonline"]["source"] = "Embedded"
        schema, flow_actions = factory()
        definition = flow["properties"]["definition"]
        definition["triggers"]["manual"]["inputs"]["schema"] = schema
        definition["actions"] = {
            **authorized_actions(),
            "Authorized_PA36B": gate(flow_actions),
        }
        files[original_definition] = (json.dumps(flow, indent=2) + "\n").encode()
        manifest = json.loads(files["manifest.json"])
        manifest["details"]["displayName"] = name
        manifest["resources"][root_id]["details"]["displayName"] = name
        manifest["resources"][root_id]["suggestedCreationType"] = "New"
        manifest["resources"][root_id]["creationType"] = "New"
        files["manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
        filename = name.lower().replace(" ", "-") + ".zip"
        with ZipFile(output / filename, "w", ZIP_DEFLATED) as package:
            for path in sorted(files):
                package.writestr(path, files[path])


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("template", type=Path)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    build(arguments.template, arguments.output)
