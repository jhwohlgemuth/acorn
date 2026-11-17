//! Cross-module browser workspace and research index tests.
use crate::research_index::{CatalogSort, ResearchIndex};
use crate::session::{Selection, SuppliedFile, Workspace, WorkspaceSource};
use serde_json::json;

#[test]
fn test_index_ignores_invalid_and_replaced_records() {
    let files = ["a.json", "b.json", "invalid.json"]
        .into_iter()
        .map(|path| SuppliedFile {
            path: path.into(),
            content: "{}".into(),
        })
        .collect();
    let mut workspace = Workspace::default().replace(Selection { files, skipped: 0 });
    let value = json!({"title":"A project","meta":{"identifier":"project-one","organizations":[{"name":"Example"}]}});
    for path in ["a.json", "b.json"] {
        workspace.validations.insert(path.into(), json!({"valid":true,"value":value}));
    }
    workspace.validations.insert("invalid.json".into(), json!({"valid":false,"value":value}));
    let index = ResearchIndex::from(&workspace);
    assert_eq!(index.activities.len(), 2);
    assert_eq!(index.projects.len(), 1);
    assert_eq!(index.projects.first().expect("one project").paths, ["a.json", "b.json"]);
    let replaced = workspace.replace(Selection {
        files: vec![SuppliedFile {
            path: "new.json".into(),
            content: "{}".into(),
        }],
        skipped: 0,
    });
    assert!(ResearchIndex::from(&replaced).activities.is_empty());
}
#[test]
fn test_index_normalizes_pid_relations() {
    let mut workspace = Workspace::default().replace(Selection {
        files: ["a.json", "b.json"]
            .into_iter()
            .map(|path| SuppliedFile {
                path: path.into(),
                content: "{}".into(),
            })
            .collect(),
        skipped: 0,
    });
    workspace.validations.insert(
        "a.json".into(),
        json!({"valid":true,"value":{"title":"Alpha","meta":{"identifier":"alpha","doi":"https://doi.org/10.1000/EXAMPLE"}}}),
    );
    workspace.validations.insert(
        "b.json".into(),
        json!({"valid":true,"value":{"title":"Beta","meta":{"identifier":"beta","doi":"10.1000/example"}}}),
    );
    let index = ResearchIndex::from(workspace);
    assert_eq!(
        index.related("a.json").iter().map(|activity| activity.path.as_str()).collect::<Vec<_>>(),
        ["b.json"]
    );
}
#[test]
fn test_index_search_and_identity_survive_restore() {
    let mut workspace = Workspace::default().replace(Selection {
        files: ["a.json", "b.json"]
            .into_iter()
            .map(|path| SuppliedFile {
                path: path.into(),
                content: "{}".into(),
            })
            .collect(),
        skipped: 0,
    });
    workspace.validations.insert(
        "a.json".into(),
        json!({"valid":true,"value":{"title":"Climate data","meta":{"identifier":"climate","contributors":[{"name":"Ada"}]}}}),
    );
    workspace.validations.insert(
        "b.json".into(),
        json!({"valid":true,"value":{"title":"Data methods","meta":{"identifier":"methods","organizations":[{"name":"Climate Lab"}]}}}),
    );
    let index = ResearchIndex::from(&workspace);
    assert_eq!(
        index.search("climate", CatalogSort::Relevance).first().expect("matching activity").path,
        "a.json"
    );
    assert_eq!(index.search("climate", CatalogSort::Alphabetical).len(), 2);
    assert_eq!(
        index.search("ada", CatalogSort::Alphabetical).first().expect("matching activity").path,
        "a.json"
    );
    let restored = serde_json::from_str::<Workspace>(&serde_json::to_string(&workspace).unwrap())
        .unwrap()
        .restored()
        .unwrap();
    assert_eq!(index, ResearchIndex::from(&restored));
    assert_eq!(index, ResearchIndex::from(restored));
}
#[test]
fn test_maximum_workspace_projects_are_deterministic() {
    let files = (0..500)
        .map(|index| SuppliedFile {
            path: format!("activity-{index:03}.json"),
            content: "{}".into(),
        })
        .collect();
    let mut workspace = Workspace::default().replace(Selection { files, skipped: 0 });
    for file in &workspace.files {
        workspace.validations.insert(
            file.path.clone(),
            json!({"valid":true,"value":{"title":"Shared title","meta":{"identifier":"shared-project"}}}),
        );
    }
    let index = ResearchIndex::from(&workspace);
    assert_eq!(index.activities.len(), 500);
    assert_eq!(index.projects.len(), 1);
    assert_eq!(index.projects.first().expect("one project").paths.len(), 500);
    assert_eq!(index.search("shared", CatalogSort::Relevance).len(), 500);
    assert_eq!(
        index.search("shared", CatalogSort::Relevance).first().expect("matching activity").path,
        "activity-000.json"
    );
}
#[test]
fn test_related_activities_and_source_links() {
    let mut workspace = Workspace::default().replace(Selection {
        files: ["a.json", "b.json", "c.json"]
            .into_iter()
            .map(|path| SuppliedFile {
                path: path.into(),
                content: "{}".into(),
            })
            .collect(),
        skipped: 0,
    });
    workspace.validations.insert("a.json".into(), json!({"valid":true,"value":{"title":"Alpha","meta":{"identifier":"alpha","organizations":[{"name":"Lab"}],"doi":"https://doi.org/10.1000/example"}}}));
    workspace.validations.insert(
        "b.json".into(),
        json!({"valid":true,"value":{"title":"Beta","meta":{"identifier":"beta","organizations":[{"name":"Lab"}],"doi":"10.1000/example"}}}),
    );
    workspace.validations.insert(
        "c.json".into(),
        json!({"valid":true,"value":{"title":"Gamma","meta":{"identifier":"gamma"}}}),
    );
    let index = ResearchIndex::from(&workspace);
    assert_eq!(
        index.related("a.json").iter().map(|activity| activity.path.as_str()).collect::<Vec<_>>(),
        ["b.json"]
    );
    let source = WorkspaceSource {
        kind: "gitlab".into(),
        host: "gitlab.com".into(),
        project: "team/project".into(),
        path: String::new(),
        revision: "a".repeat(40),
    };
    assert!(source.link("a.json").unwrap().ends_with(&format!("/{}/a.json", source.revision)));
    assert!(source.link("data/field notes.json").unwrap().ends_with("/data/field%20notes.json"));
    assert!(source.link("data/field#notes?.json").unwrap().ends_with("/data/field%23notes%3F.json"));
    assert!(source.link("../outside.json").is_none());
    assert!(WorkspaceSource {
        host: "evil/path".into(),
        ..source
    }
    .link("a.json")
    .is_none());
}
