"""Build isolated, owner-checked history/correction designer actions.

The receipt list MUST enforce unique values on nullable CorrectionParentId. These actions use TEST ONLY storage and omit
notifications. Promotion requires accepted live tests and explicit destination review.
"""
import copy
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent
SITE = 'https://ornl.sharepoint.com/sites/ResearchEnablement'
LIST = '31a7f178-6729-4252-a2bc-e06a07838f5c'
LIBRARY = 'PA19 TEST ONLY JSONL'
ALL = ['Succeeded', 'Failed', 'Skipped', 'TimedOut']
SECURE = {'secureData': {'properties': ['inputs', 'outputs']}}
source = json.loads((BASE / 'SharePoint_intake_SP.clipboard.json').read_text())
refs = source['connectionReferences']


def after(name, statuses=None):
    return {name: statuses or ['Succeeded']} if name else {}


def compose(value, previous=None):
    return {'type': 'Compose', 'inputs': value, 'runAfter': after(previous), 'runtimeConfiguration': {'secureData': {'properties': ['inputs']}}}


def condition(expression, actions, otherwise=None, previous=None):
    return {'type': 'If', 'expression': expression, 'actions': actions, 'else': {'actions': otherwise or {}}, 'runAfter': after(previous)}


def query(rows, where, previous):
    return {'type': 'Query', 'inputs': {'from': rows, 'where': where}, 'runAfter': after(previous), 'runtimeConfiguration': SECURE}


def get_items(table, filter_expression, previous=None):
    return {'type': 'OpenApiConnection', 'inputs': {'host': {
        'connectionName': 'shared_sharepointonline', 'operationId': 'GetItems',
        'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline'},
        'parameters': {'dataset': SITE, 'table': table, '$filter': filter_expression, '$orderby': 'ID asc', '$top': 1001},
        'authentication': "@parameters('$authentication')", 'retryPolicy': {'type': 'none'}},
        'runAfter': after(previous), 'runtimeConfiguration': {**SECURE, 'paginationPolicy': {'minimumItemCount': 1001}}}


def clipboard(name, definition):
    definition = json.loads(json.dumps(definition).replace("''", 'string(null)'))
    return {'id': str(uuid5(NAMESPACE_URL, SITE + '/' + name)), 'brandColor': '#8C6CFF',
            'connectionReferences': refs, 'connectorDisplayName': 'Control', 'isTrigger': False,
            'operationName': name, 'operationDefinition': definition}


# Reuse the accepted immutable receipt and exact-readback persistence actions.
intake = copy.deepcopy(source['operationDefinition'])
intake = json.loads(json.dumps(intake).replace('a904a6c6-5dd0-427b-9575-8167c4c0c412', LIST)
                    .replace('Sponsor Highlights JSONL Pilot', LIBRARY)
                    .replace('Sponsor%20Highlights%20JSONL%20Pilot', LIBRARY.replace(' ', '%20')))
def correction_fields(value):
    if isinstance(value, dict):
        inputs = value.get('inputs', {})
        if isinstance(inputs, dict) and inputs.get('host', {}).get('operationId') in ['PostItem', 'PatchItem']:
            value['inputs']['parameters']['item/CorrectionParentId'] = "@body('Parse_submission_SP')?['revision']?['supersedesSubmissionId']"
        for child in value.values():
            correction_fields(child)
    elif isinstance(value, list):
        for child in value:
            correction_fields(child)

correction_fields(intake)
intake['runAfter'] = {}
a = intake['actions']
schema = a['Parse_submission_SP']['inputs']['schema']
schema['properties']['revision'] = {'type': 'object', 'additionalProperties': False,
    'required': ['rootSubmissionId', 'supersedesSubmissionId'], 'properties': {
        name: {'type': 'string', 'minLength': 36, 'maxLength': 36}
        for name in ['rootSubmissionId', 'supersedesSubmissionId']}}
schema['required'].append('revision')
# Existing project and placement were validated when the original was admitted.
# A retired catalog item may still be corrected without rerouting it.
a.pop('Matching_projects_SP')
guard = a['Envelope_allowed_SP']
guard['expression']['and'] = guard['expression']['and'][:-2]
parent_id = "body('Parse_submission_SP')?['revision']?['supersedesSubmissionId']"
new_id = "body('Parse_submission_SP')?['submissionId']"
root_id = "body('Parse_submission_SP')?['revision']?['rootSubmissionId']"
parent = "outputs('Parent_record_EDIT')"
a['Find_parent_EDIT'] = query("@body('Decoded_history_EDIT')", f"@equals(item()?['receipt']?['submissionId'],{parent_id})", 'Caller_email_SP')
a['Find_replay_EDIT'] = query("@body('Decoded_history_EDIT')", f"@equals(item()?['receipt']?['submissionId'],{new_id})", 'Find_parent_EDIT')
a['Find_successors_EDIT'] = query("@body('Decoded_history_EDIT')", f"@equals(item()?['receipt']?['revision']?['supersedesSubmissionId'],{parent_id})", 'Find_replay_EDIT')
a.pop('Envelope_allowed_SP')
guard['runAfter'] = {}
guard['expression']['and'] += [
    {'equals': [f'@{parent}?[\'projectId\']', "@body('Parse_submission_SP')?['projectId']"]},
    {'equals': [f"@coalesce({parent}?['entry']?['global'],false)", "@coalesce(body('Parse_submission_SP')?['entry']?['global'],false)"]},
    {'equals': [f"@coalesce({parent}?['entry']?['identifier'],string(null))", "@coalesce(body('Parse_submission_SP')?['entry']?['identifier'],string(null))"]},
    {'equals': [f'@{root_id}', f"@coalesce({parent}?['revision']?['rootSubmissionId'],{parent}?['submissionId'])"]},
    {'not': {'equals': [f'@{new_id}', f'@{parent_id}']}},
    {'not': {'equals': [f'@{new_id}', f'@{root_id}']}},
    {'equals': [f"@coalesce({parent}?['entry']?['archived'],false)", False]},
    {'greaterOrEquals': ["@ticks(body('Parse_submission_SP')?['submittedAt'])", f"@ticks(coalesce({parent}?['submittedAt'],'1970-01-01T00:00:00Z'))"]},
]
# Replay is reconciled by the accepted exact-byte persistence before stale-head rejection.
# SharePoint enforces uniqueness of CorrectionParentId atomically at receipt creation.
parent_exists = {'equals': ["@length(body('Find_parent_EDIT'))", 1]}
current_or_replay = {'or': [
    {'equals': ["@length(body('Find_replay_EDIT'))", 1]},
    {'equals': ["@length(body('Find_successors_EDIT'))", 0]},
]}
a['Parent_record_EDIT'] = compose("@if(equals(length(body('Find_parent_EDIT')),1),first(body('Find_parent_EDIT'))?['receipt'],json('{}'))", 'Find_successors_EDIT')
a['Stale_revision_EDIT'] = compose("@and(equals(length(body('Find_replay_EDIT')),0),greater(length(body('Find_successors_EDIT')),0))", 'Parent_record_EDIT')
guard['expression']['and'] += [parent_exists, current_or_replay]
guard['runAfter'] = after('Stale_revision_EDIT')
a['Envelope_allowed_SP'] = guard
guard['expression']['and'].append({'equals': ["@first(body('Find_parent_EDIT'))?['exportStatus']", 'Exported']})

# Test-only originals exercise the unchanged receipt writer in isolated storage.
# Remove this mode and its response branch before promoting the correction flow.
seed = copy.deepcopy(source['operationDefinition'])
seed = json.loads(json.dumps(seed).replace('a904a6c6-5dd0-427b-9575-8167c4c0c412', LIST)
                  .replace('Sponsor Highlights JSONL Pilot', LIBRARY)
                  .replace('Sponsor%20Highlights%20JSONL%20Pilot', LIBRARY.replace(' ', '%20')))
seed['runAfter'] = {}
seed['actions'].pop('Matching_projects_SP')
seed_guard = seed['actions']['Envelope_allowed_SP']
seed_guard['runAfter'] = after('Caller_email_SP')
seed_guard['expression']['and'] = seed_guard['expression']['and'][:-2] + [
    {'equals': ["@body('Parse_submission_SP')?['projectId']", 'acorn']},
    {'equals': ["@startsWith(body('Parse_submission_SP')?['entry']?['summary'],'TEST ONLY')", True]},
]
seed = json.loads(json.dumps(seed).replace('_SP', '_SEED'))

caller = source['operationDefinition']['actions']['Caller_email_SP']['inputs']
raw = "base64ToString(concat(item()?['PayloadBase64A'],coalesce(item()?['PayloadBase64B'],string(null))))"
decoded = {'type': 'Select', 'inputs': {'from': "@body('Read_my_receipts_EDIT')?['value']", 'select': {
    'receipt': f'@json({raw})', 'raw': f'@{raw}', 'correctionParentId': "@item()?['CorrectionParentId']", 'id': "@item()?['SubmissionId']",
    'projectId': "@item()?['ProjectId']", 'exportStatus': "@item()?['ExportStatus']"}},
    'runAfter': {}, 'runtimeConfiguration': SECURE}
invalid = query("@body('Decoded_history_EDIT')", "@or(not(equals(toLower(item()?['receipt']?['submitter']?['email']),outputs('Caller_EDIT'))),not(equals(item()?['receipt']?['submissionId'],item()?['id'])),not(equals(item()?['receipt']?['projectId'],item()?['projectId'])),contains(item()?['raw'],decodeUriComponent('%0A')),contains(item()?['raw'],decodeUriComponent('%0D')),not(equals(coalesce(item()?['receipt']?['revision']?['supersedesSubmissionId'],string(null)),coalesce(item()?['correctionParentId'],string(null)))))", 'Decoded_history_EDIT')
valid_history = condition({'and': [
    {'equals': ["@length(body('Invalid_history_EDIT'))", 0]},
    {'lessOrEquals': ["@length(base64(string(body('Decoded_history_EDIT'))))", 6990504]},
]}, {'History_checked_EDIT': compose(True)}, {'History_invalid_EDIT': compose('unresolved')}, 'Invalid_history_EDIT')
bounded = condition({'and': [
    {'not': {'equals': ["@outputs('Caller_EDIT')", '']}},
    {'lessOrEquals': ["@length(body('Read_my_receipts_EDIT')?['value'])", 1000]},
]}, {'Decoded_history_EDIT': decoded, 'Invalid_history_EDIT': invalid, 'Valid_history_EDIT': valid_history},
    {'History_limit_EDIT': compose('unresolved')}, 'Read_my_receipts_EDIT')
read = get_items(LIST, "@concat('SubmitterEmail eq ',decodeUriComponent('%27'),replace(outputs('Caller_EDIT'),decodeUriComponent('%27'),concat(decodeUriComponent('%27'),decodeUriComponent('%27'))),decodeUriComponent('%27'))", 'Caller_EDIT')
checked = {'equals': ["@actions('History_checked_EDIT')?['status']", 'Succeeded']}
list_mode = condition({'and': [checked, {'equals': ["@triggerBody()?['text']", 'list']}]},
    {'History_result_EDIT': compose("@string(body('Decoded_history_EDIT'))")}, previous='Bounded_history_EDIT')
correction_mode = condition({'and': [checked, {'not': {'equals': ["@triggerBody()?['text']", 'list']}}]},
    {'Is_correction_EDIT': condition({'equals': ["@empty(json(triggerBody()?['text'])?['revision'])", False]},
        {'SharePoint_intake_SP': intake}, {'Test_original_SEED': seed})}, previous='List_mode_EDIT')
schema_check = {'type': 'OpenApiConnection', 'inputs': {'host': {
    'connectionName': 'shared_sharepointonline', 'operationId': 'HttpRequest',
    'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline'},
    'parameters': {'dataset': SITE, 'parameters/method': 'GET',
        'parameters/uri': f"_api/web/lists(guid'{LIST}')/fields/getbyinternalnameortitle('CorrectionParentId')?$select=EnforceUniqueValues,Indexed,Required,TypeAsString",
        'parameters/headers': {'Accept': 'application/json;odata=nometadata'}},
    'authentication': "@parameters('$authentication')", 'retryPolicy': {'type': 'none'}},
    'runAfter': after('Caller_EDIT')}
read['runAfter'] = after('Unique_predecessor_schema_EDIT')
bounded['expression']['and'] += [
    {'equals': ["@body('Unique_predecessor_schema_EDIT')?['EnforceUniqueValues']", True]},
    {'equals': ["@body('Unique_predecessor_schema_EDIT')?['Indexed']", True]},
    {'equals': ["@body('Unique_predecessor_schema_EDIT')?['Required']", False]},
    {'equals': ["@body('Unique_predecessor_schema_EDIT')?['TypeAsString']", 'Text']},
]
scope = {'type': 'Scope', 'runAfter': {}, 'actions': {
    'Caller_EDIT': compose(caller), 'Unique_predecessor_schema_EDIT': schema_check, 'Read_my_receipts_EDIT': read, 'Bounded_history_EDIT': bounded,
    'List_mode_EDIT': list_mode, 'Correction_mode_EDIT': correction_mode,
}}
response = copy.deepcopy(json.loads((BASE / 'SharePoint_response_SP.clipboard.json').read_text())['operationDefinition'])
response = json.loads(json.dumps(response).replace('Sponsor%20Highlights%20JSONL%20Pilot', LIBRARY.replace(' ', '%20'))
    .replace('Sponsor%20Highlights%20Intake%20Pilot', 'PA31%20TEST%20ONLY%20Receipts'))
response['runAfter'] = after('Submission_editing_EDIT', ALL)
body = response['inputs']['body']
for key, expression in list(body.items()):
    if isinstance(expression, str) and expression.startswith('@{'):
        body[key] = "@{if(equals(actions('Parse_submission_SEED')?['status'],'Succeeded')," + expression[2:-1].replace('_SP', '_SEED') + ',' + expression[2:-1] + ')}'
body['status'] = "@{if(equals(actions('History_result_EDIT')?['status'],'Succeeded'),'listed',if(equals(outputs('Stale_revision_EDIT'),true),'conflict'," + body['status'][2:-1] + '))}'
body['submissions'] = "@{if(equals(actions('History_result_EDIT')?['status'],'Succeeded'),outputs('History_result_EDIT'),'[]')}"
response['inputs']['schema']['properties']['submissions'] = {'type': 'string', 'title': 'submissions', 'x-ms-dynamically-added': True}
body['reason'] = "@{if(equals(outputs('Stale_revision_EDIT'),true),'stale_revision',string(null))}"
response['inputs']['schema']['properties']['reason'] = {'type': 'string', 'title': 'reason', 'x-ms-dynamically-added': True}
for name, definition in [('Submission_editing_EDIT', scope), ('Submission_editing_response_EDIT', response)]:
    (ROOT / f'{name}.clipboard.json').write_text(json.dumps(clipboard(name, definition), indent=2) + '\n')
print('Built isolated editing/history scope and response. Unique CorrectionParentId is required.')

# Production variant admits corrections only. New submissions keep their existing flow.
production_scope = copy.deepcopy(scope)
production_scope['actions']['Correction_mode_EDIT']['actions'] = {'SharePoint_intake_SP': intake}
production_response = copy.deepcopy(json.loads((BASE / 'SharePoint_response_SP.clipboard.json').read_text())['operationDefinition'])
production_response['runAfter'] = after('Submission_editing_EDIT', ALL)
production_body = production_response['inputs']['body']
production_body['status'] = "@{if(equals(actions('History_result_EDIT')?['status'],'Succeeded'),'listed',if(equals(outputs('Stale_revision_EDIT'),true),'conflict'," + production_body['status'][2:-1] + '))}'
for key in ['submissions', 'reason']:
    production_body[key] = body[key]
    production_response['inputs']['schema']['properties'][key] = response['inputs']['schema']['properties'][key]
for name, definition in [('Submission_editing_EDIT', production_scope), ('Submission_editing_response_EDIT', production_response)]:
    artifact = clipboard('SharePoint_response_SP' if name == 'Submission_editing_response_EDIT' else name, definition)
    serialized = json.dumps(artifact, indent=2)
    serialized = serialized.replace(LIST, 'a904a6c6-5dd0-427b-9575-8167c4c0c412').replace(LIBRARY, 'Sponsor Highlights JSONL Pilot').replace(LIBRARY.replace(' ', '%20'), 'Sponsor%20Highlights%20JSONL%20Pilot')
    assert '_SEED' not in serialized and 'TEST ONLY' not in serialized
    (ROOT / f'production-{name}.clipboard.json').write_text(serialized + '\n')
