If(!varSubmitting && varDisposition <> "exported" && varDisposition <> "conflict",
    Set(varSubmitting, true);
    Set(varSubmissionStatus, "Saving your highlight…");
    Set(varCanSend, false);
    If(IsBlank(varPendingJson),
        IfError(
            With({newId: Lower(Text(GUID())), newAt: Text(Now(), DateTimeFormat.UTC)},
                With({payload: JSON({entry: {archived: false, category: CategoryDropdown.Selected.Value, eventType: If(EventTypeDropdown.Selected.Value = "Not specified", Blank(), EventTypeDropdown.Selected.Value), global: PlacementDropdown.Selected.Value = "General update (report title page)", items: If(IsEmpty(Filter(Split(Substitute(ItemsInput.Text, Char(13), ""), Char(10)), !IsBlank(Trim(Value)))), Blank(), ForAll(Filter(Split(Substitute(ItemsInput.Text, Char(13), ""), Char(10)), !IsBlank(Trim(Value))), Trim(Value))), origin: "manual", state: StateDropdown.Selected.Value, summary: Trim(SummaryInput.Text), timestamp: Text(OccurredOnPicker.SelectedDate, DateTimeFormat.UTC)}, projectId: ProjectDropdown.Selected.ProjectId, submissionId: newId, submittedAt: newAt, submitter: {email: Lower(User().Email)}}, JSONFormat.Compact & JSONFormat.FlattenValueTables)},
                    Set(varPendingJson, payload); Set(varPendingId, newId); Set(varPendingProjectLabel, ProjectDropdown.Selected.Value); Set(varDisposition, "pending"); Set(varCanSend, true))),
            Set(varSubmissionStatus, "We couldn't prepare your highlight. Your entries are still here. Please try again.")),
        Set(varCanSend, true)
    );
    If(varCanSend,
        ClearCollect(colPendingHighlight, {payload: varPendingJson, owner: Lower(User().Email), projectLabel: varPendingProjectLabel, disposition: "pending", exportFileUrl: "", receiptUrl: ""});
        IfError(SaveData(colPendingHighlight, "SponsorHighlightSharePointPending"); true,
            Set(varCanSend, false); Set(varSubmissionStatus, "Browser recovery is unavailable. Nothing was sent. Open the published app in a regular browser window and try again."))
    );
    If(varCanSend,
        Set(varDisposition, "pending");
        IfError(
            With({receipt: ACORNSharePointPilotIntake.Run(varPendingJson)},
                If(receipt.submissionid = varPendingId && receipt.status = "received",
                    Set(varReceiptUrl, receipt.receipturl);
                    Set(varExportFileUrl, If(receipt.exportstatus = "exported" && receipt.exportfileurl = "https://ornl.sharepoint.com/sites/ResearchEnablement/Sponsor%20Highlights%20JSONL%20Pilot/" & varPendingId & ".jsonl", receipt.exportfileurl, ""));
                    Set(varDisposition, If(!IsBlank(varExportFileUrl), "exported", "received"));
                    Set(varSubmissionStatus, If(varDisposition = "exported", "Highlight received in SharePoint. Its JSONL export is verified and ready for ACORN review. This does not publish a report.", "Highlight received in SharePoint. The JSONL export is not yet confirmed. Select Retry JSONL export to finish safely with the same receipt ID."));
                    ClearCollect(colPendingHighlight, {payload: varPendingJson, owner: Lower(User().Email), projectLabel: varPendingProjectLabel, disposition: varDisposition, exportFileUrl: varExportFileUrl, receiptUrl: varReceiptUrl});
                    IfError(SaveData(colPendingHighlight, "SponsorHighlightSharePointPending"); true, Set(varSubmissionStatus, varSubmissionStatus & " Browser recovery could not store the updated receipt; a retry will safely confirm the same submission.")),
                    receipt.submissionid = varPendingId && receipt.status = "rejected",
                    Set(varDisposition, "rejected"); Set(varSubmissionStatus, "Not submitted. The highlight did not pass validation. Select Edit highlight to correct it, or contact the app owner."),
                    receipt.submissionid = varPendingId && receipt.status = "conflict",
                    Set(varDisposition, "conflict"); Set(varSubmissionStatus, "This receipt ID has different content in SharePoint. Keep this page and contact the app owner with the receipt ID below."),
                    Set(varDisposition, "pending"); Set(varSubmissionStatus, "We could not confirm the save. Your pending highlight is retained in this browser. Select Retry submission; the same ID prevents a duplicate."))),
            Set(varDisposition, "pending"); Set(varSubmissionStatus, "We could not confirm the save. Your pending highlight is retained in this browser. Select Retry submission; the same ID prevents a duplicate."))
    );
    Set(varSubmitting, false);
    Set(varShowSubmissionResult, true)
)

