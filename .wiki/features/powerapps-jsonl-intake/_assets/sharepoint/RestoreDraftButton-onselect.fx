If(!Coalesce(varRecoveryLoaded, false),
    Set(varRecoveryBlocked, false);
    ClearCollect(colPendingHighlight, {payload: "", owner: "", projectLabel: "", disposition: "", exportFileUrl: "", receiptUrl: ""}); Clear(colPendingHighlight);
    IfError(
        LoadData(colPendingHighlight, "SponsorHighlightSharePointPending", true);
        If(!IsEmpty(colPendingHighlight),
            If(CountRows(colPendingHighlight) <> 1 || First(colPendingHighlight).owner <> Lower(User().Email),
                Set(varRecoveryBlocked, true); Set(varSubmissionStatus, "This browser has a pending highlight for another account. Sign in with that account to resolve it before submitting here."),
                With({saved: First(colPendingHighlight)},
                    With({draft: ParseJSON(saved.payload)},
                        If(Text(draft.submitter.email) <> Lower(User().Email) || !IsMatch(Text(draft.submissionId), "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"),
                            Set(varRecoveryBlocked, true); Set(varSubmissionStatus, "The saved highlight could not be verified. Contact the app owner before sending another highlight."),
                            Set(varPendingJson, saved.payload); Set(varPendingId, Text(draft.submissionId)); Set(varPendingProjectLabel, saved.projectLabel); Set(varRestoredDraft, draft);
                            Set(varExportFileUrl, saved.exportFileUrl); Set(varReceiptUrl, saved.receiptUrl); Set(varDisposition, If(saved.disposition = "exported" && saved.exportFileUrl = "https://ornl.sharepoint.com/sites/ResearchEnablement/Sponsor%20Highlights%20JSONL%20Pilot/" & varPendingId & ".jsonl", "exported", saved.disposition = "received", "received", "pending"));
                            ClearCollect(colProjects, {Value: saved.projectLabel, ProjectId: Text(draft.projectId)});
                            Set(varSubmissionStatus, If(varDisposition = "exported", "Highlight received in SharePoint. Its JSONL export is verified and ready for ACORN review.", varDisposition = "received", "Highlight received in SharePoint. Select Retry JSONL export to confirm its export.", "Pending highlight recovered. Select Retry submission to confirm its save without creating a duplicate."));
                            Reset(ProjectDropdown); Reset(OccurredOnPicker); Reset(CategoryDropdown); Reset(StateDropdown); Reset(EventTypeDropdown); Reset(SummaryInput); Reset(ItemsInput); Reset(PlacementDropdown)
                        )
                    )
                )
            )
        ); Set(varRecoveryLoaded, true),
        Set(varRecoveryBlocked, true); Set(varSubmissionStatus, "Browser recovery is unavailable. Open the published app in a regular browser window. Your pending data has not been cleared.")
    );
    Set(varShowSubmissionResult, varRecoveryBlocked || !IsBlank(varPendingJson));
    If(IsBlank(varPendingJson) && !varRecoveryBlocked && !Coalesce(varCatalogLoaded, false), Select(RefreshProjectsButton))
)

