//! Browser workspace and presentation unit tests.
use crate::icon::IconName;
use acorn_web::research_index::{CatalogSort, ResearchIndex};
use acorn_web::search::SearchPattern;
use acorn_web::session::{Selection, SuppliedFile, Workspace, WorkspaceSource};
use serde_json::json;

#[test]
fn test_icon_name_parses_existing_names() {
    assert_eq!("filecheck".parse::<IconName>(), Ok(IconName::FileCheck));
    assert_eq!("folder-open".parse::<IconName>(), Ok(IconName::FolderOpen));
    assert_eq!("font-big".parse::<IconName>(), Ok(IconName::FontBig));
    assert_eq!("sort-title".parse::<IconName>(), Ok(IconName::SortTitle));
    assert_eq!("x".parse::<IconName>(), Ok(IconName::X));
    assert!("missing".parse::<IconName>().is_err());
    assert!(!IconName::FileCheck.legacy_artwork().is_empty());
    assert_eq!(IconName::FileCheck.uswds_symbol(), Some("verified"));
}
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
    let value = json!({"title":"A project","meta":{"identifier":"project-one","organizations":[{"name":"Example"}],"keywords":["visualization","geospatial"]}});
    for path in ["a.json", "b.json"] {
        workspace.validations.insert(path.into(), json!({"valid":true,"value":value}));
    }
    workspace.validations.insert("invalid.json".into(), json!({"valid":false,"value":value}));
    let index = ResearchIndex::from(&workspace);
    assert_eq!(index.activities.len(), 2);
    assert_eq!(index.projects.len(), 1);
    assert_eq!(index.projects.first().expect("one project").paths, ["a.json", "b.json"]);
    assert_eq!(index.projects.first().expect("one project").keywords, ["geospatial", "visualization"]);
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
        json!({"valid":true,"value":{"title":"Alpha","meta":{"identifier":"alpha","doi":"10.1000/alpha","related":["https://doi.org/10.1000/EXAMPLE"]}}}),
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
    assert!(index.related("b.json").is_empty());
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
fn test_index_search_respects_regex_and_case() {
    let mut workspace = Workspace::default().replace(Selection {
        files: vec![SuppliedFile {
            path: "gold/index.json".into(),
            content: "{}".into(),
        }],
        skipped: 0,
    });
    workspace.validations.insert(
        "gold/index.json".into(),
        json!({"valid":true,"value":{"title":"Gold project banner","meta":{"identifier":"gold"}}}),
    );
    let index = ResearchIndex::from(&workspace);
    let regex = SearchPattern::new("Gold.*banner", true, true).expect("valid regex");
    assert_eq!(index.search_with(&regex, CatalogSort::Alphabetical).len(), 1);
    let sensitive = SearchPattern::new("GOLD", false, true).expect("literal query");
    assert!(index.search_with(&sensitive, CatalogSort::Alphabetical).is_empty());
    let insensitive = SearchPattern::new("GOLD", false, false).expect("literal query");
    assert_eq!(index.search_with(&insensitive, CatalogSort::Alphabetical).len(), 1);
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
    workspace.validations.insert("a.json".into(), json!({"valid":true,"value":{"title":"Alpha","meta":{"identifier":"alpha","organizations":[{"name":"Lab"}],"doi":"https://doi.org/10.1000/example","relatedResources":[{"identifier":{"value":"gamma"},"relationType":"Related"},{"identifier":{"value":"beta"},"relationType":"Related","category":"sponsors"}]}}}));
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
        ["c.json"]
    );
    assert!(index.related("b.json").is_empty());
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
mod geospatial_tests {
    use acorn_web::geospatial::*;
    use acorn_web::research_index::ResearchIndex;
    use acorn_web::session::{Selection, SuppliedFile, Workspace};
    use alloc::collections::BTreeSet;
    use serde_json::json;
    use std::fs::read_to_string;
    use std::time::Instant;

    #[test]
    fn test_local_join_deduplicates_and_preserves_unresolved_locationless_records() {
        let mut workspace = Workspace::default().replace(Selection {
            files: vec![SuppliedFile {
                path: "sample.json".into(),
                content: "{}".into(),
            }],
            skipped: 0,
        });
        workspace.validations.insert("sample.json".into(), json!({"valid":true,"value":{"meta":{"ror":["01qz5mb56","https://ror.org/01qz5mb56","http://ror.org/01qz5mb56","invalid","https://ror.org/04aj4c181","https://ror.org/05bnh6r87"]}}}));
        let index: LocationIndex = serde_json::from_value(json!({"version":"fixture","date":"fixture","source":"fixture","sourceSha256":"fixture","license":"CC0-1.0","organizations":{
            "https://ror.org/01qz5mb56":{"name":"Example","locations":[{"country":"US","latitude":91,"longitude":0},{"country":"US","latitude":35,"longitude":-84}]},
            "https://ror.org/04aj4c181":{"name":"Locationless","locations":[]}
        }})).unwrap();
        let rows = markers(&workspace, &index);
        assert_eq!(
            referenced_identifiers(&workspace, &BTreeSet::new()),
            rows.iter().map(|marker| marker.id.clone()).collect()
        );
        let first = rows.iter().find(|marker| marker.id.ends_with("01qz5mb56")).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(first.id, "https://ror.org/01qz5mb56");
        assert!(first.resolved);
        assert_eq!(first.references, 3);
        assert_eq!(first.location.as_ref().unwrap().latitude, 35.0);
        assert_eq!(first.activities.len(), 1);
        assert!(rows.iter().any(|marker| marker.resolved && marker.location.is_none()));
        assert!(rows.iter().any(|marker| !marker.resolved));
        assert!(markers(&Workspace::default(), &index).is_empty());
        let pins = BTreeSet::from([
            "01qz5mb56".into(),
            "http://ror.org/01qz5mb56".into(),
            "https://ror.org/04aj4c181".into(),
            "https://ror.org/05bnh6r87".into(),
            "https://ror.org/01qz5mb56xxx".into(),
            "not-a-ror".into(),
        ]);
        let standalone = markers_with_identifiers(&Workspace::default(), &index, &pins);
        assert_eq!(
            referenced_identifiers(&Workspace::default(), &pins),
            standalone.iter().map(|marker| marker.id.clone()).collect()
        );
        assert_eq!(standalone.len(), 3);
        assert!(standalone.iter().all(|marker| marker.references == 0 && marker.activities.is_empty()));
        assert_eq!(standalone.iter().filter(|marker| marker.location.is_some()).count(), 1);
        assert!(standalone.iter().any(|marker| marker.resolved && marker.location.is_none()));
        assert!(standalone.iter().any(|marker| !marker.resolved && marker.location.is_none()));
        assert_eq!(markers_with_identifiers(&workspace, &index, &pins), rows);
        assert_eq!(canonical_ror(" 01qz5mb56 "), Some("https://ror.org/01qz5mb56".into()));
        assert_eq!(canonical_ror("https://ror.org/01qz5mb56xxx"), None);
    }
    #[test]
    fn test_maximum_bucket_projects_and_map_payload_stay_bounded_after_restore_and_replacement() {
        let source = read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/geospatial/ror-locations.json")).expect("bundled local index");
        let mut index: LocationIndex = serde_json::from_str(&source).expect("valid local index");
        let ids = index
            .organizations
            .iter()
            .filter(|(id, organization)| canonical_ror(id).is_some() && !organization.locations.is_empty())
            .take(1_000)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 1_000);
        index.organizations.retain(|id, _| ids.binary_search(id).is_ok());
        let files = (0..500)
            .map(|number| SuppliedFile {
                path: format!("activity-{number:03}.json"),
                content: "{}".into(),
            })
            .collect();
        let mut workspace = Workspace::default().replace(Selection { files, skipped: 0 });
        for (number, [first, second]) in ids.as_chunks::<2>().0.iter().enumerate() {
            workspace.validations.insert(
                format!("activity-{number:03}.json"),
                json!({"valid":true,"value":{
                    "title":format!("Project {number:03}"),
                    "meta":{"identifier":format!("project-{number:03}"),"ror":[first,second]}
                }}),
            );
        }
        let started = Instant::now();
        let rows = markers(&workspace, &index);
        let catalog = ResearchIndex::from(&workspace);
        let payload = map_payload(&rows, &catalog.projects, rows.first().map(|row| row.id.as_str()));
        let elapsed = started.elapsed();
        let encoded = payload.to_string();
        eprintln!(
            "ABW-18: 500 files, 500 projects, 1000 markers: join + payload {elapsed:?}; payload {} bytes",
            encoded.len()
        );
        assert_eq!(catalog.projects.len(), 500);
        assert_eq!(rows.len(), 1_000);
        assert!(rows.iter().all(|row| row.project_ids.len() == 1 && row.location.is_some()));
        assert!(
            encoded.len() < 1_000_000,
            "map payload must stay below one megabyte for this maximum bucket"
        );
        assert!(encoded.contains("\"version\":1"));
        assert!(!encoded.contains("\"activities\""), "map payload must not copy activity paths");
        let restored = serde_json::from_str::<Workspace>(&serde_json::to_string(&workspace).expect("serialize workspace"))
            .expect("deserialize workspace")
            .restored()
            .expect("bounded restored workspace");
        assert_eq!(markers(&restored, &index), rows);
        let replaced = restored.replace(Selection {
            files: vec![SuppliedFile {
                path: "new.json".into(),
                content: "{}".into(),
            }],
            skipped: 0,
        });
        assert!(markers(&replaced, &index).is_empty());
        assert!(ResearchIndex::from(&replaced).projects.is_empty());
    }
    #[test]
    fn test_project_memberships_share_organizations_without_duplicate_locations() {
        let files = [
            "alpha/first.json",
            "alpha/second.json",
            "beta/index.json",
            "gamma/no-id.json",
            "invalid.json",
        ]
        .into_iter()
        .map(|path| SuppliedFile {
            path: path.into(),
            content: "{}".into(),
        })
        .collect();
        let mut workspace = Workspace::default().replace(Selection { files, skipped: 0 });
        workspace.validations.insert(
            "alpha/first.json".into(),
            json!({"valid":true,"value":{"title":"Alpha","meta":{"identifier":"project-alpha","ror":["01qz5mb56","04aj4c181"]}}}),
        );
        workspace.validations.insert(
            "alpha/second.json".into(),
            json!({"valid":true,"value":{"title":"Alpha part two","meta":{"identifier":"project-alpha","ror":["01qz5mb56"]}}}),
        );
        workspace.validations.insert(
            "beta/index.json".into(),
            json!({"valid":true,"value":{"title":"Beta","meta":{"identifier":"project-beta","ror":["01qz5mb56"]}}}),
        );
        workspace.validations.insert(
            "gamma/no-id.json".into(),
            json!({"valid":true,"value":{"title":"Gamma","meta":{"ror":["05bnh6r87"]}}}),
        );
        workspace
            .validations
            .insert("invalid.json".into(), json!({"valid":false,"value":{"meta":{"ror":["01qz5mb56"]}}}));
        let index: LocationIndex = serde_json::from_value(
            json!({"version":"fixture","date":"fixture","source":"fixture","sourceSha256":"fixture","license":"CC0-1.0","organizations":{
                "https://ror.org/01qz5mb56":{"name":"Shared","locations":[{"country":"US","latitude":35,"longitude":-84}]},
                "https://ror.org/04aj4c181":{"name":"Locationless","locations":[]}
            }}),
        )
        .expect("valid location fixture");
        let rows = markers(&workspace, &index);
        let shared = rows.iter().find(|marker| marker.id.ends_with("01qz5mb56")).expect("shared organization");
        let locationless = rows
            .iter()
            .find(|marker| marker.id.ends_with("04aj4c181"))
            .expect("locationless organization");
        let unresolved = rows
            .iter()
            .find(|marker| marker.id.ends_with("05bnh6r87"))
            .expect("unresolved organization");
        assert_eq!(rows.len(), 3);
        assert_eq!(shared.references, 3);
        assert_eq!(shared.activities.len(), 3);
        assert_eq!(shared.project_ids, BTreeSet::from(["project-alpha".into(), "project-beta".into()]));
        assert_eq!(locationless.project_ids, BTreeSet::from(["project-alpha".into()]));
        assert!(locationless.location.is_none());
        assert_eq!(unresolved.project_ids, BTreeSet::from(["gamma/no-id.json".into()]));
        assert!(!unresolved.resolved);
    }
}
mod media_tests {
    use acorn_web::session::{SuppliedFile, Workspace};
    use serde_json::json;

    #[test]
    fn test_absolute_media_reference_uses_loaded_file() {
        let workspace = Workspace {
            files: vec![SuppliedFile {
                path: "gold/banner.png".into(),
                content: "data:image/png;base64,cover".into(),
            }],
            ..Workspace::default()
        };
        let value =
            json!({"meta":{"media":[{"caption":"Banner","contentUrl":"https://raw.githubusercontent.com/example/bucket/main/gold/banner.png"}]}});
        let slides = workspace.slides("gold/index.json", &value);
        assert_eq!(slides.first().map(|slide| slide.src()), Some("data:image/png;base64,cover"));
    }
    #[test]
    fn test_absolute_svg_reference_resolves_loaded_path() {
        let workspace = Workspace {
            files: vec![SuppliedFile {
                path: "gold/banner.svg".into(),
                content: "<svg/>".into(),
            }],
            ..Workspace::default()
        };
        let value = json!({"meta":{"media":[{"caption":"Gold project banner","contentUrl":"https://raw.githubusercontent.com/jhwohlgemuth/bucket/main/gold/banner.svg"}]}});
        let slides = workspace.slides("gold/index.json", &value);
        assert_eq!(slides.first().map(|slide| slide.src()), Some("data:image/svg+xml;base64,PHN2Zy8+"));
    }
    #[test]
    fn test_project_slides_keep_explicit_missing_references() {
        let workspace = Workspace {
            files: vec![SuppliedFile {
                path: "with-image/cover.png".into(),
                content: "data:image/png;base64,cover".into(),
            }],
            validations: [
                (
                    "missing/index.json".into(),
                    json!({"value":{"meta":{"media":[{"contentUrl":"missing.png","caption":"Missing cover"}]}}}),
                ),
                ("with-image/index.json".into(), json!({"value":{}})),
            ]
            .into(),
            ..Workspace::default()
        };
        let paths = vec!["missing/index.json".into(), "with-image/index.json".into()];
        let slides = workspace.project_slides(&paths);
        assert_eq!(slides.len(), 2);
        assert_eq!(slides.first().map(|slide| slide.caption()), Some("Missing cover"));
        assert_eq!(slides.get(1).map(|slide| slide.caption()), Some("cover.png"));
    }
    #[test]
    fn test_project_slides_skip_implicit_missing_images() {
        let workspace = Workspace {
            files: vec![SuppliedFile {
                path: "with-image/cover.png".into(),
                content: "data:image/png;base64,cover".into(),
            }],
            validations: [
                ("without-image/index.json".into(), json!({"value":{}})),
                ("with-image/index.json".into(), json!({"value":{}})),
            ]
            .into(),
            ..Workspace::default()
        };
        let paths = vec!["without-image/index.json".into(), "with-image/index.json".into()];
        let slides = workspace.project_slides(&paths);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides.first().map(|slide| slide.caption()), Some("cover.png"));
    }
    #[test]
    fn test_project_slides_use_one_fallback_when_empty() {
        let workspace = Workspace {
            validations: [
                ("first/index.json".into(), json!({"value":{}})),
                ("second/index.json".into(), json!({"value":{}})),
            ]
            .into(),
            ..Workspace::default()
        };
        let paths = vec!["first/index.json".into(), "second/index.json".into()];
        let slides = workspace.project_slides(&paths);
        assert_eq!(slides.len(), 1);
        assert_eq!(slides.first().map(|slide| slide.caption()), Some("No image available"));
    }
    #[test]
    fn test_video_media_does_not_hide_sibling_image_or_create_missing_slide() {
        let workspace = Workspace {
            files: vec![SuppliedFile {
                path: "project/cover.png".into(),
                content: "data:image/png;base64,cover".into(),
            }],
            ..Workspace::default()
        };
        let value = json!({"meta":{"media":[{"contentUrl":"https://example.org/movie.mp4?poster=cover.png","description":"Field recording"}]}});
        let images = workspace.slides("project/index.json", &value);
        assert_eq!(images.len(), 1);
        assert_eq!(images.first().map(|slide| slide.caption()), Some("cover.png"));
        assert_eq!(images.first().map(|slide| slide.src()), Some("data:image/png;base64,cover"));
        let fallback = Workspace::default().slides("project/index.json", &value);
        assert_eq!(fallback.first().map(|slide| slide.caption()), Some("No image available"));
    }
}
mod research_graph_tests {
    use crate::research_graph::{
        analyze_graph, entity_href, graph_analysis_within_budget, graph_parameters, keywords, metric_spectrum, pid_href, project, visible,
        GraphMetrics, DIAGRAM_EDGE_LIMIT, DIAGRAM_NODE_LIMIT,
    };
    use acorn_schema::research_activity::graph::{GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind, ResearchGraph};
    use acorn_web::{
        search::SearchPattern,
        session::{SuppliedFile, Workspace},
    };
    use alloc::collections::BTreeMap;
    use serde_json::json;

    #[test]
    fn test_entity_links_require_a_supported_identifier() {
        assert_eq!(pid_href("10.1234/example"), Some("https://doi.org/10.1234/example".into()));
        assert_eq!(pid_href("javascript:alert(1)"), None);
    }
    #[test]
    fn test_focus_radius_follows_the_selected_relationship_type() {
        let nodes = ["a", "b", "c"]
            .into_iter()
            .map(|id| GraphNode {
                id: id.into(),
                kind: GraphNodeKind::Activity,
                label: id.into(),
                source: "sample".into(),
                path: "sample.json".into(),
            })
            .collect();
        let edges = [("a", "b", GraphEdgeKind::RelatesTo), ("b", "c", GraphEdgeKind::Credits)]
            .into_iter()
            .enumerate()
            .map(|(index, (source, target, kind))| GraphEdge {
                id: index.to_string(),
                kind,
                source: source.into(),
                target: target.into(),
                path: "sample.json".into(),
                field: "relatedResources".into(),
                relation_type: None,
            })
            .collect();
        let graph = ResearchGraph { nodes, edges };
        let pattern = SearchPattern::new("", false, false);
        assert_eq!(visible(&graph, &BTreeMap::new(), &pattern, "", "", Some("a"), 2).0.len(), 3);
        assert_eq!(visible(&graph, &BTreeMap::new(), &pattern, "", "relatesTo", Some("a"), 2).0.len(), 2);
    }
    #[test]
    fn test_graph_analysis_deduplicates_links_and_measures_bridge_nodes() {
        let nodes = ["a", "b", "c", "d"]
            .into_iter()
            .map(|id| GraphNode {
                id: id.into(),
                kind: GraphNodeKind::Activity,
                label: id.into(),
                source: "sample".into(),
                path: "sample.json".into(),
            })
            .collect();
        let edges = [("a", "b"), ("b", "c"), ("c", "a"), ("c", "d"), ("b", "a"), ("a", "a")]
            .into_iter()
            .enumerate()
            .map(|(index, (source, target))| GraphEdge {
                id: index.to_string(),
                kind: GraphEdgeKind::RelatesTo,
                source: source.into(),
                target: target.into(),
                path: "sample.json".into(),
                field: "relatedResources".into(),
                relation_type: None,
            })
            .collect();
        let analysis = analyze_graph(&ResearchGraph { nodes, edges });
        let a = analysis.get("a").expect("metrics for node a");
        let c = analysis.get("c").expect("metrics for node c");
        let d = analysis.get("d").expect("metrics for node d");
        assert!((a.clustering - 1.0).abs() < 1e-12);
        assert!((c.clustering - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(d.clustering, 0.0);
        assert!((a.degree - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(c.degree, 1.0);
        assert!((a.closeness - 0.75).abs() < 1e-12);
        assert_eq!(c.closeness, 1.0);
        assert_eq!(a.betweenness, 0.0);
        assert!((c.betweenness - 2.0 / 3.0).abs() < 1e-12);
        let smallest = metric_spectrum(&analysis, d.degree, |metrics| metrics.degree);
        let middle = metric_spectrum(&analysis, a.degree, |metrics| metrics.degree);
        let largest = metric_spectrum(&analysis, c.degree, |metrics| metrics.degree);
        assert!((smallest.min - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(largest.max, 1.0);
        assert_eq!(smallest.position, 0.0);
        assert!((middle.position - 50.0).abs() < 1e-12);
        assert_eq!(largest.position, 100.0);
        let equal = BTreeMap::from([
            (
                "one".into(),
                GraphMetrics {
                    degree: 0.5,
                    ..GraphMetrics::default()
                },
            ),
            (
                "two".into(),
                GraphMetrics {
                    degree: 0.5,
                    ..GraphMetrics::default()
                },
            ),
        ]);
        let equal_spectrum = metric_spectrum(&equal, 0.5, |metrics| metrics.degree);
        assert_eq!((equal_spectrum.min, equal_spectrum.max, equal_spectrum.position), (0.5, 0.5, 50.0));
    }
    #[test]
    fn test_graph_analysis_respects_diagram_budget() {
        let nodes = (0..DIAGRAM_NODE_LIMIT)
            .map(|index| GraphNode {
                id: index.to_string(),
                kind: GraphNodeKind::Activity,
                label: index.to_string(),
                source: "sample".into(),
                path: "sample.json".into(),
            })
            .collect();
        let mut graph = ResearchGraph { nodes, edges: Vec::new() };
        assert!(graph_analysis_within_budget(&graph));
        graph.nodes.push(GraphNode {
            id: "extra".into(),
            kind: GraphNodeKind::Activity,
            label: "extra".into(),
            source: "sample".into(),
            path: "sample.json".into(),
        });
        assert!(!graph_analysis_within_budget(&graph));
        graph.nodes.pop();
        graph.edges = vec![
            GraphEdge {
                id: "edge".into(),
                kind: GraphEdgeKind::RelatesTo,
                source: "0".into(),
                target: "1".into(),
                path: "sample.json".into(),
                field: "relatedResources".into(),
                relation_type: None,
            };
            DIAGRAM_EDGE_LIMIT + 1
        ];
        assert!(!graph_analysis_within_budget(&graph));
    }
    #[test]
    fn test_graph_parameters_measure_reachable_paths() {
        let nodes = ["a", "b", "c", "d", "e"]
            .into_iter()
            .map(|id| GraphNode {
                id: id.into(),
                kind: GraphNodeKind::Activity,
                label: id.into(),
                source: "sample".into(),
                path: "sample.json".into(),
            })
            .collect();
        let edges = [("a", "b"), ("b", "c"), ("c", "d")]
            .into_iter()
            .enumerate()
            .map(|(index, (source, target))| GraphEdge {
                id: index.to_string(),
                kind: GraphEdgeKind::RelatesTo,
                source: source.into(),
                target: target.into(),
                path: "sample.json".into(),
                field: "relatedResources".into(),
                relation_type: None,
            })
            .collect();
        let graph = ResearchGraph { nodes, edges };
        let parameters = graph_parameters(&graph);
        assert_eq!(parameters.diameter, 3);
        assert_eq!(parameters.components, 2);
        assert!((parameters.average_shortest_path - 5.0 / 3.0).abs() < 1e-12);
        assert_eq!(parameters.average_clustering, 0.0);
        assert!((parameters.density - 0.3).abs() < 1e-12);
        let pattern = SearchPattern::new("", false, false);
        assert_eq!(visible(&graph, &BTreeMap::new(), &pattern, "", "", Some("a"), 3).0.len(), 4);
    }
    #[test]
    fn test_invalid_and_deleted_workspace_has_no_graph() {
        let workspace = Workspace {
            files: vec![SuppliedFile {
                path: "bad.json".into(),
                content: "{}".into(),
            }],
            validations: [("bad.json".into(), json!({"valid":false,"value":{}}))].into(),
            ..Workspace::default()
        };
        assert!(project(&workspace).expect("workspace graph projection").nodes.is_empty());
        assert!(project(&Workspace::default()).expect("empty workspace graph projection").nodes.is_empty());
    }
    #[test]
    fn test_validated_sample_projects_and_filters() {
        let sample = include_str!("../../assets/examples/publications.json");
        let mut value = serde_json::from_str::<serde_json::Value>(sample).expect("valid sample JSON");
        *value.pointer_mut("/meta/keywords").expect("sample keywords") = json!(["distinct-graph-keyword"]);
        let workspace = Workspace {
            files: vec![SuppliedFile {
                path: "publications.json".into(),
                content: sample.into(),
            }],
            validations: [("publications.json".into(), json!({"valid":true,"value":value}))].into(),
            ..Workspace::default()
        };
        let graph = project(&workspace).expect("workspace graph projection");
        let keyword_map = keywords(&workspace);
        assert!(graph.nodes.iter().any(|node| node.kind.as_ref() == "activity"));
        assert!(!graph.edges.is_empty());
        assert!(graph.nodes.len() <= DIAGRAM_NODE_LIMIT);
        let missing = SearchPattern::new("missing entity", false, false);
        let (nodes, edges) = visible(&graph, &keyword_map, &missing, "", "", None, 1);
        assert!(nodes.is_empty());
        assert!(edges.is_empty());
        let keyword = SearchPattern::new("distinct-graph-keyword", false, false);
        let (keyword_nodes, _) = visible(&graph, &keyword_map, &keyword, "", "", None, 1);
        assert_eq!(keyword_nodes.len(), 1);
        assert_eq!(keyword_nodes.first().map(|node| node.kind), Some(GraphNodeKind::Activity));
        let regex = SearchPattern::new("^distinct-graph-keyword$", true, false);
        assert_eq!(visible(&graph, &keyword_map, &regex, "", "", None, 1).0.len(), 1);
        let case_sensitive = SearchPattern::new("DISTINCT-GRAPH-KEYWORD", false, true);
        assert!(visible(&graph, &keyword_map, &case_sensitive, "", "", None, 1).0.is_empty());
        let invalid = SearchPattern::new("[", true, false);
        assert!(visible(&graph, &keyword_map, &invalid, "", "", None, 1).0.is_empty());
        let activity = graph
            .nodes
            .iter()
            .find(|node| node.kind.as_ref() == "activity")
            .expect("sample activity node");
        let project_node = graph
            .nodes
            .iter()
            .find(|node| node.kind == GraphNodeKind::Project)
            .expect("sample project node");
        assert!(entity_href(activity, &graph).is_some());
        assert_eq!(entity_href(project_node, &graph), None);
        let all = SearchPattern::new("", false, false);
        let (nearby, _) = visible(&graph, &keyword_map, &all, "", "", Some(&activity.id), 1);
        let (extended, _) = visible(&graph, &keyword_map, &all, "", "", Some(&activity.id), 2);
        assert!(extended.len() > nearby.len());
    }
}
mod search_tests {
    use acorn_web::search::SearchPattern;

    #[test]
    fn test_invalid_regex_reports_error() {
        assert!(SearchPattern::new("[", true, false).is_err());
    }
    #[test]
    fn test_literal_and_regex_case_modes() {
        let literal = SearchPattern::new("Gold banner", false, false).expect("literal query");
        assert!(literal.matches_terms("Gold project banner"));
        assert!(!literal.matches("Gold project banner"));
        let sensitive = SearchPattern::new("Gold", false, true).expect("literal query");
        assert!(!sensitive.matches("gold"));
        let regex = SearchPattern::new("Gold.*banner", true, true).expect("regex query");
        assert!(regex.matches("Gold project banner"));
        assert!(!regex.matches("gold project banner"));
    }
}
mod session_tests {
    use acorn_web::session::*;
    use data_encoding::BASE64;
    use serde_json::json;
    fn file(path: &str, content: &str) -> SuppliedFile {
        SuppliedFile {
            path: path.into(),
            content: content.into(),
        }
    }
    #[test]
    fn test_failed_selection_preserves_content_and_validation() {
        let original = Workspace::default().replace(Selection {
            files: vec![file("ok.json", "{}")],
            skipped: 0,
        });
        let next = original.replace(Selection {
            files: vec![file("../bad.json", "bad")],
            skipped: 2,
        });
        assert_eq!(next.files, original.files);
        assert_eq!(next.selected, original.selected);
        assert_eq!(next.skipped, 3);
    }
    #[test]
    fn test_identifier_labels_deduplicate_values_across_fields_and_activities() {
        let files = [file("a.json", "{}"), file("b.json", "{}")];
        let values = [
            json!({"meta":{"doi":"https://doi.org/10.1234/a","ror":"https://ror.org/01qz5mb56","contributors":[{"identifiers":[{"value":"https://orcid.org/0000-0000-0000-0001","scheme":"ORCID"},{"value":"https://isni.org/isni/000000012146438X","schemeUri":"https://isni.org"}]}],"organizations":[{"identifiers":[{"value":"https://ror.org/01qz5mb56","scheme":"ROR"}]}]}}),
            json!({"meta":{"doi":"10.1234/b","raid":"https://raid.org.au/10.25917/example","contributors":[{"identifiers":[{"value":"https://orcid.org/0000-0000-0000-0001","scheme":"orcid"}]}],"organizations":[{"identifiers":[{"value":"https://ror.org/01qz5mb56"}]}]}}),
        ];
        let records = files.iter().zip(values.iter()).collect::<Vec<_>>();
        let counts = distribution(identifier_labels(&records));
        assert_eq!(
            counts,
            [("DOI", 2), ("ISNI", 1), ("ORCID", 1), ("RAiD", 1), ("ROR", 1)]
                .into_iter()
                .map(|(label, count)| (label.to_owned(), count))
                .collect()
        );
        assert_eq!(counts.values().sum::<usize>(), identifiers(&records).len());
        let first_record = records.iter().take(1).copied().collect::<Vec<_>>();
        assert_eq!(distribution(identifier_labels(&first_record)).get("DOI"), Some(&1));
    }
    #[test]
    fn test_images_require_canonical_raster_bytes_and_do_not_seed_text_tools() {
        let bytes = b"\x89PNG\r\n\x1a\nfixture";
        let image = format!("data:image/png;base64,{}", BASE64.encode(bytes));
        let jpeg = format!("data:image/jpeg;base64,{}", BASE64.encode(b"\xff\xd8\xfffixture"));
        let webp = format!("data:image/webp;base64,{}", BASE64.encode(b"RIFFxxxxWEBPfixture"));
        let workspace = Workspace::default().replace(Selection {
            files: vec![
                file("a/cover.png", &image),
                file("a/detail.jpeg", &jpeg),
                file("a/diagram.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>"),
                file("a/chart.webp", &webp),
                file("a/index.json", "{}"),
                file("a/wrong.jpg", &image),
                file("a/invalid.webp", "data:image/webp;base64,not-base64"),
            ],
            skipped: 0,
        });
        assert_eq!(workspace.files.len(), 5);
        assert_eq!(workspace.skipped, 2);
        assert_eq!(workspace.selected.as_deref(), Some("a/index.json"));
        assert_eq!(workspace.content(), "{}");
        assert_eq!(workspace.clone().restored(), Some(workspace));
        let only_image = Workspace::default().replace(Selection {
            files: vec![file("cover.png", &image), file("diagram.svg", "<svg/>")],
            skipped: 0,
        });
        assert!(only_image.selected.is_none());
        assert!(only_image.content().is_empty());
        assert!(valid_path("a/cover.webp"));
    }
    #[test]
    fn test_limits_sorting_duplicates_and_traversal() {
        let workspace = Workspace::default().replace(Selection {
            files: vec![
                file("b/x.json", "{}"),
                file("a.txt", "text"),
                file("a.txt", "text"),
                file("/bad.json", ""),
                file("huge.txt", &"é".repeat(FILE_LIMIT)),
            ],
            skipped: 0,
        });
        assert_eq!(
            workspace.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
            ["a.txt", "b/x.json"]
        );
        assert_eq!(workspace.skipped, 3);
        assert_eq!(workspace.bytes(), 6);
        for path in ["../x.json", "a/../x.json", "a//b.json", "C:/x.json", "a\\b.json", "x.exe", "\n.json"] {
            assert!(!valid_path(path), "{path}");
        }
        assert!(!valid_path(&format!("{}x.json", "a/".repeat(20))));
        assert!(valid_path("a/é.JSON"));
        let many = Workspace::default().replace(Selection {
            files: (0..510).map(|i| file(&format!("{i:03}.txt"), "x")).collect(),
            skipped: 0,
        });
        assert_eq!(many.files.len(), 500);
        assert_eq!(many.skipped, 10);
        let large = Workspace::default().replace(Selection {
            files: (0..17).map(|i| file(&format!("{i:03}.txt"), &"x".repeat(FILE_LIMIT))).collect(),
            skipped: 0,
        });
        assert_eq!(large.bytes(), TOTAL_LIMIT);
        assert_eq!(large.skipped, 1);
    }
    #[test]
    fn test_persisted_workspace_rechecks_bounds_and_keeps_state() {
        let mut saved = Workspace::default().replace(Selection {
            files: vec![file("a.json", "{}")],
            skipped: 0,
        });
        saved.source = Some(WorkspaceSource {
            kind: "gitlab".into(),
            host: "code.ornl.gov".into(),
            project: "group/repo".into(),
            path: String::new(),
            revision: "a".repeat(40),
        });
        saved
            .validations
            .insert("a.json".into(), json!({"valid":true,"value":{"title":"forged"}}));
        let restored = saved.clone().restored().expect("valid saved workspace");
        assert_eq!(restored.files, saved.files);
        assert_eq!(restored.validations, saved.validations);
        assert_eq!(restored.source, saved.source);
        saved.files.first_mut().expect("saved file exists").path = "../a.json".into();
        assert!(saved.restored().is_none());
    }
    #[test]
    fn test_portfolio_uses_only_validated_records_and_deduplicates_entities() {
        let mut workspace = Workspace::default().replace(Selection {
            files: vec![file("a.json", "{}"), file("b.json", "{}"), file("invalid.json", "{}")],
            skipped: 0,
        });
        let value = json!({"title":"A","@type":"Project","meta":{"status":"active","doi":"https://doi.org/10.1234/x","organizations":[{"name":"Example","identifiers":[{"value":"https://ror.org/01qz5mb56"}]}]}});
        for name in ["a.json", "b.json"] {
            workspace.validations.insert(name.into(), json!({"valid":true,"value":value}));
        }
        workspace.validations.insert("invalid.json".into(), json!({"valid":false,"value":value}));
        let records = workspace.activities();
        assert_eq!(records.len(), 2);
        assert_eq!(entities(&records, "organizations").len(), 1);
        assert_eq!(identifiers(&records).len(), 2);
        assert_eq!(
            distribution(records.iter().map(|(_, value)| activity_type(value))).get("Project"),
            Some(&2)
        );
        let replaced = workspace.replace(Selection {
            files: vec![file("new.json", "{}")],
            skipped: 0,
        });
        assert!(replaced.activities().is_empty());
        assert_eq!(replaced.selected.as_deref(), Some("new.json"));
    }
}
mod template_tests {
    use acorn_core::util::MimeType;
    use acorn_schema::research_activity::ResearchActivity;
    use acorn_schema::Validate;
    #[cfg(target_arch = "wasm32")]
    use acorn_web::operations::{self, Request, Tool};
    use acorn_web::session::{Selection, Workspace};
    use acorn_web::templates::*;

    #[test]
    fn test_template_projects_fit_workspace_and_validate() {
        assert_eq!(TEMPLATES.len(), 3, "all bundled templates must load");
        for template in TEMPLATES.iter() {
            assert_eq!(template.files.len(), template.yaml_files.len(), "{} format variants", template.id);
            for (json, yaml) in template.files.iter().zip(&template.yaml_files) {
                let json_value: serde_json::Value = serde_json::from_str(&json.content).expect("JSON template must parse");
                let yaml_value: serde_json::Value = serde_norway::from_str(&yaml.content).expect("YAML template must parse");
                assert_eq!(json_value, yaml_value, "{} format variants must match", template.id);
                let activity: ResearchActivity = serde_json::from_value(yaml_value).expect("template must deserialize");
                assert!(activity.validate().is_ok(), "{} must validate", yaml.path);
            }
            for format in [MimeType::Json, MimeType::Yaml] {
                let files = template.files_for(&format);
                let loaded = Workspace::default().replace(Selection {
                    files: files.to_vec(),
                    skipped: 0,
                });
                assert_eq!(loaded.files.len(), files.len(), "{} {format} files", template.id);
                assert_eq!(loaded.skipped, 0, "{} {format} skipped files", template.id);
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_template_projects_pass_browser_validation() {
        for template in TEMPLATES.iter() {
            for file in template.files_for(&MimeType::Json) {
                let request = Request {
                    input: file.content.clone(),
                    ..Request::default()
                };
                let result = operations::run(Tool::Activity, &request).expect("template operation");
                assert_eq!(result["valid"], true, "{}: {}", file.path, result);
            }
            for file in template.files_for(&MimeType::Yaml) {
                let request = Request {
                    input: file.content.clone(),
                    format: MimeType::Yaml.file_type(),
                    ..Request::default()
                };
                let result = operations::run(Tool::Standard, &request).expect("YAML template operation");
                assert_eq!(result["valid"], true, "{}: {}", file.path, result);
            }
        }
    }
}
mod url_intent_tests {
    use crate::url_intent::*;

    fn entry(code: &[&str], template: &[&str], url: &[&str]) -> UrlEntry {
        UrlEntry {
            href: "https://example.org/acorn/review#fair".into(),
            parameters: Parameters {
                code: code.iter().map(ToString::to_string).collect(),
                template: template.iter().map(ToString::to_string).collect(),
                url: url.iter().map(ToString::to_string).collect(),
            },
            malformed: false,
        }
    }

    #[test]
    fn test_url_intent_accepts_one_value() {
        assert_eq!(entry(&["é + ?"], &[], &[]).intent(), Intent::Code("é + ?".into()));
        assert_eq!(entry(&[], &["single-activity"], &[]).intent(), Intent::Template("single-activity".into()));
        assert_eq!(
            entry(&[], &[], &["https://example.org/a.json"]).intent(),
            Intent::Url("https://example.org/a.json".into())
        );
    }
    #[test]
    fn test_url_intent_rejects_conflicts_and_bad_encoding() {
        assert!(matches!(entry(&["a", "b"], &[], &[]).intent(), Intent::Invalid(_)));
        assert!(matches!(entry(&["a"], &["single-activity"], &[]).intent(), Intent::Invalid(_)));
        assert!(matches!(entry(&[""], &[], &[]).intent(), Intent::Invalid(_)));
        assert!(matches!(entry(&[], &[], &["http://example.org/a.json"]).intent(), Intent::Invalid(_)));
        let mut malformed = entry(&["a"], &[], &[]);
        malformed.malformed = true;
        assert!(matches!(malformed.intent(), Intent::Invalid(_)));
    }
    #[test]
    fn test_url_intent_rejects_long_links() {
        let mut too_long = entry(&["a"], &[], &[]);
        too_long.href = "a".repeat(8 * 1024 + 1);
        assert!(matches!(too_long.intent(), Intent::Invalid(_)));
    }
}
