use crate::schema::datacite::{Catalog, DescriptionType, NameType, RelatedIdentifierType, RelationType, ResourceTypeGeneral, TitleType};
use serde_json::Value;

const FIXTURE_PATH: &str = "../tests/fixtures/schema/datacite.json";

fn load_fixture_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_PATH);
    std::fs::read_to_string(path).expect("failed to read datacite.json fixture")
}

#[test]
fn test_datacite_fixture_parsing() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    assert_eq!(catalog.len(), 1);
    let record = &catalog[0];
    assert_eq!(record.id, "10.5072/example-doi");
    assert_eq!(record.r#type, "dois");
    let attrs = &record.attributes;
    assert_eq!(attrs.doi, "10.5072/example-doi");
    assert_eq!(attrs.event.as_deref(), Some("publish"));
    assert_eq!(attrs.publication_year, Some(2024));
    assert_eq!(attrs.language.as_deref(), Some("en"));
    assert_eq!(attrs.url.as_deref(), Some("https://example.org/dataset/123"));
    assert_eq!(attrs.schema_version.as_deref(), Some("http://datacite.org/schema/kernel-4"));
}
#[test]
fn test_datacite_titles() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let titles = catalog[0].attributes.titles.as_ref().expect("titles should exist");
    assert_eq!(titles.len(), 1);
    assert_eq!(titles[0].title, "Example Dataset Title");
    assert_eq!(titles[0].title_type, None);
}
#[test]
fn test_datacite_creators() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let creators = catalog[0].attributes.creators.as_ref().expect("creators should exist");
    assert_eq!(creators.len(), 1);
    let creator = &creators[0];
    assert_eq!(creator.name, "Smith, Jane");
    assert_eq!(creator.name_type, Some(NameType::Personal));
    assert_eq!(creator.given_name.as_deref(), Some("Jane"));
    assert_eq!(creator.family_name.as_deref(), Some("Smith"));
    let identifiers = creator.name_identifiers.as_ref().expect("name identifiers should exist");
    assert_eq!(identifiers.len(), 1);
    assert_eq!(identifiers[0].name_identifier, "https://orcid.org/0000-0001-2345-6789");
    assert_eq!(identifiers[0].name_identifier_scheme.as_deref(), Some("ORCID"));
    let affiliations = creator.affiliation.as_ref().expect("affiliations should exist");
    assert_eq!(affiliations.len(), 1);
    assert_eq!(affiliations[0].name, "Example University");
}
#[test]
fn test_datacite_publisher() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let publisher = catalog[0].attributes.publisher.as_ref().expect("publisher should exist");
    assert_eq!(publisher.name, "Example Repository");
}
#[test]
fn test_datacite_resource_types() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let types = catalog[0].attributes.resource_types.as_ref().expect("resource types should exist");
    assert_eq!(types.resource_type_general, Some(ResourceTypeGeneral::Dataset));
    assert_eq!(types.resource_type.as_deref(), Some("Dataset"));
}
#[test]
fn test_datacite_descriptions() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let descriptions = catalog[0].attributes.descriptions.as_ref().expect("descriptions should exist");
    assert_eq!(descriptions.len(), 1);
    assert_eq!(descriptions[0].description_type, Some(DescriptionType::Abstract));
    assert!(descriptions[0].description.starts_with("This is an example dataset"));
}
#[test]
fn test_datacite_subjects() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let subjects = catalog[0].attributes.subjects.as_ref().expect("subjects should exist");
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].subject, "Climate change");
}
#[test]
fn test_datacite_related_identifiers() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let related = catalog[0]
        .attributes
        .related_identifiers
        .as_ref()
        .expect("related identifiers should exist");
    assert_eq!(related.len(), 2);
    assert_eq!(related[0].related_identifier, "10.1234/example-article");
    assert_eq!(related[0].related_identifier_type, Some(RelatedIdentifierType::Doi));
    assert_eq!(related[0].relation_type, Some(RelationType::IsCitedBy));
    assert_eq!(related[0].resource_type_general, Some(ResourceTypeGeneral::JournalArticle));
    assert_eq!(related[1].related_identifier, "https://example.org/dataset/123-v2");
    assert_eq!(related[1].related_identifier_type, Some(RelatedIdentifierType::Url));
    assert_eq!(related[1].relation_type, Some(RelationType::IsNewVersionOf));
}
#[test]
fn test_datacite_enum_serialization() {
    let title_type = TitleType::AlternativeTitle;
    let json = serde_json::to_string(&title_type).expect("failed to serialize TitleType");
    assert_eq!(json, "\"AlternativeTitle\"");
    let resource = ResourceTypeGeneral::ComputationalNotebook;
    let json = serde_json::to_string(&resource).expect("failed to serialize ResourceTypeGeneral");
    assert_eq!(json, "\"ComputationalNotebook\"");
    let relation = RelationType::IsDerivedFrom;
    let json = serde_json::to_string(&relation).expect("failed to serialize RelationType");
    assert_eq!(json, "\"IsDerivedFrom\"");
    let id_type = RelatedIdentifierType::Arxiv;
    let json = serde_json::to_string(&id_type).expect("failed to serialize RelatedIdentifierType");
    assert_eq!(json, "\"arXiv\"");
}
#[test]
fn test_datacite_fixture_snapshot() {
    let text = load_fixture_text();
    let json: Value = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    insta::assert_snapshot!(serde_json::to_string_pretty(&json).expect("failed to format json"));
}
