"""Behavioral fault matrix for generated sync graphs with synthetic connectors."""

import base64
from copy import deepcopy
import json
from pathlib import Path
import unittest

from build_workflows import build as workflow
from test_runtime import execute
from test_sync import flatten, load_builder

ROOT = Path(__file__).resolve().parent
CONFIG = json.loads((ROOT / 'sync-config.json').read_text())
OLD = b'{"submissionId":"synthetic-old"}\n'
NEW = OLD + b'{"submissionId":"synthetic-new","note":"caf\xc3\xa9"}\n'


def encoded(data):
    return base64.b64encode(data).decode()


def ready():
    return {**CONFIG, 'catalogSyncReady': True, 'approvedSourceReady': True, 'gitLabWriteReady': True}


class ApprovedService:
    def __init__(self):
        self.file = OLD
        self.snapshot = NEW
        self.revision = 'git-before'
        self.before_etag = self.after_etag = '"source-v1"'
        self.write_status = 'Succeeded'
        self.read_status = 'Succeeded'
        self.initial_status = 'Succeeded'
        self.initial_code = 200
        self.commit = True
        self.next_link = None
        self.writes = 0
        self.calls = []
        content = encoded(NEW)
        self.receipt = {
            'batchCount': 1, 'byteCount': len(NEW), 'lineCount': 2,
            'candidateFingerprint': 'b64-edge-v1:' + str(len(content)) + ':' + content[:64] + ':' + content[-64:],
            'fileIdentity': '/sites/ResearchEnablement/' + CONFIG['approvedExportLibrary'] + '/logbook.jsonl',
            'fileRevision': self.before_etag, 'lastApprovedBatchId': 'synthetic-batch',
            'runId': 'synthetic-export', 'verifiedAtUtc': '2026-11-24T11:00:00Z',
        }

    def __call__(self, name, parameters, action):
        self.calls.append(name)
        status, body, code = 'Succeeded', {}, 200
        if name == 'List_export_files_BETA':
            body = {'value': [{'Name': 'export-receipt-synthetic.json'}], '@odata.nextLink': self.next_link}
        elif name == 'Read_receipt_BETA':
            body = {'$content': encoded(json.dumps(self.receipt).encode())}
        elif name == 'Read_approved_export_BETA':
            body = {'$content': encoded(self.snapshot)}
        elif name in ('Read_export_metadata_before_BETA', 'Read_export_metadata_BETA'):
            body = {'ETag': self.before_etag if name.endswith('before_BETA') else self.after_etag, 'UniqueId': 'synthetic-source'}
        elif name in ('Read_GitLab_before_BETA', 'Read_GitLab_after_BETA'):
            status = self.initial_status if name.endswith('before_BETA') else self.read_status
            code = self.initial_code if name.endswith('before_BETA') else 200
            body = {'encoding': 'base64', 'content': encoded(self.file), 'file_path': CONFIG['approvedGitLabPath'],
                    'last_commit_id': self.revision}
        elif name == 'Update_approved_JSONL_BETA':
            self.writes += 1
            write = parameters['body/actions'][0]
            assert write['action'] == 'update'
            assert write['last_commit_id'] == self.revision
            assert action['inputs']['retryPolicy'] == {'type': 'none'}
            if self.commit:
                self.file = base64.b64decode(write['content'])
                self.revision = 'git-after'
            status = self.write_status
        else:
            raise AssertionError(name)
        return status, {'body': body, 'statusCode': code}


class CatalogService:
    def __init__(self):
        self.projects = [{'id': 'alpha', 'label': 'Alpha'}, {'id': 'beta', 'label': 'Beta'}]
        self.rows = [{'ID': 1, 'ProjectId': 'alpha', 'Title': 'Old Alpha', 'SourceRevision': 'old', '@odata.etag': '"1"'},
                     {'ID': 2, 'ProjectId': 'general', 'Title': 'General'}]
        self.next_link = None
        self.unique = True
        self.conflict = False
        self.write_status = 'Succeeded'
        self.read_status = 'Succeeded'
        self.writes = 0

    def __call__(self, name, parameters, action):
        status = 'Succeeded'
        if name == 'Load_GitLab_project_choices_BETA':
            body = {'projects': self.projects, 'revision': 'source-commit', 'source': 'config'}
        elif name == 'Read_project_id_schema_BETA':
            body = {'EnforceUniqueValues': self.unique, 'Indexed': True, 'ReadOnlyField': False, 'TypeAsString': 'Text'}
        elif name == 'Read_existing_projects_BETA':
            body = {'value': deepcopy(self.rows), '@odata.nextLink': self.next_link}
        elif name == 'Create_project_BETA':
            self.writes += 1
            row = deepcopy(parameters['item'])
            assert not any(old['ProjectId'].lower() == row['ProjectId'].lower() for old in self.rows)
            self.rows.append({**row, 'ID': len(self.rows) + 1, '@odata.etag': '"1"'})
            status, body = self.write_status, {}
        elif name == 'Update_project_BETA':
            self.writes += 1
            item = json.loads(parameters['parameters/body'])
            current = next(row for row in self.rows if row['ProjectId'].lower() == item['ProjectId'].lower())
            assert parameters['parameters/headers']['IF-MATCH'] == current['@odata.etag']
            if self.conflict:
                status = 'Failed'
            else:
                current.update(item)
                current['@odata.etag'] = '"2"'
                status = self.write_status
            body = {}
        elif name == 'Read_project_after_BETA':
            key = parameters['$filter'].removeprefix("ProjectId eq '")[:-1].replace("''", "'")
            body = {'value': [deepcopy(row) for row in self.rows if row['ProjectId'].lower() == key.lower()]}
            status = self.read_status
        else:
            raise AssertionError(name)
        return status, {'body': body, 'statusCode': 200}


class FailureTests(unittest.TestCase):
    def run_approved(self, service, config=None):
        graph = load_builder('build-approved-sync.py').build(ready() if config is None else config)
        return execute(graph['operationDefinition']['actions'], service)

    def run_catalog(self, service, config=None):
        graph = load_builder('build-catalog-sync.py').build(ready() if config is None else config)
        return execute(graph['operationDefinition']['actions'], service)

    def test_approved_changed_then_replay(self):
        service = ApprovedService()
        status, _ = self.run_approved(service)
        self.assertEqual(status, 'Succeeded')
        self.assertEqual(service.file, NEW)
        self.assertEqual(service.writes, 1)
        self.assertEqual(self.run_approved(service)[0], 'Succeeded')
        self.assertEqual(service.writes, 1)
    def test_approved_empty_existing_destination(self):
        service = ApprovedService()
        service.file = b''
        self.assertEqual(self.run_approved(service)[0], 'Succeeded')
        self.assertEqual(service.file, NEW)
    def test_approved_lost_response_reconciles_only_exact_bytes(self):
        for outcome in ('Failed', 'TimedOut'):
            for committed in (True, False):
                with self.subTest(outcome=outcome, committed=committed):
                    service = ApprovedService()
                    service.write_status, service.commit = outcome, committed
                    status, _ = self.run_approved(service)
                    self.assertEqual(status, 'Succeeded' if committed else 'Failed')
                    self.assertEqual(service.writes, 1)
                    self.assertIn('Read_GitLab_after_BETA', service.calls)
    def test_approved_missing_denied_or_timed_out_reads_never_write(self):
        for status, code in (('Failed', 404), ('Failed', 403), ('Failed', 429), ('TimedOut', 504)):
            with self.subTest(status=status, code=code):
                service = ApprovedService()
                service.initial_status, service.initial_code = status, code
                self.assertEqual(self.run_approved(service)[0], 'Failed')
                self.assertEqual(service.writes, 0)
    def test_approved_newer_divergent_or_partial_destination_never_overwritten(self):
        for content in (NEW + b'{"extra":true}\n', b'{"different":true}\n', OLD[:-1], OLD.replace(b'synthetic-old', b'SYNTHETIC-OLD')):
            with self.subTest(content=content):
                service = ApprovedService()
                service.file = content
                self.assertEqual(self.run_approved(service)[0], 'Failed')
                self.assertEqual(service.writes, 0)
                self.assertEqual(service.file, content)
    def test_approved_oversized_snapshot_never_writes(self):
        service = ApprovedService()
        service.file = b''
        service.snapshot = b'{"note":"' + b'x' * 10485760 + b'"}\n'
        content = encoded(service.snapshot)
        service.receipt.update({
            'byteCount': len(service.snapshot), 'lineCount': 1,
            'candidateFingerprint': 'b64-edge-v1:' + str(len(content)) + ':' + content[:64] + ':' + content[-64:],
        })
        status, context = self.run_approved(service)
        self.assertEqual(status, 'Failed')
        self.assertEqual(context['error'], 'UnverifiedApprovedExport')
        self.assertEqual(service.writes, 0)
    def test_approved_receipt_and_snapshot_guards(self):
        changes = (
            ('verifiedAtUtc', '2026-11-21T00:00:00Z'), ('verifiedAtUtc', '2026-11-25T00:00:00Z'),
            ('verifiedAtUtc', 'bad-date'), ('byteCount', len(NEW) - 1), ('byteCount', str(len(NEW))),
            ('fileRevision', 'wrong'), ('fileIdentity', '/wrong'), ('lastApprovedBatchId', None),
            ('candidateFingerprint', 'wrong'), ('batchCount', 0),
            ('lineCount', 1), ('runId', ' '), ('lastApprovedBatchId', ''),
        )
        for key, value in changes:
            with self.subTest(key=key, value=value):
                service = ApprovedService()
                service.receipt[key] = value
                self.assertEqual(self.run_approved(service)[0], 'Failed')
                self.assertEqual(service.writes, 0)
        service = ApprovedService()
        service.before_etag = '"earlier"'
        self.assertEqual(self.run_approved(service)[0], 'Failed')
        self.assertEqual(service.writes, 0)
    def test_approved_unreadable_readback_never_claims_success(self):
        for outcome in ('Failed', 'TimedOut'):
            service = ApprovedService()
            service.read_status = outcome
            self.assertEqual(self.run_approved(service)[0], 'Failed')
    def test_catalog_changed_then_replay_preserves_general(self):
        service = CatalogService()
        general = deepcopy(service.rows[1])
        self.assertEqual(self.run_catalog(service)[0], 'Succeeded')
        self.assertEqual(service.writes, 2)
        self.assertEqual(service.rows[1], general)
        self.assertEqual(self.run_catalog(service)[0], 'Succeeded')
        self.assertEqual(service.writes, 2)
    def test_catalog_conditional_conflict_and_readback_failure(self):
        service = CatalogService()
        service.conflict = True
        self.assertEqual(self.run_catalog(service)[0], 'Failed')
        self.assertEqual(service.rows[0]['Title'], 'Old Alpha')
        service = CatalogService()
        service.read_status = 'TimedOut'
        self.assertEqual(self.run_catalog(service)[0], 'Failed')
    def test_catalog_fallback_ids_preserve_existing_display_titles(self):
        service = CatalogService()
        service.projects[0]['label'] = 'alpha'
        self.assertEqual(self.run_catalog(service)[0], 'Succeeded')
        self.assertEqual(service.rows[0]['Title'], 'Old Alpha')
        self.assertEqual(service.rows[0]['SourceRevision'], 'source-commit')
    def test_catalog_lost_write_response_reconciles(self):
        for outcome in ('Failed', 'TimedOut'):
            service = CatalogService()
            service.write_status = outcome
            self.assertEqual(self.run_catalog(service)[0], 'Succeeded')
            self.assertEqual(service.writes, 2)
    def test_catalog_preflights_all_rows_before_any_write(self):
        for case in ('duplicate-source', 'duplicate-target', 'missing-etag', 'removal', 'empty-id', 'nonunique', 'too-many', 'reserved'):
            with self.subTest(case=case):
                service = CatalogService()
                if case == 'duplicate-source':
                    service.projects.append({'id': 'ALPHA', 'label': 'Duplicate'})
                elif case == 'duplicate-target':
                    service.rows.append({**service.rows[0], 'ID': 3, 'ProjectId': 'ALPHA'})
                elif case == 'missing-etag':
                    del service.rows[0]['@odata.etag']
                elif case == 'removal':
                    service.rows.append({**service.rows[0], 'ID': 3, 'ProjectId': 'removed'})
                elif case == 'empty-id':
                    service.projects[1]['id'] = ' '
                elif case == 'nonunique':
                    service.unique = False
                elif case == 'too-many':
                    service.projects *= 501
                elif case == 'reserved':
                    service.projects[1]['id'] = 'General'
                self.assertEqual(self.run_catalog(service)[0], 'Failed')
                self.assertEqual(service.writes, 0)
    def test_catalog_projected_capacity_includes_general(self):
        service = CatalogService()
        service.projects = [{'id': 'alpha', 'label': 'Alpha'}] + [
            {'id': f'project-{index}', 'label': f'Project {index}'} for index in range(999)
        ]
        self.assertEqual(self.run_catalog(service)[0], 'Failed')
        self.assertEqual(service.writes, 0)
    def test_disabled_flows_never_call_connectors(self):
        def no_calls(*_):
            self.fail('Disabled flow called a connector')
        for name in ('catalog', 'approved'):
            graph = load_builder('build-' + name + '-sync.py').build(CONFIG)
            self.assertEqual(execute(graph['operationDefinition']['actions'], no_calls)[0], 'Failed')
    def test_incomplete_pages_never_write(self):
        for service, run in ((ApprovedService(), self.run_approved), (CatalogService(), self.run_catalog)):
            service.next_link = 'https://example.invalid/next'
            self.assertEqual(run(service)[0], 'Failed')
            self.assertEqual(service.writes, 0)
    def test_workflow_serialization_and_disabled_defaults(self):
        for name, minute in (('catalog', 0), ('approved', 30)):
            definition = workflow('build-' + name + '-sync.py', minute)
            trigger = definition['triggers']['Recurrence']
            self.assertEqual(trigger['runtimeConfiguration']['concurrency']['runs'], 1)
            self.assertEqual(trigger['recurrence']['schedule'], {'hours': [3], 'minutes': [minute]})
            actions = flatten(definition['actions'])
            self.assertEqual(actions['Sync_configuration_BETA']['inputs']['catalogSyncReady'], False)
            self.assertEqual(actions['Sync_configuration_BETA']['inputs']['approvedSourceReady'], False)
            self.assertEqual(actions['Sync_configuration_BETA']['inputs']['gitLabWriteReady'], False)
    def test_workflow_success_record_requires_verified_scope(self):
        for name, service in (('catalog', CatalogService()), ('approved', ApprovedService())):
            definition = workflow('build-' + name + '-sync.py', 0)
            actions = flatten(definition['actions'])
            actions['Sync_configuration_BETA']['inputs'] = {**ready(), 'maxReceiptAgeHours': 48}
            service.write_status = 'TimedOut'
            status, context = execute(definition['actions'], service)
            self.assertEqual(status, 'Succeeded')
            self.assertEqual(context['outputs']['Sync_verified_BETA']['status'], 'verified')
            self.assertTrue(context['outputs']['Sync_verified_BETA']['sourceRevision'])
            if name == 'approved':
                self.assertEqual(context['outputs']['Sync_verified_BETA']['destinationRevision'], service.revision)
        definition = workflow('build-approved-sync.py', 0)
        actions = flatten(definition['actions'])
        actions['Sync_configuration_BETA']['inputs'] = {**ready(), 'maxReceiptAgeHours': 48}
        service = ApprovedService()
        service.read_status = 'TimedOut'
        status, context = execute(definition['actions'], service)
        self.assertEqual(status, 'Failed')
        self.assertNotEqual(context['actions'].get('Sync_verified_BETA', {}).get('status'), 'Succeeded')


if __name__ == '__main__':
    unittest.main()
