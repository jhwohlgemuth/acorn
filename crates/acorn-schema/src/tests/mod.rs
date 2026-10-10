#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::geonames::{CodeFormat, Country, GeonamesParser};
use crate::research_activity::aspect::Maturity;
use crate::research_activity::*;
use crate::standard::dcat::{Agent, Catalog, ConformsTo, ContactPoint, DataService, Dataset, Distribution, DocumentRef, Publisher};
use crate::util::Constant;
use crate::validation::Validate;
use crate::*;
use acorn_core::util::Searchable;
use glob::glob;
use pretty_assertions::assert_eq;

mod infrastructure;
mod types;

#[test]
fn test_classification() {
    assert_eq!(ClassificationLevel::Secret.to_string(), "SECRET");
    assert_eq!(ClassificationLevel::TopSecret.to_string(), "TOP SECRET");
    assert!(ClassificationLevel::Unclassified < ClassificationLevel::Confidential);
    assert!(ClassificationLevel::Confidential < ClassificationLevel::Secret);
    assert!(ClassificationLevel::Secret < ClassificationLevel::TopSecret);
}
#[test]
fn test_contact_formatting_preserves_unresolved_supplied_organizations() {
    let contact = crate::ContactPoint::init()
        .organization("Example Research Unit".to_string())
        .affiliation("Example Directorate".to_string())
        .build();
    let formatted = contact
        .format_with_vocabulary_report(vocabulary::VocabularyFuzzyMode::Strict)
        .expect("organization vocabulary");
    assert_eq!(formatted.value.organization, "Example Research Unit");
    assert_eq!(formatted.value.affiliation.as_deref(), Some("Example Directorate"));
    assert_eq!(
        formatted.reports[0].resolutions[0].source_locator.as_deref(),
        Some("contact.organization")
    );
    assert_eq!(formatted.reports[0].resolutions[1].source_locator.as_deref(), Some("contact.affiliation"));
}
#[test]
fn test_controlled_vocabulary_preserves_unknown_values() {
    let resolver = vocabulary::VocabularyResolver::embedded(vocabulary::VocabularyKind::Keywords).unwrap();
    let report = resolver
        .resolve(["AI", "unknown-value", "hpc", "AI"], vocabulary::VocabularyOptions::default())
        .unwrap();
    assert_eq!(
        report.lossless_values(),
        ["artificial-intelligence", "unknown-value", "high-performance-computing"]
    );
}
#[test]
fn test_dcat_url_field_validation() {
    let agent_valid = Agent {
        name: None,
        homepage: Some(OneOrMany::Many(vec!["https://example.org/agent".to_string()])),
        email: None,
        identifier: None,
    };
    assert!(agent_valid.validate().is_ok());
    let agent_invalid = Agent {
        homepage: Some(OneOrMany::Many(vec!["not a url".to_string()])),
        ..agent_valid
    };
    assert!(agent_invalid.validate().is_err());
    let distribution_valid = Distribution {
        id: None,
        jsonld_type: None,
        title: None,
        description: None,
        issued: None,
        modified: None,
        license: None,
        access_rights: None,
        rights: None,
        has_policy: None,
        access_restriction: None,
        use_restriction: None,
        access_url: OneOrMany::Many(vec!["https://example.org/access".to_string()]),
        access_service: None,
        download_url: Some(OneOrMany::Many(vec!["https://example.org/file.csv".to_string()])),
        byte_size: None,
        spatial_resolution_in_meters: None,
        temporal_resolution: None,
        conforms_to: None,
        media_type: None,
        format: None,
        compress_format: None,
        package_format: None,
        checksum: None,
    };
    assert!(distribution_valid.validate().is_ok());
    let distribution_invalid_access = Distribution {
        access_url: OneOrMany::Many(vec!["bad url".to_string()]),
        ..distribution_valid.clone()
    };
    assert!(distribution_invalid_access.validate().is_err());
    let distribution_invalid_download = Distribution {
        download_url: Some(OneOrMany::Many(vec!["still not a url".to_string()])),
        ..distribution_valid
    };
    assert!(distribution_invalid_download.validate().is_err());
    let data_service_valid = DataService {
        id: None,
        jsonld_type: None,
        title: None,
        description: None,
        identifier: None,
        issued: None,
        modified: None,
        language: None,
        publisher: None,
        creator: None,
        contact_point: None,
        keywords: None,
        themes: None,
        license: None,
        rights: None,
        access_rights: None,
        has_policy: None,
        conforms_to: None,
        landing_page: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://example.org/service".to_string())])),
        type_: None,
        version: None,
        version_notes: None,
        previous_version: None,
        has_version: None,
        has_current_version: None,
        replaces: None,
        status: None,
        qualified_relation: None,
        endpoint_url: OneOrMany::Many(vec!["https://api.example.org".to_string()]),
        endpoint_description: None,
        serves_dataset: None,
    };
    assert!(data_service_valid.validate().is_ok());
    let data_service_invalid_landing = DataService {
        landing_page: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://example.com#foo bar".to_string())])),
        ..data_service_valid.clone()
    };
    assert!(data_service_invalid_landing.validate().is_err());
    let data_service_invalid_endpoint = DataService {
        endpoint_url: OneOrMany::Many(vec!["https://example.com#foo bar".to_string()]),
        ..data_service_valid
    };
    assert!(data_service_invalid_endpoint.validate().is_err());
    let dataset_valid = Dataset {
        id: None,
        jsonld_type: None,
        title: None,
        description: None,
        identifier: None,
        issued: None,
        modified: None,
        language: None,
        publisher: None,
        creator: None,
        contact_point: None,
        keywords: None,
        themes: None,
        license: None,
        rights: None,
        access_rights: None,
        has_policy: None,
        conforms_to: None,
        landing_page: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://example.org/dataset".to_string())])),
        relation: None,
        type_: None,
        version: None,
        version_notes: None,
        previous_version: None,
        has_version: None,
        has_current_version: None,
        replaces: None,
        status: None,
        is_referenced_by: None,
        has_part: None,
        qualified_relation: None,
        first: None,
        last: None,
        previous: None,
        distribution: None,
        frequency: None,
        in_series: None,
        spatial: None,
        spatial_resolution_in_meters: None,
        temporal: None,
        temporal_resolution: None,
        was_generated_by: None,
    };
    assert!(dataset_valid.validate().is_ok());
    let dataset_invalid = Dataset {
        landing_page: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://example.com#foo bar".to_string())])),
        ..dataset_valid
    };
    assert!(dataset_invalid.validate().is_err());
    let catalog_valid = Catalog {
        id: None,
        jsonld_type: None,
        title: None,
        description: None,
        identifier: None,
        issued: None,
        modified: None,
        language: None,
        publisher: None,
        creator: None,
        contact_point: None,
        keywords: None,
        themes: None,
        license: None,
        rights: None,
        access_rights: None,
        has_policy: None,
        conforms_to: None,
        landing_page: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://example.org/catalog".to_string())])),
        relation: None,
        type_: None,
        version: None,
        version_notes: None,
        previous_version: None,
        has_version: None,
        has_current_version: None,
        replaces: None,
        status: None,
        is_referenced_by: None,
        qualified_relation: None,
        distribution: None,
        frequency: None,
        spatial: None,
        spatial_resolution_in_meters: None,
        temporal: None,
        temporal_resolution: None,
        was_generated_by: None,
        homepage: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://catalog.example.org".to_string())])),
        theme_taxonomy: None,
        dataset: None,
        service: None,
        catalog: None,
        record: None,
    };
    assert!(catalog_valid.validate().is_ok());
    let catalog_invalid_landing = Catalog {
        landing_page: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://example.com#foo bar".to_string())])),
        ..catalog_valid.clone()
    };
    assert!(catalog_invalid_landing.validate().is_err());
    let catalog_invalid_homepage = Catalog {
        homepage: Some(OneOrMany::Many(vec![DocumentRef::Uri("https://example.com#foo bar".to_string())])),
        ..catalog_valid
    };
    assert!(catalog_invalid_homepage.validate().is_err());
}
#[test]
fn test_dcat_us_catalog_deserialization() {
    let content = include_str!("../../../../tests/fixtures/schema/dcat/dcat-us-catalog.json");
    let catalog: Catalog = serde_json::from_str(content).unwrap();
    assert_eq!(catalog.jsonld_type.as_deref(), Some("dcat:Catalog"));
    assert!(matches!(catalog.publisher, Some(Publisher::Organization(_))));
    assert_eq!(catalog.service.as_ref().map(Vec::len), Some(1));
    assert!(matches!(
        catalog.homepage.as_ref().and_then(|values| values.first()),
        Some(DocumentRef::Document(_))
    ));
    assert!(catalog.validate().is_ok());
}
#[test]
fn test_dcat_us_data_service_deserialization() {
    let content = include_str!("../../../../tests/fixtures/schema/dcat/dcat-us-data-service.json");
    let service: DataService = serde_json::from_str(content).unwrap();
    assert_eq!(service.jsonld_type.as_deref(), Some("dcat:DataService"));
    assert!(matches!(service.publisher, Some(Publisher::Organization(_))));
    assert!(matches!(
        service.contact_point.as_ref().and_then(|values| values.first()),
        Some(ContactPoint::Kind(_))
    ));
    assert!(matches!(
        service.landing_page.as_ref().and_then(|values| values.first()),
        Some(DocumentRef::Uri(_))
    ));
    assert!(service.validate().is_ok());
}
#[test]
fn test_dcat_us_dataset_deserialization() {
    let content = include_str!("../../../../tests/fixtures/schema/dcat/dcat-us-dataset.json");
    let dataset: Dataset = serde_json::from_str(content).unwrap();
    assert_eq!(dataset.jsonld_type.as_deref(), Some("dcat:Dataset"));
    assert!(matches!(dataset.publisher, Some(Publisher::Organization(_))));
    assert!(matches!(
        dataset.contact_point.as_ref().and_then(|values| values.first()),
        Some(ContactPoint::Kind(_))
    ));
    assert!(matches!(
        dataset.conforms_to.as_ref().and_then(|values| values.first()),
        Some(ConformsTo::Standard(_))
    ));
    assert!(matches!(
        dataset.landing_page.as_ref().and_then(|values| values.first()),
        Some(DocumentRef::Document(_))
    ));
    assert!(dataset.validate().is_ok());
}
#[test]
fn test_dcat_w3c_backward_compat_deserialization() {
    let content = include_str!("../../../../tests/fixtures/schema/dcat.json");
    let dataset: Dataset = serde_json::from_str(content).unwrap();
    assert!(matches!(
        dataset.landing_page.as_ref().and_then(|values| values.first()),
        Some(DocumentRef::Uri(_))
    ));
    assert!(dataset.validate().is_ok());
}
#[test]
fn test_fuzzy_matching_keywords() {
    let resolver = vocabulary::VocabularyResolver::embedded(vocabulary::VocabularyKind::Keywords).unwrap();
    let strict = resolver
        .resolve(["AI", "cyb", "high-performance-computi"], vocabulary::VocabularyOptions::default())
        .unwrap();
    assert_eq!(
        strict.lossless_values(),
        ["artificial-intelligence", "cyber", "high-performance-computing"]
    );
    let relaxed = resolver
        .resolve(
            ["machine-learn"],
            vocabulary::VocabularyOptions {
                fuzzy: vocabulary::VocabularyFuzzyMode::Relaxed,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(relaxed.accepted_values(), ["machine-learning"]);
}
#[test]
fn test_fuzzy_matching_organizations() {
    let resolver = vocabulary::VocabularyResolver::embedded(vocabulary::VocabularyKind::Organizations).unwrap();
    let report = resolver
        .resolve(
            ["ORNL", "Oak Ridge National Laborato", "GSHS", "Research Accelerator Division"],
            vocabulary::VocabularyOptions::default(),
        )
        .unwrap();
    assert_eq!(
        report.accepted_values(),
        [
            "Oak Ridge National Laboratory",
            "Geospatial Science and Human Security Division",
            "Research Accelerator Division"
        ]
    );
}
#[test]
fn test_fuzzy_matching_partners() {
    let resolver = vocabulary::VocabularyResolver::embedded(vocabulary::VocabularyKind::Partners).unwrap();
    let report = resolver
        .resolve(
            ["NREL", "National Renewable Energy Lab", "iu", "kitware"],
            vocabulary::VocabularyOptions::default(),
        )
        .unwrap();
    assert_eq!(
        report.accepted_values(),
        ["National Renewable Energy Laboratory", "Indiana University", "Kitware Inc"]
    );
}
#[test]
fn test_fuzzy_matching_sponsors() {
    let resolver = vocabulary::VocabularyResolver::embedded(vocabulary::VocabularyKind::Sponsors).unwrap();
    let report = resolver
        .resolve(
            ["ORNL", "DHS", "EPA", "DOE-IN", "DARPA", "DOE-EERE"],
            vocabulary::VocabularyOptions::default(),
        )
        .unwrap();
    assert_eq!(report.accepted_values().len(), 6);
}
#[test]
fn test_fuzzy_matching_technology() {
    let resolver = vocabulary::VocabularyResolver::embedded(vocabulary::VocabularyKind::Technology).unwrap();
    let report = resolver
        .resolve(
            ["CSS", "programming_language::Rust", "https://airflow.apache.org", "foobarbaz", ""],
            vocabulary::VocabularyOptions::default(),
        )
        .unwrap();
    assert_eq!(report.accepted_values(), ["css", "rust", "airflow"]);
    assert_eq!(report.lossless_values(), ["css", "rust", "airflow", "foobarbaz"]);
}
#[test]
fn test_fuzzy_matching_treats_pattern_characters_as_literals() {
    let resolver = vocabulary::VocabularyResolver::embedded(vocabulary::VocabularyKind::Keywords).unwrap();
    let report = resolver
        .resolve(["^alpha", "alpha$", "!alpha"], vocabulary::VocabularyOptions::default())
        .unwrap();
    assert!(report.accepted_values().is_empty());
}
#[test]
fn test_geonames() {
    let countries = Constant::country_data();
    assert!(!countries.is_empty(), "Expected at least one GeoNames row");
    let us = countries.find_by_iso("US").expect("US row not found in GeoNames data");
    let Country {
        name,
        top_level_domain,
        population,
        ..
    } = us;
    assert_eq!(name, "United States");
    assert_eq!(top_level_domain, ".us");
    assert!(population.unwrap_or(0) > 0);
    assert_eq!(countries.find_by_name("Germany").unwrap().iso, "DE");
    assert_eq!(countries.find_by_name("germany").unwrap().iso, "DE");
    assert!(countries.contains("FR"), "Expected to find France in GeoNames data");
    assert!(countries.contains("fr"), "Expected to find France in GeoNames data");
    assert!(countries.contains("FRA"), "Expected to find France in GeoNames data");
    assert!(!countries.contains("XYZ"), "Expected not to find XYZ in GeoNames data");
    let alpha2_codes = Constant::country_codes(CodeFormat::Alpha2);
    assert!(alpha2_codes.contains(&"us".to_string()));
    let alpha3_codes = Constant::country_codes(CodeFormat::Alpha3);
    assert!(alpha3_codes.contains(&"usa".to_string()));
    let alpha2_languages = Constant::languages(CodeFormat::Alpha2);
    assert!(alpha2_languages.iter().all(|value| value.len() == 2));
    let alpha3_languages = Constant::languages(CodeFormat::Alpha3);
    assert!(alpha3_languages.iter().all(|value| value.contains('-')));
}
#[test]
fn test_metadata_formatting_applies_selected_fuzzy_policy() {
    let metadata = ResearchActivityMetadata {
        keywords: vec!["automatin".to_string(), "unknown-value".to_string()],
        ..ResearchActivityMetadata::default()
    };
    let strict = metadata
        .clone()
        .format_with_vocabulary_report(vocabulary::VocabularyFuzzyMode::Strict)
        .expect("strict vocabulary format");
    assert_eq!(strict.value.keywords, ["automation", "unknown-value"]);
    assert_eq!(strict.reports[0].resolutions[0].source_locator.as_deref(), Some("meta.keywords[0]"));
    let off = metadata
        .format_with_vocabulary_report(vocabulary::VocabularyFuzzyMode::Off)
        .expect("exact-only vocabulary format");
    assert_eq!(off.value.keywords, ["automatin", "unknown-value"]);
}
#[test]
fn test_organization() {
    const ORNL_ORGANIZATION_COUNT: usize = 368;
    let data = Organization::load();
    let ornl = data[0].clone();
    assert_eq!(data.len(), 1);
    assert_eq!(ornl.name, "Oak Ridge National Laboratory".to_string());
    let ornl_members = ornl.clone().members();
    assert_eq!(ornl_members.len(), ORNL_ORGANIZATION_COUNT);
    let directorates = ornl_members
        .into_iter()
        .filter(|Organization { additional_type, .. }| *additional_type == OrganizationType::Directorate)
        .collect::<Vec<Organization>>();
    assert_eq!(directorates.len(), 8);
    let center = ornl.member("Quantum Science Center").unwrap();
    assert_eq!(center.alternative_name, Some("QSC".to_string()));
    assert!(OrganizationType::Group.order() < OrganizationType::Division.order());
}
#[test]
fn test_organization_alternative_names() {
    let names = Organization::alternative_names();
    insta::assert_snapshot!(format!("{names:#?}"));
}
#[test]
fn test_organization_graph() {
    let nssd = "National Security Sciences Directorate";
    let gshs = "Geospatial Science and Human Security Division";
    let nssd_group = "Spatial Statistics";
    let ornl = Organization::load()[0].clone();
    let group = ornl.clone().member(nssd_group).unwrap();
    assert_eq!(group.name, nssd_group);
    assert_eq!(group.clone().nearest(OrganizationType::Group).unwrap().name, nssd_group);
    assert_eq!(group.clone().nearest(OrganizationType::Division).unwrap().name, gshs);
    assert_eq!(group.clone().nearest(OrganizationType::Directorate).unwrap().name, nssd);
    assert_eq!(group.nearest(OrganizationType::Ffrdc).unwrap().name, ornl.name);
    assert!(ornl.clone().nearest(OrganizationType::Group).is_none());
    assert!(ornl.clone().nearest(OrganizationType::Directorate).is_none());
    let division = ornl.member("Research Accelerator Division").unwrap();
    assert_eq!(
        division.nearest(OrganizationType::Directorate).unwrap().name,
        "Neutron Sciences Directorate"
    );
}
#[test]
fn test_read() {
    for entry in glob("./**/*").expect("Failed to read glob pattern") {
        match entry {
            | Ok(path) => println!("{:?}", path.display()),
            | Err(e) => println!("{e:?}"),
        }
    }
}
#[test]
fn test_status() {
    let data: Status = serde_json::from_str("\"active\"").unwrap();
    assert_eq!(data, Status::Active);
    let data: Status = serde_json::from_str("\"paused\"").unwrap();
    assert_eq!(data, Status::Paused);
    let data: Status = serde_json::from_str("\"on-hold\"").unwrap();
    assert_eq!(data, Status::Paused);
    let data: Status = serde_json::from_str("\"completed\"").unwrap();
    assert_eq!(data, Status::Completed);
    let data: Result<Status, _> = serde_json::from_str("\"not done\"");
    assert!(data.is_err());
}
#[test]
fn test_trl() {
    assert_eq!(TechnologyReadinessLevel::Principles, serde_json::from_str("0").unwrap());
    assert_eq!(TechnologyReadinessLevel::MissionCapable, serde_json::from_str("9").unwrap());
    assert_eq!(TechnologyReadinessLevel::MissionCapable.to_string(), "Mission Capable");
    assert!(TechnologyReadinessLevel::Research < TechnologyReadinessLevel::MissionCapable);
}
#[test]
fn test_trl_into_maturity() {
    let cases = [
        (TechnologyReadinessLevel::Principles, Maturity::Discovery),
        (TechnologyReadinessLevel::Research, Maturity::Discovery),
        (TechnologyReadinessLevel::Concept, Maturity::Concept),
        (TechnologyReadinessLevel::Feasible, Maturity::Concept),
        (TechnologyReadinessLevel::Developing, Maturity::Development),
        (TechnologyReadinessLevel::Developed, Maturity::Development),
        (TechnologyReadinessLevel::Prototype, Maturity::Prototype),
        (TechnologyReadinessLevel::Operational, Maturity::Prototype),
        (TechnologyReadinessLevel::MissionReady, Maturity::Prototype),
        (TechnologyReadinessLevel::MissionCapable, Maturity::Proven),
    ];
    cases
        .iter()
        .for_each(|(trl, maturity)| assert_eq!(Maturity::from(trl.clone()), *maturity));
}
#[test]
#[should_panic]
fn test_trl_panic() {
    assert_eq!(TechnologyReadinessLevel::Principles, serde_json::from_str("10").unwrap());
}
