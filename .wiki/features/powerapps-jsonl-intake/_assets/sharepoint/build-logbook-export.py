"""Build the scheduled, serialized SharePoint list-to-JSONL snapshot action."""
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

ROOT = Path(__file__).resolve().parent
SITE = 'https://ornl.sharepoint.com/sites/ResearchEnablement'
LIST = 'a904a6c6-5dd0-427b-9575-8167c4c0c412'
SECURE = {'secureData': {'properties': ['inputs', 'outputs']}}


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


def main():
    raw="base64ToString(concat(item()?['PayloadBase64A'],coalesce(item()?['PayloadBase64B'],string(null))))"
    content="if(empty(body('Select_original_rows_LB')),string(null),concat(join(body('Select_original_rows_LB'),decodeUriComponent('%0A')),decodeUriComponent('%0A')))"
    b64="base64(outputs('Combined_JSONL_LB'))"
    byte_count=f"sub(div(mul(length({b64}),3),4),if(endsWith({b64},'=='),2,if(endsWith({b64},'='),1,0)))"
    write=action('HttpRequest',{
        'parameters/method':'POST',
        'parameters/uri':"_api/web/GetFolderByServerRelativeUrl('/sites/ResearchEnablement/Sponsor Highlights JSONL Pilot')/Files/add(url='logbook.jsonl',overwrite=true)",
        'parameters/headers':{'Accept':'application/json;odata=nometadata','Content-Type':'application/octet-stream'},
        'parameters/body':"@outputs('Combined_JSONL_LB')"})
    publish={
      'Read_previous_logbook_LB':action('GetFileContentByPath',{'path':'/Sponsor Highlights JSONL Pilot/logbook.jsonl','inferContentType':False}),
      'Changed_logbook_LB':{'type':'If','expression':{'not':{'equals':["@body('Read_previous_logbook_LB')?['$content']",'@'+b64]}},
         'actions':{'Write_logbook_LB':write},'else':{'actions':{}},
         'runAfter':{'Read_previous_logbook_LB':['Succeeded','Failed','Skipped','TimedOut']}},
      'Read_published_logbook_LB':action('GetFileContentByPath',{'path':'/Sponsor Highlights JSONL Pilot/logbook.jsonl','inferContentType':False},'Changed_logbook_LB'),
      'Exact_logbook_LB':guard({'equals':["@body('Read_published_logbook_LB')?['$content']",'@'+b64]},
         {'Export_receipt_LB':{'type':'Compose','inputs':{'rows':"@length(body('Read_receipts_LB')?['value'])",'highWaterItemId':"@outputs('High_water_item_LB')",'verifiedAt':'@utcNow()','url':SITE+'/Sponsor%20Highlights%20JSONL%20Pilot/logbook.jsonl'},'runAfter':{}}},
         'LogbookReadbackMismatch','Combined logbook readback did not match the exact source rows.','Read_published_logbook_LB')}
    build={
      'Select_original_rows_LB':{'type':'Select','inputs':{'from':"@body('Read_receipts_LB')?['value']",'select':'@'+raw},'runAfter':{},'runtimeConfiguration':SECURE},
      'Combined_JSONL_LB':compose('@'+content,'Select_original_rows_LB'),
      'Within_byte_limit_LB':guard({'lessOrEquals':['@'+byte_count,10485760]},publish,'LogbookTooLarge','Export exceeds the 10 MiB ACORN source limit; previous logbook retained.','Combined_JSONL_LB')}
    actions={
      'Read_latest_receipt_LB':action('GetItems',{'table':LIST,'$orderby':'ID desc','$top':1}),
      'High_water_item_LB':compose("@if(empty(body('Read_latest_receipt_LB')?['value']),0,first(body('Read_latest_receipt_LB')?['value'])?['ID'])",'Read_latest_receipt_LB'),
      'Read_receipts_LB':action('GetItems',{'table':LIST,'$filter':"@concat('ID le ',string(outputs('High_water_item_LB')))",'$orderby':'ID asc','$top':5001},'High_water_item_LB'),
      'Invalid_rows_LB':{'type':'Query','inputs':{'from':"@body('Read_receipts_LB')?['value']",'where':"@or(empty(item()?['SubmissionId']),empty(item()?['PayloadBase64A']),contains("+raw+",decodeUriComponent('%0A')),contains("+raw+",decodeUriComponent('%0D')),not(equals(json("+raw+")?['submissionId'],item()?['SubmissionId'])),not(equals(json("+raw+")?['projectId'],item()?['ProjectId'])))"},'runAfter':{'Read_receipts_LB':['Succeeded']},'runtimeConfiguration':SECURE},
      'Complete_valid_rows_LB':guard({'and':[{'lessOrEquals':["@length(body('Read_receipts_LB')?['value'])",5000]},{'equals':["@length(body('Invalid_rows_LB'))",0]}]},build,'LogbookInvalidOrTruncated','Receipt limit or invalid stored payload; previous logbook retained.','Invalid_rows_LB')}
    actions['Read_receipts_LB']['runtimeConfiguration']['paginationPolicy']={'minimumItemCount':5001}
    references=json.loads((ROOT/'SharePoint_intake_SP.clipboard.json').read_text())['connectionReferences']
    definition={'type':'Scope','actions':actions,'runAfter':{}}
    result={'id':str(uuid5(NAMESPACE_URL,SITE+'/Combined_logbook_export_LB')),'brandColor':'#8C6CFF','connectionReferences':references,
            'connectorDisplayName':'Control','isTrigger':False,'operationName':'Combined_logbook_export_LB','operationDefinition':definition}
    (ROOT/'Combined_logbook_export_LB.clipboard.json').write_text(json.dumps(result,indent=2)+'\n')


if __name__=='__main__':main()
