"""Native aggregate guard/byte tests with injected receipt arrays; not pagination proof."""
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import build_acceptance as A

ROOT = Path(__file__).resolve().parent
FIXTURES = ROOT.parents[5] / 'output/pa23-20261027/boundary-fixtures'
CASES = [('rows5000',5000,None,False),('rows5001',5001,None,True),
         ('bytes_exact',162,10485760,False),('bytes_over',162,10485761,True)]


def raw_record(n, size=None):
    obj=json.loads(A.ROWS[0]);obj['submissionId']='23231027-0000-4000-8000-'+f'{n:012d}'
    obj['entry']['summary']='PA23 TEST ONLY aggregate boundary'
    raw=json.dumps(obj,ensure_ascii=False,separators=(',',':'))
    if size is not None: raw='{'+(' '*(size-len(raw.encode())))+raw[1:]
    return raw


def boundary(label,count,total,rejected,after):
    raw=raw_record(1)
    # Whitespace inside the JSON object pads records without exceeding field limits.
    # Each large record is <= 65,000 UTF-8 bytes, below the intake's 65,535 maximum.
    raw_expr="replace(base64ToString('"+A.b64(raw)+"'),'23231027-0000-4000-8000-000000000001',concat('23231027-0000-4000-8000-',formatNumber(item(),'000000000000')))"
    if total:
        last=total-161*65001-1
        padding=f"sub(if(equals(item(),162),{last},65000),{len(raw.encode())})"
        raw_expr="concat('{',take(outputs('Padding'),"+padding+"),skip("+raw_expr+",1))"
    select={'type':'Select','inputs':{'from':f'@range(1,{count})','select':{
      'ID':'@item()','SubmissionId':"@concat('23231027-0000-4000-8000-',formatNumber(item(),'000000000000'))",
      'ProjectId':'acorn','PayloadBase64A':'@take(base64('+raw_expr+'),45000)',
      'PayloadBase64B':'@skip(base64('+raw_expr+'),45000)'}},'runAfter':{}}
    original=A.export_case(label,'ID ge 1',A.PRIOR if rejected else '',rejected=rejected)['actions']
    exp=original['Exporter_'+label]['actions']
    exp['Read_latest_receipt_LB_'+label]=A.compose({'value':[{'ID':count}]})
    exp['Read_receipts_LB_'+label]=A.compose({'value':"@body('Generate_rows')"},'High_water_item_LB_'+label)
    original=A.rewrite(original,{"body('Read_latest_receipt_LB_"+label+"')":"outputs('Read_latest_receipt_LB_"+label+"')", "body('Read_receipts_LB_"+label+"')":"outputs('Read_receipts_LB_"+label+"')"})
    expected=b''.join((raw_record(n,65000 if n<162 else total-161*65001-1)+'\n').encode() for n in range(1,count+1)) if total else b''.join((raw_record(n)+'\n').encode() for n in range(1,count+1))
    sha=hashlib.sha256(expected).hexdigest()
    FIXTURES.mkdir(parents=True, exist_ok=True)
    (FIXTURES/('expected-'+label+'.jsonl')).write_bytes(expected)
    # Native readback equality checks the actual source concatenation; independent
    # downloaded bytes are compared against the separately generated fixture hash.
    condition=original['Assert_bytes_'+label]['expression']
    if not rejected: condition['and'][0]['equals'][1]="@base64(outputs('Combined_JSONL_LB_"+label+"'))"
    condition['and'].append({'equals':["@length(outputs('Read_receipts_LB_"+label+"')?['value'])",count]})
    if not rejected: condition['and'].append({'equals':["@sub(div(mul(length(body('Independent_read_"+label+"')?['$content']),3),4),if(endsWith(body('Independent_read_"+label+"')?['$content'],'=='),2,if(endsWith(body('Independent_read_"+label+"')?['$content'],'='),1,0)))",len(expected)]})
    setup={'Generate_rows':select,**original}
    setup['Seed_file_'+label]['runAfter']={'Generate_rows':['Succeeded']}
    setup=A.rewrite(setup,{'Generate_rows':'Generate_rows_'+label})
    return A.scope(setup,after),{'case':label,'rows':count,'source_bytes':len(expected),'source_sha256':sha,'rejected':rejected,'max_record_bytes':65000 if total else len(raw.encode())}


def build():
    actions={'Padding_1k':A.compose("@base64ToString('"+base64.b64encode(b' '*1024).decode()+"')"), 'Padding':A.compose("@concat("+','.join(["outputs('Padding_1k')"]*64)+")",'Padding_1k')}
    evidence=[];previous='Padding'
    for label,count,total,rejected in CASES:
        name='Case_'+label;actions[name],record=boundary(label,count,total,rejected,previous);evidence.append(record);previous=name
    clip=A.B.build(A.LIST,A.LIBRARY);clip.update(operationName='PA23_aggregate_boundaries',operationDefinition=A.visible(A.scope(actions)))
    (ROOT/'boundary-snapshots.clipboard.json').write_text(json.dumps(clip,indent=2)+'\n')
    (ROOT/'boundary-fixtures.json').write_text(json.dumps({'cases':evidence,'limits':'Native receipt arrays are injected; this does not prove connector pagination. File writes/readbacks are real test-library operations.'},indent=2)+'\n')


if __name__=='__main__': build()
