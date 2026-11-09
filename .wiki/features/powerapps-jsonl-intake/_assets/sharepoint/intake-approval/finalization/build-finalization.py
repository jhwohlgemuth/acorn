"""Build isolated PA-37A flow packages from a native Power Automate export."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5
from zipfile import ZIP_DEFLATED, ZipFile


ROOT = Path(__file__).resolve().parent
SCHEMA = json.loads((ROOT.parent / "storage" / "schema.json").read_text())
SITE = SCHEMA["site"]
BATCHES = SCHEMA["objectIds"]["batchesList"]
ITEMS = SCHEMA["objectIds"]["itemsList"]
SOURCES = (SCHEMA["source"]["listId"], SCHEMA["sourceFixtures"]["listId"])
AUTH = {"value": "@json(decodeBase64(triggerOutputs().headers['X-MS-APIM-Tokens']))['$ConnectionKey']", "type": "Raw"}
SECURE = {"secureData": {"properties": ["inputs", "outputs"]}}


def after(name: str) -> dict:
    return {name: ["Succeeded"]}


def compose(value: str, dependency: str | None = None) -> dict:
    return {"type": "Compose", "inputs": value, "runAfter": after(dependency) if dependency else {}, "runtimeConfiguration": SECURE}


def sharepoint(operation: str, parameters: dict, dependency: str | None = None) -> dict:
    return {
        "type": "OpenApiConnection",
        "inputs": {
            "host": {"apiId": "/providers/Microsoft.PowerApps/apis/shared_sharepointonline", "connectionName": "shared_sharepointonline", "operationId": operation},
            "parameters": {"dataset": SITE, **parameters},
            "authentication": AUTH,
            "retryPolicy": {"type": "none"},
        },
        "runAfter": after(dependency) if dependency else {},
        "runtimeConfiguration": SECURE,
    }


def get_items(table: str, query: str, order: str, top: int, dependency: str | None = None, paged: bool = False) -> dict:
    action = sharepoint("GetItems", {"table": table, "$filter": query, "$orderby": order, "$top": top}, dependency)
    if paged:
        action["runtimeConfiguration"]["paginationPolicy"] = {"minimumItemCount": 5001}
    return action


def stop(code: str, message: str) -> dict:
    return {"type": "Terminate", "inputs": {"runStatus": "Failed", "runError": {"code": code, "message": message}}, "runAfter": {}}


def gate(expression: str, actions: dict, code: str, message: str, dependency: str | None = None) -> dict:
    return {
        "type": "If",
        "expression": {"equals": [expression, True]},
        "actions": actions,
        "else": {"actions": {code: stop(code, message)}},
        "runAfter": after(dependency) if dependency else {},
    }


def prepare_actions() -> dict:
    caller = "@toLower(if(not(empty(triggerOutputs()?['headers']?['x-ms-user-email-encoded'])),base64ToString(triggerOutputs()?['headers']?['x-ms-user-email-encoded']),coalesce(triggerOutputs()?['headers']?['x-ms-user-email'],string(null))))"
    batch_filter = "@concat('BatchId eq ',decodeUriComponent('%27'),outputs('Batch_id_PA37A'),decodeUriComponent('%27'))"
    item_filter = "@concat('BatchId eq ',decodeUriComponent('%27'),outputs('Batch_id_PA37A'),decodeUriComponent('%27'))"
    ready = "@and(equals(length(body('Find_batch_PA37A')?['value']),1),equals(first(body('Find_batch_PA37A')?['value'])?['State']?['Value'],'Ready'),greater(int(first(body('Find_batch_PA37A')?['value'])?['SourceCount']),0))"
    complete = "@and(equals(length(body('Read_items_PA37A')?['value']),int(first(body('Find_batch_PA37A')?['value'])?['SourceCount'])),equals(length(body('Read_items_PA37A')?['value']),int(first(body('Find_batch_PA37A')?['value'])?['ItemCount'])),lessOrEquals(length(body('Read_items_PA37A')?['value']),5000),equals(length(body('Invalid_items_PA37A')),0))"
    source_id = "first(body('Find_batch_PA37A')?['value'])?['SourceListId']"
    source_filter = "@concat('Created ge ',decodeUriComponent('%27'),first(body('Find_batch_PA37A')?['value'])?['PeriodStartUtc'],decodeUriComponent('%27'),' and Created lt ',decodeUriComponent('%27'),first(body('Find_batch_PA37A')?['value'])?['PeriodEndUtc'],decodeUriComponent('%27'))"
    source_receipt = "@concat(string(item()?['ID']),'|',item()?['SubmissionId'],'|',item()?['ProjectId'],'|',string(ticks(item()?['Created'])),'|',item()?['@odata.etag'],'|',item()?['PayloadBase64A'],coalesce(item()?['PayloadBase64B'],''))"
    review_receipt = "@concat(string(item()?['SourceItemId']),'|',item()?['SourceSubmissionId'],'|',item()?['ProjectId'],'|',string(ticks(item()?['SourceCreatedUtc'])),'|',item()?['SourceETag'],'|',item()?['SourcePayloadBase64A'],coalesce(item()?['SourcePayloadBase64B'],''))"
    source_text = "base64ToString(concat(item()?['SourcePayloadBase64A'],coalesce(item()?['SourcePayloadBase64B'],'')))"
    source_object = "json(" + source_text + ")"
    draft_text = "if(empty(item()?['DraftProjectionBase64A'])," + source_text + ",base64ToString(concat(item()?['DraftProjectionBase64A'],coalesce(item()?['DraftProjectionBase64B'],''))))"
    draft_object = "json(" + draft_text + ")"
    canonical_draft = "string(setProperty(" + source_object + ",'entry',setProperty(" + source_object + "?['entry'],'summary'," + draft_object + "?['entry']?['summary'])))"
    invalid_draft = "@or(not(equals(item()?['SourceSubmissionId']," + source_object + "?['submissionId'])),not(equals(item()?['ProjectId']," + source_object + "?['projectId'])),and(not(empty(item()?['DraftProjectionBase64A'])),or(not(equals(" + draft_text + "," + canonical_draft + ")),less(length(trim(coalesce(" + draft_object + "?['entry']?['summary'],''))),1),greater(length(trim(coalesce(" + draft_object + "?['entry']?['summary'],''))),1200))))"
    source_complete = "@and(equals(length(body('Read_source_PA37A')?['value']),int(first(body('Find_batch_PA37A')?['value'])?['SourceCount'])),lessOrEquals(length(body('Read_source_PA37A')?['value']),5000),equals(length(body('Beyond_high_water_PA37A')),0),equals(length(intersection(body('Select_source_receipts_PA37A'),body('Select_review_receipts_PA37A'))),int(first(body('Find_batch_PA37A')?['value'])?['SourceCount'])))"
    source_actions = {
        "Read_source_PA37A": get_items("@" + source_id, source_filter, "ID asc", 5001, paged=True),
        "Beyond_high_water_PA37A": {
            "type": "Query",
            "inputs": {"from": "@body('Read_source_PA37A')?['value']", "where": "@greater(int(item()?['ID']),int(first(body('Find_batch_PA37A')?['value'])?['SourceHighWaterId']))"},
            "runAfter": after("Read_source_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Select_source_receipts_PA37A": {
            "type": "Select",
            "inputs": {"from": "@body('Read_source_PA37A')?['value']", "select": source_receipt},
            "runAfter": after("Beyond_high_water_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Select_review_receipts_PA37A": {
            "type": "Select",
            "inputs": {"from": "@body('Read_items_PA37A')?['value']", "select": review_receipt},
            "runAfter": after("Select_source_receipts_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Exact_source_PA37A": gate(source_complete, {"Preflight_ready_PA37A": compose("@length(body('Read_source_PA37A')?['value'])")}, "SourceChangedPA37A", "Source interval, ETags, encoded bytes, or high-water bound no longer match review receipts.", "Select_review_receipts_PA37A"),
    }
    read_actions = {
        "Read_items_PA37A": get_items(ITEMS, item_filter, "SourceItemId asc", 5001, paged=True),
        "Invalid_items_PA37A": {
            "type": "Query",
            "inputs": {
                "from": "@body('Read_items_PA37A')?['value']",
                "where": "@or(not(equals(item()?['ValidationState']?['Value'],'Valid')),not(or(equals(item()?['Decision']?['Value'],'Approve'),and(equals(item()?['Decision']?['Value'],'Exclude'),not(empty(trim(coalesce(item()?['DecisionReason'],'')))))),empty(item()?['SourceSubmissionId']),empty(item()?['SourcePayloadBase64A']),empty(item()?['SourceETag']))",
            },
            "runAfter": after("Read_items_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Invalid_drafts_PA37A": {
            "type": "Query",
            "inputs": {"from": "@body('Read_items_PA37A')?['value']", "where": invalid_draft},
            "runAfter": after("Complete_batch_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Select_source_uuids_PA37A": {
            "type": "Select",
            "inputs": {"from": "@body('Read_items_PA37A')?['value']", "select": "@item()?['SourceSubmissionId']"},
            "runAfter": after("Invalid_drafts_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Valid_drafts_PA37A": gate("@and(equals(length(body('Invalid_drafts_PA37A')),0),equals(length(body('Select_source_uuids_PA37A')),length(union(body('Select_source_uuids_PA37A'),body('Select_source_uuids_PA37A')))))", {"Bound_source_PA37A": gate("@or(equals(" + source_id + ",'" + SOURCES[0] + "'),equals(" + source_id + ",'" + SOURCES[1] + "'))", source_actions, "UnboundSourcePA37A", "Batch has no trusted materialization source.")}, "InvalidDraftPA37A", "Draft bytes, immutable identity, or source submission IDs are invalid.", "Select_source_uuids_PA37A"),
        "Complete_batch_PA37A": gate(complete, {"Drafts_checked_PA37A": compose("@length(body('Read_items_PA37A')?['value'])")}, "InvalidBatchPA37A", "Batch membership or decisions are incomplete.", "Invalid_items_PA37A"),
    }
    authorized = {
        "Batch_id_PA37A": compose("@trim(triggerBody()?['text'])"),
        "Find_batch_PA37A": get_items(BATCHES, batch_filter, "ID asc", 2, "Batch_id_PA37A"),
        "Ready_batch_PA37A": gate(ready, read_actions, "BatchNotReadyPA37A", "Batch is unavailable, empty, or not Ready.", "Find_batch_PA37A"),
    }
    return {
        "Caller_email_PA37A": compose(caller),
        "Read_finalizers_PA37A": sharepoint("HttpRequest", {"parameters/method": "GET", "parameters/uri": "_api/web/sitegroups/getbyid(43)/users", "parameters/headers": {"Accept": "application/json;odata=nometadata"}}, "Caller_email_PA37A"),
        "Caller_membership_PA37A": {
            "type": "Query",
            "inputs": {"from": "@body('Read_finalizers_PA37A')?['value']", "where": "@equals(toLower(item()?['LoginName']),concat('i:0#.f|membership|',outputs('Caller_email_PA37A')))"},
            "runAfter": after("Read_finalizers_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Authorized_PA37A": gate("@and(equals(length(body('Caller_membership_PA37A')),1),equals(length(triggerBody()?['text']),36),not(contains(triggerBody()?['text'],decodeUriComponent('%27'))))", authorized, "ForbiddenPA37A", "Only a current finalizer may prepare a batch.", "Caller_membership_PA37A"),
    }


def build(template: Path, output: Path) -> Path:
    with ZipFile(template) as source:
        files = {name: source.read(name) for name in source.namelist()}
    definition_path = next(name for name in files if name.endswith("/definition.json"))
    resource_id = definition_path.split("/")[2]
    flow = json.loads(files[definition_path])
    name = "Prepare Approval Finalization TEST ONLY"
    flow_id = str(uuid5(NAMESPACE_URL, "PA37A/" + name))
    flow["name"] = flow_id
    flow["id"] = "/providers/Microsoft.Flow/flows/" + flow_id
    flow["properties"]["displayName"] = name
    flow["properties"]["connectionReferences"]["shared_sharepointonline"]["source"] = "Embedded"
    flow["properties"]["definition"]["actions"] = prepare_actions()
    files[definition_path] = (json.dumps(flow, indent=2) + "\n").encode()
    manifest = json.loads(files["manifest.json"])
    manifest["details"]["displayName"] = name
    manifest["resources"][resource_id]["details"]["displayName"] = name
    manifest["resources"][resource_id]["suggestedCreationType"] = "New"
    manifest["resources"][resource_id]["creationType"] = "New"
    files["manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    output.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(output, "w", ZIP_DEFLATED) as package:
        for path in sorted(files):
            package.writestr(path, files[path])
    return output


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("template", type=Path)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    print(build(arguments.template, arguments.output))
