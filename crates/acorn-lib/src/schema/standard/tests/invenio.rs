#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::standard::invenio::Metadata;
use crate::test::utils::fixture_path;

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
