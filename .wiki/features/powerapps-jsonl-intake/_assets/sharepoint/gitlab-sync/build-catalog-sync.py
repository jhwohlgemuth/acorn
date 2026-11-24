"""Build a configurable, guarded GitLab-to-SharePoint scheduled scope."""
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

from sync_config import validate

ROOT = Path(__file__).resolve().parent
SITE = "@outputs('Sync_configuration_BETA')?['sharePointSiteUrl']"
PROJECTS = "@outputs('Sync_configuration_BETA')?['projectListId']"
GITLAB = 'shared_acorn-20gitlab-20logbook-5f06923fc38f4e1ce0-5fee5c9710be0fad46'
READ_RETRY = {'type': 'exponential', 'count': 3, 'interval': 'PT5S', 'maximumInterval': 'PT30S'}
SECURE = {'secureData': {'properties': ['inputs', 'outputs']}}


def action(operation, parameters, dependency=None):
    return {
        'type': 'OpenApiConnection',
        'inputs': {
            'host': {
                'connectionName': 'shared_sharepointonline',
                'operationId': operation,
                'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline',
            },
            'parameters': {'dataset': SITE, **({} if operation == 'HttpRequest' else {'table': PROJECTS}), **parameters},
            'retryPolicy': READ_RETRY if operation == 'GetItems' or parameters.get('parameters/method') == 'GET' else {'type': 'none'},
        },
        'runAfter': {dependency: ['Succeeded']} if dependency else {},
        'runtimeConfiguration': SECURE,
    }


def condition(expression, actions, otherwise, dependency):
    return {
        'type': 'If',
        'expression': expression,
        'actions': actions,
        'else': {'actions': otherwise},
        'runAfter': {dependency: ['Succeeded']} if dependency else {},
    }


def failed(code, message):
    return {
        'type': 'Terminate',
        'inputs': {'runStatus': 'Failed', 'runError': {'code': code, 'message': message}},
        'runAfter': {},
    }


def fail_inside_loop(code):
    # Power Automate prohibits Terminate inside Foreach. An invalid JSON parse
    # fails the branch and the enclosing run without writing a project row.
    return {'type': 'Compose', 'inputs': "@json('" + code + "')", 'runAfter': {}}


def query(source, where, dependency):
    return {
        'type': 'Query',
        'inputs': {'from': source, 'where': where},
        'runAfter': {dependency: ['Succeeded']} if dependency else {},
    }


def build(config=None):
    if config is None:
        config = json.loads((ROOT / 'sync-config.json').read_text())
    config = validate(config)
    catalog = "body('Load_GitLab_project_choices_BETA')"
    projects = catalog + "?['projects']"
    revision = catalog + "?['revision']"
    current = "body('Read_existing_projects_BETA')?['value']"
    project = "items('Each_GitLab_project_BETA')"
    matches = "body('Matching_SharePoint_projects_BETA')"
    title = ("if(and(equals(" + project + "?['label']," + project
             + "?['id']),not(empty(first(" + matches + ")?['Title']))),first("
             + matches + ")?['Title']," + project + "?['label'])")
    item = ("@setProperty(setProperty(setProperty(json('{}'),'Title'," + title
            + "),'ProjectId'," + project + "?['id']),'SourceRevision'," + revision + ")")
    create = action('PostItem', {'item': item})
    update = action('HttpRequest', {
        'parameters/method': 'POST',
        'parameters/uri': "@concat('_api/web/lists(guid',decodeUriComponent('%27'),outputs('Sync_configuration_BETA')?['projectListId'],decodeUriComponent('%27'),')/items(',string(first(" + matches + ")?['ID']),')')",
        'parameters/headers': {
            'Accept': 'application/json;odata=nometadata',
            'Content-Type': 'application/json;odata=nometadata',
            'IF-MATCH': "@first(" + matches + ")?['@odata.etag']",
            'X-HTTP-Method': 'MERGE',
        },
        'parameters/body': '@string(' + item[1:] + ')',
    })
    changed = condition({'or': [
        {'not': {'equals': ["@first(" + matches + ")?['Title']", '@' + title]}},
        {'not': {'equals': ["@first(" + matches + ")?['SourceRevision']", '@' + revision]}},
    ]}, {'Update_project_BETA': update}, {}, None)
    missing = condition(
        {'equals': ['@length(' + matches + ')', 0]},
        {'Create_project_BETA': create},
        {'Update_if_changed_BETA': changed},
        'Matching_SharePoint_projects_BETA',
    )
    each = {
        'type': 'Foreach',
        'foreach': '@' + projects,
        'actions': {
            'Matching_SharePoint_projects_BETA': query(
                '@' + current,
                "@equals(toLower(item()?['ProjectId']),toLower(" + project + "?['id']))",
                None,
            ),
            'Create_or_update_project_BETA': missing,
        },
        'runAfter': {},
        'runtimeConfiguration': {'concurrency': {'repetitions': 1}},
    }
    each['actions']['Read_project_after_BETA'] = action('GetItems', {
        '$filter': "@concat('ProjectId eq ',decodeUriComponent('%27'),replace(" + project + "?['id'],decodeUriComponent('%27'),concat(decodeUriComponent('%27'),decodeUriComponent('%27'))),decodeUriComponent('%27'))",
        '$top': 2,
    }, 'Create_or_update_project_BETA')
    each['actions']['Read_project_after_BETA']['runAfter'] = {
        'Create_or_update_project_BETA': ['Succeeded', 'Failed', 'TimedOut'],
    }
    readback = "body('Read_project_after_BETA')?['value']"
    each['actions']['Verify_project_readback_BETA'] = condition(
        {'and': [
            {'equals': ['@length(' + readback + ')', 1]},
            {'equals': ['@first(' + readback + ")?['ProjectId']", '@' + project + "?['id']"]},
            {'equals': ['@first(' + readback + ")?['Title']", '@' + title]},
            {'equals': ['@first(' + readback + ")?['SourceRevision']", '@' + revision]},
            {'equals': ["@empty(coalesce(body('Read_project_after_BETA')?['@odata.nextLink'],body('Read_project_after_BETA')?['odata.nextLink']))", True]},
        ]}, {}, {'Project_readback_failed_BETA': fail_inside_loop('ProjectReadbackMismatch')},
        'Read_project_after_BETA',
    )
    no_removals = condition(
        {'equals': ["@length(body('Removed_GitLab_projects_BETA'))", 0]},
        {'Each_GitLab_project_BETA': each},
        {'Catalog_removal_needs_review_BETA': failed('CatalogRemoval', 'GitLab no longer lists a SharePoint project. Review before changing choices.')},
        'Removed_GitLab_projects_BETA',
    )
    valid_set = condition(
        {'and': [
            {'equals': ["@length(body('Invalid_GitLab_projects_BETA'))", 0]},
            {'equals': ["@length(union(body('Select_GitLab_ids_BETA'),body('Select_GitLab_ids_BETA')))", '@length(' + projects + ')']},
            {'equals': ["@length(body('Invalid_SharePoint_projects_BETA'))", 0]},
            {'equals': ["@length(union(body('Select_SharePoint_ids_BETA'),body('Select_SharePoint_ids_BETA')))", '@length(' + current + ')']},
            {'lessOrEquals': ["@length(union(body('Select_SharePoint_ids_BETA'),body('Select_GitLab_ids_BETA')))", 1000]},
        ]},
        {
            'Removed_GitLab_projects_BETA': query(
                '@' + current,
                "@and(not(equals(toLower(item()?['ProjectId']),'general')),not(contains(body('Select_GitLab_ids_BETA'),toLower(item()?['ProjectId']))))",
                None,
            ),
            'Check_removed_projects_BETA': no_removals,
        },
        {'Invalid_project_set_BETA': failed('InvalidCatalog', 'GitLab project IDs are invalid or duplicated.')},
        'Invalid_SharePoint_projects_BETA',
    )
    valid_catalog = condition(
        {'and': [
            {'not': {'equals': ['@' + revision, None]}},
            {'not': {'equals': ['@' + revision, '']}},
            {'lessOrEquals': ['@length(' + revision + ')', 255]},
            {'greater': ['@length(' + projects + ')', 0]},
            {'lessOrEquals': ['@length(' + projects + ')', 1000]},
            {'lessOrEquals': ['@length(' + current + ')', 1000]},
            {'equals': ["@empty(coalesce(body('Read_existing_projects_BETA')?['@odata.nextLink'],body('Read_existing_projects_BETA')?['odata.nextLink']))", True]},
            {'or': [
                {'equals': ['@' + catalog + "?['source']", 'config']},
                {'equals': ['@' + catalog + "?['source']", 'discovery']},
            ]},
        ]},
        {
            'Select_GitLab_ids_BETA': {
                'type': 'Select',
                'inputs': {'from': '@' + projects, 'select': "@toLower(item()?['id'])"},
                'runAfter': {},
            },
            'Invalid_GitLab_projects_BETA': query(
                '@' + projects,
                "@or(empty(trim(item()?['id'])),empty(trim(item()?['label'])),not(equals(item()?['id'],trim(item()?['id']))),equals(toLower(item()?['id']),'general'),greater(length(item()?['id']),255),greater(length(item()?['label']),255))",
                'Select_GitLab_ids_BETA',
            ),
            'Select_SharePoint_ids_BETA': {
                'type': 'Select',
                'inputs': {'from': '@' + current, 'select': "@toLower(coalesce(item()?['ProjectId'],''))"},
                'runAfter': {'Invalid_GitLab_projects_BETA': ['Succeeded']},
            },
            'Invalid_SharePoint_projects_BETA': query(
                '@' + current,
                "@or(empty(trim(coalesce(item()?['ProjectId'],''))),not(equals(item()?['ProjectId'],trim(coalesce(item()?['ProjectId'],'')))),lessOrEquals(int(coalesce(item()?['ID'],0)),0),and(not(equals(toLower(coalesce(item()?['ProjectId'],'')),'general')),or(empty(item()?['@odata.etag']),equals(item()?['@odata.etag'],'*'))))",
                'Select_SharePoint_ids_BETA',
            ),
            'Validate_project_set_BETA': valid_set,
        },
        {'Invalid_catalog_BETA': failed('InvalidCatalog', 'GitLab catalog or SharePoint source is empty, malformed, or over limit.')},
        'Read_existing_projects_BETA',
    )
    scope = {
        'id': str(uuid5(NAMESPACE_URL, 'ACORN/BETA/GitLabProjectCatalogSync')),
        'brandColor': '#8C6CFF',
        'connectionReferences': {},
        'connectorDisplayName': 'Control',
        'isTrigger': False,
        'operationName': 'Reconcile_GitLab_catalog_BETA',
        'operationDefinition': {
            'type': 'Scope',
            'actions': {
                'Sync_configuration_BETA': {
                    'type': 'Compose',
                    'inputs': config,
                    'runAfter': {},
                },
                'Validate_sync_configuration_BETA': condition(
                    {'and': [
                        {'equals': ["@outputs('Sync_configuration_BETA')?['catalogSyncReady']", True]},
                        {'greater': ["@int(outputs('Sync_configuration_BETA')?['gitLabProjectId'])", 0]},
                        {'not': {'equals': ["@outputs('Sync_configuration_BETA')?['gitLabBranch']", '']}},
                        {'not': {'equals': [SITE, '']}},
                        {'not': {'equals': [PROJECTS, '']}},
                    ]}, {},
                    {'Fail_unconfigured_catalog_BETA': failed(
                        'CatalogSyncNotReady', 'Configure and approve the catalog source and destination first.')},
                    'Sync_configuration_BETA',
                ),
                'Load_GitLab_project_choices_BETA': {
                    'type': 'OpenApiConnection',
                    'inputs': {
                        'host': {
                            'connectionName': GITLAB,
                            'operationId': 'GetProjectCatalogForRepository',
                            'apiId': '/providers/Microsoft.PowerApps/apis/' + GITLAB,
                        },
                        'parameters': {
                            'projectId': "@outputs('Sync_configuration_BETA')?['gitLabProjectId']",
                            'branch': "@outputs('Sync_configuration_BETA')?['gitLabBranch']",
                        },
                        'retryPolicy': READ_RETRY,
                    },
                    'runAfter': {'Validate_sync_configuration_BETA': ['Succeeded']},
                    'runtimeConfiguration': SECURE,
                },
                'Read_project_id_schema_BETA': action('HttpRequest', {
                    'parameters/method': 'GET',
                    'parameters/uri': "@concat('_api/web/lists(guid',decodeUriComponent('%27'),outputs('Sync_configuration_BETA')?['projectListId'],decodeUriComponent('%27'),')/fields/getbyinternalnameortitle(',decodeUriComponent('%27'),'ProjectId',decodeUriComponent('%27'),')?$select=EnforceUniqueValues,Indexed,ReadOnlyField,TypeAsString')",
                    'parameters/headers': {'Accept': 'application/json;odata=nometadata'},
                }, 'Load_GitLab_project_choices_BETA'),
                'Verify_project_id_schema_BETA': condition({'and': [
                    {'equals': ["@body('Read_project_id_schema_BETA')?['EnforceUniqueValues']", True]},
                    {'equals': ["@body('Read_project_id_schema_BETA')?['Indexed']", True]},
                    {'equals': ["@body('Read_project_id_schema_BETA')?['ReadOnlyField']", False]},
                    {'equals': ["@body('Read_project_id_schema_BETA')?['TypeAsString']", 'Text']},
                ]}, {}, {'Unsafe_project_schema_BETA': failed('ProjectIdNotUnique', 'ProjectId must be indexed, writable, unique single-line text before catalog sync.')}, 'Read_project_id_schema_BETA'),
                'Read_existing_projects_BETA': action('GetItems', {
                    '$orderby': 'ProjectId asc',
                    '$top': 1001,
                }, 'Verify_project_id_schema_BETA'),
                'Validate_catalog_BETA': valid_catalog,
            },
            'runAfter': {},
        },
    }
    scope['operationDefinition']['actions']['Read_existing_projects_BETA']['runtimeConfiguration'] = {
        **SECURE,
        'paginationPolicy': {'minimumItemCount': 1001},
    }
    return scope


if __name__ == '__main__':
    (ROOT / 'catalog-sync.clipboard.json').write_text(json.dumps(build(), separators=(',', ':')) + '\n')
