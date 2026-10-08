#![allow(clippy::indexing_slicing)]
use super::*;
use crate::analyzer::fair::FairIndicatorId;
use crate::analyzer::host::context::{AnalysisOptions, ExecutionExtension, ExecutionOptions, ReportExtension};
use acorn_analysis::check_ok;
use acorn_core::options::CommonRuntime;
use acorn_schema::hardware::gpu;
use std::fs::{create_dir_all, write};
use tempfile::tempdir;

#[tokio::test]
async fn test_analysis_dispatches_compute_checks_with_supplied_gpu_inventory() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("activity.json");
    let mut activity: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json"
    )))
    .expect("RAD fixture should be valid JSON");
    activity["meta"]["resources"] = serde_json::json!([{"GPU": {"count": 2}}]);
    write(&path, serde_json::to_vec(&activity).expect("RAD fixture should serialize")).expect("RAD fixture should be written");
    let inventory = gpu::Inventory::new(vec![gpu::Device {
        identifier: "gpu-0".to_string(),
        index: Some(0),
        name: "Observed GPU".to_string(),
        memory_total_mib: None,
    }])
    .expect("GPU inventory should be valid");
    let options = AnalysisContext::init()
        .analysis(AnalysisOptions::init().gpu_inventory(inventory).build())
        .build();

    let result = <ResearchActivity as Analysis>::check(CheckCategory::Compute, &[path], Some(&options)).await;

    assert_eq!(result.status, ExecutionStatus::Executed);
    assert_eq!(result.checks.len(), 1);
    assert_eq!(result.checks.as_slice()[0].locator.as_deref(), Some("resources[0].count"));
    assert!(result.checks.as_slice()[0].is_failure());
}
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
        .report(
            ReportOptions::init()
                .extension(ReportExtension::init().exit_on_first_error(true).build())
                .build(),
        )
        .build();
    let report = analyze_paths(&[path], &options).await;
    let categories = &report.batches[0].categories;
    assert_eq!(
        categories.iter().map(|result| result.category.clone()).collect::<Vec<_>>(),
        [CheckCategory::Schema, CheckCategory::Quality, CheckCategory::Compute, CheckCategory::Fair]
    );
    assert!(categories
        .iter()
        .filter(|result| !matches!(result.category, CheckCategory::Compute | CheckCategory::Fair))
        .all(|result| result.checks.iter().any(Check::is_failure)));
    assert!(categories
        .iter()
        .find(|result| result.category == CheckCategory::Fair)
        .is_some_and(|result| result.checks.iter().all(|check| !check.is_failure())));
}
#[tokio::test]
async fn test_analyze_paths_dispatches_every_standard() {
    for standard in [
        Standard::ApplicationConfigurationResearchEnablement,
        Standard::CitationFileFormat,
        Standard::Datacite,
        Standard::Dcat,
        Standard::Document,
        Standard::DublinCore,
        Standard::Huwise,
        Standard::Invenio,
        Standard::Kitfile,
        Standard::ModelKit,
        Standard::Oci,
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
async fn test_analyze_paths_distinguishes_empty_skipped_and_unsupported_categories() {
    let options = AnalysisContext::init()
        .analysis(AnalysisOptions::init().standard(Standard::Text).build())
        .execution(
            ExecutionOptions::init()
                .common(CommonRuntime {
                    offline: true,
                    ..Default::default()
                })
                .build(),
        )
        .build();
    let quality = <Text as Analysis>::check(CheckCategory::Quality, &[], Some(&options)).await;
    let link = <Cff as Analysis>::check(CheckCategory::Link, &[], Some(&options)).await;
    let fair = <Text as Analysis>::check(CheckCategory::Fair, &[], Some(&options)).await;
    assert_eq!(quality.status, ExecutionStatus::Executed);
    assert!(link.status.is_skipped());
    assert!(fair.status.is_unsupported());
    assert!(quality.checks.is_empty() && link.checks.is_empty() && fair.checks.is_empty());
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
#[tokio::test]
async fn test_analyze_paths_validates_acre_configuration() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join(".acorn.json");
    write(&path, r#"{"schemaVersion":"1.0.0","unknown":true}"#).expect("ACRE fixture should be written");
    let options = AnalysisContext::init()
        .analysis(
            AnalysisOptions::init()
                .skip(
                    CheckCategory::iter()
                        .filter(|category| *category != CheckCategory::Schema)
                        .map(|category| category.to_string())
                        .collect(),
                )
                .build(),
        )
        .build();
    let report = analyze_paths(core::slice::from_ref(&path), &options).await;
    assert_eq!(report.batches[0].standard, Standard::ApplicationConfigurationResearchEnablement);
    assert!(report.checks().iter().any(Check::is_failure));
}
#[tokio::test]
async fn test_analyze_report_with_link_evidence_adds_only_link_results() {
    let options = AnalysisContext::default();
    let report = analyze_paths(&[], &options)
        .await
        .with_link_evidence(vec![LinkEvidence::pending(None, Some("website".to_string()), None)], &options)
        .await;
    assert_eq!(report.batches.len(), 1);
    assert_eq!(report.batches[0].standard, Standard::ResearchActivityData);
    assert!(report.batches[0].paths.is_empty());
    assert_eq!(
        report.batches[0]
            .categories
            .iter()
            .map(|value| value.category.clone())
            .collect::<Vec<_>>(),
        [CheckCategory::Link]
    );
    assert_eq!(report.batches[0].categories[0].checks.as_slice()[0].message, "Missing URL");
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
fn test_classify_paths_detects_kitfile_and_modelkit() {
    let directory = tempdir().expect("temporary directory should be created");
    let kitfile = directory.path().join("Kitfile");
    let modelkit = directory.path().join("resolved.json");
    let named_modelkit = directory.path().join("modelkit.json");
    write(&kitfile, "manifestVersion: v1.0.0\npackage:\n  name: fixture\n").expect("Kitfile should be written");
    write(
        &modelkit,
        r#"{"manifestVersion":"v1.0.0","docs":[{"path":"README.md","digest":"sha256:fixture","diffId":"sha256:fixture"}]}"#,
    )
    .expect("ModelKit should be written");
    write(&named_modelkit, "{").expect("malformed named ModelKit should be written");
    let groups = classify_paths(
        &[kitfile.clone(), modelkit.clone(), named_modelkit.clone()],
        Standard::ResearchActivityData,
    );
    assert_eq!(
        groups,
        vec![
            StandardPaths {
                standard: Standard::Kitfile,
                paths: vec![kitfile],
            },
            StandardPaths {
                standard: Standard::ModelKit,
                paths: vec![modelkit, named_modelkit],
            },
        ]
    );
}
#[test]
fn test_classify_paths_detects_only_canonical_acre_filenames() {
    let paths = [
        PathBuf::from(".acorn.json"),
        PathBuf::from(".acorn.jsonc"),
        PathBuf::from(".acorn.yaml"),
        PathBuf::from(".acorn.yml"),
        PathBuf::from("custom.json"),
    ];
    let groups = classify_paths(&paths, Standard::ResearchActivityData);
    assert_eq!(
        groups,
        vec![
            StandardPaths {
                standard: Standard::ApplicationConfigurationResearchEnablement,
                paths: paths[..4].to_vec(),
            },
            StandardPaths {
                standard: Standard::ResearchActivityData,
                paths: vec![paths[4].clone()],
            },
        ]
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
fn test_correlate_link_evidence_reuses_matching_result() {
    let uri = Some("dataset.json".to_string());
    let target = "https://example.org/data";
    let finding = FairEvidence::from_value("resource", "DataCite 4.7", &serde_json::json!({"downloadURL": target}))
        .assess()
        .into_iter()
        .find(|finding| finding.indicator.id == FairIndicatorId::RdaA1_05D)
        .expect("registry contains RDA-A1-05D");
    let fair = Check::from(finding).with_uri(uri.clone());
    let link = check_ok!(CheckCategory::Link, uri: "dataset.json").with_context(target.to_string());
    let categories = correlate_link_evidence(vec![
        CategoryChecks {
            category: CheckCategory::Link,
            checks: CheckBatch::single(link),
            status: ExecutionStatus::Executed,
        },
        CategoryChecks {
            category: CheckCategory::Fair,
            checks: CheckBatch::single(fair),
            status: ExecutionStatus::Executed,
        },
    ]);
    let finding = categories[1].checks.iter().next().and_then(Check::fair_details);
    assert!(finding.is_some_and(|finding| finding.state == FairState::Supported));
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
async fn test_kitfile_and_modelkit_quality_checks_execute() {
    let directory = tempdir().expect("temporary directory should be created");
    let kitfile = directory.path().join("Kitfile");
    write(directory.path().join("README.md"), "ready").expect("documentation should be written");
    write(&kitfile, "manifestVersion: 1.0.0\ndocs:\n  - path: README.md\n").expect("Kitfile should be written");
    let kitfile_options = AnalysisContext::init()
        .analysis(AnalysisOptions::init().standard(Standard::Kitfile).build())
        .build();
    let kitfile_report = analyze_paths(&[kitfile], &kitfile_options).await;
    assert!(kitfile_report
        .checks()
        .iter()
        .any(|check| check.category == CheckCategory::Quality && check.message.contains("manifestVersion")));
    let modelkit = directory.path().join("modelkit.json");
    write(
        &modelkit,
        r#"{"manifestVersion":"v1.0.0","docs":[{"path":"README.md","digest":"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}]}"#,
    )
    .expect("ModelKit should be written");
    let modelkit_options = AnalysisContext::init()
        .analysis(AnalysisOptions::init().standard(Standard::ModelKit).build())
        .build();
    let modelkit_report = analyze_paths(&[modelkit], &modelkit_options).await;
    assert!(modelkit_report
        .checks()
        .iter()
        .any(|check| check.category == CheckCategory::Quality && check.message.contains("diffId")));
}
#[tokio::test]
async fn test_kitfile_and_modelkit_use_schema_check_flow() {
    let directory = tempdir().unwrap();
    let cases = [
        (Standard::Kitfile, "Kitfile", "manifestVersion: v1.0.0\npackage:\n  name: empty\n"),
        (
            Standard::ModelKit,
            "modelkit.json",
            r#"{"manifestVersion":"v1.0.0","model":{"path":"model.gguf","digest":"invalid"}}"#,
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
fn test_skipped_categories_do_not_hide_link_outcomes_for_network_policy() {
    for execution in [
        ExecutionOptions::init()
            .common(CommonRuntime {
                offline: true,
                ..Default::default()
            })
            .build(),
        ExecutionOptions::init()
            .extension(ExecutionExtension::init().disable_website_checks(true).build())
            .build(),
    ] {
        let options = AnalysisContext::init().execution(execution).build();
        assert!(skipped_categories(&options).is_empty());
    }
}
#[test]
fn test_skipped_categories_include_explicit_and_implicit_values_once() {
    let options = AnalysisContext::init()
        .analysis(AnalysisOptions::init().skip(vec!["schema".to_string(), "link".to_string()]).build())
        .execution(
            ExecutionOptions::init()
                .common(CommonRuntime {
                    offline: true,
                    ..Default::default()
                })
                .build(),
        )
        .build();
    assert_eq!(skipped_categories(&options), [CheckCategory::Schema, CheckCategory::Link]);
}
