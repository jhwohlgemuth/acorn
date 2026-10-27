"""Build test-only native aggregate acceptance from the saved exporter graph."""
import base64
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
from uuid import NAMESPACE_URL, uuid5

ROOT = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('exporter', ROOT.parent / 'build-logbook-export.py')
B = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(B)
LIST = 'fcae092f-c899-4254-bf13-1108997a0a77'
LIBRARY = 'PA19 TEST ONLY JSONL'
TAG = 'pa23-20261027-a'
PRIOR = '{"test":"PA23 previous snapshot"}\n'
ALL = ['Succeeded', 'Failed', 'TimedOut', 'Skipped']


def b64(text):
    return base64.b64encode(text.encode()).decode()


def compose(value, after=None, statuses=None):
    return {'type':'Compose','inputs':value,'runAfter':{after:statuses or ['Succeeded']} if after else {}}


def names(actions):
    return list(actions) + [name for a in actions.values() for branch in (a.get('actions',{}),a.get('else',{}).get('actions',{})) for name in names(branch)]


def rewrite(value, replacements):
    if isinstance(value,dict): return {replacements.get(k,k):rewrite(v,replacements) for k,v in value.items()}
    if isinstance(value,list): return [rewrite(v,replacements) for v in value]
    if isinstance(value,str):
        for old,new in replacements.items(): value=value.replace(old,new)
    return value


def visible(value):
    if isinstance(value,dict): return {k:visible(v) for k,v in value.items() if k!='secureData'}
    if isinstance(value,list): return [visible(v) for v in value]
    return value


def scope(actions, after=None):
    return {'type':'Scope','actions':actions,'runAfter':{after:['Succeeded']} if after else {}}


def file_write(filename, text, after=None):
    return B.action('HttpRequest',{'parameters/method':'POST','parameters/uri':"_api/web/GetFolderByServerRelativeUrl('/sites/ResearchEnablement/"+LIBRARY+"')/Files/add(url='"+filename+"',overwrite=true)",'parameters/headers':{'Content-Type':'application/octet-stream'},'parameters/body':"@base64ToString('"+b64(text)+"')"},after)


def receipt(number):
    value={'entry':{'archived':False,'category':'software','eventType':None,'global':False,'items':['Synthetic acceptance only.','Unicode café Δ 🗺; escaped newline\ninside a JSON string.'],'origin':'manual','state':'in-progress','summary':f'PA23 TEST ONLY snapshot row {number}','timestamp':'2026-10-27T04:00:00.000Z'},'projectId':'acorn','submissionId':str(uuid5(NAMESPACE_URL,TAG+'/'+str(number))),'submittedAt':'2026-10-27T12:00:00.000Z','submitter':{'email':'alex.morgan@example.org'}}
    return json.dumps(value,ensure_ascii=False,separators=(',',':'))


ROWS = [receipt(i) for i in (1,2,3)]
IDS = [json.loads(row)['submissionId'] for row in ROWS]


def post_receipt(number, after=None, malformed=False):
    raw=ROWS[number-1] if not malformed else '{not valid JSON'
    identifier=IDS[number-1] if not malformed else str(uuid5(NAMESPACE_URL,TAG+'/malformed'))
    return B.action('PostItem',{'table':LIST,'item/Title':TAG+'-'+str(number) if not malformed else TAG+'-malformed','item/SubmissionId':identifier,'item/ProjectId':'acorn','item/PayloadBase64A':b64(raw),'item/PayloadBase64B':'','item/ExportStatus':'Pending'},after)


def assertion(expression, after, label):
    return B.guard(expression,{'Passed':compose(label)},'Assertion_failed',label+' did not meet its acceptance assertion.',after)


def export_case(label, query, expected, after=None, late=False, rejected=False):
    actions=deepcopy(B.build(LIST,LIBRARY)['operationDefinition']['actions'])
    filename=TAG+'-'+label+'.jsonl'
    actions['Read_latest_receipt_LB']['inputs']['parameters']['$filter']=query
    actions['Read_receipts_LB']['inputs']['parameters']['$filter']="@concat('("+query.replace("'","''")+") and ID le ',string(outputs('High_water_item_LB')))"
    if late:
        actions['Arrival_after_cutoff']=post_receipt(3,'High_water_item_LB')
        actions['Read_receipts_LB']['runAfter']={'Arrival_after_cutoff':['Succeeded']}
    pub=actions['Complete_valid_rows_LB']['actions']['Within_byte_limit_LB']['actions']
    pub=rewrite(pub,{'logbook.jsonl':filename})
    actions['Complete_valid_rows_LB']['actions']['Within_byte_limit_LB']['actions']=pub
    # Test markers preserve guard conditions while allowing final-file assertions.
    def markers(value):
        if isinstance(value,dict):
            if value.get('type')=='Terminate': return compose(value['inputs']['runError'])
            return {k:markers(v) for k,v in value.items()}
        if isinstance(value,list): return [markers(v) for v in value]
        return value
    actions=markers(actions)
    setup={'Seed_file':file_write(filename,PRIOR),'Exporter':scope(actions,'Seed_file'),
           'Independent_read':B.action('GetFileContentByPath',{'path':'/'+LIBRARY+'/'+filename,'inferContentType':False},'Exporter',ALL)}
    setup['Assert_bytes']=assertion({'and':[
        {'equals':[B.file_content('Independent_read'),b64(expected)]},
        {'equals':["@actions('Write_logbook_LB')?['status']",'Skipped' if rejected else 'Succeeded']},
        {'equals':["@actions('Export_receipt_LB')?['status']",'Skipped' if rejected else 'Succeeded']}]},'Independent_read',label)
    replacements={n:n+'_'+label for n in names(setup)}
    result=scope(rewrite(setup,dict(sorted(replacements.items(),key=lambda p:-len(p[0])))),after)
    return result


def build_recovery():
    actions={'Empty_snapshot':export_case('empty','ID eq 0','')}
    repeat=export_case('empty_repeat','ID eq 0','','Empty_snapshot')
    inner=repeat['actions'];inner.pop('Seed_file_empty_repeat')
    inner['Exporter_empty_repeat']['runAfter']={}
    repeat=rewrite(repeat,{TAG+'-empty_repeat.jsonl':TAG+'-empty.jsonl'})
    repeat['actions']['Assert_bytes_empty_repeat']['expression']['and'][1]['equals'][1]='Skipped'
    actions['Empty_repeat']=repeat
    actions['Seed_malformed']=post_receipt(1,'Empty_repeat',malformed=True)
    bad=str(uuid5(NAMESPACE_URL,TAG+'/malformed'))
    actions['Malformed_snapshot']=export_case('malformed',"SubmissionId eq '"+bad+"'",PRIOR,'Seed_malformed',rejected=True)
    result=B.build(LIST,LIBRARY)
    result.update(id=str(uuid5(NAMESPACE_URL,TAG+'/recovery')),operationName='PA23_empty_and_malformed_acceptance',operationDefinition=visible(scope(actions)))
    return result


def build_small():
    query=' or '.join("SubmissionId eq '"+i+"'" for i in IDS)
    actions={'Seed_row_1':post_receipt(1),'Seed_row_2':post_receipt(2,'Seed_row_1')}
    actions['Cutoff_snapshot']=export_case('cutoff',query,'\n'.join(ROWS[:2])+'\n','Seed_row_2',late=True)
    actions['Next_snapshot']=export_case('next',query,'\n'.join(ROWS)+'\n','Cutoff_snapshot')
    actions['Empty_snapshot']=export_case('empty','ID eq 0','','Next_snapshot')
    actions['Seed_malformed']=post_receipt(1,'Empty_snapshot',malformed=True)
    bad=str(uuid5(NAMESPACE_URL,TAG+'/malformed'))
    actions['Malformed_snapshot']=export_case('malformed',"SubmissionId eq '"+bad+"'",PRIOR,'Seed_malformed',rejected=True)
    result=B.build(LIST,LIBRARY)
    result.update(id=str(uuid5(NAMESPACE_URL,TAG)),operationName='PA23_small_snapshot_acceptance',operationDefinition=visible(scope(actions)))
    return result


if __name__=='__main__':
    (ROOT/'recovery-snapshots.clipboard.json').write_text(json.dumps(build_recovery(),indent=2)+'\n')
    (ROOT/'small-snapshots.clipboard.json').write_text(json.dumps(build_small(),indent=2)+'\n')
    (ROOT/'expected-cutoff.jsonl').write_text('\n'.join(ROWS[:2])+'\n')
    (ROOT/'expected-next.jsonl').write_text('\n'.join(ROWS)+'\n')
    (ROOT/'small-fixtures.json').write_text(json.dumps({'tag':TAG,'ids':IDS,'raw_rows':ROWS},indent=2,ensure_ascii=False)+'\n')
