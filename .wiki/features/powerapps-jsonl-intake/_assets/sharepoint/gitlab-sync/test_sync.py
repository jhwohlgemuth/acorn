"""Static safety checks for reusable BETA sync definitions."""

import copy
import importlib.util
import json
from pathlib import Path

import yaml

from sync_config import validate

ROOT = Path(__file__).resolve().parent
ASSETS = ROOT.parents[1]


def load_builder(name):
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), ROOT / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def walk(actions, *, in_loop=False, depth=1):
    for name, action in actions.items():
        assert depth <= 8, (name, 'Power Automate nesting limit', depth)
        assert len(name) <= 80, name
        for dependency in action.get('runAfter', {}):
            assert dependency in actions, (name, dependency)
        assert not (in_loop and action['type'] == 'Terminate'), name
        next_loop = in_loop or action['type'] == 'Foreach'
        walk(action.get('actions', {}), in_loop=next_loop, depth=depth + 1)
        walk(action.get('else', {}).get('actions', {}), in_loop=next_loop, depth=depth + 1)


def test_config_boundaries():
    config = json.loads((ROOT / 'sync-config.json').read_text())
    validate(config)
    assert not any(config[key] for key in (
        'approvedSourceReady', 'catalogSyncReady', 'gitLabWriteReady'))
    for key, value in (
        ('approvedGitLabPath', '.acorn/intake/logbook.jsonl'),
        ('gitLabProjectId', 0),
        ('projectListId', 'not-a-uuid'),
        ('sharePointSiteUrl', 'http://example.com'),
        ('sharePointSiteUrl', 'https://user:secret@example.com/sites/test'),
        ('sharePointSiteUrl', 'https://example.com/sites/test?redirect=elsewhere'),
        ('sharePointSiteUrl', 'https://example.com/sites/%27test'),
        ('approvedExportLibrary', '..'),
        ('approvedGitLabPath', '.acorn/approved/nested//file.jsonl'),
        ('approvedGitLabPath', '.acorn/approved/file\n.jsonl'),
        ('gitLabBranch', 'main.lock'),
        ('gitLabBranch', ' main'),
        ('gitLabProjectId', True),
        ('projectListId', '00000000-0000-0000-0000-000000000000'),
        ('maxReceiptAgeHours', 0),
        ('maxReceiptAgeHours', True),
        ('unexpectedSetting', 'not-allowed'),
    ):
        invalid = copy.deepcopy(config)
        invalid[key] = value
        try:
            validate(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError((key, value))


def flatten(actions):
    return {name: action for name, action in actions.items()} | {
        nested_name: nested_action
        for action in actions.values()
        for branch in (action.get('actions', {}), action.get('else', {}).get('actions', {}))
        for nested_name, nested_action in flatten(branch).items()
    }


def test_production_guards():
    approved = flatten(load_builder('build-approved-sync.py').build()['operationDefinition']['actions'])
    assert set(approved['Read_GitLab_after_BETA']['runAfter']['Create_or_update_GitLab_BETA']) == {
        'Succeeded', 'Failed', 'TimedOut'}
    assert 'Read_export_metadata_before_BETA' in approved
    assert 'Create_approved_JSONL_BETA' not in approved
    catalog = flatten(load_builder('build-catalog-sync.py').build()['operationDefinition']['actions'])
    assert 'Invalid_SharePoint_projects_BETA' in catalog
    assert catalog['Update_project_BETA']['inputs']['parameters']['parameters/headers']['IF-MATCH'] != '*'
    assert 'Verify_project_readback_BETA' in catalog


def test_generated_artifacts():
    from build_workflows import build as build_workflow
    for name, minute in (('catalog', 0), ('approved', 30)):
        expected = load_builder('build-' + name + '-sync.py').build()
        saved = json.loads((ROOT / (name + '-sync.clipboard.json')).read_text())
        assert saved == expected, name + ' clipboard artifact is stale'
        definition = build_workflow('build-' + name + '-sync.py', minute)
        assert json.loads((ROOT / (name + '-sync.workflow.json')).read_text()) == definition
        walk(definition['actions'])
        for action in flatten(definition['actions']).values():
            if action['type'] == 'OpenApiConnection':
                inputs = action['inputs']
                writing = inputs['host']['operationId'] in ('PostItem', 'CommitApprovedJsonl') or inputs['parameters'].get('parameters/method') == 'POST'
                assert inputs['retryPolicy'] == ({'type': 'none'} if writing else {
                    'type': 'exponential', 'count': 3, 'interval': 'PT5S', 'maximumInterval': 'PT30S',
                })


def test_flow_boundaries():
    catalog = load_builder('build-catalog-sync.py').build()['operationDefinition']
    approved = load_builder('build-approved-sync.py').build()['operationDefinition']
    for definition in (catalog, approved):
        walk(definition['actions'])
    assert 'CommitApprovedJsonl' not in json.dumps(catalog)
    assert 'CommitApprovedJsonl' in json.dumps(approved)
    assert '.acorn/intake/logbook.jsonl' not in json.dumps(approved)
    assert 'Verify_approved_receipt_BETA' in json.dumps(approved)
    assert 'GetProjectCatalogForRepository' in json.dumps(catalog)


def test_connector_operations():
    swagger = yaml.safe_load((ASSETS / 'project-catalog.swagger.yaml').read_text())
    operations = {method['operationId'] for path in swagger['paths'].values()
                  for method in path.values()}
    assert {'GetProjectCatalogForRepository', 'ReadApprovedJsonl',
            'CommitApprovedJsonl'} <= operations
    assert swagger['securityDefinitions']['api_key']['name'] == 'PRIVATE-TOKEN'


if __name__ == '__main__':
    test_config_boundaries()
    test_flow_boundaries()
    test_production_guards()
    test_generated_artifacts()
    test_connector_operations()
    print('BETA sync static checks passed')
