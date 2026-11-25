Set(varSummaryIndex,0); Set(varLedgerBusy,true); Set(varLedgerError,""); Set(varLedgerCursor,"!"); IfError(Set(varLedgerPageResponse,ReadApprovalPageTESTONLY.Run(varLedgerBatchId,varLedgerCursor,"false")); If(varLedgerPageResponse.batchid<>varLedgerBatchId,Error({Kind:ErrorKind.Validation,Message:"Batch identity changed."})); ClearCollect(colLedgerItems,ForAll(Table(ParseJSON(varLedgerPageResponse.itemsjson)),With({i:ThisRecord.Value},{sourceItemId:Value(i.sourceItemId),projectId:Text(i.projectId),sourceSummary:Text(i.sourceSummary),decision:Text(i.decision),validationState:Text(i.validationState)}))); Set(varLedgerNextCursor,varLedgerPageResponse.nextcursor); Set(varLedgerHasMore,Lower(varLedgerPageResponse.hasmore)="true"); If(IsEmpty(colLedgerItems),Set(varLedgerError,"This Ready batch has no review items."),ClearCollect(colReviewDrafts,
      ForAll(colLedgerItems As li,
        With({response: ReadApprovalItemTESTONLY.Run(varLedgerBatchId, Text(li.sourceItemId))},
          With({r: ParseJSON(response.itemjson)},
            With({src: ParseJSON(Text(r.sourcePayloadJson))},
              {id: Text(li.sourceItemId), sourceItemId: li.sourceItemId,
               itemETag: Text(r.itemETag), draftVersion: Value(r.draftVersion),
               projectId: Text(r.projectId), kind: If(IsBlank(Text(src.entry.health.status)), "activity", "status"),
               category: Text(src.entry.category), state: Text(src.entry.state),
               occurred: Left(Text(src.entry.timestamp), 10),
               summary: Coalesce(IfError(Text(ParseJSON(Text(r.draftProjectionJson)).entry.summary), Blank()), Text(src.entry.summary)),
               items: IfError(Concat(Table(src.entry.items), Text(Value), Char(10)), ""),
               eventType: Coalesce(Text(src.entry.eventType), ""), classification: "unclassified",
               decision: Lower(Text(r.decision)), savedDecision: Lower(Text(r.decision)), exclusionReason: Coalesce(Text(r.decisionReason), ""),
               health: Coalesce(IfError(Text(ParseJSON(Text(r.draftProjectionJson)).entry.health.status), Blank()), Text(src.entry.health.status), ""), healthReason: Coalesce(IfError(Text(ParseJSON(Text(r.draftProjectionJson)).entry.health.reason), Blank()), Text(src.entry.health.reason), ""), significance: Coalesce(IfError(Text(ParseJSON(Text(r.draftProjectionJson)).entry.significance), Blank()), Text(src.entry.significance), ""), evidence: Coalesce(IfError(Text(ParseJSON(Text(r.draftProjectionJson)).entry.evidence), Blank()), Text(src.entry.evidence), ""), riskKind: Coalesce(IfError(Text(ParseJSON(Text(r.draftProjectionJson)).entry.risk.kind), Blank()), Text(src.entry.risk.kind), ""), riskExplanation: Coalesce(IfError(Text(ParseJSON(Text(r.draftProjectionJson)).entry.risk.explanation), Blank()), Text(src.entry.risk.explanation), ""),
               corrects: Coalesce(Text(r.sourceSupersedes), ""), rangeMode: "batch",
               rangeStart: Left(Coalesce(Text(src.entry.range.left), Text(src.range.left), LookUp(colLedgerBatches, batchId = varLedgerBatchId).periodStartUtc), 10),
               rangeEnd: Left(Coalesce(Text(src.entry.range.right), Text(src.range.right), LookUp(colLedgerBatches, batchId = varLedgerBatchId).periodEndUtc), 10),
               sourceSummary: Text(src.entry.summary),
               sourceItems: IfError(Concat(Table(src.entry.items), Text(Value), Char(10)), ""),
               sourceDetails: If(IsBlank(Text(src.entry.health.status)), "Date " & Left(Text(src.entry.timestamp), 10) & " · Category " & Text(src.entry.category) & " · State " & Text(src.entry.state), "Health " & Text(src.entry.health.status) & " · Significance " & Text(src.entry.significance) & If(IsBlank(Text(src.entry.evidence)), "", " · Evidence " & Text(src.entry.evidence)) & If(IsBlank(Text(src.entry.risk.kind)), "", " · Risk " & Text(src.entry.risk.kind) & " — " & Text(src.entry.risk.explanation)))}
            )
          )
        )
      )
    );
    Set(varReviewId, First(colReviewDrafts).id);
    Set(varJumpProject, First(colReviewDrafts).projectId);
    Set(varPreviewOnly, false);
    Set(varLiveReviewMode, true);
    Set(varFixtureMode, false);
     Reset(ReviewGallery); Reset(ReviewState); Set(varSummaryGeneration,Coalesce(varSummaryGeneration,0)+1); ClearCollect(colReviewAiSummaries,ForAll(colReviewDrafts As item,{id:item.id,summary:If(!IsBlank(item.summary) && item.summary<>item.sourceSummary,item.summary,""),state:If(!IsBlank(item.summary) && item.summary<>item.sourceSummary,"existing","queued")})); Reset(ReviewAiTimer); Set(varSummaryIndex,1); Set(varAiSummaryItemId,Blank()); Set(varAiSummary,Blank()); Set(varAiSummaryState,"")),Set(varLedgerError,"The batch could not load. Check reviewer access and retry.")); Set(varLedgerBusy,false)
