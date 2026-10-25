If(!varSubmitting && varDisposition in ["queued", "rejected"],
    IfError(ClearData("SponsorHighlightPending"),
        Set(varSubmissionStatus, "The saved browser record could not be cleared. Please try again before starting another highlight."); Set(varShowSubmissionResult, true),
        If(varDisposition = "queued", Set(varRestoredDraft, Blank()), Set(varRestoredDraft, ParseJSON(varPendingJson))); Set(varPendingProjectLabel, Blank());
        Clear(colPendingHighlight); Set(varPendingJson, Blank()); Set(varPendingId, Blank()); Set(varCommitSha, Blank()); Set(varDisposition, Blank()); Set(varSubmissionStatus, Blank());
        Reset(OccurredOnPicker); Reset(CategoryDropdown); Reset(StateDropdown); Reset(EventTypeDropdown); Reset(SummaryInput); Reset(ItemsInput); Reset(PlacementDropdown);
        Select(RefreshProjectsButton); Set(varShowSubmissionResult, false)
    )
)
