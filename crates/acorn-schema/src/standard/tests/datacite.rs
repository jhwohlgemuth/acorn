#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::standard::datacite::{
    Affiliation, Attributes, Catalog, Date, DateType, DescriptionType, FunderIdentifier, FunderIdentifierType, FundingReference, GeoLocationPoint,
    NameIdentifier, NameType, Publisher, RelatedIdentifier, RelatedIdentifierType, RelatedItem, RelatedItemIdentifier, RelationType, Resource,
    ResourceTypeGeneral, TitleType,
};
use crate::validation::Validate;
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
fn test_datacite_date_requires_complete_timestamp() {
    let date = |value: &str| Date {
        date: value.to_string(),
        date_type: DateType::Issued,
        date_information: None,
    };
    assert!(date("2026-03-12T15:04:05Z").validate().is_ok());
    assert!(date("2026-03-12").validate().is_err());
    assert!(date("2026-03-12T15:04:05Z/2026-03-13T15:04:05Z").validate().is_err());
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
#[test]
fn test_datacite_isni_identifier_validation() {
    let isni = "https://isni.org/isni/0000000492299539";
    let name = NameIdentifier {
        name_identifier: isni.to_string(),
        name_identifier_scheme: Some("ISNI".to_string()),
        scheme_uri: Some("https://isni.org/".to_string()),
    };
    let blank_name = NameIdentifier {
        name_identifier_scheme: None,
        ..name.clone()
    };
    let identifier = NameIdentifier {
        name_identifier: "0000000492299538".to_string(),
        ..name.clone()
    };
    let mismatched_scheme = NameIdentifier {
        name_identifier_scheme: Some("ORCID".to_string()),
        ..name.clone()
    };
    let mismatched_uri = NameIdentifier {
        scheme_uri: Some("https://orcid.org/".to_string()),
        ..name.clone()
    };
    let affiliation = Affiliation {
        name: "Example".to_string(),
        affiliation_identifier: Some(isni.to_string()),
        affiliation_identifier_scheme: Some("ISNI".to_string()),
        scheme_uri: Some("https://isni.org/".to_string()),
    };
    let publisher = Publisher {
        name: "Example".to_string(),
        publisher_identifier: Some(isni.to_string()),
        publisher_identifier_scheme: Some("ISNI".to_string()),
        scheme_uri: Some("https://isni.org/".to_string()),
    };
    let funder_id = FunderIdentifier {
        funder_identifier_type: Some(FunderIdentifierType::Isni),
        value: isni.to_string(),
    };
    let funding_ref = FundingReference {
        funder_name: "Example".to_string(),
        funder_identifier: Some("0000000492299538".to_string()),
        funder_identifier_type: Some(FunderIdentifierType::Isni),
        scheme_uri: Some("https://isni.org/".to_string()),
        award_number: None,
        award_uri: None,
        award_title: None,
    };
    assert!(name.validate().is_ok());
    assert!(blank_name.validate().is_err());
    assert!(identifier.validate().is_err());
    assert!(mismatched_scheme.validate().is_err());
    assert!(mismatched_uri.validate().is_err());
    assert!(affiliation.validate().is_ok());
    assert!(publisher.validate().is_ok());
    assert!(funder_id.validate().is_ok());
    assert!(funding_ref.validate().is_err());
}
#[test]
fn test_datacite_record_propagates_identifier_validation() {
    let text = load_fixture_text();
    let mut catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let record = &mut catalog[0];
    {
        let name_identifier = &mut record.attributes.creators.as_mut().expect("creators should exist")[0]
            .name_identifiers
            .as_mut()
            .expect("name identifiers should exist")[0];
        name_identifier.name_identifier = "https://isni.org/isni/0000000492299539".to_string();
        name_identifier.name_identifier_scheme = Some("ISNI".to_string());
        name_identifier.scheme_uri = Some("https://isni.org/".to_string());
    }
    assert!(record.validate().is_ok());
    record.attributes.creators.as_mut().expect("creators should exist")[0]
        .name_identifiers
        .as_mut()
        .expect("name identifiers should exist")[0]
        .name_identifier = "0000000492299538".to_string();
    assert!(record.validate().is_err());

    let mut catalog: Catalog = serde_json::from_str(&text).expect("failed to parse datacite.json fixture");
    let record = &mut catalog[0];
    record.attributes.publisher = Some(Publisher {
        name: "Example".to_string(),
        publisher_identifier: Some("0000000492299538".to_string()),
        publisher_identifier_scheme: Some("ISNI".to_string()),
        scheme_uri: Some("https://isni.org/".to_string()),
    });
    assert!(record.validate().is_err());
}
#[test]
fn test_datacite_47_resource_types() {
    let poster = ResourceTypeGeneral::Poster;
    assert_eq!(serde_json::to_string(&poster).expect("serialize poster"), "\"Poster\"");
    let presentation = ResourceTypeGeneral::Presentation;
    assert_eq!(serde_json::to_string(&presentation).expect("serialize presentation"), "\"Presentation\"");
    let parsed: ResourceTypeGeneral = serde_json::from_str("\"Poster\"").expect("parse poster");
    assert_eq!(parsed, ResourceTypeGeneral::Poster);
}

#[test]
fn test_datacite_47_related_identifier_types() {
    let raid = RelatedIdentifierType::Raid;
    assert_eq!(serde_json::to_string(&raid).expect("serialize raid"), "\"RAiD\"");
    let rrid = RelatedIdentifierType::Rrid;
    assert_eq!(serde_json::to_string(&rrid).expect("serialize rrid"), "\"RRID\"");
    let swhid = RelatedIdentifierType::Swhid;
    assert_eq!(serde_json::to_string(&swhid).expect("serialize swhid"), "\"SWHID\"");
    let parsed: RelatedIdentifierType = serde_json::from_str("\"RAiD\"").expect("parse raid");
    assert_eq!(parsed, RelatedIdentifierType::Raid);
    let parsed: RelatedIdentifierType = serde_json::from_str("\"RRID\"").expect("parse rrid");
    assert_eq!(parsed, RelatedIdentifierType::Rrid);
    let parsed: RelatedIdentifierType = serde_json::from_str("\"SWHID\"").expect("parse swhid");
    assert_eq!(parsed, RelatedIdentifierType::Swhid);
}

#[test]
fn test_datacite_47_relation_type_other() {
    let other = RelationType::Other;
    assert_eq!(serde_json::to_string(&other).expect("serialize other"), "\"Other\"");
    let parsed: RelationType = serde_json::from_str("\"Other\"").expect("parse other");
    assert_eq!(parsed, RelationType::Other);
}

#[test]
fn test_datacite_47_relation_type_information() {
    let ri = RelatedIdentifier {
        related_identifier: "10.1234/custom".to_string(),
        related_identifier_type: Some(RelatedIdentifierType::Doi),
        relation_type: Some(RelationType::Other),
        related_metadata_scheme: None,
        scheme_uri: None,
        scheme_type: None,
        resource_type_general: Some(ResourceTypeGeneral::Dataset),
        relation_type_information: Some("Custom domain-specific relation".to_string()),
    };
    assert!(ri.validate().is_ok());
    let json = serde_json::to_string(&ri).expect("serialize related identifier");
    assert!(json.contains("relationTypeInformation"));
    let parsed: RelatedIdentifier = serde_json::from_str(&json).expect("parse related identifier");
    assert_eq!(parsed.relation_type_information.as_deref(), Some("Custom domain-specific relation"));
}

#[test]
fn test_datacite_47_related_item() {
    let item = RelatedItem {
        related_item_type: ResourceTypeGeneral::JournalArticle,
        relation_type: RelationType::IsPublishedIn,
        relation_type_information: None,
        related_item_identifier: Some(RelatedItemIdentifier {
            related_item_identifier: "10.1234/journal-article".to_string(),
            related_item_identifier_type: Some(RelatedIdentifierType::Doi),
            related_metadata_scheme: None,
            scheme_uri: None,
            scheme_type: None,
        }),
        creators: None,
        titles: None,
        publication_year: Some("2024".to_string()),
        volume: Some("10".to_string()),
        issue: Some("2".to_string()),
        number: None,
        first_page: Some("100".to_string()),
        last_page: Some("110".to_string()),
        publisher: Some("Example Publisher".to_string()),
        edition: None,
        contributors: None,
    };
    assert!(item.validate().is_ok());
    let json = serde_json::to_string(&item).expect("serialize related item");
    assert!(json.contains("relatedItemType"));
    let other = RelatedItem {
        related_item_type: ResourceTypeGeneral::Dataset,
        relation_type: RelationType::Other,
        relation_type_information: Some("Derived from instrument calibration".to_string()),
        related_item_identifier: None,
        creators: None,
        titles: None,
        publication_year: None,
        volume: None,
        issue: None,
        number: None,
        first_page: None,
        last_page: None,
        publisher: None,
        edition: None,
        contributors: None,
    };
    let json = serde_json::to_string(&other).expect("serialize related item other");
    assert!(json.contains("Other"));
    assert!(json.contains("relationTypeInformation"));
    let invalid_year = RelatedItem {
        publication_year: Some("24".to_string()),
        ..other
    };
    assert!(invalid_year.validate().is_err());
}

#[test]
fn test_datacite_47_json_fixture_with_deltas() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/schema/datacite/4.7/valid-4.7-deltas.json");
    let text = std::fs::read_to_string(path).expect("failed to read 4.7 deltas fixture");
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse 4.7 deltas fixture");
    assert_eq!(catalog.len(), 2);
    let first = &catalog[0];
    assert_eq!(
        first.attributes.resource_types.as_ref().expect("types").resource_type_general,
        Some(ResourceTypeGeneral::Poster)
    );
    let related = first.attributes.related_identifiers.as_ref().expect("related identifiers");
    assert!(related.iter().any(|r| r.related_identifier_type == Some(RelatedIdentifierType::Raid)));
    assert!(related.iter().any(|r| {
        r.related_identifier == "RRID:SCR_003512"
            && r.related_identifier_type == Some(RelatedIdentifierType::Rrid)
            && r.relation_type == Some(RelationType::Cites)
            && r.resource_type_general == Some(ResourceTypeGeneral::Software)
    }));
    assert!(related.iter().any(|r| r.related_identifier_type == Some(RelatedIdentifierType::Swhid)));
    assert!(related
        .iter()
        .any(|r| r.relation_type == Some(RelationType::Other) && r.relation_type_information.is_some()));
    let items = first.attributes.related_items.as_ref().expect("related items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].related_item_type, ResourceTypeGeneral::JournalArticle);
    let second = &catalog[1];
    assert_eq!(
        second.attributes.resource_types.as_ref().expect("types").resource_type_general,
        Some(ResourceTypeGeneral::Presentation)
    );
    for record in catalog {
        assert!(record.validate().is_ok(), "4.7 delta record should validate");
    }
}

#[test]
fn test_datacite_47_xml_fixture_with_related_item() {
    // Construct a Resource with 4.7 deltas and verify round-trip through XML serde.
    let resource = Resource {
        identifier: crate::standard::datacite::Identifier {
            identifier_type: "DOI".to_string(),
            value: "10.5072/datacite-47-xml".to_string(),
        },
        creators: crate::standard::datacite::Creators {
            creator: vec![crate::standard::datacite::Creator {
                name: "Doe, Jane".to_string(),
                name_type: Some(NameType::Personal),
                given_name: Some("Jane".to_string()),
                family_name: Some("Doe".to_string()),
                name_identifiers: None,
                affiliation: None,
            }],
        },
        titles: crate::standard::datacite::Titles {
            title: vec![crate::standard::datacite::Title {
                title: "XML Resource with RelatedItem".to_string(),
                title_type: None,
            }],
        },
        publisher: "Example Repository".to_string(),
        publication_year: 2024,
        resource_type: crate::standard::datacite::ResourceTypes {
            resource_type_general: Some(ResourceTypeGeneral::Poster),
            resource_type: Some("Conference poster".to_string()),
        },
        subjects: None,
        dates: None,
        language: None,
        related_identifiers: Some(crate::standard::datacite::RelatedIdentifiers {
            related_identifier: vec![
                RelatedIdentifier {
                    related_identifier: "https://raid.org/10.26259/5c43ca8f".to_string(),
                    related_identifier_type: Some(RelatedIdentifierType::Raid),
                    relation_type: Some(RelationType::IsPartOf),
                    related_metadata_scheme: None,
                    scheme_uri: None,
                    scheme_type: None,
                    resource_type_general: Some(ResourceTypeGeneral::Project),
                    relation_type_information: None,
                },
                RelatedIdentifier {
                    related_identifier: "swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2".to_string(),
                    related_identifier_type: Some(RelatedIdentifierType::Swhid),
                    relation_type: Some(RelationType::Cites),
                    related_metadata_scheme: None,
                    scheme_uri: None,
                    scheme_type: None,
                    resource_type_general: Some(ResourceTypeGeneral::Software),
                    relation_type_information: None,
                },
            ],
        }),
        related_items: Some(crate::standard::datacite::RelatedItems {
            related_item: vec![
                RelatedItem {
                    related_item_type: ResourceTypeGeneral::JournalArticle,
                    relation_type: RelationType::IsPublishedIn,
                    relation_type_information: None,
                    related_item_identifier: None,
                    creators: None,
                    titles: Some(vec![crate::standard::datacite::RelatedItemTitle {
                        title: "Journal of Example Studies".to_string(),
                        title_type: None,
                    }]),
                    publication_year: Some("2024".to_string()),
                    volume: Some("10".to_string()),
                    issue: Some("2".to_string()),
                    number: None,
                    first_page: Some("100".to_string()),
                    last_page: Some("110".to_string()),
                    publisher: Some("Example Publisher".to_string()),
                    edition: None,
                    contributors: None,
                },
                RelatedItem {
                    related_item_type: ResourceTypeGeneral::Dataset,
                    relation_type: RelationType::Other,
                    relation_type_information: Some("Derived from instrument calibration".to_string()),
                    related_item_identifier: Some(RelatedItemIdentifier {
                        related_item_identifier: "10.5072/related-dataset".to_string(),
                        related_item_identifier_type: Some(RelatedIdentifierType::Doi),
                        related_metadata_scheme: None,
                        scheme_uri: None,
                        scheme_type: None,
                    }),
                    creators: None,
                    titles: None,
                    publication_year: Some("2023".to_string()),
                    volume: None,
                    issue: None,
                    number: None,
                    first_page: None,
                    last_page: None,
                    publisher: None,
                    edition: None,
                    contributors: None,
                },
            ],
        }),
        sizes: None,
        formats: None,
        version: None,
        rights_list: None,
        descriptions: None,
        geo_locations: None,
        funding_references: None,
    };
    assert!(resource.validate().is_ok());
    let xml = quick_xml::se::to_string(&resource).expect("serialize 4.7 xml");
    assert!(xml.contains("Poster"));
    assert!(xml.contains("RAiD") || xml.contains("Raid"));
    let parsed: Resource = quick_xml::de::from_str(&xml).expect("parse 4.7 xml round-trip");
    assert_eq!(parsed.resource_type.resource_type_general, Some(ResourceTypeGeneral::Poster));
    let items = parsed.related_items.as_ref().expect("related items");
    assert_eq!(items.related_item.len(), 2);
    assert_eq!(items.related_item[0].related_item_type, ResourceTypeGeneral::JournalArticle);
    // Also verify fixture file is parsable as JSON (validates 4.7 deltas fixture shape)
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/schema/datacite/4.7/valid-4.7-deltas.json");
    let text = std::fs::read_to_string(path).expect("failed to read 4.7 deltas fixture");
    let catalog: Catalog = serde_json::from_str(&text).expect("parse deltas json");
    assert_eq!(catalog.len(), 2);
}
#[test]
fn test_datacite_deposit_ready_validator() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/schema/datacite/4.7/valid-4.7-deltas.json");
    let text = std::fs::read_to_string(path).expect("read deltas");
    let catalog: Catalog = serde_json::from_str(&text).expect("parse deltas");
    for record in &catalog {
        assert!(record.validate().is_ok());
        assert!(record.validate_deposit_ready().is_ok(), "4.7 deltas should be deposit-ready");
    }
    let invalid_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/schema/datacite/4.7/invalid-missing-mandatory.json");
    let text = std::fs::read_to_string(invalid_path).expect("read invalid");
    let catalog: Catalog = serde_json::from_str(&text).expect("parse invalid");
    let record = &catalog[0];
    assert!(record.validate().is_ok(), "invalid fixture is still syntactically valid");
    let err = record.validate_deposit_ready().expect_err("should be not deposit-ready");
    assert_eq!(err, vec!["titles".to_string()]);
    let recommended = record.validate_recommended_properties().expect_err("record omits recommended properties");
    assert_eq!(
        recommended,
        ["subjects", "contributors", "dates", "relatedIdentifiers", "descriptions", "geoLocations"]
            .map(str::to_string)
            .to_vec()
    );
    let optional = record.validate_optional_properties().expect_err("record omits optional properties");
    assert_eq!(
        optional,
        [
            "language",
            "alternateIdentifiers",
            "sizes",
            "formats",
            "version",
            "rightsList",
            "fundingReferences",
            "relatedItems"
        ]
        .map(str::to_string)
        .to_vec()
    );
    let xml_resource = Resource {
        identifier: crate::standard::datacite::Identifier {
            identifier_type: "DOI".to_string(),
            value: "10.5072/test".to_string(),
        },
        creators: crate::standard::datacite::Creators { creator: vec![] },
        titles: crate::standard::datacite::Titles { title: vec![] },
        publisher: "Test Publisher".to_string(),
        publication_year: 2024,
        resource_type: crate::standard::datacite::ResourceTypes {
            resource_type_general: None,
            resource_type: Some("Dataset".to_string()),
        },
        subjects: None,
        dates: None,
        language: None,
        related_identifiers: None,
        related_items: None,
        sizes: None,
        formats: None,
        version: None,
        rights_list: None,
        descriptions: None,
        geo_locations: None,
        funding_references: None,
    };
    let err = xml_resource
        .validate_deposit_ready()
        .expect_err("xml missing creators/titles/type should fail");
    assert_eq!(err, ["creators", "titles", "resourceType"].map(str::to_string).to_vec());
}
