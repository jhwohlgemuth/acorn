"""Seed bounded synthetic batches and exercise the real GetItems pagination path."""
import json
from pathlib import Path
import build_acceptance as A
import build_boundaries as B

ROOT=Path(__file__).resolve().parent
TAG='pa23-20261027-page'
PREFIX='23231027-1000-4000-8000-'
CRLF='\r\n'

def raw(n):
    return B.raw_record(n).replace('23231027-0000-4000-8000-',PREFIX)

def batch_body(numbers):
    # Nontransactional by SharePoint design; the native harness checks every 201.
    start='--batch_pa23'+CRLF+'Content-Type: multipart/mixed; boundary=changeset_pa23'+CRLF*2
    parts=[]
    for n in numbers:
        item={'Title':TAG+'-'+str(n),'SubmissionId':PREFIX+f'{n:012d}','ProjectId':'acorn','PayloadBase64A':A.b64(raw(n)),'PayloadBase64B':'','ExportStatus':'Pending'}
        parts.append('--changeset_pa23'+CRLF+'Content-Type: application/http'+CRLF+'Content-Transfer-Encoding: binary'+CRLF*2+'POST '+A.B.SITE+"/_api/web/lists(guid'"+A.LIST+"')/items HTTP/1.1"+CRLF+'Content-Type: application/json;odata=nometadata'+CRLF+'Accept: application/json;odata=nometadata'+CRLF*2+json.dumps(item,separators=(',',':'))+CRLF)
    return start+''.join(parts)+'--changeset_pa23--'+CRLF+'--batch_pa23--'+CRLF

def case(label,count,after,rejected=False):
    result=A.export_case(label,'ID gt 0',A.PRIOR if rejected else '',after,rejected=rejected)
    exp=result['actions']['Exporter_'+label]['actions']
    exp['Read_latest_receipt_LB_'+label]['inputs']['parameters']['$filter']="@concat('ID gt ',string(outputs('Lower_bound')))"
    exp['Read_receipts_LB_'+label]['inputs']['parameters']['$filter']="@concat('ID gt ',string(outputs('Lower_bound')),' and ID le ',string(outputs('High_water_item_LB_"+label+"')))"
    condition=result['actions']['Assert_bytes_'+label]['expression']
    if not rejected:condition['and'][0]['equals'][1]="@base64(outputs('Combined_JSONL_LB_"+label+"'))"
    condition['and'].append({'equals':["@length(body('Read_receipts_LB_"+label+"')?['value'])",count]})
    return result

def build():
    latest=A.B.action('GetItems',{'table':A.LIST,'$orderby':'ID desc','$top':1})
    actions={'Read_lower_bound':latest,'Lower_bound':A.compose("@first(body('Read_lower_bound')?['value'])?['ID']",'Read_lower_bound')}
    # One request of 100 writes at a time; 50 requests total, no connection burst.
    raw_expr="replace(base64ToString('"+A.b64(raw(1))+"'),'"+PREFIX+"000000000001',concat('"+PREFIX+"',formatNumber(item(),'000000000000')))"
    items={'type':'Select','inputs':{'from':"@range(add(mul(items('Seed_5000'),100),1),100)",'select':{
      'Title':"@concat('"+TAG+"-',string(item()))",'SubmissionId':"@concat('"+PREFIX+"',formatNumber(item(),'000000000000'))",'ProjectId':'acorn','PayloadBase64A':'@base64('+raw_expr+')','PayloadBase64B':'','ExportStatus':'Pending'}},'runAfter':{}}
    header='--changeset_pa23'+CRLF+'Content-Type: application/http'+CRLF+'Content-Transfer-Encoding: binary'+CRLF*2+'POST '+A.B.SITE+"/_api/web/lists(guid'"+A.LIST+"')/items HTTP/1.1"+CRLF+'Content-Type: application/json;odata=nometadata'+CRLF+'Accept: application/json;odata=nometadata'+CRLF*2
    prefix='--batch_pa23'+CRLF+'Content-Type: multipart/mixed; boundary=changeset_pa23'+CRLF*2
    suffix='--changeset_pa23--'+CRLF+'--batch_pa23--'+CRLF
    parts={'type':'Select','inputs':{'from':"@body('Generate_items')",'select':"@concat(base64ToString('"+A.b64(header)+"'),string(item()),decodeUriComponent('%0D%0A'))"},'runAfter':{'Generate_items':['Succeeded']}}
    post=A.B.action('HttpRequest',{'parameters/method':'POST','parameters/uri':'_api/$batch','parameters/headers':{'Content-Type':'multipart/mixed; boundary=batch_pa23','Accept':'application/json'},'parameters/body':"@concat(base64ToString('"+A.b64(prefix)+"'),join(body('Generate_parts'),string(null)),base64ToString('"+A.b64(suffix)+"'))"},'Generate_parts')
    assertion=A.assertion({'equals':["@sub(length(split(base64ToString(body('Seed_batch')?['$content']),'HTTP/1.1 201 Created')),1)",100]},'Seed_batch','100 created rows')
    # Terminate cannot be nested in Foreach; fail with an invalid expression only
    # on a mismatched count, and never retry a partially completed changeset.
    assertion['else']['actions']={'Seed_count_mismatch':A.compose("@int('PA23 seed count mismatch')")}
    actions['Seed_5000']={'type':'Foreach','foreach':'@range(0,50)','actions':{'Generate_items':items,'Generate_parts':parts,'Seed_batch':post,'Verify_batch':assertion},'runAfter':{'Lower_bound':['Succeeded']},'runtimeConfiguration':{'concurrency':{'repetitions':1}}}
    actions['Page_5000']=case('page5000',5000,'Seed_5000')
    one=A.B.action('PostItem',{'table':A.LIST,'item/Title':TAG+'-5001','item/SubmissionId':PREFIX+f'{5001:012d}','item/ProjectId':'acorn','item/PayloadBase64A':A.b64(raw(5001)),'item/PayloadBase64B':'','item/ExportStatus':'Pending'},'Page_5000')
    actions['Seed_5001']=one
    actions['Page_5001']=case('page5001',5001,'Seed_5001',True)
    clip=A.B.build(A.LIST,A.LIBRARY);clip.update(operationName='PA23_real_pagination',operationDefinition=A.visible(A.scope(actions)))
    (ROOT/'pagination-snapshots.clipboard.json').write_text(json.dumps(clip,indent=2)+'\n')
    # Small preflight seeds only two distinct UUIDs; never contributes to the 5000 slice.
    sample=A.B.action('HttpRequest',{'parameters/method':'POST','parameters/uri':'_api/$batch','parameters/headers':{'Content-Type':'multipart/mixed; boundary=batch_pa23','Accept':'application/json'},'parameters/body':batch_body([9001,9002])})
    clip.update(operationName='PA23_batch_seed_preflight',operationDefinition=A.visible(A.scope({'Seed_batch_preflight':sample})))
    (ROOT/'pagination-preflight.clipboard.json').write_text(json.dumps(clip,indent=2)+'\n')

if __name__=='__main__':build()
