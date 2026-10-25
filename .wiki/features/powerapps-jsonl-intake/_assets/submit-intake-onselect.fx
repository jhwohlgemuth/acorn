If(!varSubmitting && varDisposition <> "queued" && varDisposition <> "conflict",
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
        ClearCollect(colPendingHighlight, {payload: varPendingJson, owner: Lower(User().Email), projectLabel: varPendingProjectLabel, disposition: "pending", commitSha: ""});
        IfError(SaveData(colPendingHighlight, "SponsorHighlightPending"); true,
            Set(varCanSend, false); Set(varSubmissionStatus, "Browser recovery is unavailable. Nothing was sent. Open the published app in a regular browser window and try again."))
    );
    If(varCanSend,
        Set(varDisposition, "pending");
        IfError(
            With({receipt: 'ACORNLogbookGitLabJSONLIntake(Draft)'.Run(varPendingJson)},
                If(receipt.submissionid = varPendingId && receipt.disposition = "queued" && IsMatch(receipt.commitsha, "[a-fA-F0-9]{40}"),
                    Set(varDisposition, "queued"); Set(varCommitSha, receipt.commitsha); Set(varSubmissionStatus, "Saved to the logbook. Your highlight is awaiting ACORN review.");
                    ClearCollect(colPendingHighlight, {payload: varPendingJson, owner: Lower(User().Email), projectLabel: varPendingProjectLabel, disposition: "queued", commitSha: varCommitSha});
                    IfError(SaveData(colPendingHighlight, "SponsorHighlightPending"); true, Set(varSubmissionStatus, "Saved to the logbook. Browser recovery could not store the receipt; a retry will safely confirm this same submission.")),
                    receipt.submissionid = varPendingId && receipt.disposition = "rejected",
                    Set(varDisposition, "rejected"); Set(varSubmissionStatus, "Not submitted. " & Left(receipt.message, 300) & " Select Edit highlight to correct it."),
                    receipt.submissionid = varPendingId && receipt.disposition = "conflict",
                    Set(varDisposition, "conflict"); Set(varSubmissionStatus, "This submission ID has different content in the logbook. Keep this page and contact the app owner with the receipt ID below."),
                    Set(varDisposition, "pending"); Set(varSubmissionStatus, "We could not confirm the save. Your pending highlight is retained in this browser. Select Retry submission; the same ID prevents a duplicate.")
                )
            ),
            Set(varDisposition, "pending"); Set(varSubmissionStatus, "We could not confirm the save. Your pending highlight is retained in this browser. Select Retry submission; the same ID prevents a duplicate.")
        )
    );
    Set(varSubmitting, false);
    Set(varShowSubmissionResult, true)
)
