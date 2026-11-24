"""Offline expression and change-boundary checks; hosted loop behavior is separate."""
import copy
import importlib.util
import json
from pathlib import Path
import sys
import unittest
import zipfile

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT.parent / 'gitlab-sync'))
from test_runtime import evaluate

SPEC = importlib.util.spec_from_file_location('exporter', ROOT.parent / 'build-logbook-export.py')
EXPORTER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EXPORTER)
BEFORE = json.loads((ROOT / 'before.definition.json').read_text())
SCOPE = BEFORE['properties']['definition']['actions']['Combined_logbook_export_LB']
PATCHED = EXPORTER.retry_latest_receipt(SCOPE['actions'])
LOOP = PATCHED['Retry_latest_receipt_LB']
MESSAGE = "Http request failed as there is an error: 'An error occurred while sending the request.'."


def stops(status, code=None, message='', error_code=''):
    context = {
        'actions': {'Read_latest_receipt_LB': {
            'status': status, 'error': {'code': error_code, 'message': message},
        }},
        'outputs': {'Read_latest_receipt_LB': {'statusCode': code}},
    }
    return evaluate(LOOP['expression'], context)


class RetryTests(unittest.TestCase):
    def test_exact_transport_failure_retries(self):
        self.assertFalse(stops('Failed', 400, MESSAGE, 'BadRequest'))
        self.assertFalse(stops('Failed', None, MESSAGE, 'BadRequest'))
    def test_generated_artifact_matches(self):
        artifact = json.loads((ROOT.parent / 'Combined_logbook_export_LB.clipboard.json').read_text())
        self.assertEqual(artifact, EXPORTER.build())
    def test_initial_read_and_publication_are_preserved(self):
        original = copy.deepcopy(SCOPE['actions'])
        restored = copy.deepcopy(PATCHED)
        wrapped = restored.pop('Retry_latest_receipt_LB')
        restored['Read_latest_receipt_LB'] = wrapped['actions']['Read_latest_receipt_LB']
        restored['Read_latest_receipt_LB']['runAfter'] = original['Read_latest_receipt_LB']['runAfter']
        restored['High_water_item_LB']['runAfter'] = original['High_water_item_LB']['runAfter']
        self.assertEqual(restored, original)
        self.assertEqual(SCOPE['actions'], original)
    def test_package_changes_only_the_read_wrapper(self):
        expected = copy.deepcopy(BEFORE)
        expected['properties']['definition']['actions']['Combined_logbook_export_LB']['actions'] = PATCHED
        self.assertEqual(json.loads((ROOT / 'after.definition.json').read_text()), expected)
        with zipfile.ZipFile(ROOT / 'before.native.zip') as before, zipfile.ZipFile(ROOT / 'repair.import.zip') as after:
            self.assertEqual(before.namelist(), after.namelist())
            for name in before.namelist():
                with self.subTest(name=name):
                    if name.endswith('/definition.json'):
                        self.assertEqual(json.loads(after.read(name)), expected)
                    else:
                        self.assertEqual(before.read(name), after.read(name))
    def test_limits_and_success_dependency(self):
        self.assertEqual(LOOP['limit'], {'count': 3, 'timeout': 'PT2M'})
        self.assertEqual(LOOP['operationOptions'], 'FailWhenLimitsReached')
        self.assertEqual(PATCHED['High_water_item_LB']['runAfter'], {'Retry_latest_receipt_LB': ['Succeeded']})
        delay = LOOP['actions']['Wait_before_receipt_retry_LB']
        self.assertEqual(delay['expression'], {'greater': ["@iterationIndexes('Retry_latest_receipt_LB')", 0]})
        self.assertEqual(delay['actions']['Receipt_retry_delay_LB']['inputs']['interval'], {'count': 10, 'unit': 'Second'})
    def test_missing_or_permanent_errors_stop(self):
        for code in (None, 400, 401, 403, 404, 409, 422):
            with self.subTest(code=code):
                self.assertTrue(stops('Failed', code, 'Invalid request', 'BadRequest'))
        self.assertTrue(stops('Failed', 403, MESSAGE, 'Forbidden'))
    def test_success_stops(self):
        self.assertTrue(stops('Succeeded', 200))
    def test_timeouts_throttling_and_server_errors_retry(self):
        self.assertFalse(stops('TimedOut'))
        for code in (408, 429, 500, 502, 503, 504, 599):
            with self.subTest(code=code):
                self.assertFalse(stops('Failed', code))


if __name__ == '__main__':
    unittest.main()
