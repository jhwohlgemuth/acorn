use super::*;
use crate::io::mcp::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use acorn_schema::standard::crosswalk::CrosswalkStandard;
use acorn_schema::vocabulary::VocabularyFuzzyMode;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn cff_input() -> FormatPreviewInput {
    FormatPreviewInput {
        input: InputReference::File(PathBuf::from("CITATION.cff")),
        standard: FormatStandard::CitationFileFormat,
        enrich: false,
        providers: Vec::new(),
        fuzzy: VocabularyFuzzyMode::Strict,
    }
}
#[test]
fn test_crosswalk_preview_surfaces_loss_warnings_and_invalid_input_does_not_write() {
    let root = tempdir().expect("temporary root");
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../../tests/fixtures/schema/datacite.json")).expect("fixture JSON");
    let record = fixture.as_array().and_then(|records| records.first()).expect("DataCite record");
    let input = CrosswalkPreviewInput {
        input: InputReference::Text(serde_json::to_string(record).expect("record JSON")),
        source: CrosswalkStandard::Datacite,
        target: CrosswalkStandard::Dcat,
        input_format: Some(CrosswalkFormat::Json),
        output_format: Some(CrosswalkFormat::Json),
    };
    let preview = input.clone().preview(root.path()).expect("crosswalk preview");
    assert!(!preview.content.is_empty());
    assert!(!preview.warnings.is_empty());
    let invalid = CrosswalkApplyInput {
        input: CrosswalkPreviewInput {
            input: InputReference::Text("invalid".into()),
            ..input
        },
        destination: PathBuf::from("converted.json"),
        overwrite: false,
    };
    assert!(invalid.apply(root.path()).is_err());
    assert!(!root.path().join("converted.json").exists());
}
#[tokio::test]
async fn test_format_directory_matches_cli_selection() {
    let root = tempdir().expect("temporary root");
    fs::write(
        root.path().join(".acorn.jsonc"),
        include_str!("../../../../../../../tests/fixtures/config/basic.jsonc"),
    )
    .expect("ACRE fixture");
    fs::write(
        root.path().join("CITATION.cff"),
        include_str!("../../../../../../../tests/fixtures/CITATION.cff"),
    )
    .expect("other fixture");
    let preview = FormatPreviewInput {
        input: InputReference::Directory(PathBuf::from(".")),
        standard: FormatStandard::ApplicationConfigurationResearchEnablement,
        enrich: false,
        providers: Vec::new(),
        fuzzy: VocabularyFuzzyMode::Strict,
    }
    .preview(root.path(), true)
    .await
    .expect("ACRE directory preview");
    assert_eq!(preview.changes.len(), 1);
    assert!(preview.changes.first().is_some_and(|change| change.source.ends_with(".acorn.jsonc")));
    assert!(preview.failures.is_empty());
}
#[tokio::test]
async fn test_format_enrichment_is_unavailable_offline() {
    let root = tempdir().expect("temporary root");
    let input = FormatPreviewInput { enrich: true, ..cff_input() };
    assert!(input.preview(root.path(), true).await.is_err());
}
#[tokio::test]
async fn test_format_invalid_input_never_writes() {
    let root = tempdir().expect("temporary root");
    fs::write(root.path().join("invalid.cff"), "not: a valid cff\n").expect("fixture");
    let input = FormatApplyInput {
        input: FormatPreviewInput {
            input: InputReference::File(PathBuf::from("invalid.cff")),
            ..cff_input()
        },
        destination: PathBuf::from("out.cff"),
        overwrite: false,
    };
    assert!(input.apply(root.path(), true).await.is_err());
    assert!(!root.path().join("out.cff").exists());
}
#[tokio::test]
async fn test_format_preview_matches_shared_cli_normalization() {
    let root = tempdir().expect("temporary root");
    fs::write(
        root.path().join("CITATION.cff"),
        include_str!("../../../../../../../tests/fixtures/CITATION.cff"),
    )
    .expect("fixture");
    let shared = normalize_metadata_file::<acorn_schema::standard::cff::Cff>(&root.path().join("CITATION.cff"), VocabularyFuzzyMode::Strict)
        .expect("shared CLI normalization");
    let preview = cff_input().preview(root.path(), true).await.expect("native preview");
    assert_eq!(preview.changes.len(), 1);
    assert_eq!(
        preview.changes.first().map(|change| change.content.as_str()),
        Some(shared.content.as_str())
    );
    assert!(root.path().join("CITATION.cff").exists());
}
#[tokio::test]
async fn test_format_supported_standard_matrix() {
    let root = tempdir().expect("temporary root");
    let datacite: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../../tests/fixtures/schema/datacite.json")).expect("DataCite fixture");
    let first_datacite = datacite.as_array().and_then(|records| records.first()).expect("DataCite record");
    let invenio: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../../tests/fixtures/schema/invenio.json")).expect("Invenio fixture");
    let first_invenio = invenio.as_array().and_then(|records| records.first()).expect("Invenio record");
    let rad = serde_json::to_string(&acorn_schema::research_activity::ResearchActivity::default()).expect("RAD default");
    let cases = [
        (".acorn.jsonc", include_str!("../../../../../../../tests/fixtures/config/basic.jsonc").to_string(), FormatStandard::ApplicationConfigurationResearchEnablement),
        ("CITATION.cff", include_str!("../../../../../../../tests/fixtures/CITATION.cff").to_string(), FormatStandard::CitationFileFormat),
        ("datacite.json", serde_json::to_string(first_datacite).expect("DataCite JSON"), FormatStandard::Datacite),
        ("dublin.json", "{\"descriptions\":[{\"subject\":{\"kind\":\"iri\",\"value\":\"https://example.org/item\"},\"properties\":{\"http://purl.org/dc/terms/title\":[{\"kind\":\"literal\",\"value\":{\"value\":\"Example\"}}]},\"types\":[]}]}".to_string(), FormatStandard::DublinCore),
        ("invenio.json", serde_json::to_string(&serde_json::json!({"metadata": first_invenio})).expect("Invenio JSON"), FormatStandard::Invenio),
        ("rad.json", rad, FormatStandard::ResearchActivityData),
    ];
    for (name, content, standard) in cases {
        fs::write(root.path().join(name), content).expect("fixture");
        let output = FormatPreviewInput {
            input: InputReference::File(PathBuf::from(name)),
            standard,
            enrich: false,
            providers: Vec::new(),
            fuzzy: VocabularyFuzzyMode::Strict,
        }
        .preview(root.path(), true)
        .await;
        assert!(output.is_ok(), "{name}: {output:?}");
    }
}
#[tokio::test]
async fn test_registered_format_apply_requires_exact_destination_grant() {
    let root = tempdir().expect("temporary root");
    fs::write(
        root.path().join("CITATION.cff"),
        include_str!("../../../../../../../tests/fixtures/CITATION.cff"),
    )
    .expect("fixture");
    let registry = ToolRegistry::default().register_format_capabilities(root.path(), true).expect("registry");
    let selection = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    let selected = registry
        .select_capability_binding("acorn.format::apply", None, None, selection)
        .expect("selection");
    let arguments = json!({
        "input": {"input":{"kind":"file","value":"CITATION.cff"},"standard":"citation-file-format"},
        "destination":"out.cff"
    });
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                selection,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("out.cff").exists());
    let approved = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                selection,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.format::apply".into(),
                    binding_id: "native:acorn.format::apply".into(),
                    arguments,
                    destination: "out.cff".into(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(approved.outcome, ExecutionOutcome::Completed);
    assert!(root.path().join("out.cff").exists());
}
#[tokio::test]
async fn test_registered_format_preview_matches_direct_result() {
    let root = tempdir().expect("temporary root");
    fs::write(
        root.path().join("CITATION.cff"),
        include_str!("../../../../../../../tests/fixtures/CITATION.cff"),
    )
    .expect("fixture");
    let input = cff_input();
    let expected = serde_json::to_value(input.clone().preview(root.path(), true).await.expect("direct")).expect("json");
    let registry = ToolRegistry::default().register_format_capabilities(root.path(), true).expect("registry");
    let selection = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    let selected = registry
        .select_capability_binding("acorn.format::preview", None, None, selection)
        .expect("selection");
    let actual = registry
        .execute_selected(
            &selected,
            serde_json::to_value(input).expect("args"),
            ExecutionContext {
                grounded: true,
                selection,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(actual.outcome, ExecutionOutcome::Completed);
    assert_eq!(actual.content, Some(expected));
    assert!(actual.artifacts.is_empty());
}
