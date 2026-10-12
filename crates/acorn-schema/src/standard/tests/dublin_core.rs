use crate::codec::DocumentCodec;
use crate::standard::dublin_core::{
    relationships, term, terms, DublinCoreDescription, DublinCoreDocument, DublinCoreLiteral, DublinCoreNamespace, DublinCoreRelationshipKind,
    DublinCoreResource, DublinCoreTermKind, DublinCoreValue, DCAM_NAMESPACE, DCTERMS_NAMESPACE, ELEMENTS_NAMESPACE, TYPE_NAMESPACE,
};
use crate::standard::Standard;
use crate::Validate;
use acorn_core::util::MimeType;
use alloc::collections::BTreeMap;
use pretty_assertions::assert_eq;

fn document() -> DublinCoreDocument {
    DublinCoreDocument::new(vec![DublinCoreDescription {
        properties: BTreeMap::from([
            (
                format!("{DCTERMS_NAMESPACE}creator"),
                vec![DublinCoreValue::Resource(DublinCoreResource::blank_node("creator"))],
            ),
            (
                format!("{DCTERMS_NAMESPACE}title"),
                vec![
                    DublinCoreValue::Literal(DublinCoreLiteral {
                        datatype: None,
                        language: Some("en".to_string()),
                        value: "A title".to_string(),
                    }),
                    DublinCoreValue::Literal(DublinCoreLiteral {
                        datatype: None,
                        language: Some("en".to_string()),
                        value: "A title".to_string(),
                    }),
                ],
            ),
        ]),
        subject: DublinCoreResource::iri("https://example.org/work"),
        types: vec![format!("{TYPE_NAMESPACE}Text"), format!("{TYPE_NAMESPACE}Text")],
    }])
}
#[test]
fn test_document_normalization_is_deterministic() {
    let document = document();
    let description = document.descriptions.first().expect("description");
    assert_eq!(description.types.len(), 1);
    assert_eq!(
        description
            .properties
            .get(&format!("{DCTERMS_NAMESPACE}title"))
            .expect("title values")
            .len(),
        1
    );
    assert!(document.validate().is_ok());
    assert_eq!(document.clone().normalized(), document);
}
#[test]
fn test_document_preserves_extension_predicates() {
    let mut document = document();
    document.descriptions.first_mut().expect("description").properties.insert(
        "https://example.org/vocab/custom".to_string(),
        vec![DublinCoreValue::Literal(DublinCoreLiteral {
            datatype: Some("https://example.org/datatypes/custom".to_string()),
            language: None,
            value: "custom".to_string(),
        })],
    );
    let encoded = serde_json::to_string(&document).expect("serialize graph");
    let decoded: DublinCoreDocument = serde_json::from_str(&encoded).expect("deserialize graph");
    assert_eq!(decoded, document);
    assert!(decoded.validate().is_ok());
}
#[test]
fn test_document_rejects_invalid_graph_values() {
    let invalid = DublinCoreDocument::new(vec![DublinCoreDescription {
        properties: BTreeMap::from([(
            format!("{DCTERMS_NAMESPACE}title"),
            vec![DublinCoreValue::Literal(DublinCoreLiteral {
                datatype: Some("relative".to_string()),
                language: Some("en".to_string()),
                value: "Title".to_string(),
            })],
        )]),
        subject: DublinCoreResource::blank_node("_:invalid"),
        types: vec!["relative".to_string()],
    }]);
    let report = invalid.validate().expect_err("invalid graph must fail");
    assert!(report.len() >= 4);
}
#[test]
fn test_document_requires_a_dcmi_statement() {
    let document = DublinCoreDocument::new(vec![DublinCoreDescription {
        properties: BTreeMap::from([(
            "https://example.org/vocab/name".to_string(),
            vec![DublinCoreValue::Literal(DublinCoreLiteral {
                datatype: None,
                language: None,
                value: "Extension only".to_string(),
            })],
        )]),
        subject: DublinCoreResource::iri("https://example.org/work"),
        types: Vec::new(),
    }]);
    assert!(document.validate().is_err());
}
#[test]
fn test_document_round_trips_native_formats() {
    [MimeType::Json, MimeType::Yaml].into_iter().for_each(|format| {
        let document = document();
        let encoded = document.encode(&format).expect("encode native graph");
        let decoded = DublinCoreDocument::decode(&encoded, &format).expect("decode native graph");
        assert_eq!(decoded, document);
    });
}
#[test]
fn test_document_round_trips_rdf_formats_deterministically() {
    [MimeType::LdJson, MimeType::RdfXml, MimeType::Turtle].into_iter().for_each(|format| {
        let first_encoding = document().encode(&format).expect("encode RDF graph");
        let first_document = DublinCoreDocument::decode(&first_encoding, &format).expect("decode RDF graph");
        let second_encoding = first_document.encode(&format).expect("re-encode RDF graph");
        let second_document = DublinCoreDocument::decode(&second_encoding, &format).expect("re-decode RDF graph");
        assert_eq!(second_document, first_document);
        assert_eq!(second_encoding, first_encoding);
    });
}
#[test]
fn test_json_ld_accepts_inline_context_and_preserves_extensions() {
    let content = r#"{
        "@context": {
            "dcmitype": "http://purl.org/dc/dcmitype/",
            "dcterms": "http://purl.org/dc/terms/",
            "ex": "https://example.org/vocab/"
        },
        "@id": "https://example.org/work",
        "@type": "dcmitype:Text",
        "dcterms:title": { "@value": "A title", "@language": "en" },
        "ex:custom": { "@value": "7", "@type": "http://www.w3.org/2001/XMLSchema#integer" }
    }"#;
    let decoded = DublinCoreDocument::decode(content, &MimeType::LdJson).expect("decode compact JSON-LD");
    let description = decoded.descriptions.first().expect("description");
    assert_eq!(description.types, vec![format!("{TYPE_NAMESPACE}Text")]);
    assert_eq!(
        description
            .properties
            .get("https://example.org/vocab/custom")
            .expect("extension values")
            .len(),
        1
    );
}
#[test]
fn test_json_ld_rejects_remote_contexts_and_relative_iris() {
    let remote_context = r#"{
        "@context": "https://example.org/context.jsonld",
        "@id": "https://example.org/work",
        "http://purl.org/dc/terms/title": "A title"
    }"#;
    let relative_iri = r#"{
        "@id": "work",
        "http://purl.org/dc/terms/title": "A title"
    }"#;
    assert!(DublinCoreDocument::decode(remote_context, &MimeType::LdJson).is_err());
    assert!(DublinCoreDocument::decode(relative_iri, &MimeType::LdJson).is_err());
}
#[test]
fn test_json_ld_standard_detection_recognizes_compact_and_native_documents() {
    let compact = serde_json::json!({
        "@context": { "dcterms": DCTERMS_NAMESPACE },
        "@id": "https://example.org/work",
        "dcterms:title": "A title"
    });
    let native = serde_json::to_value(document()).expect("serialize native graph");
    assert_eq!(compact.as_object().and_then(Standard::detect_object), Some(Standard::DublinCore));
    assert_eq!(native.as_object().and_then(Standard::detect_object), Some(Standard::DublinCore));
}
#[test]
fn test_rdf_standard_detection_distinguishes_dcat_dublin_core_and_unknown_vocabularies() {
    let dcat = "@prefix dcat: <http://www.w3.org/ns/dcat#> .\n<https://example.org/dataset> a dcat:Dataset .\n";
    let dublin_core = "@prefix dcterms: <http://purl.org/dc/terms/> .\n<https://example.org/work> dcterms:title \"Example\" .\n";
    let unknown = "@prefix schema: <https://schema.org/> .\n<https://example.org/work> a schema:CreativeWork .\n";
    assert_eq!(Standard::detect_rdf(dcat, &MimeType::Turtle).expect("detect DCAT"), Some(Standard::Dcat));
    assert_eq!(
        Standard::detect_rdf(dublin_core, &MimeType::Turtle).expect("detect Dublin Core"),
        Some(Standard::DublinCore)
    );
    assert_eq!(Standard::detect_rdf(unknown, &MimeType::Turtle).expect("detect unknown graph"), None);
}
#[test]
fn test_registry_contains_all_published_vocabulary_terms() {
    assert_eq!(terms().len(), 129);
    assert_eq!(
        terms()
            .iter()
            .filter(|value| value.namespace == DublinCoreNamespace::MetadataTerms && value.kind == DublinCoreTermKind::Property)
            .count(),
        55
    );
    assert_eq!(
        terms()
            .iter()
            .filter(|value| value.namespace == DublinCoreNamespace::MetadataElements && value.kind == DublinCoreTermKind::Property)
            .count(),
        15
    );
    assert_eq!(
        terms()
            .iter()
            .filter(|value| value.namespace == DublinCoreNamespace::TypeVocabulary && value.kind == DublinCoreTermKind::Class)
            .count(),
        12
    );
    assert_eq!(
        terms()
            .iter()
            .filter(|value| value.namespace == DublinCoreNamespace::AbstractModel)
            .count(),
        4
    );
    assert_eq!(term(&format!("{DCTERMS_NAMESPACE}title")).map(|value| value.label), Some("Title"));
    assert_eq!(term(&format!("{ELEMENTS_NAMESPACE}title")).map(|value| value.label), Some("Title"));
    assert_eq!(term(&format!("{TYPE_NAMESPACE}Dataset")).map(|value| value.label), Some("Dataset"));
    assert_eq!(
        term(&format!("{DCAM_NAMESPACE}rangeIncludes")).map(|value| value.label),
        Some("Range Includes")
    );
    assert_eq!(relationships().len(), 78);
    assert!(relationships().iter().any(|relationship| {
        relationship.subject == format!("{DCTERMS_NAMESPACE}medium")
            && relationship.kind == DublinCoreRelationshipKind::DomainIncludes
            && relationship.object == format!("{DCTERMS_NAMESPACE}PhysicalResource")
    }));
    assert_eq!(DublinCoreNamespace::MetadataTerms.modified(), "2012-06-14");
}
#[test]
fn test_syntax_encoding_schemes_validate_lexical_values() {
    let valid = [
        ("Box", "northlimit=90; southlimit=-90; eastlimit=180; westlimit=-180"),
        ("ISO3166", "US"),
        ("ISO639-3", "eng"),
        ("Period", "start=2024-01-01; end=2024-12-31"),
        ("Point", "north=35.9; east=-84.3"),
        ("RFC5646", "en-US"),
        ("URI", "https://example.org/resource"),
        ("W3CDTF", "2024-01-01T12:30:00Z"),
    ];
    valid.into_iter().for_each(|(datatype, value)| {
        let literal = DublinCoreLiteral {
            datatype: Some(format!("{DCTERMS_NAMESPACE}{datatype}")),
            language: None,
            value: value.to_string(),
        };
        assert!(literal.validate().is_ok(), "{datatype}: {value}");
    });
    let invalid = DublinCoreLiteral {
        datatype: Some(format!("{DCTERMS_NAMESPACE}Point")),
        language: None,
        value: "north=91; east=0".to_string(),
    };
    assert!(invalid.validate().is_err());
}
#[test]
fn test_turtle_preserves_repeated_typed_and_extension_values() {
    let content = r#"
        @prefix dcterms: <http://purl.org/dc/terms/> .
        @prefix ex: <https://example.org/vocab/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        <https://example.org/work>
            dcterms:title "A title"@en, "7"^^xsd:integer ;
            ex:creator _:creator .
        _:creator ex:name "Ada" .
    "#;
    let decoded = DublinCoreDocument::decode(content, &MimeType::Turtle).expect("decode Turtle graph");
    let work = decoded
        .descriptions
        .iter()
        .find(|description| description.subject == DublinCoreResource::iri("https://example.org/work"))
        .expect("work description");
    assert_eq!(work.properties.get(&format!("{DCTERMS_NAMESPACE}title")).expect("title values").len(), 2);
    assert_eq!(work.properties.get("https://example.org/vocab/creator").expect("creator values").len(), 1);
    assert_eq!(decoded.descriptions.len(), 2);
}
