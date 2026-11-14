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


def walk(actions, *, in_loop=False):
    for name, action in actions.items():
        for dependency in action.get('runAfter', {}):
            assert dependency in actions, (name, dependency)
        assert not (in_loop and action['type'] == 'Terminate'), name
        next_loop = in_loop or action['type'] == 'Foreach'
        walk(action.get('actions', {}), in_loop=next_loop)
        walk(action.get('else', {}).get('actions', {}), in_loop=next_loop)


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
    ):
        invalid = copy.deepcopy(config)
        invalid[key] = value
        try:
            validate(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError((key, value))


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
    test_connector_operations()
    print('BETA sync static checks passed')
