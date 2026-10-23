use super::super::metadata::analyze_metadata_target;
use acorn_core::Location;
use acorn_schema::standard::Standard;
use std::fs;
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

#[tokio::test]
async fn test_confirmed_file_uses_standard_aware_native_analysis() {
    let source = Location::from("CITATION.cff");
    let result = analyze_metadata_target(&fixtures(), &source, None, true).await.expect("CFF analysis");
    assert_eq!(result.attachment.expect("single standard").standard, Standard::CitationFileFormat);
    assert_eq!(result.report.batches.len(), 1);
    assert!(result.summary.contains("citation-file-format"));
    assert!(result.summary.len() <= 8192);
}
#[tokio::test]
async fn test_explicit_local_standard_matrix_keeps_the_selected_dispatch() {
    let cases = [
        (Standard::ApplicationConfigurationResearchEnablement, "data/valid_project/index.json"),
        (Standard::CitationFileFormat, "CITATION.cff"),
        (Standard::Ckan, "schema/ckan/0sbdv-gh024.json"),
        (Standard::Datacite, "schema/datacite.json"),
        (Standard::Dcat, "schema/dcat.json"),
        (Standard::Document, "modelkit/README.md"),
        (Standard::DublinCore, "schema/dcat.json"),
        (Standard::Huwise, "schema/huwise.json"),
        (Standard::Invenio, "schema/invenio.json"),
        (Standard::Kitfile, "modelkit/Kitfile"),
        (Standard::ModelKit, "modelkit/modelkit.json"),
        (Standard::Raid, "raid/response_02.json"),
        (Standard::ResearchActivityData, "data/valid_project/index.json"),
        (Standard::Text, "plain.txt"),
    ];
    for (standard, path) in cases {
        let analysis = analyze_metadata_target(&fixtures(), &Location::from(path), Some(standard), true)
            .await
            .expect("standard analysis");
        assert_eq!(analysis.report.batches.first().expect("one batch").standard, standard);
        assert_eq!(analysis.attachment.expect("single standard").standard, standard);
    }
}
#[tokio::test]
async fn test_missing_remote_and_unsupported_targets_fail_closed() {
    let root = fixtures();
    assert!(analyze_metadata_target(&root, &Location::from("missing.json"), None, true).await.is_err());
    assert!(
        analyze_metadata_target(&root, &Location::from("https://example.org/metadata.json"), None, true)
            .await
            .is_err()
    );
    assert!(analyze_metadata_target(&root, &Location::from("data/valid_project/00.png"), None, true)
        .await
        .is_err());
}
#[tokio::test]
async fn test_mixed_folder_keeps_stable_order_and_requires_standard_choice() {
    let directory = tempfile::tempdir().expect("temp root");
    fs::copy(fixtures().join("CITATION.cff"), directory.path().join("CITATION.cff")).expect("CFF fixture");
    fs::copy(fixtures().join("data/valid_project/index.json"), directory.path().join("index.json")).expect("RAD fixture");
    fs::write(directory.path().join("image.png"), b"unsupported").expect("unsupported fixture");
    let source = Location::from(".");
    let first = analyze_metadata_target(directory.path(), &source, None, true)
        .await
        .expect("mixed analysis");
    let second = analyze_metadata_target(directory.path(), &source, None, true)
        .await
        .expect("repeat analysis");
    assert!(first.attachment.is_none());
    assert_eq!(first.report.batches.len(), 2);
    assert_eq!(
        first.report.batches.iter().map(|batch| batch.standard).collect::<Vec<_>>(),
        second.report.batches.iter().map(|batch| batch.standard).collect::<Vec<_>>()
    );
    assert!(first.report.input_failures.iter().any(|failure| failure.reason == "unsupported-format"));
    let selected = analyze_metadata_target(directory.path(), &source, Some(Standard::ResearchActivityData), true)
        .await
        .expect("selected RAD files");
    assert_eq!(selected.report.batches.len(), 1);
    assert!(selected
        .report
        .input_failures
        .iter()
        .any(|failure| failure.reason == "different-standard"));
    assert_eq!(selected.attachment.expect("selected standard").standard, Standard::ResearchActivityData);
}
