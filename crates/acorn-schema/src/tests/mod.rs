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
fn test_classification() {
    assert_eq!(ClassificationLevel::Secret.to_string(), "SECRET");
    assert_eq!(ClassificationLevel::TopSecret.to_string(), "TOP SECRET");
    assert!(ClassificationLevel::Unclassified < ClassificationLevel::Confidential);
    assert!(ClassificationLevel::Confidential < ClassificationLevel::Secret);
    assert!(ClassificationLevel::Secret < ClassificationLevel::TopSecret);
}
#[test]
fn test_fuzzy_matching_keywords() {
    let name = "keywords".to_string();
    let exact: String = "critical-infrastructure".into();
    let misspelled: String = "cyb".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    assert_eq!(resolve_from_csv_asset(name.clone(), misspelled), Some("cyber".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "machine-learn".into()),
        Some("machine-learning".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "Automation".into()), Some("automation".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "math".into()), Some("mathematics".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "mathematics".into()), Some("mathematics".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "ml".into()), Some("machine-learning".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "ai".into()), Some("artificial-intelligence".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "stats".into()), Some("statistics".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "statistics".into()), Some("statistics".into()));
    let exact: String = "high-performance-computing".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "hpc".into()), Some(exact.clone()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "high-performance-computi".into()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
}
#[test]
fn test_controlled_vocabulary_normalizes_aliases_and_discards_unknown_values() {
    assert_eq!(
        ControlledVocabulary::normalize("keywords", ["AI", "unknown-value", "hpc", "AI"]).into_values(),
        ["artificial-intelligence", "high-performance-computing"]
    );
}
#[test]
fn test_fuzzy_matching_organizations() {
    let exact: String = "Oak Ridge National Laboratory".into();
    assert_eq!(resolve_from_organization_json(exact.clone()), Some(exact.clone()));
    assert_eq!(resolve_from_organization_json("ORNL".into()), Some(exact.clone()));
    assert_eq!(resolve_from_organization_json("Oak Ridge National Laborato".into()), Some(exact.clone()));
    let exact: String = "Geospatial Science and Human Security Division".into();
    assert_eq!(resolve_from_organization_json("GSHS".into()), Some(exact.clone()));
    assert_eq!(resolve_from_organization_json(exact.clone()), Some(exact.clone()));
    assert_eq!(
        resolve_from_organization_json("Geospatial Science & Human Security".into()),
        Some(exact.clone())
    );
    assert_eq!(
        resolve_from_organization_json("Geospatial Science and Human Security".into()),
        Some(exact.clone())
    );
    assert_eq!(
        resolve_from_organization_json("Geospatial Science and Human Security".to_lowercase()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_organization_json("PSD".into()), Some("Physical Sciences Directorate".into()));
    assert_eq!(
        resolve_from_organization_json("Research Accelerator Division".into()),
        Some("Research Accelerator Division".into())
    );
}
#[cfg(feature = "std")]
#[test]
fn test_fuzzy_matching_retains_nucleo_pattern_semantics() {
    let values = || vec![vec!["Alpha Beta".to_string()], vec!["Beta Alpha".to_string()]];
    assert_eq!(resolve_from_list_of_lists("^alpha".to_string(), values()), Some("Alpha Beta".to_string()));
    assert_eq!(resolve_from_list_of_lists("alpha$".to_string(), values()), Some("Beta Alpha".to_string()));
}
#[test]
fn test_fuzzy_matching_partners() {
    let name = "partners".to_string();
    let exact: String = "National Renewable Energy Laboratory".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "NREL".into()), Some(exact.clone()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "National Renewable Energy Lab".into()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Indiana University".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "iu".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Kitware Inc".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "kitware".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "Kitware, Inc.".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
}
#[test]
fn test_fuzzy_matching_sponsors() {
    let name = "sponsors".to_string();
    let exact: String = "Oak Ridge National Laboratory".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "ORNL".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Department of Energy".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "Dept of Energy".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "Dept. of Energy".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Department of Homeland Security".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "DHS".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Environmental Protection Agency".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), " epa".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Office of Intelligence and Counterintelligence".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "DOE-IN".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Defense Advanced Research Projects Agency".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "darpa".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Office of Energy Efficiency and Renewable Energy".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "eere".into()), Some(exact.clone()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "Energy Efficiency and Renewable Energy".into()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
}
#[test]
fn test_fuzzy_matching_techology() {
    let name = "technology".to_string();
    let exact: String = "react".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    assert_eq!(resolve_from_csv_asset(name.clone(), "astro".into()), Some("astro".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "CSS".into()), Some("css".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "React.js".into()), Some("react".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "ReactJS".into()), Some("react".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "rs".into()), Some("rust".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "jl".into()), Some("julia".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "VHDL".into()), Some("vhdl".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "programming_language::Rust".into()),
        Some("rust".into())
    );
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "programming_language::R_language".into()),
        Some("r".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "r".into()), Some("r".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "Geospatial Data Abstraction Lib".into()),
        Some("gdal".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "node.js".into()), Some("node".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "js".into()), Some("javascript".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "language::Ecmascript".into()),
        Some("javascript".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "kt".into()), Some("kotlin".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "Redis Open Source".into()), Some("redis".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "scss".into()), Some("sass".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "TypeSpec".into()), Some("typespec".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "pwsh".into()), Some("powershell".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "https://airflow.apache.org".into()),
        Some("airflow".into())
    );
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "http://airflow.apache.org".into()),
        Some("airflow".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "airflow.apache.org".into()), Some("airflow".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "foobarbaz".into()), None);
    assert_eq!(resolve_from_csv_asset(name.clone(), "".into()), None);
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
#[should_panic]
fn test_trl_panic() {
    assert_eq!(TechnologyReadinessLevel::Principles, serde_json::from_str("10").unwrap());
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
