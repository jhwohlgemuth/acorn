#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::geonames::{CodeFormat, Country, GeonamesParser};
use crate::schema::research_activity::aspect::Maturity;
use crate::schema::research_activity::*;
use crate::schema::standard::dcat::{Agent, Catalog, ConformsTo, ContactPoint, DataService, Dataset, Distribution, DocumentRef, Publisher};
use crate::schema::validate::*;
use crate::schema::*;
use crate::util::constants::env::DATABASE_PATH;
use crate::util::{Constant, Searchable};
use glob::glob;
use pretty_assertions::assert_eq;
use proptest::prelude::*;

const VALID_ARK_VALUES: [&str; 8] = [
    "https://n2t.net/ark:99166/w66d60p2",
    "https://n2t.net/ark:/99166/w66d60p2",
    "ark:1234/w5678",
    "https://n2t.net/ark:12148/btv1b8449691v/f29",
    "https://n2t.net/ark:12148/btv1b8449691v/f29/abc.png",
    "ark:12148/btv1b8449691v/f29/abc.TIFF",
    "https://example.org/ark:12345/x6np1wh8k/c3/s5.v7.xsl",
    // https://gallica.bnf.fr/ark:/12148/btv1b8449691v
    "ark:/12148/btv1b8449691v",
];
const VALID_DOI_VALUES: [&str; 31] = [
    // ACORN DOI
    "10.11578/dc.20250604.1",
    "10.1000/182",
    "10.97812345/99990",
    "https://doi.org/10.11578/dc.20250604.1",
    "https://doi.org/10.1000/182",
    "https://doi.org/10.97812345/99990",
    "10.1038/s41562-018-0399-z",
    "10.1038/533452a",
    "10.1109/eScience51609.2021.00010",
    "10.1109/ACCESS.2025.3542334",
    "10.1016/j.jbi.2008.04.010",
    "10.48550/arXiv.2312.10997",
    "10.1098/rsos.171511",
    "10.3233/ISU-2010-0613",
    "10.48550/arXiv.2411.06237",
    "10.2139/ssrn.4900122",
    "10.18653/v1/2025.naacl-long.243",
    "10.1007/978-3-642-38288-8_33",
    "10.48550/arXiv.2501.07391",
    "10.48550/arXiv.2504.01990",
    "10.1109/I-SMAC61858.2024.10714814",
    "10.48550/arXiv.2506.06576",
    "10.1186/s13326-025-00327-4",
    "10.1162/dint_a_00186",
    "10.1609/aaai.v39i24.34743",
    "10.48550/arXiv.2504.16736",
    "10.1002/asi.22636",
    "10.1093/gigascience/giy023",
    "10.1162/99608f92.e1f349c2",
    "10.5479/10088/113528",
    "10.1038/s41597-020-0486-7",
];
const VALID_ORCID_VALUES: [&str; 14] = [
    // Jason Wohlgemuth
    "https://orcid.org/0000-0002-2057-9115",
    // Audrey Carson
    "https://orcid.org/0009-0005-5568-6526",
    "0000-0003-1485-2741",
    "0009-0005-8526-4332",
    "0009-0001-1431-2393",
    "0009-0007-2591-8394",
    "0009-0006-8870-3625",
    "0009-0004-4438-9406",
    "0009-0003-1201-0767",
    "0000-0002-2845-8668",
    "0000-0002-0014-1319",
    "0000-0001-9034-3389",
    "0000-0002-0065-494X",
    "0000-0002-2816-415X",
];
// Downloaded from https://zenodo.org/records/14728473
const VALID_ROR_VALUES: [&str; 28] = [
    "https://ror.org/01qz5mb56",
    "01qz5mb56",
    "https://ror.org/04ttjf776",
    "https://ror.org/01rxfrp27",
    "https://ror.org/023q4bk22",
    "https://ror.org/006jxzx88",
    "https://ror.org/00wfvh315",
    "https://ror.org/05ktbsm52",
    "https://ror.org/00nx6aa03",
    "https://ror.org/02k3cxs74",
    "https://ror.org/046fa4y88",
    "https://ror.org/02d439m40",
    "https://ror.org/04h08p482",
    "https://ror.org/01zctcs90",
    "https://ror.org/05m7zw681",
    "https://ror.org/03awtex73",
    "https://ror.org/00kv9pj15",
    "https://ror.org/03mjtdk61",
    "https://ror.org/04yyp8h20",
    "https://ror.org/022rkxt86",
    "https://ror.org/01wddqe20",
    "https://ror.org/03kwrfk72",
    "https://ror.org/037mpqg03",
    "https://ror.org/05cggb038",
    "https://ror.org/050b31k83",
    "https://ror.org/01q8f6705",
    "https://ror.org/03t40cz74",
    "https://ror.org/03ebg0v16",
];

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod custom_deserialization {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(default, deserialize_with = "optional_string_or_seq")]
        title: Option<Vec<String>>,
        #[serde(default, deserialize_with = "optional_string_or_seq")]
        desc: Option<Vec<String>>,
    }
    #[test]
    fn test_opt_string_or_seq_absent() {
        let json = r#"{"title": ["hello"]}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, Some(vec!["hello".to_string()]));
        assert_eq!(result.desc, None);
    }
    #[test]
    fn test_opt_string_or_seq_single() {
        let json = r#"{"title": "hello", "desc": "world"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, Some(vec!["hello".to_string()]));
        assert_eq!(result.desc, Some(vec!["world".to_string()]));
    }
    #[test]
    fn test_opt_string_or_seq_null() {
        let json = r#"{"title": null, "desc": ["hello"]}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, None);
        assert_eq!(result.desc, Some(vec!["hello".to_string()]));
    }
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
fn test_date_validation() {
    let valid = ["2000-01-01", "2025-06-04", "1950-12-28"];
    let invalid = [
        // "3000-01-01",
        "1949-01-02",
        "01-02-2023",
        "1234",
        "foo",
        "42",
        "2025/06/04",
    ];
    for x in valid.iter() {
        assert!(is_date(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid ISO 8601 date (YYYY-MM-DD)");
    }
    for x in invalid.iter() {
        assert!(is_date(x).is_err(), "=> [REASON] \"{x}\" IS valid ISO 8601 date (YYYY-MM-DD)");
    }
    let valid_years = ["2000", "2025", "1950", "0999"];
    let invalid_years = ["3000", "999", "42"];
    for x in valid_years.iter() {
        assert!(is_year(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid ISO 8601 year (YYYY)");
    }
    for x in invalid_years.iter() {
        assert!(is_year(x).is_err(), "=> [REASON] \"{x}\" IS valid ISO 8601 year (YYYY)");
    }
    let valid_timestamps = [
        "2004-03-02/2005-06-02",
        "2026-03-12T15:04:05Z/2026-03-13T15:04:05Z",
        "2026-03-12",
        "2026-03-12T15:04:05Z",
        "2026-03-12T15:04:05+00:00",
        "2026-03-12T15:04:05.123456Z",
    ];
    let invalid_timestamps = ["2026/03/12T15:04:05Z", "not-a-date", "2026-13-12T15:04:05Z"];
    for x in valid_timestamps.iter() {
        assert!(is_rfc3339(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid RFC3339/ISO 8601 date-time");
    }
    for x in invalid_timestamps.iter() {
        assert!(is_rfc3339(x).is_err(), "=> [REASON] \"{x}\" IS valid RFC3339/ISO 8601 date-time");
    }
}
#[test]
fn test_format_phone_number() {
    let no_country_code = ["555-123-4567", "555.123.4567", "(555) 123-4567"];
    for x in no_country_code.iter() {
        assert_eq!(format_phone_number(x), Ok("555.123.4567".to_string()));
    }
    let with_country_code = ["+1 (555) 123-4567", "+1 (555) 123.4567", "+15551234567"];
    for x in with_country_code.iter() {
        assert_eq!(format_phone_number(x), Ok("+1.555.123.4567".to_string()));
    }
}
#[test]
fn test_format_timestamp() {
    let normalized = format_timestamp("2026-03-10T15:37:24.355614500+00:00");
    assert_eq!(normalized, "2026-03-10T15:37:24.355614+00:00");
    let passthrough = format_timestamp("not-a-date");
    assert_eq!(passthrough, "not-a-date");
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
fn test_has_image_extension() {
    let valid = [
        "foo.png",
        "bar.jpg",
        "baz.jpeg",
        "qux.svg",
        "https://example.com/foo.PNG",
        "https://example.com/bar.JPEG",
    ];
    let invalid = ["foo.pNg", "bar.jpx", "qux.sVG", "https://example.com/fooPNG"];
    for x in valid.iter() {
        assert!(
            has_image_extension(x).is_ok(),
            "=> [REASON] \"{x}\" does NOT HAVE a valid image extension"
        );
    }
    for x in invalid.iter() {
        assert!(has_image_extension(x).is_err(), "=> [REASON] \"{x}\" HAS a valid image extension");
    }
}
#[test]
fn test_is_ark() {
    let valid = VALID_ARK_VALUES;
    let invalid = [
        "978-12345-99990",
        // HTTP not supported
        "http://n2t.net/ark:99166/w66d60p2",
        // Missing ark: label
        "https://n2t.net/99166/w66d60p2",
        // Shoulder starts with number
        "https://n2t.net/99166/9w66d60p2",
        // Shoulder contains letter "l"
        "https://n2t.net/99166/lw66d60p2",
    ];
    for x in valid.iter() {
        assert!(is_ark(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ARK");
    }
    for x in invalid.iter() {
        assert!(is_ark(x).is_err(), "=> [REASON] \"{x}\" IS a valid ARK");
    }
    match is_ark("https://n2t.net/99166/w66d60p2") {
        | Ok(_) => panic!(),
        | Err(err) => assert_eq!(err.to_string(), "Provide valid ARK value"),
    }
}
#[test]
fn test_is_doi() {
    let valid = VALID_DOI_VALUES;
    let invalid = [
        "978-12345-99990",
        // Pretty sure 11 is not a valid DOI directory indicator
        "11.11578/dc.20250604.1",
        "<geo coords=\"10.4515260,51.1656910\"></geo>",
        // 10.5555 is not a DOI prefix, but rather a handle prefix
        "10.5555/182",
    ];
    for x in valid.iter() {
        assert!(is_doi(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid DOI");
    }
    for x in invalid.iter() {
        assert!(is_doi(x).is_err(), "=> [REASON] \"{x}\" IS a valid DOI");
    }
}
#[test]
fn test_is_ip6() {
    let valid = [
        "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        "FE80:0000:0000:0000:0202:B3FF:FE1E:8329",
        "::",
        "::::",
    ];
    let invalid = ["192.168.1.1", "127.0.0.1", "test:test:test:test:test:test:test:test"];
    for x in valid.iter() {
        assert!(is_ip6(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid IP6 address");
    }
    for x in invalid.iter() {
        assert!(is_ip6(x).is_err(), "=> [REASON] \"{x}\" IS a valid IP6 address");
    }
}
#[test]
fn test_is_kebabcase() {
    let valid = ["this-is-valid", "thisisvalid"];
    let invalid = ["this_is_not_valid", "ThisIsNotValid", "this is not valid", "This-Is-Not-Valid"];
    for x in valid.iter() {
        assert!(is_kebabcase(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid kebab-case");
    }
    for x in invalid.iter() {
        assert!(is_kebabcase(x).is_err(), "=> [REASON] \"{x}\" IS valid kebab-case");
    }
}
#[test]
fn test_is_commit() {
    let valid = [
        "a1b2c3d",
        "A1B2C3D4",
        "0123456789abcdef0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ];
    let invalid = [
        "",
        "123456",
        "xyzxyzx",
        "g123456",
        "123456 ",
        "12345-678",
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef0",
    ];
    for x in valid.iter() {
        assert!(is_commit(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid commit hash");
    }
    for x in invalid.iter() {
        assert!(is_commit(x).is_err(), "=> [REASON] \"{x}\" IS a valid commit hash");
    }
}
#[test]
fn test_is_urls() {
    let valid = [
        "http://www.example.com".to_string(),
        "https://example.com".to_string(),
        "https://example.com#foo".to_string(),
        "https://example.com:1337#foo?bar".to_string(),
        "http://example.com:1337#foo?bar=42".to_string(),
    ];
    let invalid = [
        "http://www.example_com".to_string(),
        "https://examp*le.com".to_string(),
        "https://example.com#foo bar".to_string(),
        "https//example.com:1337#foo?bar".to_string(),
    ];
    assert!(is_urls(&valid).is_ok());
    assert!(is_urls(&invalid).is_err());
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
fn test_is_orcid() {
    let valid = VALID_ORCID_VALUES;
    let invalid = [
        "0000-0002-1823-1234",
        "https://orcid.org/0000-0002-1823-12345",
        "https://orcid.com/0000-0002-1823-1234",
    ];
    for x in valid.iter() {
        assert!(is_orcid(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ORCiD");
    }
    for x in invalid.iter() {
        assert!(is_orcid(x).is_err(), "=> [REASON] \"{x}\" IS a valid ORCiD");
    }
}
#[test]
fn test_is_phone_number() {
    let valid = [
        "555-123-4567",
        "555.123.4567",
        "+1 (555) 123-4567",
        "+1 (555) 123.4567",
        "+15551234567",
        "(555) 123-4567",
    ];
    let invalid = [
        // Just a number
        "42",
        // Missing country code sign
        "1 (555) 123-4567",
        // Wrong number of digits
        "1234-567",
        // Missing area code
        "123-4567",
        // Fake
        "555.555.5555",
        // Fake
        "555-555-5555",
    ];
    for x in valid.iter() {
        assert!(is_phone_number(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid phone number");
    }
    for x in invalid.iter() {
        assert!(is_phone_number(x).is_err(), "=> [REASON] \"{x}\" IS a valid phone number");
    }
}
#[test]
fn test_is_semantic_version() {
    let valid = ["1", "1.2", "1.2.3", "0.0.0", "999.10.42"];
    let invalid = ["v1.2.3", "1.2.3.4", "alpha", "1.2.x", "2.1.0-alpha.1", "1.0.0+build.42"];
    for x in valid.iter() {
        assert!(is_semantic_version(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid semantic version");
    }
    for x in invalid.iter() {
        assert!(is_semantic_version(x).is_err(), "=> [REASON] \"{x}\" IS a valid semantic version");
    }
}
#[test]
fn test_is_ror() {
    let valid = VALID_ROR_VALUES;
    let invalid = ["978-12345-99990", "10.1000/182", "10.97812345/99990", "3t40cz74", "03ebg0v16-"];
    for x in valid.iter() {
        assert!(is_ror(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ROR");
    }
    for x in invalid.iter() {
        assert!(is_ror(x).is_err(), "=> [REASON] \"{x}\" IS a valid ROR");
    }
}
#[test]
fn test_is_attribute_ror_list() {
    let valid = VALID_ROR_VALUES.into_iter().map(|x| x.to_string()).collect::<Vec<String>>();
    assert!(is_attribute_ror_list(&valid).is_ok());
    let invalid = ["https://doi.org/10.1000/182".to_string()];
    assert!(is_attribute_ror_list(&invalid).is_err());
}
#[test]
fn test_is_country_code() {
    let valid_codes = ["US", "FR", "DE", "GB", "JP", "us", "fr", "de", "gb", "jp"];
    for code in valid_codes.iter() {
        assert!(
            is_country_code(code).is_ok(),
            "Expected {code} to be valid ISO 3166-1 alpha-2 country code"
        );
    }
    let invalid_codes = [
        "USA",  // 3 letters
        "U",    // 1 letter
        "U.S",  // 3 chars with dot
        "US1",  // contains digit
        "U-S",  // contains dash
        "XX",   // invalid country code
        "ZZ",   // invalid country code
        "",     // empty
        "U",    // single letter
        "USa ", // 4 chars with space
    ];
    for code in invalid_codes.iter() {
        assert!(
            is_country_code(code).is_err(),
            "Expected {code} to be invalid ISO 3166-1 alpha-2 country code"
        );
    }
}
#[test]
fn test_is_iso_639_language_code_validation() {
    assert!(is_iso_639_language_code("en").is_ok());
    assert!(is_iso_639_language_code("EN").is_ok());
    assert!(is_iso_639_language_code("en-US").is_ok());
    assert!(is_iso_639_language_code("eng").is_err());
    assert!(is_iso_639_language_code("zz-ZZ").is_err());
    assert!(is_iso_639_1_language_code("en").is_ok());
    assert!(is_iso_639_1_language_code("EN").is_ok());
    assert!(is_iso_639_1_language_code("eng").is_err());
    assert!(is_iso_639_1_language_code("zz").is_err());
}
#[test]
fn test_is_latlon() {
    let valid_latitude = [-90_f64, -45.7_f64, 0., 10_f64, 90_f64];
    for x in valid_latitude.iter() {
        assert!(is_latitude(*x).is_ok(), "=> [REASON] \"{x}\" IS NOT a valid latitude value");
    }
    let invalid_latitude = [-90.5_f64];
    for x in invalid_latitude.iter() {
        assert!(is_latitude(*x).is_err(), "=> [REASON] \"{x}\" IS a valid latitude value");
    }

    let valid_longitude = [-180_f64, -90_f64, 0., 90_f64, 180_f64];
    for x in valid_longitude.iter() {
        assert!(is_longitude(*x).is_ok(), "=> [REASON] \"{x}\" IS NOT a valid longitude value");
    }
    let invalid_longitude = [-180.5_f64, 180.5_f64];
    for x in invalid_longitude.iter() {
        assert!(is_longitude(*x).is_err(), "=> [REASON] \"{x}\" IS a valid longitude value");
    }
}
proptest! {
    #[test]
    fn prop_is_latitude_accepts_values_in_range(value in -90.0f64..=90.0f64) {
        prop_assert!(is_latitude(value).is_ok());
    }
    #[test]
    fn prop_is_latitude_rejects_values_out_of_range(
        value in prop_oneof![-1000000.0f64..-90.0000001f64, 90.0000001f64..1000000.0f64]
    ) {
        prop_assert!(is_latitude(value).is_err());
    }
    #[test]
    fn prop_is_longitude_accepts_values_in_range(value in -180.0f64..=180.0f64) {
        prop_assert!(is_longitude(value).is_ok());
    }
    #[test]
    fn prop_is_longitude_rejects_values_out_of_range(
        value in prop_oneof![-1000000.0f64..-180.0000001f64, 180.0000001f64..1000000.0f64]
    ) {
        prop_assert!(is_longitude(value).is_err());
    }
}
#[tokio::test]
async fn test_is_license() {
    let path = crate::test::utils::temp_database_with_licenses()
        .await
        .expect("failed to initialize temporary test database");
    let path_string = path.to_string_lossy().to_string();
    temp_env::with_vars([(DATABASE_PATH, Some(path_string.as_str()))], || {
        assert!(is_license("MIT").is_ok());
        assert!(is_license("mit").is_ok());
        assert!(is_license("MIT License").is_ok());
        assert!(is_license("Not-A-Real-License").is_err());
        assert!(is_license("").is_err());
    });
}
#[test]
#[ignore = "requires isolated process (run with cargo nextest)"]
fn test_is_license_invalid_database_path() {
    let dir = std::env::temp_dir().join("acorn_test_invalid_db");
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    let corrupt_db = dir.join("corrupt.db");
    std::fs::write(&corrupt_db, b"not a sqlite database").expect("failed to write corrupt db");
    let path_string = corrupt_db.to_string_lossy().to_string();
    temp_env::with_vars([(DATABASE_PATH, Some(path_string.as_str()))], || {
        let result = is_license("MIT");
        assert!(result.is_err());
        let why = result.unwrap_err();
        assert_eq!(why.message.as_deref(), Some("Failed to access local database for license validation"));
        assert_eq!(why.params.get("value").and_then(|value| value.as_str()), Some("MIT"));
    });
    std::fs::remove_dir_all(&dir).ok();
}
#[test]
fn test_is_license_missing_database_path_env() {
    temp_env::with_vars([(DATABASE_PATH, None::<&str>)], || {
        let result = is_license("MIT");
        assert!(result.is_err());
        let why = result.unwrap_err();
        assert_eq!(why.params.get("value").and_then(|value| value.as_str()), Some("MIT"));
    });
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
#[test]
fn test_scalar_enum_validation_helpers() {
    assert!(IntegerOrString::Number(42).validate().is_ok());
    assert!(IntegerOrString::Text("42".to_string()).validate().is_ok());
    assert!(NumberOrString::Number(42.5).validate().is_ok());
    assert!(NumberOrString::Text("42.5".to_string()).validate().is_ok());
    assert!(YearValue::Number(2025).validate().is_ok());
    assert!(YearValue::Text("2025".to_string()).validate().is_ok());
    let valid_month_values = [
        MonthValue::Number(1),
        MonthValue::Number(12),
        MonthValue::Text("1".to_string()),
        MonthValue::Text("12".to_string()),
    ];
    valid_month_values
        .iter()
        .for_each(|value| assert!(value.validate().is_ok(), "expected valid month value: {value:?}"));

    let invalid_month_values = [MonthValue::Number(0), MonthValue::Text("13".to_string())];
    invalid_month_values.iter().for_each(|value| {
        let errors = value.validate().expect_err("expected invalid month value to fail validation");
        assert!(errors.errors().contains_key("month"));
    });
}
#[test]
fn test_validate_postal_code() {
    let valid_values = [
        PostalCode::Number(37830),
        PostalCode::Text("SW1A 1AA".to_string()),
        PostalCode::Text("75008".to_string()),
    ];
    valid_values
        .iter()
        .for_each(|value| assert!(value.validate().is_ok(), "expected postal code to be valid: {value:?}"));
    let invalid_values = [
        PostalCode::Number(-1),
        PostalCode::Text("   ".to_string()),
        PostalCode::Text("123456789012345678901".to_string()),
        PostalCode::Text("12_345".to_string()),
    ];
    invalid_values.iter().for_each(|value| {
        let errors = value.validate().expect_err("expected postal code to fail validation");
        assert!(errors.errors().contains_key("postal_code"));
    });
}
#[test]
fn test_validate_is_unix_epoch() {
    let valid = [1759017645];
    let invalid = [42, 123456789];
    for x in valid.iter() {
        assert!(is_unix_epoch(*x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid Unix epoch timestamp");
    }
    for x in invalid.iter() {
        assert!(is_unix_epoch(*x).is_err(), "=> [REASON] \"{x}\" IS a valid Unix epoch timestamp");
    }
}
