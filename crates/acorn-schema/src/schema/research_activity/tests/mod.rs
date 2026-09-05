#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::agent::{CostDetails, LimitDetails, Model, ModelDetails, Weight, Weights};
use crate::schema::hardware::{CpuArchitecture, GpuArchitecture, Resource, Vendor};
use crate::schema::research_activity::aspect::{Autonomy, Availability, Data, DataDescription, Modality, Motivity, Quality, SoftwarePortability};
use crate::schema::research_activity::output::{Access, Affiliation, Award, Contributor, Funding};
use crate::schema::research_activity::*;
use crate::schema::standard::cff::{Agent, Cff, IdentifierType};
use crate::schema::*;
use crate::validation::Validate;
use acorn_core::util::SemanticVersion;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

#[test]
fn test_activity_contributor_nested_validation() {
    let contributor = ActivityContributor::init().name("Doe, Jane").build();
    assert!(contributor.validate().is_ok());
    let invalid = ActivityContributor::init().name("").build();
    assert!(invalid.validate().is_err());
    let with_identifiers = ActivityContributor::init()
        .name("Doe, Jane")
        .identifiers(vec![AgentIdentifier::init().value("https://orcid.org/0000-0001-2345-6789").build()])
        .build();
    assert!(with_identifiers.validate().is_ok());
}
#[test]
fn test_agent_identifier_round_trips() {
    let agent = AgentIdentifier::init()
        .value("https://orcid.org/0000-0001-2345-6789")
        .scheme("ORCID")
        .scheme_uri("https://orcid.org")
        .build();
    assert!(agent.validate().is_ok());
    let json = serde_json::to_string(&agent).expect("serialize");
    let parsed: AgentIdentifier = serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed.value, agent.value);
}
#[test]
fn test_alias_migration_accepts_canonical_fields() {
    let image: ImageObject = serde_json::from_value(serde_json::json!({
        "caption": "Caption",
        "contentUrl": "image.png"
    }))
    .expect("canonical image contentUrl should deserialize");
    let video: VideoObject = serde_json::from_value(serde_json::json!({
        "contentSize": null,
        "contentUrl": "https://example.com/video.mp4",
        "description": "Description",
        "duration": null,
        "height": null,
        "width": null
    }))
    .expect("canonical video contentUrl should deserialize");
    let contact: ContactPoint = serde_json::from_value(serde_json::json!({
        "@context": null,
        "@type": null,
        "jobTitle": "Researcher",
        "givenName": "First",
        "familyName": "Last",
        "identifier": null,
        "email": "first_last@example.com",
        "telephone": "123-456-7890",
        "url": "https://example.com",
        "organization": "Example",
        "affiliation": null
    }))
    .expect("canonical contact url should deserialize");
    let sections = Sections::init().build();
    let serialized = serde_json::to_value(&sections).expect("sections should serialize");
    let parsed: Sections = serde_json::from_value(serialized).expect("canonical impact should deserialize");
    assert!(image.validate().is_ok());
    assert!(video.validate().is_ok());
    assert!(contact.validate().is_ok());
    assert!(parsed.validate().is_ok());
}
#[test]
fn test_alias_migration_accepts_deprecated_fields_with_canonical_output() {
    ["href", "url"].into_iter().for_each(|field| {
        let mut value = serde_json::json!({"caption": "Caption"});
        value[field] = serde_json::json!("image.png");
        let image: ImageObject = serde_json::from_value(value).expect("deprecated image URL should deserialize");
        let serialized = serde_json::to_value(image).expect("image should serialize");
        assert_eq!(serialized["contentUrl"], "image.png");
        assert!(serialized.get(field).is_none());
    });
    ["href", "url"].into_iter().for_each(|field| {
        let mut value = serde_json::json!({"description": "Description"});
        value[field] = serde_json::json!("https://example.com/video.mp4");
        let video: VideoObject = serde_json::from_value(value).expect("deprecated video URL should deserialize");
        let serialized = serde_json::to_value(video).expect("video should serialize");
        assert_eq!(serialized["contentUrl"], "https://example.com/video.mp4");
        assert!(serialized.get(field).is_none());
    });
    let mut contact = serde_json::to_value(ContactPoint::init().build()).expect("contact should serialize");
    contact.as_object_mut().expect("contact is an object").remove("url");
    contact["profile"] = serde_json::json!("https://example.com");
    let contact: ContactPoint = serde_json::from_value(contact).expect("deprecated contact profile should deserialize");
    let contact = serde_json::to_value(contact).expect("contact should serialize");
    assert_eq!(contact["url"], "https://example.com");
    assert!(contact.get("profile").is_none());
    let mut sections = serde_json::to_value(Sections::init().build()).expect("sections should serialize");
    let impact = sections.as_object_mut().expect("sections is an object").remove("impact");
    sections["outcomes"] = impact.expect("serialized sections contain impact");
    let sections: Sections = serde_json::from_value(sections).expect("deprecated outcomes should deserialize");
    let sections = serde_json::to_value(sections).expect("sections should serialize");
    assert!(sections.get("impact").is_some());
    assert!(sections.get("outcomes").is_none());
}
#[test]
fn test_metadata_aliases_and_flattened_values_use_canonical_fields() {
    let mut metadata = serde_json::to_value(ResearchActivityMetadata::default()).expect("metadata should serialize");
    let metadata = metadata.as_object_mut().expect("metadata is an object");
    metadata.insert(
        "dates".to_string(),
        serde_json::json!([{"dateType": "StartDate", "date": "2024-01-15T10:00:00Z"}]),
    );
    metadata.insert("contributors".to_string(), serde_json::json!(" Doe, Jane "));
    metadata.insert("organizations".to_string(), serde_json::json!([" Example Organization "]));
    metadata.insert(
        "sponsors".to_string(),
        serde_json::json!([" Example Funder ", {"name": "Structured Funder", "awards": []}]),
    );
    metadata.insert("related".to_string(), serde_json::json!(" related-activity "));
    metadata.insert("languages".to_string(), serde_json::json!([" eng ", {"id": "fra", "schemaUri": null}]));
    metadata.insert("subjects".to_string(), serde_json::json!(" Climate change "));
    metadata.insert("spatialCoverage".to_string(), serde_json::json!(" US-TN "));
    metadata.insert("rights".to_string(), serde_json::json!(" Copyright Example "));
    metadata.insert("access".to_string(), serde_json::json!("open"));
    let parsed: ResearchActivityMetadata =
        serde_json::from_value(serde_json::Value::Object(metadata.clone())).expect("aliases and flattened values should deserialize");
    assert_eq!(parsed.typed_dates.as_ref().expect("dates").len(), 1);
    assert_eq!(parsed.contributors.as_ref().expect("contributors")[0].name, "Doe, Jane");
    assert_eq!(parsed.organizations.as_ref().expect("organizations")[0].name, "Example Organization");
    assert_eq!(parsed.funders.as_ref().expect("funders")[0].name, "Example Funder");
    assert_eq!(
        parsed.related_resources.as_ref().expect("related resources")[0].identifier.value,
        "related-activity"
    );
    assert_eq!(parsed.languages.as_ref().expect("languages")[0].id, "eng");
    assert_eq!(parsed.subjects.as_ref().expect("subjects")[0].subject, "Climate change");
    assert_eq!(parsed.spatial_coverage.as_ref().expect("spatial coverage")[0].id, "US-TN");
    assert_eq!(parsed.rights.as_ref().expect("rights")[0].rights.as_deref(), Some("Copyright Example"));
    assert!(matches!(
        parsed.access.as_ref().and_then(|access| access.access.as_ref()),
        Some(RightsAccess::Open)
    ));
    let canonical = serde_json::to_value(parsed).expect("metadata should serialize canonically");
    for alias in ["dates", "sponsors", "related"] {
        assert!(canonical.get(alias).is_none());
    }
    for field in ["typedDates", "funders", "relatedResources"] {
        assert!(canonical.get(field).is_some());
    }
    let mut context = serde_json::to_value(ResearchActivityMetadataContext::default()).expect("context should serialize");
    let context = context.as_object_mut().expect("context is an object");
    let funders = context.remove("funders").expect("context contains funders");
    let related = context.remove("relatedResources").expect("context contains related resources");
    context.insert("sponsors".to_string(), funders);
    context.insert("related".to_string(), related);
    let context: ResearchActivityMetadataContext =
        serde_json::from_value(serde_json::Value::Object(context.clone())).expect("legacy context aliases should deserialize");
    let canonical = serde_json::to_value(context).expect("context should serialize canonically");
    assert!(canonical.get("sponsors").is_none());
    assert!(canonical.get("related").is_none());
    assert!(canonical.get("funders").is_some());
    assert!(canonical.get("relatedResources").is_some());
}
#[test]
fn test_datacite_aligned_metadata_fixture_supports_simple_and_detailed_forms() {
    let path = fixtures_dir().join("data/valid_project_datacite/index.json");
    let content = std::fs::read_to_string(path).expect("DataCite-aligned RAD fixture should exist");
    let activity: ResearchActivity = eserde::json::from_str(&content).expect("DataCite-aligned RAD fixture should deserialize");
    let validation = activity.validate();
    assert!(validation.is_ok(), "DataCite-aligned RAD fixture should validate: {validation:?}");
    let contributors = activity.meta.contributors.as_ref().expect("contributors should be present");
    assert_eq!(contributors[0].name, "Doe, Jane");
    assert_eq!(contributors[1].identifiers.len(), 1);
    let funders = activity.meta.funders.as_ref().expect("funders should be present");
    assert_eq!(funders[0].name, "U.S. Department of Energy");
    assert_eq!(funders[1].awards.len(), 1);
    let languages = activity.meta.languages.as_ref().expect("languages should be present");
    assert_eq!(languages[0].id, "eng");
    assert_eq!(languages[1].schema_uri.as_deref(), Some("https://iso639-3.sil.org/code/fra"));
    let canonical = serde_json::to_value(activity).expect("fixture should serialize canonically");
    let metadata = &canonical["meta"];
    assert!(metadata.get("dates").is_none());
    assert!(metadata.get("typedDates").is_some());
    for field in [
        "contributors",
        "organizations",
        "funders",
        "relatedResources",
        "languages",
        "subjects",
        "spatialCoverage",
        "rights",
    ] {
        assert!(metadata[field].as_array().expect("metadata field should be an array")[0].is_object());
    }
    assert!(metadata["access"].is_object());
}
#[test]
fn test_invalid_datacite_aligned_metadata_fixture_reports_nested_errors() {
    let path = fixtures_dir().join("data/invalid_project_datacite/index.json");
    let content = std::fs::read_to_string(path).expect("invalid DataCite-aligned RAD fixture should exist");
    let activity: ResearchActivity =
        eserde::json::from_str(&content).expect("invalid DataCite-aligned RAD fixture should deserialize before validation");
    let report = activity
        .validate()
        .expect_err("invalid DataCite-aligned RAD fixture should fail validation");
    let paths = report.iter().map(|issue| issue.path.as_str()).collect::<Vec<_>>();
    [
        "meta.typed_dates[0].date",
        "meta.contributors[0].name",
        "meta.contributors[1].identifiers[0].scheme_uri",
        "meta.organizations[0].name",
        "meta.organizations[1].identifiers[0].scheme_uri",
        "meta.funders[0].name",
        "meta.funders[1].ror",
        "meta.funders[1].awards[0].doi",
        "meta.funders[1].awards[0].url",
        "meta.related_resources[0].identifier.value",
        "meta.related_resources[1].identifier.value",
        "meta.related_resources[1].relation_type",
        "meta.languages[0].id",
        "meta.languages[1].id",
        "meta.languages[1].schema_uri",
        "meta.subjects[0].subject",
        "meta.subjects[1].scheme_uri",
        "meta.subjects[1].value_uri",
        "meta.spatial_coverage[1].schema_uri",
        "meta.rights[1].rights_uri",
        "meta.rights[1].scheme_uri",
        "meta.access.rights_uri",
    ]
    .into_iter()
    .for_each(|expected| assert!(paths.contains(&expected), "missing validation path {expected}: {paths:?}"));
}
#[test]
fn test_alias_migration_retains_established_compatibility_fields() {
    let mut contact = serde_json::to_value(
        ContactPoint::init()
            .identifier("https://orcid.org/0000-0002-2057-9115".to_string())
            .build(),
    )
    .expect("contact should serialize");
    let contact = contact.as_object_mut().expect("contact is an object");
    let identifier = contact.remove("identifier").expect("serialized contact contains identifier");
    let telephone = contact.remove("telephone").expect("serialized contact contains telephone");
    contact.insert("orcid".to_string(), identifier);
    contact.insert("phone".to_string(), telephone);
    assert!(serde_json::from_value::<ContactPoint>(serde_json::Value::Object(contact.clone())).is_ok());
    let media = vec![MediaObject::Image(
        ImageObject::init()
            .caption("Caption".to_string())
            .content_url("image.png".to_string())
            .build(),
    )];
    let mut metadata = serde_json::to_value(ResearchActivityMetadata::init().identifier("identifier".to_string()).media(media).build())
        .expect("metadata should serialize");
    let metadata = metadata.as_object_mut().expect("metadata is an object");
    let identifier = metadata.remove("identifier").expect("serialized metadata contains identifier");
    let media = metadata.remove("media").expect("serialized metadata contains media");
    metadata.insert("id".to_string(), identifier);
    metadata.insert("graphics".to_string(), media);
    assert!(serde_json::from_value::<ResearchActivityMetadata>(serde_json::Value::Object(metadata.clone())).is_ok());
}
#[test]
fn test_alloc_feature_serialization() {
    let id = TypedIdentifier::init()
        .value("swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2")
        .scheme("SWHID")
        .build();
    let json = serde_json::to_string(&id).expect("json");
    let yaml = serde_norway::to_string(&id).expect("yaml");
    assert!(json.contains("swh:1:cnt"));
    assert!(yaml.contains("swh:1:cnt"));
}
#[test]
fn test_aspect_model_markdown_includes_nested_fields() {
    let details = ModelDetails::init()
        .name("model".to_string())
        .limit(LimitDetails {
            context: 128_000,
            input: Some(120_000),
            output: Some(8_000),
        })
        .cost(CostDetails {
            cache_write: Some(1.5),
            reasoning: Some(2.5),
            ..CostDetails::default()
        })
        .weights(Weights(vec![Weight {
            label: "Q4".to_string(),
            url: "https://example.com/model.gguf".to_string(),
            is_open: Some(true),
            quantization: None,
            size: Some(42),
        }]))
        .build();
    let markdown = Data::Model(Box::new(Model::LLM(details))).to_markdown();
    for expected in [
        "- Limits",
        "- Context: 128000",
        "- Input: 120000",
        "- Output: 8000",
        "- Cache Write: 1.5",
        "- Reasoning: 2.5",
        "- Weights",
        "- Label: Q4",
        "- Open: true",
        "- Size: 42",
    ] {
        assert!(markdown.contains(expected), "missing '{expected}' in:\n{markdown}");
    }
}
#[test]
fn test_aspect_to_markdown() {
    let aspect = AspectFramework::init()
        .data(vec![
            Data::Real(
                DataDescription::init()
                    .availability(Availability::Unrestricted)
                    .license("CC-BY-4.0".to_string())
                    .modality(Modality::Text)
                    .quality(Quality::Gold)
                    .build(),
            ),
            Data::Model(Box::new(Model::LLM(
                ModelDetails::init()
                    .family("llama".to_string())
                    .name("llama-3.1".to_string())
                    .version(SemanticVersion::from("2.1"))
                    .parameters(8)
                    .build(),
            ))),
        ])
        .portability(SoftwarePortability::Containerized)
        .motivity(Motivity::Adaptive)
        .autonomy(Autonomy::MachineAssisted)
        .maturity(TechnologyReadinessLevel::Prototype)
        .build();
    let expected = r#"## ASPECT
- Portability: Containerized
- Maturity: Prototype
- Autonomy: Machine-assisted
- Motivity: Adaptive

### Data
- Real
  - Availability: Unrestricted
  - License: CC-BY-4.0
  - Modality: text
  - Quality: Gold
- Model
  - Kind: LLM
  - Family: llama
  - Name: llama-3.1
  - Parameters: 8B
  - Version: 2.1.0"#;
    assert_eq!(aspect.to_markdown().replace("\r\n", "\n").trim_end(), expected);
    let rad = ResearchActivity::init().aspect(aspect).build();
    let parsed = ResearchActivity::from_markdown(&rad.to_markdown()).expect("ASPECT Markdown should parse");
    assert_eq!(
        serde_json::to_value(parsed).expect("parsed RAD should serialize"),
        serde_json::to_value(rad).expect("source RAD should serialize")
    );
}
#[test]
fn test_canonical_markdown_rejects_invalid_contact_shape() {
    let canonical = ResearchActivity::default().to_markdown();
    let cases = [
        (canonical.replace("- URL: https://example.com\n", ""), "contact is missing 'url'"),
        (
            canonical.replace("- Organization:", "- Unknown:\n- Organization:"),
            "Unknown Markdown RAD contact field 'unknown'",
        ),
        (
            canonical.replace("- URL: https://example.com", "- URL: https://example.com\n- URL: https://example.org"),
            "Duplicate Markdown RAD contact field 'url'",
        ),
    ];
    for (markdown, expected) in cases {
        let error = ResearchActivity::from_markdown(&markdown).expect_err("invalid canonical contact should fail");
        assert!(error.to_string().contains(expected), "missing '{expected}' in: {error}");
    }
}
#[test]
fn test_controlled_subject_and_rights() {
    let subject = ControlledSubject::init()
        .subject("Climate change")
        .subject_scheme("TestScheme")
        .scheme_uri("https://example.org/scheme")
        .build();
    assert!(subject.validate().is_ok());
    let rights = StructuredRights::init()
        .rights_identifier("CC-BY-4.0")
        .rights_uri("https://creativecommons.org/licenses/by/4.0/")
        .access(RightsAccess::Open)
        .build();
    assert!(rights.validate().is_ok());
    assert_eq!(serde_json::to_value(&rights).expect("serialize")["access"], "open");
    let invalid = StructuredRights::init().rights_uri("not a url").build();
    assert!(invalid.validate().is_err());
    let invalid_license = StructuredRights::init().rights_identifier("").build();
    assert!(invalid_license.validate().is_err());
    assert!(serde_json::from_str::<StructuredRights>(r#"{"access":"unexpected"}"#).is_err());
}
#[test]
fn test_dated_roles_require_timestamp_offset() {
    let agent = AgentIdentifier::init().value("https://orcid.org/0000-0001-2345-6789").build();
    let role = DatedAgentRole::init()
        .agent(agent)
        .start_date("2024-01-15T10:00:00Z")
        .end_date("2025-01-15T10:00:00Z")
        .build();
    assert!(role.validate().is_ok());
    let bad = DatedAgentRole::init()
        .agent(AgentIdentifier::init().value("x").build())
        .start_date("2024-01-15")
        .build();
    assert!(bad.validate().is_err());
    let org = DatedOrganizationRole::init()
        .organization(TypedIdentifier::init().value("https://ror.org/04xm1d337").build())
        .role("LeadResearchOrganisation")
        .start_date("2024-01-15T10:00:00Z")
        .build();
    assert!(org.validate().is_ok());
}
#[test]
fn test_deserialize_metadata_with_empty_cpu_and_gpu() {
    let json = r#"{
        "identifier": "empty-resources",
        "archive": false,
        "draft": true,
        "status": "active",
        "keywords": [],
        "technology": [],
        "resources": [
            { "CPU": {} },
            { "GPU": {} }
        ]
    }"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).expect("should deserialize empty CPU and GPU");
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 2);
    assert!(matches!(
        &resources[0],
        Resource::CPU {
            architecture: None,
            cores: None,
            count: Some(1),
            memory: None,
            required: None,
            threads: None,
            vendor: None,
        }
    ));
    assert!(matches!(
        &resources[1],
        Resource::GPU {
            architecture: None,
            compute_capability: None,
            count: Some(1),
            memory: None,
            required: None,
            vendor: None,
            ..
        }
    ));
}
#[test]
fn test_deserialize_metadata_with_gpu_resources() {
    let json = r#"{
        "identifier": "ml-training",
        "archive": false,
        "draft": false,
        "status": "active",
        "keywords": [],
        "technology": [],
        "resources": [
            {
                "GPU": {
                    "architecture": "Hopper",
                    "compute_capability": 9.0,
                    "count": 8,
                    "memory": 80,
                    "vendor": "NVIDIA"
                }
            }
        ]
    }"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).expect("should deserialize GPU resources");
    assert_eq!(meta.identifier, "ml-training");
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 1);
    assert!(matches!(
        &resources[0],
        Resource::GPU {
            architecture: Some(GpuArchitecture::Hopper),
            compute_capability: Some(cc),
            count: Some(8),
            memory: Some(Memory { amount: 80.0, unit: MemoryUnit::GB }),
            required: None,
            vendor: Some(Vendor::Nvidia),
            ..
        } if (*cc - 9.0_f32).abs() < f32::EPSILON
    ));
}
#[test]
fn test_deserialize_metadata_with_mixed_resources() {
    let json = r#"{
        "identifier": "hybrid-compute",
        "archive": false,
        "draft": true,
        "status": "active",
        "keywords": [],
        "technology": [],
        "resources": [
            {
                "CPU": {
                    "architecture": "x86_64",
                    "cores": 64,
                    "memory": 512,
                    "threads": 128,
                    "vendor": "Intel"
                }
            },
            "TPU",
            "FPGA",
            {
                "GPU": {
                    "count": 2,
                    "vendor": "AMD"
                }
            }
        ]
    }"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).expect("should deserialize mixed resources");
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 4);
    assert!(matches!(
        &resources[0],
        Resource::CPU {
            cores: Some(64),
            required: None,
            threads: Some(128),
            vendor: Some(Vendor::Intel),
            ..
        }
    ));
    assert!(matches!(resources[1], Resource::TPU { .. }));
    assert!(matches!(resources[2], Resource::FPGA { .. }));
    assert!(matches!(
        &resources[3],
        Resource::GPU {
            architecture: None,
            compute_capability: None,
            count: Some(2),
            required: None,
            vendor: Some(Vendor::AMD),
            ..
        }
    ));
}
#[test]
fn test_deserialize_metadata_with_partial_gpu_fields() {
    let json = r#"{
        "identifier": "sparse-gpu",
        "archive": false,
        "draft": true,
        "status": "active",
        "keywords": [],
        "technology": [],
        "resources": [
            {
                "GPU": {
                    "memory": 24
                }
            }
        ]
    }"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).expect("should deserialize partial GPU fields");
    let resources = meta.resources.expect("resources should be present");
    assert!(matches!(
        &resources[0],
        Resource::GPU {
            architecture: None,
            compute_capability: None,
            count: Some(1),
            memory: Some(Memory {
                amount: 24.0,
                unit: MemoryUnit::GB
            }),
            required: None,
            vendor: None,
            ..
        }
    ));
}
#[test]
fn test_deserialize_metadata_with_string_cpu_and_gpu() {
    let json = r#"{
        "identifier": "string-resources",
        "archive": false,
        "draft": true,
        "status": "active",
        "keywords": [],
        "technology": [],
        "resources": ["CPU", "GPU"]
    }"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).expect("should deserialize string CPU and GPU");
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 2);
    assert!(matches!(
        &resources[0],
        Resource::CPU {
            architecture: None,
            cores: None,
            count: Some(1),
            memory: None,
            required: None,
            threads: None,
            vendor: None,
        }
    ));
    assert!(matches!(
        &resources[1],
        Resource::GPU {
            architecture: None,
            compute_capability: None,
            count: Some(1),
            memory: None,
            required: None,
            vendor: None,
            ..
        }
    ));
}
#[test]
fn test_deserialize_metadata_with_unit_resources() {
    let json = r#"{
        "identifier": "exotic-compute",
        "archive": false,
        "draft": true,
        "status": "active",
        "keywords": [],
        "technology": [],
        "resources": ["Quantum", "Neuromorphic", "NPU", "ASIC", "DSP", "unique-hardware"]
    }"#;
    let meta: ResearchActivityMetadata = serde_json::from_str(json).expect("should deserialize unit resource variants");
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 6);
    assert!(matches!(resources[0], Resource::Quantum { .. }));
    assert!(matches!(resources[1], Resource::Neuromorphic { .. }));
    assert!(matches!(resources[2], Resource::NPU { .. }));
    assert!(matches!(resources[3], Resource::ASIC { .. }));
    assert!(matches!(resources[4], Resource::DSP { .. }));
    assert!(matches!(&resources[5], Resource::Other(s) if s == "unique-hardware"));
}
#[test]
fn test_is_attribute_areas() {
    let valid = ["x".repeat(10), "x".repeat(40)];
    let invalid = ["x".repeat(41), "x".repeat(100)];
    for x in valid.iter() {
        assert!(is_attribute_areas(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_attribute_areas(core::slice::from_ref(x)).is_err());
    }
}
#[test]
fn test_is_attribute_capabilities() {
    let valid = ["x".repeat(10), "x".repeat(300)];
    let invalid = ["x".repeat(301), "x".repeat(400)];
    for x in valid.iter() {
        assert!(is_attribute_capabilities(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_attribute_capabilities(core::slice::from_ref(x)).is_err());
    }
}
#[test]
fn test_is_attribute_impact() {
    let valid = ["X".repeat(10), "X".repeat(150)];
    let invalid = ["X".repeat(151), "X".repeat(500)];
    for x in valid.iter() {
        assert!(is_attribute_impact(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_attribute_impact(core::slice::from_ref(x)).is_err());
    }
    assert!(is_attribute_impact(&[
        "This is an impact statement with no period".to_string(),
        "This is another impact statement with no period".to_string(),
        "This is a third impact statement with no period".to_string(),
    ])
    .is_ok());
    assert!(is_attribute_impact(&[
        "This is an impact statement with no period".to_string(),
        "This is another impact statement with no period".to_string(),
        "This is an impact statement with a period.".to_string(),
    ])
    .is_err());
    assert!(is_attribute_impact(&["starts with lowercase impact statement".to_string()]).is_err());
    assert!(is_attribute_impact(&[
        "Starts with uppercase impact statement".to_string(),
        "Another uppercase impact statement".to_string(),
    ])
    .is_ok());
}
#[test]
fn test_is_attribute_publication_identifier() {
    assert!(matches!(
        PublicationIdentifierType::from("10.1000/182"),
        PublicationIdentifierType::Doi(_)
    ));
    assert!(matches!(
        PublicationIdentifierType::from("arXiv:2106.09685"),
        PublicationIdentifierType::Arxiv(_)
    ));
    assert!(matches!(
        PublicationIdentifierType::from("2106.09685"),
        PublicationIdentifierType::Unknown
    ));
    let valid = ["10.1000/182".to_string(), "10.97812345/99990".to_string(), "arXiv:2106.09685".to_string()];
    assert!(is_attribute_publication_identifier_list(&valid).is_ok());
    let invalid = ["https://not.doi.org/10.1000/182".to_string()];
    assert!(is_attribute_publication_identifier_list(&invalid).is_err());
}
#[test]
fn test_malformed_fixture_has_multiple_eserde_errors() {
    let path = fixtures_dir().join("../../tests/fixtures/malformed-rad.json");
    let content = std::fs::read_to_string(&path).expect("fixture should exist");
    let result = eserde::json::from_str::<ResearchActivity>(&content);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.len() >= 2, "expected multiple errors, got {}: {:?}", errors.len(), errors);
}
#[test]
fn test_markdown_body_values_round_trip_structural_text() {
    let data = ResearchActivity {
        title: "Title with\n## Contact".to_string(),
        subtitle: Some("Subtitle &amp;\n## Areas".to_string()),
        sections: Sections {
            mission: "Purpose with\n## Contact".to_string(),
            approach: vec!["Approach with\n## Areas &amp;".to_string()],
            ..Sections::default()
        },
        contact: ContactPoint {
            organization: "Organization with\n## Mission".to_string(),
            ..ContactPoint::default()
        },
        ..ResearchActivity::default()
    };
    let markdown = data.to_markdown();
    assert!(markdown.contains("&#10;## Contact"));
    let parsed = ResearchActivity::from_markdown(&markdown).expect("encoded structural text should parse");
    assert_eq!(
        serde_json::to_value(parsed).expect("parsed RAD should serialize"),
        serde_json::to_value(data).expect("source RAD should serialize")
    );
}
#[test]
fn test_markdown_normalizes_empty_required_frontmatter_lists() {
    let canonical = ResearchActivity::default().to_markdown();
    for replacement in ["", " null"] {
        let markdown = canonical
            .replace("  keywords: []", &format!("  keywords:{replacement}"))
            .replace("  technology: []", &format!("  technology:{replacement}"));
        let parsed = ResearchActivity::from_markdown(&markdown).expect("empty required frontmatter lists should parse");
        assert!(parsed.meta.keywords.is_empty());
        assert!(parsed.meta.technology.is_empty());
        let serialized = parsed.to_markdown();
        assert!(serialized.contains("  keywords: []"));
        assert!(serialized.contains("  technology: []"));
    }
}
#[test]
fn test_markdown_recognizes_quoted_schema_discriminator() {
    let markdown = ResearchActivity::default()
        .to_markdown()
        .replace("schema: acorn/research-activity", "schema: 'acorn/research-activity'");
    assert!(ResearchActivity::is_markdown(markdown.as_str()));
    assert!(ResearchActivity::from_markdown(&markdown).is_ok());
}
#[test]
fn test_markdown_rejects_malformed_optional_section() {
    let markdown = ResearchActivity::default()
        .to_markdown()
        .replace("## Contact", "## Achievement\nnot a list\n\n## Contact");
    let error = ResearchActivity::from_markdown(&markdown).expect_err("malformed optional section should fail");
    assert!(error.to_string().contains("section 'Achievement' must contain only '- ' list items"));
}
#[test]
fn test_markdown_rejects_non_list_required_frontmatter_value() {
    let markdown = ResearchActivity::default()
        .to_markdown()
        .replace("  technology: []", "  technology: Rust");
    let error = ResearchActivity::from_markdown(&markdown).expect_err("non-list technology should fail");
    assert!(error.contains("invalid type: string \"Rust\", expected a sequence"));
}
#[test]
fn test_markdown_rejects_undiscriminated_format() {
    let canonical = ResearchActivity::default().to_markdown();
    let body = &canonical[canonical.find("# ").expect("canonical Markdown should contain a title")..];
    let body = body
        .replace(
            "- Job Title: Researcher\n- Given Name: First\n- Family Name: Last",
            "- Role: Researcher\n- Name: First Last",
        )
        .replace("- Email:", "- email:")
        .replace("\n- URL: https://example.com\n- Organization: Some Organization", "");
    let markdown = format!(
        "---\nclassification: UNCLASSIFIED\narchive: false\ndraft: true\nstatus: active\nidentifier: some-research-project\nkeywords: []\ntechnology: []\n---\n{body}"
    );
    assert!(!ResearchActivity::is_markdown(markdown.as_str()));
    let error = ResearchActivity::from_markdown(&markdown).expect_err("Markdown without a schema discriminator should fail");
    assert!(error.to_string().contains("Failed to decode Markdown RAD frontmatter"));
}
#[test]
fn test_markdown_rejects_unknown_section() {
    let markdown = ResearchActivity::default()
        .to_markdown()
        .replace("## Contact", "## Unknown\nvalue\n\n## Contact");
    let error = ResearchActivity::from_markdown(&markdown).expect_err("unknown section should fail");
    assert!(error.to_string().contains("Unknown Markdown RAD section '## Unknown'"));
}
#[test]
fn test_metadata() {
    const DEFAULT_HREF: &str = "00.png";
    const DEFAULT_CAPTION: &str = "";
    let meta = ResearchActivityMetadata::init().identifier("test-data".to_string()).build();
    assert_eq!(meta.identifier, "test-data".to_string());
    assert_eq!(meta.first_image_content_url(), DEFAULT_HREF);
    let href = "abc.png";
    let caption = "hello world";
    let graphics = vec![MediaObject::Image(
        ImageObject::init().caption(caption.to_owned()).content_url(href.to_owned()).build(),
    )];
    let meta = ResearchActivityMetadata::init()
        .identifier("test-data".to_string())
        .media(graphics)
        .build();
    assert_eq!(meta.clone().first_image_content_url(), href);
    assert_eq!(meta.first_image_caption(), caption);
    let meta = ResearchActivityMetadata::init().identifier("test-data".to_string()).media(vec![]).build();
    assert_eq!(meta.clone().first_image_content_url(), DEFAULT_HREF);
    assert_eq!(meta.first_image_caption(), DEFAULT_CAPTION);
    let graphics = vec![MediaObject::Image(ImageObject::init().caption("".to_owned()).build())];
    let meta = ResearchActivityMetadata::init()
        .identifier("test-data".to_string())
        .media(graphics)
        .build();
    assert_eq!(meta.clone().first_image_content_url(), DEFAULT_HREF);
    assert_eq!(meta.first_image_caption(), DEFAULT_CAPTION);
}
#[test]
fn test_metadata_resources_roundtrip() {
    let meta = ResearchActivityMetadata::init()
        .identifier("roundtrip-test".to_string())
        .resources(vec![Resource::GPU {
            architecture: None,
            backend: None,
            compute_capability: None,
            count: Some(1),
            memory: Some(Memory::gb(24)),
            name: None,
            required: None,
            vendor: Some(Vendor::Intel),
        }])
        .build();
    let json = serde_json::to_string(&meta).expect("serialization should succeed");
    let parsed: ResearchActivityMetadata = serde_json::from_str(&json).expect("deserialization should succeed");
    let resources = parsed.resources.expect("resources should survive roundtrip");
    assert_eq!(resources.len(), 1);
    assert!(matches!(
        &resources[0],
        Resource::GPU {
            count: Some(1),
            memory: Some(Memory {
                amount: 24.0,
                unit: MemoryUnit::GB
            }),
            required: None,
            vendor: Some(Vendor::Intel),
            ..
        }
    ));
}
#[test]
fn test_metadata_validates_handles() {
    let valid = ResearchActivityMetadata::init()
        .handle(vec!["10013/epic.10033".to_string(), "https://hdl.handle.net/20.5000/example".to_string()])
        .build();
    assert!(valid.validate().is_ok());
    let invalid = ResearchActivityMetadata::init().handle(vec!["not-a-handle".to_string()]).build();
    assert!(invalid.validate().is_err());
    assert_eq!(
        ResearchActivityMetadataContext::default().handle,
        "https://codemeta.github.io/terms/identifier"
    );
}
#[test]
fn test_metadata_validates_publication_identifiers() {
    let valid = ResearchActivityMetadata::init()
        .doi(vec!["10.1000/xyz123".to_string(), "arXiv:2106.09685v2".to_string()])
        .build();
    assert!(valid.validate().is_ok());
    let invalid = ResearchActivityMetadata::init().doi(vec!["2106.09685".to_string()]).build();
    assert!(invalid.validate().is_err());
}
#[test]
fn test_metadata_with_minimal_resources() {
    let meta = ResearchActivityMetadata::init()
        .identifier("quantum-project".to_string())
        .resources(vec![
            Resource::Quantum {
                count: None,
                model: None,
                paradigm: None,
                required: None,
                qubits: None,
                topology: None,
                vendor: None,
            },
            Resource::FPGA {
                architecture: None,
                count: None,
                logic_elements: None,
                memory: None,
                required: None,
                vendor: None,
            },
        ])
        .build();
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 2);
    assert!(matches!(resources[0], Resource::Quantum { .. }));
    assert!(matches!(resources[1], Resource::FPGA { .. }));
}
#[test]
fn test_metadata_with_resources() {
    let meta = ResearchActivityMetadata::init()
        .identifier("gpu-project".to_string())
        .resources(vec![
            Resource::GPU {
                architecture: Some(GpuArchitecture::Ampere),
                backend: None,
                compute_capability: Some(8.0),
                count: Some(4),
                memory: Some(Memory::gb(80)),
                name: None,
                required: None,
                vendor: Some(Vendor::Nvidia),
            },
            Resource::CPU {
                architecture: Some(CpuArchitecture::X86_64),
                cores: Some(32),
                count: None,
                memory: Some(Memory::gb(256)),
                required: None,
                threads: Some(64),
                vendor: Some(Vendor::AMD),
            },
        ])
        .build();
    assert_eq!(meta.identifier, "gpu-project");
    let resources = meta.resources.expect("resources should be present");
    assert_eq!(resources.len(), 2);
    assert!(matches!(
        &resources[0],
        Resource::GPU {
            vendor: Some(Vendor::Nvidia),
            ..
        }
    ));
    assert!(matches!(
        &resources[1],
        Resource::CPU {
            vendor: Some(Vendor::AMD),
            ..
        }
    ));
}
#[test]
fn test_metadata_without_resources() {
    let meta = ResearchActivityMetadata::init().identifier("basic-project".to_string()).build();
    assert!(meta.resources.is_none());
}
#[test]
fn test_one_or_many_license_preserves_wire_shape() {
    let license = OneOrMany::Many(vec!["Apache-2.0".to_string(), "MIT".to_string()]);
    assert_eq!(serde_json::to_string(&license).unwrap(), r#"["Apache-2.0","MIT"]"#);
    assert_eq!(
        serde_json::from_str::<OneOrMany<String>>(r#""MIT""#).unwrap(),
        OneOrMany::One("MIT".to_string())
    );
}
#[test]
fn test_research_activity_default() {
    let data = ResearchActivity::default();
    let actual = data.to_markdown().replace("\r\n", "\n");
    assert!(actual.starts_with("---\n"));
    assert!(actual.contains("schema: acorn/research-activity"));
    assert!(actual.contains("meta:\n  archive: false"));
    assert!(actual.contains("# Research Activity Title"));
    assert!(actual.contains("- Given Name: First"));
    assert!(actual.contains("- Family Name: Last"));
    assert!(actual.contains("- URL: https://example.com"));
    let parsed = ResearchActivity::from_markdown(&actual).expect("canonical Markdown should parse");
    assert_eq!(
        serde_json::to_value(parsed).expect("parsed RAD should serialize"),
        serde_json::to_value(data).expect("source RAD should serialize")
    );
}
#[test]
fn test_research_activity_into_cff() {
    let rad = ResearchActivity::default();
    let cff: Cff = rad.into();
    assert_eq!(cff.title, "Research Activity Title");
    assert_eq!(cff.abstract_text, Some("Purpose of the research".to_string()));
    assert_eq!(cff.cff_version, "1.2.0");
    assert!(matches!(&cff.authors[0], Agent::Person(_)));
    assert!(matches!(&cff.contact.as_ref().unwrap()[0], Agent::Person(_)));
    assert_eq!(cff.doi, None);
    assert_eq!(cff.keywords, None);
    let rad_with_doi = ResearchActivity::init()
        .meta(ResearchActivityMetadata::init().doi(vec!["10.1000/xyz123".to_string()]).build())
        .build();
    let cff_with_doi: Cff = rad_with_doi.into();
    assert_eq!(cff_with_doi.doi, Some("10.1000/xyz123".to_string()));

    let rad_with_multiple_dois = ResearchActivity::init()
        .meta(
            ResearchActivityMetadata::init()
                .doi(vec!["10.1000/xyz123".to_string(), "10.1000/xyz456".to_string()])
                .build(),
        )
        .build();
    let cff_with_multiple_dois: Cff = rad_with_multiple_dois.into();
    assert_eq!(cff_with_multiple_dois.doi, None);
    assert_eq!(cff_with_multiple_dois.identifiers.as_ref().map(Vec::len), Some(2));
    let identifier_values = cff_with_multiple_dois
        .identifiers
        .as_ref()
        .expect("identifiers should exist")
        .iter()
        .map(|value| value.value.clone())
        .collect::<Vec<_>>();
    let identifier_types = cff_with_multiple_dois
        .identifiers
        .as_ref()
        .expect("identifiers should exist")
        .iter()
        .map(|value| value.kind.clone())
        .collect::<Vec<_>>();
    assert_eq!(identifier_values, vec!["10.1000/xyz123".to_string(), "10.1000/xyz456".to_string()]);
    assert_eq!(identifier_types, vec![IdentifierType::Doi, IdentifierType::Doi]);
    let rad_with_arxiv = ResearchActivity::init()
        .meta(
            ResearchActivityMetadata::init()
                .doi(vec!["10.1000/xyz123".to_string(), "arXiv:2106.09685".to_string()])
                .build(),
        )
        .build();
    let cff_with_arxiv: Cff = rad_with_arxiv.into();
    assert_eq!(cff_with_arxiv.doi, None);
    let identifiers = cff_with_arxiv.identifiers.expect("identifiers should exist");
    assert_eq!(identifiers.first().map(|identifier| identifier.kind.clone()), Some(IdentifierType::Doi));
    let arxiv = identifiers.get(1).cloned();
    assert_eq!(arxiv.as_ref().map(|identifier| identifier.kind.clone()), Some(IdentifierType::Other));
    assert_eq!(arxiv.map(|identifier| identifier.value), Some("arXiv:2106.09685".to_string()));
    let swhid = "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a";
    let rad_with_swhid = ResearchActivity::init()
        .meta(ResearchActivityMetadata::init().swhid(vec![swhid.to_string()]).build())
        .build();
    let cff_with_swhid: Cff = rad_with_swhid.into();
    let identifier = cff_with_swhid
        .identifiers
        .and_then(|identifiers| identifiers.into_iter().next())
        .expect("SWHID should be exported as a CFF identifier");
    assert_eq!(identifier.kind, IdentifierType::Swh);
    assert_eq!(identifier.value, swhid);
    let rad_with_handle = ResearchActivity::init()
        .meta(ResearchActivityMetadata::init().handle(vec!["hdl:10013/epic.10033".to_string()]).build())
        .build();
    let cff_with_handle: Cff = rad_with_handle.into();
    let identifier = cff_with_handle
        .identifiers
        .and_then(|identifiers| identifiers.into_iter().next())
        .expect("Handle should be exported as a CFF identifier");
    assert_eq!(identifier.kind, IdentifierType::Other);
    assert_eq!(identifier.value, "10013/epic.10033");
    assert_eq!(identifier.description.as_deref(), Some("Handle identifier"));
}
#[test]
fn test_research_activity_metadata_without_outputs_remains_compatible() {
    let serialized = serde_json::to_string(&ResearchActivityMetadata::default()).expect("default metadata should serialize");
    let parsed: ResearchActivityMetadata = serde_json::from_str(&serialized).expect("metadata without outputs should deserialize");
    assert!(parsed.outputs.is_none());
}
#[test]
fn test_research_output_rejects_invalid_identifiers_and_date() {
    let output = ResearchOutput::init()
        .doi("not-a-doi".to_string())
        .title("Invalid structured research output".to_string())
        .publication_year(99999)
        .dates(vec![TypedActivityDate::init()
            .date_type(ActivityDateType::Issued)
            .date("January 31".to_string())
            .build()])
        .contributors(vec![Contributor::init()
            .name("Example Researcher")
            .orcid("not-an-orcid".to_string())
            .affiliations(vec![Affiliation::init()
                .name("Example Organization")
                .ror("not-a-ror".to_string())
                .build()])
            .build()])
        .build();
    assert!(output.validate().is_err());
}
#[test]
fn test_research_output_roundtrip_and_validation() {
    let output = ResearchOutput::init()
        .identifier("https://openalex.org/W2741809807".to_string())
        .doi("10.1038/nature12373".to_string())
        .title("Structured research output".to_string())
        .kind("article".to_string())
        .publication_year(2024)
        .dates(vec![TypedActivityDate::init()
            .date_type(ActivityDateType::Issued)
            .date("2024-01-31T00:00:00Z".to_string())
            .build()])
        .contributors(vec![Contributor::init()
            .name("Example Researcher")
            .orcid("0000-0002-1825-0097".to_string())
            .affiliations(vec![Affiliation::init()
                .name("Oak Ridge National Laboratory")
                .ror("https://ror.org/01qz5mb56".to_string())
                .build()])
            .build()])
        .funding(vec![Funding::init()
            .name("Example Funder")
            .awards(vec![Award::init().identifier("AWARD-1").build()])
            .build()])
        .access(Access::init().open(true).status("gold").build())
        .build();
    let metadata = ResearchActivityMetadata::init().outputs(vec![output]).build();
    let serialized = serde_json::to_string(&metadata).expect("research output metadata should serialize");
    let parsed: ResearchActivityMetadata = serde_json::from_str(&serialized).expect("research output metadata should deserialize");
    assert!(parsed.validate().is_ok());
    assert_eq!(parsed.outputs.expect("outputs should be retained").len(), 1);
}
#[test]
fn test_to_prose() {
    let default = ResearchActivity::default();
    let prose = default.to_prose();
    assert!(prose.starts_with("Research Activity Title"));
    assert!(prose.contains("## Mission"));
    assert!(prose.contains("## Challenge"));
    assert!(prose.contains("## Approach"));
    assert!(prose.contains("## Impact"));
    assert!(!prose.contains("---"));
    assert!(!prose.contains("classification"));
    assert!(!prose.contains("@"));
    insta::assert_snapshot!("to_prose_default", prose);
    let with_subtitle = ResearchActivity::init()
        .title("Test Title".to_string())
        .subtitle("A subtitle".to_string())
        .build();
    let prose = with_subtitle.to_prose();
    assert!(prose.contains("Test Title"));
    assert!(prose.contains("A subtitle"));
    insta::assert_snapshot!("to_prose_with_subtitle", prose);
    let with_websites = ResearchActivity::init()
        .meta(
            ResearchActivityMetadata::init()
                .websites(vec![Website::at("https://example.com").description("Example").build()])
                .build(),
        )
        .build();
    let prose = with_websites.to_prose();
    assert!(prose.contains("example.com"));
    insta::assert_snapshot!("to_prose_with_websites", prose);
}
#[test]
fn test_typed_activity_date_requires_timestamp() {
    let date = TypedActivityDate::init()
        .date_type(ActivityDateType::StartDate)
        .date("2024-01-15T10:00:00Z")
        .build();
    assert!(date.validate().is_ok());
    assert_eq!(serde_json::to_value(&date).expect("serialize")["dateType"], "StartDate");
    let invalid = TypedActivityDate::init()
        .date_type(ActivityDateType::StartDate)
        .date("2024-01-15")
        .build();
    assert!(invalid.validate().is_err());
    assert!(serde_json::from_str::<TypedActivityDate>(r#"{"dateType":"Unexpected","date":"2024-01-15T10:00:00Z"}"#).is_err());
}
#[test]
fn test_typed_identifier_serializes_and_validates() {
    let id = TypedIdentifier::init()
        .value("10.5072/example")
        .scheme("DOI")
        .scheme_uri("https://doi.org")
        .build();
    assert!(id.validate().is_ok());
    let json = serde_json::to_string(&id).expect("serialize");
    assert!(json.contains("10.5072/example"));
    let parsed: TypedIdentifier = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.value, "10.5072/example");
    let invalid = TypedIdentifier::init().value("").build();
    assert!(invalid.validate().is_err());
    let bad = TypedIdentifier::init().value("10.5072/x").scheme_uri("not a url").build();
    assert!(bad.validate().is_err());
}
#[test]
fn test_typed_related_resource_with_other() {
    let resource = TypedRelatedResource::init()
        .identifier(TypedIdentifier::init().value("10.5072/related").scheme("DOI").build())
        .relation_type("Other")
        .relation_type_information("Custom domain relation")
        .resource_type_general("Dataset")
        .category(RelatedResourceCategory::Related)
        .build();
    assert!(resource.validate().is_ok());
    let json = serde_json::to_string(&resource).expect("serialize");
    assert!(json.contains("Other"));
    assert!(json.contains("Custom domain relation"));
    assert!(json.contains(r#""category":"related""#));
    assert!(serde_json::from_str::<TypedRelatedResource>(
        r#"{"identifier":{"value":"10.5072/related"},"relationType":"Other","category":"unexpected"}"#
    )
    .is_err());
}
