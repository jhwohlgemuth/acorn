#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::io::InputOutput;
use crate::prelude::PathBuf;
use crate::schema::agent::{Model, ModelDetails};
use crate::schema::hardware::{CpuArchitecture, GpuArchitecture, Resource, Vendor};
use crate::schema::research_activity::aspect::{Autonomy, Availability, Data, DataDescription, Modality, Motivity, Quality, SoftwarePortability};
use crate::schema::research_activity::*;
use crate::schema::standard::cff::{Agent, Cff, IdentifierType};
use crate::schema::*;
use crate::util::SemanticVersion;
use pretty_assertions::assert_eq;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests/fixtures")
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
fn test_research_activity_default() {
    let expected = r#"---
classification: UNCLASSIFIED
archive: false
draft: true
status: active
identifier: some-research-project
keywords: []
technology: []
---
# Research Activity Title

## Mission
Purpose of the research

## Challenge
Reason for the research

## Approach
- List of actions taken to perform the research

## Impact
- List of tangible proof that validates the research approach

## Focus
Focus of the research

## Areas
- Areas of research

## Contact
- Role: Researcher
- Name: First Last
- email: [first_last@example.com](mailto:first_last@example.com)
- Telephone: 123-456-7890
"#;
    let data = ResearchActivity::default();
    let actual = data.to_markdown().replace("\r\n", "\n");
    assert_eq!(actual, expected);
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
}
#[test]
fn test_research_activity_format() {
    //
    // with changes
    //
    let path = Some(fixtures_dir().join("data/format/changes"));
    let pre = ResearchActivity::read(fixtures_dir().join("data/format/changes/index.json")).unwrap();
    assert!(pre.meta.media.is_none());
    assert!(pre.contact.affiliation.is_none());
    let post = pre.format_with(path.clone());
    assert_eq!(post.meta.media.unwrap()[0].clone().content_url(), Some("42.png".to_string()));
    assert_eq!(post.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    //
    // with unresolved changes
    //
    let pre = ResearchActivity::read(fixtures_dir().join("data/format/unresolved_changes/index.json")).unwrap();
    assert!(pre.meta.media.is_some());
    assert_eq!(pre.contact.affiliation, Some("Not an actual affiliation".to_string()));
    let post = pre.format_with(None);
    assert!(post.clone().meta.media.unwrap()[0].clone().content_url().is_none());
    assert_eq!(post.clone().meta.first_image().unwrap().description(), "".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // with more unresolved changes
    //
    let mut pre = post.clone().copy();
    pre.contact.organization = "Not an actual organization".to_string();
    pre.contact.affiliation = None;
    let post = pre.format_with(None);
    assert_eq!(post.contact.organization, "".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // with ORNL as organization and no affiliation
    //
    let mut pre = post.clone().copy();
    pre.contact.organization = "Oak Ridge National Laboratory".to_string();
    pre.contact.affiliation = None;
    let post = pre.format_with(None);
    assert_eq!(post.contact.organization, "Oak Ridge National Laboratory".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // without changes
    //
    let path = Some(fixtures_dir().join("data/format/no_changes"));
    let pre = ResearchActivity::read(fixtures_dir().join("data/format/no_changes/index.json")).unwrap();
    assert_eq!(pre.clone().meta.media.unwrap()[0].clone().content_url(), Some("00.png".to_string()));
    assert_eq!(pre.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    let post = pre.clone().format_with(path);
    assert_eq!(post.meta.media.unwrap()[0].clone().content_url(), Some("42.png".to_string()));
    assert_eq!(post.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
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
fn test_is_attribute_doi() {
    let valid = ["10.1000/182".to_string(), "10.97812345/99990".to_string()];
    assert!(is_attribute_doi_list(&valid).is_ok());
    let invalid = ["https://not.doi.org/10.1000/182".to_string()];
    assert!(is_attribute_doi_list(&invalid).is_err());
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
                .websites(vec![Website {
                    description: "Example".to_string(),
                    url: "https://example.com".to_string(),
                }])
                .build(),
        )
        .build();
    let prose = with_websites.to_prose();
    assert!(prose.contains("example.com"));
    insta::assert_snapshot!("to_prose_with_websites", prose);
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
fn test_metadata_without_resources() {
    let meta = ResearchActivityMetadata::init().identifier("basic-project".to_string()).build();
    assert!(meta.resources.is_none());
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
fn test_malformed_fixture_has_multiple_eserde_errors() {
    let path = fixtures_dir().join("../../tests/fixtures/malformed-rad.json");
    let content = std::fs::read_to_string(&path).expect("fixture should exist");
    let result = eserde::json::from_str::<ResearchActivity>(&content);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.len() >= 2, "expected multiple errors, got {}: {:?}", errors.len(), errors);
}
#[test]
fn test_read_jsonc_with_comments() {
    let content = r#"{
        // This is a comment
        "title": "JSONC Project",
        "sections": {
            "mission": "Mission statement",
            "challenge": "Challenge description",
            "approach": ["Approach item"],
            "impact": ["Impact item"],
            "research": {
                "focus": "Focus area",
                "areas": ["Area 1"]
            }
        },
        "contact": {
            "title": "PI",
            "first": "Jane",
            "last": "Doe"
        }
    }"#;
    let result = crate::io::jsonc_parse_value(content);
    assert!(result.is_ok(), "JSONC parse should succeed with comments");
    let value = result.unwrap();
    assert_eq!(value["title"], "JSONC Project");
    assert_eq!(value["sections"]["mission"], "Mission statement");
}
#[test]
fn test_read_jsonc_with_trailing_commas() {
    let content = r#"{
        "title": "Trailing Commas",
        "sections": {
            "mission": "Mission",
            "challenge": "Challenge",
            "approach": ["Approach",],
            "impact": ["Impact",],
            "research": {
                "focus": "Focus",
                "areas": ["Area",],
            },
        },
        "contact": {
            "title": "PI",
            "first": "Jane",
            "last": "Doe",
        },
    }"#;
    let result = crate::io::jsonc_parse_value(content);
    assert!(result.is_ok(), "JSONC parse should succeed with trailing commas");
    let value = result.unwrap();
    assert_eq!(value["title"], "Trailing Commas");
}
#[test]
fn test_read_jsonc_rejects_single_quotes() {
    let content = r#"{'title': 'single quotes'}"#;
    let result = crate::io::jsonc_parse_value(content);
    assert!(result.is_err(), "JSONC parse should reject single quotes");
}
#[test]
fn test_read_jsonc_rejects_unquoted_keys() {
    let content = r#"{ title: 'unquoted' }"#;
    let result = crate::io::jsonc_parse_value(content);
    assert!(result.is_err(), "JSONC parse should reject unquoted keys");
}
#[test]
fn test_malformed_jsonc_reports_error() {
    let content = r#"{
        // Valid comment
        "title": 123,
        "sections": "not an object"
    }"#;
    let result = crate::io::jsonc_parse_value(content);
    assert!(result.is_ok(), "JSONC syntax should parse, but deserialization may fail");
    let value = result.unwrap();
    let rad = eserde::json::from_str::<ResearchActivity>(&serde_json::to_string(&value).unwrap());
    assert!(rad.is_err(), "JSONC with wrong types should fail schema validation");
}
