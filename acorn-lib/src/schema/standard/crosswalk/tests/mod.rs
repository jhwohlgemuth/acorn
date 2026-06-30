#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::standard::crosswalk::*;
use crate::schema::standard::crosswalk::{
    mapping::{
        datacite_to_dcat, datacite_to_huwise, datacite_to_invenio, dcat_to_datacite, extract_year, huwise_to_datacite, invenio_to_datacite,
        year_to_date,
    },
    FieldValue, Fields, SchemaBuilder, SchemaExtractor,
};
use crate::schema::standard::{datacite, dcat, huwise, invenio};
use crate::schema::OneOrMany;
use proptest::prelude::*;
use serde_json::json;

#[test]
fn mapping_apply_basic() {
    let mut source = Fields::new();
    source.insert("title", FieldValue::String("Test Title".to_string()));
    let mut target = Fields::new();
    let rule = FieldRule::new("title", "name");
    rule.apply(&source, &mut target).ok();
    assert_eq!(target.get_string_opt("name"), Some("Test Title".to_string()));
}
#[test]
fn mapping_missing_optional_field() {
    let source = Fields::new();
    let mut target = Fields::new();
    let rule = FieldRule::new("title", "name");
    let missing = rule.apply(&source, &mut target).ok().flatten();
    assert_eq!(missing, Some("title".to_string()));
    assert!(target.is_empty());
}
#[test]
fn mapping_required_field_fails() {
    let source = Fields::new();
    let mut target = Fields::new();
    let rule = FieldRule::new("title", "name").required();
    let err = rule.apply(&source, &mut target);
    assert!(err.is_err());
    match err {
        | Err(CrosswalkError::MissingRequiredField(f)) => {
            assert_eq!(f, "title");
        }
        | _ => panic!("expected MissingRequiredField error"),
    }
}
#[test]
fn extract_datacite_record_basic() {
    let record = datacite::Record {
        id: "10.5555/12345678".to_string(),
        kind: "dois".to_string(),
        attributes: datacite::Attributes {
            doi: "10.5555/12345678".to_string(),
            event: None,
            titles: Some(vec![datacite::Title {
                title: "Test Dataset".to_string(),
                title_type: None,
            }]),
            creators: None,
            publisher: None,
            publication_year: Some(2024),
            resource_types: None,
            url: None,
            subjects: None,
            contributors: None,
            dates: None,
            language: Some("en".to_string()),
            alternate_identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            version: None,
            rights_list: None,
            descriptions: None,
            geo_locations: None,
            funding_references: None,
            schema_version: None,
        },
    };
    let fields = record.extract_fields();
    assert_eq!(fields.get_string("doi").ok(), Some("10.5555/12345678".to_string()));
    assert_eq!(fields.get_string("title").ok(), Some("Test Dataset".to_string()));
    assert_eq!(fields.get_number("publication-year").ok(), Some(2024.0));
}
#[test]
fn extract_dcat_dataset_basic() {
    let dataset = dcat::Dataset {
        id: None,
        jsonld_type: None,
        title: Some(OneOrMany::Many(vec!["Test Dataset".to_string()])),
        description: Some(OneOrMany::Many(vec!["A test dataset".to_string()])),
        identifier: Some(OneOrMany::Many(vec!["10.5555/12345678".to_string()])),
        issued: Some("2024-01-15".to_string()),
        modified: None,
        language: Some(OneOrMany::Many(vec!["en".to_string()])),
        publisher: None,
        creator: None,
        contact_point: None,
        keywords: Some(OneOrMany::Many(vec!["test".to_string(), "data".to_string()])),
        themes: None,
        license: None,
        rights: None,
        access_rights: None,
        has_policy: None,
        conforms_to: None,
        landing_page: None,
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
    let fields = dataset.extract_fields();
    assert_eq!(fields.get_string_vec("identifier").ok(), Some(vec!["10.5555/12345678".to_string()]));
    assert_eq!(fields.get_string("title").ok(), Some("Test Dataset".to_string()));
    assert_eq!(fields.get_string_vec_opt("keywords"), Some(vec!["test".to_string(), "data".to_string()]));
}
#[test]
fn build_datacite_from_fields() {
    let mut fields = Fields::new();
    fields.insert("doi", FieldValue::String("10.5555/87654321".to_string()));
    fields.insert("title", FieldValue::String("Built Record".to_string()));
    fields.insert("publication-year", FieldValue::Number(2025.0));

    let record = datacite::Record::build_from_fields(&fields);
    assert!(record.is_ok());

    let record = record.unwrap();
    assert_eq!(record.attributes.doi, "10.5555/87654321");
    assert_eq!(record.attributes.titles.as_ref().map(|t| &t[0].title), Some(&"Built Record".to_string()));
    assert_eq!(record.attributes.publication_year, Some(2025));
}
#[test]
fn build_dcat_from_fields() {
    let mut fields = Fields::new();
    fields.insert("identifier", FieldValue::StringVec(vec!["http://example.org/dataset/1".to_string()]));
    fields.insert("title", FieldValue::String("Built Dataset".to_string()));
    fields.insert("issued", FieldValue::Date("2025-03-20".to_string()));
    let dataset = dcat::Dataset::build_from_fields(&fields);
    assert!(dataset.is_ok());
    let dataset = dataset.unwrap();
    assert_eq!(
        dataset.identifier,
        Some(OneOrMany::Many(vec!["http://example.org/dataset/1".to_string()]))
    );
    assert_eq!(dataset.title, Some(OneOrMany::Many(vec!["Built Dataset".to_string()])));
    assert_eq!(dataset.issued, Some("2025-03-20".to_string()));
}
#[test]
fn from_datacite_to_dcat() {
    let record = datacite::Record {
        id: "10.5555/12345678".to_string(),
        kind: "dois".to_string(),
        attributes: datacite::Attributes {
            doi: "10.5555/12345678".to_string(),
            event: None,
            titles: Some(vec![datacite::Title {
                title: "Science Dataset".to_string(),
                title_type: None,
            }]),
            creators: None,
            publisher: None,
            publication_year: Some(2024),
            resource_types: None,
            url: Some("https://example.org/dataset".to_string()),
            subjects: None,
            contributors: None,
            dates: None,
            language: Some("en".to_string()),
            alternate_identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            version: None,
            rights_list: None,
            descriptions: Some(vec![datacite::Description {
                description: "A scientific dataset".to_string(),
                description_type: None,
                language: None,
            }]),
            geo_locations: None,
            funding_references: None,
            schema_version: None,
        },
    };
    let dataset: dcat::Dataset = record.try_into().unwrap();
    assert_eq!(dataset.title, Some(OneOrMany::Many(vec!["Science Dataset".to_string()])));
    assert_eq!(dataset.description, Some(OneOrMany::Many(vec!["A scientific dataset".to_string()])));
    assert_eq!(dataset.language, Some(OneOrMany::Many(vec!["en".to_string()])));
    assert_eq!(dataset.jsonld_type.as_deref(), Some("dcat:Dataset"));
    assert!(matches!(
        dataset.landing_page.as_ref().and_then(|values| values.first()),
        Some(dcat::DocumentRef::Uri(_))
    ));
}
#[test]
fn from_dcat_to_datacite() {
    let dataset = dcat::Dataset {
        id: None,
        jsonld_type: None,
        title: Some(OneOrMany::Many(vec!["Open Data Collection".to_string()])),
        description: Some(OneOrMany::Many(vec!["Public data for research".to_string()])),
        identifier: Some(OneOrMany::Many(vec!["https://doi.org/10.5555/87654321".to_string()])),
        issued: Some("2024-06-15".to_string()),
        modified: None,
        language: Some(OneOrMany::Many(vec!["fr".to_string()])),
        publisher: None,
        creator: None,
        contact_point: None,
        keywords: Some(OneOrMany::Many(vec!["open".to_string(), "data".to_string()])),
        themes: None,
        license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
        rights: None,
        access_rights: None,
        has_policy: None,
        conforms_to: None,
        landing_page: None,
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
    let record: datacite::Record = dataset.try_into().unwrap();
    assert_eq!(record.attributes.doi, "https://doi.org/10.5555/87654321");
    assert_eq!(
        record.attributes.titles.as_ref().map(|t| &t[0].title),
        Some(&"Open Data Collection".to_string())
    );
    assert_eq!(
        record.attributes.descriptions.as_ref().map(|d| &d[0].description),
        Some(&"Public data for research".to_string())
    );
    assert_eq!(record.attributes.language, Some("fr".to_string()));
}
#[test]
fn extract_invenio_record_basic() {
    let record = invenio::Record {
        schema: Some("https://inveniordm.docs.cern.ch/reference/metadata/".to_string()),
        id: Some("test-invenio-001".to_string()),
        pid: Some(invenio::PersistentIdentifier {
            pk: Some(1),
            status: Some("R".to_string()),
        }),
        pids: None,
        parent: None,
        access: None,
        metadata: Some(invenio::Metadata {
            resource_type: None,
            title: Some("Invenio Test Dataset".to_string()),
            publication_date: Some("2024-06-15".to_string()),
            creators: None,
            additional_titles: None,
            description: None,
            additional_descriptions: None,
            rights: None,
            copyright: None,
            contributors: None,
            subjects: None,
            languages: None,
            dates: None,
            version: None,
            publisher: None,
            identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            locations: None,
            funding: None,
            references: None,
        }),
        custom_fields: None,
        files: None,
        tombstone: None,
        created: Some("2024-06-15T10:00:00Z".to_string()),
        updated: Some("2024-06-16T12:00:00Z".to_string()),
    };
    let fields = record.extract_fields();
    assert_eq!(fields.get_string("identifier").ok(), Some("1".to_string()));
    assert_eq!(fields.get_string("title").ok(), Some("Invenio Test Dataset".to_string()));
    assert_eq!(fields.get_number("publication-year").ok(), Some(2024.0));
}
#[test]
fn build_invenio_from_fields() {
    let mut fields = Fields::new();
    fields.insert("identifier", FieldValue::String("invenio-built-001".to_string()));
    fields.insert("title", FieldValue::String("Built Invenio Record".to_string()));
    fields.insert("publication-year", FieldValue::Number(2025.0));
    let record = invenio::Record::build_from_fields(&fields);
    assert!(record.is_ok());
    let record = record.unwrap();
    assert_eq!(record.id, Some("invenio-built-001".to_string()));
    assert_eq!(
        record.metadata.as_ref().and_then(|m| m.title.clone()),
        Some("Built Invenio Record".to_string())
    );
    assert_eq!(
        record.metadata.as_ref().and_then(|m| m.publication_date.clone()),
        Some("2025-01-01".to_string())
    );
}
#[test]
fn from_invenio_to_datacite() {
    let record = invenio::Record {
        schema: Some("https://inveniordm.docs.cern.ch/reference/metadata/".to_string()),
        id: Some("invenio-convert-001".to_string()),
        pid: Some(invenio::PersistentIdentifier {
            pk: None,
            status: Some("U".to_string()),
        }),
        pids: None,
        parent: None,
        access: None,
        metadata: Some(invenio::Metadata {
            resource_type: None,
            title: Some("Invenio to DataCite Test".to_string()),
            publication_date: Some("2024-03-01".to_string()),
            creators: None,
            additional_titles: None,
            description: Some("A test record".to_string()),
            additional_descriptions: None,
            rights: None,
            copyright: None,
            contributors: None,
            subjects: None,
            languages: Some(vec![invenio::Language { id: "en".to_string() }]),
            dates: None,
            version: None,
            publisher: None,
            identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            locations: None,
            funding: None,
            references: None,
        }),
        custom_fields: None,
        files: None,
        tombstone: None,
        created: Some("2024-03-01T00:00:00Z".to_string()),
        updated: None,
    };
    let datacite_record: datacite::Record = record.try_into().unwrap();
    assert_eq!(datacite_record.attributes.doi, "invenio-convert-001");
    assert_eq!(
        datacite_record.attributes.titles.as_ref().map(|t| &t[0].title),
        Some(&"Invenio to DataCite Test".to_string())
    );
    assert_eq!(
        datacite_record.attributes.descriptions.as_ref().map(|d| &d[0].description),
        Some(&"A test record".to_string())
    );
    assert_eq!(datacite_record.attributes.language, Some("en".to_string()));
}
#[test]
fn from_datacite_to_invenio() {
    let record = datacite::Record {
        id: "10.5555/datacite-to-inv".to_string(),
        kind: "dois".to_string(),
        attributes: datacite::Attributes {
            doi: "10.5555/datacite-to-inv".to_string(),
            event: None,
            titles: Some(vec![datacite::Title {
                title: "DataCite to Invenio Test".to_string(),
                title_type: None,
            }]),
            creators: None,
            publisher: None,
            publication_year: Some(2024),
            resource_types: None,
            url: None,
            subjects: None,
            contributors: None,
            dates: None,
            language: Some("de".to_string()),
            alternate_identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            version: Some("1.0".to_string()),
            rights_list: None,
            descriptions: Some(vec![datacite::Description {
                description: "A datacite test record".to_string(),
                description_type: None,
                language: None,
            }]),
            geo_locations: None,
            funding_references: None,
            schema_version: None,
        },
    };
    let invenio_record: invenio::Record = record.try_into().unwrap();
    assert_eq!(invenio_record.id, Some("10.5555/datacite-to-inv".to_string()));
    assert_eq!(
        invenio_record.metadata.as_ref().and_then(|m| m.title.clone()),
        Some("DataCite to Invenio Test".to_string())
    );
    assert_eq!(invenio_record.metadata.as_ref().and_then(|m| m.version.clone()), Some("1.0".to_string()));
    assert_eq!(
        invenio_record.metadata.as_ref().and_then(|m| m.languages.as_ref()).map(|l| &l[0].id),
        Some(&"de".to_string())
    );
}
#[test]
fn extract_huwise_dataset_with_datacite_block() {
    let dataset = huwise::Dataset {
        dataset_id: "huwise-001".to_string(),
        has_attachments: false,
        attachments_count: 0,
        has_records: false,
        fields: json!([]),
        metas: huwise::Meta {
            datacite: Some(huwise::Datacite {
                identifier: Some("10.5555/huwise-test".to_string()),
                title: Some("HuWise DataCite Block Test".to_string()),
                alternative_title: None,
                publisher: None,
                creator: Some(json!(["Jane Smith", "John Doe"])),
                publication_year: Some("2024".to_string()),
                subject: Some(vec!["data".to_string(), "science".to_string()]),
                contributor: None,
                date: None,
                language: Some("en".to_string()),
                resource_type: Some(json!("Dataset")),
                alternate_identifier: None,
                related_identifier: None,
                size: None,
                format: None,
                version: Some("1.0".to_string()),
                rights: Some(json!("CC-BY-4.0")),
                description: Some("Test dataset with DataCite metadata".to_string()),
                geolocation: None,
            }),
            r#default: None,
            dublin_core: None,
            dcat: None,
            dcat_ap: None,
            custom_template: None,
        },
        features: vec![],
    };
    let fields = dataset.extract_fields();
    assert_eq!(fields.get_string("identifier").ok(), Some("huwise-001".to_string()));
    assert_eq!(fields.get_string("title").ok(), Some("HuWise DataCite Block Test".to_string()));
    assert_eq!(fields.get_number("publication-year").ok(), Some(2024.0));
    assert_eq!(
        fields.get_string_vec("creators").ok(),
        Some(vec!["Jane Smith".to_string(), "John Doe".to_string()])
    );
}
#[test]
fn extract_huwise_dataset_with_default_block() {
    let dataset = huwise::Dataset {
        dataset_id: "huwise-default-001".to_string(),
        has_attachments: false,
        attachments_count: 0,
        has_records: false,
        fields: json!([]),
        metas: huwise::Meta {
            datacite: None,
            r#default: Some(huwise::DefaultMeta {
                title: Some("HuWise Default Block Test".to_string()),
                description: Some("A comprehensive test dataset".to_string()),
                keyword: Some(vec!["test".to_string(), "default".to_string()]),
                language: Some("en".to_string()),
                publisher: Some("Test Publisher".to_string()),
                license: Some("CC-BY-4.0".to_string()),
                modified: Some("2024-01-15T10:00:00Z".to_string()),
                theme: None,
                metadata_languages: None,
                timezone: None,
                modified_updates_on_metadata_change: None,
                modified_updates_on_data_change: None,
                data_processed: None,
                metadata_processed: None,
                geographic_reference: None,
                geographic_reference_auto: None,
                territory: None,
                geometry_types: None,
                bbox: None,
                references: None,
                records_count: None,
                attributions: None,
                source_domain: None,
                source_domain_title: None,
                source_domain_address: None,
                source_dataset: None,
                shared_catalog: None,
                federated: None,
                parent_domain: None,
                update_frequency: None,
                license_url: None,
            }),
            dublin_core: None,
            dcat: None,
            dcat_ap: None,
            custom_template: None,
        },
        features: vec![],
    };
    let fields = dataset.extract_fields();
    assert_eq!(fields.get_string("title").ok(), Some("HuWise Default Block Test".to_string()));
    assert_eq!(fields.get_string("description").ok(), Some("A comprehensive test dataset".to_string()));
    assert_eq!(
        fields.get_string_vec("subjects").ok(),
        Some(vec!["test".to_string(), "default".to_string()])
    );
}
#[test]
fn build_huwise_from_fields() {
    let mut fields = Fields::new();
    fields.insert("identifier", FieldValue::String("huwise-built-001".to_string()));
    fields.insert("title", FieldValue::String("Built HuWise Dataset".to_string()));
    fields.insert("description", FieldValue::String("Reconstructed from FieldMap".to_string()));
    fields.insert("creators", FieldValue::StringVec(vec!["Alice".to_string(), "Bob".to_string()]));
    fields.insert("publication-year", FieldValue::Number(2025.0));
    fields.insert("license", FieldValue::String("CC-BY-4.0".to_string()));
    let dataset = huwise::Dataset::build_from_fields(&fields);
    assert!(dataset.is_ok());
    let dataset = dataset.unwrap();
    assert_eq!(dataset.dataset_id, "huwise-built-001");
    assert_eq!(
        dataset.metas.datacite.as_ref().and_then(|d| d.title.clone()),
        Some("Built HuWise Dataset".to_string())
    );
    assert_eq!(
        dataset.metas.datacite.as_ref().and_then(|d| d.publication_year.clone()),
        Some("2025".to_string())
    );
}
#[test]
fn from_huwise_to_datacite() {
    let dataset = huwise::Dataset {
        dataset_id: "huwise-to-dc-001".to_string(),
        has_attachments: false,
        attachments_count: 0,
        has_records: false,
        fields: json!([]),
        metas: huwise::Meta {
            datacite: Some(huwise::Datacite {
                identifier: Some("10.5555/huwise-convert".to_string()),
                title: Some("HuWise to DataCite Test".to_string()),
                alternative_title: None,
                publisher: Some("Publisher".to_string()),
                creator: Some(json!(["Test Author"])),
                publication_year: Some("2024".to_string()),
                subject: None,
                contributor: None,
                date: None,
                description: Some("Test conversion".to_string()),
                language: Some("en".to_string()),
                resource_type: Some(json!("Dataset")),
                alternate_identifier: None,
                related_identifier: None,
                size: None,
                format: None,
                version: None,
                rights: Some(json!("CC-BY-4.0")),
                geolocation: None,
            }),
            r#default: None,
            dublin_core: None,
            dcat: None,
            dcat_ap: None,
            custom_template: None,
        },
        features: vec![],
    };
    let datacite_record: datacite::Record = dataset.try_into().unwrap();
    assert_eq!(
        datacite_record.attributes.titles.as_ref().map(|t| &t[0].title),
        Some(&"HuWise to DataCite Test".to_string())
    );
    assert_eq!(
        datacite_record.attributes.descriptions.as_ref().map(|d| &d[0].description),
        Some(&"Test conversion".to_string())
    );
    assert_eq!(datacite_record.attributes.language, Some("en".to_string()));
}

proptest! {
    #[test]
    fn prop_rule_identity_string_roundtrip(value in "\\PC{0,64}") {
        let mut source = Fields::new();
        source.insert("src", FieldValue::String(value.clone()));
        let mut mid = Fields::new();
        prop_assert!(FieldRule::new("src", "mid").apply(&source, &mut mid).is_ok());
        let mut target = Fields::new();
        prop_assert!(FieldRule::new("mid", "tgt").apply(&mid, &mut target).is_ok());
        prop_assert_eq!(target.get_string("tgt").ok(), Some(value));
    }
    #[test]
    fn prop_rule_identity_stringvec_roundtrip(values in prop::collection::vec("\\PC{0,32}", 0..8)) {
        let mut source = Fields::new();
        source.insert("src", FieldValue::StringVec(values.clone()));
        let mut mid = Fields::new();
        prop_assert!(FieldRule::new("src", "mid").apply(&source, &mut mid).is_ok());
        let mut target = Fields::new();
        prop_assert!(FieldRule::new("mid", "tgt").apply(&mid, &mut target).is_ok());
        prop_assert_eq!(target.get_string_vec_opt("tgt"), Some(values));
    }
    #[test]
    fn prop_year_transform_roundtrip(year in 1000i32..=9999) {
        let mut source = Fields::new();
        source.insert("y", FieldValue::Number(year as f64));
        let mut mid = Fields::new();
        let fwd = FieldRule::new("y", "d").with_transform(year_to_date);
        prop_assert!(fwd.apply(&source, &mut mid).is_ok());
        let mut target = Fields::new();
        let rev = FieldRule::new("d", "y").with_transform(extract_year);
        prop_assert!(rev.apply(&mid, &mut target).is_ok());
        prop_assert_eq!(target.get_number("y").ok(), Some(year as f64));
    }
    #[test]
    fn prop_mappings_never_panic(
        strings in prop::collection::vec("\\PC{0,32}", 0..10),
        numbers in prop::collection::vec(any::<f64>(), 0..10),
    ) {
        let mut source = Fields::new();
        for (i, s) in strings.iter().enumerate() {
            source.insert(format!("str-{i}"), FieldValue::String(s.clone()));
        }
        for (i, n) in numbers.iter().enumerate() {
            source.insert(format!("num-{i}"), FieldValue::Number(*n));
        }
        for mapping_fn in &[
            datacite_to_dcat as fn() -> FieldMapping,
            dcat_to_datacite as fn() -> FieldMapping,
            datacite_to_huwise as fn() -> FieldMapping,
            huwise_to_datacite as fn() -> FieldMapping,
            datacite_to_invenio as fn() -> FieldMapping,
            invenio_to_datacite as fn() -> FieldMapping,
        ] {
            let mapping = mapping_fn();
            let mut target = Fields::new();
            let _ = mapping.apply(&source, &mut target);
        }
    }
    #[test]
    fn prop_dcat_datacite_roundtrip(title in "\\PC{0,64}", desc in "\\PC{0,128}") {
        let mut source = Fields::new();
        source.insert("identifier", FieldValue::StringVec(vec!["10.5555/test".to_string()]));
        source.insert("title", FieldValue::StringVec(vec![title.clone()]));
        source.insert("description", FieldValue::StringVec(vec![desc.clone()]));
        let mut data = Fields::new();
        prop_assert!(dcat_to_datacite().apply(&source, &mut data).is_ok());
        let mut back = Fields::new();
        prop_assert!(datacite_to_dcat().apply(&data, &mut back).is_ok());
        prop_assert_eq!(back.get_string_vec_opt("title"), Some(vec![title.clone()]));
        prop_assert_eq!(back.get_string_vec_opt("description"), Some(vec![desc.clone()]));
    }
    #[test]
    fn prop_datacite_dcat_roundtrip(title in "\\PC{0,64}", publisher in "\\PC{0,64}") {
        let mut source = Fields::new();
        source.insert("title", FieldValue::String(title.clone()));
        source.insert("publisher", FieldValue::String(publisher.clone()));
        let mut dcat_fields = Fields::new();
        prop_assert!(datacite_to_dcat().apply(&source, &mut dcat_fields).is_ok());
        let mut back = Fields::new();
        prop_assert!(dcat_to_datacite().apply(&dcat_fields, &mut back).is_ok());
        prop_assert_eq!(back.get_string_opt("title"), Some(title));
        prop_assert_eq!(back.get_string_opt("publisher"), Some(publisher));
    }
}
