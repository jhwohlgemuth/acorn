use crate::workspace::{file_format, file_loadable, tag_colors, top_topics, tree_children, visible_file_paths, TAG_COLOR_COUNT};
use acorn_web::operations::Tool;
use acorn_web::research_index::Project;
use acorn_web::search::SearchPattern;
use acorn_web::session::SuppliedFile;
use alloc::collections::BTreeSet;
use serde_json::json;

#[test]
fn test_file_format_for_loaded_content() {
    assert_eq!(file_format("record.json"), Some("json"));
    assert_eq!(file_format("record.yaml"), Some("yaml"));
    assert_eq!(file_format("record.md"), Some("markdown"));
    assert_eq!(file_format("record.jsonld"), Some("json-ld"));
    assert_eq!(file_format("cover.svg"), None);
}
#[test]
fn test_loaded_file_matches_page_input() {
    let json = SuppliedFile {
        path: "project/index.json".into(),
        content: "{}".into(),
    };
    let yaml = SuppliedFile {
        path: "project/index.yaml".into(),
        content: "title: Test".into(),
    };
    let svg = SuppliedFile {
        path: "project/banner.svg".into(),
        content: "<svg/>".into(),
    };
    let long_text = SuppliedFile {
        path: "project/notes.txt".into(),
        content: "x".repeat(Tool::Inspect.limit() + 1),
    };
    assert!(file_loadable(&json, Tool::Activity));
    assert!(file_loadable(&json, Tool::Format));
    assert!(!file_loadable(&yaml, Tool::Activity));
    assert!(file_loadable(&yaml, Tool::Format));
    for extension in ["jsonld", "rdf", "ttl"] {
        let file = SuppliedFile {
            path: format!("project/record.{extension}"),
            content: "source".into(),
        };
        assert!(file_loadable(&file, Tool::Format));
    }
    assert!(file_loadable(
        &SuppliedFile {
            path: "project/index.md".into(),
            content: "# Project".into()
        },
        Tool::Format
    ));
    assert!(file_loadable(
        &SuppliedFile {
            path: "project/index.zonf".into(),
            content: "{}".into()
        },
        Tool::Format
    ));
    assert!(file_loadable(
        &SuppliedFile {
            path: "project/index.jsonc".into(),
            content: "{}".into()
        },
        Tool::Format
    ));
    assert!(file_loadable(&yaml, Tool::Standard));
    assert!(!file_loadable(&svg, Tool::Standard));
    assert!(!file_loadable(&long_text, Tool::Inspect));
}
#[test]
fn test_tag_colors_repeat_after_palette_and_ignore_project_order() {
    let tags = (0..=TAG_COLOR_COUNT).map(|index| format!("tag-{index:02}")).collect::<Vec<_>>();
    let first = tags.first().expect("first tag");
    let wrapped = tags.get(TAG_COLOR_COUNT).expect("tag after the palette");
    let project = |keywords: Vec<String>| Project {
        id: String::new(),
        title: String::new(),
        summary: String::new(),
        paths: Vec::new(),
        organizations: Vec::new(),
        keywords,
    };
    let projects = [project(tags.clone()), project(vec![first.clone()])];
    let colors = tag_colors(&projects);
    assert_eq!(colors.len(), TAG_COLOR_COUNT + 1);
    assert_eq!(colors.values().copied().collect::<BTreeSet<_>>().len(), TAG_COLOR_COUNT);
    assert_eq!(colors.get(first.as_str()), colors.get(wrapped.as_str()));
    assert_eq!(colors, tag_colors(&[projects[1].clone(), projects[0].clone()]));
}
#[test]
fn test_top_topics_counts_activities_once_and_sorts_by_frequency() {
    let files = (0..4)
        .map(|index| SuppliedFile {
            path: format!("{index}.json"),
            content: String::new(),
        })
        .collect::<Vec<_>>();
    let values = [
        json!({"meta":{"keywords":["mapping","mapping","visualization"]}}),
        json!({"meta":{"keywords":["mapping","geospatial"]}}),
        json!({"meta":{"keywords":["geospatial","visualization"]}}),
        json!({"meta":{"keywords":["mapping"]}}),
    ];
    let records = files.iter().zip(values.iter()).collect::<Vec<_>>();
    assert_eq!(
        top_topics(&records),
        [("mapping".into(), 3), ("geospatial".into(), 2), ("visualization".into(), 2)]
    );
    assert!(top_topics(&[]).is_empty());
}
#[test]
fn test_tree_children_keep_folders_first_in_both_directions() {
    let paths = vec!["z.txt".into(), "b/index.json".into(), "a.txt".into(), "a/index.json".into()];
    assert_eq!(
        tree_children(&paths, "", false),
        vec![(false, "a".into()), (false, "b".into()), (true, "a.txt".into()), (true, "z.txt".into())]
    );
    assert_eq!(
        tree_children(&paths, "", true),
        vec![(false, "b".into()), (false, "a".into()), (true, "z.txt".into()), (true, "a.txt".into())]
    );
}
#[test]
fn test_visible_file_paths_preserve_matching_hierarchy() {
    let files = vec![
        SuppliedFile {
            path: "Gold/index.json".into(),
            content: "{}".into(),
        },
        SuppliedFile {
            path: "Gold/banner.svg".into(),
            content: "<svg/>".into(),
        },
        SuppliedFile {
            path: "Other/notes.txt".into(),
            content: "notes".into(),
        },
    ];
    let gold = SearchPattern::new("gold", false, false).expect("literal query");
    let all = SearchPattern::new("", false, false).expect("empty query");
    assert_eq!(
        visible_file_paths(&files, &gold, "all", None, None),
        vec!["Gold/index.json", "Gold/banner.svg"]
    );
    assert_eq!(visible_file_paths(&files, &all, "image", None, None), vec!["Gold/banner.svg"]);
    assert_eq!(
        visible_file_paths(&files, &gold, "loadable", Some(Tool::Activity), None),
        vec!["Gold/index.json"]
    );
    let case_sensitive = SearchPattern::new("gold", false, true).expect("literal query");
    assert!(visible_file_paths(&files, &case_sensitive, "all", None, None).is_empty());
    let regex = SearchPattern::new(r"Gold/.*\.svg$", true, true).expect("regex query");
    assert_eq!(visible_file_paths(&files, &regex, "all", None, None), vec!["Gold/banner.svg"]);
    let selected = BTreeSet::from(["json".to_owned(), "text".to_owned()]);
    assert_eq!(
        visible_file_paths(&files, &all, "all", None, Some(&selected)),
        vec!["Gold/index.json", "Other/notes.txt"]
    );
    assert!(visible_file_paths(&files, &all, "all", None, Some(&BTreeSet::new())).is_empty());
}
