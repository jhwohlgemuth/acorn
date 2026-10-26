Set(varProjectsLoading, true); Set(varProjectCatalogError, ""); Set(varCatalogLoaded, false);
ClearCollect(colProjects, {Value: "Select a project", ProjectId: ""});
IfError(
    Set(varProjectCatalogResponse, ACORNSharePointPilotCatalog.Run());
    With({catalog: ParseJSON(varProjectCatalogResponse.catalogjson)},
        If(!IsBlank(Text(catalog.error)) || Text(catalog.source) <> "sharepoint",
            Set(varProjectCatalogError, "The project list is unavailable. Contact the app owner."),
            Collect(colProjects, ForAll(Table(catalog.projects), {Value: Text(ThisRecord.Value.label), ProjectId: Text(ThisRecord.Value.id)}));
            Set(varCatalogLoaded, true))),
    ClearCollect(colProjects, {Value: "Select a project", ProjectId: ""});
    Set(varProjectCatalogError, "Projects could not be loaded. Please try Refresh Project List."));
Set(varProjectsLoading, false); Reset(ProjectDropdown)
