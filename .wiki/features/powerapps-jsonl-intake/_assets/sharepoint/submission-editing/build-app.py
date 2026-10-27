import copy,json
from pathlib import Path
b=Path('.wiki/features/powerapps-jsonl-intake/_assets/sharepoint')
o=b/'submission-editing'
p=json.loads((b/'report-placement/current-properties.json').read_text())
for c,props in json.loads((b/'report-placement/control-properties.json').read_text()).items(): p.setdefault(c,{}).update(props)
flow='PA31TESTONLYSharePointSubmissionEditing'
reset='Reset(ProjectDropdown); Reset(OccurredOnPicker); Reset(CategoryDropdown); Reset(StateDropdown); Reset(EventTypeDropdown); Reset(SummaryInput); Reset(ItemsInput); Reset(PlacementDropdown)'
clear='Set(varCorrectionParent, Blank()); Set(varCorrectionRoot, Blank()); Set(varEditingProjectId, Blank()); Set(varRestoredDraft, Blank()); Set(varPendingProjectLabel, Blank())'
submit=p['SubmitButton']['OnSelect']
start=submit.index('With({payload: JSON(')
end=submit.index(', JSONFormat.Compact & JSONFormat.FlattenValueTables)},',start)
record=submit[start+len('With({payload: JSON('):end]
record=record.replace('projectId: If(PlacementDropdown.Selected.Value', 'projectId: If(!IsBlank(varCorrectionParent), varEditingProjectId, PlacementDropdown.Selected.Value')
submit=submit[:start]+'With({base: '+record+'}, With({payload: If(IsBlank(varCorrectionParent), JSON(base, JSONFormat.Compact & JSONFormat.FlattenValueTables), JSON(Patch(base, {revision: {rootSubmissionId: varCorrectionRoot, supersedesSubmissionId: varCorrectionParent}}), JSONFormat.Compact & JSONFormat.FlattenValueTables))},'+submit[end+len(', JSONFormat.Compact & JSONFormat.FlattenValueTables)},'):]
submit=submit.replace('Set(varCanSend, true))),','Set(varCanSend, true)))),')
# Test copy routes both originals and corrections only to the isolated flow.
submit=submit.replace('ACORNSharePointPilotIntake.Run(varPendingJson)',flow+'.Run(varPendingJson)')
submit=submit.replace('Set(varCanSend, false);','Set(varCanSend, false); Set(varCorrectionConflict, false);',1)
submit=submit.replace('Set(varDisposition, "conflict"); Set(varSubmissionStatus, "This receipt ID has different content in SharePoint. Keep this page and contact the app owner with the receipt ID below.")','Set(varDisposition, "conflict"); Set(varCorrectionConflict, receipt.reason = "stale_revision"); Set(varSubmissionStatus, If(varCorrectionConflict, "A newer correction already exists. Nothing from this edit was saved. Return to the form and choose Load latest submission to review the newer version.", "This receipt ID has different content in SharePoint. Keep this page and contact the app owner with the receipt ID below."))')
p['SubmitButton']['OnSelect']=submit
p['SubmitButton']['Text']='If(varSubmitting, "Saving…", varDisposition = "received", "Retry JSONL export", !IsBlank(varPendingJson) && varDisposition = "pending", "Retry submission", !IsBlank(varCorrectionParent), "Save correction", "Submit highlight")'
p['RestoreDraftButton']['OnSelect']=p['RestoreDraftButton']['OnSelect'].replace('Set(varRestoredDraft, draft);','Set(varRestoredDraft, draft); Set(varCorrectionParent, Text(draft.revision.supersedesSubmissionId)); Set(varCorrectionRoot, Text(draft.revision.rootSubmissionId)); Set(varEditingProjectId, Text(draft.projectId));')
p['EditDraftButton']['OnSelect']=p['EditDraftButton']['OnSelect'].replace('If(varDisposition = "exported", Set(varRestoredDraft, Blank()),','If(varDisposition = "exported", '+clear+',').replace('Set(varPendingProjectLabel, Blank());','Set(varPendingProjectLabel, If(IsBlank(varCorrectionParent), Blank(), varPendingProjectLabel));')
for c in ['PlacementDropdown','ProjectDropdown','RefreshProjectsButton']:
 old=p.get(c,{}).get('DisplayMode','If(varSubmitting || !IsBlank(varPendingJson), DisplayMode.Disabled, DisplayMode.Edit)')
 p.setdefault(c,{})['DisplayMode']='If(!IsBlank(varCorrectionParent), DisplayMode.Disabled, '+old+')'
p['ProjectHelp']['Text']='If(!IsBlank(varCorrectionParent), "This correction stays with " & varEditingProjectId & ". Project and report placement cannot be changed here.", '+p['ProjectHelp']['Text']+')'
p['SubmitLayout']={'Visible':'!Coalesce(varShowSubmissionResult, false) && !Coalesce(varHistoryOpen, false)'}
p['SubmissionResultPanel']={'Visible':'Coalesce(varShowSubmissionResult, false) && !Coalesce(varHistoryOpen, false)'}
p['SubmissionContext']={'Text':'If(!IsBlank(varCorrectionParent), "CORRECTION // Original receipt retained. Project and report placement stay fixed.", "Submitted by " & User().FullName)'}
p['SubmitEntryScreen']={'OnVisible':'Set(varHistoryOpen, false); Set(varHistoryLoading, false); Set(varCorrectionConflict, false); Select(RestoreDraftButton)'}
# Preserve original UI properties; emit only the properties involved in editing.
keep={'SubmitButton':['OnSelect','Text'], 'RestoreDraftButton':['OnSelect'],'EditDraftButton':['OnSelect'],'PlacementDropdown':['DisplayMode'],'ProjectDropdown':['DisplayMode'],'RefreshProjectsButton':['DisplayMode'],'ProjectHelp':['Text'],'SubmitLayout':['Visible'],'SubmissionResultPanel':['Visible'],'SubmissionContext':['Text'],'SubmitEntryScreen':['OnVisible']}
p={c:{k:p[c][k] for k in keys} for c,keys in keep.items()}
for props in p.values():
 for k,v in props.items(): props[k]=v.replace('SponsorHighlightSharePointPending','SponsorHighlightPA31Pending').replace('Sponsor%20Highlights%20JSONL%20Pilot','PA19%20TEST%20ONLY%20JSONL')
(o/'test-control-properties.json').write_text(json.dumps(p,indent=2)+'\n')

load=f'''If(!varSubmitting && !varHistoryLoading && (IsBlank(varPendingJson) || varDisposition = "exported"),
 Set(varHistoryOpen, true); Set(varHistoryLoading, true); Set(varHistoryError, ""); Clear(colSubmissionHistory);
 IfError(With({{history: {flow}.Run("list")}},
  If(history.status = "listed",
   ClearCollect(colSubmissionHistory, ForAll(Table(ParseJSON(history.submissions)), With({{row: ThisRecord.Value}}, {{receiptId: Text(row.receipt.submissionId), rootId: Coalesce(Text(row.receipt.revision.rootSubmissionId), Text(row.receipt.submissionId)), supersedesId: Text(row.receipt.revision.supersedesSubmissionId), raw: Text(row.raw), summary: Text(row.receipt.entry.summary), projectId: Text(row.receipt.projectId), submittedAt: Text(row.receipt.submittedAt), exportStatus: Text(row.exportStatus), label: Text(DateTimeValue(Text(row.receipt.submittedAt)), "mmm d, yyyy h:mm AM/PM") & " · " & Left(Text(row.receipt.entry.summary), 100)}}))),
   Set(varHistoryError, "We couldn't verify your complete submission history. Try again or contact the app owner."))),
  Set(varHistoryError, "Your submissions could not be loaded. Your current form is unchanged. Try again."));
 Set(varHistoryLoading, false); Reset(HistoryDropdown)
)'''
edit=f'''If(!IsBlank(HistoryDropdown.Selected.receiptId) && HistoryDropdown.Selected.exportStatus = "Exported" && !varRecoveryBlocked && (IsBlank(varPendingJson) || varDisposition = "exported"),
 IfError(ClearData("SponsorHighlightPA31Pending"), Set(varHistoryError, "Browser recovery could not clear the previous confirmed receipt. Try again."),
 With({{chosen: HistoryDropdown.Selected}},
  Set(varRestoredDraft, ParseJSON(chosen.raw)); Set(varCorrectionParent, chosen.receiptId); Set(varCorrectionRoot, chosen.rootId); Set(varEditingProjectId, chosen.projectId);
  Set(varPendingProjectLabel, Coalesce(LookUp(colProjects, ProjectId = chosen.projectId, Value), chosen.projectId));
  ClearCollect(colProjects, {{Value: varPendingProjectLabel, ProjectId: chosen.projectId}}); Set(varCatalogLoaded, true);
  Clear(colPendingHighlight); Set(varPendingJson, Blank()); Set(varPendingId, Blank()); Set(varDisposition, Blank()); Set(varExportFileUrl, Blank()); Set(varReceiptUrl, Blank());
  Set(varSubmissionStatus, "Editing a saved submission. Saving creates a new correction receipt; the original remains in the logbook."); Set(varShowSubmissionResult, false); Set(varHistoryOpen, false); {reset}
 )))'''
cancel=f'If(!varSubmitting && IsBlank(varPendingJson), {clear}; Set(varSubmissionStatus, Blank()); {reset}; Select(RefreshProjectsButton))'
stale=f'''If(varCorrectionConflict && varDisposition = "conflict" && !varSubmitting,
 IfError(ClearData("SponsorHighlightPA31Pending"), Set(varSubmissionStatus, "The stale edit could not be cleared from this browser. Try again."),
  Clear(colPendingHighlight); Set(varPendingJson, Blank()); Set(varPendingId, Blank()); Set(varDisposition, Blank()); Set(varExportFileUrl, Blank()); Set(varReceiptUrl, Blank()); Set(varCorrectionConflict, false); Set(varShowSubmissionResult, false); {clear}; {reset}; Select(MySubmissionsButton)))'''

def node(name, control, props, children=None,variant=None):
 d={'Control':control}
 if variant:d['Variant']=variant
 d['Properties']={k:'='+v for k,v in props.items()}
 if children:d['Children']=children
 return {name:d}

def yaml_node(n, indent=0):
 lines=[];pad=' '*indent
 for name,v in n.items():
  lines.append(pad+'- '+name+':')
  for k,x in v.items():
   if k=='Properties':
    lines.append(pad+'    Properties:')
    for prop,expr in x.items(): lines.append(pad+'      '+prop+': '+json.dumps(expr,ensure_ascii=False))
   elif k=='Children':
    lines.append(pad+'    Children:')
    for child in x:lines+=yaml_node(child,indent+6)
   else:lines.append(pad+'    '+k+': '+x)
 return lines
button={'BorderThickness':'0','Color':'Color.White','Fill':'ColorValue("#007833")','Font':'Font.\'Segoe UI\'','Height':'44','Size':'14','RadiusTopLeft':'6','RadiusTopRight':'6','RadiusBottomLeft':'6','RadiusBottomRight':'6','Width':'180'}
label={'TabIndex':'0','FocusedBorderThickness':'2','Color':'ColorValue("#123B31")','Font':'Font.\'Segoe UI\'','Size':'14','Width':'Parent.Width - 48','X':'24','Height':'64'}
# Insert as a direct SubmitLayout child after SubmitHeader and before EntryForm.
nav=node('MySubmissionsButton','Classic/Button@2.2.0',{**button,'AlignInContainer':'AlignInContainer.Center','Text':'"My Submissions"','OnSelect':load,'DisplayMode':'If(varSubmitting || varHistoryLoading || (!IsBlank(varPendingJson) && varDisposition <> "exported"), DisplayMode.Disabled, DisplayMode.Edit)'})
controls=[
 node('HistoryTitle','Label@2.5.1',{**label,'Text':'"My Submissions"','FontWeight':'FontWeight.Semibold','Size':'26','Y':'18','Height':'50'}),
 node('HistoryHelp','Label@2.5.1',{**label,'Text':'"Choose a saved highlight to correct. Earlier receipts stay in the logbook. Project and report placement remain fixed."','Y':'76','Height':'70'}),
 node('HistoryDropdown','Classic/DropDown@2.3.1',{'AccessibleLabel':'"Select a submission to edit"','Items':'SortByColumns(Filter(colSubmissionHistory As candidate, !(candidate.receiptId in colSubmissionHistory.supersedesId)), "submittedAt", SortOrder.Descending)','Items.Value':'label','Default':'Blank()','AllowEmptySelection':'true','DisplayMode':'If(varHistoryLoading || !IsBlank(varHistoryError) || IsEmpty(colSubmissionHistory), DisplayMode.Disabled, DisplayMode.Edit)','X':'24','Y':'154','Width':'Parent.Width - 48','Height':'48','Color':'ColorValue("#123B31")','Fill':'ColorValue("#F0F5F2")','Size':'14'}),
 node('HistorySummary','Label@2.5.1',{**label,'Text':'Coalesce(HistoryDropdown.Selected.summary, "Select a submission above")','FontWeight':'FontWeight.Semibold','Y':'218','Height':'90'}),
 node('HistoryReceipt','Label@2.5.1',{**label,'Text':'If(!IsBlank(HistoryDropdown.Selected.receiptId), "Receipt: " & HistoryDropdown.Selected.receiptId & Char(10) & "Destination: " & HistoryDropdown.Selected.projectId & " · " & HistoryDropdown.Selected.exportStatus)','Y':'312','Height':'60','Size':'11'}),
 node('HistoryStatus','Label@2.5.1',{**label,'Text':'If(varHistoryLoading, "Loading your submissions…", !IsBlank(varHistoryError), varHistoryError, IsEmpty(colSubmissionHistory), "You have no saved submissions yet.", !IsBlank(HistoryDropdown.Selected.receiptId) && HistoryDropdown.Selected.exportStatus <> "Exported", "This receipt has an unfinished export. Reconcile it before editing.", "Saving a correction does not replace a published report. Operators review corrections before applying them.")','Y':'384','Height':'100'}),
 node('HistoryEditButton','Classic/Button@2.2.0',{**button,'Text':'"Edit selected"','OnSelect':edit,'DisplayMode':'If(varHistoryLoading || varRecoveryBlocked || IsBlank(HistoryDropdown.Selected.receiptId) || HistoryDropdown.Selected.exportStatus <> "Exported", DisplayMode.Disabled, DisplayMode.Edit)','X':'24','Y':'Parent.Height - 72','Width':'Min(180, (Parent.Width - 60) / 2)'}),
 node('HistoryBackButton','Classic/Button@2.2.0',{**button,'Text':'"Back to form"','OnSelect':'Set(varHistoryOpen, false)','X':'Parent.Width - Self.Width - 24','Y':'Parent.Height - 72','Width':'Min(180, (Parent.Width - 60) / 2)','Fill':'ColorValue("#466659")'}),
]
card=node('HistoryCard','GroupContainer@1.5.0',{'Fill':'Color.White','X':'Max(16, (Parent.Width - Self.Width) / 2)','Y':'20','Width':'Min(800, Parent.Width - 32)','Height':'Max(600, Parent.Height - 40)','RadiusTopLeft':'12','RadiusTopRight':'12','RadiusBottomLeft':'12','RadiusBottomRight':'12'},controls,'ManualLayout')
panel=node('SubmissionHistoryPanel','GroupContainer@1.5.0',{'Fill':'ColorValue("#EDF3EF")','Width':'Parent.Width','Height':'Parent.Height','Visible':'Coalesce(varHistoryOpen, false)'},[card],'ManualLayout')
cancel_node=node('CancelCorrectionButton','Classic/Button@2.2.0',{**button,'Text':'If(varCorrectionConflict, "Load latest submission", "Cancel editing")','OnSelect':'If(varCorrectionConflict, '+stale+', '+cancel+')','Visible':'!IsBlank(varCorrectionParent) && (IsBlank(varPendingJson) || varCorrectionConflict)','DisplayMode':'If(varSubmitting, DisplayMode.Disabled, DisplayMode.Edit)','AlignInContainer':'AlignInContainer.Stretch','Fill':'ColorValue("#466659")'})
for name,n in [('history-button',nav),('history-panel',panel),('cancel-correction',cancel_node)]: (o/(name+'.pa.yaml')).write_text('\n'.join(yaml_node(n))+'\n')
print('Built test-only editing properties and UI controls')

# Isolated acceptance harness. Never install these controls in the operational app.
harness_children=[
 node('TestHarnessTitle','Label@2.5.1',{**label,'Text':'"PA31 TEST ONLY // Correction acceptance"','Size':'22','Y':'20','Height':'48'}),
 node('TestRequestA','Classic/TextInput@2.3.2',{'Default':'"list"','Mode':'TextMode.MultiLine','X':'24','Y':'88','Width':'Parent.Width - 48','Height':'150','Size':'12','AccessibleLabel':'"Test request A"'}),
 node('TestRequestB','Classic/TextInput@2.3.2',{'Default':'"list"','Mode':'TextMode.MultiLine','X':'24','Y':'250','Width':'Parent.Width - 48','Height':'150','Size':'12','AccessibleLabel':'"Test request B"'}),
 node('TestRunOne','Classic/Button@2.2.0',{**button,'Text':'"Run request A"','X':'24','Y':'416','OnSelect':f'Set(varTestStatusA, "Running…"); Set(varTestStatusB, ""); IfError(With({{r: {flow}.Run(TestRequestA.Text)}}, Set(varTestStatusA, JSON({{status: r.status, reason: r.reason, submissionId: r.submissionid, exportStatus: r.exportstatus}}, JSONFormat.Compact))), Set(varTestStatusA, FirstError.Message))'}),
 node('TestRunBoth','Classic/Button@2.2.0',{**button,'Text':'"Run concurrently"','X':'220','Y':'416','OnSelect':f'Set(varTestStatusA, "Running…"); Set(varTestStatusB, "Running…"); Concurrent(IfError(With({{r: {flow}.Run(TestRequestA.Text)}}, Set(varTestStatusA, JSON({{status: r.status, reason: r.reason, submissionId: r.submissionid, exportStatus: r.exportstatus}}, JSONFormat.Compact))), Set(varTestStatusA, FirstError.Message)), IfError(With({{r: {flow}.Run(TestRequestB.Text)}}, Set(varTestStatusB, JSON({{status: r.status, reason: r.reason, submissionId: r.submissionid, exportStatus: r.exportstatus}}, JSONFormat.Compact))), Set(varTestStatusB, FirstError.Message)))'}),
 node('TestStatusA','Label@2.5.1',{**label,'Text':'varTestStatusA','Y':'480','Height':'76','Size':'13'}),
 node('TestStatusB','Label@2.5.1',{**label,'Text':'varTestStatusB','Y':'560','Height':'76','Size':'13'}),
 node('TestHarnessBack','Classic/Button@2.2.0',{**button,'Text':'"Back to form"','X':'24','Y':'650','OnSelect':'Set(varTestHarness, false)'})]
harness=node('TestHarnessPanel','GroupContainer@1.5.0',{'Fill':'Color.White','Width':'Parent.Width','Height':'Parent.Height','Visible':'Coalesce(varTestHarness, false)'},harness_children,'ManualLayout')
harness_button=node('TestHarnessButton','Classic/Button@2.2.0',{**button,'Text':'"Acceptance tests"','X':'224','Y':'152','Height':'40','OnSelect':'Set(varTestHarness, true)'})
for name,n in [('test-harness',harness),('test-harness-button',harness_button)]: (o/(name+'.pa.yaml')).write_text('\n'.join(yaml_node(n))+'\n')

# Operational form: preserve its original intake binding and browser recovery key.
production_flow = 'ACORNSharePointSubmissionEditing'
production_properties = copy.deepcopy(p)
for properties in production_properties.values():
 for key, value in properties.items():
  properties[key] = value.replace(flow + '.Run(varPendingJson)', 'If(IsBlank(Text(ParseJSON(varPendingJson).revision.supersedesSubmissionId)), Patch(ACORNSharePointPilotIntake.Run(varPendingJson), {reason: ""}), ' + production_flow + '.Run(varPendingJson))').replace(flow, production_flow).replace('SponsorHighlightPA31Pending', 'SponsorHighlightSharePointPending').replace('PA19%20TEST%20ONLY%20JSONL', 'Sponsor%20Highlights%20JSONL%20Pilot')
(o/'production-control-properties.json').write_text(json.dumps(production_properties, indent=2)+'\n')
for name in ['history-button', 'history-panel', 'cancel-correction']:
 text=(o/(name+'.pa.yaml')).read_text().replace(flow, production_flow).replace('SponsorHighlightPA31Pending', 'SponsorHighlightSharePointPending')
 (o/('production-'+name+'.pa.yaml')).write_text(text)
