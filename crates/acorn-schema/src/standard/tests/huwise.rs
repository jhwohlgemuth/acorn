#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use super::canonical_json;
use crate::standard::huwise::{Catalog, DublinCoreType};
use crate::standard::zon;
use crate::validation::Validate;
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};
use serde_json::Value;

const FIXTURE_PATH: &str = "../../tests/fixtures/schema/ods.json";
fn load_fixture_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_PATH);
    std::fs::read_to_string(path).expect("failed to read ods.json fixture")
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|value| value.parse().ok()).unwrap_or(64),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/standard/huwise.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_dataset_round_trips_in_supported_formats(
        id in "[a-z][a-z0-9-]{0,24}",
        title in prop_oneof![Just("𝄞 dataset".to_string()), Just("Alpha\r\nBeta".to_string()), "[A-Za-z0-9][A-Za-z0-9 ]{0,39}"],
        creator in any::<String>(),
        has_records in any::<bool>(),
        attachments_count in 0_u64..100,
    ) {
        let source = serde_json::json!({
            "dataset_id": id,
            "has_attachments": attachments_count > 0,
            "attachments_count": attachments_count,
            "has_records": has_records,
            "fields": {},
            "metas": {"default": {"title": title}, "dcat": {"creator": creator}},
            "features": [],
        });
        let dataset: crate::standard::huwise::Dataset = serde_json::from_value(source).expect("generated HuWise dataset");
        prop_assert!(dataset.validate().is_ok());
        let expected = serde_json::to_value(&dataset).expect("serialize HuWise dataset");
        let json: crate::standard::huwise::Dataset = serde_json::from_str(&serde_json::to_string(&dataset).expect("JSON encoding")).expect("JSON decoding");
        let yaml: crate::standard::huwise::Dataset = serde_norway::from_str(&serde_norway::to_string(&dataset).expect("YAML encoding")).expect("YAML decoding");
        let zon: crate::standard::huwise::Dataset = zon::decode(&zon::encode(&dataset).expect("ZON encoding")).expect("ZON decoding");
        for decoded in [json, yaml, zon] {
            let value = serde_json::to_value(&decoded).expect("serialize decoded dataset");
            prop_assert_eq!(&value, &expected);
            prop_assert_eq!(decoded.validate().is_ok(), dataset.validate().is_ok());
        }
    }
}
#[test]
fn test_ods_fixture_parsing() {
    let text = load_fixture_text();
    let catalog: Catalog = serde_json::from_str(&text).expect("failed to parse ods.json fixture");
    assert_eq!(catalog.len(), 1);
    let dataset = &catalog[0];
    assert_eq!(dataset.dataset_id, "crushed-stone-operations");
    assert!(!dataset.has_attachments);
    assert_eq!(dataset.attachments_count, 0);
    assert!(!dataset.has_records);
    let fields = dataset.fields.as_object().expect("fields should be a JSON object");
    assert!(fields.is_empty());
    assert!(dataset.features.is_empty());
    let metas = dataset.metas.dcat.as_ref().expect("dcat metadata should exist");
    assert_eq!(metas.issued.as_deref(), Some("2017-06-12"));
    assert_eq!(metas.creator.as_deref(), Some("HostedByHIFLD"));
    assert_eq!(metas.access_rights, None);
    let default_meta = dataset.metas.r#default.as_ref().expect("default metadata should exist");
    assert_eq!(
        default_meta.title.as_deref(),
        Some("Dataset does not exist anymore: Crushed Stone Operations")
    );
    assert_eq!(default_meta.records_count, Some(0));
    assert_eq!(default_meta.federated, Some(false));
    let dublin_core = dataset.metas.dublin_core.as_ref().expect("dublin-core metadata should exist");
    assert_eq!(dublin_core.kind, Some(DublinCoreType::Dataset));
    assert_eq!(dublin_core.language.as_deref(), Some("eng"));
    assert_eq!(dublin_core.spatial.as_deref(), Some("United States"));
    let dcat_ap = dataset.metas.dcat_ap.as_ref().expect("dcat_ap metadata should exist");
    assert_eq!(dcat_ap.publisher_name.as_deref(), Some("GeoPlatform ArcGIS Online"));
    assert_eq!(dcat_ap.keyword.as_ref().map(Vec::len), Some(16));
    let custom_template = dataset.metas.custom_template.as_ref().expect("custom-template metadata should exist");
    assert_eq!(custom_template.source_of_data.as_deref(), Some("Sample Data Provided"));
    let datacite = dataset.metas.datacite.as_ref().expect("datacite metadata should exist");
    assert_eq!(datacite.title.as_deref(), Some("Crushed Stone Operations"));
    assert_eq!(datacite.publisher.as_deref(), Some(""));
    assert_eq!(datacite.publication_year.as_deref(), Some("2017"));
}
#[test]
fn test_ods_fixture_snapshot() {
    let text = load_fixture_text();
    let json: Value = serde_json::from_str(&text).map(canonical_json).expect("failed to parse ods.json fixture");
    insta::assert_snapshot!(serde_json::to_string_pretty(&json).expect("failed to format json"));
}
