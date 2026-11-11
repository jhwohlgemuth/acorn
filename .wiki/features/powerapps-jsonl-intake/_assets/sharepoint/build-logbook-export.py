"""Build the scheduled SharePoint JSONL exporters and validate approved candidates."""
import base64
import binascii
import hashlib
import json
from datetime import datetime
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

ROOT = Path(__file__).resolve().parent
SITE = 'https://ornl.sharepoint.com/sites/ResearchEnablement'
LIST = 'a904a6c6-5dd0-427b-9575-8167c4c0c412'
SECURE = {'secureData': {'properties': ['inputs', 'outputs']}}


class ApprovedExportError(ValueError):
    """A candidate validation failure that must leave the published file untouched."""

    def __init__(self, code):
        super().__init__(code)
        self.code = code


def _jsonl_object(value):
    if not value or b'\n' in value or b'\r' in value or len(value) > 65535:
        raise ApprovedExportError('InvalidJSONLRow')
    try:
        parsed = json.loads(value.decode('utf-8'))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ApprovedExportError('InvalidJSONLRow') from error
    if not isinstance(parsed, dict) or not isinstance(parsed.get('submissionId'), str) or not parsed['submissionId']:
        raise ApprovedExportError('InvalidJSONLRow')
    return parsed


def _fingerprint(value):
    encoded = base64.b64encode(value).decode('ascii')
    return f'b64-edge-v1:{len(encoded)}:{encoded[:64]}:{encoded[-64:]}'


def approved_candidate(baseline, batches, frozen_files):
    """Return exact baseline plus Approved frozen rows, or raise before publication."""
    if baseline and (not baseline.endswith(b'\n') or baseline.endswith(b'\n\n')):
        raise ApprovedExportError('InvalidBaseline')
    baseline_rows = baseline[:-1].split(b'\n') if baseline else []
    if len(baseline_rows) > 5000 or len(baseline) > 10485760:
        raise ApprovedExportError('ExportBoundExceeded')
    seen = set()
    for row in baseline_rows:
        submission_id = _jsonl_object(row)['submissionId']
        if submission_id in seen:
            raise ApprovedExportError('DuplicateSubmissionId')
        seen.add(submission_id)
    approved = [batch for batch in batches if batch['state'] == 'Approved']
    if any(not batch.get('batchId') or not batch.get('approvedAtUtc') for batch in approved):
        raise ApprovedExportError('InvalidBatch')
    if len({batch['batchId'] for batch in approved}) != len(approved):
        raise ApprovedExportError('DuplicateBatchId')
    approved.sort(key=lambda batch: (batch['approvedAtUtc'], batch['batchId']))
    candidate = bytearray(baseline)
    count = len(baseline_rows)
    for batch in approved:
        try:
            datetime.fromisoformat(batch['approvedAtUtc'].replace('Z', '+00:00'))
            manifest_bytes = batch['manifestBytes']
            manifest = json.loads(manifest_bytes.decode('utf-8'))
        except (KeyError, ValueError, UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ApprovedExportError('InvalidManifest') from error
        included = manifest.get('included')
        excluded = manifest.get('excluded')
        if (not isinstance(included, list) or not included or not isinstance(excluded, list)
                or manifest.get('batchId') != batch['batchId']
                or manifest.get('sourceListId') != batch.get('sourceListId')
                or manifest.get('sourceCount') != batch.get('sourceCount')
                or len(included) != batch.get('approvedCount')
                or len(excluded) != batch.get('excludedCount')
                or len(included) + len(excluded) != batch.get('sourceCount')
                or batch.get('unresolvedCount') != 0
                or batch.get('itemCount') != batch.get('sourceCount')
                or not batch.get('periodKey')
                or not batch.get('periodStartUtc')
                or not batch.get('periodEndUtc')
                or not batch.get('policyRevision')
                or not batch.get('approvedByEmail')
                or batch.get('projectionDigest') != _fingerprint(manifest_bytes)):
            raise ApprovedExportError('InvalidManifest')
        source_ids = [row.get('sourceItemId') for row in included + excluded]
        if any(type(source_id) is not int or source_id < 1 for source_id in source_ids):
            raise ApprovedExportError('InvalidManifest')
        if len(set(source_ids)) != len(source_ids):
            raise ApprovedExportError('DuplicateSourceItemId')
        included_ids = [row.get('sourceItemId') for row in included]
        if included_ids != sorted(included_ids):
            raise ApprovedExportError('InvalidItemOrder')
        for exclusion in excluded:
            if not isinstance(exclusion.get('reason'), str) or not exclusion['reason'].strip():
                raise ApprovedExportError('InvalidExclusion')
        for row in included:
            if row.get('decision') != 'Approve':
                raise ApprovedExportError('InvalidDecision')
            encoded = row.get('payloadBase64')
            try:
                frozen = base64.b64decode(encoded, validate=True)
            except (TypeError, ValueError, binascii.Error) as error:
                raise ApprovedExportError('InvalidFrozenBytes') from error
            if frozen_files.get((batch['batchId'], row['sourceItemId'])) != frozen:
                raise ApprovedExportError('FrozenBytesChanged')
            projected = _jsonl_object(frozen)
            approved_id = row.get('approvedSubmissionId')
            if (projected['submissionId'] != approved_id or approved_id in seen
                    or manifest.get('mapping', {}).get(row.get('sourceSubmissionId')) != approved_id):
                raise ApprovedExportError('DuplicateSubmissionId')
            revision = projected.get('revision')
            predecessor = row.get('approvedSupersedes')
            if revision:
                if (not isinstance(revision, dict)
                        or revision.get('supersedesSubmissionId') != predecessor
                        or predecessor not in seen
                        or revision.get('rootSubmissionId') not in seen):
                    raise ApprovedExportError('InvalidAncestry')
            elif predecessor is not None:
                raise ApprovedExportError('InvalidAncestry')
            seen.add(approved_id)
            candidate.extend(frozen)
            candidate.extend(b'\n')
            count += 1
            if count > 5000 or len(candidate) > 10485760:
                raise ApprovedExportError('ExportBoundExceeded')
    return bytes(candidate)


def action(operation, parameters, dependency=None, statuses=None):
    return {'type': 'OpenApiConnection', 'inputs': {
        'host': {'connectionName': 'shared_sharepointonline', 'operationId': operation,
                 'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline'},
        'parameters': {'dataset': SITE, **parameters},
        'authentication': "@parameters('$authentication')", 'retryPolicy': {'type': 'none'}},
        'runAfter': {dependency: statuses or ['Succeeded']} if dependency else {},
        'runtimeConfiguration': {'secureData': {'properties': ['inputs', 'outputs']}}}


def compose(value, dependency):
    return {'type': 'Compose', 'inputs': value, 'runAfter': {dependency: ['Succeeded']},
            'runtimeConfiguration': {'secureData': {'properties': ['inputs']}}}


def fail(code, message):
    return {'type': 'Terminate', 'inputs': {'runStatus': 'Failed', 'runError': {'code': code, 'message': message}}, 'runAfter': {}}


def guard(expression, actions, code, message, dependency):
    return {'type': 'If', 'expression': expression, 'actions': actions,
            'else': {'actions': {code: fail(code, message)}}, 'runAfter': {dependency: ['Succeeded']}}


def file_content(name):
    """Normalize a proven zero-byte connector response without accepting missing data."""
    body="body('"+name+"')?['$content']"
    return ("@if(and(equals(actions('"+name+"')?['status'],'Succeeded'),"
            "equals(outputs('"+name+"')?['statusCode'],200),"
            "equals(string(outputs('"+name+"')?['headers']?['Content-Length']),'0'),"
            "equals("+body+",null)),string(null),"+body+")")


def build(receipt_list=LIST, library='Sponsor Highlights JSONL Pilot'):
    raw="base64ToString(concat(item()?['PayloadBase64A'],coalesce(item()?['PayloadBase64B'],string(null))))"
    content="if(empty(body('Select_original_rows_LB')),string(null),concat(join(body('Select_original_rows_LB'),decodeUriComponent('%0A')),decodeUriComponent('%0A')))"
    b64="base64(outputs('Combined_JSONL_LB'))"
    byte_count=f"sub(div(mul(length({b64}),3),4),if(endsWith({b64},'=='),2,if(endsWith({b64},'='),1,0)))"
    previous_status="@actions('Read_previous_logbook_LB')?['status']"
    previous_content=file_content('Read_previous_logbook_LB')
    confirmed_missing={'and':[
        {'equals':[previous_status,'Failed']},
        {'equals':["@outputs('Read_previous_logbook_LB')?['statusCode']",404]}]}
    readable={'and':[
        {'equals':[previous_status,'Succeeded']},
        {'not':{'equals':[previous_content,None]}}]}
    write=action('HttpRequest',{
        'parameters/method':'POST',
        'parameters/uri':"_api/web/GetFolderByServerRelativeUrl('/sites/ResearchEnablement/"+library+"')/Files/add(url='logbook.jsonl',overwrite=@{if(equals(actions('Read_previous_logbook_LB')?['status'],'Succeeded'),'true','false')})",
        'parameters/headers':{'Accept':'application/json;odata=nometadata','Content-Type':'application/octet-stream'},
        'parameters/body':"@outputs('Combined_JSONL_LB')"})
    reconcile={
      'Changed_logbook_LB':{'type':'If','expression':{'not':{'equals':[previous_content,'@'+b64]}},
         'actions':{'Write_logbook_LB':write},'else':{'actions':{}},'runAfter':{}},
      'Read_published_logbook_LB':action('GetFileContentByPath',{'path':'/'+library+'/logbook.jsonl','inferContentType':False},
         'Changed_logbook_LB',['Succeeded','Failed','TimedOut']),
      'Exact_logbook_LB':guard({'and':[
         {'equals':["@actions('Read_published_logbook_LB')?['status']",'Succeeded']},
         {'equals':[file_content('Read_published_logbook_LB'),'@'+b64]}]},
         {'Export_receipt_LB':{'type':'Compose','inputs':{
             'rows':"@length(body('Read_receipts_LB')?['value'])",
             'highWaterItemId':"@outputs('High_water_item_LB')",'verifiedAt':'@utcNow()',
             'url':SITE+'/'+library.replace(' ','%20')+'/logbook.jsonl'},'runAfter':{}}},
         'LogbookOutcomeUnresolved','Combined logbook outcome is unresolved. Read and compare exact expected bytes before retrying; the prior file is not guaranteed retained. Do not roll back over a later snapshot.',
         'Read_published_logbook_LB')}
    reconcile['Exact_logbook_LB']['runAfter']['Read_published_logbook_LB']=['Succeeded','Failed','TimedOut','Skipped']
    publish={
      'Read_previous_logbook_LB':action('GetFileContentByPath',{'path':'/'+library+'/logbook.jsonl','inferContentType':False}),
      'Readable_or_missing_LB':guard({'or':[readable,confirmed_missing]},reconcile,
         'LogbookPreviousReadUnavailable','Previous logbook was not read and no confirmed 404 was returned. No write was attempted.',
         'Read_previous_logbook_LB')}
    publish['Readable_or_missing_LB']['runAfter']['Read_previous_logbook_LB']=['Succeeded','Failed','TimedOut','Skipped']
    build={
      'Select_original_rows_LB':{'type':'Select','inputs':{'from':"@body('Read_receipts_LB')?['value']",'select':'@'+raw},'runAfter':{},'runtimeConfiguration':SECURE},
      'Combined_JSONL_LB':compose('@'+content,'Select_original_rows_LB'),
      'Within_byte_limit_LB':guard({'lessOrEquals':['@'+byte_count,10485760]},publish,'LogbookTooLarge','Export exceeds the 10 MiB ACORN source limit; previous logbook retained.','Combined_JSONL_LB')}
    actions={
      'Read_latest_receipt_LB':action('GetItems',{'table':receipt_list,'$orderby':'ID desc','$top':1}),
      'High_water_item_LB':compose("@if(empty(body('Read_latest_receipt_LB')?['value']),0,first(body('Read_latest_receipt_LB')?['value'])?['ID'])",'Read_latest_receipt_LB'),
      'Read_receipts_LB':action('GetItems',{'table':receipt_list,'$filter':"@concat('ID le ',string(outputs('High_water_item_LB')))",'$orderby':'ID asc','$top':5001},'High_water_item_LB'),
      'Invalid_rows_LB':{'type':'Query','inputs':{'from':"@body('Read_receipts_LB')?['value']",'where':"@or(empty(item()?['SubmissionId']),empty(item()?['PayloadBase64A']),contains("+raw+",decodeUriComponent('%0A')),contains("+raw+",decodeUriComponent('%0D')),not(equals(json("+raw+")?['submissionId'],item()?['SubmissionId'])),not(equals(json("+raw+")?['projectId'],item()?['ProjectId'])))"},'runAfter':{'Read_receipts_LB':['Succeeded']},'runtimeConfiguration':SECURE},
      'Complete_valid_rows_LB':guard({'and':[{'lessOrEquals':["@length(body('Read_receipts_LB')?['value'])",5000]},{'equals':["@length(body('Invalid_rows_LB'))",0]}]},build,'LogbookInvalidOrTruncated','Receipt limit or invalid stored payload; previous logbook retained.','Invalid_rows_LB')}
    actions['Read_receipts_LB']['runtimeConfiguration']['paginationPolicy']={'minimumItemCount':5001}
    references=json.loads((ROOT/'SharePoint_intake_SP.clipboard.json').read_text())['connectionReferences']
    definition={'type':'Scope','actions':actions,'runAfter':{}}
    result={'id':str(uuid5(NAMESPACE_URL,SITE+'/Combined_logbook_export_LB')),'brandColor':'#8C6CFF','connectionReferences':references,
            'connectorDisplayName':'Control','isTrigger':False,'operationName':'Combined_logbook_export_LB','operationDefinition':definition}
    return result


def _assertion(expression, code, dependency):
    return compose("@if(" + expression + ",true,json('PA38_" + code + "'))", dependency)


def _byte_count(encoded):
    return ("sub(div(mul(length(" + encoded + "),3),4),"
            "if(endsWith(" + encoded + ",'=='),2,"
            "if(endsWith(" + encoded + ",'='),1,0)))")


def _variable(name, kind, value, dependency=None):
    return {'type': 'InitializeVariable', 'inputs': {'variables': [
        {'name': name, 'type': kind, 'value': value}]},
        'runAfter': {dependency: ['Succeeded']} if dependency else {}}


def _append(name, value, dependency):
    return {'type': 'AppendToArrayVariable', 'inputs': {'name': name, 'value': value},
            'runAfter': {dependency: ['Succeeded']}}


def _foreach(source, actions, dependency):
    return {'type': 'Foreach', 'foreach': source, 'actions': actions,
            'runAfter': {dependency: ['Succeeded']},
            'runtimeConfiguration': {'concurrency': {'repetitions': 1}}}


def build_approved():
    """Build an isolated Approved-only scheduled scope; never mutate the old scope."""
    exporter = ROOT / 'intake-approval' / 'exporter'
    baseline = json.loads((exporter / 'baseline.json').read_text())
    schema = json.loads((ROOT / 'intake-approval' / 'storage' / 'schema.json').read_text())
    batches = schema['objectIds']['batchesList']
    approved_library = schema['library']['name']
    collected_library = 'Sponsor Highlights Collected Export TEST ONLY'
    expected_baseline = baseline['contentBase64']
    batch = "items('Each_approved_batch_PA38')"
    manifest = "outputs('Manifest_object_PA38')"
    row = "items('Each_approved_row_PA38')"
    row_content = "body('Read_frozen_row_PA38')?['$content']"
    source_id = "string(" + row + "?['sourceItemId'])"
    frozen_path = ("@concat('/" + approved_library + "/Approved/'," + batch
                   + "?['BatchId'],'/'," + source_id + ",'.jsonl')")
    manifest_path = ("@concat('/" + approved_library + "/Approved/'," + batch
                     + "?['BatchId'],'/manifest.json')")
    fingerprint_b64 = "base64(outputs('Manifest_text_PA38'))"
    fingerprint = ("concat('b64-edge-v1:',string(length(" + fingerprint_b64 + "))"
                   ",':',substring(" + fingerprint_b64 + ",0,if(greater(length("
                   + fingerprint_b64 + "),64),64,length(" + fingerprint_b64 + "))),':'"
                   ",substring(" + fingerprint_b64 + ",if(greater(length("
                   + fingerprint_b64 + "),64),sub(length(" + fingerprint_b64
                   + "),64),0),if(greater(length(" + fingerprint_b64
                   + "),64),64,length(" + fingerprint_b64 + "))))")
    row_json = "outputs('Frozen_object_PA38')"
    revision = row_json + "?['revision']"
    row_valid = (
        "and(equals(" + row_content + "," + row + "?['payloadBase64']),"
        "not(contains(base64ToString(" + row_content + "),decodeUriComponent('%0A'))),"
        "not(contains(base64ToString(" + row_content + "),decodeUriComponent('%0D'))),"
        "lessOrEquals(" + _byte_count(row_content) + ",65535),"
        "equals(" + row_json + "?['submissionId']," + row + "?['approvedSubmissionId']),"
        "equals(" + row + "?['decision'],'Approve'),"
        "equals(" + manifest + "?['mapping']?[" + row + "?['sourceSubmissionId']],"
        + row + "?['approvedSubmissionId']),"
        "not(contains(variables('SeenIdsPA38')," + row + "?['approvedSubmissionId'])),"
        "not(contains(variables('SeenSourceIdsPA38')," + source_id + ")),"
        "greater(int(" + row + "?['sourceItemId']),variables('PreviousIncludedSourceIdPA38')),"
        "or(and(empty(" + revision + "),empty(" + row + "?['approvedSupersedes'])),and("
        "equals(" + revision + "?['supersedesSubmissionId']," + row + "?['approvedSupersedes']),"
        "contains(variables('SeenIdsPA38')," + revision + "?['supersedesSubmissionId']),"
        "contains(variables('SeenIdsPA38')," + revision + "?['rootSubmissionId']))))"
    )
    included = {
        'Read_frozen_row_PA38': action('GetFileContentByPath', {'path': frozen_path,
                                'inferContentType': False}),
        'Frozen_object_PA38': compose('@json(base64ToString(' + row_content + '))',
                                      'Read_frozen_row_PA38'),
        'Valid_frozen_row_PA38': _assertion(row_valid, 'InvalidFrozenRow', 'Frozen_object_PA38'),
        'Remember_included_order_PA38': {'type': 'SetVariable',
            'inputs': {'name': 'PreviousIncludedSourceIdPA38',
                       'value': '@int(' + row + "?['sourceItemId'])"},
            'runAfter': {'Valid_frozen_row_PA38': ['Succeeded']}},
        'Remember_source_PA38': _append('SeenSourceIdsPA38', '@' + source_id,
                                       'Remember_included_order_PA38'),
        'Remember_id_PA38': _append('SeenIdsPA38', '@' + row + "?['approvedSubmissionId']",
                                   'Remember_source_PA38'),
        'Remember_line_PA38': _append('ApprovedLinesPA38',
                                     '@base64ToString(' + row_content + ')',
                                     'Remember_id_PA38'),
    }
    excluded_row = "items('Each_excluded_row_PA38')"
    excluded = {
        'Valid_exclusion_PA38': _assertion(
            "and(greater(int(" + excluded_row + "?['sourceItemId']),0),"
            "not(empty(trim(coalesce(" + excluded_row + "?['reason'],'')))),"
            "not(contains(variables('SeenSourceIdsPA38'),string(" + excluded_row
            + "?['sourceItemId']))))", 'InvalidExclusion', None),
        'Remember_exclusion_PA38': _append('SeenSourceIdsPA38',
                                           '@string(' + excluded_row + "?['sourceItemId'])",
                                           'Valid_exclusion_PA38'),
    }
    excluded['Valid_exclusion_PA38']['runAfter'] = {}
    manifest_valid = (
        "and(equals(" + manifest + "?['batchId']," + batch + "?['BatchId']),"
        "equals(" + manifest + "?['sourceListId']," + batch + "?['SourceListId']),"
        "equals(int(" + manifest + "?['sourceCount']),int(" + batch + "?['SourceCount'])),"
        "equals(int(" + batch + "?['ItemCount']),int(" + batch + "?['SourceCount'])),"
        "equals(length(" + manifest + "?['included']),int(" + batch + "?['ApprovedCount'])),"
        "equals(length(" + manifest + "?['excluded']),int(" + batch + "?['ExcludedCount'])),"
        "equals(add(length(" + manifest + "?['included']),length(" + manifest
        + "?['excluded'])),int(" + batch + "?['SourceCount'])),"
        "equals(int(" + batch + "?['UnresolvedCount']),0),"
        "greater(length(" + manifest + "?['included']),0),"
        "not(empty(" + batch + "?['ApprovedByEmail'])),"
        "not(empty(" + batch + "?['ApprovedAtUtc'])),"
        "not(empty(" + batch + "?['PeriodKey'])),"
        "not(empty(" + batch + "?['PeriodStartUtc'])),"
        "not(empty(" + batch + "?['PeriodEndUtc'])),"
        "not(empty(" + batch + "?['PolicyRevision'])),"
        "equals(outputs('Manifest_fingerprint_PA38')," + batch + "?['ProjectionDigest']))"
    )
    each_batch = {
        'Unique_batch_PA38': _assertion(
            "and(equals(" + batch + "?['State']?['Value'],'Approved'),"
            "not(contains(variables('SeenBatchIdsPA38')," + batch + "?['BatchId'])))",
            'DuplicateOrUnapprovedBatch', None),
        'Remember_batch_PA38': _append('SeenBatchIdsPA38', '@' + batch + "?['BatchId']",
                                       'Unique_batch_PA38'),
        'Reset_included_order_PA38': {'type': 'SetVariable',
            'inputs': {'name': 'PreviousIncludedSourceIdPA38', 'value': 0},
            'runAfter': {'Remember_batch_PA38': ['Succeeded']}},
        'Reset_source_ids_PA38': {'type': 'SetVariable',
                                  'inputs': {'name': 'SeenSourceIdsPA38', 'value': []},
                                  'runAfter': {'Reset_included_order_PA38': ['Succeeded']}},
        'Read_manifest_PA38': action('GetFileContentByPath',
                                     {'path': manifest_path, 'inferContentType': False},
                                     'Reset_source_ids_PA38'),
        'Manifest_text_PA38': compose('@base64ToString(body(\'Read_manifest_PA38\')?[\'$content\'])',
                                      'Read_manifest_PA38'),
        'Manifest_object_PA38': compose("@json(outputs('Manifest_text_PA38'))",
                                        'Manifest_text_PA38'),
        'Manifest_fingerprint_PA38': compose('@' + fingerprint,
                                             'Manifest_object_PA38'),
        'Valid_manifest_PA38': _assertion(manifest_valid, 'InvalidApprovedManifest',
                                           'Manifest_fingerprint_PA38'),
        'Each_excluded_row_PA38': _foreach('@' + manifest + "?['excluded']", excluded,
                                           'Valid_manifest_PA38'),
        'Each_approved_row_PA38': _foreach('@' + manifest + "?['included']", included,
                                           'Each_excluded_row_PA38'),
    }
    each_batch['Unique_batch_PA38']['runAfter'] = {}
    baseline_content = "body('Read_baseline_PA38')?['$content']"
    baseline_row = "items('Each_baseline_row_PA38')"
    baseline_actions = {
        'Baseline_object_PA38': compose('@json(' + baseline_row + ')', None),
        'Valid_baseline_row_PA38': _assertion(
            "and(not(empty(outputs('Baseline_object_PA38')?['submissionId'])),"
            "not(contains(variables('SeenIdsPA38'),outputs('Baseline_object_PA38')?['submissionId'])))",
            'InvalidBaselineRow', 'Baseline_object_PA38'),
        'Remember_baseline_id_PA38': _append('SeenIdsPA38',
                                            "@outputs('Baseline_object_PA38')?['submissionId']",
                                            'Valid_baseline_row_PA38'),
    }
    baseline_actions['Baseline_object_PA38']['runAfter'] = {}
    previous = build(library=collected_library)['operationDefinition']['actions']
    publish = previous['Complete_valid_rows_LB']['actions']['Within_byte_limit_LB']['actions']
    receipt = publish['Readable_or_missing_LB']['actions']['Exact_logbook_LB']['actions']['Export_receipt_LB']
    receipt['inputs'] = {
        'runId': '@workflow()?[' + "'run'" + ']?[' + "'name'" + ']',
        'baselineSha256': baseline['sha256'],
        'batchCount': "@length(variables('SeenBatchIdsPA38'))",
        'lineCount': "@add(length(body('Nonempty_baseline_rows_PA38')),length(variables('ApprovedLinesPA38')))",
        'byteCount': '@' + _byte_count("base64(outputs('Combined_JSONL_LB'))"),
        'candidateFingerprint': '@' + _fingerprint_expression("base64(outputs('Combined_JSONL_LB'))"),
        'lastApprovedBatchId': "@if(empty(variables('SeenBatchIdsPA38')),null,last(variables('SeenBatchIdsPA38')))",
        'fileIdentity': '/sites/ResearchEnablement/' + collected_library + '/logbook.jsonl',
        'fileRevision': "@coalesce(body('Read_published_metadata_PA38')?['ETag'],body('Read_published_metadata_PA38')?['@odata.etag'])",
        'verifiedAtUtc': '@utcNow()',
    }
    receipt_path = ("/" + collected_library + "/export-receipt-@{workflow()?['run']?['name']}.json")
    receipt_uri = ("_api/web/GetFolderByServerRelativeUrl('/sites/ResearchEnablement/"
                   + collected_library + "')/Files/add(url='export-receipt-@{workflow()?['run']?['name']}.json',overwrite=false)")
    receipt_actions = publish['Readable_or_missing_LB']['actions']['Exact_logbook_LB']['actions']
    receipt_actions['Read_published_metadata_PA38'] = {
        'type': 'OpenApiConnection',
        'inputs': {
            'host': {'connectionName': 'shared_sharepointonline',
                     'operationId': 'HttpRequest',
                     'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline'},
            'parameters': {'dataset': SITE, 'parameters/method': 'GET',
                           'parameters/uri': (
                               "_api/web/GetFileByServerRelativeUrl('/sites/ResearchEnablement/"
                               + collected_library + "/logbook.jsonl')?$select=ETag,UniqueId"),
                           'parameters/headers': {
                               'Accept': 'application/json;odata=nometadata'}},
            'authentication': "@parameters('$authentication')",
            'retryPolicy': {'type': 'none'}},
        'runAfter': {}}
    receipt_actions['Exact_metadata_PA38'] = _assertion(
        "not(empty(coalesce(body('Read_published_metadata_PA38')?['ETag'],body('Read_published_metadata_PA38')?['@odata.etag'])))",
        'MissingFileRevision', 'Read_published_metadata_PA38')
    receipt['runAfter'] = {'Exact_metadata_PA38': ['Succeeded']}
    receipt_actions['Publish_receipt_PA38'] = {
        'type': 'OpenApiConnection',
        'inputs': {
            'host': {'connectionName': 'shared_sharepointonline',
                     'operationId': 'HttpRequest',
                     'apiId': '/providers/Microsoft.PowerApps/apis/shared_sharepointonline'},
            'parameters': {'dataset': SITE, 'parameters/method': 'POST',
                           'parameters/uri': receipt_uri,
                           'parameters/headers': {
                               'Accept': 'application/json;odata=nometadata',
                               'Content-Type': 'application/json'},
                           'parameters/body': "@string(outputs('Export_receipt_LB'))"},
            'authentication': "@parameters('$authentication')",
            'retryPolicy': {'type': 'none'}},
        'runAfter': {'Export_receipt_LB': ['Succeeded']}}
    receipt_actions['Read_receipt_PA38'] = action(
        'GetFileContentByPath', {'path': receipt_path, 'inferContentType': False},
        'Publish_receipt_PA38')
    receipt_actions['Exact_receipt_PA38'] = _assertion(
        "equals(body('Read_receipt_PA38')?['$content'],base64(string(outputs('Export_receipt_LB'))))",
        'ReceiptReadbackMismatch', 'Read_receipt_PA38')
    actions = {
        'Initialize_seen_ids_PA38': _variable('SeenIdsPA38', 'Array', []),
        'Initialize_seen_batches_PA38': _variable('SeenBatchIdsPA38', 'Array', [], 'Initialize_seen_ids_PA38'),
        'Initialize_source_ids_PA38': _variable('SeenSourceIdsPA38', 'Array', [], 'Initialize_seen_batches_PA38'),
        'Initialize_lines_PA38': _variable('ApprovedLinesPA38', 'Array', [], 'Initialize_source_ids_PA38'),
        'Initialize_included_order_PA38': _variable('PreviousIncludedSourceIdPA38', 'Integer', 0,
                                                   'Initialize_lines_PA38'),
        'Read_baseline_PA38': action('GetFileContentByPath',
                                     {'path': '/' + collected_library + '/baseline.jsonl',
                                      'inferContentType': False}, 'Initialize_included_order_PA38'),
        'Exact_baseline_PA38': _assertion(
            "equals(" + baseline_content + ",'" + expected_baseline + "')",
            'BaselineChanged', 'Read_baseline_PA38'),
        'Baseline_text_PA38': compose('@base64ToString(' + baseline_content + ')',
                                      'Exact_baseline_PA38'),
        'Nonempty_baseline_rows_PA38': {'type': 'Query',
            'inputs': {'from': "@split(outputs('Baseline_text_PA38'),decodeUriComponent('%0A'))",
                       'where': '@not(empty(item()))'},
            'runAfter': {'Baseline_text_PA38': ['Succeeded']}, 'runtimeConfiguration': SECURE},
        'Each_baseline_row_PA38': _foreach("@body('Nonempty_baseline_rows_PA38')",
                                           baseline_actions, 'Nonempty_baseline_rows_PA38'),
        'Read_approved_batches_PA38': action('GetItems',
            {'table': batches, '$filter': "State eq 'Approved'",
             '$orderby': 'ApprovedAtUtc asc,BatchId asc', '$top': 5001},
            'Each_baseline_row_PA38'),
        'Each_approved_batch_PA38': _foreach("@body('Read_approved_batches_PA38')?['value']",
                                             each_batch, 'Read_approved_batches_PA38'),
        'Combined_JSONL_LB': compose(
            "@concat(outputs('Baseline_text_PA38'),if(empty(variables('ApprovedLinesPA38')),'',"
            "concat(join(variables('ApprovedLinesPA38'),decodeUriComponent('%0A')),decodeUriComponent('%0A'))))",
            'Each_approved_batch_PA38'),
        'Within_candidate_bounds_PA38': guard({'and': [
            {'lessOrEquals': ["@add(length(body('Nonempty_baseline_rows_PA38')),length(variables('ApprovedLinesPA38')))", 5000]},
            {'lessOrEquals': ['@' + _byte_count("base64(outputs('Combined_JSONL_LB'))"), 10485760]}]},
            publish, 'ApprovedExportBoundExceeded', 'Approved export exceeds the total row or byte bound.',
            'Combined_JSONL_LB'),
    }
    actions['Read_approved_batches_PA38']['runtimeConfiguration']['paginationPolicy'] = {'minimumItemCount': 5001}
    initializers = {name: actions.pop(name) for name in (
        'Initialize_seen_ids_PA38', 'Initialize_seen_batches_PA38',
        'Initialize_source_ids_PA38', 'Initialize_lines_PA38',
        'Initialize_included_order_PA38')}
    actions['Read_baseline_PA38']['runAfter'] = {}
    template = build()
    template['operationName'] = 'Approved_logbook_export_PA38'
    template['operationDefinition']['actions'] = actions
    template['operationDefinition']['runAfter'] = {'Initialize_included_order_PA38': ['Succeeded']}
    template['initializers'] = initializers
    template['id'] = str(uuid5(NAMESPACE_URL, SITE + '/Approved_logbook_export_PA38'))
    return template


def _fingerprint_expression(encoded):
    return ("concat('b64-edge-v1:',string(length(" + encoded + ")),':'"
            ",substring(" + encoded + ",0,if(greater(length(" + encoded + "),64),64,length("
            + encoded + "))),':',substring(" + encoded + ",if(greater(length("
            + encoded + "),64),sub(length(" + encoded + "),64),0),"
            "if(greater(length(" + encoded + "),64),64,length(" + encoded + "))))")


def main():
    (ROOT/'Combined_logbook_export_LB.clipboard.json').write_text(json.dumps(build(),indent=2)+'\n')


if __name__=='__main__':main()
