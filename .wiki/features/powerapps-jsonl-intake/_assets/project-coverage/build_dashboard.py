"""Build isolated standard-SharePoint dashboard actions for the classic designer."""
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

ROOT = Path(__file__).resolve().parent
SITE = 'https://ornl.sharepoint.com/sites/ResearchEnablement'
INTAKE = 'a904a6c6-5dd0-427b-9575-8167c4c0c412'
CATALOG = '88dfa78d-a62f-4e0f-aae5-0d5ea52758ed'
MONTHS = 'Sponsor Highlights Months'
COVERAGE = 'Sponsor Highlights Coverage'
REFS = json.loads((ROOT.parent/'sharepoint/SharePoint_intake_SP.clipboard.json').read_text())['connectionReferences']


def after(name):
    return {name: ['Succeeded']} if name else {}


def compose(value, dependency=None):
    return {'type': 'Compose', 'inputs': value, 'runAfter': after(dependency)}


def query(source, condition, dependency=None):
    return {'type': 'Query', 'inputs': {'from': source, 'where': condition}, 'runAfter': after(dependency)}


def http(method, uri, body=None, dependency=None, headers=None):
    params = {'dataset': SITE, 'parameters/method': method, 'parameters/uri': uri,
              'parameters/headers': {'Accept': 'application/json;odata=nometadata', 'Content-Type': 'application/json;odata=nometadata', **(headers or {})}}
    if body is not None:
        params['parameters/body'] = body if isinstance(body, str) else json.dumps(body)
    return {'type': 'OpenApiConnection', 'inputs': {'host': {
        'connectionName': 'shared_sharepointonline', 'operationId': 'HttpRequest',
        'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline'},
        'parameters': params, 'authentication': "@parameters('$authentication')", 'retryPolicy': {'type': 'none'}}, 'runAfter': after(dependency)}


def get_items(table, dependency=None):
    return {'type': 'OpenApiConnection', 'inputs': {'host': {
        'connectionName': 'shared_sharepointonline', 'operationId': 'GetItems',
        'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline'},
        'parameters': {'dataset': SITE, 'table': table, '$orderby': 'ID asc', '$top': 5001},
        'authentication': "@parameters('$authentication')"}, 'runAfter': after(dependency),
        'runtimeConfiguration': {'paginationPolicy': {'minimumItemCount': 5001}, 'secureData': {'properties': ['inputs', 'outputs']}}}


def scope(actions, dependency=None):
    return {'type': 'Scope', 'actions': actions, 'runAfter': after(dependency)}


def condition(expression, yes, no=None, dependency=None):
    return {'type': 'If', 'expression': expression, 'actions': yes, 'else': {'actions': no or {}}, 'runAfter': after(dependency)}


def fail(message):
    return {'type': 'Terminate', 'inputs': {'runStatus': 'Failed', 'runError': {'code': 'CoverageIncomplete', 'message': message}}, 'runAfter': {}}


def package(name, actions):
    return {'id': str(uuid5(NAMESPACE_URL, SITE+'/'+name)), 'brandColor': '#8C6CFF', 'connectionReferences': REFS,
            'connectorDisplayName': 'Control', 'isTrigger': False, 'operationName': name,
            'operationDefinition': scope(actions)}


def list_uri(title):
    return "_api/web/lists/getbytitle('"+title+"')"


def provision():
    actions = {}
    schemas = {
        MONTHS: [('ExpectedProjects', 'Note'), ('ProjectsExpected', 'Number'), ('ProjectsSubmitted', 'Number'), ('HighlightCount', 'Number'), ('GeneralCount', 'Number'), ('IsCurrentMonth', 'Boolean'), ('RefreshedAt', 'DateTime'), ('RosterBasis', 'Text'), ('ReceiptItemIds', 'Note'), ('ReceiptLinks', 'Note')],
        COVERAGE: [('CoverageKey', 'Text'), ('MonthKey', 'Text'), ('ProjectId', 'Note'), ('HighlightCount', 'Number'), ('FirstReceived', 'DateTime'), ('LastReceived', 'DateTime'), ('RefreshedAt', 'DateTime'), ('ReceiptItemIds', 'Note'), ('ReceiptLinks', 'Note')]}
    previous = None
    for n, (title, fields) in enumerate(schemas.items()):
        prefix = f'List_{n}'
        actions[prefix] = http('GET', list_uri(title)+'?$select=Id', dependency=previous)
        create = condition({'equals': [f"@outputs('{prefix}')?['statusCode']", 404]},
            {prefix+'_create': http('POST', '_api/web/lists', {'BaseTemplate': 100, 'Description': 'Derived project coverage; intake receipts remain authoritative.', 'Title': title})},
            {prefix+'_readable': condition({'equals': [f"@outputs('{prefix}')?['statusCode']", 200]}, {}, {prefix+'_stop': fail('Cannot verify dashboard list access.')})})
        create['runAfter'] = {prefix: ['Succeeded', 'Failed', 'TimedOut']}
        actions[prefix+'_ensure'] = create
        previous = prefix+'_ensure'
        if title == MONTHS:
            actions[prefix+'_unique_month'] = http('POST', list_uri(title)+"/fields/getbyinternalnameortitle('Title')", {'Indexed': True, 'EnforceUniqueValues': True}, previous, {'IF-MATCH': '*', 'X-HTTP-Method': 'MERGE'})
            previous = prefix+'_unique_month'
        for name, kind in fields:
            field = prefix+'_'+name
            actions[field] = http('GET', list_uri(title)+f"/fields?$filter=InternalName eq '{name}'&$select=InternalName", dependency=previous)
            attrs = ' RichText="FALSE" NumLines="6"' if kind == 'Note' else ''
            if name == 'CoverageKey':
                attrs = ' Indexed="TRUE" EnforceUniqueValues="TRUE"'
            xml = f'<Field Type="{kind}" Name="{name}" StaticName="{name}" DisplayName="{name}"{attrs}/>'
            ensure = condition({'equals': [f"@outputs('{field}')?['statusCode']", 200]},
                {field+'_missing': condition({'equals': [f"@length(body('{field}')?['value'])", 0]},
                    {field+'_create': http('POST', list_uri(title)+'/fields/createfieldasxml', {'parameters': {'SchemaXml': xml, 'Options': 0}})})},
                {field+'_stop': fail('Cannot verify dashboard column.')})
            ensure['runAfter'] = {field: ['Succeeded', 'Failed', 'TimedOut']}
            actions[field+'_ensure'] = ensure
            previous = field+'_ensure'
    return package('Provision_coverage', actions)


def refresh():
    a = {'Read_receipts_COV': get_items(INTAKE), 'Read_catalog_COV': get_items(CATALOG, 'Read_receipts_COV')}
    a['Read_first_month_COV'] = http('GET', list_uri(MONTHS)+'/items?$orderby=Title asc&$top=5001', dependency='Read_catalog_COV')
    a['Read_coverage_COV'] = http('GET', list_uri(COVERAGE)+'/items?$top=5001', dependency='Read_first_month_COV')
    raw = "json(base64ToString(concat(item()?['PayloadBase64A'],coalesce(item()?['PayloadBase64B'],string(null)))))"
    a['Decode_receipts_COV'] = {'type': 'Select', 'inputs': {'from': "@body('Read_receipts_COV')?['value']", 'select': {
        'created': "@item()?['Created']", 'id': "@item()?['SubmissionId']", 'itemId': "@string(item()?['ID'])", 'project': "@item()?['ProjectId']",
        'revision': '@'+raw+"?['revision']", 'payloadId': '@'+raw+"?['submissionId']", 'payloadProject': '@'+raw+"?['projectId']",
        'summary': '@'+raw+"?['entry']?['summary']", 'parent': "@item()?['CorrectionParentId']"}}, 'runAfter': after('Read_coverage_COV'), 'runtimeConfiguration': {'secureData': {'properties': ['inputs', 'outputs']}}}
    a['Invalid_receipts_COV'] = query("@body('Decode_receipts_COV')", "@or(empty(item()?['id']),empty(item()?['project']),empty(item()?['created']),not(equals(item()?['id'],item()?['payloadId'])),not(equals(item()?['project'],item()?['payloadProject'])),not(equals(coalesce(item()?['parent'],string(null)),coalesce(item()?['revision']?['supersedesSubmissionId'],string(null)))))", 'Decode_receipts_COV')
    work = {
        'Originals_COV': query("@body('Decode_receipts_COV')", "@empty(item()?['revision'])"),
        'Projects_COV': query("@body('Read_catalog_COV')?['value']", "@and(not(empty(item()?['ProjectId'])),not(equals(item()?['ProjectId'],'general')))" , 'Originals_COV'),
        'Roster_COV': {'type': 'Select', 'inputs': {'from': "@body('Projects_COV')", 'select': {'catalogId': "@item()?['ID']", 'id': "@item()?['ProjectId']", 'name': "@item()?['Title']"}}, 'runAfter': after('Projects_COV')},
        'Current_month_COV': compose("@formatDateTime(convertTimeZone(utcNow(),'UTC','Eastern Standard Time'),'yyyy-MM')", 'Roster_COV'),
        'Initial_start_COV': compose("@formatDateTime(addToTime(concat(outputs('Current_month_COV'),'-01T00:00:00Z'),-11,'Month'),'yyyy-MM')", 'Current_month_COV'),
        'Receipt_start_COV': compose("@if(empty(body('Originals_COV')),outputs('Initial_start_COV'),formatDateTime(convertTimeZone(first(body('Originals_COV'))?['created'],'UTC','Eastern Standard Time'),'yyyy-MM'))", 'Initial_start_COV'),
        'Retained_start_COV': compose("@if(empty(body('Read_first_month_COV')?['value']),outputs('Initial_start_COV'),first(body('Read_first_month_COV')?['value'])?['Title'])", 'Receipt_start_COV'),
        'Earlier_start_COV': compose("@if(less(outputs('Receipt_start_COV'),outputs('Retained_start_COV')),outputs('Receipt_start_COV'),outputs('Retained_start_COV'))", 'Retained_start_COV'),
        'Start_month_COV': compose("@if(less(outputs('Earlier_start_COV'),outputs('Initial_start_COV')),outputs('Earlier_start_COV'),outputs('Initial_start_COV'))", 'Earlier_start_COV'),
        'Month_count_COV': compose("@add(add(mul(sub(int(substring(outputs('Current_month_COV'),0,4)),int(substring(outputs('Start_month_COV'),0,4))),12),sub(int(substring(outputs('Current_month_COV'),5,2)),int(substring(outputs('Start_month_COV'),5,2)))),1)", 'Start_month_COV')}
    per_month = {
        'Month_key_COV': compose("@formatDateTime(addToTime(concat(outputs('Start_month_COV'),'-01T00:00:00Z'),items('For_month_COV'),'Month'),'yyyy-MM')"),
        'Read_month_COV': query("@body('Read_first_month_COV')?['value']", "@equals(item()?['Title'],outputs('Month_key_COV'))", 'Month_key_COV'),
        'Month_roster_COV': compose("@if(empty(body('Read_month_COV')),body('Roster_COV'),json(first(body('Read_month_COV'))?['ExpectedProjects']))", 'Read_month_COV'),
        'Month_receipts_COV': query("@body('Originals_COV')", "@equals(formatDateTime(convertTimeZone(item()?['created'],'UTC','Eastern Standard Time'),'yyyy-MM'),outputs('Month_key_COV'))", 'Month_roster_COV'),
        'General_COV': query("@body('Month_receipts_COV')", "@equals(item()?['project'],'general')", 'Month_receipts_COV'),
        'Submitted_project_ids_COV': {'type': 'Select', 'inputs': {'from': "@body('Month_receipts_COV')", 'select': "@item()?['project']"}, 'runAfter': after('General_COV')},
        'Covered_roster_COV': query("@outputs('Month_roster_COV')", "@contains(body('Submitted_project_ids_COV'),item()?['id'])", 'Submitted_project_ids_COV')}
    project = {
        'Project_receipts_COV': query("@body('Month_receipts_COV')", "@equals(item()?['project'],items('For_project_COV')?['id'])"),
        'Coverage_key_COV': compose("@concat(outputs('Month_key_COV'),'|',string(items('For_project_COV')?['catalogId']))", 'Project_receipts_COV'),
        'Read_project_COV': query("@body('Read_coverage_COV')?['value']", "@equals(item()?['CoverageKey'],outputs('Coverage_key_COV'))", 'Coverage_key_COV')}
    fields = {'Title': "@items('For_project_COV')?['name']", 'CoverageKey': "@outputs('Coverage_key_COV')", 'MonthKey': "@outputs('Month_key_COV')", 'ProjectId': "@items('For_project_COV')?['id']", 'HighlightCount': "@length(body('Project_receipts_COV'))", 'FirstReceived': "@if(empty(body('Project_receipts_COV')),null,first(body('Project_receipts_COV'))?['created'])", 'LastReceived': "@if(empty(body('Project_receipts_COV')),null,last(body('Project_receipts_COV'))?['created'])", 'RefreshedAt': '@utcNow()'}
    project['Project_item_ids_COV'] = {'type': 'Select', 'inputs': {'from': "@body('Project_receipts_COV')", 'select': "@item()?['itemId']"}, 'runAfter': after('Read_project_COV')}
    fields['ReceiptItemIds'] = "@join(body('Project_item_ids_COV'),';')"
    project['Project_links_COV'] = {'type': 'Select', 'inputs': {'from': "@body('Project_receipts_COV')", 'select': "@concat(item()?['itemId'],'|',replace(replace(replace(coalesce(item()?['summary'],'Summary unavailable'),'%','%25'),';','%3B'),'|','%7C'))"}, 'runAfter': after('Project_item_ids_COV')}
    fields['ReceiptLinks'] = "@join(body('Project_links_COV'),';')"
    # Compose preserves numeric/null types before HTTP serialization.
    project['Project_values_COV'] = compose(fields, 'Project_links_COV')
    project['Upsert_project_COV'] = condition({'equals': ["@length(body('Read_project_COV'))", 0]},
        {'Create_project_COV': http('POST', list_uri(COVERAGE)+'/items', "@string(outputs('Project_values_COV'))")},
        {'Update_project_COV': http('POST', list_uri(COVERAGE)+"/items(@{first(body('Read_project_COV'))?['Id']})", "@string(outputs('Project_values_COV'))", headers={'IF-MATCH': '*', 'X-HTTP-Method': 'MERGE'})}, 'Project_values_COV')
    project['Upsert_project_COV']['else']['actions'] = {'Project_changed_COV': condition(
        {'or': [{'not': {'equals': ["@first(body('Read_project_COV'))?['HighlightCount']", "@length(body('Project_receipts_COV'))"]}},
                {'not': {'equals': ["@first(body('Read_project_COV'))?['Title']", "@items('For_project_COV')?['name']"]}},
                {'not': {'equals': ["@coalesce(first(body('Read_project_COV'))?['ReceiptItemIds'],string(null))", fields['ReceiptItemIds']]}},
                {'not': {'equals': ["@coalesce(first(body('Read_project_COV'))?['ReceiptLinks'],string(null))", fields['ReceiptLinks']]}}]},
        project['Upsert_project_COV']['else']['actions'])}
    per_month['For_project_COV'] = {'type': 'Foreach', 'foreach': "@outputs('Month_roster_COV')", 'actions': project, 'runAfter': after('Covered_roster_COV'), 'runtimeConfiguration': {'concurrency': {'repetitions': 1}}}
    month_values = {'Title': "@outputs('Month_key_COV')", 'ExpectedProjects': "@string(outputs('Month_roster_COV'))", 'ProjectsExpected': "@length(outputs('Month_roster_COV'))", 'ProjectsSubmitted': "@length(body('Covered_roster_COV'))", 'HighlightCount': "@length(body('Month_receipts_COV'))", 'GeneralCount': "@length(body('General_COV'))", 'IsCurrentMonth': "@equals(outputs('Month_key_COV'),outputs('Current_month_COV'))", 'RefreshedAt': '@utcNow()', 'RosterBasis': "@if(empty(body('Read_month_COV')),if(equals(outputs('Month_key_COV'),outputs('Current_month_COV')),'Catalog snapshot at first refresh','Historical backfill using catalog at dashboard setup'),first(body('Read_month_COV'))?['RosterBasis'])"}
    per_month['Month_item_ids_COV'] = {'type': 'Select', 'inputs': {'from': "@body('Month_receipts_COV')", 'select': "@item()?['itemId']"}, 'runAfter': after('For_project_COV')}
    month_values['ReceiptItemIds'] = "@join(body('Month_item_ids_COV'),';')"
    per_month['Month_links_COV'] = {'type': 'Select', 'inputs': {'from': "@body('Month_receipts_COV')", 'select': "@concat(item()?['itemId'],'|',replace(replace(replace(coalesce(item()?['summary'],'Summary unavailable'),'%','%25'),';','%3B'),'|','%7C'))"}, 'runAfter': after('Month_item_ids_COV')}
    month_values['ReceiptLinks'] = "@join(body('Month_links_COV'),';')"
    per_month['Month_values_COV'] = compose(month_values, 'Month_links_COV')
    per_month['Upsert_month_COV'] = condition({'equals': ["@length(body('Read_month_COV'))", 0]},
        {'Create_month_COV': http('POST', list_uri(MONTHS)+'/items', "@string(outputs('Month_values_COV'))")},
        {'Update_month_COV': http('POST', list_uri(MONTHS)+"/items(@{first(body('Read_month_COV'))?['Id']})", "@string(outputs('Month_values_COV'))", headers={'IF-MATCH': '*', 'X-HTTP-Method': 'MERGE'})}, 'Month_values_COV')
    per_month['Upsert_month_COV']['else']['actions'] = {'Month_changed_COV': condition(
        {'or': [{'equals': ["@outputs('Month_key_COV')", "@outputs('Current_month_COV')"]},
                {'equals': ["@first(body('Read_month_COV'))?['IsCurrentMonth']", True]},
                {'not': {'equals': ["@first(body('Read_month_COV'))?['HighlightCount']", "@length(body('Month_receipts_COV'))"]}},
                {'not': {'equals': ["@first(body('Read_month_COV'))?['ProjectsSubmitted']", "@length(body('Covered_roster_COV'))"]}},
                {'not': {'equals': ["@coalesce(first(body('Read_month_COV'))?['ReceiptItemIds'],string(null))", month_values['ReceiptItemIds']]}},
                {'not': {'equals': ["@coalesce(first(body('Read_month_COV'))?['ReceiptLinks'],string(null))", month_values['ReceiptLinks']]}}]},
        per_month['Upsert_month_COV']['else']['actions'])}
    work['Bound_months_COV'] = condition({'and': [{'greater': ["@outputs('Month_count_COV')", 0]}, {'lessOrEquals': ["@outputs('Month_count_COV')", 120]}]},
        {'For_month_COV': {'type': 'Foreach', 'foreach': "@range(0,outputs('Month_count_COV'))", 'actions': per_month, 'runAfter': {}, 'runtimeConfiguration': {'concurrency': {'repetitions': 1}}}}, {'Stop_months_COV': fail('Month range is invalid or exceeds ten years.')}, 'Month_count_COV')
    a['Bounded_valid_input_COV'] = condition({'and': [{'lessOrEquals': ["@length(body('Read_receipts_COV')?['value'])", 5000]}, {'lessOrEquals': ["@length(body('Read_catalog_COV')?['value'])", 5000]}, {'equals': ["@length(body('Invalid_receipts_COV'))", 0]}, {'lessOrEquals': ["@length(body('Read_first_month_COV')?['value'])", 120]}, {'lessOrEquals': ["@length(body('Read_coverage_COV')?['value'])", 5000]}, {'equals': ["@empty(body('Read_first_month_COV')?['odata.nextLink'])", True]}, {'equals': ["@empty(body('Read_coverage_COV')?['odata.nextLink'])", True]}]}, work, {'Stop_input_COV': fail('Invalid or truncated source. Dashboard refresh stopped; inspect last refreshed timestamps.')}, 'Invalid_receipts_COV')
    return package('Refresh_project_coverage', a)


if __name__ == '__main__':
    for name, data in [('provision.clipboard.json', provision()), ('refresh.clipboard.json', refresh())]:
        (ROOT/name).write_text(json.dumps(data, indent=2)+'\n')
