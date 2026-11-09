"""Build the PA-36A review-item materialization clipboard scope."""

import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5


ROOT = Path(__file__).resolve().parent
SHAREPOINT = ROOT.parent.parent
SCHEMA = json.loads((ROOT / "schema.json").read_text())
SITE = SCHEMA["site"]
SOURCE = SCHEMA["source"]["listId"]
BATCHES = SCHEMA["objectIds"]["batchesList"]
ITEMS = SCHEMA["objectIds"]["itemsList"]
SECURE = {"secureData": {"properties": ["inputs", "outputs"]}}
EMPTY = "substring('x',0,0)"


def after(name):
    return {name: ["Succeeded"]}


def connector(operation, parameters, dependency=None, pagination=False):
    action = {
        "type": "OpenApiConnection",
        "inputs": {
            "host": {
                "connectionName": "shared_sharepointonline",
                "operationId": operation,
                "apiId": "/providers/Microsoft.PowerApps/apis/shared_sharepointonline",
            },
            "parameters": {"dataset": SITE, **parameters},
            "authentication": "@parameters('$authentication')",
            "retryPolicy": {"type": "none"},
        },
        "runAfter": after(dependency) if dependency else {},
        "runtimeConfiguration": {"secureData": {"properties": ["inputs", "outputs"]}},
    }
    if pagination:
        action["runtimeConfiguration"]["paginationPolicy"] = {"minimumItemCount": 5001}
    return action


def data_action(kind, inputs, dependency=None):
    return {
        "type": kind,
        "inputs": inputs,
        "runAfter": after(dependency) if dependency else {},
        "runtimeConfiguration": SECURE,
    }


def stop(code, message):
    return {
        "type": "Terminate",
        "inputs": {"runStatus": "Failed", "runError": {"code": code, "message": message}},
        "runAfter": {},
    }


def gate(expression, yes, code, message, dependency=None, failure=None):
    return {
        "type": "If",
        "expression": expression,
        "actions": yes,
        "else": {"actions": {code: failure or stop(code, message)}},
        "runAfter": after(dependency) if dependency else {},
    }


def encoded(row="item()"):
    return f"concat(coalesce({row}?['PayloadBase64A'],{EMPTY}),coalesce({row}?['PayloadBase64B'],{EMPTY}))"


def payload(row="item()"):
    return f"json(base64ToString({encoded(row)}))"


def batch(field):
    return f"@first(body('Find_batch_mat_PA36A')?['value'])?['{field}']"


def source_filter():
    start = "first(body('Find_batch_mat_PA36A')?['value'])?['PeriodStartUtc']"
    end = "first(body('Find_batch_mat_PA36A')?['value'])?['PeriodEndUtc']"
    return (
        "@concat('Created ge ',decodeUriComponent('%27')," + start
        + ",decodeUriComponent('%27'),' and Created lt ',decodeUriComponent('%27'),"
        + end
        + ",decodeUriComponent('%27'),' and ID le ',string(outputs('High_water_PA36A')))"
    )


def membership(source_row="item()"):
    return f"concat(triggerBody()?['text'],'|',{source_row}?['SubmissionId'])"


def sort_key(source_row="item()"):
    return (
        f"concat({source_row}?['ProjectId'],'|',"
        f"formatDateTime({source_row}?['Created'],'yyyyMMddHHmmssfffffff'),'|',"
        f"{source_row}?['SubmissionId'])"
    )


def source_fingerprint():
    return {
        "key": "@" + membership(),
        "sourceItemId": "@item()?['ID']",
        "submissionId": "@item()?['SubmissionId']",
        "payloadA": "@item()?['PayloadBase64A']",
        "payloadB": "@coalesce(item()?['PayloadBase64B']," + EMPTY + ")",
        "etag": "@item()?['@odata.etag']",
        "createdTicks": "@ticks(item()?['Created'])",
        "projectId": "@item()?['ProjectId']",
        "sortKey": "@" + sort_key(),
        "revisionRoot": "@coalesce(" + payload() + "?['revision']?['rootSubmissionId']," + EMPTY + ")",
        "supersedes": "@coalesce(" + payload() + "?['revision']?['supersedesSubmissionId']," + EMPTY + ")",
    }


def item_fingerprint():
    return {
        "key": "@item()?['MembershipKey']",
        "sourceItemId": "@item()?['SourceItemId']",
        "submissionId": "@item()?['SourceSubmissionId']",
        "payloadA": "@item()?['SourcePayloadBase64A']",
        "payloadB": "@coalesce(item()?['SourcePayloadBase64B']," + EMPTY + ")",
        "etag": "@item()?['SourceETag']",
        "createdTicks": "@ticks(item()?['SourceCreatedUtc'])",
        "projectId": "@item()?['ProjectId']",
        "sortKey": "@item()?['ReportSortKey']",
        "revisionRoot": "@coalesce(item()?['SourceRevisionRoot']," + EMPTY + ")",
        "supersedes": "@coalesce(item()?['SourceSupersedes']," + EMPTY + ")",
    }


def fingerprint_text():
    return "@string(setProperty(item(),'sourceItemId',string(item()?['sourceItemId'])))"


def select(name, rows, fields, dependency):
    return name, data_action("Select", {"from": rows, "select": fields}, dependency)


def source_items(name, dependency):
    return connector(
        "GetItems",
        {"table": SOURCE, "$filter": source_filter(), "$orderby": "ID asc", "$top": 5001},
        dependency,
        pagination=True,
    )


def review_items(name, dependency):
    return connector(
        "GetItems",
        {
            "table": ITEMS,
            "$filter": "@concat('BatchId eq ',decodeUriComponent('%27'),triggerBody()?['text'],decodeUriComponent('%27'))",
            "$orderby": "SourceItemId asc",
            "$top": 5001,
        },
        dependency,
        pagination=True,
    )


def batch_update(state, dependency):
    fields = {
        "table": BATCHES,
        "id": batch("ID"),
        "item/Title": batch("Title"),
        "item/BatchId": batch("BatchId"),
        "item/PeriodKey": batch("PeriodKey"),
        "item/PolicyRevision": batch("PolicyRevision"),
        "item/PeriodStartUtc": batch("PeriodStartUtc"),
        "item/PeriodEndUtc": batch("PeriodEndUtc"),
        "item/CreatedByEmail": batch("CreatedByEmail"),
        "item/SourceHighWaterId": "@outputs('High_water_PA36A')",
        "item/SourceListId": SOURCE,
        "item/SourceCount": "@outputs('Source_count_PA36A')",
        "item/State/Value": state,
        "item/MaterializationRunId": "@workflow()?['run']?['name']",
    }
    if state == "Ready":
        fields.update(
            {
                "item/MaterializedAtUtc": "@utcNow()",
                "item/ItemCount": "@outputs('Source_count_PA36A')",
                "item/ApprovedCount": 0,
                "item/ExcludedCount": 0,
                "item/UnresolvedCount": "@outputs('Source_count_PA36A')",
            }
        )
    return connector("PatchItem", fields, dependency)


def create_item():
    fields = {
        "table": ITEMS,
        "item/Title": "@" + membership(),
        "item/MembershipKey": "@" + membership(),
        "item/BatchId": "@triggerBody()?['text']",
        "item/SourceItemId": "@item()?['ID']",
        "item/SourceSubmissionId": "@item()?['SubmissionId']",
        "item/SourcePayloadBase64A": "@item()?['PayloadBase64A']",
        "item/SourcePayloadBase64B": "@coalesce(item()?['PayloadBase64B']," + EMPTY + ")",
        "item/SourceETag": "@item()?['@odata.etag']",
        "item/SourceCreatedUtc": "@item()?['Created']",
        "item/SourceRevisionRoot": "@coalesce(" + payload() + "?['revision']?['rootSubmissionId']," + EMPTY + ")",
        "item/SourceSupersedes": "@coalesce(" + payload() + "?['revision']?['supersedesSubmissionId']," + EMPTY + ")",
        "item/ProjectId": "@item()?['ProjectId']",
        "item/ReportSortKey": "@" + sort_key(),
        "item/DraftVersion": 1,
        "item/Decision/Value": "Unresolved",
        "item/ValidationState/Value": "Valid",
    }
    return connector("PostItem", fields)


def build_actions():
    source = "@body('Read_bounded_source_PA36A')?['value']"
    invalid = (
        "@or(equals(item()?['ID'],null),empty(item()?['Created']),empty(item()?['SubmissionId']),"
        "empty(item()?['ProjectId']),empty(item()?['PayloadBase64A']),empty(item()?['@odata.etag']),"
        f"greater(length({encoded()}),87380),"
        f"not(equals({payload()}?['submissionId'],item()?['SubmissionId'])),"
        f"not(equals({payload()}?['projectId'],item()?['ProjectId'])),"
        f"empty({payload()}?['entry']?['summary']),"
        f"and(not(empty({payload()}?['revision'])),"
        f"or(empty({payload()}?['revision']?['rootSubmissionId']),"
        f"empty({payload()}?['revision']?['supersedesSubmissionId']),"
        f"equals({payload()}?['revision']?['supersedesSubmissionId'],item()?['SubmissionId']))))"
    )
    actions = {
        "Invalid_source_rows_PA36A": data_action("Query", {"from": source, "where": invalid}),
    }
    valid = {
        "Corrections_PA36A": data_action(
            "Query",
            {"from": source, "where": "@not(empty(" + payload() + "?['revision']?['supersedesSubmissionId']))"},
        ),
    }
    parent_filter = (
        "@concat('SubmissionId eq ',decodeUriComponent('%27'),"
        + payload() + "?['revision']?['supersedesSubmissionId'],decodeUriComponent('%27'))"
    )
    parent = connector("GetItems", {"table": SOURCE, "$filter": parent_filter, "$top": 2})
    parent_row = "first(body('Find_parent_PA36A')?['value'])"
    parent_payload = payload(parent_row)
    correction_root = payload() + "?['revision']?['rootSubmissionId']"
    parent_valid = {
        "and": [
            {"equals": ["@length(body('Find_parent_PA36A')?['value'])", 1]},
            {"less": ["@" + parent_row + "?['ID']", "@item()?['ID']"]},
            {"equals": ["@coalesce(" + parent_payload + "?['revision']?['rootSubmissionId']," + parent_row + "?['SubmissionId'])", "@" + correction_root]},
        ]
    }
    correction_loop = {
        "type": "Foreach",
        "foreach": "@body('Corrections_PA36A')",
        "actions": {
            "Find_parent_PA36A": parent,
            "Parent_valid_PA36A": gate(
                parent_valid,
                {},
                "InvalidCorrectionGraphPA36A",
                "Correction parent/root is missing or inconsistent.",
                "Find_parent_PA36A",
                failure={"type": "Compose", "inputs": "@json('InvalidCorrectionGraphPA36A')", "runAfter": {}},
            ),
        },
        "runAfter": after("Corrections_PA36A"),
        "runtimeConfiguration": {"concurrency": {"repetitions": 1}},
    }
    valid["Check_corrections_PA36A"] = correction_loop
    valid["Read_existing_items_PA36A"] = review_items("Read_existing_items_PA36A", "Check_corrections_PA36A")
    valid["Select_source_before_PA36A"] = select(
        "Select_source_before_PA36A", source, source_fingerprint(), "Read_existing_items_PA36A"
    )[1]
    valid["Select_existing_before_PA36A"] = select(
        "Select_existing_before_PA36A", "@body('Read_existing_items_PA36A')?['value']", item_fingerprint(), "Select_source_before_PA36A"
    )[1]
    valid["Select_source_before_text_PA36A"] = select(
        "Select_source_before_text_PA36A", "@body('Select_source_before_PA36A')", fingerprint_text(), "Select_existing_before_PA36A"
    )[1]
    valid["Select_existing_before_text_PA36A"] = select(
        "Select_existing_before_text_PA36A", "@body('Select_existing_before_PA36A')", fingerprint_text(), "Select_source_before_text_PA36A"
    )[1]
    subset = {
        "and": [
            {"lessOrEquals": ["@length(body('Select_existing_before_PA36A'))", "@outputs('Source_count_PA36A')"]},
            {"equals": ["@length(intersection(body('Select_existing_before_text_PA36A'),body('Select_source_before_text_PA36A')))", "@length(body('Select_existing_before_PA36A'))"]},
        ]
    }
    commit = {
        "Mark_materializing_PA36A": batch_update("Materializing", None),
        "Select_existing_keys_PA36A": data_action(
            "Select",
            {"from": "@body('Read_existing_items_PA36A')?['value']", "select": "@item()?['MembershipKey']"},
            "Mark_materializing_PA36A",
        ),
        "Missing_source_rows_PA36A": data_action(
            "Query",
            {
                "from": source,
                "where": "@not(contains(body('Select_existing_keys_PA36A')," + membership() + "))",
            },
            "Select_existing_keys_PA36A",
        ),
        "Create_missing_items_PA36A": {
            "type": "Foreach",
            "foreach": "@body('Missing_source_rows_PA36A')",
            "actions": {"Create_review_item_PA36A": create_item()},
            "runAfter": after("Missing_source_rows_PA36A"),
            "runtimeConfiguration": {"concurrency": {"repetitions": 1}},
        },
        "Read_source_again_PA36A": source_items("Read_source_again_PA36A", "Create_missing_items_PA36A"),
        "Read_items_again_PA36A": review_items("Read_items_again_PA36A", "Read_source_again_PA36A"),
        "Select_source_again_PA36A": select(
            "Select_source_again_PA36A", "@body('Read_source_again_PA36A')?['value']", source_fingerprint(), "Read_items_again_PA36A"
        )[1],
        "Select_items_again_PA36A": select(
            "Select_items_again_PA36A", "@body('Read_items_again_PA36A')?['value']", item_fingerprint(), "Select_source_again_PA36A"
        )[1],
        "Select_source_again_text_PA36A": select(
            "Select_source_again_text_PA36A", "@body('Select_source_again_PA36A')", fingerprint_text(), "Select_items_again_PA36A"
        )[1],
        "Select_items_again_text_PA36A": select(
            "Select_items_again_text_PA36A", "@body('Select_items_again_PA36A')", fingerprint_text(), "Select_source_again_text_PA36A"
        )[1],
    }
    reconciled = {
        "and": [
            {"equals": ["@length(body('Select_source_again_PA36A'))", "@outputs('Source_count_PA36A')"]},
            {"equals": ["@length(body('Select_items_again_PA36A'))", "@outputs('Source_count_PA36A')"]},
            {"equals": ["@body('Select_source_again_text_PA36A')", "@body('Select_source_before_text_PA36A')"]},
            {"equals": ["@length(intersection(body('Select_source_again_text_PA36A'),body('Select_items_again_text_PA36A')))", "@outputs('Source_count_PA36A')"]},
        ]
    }
    commit["Reconciled_PA36A"] = gate(
        reconciled,
        {"Mark_ready_PA36A": batch_update("Ready", None)},
        "SourceOrItemsChangedPA36A",
        "Source ETag, exact encoded payload, or membership changed during materialization.",
        "Select_items_again_text_PA36A",
    )
    valid["Existing_subset_PA36A"] = gate(
        subset, commit, "ExistingReviewMismatchPA36A", "Existing review items differ from captured source bytes/ETags.", "Select_existing_before_text_PA36A"
    )
    actions["Source_valid_PA36A"] = gate(
        {"and": [
            {"equals": ["@length(body('Invalid_source_rows_PA36A'))", 0]},
            {"lessOrEquals": ["@outputs('Source_count_PA36A')", 5000]},
        ]},
        valid,
        "InvalidSourcePayloadPA36A",
        "Source rows are malformed, inconsistent, or exceed the bounded count.",
        "Invalid_source_rows_PA36A",
    )
    return actions


def build():
    reference = json.loads((SHAREPOINT / "SharePoint_intake_SP.clipboard.json").read_text())
    name = "Materialize_review_items_PA36A"
    return {
        "id": str(uuid5(NAMESPACE_URL, SITE + "/" + name)),
        "brandColor": "#8C6CFF",
        "connectionReferences": reference["connectionReferences"],
        "connectorDisplayName": "Control",
        "isTrigger": False,
        "operationName": name,
        "operationDefinition": {"type": "Scope", "actions": build_actions(), "runAfter": {}},
    }


if __name__ == "__main__":
    (ROOT / "materialization.json").write_text(json.dumps(build(), indent=2) + "\n")
