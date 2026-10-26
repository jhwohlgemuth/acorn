If(!varSubmitting && varDisposition in ["exported", "rejected"],
    IfError(ClearData("SponsorHighlightSharePointPending"),
        Set(varSubmissionStatus, "The saved browser record could not be cleared. Please try again before starting another highlight."); Set(varShowSubmissionResult, true),
        If(varDisposition = "exported", Set(varRestoredDraft, Blank()), Set(varRestoredDraft, ParseJSON(varPendingJson))); Set(varPendingProjectLabel, Blank());
        Clear(colPendingHighlight); Set(varPendingJson, Blank()); Set(varPendingId, Blank()); Set(varExportFileUrl, Blank()); Set(varReceiptUrl, Blank()); Set(varDisposition, Blank()); Set(varSubmissionStatus, Blank());
        Reset(OccurredOnPicker); Reset(CategoryDropdown); Reset(StateDropdown); Reset(EventTypeDropdown); Reset(SummaryInput); Reset(ItemsInput); Reset(PlacementDropdown);
        Select(RefreshProjectsButton); Set(varShowSubmissionResult, false)
    )
)

