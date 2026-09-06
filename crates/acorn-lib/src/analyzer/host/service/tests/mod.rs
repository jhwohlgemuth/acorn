#![allow(clippy::indexing_slicing)]
use super::*;
use crate::analyzer::host::context::{AnalysisOptions, ExecutionOptions};
use crate::{check_err, check_ok};
use std::fs::{create_dir_all, write};
use tempfile::tempdir;

#[tokio::test]
async fn test_analyze_paths_collects_independent_failures_despite_early_exit_policy() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("CITATION.cff");
    write(&path, "invalid: [").expect("invalid CFF fixture should be written");
    let options = AnalysisContext::init()
        .analysis(
            AnalysisOptions::init()
                .skip(
                    [
                        CheckCategory::Link,
                        CheckCategory::Prose,
                        CheckCategory::Readability,
                        CheckCategory::Crosswalk,
                    ]
                    .into_iter()
                    .map(|category| category.to_string())
                    .collect(),
                )
                .build(),
        )
        .report(ReportOptions::init().exit_on_first_error(true).build())
        .build();
    let report = analyze_paths(&[path], &options).await;
    let categories = &report.batches[0].categories;
    assert_eq!(
        categories.iter().map(|result| result.category.clone()).collect::<Vec<_>>(),
        [CheckCategory::Schema, CheckCategory::Quality]
    );
    assert!(categories.iter().all(|result| result.checks.iter().any(Check::is_failure)));
}

#[tokio::test]
async fn test_analyze_paths_dispatches_every_standard() {
    for standard in [
        Standard::CitationFileFormat,
        Standard::Datacite,
        Standard::Dcat,
        Standard::Document,
        Standard::DublinCore,
        Standard::Huwise,
        Standard::Invenio,
        Standard::Kitfile,
        Standard::ModelKit,
        Standard::Raid,
        Standard::ResearchActivityData,
        Standard::Text,
    ] {
        let options = AnalysisContext::init()
            .analysis(
                AnalysisOptions::init()
                    .skip(CheckCategory::iter().map(|value| value.to_string()).collect())
                    .standard(standard)
                    .build(),
            )
            .build();
        let report = analyze_paths(&[PathBuf::from("missing.input")], &options).await;
        assert_eq!(report.batches.len(), 1, "missing batch for {standard}");
        assert_eq!(report.batches[0].standard, standard);
        assert_eq!(report.batches[0].paths, [PathBuf::from("missing.input")]);
        assert!(report.batches[0].categories.is_empty());
    }
}
#[tokio::test]
async fn test_analyze_paths_preserves_standard_and_path_group_order() {
    let options = AnalysisContext::init()
        .analysis(
            AnalysisOptions::init()
                .skip(CheckCategory::iter().map(|value| value.to_string()).collect())
                .build(),
        )
        .build();
    let paths = [
        PathBuf::from("CITATION.cff"),
        PathBuf::from("activity.json"),
        PathBuf::from("notes.md"),
        PathBuf::from("activity.yaml"),
    ];
    let report = analyze_paths(&paths, &options).await;
    assert_eq!(
        report
            .batches
            .iter()
            .map(|batch| (batch.standard, batch.paths.clone()))
            .collect::<Vec<_>>(),
        [
            (Standard::CitationFileFormat, vec![PathBuf::from("CITATION.cff")]),
            (
                Standard::ResearchActivityData,
                vec![PathBuf::from("activity.json"), PathBuf::from("activity.yaml")]
            ),
            (Standard::Text, vec![PathBuf::from("notes.md")]),
        ]
    );
}
#[test]
fn test_analyze_report_with_checks_groups_in_category_order_and_omits_empty_groups() {
    let options = AnalysisContext::default();
    let report = AnalysisReport {
        batches: Vec::new(),
        documents: DocumentIndexes::new(),
        skipped_categories: Vec::new(),
    }
    .with_checks(
        vec![
            check_ok!(CheckCategory::Prose, message: "prose"),
            check_err!(CheckCategory::Schema, message: "schema"),
            check_ok!(CheckCategory::Prose, message: "second prose"),
        ]
        .into(),
        &options,
    );
    assert_eq!(report.batches.len(), 1);
    assert_eq!(report.batches[0].standard, Standard::ResearchActivityData);
    assert!(report.batches[0].paths.is_empty());
    assert_eq!(
        report.batches[0]
            .categories
            .iter()
            .map(|value| value.category.clone())
            .collect::<Vec<_>>(),
        [CheckCategory::Schema, CheckCategory::Prose]
    );
    assert_eq!(report.batches[0].categories[1].checks.len(), 2);
    assert_eq!(
        report.batches[0].categories[1]
            .checks
            .iter()
            .map(|check| check.message.as_str())
            .collect::<Vec<_>>(),
        ["prose", "second prose"]
    );
}
#[test]
fn test_classify_paths_applies_explicit_standard_to_every_path() {
    let groups = classify_paths(&[PathBuf::from("one.json"), PathBuf::from("two.yaml")], Standard::Datacite);
    assert_eq!(
        groups,
        vec![StandardPaths {
            standard: Standard::Datacite,
            paths: vec![PathBuf::from("one.json"), PathBuf::from("two.yaml")],
        }]
    );
}
#[test]
fn test_classify_paths_preserves_mixed_path_order() {
    let groups = classify_paths(
        &[
            PathBuf::from("CITATION.cff"),
            PathBuf::from("activity.json"),
            PathBuf::from("notes.md"),
            PathBuf::from("activity.jsonc"),
            PathBuf::from("report.docx"),
            PathBuf::from("paper.pdf"),
            PathBuf::from("activity.yaml"),
        ],
        Standard::ResearchActivityData,
    );
    assert_eq!(
        groups,
        vec![
            StandardPaths {
                standard: Standard::CitationFileFormat,
                paths: vec![PathBuf::from("CITATION.cff")],
            },
            StandardPaths {
                standard: Standard::ResearchActivityData,
                paths: vec![
                    PathBuf::from("activity.json"),
                    PathBuf::from("activity.jsonc"),
                    PathBuf::from("activity.yaml")
                ],
            },
            StandardPaths {
                standard: Standard::Text,
                paths: vec![PathBuf::from("notes.md")],
            },
            StandardPaths {
                standard: Standard::Document,
                paths: vec![PathBuf::from("report.docx"), PathBuf::from("paper.pdf")],
            },
        ]
    );
}
#[test]
fn test_classify_paths_treats_unreadable_markdown_as_text() {
    let path = PathBuf::from("missing.md");
    let groups = classify_paths(core::slice::from_ref(&path), Standard::ResearchActivityData);
    assert_eq!(
        groups,
        [StandardPaths {
            standard: Standard::Text,
            paths: vec![path]
        }]
    );
}
#[test]
fn test_document_indexes_keep_first_source_when_display_keys_collide() {
    let directory = tempdir().expect("temporary directory should be created");
    let left = directory.path().join("left").join("shared").join("index.yaml");
    let right = directory.path().join("right").join("shared").join("index.yaml");
    create_dir_all(left.parent().expect("left parent should exist")).expect("left parent should be created");
    create_dir_all(right.parent().expect("right parent should exist")).expect("right parent should be created");
    write(&left, "title: left\n").expect("left fixture should be written");
    write(&right, "title: right\n").expect("right fixture should be written");
    let key = left.file_name_with_parent();
    assert_eq!(key, right.file_name_with_parent());
    let indexes = document_indexes(&[left, right]);
    assert_eq!(indexes.len(), 1);
    assert_eq!(
        indexes.get(&key).expect("collision index should exist").document().content,
        "title: left\n"
    );
}
#[tokio::test]
async fn test_flatten_checks_preserves_future_and_batch_order() {
    let futures = [
        core::future::ready(
            vec![
                check_ok!(CheckCategory::Quality, message: "first"),
                check_ok!(CheckCategory::Quality, message: "second"),
            ]
            .into(),
        ),
        core::future::ready(CheckBatch::single(check_ok!(CheckCategory::Quality, message: "third"))),
    ];
    let checks = <Text as Analysis>::flatten_checks(futures).await;
    assert_eq!(
        checks.iter().map(|check| check.message.as_str()).collect::<Vec<_>>(),
        ["first", "second", "third"]
    );
}
#[tokio::test]
async fn test_kitfile_and_modelkit_use_schema_check_flow() {
    let directory = tempdir().unwrap();
    let cases = [
        (Standard::Kitfile, "Kitfile", "manifestVersion: 1.0.0\npackage:\n  name: empty\n"),
        (
            Standard::ModelKit,
            "modelkit.json",
            r#"{"manifestVersion":"1.0.0","model":{"path":"model.gguf","digest":"invalid"}}"#,
        ),
    ];
    for (standard, name, content) in cases {
        let path = directory.path().join(name);
        write(&path, content).unwrap();
        let options = AnalysisContext::init()
            .analysis(AnalysisOptions::init().standard(standard).build())
            .build();
        let report = analyze_paths(&[path], &options).await;
        assert!(report
            .checks()
            .iter()
            .any(|check| check.category == CheckCategory::Schema && check.is_failure()));
    }
}
#[test]
fn test_skipped_categories_include_explicit_and_implicit_values_once() {
    let options = AnalysisContext::init()
        .analysis(AnalysisOptions::init().skip(vec!["schema".to_string(), "link".to_string()]).build())
        .execution(ExecutionOptions::init().offline(true).build())
        .build();
    assert_eq!(skipped_categories(&options), [CheckCategory::Schema, CheckCategory::Link]);
}
#[test]
fn test_skipped_categories_include_link_for_each_network_policy() {
    for execution in [
        ExecutionOptions::init().offline(true).build(),
        ExecutionOptions::init().disable_website_checks(true).build(),
    ] {
        let options = AnalysisContext::init().execution(execution).build();
        assert_eq!(skipped_categories(&options), [CheckCategory::Link]);
    }
}
