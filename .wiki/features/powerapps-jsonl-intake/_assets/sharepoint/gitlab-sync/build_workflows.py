"""Build reviewable scheduled definitions without importing or enabling flows."""

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def build(builder_name, minute):
    spec = importlib.util.spec_from_file_location(builder_name.replace('-', '_'), ROOT / builder_name)
    builder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(builder)
    scope = builder.build()
    evidence = ({
        'sourceRevision': "@outputs('Receipt_object_BETA')?['fileRevision']",
        'sourceReceiptRunId': "@outputs('Receipt_object_BETA')?['runId']",
        'lastApprovedBatchId': "@outputs('Receipt_object_BETA')?['lastApprovedBatchId']",
        'destinationRevision': "@body('Read_GitLab_after_BETA')?['last_commit_id']",
        'lineCount': "@outputs('Receipt_object_BETA')?['lineCount']",
    } if builder_name == 'build-approved-sync.py' else {
        'sourceRevision': "@body('Load_GitLab_project_choices_BETA')?['revision']",
        'projectCount': "@length(body('Load_GitLab_project_choices_BETA')?['projects'])",
    })
    return {
        '$schema': 'https://schema.management.azure.com/providers/Microsoft.Logic/schemas/2016-06-01/workflowdefinition.json#',
        'contentVersion': '1.0.0.0',
        'parameters': {
            '$authentication': {'defaultValue': {}, 'type': 'SecureObject'},
            '$connections': {'defaultValue': {}, 'type': 'Object'},
        },
        'triggers': {
            'Recurrence': {
                'type': 'Recurrence',
                'recurrence': {'frequency': 'Day', 'interval': 1, 'timeZone': 'Eastern Standard Time',
                               'schedule': {'hours': [3], 'minutes': [minute]}},
                'runtimeConfiguration': {'concurrency': {'runs': 1}},
            },
        },
        'actions': {
            scope['operationName']: scope['operationDefinition'],
            'Sync_verified_BETA': {
                'type': 'Compose',
                'inputs': {**evidence, 'status': 'verified', 'runId': "@workflow()?['run']?['name']", 'verifiedAtUtc': '@utcNow()'},
                'runAfter': {scope['operationName']: ['Succeeded']},
            },
            'Sync_failed_BETA': {
                'type': 'Terminate',
                'inputs': {'runStatus': 'Failed', 'runError': {
                    'code': 'SyncNotVerified',
                    'message': 'Sync did not verify. Inspect the failed action; reconcile destination state before retrying. Never restore over a newer revision.',
                }},
                'runAfter': {scope['operationName']: ['Failed', 'TimedOut']},
            },
        },
        'outputs': {},
    }


if __name__ == '__main__':
    for name, minute in (('catalog', 0), ('approved', 30)):
        definition = build('build-' + name + '-sync.py', minute)
        (ROOT / (name + '-sync.workflow.json')).write_text(json.dumps(definition, indent=2) + '\n')
