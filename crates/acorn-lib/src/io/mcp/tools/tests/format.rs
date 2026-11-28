use super::*;
use crate::io::mcp::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use acorn_core::util::MimeType;
use acorn_schema::codec::DocumentCodec;
use acorn_schema::standard::cff::Cff;
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
fn test_acre_native_format_differs_from_typed_codec_for_supported_syntaxes() {
    use crate::io::config::application::ApplicationConfiguration;

    let root = tempdir().expect("temporary root");
    for (name, source, mime) in [
        (".acorn.json", "{\"models\":[\"acme/example\"]}", MimeType::Json),
        (".acorn.jsonc", "// keep comment\n{\"models\":[\"acme/example\",]}", MimeType::Jsonc),
        (".acorn.yaml", "models:\n  - acme/example\n", MimeType::Yaml),
        (".acorn.ron", "// remove comment\n{\"models\":[\"acme/example\"]}", MimeType::Ron),
    ] {
        let path = root.path().join(name);
        fs::write(&path, source).expect("fixture");
        let native = format_acre(&path, source).expect("native ACRE format");
        let codec = ApplicationConfiguration::decode(source, &mime)
            .expect("typed ACRE decode")
            .encode(&mime)
            .expect("typed ACRE encode");
        assert_ne!(native, codec, "{name} unexpectedly matches the typed codec");
        let native_model = ApplicationConfiguration::decode(&native, &mime).expect("native result parses");
        let codec_model = ApplicationConfiguration::decode(&codec, &mime).expect("codec result parses");
        assert_eq!(
            serde_json::to_value(native_model).expect("native model"),
            serde_json::to_value(codec_model).expect("codec model")
        );
        fs::write(&path, &native).expect("formatted fixture");
        assert_eq!(format_acre(&path, &native).expect("idempotent native format"), native);
        if mime == MimeType::Jsonc {
            assert!(native.contains("keep comment"));
            assert!(!codec.contains("keep comment"));
        }
        if mime == MimeType::Ron {
            assert!(!native.contains("remove comment"));
        }
    }
    let invalid = root.path().join("invalid.json");
    fs::write(&invalid, "{\"unknown\":true}").expect("invalid fixture");
    assert!(format_acre(&invalid, "{\"unknown\":true}").is_err());
}
#[tokio::test]
async fn test_acre_native_preview_rejects_oversized_input() {
    let root = tempdir().expect("temporary root");
    let source = format!("{{}}{}", " ".repeat(32 * 1024 * 1024));
    fs::write(root.path().join(".acorn.json"), &source).expect("oversized fixture");
    let input = FormatPreviewInput {
        input: InputReference::File(PathBuf::from(".acorn.json")),
        standard: FormatStandard::ApplicationConfigurationResearchEnablement,
        ..cff_input()
    };
    assert!(input.preview(root.path(), true).await.is_err());
    assert_eq!(fs::read_to_string(root.path().join(".acorn.json")).expect("unchanged source"), source);
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
async fn test_datacite_preview_matches_portable_json_and_yaml() {
    let root = tempdir().expect("temporary root");
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../../../tests/fixtures/schema/datacite.json")).expect("DataCite fixture");
    let first = fixture.as_array().and_then(|records| records.first()).expect("first DataCite record");
    let source = serde_json::to_string(first).expect("DataCite JSON");
    let record = acorn_schema::standard::datacite::Record::decode(&source, &MimeType::Json).expect("typed DataCite JSON");
    for (name, content, mime) in [
        ("datacite.json", source, MimeType::Json),
        ("datacite.yaml", record.encode(&MimeType::Yaml).expect("DataCite YAML"), MimeType::Yaml),
    ] {
        let path = root.path().join(name);
        fs::write(&path, &content).expect("fixture");
        let shared = normalize_metadata_file::<acorn_schema::standard::datacite::Record>(&path, VocabularyFuzzyMode::Strict)
            .expect("shared CLI normalization");
        let expected = acorn_schema::standard::datacite::Record::decode(&content, &mime)
            .expect("portable DataCite decode")
            .encode(&mime)
            .expect("portable DataCite encode");
        assert_eq!(shared.content, expected, "{name} differs from portable codec");
        let preview = FormatPreviewInput {
            input: InputReference::File(PathBuf::from(name)),
            standard: FormatStandard::Datacite,
            ..cff_input()
        }
        .preview(root.path(), true)
        .await
        .expect("native preview");
        assert_eq!(preview.changes.len(), 1);
        assert_eq!(preview.changes.first().map(|change| change.content.as_str()), Some(expected.as_str()));
        assert_eq!(fs::read_to_string(path).expect("unchanged source"), content);
    }
}
#[tokio::test]
async fn test_dublin_core_preview_matches_portable_rdf_formats() {
    let root = tempdir().expect("temporary root");
    let source = include_str!("../../../../../../../tests/fixtures/schema/dublin-core-format.json");
    let document = acorn_schema::standard::dublin_core::DublinCoreDocument::decode(source, &MimeType::Json).expect("typed Dublin Core JSON");
    for (name, mime) in [("dublin-core.rdf", MimeType::RdfXml), ("dublin-core.ttl", MimeType::Turtle)] {
        let path = root.path().join(name);
        let content = document.encode(&mime).expect("RDF fixture");
        fs::write(&path, &content).expect("fixture");
        let shared = normalize_metadata_file::<acorn_schema::standard::dublin_core::DublinCoreDocument>(&path, VocabularyFuzzyMode::Strict)
            .expect("shared CLI normalization");
        let expected = acorn_schema::standard::dublin_core::DublinCoreDocument::decode(&content, &mime)
            .expect("portable Dublin Core decode")
            .encode(&mime)
            .expect("portable Dublin Core encode");
        assert_eq!(shared.content, expected, "{name} differs from portable codec");
        let preview = FormatPreviewInput {
            input: InputReference::File(PathBuf::from(name)),
            standard: FormatStandard::DublinCore,
            ..cff_input()
        }
        .preview(root.path(), true)
        .await
        .expect("native preview");
        assert_eq!(preview.changes.len(), 1);
        assert_eq!(preview.changes.first().map(|change| change.content.as_str()), Some(expected.as_str()));
        assert_eq!(fs::read_to_string(path).expect("unchanged source"), content);
    }
}
#[tokio::test]
async fn test_dublin_core_preview_matches_portable_structured_formats() {
    let root = tempdir().expect("temporary root");
    let source = include_str!("../../../../../../../tests/fixtures/schema/dublin-core-format.json");
    let document = acorn_schema::standard::dublin_core::DublinCoreDocument::decode(source, &MimeType::Json).expect("typed Dublin Core JSON");
    for (name, content, mime) in [
        ("dublin-core.json", source.to_string(), MimeType::Json),
        (
            "dublin-core.jsonld",
            document.encode(&MimeType::LdJson).expect("Dublin Core JSON-LD"),
            MimeType::LdJson,
        ),
        (
            "dublin-core.yaml",
            document.encode(&MimeType::Yaml).expect("Dublin Core YAML"),
            MimeType::Yaml,
        ),
    ] {
        let path = root.path().join(name);
        fs::write(&path, &content).expect("fixture");
        let shared = normalize_metadata_file::<acorn_schema::standard::dublin_core::DublinCoreDocument>(&path, VocabularyFuzzyMode::Strict)
            .expect("shared CLI normalization");
        let expected = acorn_schema::standard::dublin_core::DublinCoreDocument::decode(&content, &mime)
            .expect("portable Dublin Core decode")
            .encode(&mime)
            .expect("portable Dublin Core encode");
        assert_eq!(shared.content, expected, "{name} differs from portable codec");
        let preview = FormatPreviewInput {
            input: InputReference::File(PathBuf::from(name)),
            standard: FormatStandard::DublinCore,
            ..cff_input()
        }
        .preview(root.path(), true)
        .await
        .expect("native preview");
        assert_eq!(preview.changes.len(), 1);
        assert_eq!(preview.changes.first().map(|change| change.content.as_str()), Some(expected.as_str()));
        assert_eq!(fs::read_to_string(path).expect("unchanged source"), content);
    }
}
#[tokio::test]
async fn test_format_directory_matches_cli_selection() {
    let root = tempdir().expect("temporary root");
    fs::write(
        root.path().join(".acorn.jsonc"),
        include_str!("../../../../../../../tests/fixtures/config/basic.jsonc"),
    )
    .expect("ACRE fixture");
    fs::write(root.path().join(".acorn.ron"), "// removed\n{\"models\":[\"acme/example\"]}").expect("RON fixture");
    fs::write(root.path().join("custom.ron"), "{}").expect("noncanonical RON fixture");
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
    assert_eq!(preview.changes.len(), 2);
    assert!(preview.changes.iter().any(|change| change.source.ends_with(".acorn.jsonc")));
    assert!(preview
        .changes
        .iter()
        .any(|change| change.source.ends_with(".acorn.ron") && !change.content.contains("removed")));
    assert!(preview.failures.is_empty());
}
#[tokio::test]
async fn test_format_enrichment_is_unavailable_offline() {
    let root = tempdir().expect("temporary root");
    let input = FormatPreviewInput { enrich: true, ..cff_input() };
    assert!(input.preview(root.path(), true).await.is_err());
}
#[tokio::test]
async fn test_format_explicit_ron_preview_and_apply() {
    let root = tempdir().expect("temporary root");
    let source = "// removed\n{\"models\":[\"acme/example\"]}";
    fs::write(root.path().join("custom.ron"), source).expect("RON fixture");
    let input = FormatPreviewInput {
        input: InputReference::File(PathBuf::from("custom.ron")),
        standard: FormatStandard::ApplicationConfigurationResearchEnablement,
        enrich: false,
        providers: Vec::new(),
        fuzzy: VocabularyFuzzyMode::Strict,
    };
    let preview = input.clone().preview(root.path(), true).await.expect("RON preview");
    assert_eq!(preview.changes.len(), 1, "RON preview changes: {:?}", preview.changes);
    let change = preview.changes.first().expect("one RON change");
    assert_eq!(change.original, source);
    assert!(!change.diff.is_empty());
    assert!(!change.content.contains("removed"));
    assert!(change.content.ends_with('\n'));
    assert_eq!(fs::read_to_string(root.path().join("custom.ron")).expect("source"), source);
    let applied = FormatApplyInput {
        input,
        destination: PathBuf::from("formatted.ron"),
        overwrite: false,
    }
    .apply(root.path(), true)
    .await
    .expect("RON apply");
    assert!(applied.applied);
    assert_eq!(fs::read_to_string(applied.destination).expect("formatted RON"), change.content);
    assert_eq!(fs::read_to_string(root.path().join("custom.ron")).expect("source"), source);
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
    let cff = include_str!("../../../../../../../tests/fixtures/CITATION.cff");
    let record = Cff::decode(cff, &MimeType::Cff).expect("CFF fixture");
    let cases = [
        ("CITATION.cff", cff.to_string(), MimeType::Cff),
        ("citation.json", record.encode(&MimeType::Json).expect("JSON fixture"), MimeType::Json),
        ("citation.yaml", cff.to_string(), MimeType::Yaml),
    ];
    for (name, source, mime) in cases {
        let path = root.path().join(name);
        fs::write(&path, &source).expect("fixture");
        let shared = normalize_metadata_file::<Cff>(&path, VocabularyFuzzyMode::Strict).expect("shared CLI normalization");
        let expected = Cff::decode(&source, &mime)
            .expect("portable CFF decode")
            .encode(&mime)
            .expect("portable CFF encode");
        assert_eq!(shared.content, expected, "{name} differs from portable codec");
        let preview = FormatPreviewInput {
            input: InputReference::File(PathBuf::from(name)),
            ..cff_input()
        }
        .preview(root.path(), true)
        .await
        .expect("native preview");
        assert_eq!(preview.changes.len(), 1);
        assert_eq!(preview.changes.first().map(|change| change.content.as_str()), Some(expected.as_str()));
        assert_eq!(fs::read_to_string(path).expect("unchanged source"), source);
    }
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
    let rad = serde_json::to_string(&acorn_schema::standard::rads::ResearchActivity::default()).expect("RAD default");
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
async fn test_invenio_preview_matches_portable_json_and_yaml() {
    let root = tempdir().expect("temporary root");
    let source = include_str!("../../../../../../../tests/fixtures/schema/invenio-format.json");
    let record = acorn_schema::standard::invenio::Record::decode(source, &MimeType::Json).expect("typed Invenio JSON");
    for (name, content, mime) in [
        ("invenio.json", source.to_string(), MimeType::Json),
        ("invenio.yaml", record.encode(&MimeType::Yaml).expect("Invenio YAML"), MimeType::Yaml),
    ] {
        let path = root.path().join(name);
        fs::write(&path, &content).expect("fixture");
        let shared =
            normalize_metadata_file::<acorn_schema::standard::invenio::Record>(&path, VocabularyFuzzyMode::Strict).expect("shared CLI normalization");
        let expected = acorn_schema::standard::invenio::Record::decode(&content, &mime)
            .expect("portable Invenio decode")
            .encode(&mime)
            .expect("portable Invenio encode");
        assert_eq!(shared.content, expected, "{name} differs from portable codec");
        let preview = FormatPreviewInput {
            input: InputReference::File(PathBuf::from(name)),
            standard: FormatStandard::Invenio,
            ..cff_input()
        }
        .preview(root.path(), true)
        .await
        .expect("native preview");
        assert_eq!(preview.changes.len(), 1);
        assert_eq!(preview.changes.first().map(|change| change.content.as_str()), Some(expected.as_str()));
        assert_eq!(fs::read_to_string(path).expect("unchanged source"), content);
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
