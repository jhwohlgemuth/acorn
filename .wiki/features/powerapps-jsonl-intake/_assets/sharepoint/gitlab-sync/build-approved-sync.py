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
READ_RETRY = {'type': 'exponential', 'count': 3, 'interval': 'PT5S', 'maximumInterval': 'PT30S'}
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
            'retryPolicy': READ_RETRY if operation != 'CommitApprovedJsonl' else {'type': 'none'},
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
    # WDL startsWith ignores case; compare the re-encoded prefix for exact bytes.
    publish = {
        'Read_GitLab_before_BETA': action('ReadApprovedJsonl', gitlab_read, gitlab=True),
        'GitLab_read_outcome_BETA': {
            'type': 'If',
            'expression': {'and': [
                {'equals': ["@actions('Read_GitLab_before_BETA')?['status']", 'Succeeded']},
                {'equals': ["@body('Read_GitLab_before_BETA')?['encoding']", 'base64']},
                {'equals': ["@body('Read_GitLab_before_BETA')?['file_path']", '@' + CONFIG + "?['approvedGitLabPath']"]},
                {'not': {'equals': ["@body('Read_GitLab_before_BETA')?['content']", None]}},
                {'not': {'equals': ["@coalesce(body('Read_GitLab_before_BETA')?['last_commit_id'],'')", '']}},
                {'equals': ['@base64(take(base64ToString(' + EXPORT + '),length(base64ToString(' + existing_content + '))))',
                            '@' + existing_content]},
                {'or': [
                    {'equals': ['@' + existing_content, '']},
                    {'equals': ["@endsWith(base64ToString(" + existing_content + "),decodeUriComponent('%0A'))", True]},
                ]},
            ]},
            'actions': {
                'Create_or_update_GitLab_BETA': update_if_changed,
                'Read_GitLab_after_BETA': action('ReadApprovedJsonl', gitlab_read,
                                                'Create_or_update_GitLab_BETA', gitlab=True,
                                                statuses=['Succeeded', 'Failed', 'TimedOut']),
                'Verify_GitLab_bytes_BETA': gate(
                    {'and': [
                        {'equals': ["@actions('Read_GitLab_after_BETA')?['status']", 'Succeeded']},
                        {'equals': ["@body('Read_GitLab_after_BETA')?['encoding']", 'base64']},
                        {'not': {'equals': ["@coalesce(body('Read_GitLab_after_BETA')?['last_commit_id'],'')", '']}},
                        {'equals': ['@' + after_content, '@' + EXPORT]},
                        {'equals': ["@body('Read_GitLab_after_BETA')?['file_path']",
                                    '@' + CONFIG + "?['approvedGitLabPath']"]},
                    ]}, {}, 'GitLabReadbackMismatch',
                    'Approved GitLab file does not match the finalized SharePoint bytes.',
                    'Read_GitLab_after_BETA'),
            },
            'else': {'actions': {
                'Fail_GitLabReadUnavailable_BETA': fail(
                    'GitLabReadUnavailable', 'An existing dedicated GitLab file with readable revision and an exact complete-line prefix of the approved snapshot is required. No write was attempted.'),
            }},
            'runAfter': {'Read_GitLab_before_BETA': ['Succeeded', 'Failed', 'TimedOut']},
        },
    }
    publish['GitLab_read_outcome_BETA']['actions']['Verify_GitLab_bytes_BETA']['runAfter'] = {
        'Read_GitLab_after_BETA': ['Succeeded', 'Failed', 'TimedOut', 'Skipped'],
    }
    metadata_before = "body('Read_export_metadata_before_BETA')"
    metadata_after = "body('Read_export_metadata_BETA')"
    before_etag = "coalesce(" + metadata_before + "?['ETag']," + metadata_before + "?['@odata.etag'])"
    after_etag = "coalesce(" + metadata_after + "?['ETag']," + metadata_after + "?['@odata.etag'])"
    verified = gate(
        {'and': [
            {'greater': ["@int(" + RECEIPT + "?['batchCount'])", 0]},
            {'greater': ["@int(" + RECEIPT + "?['lineCount'])", 0]},
            {'not': {'equals': ['@trim(' + RECEIPT + "?['runId'])", '']}},
            {'not': {'equals': ['@trim(' + RECEIPT + "?['lastApprovedBatchId'])", '']}},
            {'not': {'equals': ['@' + RECEIPT + "?['lastApprovedBatchId']", None]}},
            {'equals': ['@' + RECEIPT + "?['fileIdentity']", '@' + file_identity]},
            {'equals': ['@' + RECEIPT + "?['candidateFingerprint']",
                        '@' + fingerprint(EXPORT)]},
            {'equals': ["@int(" + RECEIPT + "?['byteCount'])",
                        '@' + byte_count(EXPORT)]},
            {'equals': ['@' + RECEIPT + "?['fileRevision']",
                        '@' + after_etag]},
            {'not': {'equals': ['@coalesce(' + before_etag + ",'')", '']}},
            {'equals': ['@' + before_etag, '@' + after_etag]},
            {'not': {'equals': ['@coalesce(' + metadata_before + "?['UniqueId'],'')", '']}},
            {'equals': ['@' + metadata_before + "?['UniqueId']", '@' + metadata_after + "?['UniqueId']"]},
            {'lessOrEquals': ['@' + byte_count(EXPORT), 10485760]},
            {'equals': ["@endsWith(base64ToString(" + EXPORT + "),decodeUriComponent('%0A'))", True]},
            {'equals': ["@contains(base64ToString(" + EXPORT + "),decodeUriComponent('%0D'))", False]},
            {'equals': ["@contains(base64ToString(" + EXPORT + "),decodeUriComponent('%0A%0A'))", False]},
            {'equals': ["@startsWith(base64ToString(" + EXPORT + "),decodeUriComponent('%0A'))", False]},
            {'equals': ["@sub(length(split(base64ToString(" + EXPORT + "),decodeUriComponent('%0A'))),1)", '@' + RECEIPT + "?['lineCount']"]},
            {'lessOrEquals': ["@ticks(" + RECEIPT + "?['verifiedAtUtc'])", '@ticks(utcNow())']},
            {'greaterOrEquals': ["@ticks(" + RECEIPT + "?['verifiedAtUtc'])",
                                 "@ticks(addHours(utcNow(),sub(0," + CONFIG + "?['maxReceiptAgeHours'])))"]},
        ]}, publish, 'UnverifiedApprovedExport',
        'Receipt is stale or receipt, finalized bytes, and stable SharePoint file revision do not agree.',
        'Read_export_metadata_BETA')
    receipt_fields = {
        key: {'type': 'integer' if key in ('batchCount', 'byteCount', 'lineCount') else 'string'}
        for key in ('batchCount', 'byteCount', 'candidateFingerprint', 'fileIdentity',
                    'fileRevision', 'lastApprovedBatchId', 'lineCount', 'runId', 'verifiedAtUtc')
    }
    read_receipt = {
        'Read_receipt_BETA': action('GetFileContentByPath',
                                    {'path': '@' + receipt_path, 'inferContentType': False}),
        'Receipt_object_BETA': compose(
            "@json(base64ToString(body('Read_receipt_BETA')?['$content']))", 'Read_receipt_BETA'),
        'Validate_receipt_shape_BETA': {
            'type': 'ParseJson',
            'inputs': {'content': '@' + RECEIPT, 'schema': {
                'type': 'object', 'properties': receipt_fields, 'required': list(receipt_fields),
            }},
            'runAfter': {'Receipt_object_BETA': ['Succeeded']},
            'runtimeConfiguration': SECURE,
        },
        'Read_export_metadata_before_BETA': action('HttpRequest', {
            'parameters/method': 'GET',
            'parameters/uri': metadata_uri,
            'parameters/headers': {'Accept': 'application/json;odata=nometadata'},
        }, 'Validate_receipt_shape_BETA'),
        'Read_approved_export_BETA': action('GetFileContentByPath',
                                            {'path': '@' + export_path, 'inferContentType': False},
                                            'Read_export_metadata_before_BETA'),
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
                {'equals': ["@empty(coalesce(body('List_export_files_BETA')?['@odata.nextLink'],body('List_export_files_BETA')?['odata.nextLink'],body('List_export_files_BETA')?['d']?['__next']))", True]},
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
