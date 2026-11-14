"""Build a receipt-gated, configurable approved JSONL sync scope."""

import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

from sync_config import validate

ROOT = Path(__file__).resolve().parent
GITLAB = 'shared_acorn-20gitlab-20logbook-5f06923fc38f4e1ce0-5fee5c9710be0fad46'
SHAREPOINT = 'shared_sharepointonline'
CONFIG = "outputs('Sync_configuration_BETA')"
SITE = '@' + CONFIG + "?['sharePointSiteUrl']"
LIBRARY = CONFIG + "?['approvedExportLibrary']"
EXPORT = "body('Read_approved_export_BETA')?['$content']"
RECEIPT = "outputs('Receipt_object_BETA')"
SECURE = {'secureData': {'properties': ['inputs', 'outputs']}}


def action(operation, parameters, dependency=None, *, gitlab=False, statuses=None):
    connector = GITLAB if gitlab else SHAREPOINT
    return {
        'type': 'OpenApiConnection',
        'inputs': {
            'host': {
                'connectionName': connector,
                'operationId': operation,
                'apiId': '/providers/Microsoft.PowerApps/apis/' + connector,
            },
            'parameters': parameters if gitlab else {'dataset': SITE, **parameters},
            'retryPolicy': {'type': 'none'},
        },
        'runAfter': {dependency: statuses or ['Succeeded']} if dependency else {},
        'runtimeConfiguration': SECURE,
    }


def compose(value, dependency=None):
    return {
        'type': 'Compose',
        'inputs': value,
        'runAfter': {dependency: ['Succeeded']} if dependency else {},
        'runtimeConfiguration': SECURE,
    }


def fail(code, message):
    return {
        'type': 'Terminate',
        'inputs': {'runStatus': 'Failed', 'runError': {'code': code, 'message': message}},
        'runAfter': {},
    }


def gate(expression, then, code, message, dependency):
    return {
        'type': 'If',
        'expression': expression,
        'actions': then,
        'else': {'actions': {'Fail_' + code + '_BETA': fail(code, message)}},
        'runAfter': {dependency: ['Succeeded']},
    }


def fingerprint(encoded):
    size = 'length(' + encoded + ')'
    edge = 'if(greater(' + size + ',64),64,' + size + ')'
    tail = 'if(greater(' + size + ',64),sub(' + size + ',64),0)'
    return ("concat('b64-edge-v1:',string(" + size + "),':',substring(" + encoded
            + ',0,' + edge + "),':',substring(" + encoded + ',' + tail + ',' + edge + '))')


def byte_count(encoded):
    return ("sub(div(mul(length(" + encoded + "),3),4),"
            "if(endsWith(" + encoded + ",'=='),2,"
            "if(endsWith(" + encoded + ",'='),1,0)))")


def build(config=None):
    if config is None:
        config = json.loads((ROOT / 'sync-config.json').read_text())
    config = validate(config)
    site_path = 'uriPath(' + CONFIG + "?['sharePointSiteUrl'])"
    relative_library = "concat('/'," + LIBRARY + ')'
    export_path = "concat(" + relative_library + ",'/', 'logbook.jsonl')"
    receipt_path = "concat(" + relative_library + ",'/',first(body('Receipt_files_BETA'))?['Name'])"
    file_identity = "concat(" + site_path + ",'/'," + LIBRARY + ",'/', 'logbook.jsonl')"
    folder_uri = ("@concat('_api/web/GetFolderByServerRelativeUrl(',decodeUriComponent('%27'),"
                  + site_path + ",'/'," + LIBRARY
                  + ",decodeUriComponent('%27'),')/Files?$orderby=TimeCreated desc&$top=5001')")
    metadata_uri = ("@concat('_api/web/GetFileByServerRelativeUrl(',decodeUriComponent('%27'),"
                    + file_identity + ",decodeUriComponent('%27'),')?$select=ETag,UniqueId')")
    gitlab_read = {
        'projectId': '@' + CONFIG + "?['gitLabProjectId']",
        'filePath': '@' + CONFIG + "?['approvedGitLabPath']",
        'ref': '@' + CONFIG + "?['gitLabBranch']",
    }
    commit_common = {
        'projectId': gitlab_read['projectId'],
        'body/branch': gitlab_read['ref'],
        'body/commit_message': "@concat('approved-jsonl-sync:'," + RECEIPT + "?['runId'])",
    }
    base_write = ("setProperty(setProperty(setProperty(setProperty(json('{}'),'file_path',"
                  + CONFIG + "?['approvedGitLabPath']),'content'," + EXPORT
                  + "),'encoding','base64'),'action',")
    create = action('CommitApprovedJsonl', {
        **commit_common,
        'body/actions': "@createArray(" + base_write + "'create'))",
    }, gitlab=True)
    update = action('CommitApprovedJsonl', {
        **commit_common,
        'body/actions': ("@createArray(setProperty(" + base_write
                         + "'update'),'last_commit_id',body('Read_GitLab_before_BETA')?['last_commit_id']))"),
    }, gitlab=True)
    existing_content = ("replace(replace(body('Read_GitLab_before_BETA')?['content'],"
                        "decodeUriComponent('%0A'),''),decodeUriComponent('%0D'),'')")
    after_content = ("replace(replace(body('Read_GitLab_after_BETA')?['content'],"
                     "decodeUriComponent('%0A'),''),decodeUriComponent('%0D'),'')")
    update_if_changed = {
        'type': 'If',
        'expression': {'equals': ['@' + existing_content, '@' + EXPORT]},
        'actions': {},
        'else': {'actions': {'Update_approved_JSONL_BETA': update}},
        'runAfter': {},
    }
    create_or_update = {
        'type': 'If',
        'expression': {'equals': ["@actions('Read_GitLab_before_BETA')?['status']", 'Failed']},
        'actions': {'Create_approved_JSONL_BETA': create},
        'else': {'actions': {'Update_only_if_changed_BETA': update_if_changed}},
        'runAfter': {},
    }
    publish = {
        'Read_GitLab_before_BETA': action('ReadApprovedJsonl', gitlab_read, gitlab=True),
        'GitLab_read_outcome_BETA': {
            'type': 'If',
            'expression': {'or': [
                {'equals': ["@actions('Read_GitLab_before_BETA')?['status']", 'Succeeded']},
                {'and': [
                    {'equals': ["@actions('Read_GitLab_before_BETA')?['status']", 'Failed']},
                    {'equals': ["@outputs('Read_GitLab_before_BETA')?['statusCode']", 404]},
                ]},
            ]},
            'actions': {
                'Create_or_update_GitLab_BETA': create_or_update,
                'Read_GitLab_after_BETA': action('ReadApprovedJsonl', gitlab_read,
                                                'Create_or_update_GitLab_BETA', gitlab=True),
                'Verify_GitLab_bytes_BETA': gate(
                    {'and': [
                        {'equals': ['@' + after_content, '@' + EXPORT]},
                        {'equals': ["@body('Read_GitLab_after_BETA')?['file_path']",
                                    '@' + CONFIG + "?['approvedGitLabPath']"]},
                    ]}, {}, 'GitLabReadbackMismatch',
                    'Approved GitLab file does not match the finalized SharePoint bytes.',
                    'Read_GitLab_after_BETA'),
            },
            'else': {'actions': {
                'Fail_GitLabReadUnavailable_BETA': fail(
                    'GitLabReadUnavailable', 'GitLab read failed without a confirmed missing file.'),
            }},
            'runAfter': {'Read_GitLab_before_BETA': ['Succeeded', 'Failed', 'TimedOut']},
        },
    }
    verified = gate(
        {'and': [
            {'greater': ["@int(" + RECEIPT + "?['batchCount'])", 0]},
            {'greater': ["@int(" + RECEIPT + "?['lineCount'])", 0]},
            {'not': {'equals': ['@' + RECEIPT + "?['lastApprovedBatchId']", None]}},
            {'equals': ['@' + RECEIPT + "?['fileIdentity']", '@' + file_identity]},
            {'equals': ['@' + RECEIPT + "?['candidateFingerprint']",
                        '@' + fingerprint(EXPORT)]},
            {'equals': ["@int(" + RECEIPT + "?['byteCount'])",
                        '@' + byte_count(EXPORT)]},
            {'equals': ['@' + RECEIPT + "?['fileRevision']",
                        "@coalesce(body('Read_export_metadata_BETA')?['ETag'],"
                        "body('Read_export_metadata_BETA')?['@odata.etag'])"]},
        ]}, publish, 'UnverifiedApprovedExport',
        'Receipt, finalized bytes, and SharePoint file revision do not agree.',
        'Read_export_metadata_BETA')
    read_receipt = {
        'Read_receipt_BETA': action('GetFileContentByPath',
                                    {'path': '@' + receipt_path, 'inferContentType': False}),
        'Receipt_object_BETA': compose(
            "@json(base64ToString(body('Read_receipt_BETA')?['$content']))", 'Read_receipt_BETA'),
        'Read_approved_export_BETA': action('GetFileContentByPath',
                                            {'path': '@' + export_path, 'inferContentType': False},
                                            'Receipt_object_BETA'),
        'Read_export_metadata_BETA': action('HttpRequest', {
            'parameters/method': 'GET',
            'parameters/uri': metadata_uri,
            'parameters/headers': {'Accept': 'application/json;odata=nometadata'},
        }, 'Read_approved_export_BETA'),
        'Verify_approved_receipt_BETA': verified,
    }
    actions = {
        'Sync_configuration_BETA': compose(config),
        'Validate_sync_configuration_BETA': gate(
            {'and': [
                {'equals': ['@' + CONFIG + "?['approvedSourceReady']", True]},
                {'equals': ['@' + CONFIG + "?['gitLabWriteReady']", True]},
                {'greater': ["@int(" + CONFIG + "?['gitLabProjectId'])", 0]},
                {'not': {'equals': ['@' + CONFIG + "?['gitLabBranch']", '']}},
                {'startsWith': ['@' + CONFIG + "?['approvedGitLabPath']", '.acorn/approved/']},
                {'endsWith': ['@' + CONFIG + "?['approvedGitLabPath']", '.jsonl']},
            ]}, {}, 'ApprovedSyncNotReady',
            'Approval source and dedicated GitLab destination must be verified before publishing.',
            'Sync_configuration_BETA'),
        'List_export_files_BETA': action('HttpRequest', {
            'parameters/method': 'GET',
            'parameters/uri': folder_uri,
            'parameters/headers': {'Accept': 'application/json;odata=nometadata'},
        }, 'Validate_sync_configuration_BETA'),
        'Receipt_files_BETA': {
            'type': 'Query',
            'inputs': {
                'from': "@body('List_export_files_BETA')?['value']",
                'where': "@and(startsWith(item()?['Name'],'export-receipt-'),endsWith(item()?['Name'],'.json'))",
            },
            'runAfter': {'List_export_files_BETA': ['Succeeded']},
        },
        'Find_latest_receipt_BETA': gate(
            {'and': [
                {'greater': ["@length(body('Receipt_files_BETA'))", 0]},
                {'lessOrEquals': ["@length(body('List_export_files_BETA')?['value'])", 5000]},
            ]}, read_receipt, 'NoFinalizedApprovedReceipt',
            'No bounded finalized approved export receipt was found.', 'Receipt_files_BETA'),
    }
    return {
        'id': str(uuid5(NAMESPACE_URL, 'ACORN/BETA/ApprovedJsonlGitLabSync')),
        'brandColor': '#8C6CFF',
        'connectionReferences': {},
        'connectorDisplayName': 'Control',
        'isTrigger': False,
        'operationName': 'Approved_JSONL_GitLab_sync_BETA',
        'operationDefinition': {'type': 'Scope', 'actions': actions, 'runAfter': {}},
    }


if __name__ == '__main__':
    (ROOT / 'approved-sync.clipboard.json').write_text(json.dumps(build(), indent=2) + '\n')
