"""Build the isolated PA-38 native import package from the accepted exporter template."""
import importlib.util
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5
from zipfile import ZIP_DEFLATED, ZipFile

ROOT = Path(__file__).resolve().parent
SHAREPOINT = ROOT.parent.parent
TEMPLATE = SHAREPOINT / 'backup' / 'PA24AcceptedRepairedExporter20261027_20261027131627.zip'
NAME = 'Approval Gated Combined Logbook Export TEST ONLY'


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    builder = load(SHAREPOINT / 'build-logbook-export.py', 'pa38_builder')
    validator = load(SHAREPOINT / 'intake-approval' / 'finalization' / 'build-finalization.py', 'pa37_builder')
    result = builder.build_approved()
    initializers = result.pop('initializers')
    actions = {**initializers, result['operationName']: result['operationDefinition']}
    validator.validate_actions(actions)
    (ROOT / 'generated-definition.json').write_text(json.dumps(actions, indent=2) + '\n')
    with ZipFile(TEMPLATE) as source:
        files = {name: source.read(name) for name in source.namelist()}
    definition_path = next(path for path in files if path.endswith('/definition.json'))
    resource_id = definition_path.split('/')[2]
    flow = json.loads(files[definition_path])
    flow_id = str(uuid5(NAMESPACE_URL, 'PA38A/' + NAME))
    flow['name'] = flow_id
    flow['id'] = '/providers/Microsoft.Flow/flows/' + flow_id
    flow['properties']['displayName'] = NAME
    flow['properties']['definition']['actions'] = actions
    trigger = flow['properties']['definition']['triggers']['Recurrence']
    trigger['recurrence'] = {'frequency': 'Day', 'interval': 1,
                             'startTime': '2070-01-01T00:00:00.000Z'}
    trigger['evaluatedRecurrence'] = trigger['recurrence'].copy()
    files[definition_path] = (json.dumps(flow, indent=2) + '\n').encode()
    manifest = json.loads(files['manifest.json'])
    manifest['details']['displayName'] = NAME
    manifest['resources'][resource_id]['details']['displayName'] = NAME
    manifest['resources'][resource_id]['suggestedCreationType'] = 'Update'
    manifest['resources'][resource_id]['creationType'] = 'Existing, New, Update'
    files['manifest.json'] = (json.dumps(manifest, indent=2) + '\n').encode()
    package = ROOT / 'ApprovalGatedCombinedLogbookExportTESTONLY.zip'
    with ZipFile(package, 'w', ZIP_DEFLATED) as target:
        for path in sorted(files):
            target.writestr(path, files[path])
    print(package)


if __name__ == '__main__':
    main()
