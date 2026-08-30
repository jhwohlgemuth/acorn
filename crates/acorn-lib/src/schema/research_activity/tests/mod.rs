#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::io::adapter::migrate::Migration;
use crate::io::InputOutput;
use crate::prelude::std::{read, write, PathBuf};
use crate::schema::agent::{CostDetails, LimitDetails, Model, ModelDetails, Weight, Weights};
use crate::schema::hardware::{CpuArchitecture, GpuArchitecture, Resource, Vendor};
use crate::schema::research_activity::aspect::{Autonomy, Availability, Data, DataDescription, Modality, Motivity, Quality, SoftwarePortability};
use crate::schema::research_activity::output::{Access, Affiliation, Award, Contributor, Funding};
use crate::schema::research_activity::*;
use crate::schema::standard::cff::{Agent, Cff, IdentifierType};
use crate::schema::*;
use crate::util::SemanticVersion;
use crate::validation::Validate;
use pretty_assertions::assert_eq;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
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
fn test_field_migrations_detect_deprecated_rad_paths() {
    let value = serde_json::json!({
        "meta": {"media": [{"href": "image.png"}, {"url": "video.mp4"}]},
        "contact": {"profile": "https://example.com"},
        "sections": {"outcomes": ["Outcome"]}
    });
    assert_eq!(
        ResearchActivity::field_migrations(&value),
        vec![
            ("meta.media[].href", "meta.media[].contentUrl"),
            ("meta.media[].url", "meta.media[].contentUrl"),
            ("contact.profile", "contact.url"),
            ("sections.outcomes", "sections.impact")
        ]
    );
}
#[test]
fn test_migrate_allows_strict_read() {
    let mut value = serde_json::to_value(ResearchActivity::default()).expect("default RAD should serialize");
    value["meta"]["media"] = serde_json::json!([{"caption": "Caption", "href": "image.png"}]);
    let impact = value["sections"].as_object_mut().expect("sections is an object").remove("impact");
    value["sections"]["outcomes"] = impact.expect("serialized sections contain impact");
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("legacy.json");
    write(&path, serde_json::to_vec(&value).expect("legacy RAD should serialize")).expect("legacy RAD fixture should be written");
    assert!(ResearchActivity::read(path.clone()).is_ok());
    let why = ResearchActivity::read_strict(path.clone()).expect_err("strict reads should reject deprecated fields");
    assert!(why.to_string().contains("meta.media[].href"));
    assert!(why.to_string().contains("meta.media[].contentUrl"));
    assert!(ResearchActivity::migrate(path.clone()).expect("legacy RAD should migrate"));
    assert!(ResearchActivity::read_strict(path.clone()).is_ok());
    let canonical: serde_json::Value =
        serde_json::from_slice(&read(&path).expect("canonical RAD should be read")).expect("canonical RAD should deserialize as JSON");
    assert_eq!(canonical["meta"]["media"][0]["contentUrl"], "image.png");
    assert!(canonical["meta"]["media"][0].get("href").is_none());
    assert!(canonical["sections"].get("impact").is_some());
    assert!(canonical["sections"].get("outcomes").is_none());
    assert!(!ResearchActivity::migrate(path).expect("canonical RAD should remain unchanged"));
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
fn test_metadata_validates_publication_identifiers() {
    let valid = ResearchActivityMetadata::init()
        .doi(vec!["10.1000/xyz123".to_string(), "arXiv:2106.09685v2".to_string()])
        .build();
    assert!(valid.validate().is_ok());
    let invalid = ResearchActivityMetadata::init().doi(vec!["2106.09685".to_string()]).build();
    assert!(invalid.validate().is_err());
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
fn test_markdown_rejects_unknown_section() {
    let markdown = ResearchActivity::default()
        .to_markdown()
        .replace("## Contact", "## Unknown\nvalue\n\n## Contact");
    let error = ResearchActivity::from_markdown(&markdown).expect_err("unknown section should fail");
    assert!(error.to_string().contains("Unknown Markdown RAD section '## Unknown'"));
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
                .websites(vec![Website::at("https://example.com").description("Example").build()])
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

#[test]
fn test_research_output_roundtrip_and_validation() {
    let output = ResearchOutput::init()
        .identifier("https://openalex.org/W2741809807".to_string())
        .doi("10.1038/nature12373".to_string())
        .title("Structured research output".to_string())
        .kind("article".to_string())
        .publication_date("2024-01-31".to_string())
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
fn test_one_or_many_license_preserves_wire_shape() {
    let license = OneOrMany::Many(vec!["Apache-2.0".to_string(), "MIT".to_string()]);
    assert_eq!(serde_json::to_string(&license).unwrap(), r#"["Apache-2.0","MIT"]"#);
    assert_eq!(
        serde_json::from_str::<OneOrMany<String>>(r#""MIT""#).unwrap(),
        OneOrMany::One("MIT".to_string())
    );
}

#[test]
fn test_research_output_rejects_invalid_identifiers_and_date() {
    let output = ResearchOutput::init()
        .doi("not-a-doi".to_string())
        .title("Invalid structured research output".to_string())
        .publication_date("January 31".to_string())
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
fn test_research_activity_metadata_without_outputs_remains_compatible() {
    let serialized = serde_json::to_string(&ResearchActivityMetadata::default()).expect("default metadata should serialize");
    let parsed: ResearchActivityMetadata = serde_json::from_str(&serialized).expect("metadata without outputs should deserialize");
    assert!(parsed.outputs.is_none());
}
