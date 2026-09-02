#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::schema::standard::cff::{Agent, Cff, CffType, IdentifierType, ReferenceType};
use crate::schema::validate::{PostalCode, YearValue};
use crate::schema::OneOrMany;
use acorn_core::util::{MarkdownSupport, ToProse};

const MINIMAL_YAML: &str = r#"
authors:
  - family-names: Druskat
    given-names: Stephan
cff-version: "1.2.0"
message: "If you use this software, please cite it using these metadata."
title: "My Research Software"
"#;
const TYPICAL_JSON: &str = r#"
{
  "abstract": "This is my awesome research software.",
  "authors": [
    {
      "family-names": "Druskat",
      "given-names": "Stephan",
      "orcid": "https://orcid.org/1234-5678-9101-1121",
      "postal-code": "40210"
    }
  ],
  "cff-version": "1.2.0",
  "date-released": "2021-07-18",
  "identifiers": [
    {
      "description": "Version DOI",
      "type": "doi",
      "value": "10.5281/zenodo.123457"
    },
    {
      "description": "Project website",
      "type": "url",
      "value": "https://example.org/my-research-software"
    }
  ],
  "keywords": ["awesome software", "research"],
  "license": ["Apache-2.0", "MIT"],
  "message": "If you use this software, please cite both the article from preferred-citation and the software itself.",
  "preferred-citation": {
    "authors": [
      {
        "family-names": "Druskat",
        "given-names": "Stephan",
        "postal-code": 12345
      }
    ],
    "title": "Software paper about My Research Software",
    "type": "article",
    "year": 2021
  },
  "references": [
    {
      "authors": [
        {
          "name": "The Dependency Project"
        }
      ],
      "repository-code": "https://github.com/dependency-project/dependency",
      "title": "Dependency",
      "type": "software",
      "version": "0.13.4",
      "year": "2020"
    }
  ],
  "repository-code": "https://github.com/citation-file-format/my-research-software",
  "title": "My Research Software",
  "type": "software",
  "version": "0.11.2"
}
"#;
#[test]
fn test_cff_minimal_yaml_round_trip() {
    let parsed: Cff = serde_norway::from_str(MINIMAL_YAML).expect("failed to parse minimal CFF yaml");
    let encoded = serde_norway::to_string(&parsed).expect("failed to serialize minimal CFF yaml");
    let reparsed: Cff = serde_norway::from_str(&encoded).expect("failed to deserialize serialized minimal CFF yaml");
    assert_eq!(parsed, reparsed);
    assert_eq!(parsed.cff_version, "1.2.0");
    assert_eq!(parsed.title, "My Research Software");
    assert_eq!(parsed.kind, None);
    assert_eq!(parsed.authors.len(), 1);
    assert!(matches!(parsed.authors[0], Agent::Person(_)));
}
#[test]
fn test_cff_typical_json_round_trip() {
    let parsed: Cff = serde_json::from_str(TYPICAL_JSON).expect("failed to parse typical CFF json");
    let encoded = serde_json::to_string_pretty(&parsed).expect("failed to serialize typical CFF json");
    let reparsed: Cff = serde_json::from_str(&encoded).expect("failed to deserialize serialized typical CFF json");
    assert_eq!(parsed, reparsed);
    assert_eq!(parsed.kind, Some(CffType::Software));
    assert_eq!(parsed.identifiers.as_ref().map(Vec::len), Some(2));
    assert_eq!(parsed.references.as_ref().map(Vec::len), Some(1));
    let id_types: Vec<_> = parsed
        .identifiers
        .as_ref()
        .expect("identifiers should exist")
        .iter()
        .map(|value| value.kind.clone())
        .collect();
    assert_eq!(id_types, vec![IdentifierType::Doi, IdentifierType::Url]);
    match parsed.license {
        | Some(OneOrMany::Many(values)) => assert_eq!(values, vec!["Apache-2.0", "MIT"]),
        | _ => panic!("expected multiple license values"),
    }
    let preferred = parsed.preferred_citation.as_ref().expect("preferred citation should exist");
    assert_eq!(preferred.kind, ReferenceType::Article);
    assert_eq!(preferred.year, Some(YearValue::Number(2021)));
    match &preferred.authors[0] {
        | Agent::Person(person) => assert_eq!(person.postal_code, Some(PostalCode::Number(12345))),
        | Agent::Entity(_) => panic!("expected preferred-citation author to be a person"),
    }
    let reference = parsed
        .references
        .as_ref()
        .expect("references should exist")
        .first()
        .expect("expected one reference");
    assert_eq!(reference.kind, ReferenceType::Software);
    assert_eq!(reference.year, Some(YearValue::Text("2020".to_string())));
    assert!(matches!(reference.authors[0], Agent::Entity(_)));
}
#[test]
fn test_cff_enum_round_trip_values() {
    let work_type = CffType::Dataset;
    let work_type_json = serde_json::to_string(&work_type).expect("failed to serialize CffType");
    assert_eq!(work_type_json, "\"dataset\"");
    let round_trip_work_type: CffType = serde_json::from_str(&work_type_json).expect("failed to deserialize CffType");
    assert_eq!(round_trip_work_type, CffType::Dataset);
    let reference_type = ReferenceType::ConferencePaper;
    let reference_type_json = serde_json::to_string(&reference_type).expect("failed to serialize ReferenceType");
    assert_eq!(reference_type_json, "\"conference-paper\"");
    let round_trip_reference_type: ReferenceType = serde_json::from_str(&reference_type_json).expect("failed to deserialize ReferenceType");
    assert_eq!(round_trip_reference_type, ReferenceType::ConferencePaper);
}

#[test]
fn test_cff_to_prose_and_markdown_include_primary_fields() {
    let cff = Cff {
        title: "My Research Software".to_string(),
        message: "Please cite this work".to_string(),
        abstract_text: Some("A compact abstract".to_string()),
        keywords: Some(vec!["research".to_string(), "software".to_string()]),
        ..Cff::default()
    };
    let prose = cff.to_prose();
    let markdown = cff.to_markdown();
    assert!(prose.contains("My Research Software"));
    assert!(prose.contains("Please cite this work"));
    assert!(markdown.contains("title: My Research Software"));
    assert!(markdown.contains("message: Please cite this work"));
}
