#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::standard::datacite::{
    Catalog, DateType, DescriptionType, FunderIdentifierType, GeoLocationPoint, NameType, RelatedIdentifierType, RelationType, Resource,
    ResourceTypeGeneral, TitleType,
};
use serde_json::Value;

const FIXTURE_PATH: &str = "../../tests/fixtures/schema/datacite.json";
const XML_FIXTURE_PATH: &str = "../../tests/fixtures/schema/datacite.xml";

fn load_fixture_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_PATH);
    std::fs::read_to_string(path).expect("failed to read datacite.json fixture")
}
fn load_xml_fixture_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(XML_FIXTURE_PATH);
    std::fs::read_to_string(path).expect("failed to read datacite.xml fixture")
}

#[test]
fn test_datacite_fixture_parsing() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    assert_eq!(catalog.len(), 1);
    let record = &catalog[0];
    assert_eq!(record.id, "10.5072/example-doi");
    assert_eq!(record.kind, "dois");
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
#[test]
fn test_datacite_xml_fixture_parsing() {
    let text = load_xml_fixture_text();
    let resource: Resource = quick_xml::de::from_str(&text).expect("failed to parse datacite.xml fixture");
    // Identifier
    assert_eq!(resource.identifier.identifier_type, "DOI");
    assert_eq!(resource.identifier.value.trim(), "10.1594/PANGAEA.771774");
    // Creators
    assert_eq!(resource.creators.creator.len(), 4);
    assert_eq!(resource.creators.creator[0].name, "Hillenbrand, Claus-Dieter");
    assert_eq!(resource.creators.creator[0].given_name.as_deref(), Some("Claus-Dieter"));
    assert_eq!(resource.creators.creator[0].family_name.as_deref(), Some("Hillenbrand"));
    let name_ids = resource.creators.creator[0]
        .name_identifiers
        .as_ref()
        .expect("should have name identifiers");
    assert_eq!(name_ids.len(), 1);
    assert_eq!(name_ids[0].name_identifier_scheme.as_deref(), Some("ORCID"));
    assert_eq!(name_ids[0].scheme_uri.as_deref(), Some("https://orcid.org"));
    assert!(name_ids[0].name_identifier.trim().starts_with("https://orcid.org/"));
    // Titles
    assert_eq!(resource.titles.title.len(), 1);
    assert_eq!(
        resource.titles.title[0].title,
        "Last glacial maximum sediment record of the southern Weddell Sea shelf"
    );
    // Publisher & year
    assert_eq!(resource.publisher, "PANGAEA");
    assert_eq!(resource.publication_year, 2011);
    // Resource type
    assert_eq!(resource.resource_type.resource_type_general, Some(ResourceTypeGeneral::Collection));
    assert_eq!(
        resource.resource_type.resource_type.as_deref(),
        Some("Supplementary Publication Series of Datasets")
    );
    // Subjects
    let subjects = resource.subjects.as_ref().expect("subjects should exist");
    assert_eq!(subjects.subject.len(), 21);
    assert_eq!(subjects.subject[0].subject, "Gravity corer");
    assert_eq!(subjects.subject[0].subject_scheme.as_deref(), Some("Method"));
    // Dates
    let dates = resource.dates.as_ref().expect("dates should exist");
    assert_eq!(dates.date.len(), 1);
    assert_eq!(dates.date[0].date_type, DateType::Issued);
    assert_eq!(dates.date[0].date, "2011");
    // Language
    assert_eq!(resource.language.as_deref(), Some("en"));
    // Related identifiers
    let related = resource.related_identifiers.as_ref().expect("related identifiers should exist");
    assert_eq!(related.related_identifier.len(), 1);
    assert_eq!(related.related_identifier[0].related_identifier, "10.1016/j.quascirev.2011.11.017");
    assert_eq!(related.related_identifier[0].related_identifier_type, Some(RelatedIdentifierType::Doi));
    assert_eq!(related.related_identifier[0].relation_type, Some(RelationType::IsSupplementTo));
    // Sizes & formats
    let sizes = resource.sizes.as_ref().expect("sizes should exist");
    assert_eq!(sizes.size, vec!["2 datasets"]);
    let formats = resource.formats.as_ref().expect("formats should exist");
    assert_eq!(formats.format, vec!["application/zip"]);
    // Version
    assert_eq!(resource.version.as_deref(), Some(""));
    // Rights
    let rights_list = resource.rights_list.as_ref().expect("rights list should exist");
    assert_eq!(rights_list.rights.len(), 1);
    // assert_eq!(rights_list.rights[0].rights_identifier.as_deref(), Some("cc-by-3.0"));
    assert_eq!(rights_list.rights[0].rights_identifier_scheme.as_deref(), Some("SPDX"));
    assert_eq!(
        rights_list.rights[0].rights_uri.as_deref(),
        Some("https://creativecommons.org/licenses/by/3.0/legalcode")
    );
    assert_eq!(rights_list.rights[0].scheme_uri.as_deref(), Some("https://spdx.org/licenses/"));
    assert_eq!(rights_list.rights[0].rights.as_deref(), Some("Creative Commons Attribution 3.0 Unported"));
    // Descriptions
    let descriptions = resource.descriptions.as_ref().expect("descriptions should exist");
    assert_eq!(descriptions.description.len(), 3);
    assert_eq!(descriptions.description[0].description_type, Some(DescriptionType::Abstract));
    assert_eq!(descriptions.description[1].description_type, Some(DescriptionType::Methods));
    assert_eq!(descriptions.description[2].description_type, Some(DescriptionType::Other));
    assert!(descriptions.description[0].description.contains("grounded ice-sheet extent"));
    // GeoLocations
    let geo_locations = resource.geo_locations.as_ref().expect("geo locations should exist");
    assert_eq!(geo_locations.geo_location.len(), 9);
    let geo_box = geo_locations.geo_location[0]
        .geo_location_box
        .as_ref()
        .expect("first geolocation should have a box");
    assert_eq!(geo_box.west_bound_longitude, -61.321666666667);
    assert_eq!(geo_box.east_bound_longitude, -25.716666666667);
    assert_eq!(geo_box.south_bound_latitude, -78.088166666667);
    assert_eq!(geo_box.north_bound_latitude, -73.486166666667);
    assert_eq!(geo_locations.geo_location[1].geo_location_place.as_deref(), Some("Weddell Sea"));
    // Point location (Disko Bay)
    let point = geo_locations.geo_location[6]
        .geo_location_point
        .as_ref()
        .expect("Disko Bay should have a point");
    assert_eq!(
        point,
        &GeoLocationPoint {
            longitude: -52.0,
            latitude: 69.0,
        }
    );
    // Polygon location (Triangle Park)
    let polygon = geo_locations.geo_location[8]
        .geo_location_polygon
        .as_ref()
        .expect("Triangle Park should have a polygon");
    assert_eq!(polygon.polygon_points.len(), 5);
    let in_point = polygon.in_polygon_point.as_ref().expect("polygon should have in-polygon point");
    assert_eq!(
        in_point,
        &GeoLocationPoint {
            longitude: -123.108041,
            latitude: 49.272001,
        }
    );
    // Funding references
    let funding_refs = resource.funding_references.as_ref().expect("funding references should exist");
    assert_eq!(funding_refs.funding_reference.len(), 2);
    assert_eq!(funding_refs.funding_reference[0].funder_name, "European Commission");
    let funder_id = funding_refs.funding_reference[0]
        .funder_identifier
        .as_ref()
        .expect("should have funder identifier");
    assert_eq!(funder_id.funder_identifier_type, Some(FunderIdentifierType::CrossrefFunderId));
    assert_eq!(funder_id.value, "https://doi.org/10.13039/501100000780");
    let award = funding_refs.funding_reference[0].award_number.as_ref().expect("should have award number");
    assert_eq!(award.value, "282625");
    assert_eq!(award.award_uri.as_deref(), Some("https://cordis.europa.eu/project/rcn/100180_en.html"));
    assert!(funding_refs.funding_reference[0]
        .award_title
        .as_ref()
        .expect("should have award title")
        .contains("BIOdiversity"));
}
