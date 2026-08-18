#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::standard::invenio::{CodeMeta, Metadata, PersonOrOrgIdentifier, PersonOrOrgIdentifierScheme};
use crate::test::utils::fixture_path;
use crate::util::constants::env::DATABASE_PATH;
use validator::Validate;

fn load_fixture_text() -> String {
    std::fs::read_to_string(fixture_path("schema/invenio.json")).expect("failed to read invenio.json fixture")
}

#[test]
fn test_invenio_fixture_parsing() {
    let text = load_fixture_text();
    let catalog: Vec<Metadata> = serde_json::from_str(&text).expect("failed to parse invenio.json fixture");
    assert!(!catalog.is_empty(), "catalog should not be empty");
    assert_eq!(catalog.len(), 234, "should have 234 records");
}
#[test]
fn test_invenio_first_record() {
    let text = load_fixture_text();
    let catalog: Vec<Metadata> = serde_json::from_str(&text).expect("failed to parse invenio.json fixture");
    let metadata = &catalog[0];
    // Test basic metadata fields
    assert!(metadata.title.is_some(), "title should exist");
    let title = metadata.title.as_ref().unwrap();
    assert!(
        title.contains("Utility-Scale") || title.contains("Solar"),
        "title should mention solar or Utility-Scale"
    );
    // Test resource type
    assert!(metadata.resource_type.is_some(), "resource_type should exist");
    let res_type = metadata.resource_type.as_ref().unwrap();
    assert_eq!(res_type.id.as_deref(), Some("dataset"), "resource type should be dataset");
    // Test creators
    assert!(metadata.creators.is_some(), "creators should exist");
    let creators = metadata.creators.as_ref().unwrap();
    assert!(!creators.is_empty(), "should have at least 1 creator");
    let first_creator = &creators[0];
    assert!(
        first_creator.person_or_org.given_name.is_some() || first_creator.person_or_org.name.is_some(),
        "creator should have a name"
    );
    // Test publication date
    assert!(metadata.publication_date.is_some(), "publication_date should exist");
    // Test subjects
    assert!(metadata.subjects.is_some(), "subjects should exist");
    let subjects = metadata.subjects.as_ref().unwrap();
    assert!(!subjects.is_empty(), "should have at least 1 subject");
    // Test languages
    assert!(metadata.languages.is_some(), "languages should exist");
    let languages = metadata.languages.as_ref().unwrap();
    assert!(!languages.is_empty(), "should have at least 1 language");
    // Test contributors
    assert!(metadata.contributors.is_some(), "contributors should exist");
    let contributors = metadata.contributors.as_ref().unwrap();
    assert!(!contributors.is_empty(), "should have at least 1 contributor");
}
#[test]
fn test_invenio_metadata_types() {
    let text = load_fixture_text();
    let catalog: Vec<Metadata> = serde_json::from_str(&text).expect("failed to parse invenio.json fixture");
    // Test that we can iterate through records
    for (i, metadata) in catalog.iter().take(10).enumerate() {
        assert!(metadata.title.is_some(), "record {} should have a title", i + 1);
        assert!(metadata.resource_type.is_some(), "record {} should have a resource_type", i + 1);
        assert!(metadata.creators.is_some(), "record {} should have creators", i + 1);
    }
}
#[test]
fn test_person_or_org_identifier_schemes() {
    let valid = [
        (PersonOrOrgIdentifierScheme::Orcid, "https://orcid.org/0000-0002-2057-9115"),
        (PersonOrOrgIdentifierScheme::Isni, "https://isni.org/isni/0000000492299539"),
        (PersonOrOrgIdentifierScheme::Gnd, "118540238"),
        (PersonOrOrgIdentifierScheme::Ror, "https://ror.org/01qz5mb56"),
    ];
    valid.into_iter().for_each(|(scheme, identifier)| {
        assert!(PersonOrOrgIdentifier {
            scheme,
            identifier: identifier.to_string(),
        }
        .validate()
        .is_ok());
    });
    assert!(PersonOrOrgIdentifier {
        scheme: PersonOrOrgIdentifierScheme::Isni,
        identifier: "https://orcid.org/0000-0002-2057-9115".to_string(),
    }
    .validate()
    .is_err());
    assert!(serde_json::from_str::<PersonOrOrgIdentifier>(r#"{"scheme":"custom","identifier":"123"}"#).is_err());
}
#[test]
fn test_codemeta_programming_language_validation() {
    let path = crate::test::utils::temp_database_with_programming_languages().expect("failed to initialize temporary programming-language database");
    let path_string = path.to_string_lossy().to_string();
    temp_env::with_vars([(DATABASE_PATH, Some(path_string.as_str()))], || {
        let valid = CodeMeta {
            code_repository: None,
            programming_language: Some("Rust".to_string()),
            development_status: None,
        };
        assert!(valid.validate().is_ok());
        assert!(CodeMeta {
            programming_language: Some("Not A Language".to_string()),
            ..valid
        }
        .validate()
        .is_err());
    });
}
