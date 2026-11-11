"""Build isolated PA-37A/B flow packages from a native Power Automate export."""

from __future__ import annotations

import argparse
from copy import deepcopy
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5
from zipfile import ZIP_DEFLATED, ZipFile


ROOT = Path(__file__).resolve().parent
SCHEMA = json.loads((ROOT.parent / "storage" / "schema.json").read_text())
SITE = SCHEMA["site"]
BATCHES = SCHEMA["objectIds"]["batchesList"]
ITEMS = SCHEMA["objectIds"]["itemsList"]
LIBRARY = SCHEMA["objectIds"]["approvedProjectionsLibrary"]
LIBRARY_PATH = SCHEMA["library"]["path"]
SOURCES = (SCHEMA["source"]["listId"], SCHEMA["sourceFixtures"]["listId"], SCHEMA["sourcePa31"]["listId"])


def trusted_source(source_id: str) -> str:
    return "@or(" + ",".join("equals(" + source_id + ",'" + list_id + "')" for list_id in SOURCES) + ")"
AUTH = {"value": "@json(decodeBase64(triggerOutputs().headers['X-MS-APIM-Tokens']))['$ConnectionKey']", "type": "Raw"}
SECURE = {"secureData": {"properties": ["inputs", "outputs"]}}
COMPOSE_SECURE = {"secureData": {"properties": ["inputs"]}}


def after(name: str) -> dict:
    return {name: ["Succeeded"]}


def compose(value: str, dependency: str | None = None) -> dict:
    return {"type": "Compose", "inputs": value, "runAfter": after(dependency) if dependency else {}, "runtimeConfiguration": deepcopy(COMPOSE_SECURE)}


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
        "runtimeConfiguration": deepcopy(SECURE),
    }


def get_items(table: str, query: str, order: str, top: int, dependency: str | None = None, paged: bool = False) -> dict:
    action = sharepoint("GetItems", {"table": table, "$filter": query, "$orderby": order, "$top": top}, dependency)
    if paged:
        action["runtimeConfiguration"]["paginationPolicy"] = {"minimumItemCount": 5001}
    return action


def variable(name: str, kind: str, value: object, dependency: str | None = None) -> dict:
    return {"type": "InitializeVariable", "inputs": {"variables": [{"name": name, "type": kind, "value": value}]}, "runAfter": after(dependency) if dependency else {}}


def set_variable(name: str, value: str, dependency: str) -> dict:
    return {"type": "SetVariable", "inputs": {"name": name, "value": value}, "runAfter": after(dependency)}


def append_variable(name: str, value: str, dependency: str | None = None) -> dict:
    return {"type": "AppendToArrayVariable", "inputs": {"name": name, "value": value}, "runAfter": after(dependency) if dependency else {}}


def increment_variable(name: str, value: str, dependency: str) -> dict:
    return {"type": "IncrementVariable", "inputs": {"name": name, "value": value}, "runAfter": after(dependency)}


def create_file(folder: str, name: str, content: str, dependency: str) -> dict:
    return sharepoint("CreateFile", {"folderPath": folder, "name": name, "body": content}, dependency)


def read_file(path: str, dependency: str | None = None) -> dict:
    return sharepoint("GetFileContentByPath", {"path": path, "inferContentType": False}, dependency)


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


def loop_gate(expression: str, actions: dict, code: str, message: str, dependency: str | None = None) -> dict:
    # Terminate cannot be nested under Foreach; a failed Compose stops this item and the loop.
    return {
        "type": "If",
        "expression": {"equals": [expression, True]},
        "actions": actions,
        "else": {"actions": {code: compose("@json('PA37A_" + code + ": " + message.replace("'", "''") + "')")}},
        "runAfter": after(dependency) if dependency else {},
    }


def assertion_gate(expression: str, code: str, message: str, dependency: str) -> dict:
    return gate(expression, {f"{code}_passed": compose("@true")}, code, message, dependency)


def ancestor_actions(label: str, source: str, dependency: str | None = None) -> dict:
    source_name = f"{label}_source_PA37A"
    lookup_name = f"Lookup_{label}_PA37A"
    unique_name = f"Unique_{label}_PA37A"
    batch_name = f"Prior_batch_{label}_PA37A"
    prior_name = f"Approved_prior_{label}_PA37A"
    prior_unique_name = f"Unique_prior_id_{label}_PA37A"
    mapped_name = f"Mapped_{label}_PA37A"
    prior_filter = "@concat('SourceSubmissionId eq ',decodeUriComponent('%27'),replace(outputs('" + source_name + "'),decodeUriComponent('%27'),concat(decodeUriComponent('%27'),decodeUriComponent('%27'))),decodeUriComponent('%27'))"
    batch_filter = "@concat('BatchId eq ',decodeUriComponent('%27'),first(body('" + lookup_name + "')?['value'])?['BatchId'],decodeUriComponent('%27'))"
    prior_id = "coalesce(first(body('" + lookup_name + "')?['value'])?['ApprovedSubmissionId'],first(body('" + lookup_name + "')?['value'])?['SourceSubmissionId'])"
    lookup = {
        lookup_name: get_items(ITEMS, prior_filter, "ID asc", 2),
        unique_name: loop_gate("@and(equals(length(body('" + lookup_name + "')?['value']),1),not(equals(first(body('" + lookup_name + "')?['value'])?['BatchId'],outputs('Batch_id_PA37A'))),equals(first(body('" + lookup_name + "')?['value'])?['Decision']?['Value'],'Approve'))", {batch_name: get_items(BATCHES, batch_filter, "ID asc", 2)}, f"Ambiguous{label}PA37A", "Correction ancestor is missing, excluded, or ambiguous.", lookup_name),
        prior_name: loop_gate("@and(equals(length(body('" + batch_name + "')?['value']),1),equals(first(body('" + batch_name + "')?['value'])?['State']?['Value'],'Approved'),lessOrEquals(ticks(first(body('" + batch_name + "')?['value'])?['PeriodEndUtc']),ticks(first(body('Find_batch_PA37A')?['value'])?['PeriodStartUtc'])),not(empty(" + prior_id + ")))", {mapped_name: compose("@setProperty(variables('ProjectionMap'),outputs('" + source_name + "')," + prior_id + ")")}, f"Unapproved{label}PA37A", "Correction ancestor has no earlier Approved batch.", unique_name),
        prior_unique_name: loop_gate("@not(contains(variables('UsedApprovedIds')," + prior_id + "))", {f"Prior_id_unique_{label}_PA37A": compose("@" + prior_id)}, f"DuplicatePriorApprovedId{label}PA37A", "Distinct prior ancestors resolve to one approved submission ID.", prior_name),
        f"Store_{label}_PA37A": set_variable("ProjectionMap", "@outputs('" + mapped_name + "')", prior_unique_name),
        f"Remember_{label}_PA37A": append_variable("UsedApprovedIds", "@" + prior_id, f"Store_{label}_PA37A"),
    }
    return {
        source_name: compose(source, dependency),
        f"Resolve_{label}_PA37A": {
            "type": "If",
            "expression": {"equals": ["@contains(variables('ProjectionMap'),outputs('" + source_name + "'))", True]},
            "actions": {f"Known_{label}_PA37A": compose("@outputs('" + source_name + "')")},
            "else": {"actions": lookup},
            "runAfter": after(source_name),
        },
    }


def candidate_actions() -> dict:
    row = "items('Each_review_item_PA37A')"
    source_text = "base64ToString(concat(" + row + "?['SourcePayloadBase64A'],coalesce(" + row + "?['SourcePayloadBase64B'],'')))"
    draft_text = "if(empty(" + row + "?['DraftProjectionBase64A'])," + source_text + ",base64ToString(concat(" + row + "?['DraftProjectionBase64A'],coalesce(" + row + "?['DraftProjectionBase64B'],''))))"
    source_id = "outputs('Source_object_PA37A')?['submissionId']"
    revision = "outputs('Source_object_PA37A')?['revision']"
    mapped_root = "variables('ProjectionMap')?[" + revision + "?['rootSubmissionId']]"
    mapped_prior = "variables('ProjectionMap')?[" + revision + "?['supersedesSubmissionId']]"
    projected = "if(empty(" + revision + "),outputs('Draft_object_PA37A'),setProperty(outputs('Draft_object_PA37A'),'revision',setProperty(setProperty(outputs('Draft_object_PA37A')?['revision'],'rootSubmissionId'," + mapped_root + "),'supersedesSubmissionId'," + mapped_prior + ")))"
    changed = "not(and(equals(outputs('Draft_text_PA37A'),outputs('Source_text_PA37A')),equals(outputs('Projected_base_PA37A')?['revision']," + revision + ")))"
    approved_id = "if(outputs('Changed_PA37A'),if(empty(" + row + "?['ApprovedSubmissionId']),outputs('New_approval_id_PA37A')," + row + "?['ApprovedSubmissionId'])," + source_id + ")"
    update_uri = "@concat('_api/web/lists(guid%27" + ITEMS + "%27)/items(',string(" + row + "?['ID']),')')"
    update = sharepoint("HttpRequest", {
        "parameters/method": "POST",
        "parameters/uri": update_uri,
        "parameters/headers": {"Accept": "application/json;odata=nometadata", "Content-Type": "application/json;odata=nometadata", "IF-MATCH": "@" + row + "?['@odata.etag']", "X-HTTP-Method": "MERGE"},
        "parameters/body": "@setProperty(json('{}'),'ApprovedSubmissionId',outputs('New_approval_id_PA37A'))",
    })
    assign = {
        "Write_approval_id_PA37A": update,
        "Reread_approval_item_PA37A": get_items(ITEMS, "@concat('ID eq ',string(" + row + "?['ID']))", "ID asc", 2, "Write_approval_id_PA37A"),
        "Exact_approval_id_PA37A": loop_gate("@and(equals(length(body('Reread_approval_item_PA37A')?['value']),1),equals(first(body('Reread_approval_item_PA37A')?['value'])?['ApprovedSubmissionId'],outputs('New_approval_id_PA37A')),equals(first(body('Reread_approval_item_PA37A')?['value'])?['DraftVersion']," + row + "?['DraftVersion']),equals(first(body('Reread_approval_item_PA37A')?['value'])?['Decision']?['Value'],'Approve'))", {"Id_persisted_PA37A": compose("@outputs('New_approval_id_PA37A')")}, "IdWriteRacePA37A", "Approval identity write could not be reconciled.", "Reread_approval_item_PA37A"),
    }
    candidate_json = "if(outputs('Changed_PA37A'),string(setProperty(outputs('Projected_base_PA37A'),'submissionId',outputs('Approved_id_PA37A'))),outputs('Source_text_PA37A'))"
    folder = "@concat('" + LIBRARY_PATH + "/Pending/',outputs('Batch_id_PA37A'),'/',outputs('Candidate_run_id_PA37A'))"
    file_name = "@concat(string(" + row + "?['SourceItemId']),'.jsonl')"
    path = "@concat('" + LIBRARY_PATH + "/Pending/',outputs('Batch_id_PA37A'),'/',outputs('Candidate_run_id_PA37A'),'/',string(" + row + "?['SourceItemId']),'.jsonl')"
    current_etag = "if(and(outputs('Changed_PA37A'),empty(" + row + "?['ApprovedSubmissionId'])),first(body('Reread_approval_item_PA37A')?['value'])?['@odata.etag']," + row + "?['@odata.etag'])"
    captured_draft = "setProperty(setProperty(json('{}'),'draftProjectionBase64A'," + row + "?['DraftProjectionBase64A']),'draftProjectionBase64B'," + row + "?['DraftProjectionBase64B'])"
    record = "@setProperty(setProperty(setProperty(setProperty(setProperty(setProperty(setProperty(setProperty(setProperty(" + captured_draft + ",'sourceItemId'," + row + "?['SourceItemId']),'sourceSubmissionId'," + source_id + "),'approvedSubmissionId',outputs('Approved_id_PA37A')),'sourceETag'," + row + "?['SourceETag']),'reviewETag'," + current_etag + "),'draftVersion'," + row + "?['DraftVersion']),'decision','Approve'),'approvedSupersedes',if(empty(" + revision + "),null," + mapped_prior + ")),'payloadBase64',base64(outputs('Candidate_json_PA37A')))"
    approve = {
        "Source_text_PA37A": compose("@" + source_text),
        "Source_object_PA37A": compose("@json(outputs('Source_text_PA37A'))", "Source_text_PA37A"),
        "Draft_text_PA37A": compose("@" + draft_text, "Source_object_PA37A"),
        "Draft_object_PA37A": compose("@json(outputs('Draft_text_PA37A'))", "Draft_text_PA37A"),
        "Source_identity_PA37A": loop_gate("@and(equals(" + source_id + "," + row + "?['SourceSubmissionId']),not(empty(" + source_id + ")),equals(outputs('Draft_object_PA37A')?['submissionId']," + source_id + "))", {"Identity_checked_PA37A": compose("@" + source_id)}, "InvalidIdentityPA37A", "Source or draft identity changed.", "Draft_object_PA37A"),
        "Revision_PA37A": {"type": "If", "expression": {"equals": ["@not(empty(" + revision + "))", True]}, "actions": {**ancestor_actions("root", "@" + revision + "?['rootSubmissionId']"), **ancestor_actions("predecessor", "@" + revision + "?['supersedesSubmissionId']", "Resolve_root_PA37A")}, "else": {"actions": {"No_revision_PA37A": compose("@" + source_id)}}, "runAfter": after("Source_identity_PA37A")},
        "Projected_base_PA37A": compose("@" + projected, "Revision_PA37A"),
        "Changed_PA37A": compose("@" + changed, "Projected_base_PA37A"),
        "New_approval_id_PA37A": compose("@guid()", "Changed_PA37A"),
        "Approved_id_PA37A": compose("@" + approved_id, "New_approval_id_PA37A"),
        "Unique_approved_id_PA37A": loop_gate("@and(not(contains(variables('UsedApprovedIds'),outputs('Approved_id_PA37A'))),or(not(outputs('Changed_PA37A')),not(contains(body('Select_source_uuids_PA37A'),outputs('Approved_id_PA37A')))),or(outputs('Changed_PA37A'),empty(" + row + "?['ApprovedSubmissionId']),equals(" + row + "?['ApprovedSubmissionId']," + source_id + ")))", {"Approved_id_unique_PA37A": compose("@outputs('Approved_id_PA37A')")}, "ApprovalCollisionPA37A", "Approved submission ID collides with a source or approved ID.", "Approved_id_PA37A"),
        "Persist_id_PA37A": {"type": "If", "expression": {"equals": ["@and(outputs('Changed_PA37A'),empty(" + row + "?['ApprovedSubmissionId']))", True]}, "actions": assign, "else": {"actions": {"Id_already_stable_PA37A": compose("@" + source_id)}}, "runAfter": after("Unique_approved_id_PA37A")},
        "Candidate_json_PA37A": compose("@" + candidate_json, "Persist_id_PA37A"),
        "Write_candidate_PA37A": create_file(folder, file_name, "@base64ToBinary(base64(outputs('Candidate_json_PA37A')))", "Candidate_json_PA37A"),
        "Read_candidate_PA37A": read_file(path, "Write_candidate_PA37A"),
        "Exact_candidate_PA37A": loop_gate("@equals(base64ToString(body('Read_candidate_PA37A')?['$content']),outputs('Candidate_json_PA37A'))", {"Candidate_verified_PA37A": compose("@length(outputs('Candidate_json_PA37A'))")}, "CandidateReadbackPA37A", "Pending candidate bytes did not reread exactly.", "Read_candidate_PA37A"),
        "Next_mapping_PA37A": compose("@setProperty(variables('ProjectionMap')," + source_id + ",outputs('Approved_id_PA37A'))", "Exact_candidate_PA37A"),
        "Store_mapping_PA37A": set_variable("ProjectionMap", "@outputs('Next_mapping_PA37A')", "Next_mapping_PA37A"),
        "Store_approved_id_PA37A": append_variable("UsedApprovedIds", "@outputs('Approved_id_PA37A')", "Store_mapping_PA37A"),
        "Store_candidate_PA37A": append_variable("CandidateRows", record, "Store_approved_id_PA37A"),
    }
    excluded = "@setProperty(setProperty(setProperty(setProperty(setProperty(setProperty(" + captured_draft + ",'sourceItemId'," + row + "?['SourceItemId']),'sourceSubmissionId'," + row + "?['SourceSubmissionId']),'sourceETag'," + row + "?['SourceETag']),'reviewETag'," + row + "?['@odata.etag']" + "),'draftVersion'," + row + "?['DraftVersion']" + "),'reason'," + row + "?['DecisionReason'])"
    return {
        "Decision_PA37A": {
            "type": "If",
            "expression": {"equals": ["@equals(" + row + "?['Decision']?['Value'],'Approve')", True]},
            "actions": approve,
            "else": {"actions": {"Store_exclusion_PA37A": append_variable("ExcludedRows", excluded)}},
            "runAfter": {},
        }
    }


def projection_actions(dependency: str) -> dict:
    batch_id = "outputs('Batch_id_PA37A')"
    manifest_path = "@concat('" + LIBRARY_PATH + "/Pending/'," + batch_id + ",'/',outputs('Candidate_run_id_PA37A'),'/manifest.json')"
    manifest = "@setProperty(setProperty(setProperty(setProperty(setProperty(setProperty(setProperty(json('{}'),'batchId'," + batch_id + "),'candidateRunId',outputs('Candidate_run_id_PA37A')),'sourceListId',first(body('Find_batch_PA37A')?['value'])?['SourceListId']),'sourceCount',int(first(body('Find_batch_PA37A')?['value'])?['SourceCount'])),'included',variables('CandidateRows')),'excluded',variables('ExcludedRows')),'mapping',variables('ProjectionMap'))"
    return {
        "Create_pending_folder_PA37A": sharepoint("CreateNewFolder", {"table": LIBRARY, "parameters/path": "@concat('Pending/'," + batch_id + ",'/',outputs('Candidate_run_id_PA37A'))"}, dependency),
        "Each_review_item_PA37A": {
            "type": "Foreach",
            "foreach": "@body('Read_items_PA37A')?['value']",
            "actions": candidate_actions(),
            "runAfter": after("Create_pending_folder_PA37A"),
            "runtimeConfiguration": {"concurrency": {"repetitions": 1}},
        },
        "Manifest_object_PA37A": compose(manifest, "Each_review_item_PA37A"),
        "Manifest_json_PA37A": compose("@string(outputs('Manifest_object_PA37A'))", "Manifest_object_PA37A"),
        "Write_manifest_PA37A": create_file("@concat('" + LIBRARY_PATH + "/Pending/'," + batch_id + ",'/',outputs('Candidate_run_id_PA37A'))", "manifest.json", "@base64ToBinary(base64(outputs('Manifest_json_PA37A')))", "Manifest_json_PA37A"),
        "Read_manifest_PA37A": read_file(manifest_path, "Write_manifest_PA37A"),
        "Exact_manifest_PA37A": gate("@and(greater(length(variables('CandidateRows')),0),equals(add(length(variables('CandidateRows')),length(variables('ExcludedRows'))),int(first(body('Find_batch_PA37A')?['value'])?['SourceCount'])),equals(base64ToString(body('Read_manifest_PA37A')?['$content']),outputs('Manifest_json_PA37A')))", {
            "Respond_prepared_PA37A": {
                "type": "Response", "kind": "PowerApp",
                "inputs": {
                    "schema": {"type": "object", "properties": {"status": {"title": "status", "type": "string", "x-ms-dynamically-added": True}, "manifestpath": {"title": "manifestpath", "type": "string", "x-ms-dynamically-added": True}, "includedcount": {"title": "includedcount", "type": "integer", "x-ms-dynamically-added": True}}, "additionalProperties": {}},
                    "statusCode": 200,
                    "body": {"status": "prepared", "manifestpath": manifest_path, "includedcount": "@length(variables('CandidateRows'))"},
                },
            }
        }, "ManifestReadbackPA37A", "Pending manifest bytes or counts did not reread exactly.", "Read_manifest_PA37A"),
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
    canonical_draft = "if(equals(" + draft_object + "?['entry']?['summary']," + source_object + "?['entry']?['summary'])," + source_text + ",string(setProperty(" + source_object + ",'entry',setProperty(" + source_object + "?['entry'],'summary'," + draft_object + "?['entry']?['summary']))))"
    invalid_draft = "@or(not(equals(item()?['SourceSubmissionId']," + source_object + "?['submissionId'])),not(equals(item()?['ProjectId']," + source_object + "?['projectId'])),and(not(empty(item()?['DraftProjectionBase64A'])),or(not(equals(" + draft_text + "," + canonical_draft + ")),less(length(trim(coalesce(" + draft_object + "?['entry']?['summary'],''))),1),greater(length(trim(coalesce(" + draft_object + "?['entry']?['summary'],''))),1200))))"
    invalid_item = (
        "@or("
        "not(equals(item()?['ValidationState']?['Value'],'Valid')),"
        "not(or(equals(item()?['Decision']?['Value'],'Approve'),"
        "and(equals(item()?['Decision']?['Value'],'Exclude'),"
        "not(empty(trim(coalesce(item()?['DecisionReason'],''))))))),"
        "empty(item()?['SourceSubmissionId']),"
        "empty(item()?['SourcePayloadBase64A']),"
        "empty(item()?['SourceETag']))"
    )
    source_complete = "@and(equals(length(body('Read_source_PA37A')?['value']),int(first(body('Find_batch_PA37A')?['value'])?['SourceCount'])),lessOrEquals(length(body('Read_source_PA37A')?['value']),5000),equals(length(body('Beyond_high_water_PA37A')),0),equals(length(intersection(body('Select_source_receipts_PA37A'),body('Select_review_receipts_PA37A'))),int(first(body('Find_batch_PA37A')?['value'])?['SourceCount'])))"
    actions = {
        "Initialize_mapping_PA37A": variable("ProjectionMap", "Object", {}),
        "Initialize_candidates_PA37A": variable("CandidateRows", "Array", [], "Initialize_mapping_PA37A"),
        "Initialize_exclusions_PA37A": variable("ExcludedRows", "Array", [], "Initialize_candidates_PA37A"),
        "Initialize_used_ids_PA37A": variable("UsedApprovedIds", "Array", [], "Initialize_exclusions_PA37A"),
        "Caller_email_PA37A": compose(caller, "Initialize_used_ids_PA37A"),
        "Read_finalizers_PA37A": sharepoint("HttpRequest", {"parameters/method": "GET", "parameters/uri": "_api/web/sitegroups/getbyid(43)/users", "parameters/headers": {"Accept": "application/json;odata=nometadata"}}, "Caller_email_PA37A"),
        "Caller_membership_PA37A": {
            "type": "Query",
            "inputs": {"from": "@body('Read_finalizers_PA37A')?['value']", "where": "@equals(toLower(item()?['LoginName']),concat('i:0#.f|membership|',outputs('Caller_email_PA37A')))"},
            "runAfter": after("Read_finalizers_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Authorized_PA37A": assertion_gate("@and(equals(length(body('Caller_membership_PA37A')),1),equals(length(triggerBody()?['text']),36),not(contains(triggerBody()?['text'],decodeUriComponent('%27'))))", "ForbiddenPA37A", "Only a current finalizer may prepare a batch.", "Caller_membership_PA37A"),
        "Batch_id_PA37A": compose("@trim(triggerBody()?['text'])", "Authorized_PA37A"),
        "Candidate_run_id_PA37A": compose("@guid()", "Batch_id_PA37A"),
        "Find_batch_PA37A": get_items(BATCHES, batch_filter, "ID asc", 2, "Candidate_run_id_PA37A"),
        "Ready_batch_PA37A": assertion_gate(ready, "BatchNotReadyPA37A", "Batch is unavailable, empty, or not Ready.", "Find_batch_PA37A"),
        "Read_items_PA37A": get_items(ITEMS, item_filter, "SourceItemId asc", 5001, "Ready_batch_PA37A", paged=True),
        "Invalid_items_PA37A": {
            "type": "Query",
            "inputs": {
                "from": "@body('Read_items_PA37A')?['value']",
                "where": invalid_item,
            },
            "runAfter": after("Read_items_PA37A"),
            "runtimeConfiguration": SECURE,
        },
        "Complete_batch_PA37A": assertion_gate(complete, "InvalidBatchPA37A", "Batch membership or decisions are incomplete.", "Invalid_items_PA37A"),
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
        "Valid_drafts_PA37A": assertion_gate("@and(equals(length(body('Invalid_drafts_PA37A')),0),equals(length(body('Select_source_uuids_PA37A')),length(union(body('Select_source_uuids_PA37A'),body('Select_source_uuids_PA37A')))))", "InvalidDraftPA37A", "Draft bytes, immutable identity, or source submission IDs are invalid.", "Select_source_uuids_PA37A"),
        "Bound_source_PA37A": assertion_gate(trusted_source(source_id), "UnboundSourcePA37A", "Batch has no trusted materialization source.", "Valid_drafts_PA37A"),
        "Read_source_PA37A": get_items("@" + source_id, source_filter, "ID asc", 5001, "Bound_source_PA37A", paged=True),
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
        "Exact_source_PA37A": assertion_gate(source_complete, "SourceChangedPA37A", "Source interval, ETags, encoded bytes, or high-water bound no longer match review receipts.", "Select_review_receipts_PA37A"),
    }
    return {**actions, **projection_actions("Exact_source_PA37A")}


def package_actions(template: Path, output: Path, name: str, phase: str, actions: dict, update_existing: bool = False) -> Path:
    validate_actions(actions)
    with ZipFile(template) as source:
        files = {name: source.read(name) for name in source.namelist()}
    definition_path = next(name for name in files if name.endswith("/definition.json"))
    resource_id = definition_path.split("/")[2]
    flow = json.loads(files[definition_path])
    flow_id = str(uuid5(NAMESPACE_URL, phase + "/" + name))
    flow["name"] = flow_id
    flow["id"] = "/providers/Microsoft.Flow/flows/" + flow_id
    flow["properties"]["displayName"] = name
    flow["properties"]["connectionReferences"]["shared_sharepointonline"]["source"] = "Embedded"
    flow["properties"]["definition"]["actions"] = actions
    files[definition_path] = (json.dumps(flow, indent=2) + "\n").encode()
    manifest = json.loads(files["manifest.json"])
    manifest["details"]["displayName"] = name
    manifest["resources"][resource_id]["details"]["displayName"] = name
    manifest["resources"][resource_id]["suggestedCreationType"] = "Update" if update_existing else "New"
    manifest["resources"][resource_id]["creationType"] = "Existing, New, Update" if update_existing else "New"
    files["manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    output.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(output, "w", ZIP_DEFLATED) as package:
        for path in sorted(files):
            package.writestr(path, files[path])
    return output


def validate_actions(actions: dict) -> None:
    """Reject package-shape errors already observed in native imports."""
    response_schemas: dict[int, dict] = {}

    def visit(block: dict, depth: int, inside_loop: bool) -> None:
        if depth > 6:
            raise ValueError("Power Automate action depth exceeds the accepted package shape")
        for name, action in block.items():
            if not set(action.get("runAfter", {})).issubset(block):
                raise ValueError(f"{name}: runAfter references a non-sibling action")
            kind = action["type"]
            if kind in ("Query", "Select"):
                source = action["inputs"]["from"]
                if not isinstance(source, list) and not (isinstance(source, str) and source.startswith("@")):
                    raise ValueError(f"{name}: from must be an array expression")
            if kind == "Compose" and "outputs" in action.get("runtimeConfiguration", {}).get("secureData", {}).get("properties", []):
                raise ValueError(f"{name}: Compose cannot secure outputs")
            if kind == "Terminate" and inside_loop:
                raise ValueError(f"{name}: Terminate cannot run inside Foreach")
            if kind == "Response":
                status = action["inputs"]["statusCode"]
                schema = action["inputs"]["schema"]
                if status in response_schemas and response_schemas[status] != schema:
                    raise ValueError(f"{name}: response schema differs for status {status}")
                response_schemas[status] = schema
            if kind == "OpenApiConnection" and action["inputs"]["host"]["operationId"] == "CreateNewFolder":
                if "parameters/path" not in action["inputs"]["parameters"]:
                    raise ValueError(f"{name}: CreateNewFolder needs parameters/path")
            if "actions" in action:
                visit(action["actions"], depth + 1, inside_loop or kind == "Foreach")
            if "else" in action:
                visit(action["else"].get("actions", {}), depth + 1, inside_loop)

    visit(actions, 1, False)


def build(template: Path, output: Path, update_existing: bool = False) -> Path:
    return package_actions(template, output, "Prepare Approval Finalization TEST ONLY", "PA37A", prepare_actions(), update_existing)


def select_rows(source: str, expression: str, dependency: str) -> dict:
    return {"type": "Select", "inputs": {"from": source, "select": expression},
            "runAfter": after(dependency), "runtimeConfiguration": deepcopy(SECURE)}


def filter_rows(source: str, expression: str, dependency: str | None = None) -> dict:
    return {"type": "Query", "inputs": {"from": source, "where": expression},
            "runAfter": after(dependency) if dependency else {}, "runtimeConfiguration": deepcopy(SECURE)}


def merge_batch(fields: str, etag: str, dependency: str) -> dict:
    return sharepoint("HttpRequest", {
        "parameters/method": "POST",
        "parameters/uri": "@concat('_api/web/lists(guid%27" + BATCHES + "%27)/items(',string(first(body('Find_batch_PA37B')?['value'])?['ID']),')')",
        "parameters/headers": {"Accept": "application/json;odata=nometadata", "Content-Type": "application/json;odata=nometadata", "IF-MATCH": etag, "X-HTTP-Method": "MERGE"},
        "parameters/body": fields,
    }, dependency)


def uuid_is_safe(expression: str) -> str:
    remainder = expression
    for character in "0123456789abcdef-":
        remainder = "replace(" + remainder + ",'" + character + "','')"
    return "and(equals(length(" + expression + "),36),equals(" + remainder + ",''))"


def finalization_row_actions() -> dict:
    row = "items('Each_review_row_PA37B')"
    included = "body('Match_included_PA37B')"
    excluded = "body('Match_excluded_PA37B')"
    chosen = "if(equals(length(" + included + "),1),first(" + included + "),first(" + excluded + "))"
    fields = (
        ("SourceItemId", "sourceItemId"),
        ("SourceSubmissionId", "sourceSubmissionId"),
        ("SourceETag", "sourceETag"),
        ("DraftVersion", "draftVersion"),
        ("DraftProjectionBase64A", "draftProjectionBase64A"),
        ("DraftProjectionBase64B", "draftProjectionBase64B"),
    )
    checks = ["equals(" + row + "?['" + field + "'],outputs('Manifest_row_PA37B')?['" + key + "'])" for field, key in fields]
    checks.extend((
        "equals(" + row + "?['@odata.etag'],outputs('Manifest_row_PA37B')?['reviewETag'])",
        "equals(" + row + "?['Decision']?['Value'],if(equals(length(" + included + "),1),'Approve','Exclude'))",
        "or(equals(length(" + included + "),1),equals(coalesce(" + row + "?['DecisionReason'],''),coalesce(outputs('Manifest_row_PA37B')?['reason'],'')))",
        "or(equals(length(" + excluded + "),1),equals(" + row + "?['ApprovedSubmissionId'],outputs('Manifest_row_PA37B')?['approvedSubmissionId']))",
    ))
    path = "@concat('" + LIBRARY_PATH + "/Pending/',outputs('Batch_id_PA37B'),'/',outputs('Candidate_id_PA37B'),'/',string(" + row + "?['SourceItemId']),'.jsonl')"
    source_text = "base64ToString(concat(" + row + "?['SourcePayloadBase64A'],coalesce(" + row + "?['SourcePayloadBase64B'],'')))"
    draft_text = "if(empty(" + row + "?['DraftProjectionBase64A'])," + source_text + ",base64ToString(concat(" + row + "?['DraftProjectionBase64A'],coalesce(" + row + "?['DraftProjectionBase64B'],''))))"
    revision = "outputs('Source_object_PA37B')?['revision']"
    mapped_root = "outputs('Manifest_object_PA37B')?['mapping']?[" + revision + "?['rootSubmissionId']]"
    mapped_prior = "outputs('Manifest_object_PA37B')?['mapping']?[" + revision + "?['supersedesSubmissionId']]"
    projected = "if(empty(" + revision + "),outputs('Draft_object_PA37B'),setProperty(outputs('Draft_object_PA37B'),'revision',setProperty(setProperty(outputs('Draft_object_PA37B')?['revision'],'rootSubmissionId'," + mapped_root + "),'supersedesSubmissionId'," + mapped_prior + ")))"
    changed = "not(and(equals(outputs('Draft_text_PA37B'),outputs('Source_text_PA37B')),equals(outputs('Projected_object_PA37B')?['revision']," + revision + ")))"
    expected = "if(outputs('Changed_PA37B'),string(setProperty(outputs('Projected_object_PA37B'),'submissionId',outputs('Manifest_row_PA37B')?['approvedSubmissionId'])),outputs('Source_text_PA37B'))"
    projection_checks = [
        "equals(outputs('Source_object_PA37B')?['submissionId']," + row + "?['SourceSubmissionId'])",
        "equals(outputs('Draft_object_PA37B')?['submissionId']," + row + "?['SourceSubmissionId'])",
        "equals(outputs('Manifest_object_PA37B')?['mapping']?[" + row + "?['SourceSubmissionId']],outputs('Manifest_row_PA37B')?['approvedSubmissionId'])",
        "equals(outputs('Manifest_row_PA37B')?['approvedSupersedes'],if(empty(" + revision + "),null," + mapped_prior + "))",
        "or(empty(" + revision + "),and(not(empty(" + mapped_root + ")),not(empty(" + mapped_prior + "))))",
        "equals(outputs('Manifest_row_PA37B')?['payloadBase64'],base64(outputs('Expected_json_PA37B')))",
    ]
    return {
        "Match_included_PA37B": filter_rows("@outputs('Manifest_object_PA37B')?['included']", "@equals(item()?['sourceItemId']," + row + "?['SourceItemId'])"),
        "Match_excluded_PA37B": filter_rows("@outputs('Manifest_object_PA37B')?['excluded']", "@equals(item()?['sourceItemId']," + row + "?['SourceItemId'])", "Match_included_PA37B"),
        "One_manifest_row_PA37B": loop_gate(
            "@equals(add(length(" + included + "),length(" + excluded + ")),1)",
            {"Matched_row_PA37B": compose("@" + chosen)},
            "AmbiguousManifestRowPA37B", "Review item has no unique pending manifest row.", "Match_excluded_PA37B"),
        "Manifest_row_PA37B": compose("@" + chosen, "One_manifest_row_PA37B"),
        "Exact_review_row_PA37B": loop_gate(
            "@and(" + ",".join(checks) + ")",
            {"Review_row_verified_PA37B": compose("@" + row + "?['SourceItemId']")},
            "ReviewChangedPA37B", "Review snapshot changed.", "Manifest_row_PA37B"),
        "Verify_included_PA37B": {
            "type": "If",
            "expression": {"equals": ["@equals(length(" + included + "),1)", True]},
            "actions": {
                "Source_text_PA37B": compose("@" + source_text),
                "Source_object_PA37B": compose("@json(outputs('Source_text_PA37B'))", "Source_text_PA37B"),
                "Draft_text_PA37B": compose("@" + draft_text, "Source_object_PA37B"),
                "Draft_object_PA37B": compose("@json(outputs('Draft_text_PA37B'))", "Draft_text_PA37B"),
                "Projected_object_PA37B": compose("@" + projected, "Draft_object_PA37B"),
                "Changed_PA37B": compose("@" + changed, "Projected_object_PA37B"),
                "Expected_json_PA37B": compose("@" + expected, "Changed_PA37B"),
                "Exact_projection_PA37B": loop_gate("@and(" + ",".join(projection_checks) + ")", {"Projection_verified_PA37B": compose("@" + row + "?['SourceItemId']")}, "ProjectionChangedPA37B", "Candidate projection or mapping differs from immutable source and review draft.", "Expected_json_PA37B"),
                "Read_pending_row_PA37B": read_file(path, "Exact_projection_PA37B"),
                "Exact_pending_row_PA37B": loop_gate(
                    "@equals(body('Read_pending_row_PA37B')?['$content'],outputs('Manifest_row_PA37B')?['payloadBase64'])",
                    {"Remember_pending_bytes_PA37B": append_variable("VerifiedRows", "@outputs('Manifest_row_PA37B')")},
                    "CandidateBytesChangedPA37B", "Pending candidate bytes changed.", "Read_pending_row_PA37B"),
                "Count_payload_bytes_PA37B": increment_variable(
                    "TotalPayloadBytes",
                    "@sub(div(mul(length(outputs('Manifest_row_PA37B')?['payloadBase64']),3),4),if(endsWith(outputs('Manifest_row_PA37B')?['payloadBase64'],'=='),2,if(endsWith(outputs('Manifest_row_PA37B')?['payloadBase64'],'='),1,0)))",
                    "Exact_pending_row_PA37B"),
            },
            "else": {"actions": {"Exclusion_verified_PA37B": compose("@" + row + "?['SourceItemId']")}},
            "runAfter": after("Exact_review_row_PA37B"),
        },
    }


def freeze_row_actions() -> dict:
    row = "items('Each_frozen_row_PA37B')"
    folder = "@concat('" + LIBRARY_PATH + "/Approved/',outputs('Batch_id_PA37B'))"
    path = "@concat('" + LIBRARY_PATH + "/Approved/',outputs('Batch_id_PA37B'),'/',string(" + row + "?['sourceItemId']),'.jsonl')"
    return {
        "Write_frozen_row_PA37B": create_file(folder, "@concat(string(" + row + "?['sourceItemId']),'.jsonl')", "@base64ToBinary(" + row + "?['payloadBase64'])", None),
        "Read_frozen_row_PA37B": read_file(path, "Write_frozen_row_PA37B"),
        "Exact_frozen_row_PA37B": loop_gate(
            "@equals(body('Read_frozen_row_PA37B')?['$content']," + row + "?['payloadBase64'])",
            {"Frozen_row_verified_PA37B": compose("@" + row + "?['sourceItemId']")},
            "FrozenReadbackPA37B", "Approved row did not reread exactly.", "Read_frozen_row_PA37B"),
    }


def object_expression(fields: tuple[tuple[str, str], ...]) -> str:
    expression = "json('{}')"
    for name, value in fields:
        expression = "setProperty(" + expression + ",'" + name + "'," + value + ")"
    return "@" + expression


def finalization_actions() -> dict:
    caller = "@toLower(if(not(empty(triggerOutputs()?['headers']?['x-ms-user-email-encoded'])),base64ToString(triggerOutputs()?['headers']?['x-ms-user-email-encoded']),coalesce(triggerOutputs()?['headers']?['x-ms-user-email'],string(null))))"
    batch = "first(body('Find_batch_PA37B')?['value'])"
    manifest = "outputs('Manifest_object_PA37B')"
    source_id = batch + "?['SourceListId']"
    batch_filter = "@concat('BatchId eq ',decodeUriComponent('%27'),outputs('Batch_id_PA37B'),decodeUriComponent('%27'))"
    item_filter = batch_filter
    pending_path = "@concat('" + LIBRARY_PATH + "/Pending/',outputs('Batch_id_PA37B'),'/',outputs('Candidate_id_PA37B'),'/manifest.json')"
    approved_path = "@concat('" + LIBRARY_PATH + "/Approved/',outputs('Batch_id_PA37B'),'/manifest.json')"
    source_filter = "@concat('Created ge ',decodeUriComponent('%27')," + batch + "?['PeriodStartUtc'],decodeUriComponent('%27'),' and Created lt ',decodeUriComponent('%27')," + batch + "?['PeriodEndUtc'],decodeUriComponent('%27'))"
    source_receipt = "@concat(string(item()?['ID']),'|',item()?['SubmissionId'],'|',item()?['ProjectId'],'|',string(ticks(item()?['Created'])),'|',item()?['@odata.etag'],'|',item()?['PayloadBase64A'],coalesce(item()?['PayloadBase64B'],''))"
    review_receipt = "@concat(string(item()?['SourceItemId']),'|',item()?['SourceSubmissionId'],'|',item()?['ProjectId'],'|',string(ticks(item()?['SourceCreatedUtc'])),'|',item()?['SourceETag'],'|',item()?['SourcePayloadBase64A'],coalesce(item()?['SourcePayloadBase64B'],''))"
    initial_source = "body('Read_source_PA37B')?['value']"
    initial_items = "body('Read_items_PA37B')?['value']"
    exact_source = "@and(equals(length(" + initial_source + "),int(" + batch + "?['SourceCount'])),lessOrEquals(length(" + initial_source + "),5000),equals(length(body('Beyond_high_water_PA37B')),0),equals(length(intersection(body('Source_receipts_PA37B'),body('Review_receipts_PA37B'))),int(" + batch + "?['SourceCount'])))"
    manifest_valid = "@and(equals(" + manifest + "?['batchId'],outputs('Batch_id_PA37B')),equals(" + manifest + "?['candidateRunId'],outputs('Candidate_id_PA37B')),equals(" + manifest + "?['sourceListId']," + source_id + "),equals(int(" + manifest + "?['sourceCount']),int(" + batch + "?['SourceCount'])),greater(length(" + manifest + "?['included']),0),equals(add(length(" + manifest + "?['included']),length(" + manifest + "?['excluded'])),int(" + batch + "?['SourceCount'])),lessOrEquals(int(" + batch + "?['SourceCount']),5000))"
    complete = "@and(equals(length(" + initial_items + "),int(" + batch + "?['SourceCount'])),equals(length(" + initial_items + "),int(" + batch + "?['ItemCount'])),equals(length(body('Invalid_items_PA37B')),0))"
    read_final = {
        "Read_existing_manifest_PA37B": read_file(approved_path),
        "Respond_existing_PA37B": {"type": "Response", "kind": "PowerApp", "inputs": {"schema": {"type": "object", "properties": {"status": {"title": "status", "type": "string", "x-ms-dynamically-added": True}, "manifestjson": {"title": "manifestjson", "type": "string", "x-ms-dynamically-added": True}}, "additionalProperties": {}}, "statusCode": 200, "body": {"status": "approved", "manifestjson": "@base64ToString(body('Read_existing_manifest_PA37B')?['$content'])"}}, "runAfter": after("Read_existing_manifest_PA37B")},
        "Stop_existing_PA37B": {"type": "Terminate", "inputs": {"runStatus": "Succeeded"}, "runAfter": after("Respond_existing_PA37B")},
    }
    digest_base64 = "base64(outputs('Manifest_json_PA37B'))"
    fingerprint = "concat('b64-edge-v1:',string(length(" + digest_base64 + ")),':',substring(" + digest_base64 + ",0,if(greater(length(" + digest_base64 + "),64),64,length(" + digest_base64 + "))),':',substring(" + digest_base64 + ",if(greater(length(" + digest_base64 + "),64),sub(length(" + digest_base64 + "),64),0),if(greater(length(" + digest_base64 + "),64),64,length(" + digest_base64 + "))))"
    commit_body = object_expression((
        ("State", "'Approved'"), ("ApprovedByEmail", "outputs('Caller_email_PA37B')"),
        ("ApprovedAtUtc", "utcNow()"), ("ProjectionDigest", fingerprint),
        ("ApprovedCount", "length(" + manifest + "?['included'])"),
        ("ExcludedCount", "length(" + manifest + "?['excluded'])"),
        ("UnresolvedCount", "0"),
    ))
    actions = {
        "Initialize_verified_PA37B": variable("VerifiedRows", "Array", []),
        "Initialize_total_PA37B": variable("TotalPayloadBytes", "Integer", 0, "Initialize_verified_PA37B"),
        "Caller_email_PA37B": compose(caller, "Initialize_total_PA37B"),
        "Read_finalizers_PA37B": sharepoint("HttpRequest", {"parameters/method": "GET", "parameters/uri": "_api/web/sitegroups/getbyid(43)/users", "parameters/headers": {"Accept": "application/json;odata=nometadata"}}, "Caller_email_PA37B"),
        "Caller_membership_PA37B": filter_rows("@body('Read_finalizers_PA37B')?['value']", "@equals(toLower(item()?['LoginName']),concat('i:0#.f|membership|',outputs('Caller_email_PA37B')))", "Read_finalizers_PA37B"),
        "Authorized_PA37B": assertion_gate("@equals(length(body('Caller_membership_PA37B')),1)", "ForbiddenPA37B", "Only a current finalizer may commit a batch.", "Caller_membership_PA37B"),
        "Request_parts_PA37B": compose("@split(coalesce(triggerBody()?['text'],''),'|')", "Authorized_PA37B"),
        "Valid_request_PA37B": assertion_gate("@equals(length(outputs('Request_parts_PA37B')),2)", "InvalidRequestPA37B", "Expected batchId|candidateRunId.", "Request_parts_PA37B"),
        "Batch_id_PA37B": compose("@toLower(first(outputs('Request_parts_PA37B')))", "Valid_request_PA37B"),
        "Candidate_id_PA37B": compose("@toLower(last(outputs('Request_parts_PA37B')))", "Batch_id_PA37B"),
        "Safe_ids_PA37B": assertion_gate("@and(" + uuid_is_safe("outputs('Batch_id_PA37B')") + "," + uuid_is_safe("outputs('Candidate_id_PA37B')") + ")", "UnsafePathPA37B", "Batch or candidate ID is not a safe UUID.", "Candidate_id_PA37B"),
        "Find_batch_PA37B": get_items(BATCHES, batch_filter, "ID asc", 2, "Safe_ids_PA37B"),
        "Unique_batch_PA37B": assertion_gate("@equals(length(body('Find_batch_PA37B')?['value']),1)", "MissingBatchPA37B", "Batch not found uniquely.", "Find_batch_PA37B"),
        "Already_approved_PA37B": {"type": "If", "expression": {"equals": ["@equals(" + batch + "?['State']?['Value'],'Approved')", True]}, "actions": read_final, "else": {"actions": {"Not_approved_PA37B": compose("@true")}}, "runAfter": after("Unique_batch_PA37B")},
        "Already_finalizing_PA37B": {"type": "If", "expression": {"equals": ["@equals(" + batch + "?['State']?['Value'],'Finalizing')", True]}, "actions": {
            "Respond_in_progress_PA37B": {"type": "Response", "kind": "PowerApp", "inputs": {"schema": {"type": "object", "properties": {"status": {"title": "status", "type": "string", "x-ms-dynamically-added": True}, "manifestjson": {"title": "manifestjson", "type": "string", "x-ms-dynamically-added": True}}, "additionalProperties": {}}, "statusCode": 200, "body": {"status": "in-progress", "manifestjson": ""}}},
            "Stop_in_progress_PA37B": {"type": "Terminate", "inputs": {"runStatus": "Succeeded"}, "runAfter": after("Respond_in_progress_PA37B")},
        }, "else": {"actions": {"Not_finalizing_PA37B": compose("@true")}}, "runAfter": after("Already_approved_PA37B")},
        "Ready_batch_PA37B": assertion_gate("@and(equals(" + batch + "?['State']?['Value'],'Ready'),greater(int(" + batch + "?['SourceCount']),0))", "NotReadyPA37B", "Batch is not complete and Ready.", "Already_finalizing_PA37B"),
        "Bound_source_PA37B": assertion_gate(trusted_source(source_id), "UnboundSourcePA37B", "Batch source list is not trusted.", "Ready_batch_PA37B"),
        "Read_pending_manifest_PA37B": read_file(pending_path, "Bound_source_PA37B"),
        "Manifest_json_PA37B": compose("@base64ToString(body('Read_pending_manifest_PA37B')?['$content'])", "Read_pending_manifest_PA37B"),
        "Manifest_object_PA37B": compose("@json(outputs('Manifest_json_PA37B'))", "Manifest_json_PA37B"),
        "Valid_manifest_PA37B": assertion_gate(manifest_valid, "InvalidManifestPA37B", "Pending manifest identity or counts do not match Ready batch.", "Manifest_object_PA37B"),
        "Invalid_included_PA37B": filter_rows("@" + manifest + "?['included']", "@or(empty(item()?['approvedSubmissionId']),empty(item()?['sourceSubmissionId']),empty(item()?['payloadBase64']),empty(item()?['reviewETag']))", "Valid_manifest_PA37B"),
        "Approved_ids_PA37B": select_rows("@" + manifest + "?['included']", "@item()?['approvedSubmissionId']", "Invalid_included_PA37B"),
        "Unique_approved_ids_PA37B": assertion_gate("@and(equals(length(body('Invalid_included_PA37B')),0),equals(length(body('Approved_ids_PA37B')),length(union(body('Approved_ids_PA37B'),body('Approved_ids_PA37B')))))", "DuplicateApprovalIdPA37B", "Approved identity or payload is missing or duplicated.", "Approved_ids_PA37B"),
        "Read_items_PA37B": get_items(ITEMS, item_filter, "SourceItemId asc", 5001, "Unique_approved_ids_PA37B", paged=True),
        "Invalid_items_PA37B": filter_rows("@" + initial_items, "@or(not(equals(item()?['ValidationState']?['Value'],'Valid')),not(or(equals(item()?['Decision']?['Value'],'Approve'),and(equals(item()?['Decision']?['Value'],'Exclude'),not(empty(trim(coalesce(item()?['DecisionReason'],''))))))),empty(item()?['SourceSubmissionId']),empty(item()?['SourcePayloadBase64A']),empty(item()?['SourceETag']))", "Read_items_PA37B"),
        "Complete_batch_PA37B": assertion_gate(complete, "IncompleteBatchPA37B", "Ready batch counts, decisions, or review items differ.", "Invalid_items_PA37B"),
        "Read_source_PA37B": get_items("@" + source_id, source_filter, "ID asc", 5001, "Complete_batch_PA37B", paged=True),
        "Beyond_high_water_PA37B": filter_rows("@" + initial_source, "@greater(int(item()?['ID']),int(" + batch + "?['SourceHighWaterId']))", "Read_source_PA37B"),
        "Source_receipts_PA37B": select_rows("@" + initial_source, source_receipt, "Beyond_high_water_PA37B"),
        "Review_receipts_PA37B": select_rows("@" + initial_items, review_receipt, "Source_receipts_PA37B"),
        "Exact_source_PA37B": assertion_gate(exact_source, "SourceChangedPA37B", "Source interval, ETags, or bytes changed.", "Review_receipts_PA37B"),
        "Each_review_row_PA37B": {"type": "Foreach", "foreach": "@" + initial_items, "actions": finalization_row_actions(), "runAfter": after("Exact_source_PA37B"), "runtimeConfiguration": {"concurrency": {"repetitions": 1}}},
        "Verified_rows_PA37B": assertion_gate("@equals(length(variables('VerifiedRows')),length(" + manifest + "?['included']))", "UnverifiedCandidatePA37B", "Included candidate files did not all verify.", "Each_review_row_PA37B"),
        "Size_bound_PA37B": assertion_gate("@lessOrEquals(add(variables('TotalPayloadBytes'),sub(length(variables('VerifiedRows')),1)),10485760)", "PayloadTooLargePA37B", "Combined JSONL exceeds 10 MiB.", "Verified_rows_PA37B"),
        "Acquire_lease_PA37B": merge_batch(object_expression((("State", "'Finalizing'"),)), "@" + batch + "?['@odata.etag']", "Size_bound_PA37B"),
        "Read_leased_batch_PA37B": get_items(BATCHES, batch_filter, "ID asc", 2, "Acquire_lease_PA37B"),
        "Lease_owned_PA37B": assertion_gate("@and(equals(length(body('Read_leased_batch_PA37B')?['value']),1),equals(first(body('Read_leased_batch_PA37B')?['value'])?['State']?['Value'],'Finalizing'),empty(first(body('Read_leased_batch_PA37B')?['value'])?['ApprovedAtUtc']))", "LeaseLostPA37B", "Finalization lease could not be confirmed.", "Read_leased_batch_PA37B"),
        "Create_approved_folder_PA37B": sharepoint("CreateNewFolder", {"table": LIBRARY, "parameters/path": "@concat('Approved/',outputs('Batch_id_PA37B'))"}, "Lease_owned_PA37B"),
        "Each_frozen_row_PA37B": {"type": "Foreach", "foreach": "@" + manifest + "?['included']", "actions": freeze_row_actions(), "runAfter": after("Create_approved_folder_PA37B"), "runtimeConfiguration": {"concurrency": {"repetitions": 1}}},
        "Write_frozen_manifest_PA37B": create_file("@concat('" + LIBRARY_PATH + "/Approved/',outputs('Batch_id_PA37B'))", "manifest.json", "@base64ToBinary(base64(outputs('Manifest_json_PA37B')))", "Each_frozen_row_PA37B"),
        "Read_frozen_manifest_PA37B": read_file(approved_path, "Write_frozen_manifest_PA37B"),
        "Exact_frozen_manifest_PA37B": assertion_gate("@equals(base64ToString(body('Read_frozen_manifest_PA37B')?['$content']),outputs('Manifest_json_PA37B'))", "FrozenManifestChangedPA37B", "Frozen manifest did not reread exactly.", "Read_frozen_manifest_PA37B"),
        "Reread_source_PA37B": get_items("@" + source_id, source_filter, "ID asc", 5001, "Exact_frozen_manifest_PA37B", paged=True),
        "Reread_items_PA37B": get_items(ITEMS, item_filter, "SourceItemId asc", 5001, "Reread_source_PA37B", paged=True),
        "Reread_source_receipts_PA37B": select_rows("@body('Reread_source_PA37B')?['value']", source_receipt, "Reread_items_PA37B"),
        "Review_etags_PA37B": select_rows("@" + initial_items, "@concat(string(item()?['ID']),'|',item()?['@odata.etag'])", "Reread_source_receipts_PA37B"),
        "Reread_review_etags_PA37B": select_rows("@body('Reread_items_PA37B')?['value']", "@concat(string(item()?['ID']),'|',item()?['@odata.etag'])", "Review_etags_PA37B"),
        "Still_exact_PA37B": assertion_gate("@and(equals(length(body('Reread_source_receipts_PA37B')),length(body('Source_receipts_PA37B'))),equals(length(intersection(body('Reread_source_receipts_PA37B'),body('Source_receipts_PA37B'))),length(body('Source_receipts_PA37B'))),equals(length(body('Reread_review_etags_PA37B')),length(body('Review_etags_PA37B'))),equals(length(intersection(body('Reread_review_etags_PA37B'),body('Review_etags_PA37B'))),length(body('Review_etags_PA37B'))))", "PostLeaseRacePA37B", "Source or review snapshot changed during freeze.", "Reread_review_etags_PA37B"),
        "Commit_approved_PA37B": merge_batch(commit_body, "@first(body('Read_leased_batch_PA37B')?['value'])?['@odata.etag']", "Still_exact_PA37B"),
        "Read_committed_batch_PA37B": get_items(BATCHES, batch_filter, "ID asc", 2, "Commit_approved_PA37B"),
        "Committed_PA37B": assertion_gate("@and(equals(length(body('Read_committed_batch_PA37B')?['value']),1),equals(first(body('Read_committed_batch_PA37B')?['value'])?['State']?['Value'],'Approved'),equals(toLower(first(body('Read_committed_batch_PA37B')?['value'])?['ApprovedByEmail']),outputs('Caller_email_PA37B')),not(empty(first(body('Read_committed_batch_PA37B')?['value'])?['ApprovedAtUtc'])),equals(first(body('Read_committed_batch_PA37B')?['value'])?['ProjectionDigest']," + fingerprint + "),equals(int(first(body('Read_committed_batch_PA37B')?['value'])?['ApprovedCount']),length(" + manifest + "?['included'])),equals(int(first(body('Read_committed_batch_PA37B')?['value'])?['ExcludedCount']),length(" + manifest + "?['excluded'])),equals(int(first(body('Read_committed_batch_PA37B')?['value'])?['UnresolvedCount']),0))", "CommitUncertainPA37B", "Approved commit could not be verified.", "Read_committed_batch_PA37B"),
        "Respond_approved_PA37B": {"type": "Response", "kind": "PowerApp", "inputs": {"schema": {"type": "object", "properties": {"status": {"title": "status", "type": "string", "x-ms-dynamically-added": True}, "manifestjson": {"title": "manifestjson", "type": "string", "x-ms-dynamically-added": True}}, "additionalProperties": {}}, "statusCode": 200, "body": {"status": "approved", "manifestjson": "@outputs('Manifest_json_PA37B')"}}, "runAfter": after("Committed_PA37B")},
    }
    return actions


def approved_reader_actions() -> dict:
    caller = "@toLower(if(not(empty(triggerOutputs()?['headers']?['x-ms-user-email-encoded'])),base64ToString(triggerOutputs()?['headers']?['x-ms-user-email-encoded']),coalesce(triggerOutputs()?['headers']?['x-ms-user-email'],string(null))))"
    batch_filter = "@concat('BatchId eq ',decodeUriComponent('%27'),outputs('Batch_id_PA37R'),decodeUriComponent('%27'))"
    path = "@concat('" + LIBRARY_PATH + "/Approved/',outputs('Batch_id_PA37R'),'/manifest.json')"
    row = "items('Each_approved_row_PA37R')"
    row_path = "@concat('" + LIBRARY_PATH + "/Approved/',outputs('Batch_id_PA37R'),'/',string(" + row + "?['sourceItemId']),'.jsonl')"
    manifest = "outputs('Approved_manifest_PA37R')"
    return {
        "Initialize_lines_PA37R": variable("FrozenLines", "Array", []),
        "Caller_email_PA37R": compose(caller, "Initialize_lines_PA37R"),
        "Read_exporters_PA37R": sharepoint("HttpRequest", {"parameters/method": "GET", "parameters/uri": "_api/web/sitegroups/getbyid(44)/users", "parameters/headers": {"Accept": "application/json;odata=nometadata"}}, "Caller_email_PA37R"),
        "Read_finalizers_PA37R": sharepoint("HttpRequest", {"parameters/method": "GET", "parameters/uri": "_api/web/sitegroups/getbyid(43)/users", "parameters/headers": {"Accept": "application/json;odata=nometadata"}}, "Read_exporters_PA37R"),
        "Exporter_membership_PA37R": filter_rows("@body('Read_exporters_PA37R')?['value']", "@equals(toLower(item()?['LoginName']),concat('i:0#.f|membership|',outputs('Caller_email_PA37R')))", "Read_finalizers_PA37R"),
        "Finalizer_membership_PA37R": filter_rows("@body('Read_finalizers_PA37R')?['value']", "@equals(toLower(item()?['LoginName']),concat('i:0#.f|membership|',outputs('Caller_email_PA37R')))", "Exporter_membership_PA37R"),
        "Authorized_PA37R": assertion_gate("@greater(add(length(body('Exporter_membership_PA37R')),length(body('Finalizer_membership_PA37R'))),0)", "ForbiddenPA37R", "Current exporter or TEST ONLY finalizer membership is required.", "Finalizer_membership_PA37R"),
        "Batch_id_PA37R": compose("@toLower(trim(coalesce(triggerBody()?['text'],'')))", "Authorized_PA37R"),
        "Safe_id_PA37R": assertion_gate("@" + uuid_is_safe("outputs('Batch_id_PA37R')"), "UnsafeIdPA37R", "Batch ID is not a safe UUID.", "Batch_id_PA37R"),
        "Find_batch_PA37R": get_items(BATCHES, batch_filter, "ID asc", 2, "Safe_id_PA37R"),
        "Approved_only_PA37R": assertion_gate("@and(equals(length(body('Find_batch_PA37R')?['value']),1),equals(first(body('Find_batch_PA37R')?['value'])?['State']?['Value'],'Approved'))", "NotApprovedPA37R", "No Approved projection is available.", "Find_batch_PA37R"),
        "Read_manifest_PA37R": read_file(path, "Approved_only_PA37R"),
        "Manifest_json_PA37R": compose("@base64ToString(body('Read_manifest_PA37R')?['$content'])", "Read_manifest_PA37R"),
        "Approved_manifest_PA37R": compose("@json(outputs('Manifest_json_PA37R'))", "Manifest_json_PA37R"),
        "Valid_manifest_PA37R": assertion_gate("@and(equals(" + manifest + "?['batchId'],outputs('Batch_id_PA37R')),equals(" + manifest + "?['sourceListId'],first(body('Find_batch_PA37R')?['value'])?['SourceListId']),equals(length(" + manifest + "?['included']),int(first(body('Find_batch_PA37R')?['value'])?['ApprovedCount'])),equals(length(" + manifest + "?['excluded']),int(first(body('Find_batch_PA37R')?['value'])?['ExcludedCount'])),greater(length(" + manifest + "?['included']),0),lessOrEquals(add(length(" + manifest + "?['included']),length(" + manifest + "?['excluded'])),5000))", "InvalidApprovedManifestPA37R", "Approved manifest identity or counts are inconsistent.", "Approved_manifest_PA37R"),
        "Each_approved_row_PA37R": {"type": "Foreach", "foreach": "@" + manifest + "?['included']", "actions": {
            "Read_frozen_row_PA37R": read_file(row_path),
            "Exact_frozen_row_PA37R": loop_gate("@equals(body('Read_frozen_row_PA37R')?['$content']," + row + "?['payloadBase64'])", {"Remember_line_PA37R": append_variable("FrozenLines", "@base64ToString(body('Read_frozen_row_PA37R')?['$content'])")}, "FrozenBytesChangedPA37R", "Approved file bytes differ from immutable manifest.", "Read_frozen_row_PA37R"),
        }, "runAfter": after("Valid_manifest_PA37R"), "runtimeConfiguration": {"concurrency": {"repetitions": 1}}},
        "Complete_read_PA37R": assertion_gate("@equals(length(variables('FrozenLines')),length(" + manifest + "?['included']))", "IncompleteApprovedReadPA37R", "Approved files did not all verify.", "Each_approved_row_PA37R"),
        "Respond_approved_PA37R": {"type": "Response", "kind": "PowerApp", "inputs": {"schema": {"type": "object", "properties": {"manifestjson": {"title": "manifestjson", "type": "string", "x-ms-dynamically-added": True}, "jsonl": {"title": "jsonl", "type": "string", "x-ms-dynamically-added": True}}, "additionalProperties": {}}, "statusCode": 200, "body": {"manifestjson": "@outputs('Manifest_json_PA37R')", "jsonl": "@join(variables('FrozenLines'),decodeUriComponent('%0A'))"}}, "runAfter": after("Complete_read_PA37R")},
    }


def build_b(template: Path, output: Path, update_existing: bool = False) -> Path:
    return package_actions(template, output, "Finalize Approval Batch TEST ONLY", "PA37B", finalization_actions(), update_existing)


def build_reader(template: Path, output: Path, update_existing: bool = False) -> Path:
    return package_actions(template, output, "Read Approved Batch TEST ONLY", "PA37R", approved_reader_actions(), update_existing)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--flow", choices=("prepare", "finalize", "reader"), default="prepare")
    parser.add_argument("--update-existing", action="store_true")
    parser.add_argument("template", type=Path)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    builder = {"prepare": build, "finalize": build_b, "reader": build_reader}[arguments.flow]
    print(builder(arguments.template, arguments.output, arguments.update_existing))
