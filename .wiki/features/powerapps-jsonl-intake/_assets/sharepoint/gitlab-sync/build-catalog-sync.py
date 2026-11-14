"""Build a configurable, guarded GitLab-to-SharePoint scheduled scope."""
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

from sync_config import validate

ROOT = Path(__file__).resolve().parent
SITE = "@outputs('Sync_configuration_BETA')?['sharePointSiteUrl']"
PROJECTS = "@outputs('Sync_configuration_BETA')?['projectListId']"
GITLAB = 'shared_acorn-20gitlab-20logbook-5f06923fc38f4e1ce0-5fee5c9710be0fad46'
SHAREPOINT_REFERENCE = {
    'shared_sharepointonline': {
        'connectionName': 'shared-sharepointonl-4aa12f84-9554-41c4-a184-b21799bd4104',
        'source': 'Embedded',
        'id': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline',
        'tier': 'NotSpecified',
        'apiName': 'sharepointonline',
        'isProcessSimpleApiReferenceConversionAlreadyDone': True,
    }
}


def action(operation, parameters, dependency=None):
    return {
        'type': 'OpenApiConnection',
        'inputs': {
            'host': {
                'connectionName': 'shared_sharepointonline',
                'operationId': operation,
                'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline',
            },
            'parameters': {'dataset': SITE, 'table': PROJECTS, **parameters},
            'retryPolicy': {'type': 'none'},
        },
        'runAfter': {dependency: ['Succeeded']} if dependency else {},
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
    item = ("@setProperty(setProperty(setProperty(json('{}'),'Title'," + project
            + "?['label']),'ProjectId'," + project + "?['id']),'SourceRevision'," + revision + ")")
    create = action('PostItem', {'item': item})
    update = action('PatchItem', {
        'id': "@first(" + matches + ")?['ID']",
        'item': item,
    })
    changed = condition({'or': [
        {'not': {'equals': ["@first(" + matches + ")?['Title']", '@' + project + "?['label']"]}},
        {'not': {'equals': ["@first(" + matches + ")?['SourceRevision']", '@' + revision]}},
    ]}, {'Update_project_BETA': update}, {}, None)
    missing = condition(
        {'equals': ['@length(' + matches + ')', 0]},
        {'Create_project_BETA': create},
        {'Update_if_changed_BETA': changed},
        None,
    )
    cardinality = condition(
        {'lessOrEquals': ['@length(' + matches + ')', 1]},
        {'Create_or_update_project_BETA': missing},
        {'Duplicate_SharePoint_project_BETA': fail_inside_loop('DuplicateProject')},
        'Matching_SharePoint_projects_BETA',
    )
    each = {
        'type': 'Foreach',
        'foreach': '@' + projects,
        'actions': {
            'Matching_SharePoint_projects_BETA': query(
                '@' + current,
                "@equals(item()?['ProjectId']," + project + "?['id'])",
                None,
            ),
            'Check_project_cardinality_BETA': cardinality,
        },
        'runAfter': {},
        'runtimeConfiguration': {'concurrency': {'repetitions': 1}},
    }
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
        ]},
        {
            'Removed_GitLab_projects_BETA': query(
                '@' + current,
                "@and(not(equals(item()?['ProjectId'],'general')),not(contains(body('Select_GitLab_ids_BETA'),item()?['ProjectId'])))",
                None,
            ),
            'Check_removed_projects_BETA': no_removals,
        },
        {'Invalid_project_set_BETA': failed('InvalidCatalog', 'GitLab project IDs are invalid or duplicated.')},
        'Invalid_GitLab_projects_BETA',
    )
    valid_catalog = condition(
        {'and': [
            {'not': {'equals': ['@' + revision, None]}},
            {'not': {'equals': ['@' + revision, '']}},
            {'greater': ['@length(' + projects + ')', 0]},
            {'lessOrEquals': ['@length(' + projects + ')', 1000]},
            {'lessOrEquals': ['@length(' + current + ')', 1000]},
            {'or': [
                {'equals': ['@' + catalog + "?['source']", 'config']},
                {'equals': ['@' + catalog + "?['source']", 'discovery']},
            ]},
        ]},
        {
            'Select_GitLab_ids_BETA': {
                'type': 'Select',
                'inputs': {'from': '@' + projects, 'select': "@item()?['id']"},
                'runAfter': {},
            },
            'Invalid_GitLab_projects_BETA': query(
                '@' + projects,
                "@or(empty(item()?['id']),empty(item()?['label']),equals(item()?['id'],'general'),greater(length(item()?['id']),512),greater(length(item()?['label']),255))",
                'Select_GitLab_ids_BETA',
            ),
            'Validate_project_set_BETA': valid_set,
        },
        {'Invalid_catalog_BETA': failed('InvalidCatalog', 'GitLab catalog or SharePoint source is empty, malformed, or over limit.')},
        'Read_existing_projects_BETA',
    )
    scope = {
        'id': str(uuid5(NAMESPACE_URL, 'ACORN/BETA/GitLabProjectCatalogSync')),
        'brandColor': '#8C6CFF',
        'connectionReferences': SHAREPOINT_REFERENCE,
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
                    },
                    'runAfter': {'Validate_sync_configuration_BETA': ['Succeeded']},
                },
                'Read_existing_projects_BETA': action('GetItems', {
                    '$orderby': 'ProjectId asc',
                    '$top': 1001,
                }, 'Load_GitLab_project_choices_BETA'),
                'Validate_catalog_BETA': valid_catalog,
            },
            'runAfter': {},
        },
    }
    scope['operationDefinition']['actions']['Read_existing_projects_BETA']['runtimeConfiguration'] = {
        'paginationPolicy': {'minimumItemCount': 1001},
    }
    scope['connectionReferences'].update(
        json.loads((ROOT.parents[1] / 'final-intake-scope.clipboard.json').read_text()).get('connectionReferences', {})
    )
    return scope


if __name__ == '__main__':
    (ROOT / 'catalog-sync.clipboard.json').write_text(json.dumps(build(), separators=(',', ':')) + '\n')
