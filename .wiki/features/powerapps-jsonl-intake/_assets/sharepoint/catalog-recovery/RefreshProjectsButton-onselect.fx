If(!varProjectsLoading && !varSubmitting && IsBlank(varPendingJson),
    Set(varProjectsLoading, true); Set(varProjectCatalogError, ""); Set(varCatalogLoaded, false);
    ClearCollect(colProjects, {Value: "Select a project", ProjectId: ""});
    ClearCollect(colCatalogCandidate, {Value: "", ProjectId: "", ValidTypes: true}); Clear(colCatalogCandidate);
    Set(varProjectCatalogResponse, IfError(
        ACORNSharePointPilotCatalog.Run(),
        Set(varProjectCatalogError, If(FirstError.Details.HttpStatusCode in [401, 403],
            "Project access was denied. Ask the app owner to check your form access and the catalog connection, then retry.",
            "The project service could not be reached. Check your connection and retry. If this continues, contact the app owner.")); {catalogjson: ""}
    ));
    If(IsBlank(varProjectCatalogError),
        IfError(
            With({catalog: ParseJSON(varProjectCatalogResponse.catalogjson)},
                If(!IsBlank(Text(catalog.error)),
                    Set(varProjectCatalogError, Switch(Text(catalog.error),
                        "access_denied", "Project access was denied. Ask the app owner to check your form access and the catalog connection, then retry.",
                        "catalog_limit", "The project list exceeds the supported limit. Contact the app owner; no projects were loaded.",
                        "unavailable", "The project service is temporarily unavailable. Retry shortly. If this continues, contact the app owner.",
                        "The project list could not be verified. Retry, or contact the app owner if this continues.")),
                    Text(catalog.source) <> "sharepoint" || !StartsWith(TrimEnds(JSON(catalog.projects)), "["),
                    Set(varProjectCatalogError, "The project list could not be verified. Retry, or contact the app owner if this continues."),
                    ClearCollect(colCatalogCandidate, ForAll(Table(catalog.projects), {Value: Text(ThisRecord.Value.label), ProjectId: Text(ThisRecord.Value.id), ValidTypes: StartsWith(JSON(ThisRecord.Value.id), Char(34)) && StartsWith(JSON(ThisRecord.Value.label), Char(34))}));
                    If(CountRows(colCatalogCandidate) > 1000 ||
                       CountRows(Distinct(colCatalogCandidate, ProjectId)) <> CountRows(colCatalogCandidate) ||
                       CountIf(colCatalogCandidate, !ValidTypes || IsBlank(TrimEnds(Value)) || IsBlank(TrimEnds(ProjectId)) || Len(Value) > 512 || Len(ProjectId) > 512) > 0,
                        Set(varProjectCatalogError, "The project list could not be verified. Retry, or contact the app owner if this continues."),
                        IsEmpty(colCatalogCandidate),
                        Set(varProjectCatalogError, "No projects are available yet. Ask the app owner to update the project list, then retry."),
                        Collect(colProjects, ShowColumns(colCatalogCandidate, Value, ProjectId)); Set(varCatalogLoaded, true); Set(varCatalogEverLoaded, true)
                    )
                )
            ); true,
            Set(varProjectCatalogError, "The project list could not be verified. Retry, or contact the app owner if this continues."); false
        )
    );
    If(!varCatalogLoaded, ClearCollect(colProjects, {Value: "Select a project", ProjectId: ""}));
    Clear(colCatalogCandidate); Set(varProjectsLoading, false); Reset(ProjectDropdown)
)
