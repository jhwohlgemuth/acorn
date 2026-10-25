If(!Coalesce(varRecoveryLoaded, false),
    Set(varRecoveryBlocked, false);
    ClearCollect(colPendingHighlight, {payload: "", owner: "", projectLabel: "", disposition: "", commitSha: ""}); Clear(colPendingHighlight);
    IfError(
        LoadData(colPendingHighlight, "SponsorHighlightPending", true);
        If(!IsEmpty(colPendingHighlight),
            If(CountRows(colPendingHighlight) <> 1 || First(colPendingHighlight).owner <> Lower(User().Email),
                Set(varRecoveryBlocked, true); Set(varSubmissionStatus, "This browser has a pending highlight for another account. Sign in with that account to resolve it before submitting here."),
                With({saved: First(colPendingHighlight)},
                    With({draft: ParseJSON(saved.payload)},
                        If(Text(draft.submitter.email) <> Lower(User().Email) || !IsMatch(Text(draft.submissionId), "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"),
                            Set(varRecoveryBlocked, true); Set(varSubmissionStatus, "The saved highlight could not be verified. Contact the app owner before sending another highlight."),
                            Set(varPendingJson, saved.payload); Set(varPendingId, Text(draft.submissionId)); Set(varPendingProjectLabel, saved.projectLabel); Set(varRestoredDraft, draft);
                            Set(varCommitSha, saved.commitSha); Set(varDisposition, If(saved.disposition = "queued" && IsMatch(saved.commitSha, "[a-fA-F0-9]{40}"), "queued", "pending"));
                            ClearCollect(colProjects, {Value: saved.projectLabel, ProjectId: Text(draft.projectId)});
                            Set(varSubmissionStatus, If(varDisposition = "queued", "Saved to the logbook. Your highlight is awaiting ACORN review.", "Pending highlight recovered. Select Retry submission to confirm its save without creating a duplicate."));
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
