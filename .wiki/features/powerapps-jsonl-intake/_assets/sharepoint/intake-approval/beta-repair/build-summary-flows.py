"""Build separate background AI and standard reviewer request native packages."""
import copy
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5
from zipfile import ZIP_DEFLATED, ZipFile

ROOT = Path(__file__).resolve().parent
SITE = 'https://ornl.sharepoint.com/sites/ResearchEnablement'
QUEUE = '442cdec0-52b9-4f24-97b6-092961312c41'
SECURE = {'secureData': {'properties': ['inputs', 'outputs']}}
SECURE_INPUTS = {'secureData': {'properties': ['inputs']}}
ALL = ['Succeeded', 'Failed', 'Skipped', 'TimedOut']

def read_package(path):
    with ZipFile(path) as archive:
        files = {name: archive.read(name) for name in archive.namelist()}
    path = next(name for name in files if name.endswith('/definition.json'))
    return files, path, json.loads(files[path])

def after(name):
    return {name: ['Succeeded']}

def compose(value, dependency=None):
    return {'type': 'Compose', 'inputs': value, 'runAfter': after(dependency) if dependency else {}, 'runtimeConfiguration': {'secureData': {'properties': ['inputs']}}}

def sp(operation, parameters, dependency=None):
    return {'type': 'OpenApiConnection', 'inputs': {'host': {'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline', 'connectionName': 'shared_sharepointonline', 'operationId': operation}, 'parameters': {'dataset': SITE, **parameters}, 'authentication': "@parameters('$authentication')"}, 'runAfter': after(dependency) if dependency else {}, 'runtimeConfiguration': copy.deepcopy(SECURE)}

def condition(expression, actions, dependency=None):
    return {'type': 'If', 'expression': {'equals': [expression, True]}, 'actions': actions, 'else': {'actions': {}}, 'runAfter': after(dependency) if dependency else {}}

def patch_response(value, dependency=None):
    return sp('PatchItem', {'table': QUEUE, 'id': "@triggerBody()?['ID']", 'item/Title': "@triggerBody()?['Title']", 'item/ResponseJson': value}, dependency)

def background_actions(prompt):
    prompt = copy.deepcopy(prompt)
    prompt['inputs']['parameters']['item/requestv2/text'] = "@body('Parse_request')?['content']"
    prompt['runAfter'] = {}
    prompt['inputs']['retryPolicy'] = {'type': 'none'}
    ready = {'status': 'ready', 'summary': "@trim(coalesce(outputs('Run_a_prompt')?['body/responsev2/predictionOutput/text'],''))"}
    generation = {'Run_a_prompt': prompt,
        'Valid_summary': condition("@and(greater(length(trim(coalesce(outputs('Run_a_prompt')?['body/responsev2/predictionOutput/text'],''))),0),lessOrEquals(length(trim(coalesce(outputs('Run_a_prompt')?['body/responsev2/predictionOutput/text'],''))),1200))", {'Ready_response': compose(ready), 'Save_summary': patch_response("@string(outputs('Ready_response'))", 'Ready_response')}, 'Run_a_prompt')}
    generation['Valid_summary']['else']['actions']['Save_invalid_result'] = patch_response('{"status":"error","summary":""}')
    generation['Save_failure'] = patch_response('{"status":"error","summary":""}')
    generation['Save_failure']['runAfter'] = {'Run_a_prompt': ['Failed', 'TimedOut']}
    schema = {'type': 'object', 'additionalProperties': False, 'required': ['content', 'owner', 'requestId'], 'properties': {'content': {'type': 'string', 'minLength': 1, 'maxLength': 12000}, 'owner': {'type': 'string', 'minLength': 1}, 'requestId': {'type': 'string', 'minLength': 36, 'maxLength': 36}}}
    return {'Parse_request': {'type': 'ParseJson', 'inputs': {'content': "@triggerBody()?['RequestJson']", 'schema': schema}, 'runAfter': {}, 'runtimeConfiguration': copy.deepcopy(SECURE_INPUTS)},
        'Valid_request': condition("@and(equals(toLower(coalesce(triggerBody()?['Author']?['Claims'],'')),'i:0#.f|membership|o9w@ornl.gov'),equals(body('Parse_request')?['requestId'],triggerBody()?['Title']),empty(triggerBody()?['ResponseJson']),not(empty(trim(body('Parse_request')?['content']))))", generation, 'Parse_request')}

def request_actions(response):
    actions = {
        'Caller_email': compose("@toLower(if(not(empty(triggerOutputs()?['headers']?['x-ms-user-email-encoded'])),base64ToString(triggerOutputs()?['headers']?['x-ms-user-email-encoded']),coalesce(triggerOutputs()?['headers']?['x-ms-user-email'],'')))"),
        'Read_reviewers': sp('HttpRequest', {'parameters/method': 'GET', 'parameters/uri': '_api/web/sitegroups/getbyid(42)/users', 'parameters/headers': {'Accept': 'application/json;odata=nometadata'}}, 'Caller_email'),
        'Caller_membership': {'type': 'Query', 'inputs': {'from': "@body('Read_reviewers')?['value']", 'where': "@equals(toLower(item()?['LoginName']),concat('i:0#.f|membership|',outputs('Caller_email')))"}, 'runAfter': after('Read_reviewers'), 'runtimeConfiguration': copy.deepcopy(SECURE_INPUTS)}}
    request = {'Request_id': compose('@guid()'),
        'Request_json': compose({'content': "@triggerBody()?['text']", 'owner': "@outputs('Caller_email')", 'requestId': "@outputs('Request_id')"}, 'Request_id'),
        'Create_request': sp('PostItem', {'table': QUEUE, 'item/Title': "@outputs('Request_id')", 'item/RequestJson': "@string(outputs('Request_json'))"}, 'Request_json'),
        'Wait_for_summary': {'type': 'Until', 'expression': "@not(empty(body('Read_result')?['ResponseJson']))", 'limit': {'count': 18, 'timeout': 'PT95S'}, 'actions': {'Wait': {'type': 'Wait', 'inputs': {'interval': {'count': 5, 'unit': 'Second'}}, 'runAfter': {}}, 'Read_result': sp('GetItem', {'table': QUEUE, 'id': "@body('Create_request')?['ID']"}, 'Wait')}, 'runAfter': after('Create_request')}}
    actions['Authorized_request'] = condition("@and(not(empty(outputs('Caller_email'))),equals(length(body('Caller_membership')),1),greater(length(trim(coalesce(triggerBody()?['text'],''))),0),lessOrEquals(length(coalesce(triggerBody()?['text'],'')),12000))", request, 'Caller_membership')
    response = copy.deepcopy(response)
    response['runAfter'] = {'Authorized_request': ALL}
    response['inputs']['body']['summary'] = "@{if(equals(json(if(empty(body('Read_result')?['ResponseJson']),'{\"status\":\"pending\",\"summary\":\"\"}',body('Read_result')?['ResponseJson']))?['status'],'ready'),json(body('Read_result')?['ResponseJson'])?['summary'],'')}"
    actions['Respond_to_a_Power_App_or_flow'] = response
    return actions

def write_package(template, flow, name, filename, connection_sources, new=False):
    files, path, _ = template
    files = dict(files)
    folder = path.rsplit('/', 1)[0]
    resource_id = folder.split('/')[-1]
    manifest = json.loads(files['manifest.json'])
    root_resource = copy.deepcopy(manifest['resources'][resource_id])
    resources = {resource_id: root_resource}
    api_map, connection_map, references = {}, {}, {}
    for source_files, source_path, source_flow in connection_sources:
        source_folder = source_path.rsplit('/', 1)[0]
        source_manifest = json.loads(source_files['manifest.json'])
        api_map.update(json.loads(source_files[source_folder + '/apisMap.json']))
        connection_map.update(json.loads(source_files[source_folder + '/connectionsMap.json']))
        references.update(source_flow['properties']['connectionReferences'])
        resources.update({key: value for key, value in source_manifest['resources'].items() if value['type'] != 'Microsoft.Flow/flows'})
    flow = copy.deepcopy(flow)
    flow['properties']['displayName'] = name
    flow['properties']['connectionReferences'] = references
    if new:
        flow_id = str(uuid5(NAMESPACE_URL, name))
        flow['name'] = flow_id
        flow['id'] = '/providers/Microsoft.Flow/flows/' + flow_id
    root_resource['details']['displayName'] = name
    root_resource['suggestedCreationType'] = 'New' if new else 'Update'
    root_resource['dependsOn'] = sorted([*api_map.values(), *connection_map.values()])
    manifest['details']['displayName'] = name
    manifest['details']['description'] = 'Separate background AI execution from standard SharePoint reviewer requests.'
    manifest['resources'] = resources
    for entry, data in [(path, flow), ('manifest.json', manifest), (folder+'/apisMap.json',api_map), (folder+'/connectionsMap.json',connection_map)]:
        files[entry] = (json.dumps(data, indent=2)+'\n').encode()
    with ZipFile(ROOT / filename, 'w', ZIP_DEFLATED) as archive:
        for entry in sorted(files):
            archive.writestr(entry, files[entry])
    (ROOT / filename.replace('.zip','.json')).write_text(json.dumps(flow, indent=2)+'\n')

def main():
    ai = read_package(ROOT / 'summary-before.zip')
    standard = read_package(ROOT.parent / 'acceptance/save-pa40-pre-repair.zip')
    original = ai[2]
    definition = original['properties']['definition']
    prompt = definition['actions']['Run_a_prompt']
    response = next(action for action in definition['actions'].values() if action['type'] == 'Response')
    background = copy.deepcopy(original)
    trigger = sp('GetOnNewItems', {'table': QUEUE})
    trigger.pop('runAfter')
    trigger['recurrence'] = {'frequency': 'Minute', 'interval': 1}
    trigger['splitOn'] = "@triggerOutputs()?['body/value']"
    background['properties']['definition']['triggers'] = {'When_a_request_is_created': trigger}
    background['properties']['definition']['actions'] = background_actions(prompt)
    request = copy.deepcopy(original)
    request['properties']['definition']['actions'] = request_actions(response)
    write_package(ai, background, '[BETA] Generate Reviewer Summary', 'background-summary.zip', [ai, standard], new=True)
    write_package(ai, request, '[BETA] Request Reviewer Summary', 'standard-summary-request.zip', [standard])
    assert set(request['properties']['definition']['triggers']) == set(definition['triggers'])
    assert 'shared_commondataserviceforapps' not in (ROOT / 'standard-summary-request.json').read_text()
    assert 'Run_a_prompt' not in (ROOT / 'standard-summary-request.json').read_text()
    print('Built background-summary.zip and standard-summary-request.zip; standard package has no AI connector or prompt action.')

if __name__ == '__main__':
    main()
